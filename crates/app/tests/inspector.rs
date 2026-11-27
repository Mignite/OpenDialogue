use egui_kittest::kittest::Queryable;
use rod_app::comandos::Comando;
use rod_app::editor_app::EditorApp;
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
    rod_app::editor_app::update(
        &mut app,
        rod_app::editor_app::MensajeEditor::Seleccionar("cap-1".into()),
    );
    app
}

fn campo(h: &mut egui_kittest::Harness<'_, EditorApp>) {
    h.state_mut().texto.foco = true;
    for _ in 0..3 {
        h.step();
    }
}

fn escribir(h: &mut egui_kittest::Harness<'_, EditorApp>, texto: &str) {
    h.input_mut()
        .events
        .push(egui::Event::Text(texto.to_owned()));
    h.step();
    for _ in 0..3 {
        h.step();
    }
}

#[test]
fn escribir_en_la_barra_y_confirmar_cambia_el_texto() {
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, editor: &mut EditorApp| {
                let _ = rod_app::ui::texto::mostrar(ui, editor, &ES);
            },
            editor(),
        );
    campo(&mut h);
    escribir(&mut h, " Hola");
    rod_app::editor_app::EstadoTexto::cerrar_sesion(h.state_mut());
    assert_eq!(
        h.state().proyecto().captions[0].texto,
        "texto original Hola"
    );
}

#[test]
fn borrar_por_comando_muestra_toast_y_deshacer_restaura() {
    let mut app = editor();
    let _ = rod_app::editor_app::update(
        &mut app,
        rod_app::editor_app::MensajeEditor::Comando(Comando::Eliminar),
    );
    assert!(app.proyecto().captions.is_empty());

    let mut toasts = Toasts::default();
    toasts.push(ES.linea_eliminada.to_owned(), true);
    let mut th = egui_kittest::Harness::builder().build_state(
        |ctx, (toasts, deshecho): &mut (Toasts, bool)| {
            if toasts.mostrar(ctx, &ES).is_some() {
                *deshecho = true;
            }
        },
        (toasts, false),
    );
    th.get_by_label(ES.comando(Comando::Deshacer)).click();
    th.run();
    assert!(th.state().1);
}

#[test]
fn shift_enter_deja_linea_y_enter_termina_la_edicion() {
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, editor: &mut EditorApp| {
                let _ = rod_app::ui::texto::mostrar(ui, editor, &ES);
            },
            editor(),
        );
    campo(&mut h);
    escribir(&mut h, " Hola");
    h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::Enter);
    for _ in 0..3 {
        h.step();
    }
    escribir(&mut h, "mundo");
    assert_eq!(
        h.state().proyecto().captions[0].texto,
        "texto original Hola\nmundo"
    );
    h.key_press(egui::Key::Enter);
    for _ in 0..3 {
        h.step();
    }
    assert!(!h.state().texto.foco);
}
