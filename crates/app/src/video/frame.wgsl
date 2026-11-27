// Quad con un frame encima, dentro del widget.
//
// Sin vertex buffer: los 4 vértices salen del índice. Con TriangleStrip,
// i = 0..3 recorre las esquinas en el orden (0,0) (1,0) (0,1) (1,1).
//
// NDC -1..1 es el **widget**, no la ventana: iced_wgpu pone un viewport por
// primitivo (`set_viewport(bounds…)`) antes de dibujarlo. Por eso aquí no hace
// falta saber dónde cae el widget en la pantalla; el letterbox se calcula solo
// por proporción y vive en la CPU (uniform `Escala`).

struct Salida {
    // Posición en clip space. wgpu espera y = +1 arriba.
    @builtin(position) posicion: vec4<f32>,
    // Coordenada de textura, interpolada por la GPU. v = 0 es la primera
    // fila del buffer que subimos con write_texture.
    @location(0) uv: vec2<f32>,
};

// Deben coincidir con el bind_group_layout: grupo 0, binding 0, 1 y 2.
@group(0) @binding(0) var frame: texture_2d<f32>;
@group(0) @binding(1) var muestreo: sampler;

// Cuánto encoger el quad para que el video conserve su proporción dentro de un
// widget de otra forma. x = 1 e y < 1 deja barras arriba y abajo (letterbox).
struct Escala {
    x: f32,
    y: f32,
};

@group(0) @binding(2) var<uniform> escala: Escala;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> Salida {
    let uv = vec2<f32>(f32(i & 1u), f32((i >> 1u) & 1u));

    // uv va de 0..1 y el clip space de -1..1: así se mapea.
    // El 1.0 - en y concilia "arriba" en NDC con "primera fila" en la textura.
    //
    // La escala se aplica a la posición y NO a la uv: el quad se encoge y sigue
    // cubriendo la textura completa, que es justo lo que hace el letterbox.
    let posicion = vec4<f32>(
        (uv.x * 2.0 - 1.0) * escala.x,
        (1.0 - uv.y * 2.0) * escala.y,
        0.0,
        1.0,
    );

    var salida: Salida;
    salida.posicion = posicion;
    salida.uv = uv;
    return salida;
}

@fragment
fn fs_main(entrada: Salida) -> @location(0) vec4<f32> {
    // El cuarto byte de `rgb0` es basura (mpv no lo inicializa). Forzamos
    // alfa 1 aquí en vez de recorrer 2 millones de píxeles en CPU cada frame.
    let color = textureSample(frame, muestreo, entrada.uv);
    return vec4<f32>(color.rgb, 1.0);
}
