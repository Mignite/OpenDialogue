use rod_app::ui::estilos_render::{componer, escala_de_preview, render_subtitle};

fn preset(nombre_fuente: &str) -> rod_core::PresetAss {
    let mut preset = rod_core::preset_por_defecto();
    preset.id = "p-preview".into();
    preset.fontname = nombre_fuente.into();
    preset.fontsize = 40.0;
    preset
}

#[test]
fn preview_y_overlay_son_byte_identicos() {
    let preset = preset("Arial");
    let texto = "Hola mundo";
    let preview = render_subtitle(&preset, texto, 1.0);

    let caption = rod_core::Caption {
        id: "cap-preview".into(),
        inicio: 0.0,
        fin: 1.0,
        texto: texto.into(),
        hablante_id: Some("h-preview".into()),
    };
    let hablante = rod_core::Hablante {
        id: "h-preview".into(),
        nombre: "Preview".into(),
        tecla: "1".into(),
        color: String::new(),
        preset_id: Some(preset.id.clone()),
    };
    let ass = rod_core::build_ass(
        &[caption],
        &[hablante],
        |_| preset.clone(),
        rod_app::preview::CANVAS.0,
        rod_app::preview::CANVAS.1,
    );
    let mut subs =
        rod_subs::Subtitulos::nuevo(rod_app::preview::CANVAS.0, rod_app::preview::CANVAS.1, &ass);
    let overlay = componer(
        rod_app::preview::CANVAS.0,
        rod_app::preview::CANVAS.1,
        subs.render(100),
    );

    assert_eq!(preview.as_raw(), overlay.as_raw());
}

#[test]
fn escala_minima_legible() {
    assert!(escala_de_preview(8.0, 64.0) >= 20.0);
    assert!(escala_de_preview(1000.0, 64.0) > 20.0);
}

#[test]
fn fuente_ausente_no_panica() {
    let bitmap = render_subtitle(&preset("FuenteQueNoExiste"), "Hola", 1.0);
    assert!(bitmap.as_raw().iter().any(|canal| *canal != 0));
}
