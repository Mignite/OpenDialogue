use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use egui_kittest::kittest::Queryable;
use rod_app::comandos::MapaAtajos;
use rod_app::editor_app::EditorApp;
use rod_app::textos::ES;
use rod_app::ui::editor::MedidasEditor;

fn editor_543() -> EditorApp {
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
    for n in 0..543 {
        proyecto.captions.push(rod_core::Caption {
            id: format!("c{n}"),
            inicio: n as f64 * 0.03,
            fin: n as f64 * 0.03 + 0.025,
            texto: format!("Línea {n}"),
            hablante_id: Some(format!("h{}", n % 7)),
        });
    }
    EditorApp::desde_proyecto_en(proyecto, None, 0.0, &std::env::temp_dir())
        .expect("abre el clip de prueba")
}

#[test]
fn a_1100_px_el_editor_sigue_dibujando_sus_pestanas() {
    let app = editor_543();
    let medidas = MedidasEditor::default();
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1100.0, 700.0))
        .build_state(
            |ctx, (editor, medidas): &mut (EditorApp, MedidasEditor)| {
                rod_app::ui::editor::show(
                    ctx,
                    editor,
                    wgpu::TextureFormat::Bgra8Unorm,
                    Arc::new(Mutex::new(None)),
                    &ES,
                    &MapaAtajos::por_defecto(),
                    medidas,
                );
            },
            (app, medidas),
        );
    h.run();
    for etiqueta in ["Subtítulos", "Hablantes", "Estilos"] {
        assert!(h.query_all_by_label(etiqueta).count() >= 1, "{etiqueta}");
    }
}

#[test]
fn el_pintado_del_editor_con_543_lineas_baja_de_ocho_ms() {
    let mut app = editor_543();
    let ctx = egui::Context::default();
    let inicio = Instant::now();
    for _ in 0..30 {
        let _ = ctx.run(Default::default(), |ctx| {
            rod_app::ui::editor::show(
                ctx,
                &mut app,
                wgpu::TextureFormat::Bgra8Unorm,
                Arc::new(Mutex::new(None)),
                &ES,
                &MapaAtajos::por_defecto(),
                &mut MedidasEditor::default(),
            );
        });
    }
    let media = inicio.elapsed() / 30;
    // En debug el painter de egui no está optimizado; el objetivo de 8 ms se
    // mide en release, que es lo que se entrega. El tope de debug cubre el
    // coste extra de los anillos de foco y la toolbar ampliada sin confundirlo
    // con una regresión de 60 fps.
    let limite = if cfg!(debug_assertions) {
        // Solo una cordura: en debug egui no está optimizado y el resultado
        // depende de la carga del runner (31 ms en local, 43 ms en CI). El
        // objetivo de 60 fps se verifica con el gate de release de abajo.
        Duration::from_millis(120)
    } else {
        Duration::from_millis(8)
    };
    assert!(media < limite, "media {media:?}, límite {limite:?}");
}
