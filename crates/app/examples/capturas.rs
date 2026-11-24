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

fn capturar(nombre: &str, ancho: f32, alto: f32, estado: EstadoInicio) {
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
                rod_app::ui::inicio::mostrar(ui, estado);
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
                        rod_app::ui::inicio::mostrar(ui, estado);
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
    );
}
