#[test]
fn alto_extra_no_guarda_altura_hinchada() {
    use rod_app::editor_app::EditorApp;
    use rod_app::ui::editor::MedidasEditor;
    use rod_app::video::Gpu;
    use std::sync::{Arc, Mutex};

    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let mut editor = EditorApp::desde_video(video).expect("abre el clip de prueba");
    editor.texto.lineas = 3;
    let estado = (
        editor,
        MedidasEditor {
            alto_timeline: 300.0,
            ..Default::default()
        },
        Arc::new(Mutex::new(None)),
    );
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1200.0, 700.0))
        .build_state(
            |ctx, (e, m, gpu): &mut (EditorApp, MedidasEditor, Arc<Mutex<Option<Gpu>>>)| {
                let _ = rod_app::ui::editor::show(
                    ctx,
                    e,
                    wgpu::TextureFormat::Rgba8Unorm,
                    gpu.clone(),
                    &rod_app::textos::EN,
                    &rod_app::comandos::MapaAtajos::por_defecto(),
                    m,
                );
            },
            estado,
        );
    h.run();
    assert_eq!(h.state().1.alto_timeline, 300.0);
}

#[test]
fn tope_respeta_la_ventana_y_el_suelo() {
    assert_eq!(rod_app::ui::editor::alto_maximo_timeline(900.0), 630.0);
    assert_eq!(rod_app::ui::editor::alto_maximo_timeline(200.0), 160.0);
    let prefs = rod_app::preferencias::Preferencias {
        alto_timeline: 50_000.0,
        ..Default::default()
    };
    let medidas = rod_app::ui::editor::MedidasEditor::from(&prefs);
    assert_eq!(medidas.alto_timeline, 50_000.0);
}
