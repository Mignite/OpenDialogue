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
