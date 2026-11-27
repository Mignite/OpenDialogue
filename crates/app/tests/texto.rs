use rod_app::editor_app::{EditorApp, MensajeEditor};
use rod_app::textos::ES;
use rod_app::ui::tema;

const VIDEO: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/clip-15s-2audio.mp4"
);

fn editor() -> EditorApp {
    let mut proyecto = rod_core::Proyecto::nuevo(VIDEO);
    proyecto.captions.push(rod_core::Caption {
        id: "cap-1".into(),
        inicio: 1.0,
        fin: 2.0,
        texto: "uno".into(),
        hablante_id: None,
    });
    proyecto.captions.push(rod_core::Caption {
        id: "cap-2".into(),
        inicio: 3.0,
        fin: 4.0,
        texto: "dos".into(),
        hablante_id: None,
    });
    EditorApp::desde_proyecto_en(proyecto, None, 0.0, &std::env::temp_dir())
        .expect("abre el clip de prueba")
}

#[test]
fn objetivo_es_seleccion_o_playhead() {
    let mut app = editor();
    rod_app::editor_app::update(&mut app, MensajeEditor::Seleccionar("cap-2".into()));
    assert_eq!(
        rod_app::editor_app::EstadoTexto::objetivo(&app).as_deref(),
        Some("cap-2")
    );

    app.editor_mut().seleccion.clear();
    rod_app::editor_app::update(&mut app, MensajeEditor::Step(3.5));
    assert_eq!(
        rod_app::editor_app::EstadoTexto::objetivo(&app).as_deref(),
        Some("cap-2")
    );

    rod_app::editor_app::update(&mut app, MensajeEditor::Step(8.0));
    assert_eq!(rod_app::editor_app::EstadoTexto::objetivo(&app), None);
}

#[test]
fn cerrar_sesion_por_inactividad_es_un_solo_paso() {
    let mut app = editor();
    rod_app::editor_app::update(&mut app, MensajeEditor::Seleccionar("cap-1".into()));
    app.texto.foco = true;
    app.texto.original = "uno".into();
    app.texto.borrador = "uno".into();
    for letra in "abcde".chars() {
        app.texto.borrador.push(letra);
        let borrador = app.texto.borrador.clone();
        rod_app::editor_app::update(
            &mut app,
            MensajeEditor::EditarEnVivo("cap-1".into(), borrador),
        );
        app.texto.ultimo_tipeo = Some(std::time::Instant::now());
    }
    app.texto.ultimo_tipeo =
        Some(std::time::Instant::now() - std::time::Duration::from_millis(1100));
    rod_app::editor_app::EstadoTexto::cerrar_sesion(&mut app);
    assert!(!app.texto.foco);
    assert_eq!(app.proyecto().captions[0].texto, "unoabcde");
    assert!(app.editor_mut().puede_deshacer());
    assert!(app.editor_mut().deshacer());
    assert_eq!(app.proyecto().captions[0].texto, "uno");
    assert!(
        !app.editor_mut().deshacer(),
        "el tipeo debía ser un solo paso de historial"
    );
}

#[test]
fn enter_crea_linea_de_dos_segundos_sin_objetivo() {
    let mut app = editor();
    rod_app::editor_app::update(&mut app, MensajeEditor::Step(8.0));
    app.texto.borrador = "hola".into();
    app.texto.nueva_linea = true;
    rod_app::editor_app::EstadoTexto::cerrar_sesion(&mut app);
    let nueva = app
        .proyecto()
        .captions
        .iter()
        .find(|caption| caption.texto == "hola")
        .expect("creó la línea");
    assert!((nueva.inicio - 8.0).abs() < 0.001);
    assert!((nueva.fin - 10.0).abs() < 0.001);
}

#[test]
fn esc_revierte_y_devuelve_foco_a_timeline() {
    let mut app = editor();
    rod_app::editor_app::update(&mut app, MensajeEditor::Seleccionar("cap-1".into()));
    app.texto.foco = true;
    app.texto.original = "uno".into();
    app.texto.borrador = "cambiado".into();
    rod_app::editor_app::update(
        &mut app,
        MensajeEditor::EditarEnVivo("cap-1".into(), "cambiado".into()),
    );
    rod_app::editor_app::EstadoTexto::revertir(&mut app);
    assert!(!app.texto.foco);
    assert_eq!(app.texto.borrador, "uno");
    assert_eq!(app.proyecto().captions[0].texto, "uno");
}

#[test]
fn campo_crece_a_tres_lineas_y_empuja_timeline() {
    let mut app = editor();
    app.texto.lineas = 3;
    let medidas = rod_app::ui::editor::MedidasEditor::default();
    let alto = rod_app::ui::editor::alto_timeline(&app, &medidas, tema::ANIMACION, 1440.0);
    assert_eq!(app.texto.lineas, 3);
    assert!(
        alto <= medidas.alto_timeline - 2.0 * tema::ALTO_BARRA_TEXTO + 0.001,
        "alto={alto}, base={}",
        medidas.alto_timeline
    );
    let _ = &ES;
}
