//! Color efectivo de un hablante en la UI.

use crate::{Hablante, PresetAss};

/// Color efectivo de un hablante: el de su preset `.ass`, o blanco si no tiene.
///
/// `Hablante.color` existe solo para leer proyectos viejos: el formato `.ass`
/// no tiene noción de color de hablante, así que usar ese campo haría que la UI
/// mostrara un color que el archivo exportado no tiene.
/// Colores de identidad de la UI (no del texto del subtítulo). Doce muestras
/// accesibles sobre fondo oscuro; el azul de acento queda reservado.
pub fn paleta_identidad() -> &'static [&'static str] {
    &[
        "#FF8A6B", "#F2B84B", "#C9D45C", "#A8E06B", "#7BD17F", "#3FC4B8", "#7FE0E8", "#6BB6F0",
        "#8FA9FF", "#B39BFF", "#E0A8F0", "#F27BB5",
    ]
}

/// ¿Es este hex una de las doce muestras? Se compara sin distinción de caso.
pub fn es_color_de_paleta(hex: &str) -> bool {
    paleta_identidad()
        .iter()
        .any(|paleta| paleta.eq_ignore_ascii_case(hex.trim()))
}

/// Color de identidad: el elegido por el usuario si está en la paleta; si no,
/// el que le toca por su posición en el proyecto. Determinista y sin campo
/// nuevo en el `.cdp`.
pub fn color_identidad(hablante: &Hablante, indice: usize) -> &'static str {
    if es_color_de_paleta(&hablante.color) {
        paleta_identidad()
            .iter()
            .copied()
            .find(|paleta| paleta.eq_ignore_ascii_case(hablante.color.trim()))
            .unwrap_or(paleta_identidad()[0])
    } else {
        paleta_identidad()[indice % paleta_identidad().len()]
    }
}

pub fn color_hablante(hablante: &Hablante, presets: &[PresetAss]) -> String {
    hablante
        .preset_id
        .as_deref()
        .and_then(|id| presets.iter().find(|preset| preset.id == id))
        .map_or_else(|| "#FFFFFF".to_string(), |preset| preset.color.clone())
}

#[cfg(test)]
mod tests {
    use super::color_hablante;
    use crate::{Hablante, PresetAss, preset_por_defecto};

    // Casos portados de `__tests__/colorHablante.test.ts`, uno por `it`.

    fn hablante(preset_id: Option<&str>, color: &str) -> Hablante {
        Hablante {
            id: "sp1".into(),
            nombre: "Ana".into(),
            tecla: "1".into(),
            color: color.into(),
            preset_id: preset_id.map(str::to_string),
        }
    }

    fn rojo() -> PresetAss {
        PresetAss {
            id: "p-rojo".into(),
            nombre: "Rojo".into(),
            color: "#E85D4E".into(),
            ..preset_por_defecto()
        }
    }

    fn azul() -> PresetAss {
        PresetAss {
            id: "p-azul".into(),
            nombre: "Azul".into(),
            color: "#4EA8E8".into(),
            ..preset_por_defecto()
        }
    }

    #[test]
    fn without_a_preset_it_returns_white() {
        assert_eq!(color_hablante(&hablante(None, "#E85D4E"), &[]), "#FFFFFF");
    }

    #[test]
    fn without_a_preset_id_it_returns_white_even_with_presets() {
        assert_eq!(
            color_hablante(&hablante(None, "#E85D4E"), &[rojo(), azul()]),
            "#FFFFFF"
        );
    }

    #[test]
    fn an_id_that_does_not_exist_returns_white() {
        assert_eq!(
            color_hablante(&hablante(Some("p-fantasma"), "#E85D4E"), &[rojo()]),
            "#FFFFFF"
        );
    }

    #[test]
    fn with_a_preset_it_returns_the_preset_color_not_the_saved_one() {
        assert_eq!(
            color_hablante(&hablante(Some("p-azul"), "#E85D4E"), &[rojo(), azul()]),
            "#4EA8E8"
        );
    }

    #[test]
    fn ignores_the_speaker_color_completely() {
        assert_eq!(
            color_hablante(&hablante(Some("p-rojo"), "#FF00FF"), &[rojo()]),
            "#E85D4E"
        );
    }

    #[test]
    fn the_default_preset_is_white() {
        let por_defecto = preset_por_defecto();
        assert_eq!(
            color_hablante(&hablante(Some(&por_defecto.id), "#E85D4E"), &[por_defecto]),
            "#FFFFFF"
        );
    }

    fn hablante_color(color: Option<&str>) -> Hablante {
        hablante(None, color.unwrap_or(""))
    }

    #[test]
    fn la_paleta_tiene_doce_colores_unicos_y_ninguno_es_el_acento() {
        use super::paleta_identidad;

        let paleta = paleta_identidad();
        assert_eq!(paleta.len(), 12);
        let mut vistos = std::collections::HashSet::new();
        for color in paleta {
            assert!(vistos.insert(*color), "{color} repetido");
            assert_ne!(*color, "#3B8EFF");
        }
    }

    #[test]
    fn un_color_de_paleta_guardado_manda_sobre_el_indice() {
        use super::color_identidad;

        let h = hablante_color(Some("#F27BB5"));
        assert_eq!(color_identidad(&h, 0), "#F27BB5");
        assert_eq!(color_identidad(&h, 5), "#F27BB5");
    }

    #[test]
    fn un_color_invalido_o_blanco_cae_al_indice_sin_repetir_en_los_doce_primeros() {
        use super::{color_identidad, es_color_de_paleta};

        for i in 0..12 {
            let color = color_identidad(&hablante_color(None), i);
            assert!(es_color_de_paleta(color));
            assert_ne!(color, color_identidad(&hablante_color(None), (i + 1) % 12));
        }
        assert_eq!(
            color_identidad(&hablante_color(None), 12),
            color_identidad(&hablante_color(None), 0)
        );
    }
}
