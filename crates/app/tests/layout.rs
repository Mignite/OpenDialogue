#[test]
fn rueda_simple_baja_las_pistas_y_shift_hace_pan() {
    use rod_app::ui::timeline::{Rueda, gesto_rueda};

    // winit entrega delta.y = -40 al girar hacia abajo: hay que avanzar filas.
    assert_eq!(
        gesto_rueda(egui::Modifiers::NONE, egui::vec2(0.0, -40.0)),
        Rueda::Pistas(1.0)
    );
    // Con Shift, egui pone la muesca en .x y deja .y a 0.
    assert_eq!(
        gesto_rueda(egui::Modifiers::SHIFT, egui::vec2(40.0, 0.0)),
        Rueda::Pan(1.0)
    );
    assert_eq!(
        gesto_rueda(egui::Modifiers::CTRL, egui::vec2(0.0, -40.0)),
        Rueda::Zoom(-1.0)
    );
}

#[test]
fn scroll_se_ajusta_al_quitar_hablantes() {
    use rod_app::ui::timeline::scroll_visible;

    // Tres filas desplazadas con 12 hablantes; al quedar 4 no queda recorrido.
    assert_eq!(scroll_visible(3.0, 13, 700.0), 1.0);
    assert_eq!(scroll_visible(3.0, 5, 700.0), 0.0);
}

#[test]
fn sin_scrollbar_cuando_todas_caben() {
    use rod_app::ui::timeline::{max_scroll_pistas, scroll_desde_barra};

    assert_eq!(max_scroll_pistas(4, 700.0), 0.0);
    assert_eq!(scroll_desde_barra(350.0, 700.0, 4), 0.0);
}

#[test]
fn scrollbar_mapa_el_tamanio_de_la_barra() {
    use rod_app::ui::timeline::scroll_desde_barra;

    assert_eq!(scroll_desde_barra(0.0, 700.0, 15), 0.0);
    assert_eq!(scroll_desde_barra(700.0, 700.0, 15), 3.0);
}

#[test]
fn limites_de_scroll_por_pistas() {
    use rod_app::ui::timeline::{desplazar, filas_visibles, max_scroll_pistas};

    assert_eq!(filas_visibles(700.0), 12);
    assert_eq!(max_scroll_pistas(15, 700.0), 3.0);
    assert_eq!(max_scroll_pistas(4, 700.0), 0.0);
    assert_eq!(desplazar(2.0, 5.0, 15, 700.0), 3.0);
    assert_eq!(desplazar(0.0, -1.0, 15, 700.0), 0.0);
}

#[test]
fn clic_con_scroll_selecciona_el_carril_visible() {
    use rod_app::timeline::{ALTO_CARRIL, ALTO_REGLA};
    use rod_app::ui::timeline::punto;

    let p = punto(egui::vec2(300.0, ALTO_REGLA + 20.0), 2.0);
    assert_eq!(p.y, ALTO_REGLA + 2.0 * ALTO_CARRIL + 20.0);
}

#[test]
fn regla_sigue_busca_tiempo_con_scroll() {
    use rod_app::timeline::ALTO_REGLA;
    use rod_app::ui::timeline::punto;

    let p = punto(egui::vec2(300.0, 8.0), 2.0);
    assert!(p.y < ALTO_REGLA);
}

#[test]
fn alto_extra_no_guarda_altura_hinchada() {
    use rod_app::editor_app::EditorApp;
    use rod_app::ui::editor::MedidasEditor;
    use rod_app::video::Gpu;
    use std::sync::{Arc, Mutex};

    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let mut editor = EditorApp::desde_video(video).expect("abre el clip de prueba");
    editor.texto.lineas = 3;
    let estado = (
        editor,
        MedidasEditor {
            alto_timeline: 300.0,
            ..Default::default()
        },
        Arc::new(Mutex::new(None)),
    );
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1200.0, 700.0))
        .build_state(
            |ctx, (e, m, gpu): &mut (EditorApp, MedidasEditor, Arc<Mutex<Option<Gpu>>>)| {
                let _ = rod_app::ui::editor::show(
                    ctx,
                    e,
                    wgpu::TextureFormat::Rgba8Unorm,
                    gpu.clone(),
                    &rod_app::textos::EN,
                    &rod_app::comandos::MapaAtajos::por_defecto(),
                    m,
                );
            },
            estado,
        );
    h.run();
    assert_eq!(h.state().1.alto_timeline, 300.0);
}

#[test]
fn arrastrar_el_divisor_encoge_el_timeline() {
    use rod_app::editor_app::EditorApp;
    use rod_app::ui::editor::MedidasEditor;
    use rod_app::video::Gpu;
    use std::sync::{Arc, Mutex};

    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let editor = EditorApp::desde_video(video).expect("abre el clip de prueba");
    let estado = (
        editor,
        MedidasEditor {
            alto_timeline: 490.0,
            ..Default::default()
        },
        Arc::new(Mutex::new(None)),
    );
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1200.0, 700.0))
        .build_state(
            |ctx, (e, m, gpu): &mut (EditorApp, MedidasEditor, Arc<Mutex<Option<Gpu>>>)| {
                let _ = rod_app::ui::editor::show(
                    ctx,
                    e,
                    wgpu::TextureFormat::Rgba8Unorm,
                    gpu.clone(),
                    &rod_app::textos::EN,
                    &rod_app::comandos::MapaAtajos::por_defecto(),
                    m,
                );
            },
            estado,
        );
    h.run();
    // El borde del panel cae en 219 (210 del alto más 9 px de margen del frame).
    // El borde del panel inferior depende de la altura real del panel, que
    // ahora descuenta la barra de texto. Se calcula, no se fija a ojo.
    let alto_abajo = rod_app::ui::editor::alto_timeline(&h.state().0, &h.state().1, 0.0, 1200.0);
    let borde_y = 700.0 - alto_abajo + 9.0;
    h.input_mut()
        .events
        .push(egui::Event::PointerMoved(egui::pos2(600.0, borde_y)));
    h.step();
    h.input_mut().events.push(egui::Event::PointerButton {
        pos: egui::pos2(600.0, borde_y),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    h.step();
    h.input_mut()
        .events
        .push(egui::Event::PointerMoved(egui::pos2(600.0, 500.0)));
    h.step();
    h.input_mut().events.push(egui::Event::PointerButton {
        pos: egui::pos2(600.0, 500.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.step();
    assert!(
        h.state().1.alto_timeline < 300.0,
        "el divisor no encogió: {}",
        h.state().1.alto_timeline
    );
}

#[test]
fn buscar_fuente_no_cierra_el_desplegable() {
    use egui_kittest::kittest::Queryable;
    use rod_app::editor_app::EditorApp;
    use rod_app::textos::EN;

    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let editor = EditorApp::desde_video(video).expect("abre el clip de prueba");
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(320.0, 700.0))
        .build_ui_state(
            |ui, estado: &mut EditorApp| {
                let _ = rod_app::ui::paneles::estilos::mostrar(ui, estado, &EN);
            },
            editor,
        );
    rod_app::ui::tema::aplicar(&h.ctx);
    h.run();
    h.get_by_role(egui::accesskit::Role::ComboBox).click();
    h.run();
    // El campo de búsqueda del popup es el único TextInput vacío.
    let busqueda = h
        .get_all_by_role(egui::accesskit::Role::TextInput)
        .find(|nodo| nodo.value().unwrap_or_default().is_empty())
        .expect("campo de búsqueda visible");
    busqueda.click();
    busqueda.type_text("Ari");
    h.run();
    assert!(
        h.query_by_role_and_label(egui::accesskit::Role::Button, "Arial")
            .is_some(),
        "el popup se cerró al buscar"
    );
}

#[test]
fn speakers_no_desborda_a_300px() {
    use rod_app::editor_app::EditorApp;
    use rod_app::textos::EN;

    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let mut editor = EditorApp::desde_video(video).expect("abre el clip de prueba");
    editor
        .editor_mut()
        .proyecto_mut()
        .hablantes
        .push(rod_core::Hablante {
            id: "h1".into(),
            nombre: "Hablante 1".into(),
            tecla: "1".into(),
            color: String::new(),
            preset_id: None,
        });
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(300.0, 700.0))
        .build_ui_state(
            |ui, estado: &mut EditorApp| {
                let _ = rod_app::ui::paneles::hablantes::mostrar(ui, estado, &EN);
            },
            editor,
        );
    rod_app::ui::tema::aplicar(&h.ctx);
    h.run();
    for recorte in &h.output().shapes {
        let clip = recorte.clip_rect;
        // El área sensible transparente del arnés es más ancha; no pinta nada.
        if let egui::Shape::Rect(r) = &recorte.shape
            && r.fill.a() == 0
            && r.stroke.width == 0.0
        {
            continue;
        }
        let rect = match &recorte.shape {
            egui::Shape::Rect(r) => r.rect,
            egui::Shape::Text(t) => t.galley.rect.translate(t.pos.to_vec2()),
            _ => continue,
        };
        if clip.is_finite() && clip.max.x > 0.0 && rect.max.x > 0.0 {
            assert!(
                rect.max.x <= 301.0,
                "control fuera del panel: {rect:?} en {clip:?}"
            );
        }
    }
}

#[test]
fn borrar_caption_no_toca_la_seleccion() {
    use rod_app::editor_app::{EditorApp, MensajeEditor, update};

    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let mut editor = EditorApp::desde_video(video).expect("abre el clip de prueba");
    for id in ["c0", "c1"] {
        editor.editor_mut().agregar(rod_core::Caption {
            id: id.into(),
            inicio: 0.0,
            fin: 1.0,
            texto: "hola".into(),
            hablante_id: None,
        });
    }
    update(&mut editor, MensajeEditor::Seleccionar("c0".into()));
    let _ = update(&mut editor, MensajeEditor::BorrarCaption("c1".into()));
    let ids: Vec<_> = editor
        .proyecto()
        .captions
        .iter()
        .map(|caption| caption.id.as_str())
        .collect();
    assert_eq!(ids, vec!["c0"]);
    assert_eq!(editor.seleccion(), &["c0".to_owned()]);
    let _ = update(&mut editor, MensajeEditor::Deshacer);
    assert_eq!(editor.proyecto().captions.len(), 2);
}

#[test]
fn el_picker_de_color_aparece_con_un_solo_clic() {
    use egui_kittest::kittest::Queryable;
    use rod_app::editor_app::EditorApp;
    use rod_app::textos::EN;

    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let mut editor = EditorApp::desde_video(video).expect("abre el clip de prueba");
    let mut preset = rod_core::preset_por_defecto();
    preset.id = "p-color".into();
    preset.nombre = "Color".into();
    editor.presets_mut().push(preset);
    editor.preset_seleccionado = Some("p-color".into());
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(320.0, 700.0))
        .build_ui_state(
            |ui, estado: &mut EditorApp| {
                let mensajes = rod_app::ui::paneles::estilos::mostrar(ui, estado, &EN);
                for mensaje in mensajes {
                    let _ = rod_app::editor_app::update(estado, mensaje);
                }
            },
            editor,
        );
    rod_app::ui::tema::aplicar(&h.ctx);
    h.run();
    let swatch = h
        .get_all_by_role_and_label(egui::accesskit::Role::Button, "")
        .next()
        .expect("swatch de color");
    let antes = h
        .query_all_by_role(egui::accesskit::Role::SpinButton)
        .count();
    swatch.click();
    h.step();
    h.run();
    assert!(
        h.query_all_by_role(egui::accesskit::Role::SpinButton)
            .count()
            > antes + 2,
        "el picker completo no apareció tras un clic"
    );
}

#[test]
fn ensanchar_y_encoger_el_panel_lateral_no_vuelve_al_inicial() {
    use rod_app::editor_app::EditorApp;
    use rod_app::ui::editor::MedidasEditor;
    use rod_app::video::Gpu;
    use std::sync::{Arc, Mutex};

    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let editor = EditorApp::desde_video(video).expect("abre el clip de prueba");
    let estado = (editor, MedidasEditor::default(), Arc::new(Mutex::new(None)));
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1200.0, 700.0))
        .build_state(
            |ctx, (e, m, gpu): &mut (EditorApp, MedidasEditor, Arc<Mutex<Option<Gpu>>>)| {
                let _ = rod_app::ui::editor::show(
                    ctx,
                    e,
                    wgpu::TextureFormat::Rgba8Unorm,
                    gpu.clone(),
                    &rod_app::textos::EN,
                    &rod_app::comandos::MapaAtajos::por_defecto(),
                    m,
                );
            },
            estado,
        );
    h.run();
    let inicial = h.state().1.ancho_panel;
    // El borde del SidePanel está en 1200 - ancho, con unos px de margen.
    // El borde del SidePanel lleva el margen del frame (8 px).
    let borde = 1200.0 - inicial + 8.0;
    h.input_mut()
        .events
        .push(egui::Event::PointerMoved(egui::pos2(borde, 350.0)));
    h.step();
    h.input_mut().events.push(egui::Event::PointerButton {
        pos: egui::pos2(borde, 350.0),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    h.step();
    h.input_mut()
        .events
        .push(egui::Event::PointerMoved(egui::pos2(borde - 20.0, 350.0)));
    h.step();
    h.input_mut().events.push(egui::Event::PointerButton {
        pos: egui::pos2(borde - 20.0, 350.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.step();
    assert!(
        h.state().1.ancho_panel > inicial + 10.0,
        "el panel volvió a {:.1} desde {inicial:.1}",
        h.state().1.ancho_panel
    );
}

#[test]
fn enfocar_el_texto_no_mueve_el_playhead() {
    use rod_app::editor_app::{EditorApp, MensajeEditor, update};

    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let mut editor = EditorApp::desde_video(video).expect("abre el clip de prueba");
    editor.editor_mut().agregar(rod_core::Caption {
        id: "cap-texto".into(),
        inicio: 1.0,
        fin: 2.0,
        texto: "hola".into(),
        hablante_id: None,
    });
    assert!((editor.tiempo_ui() - 0.0).abs() < 0.5);
    let _ = update(
        &mut editor,
        MensajeEditor::SeleccionarSinSeek("cap-texto".into()),
    );
    assert_eq!(editor.seleccion(), &["cap-texto".to_owned()]);
    assert!(
        (editor.tiempo_ui() - 0.0).abs() < 0.5,
        "el playhead se movió a {}",
        editor.tiempo_ui()
    );
}

#[test]
fn tope_respeta_la_ventana_y_el_suelo() {
    assert_eq!(rod_app::ui::editor::alto_maximo_timeline(900.0), 630.0);
    assert_eq!(rod_app::ui::editor::alto_maximo_timeline(200.0), 160.0);
    let prefs = rod_app::preferencias::Preferencias {
        alto_timeline: 50_000.0,
        ..Default::default()
    };
    let medidas = rod_app::ui::editor::MedidasEditor::from(&prefs);
    assert_eq!(medidas.alto_timeline, 50_000.0);
}
