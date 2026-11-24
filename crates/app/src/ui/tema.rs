//! El tema corporativo portado a `egui::Style`. Los colores son los mismos que
//! en `tema.rs` (iced): gris neutro, un solo acento azul y ninguna sombra.

use egui::{Color32, CornerRadius, FontId, Style, TextStyle, Visuals};

/// Chrome: la estructura alrededor del trabajo.
pub const FONDO: Color32 = Color32::from_rgb(29, 30, 34);
/// Monitor de video: lo más oscuro, para que el frame mande.
pub const SUPERFICIE: Color32 = Color32::from_rgb(11, 11, 13);
/// Paneles e inspectores: un paso por encima del fondo, no otro negro.
pub const PANEL: Color32 = Color32::from_rgb(37, 38, 43);
/// Carriles y superficies elevadas.
pub const LATERAL: Color32 = Color32::from_rgb(32, 33, 38);
pub const BORDE: Color32 = Color32::from_rgb(72, 74, 82);
pub const TEXTO: Color32 = Color32::from_rgb(235, 236, 239);
pub const SECUNDARIO: Color32 = Color32::from_rgb(182, 185, 192);
pub const DIM: Color32 = Color32::from_rgb(126, 129, 137);
pub const ACENTO: Color32 = Color32::from_rgb(20, 115, 230);
/// Sin guardar y pista activa; nunca tiñe un carril de hablante.
pub const AMBAR: Color32 = Color32::from_rgb(229, 169, 59);

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

/// Tipografía mono para el chrome y las etiquetas técnicas.
pub fn mono_public() -> egui::FontId {
    egui::FontId::monospace(10.0)
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

/// Marco de un panel: fondo, borde de un píxel y esquinas suaves.
pub fn panel(fill: Color32) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::Margin::symmetric(10, 8),
        outer_margin: egui::Margin::ZERO,
        corner_radius: egui::CornerRadius::same(RADIO_SUPERFICIE as u8),
        shadow: egui::Shadow::NONE,
        fill,
        stroke: egui::Stroke::new(1.0_f32, BORDE),
    }
}

/// Etiqueta de sección: pequeña, en mono y apagada. Estructura, no adorno.
pub fn etiqueta(ui: &mut egui::Ui, texto: &str) {
    ui.label(
        egui::RichText::new(texto.to_uppercase())
            .font(mono_public())
            .size(9.0)
            .color(DIM),
    );
}

/// Chip compacto para el estado de la cabecera.
pub fn chip(
    ui: &mut egui::Ui,
    texto: impl Into<egui::WidgetText>,
    color: Color32,
) -> egui::Response {
    ui.add(
        egui::Button::new(texto)
            .fill(Color32::TRANSPARENT)
            .corner_radius(egui::CornerRadius::same(RADIO_CONTROL as u8))
            .stroke(egui::Stroke::new(1.0_f32, color)),
    )
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
