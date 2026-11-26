#[test]
fn sin_scrollbar_cuando_todas_caben() {
    use rod_app::ui::timeline::{max_scroll_pistas, scroll_desde_barra};

    assert_eq!(max_scroll_pistas(4, 700.0), 0.0);
    assert_eq!(scroll_desde_barra(350.0, 700.0, 4), 0.0);
}

#[test]
fn scrollbar_mapa_el_tamanio_de_la_barra() {
    use rod_app::ui::timeline::scroll_desde_barra;

    assert_eq!(scroll_desde_barra(0.0, 700.0, 15), 0.0);
    assert_eq!(scroll_desde_barra(700.0, 700.0, 15), 3.0);
}

#[test]
fn limites_de_scroll_por_pistas() {
    use rod_app::ui::timeline::{desplazar, filas_visibles, max_scroll_pistas};

    assert_eq!(filas_visibles(700.0), 12);
    assert_eq!(max_scroll_pistas(15, 700.0), 3.0);
    assert_eq!(max_scroll_pistas(4, 700.0), 0.0);
    assert_eq!(desplazar(2.0, 5.0, 15, 700.0), 3.0);
    assert_eq!(desplazar(0.0, -1.0, 15, 700.0), 0.0);
}

#[test]
fn clic_con_scroll_selecciona_el_carril_visible() {
    use rod_app::timeline::{ALTO_CARRIL, ALTO_REGLA};
    use rod_app::ui::timeline::punto;

    let p = punto(egui::vec2(300.0, ALTO_REGLA + 20.0), 2.0);
    assert_eq!(p.y, ALTO_REGLA + 2.0 * ALTO_CARRIL + 20.0);
}

#[test]
fn regla_sigue_busca_tiempo_con_scroll() {
    use rod_app::timeline::ALTO_REGLA;
    use rod_app::ui::timeline::punto;

    let p = punto(egui::vec2(300.0, 8.0), 2.0);
    assert!(p.y < ALTO_REGLA);
}

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
