use rod_app::comandos::Comando;
use rod_app::editor_app::{EditorApp, EstadoTexto, MensajeEditor};
use rod_app::timeline::Vista;
use std::time::{Duration, Instant};

const VIDEO: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/clip-15s-2audio.mp4"
);

fn editor() -> EditorApp {
    let mut proyecto = rod_core::Proyecto::nuevo(VIDEO);
    proyecto.captions.push(rod_core::Caption {
        id: "c1".into(),
        inicio: 1.0,
        fin: 2.0,
        texto: "".into(),
        hablante_id: None,
    });
    EditorApp::desde_proyecto_en(proyecto, None, 0.0, &std::env::temp_dir())
        .expect("abre el clip de prueba")
}

#[test]
fn el_slider_de_zoom_usa_la_escala_real_de_la_vista() {
    assert!((rod_app::ui::timeline::zoom_desde_slider(0.0) - Vista::ESCALA_MINIMA).abs() < 1e-9);
    assert!((rod_app::ui::timeline::zoom_desde_slider(100.0) - Vista::ESCALA_MAXIMA).abs() < 1e-9);
    let escala = rod_app::ui::timeline::zoom_desde_slider(50.0);
    assert!(escala > Vista::ESCALA_MINIMA && escala < Vista::ESCALA_MAXIMA);
    assert!((rod_app::ui::timeline::slider_desde_zoom(escala) - 50.0).abs() < 0.01);
}

#[test]
fn la_sesion_se_cierra_solo_tras_un_segundo_sin_teclear() {
    let ahora = Instant::now();
    assert!(!rod_app::ui::texto::sesion_vencida(
        Some(ahora),
        ahora + Duration::from_millis(900)
    ));
    assert!(rod_app::ui::texto::sesion_vencida(
        Some(ahora),
        ahora + Duration::from_millis(1100)
    ));
}

#[test]
fn editar_texto_por_comando_enfoca_la_barra() {
    let mut app = editor();
    let _ = rod_app::editor_app::update(&mut app, MensajeEditor::Comando(Comando::EditarTexto));
    assert!(app.texto.foco);
}

#[test]
fn esc_con_texto_vacio_no_pierde_la_primera_letra() {
    let mut app = editor();
    app.editor_mut().seleccionar_uno("c1");
    app.texto.foco = true;
    app.texto.original = "".into();
    app.texto.borrador = "a".into();
    EstadoTexto::revertir(&mut app);
    assert_eq!(app.proyecto().captions[0].texto, "");
    assert_eq!(app.texto.borrador, "");
}

#[test]
fn marquesina_con_shift_suma_a_la_seleccion_previa() {
    let mut app = editor();
    app.editor_mut().seleccionar_uno("c1");
    app.set_modificadores(egui::Modifiers::SHIFT);
    let ids = vec!["c1".to_owned(), "c2".to_owned()];
    EstadoTexto::sumar_seleccion(&mut app, ids);
    assert!(app.editor_mut().seleccion.contains(&"c1".to_owned()));
    assert!(app.editor_mut().seleccion.contains(&"c2".to_owned()));
}

#[test]
fn ctrl_y_shift_en_lista_actualizan_la_seleccion() {
    let mut app = editor();
    app.editor_mut().seleccionar_uno("c1");
    let _ = rod_app::editor_app::update(
        &mut app,
        MensajeEditor::SeleccionarConModificador {
            id: "c1".into(),
            ctrl: true,
            shift: false,
        },
    );
    assert!(app.editor_mut().seleccion.is_empty());
    let _ = rod_app::editor_app::update(
        &mut app,
        MensajeEditor::SeleccionarConModificador {
            id: "c1".into(),
            ctrl: false,
            shift: true,
        },
    );
    assert!(app.editor_mut().seleccion.contains(&"c1".to_owned()));
}

#[test]
fn el_alto_base_de_la_timeline_no_se_recalcula_con_el_campo_abierto() {
    use rod_app::ui::editor::{MedidasEditor, alto_timeline};
    let mut app = editor();
    app.texto.lineas = 3;
    let medidas = MedidasEditor::default();
    let reducido = alto_timeline(&app, &medidas, 0.14, 1440.0);
    assert!(reducido < medidas.alto_timeline);
    app.texto.lineas = 1;
    let normal = alto_timeline(&app, &medidas, 0.14, 1440.0);
    assert!((normal - medidas.alto_timeline).abs() < 0.01);
}
