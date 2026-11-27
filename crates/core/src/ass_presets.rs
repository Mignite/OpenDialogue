//! Persistencia de los presets `.ass` en `ass-presets.json`.
//!
//! El archivo lo comparten la app Tauri y esta, así que el formato es JSON con
//! campos en camelCase (`outlineColor`, `marginL`…). Se usa `serde_json` en vez
//! de un parser propio porque el formato tiene que ser el mismo en los dos
//! lados: reinventarlo solo agrega una fuente de desacuerdo.

use serde_json::{Map, Value};

use crate::{PresetAss, preset_por_defecto};

const CAMPOS_TEXTO: [&str; 5] = ["id", "nombre", "fontname", "color", "outlineColor"];
const CAMPOS_NUMERO: [&str; 7] = [
    "fontsize",
    "outline",
    "shadow",
    "alignment",
    "marginL",
    "marginR",
    "marginV",
];

/// Un preset sin ninguno de sus campos obligatorios no es un preset: se descarta
/// en vez de rellenarse a medias y exportar un `.ass` con campos en cero.
fn preset_desde_valor(valor: &Value) -> Option<PresetAss> {
    let objeto = valor.as_object()?;
    for campo in CAMPOS_TEXTO {
        if !objeto.get(campo).is_some_and(Value::is_string) {
            return None;
        }
    }
    for campo in CAMPOS_NUMERO {
        let numero = objeto.get(campo).and_then(Value::as_f64)?;
        if !numero.is_finite() {
            return None;
        }
    }

    let texto = |campo: &str| objeto[campo].as_str().unwrap_or_default().to_string();
    let numero = |campo: &str| objeto[campo].as_f64().unwrap_or(0.0);
    // Los presets guardados antes de que existieran bold/italic llegan sin esas
    // claves: se rellenan a `false`. Y un valor que no sea booleano (guarda
    // corrupta) se descarta en vez de propagarse al `.ass`.
    let booleano = |campo: &str| matches!(objeto.get(campo), Some(Value::Bool(true)));

    Some(PresetAss {
        id: texto("id"),
        nombre: texto("nombre"),
        fontname: texto("fontname"),
        fontsize: numero("fontsize"),
        color: texto("color"),
        outline_color: texto("outlineColor"),
        outline: numero("outline"),
        shadow: numero("shadow"),
        alignment: numero("alignment") as i32,
        margin_l: numero("marginL") as i32,
        margin_r: numero("marginR") as i32,
        margin_v: numero("marginV") as i32,
        bold: booleano("bold"),
        italic: booleano("italic"),
    })
}

fn preset_a_valor(preset: &PresetAss) -> Value {
    let mut objeto = Map::new();
    objeto.insert("id".into(), Value::String(preset.id.clone()));
    objeto.insert("nombre".into(), Value::String(preset.nombre.clone()));
    objeto.insert("fontname".into(), Value::String(preset.fontname.clone()));
    objeto.insert("fontsize".into(), Value::from(preset.fontsize));
    objeto.insert("color".into(), Value::String(preset.color.clone()));
    objeto.insert(
        "outlineColor".into(),
        Value::String(preset.outline_color.clone()),
    );
    objeto.insert("outline".into(), Value::from(preset.outline));
    objeto.insert("shadow".into(), Value::from(preset.shadow));
    objeto.insert("alignment".into(), Value::from(preset.alignment));
    objeto.insert("marginL".into(), Value::from(preset.margin_l));
    objeto.insert("marginR".into(), Value::from(preset.margin_r));
    objeto.insert("marginV".into(), Value::from(preset.margin_v));
    objeto.insert("bold".into(), Value::Bool(preset.bold));
    objeto.insert("italic".into(), Value::Bool(preset.italic));
    Value::Object(objeto)
}

/// Presets desde el JSON guardado. Cualquier problema (archivo ausente, JSON
/// corrupto, lista vacía, entradas incompletas) cae a la semilla de un preset
/// por defecto: la app tiene que abrir siempre con algo usable.
pub fn presets_desde_json(texto: Option<&str>) -> Vec<PresetAss> {
    let Some(texto) = texto.filter(|texto| !texto.is_empty()) else {
        return semilla();
    };
    let Ok(data) = serde_json::from_str::<Value>(texto) else {
        return semilla();
    };

    let lista: Vec<PresetAss> = data
        .get("presets")
        .and_then(Value::as_array)
        .map(|entradas| entradas.iter().filter_map(preset_desde_valor).collect())
        .unwrap_or_default();

    if lista.is_empty() { semilla() } else { lista }
}

pub fn presets_a_json(presets: &[PresetAss]) -> String {
    let mut raiz = Map::new();
    raiz.insert(
        "presets".into(),
        Value::Array(presets.iter().map(preset_a_valor).collect()),
    );
    // Igual que `JSON.stringify({presets}, null, 2)`: el archivo lo lee también
    // la app Tauri y un diff legible ayuda cuando alguien lo edita a mano.
    serde_json::to_string_pretty(&Value::Object(raiz)).unwrap_or_default()
}

fn semilla() -> Vec<PresetAss> {
    vec![preset_por_defecto()]
}

/// Un preset nuevo: los valores del default con un id propio.
pub fn nuevo_preset() -> PresetAss {
    nuevo_preset_desde(&preset_por_defecto())
}

/// Copia `base` con un id nuevo. Sirve para "duplicar" y para crear un preset
/// a partir de otro sin arrastrar su identidad.
pub fn nuevo_preset_desde(base: &PresetAss) -> PresetAss {
    PresetAss {
        id: id_nuevo(),
        ..base.clone()
    }
}

/// `preset-{millis en base36}-{contador}`. El contador hace falta porque dos
/// llamadas en el mismo milisegundo (duplicar con doble clic) darían el mismo
/// id y una pisaría a la otra.
fn id_nuevo() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static CONTADOR: AtomicU64 = AtomicU64::new(0);

    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duracion| duracion.as_millis());
    let contador = CONTADOR.fetch_add(1, Ordering::Relaxed);
    format!("preset-{}-{}", base36(millis as u64), base36(contador))
}

fn base36(mut valor: u64) -> String {
    const DIGITOS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if valor == 0 {
        return "0".into();
    }
    let mut salida = Vec::new();
    while valor > 0 {
        salida.push(DIGITOS[(valor % 36) as usize]);
        valor /= 36;
    }
    salida.reverse();
    String::from_utf8(salida).unwrap_or_default()
}

/// `"Default"` → `"Default 2"`, y si ya existe, `"Default 3"`. Evita el
/// "Default copia copia" de concatenar la palabra cada vez: el número anterior
/// se descarta antes de numerar. Si la base está libre se devuelve tal cual; si
/// viene vacía se parte de `"Preset"`.
pub fn nombre_libre(base: &str, existentes: &[String]) -> String {
    let limpio = sin_sufijo_numerico(base.trim());
    let limpio = if limpio.is_empty() { "Preset" } else { limpio };
    if !existentes.iter().any(|nombre| nombre == limpio) {
        return limpio.to_string();
    }
    let mut n = 2;
    loop {
        let candidato = format!("{limpio} {n}");
        if !existentes.contains(&candidato) {
            return candidato;
        }
        n += 1;
    }
}

/// Quita un sufijo ` <dígitos>` final: `"Default 2"` → `"Default"`, y
/// `"Default"` → `"Default"`.
fn sin_sufijo_numerico(nombre: &str) -> &str {
    let sin_digitos = nombre.trim_end_matches(|c: char| c.is_ascii_digit());
    if sin_digitos.len() == nombre.len() {
        return nombre;
    }
    let recortado = sin_digitos.trim_end();
    if recortado.len() == sin_digitos.len() {
        // Los dígitos no estaban separados por espacio ("Cine2"): es parte del
        // nombre, no un contador.
        return nombre;
    }
    recortado
}

#[cfg(test)]
mod tests {
    use super::{
        nombre_libre, nuevo_preset, nuevo_preset_desde, presets_a_json, presets_desde_json,
    };
    use crate::{PresetAss, preset_por_defecto};

    // Casos portados de `__tests__/assPresets.test.ts`, uno por `it`.

    fn p1() -> PresetAss {
        PresetAss {
            id: "p1".into(),
            nombre: "Blanco".into(),
            ..preset_por_defecto()
        }
    }

    fn nombres(lista: &[&str]) -> Vec<String> {
        lista.iter().map(|nombre| nombre.to_string()).collect()
    }

    use serde_json::json;

    #[test]
    fn seeds_the_default_preset_when_there_is_no_file() {
        let presets = presets_desde_json(None);
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].nombre, "Default");
        assert_eq!(presets[0].color, "#FFFFFF");
        assert_eq!(presets[0].shadow, 1.0);
    }

    #[test]
    fn seeds_the_default_when_the_json_is_corrupt() {
        assert_eq!(presets_desde_json(Some("{no es json")).len(), 1);
    }

    #[test]
    fn seeds_the_default_when_the_list_is_empty_or_not_an_array() {
        assert_eq!(presets_desde_json(Some(r#"{"presets":[]}"#)).len(), 1);
        assert_eq!(presets_desde_json(Some(r#"{"presets":"nope"}"#)).len(), 1);
    }

    #[test]
    fn discards_entries_with_missing_fields_or_odd_types() {
        let texto = json!({
            "presets": [
                {
                    "id": "p1",
                    "nombre": "Blanco",
                    "fontname": "Arial",
                    "fontsize": 48,
                    "color": "#FFFFFF",
                    "outlineColor": "#000000",
                    "outline": 2,
                    "shadow": 1,
                    "alignment": 2,
                    "marginL": 10,
                    "marginR": 10,
                    "marginV": 40,
                },
                {"id": "roto", "nombre": "Sin números"},
                {
                    "nombre": "Sin id",
                    "fontname": "X",
                    "fontsize": "grande",
                    "color": "#fff",
                    "outlineColor": "#000",
                    "outline": 1,
                    "shadow": 1,
                    "alignment": 2,
                    "marginL": 1,
                    "marginR": 1,
                    "marginV": 1,
                },
            ],
        });
        let presets = presets_desde_json(Some(&texto.to_string()));
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].id, "p1");
    }

    #[test]
    fn reads_a_valid_list() {
        let mut segundo = p1();
        segundo.id = "p2".into();
        let texto = presets_a_json(&[p1(), segundo]);
        let presets = presets_desde_json(Some(&texto));
        let ids: Vec<&str> = presets.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["p1", "p2"]);
    }

    // Los presets guardados antes de que existieran bold/italic llegan sin esas
    // claves. No deben rechazarse ni quedar como ausentes, que se colarían en
    // el `.ass`.
    #[test]
    fn fills_bold_and_italic_with_false_for_old_presets() {
        let viejo = json!({
            "presets": [{
                "id": "viejo",
                "nombre": "Viejo",
                "fontname": "Arial",
                "fontsize": 48,
                "color": "#FFFFFF",
                "outlineColor": "#000000",
                "outline": 2,
                "shadow": 1,
                "alignment": 2,
                "marginL": 10,
                "marginR": 10,
                "marginV": 40,
            }],
        });
        let presets = presets_desde_json(Some(&viejo.to_string()));
        assert_eq!(presets.len(), 1);
        assert!(!presets[0].bold);
        assert!(!presets[0].italic);
    }

    #[test]
    fn keeps_bold_and_italic_from_a_preset_that_has_them() {
        let mut con_peso = p1();
        con_peso.bold = true;
        con_peso.italic = true;
        let texto = presets_a_json(&[con_peso]);
        let presets = presets_desde_json(Some(&texto));
        assert!(presets[0].bold);
        assert!(presets[0].italic);
    }

    #[test]
    fn discards_a_bold_that_is_not_boolean_instead_of_propagating_it() {
        let valor = json!({
            "presets": [{
                "id": "p1",
                "nombre": "Blanco",
                "fontname": "Arial",
                "fontsize": 48,
                "color": "#FFFFFF",
                "outlineColor": "#000000",
                "outline": 2,
                "shadow": 1,
                "alignment": 2,
                "marginL": 10,
                "marginR": 10,
                "marginV": 40,
                "bold": "sí",
                "italic": 1,
            }],
        });
        let presets = presets_desde_json(Some(&valor.to_string()));
        assert!(!presets[0].bold);
        assert!(!presets[0].italic);
    }

    #[test]
    fn survives_the_json_round_trip() {
        let mut segundo = p1();
        segundo.id = "p2".into();
        segundo.nombre = "Con sombra".into();
        segundo.shadow = 4.0;
        let lista = [p1(), segundo];
        let presets = presets_desde_json(Some(&presets_a_json(&lista)));
        assert_eq!(presets, lista);
    }

    #[test]
    fn numbers_from_two_without_touching_the_base_name() {
        assert_eq!(nombre_libre("Default", &nombres(&["Default"])), "Default 2");
    }

    #[test]
    fn skips_the_numbers_already_taken() {
        assert_eq!(
            nombre_libre("Default", &nombres(&["Default", "Default 2"])),
            "Default 3"
        );
    }

    #[test]
    fn does_not_concatenate_the_counter_onto_itself() {
        assert_eq!(
            nombre_libre("Default 2", &nombres(&["Default", "Default 2"])),
            "Default 3"
        );
    }

    #[test]
    fn returns_the_base_as_is_when_it_is_free() {
        assert_eq!(nombre_libre("Cine", &[]), "Cine");
    }

    #[test]
    fn starts_from_preset_when_the_base_is_empty() {
        assert_eq!(nombre_libre("", &[]), "Preset");
        assert_eq!(nombre_libre("  ", &nombres(&["Preset"])), "Preset 2");
    }

    #[test]
    fn a_new_preset_starts_from_the_default_with_a_new_id_and_name() {
        let preset = nuevo_preset();
        assert_ne!(preset.id, preset_por_defecto().id);
        assert_eq!(preset.fontsize, preset_por_defecto().fontsize);
    }

    #[test]
    fn accepts_partial_overrides() {
        let base = PresetAss {
            nombre: "Cine".into(),
            fontsize: 64.0,
            alignment: 8,
            ..preset_por_defecto()
        };
        let preset = nuevo_preset_desde(&base);
        assert_eq!(preset.nombre, "Cine");
        assert_eq!(preset.fontsize, 64.0);
        assert_eq!(preset.alignment, 8);
        assert_ne!(preset.id, base.id);
    }

    #[test]
    fn generates_different_ids_in_two_consecutive_calls() {
        assert_ne!(nuevo_preset().id, nuevo_preset().id);
    }
}
