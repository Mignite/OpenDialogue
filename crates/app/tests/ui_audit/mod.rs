#![cfg(feature = "test-video")]

//! Arnés de capturas de la auditoría visual.
//!
//! No es un test: lo usa `examples/ui_audit.rs` para renderizar la matriz de
//! pantallas con egui_kittest + wgpu, sin abrir ventana ni depender de mpv para
//! el contenido del video (se inyecta un frame fijo).
//!
//! Reglas del arnés:
//! - Todo fixture se genera aquí; no se lee ningún archivo externo salvo el
//!   clip de prueba que ya vive en `testdata/`.
//! - El tamaño del PNG es el de la ventana en puntos (ppp = 1).
//! - El fondo se pinta siempre: el harness usa `Frame::NONE`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use rod_app::comandos::MapaAtajos;
use rod_app::editor_app::{self, EditorApp, MensajeEditor, Panel};
use rod_app::inicio::EstadoInicio;
use rod_app::preferencias::Preferencias;
use rod_app::recientes::Reciente;
use rod_app::textos::{FechaRelativa, Idioma};
use rod_app::ui::{self, chrome, tema};
use rod_core::{Caption, Hablante, PresetAss, Proyecto};

pub const TAMANOS: [(f32, f32); 4] = [
    (1100.0, 700.0),
    (1600.0, 900.0),
    (1920.0, 1080.0),
    (2560.0, 1440.0),
];

pub fn etiqueta_tamano(ancho: f32, alto: f32) -> String {
    format!("{ancho:.0}x{alto:.0}")
}

pub fn carpeta(fase: &str) -> PathBuf {
    // Cada área renderiza sus capturas en una carpeta propia durante la Fase 3
    // para que dos subagentes no escriban el mismo PNG a la vez.
    let fase = std::env::var("UI_AUDIT_FASE").unwrap_or_else(|_| fase.to_owned());
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("ui-audit")
        .join(fase)
}

pub fn ruta_video() -> String {
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    )
    .to_owned()
}

fn base_temporal() -> PathBuf {
    let base = std::env::temp_dir().join("rod-ui-audit");
    let _ = std::fs::create_dir_all(&base);
    base
}

// ── Frame fijo ──────────────────────────────────────────────────────────────

/// Patrón de prueba 1920×1080: barras, rejilla, círculo y bandas diagonales.
/// Da contraste real al overlay y permite ver el letterbox y el recorte.
pub fn frame_prueba() -> Vec<u8> {
    static CACHE: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    CACHE.get_or_init(generar_frame).clone()
}

fn generar_frame() -> Vec<u8> {
    const ANCHO: usize = 1920;
    const ALTO: usize = 1080;
    let mut pixeles = vec![0u8; ANCHO * ALTO * 4];
    for y in 0..ALTO {
        for x in 0..ANCHO {
            let i = (y * ANCHO + x) * 4;
            let fx = x as f32 / ANCHO as f32;
            let fy = y as f32 / ALTO as f32;
            // Degradado azul verdoso oscuro.
            let mut r = (12.0 + 28.0 * fx) as u8;
            let mut g = (18.0 + 44.0 * fy) as u8;
            let mut b = (34.0 + 70.0 * (1.0 - fy)) as u8;
            // Barras de color en la franja superior.
            if y < 72 {
                let barras = [
                    [196, 62, 54],
                    [206, 148, 44],
                    [196, 190, 58],
                    [70, 168, 96],
                    [52, 146, 190],
                    [86, 92, 186],
                    [162, 82, 178],
                ];
                let barra = (x * barras.len() / ANCHO).min(barras.len() - 1);
                [r, g, b] = barras[barra];
            }
            // Rejilla de tercios y centro.
            if x % 240 == 0 || y % 180 == 0 {
                r = r.saturating_add(16);
                g = g.saturating_add(18);
                b = b.saturating_add(20);
            }
            let dx = fx - 0.5;
            let dy = fy - 0.5;
            if (dx * dx + dy * dy).sqrt() < 0.16 {
                r = r.saturating_add(34);
                g = g.saturating_add(30);
                b = b.saturating_add(22);
            }
            // Banda diagonal tenue.
            if ((fx + fy) * 8.0).fract() < 0.08 {
                r = r.saturating_add(12);
                b = b.saturating_add(18);
            }
            pixeles[i] = r;
            pixeles[i + 1] = g;
            pixeles[i + 2] = b;
            pixeles[i + 3] = 255;
        }
    }
    pixeles
}

// ── Fixtures de proyecto ────────────────────────────────────────────────────

const NOMBRES: [&str; 7] = [
    "Ana Torres",
    "Bruno Salas",
    "Carla Restrepo",
    "Diego Muñoz",
    "Elena Ochoa",
    "Fabián Vélez",
    "Gala Ríos",
];

const LARGOS: [&str; 4] = [
    "Cuando el documental se estrenó en el festival, nadie esperaba que una historia contada en voz baja terminara ocupando la sala principal durante tres semanas seguidas.",
    "La cámara recorre el pasillo vacío mientras la narración explica que aquel edificio había sido escuela, luego almacén y, finalmente, el lugar donde todo el barrio aprendió a organizarse.",
    "Hay una pausa larga antes de la respuesta; el entrevistado mira por la ventana, duda, y decide contar la parte que nunca había contado frente a un micrófono.",
    "El montaje alterna el presente con las cintas de archivo, y esa mezcla de texturas —el grano de 1998 junto al digital de hoy— sostiene el tono elegido por la directora.",
];

fn texto_de_linea(indice: usize) -> String {
    match indice % 7 {
        0 => "Sí.".to_owned(),
        1 => "Una línea corta.".to_owned(),
        2 => format!(
            "Dos líneas de texto que el editor debe\nsaber mostrar sin romper el ritmo de {indice}."
        ),
        3 => LARGOS[indice % LARGOS.len()].to_owned(),
        4 => "«Cita con comillas» — y un guion largo…".to_owned(),
        5 => format!("Números: 1, 2, 3 · 42 % · {indice}.91 s"),
        _ => format!("Texto normal de la línea {indice}."),
    }
}

/// Proyecto sintético: líneas variadas, sin hablante cada 13, solapes cada 17.
pub fn make_proyecto(lineas: usize, hablantes: usize, seed: u64, duracion: f64) -> Proyecto {
    let mut proyecto = Proyecto::nuevo(ruta_video());
    for i in 0..hablantes.min(NOMBRES.len()) {
        proyecto.hablantes.push(Hablante {
            id: format!("h{i}"),
            nombre: NOMBRES[i].to_owned(),
            tecla: (i + 1).to_string(),
            color: String::new(),
            preset_id: None,
        });
    }
    if lineas == 0 {
        return proyecto;
    }
    let slot = (duracion / lineas as f64).max(0.6);
    let mut fin_anterior = 0.0;
    for i in 0..lineas {
        let mut inicio = i as f64 * slot + slot * 0.12;
        if i > 0 && i % 17 == 0 {
            inicio = (fin_anterior - slot * 0.35).max(0.0);
        }
        let fin = inicio + (slot * 0.58).max(0.5);
        fin_anterior = fin;
        let hablante_id = if hablantes == 0 || i % 13 == 12 {
            None
        } else {
            Some(format!("h{}", (i + seed as usize) % hablantes.min(NOMBRES.len())))
        };
        proyecto.captions.push(Caption {
            id: format!("cap-{i}"),
            inicio,
            fin,
            texto: texto_de_linea(i),
            hablante_id,
        });
    }
    proyecto
}

pub fn presets_fixture() -> Vec<PresetAss> {
    vec![
        PresetAss {
            id: "p-kiwi".into(),
            nombre: "Kiwi".into(),
            fontname: "Arial".into(),
            fontsize: 48.0,
            color: "#F2F2F2".into(),
            outline_color: "#101010".into(),
            outline: 2.0,
            shadow: 1.0,
            ..rod_core::preset_por_defecto()
        },
        PresetAss {
            id: "p-citron".into(),
            nombre: "Citron".into(),
            fontname: "Georgia".into(),
            fontsize: 42.0,
            color: "#FFD84A".into(),
            ..rod_core::preset_por_defecto()
        },
        PresetAss {
            id: "p-coral".into(),
            nombre: "Coral".into(),
            fontname: "Verdana".into(),
            fontsize: 54.0,
            color: "#FF6B4A".into(),
            bold: true,
            ..rod_core::preset_por_defecto()
        },
    ]
}

fn picos_fixture(duracion: f64) -> rod_player::waveform::Picos {
    let por_segundo = 100u32;
    let n = (duracion.max(1.0) * f64::from(por_segundo)) as usize;
    let max = (0..n)
        .map(|i| {
            let t = i as f32 / 40.0;
            (0.18 + 0.22 * (t * 0.7).sin().abs() + 0.10 * (t * 3.1).cos().abs()).min(1.0)
        })
        .collect();
    rod_player::waveform::Picos {
        por_segundo,
        max,
        rms: Vec::new(),
    }
}

// ── Fixtures de recientes ───────────────────────────────────────────────────

fn reciente(
    nombre: &str,
    lineas: usize,
    hablantes: usize,
    horas: u64,
    duracion: f64,
    tira: &[(&str, f32)],
) -> Reciente {
    Reciente {
        ruta: PathBuf::from(format!("C:/ui-audit/{nombre}")),
        lineas,
        hablantes,
        fecha: FechaRelativa { dias: 0, horas },
        duracion,
        tira: tira.iter().map(|(c, s)| ((*c).to_owned(), *s)).collect(),
    }
}

/// Cinco recientes: incluye uno sin video y uno vacío. Las miniaturas se
/// escriben en la caché real de la app (es una caché, se sobrescribe).
pub fn recientes_fixture() -> Vec<Reciente> {
    let frame = frame_prueba();
    let paleta = rod_core::color_hablante::paleta_identidad();
    let lista = vec![
        reciente(
            "documental-final.cdp",
            543,
            7,
            0,
            1440.0,
            &[
                (paleta[0], 320.0),
                (paleta[1], 260.0),
                (paleta[3], 410.0),
                (paleta[6], 300.0),
                (paleta[8], 150.0),
            ],
        ),
        // Va segundo a propósito: la tarjeta sin video (placeholder + chip
        // ámbar) tiene que entrar en la fila visible de tres.
        reciente(
            "sin-video.cdp",
            128,
            4,
            72,
            600.0,
            &[(paleta[7], 260.0), (paleta[10], 340.0)],
        ),
        reciente(
            "presa.cdp",
            19,
            3,
            2,
            92.0,
            &[(paleta[2], 30.0), (paleta[5], 24.0), (paleta[9], 38.0)],
        ),
        reciente(
            "entrevista-calle.cdp",
            1,
            1,
            30,
            12.0,
            &[(paleta[4], 12.0)],
        ),
        reciente("vacio.cdp", 0, 0, 240, 0.0, &[]),
    ];
    let cache = rod_app::thumbnail::carpeta_cache();
    for (i, item) in lista.iter().enumerate() {
        if item.ruta.file_name().is_some_and(|n| n == "sin-video.cdp") {
            // Sin miniatura: la tarjeta debe mostrar el placeholder y el chip.
            let _ = std::fs::remove_file(rod_app::thumbnail::ruta_cache(&item.ruta));
            continue;
        }
        let mut tinte = frame.clone();
        for pixel in tinte.as_chunks_mut::<4>().0.iter_mut() {
            let f = 0.55 + 0.08 * i as f32;
            pixel[0] = (pixel[0] as f32 * f).min(255.0) as u8;
            pixel[1] = (pixel[1] as f32 * f).min(255.0) as u8;
            pixel[2] = (pixel[2] as f32 * f).min(255.0) as u8;
        }
        let _ = rod_app::thumbnail::guardar_en(&cache, &item.ruta, &tinte, 1920, 1080);
    }
    lista
}

pub fn aviso_recovery() -> rod_app::recovery_io::AvisoRecovery {
    let proyecto = make_proyecto(19, 3, 1, 92.0);
    let ahora = rod_app::recovery_io::ms_ahora();
    let recovery = rod_core::construir_recovery(proyecto, "C:/ui-audit/presa.cdp", ahora);
    let texto = rod_core::formatear_recovery(&recovery, "presa.cdp", ahora);
    rod_app::recovery_io::AvisoRecovery { recovery, texto }
}

// ── Construcción del editor ────────────────────────────────────────────────

pub fn editor_de(proyecto: Proyecto, panel: Panel, duracion: f64) -> EditorApp {
    let mut app = EditorApp::desde_proyecto_en(proyecto, None, 0.0, &base_temporal())
        .expect("abre el clip de prueba");
    *app.presets_mut() = presets_fixture();
    app.preset_seleccionado = Some("p-kiwi".into());
    // El overlay se juzga mejor con un texto largo; se elige el más cercano a
    // un tercio del video para que el playhead caiga dentro de la vista.
    let candidatas: Vec<(usize, f64, usize, String)> = app
        .proyecto()
        .captions
        .iter()
        .enumerate()
        .map(|(i, caption)| {
            (
                i,
                (caption.inicio + caption.fin) / 2.0,
                caption.texto.chars().count(),
                caption.id.clone(),
            )
        })
        .collect();
    let objetivo = duracion / 3.0;
    let elegida = candidatas
        .iter()
        .filter(|c| c.2 >= 100)
        .min_by(|a, b| {
            (a.1 - objetivo)
                .abs()
                .total_cmp(&(b.1 - objetivo).abs())
        })
        .or_else(|| candidatas.first());
    let (posicion, id) = match elegida {
        Some(c) => (c.1, Some(c.3.clone())),
        None => (0.0, None),
    };
    if let Some(id) = id {
        editor_app::update(&mut app, MensajeEditor::Seleccionar(id));
    }
    if panel != Panel::Subtitulos {
        editor_app::update(&mut app, MensajeEditor::Panel(panel));
    }
    app.preparar_captura(duracion, posicion, frame_prueba());
    app.picos = Some(picos_fixture(duracion));
    app
}

// ── Marcos de pantalla ─────────────────────────────────────────────────────

pub struct MarcoInicio {
    pub estado: EstadoInicio,
    pub prefs: Preferencias,
}

pub struct MarcoEditor {
    pub editor: EditorApp,
    pub medidas: ui::editor::MedidasEditor,
    pub gpu: Arc<Mutex<Option<rod_app::video::Gpu>>>,
    pub formato: wgpu::TextureFormat,
    /// Estado del punto de la cabecera: `true` = cambios sin guardar.
    pub sucio: bool,
    /// Ancho del eje que publica el widget de timeline en el primer frame.
    pub ancho_eje: Option<f32>,
}

fn fondo(ctx: &egui::Context) {
    ctx.layer_painter(egui::LayerId::background())
        .rect_filled(ctx.content_rect(), 0.0, tema::FONDO);
}

pub fn construir_inicio(
    ancho: f32,
    alto: f32,
    estado: EstadoInicio,
    idioma: Idioma,
) -> Harness<'static, MarcoInicio> {
    let textos = idioma.textos();
    let mut harness = Harness::builder()
        .wgpu()
        .with_size(egui::vec2(ancho, alto))
        .build_state(
            move |ctx, marco: &mut MarcoInicio| {
                fondo(ctx);
                let _ = chrome::mostrar(
                    ctx,
                    textos,
                    &MapaAtajos::por_defecto(),
                    chrome::EstadoCabecera {
                        en_inicio: true,
                        proyecto: None,
                        sucio: false,
                    },
                );
                egui::CentralPanel::default()
                    .frame(tema::panel(tema::FONDO))
                    .show(ctx, |ui| {
                        let _ = ui::inicio::mostrar(ui, &mut marco.estado, textos, &mut marco.prefs);
                    });
            },
            MarcoInicio {
                estado,
                prefs: Preferencias::default(),
            },
        );
    tema::aplicar(&harness.ctx);
    harness.run();
    harness
}

pub fn construir_editor(
    ancho: f32,
    alto: f32,
    editor: EditorApp,
    idioma: Idioma,
) -> Harness<'static, MarcoEditor> {
    let textos = idioma.textos();
    let nombre = editor_app::nombre_proyecto(&editor).to_owned();
    let medidas = ui::editor::MedidasEditor::default();
    let mut harness = Harness::builder()
        .wgpu()
        .with_size(egui::vec2(ancho, alto))
        .build_state(
            move |ctx, marco: &mut MarcoEditor| {
                fondo(ctx);
                let _ = chrome::mostrar(
                    ctx,
                    textos,
                    &MapaAtajos::por_defecto(),
                    chrome::EstadoCabecera {
                        en_inicio: false,
                        proyecto: Some(&nombre),
                        sucio: marco.sucio,
                    },
                );
                let formato = marco.formato;
                let gpu = Arc::clone(&marco.gpu);
                for mensaje in ui::editor::show(
                    ctx,
                    &mut marco.editor,
                    formato,
                    gpu,
                    textos,
                    &MapaAtajos::por_defecto(),
                    &mut marco.medidas,
                ) {
                    if let MensajeEditor::Timeline(rod_app::timeline::Mensaje::Ancho(ancho)) =
                        mensaje
                    {
                        marco.ancho_eje = Some(ancho);
                    }
                }
            },
            MarcoEditor {
                editor,
                medidas,
                gpu: Arc::new(Mutex::new(None)),
                formato: wgpu::TextureFormat::Rgba8Unorm,
                sucio: true,
                ancho_eje: None,
            },
        );
    tema::aplicar(&harness.ctx);
    harness.run();
    let ancho_eje = harness.state().ancho_eje.unwrap_or(0.0);
    harness.state_mut().editor.encuadrar_timeline(ancho_eje);
    harness.run();
    harness
}

// ── Capas: ajustes, atajos, toast ──────────────────────────────────────────

pub struct MarcoAjustes {
    pub inicio: MarcoInicio,
    pub ajustes: ui::ajustes::EstadoAjustes,
}

pub struct MarcoAtajos {
    pub inicio: MarcoInicio,
    pub atajos: ui::atajos::EstadoAtajos,
}

pub struct MarcoToast {
    pub editor: MarcoEditor,
    pub toasts: ui::toast::Toasts,
}

pub fn capturar_ajustes(
    fase: &str,
    nombre: &str,
    ancho: f32,
    alto: f32,
    estado: EstadoInicio,
    idioma: Idioma,
    pestana: ui::ajustes::Pestana,
) {
    let textos = idioma.textos();
    let mut prefs = Preferencias::default();
    prefs.idioma = idioma;
    let mut ajustes = ui::ajustes::EstadoAjustes::default();
    ajustes.pestana = pestana;
    ajustes.abrir();
    let mut harness = Harness::builder()
        .wgpu()
        .with_size(egui::vec2(ancho, alto))
        .build_state(
            move |ctx, marco: &mut MarcoAjustes| {
                fondo(ctx);
                let _ = chrome::mostrar(
                    ctx,
                    textos,
                    &MapaAtajos::por_defecto(),
                    chrome::EstadoCabecera {
                        en_inicio: true,
                        proyecto: None,
                        sucio: false,
                    },
                );
                egui::CentralPanel::default()
                    .frame(tema::panel(tema::FONDO))
                    .show(ctx, |ui| {
                        let _ = ui::inicio::mostrar(
                            ui,
                            &mut marco.inicio.estado,
                            textos,
                            &mut marco.inicio.prefs,
                        );
                    });
                let _ = ui::ajustes::mostrar(ctx, &mut marco.inicio.prefs, &mut marco.ajustes);
            },
            MarcoAjustes {
                inicio: MarcoInicio { estado, prefs },
                ajustes,
            },
        );
    tema::aplicar(&harness.ctx);
    harness.run();
    guardar(&mut harness, fase, nombre);
}

pub fn capturar_atajos(
    fase: &str,
    nombre: &str,
    ancho: f32,
    alto: f32,
    estado: EstadoInicio,
    idioma: Idioma,
) {
    let textos = idioma.textos();
    let mut prefs = Preferencias::default();
    prefs.idioma = idioma;
    let mut harness = Harness::builder()
        .wgpu()
        .with_size(egui::vec2(ancho, alto))
        .build_state(
            move |ctx, marco: &mut MarcoAtajos| {
                fondo(ctx);
                let _ = chrome::mostrar(
                    ctx,
                    textos,
                    &MapaAtajos::por_defecto(),
                    chrome::EstadoCabecera {
                        en_inicio: true,
                        proyecto: None,
                        sucio: false,
                    },
                );
                egui::CentralPanel::default()
                    .frame(tema::panel(tema::FONDO))
                    .show(ctx, |ui| {
                        let _ = ui::inicio::mostrar(
                            ui,
                            &mut marco.inicio.estado,
                            textos,
                            &mut marco.inicio.prefs,
                        );
                    });
                ui::atajos::mostrar(
                    ctx,
                    textos,
                    &MapaAtajos::por_defecto(),
                    &mut marco.atajos,
                );
            },
            MarcoAtajos {
                inicio: MarcoInicio { estado, prefs },
                atajos: ui::atajos::EstadoAtajos { abierto: true },
            },
        );
    tema::aplicar(&harness.ctx);
    harness.run();
    guardar(&mut harness, fase, nombre);
}

pub fn capturar_toast(
    fase: &str,
    nombre: &str,
    ancho: f32,
    alto: f32,
    proyecto: Proyecto,
    idioma: Idioma,
    texto: &str,
) {
    let textos = idioma.textos();
    let editor = editor_de(proyecto, Panel::Subtitulos, 1440.0);
    let mut toasts = ui::toast::Toasts::default();
    toasts.push(texto.to_owned(), true);
    let nombre_proyecto = editor_app::nombre_proyecto(&editor).to_owned();
    let mut harness = Harness::builder()
        .wgpu()
        .with_size(egui::vec2(ancho, alto))
        .build_state(
            move |ctx, marco: &mut MarcoToast| {
                fondo(ctx);
                let _ = chrome::mostrar(
                    ctx,
                    textos,
                    &MapaAtajos::por_defecto(),
                    chrome::EstadoCabecera {
                        en_inicio: false,
                        proyecto: Some(&nombre_proyecto),
                        sucio: true,
                    },
                );
                let formato = marco.editor.formato;
                let gpu = Arc::clone(&marco.editor.gpu);
                for mensaje in ui::editor::show(
                    ctx,
                    &mut marco.editor.editor,
                    formato,
                    gpu,
                    textos,
                    &MapaAtajos::por_defecto(),
                    &mut marco.editor.medidas,
                ) {
                    if let MensajeEditor::Timeline(rod_app::timeline::Mensaje::Ancho(ancho)) =
                        mensaje
                    {
                        marco.editor.ancho_eje = Some(ancho);
                    }
                }
                let _ = marco.toasts.mostrar(ctx, textos);
            },
            MarcoToast {
                editor: MarcoEditor {
                    editor,
                    medidas: ui::editor::MedidasEditor::default(),
                    gpu: Arc::new(Mutex::new(None)),
                    formato: wgpu::TextureFormat::Rgba8Unorm,
                    sucio: true,
                    ancho_eje: None,
                },
                toasts,
            },
        );
    tema::aplicar(&harness.ctx);
    harness.run();
    let ancho_eje = harness.state().editor.ancho_eje.unwrap_or(0.0);
    harness
        .state_mut()
        .editor
        .editor
        .encuadrar_timeline(ancho_eje);
    harness.run();
    guardar(&mut harness, fase, nombre);
}

// ── Guardado ───────────────────────────────────────────────────────────────
pub fn guardar<S>(harness: &mut Harness<'_, S>, fase: &str, nombre: &str) {
    if let Ok(solo) = std::env::var("UI_AUDIT_SOLO")
        && !solo.split(',').any(|token| nombre.contains(token.trim()))
    {
        return;
    }
    let imagen = harness.render().expect("renderiza con wgpu");
    let raiz = carpeta(fase);
    std::fs::create_dir_all(&raiz).expect("crea la carpeta de capturas");
    let ruta = raiz.join(format!("{nombre}.png"));
    image::DynamicImage::ImageRgba8(imagen)
        .save(&ruta)
        .expect("escribe el PNG");
    println!("{}", ruta.display());
}

pub fn capturar_inicio(
    fase: &str,
    nombre: &str,
    ancho: f32,
    alto: f32,
    estado: EstadoInicio,
    idioma: Idioma,
    ajuste: impl FnOnce(&mut Harness<'static, MarcoInicio>),
) {
    let mut harness = construir_inicio(ancho, alto, estado, idioma);
    ajuste(&mut harness);
    guardar(&mut harness, fase, nombre);
}

pub fn capturar_editor(
    fase: &str,
    nombre: &str,
    ancho: f32,
    alto: f32,
    proyecto: Proyecto,
    panel: Panel,
    duracion: f64,
    idioma: Idioma,
    ajuste: impl FnOnce(&mut Harness<'static, MarcoEditor>),
) {
    let editor = editor_de(proyecto, panel, duracion);
    let mut harness = construir_editor(ancho, alto, editor, idioma);
    ajuste(&mut harness);
    guardar(&mut harness, fase, nombre);
}

pub fn abrir_menu_editor(harness: &mut Harness<'static, MarcoEditor>, etiqueta: &str) {
    harness.get_by_label(etiqueta).click();
    harness.run();
}

/// Simula que el SO está arrastrando un archivo sobre la ventana.
pub fn arrastrando_archivo(harness: &mut Harness<'static, MarcoInicio>) {
    for _ in 0..3 {
        harness.input_mut().hovered_files.push(egui::HoveredFile {
            path: None,
            mime: "video/mp4".into(),
        });
        harness.step();
    }
}

/// Mueve el puntero y deja correr el reloj para que el tooltip (0.4 s) salga.
pub fn apuntar_y_esperar(harness: &mut Harness<'static, MarcoEditor>, x: f32, y: f32) {
    for _ in 0..32 {
        harness
            .input_mut()
            .events
            .push(egui::Event::PointerMoved(egui::pos2(x, y)));
        harness.step();
    }
}

// ── Hoja de contacto ───────────────────────────────────────────────────────

/// Compone `index.png` con todas las capturas de la carpeta, 5 por fila.
pub fn hoja_contacto(fase: &str) {
    let raiz = carpeta(fase);
    let mut entradas: Vec<(String, egui::ColorImage)> = std::fs::read_dir(&raiz)
        .expect("lee la carpeta de capturas")
        .filter_map(|entrada| entrada.ok())
        .map(|entrada| entrada.path())
        .filter(|ruta| ruta.extension().is_some_and(|e| e == "png"))
        .filter(|ruta| ruta.file_name().is_some_and(|n| n != "index.png"))
        // Una sola medida por hoja: 87 capturas completas dan una tira de
        // 5000 px que nadie revisa. Las demás quedan en archivos.
        .filter(|ruta| {
            let nombre = ruta.file_name().unwrap_or_default().to_string_lossy();
            nombre.contains("1600x900") || nombre.contains("-es")
        })
        .filter_map(|ruta| {
            let nombre = ruta.file_stem()?.to_string_lossy().into_owned();
            let imagen = image::open(&ruta).ok()?.to_rgba8();
            // La textura de egui no pasa de 2048 px de lado: la miniatura de la
            // hoja se reduce a 440 px de ancho (16:9).
            let imagen = image::imageops::resize(
                &imagen,
                440,
                247,
                image::imageops::FilterType::Triangle,
            );
            let (ancho, alto) = imagen.dimensions();
            let color = egui::ColorImage::from_rgba_unmultiplied(
                [ancho as usize, alto as usize],
                imagen.as_raw(),
            );
            Some((nombre, color))
        })
        .collect();
    entradas.sort_by(|a, b| a.0.cmp(&b.0));
    if entradas.is_empty() {
        return;
    }

    const COLUMNAS: usize = 5;
    const CELDA: f32 = 456.0;
    const ALTO_IMG: f32 = 256.0;
    let filas = entradas.len().div_ceil(COLUMNAS);
    let ancho = CELDA * COLUMNAS as f32 + 40.0;
    let alto = (ALTO_IMG + 42.0) * filas as f32 + 40.0;

    let mut harness = Harness::builder()
        .wgpu()
        .with_size(egui::vec2(ancho, alto))
        .build_ui_state(
            |ui, entradas: &mut Vec<(String, egui::ColorImage)>| {
                fondo(ui.ctx());
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        ui.add_space(16.0);
                        for fila in entradas.chunks(COLUMNAS) {
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                for (nombre, imagen) in fila {
                                    ui.vertical(|ui| {
                                        ui.set_width(CELDA - 8.0);
                                        let textura = ui.ctx().load_texture(
                                            format!("hoja-{nombre}"),
                                            imagen.clone(),
                                            egui::TextureOptions::LINEAR,
                                        );
                                        let aspecto =
                                            textura.size()[0] as f32 / textura.size()[1] as f32;
                                        let tam = egui::vec2(
                                            (ALTO_IMG * aspecto).min(CELDA - 8.0),
                                            ALTO_IMG,
                                        );
                                        ui.add(egui::Image::new((textura.id(), tam)));
                                        ui.label(
                                            egui::RichText::new(nombre)
                                                .font(tema::fuente(tema::Estilo::Mono))
                                                .color(tema::TEXTO_2),
                                        );
                                    });
                                }
                            });
                            ui.add_space(8.0);
                        }
                    });
            },
            entradas,
        );
    tema::aplicar(&harness.ctx);
    harness.run();
    let imagen = harness.render().expect("renderiza la hoja");
    let ruta = raiz.join("index.png");
    image::DynamicImage::ImageRgba8(imagen)
        .save(&ruta)
        .expect("escribe index.png");
    println!("{}", ruta.display());
}

/// El nombre del archivo de captura: `pantalla__estado__WxH.png`.
pub fn nombre_captura(pantalla: &str, estado: &str, ancho: f32, alto: f32) -> String {
    format!("{pantalla}__{estado}__{}", etiqueta_tamano(ancho, alto))
}

/// Variante sin tamaño (para hojas de una sola medida).
pub fn nombre_simple(pantalla: &str, estado: &str) -> String {
    format!("{pantalla}__{estado}")
}
