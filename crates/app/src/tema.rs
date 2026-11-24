//! Minimalismo corporativo tipo PhotoCraft: gris medio, hairlines y un solo
//! acento azul para CTA, foco y selección.
//!
//! Los colores de los hablantes NO viven aquí: son semánticos (identifican el
//! carril en el timeline y el color en el `.ass`) y se respetan donde significan
//! algo. El azul tiñe solo la chrome.

use iced::font::{Family, Weight};
use iced::widget::button;
use iced::{Background, Border, Color, Font, Shadow};

/// Barra de menú y fondo de la aplicación.
pub const CHROME: Color = Color::from_rgb(74.0 / 255.0, 74.0 / 255.0, 74.0 / 255.0); // #4A4A4A
/// Preview de video: lo más oscuro de la interfaz.
pub const SUPERFICIE: Color = Color::from_rgb(14.0 / 255.0, 14.0 / 255.0, 15.0 / 255.0); // #0E0E0F
pub const PANEL: Color = Color::from_rgb(44.0 / 255.0, 44.0 / 255.0, 44.0 / 255.0); // #2C2C2C
pub const LATERAL: Color = Color::from_rgb(41.0 / 255.0, 41.0 / 255.0, 41.0 / 255.0); // #292929
pub const RAISED: Color = Color::from_rgb(64.0 / 255.0, 64.0 / 255.0, 64.0 / 255.0); // #404040
pub const SUNK: Color = Color::from_rgb(35.0 / 255.0, 35.0 / 255.0, 35.0 / 255.0); // #232323
pub const HAIRLINE: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.10);
pub const LINE2: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.18);
pub const TEXTO: Color = Color::from_rgb(232.0 / 255.0, 232.0 / 255.0, 232.0 / 255.0); // #E8E8E8
pub const SECUNDARIO: Color = Color::from_rgb(184.0 / 255.0, 184.0 / 255.0, 184.0 / 255.0); // #B8B8B8
pub const DIM: Color = Color::from_rgb(140.0 / 255.0, 140.0 / 255.0, 140.0 / 255.0); // #8C8C8C
pub const ACENTO: Color = Color::from_rgb(20.0 / 255.0, 115.0 / 255.0, 230.0 / 255.0); // #1473E6
pub const ACENTO_HI: Color = Color::from_rgb(43.0 / 255.0, 130.0 / 255.0, 232.0 / 255.0); // #2B82E8
pub const ACENTO_LINE: Color = Color::from_rgba(20.0 / 255.0, 115.0 / 255.0, 230.0 / 255.0, 0.42);
pub const ACENTO_WASH: Color = Color::from_rgba(20.0 / 255.0, 115.0 / 255.0, 230.0 / 255.0, 0.11);

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
/// cae al gris secundario, nunca al azul: el azul es de la chrome.
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
        background: Some(Background::Color(CHROME)),
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

/// Columna estrecha de navegación del arranque.
pub fn lateral(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(LATERAL)),
        text_color: Some(TEXTO),
        border: Border {
            color: HAIRLINE,
            width: 1.0,
            ..Default::default()
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
        background: Some(Background::Color(ACENTO_WASH)),
        text_color: Some(TEXTO),
        border: Border {
            color: ACENTO_LINE,
            width: 1.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Marca de la barra de menú: cuadrado mostaza con la R en negro.
pub fn marca_solida(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(ACENTO)),
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

/// Marca de registro de la zona de arrastre: cuadrado de 18 px sin relleno.
pub fn marca(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        border: Border {
            color: ACENTO_LINE,
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
            ACENTO_HI
        } else {
            ACENTO
        })),
        text_color: Color::WHITE,
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
        background: Background::Color(SUNK),
        border: Border {
            color: HAIRLINE,
            width: 1.0,
            radius: RADIO.into(),
        },
        icon: SECUNDARIO,
        placeholder: SECUNDARIO,
        value: TEXTO,
        selection: ACENTO,
    }
}

pub fn aviso(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(ACENTO_WASH)),
        text_color: Some(ACENTO),
        border: Border {
            color: ACENTO_LINE,
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
        bar_background: Background::Color(CHROME),
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
        path: Background::Color(ACENTO),
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
    fn el_acento_es_azul_y_el_preview_es_casi_negro() {
        const {
            assert!(ACENTO.b > ACENTO.r);
            assert!(ACENTO.b > ACENTO.g);
            assert!(SUPERFICIE.r < 0.1 && SUPERFICIE.g < 0.1 && SUPERFICIE.b < 0.1);
        }
    }

    #[test]
    fn secundario_es_el_gris_del_registro_corporativo() {
        // Registro Adobe/PhotoCraft: #B8B8B8 = 184/255.
        assert!((SECUNDARIO.r - 184.0 / 255.0).abs() < 0.001);
        assert!((SECUNDARIO.g - 184.0 / 255.0).abs() < 0.001);
        assert!((SECUNDARIO.b - 184.0 / 255.0).abs() < 0.001);
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
    fn color_de_hex_invalido_no_devuelve_el_acento() {
        for entrada in ["", "rojo", "#123", "#GGGGGG", "#1234567"] {
            assert_eq!(color_de_hex(entrada), SECUNDARIO, "{entrada}");
            assert_ne!(color_de_hex(entrada), ACENTO);
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
            lateral(&tema),
            marca_solida(&tema),
            marca(&tema),
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
        assert_ne!(ACENTO, Color::from_rgb(0.298, 0.553, 1.0));
        assert_ne!(ACENTO, Color::from_rgb(0.851, 0.333, 0.247));
        assert_ne!(ACENTO, Color::from_rgb(0.247, 0.647, 0.478));
    }
}
