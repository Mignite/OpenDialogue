//! El widget de video: pinta el buffer de mpv como una textura propia.
//!
//! Tres tipos, tres vidas:
//!
//! - [`Video`] es el `Program` del widget. Se crea en `view`, uno por frame.
//! - [`Frame`] es el dato de ese frame: píxeles y tamaño.
//! - [`Gpu`] es el estado de GPU (pipeline, textura, bind group). Se crea una
//!   vez por proceso y no depende de cuántos frames pasen.

use std::fmt;
use std::sync::Arc;

use iced::Rectangle;
use iced::wgpu;
use iced::widget::shader;

pub mod geometria;

/// Bytes por instancia de capa: 3 atributos `float32x4`.
const CAPA_BYTES: u64 = 3 * 16;

pub struct Video {
    pub pixels: Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
    /// Capas de subtítulo ya rasterizadas para este instante. Vacío = sin
    /// subtítulos.
    pub capas: Arc<Vec<Capa>>,
}

/// Una capa de subtítulo lista para la GPU: la máscara de libass copiada y su
/// color. Se copia porque los punteros de libass solo valen hasta el siguiente
/// render, y el widget tiene que sobrevivir a eso.
#[derive(Debug, Clone)]
pub struct Capa {
    /// Esquina superior izquierda dentro del frame del video.
    pub x: i32,
    pub y: i32,
    pub ancho: u32,
    pub alto: u32,
    pub stride: usize,
    /// RGB en sRGB y alfa de la capa (0 = transparente).
    pub color: [f32; 4],
    pub mascara: Vec<u8>,
}

impl Capa {
    /// Traduce un bitmap de libass. El color viene `0xRRGGBBAA` con el alfa
    /// invertido: `0` es opaco y `255` invisible.
    pub fn desde(bitmap: &rod_subs::Bitmap) -> Self {
        let color = bitmap.color;
        Self {
            x: bitmap.x,
            y: bitmap.y,
            ancho: bitmap.ancho,
            alto: bitmap.alto,
            stride: bitmap.stride,
            color: [
                (color >> 24) as u8 as f32 / 255.0,
                ((color >> 16) & 0xFF) as u8 as f32 / 255.0,
                ((color >> 8) & 0xFF) as u8 as f32 / 255.0,
                (255 - (color & 0xFF)) as f32 / 255.0,
            ],
            mascara: bitmap.alpha().to_vec(),
        }
    }
}

impl<Message> shader::Program<Message> for Video {
    type State = ();
    type Primitive = Frame;

    fn draw(&self, _state: &(), _cursor: iced::mouse::Cursor, _bounds: Rectangle) -> Frame {
        Frame {
            pixels: Arc::clone(&self.pixels),
            width: self.width,
            height: self.height,
            capas: Arc::clone(&self.capas),
        }
    }
}

pub struct Frame {
    pixels: Arc<Vec<u8>>,
    width: u32,
    height: u32,
    capas: Arc<Vec<Capa>>,
}

impl shader::Primitive for Frame {
    type Pipeline = Gpu;

    fn prepare(
        &self,
        gpu: &mut Gpu,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        _viewport: &shader::Viewport,
    ) {
        gpu.ensure_texture(device, self.width, self.height);

        // Letterbox: el widget puede tener otra forma que el video. Se encoge el
        // quad en el eje que sobra en vez de deformarlo.
        //
        // Solo importa la proporción: iced ya deja el origen del clip space en el
        // rect del widget (pone un viewport por primitivo), así que el quad
        // centrado en NDC queda centrado en el widget.
        let escala =
            geometria::escala_letterbox((self.width, self.height), (bounds.width, bounds.height));

        let mut uniforms = [0u8; 16];
        uniforms[0..4].copy_from_slice(&escala.0.to_le_bytes());
        uniforms[4..8].copy_from_slice(&escala.1.to_le_bytes());
        queue.write_buffer(&gpu.uniforms, 0, &uniforms);

        let Some(texture) = gpu.texture.as_ref() else {
            return;
        };

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.width * 4),
                rows_per_image: Some(self.height),
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        gpu.subir_capas(device, queue, &self.capas, self.width, self.height, escala);
    }

    fn draw(&self, gpu: &Gpu, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        let Some(bind_group) = gpu.bind_group.as_ref() else {
            return false;
        };

        render_pass.set_pipeline(&gpu.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..4, 0..1);

        // Los subtítulos son un segundo draw call en el mismo pase: el pase ya
        // tiene el video pintado y el blending lo mezcla contra él.
        if gpu.capas_dibujadas > 0
            && let Some(capas) = gpu.capas_bind_group.as_ref()
        {
            render_pass.set_pipeline(&gpu.capas_pipeline);
            render_pass.set_bind_group(0, capas, &[]);
            render_pass.set_vertex_buffer(0, gpu.instancias.slice(..));
            render_pass.draw(0..6, 0..gpu.capas_dibujadas);
        }

        true
    }
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Frame")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("bytes", &self.pixels.len())
            .finish()
    }
}

pub struct Gpu {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    texture_format: wgpu::TextureFormat,
    texture: Option<wgpu::Texture>,
    bind_group: Option<wgpu::BindGroup>,
    size: (u32, u32),

    // Subtítulos: un atlas de máscaras y un buffer de instancias con el
    // rectángulo, la uv y el color de cada capa.
    capas_pipeline: wgpu::RenderPipeline,
    capas_layout: wgpu::BindGroupLayout,
    atlas: Option<wgpu::Texture>,
    capas_bind_group: Option<wgpu::BindGroup>,
    atlas_size: (u32, u32),
    instancias: wgpu::Buffer,
    instancias_bytes: u64,
    capas_dibujadas: u32,
}

impl shader::Pipeline for Gpu {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rod-app frame shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("frame.wgsl").into()),
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rod-app frame layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rod-app frame pipeline layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rod-app frame pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    // El frame llega opaco: sin blending, se escribe tal cual.
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("rod-app frame sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rod-app frame escala"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // --- Subtítulos ---------------------------------------------------

        let subs_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rod-app subs shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("subs.wgsl").into()),
        });

        let capas_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rod-app subs layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let capas_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("rod-app subs pipeline layout"),
                bind_group_layouts: &[&capas_layout],
                push_constant_ranges: &[],
            });

        // Un vec4 por atributo: rect en clip space, uv en el atlas, color.
        let atributos = wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4];

        let capas_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rod-app subs pipeline"),
            layout: Some(&capas_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &subs_shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: CAPA_BYTES,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &atributos,
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &subs_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    // Alfa normal: lo que la capa no tapa, deja ver el video.
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let instancias = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rod-app subs instancias"),
            size: CAPA_BYTES * 64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Si el target del frame es sRGB, la textura tiene que serlo también:
        // así la decodificación al muestrear y la codificación al escribir se
        // cancelan y los colores salen como los dio mpv.
        let texture_format = if format.is_srgb() {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };

        Self {
            pipeline,
            layout,
            sampler,
            uniforms,
            texture_format,
            texture: None,
            bind_group: None,
            size: (0, 0),
            capas_pipeline,
            capas_layout,
            atlas: None,
            capas_bind_group: None,
            atlas_size: (0, 0),
            instancias,
            instancias_bytes: CAPA_BYTES * 64,
            capas_dibujadas: 0,
        }
    }
}

impl Gpu {
    /// Crea la textura y su bind group la primera vez, y los rehace solo si el
    /// tamaño del frame cambia. `write_texture` reescribe el contenido.
    fn ensure_texture(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.size == (width, height) && self.texture.is_some() {
            return;
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rod-app frame texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.texture_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rod-app frame bind group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(
                        self.uniforms.as_entire_buffer_binding(),
                    ),
                },
            ],
        });

        self.texture = Some(texture);
        self.bind_group = Some(bind_group);
        self.size = (width, height);
    }

    /// Empaqueta las máscaras en un atlas de una columna y arma una instancia
    /// por capa con su rectángulo en clip space (ya con el letterbox) y su uv.
    ///
    /// Rehace la textura solo si cambia el tamaño del atlas: el contenido se
    /// reescribe, que es lo barato.
    fn subir_capas(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        capas: &[Capa],
        video_w: u32,
        video_h: u32,
        (escala_x, escala_y): (f32, f32),
    ) {
        self.capas_dibujadas = 0;
        if capas.is_empty() || video_w == 0 || video_h == 0 {
            return;
        }

        let ancho = capas.iter().map(|c| c.ancho).max().unwrap_or(1).max(1);
        let alto: u32 = capas.iter().map(|c| c.alto).sum::<u32>().max(1);

        if self.atlas_size != (ancho, alto) || self.atlas.is_none() {
            let textura = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("rod-app subs atlas"),
                size: wgpu::Extent3d {
                    width: ancho,
                    height: alto,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Solo cobertura: un canal. El color va por instancia, así que
                // dos capas del mismo texto con distinto color comparten máscara.
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let vista = textura.create_view(&wgpu::TextureViewDescriptor::default());
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("rod-app subs bind group"),
                layout: &self.capas_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&vista),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            self.atlas = Some(textura);
            self.capas_bind_group = Some(bind_group);
            self.atlas_size = (ancho, alto);
        }

        let mut pixeles = vec![0u8; (ancho * alto) as usize];
        let mut instancias = Vec::with_capacity(capas.len() * CAPA_BYTES as usize);
        let mut fila_atlas = 0u32;

        for capa in capas {
            // Una fila de la máscara mide `stride`, pero la última puede venir
            // más corta: se copia lo que hay, sin pasarse del buffer.
            for fila in 0..capa.alto as usize {
                let origen = fila * capa.stride;
                let disponibles = capa.mascara.len().saturating_sub(origen);
                let bytes = (capa.ancho as usize).min(disponibles);
                if bytes == 0 {
                    break;
                }
                let destino = (fila_atlas as usize + fila) * ancho as usize;
                pixeles[destino..destino + bytes]
                    .copy_from_slice(&capa.mascara[origen..origen + bytes]);
            }

            // El pixel del video va de 0..video a 0..frame, y de ahí a clip
            // space (con la y invertida y el letterbox ya aplicado).
            let [x0, y0, x1, y1] = geometria::rect_de_capa(
                capa.x,
                capa.y,
                capa.ancho,
                capa.alto,
                (video_w, video_h),
                (escala_x, escala_y),
            );

            let u0 = 0.0;
            let u1 = capa.ancho as f32 / ancho as f32;
            let v0 = fila_atlas as f32 / alto as f32;
            let v1 = (fila_atlas + capa.alto) as f32 / alto as f32;

            for valor in [
                x0,
                y0,
                x1,
                y1,
                u0,
                v0,
                u1,
                v1,
                capa.color[0],
                capa.color[1],
                capa.color[2],
                capa.color[3],
            ] {
                instancias.extend_from_slice(&valor.to_le_bytes());
            }

            fila_atlas += capa.alto;
        }

        if let Some(textura) = self.atlas.as_ref() {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: textura,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &pixeles,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(ancho),
                    rows_per_image: Some(alto),
                },
                wgpu::Extent3d {
                    width: ancho,
                    height: alto,
                    depth_or_array_layers: 1,
                },
            );
        }

        let bytes = instancias.len() as u64;
        if bytes > self.instancias_bytes {
            let nuevo = bytes.next_power_of_two();
            self.instancias = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("rod-app subs instancias"),
                size: nuevo,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instancias_bytes = nuevo;
        }

        queue.write_buffer(&self.instancias, 0, &instancias);
        self.capas_dibujadas = capas.len() as u32;
    }
}
