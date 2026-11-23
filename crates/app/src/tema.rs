//! Sistema visual swiss/brutalista: negro, hairlines, un solo acento mostaza.
//!
//! Los colores de los hablantes NO viven aquí: son semánticos (identifican el
//! carril en el timeline y el color en el `.ass`) y se respetan donde significan
//! algo. El mostaza tiñe solo la chrome.

use iced::widget::button;
use iced::{Background, Border, Color, Shadow};

pub const SUPERFICIE: Color = Color::from_rgb(0.047, 0.047, 0.051); // #0C0C0D
pub const PANEL: Color = Color::from_rgb(0.078, 0.078, 0.086); // #141416
pub const HAIRLINE: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.15);
pub const TEXTO: Color = Color::from_rgb(0.953, 0.953, 0.953); // #F3F3F3
pub const SECUNDARIO: Color = Color::from_rgb(0.60, 0.60, 0.60);
pub const MOSTAZA: Color = Color::from_rgb(0.851, 0.643, 0.255); // #D9A441

const RADIO: f32 = 3.0;

pub fn fondo(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(SUPERFICIE)),
        text_color: Some(TEXTO),
        ..Default::default()
    }
}

pub fn panel(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(PANEL)),
        text_color: Some(TEXTO),
        border: Border {
            color: HAIRLINE,
            width: 1.0,
            radius: RADIO.into(),
        },
        ..Default::default()
    }
}

pub fn boton_primario(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(MOSTAZA)),
        text_color: SUPERFICIE,
        border: Border {
            radius: RADIO.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn boton_secundario(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(PANEL)),
        text_color: TEXTO,
        border: Border {
            color: HAIRLINE,
            width: 1.0,
            radius: RADIO.into(),
        },
        ..Default::default()
    }
}

pub fn boton_menu(_theme: &iced::Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: if status == button::Status::Disabled {
            SECUNDARIO
        } else {
            TEXTO
        },
        border: Border {
            radius: RADIO.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn barra_menu(
    _theme: &iced::Theme,
    _status: iced_aw::style::Status,
) -> iced_aw::style::menu_bar::Style {
    iced_aw::style::menu_bar::Style {
        bar_background: Background::Color(SUPERFICIE),
        bar_border: Border {
            color: HAIRLINE,
            width: 0.0,
            radius: 0.0.into(),
        },
        bar_shadow: Shadow::default(),
        menu_background: Background::Color(PANEL),
        menu_border: Border {
            color: HAIRLINE,
            width: 1.0,
            radius: RADIO.into(),
        },
        menu_shadow: Shadow::default(),
        path: Background::Color(MOSTAZA),
        path_border: Border {
            radius: RADIO.into(),
            ..Default::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_acento_es_mostaza_y_la_superficie_es_casi_negra() {
        const {
            assert!(MOSTAZA.r > MOSTAZA.b);
            assert!(MOSTAZA.g > MOSTAZA.b);
            assert!(SUPERFICIE.r < 0.1 && SUPERFICIE.g < 0.1 && SUPERFICIE.b < 0.1);
        }
    }

    #[test]
    fn el_color_de_los_hablantes_no_es_el_acento() {
        // Los presets del oráculo usan azul, rojo, etc. El mostaza no debe
        // sustituirlos: solo tiñe la chrome.
        assert_ne!(MOSTAZA, Color::from_rgb(0.231, 0.510, 0.965));
    }
}
