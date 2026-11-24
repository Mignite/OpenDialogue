//! Capturas deterministas de la UI con egui_kittest, sin abrir ventana.
//!
//! Uso: `cargo run -p rod-app --example capturas -- antes`
//! Las escribe en `docs/checklists/rediseno/`.

use std::path::PathBuf;

use egui_kittest::Harness;
use rod_app::inicio::{self, EstadoInicio};
use rod_app::recientes::Reciente;

fn reciente(nombre: &str, lineas: usize, hablantes: usize) -> Reciente {
    Reciente {
        ruta: PathBuf::from(format!("C:/proyectos/{nombre}")),
        lineas,
        hablantes,
        fecha: Default::default(),
        duracion: 0.0,
        tira: Vec::new(),
    }
}

fn capturar(
    nombre: &str,
    ancho: f32,
    alto: f32,
    estado: EstadoInicio,
    idioma: rod_app::textos::Idioma,
) {
    let mut harness = Harness::builder()
        .wgpu()
        .with_size(egui::vec2(ancho, alto))
        .build_ui_state(
            |ui, estado: &mut EstadoInicio| {
                // El harness usa Frame::NONE: sin esto el fondo fuera del
                // contenido del Inicio sale transparente (negro) en el PNG.
                let ctx = ui.ctx().clone();
                let fondo = ctx.layer_painter(egui::LayerId::background());
                fondo.rect_filled(ctx.content_rect(), 0.0, rod_app::ui::tema::FONDO);
                let mut prefs = rod_app::preferencias::Preferencias::default();
                rod_app::ui::inicio::mostrar(ui, estado, idioma.textos(), &mut prefs);
            },
            estado,
        );
    // La app aplica el tema en `main.rs`; el arnés arranca con el estilo de egui.
    rod_app::ui::tema::aplicar(&harness.ctx);
    harness.run();

    guardar(&mut harness, nombre);
}

fn capturar_con_cabecera(nombre: &str, ancho: f32, alto: f32, estado: EstadoInicio) {
    let mut harness = Harness::builder()
        .wgpu()
        .with_size(egui::vec2(ancho, alto))
        .build_state(
            |ctx, estado: &mut EstadoInicio| {
                rod_app::ui::chrome::mostrar(
                    ctx,
                    &rod_app::textos::EN,
                    &rod_app::comandos::MapaAtajos::por_defecto(),
                    rod_app::ui::chrome::EstadoCabecera {
                        en_inicio: true,
                        proyecto: None,
                        sucio: false,
                    },
                );
                egui::CentralPanel::default()
                    .frame(rod_app::ui::tema::panel(rod_app::ui::tema::FONDO))
                    .show(ctx, |ui| {
                        let mut prefs = rod_app::preferencias::Preferencias::default();
                        rod_app::ui::inicio::mostrar(ui, estado, &rod_app::textos::EN, &mut prefs);
                    });
            },
            estado,
        );
    rod_app::ui::tema::aplicar(&harness.ctx);
    harness.run();
    guardar(&mut harness, nombre);
}

fn guardar<S>(harness: &mut Harness<'_, S>, nombre: &str) {
    let imagen = harness.render().expect("renderiza con wgpu");
    let raiz =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/checklists/rediseno");
    std::fs::create_dir_all(&raiz).expect("crea docs/checklists/rediseno");
    let ruta = raiz.join(format!("{nombre}.png"));
    image::DynamicImage::ImageRgba8(imagen)
        .save(&ruta)
        .expect("escribe el PNG");
    println!("{}", ruta.display());
}

fn capturar_editor(nombre: &str, ancho: f32, alto: f32, idioma: rod_app::textos::Idioma) {
    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let mut proyecto = rod_core::Proyecto::nuevo(video);
    for i in 0..7 {
        proyecto.hablantes.push(rod_core::Hablante {
            id: format!("h{i}"),
            nombre: format!("Hablante {i}"),
            tecla: (i + 1).to_string(),
            color: String::new(),
            preset_id: None,
        });
    }
    for n in 0..40 {
        let inicio = n as f64 * 0.38;
        proyecto.captions.push(rod_core::Caption {
            id: format!("cap-{n}"),
            inicio,
            fin: inicio + 0.3,
            texto: format!("Línea {n} · hablante {}", n % 7),
            hablante_id: Some(format!("h{}", n % 7)),
        });
    }
    let app = rod_app::editor_app::EditorApp::desde_proyecto_en(
        proyecto,
        None,
        0.0,
        &std::env::temp_dir(),
    )
    .expect("abre el clip de prueba");
    let mut harness = Harness::builder()
        .wgpu()
        .with_size(egui::vec2(ancho, alto))
        .build_state(
            |ctx, (editor, medidas)| {
                rod_app::ui::tema::aplicar(ctx);
                rod_app::ui::editor::show(
                    ctx,
                    editor,
                    wgpu::TextureFormat::Rgba8Unorm,
                    std::sync::Arc::new(std::sync::Mutex::new(None)),
                    idioma.textos(),
                    &rod_app::comandos::MapaAtajos::por_defecto(),
                    medidas,
                );
            },
            (app, rod_app::ui::editor::MedidasEditor::default()),
        );
    harness.run();
    guardar(&mut harness, nombre);
}

fn main() {
    let fase = std::env::args().nth(1).unwrap_or_else(|| "antes".into());

    capturar_con_cabecera(
        &format!("{fase}-cabecera-inicio"),
        1440.0,
        900.0,
        inicio::nuevo(vec![reciente("RobloxTerror.cdp", 543, 7)], None),
    );
    capturar(
        &format!("{fase}-inicio-vacio"),
        1440.0,
        900.0,
        inicio::nuevo(Vec::new(), None),
        rod_app::textos::Idioma::En,
    );
    capturar(
        &format!("{fase}-inicio-3"),
        1440.0,
        900.0,
        inicio::nuevo(
            vec![
                reciente("presa.cdp", 19, 3),
                reciente("RobloxTerror.cdp", 543, 7),
                reciente("proyecto.cdp", 543, 7),
            ],
            None,
        ),
        rod_app::textos::Idioma::En,
    );

    if fase == "diagnostico" {
        capturar(
            "diagnostico-inicio-1914",
            1914.0,
            1015.0,
            inicio::nuevo(
                vec![
                    reciente("presa.cdp", 19, 3),
                    reciente("RobloxTerror.cdp", 543, 7),
                    reciente("proyecto.cdp", 543, 7),
                ],
                None,
            ),
            rod_app::textos::Idioma::En,
        );
    }

    if fase == "finales" {
        for ancho in [1100.0, 1440.0, 2560.0] {
            let alto = if ancho == 1100.0 { 700.0 } else { 900.0 };
            capturar(
                &format!("finales-inicio-{ancho:.0}"),
                ancho,
                alto,
                inicio::nuevo(
                    vec![
                        reciente("presa.cdp", 19, 3),
                        reciente("RobloxTerror.cdp", 543, 7),
                    ],
                    None,
                ),
                rod_app::textos::Idioma::En,
            );
            capturar_editor(
                &format!("finales-editor-{ancho:.0}"),
                ancho,
                alto,
                rod_app::textos::Idioma::En,
            );
        }
        capturar(
            "finales-inicio-es-1100",
            1100.0,
            700.0,
            inicio::nuevo(vec![reciente("RobloxTerror.cdp", 543, 7)], None),
            rod_app::textos::Idioma::Es,
        );
    }
}
