//! Color efectivo de un hablante en la UI.

use crate::{Hablante, PresetAss};

/// Color efectivo de un hablante: el de su preset `.ass`, o blanco si no tiene.
///
/// `Hablante.color` existe solo para leer proyectos viejos: el formato `.ass`
/// no tiene noción de color de hablante, así que usar ese campo haría que la UI
/// mostrara un color que el archivo exportado no tiene.
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
}
