//! Sistema visual swiss/brutalista: negro, hairlines, un solo acento mostaza.
//!
//! Los colores de los hablantes NO viven aquí: son semánticos (identifican el
//! carril en el timeline y el color en el `.ass`) y se respetan donde significan
//! algo. El mostaza tiñe solo la chrome.

use iced::font::{Family, Weight};
use iced::widget::button;
use iced::{Background, Border, Color, Font, Shadow};

pub const SUPERFICIE: Color = Color::from_rgb(0.047, 0.047, 0.051); // #0C0C0D
pub const PANEL: Color = Color::from_rgb(0.078, 0.078, 0.086); // #141416
pub const RAISED: Color = Color::from_rgb(0.102, 0.102, 0.114); // #1A1A1D
pub const SUNK: Color = Color::from_rgb(0.063, 0.063, 0.071); // #101012
pub const HAIRLINE: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.15);
pub const LINE2: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.32);
pub const TEXTO: Color = Color::from_rgb(0.953, 0.953, 0.953); // #F3F3F3
pub const SECUNDARIO: Color = Color::from_rgb(154.0 / 255.0, 154.0 / 255.0, 154.0 / 255.0); // #9A9A9A
pub const DIM: Color = Color::from_rgb(107.0 / 255.0, 107.0 / 255.0, 109.0 / 255.0); // #6B6B6D
pub const MOSTAZA: Color = Color::from_rgb(0.851, 0.643, 0.255); // #D9A441
pub const MOSTAZA_HI: Color = Color::from_rgb(0.949, 0.757, 0.373); // #F2C15F
pub const MOSTAZA_LINE: Color = Color::from_rgba(0.851, 0.643, 0.255, 0.55);
pub const MOSTAZA_WASH: Color = Color::from_rgb(0.090, 0.071, 0.027); // #171207

/// Display apretado y monospace para etiquetas: no se carga ningún archivo,
/// así que funciona igual en Windows y en el CI de Ubuntu.
pub const DISPLAY: Font = Font {
    family: Family::SansSerif,
    weight: Weight::Black,
    ..Font::DEFAULT
};
pub const MONO: Font = Font::MONOSPACE;

const RADIO: f32 = 3.0;

/// Color semántico del `.ass` de un hablante. Un valor ausente o mal escrito
/// cae al gris secundario, nunca al mostaza: el mostaza es de la chrome.
pub fn color_de_hex(valor: &str) -> Color {
    let hexadecimal = valor.strip_prefix('#').unwrap_or(valor);
    if hexadecimal.len() != 6 {
        return SECUNDARIO;
    }
    let components = [0, 2, 4].map(|inicio| {
        hexadecimal
            .get(inicio..inicio + 2)
            .and_then(|par| u8::from_str_radix(par, 16).ok())
            .map(|componente| f32::from(componente) / 255.0)
    });
    let [r, g, b] = components;
    match (r, g, b) {
        (Some(r), Some(g), Some(b)) => Color::from_rgb(r, g, b),
        _ => SECUNDARIO,
    }
}

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

/// Superficie hundida: la zona de arrastre y los campos.
pub fn sunk(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(SUNK)),
        text_color: Some(TEXTO),
        border: Border {
            color: LINE2,
            width: 1.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Aviso con fondo mostaza muy diluido: tarjeta de recuperación.
pub fn wash(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(MOSTAZA_WASH)),
        text_color: Some(TEXTO),
        border: Border {
            color: MOSTAZA_LINE,
            width: 1.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Línea de 1 px: regla, borde inferior de la barra de menú, cruz de registro.
pub fn hairline(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(HAIRLINE)),
        ..Default::default()
    }
}

pub fn boton_primario(_theme: &iced::Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(if status == button::Status::Hovered {
            MOSTAZA_HI
        } else {
            MOSTAZA
        })),
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

pub fn entrada(
    _theme: &iced::Theme,
    _status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    iced::widget::text_input::Style {
        background: Background::Color(PANEL),
        border: Border {
            color: HAIRLINE,
            width: 1.0,
            radius: RADIO.into(),
        },
        icon: SECUNDARIO,
        placeholder: SECUNDARIO,
        value: TEXTO,
        selection: MOSTAZA,
    }
}

pub fn aviso(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(MOSTAZA_WASH)),
        text_color: Some(MOSTAZA),
        border: Border {
            color: MOSTAZA_LINE,
            width: 1.0,
            radius: RADIO.into(),
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
    fn secundario_es_el_gris_exacto_de_la_spec() {
        // 0.60 era una aproximación; la spec fija #9A9A9A = 154/255.
        assert!((SECUNDARIO.r - 154.0 / 255.0).abs() < 0.001);
        assert!((SECUNDARIO.g - 154.0 / 255.0).abs() < 0.001);
        assert!((SECUNDARIO.b - 154.0 / 255.0).abs() < 0.001);
    }

    fn radio_valido(radio: iced::border::Radius) -> bool {
        [
            radio.top_left,
            radio.top_right,
            radio.bottom_right,
            radio.bottom_left,
        ]
        .iter()
        .all(|valor| *valor <= 3.0)
    }

    #[test]
    fn color_de_hex_lee_seis_digitos() {
        let azul = color_de_hex("#4C8DFF");
        assert!((azul.r - 76.0 / 255.0).abs() < 0.001);
        assert!((azul.g - 141.0 / 255.0).abs() < 0.001);
        assert!((azul.b - 1.0).abs() < 0.001);
        assert_eq!(color_de_hex("ffffff"), Color::WHITE);
    }

    #[test]
    fn color_de_hex_invalido_no_devuelve_mostaza() {
        for entrada in ["", "rojo", "#123", "#GGGGGG", "#1234567"] {
            assert_eq!(color_de_hex(entrada), SECUNDARIO, "{entrada}");
            assert_ne!(color_de_hex(entrada), MOSTAZA);
        }
    }

    #[test]
    fn ningun_estilo_tiene_sombra_ni_radio_mayor_a_tres() {
        let tema = iced::Theme::Dark;
        for estilo in [
            fondo(&tema),
            panel(&tema),
            aviso(&tema),
            sunk(&tema),
            wash(&tema),
        ] {
            assert_eq!(estilo.shadow, Shadow::default());
            assert!(radio_valido(estilo.border.radius));
        }
        for estilo in [
            boton_primario(&tema, button::Status::Active),
            boton_secundario(&tema, button::Status::Hovered),
            boton_menu(&tema, button::Status::Disabled),
        ] {
            assert_eq!(estilo.shadow, Shadow::default());
            assert!(radio_valido(estilo.border.radius));
        }
        let campo = entrada(&tema, iced::widget::text_input::Status::Active);
        assert!(radio_valido(campo.border.radius));
        let barra = barra_menu(&tema, iced_aw::style::Status::Active);
        assert_eq!(barra.bar_shadow, Shadow::default());
        assert_eq!(barra.menu_shadow, Shadow::default());
        assert!(radio_valido(barra.menu_border.radius));
        assert!(radio_valido(barra.path_border.radius));
    }

    #[test]
    fn el_color_de_los_hablantes_no_es_el_acento() {
        // Los presets del oráculo usan azul, rojo, etc. El mostaza no debe
        // sustituirlos: solo tiñe la chrome.
        assert_ne!(MOSTAZA, Color::from_rgb(0.231, 0.510, 0.965));
    }
}
