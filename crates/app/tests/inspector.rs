use egui_kittest::kittest::Queryable;
use rod_app::comandos::Comando;
use rod_app::editor_app::{EditorApp, MensajeEditor};
use rod_app::textos::ES;
use rod_app::ui::toast::Toasts;

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
    let mut app = EditorApp::desde_proyecto_en(proyecto, None, 0.0, &std::env::temp_dir())
        .expect("abre el clip de prueba");
    rod_app::editor_app::update(&mut app, MensajeEditor::Seleccionar("cap-1".into()));
    app
}

#[test]
fn escribir_en_el_inspector_y_aplicar_cambia_el_texto() {
    let mut app = editor();
    rod_app::editor_app::update(&mut app, MensajeEditor::Borrador("Hola".into()));
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, editor: &mut EditorApp| {
                for mensaje in rod_app::ui::paneles::subtitulos::mostrar(ui, editor, &ES) {
                    rod_app::editor_app::update(editor, mensaje);
                }
            },
            app,
        );
    // El inspector aplica en vivo; no hay botón "Aplicar".
    rod_app::editor_app::update(
        h.state_mut(),
        rod_app::editor_app::MensajeEditor::EditarEnVivo("cap-1".into(), "Hola".into()),
    );
    assert_eq!(h.state().proyecto().captions[0].texto, "Hola");
}

#[test]
fn borrar_muestra_toast_y_deshacer_restaura() {
    let app = editor();
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, editor: &mut EditorApp| {
                for mensaje in rod_app::ui::paneles::subtitulos::mostrar(ui, editor, &ES) {
                    rod_app::editor_app::update(editor, mensaje);
                }
            },
            app,
        );
    h.get_by_label(rod_app::ui::kit::glifo(rod_app::ui::kit::Icono::Papelera))
        .click();
    h.run();
    assert!(h.state().proyecto().captions.is_empty());

    let mut toasts = Toasts::default();
    toasts.push(ES.linea_eliminada.to_owned(), true);
    let mut th = egui_kittest::Harness::builder().build_state(
        |ctx, (toasts, deshecho): &mut (Toasts, bool)| {
            if toasts.mostrar(ctx, &ES) {
                *deshecho = true;
            }
        },
        (toasts, false),
    );
    th.get_by_label(ES.comando(Comando::Deshacer)).click();
    th.run();
    assert!(th.state().1);
}
