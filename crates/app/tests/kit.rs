//! Regresiones del kit: iconos sin tofu y pestanas pulsables.

use egui_kittest::kittest::Queryable;
use rod_app::editor_app::Panel;

#[test]
fn todos_los_iconos_existen_en_la_fuente() {
    use rod_app::ui::kit::{Icono, glifo};

    let bytes = egui_phosphor::Variant::Regular.font_bytes();
    let cara = ttf_parser::Face::parse(bytes, 0).expect("fuente válida");
    for icono in Icono::TODAS {
        let texto = glifo(*icono);
        let mut chars = texto.chars();
        let c = chars.next().expect("un glifo");
        assert!(chars.next().is_none(), "{icono:?} debe ser un solo glifo");
        assert!(
            cara.glyph_index(c).is_some(),
            "{icono:?} no está en la fuente"
        );
    }
}

#[test]
fn tab_bar_devuelve_la_pestana_pulsada() {
    let pulsada = std::cell::Cell::new(None);
    let mut harness = egui_kittest::Harness::builder().build_ui(|ui| {
        if let Some(p) = rod_app::ui::kit::tab_bar(
            ui,
            &[(Panel::Hablantes, "Hablantes", Some(3))],
            Panel::Subtitulos,
        ) {
            pulsada.set(Some(p));
        }
    });
    harness.get_by_label("Hablantes").click();
    harness.run();
    assert_eq!(pulsada.get(), Some(Panel::Hablantes));
}
