//! Edición de un preset `.ass` desde los campos del panel de estilos.
//!
//! La conversión de texto a valor vive aquí y no en el `view` por dos razones:
//! iced solo produce `String`, y un `text_input` escribe una tecla por evento.
//! Sin esta función, cada tecla tendría que repetir «¿es número?, ¿está en
//! rango?, ¿y si está a medias?» dentro del `update`, y no habría forma de
//! probarlo sin abrir la ventana.

use rod_core::PresetAss;

/// Campos que se pueden tocar desde el panel. Los que el oráculo muestra con
/// un control propio (fuente instalada, color) siguen siendo campos de texto
/// aquí: ver el checklist por qué.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CampoPreset {
    Nombre,
    Fontname,
    Fontsize,
    Outline,
    Shadow,
    Alignment,
    MarginL,
    MarginR,
    MarginV,
    Color,
    OutlineColor,
    Bold,
    Italic,
}

/// Aplica `texto` a `campo`. Devuelve `false` si el texto no es un valor
/// aceptable y deja el preset intacto: así el `text_input` puede seguir
/// mostrando lo que el usuario escribió (`"4."`, `""`, `"abc"`) mientras tanto.
///
/// Los booleanos llegan como `"true"`/`"false"` porque son los dos botones
/// del panel; pasar por la misma función mantiene una sola puerta de edición.
pub fn aplicar_campo(preset: &mut PresetAss, campo: CampoPreset, texto: &str) -> bool {
    match campo {
        CampoPreset::Nombre => preset.nombre = texto.to_owned(),
        CampoPreset::Fontname => preset.fontname = texto.to_owned(),
        CampoPreset::Color => preset.color = texto.to_owned(),
        CampoPreset::OutlineColor => preset.outline_color = texto.to_owned(),
        CampoPreset::Bold | CampoPreset::Italic => match texto {
            "true" => setear_bool(preset, campo, true),
            "false" => setear_bool(preset, campo, false),
            _ => return false,
        },
        CampoPreset::Alignment => {
            let Ok(valor) = texto.parse::<i32>() else {
                return false;
            };
            // libass usa la numeración del teclado numérico: 1 abajo-izq …
            // 9 arriba-der. Fuera de 1..=9 el cuadro de alineación no tiene
            // botón que lo represente.
            if !(1..=9).contains(&valor) {
                return false;
            }
            preset.alignment = valor;
        }
        CampoPreset::MarginL | CampoPreset::MarginR | CampoPreset::MarginV => {
            let Ok(valor) = texto.parse::<i32>() else {
                return false;
            };
            match campo {
                CampoPreset::MarginL => preset.margin_l = valor,
                CampoPreset::MarginR => preset.margin_r = valor,
                _ => preset.margin_v = valor,
            }
        }
        CampoPreset::Fontsize | CampoPreset::Outline | CampoPreset::Shadow => {
            // Rust acepta `"4."` como 4.0. Si se aplicara, el borrador se
            // descartaría a mités de un decimal y el campo saltaría a `4`
            // antes de que el usuario escriba el `5`.
            if texto.trim_end().ends_with('.') {
                return false;
            }
            let Ok(valor) = texto.parse::<f64>() else {
                return false;
            };
            // `parse::<f64>()` acepta `NaN` e `inf`: un `.ass` con esos valores
            // no se puede rasterizar, así que se descartan aquí.
            if !valor.is_finite() {
                return false;
            }
            match campo {
                CampoPreset::Fontsize => preset.fontsize = valor,
                CampoPreset::Outline => preset.outline = valor,
                _ => preset.shadow = valor,
            }
        }
    }
    true
}

fn setear_bool(preset: &mut PresetAss, campo: CampoPreset, valor: bool) {
    match campo {
        CampoPreset::Bold => preset.bold = valor,
        CampoPreset::Italic => preset.italic = valor,
        _ => {}
    }
}

/// Valor que debe mostrar un campo, con el borrador por encima del preset
/// cuando hay uno pendiente. El prefijo del borrador es el id del preset:
/// cambiar de preset no debe arrastrar un texto a medias del anterior.
pub fn valor_campo(
    preset: &PresetAss,
    campo: CampoPreset,
    borradores: &std::collections::HashMap<(String, CampoPreset), String>,
) -> String {
    if let Some(borrador) = borradores.get(&(preset.id.clone(), campo)) {
        return borrador.clone();
    }
    match campo {
        CampoPreset::Nombre => preset.nombre.clone(),
        CampoPreset::Fontname => preset.fontname.clone(),
        CampoPreset::Fontsize => preset.fontsize.to_string(),
        CampoPreset::Outline => preset.outline.to_string(),
        CampoPreset::Shadow => preset.shadow.to_string(),
        CampoPreset::Alignment => preset.alignment.to_string(),
        CampoPreset::MarginL => preset.margin_l.to_string(),
        CampoPreset::MarginR => preset.margin_r.to_string(),
        CampoPreset::MarginV => preset.margin_v.to_string(),
        CampoPreset::Color => preset.color.clone(),
        CampoPreset::OutlineColor => preset.outline_color.clone(),
        CampoPreset::Bold => preset.bold.to_string(),
        CampoPreset::Italic => preset.italic.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{CampoPreset, aplicar_campo, valor_campo};
    use rod_core::preset_por_defecto;
    use std::collections::HashMap;

    #[test]
    fn acepta_decimales_y_enteros() {
        let mut preset = preset_por_defecto();
        assert!(aplicar_campo(&mut preset, CampoPreset::Fontsize, "72.5"));
        assert_eq!(preset.fontsize, 72.5);
        assert!(aplicar_campo(&mut preset, CampoPreset::MarginV, "60"));
        assert_eq!(preset.margin_v, 60);
        assert!(aplicar_campo(&mut preset, CampoPreset::Outline, "2.25"));
        assert_eq!(preset.outline, 2.25);
    }

    #[test]
    fn texto_invalido_no_toca_el_preset() {
        let mut preset = preset_por_defecto();
        let antes = preset.clone();
        for texto in ["", "abc", "4.", "NaN", "inf", "1e400"] {
            assert!(
                !aplicar_campo(&mut preset, CampoPreset::Fontsize, texto),
                "{texto}"
            );
        }
        assert_eq!(preset, antes);
    }

    #[test]
    fn la_alineacion_solo_va_de_uno_a_nueve() {
        let mut preset = preset_por_defecto();
        assert!(aplicar_campo(&mut preset, CampoPreset::Alignment, "7"));
        assert_eq!(preset.alignment, 7);
        for texto in ["0", "10", "-1", "2.0"] {
            assert!(
                !aplicar_campo(&mut preset, CampoPreset::Alignment, texto),
                "{texto}"
            );
        }
        assert_eq!(preset.alignment, 7);
    }

    #[test]
    fn bold_e_italic_son_booleanos_explicitos() {
        let mut preset = preset_por_defecto();
        assert!(aplicar_campo(&mut preset, CampoPreset::Bold, "true"));
        assert!(aplicar_campo(&mut preset, CampoPreset::Italic, "true"));
        assert!(preset.bold && preset.italic);
        assert!(aplicar_campo(&mut preset, CampoPreset::Bold, "false"));
        assert!(!preset.bold && preset.italic);
        assert!(!aplicar_campo(&mut preset, CampoPreset::Italic, "sí"));
        assert!(preset.italic);
    }

    #[test]
    fn los_campos_de_texto_aceptan_cualquier_contenido() {
        let mut preset = preset_por_defecto();
        assert!(aplicar_campo(&mut preset, CampoPreset::Nombre, ""));
        assert!(aplicar_campo(&mut preset, CampoPreset::Fontname, "Georgia"));
        assert!(aplicar_campo(&mut preset, CampoPreset::Color, "#FFE9A8"));
        assert_eq!(preset.fontname, "Georgia");
        assert_eq!(preset.color, "#FFE9A8");
    }

    #[test]
    fn el_borrador_solo_aplica_al_preset_y_campo_correctos() {
        let preset = preset_por_defecto();
        let mut otros = preset.clone();
        otros.id = "otro".into();
        let mut borradores = HashMap::new();
        borradores.insert((preset.id.clone(), CampoPreset::Fontsize), "4.".into());
        assert_eq!(
            valor_campo(&preset, CampoPreset::Fontsize, &borradores),
            "4."
        );
        assert_eq!(
            valor_campo(&otros, CampoPreset::Fontsize, &borradores),
            "48"
        );
        assert_eq!(valor_campo(&preset, CampoPreset::Shadow, &borradores), "1");
    }
}
