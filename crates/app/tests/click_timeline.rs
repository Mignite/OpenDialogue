use rod_app::editor_app::EditorApp;
use rod_app::textos::ES;

const VIDEO: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/clip-15s-2audio.mp4"
);

fn app() -> EditorApp {
    let mut proyecto = rod_core::Proyecto::nuevo(VIDEO);
    for i in 0..2 {
        proyecto.hablantes.push(rod_core::Hablante {
            id: format!("h{i}"),
            nombre: format!("H{i}"),
            tecla: (i + 1).to_string(),
            color: String::new(),
            preset_id: None,
        });
    }
    for n in 0..6 {
        proyecto.captions.push(rod_core::Caption {
            id: format!("c{n}"),
            inicio: n as f64 * 2.0,
            fin: n as f64 * 2.0 + 0.8,
            texto: format!("L{n}"),
            hablante_id: Some(format!("h{}", n % 2)),
        });
    }
    EditorApp::desde_proyecto_en(proyecto, None, 0.0, &std::env::temp_dir()).unwrap()
}

fn clic_sin_arrastre(x: f32, y: f32) -> (f64, Vec<String>, bool) {
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1000.0, 700.0))
        .build_ui_state(
            |ui, app: &mut EditorApp| {
                let msgs = rod_app::ui::timeline::mostrar(ui, app, &ES);
                for m in msgs {
                    rod_app::editor_app::update(app, m);
                }
            },
            app(),
        );
    h.run();
    h.input_mut()
        .events
        .push(egui::Event::PointerMoved(egui::pos2(x, y)));
    h.step();
    h.input_mut().events.push(egui::Event::PointerButton {
        pos: egui::pos2(x, y),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    h.step();
    h.input_mut().events.push(egui::Event::PointerButton {
        pos: egui::pos2(x, y),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::default(),
    });
    h.step();
    (
        h.state().tiempo_ui(),
        h.state().seleccion().to_vec(),
        h.state().frame_pendiente,
    )
}

#[test]
fn clic_sin_arrastre_busca_el_tiempo_bajo_el_cursor() {
    let (tiempo, _, pendiente) = clic_sin_arrastre(400.0, 110.0);
    assert!(tiempo > 1.0, "el clic no movió el playhead: {tiempo}");
    assert!(pendiente, "el seek en pausa debe pedir otro frame");
}

#[test]
fn clic_sobre_un_clip_lo_selecciona() {
    let (_, seleccion, _) = clic_sin_arrastre(125.0, 110.0);
    assert_eq!(seleccion, vec!["c0".to_owned()]);
}
