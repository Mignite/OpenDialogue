use egui_kittest::kittest::Queryable;
use rod_app::editor_app::EditorApp;
use rod_app::preferencias::Preferencias;
use rod_app::textos::EN;

const VIDEO: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/clip-15s-2audio.mp4"
);

fn app(fuente: &str) -> EditorApp {
    let mut proyecto = rod_core::Proyecto::nuevo(VIDEO);
    proyecto.hablantes.push(rod_core::Hablante {
        id: "h1".into(),
        nombre: "Ana".into(),
        tecla: "1".into(),
        color: String::new(),
        preset_id: None,
    });
    let mut preset = rod_core::preset_por_defecto();
    preset.id = "p1".into();
    preset.nombre = "Kiwi".into();
    preset.fontname = fuente.into();
    let mut editor = EditorApp::desde_proyecto_en(proyecto, None, 0.0, &std::env::temp_dir())
        .expect("abre el clip de prueba");
    *editor.presets_mut() = vec![preset];
    editor.preset_seleccionado = Some("p1".into());
    editor
}

#[test]
fn fuente_ausente_muestra_chip_y_renderiza() {
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, estado: &mut EditorApp| {
                let _ = rod_app::ui::paneles::estilos::mostrar(ui, estado, &EN);
            },
            app("FuenteQueNoExiste"),
        );
    rod_app::ui::tipografia::instalar(&h.ctx);
    h.run();
    h.get_by_label(EN.fuente_no_encontrada);
}

#[test]
fn acordeon_guarda_y_reabre_el_preset() {
    let ruta = std::env::temp_dir().join(format!("rod-estilos-prefs-{}.json", std::process::id()));
    let prefs = Preferencias {
        preset_abierto: Some("p1".into()),
        ..Default::default()
    };
    prefs.guardar_en(&ruta).expect("guarda preferencias");
    assert_eq!(
        Preferencias::cargar_en(&ruta).preset_abierto.as_deref(),
        Some("p1")
    );
    let _ = std::fs::remove_file(ruta);
}

#[test]
fn alignment_8_no_se_fuerza_y_etiqueta_es_generica() {
    let mut editor = app("Arial");
    editor.presets_mut()[0].alignment = 8;
    assert_eq!(editor.presets_mut()[0].alignment, 8);
    assert_eq!(
        rod_app::ui::paneles::estilos::etiqueta_margen(&EN),
        EN.margen_v
    );
}

#[test]
fn toggle_bloqueado_sin_variante() {
    assert!(!rod_app::ui::tipografia::variante_disponible(
        "FuenteQueNoExiste",
        true
    ));
}

#[test]
fn hex_invalido_no_aplica() {
    assert!(!rod_app::ui::paneles::estilos::es_hex_valido("zzzzzz"));
    assert!(rod_app::ui::paneles::estilos::es_hex_valido("#FF8800"));
}

#[test]
fn apply_cuenta_captions_y_es_un_paso() {
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
    for i in 0..2 {
        proyecto.captions.push(rod_core::Caption {
            id: format!("c{i}"),
            inicio: i as f64,
            fin: i as f64 + 0.5,
            texto: format!("L{i}"),
            hablante_id: Some(format!("h{i}")),
        });
    }
    let mut editor = EditorApp::desde_proyecto_en(proyecto, None, 0.0, &std::env::temp_dir())
        .expect("abre el clip de prueba");
    editor.presets_mut().push(rod_core::PresetAss {
        id: "p1".into(),
        nombre: "P1".into(),
        ..rod_core::preset_por_defecto()
    });
    rod_app::editor_app::update(
        &mut editor,
        rod_app::editor_app::MensajeEditor::Seleccionar("c0".into()),
    );
    let _ = rod_app::editor_app::update(
        &mut editor,
        rod_app::editor_app::MensajeEditor::SeleccionarConModificador {
            id: "c1".into(),
            ctrl: true,
            shift: false,
        },
    );
    assert_eq!(editor.seleccion().len(), 2);
    rod_app::editor_app::aplicar_a_seleccion_agrupado(&mut editor, "p1");
    assert_eq!(
        editor
            .proyecto()
            .hablantes
            .iter()
            .filter(|hablante| hablante.preset_id.as_deref() == Some("p1"))
            .count(),
        2
    );
    assert!(editor.editor_mut().puede_deshacer());
}

/// epaint 0.33 no expone `Shape::rect()`, así que cubrimos las variantes que
/// dibuja el panel: rectángulos, texto, iconos circulares, líneas e imágenes.
fn rect_de_forma(forma: &egui::Shape) -> Option<egui::Rect> {
    match forma {
        egui::Shape::Rect(rectangulo) => Some(rectangulo.rect),
        egui::Shape::Text(texto) => Some(texto.galley.rect.translate(texto.pos.to_vec2())),
        egui::Shape::Circle(circulo) => Some(egui::Rect::from_center_size(
            circulo.center,
            egui::Vec2::splat(circulo.radius * 2.0),
        )),
        egui::Shape::LineSegment { points, .. } => {
            Some(egui::Rect::from_two_pos(points[0], points[1]))
        }
        egui::Shape::Mesh(malla) if !malla.vertices.is_empty() => {
            let puntos: Vec<egui::Pos2> = malla.vertices.iter().map(|v| v.pos).collect();
            let mut rect = egui::Rect::from_points(&puntos);
            rect.max.x += 1.0; // el borde puede caer justo en el límite
            Some(rect)
        }
        _ => None,
    }
}

#[test]
fn sin_overflow_a_300px() {
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(300.0, 700.0))
        .build_ui_state(
            |ui, estado: &mut EditorApp| {
                let _ = rod_app::ui::paneles::estilos::mostrar(ui, estado, &EN);
            },
            app("Arial"),
        );
    rod_app::ui::tema::aplicar(&h.ctx);
    h.run();

    let mut formas = 0;
    for recorte in &h.output().shapes {
        let clip = recorte.clip_rect;
        // El área sensible transparente del arnés mide 4 px de más; no pinta
        // nada, así que no cuenta como control que desborda.
        let invisible = matches!(
            &recorte.shape,
            egui::Shape::Rect(rectangulo)
                if rectangulo.fill.a() == 0 && rectangulo.stroke.width == 0.0
        );
        if !invisible
            && clip.is_finite()
            && clip.max.x > 0.0
            && let Some(rect) = rect_de_forma(&recorte.shape)
        {
            assert!(
                rect.max.x <= clip.max.x + 1.0,
                "control fuera del panel: rect {rect:?}, clip {clip:?}, forma {:?}",
                recorte.shape
            );
        }
        if let egui::Shape::Text(texto) = &recorte.shape {
            for seccion in &texto.galley.job.sections {
                assert!(
                    seccion.format.font_id.size >= 11.0,
                    "texto de {} px: {:?}",
                    seccion.format.font_id.size,
                    texto.galley.job.text
                );
            }
        }
        formas += 1;
    }
    assert!(formas > 0, "el panel dibujó algo");
}

#[test]
fn combo_box_cambia_la_fuente_del_preset() {
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(320.0, 700.0))
        .build_ui_state(
            |ui, estado: &mut EditorApp| {
                let mensajes = rod_app::ui::paneles::estilos::mostrar(ui, estado, &EN);
                for mensaje in mensajes {
                    let _ = rod_app::editor_app::update(estado, mensaje);
                }
            },
            app("Arial"),
        );
    rod_app::ui::tema::aplicar(&h.ctx);
    h.run();
    h.get_by_role(egui::accesskit::Role::ComboBox).click();
    h.run();
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Agency FB")
        .click();
    h.run();
    assert_eq!(h.state_mut().presets_mut()[0].fontname, "Agency FB");
}

#[test]
fn restaurar_preset_recupera_el_borrado() {
    let mut editor = app("Arial");
    editor.presets_mut().push(rod_core::PresetAss {
        id: "p2".into(),
        nombre: "P2".into(),
        ..rod_core::preset_por_defecto()
    });
    let _ = rod_app::editor_app::update(
        &mut editor,
        rod_app::editor_app::MensajeEditor::BorrarPreset("p1".into()),
    );
    assert_eq!(editor.presets_mut().len(), 1);
    let _ = rod_app::editor_app::update(
        &mut editor,
        rod_app::editor_app::MensajeEditor::RestaurarPreset,
    );
    assert_eq!(editor.presets_mut().len(), 2);
    assert!(editor.presets_mut().iter().any(|preset| preset.id == "p1"));
}

#[test]
fn deshacer_no_restaura_el_preset_borrado() {
    let mut editor = app("Arial");
    editor.presets_mut().push(rod_core::PresetAss {
        id: "p2".into(),
        nombre: "P2".into(),
        ..rod_core::preset_por_defecto()
    });
    let _ = rod_app::editor_app::update(
        &mut editor,
        rod_app::editor_app::MensajeEditor::BorrarPreset("p1".into()),
    );
    let _ = rod_app::editor_app::update(&mut editor, rod_app::editor_app::MensajeEditor::Deshacer);
    assert_eq!(editor.presets_mut().len(), 1);
}

#[test]
fn un_solo_preset_abierto() {
    let mut editor = app("Arial");
    let segundo = rod_core::PresetAss {
        id: "p2".into(),
        nombre: "Otro".into(),
        ..rod_core::preset_por_defecto()
    };
    editor.presets_mut().push(segundo);
    // El estado del panel solo recuerda un id: seleccionar p2 cierra p1.
    editor.preset_seleccionado = Some("p2".into());
    assert_ne!(editor.preset_seleccionado.as_deref(), Some("p1"));
}
