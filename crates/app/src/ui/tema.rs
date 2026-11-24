//! El tema corporativo portado a `egui::Style`. Los colores son los mismos que
//! en `tema.rs` (iced): gris neutro, un solo acento azul y ninguna sombra.

use egui::{Color32, CornerRadius, FontId, Style, TextStyle, Visuals};

pub const FONDO: Color32 = Color32::from_rgb(74, 74, 74);
pub const SUPERFICIE: Color32 = Color32::from_rgb(14, 14, 15);
pub const PANEL: Color32 = Color32::from_rgb(44, 44, 44);
pub const LATERAL: Color32 = Color32::from_rgb(41, 41, 41);
pub const TEXTO: Color32 = Color32::from_rgb(232, 232, 232);
pub const SECUNDARIO: Color32 = Color32::from_rgb(184, 184, 184);
pub const DIM: Color32 = Color32::from_rgb(140, 140, 140);
pub const ACENTO: Color32 = Color32::from_rgb(20, 115, 230);

/// Controles: botones y campos.
pub const RADIO_CONTROL: f32 = 4.0;
/// Superficies: paneles, tarjetas y avisos.
pub const RADIO_SUPERFICIE: f32 = 6.0;

/// Se usan las familias que egui ya trae registradas. Pedir `FontFamily::Name`
/// sin haber cargado esa fuente hace que epaint entre en pánico al dibujar el
/// primer texto, así que no se nombra ninguna familia externa.
fn familia() -> egui::FontFamily {
    egui::FontFamily::Proportional
}

fn mono() -> egui::FontFamily {
    egui::FontFamily::Monospace
}

/// Color semántico de un hablante. Un valor ausente o mal escrito cae al
/// gris secundario, nunca al azul: el azul es de la chrome.
pub fn color_de_hex(valor: &str) -> Color32 {
    let hexadecimal = valor.strip_prefix('#').unwrap_or(valor);
    if hexadecimal.len() != 6 {
        return SECUNDARIO;
    }
    let componentes = [0, 2, 4].map(|inicio| {
        hexadecimal
            .get(inicio..inicio + 2)
            .and_then(|par| u8::from_str_radix(par, 16).ok())
    });
    match componentes {
        [Some(r), Some(g), Some(b)] => Color32::from_rgb(r, g, b),
        _ => SECUNDARIO,
    }
}

pub fn estilo() -> Style {
    let mut estilo = Style::default();
    let mut visuals = Visuals::dark();
    visuals.override_text_color = Some(TEXTO);
    visuals.panel_fill = FONDO;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = SUPERFICIE;
    visuals.faint_bg_color = LATERAL;
    visuals.selection.bg_fill = ACENTO;
    visuals.hyperlink_color = ACENTO;

    for estado in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        estado.corner_radius = CornerRadius::same(RADIO_CONTROL as u8);
    }
    visuals.widgets.active.bg_fill = ACENTO;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(56, 56, 56);
    visuals.window_corner_radius = CornerRadius::same(RADIO_SUPERFICIE as u8);
    visuals.menu_corner_radius = CornerRadius::same(RADIO_SUPERFICIE as u8);

    estilo.visuals = visuals;
    estilo
        .text_styles
        .insert(TextStyle::Body, FontId::new(12.0, familia()));
    estilo
        .text_styles
        .insert(TextStyle::Button, FontId::new(12.0, familia()));
    estilo
        .text_styles
        .insert(TextStyle::Heading, FontId::new(20.0, familia()));
    estilo
        .text_styles
        .insert(TextStyle::Monospace, FontId::new(10.0, mono()));
    estilo
        .text_styles
        .insert(TextStyle::Small, FontId::new(10.0, familia()));
    estilo
}

#[cfg(test)]
mod tests {
    #[test]
    fn acento_es_azul_corporativo() {
        assert_eq!(
            super::ACENTO,
            egui::Color32::from_hex("#1473E6").expect("hex válido")
        );
    }

    #[test]
    fn la_superficie_es_mas_redondeada_que_el_control() {
        const { assert!(super::RADIO_SUPERFICIE > super::RADIO_CONTROL) };
    }

    #[test]
    fn los_controles_son_menos_redondeados_que_las_ventanas() {
        let estilo = super::estilo();
        assert_eq!(
            estilo.visuals.widgets.active.corner_radius,
            egui::CornerRadius::same(super::RADIO_CONTROL as u8)
        );
        assert_eq!(
            estilo.visuals.window_corner_radius,
            egui::CornerRadius::same(super::RADIO_SUPERFICIE as u8)
        );
    }
}
