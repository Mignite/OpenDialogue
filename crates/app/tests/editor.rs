use std::sync::{Arc, Mutex};

use egui_kittest::kittest::Queryable;
use rod_app::comandos::MapaAtajos;
use rod_app::editor_app::{EditorApp, MensajeEditor};
use rod_app::textos::ES;
use rod_app::ui::editor::MedidasEditor;

type Estado = (EditorApp, MedidasEditor);

fn editor() -> EditorApp {
    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    EditorApp::desde_proyecto_en(
        rod_core::Proyecto::nuevo(video),
        None,
        0.0,
        &std::env::temp_dir(),
    )
    .expect("abre el clip de prueba")
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

/// El ancho del panel no debe crecer solo. Un `add_space(available - ancho)`
/// en una fila desbordaba el `min_rect` un píxel por frame y el `SidePanel`
/// lo guardaba como estado, hasta tocar el máximo.
#[test]
fn el_ancho_del_panel_no_crece_solo() {
    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let mut proyecto = rod_core::Proyecto::nuevo(video);
    proyecto.captions.push(rod_core::Caption {
        id: "cap-1".into(),
        inicio: 7.13,
        fin: 9.13,
        texto: "hola".into(),
        hablante_id: None,
    });
    let mut app = EditorApp::desde_proyecto_en(proyecto, None, 0.0, &std::env::temp_dir())
        .expect("abre el clip de prueba");
    rod_app::editor_app::update(&mut app, MensajeEditor::Seleccionar("cap-1".into()));

    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_state(
            |ctx, (editor, medidas): &mut Estado| {
                rod_app::ui::tema::aplicar(ctx);
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
            (app, MedidasEditor::default()),
        );

    for _ in 0..10 {
        h.run();
    }
    let inicial = h.state().1.ancho_panel;
    for _ in 0..30 {
        h.run();
    }
    let posterior = h.state().1.ancho_panel;
    assert!(
        (posterior - inicial).abs() < 1.0,
        "el ancho creció de {inicial} a {posterior}"
    );
}
