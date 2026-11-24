use egui::Key;
use egui_kittest::kittest::Queryable;
use rod_app::editor_app::EditorApp;
use rod_app::textos::ES;

fn editor() -> EditorApp {
    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let mut proyecto = rod_core::Proyecto::nuevo(video);
    proyecto.captions.push(rod_core::Caption {
        id: "cap-1".into(),
        inicio: 1.0,
        fin: 2.0,
        texto: "texto original".into(),
        hablante_id: None,
    });
    EditorApp::desde_proyecto_en(proyecto, None, 0.0, &std::env::temp_dir())
        .expect("abre el clip de prueba")
}

#[test]
fn doble_clic_abre_editor_y_escape_no_cambia_el_texto() {
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, editor: &mut EditorApp| {
                for mensaje in rod_app::ui::paneles::subtitulos::mostrar(ui, editor, &ES) {
                    rod_app::editor_app::update(editor, mensaje);
                }
            },
            editor(),
        );
    h.run();
    h.get_by_label("Editar").click();
    h.run();
    assert!(h.state().texto_inline_activo());
    // El campo inline debe tomar el foco: si no, escribir dispara atajos.
    assert!(h.ctx.memory(|memoria| memoria.focused().is_some()));
    h.key_press(Key::Escape);
    h.run();
    assert_eq!(h.state().proyecto().captions[0].texto, "texto original");
    assert!(!h.state().texto_inline_activo());
}
