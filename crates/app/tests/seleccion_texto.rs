use rod_app::editor_app::{EditorApp, EstadoTexto, MensajeEditor};
use rod_app::textos::ES;
use rod_app::ui::tema;

const VIDEO: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/clip-15s-2audio.mp4"
);

fn editor() -> EditorApp {
    let mut proyecto = rod_core::Proyecto::nuevo(VIDEO);
    for (id, inicio) in [("c1", 1.0), ("c2", 2.0), ("c3", 5.0)] {
        proyecto.captions.push(rod_core::Caption {
            id: id.into(),
            inicio,
            fin: inicio + 0.8,
            texto: id.into(),
            hablante_id: None,
        });
    }
    EditorApp::desde_proyecto_en(proyecto, None, 0.0, &std::env::temp_dir())
        .expect("abre el clip de prueba")
}

#[test]
fn clic_en_un_clip_de_la_timeline_lo_selecciona() {
    let app = editor();
    let vista = rod_app::timeline::Vista::nueva(0.02);
    let punto =
        rod_app::timeline::Punto::new(vista.x_de(1.2), rod_app::timeline::ALTO_REGLA + 10.0);
    let respuesta = rod_app::timeline::pulsar(app.proyecto(), vista, punto, &[], true);
    match respuesta {
        rod_app::timeline::Respuesta::Empezar { seleccion, .. } => {
            assert_eq!(seleccion, vec!["c1".to_owned()]);
        }
        otro => panic!("se esperaba Empezar, llegó {otro:?}"),
    }
}

#[test]
fn el_campo_se_vacia_cuando_el_playhead_pasa_la_linea() {
    let mut app = editor();
    app.editor_mut().seleccion.clear();
    let _ = rod_app::editor_app::update(&mut app, MensajeEditor::Step(1.2));
    rod_app::ui::texto::sincronizar_objetivo(&mut app);
    assert_eq!(app.texto.borrador, "c1");

    let _ = rod_app::editor_app::update(&mut app, MensajeEditor::Step(2.8));
    rod_app::ui::texto::sincronizar_objetivo(&mut app);
    assert!(app.texto.borrador.is_empty());
    assert!(app.texto.objetivo.is_none());
}

#[test]
fn el_playhead_autoselecciona_la_linea_bajo_el_cursor() {
    let mut app = editor();
    let _ = rod_app::editor_app::update(&mut app, MensajeEditor::Step(2.2));
    rod_app::editor_app::autoseleccionar_bajo_playhead(&mut app);
    assert_eq!(app.editor_mut().seleccion, vec!["c2".to_owned()]);
}

#[test]
fn tras_enter_el_campo_sigue_mostrando_el_texto_editado() {
    let mut app = editor();
    app.editor_mut().seleccionar_uno("c1");
    app.texto.objetivo = Some("c1".into());
    app.texto.original = "c1".into();
    app.texto.borrador = "c1 editado".into();
    app.texto.foco = true;
    EstadoTexto::cerrar_sesion(&mut app);
    rod_app::ui::texto::sincronizar_objetivo(&mut app);
    assert_eq!(app.proyecto().captions[0].texto, "c1 editado");
    assert_eq!(app.texto.borrador, "c1 editado");
}

#[test]
fn el_borde_del_campo_es_rayado_sin_linea_y_de_acento_con_foco() {
    assert_eq!(
        rod_app::ui::texto::color_borde_campo(false, false),
        tema::COLOR_SIN_HABLANTE
    );
    assert_eq!(
        rod_app::ui::texto::color_borde_campo(true, false),
        tema::BORDE
    );
    assert_eq!(
        rod_app::ui::texto::color_borde_campo(true, true),
        tema::ACENTO
    );
}

#[test]
fn clic_en_la_barra_selecciona_la_linea_que_edita() {
    let mut app = editor();
    let _ = rod_app::editor_app::update(&mut app, MensajeEditor::Step(1.2));
    app.editor_mut().seleccion.clear();
    let mensaje = rod_app::ui::texto::mensaje_al_enfocar(&app);
    assert!(matches!(
        mensaje,
        Some(MensajeEditor::Seleccionar(ref id)) if id == "c1"
    ));
    let _ = &ES;
}
