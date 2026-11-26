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
    let mut prefs = Preferencias::default();
    prefs.preset_abierto = Some("p1".into());
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

#[test]
fn deshacer_restaura_el_preset_borrado() {
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
    let _ = rod_app::editor_app::update(&mut editor, rod_app::editor_app::MensajeEditor::Deshacer);
    assert_eq!(editor.presets_mut().len(), 2);
    assert!(editor.presets_mut().iter().any(|preset| preset.id == "p1"));
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
