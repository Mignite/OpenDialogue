use std::cell::RefCell;
use std::rc::Rc;

use egui_kittest::kittest::Queryable;
use rod_app::editor_app::{CampoHablante, EditorApp, MensajeEditor};

fn editor() -> EditorApp {
    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let mut proyecto = rod_core::Proyecto::nuevo(video);
    proyecto.hablantes.push(rod_core::Hablante {
        id: "h0".into(),
        nombre: "Ana".into(),
        tecla: "1".into(),
        color: String::new(),
        preset_id: None,
    });
    EditorApp::desde_proyecto_en(proyecto, None, 0.0, &std::env::temp_dir())
        .expect("abre el clip de prueba")
}

#[test]
fn el_selector_solo_ofrece_los_doce_colores_y_guarda_el_elegido() {
    let mensajes = Rc::new(RefCell::new(Vec::new()));
    let recolector = Rc::clone(&mensajes);
    let mut harness = egui_kittest::Harness::new_ui_state(
        move |ui, editor: &mut EditorApp| {
            recolector
                .borrow_mut()
                .extend(rod_app::ui::paneles::hablantes::mostrar(
                    ui,
                    editor,
                    &rod_app::textos::ES,
                ));
        },
        editor(),
    );
    // El punto abre el popover; la muestra elegida está dentro.
    harness
        .get_by_label(rod_core::paleta_identidad()[0])
        .click();
    harness.run();
    harness
        .get_by_label(rod_core::paleta_identidad()[3])
        .click();
    harness.run();

    let elegidos: Vec<_> = mensajes
        .borrow()
        .iter()
        .filter_map(|mensaje| match mensaje {
            MensajeEditor::EditarHablante(_, CampoHablante::Color, hex) => Some(hex.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(elegidos, vec![rod_core::paleta_identidad()[3].to_owned()]);
}
