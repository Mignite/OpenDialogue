use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Caption {
    pub id: String,
    pub inicio: f64,
    pub fin: f64,
    pub texto: String,
    pub hablante_id: Option<String>,
}

pub fn parse_srt_time(input: &str) -> f64 {
    let timestamp = input.split_whitespace().next().unwrap_or_default();
    let mut fields = timestamp.split(':');
    let (Some(hours), Some(minutes), Some(seconds), None) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return 0.0;
    };

    let Some((whole_seconds, fraction)) = seconds.split_once([',', '.']) else {
        return 0.0;
    };
    let Some(hours) = parse_number(hours) else {
        return 0.0;
    };
    let Some(minutes) = parse_number(minutes) else {
        return 0.0;
    };
    let Some(seconds) = parse_number(whole_seconds) else {
        return 0.0;
    };
    let Some(fraction) = parse_number(&format!("0.{fraction}")) else {
        return 0.0;
    };

    hours * 3600.0 + minutes * 60.0 + seconds + fraction
}

fn parse_number(value: &str) -> Option<f64> {
    value.parse().ok()
}

pub fn parse_srt(text: &str) -> Vec<Caption> {
    let normalized = text.replace('\r', "");
    let mut blocks = Vec::new();
    let mut block = Vec::new();

    for line in normalized.lines() {
        if line.trim().is_empty() {
            if !block.is_empty() {
                blocks.push(std::mem::take(&mut block));
            }
        } else {
            block.push(line);
        }
    }
    if !block.is_empty() {
        blocks.push(block);
    }

    let mut captions = Vec::new();
    for lines in blocks {
        // Un bloque de una sola línea no es un cue: el oráculo exige al menos
        // dos (`lineas.length < 2`, srt.ts), así que una línea de tiempo suelta
        // se ignora en vez de convertirse en cue vacío.
        if lines.len() < 2 {
            continue;
        }
        let Some(time_index) = lines.iter().position(|line| line.contains("-->")) else {
            continue;
        };

        let Some((start, end)) = lines[time_index].split_once("-->") else {
            continue;
        };
        // Un bloque con tiempo y sin texto es un subtítulo vacío, no basura: el
        // oráculo lo conserva con `texto: ""` (srt.ts). Descartarlo perdía cues
        // en blanco al importar y al hacer round-trip.
        let text = lines[time_index + 1..].join("\n");
        captions.push(Caption {
            id: format!("cap-{}", captions.len() + 1),
            inicio: parse_srt_time(start),
            fin: parse_srt_time(end),
            texto: text,
            hablante_id: None,
        });
    }

    captions.sort_by(|left, right| {
        left.inicio
            .partial_cmp(&right.inicio)
            .unwrap_or(Ordering::Equal)
    });
    captions
}

pub fn format_srt_timestamp(seconds: f64) -> String {
    let seconds = seconds.max(0.0);
    let mut milliseconds = ((seconds - seconds.floor()) * 1000.0).round() as u64;
    let mut hours = (seconds / 3600.0).floor() as u64;
    let mut minutes = ((seconds % 3600.0) / 60.0).floor() as u64;
    let mut whole_seconds = (seconds % 60.0).floor() as u64;

    if milliseconds >= 1000 {
        milliseconds = 0;
        whole_seconds += 1;
        if whole_seconds >= 60 {
            whole_seconds = 0;
            minutes += 1;
            if minutes >= 60 {
                minutes = 0;
                hours += 1;
            }
        }
    }

    format!("{hours:02}:{minutes:02}:{whole_seconds:02},{milliseconds:03}")
}

pub fn build_srt(captions: &[Caption]) -> String {
    captions
        .iter()
        .enumerate()
        .map(|(index, caption)| {
            format!(
                "{}\n{} --> {}\n{}\n",
                index + 1,
                format_srt_timestamp(caption.inicio),
                format_srt_timestamp(caption.fin),
                caption.texto
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Nombre del `.srt` de un hablante en el export por hablante.
///
/// Tres cosas que parecen detalles y rompen el export si faltan:
/// - los caracteres prohibidos de Windows (`\ / : * ? " < > |`) pasan a `_`, o
///   el archivo no se puede crear;
/// - los nombres de dispositivo reservados (`CON`, `NUL`, `COM1`…) se prefijan
///   con `_`, porque `NUL.srt` sigue siendo el dispositivo;
/// - la lista de nombres usados se compara sin distinguir mayúsculas: en
///   Windows y macOS `Ana.srt` y `ana.srt` son el mismo archivo y la segunda
///   escritura pisaba a la primera.
///
/// El nombre devuelto conserva las mayúsculas del hablante.
pub fn nombre_archivo_hablante_unico(nombre: &str, usados: &[String]) -> String {
    const PROHIBIDOS: [char; 9] = ['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

    let mut base: String = nombre
        .chars()
        .map(|c| if PROHIBIDOS.contains(&c) { '_' } else { c })
        .collect();

    let antes_del_punto = base
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end_matches([' ', '.']);
    if es_nombre_reservado_windows(antes_del_punto) {
        base = format!("_{base}");
    }

    let usados_minusculas: Vec<String> = usados.iter().map(|u| u.to_lowercase()).collect();
    let libre = |candidato: &str| -> bool {
        let candidato = candidato.to_lowercase();
        !usados_minusculas.contains(&candidato)
    };

    let primero = format!("{base}.srt");
    if libre(&primero) {
        return primero;
    }

    let mut n = 2;
    loop {
        let candidato = format!("{base}_{n}.srt");
        if libre(&candidato) {
            return candidato;
        }
        n += 1;
    }
}

/// `true` si Windows no deja crear un archivo con ese nombre, sin importar la
/// extensión. Se compara solo el nombre anterior al primer punto, ya saneado:
/// `CONyecto` y `COM10` son legales, `COM0` y `LPT0` también (el rango
/// documentado es 1-9). Los dígitos superíndice `¹²³` cuentan como dígitos.
fn es_nombre_reservado_windows(nombre: &str) -> bool {
    let nombre = nombre.to_lowercase();
    if matches!(nombre.as_str(), "con" | "prn" | "aux" | "nul") {
        return true;
    }

    for prefijo in ["com", "lpt"] {
        let Some(resto) = nombre.strip_prefix(prefijo) else {
            continue;
        };
        let mut caracteres = resto.chars();
        if let (Some(unico), None) = (caracteres.next(), caracteres.next())
            && matches!(unico, '1'..='9' | '¹' | '²' | '³')
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::{
        Caption, build_srt, format_srt_timestamp, nombre_archivo_hablante_unico, parse_srt,
        parse_srt_time,
    };

    // Los casos de este módulo vienen de `src/utils/__tests__/srt.test.ts` y
    // `export-nombres.test.ts` de la app Tauri, uno por `it`.

    #[test]
    fn parses_srt_times() {
        assert!((parse_srt_time("00:01:30,500") - 90.5).abs() < 0.001);
        assert!((parse_srt_time("00:00:05.250") - 5.25).abs() < 0.001);
        assert!((parse_srt_time("01:02:03,004") - 3723.004).abs() < 0.001);
        assert_eq!(parse_srt_time(""), 0.0);
        assert_eq!(parse_srt_time("foo"), 0.0);
    }

    #[test]
    fn parses_short_fractions_as_tenths_not_milliseconds() {
        // "5" son 5 décimas (0.5 s), no 5 ms: la cola fraccional es decimal.
        assert!((parse_srt_time("00:00:01,5") - 1.5).abs() < 0.001);
        assert!((parse_srt_time("00:00:01,50") - 1.5).abs() < 0.001);
    }

    #[test]
    fn formats_and_rolls_over_srt_timestamps() {
        assert_eq!(format_srt_timestamp(90.5), "00:01:30,500");
        assert_eq!(format_srt_timestamp(-1.0), "00:00:00,000");
        assert_eq!(format_srt_timestamp(3661.001), "01:01:01,001");
        // 1.9995 redondea a 1000 ms: hay que subir el segundo, no imprimir 1000.
        assert_eq!(format_srt_timestamp(1.9995), "00:00:02,000");
        assert_eq!(format_srt_timestamp(59.9996), "00:01:00,000");
        assert_eq!(format_srt_timestamp(1.999), "00:00:01,999");
    }

    const MUESTRA: &str = "1\n00:00:01,000 --> 00:00:04,000\nHello world\n\n\
                           2\n00:00:05,000 --> 00:00:08,500\nThis is a test\nWith two lines";

    #[test]
    fn parses_basic_content_and_keeps_multi_line_text() {
        let captions = parse_srt(MUESTRA);
        assert_eq!(captions.len(), 2);
        assert!((captions[0].inicio - 1.0).abs() < 0.001);
        assert!((captions[0].fin - 4.0).abs() < 0.001);
        assert_eq!(captions[0].texto, "Hello world");
        assert!((captions[1].inicio - 5.0).abs() < 0.001);
        assert!((captions[1].fin - 8.5).abs() < 0.001);
        assert_eq!(captions[1].texto, "This is a test\nWith two lines");
        assert_eq!(captions[0].hablante_id, None);
    }

    #[test]
    fn empty_input_yields_no_captions() {
        assert!(parse_srt("").is_empty());
        assert!(parse_srt("  ").is_empty());
    }

    #[test]
    fn assigns_unique_ids_with_the_expected_prefix() {
        let captions = parse_srt(MUESTRA);
        assert!(captions[0].id.starts_with("cap-"), "{}", captions[0].id);
        assert!(captions[1].id.starts_with("cap-"), "{}", captions[1].id);
        assert_ne!(captions[0].id, captions[1].id);
    }

    #[test]
    fn parses_blocks_with_space_only_separators_and_sorts_them() {
        let input = "1\n00:00:05,000 --> 00:00:06,000\nSecond\n \n\
                     2\n00:00:01,000 --> 00:00:02,000\nFirst";
        let captions = parse_srt(input);
        assert_eq!(captions.len(), 2);
        assert_eq!(captions[0].texto, "First");
        assert_eq!(captions[1].texto, "Second");
    }

    #[test]
    fn keeps_a_cue_without_text() {
        // Bloque con número y tiempo pero sin líneas de texto: el oráculo lo
        // conserva con texto vacío. Antes lo descartaba y el round-trip perdía
        // cues en blanco.
        let captions = parse_srt("1\n00:00:01,000 --> 00:00:02,000");
        assert_eq!(captions.len(), 1);
        assert_eq!(captions[0].texto, "");
    }

    #[test]
    fn ignores_a_lone_timestamp_line() {
        // Sin número de secuencia el bloque tiene una sola línea y el oráculo lo
        // descarta (`lineas.length < 2`), no lo convierte en cue vacío.
        assert!(parse_srt("00:00:01,000 --> 00:00:02,000").is_empty());
    }

    fn ejemplo() -> Vec<Caption> {
        vec![
            Caption {
                id: "c1".into(),
                inicio: 1.0,
                fin: 4.0,
                texto: "Hello world".into(),
                hablante_id: None,
            },
            Caption {
                id: "c2".into(),
                inicio: 5.0,
                fin: 8.5,
                texto: "Line 1\nLine 2".into(),
                hablante_id: None,
            },
        ]
    }

    #[test]
    fn builds_valid_srt_text() {
        let srt = build_srt(&ejemplo());
        assert!(srt.contains("00:00:01,000 --> 00:00:04,000"), "{srt}");
        assert!(srt.contains("Hello world"));
        assert!(srt.contains("00:00:05,000 --> 00:00:08,500"));
        assert!(srt.contains("Line 1\nLine 2"));
        assert!(srt.starts_with("1\n"), "la numeración empieza en 1: {srt}");
    }

    #[test]
    fn roundtrips_through_parse_and_build() {
        let original = ejemplo();
        let reparsed = parse_srt(&build_srt(&original));
        assert_eq!(reparsed.len(), original.len());
        for (parsed, esperado) in reparsed.iter().zip(&original) {
            assert!((parsed.inicio - esperado.inicio).abs() < 0.01);
            assert!((parsed.fin - esperado.fin).abs() < 0.01);
            assert_eq!(parsed.texto, esperado.texto);
            assert_eq!(parsed.hablante_id, esperado.hablante_id);
        }
    }

    fn usados(lista: &[&str]) -> Vec<String> {
        lista.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn speaker_file_name_uses_the_name_with_extension() {
        assert_eq!(nombre_archivo_hablante_unico("Ana", &[]), "Ana.srt");
    }

    #[test]
    fn speaker_file_name_suffixes_repeated_names() {
        assert_eq!(
            nombre_archivo_hablante_unico("Ana", &usados(&["Ana.srt"])),
            "Ana_2.srt"
        );
        assert_eq!(
            nombre_archivo_hablante_unico("Ana", &usados(&["Ana.srt", "Ana_2.srt"])),
            "Ana_3.srt"
        );
    }

    #[test]
    fn speaker_file_name_suffixes_even_when_the_base_already_has_a_suffix() {
        assert_eq!(nombre_archivo_hablante_unico("Ana_2", &[]), "Ana_2.srt");
        assert_eq!(
            nombre_archivo_hablante_unico("Ana_2", &usados(&["Ana.srt", "Ana_2.srt"])),
            "Ana_2_2.srt"
        );
    }

    #[test]
    fn names_that_sanitize_to_the_same_text_collide() {
        let mut utilizados: Vec<String> = Vec::new();
        let primero = nombre_archivo_hablante_unico("A/B", &utilizados);
        utilizados.push(primero.clone());
        let segundo = nombre_archivo_hablante_unico("A:B", &utilizados);
        assert_eq!(primero, "A_B.srt");
        assert_eq!(segundo, "A_B_2.srt");
    }

    #[test]
    fn speaker_file_name_does_not_confuse_similar_names() {
        assert_eq!(
            nombre_archivo_hablante_unico("Ana", &usados(&["Ana_2.srt"])),
            "Ana.srt"
        );
    }

    #[test]
    fn speaker_file_name_collides_ignoring_case() {
        let mut utilizados: Vec<String> = Vec::new();
        let primero = nombre_archivo_hablante_unico("Ana", &utilizados);
        utilizados.push(primero.clone());
        let segundo = nombre_archivo_hablante_unico("ana", &utilizados);
        assert_eq!(primero, "Ana.srt");
        assert_eq!(segundo, "ana_2.srt");
        // El nombre devuelto conserva las mayúsculas del hablante.
        assert_eq!(
            nombre_archivo_hablante_unico("ana", &usados(&["ANA.srt", "ana_2.srt"])),
            "ana_3.srt"
        );
    }

    #[test]
    fn prefixes_windows_reserved_device_names() {
        for base in [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM9", "LPT1", "LPT9", "con", "nul", "com1",
            "COM¹", "LPT²",
        ] {
            assert_eq!(
                nombre_archivo_hablante_unico(base, &[]),
                format!("_{base}.srt"),
                "{base} es un dispositivo reservado"
            );
        }
    }

    #[test]
    fn does_not_reserve_names_that_only_start_like_a_device() {
        for base in ["CONyecto", "NULO", "COM10", "LPT0", "CONSORCIO"] {
            assert_eq!(
                nombre_archivo_hablante_unico(base, &[]),
                format!("{base}.srt"),
                "{base} no es un dispositivo reservado"
            );
        }
    }

    #[test]
    fn the_reserved_prefix_also_goes_through_deduplication() {
        let mut utilizados = vec![nombre_archivo_hablante_unico("NUL", &[])];
        utilizados.push(nombre_archivo_hablante_unico("NUL", &utilizados));
        assert_eq!(utilizados, usados(&["_NUL.srt", "_NUL_2.srt"]));
    }
}
