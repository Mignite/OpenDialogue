//! El `.cdp`: un proyecto como JSON.
//!
//! Formato y nombres de clave son los del oráculo (`src-tauri/src/lib.rs`,
//! `guardar_proyecto`/`cargar_proyecto`): `{ ruta_video, hablantes, captions,
//! playhead }` con `presetId` en el hablante y `hablante_id` en el caption. Un
//! archivo guardado por cualquiera de las dos apps abre en la otra, que es lo
//! que permite seguir usando la Tauri mientras esta crece.
//!
//! Aquí solo hay texto ↔ struct; el disco lo toca `rod-app`. Bonito (2 espacios)
//! porque el archivo se versiona con git y se lee a mano.

use serde_json::Error;

use crate::Proyecto;

pub fn a_json(proyecto: &Proyecto) -> Result<String, Error> {
    let mut texto = serde_json::to_string_pretty(proyecto)?;
    // `serde_json` no cierra con \n; git y los editores sí lo esperan, y el
    // oráculo escribe el archivo sin él (una diferencia de un byte inofensiva).
    texto.push('\n');
    Ok(texto)
}

pub fn desde_json(texto: &str) -> Result<Proyecto, Error> {
    serde_json::from_str(texto)
}

#[cfg(test)]
mod tests {
    use super::{a_json, desde_json};
    use crate::{Caption, Hablante, Proyecto};

    fn proyecto() -> Proyecto {
        Proyecto {
            ruta_video: "C:/videos/clip.mp4".into(),
            hablantes: vec![Hablante {
                id: "sp-1".into(),
                nombre: "Ana".into(),
                tecla: "1".into(),
                color: "#3B82F6".into(),
                preset_id: Some("p-1".into()),
            }],
            captions: vec![Caption {
                id: "cap-1".into(),
                inicio: 1.0,
                fin: 2.5,
                texto: "Hola, ¿qué tal?".into(),
                hablante_id: Some("sp-1".into()),
            }],
            playhead: 7.25,
        }
    }

    #[test]
    fn ida_y_vuelta_sin_perder_nada() {
        let original = proyecto();
        let texto = a_json(&original).expect("serializa");
        let vuelta = desde_json(&texto).expect("deserializa");
        assert_eq!(vuelta, original);
    }

    #[test]
    fn las_claves_son_las_del_oraculo() {
        let texto = a_json(&proyecto()).expect("serializa");
        for clave in [
            "\"ruta_video\"",
            "\"hablantes\"",
            "\"captions\"",
            "\"playhead\"",
            "\"hablante_id\"",
            "\"presetId\"",
        ] {
            assert!(texto.contains(clave), "falta {clave} en\n{texto}");
        }
        // `preset_id` en snake_case sería un archivo que el oráculo no lee.
        assert!(!texto.contains("\"preset_id\""));
    }

    #[test]
    fn es_json_bonito_y_termina_en_salto() {
        let texto = a_json(&proyecto()).expect("serializa");
        assert!(texto.starts_with("{\n  \"ruta_video\""));
        // git y los editores esperan un archivo de texto que acabe en \n.
        assert!(texto.ends_with('\n'));
    }

    #[test]
    fn un_proyecto_viejo_sin_preset_ni_playhead_carga() {
        // Lo que guardaba la app antes de tener presets de estilo.
        let viejo = r##"{
            "ruta_video": "v.mp4",
            "hablantes": [
                {"id": "h1", "nombre": "A", "tecla": "1", "color": "#fff"}
            ],
            "captions": [
                {"id": "c1", "inicio": 0.0, "fin": 1.0, "texto": "x", "hablante_id": null}
            ]
        }"##;
        let proyecto = desde_json(viejo).expect("un archivo viejo carga");
        assert_eq!(proyecto.playhead, 0.0);
        assert_eq!(proyecto.hablantes[0].preset_id, None);
        assert_eq!(proyecto.captions[0].hablante_id, None);
    }

    #[test]
    fn un_json_roto_es_un_error_y_no_un_panico() {
        assert!(desde_json("{\"ruta_video\": ").is_err());
        assert!(desde_json("[]").is_err(), "una lista no es un proyecto");
    }
}
