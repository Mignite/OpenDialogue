use std::sync::{Arc, Mutex};

use egui_kittest::kittest::Queryable;
use rod_app::comandos::MapaAtajos;
use rod_app::editor_app::EditorApp;
use rod_app::textos::ES;
use rod_app::ui::editor::MedidasEditor;

type Estado = (EditorApp, MedidasEditor);

fn editor() -> EditorApp {
    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    EditorApp::desde_video(video).expect("abre el clip de prueba")
}

#[test]
fn el_editor_pinta_monitor_panel_y_timeline() {
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_state(
            |ctx, (editor, medidas): &mut Estado| {
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
            (editor(), MedidasEditor::default()),
        );
    h.run();
    for etiqueta in ["Subtítulos", "Hablantes", "Estilos"] {
        assert!(h.query_all_by_label(etiqueta).count() >= 1, "{etiqueta}");
    }
}
