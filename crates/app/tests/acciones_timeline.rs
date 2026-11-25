use rod_app::comandos::Comando;
use rod_app::editor_app::{EditorApp, MensajeEditor};
use rod_app::timeline::{self, Vista};

const VIDEO: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/clip-15s-2audio.mp4"
);

fn editor() -> EditorApp {
    let mut proyecto = rod_core::Proyecto::nuevo(VIDEO);
    for (id, inicio) in [("c1", 1.0), ("c2", 3.0), ("c3", 5.0)] {
        proyecto.captions.push(rod_core::Caption {
            id: id.into(),
            inicio,
            fin: inicio + 1.0,
            texto: id.into(),
            hablante_id: None,
        });
    }
    EditorApp::desde_proyecto_en(proyecto, None, 0.0, &std::env::temp_dir())
        .expect("abre el clip de prueba")
}

#[test]
fn borrar_multiples_y_deshacer_restaura_el_grupo() {
    let mut app = editor();
    app.editor_mut().seleccionar_uno("c1");
    app.editor_mut().alternar_seleccion("c2");
    let _ = rod_app::editor_app::update(&mut app, MensajeEditor::Comando(Comando::Eliminar));
    assert_eq!(app.proyecto().captions.len(), 1);
    let _ = rod_app::editor_app::update(&mut app, MensajeEditor::Deshacer);
    assert_eq!(app.proyecto().captions.len(), 3);
    assert_eq!(app.proyecto().captions[0].id, "c1");
}

#[test]
fn entradas_y_salidas_se_fijan_en_el_playhead() {
    let mut app = editor();
    let _ = rod_app::editor_app::update(&mut app, MensajeEditor::Step(3.5));
    app.editor_mut().seleccionar_uno("c2");
    let _ = rod_app::editor_app::update(&mut app, MensajeEditor::Comando(Comando::Entrada));
    let _ = rod_app::editor_app::update(&mut app, MensajeEditor::Step(1.0));
    let _ = rod_app::editor_app::update(&mut app, MensajeEditor::Comando(Comando::Salida));
    let caption = app
        .proyecto()
        .captions
        .iter()
        .find(|caption| caption.id == "c2")
        .unwrap();
    assert!((caption.inicio - 3.5).abs() < 0.001);
    assert!((caption.fin - 4.5).abs() < 0.001);
}

#[test]
fn marquesina_y_lista_comparten_seleccion() {
    let app = editor();
    let vista = Vista::nueva(0.02);
    let punto = timeline::Punto {
        x: vista.x_de(2.5),
        y: timeline::ALTO_REGLA + 10.0,
    };
    let respuesta = timeline::pulsar(app.proyecto(), vista, punto, &[]);
    let arrastre = match respuesta {
        timeline::Respuesta::Empezar { arrastre, .. } => arrastre,
        _ => panic!("se esperaba marquesina"),
    };
    let marco = timeline::Marco::nuevo(
        vista,
        timeline::Punto {
            x: vista.x_de(6.5),
            y: timeline::ALTO_REGLA + 10.0,
        },
        app.proyecto().captions.len() as f64 + 10.0,
        3.5,
    );
    let ids = arrastre.ids_de_marquesina(app.proyecto(), marco);
    assert!(ids.contains(&"c2".to_owned()));
    assert!(ids.contains(&"c3".to_owned()));
}
