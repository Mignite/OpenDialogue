use crate::Caption;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hablante {
    pub id: String,
    pub nombre: String,
    pub tecla: String,
    pub color: String,
    /// Proyectos viejos no traen la clave: `None` es «sin estilo asignado» y
    /// al exportar se usa el preset por defecto, como en el oráculo.
    #[serde(default, rename = "presetId")]
    pub preset_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PresetAss {
    pub id: String,
    pub nombre: String,
    pub fontname: String,
    pub fontsize: f64,
    pub color: String,
    pub outline_color: String,
    pub outline: f64,
    pub shadow: f64,
    pub alignment: i32,
    pub margin_l: i32,
    pub margin_r: i32,
    pub margin_v: i32,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EstiloAss {
    pub nombre: String,
    pub fontname: String,
    pub fontsize: f64,
    pub primary_colour: String,
    pub outline_colour: String,
    pub outline: f64,
    pub shadow: f64,
    pub alignment: i32,
    pub margin_l: i32,
    pub margin_r: i32,
    pub margin_v: i32,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssParseResult {
    pub captions: Vec<Caption>,
    pub hablantes: Vec<Hablante>,
    pub styles: Vec<EstiloAss>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentoAss {
    pub inicio: f64,
    pub fin: f64,
    pub captions: Vec<Caption>,
}

pub const ASS_STYLE_FORMAT: &str = "Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding";
pub const ASS_EVENTS_FORMAT: &str =
    "Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text";

pub fn hex_to_ass_color(hex: &str) -> String {
    hex_to_ass_color_with_alpha(hex, "00")
}

/// Nombre que se escribe en el campo `Name` de un `Dialogue:`.
///
/// La coma es el separador de campos de la línea, así que un nombre con coma
/// ("Smith, John") corre todos los campos siguientes al parsear. La coma y los
/// espacios que la siguen se funden en **un** espacio, no en dos: reemplazar la
/// coma por un espacio deja `"Smith  John"` y el nombre aparece con doble
/// espacio en el reproductor.
pub fn sanitize_nombre_dialogo(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut chars = name.chars().peekable();
    while let Some(c) = chars.next() {
        if c == ',' {
            out.push(' ');
            while chars.peek().is_some_and(|c| c.is_whitespace()) {
                chars.next();
            }
        } else {
            out.push(c);
        }
    }
    out.trim().to_string()
}

/// El preset con el que arranca un proyecto nuevo. Es `DEFAULT_PRESET_ASS` del
/// oráculo (`constants.ts`): Arial 48 blanco, borde negro de 2, sombra 1,
/// alineado abajo-centro, márgenes 10/10/40, sin bold ni italic.
pub fn preset_por_defecto() -> PresetAss {
    PresetAss {
        id: "preset-default".into(),
        nombre: "Default".into(),
        fontname: "Arial".into(),
        fontsize: 48.0,
        color: "#FFFFFF".into(),
        outline_color: "#000000".into(),
        outline: 2.0,
        shadow: 1.0,
        alignment: 2,
        margin_l: 10,
        margin_r: 10,
        margin_v: 40,
        bold: false,
        italic: false,
    }
}

pub fn overrides_de_linea(line: &PresetAss, base: &PresetAss) -> String {
    let mut overrides = Vec::new();
    if line.fontname != base.fontname {
        overrides.push(format!("\\fn{}", line.fontname));
    }
    if line.fontsize != base.fontsize {
        overrides.push(format!("\\fs{}", line.fontsize));
    }
    if line.bold != base.bold {
        overrides.push(if line.bold { "\\b1" } else { "\\b0" }.into());
    }
    if line.italic != base.italic {
        overrides.push(if line.italic { "\\i1" } else { "\\i0" }.into());
    }
    if !line.color.eq_ignore_ascii_case(&base.color) {
        overrides.push(format!("\\c{}&", hex_to_ass_color(&line.color)));
    }
    if !line.outline_color.eq_ignore_ascii_case(&base.outline_color) {
        overrides.push(format!("\\3c{}&", hex_to_ass_color(&line.outline_color)));
    }
    if line.outline != base.outline {
        overrides.push(format!("\\bord{}", line.outline));
    }
    if line.shadow != base.shadow {
        overrides.push(format!("\\shad{}", line.shadow));
    }
    if overrides.is_empty() {
        String::new()
    } else {
        format!("{{{}}}", overrides.join(""))
    }
}

pub fn fusionar_lineas<F>(captions: &[Caption], preset_de: F, base: &PresetAss) -> String
where
    F: Fn(&Caption) -> PresetAss,
{
    captions
        .iter()
        .map(|caption| {
            let preset = preset_de(caption);
            format!(
                "{}{}",
                overrides_de_linea(&preset, base),
                escape_ass_text(&caption.texto)
            )
        })
        .collect::<Vec<_>>()
        .join("\\N")
}

fn estilo_ass(name: &str, preset: &PresetAss) -> String {
    let bold = if preset.bold { -1 } else { 0 };
    let italic = if preset.italic { -1 } else { 0 };
    format!(
        "Style: {name},{},{},{},{},{},&H00000000,{bold},{italic},0,0,100,100,0,0,1,{},{},{},{},{},{},1",
        preset.fontname,
        preset.fontsize,
        hex_to_ass_color(&preset.color),
        hex_to_ass_color(&preset.color),
        hex_to_ass_color(&preset.outline_color),
        preset.outline,
        preset.shadow,
        preset.alignment,
        preset.margin_l,
        preset.margin_r,
        preset.margin_v
    )
}

pub fn build_ass<F>(
    captions: &[Caption],
    hablantes: &[Hablante],
    preset_de: F,
    res_x: u32,
    res_y: u32,
) -> String
where
    F: Fn(&str) -> PresetAss + Copy,
{
    let style_by_id: HashMap<_, _> = hablantes
        .iter()
        .enumerate()
        .map(|(index, speaker)| (speaker.id.as_str(), format!("H{}", index + 1)))
        .collect();
    let style_for = |speaker_id: Option<&str>| {
        speaker_id
            .and_then(|id| style_by_id.get(id).cloned())
            .unwrap_or_else(|| "Default".into())
    };
    let preset_for = |speaker_id: Option<&str>| preset_de(speaker_id.unwrap_or(""));

    let styles = hablantes
        .iter()
        .enumerate()
        .map(|(index, speaker)| estilo_ass(&format!("H{}", index + 1), &preset_de(&speaker.id)))
        .chain(std::iter::once(estilo_ass("Default", &preset_de(""))))
        .collect::<Vec<_>>();

    let mut segments = segmentar_por_solape(captions);
    segments.sort_by(|left, right| left.inicio.total_cmp(&right.inicio));
    let events = segments
        .iter()
        .map(|segment| {
            let first = &segment.captions[0];
            let speaker_id = first.hablante_id.as_deref();
            let base = preset_for(speaker_id);
            let name = speaker_id
                .and_then(|id| hablantes.iter().find(|speaker| speaker.id == id))
                .map(|speaker| {
                    if speaker.nombre.is_empty() {
                        sanitize_nombre_dialogo(if speaker.tecla.is_empty() {
                            &speaker.id
                        } else {
                            &speaker.tecla
                        })
                    } else {
                        sanitize_nombre_dialogo(&speaker.nombre)
                    }
                })
                .unwrap_or_default();
            format!(
                "Dialogue: 0,{},{},{},{},0,0,{},,{}",
                format_ass_time(segment.inicio),
                format_ass_time(segment.fin),
                style_for(speaker_id),
                name,
                base.margin_v,
                fusionar_lineas(
                    &segment.captions,
                    |caption| preset_for(caption.hablante_id.as_deref()),
                    &base
                )
            )
        })
        .collect::<Vec<_>>();

    format!(
        "[Script Info]\nScriptType: v4.00+\nPlayResX: {res_x}\nPlayResY: {res_y}\nWrapStyle: 0\nScaledBorderAndShadow: yes\n\n[V4+ Styles]\nFormat: {ASS_STYLE_FORMAT}\n{}\n\n[Events]\nFormat: {ASS_EVENTS_FORMAT}\n{}\n",
        styles.join("\n"),
        events.join("\n")
    )
}

fn split_by_format(line: &str, format: &[String]) -> HashMap<String, String> {
    let mut fields = Vec::new();
    let mut rest = line;
    for _ in 0..format.len().saturating_sub(1) {
        if let Some((field, remaining)) = rest.split_once(',') {
            fields.push(field);
            rest = remaining;
        } else {
            fields.push(rest);
            rest = "";
        }
    }
    fields.push(rest);
    format
        .iter()
        .enumerate()
        .map(|(index, key)| {
            (
                key.trim().to_string(),
                fields.get(index).unwrap_or(&"").trim().to_string(),
            )
        })
        .collect()
}

fn number_or(value: Option<&String>, fallback: f64) -> f64 {
    value
        .and_then(|value| value.parse().ok())
        .unwrap_or(fallback)
}

pub fn parse_ass(text: &str) -> Result<AssParseResult, String> {
    let normalized = text.replace('\r', "");
    if !normalized
        .lines()
        .any(|line| line.trim().eq_ignore_ascii_case("ScriptType: v4.00+"))
    {
        return Err("solo .ass v4.00+ (ScriptType), no .ssa v4.00".into());
    }

    let mut section = String::new();
    let mut style_format = ASS_STYLE_FORMAT
        .split(',')
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut event_format = ASS_EVENTS_FORMAT
        .split(',')
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut styles = Vec::new();
    let mut events = Vec::new();
    let mut last_event: Option<usize> = None;
    let known = [
        "Format:",
        "Style:",
        "Dialogue:",
        "Comment:",
        "Picture:",
        "Sound:",
        "Movie:",
        "Command:",
    ];

    for raw_line in normalized.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            section = line.to_ascii_lowercase();
            last_event = None;
            continue;
        }
        if section == "[v4+ styles]" {
            if let Some(format) = line.strip_prefix("Format:") {
                style_format = format
                    .split(',')
                    .map(|value| value.trim().to_string())
                    .collect();
            } else if let Some(style) = line.strip_prefix("Style:") {
                let fields = split_by_format(style, &style_format);
                styles.push(EstiloAss {
                    nombre: fields.get("Name").cloned().unwrap_or_default(),
                    fontname: fields
                        .get("Fontname")
                        .cloned()
                        .unwrap_or_else(|| "Arial".into()),
                    fontsize: number_or(fields.get("Fontsize"), 48.0),
                    primary_colour: fields
                        .get("PrimaryColour")
                        .cloned()
                        .unwrap_or_else(|| "&H00FFFFFF".into()),
                    outline_colour: fields
                        .get("OutlineColour")
                        .cloned()
                        .unwrap_or_else(|| "&H00000000".into()),
                    outline: number_or(fields.get("Outline"), 2.0),
                    shadow: number_or(fields.get("Shadow"), 0.0),
                    alignment: number_or(fields.get("Alignment"), 2.0) as i32,
                    margin_l: number_or(fields.get("MarginL"), 10.0) as i32,
                    margin_r: number_or(fields.get("MarginR"), 10.0) as i32,
                    margin_v: number_or(fields.get("MarginV"), 40.0) as i32,
                    bold: number_or(fields.get("Bold"), 0.0) != 0.0,
                    italic: number_or(fields.get("Italic"), 0.0) != 0.0,
                });
            }
        } else if section == "[events]" {
            if let Some(format) = line.strip_prefix("Format:") {
                event_format = format
                    .split(',')
                    .map(|value| value.trim().to_string())
                    .collect();
            } else if let Some(dialogue) = line.strip_prefix("Dialogue:") {
                let event = split_by_format(dialogue, &event_format);
                events.push(event);
                last_event = Some(events.len() - 1);
            } else if let Some(index) = last_event
                && !known.iter().any(|prefix| line.starts_with(prefix))
            {
                // La continuación se pega al evento YA guardado. Guardar un
                // clon aparte dejaba la continuación en el clon y el evento
                // real sin el salto: el `Dialogue` multilínea de Aegisub
                // perdía sus líneas 2..n.
                let Some(event) = events.get_mut(index) else {
                    continue;
                };
                let text = event.entry("Text".into()).or_default();
                if !text.is_empty() {
                    text.push('\n');
                }
                text.push_str(line);
            }
        }
    }

    let mut names_by_style = HashMap::new();
    for event in &events {
        let style = event.get("Style").cloned().unwrap_or_default();
        if !style.is_empty() && style != "Default" {
            names_by_style
                .entry(style)
                .or_insert_with(|| event.get("Name").cloned().unwrap_or_default());
        }
    }

    let mut speakers = Vec::new();
    let mut known_speakers = HashSet::new();
    for style in &styles {
        if style.nombre == "Default" || style.nombre.is_empty() {
            continue;
        }
        known_speakers.insert(style.nombre.clone());
        let name = names_by_style
            .get(&style.nombre)
            .filter(|name| !name.is_empty())
            .cloned()
            .unwrap_or_else(|| style.nombre.clone());
        speakers.push(Hablante {
            id: style.nombre.clone(),
            nombre: name,
            tecla: String::new(),
            color: ass_color_to_hex(&style.primary_colour),
            preset_id: None,
        });
    }
    for style in names_by_style.keys() {
        if known_speakers.insert(style.clone()) {
            speakers.push(Hablante {
                id: style.clone(),
                nombre: names_by_style
                    .get(style)
                    .filter(|name| !name.is_empty())
                    .cloned()
                    .unwrap_or_else(|| style.clone()),
                tecla: String::new(),
                color: "#FFFFFF".into(),
                preset_id: None,
            });
        }
    }

    let captions = events
        .iter()
        .enumerate()
        .map(|(index, event)| {
            let style = event.get("Style").cloned().unwrap_or_default();
            Caption {
                id: format!("cap-{}", index + 1),
                inicio: parse_ass_time(event.get("Start").map_or("", String::as_str)),
                fin: parse_ass_time(event.get("End").map_or("", String::as_str)),
                texto: unescape_ass_text(event.get("Text").map_or("", String::as_str)),
                hablante_id: (style != "Default" && !style.is_empty()).then_some(style),
            }
        })
        .collect();

    Ok(AssParseResult {
        captions,
        hablantes: speakers,
        styles,
    })
}

pub fn preset_desde_estilos(styles: &[EstiloAss], nombre: &str, id: &str) -> Option<PresetAss> {
    let base = styles.iter().find(|style| style.nombre == "Default")?;
    Some(PresetAss {
        id: id.into(),
        nombre: nombre.into(),
        fontname: base.fontname.clone(),
        fontsize: base.fontsize,
        color: ass_color_to_hex(&base.primary_colour),
        outline_color: ass_color_to_hex(&base.outline_colour),
        outline: base.outline,
        shadow: base.shadow,
        alignment: base.alignment,
        margin_l: base.margin_l,
        margin_r: base.margin_r,
        margin_v: base.margin_v,
        bold: base.bold,
        italic: base.italic,
    })
}

pub fn preset_para_exportar(
    hablantes: &[Hablante],
    presets: &HashMap<String, PresetAss>,
    asignacion: &HashMap<String, String>,
    hablante_id: Option<&str>,
    sin_hablante: &PresetAss,
) -> PresetAss {
    let Some(hablante_id) = hablante_id else {
        return sin_hablante.clone();
    };
    let speaker = hablantes.iter().find(|speaker| speaker.id == hablante_id);
    if let Some(id) = asignacion.get(hablante_id)
        && let Some(preset) = presets.get(id)
    {
        return preset.clone();
    }
    if let Some(id) = speaker.and_then(|speaker| speaker.preset_id.as_ref())
        && let Some(preset) = presets.get(id)
    {
        return preset.clone();
    }
    sin_hablante.clone()
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineaOverlay {
    pub id: String,
    pub texto: String,
    pub preset: PresetAss,
}

pub fn lineas_overlay_activas(
    captions: &[Caption],
    hablantes: &[Hablante],
    presets: &[PresetAss],
) -> Vec<LineaOverlay> {
    let Some(base) = presets.first() else {
        return Vec::new();
    };
    let by_id: HashMap<_, _> = presets
        .iter()
        .map(|preset| (preset.id.as_str(), preset))
        .collect();
    let mut active = captions.to_vec();
    active.sort_by(|left, right| {
        left.inicio
            .total_cmp(&right.inicio)
            .then_with(|| left.fin.total_cmp(&right.fin))
    });
    active
        .into_iter()
        .map(|caption| {
            let preset = caption
                .hablante_id
                .as_deref()
                .and_then(|id| hablantes.iter().find(|speaker| speaker.id == id))
                .and_then(|speaker| speaker.preset_id.as_deref())
                .and_then(|id| by_id.get(id))
                .copied()
                .unwrap_or(base)
                .clone();
            LineaOverlay {
                id: caption.id,
                texto: caption.texto,
                preset,
            }
        })
        .collect()
}

pub fn hex_to_ass_color_with_alpha(hex: &str, alpha: &str) -> String {
    let clean = hex.trim().strip_prefix('#').unwrap_or(hex.trim());
    if clean.len() != 6 || !clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return format!("&H{alpha}000000").to_uppercase();
    }

    let red = &clean[0..2];
    let green = &clean[2..4];
    let blue = &clean[4..6];
    format!("&H{alpha}{blue}{green}{red}").to_uppercase()
}

pub fn ass_color_to_hex(color: &str) -> String {
    let clean = color.trim().trim_end_matches('&');
    let Some(hex) = clean
        .strip_prefix("&H")
        .or_else(|| clean.strip_prefix("&h"))
    else {
        return "#FFFFFF".into();
    };
    let (blue, green, red) = match hex.len() {
        6 => (&hex[0..2], &hex[2..4], &hex[4..6]),
        8 => (&hex[2..4], &hex[4..6], &hex[6..8]),
        _ => return "#FFFFFF".into(),
    };
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return "#FFFFFF".into();
    }
    format!("#{red}{green}{blue}").to_uppercase()
}

pub fn format_ass_time(seconds: f64) -> String {
    let seconds = if seconds.is_finite() && seconds >= 0.0 {
        seconds
    } else {
        0.0
    };
    let total_centiseconds = (seconds * 100.0).round() as u64;
    let centiseconds = total_centiseconds % 100;
    let total_seconds = total_centiseconds / 100;
    let whole_seconds = total_seconds % 60;
    let minutes = (total_seconds / 60) % 60;
    let hours = total_seconds / 3600;
    format!("{hours}:{minutes:02}:{whole_seconds:02}.{centiseconds:02}")
}

pub fn parse_ass_time(input: &str) -> f64 {
    let trimmed = input.trim();
    let Some((hours, rest)) = trimmed.split_once(':') else {
        return 0.0;
    };
    let Some((minutes, rest)) = rest.split_once(':') else {
        return 0.0;
    };
    let Some((seconds, centiseconds)) = rest.split_once(['.', ',']) else {
        return 0.0;
    };
    let (Some(hours), Some(minutes), Some(seconds), Some(centiseconds)) = (
        parse_u64(hours),
        parse_u64(minutes),
        parse_u64(seconds),
        parse_u64(centiseconds),
    ) else {
        return 0.0;
    };
    hours as f64 * 3600.0 + minutes as f64 * 60.0 + seconds as f64 + centiseconds as f64 / 100.0
}

fn parse_u64(value: &str) -> Option<u64> {
    value.parse().ok()
}

pub fn escape_ass_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace("\r\n", "\\N")
        .replace(['\r', '\n'], "\\N")
}

pub fn unescape_ass_text(text: &str) -> String {
    let mut without_overrides = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{'
            && (i == 0 || chars[i - 1] != '\\')
            && let Some(end) = chars[i + 1..].iter().position(|&c| c == '}')
        {
            i += end + 2;
            continue;
        }
        without_overrides.push(chars[i]);
        i += 1;
    }

    without_overrides
        .replace("\\{", "{")
        .replace("\\}", "}")
        .replace("\\\\", "\\")
        .replace("\\N", "\n")
        .replace("\\n", "\n")
}

pub fn segmentar_por_solape(captions: &[Caption]) -> Vec<SegmentoAss> {
    let mut sorted: Vec<_> = captions
        .iter()
        .filter(|caption| caption.fin > caption.inicio)
        .cloned()
        .collect();
    sorted.sort_by(|left, right| {
        left.inicio
            .total_cmp(&right.inicio)
            .then_with(|| left.fin.total_cmp(&right.fin))
    });

    let mut segments = Vec::new();
    let mut active = Vec::new();
    let mut next_index = 0;
    let mut time = sorted.first().map_or(0.0, |caption| caption.inicio);

    while next_index < sorted.len() || !active.is_empty() {
        let next_start = sorted
            .get(next_index)
            .map_or(f64::INFINITY, |caption| caption.inicio);
        let next_end = active
            .iter()
            .map(|caption: &Caption| caption.fin)
            .fold(f64::INFINITY, f64::min);
        let next_time = next_start.min(next_end);

        if !active.is_empty() {
            segments.push(SegmentoAss {
                inicio: time,
                fin: next_time,
                captions: active.clone(),
            });
        }

        active.retain(|caption| caption.fin > next_time);
        while let Some(caption) = sorted.get(next_index) {
            if caption.inicio > next_time {
                break;
            }
            if caption.fin > next_time {
                active.push(caption.clone());
            }
            next_index += 1;
        }
        time = next_time;
    }

    segments
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{Caption, Hablante, PresetAss, build_ass, parse_ass};

    use super::{
        ASS_EVENTS_FORMAT, ASS_STYLE_FORMAT, ass_color_to_hex, escape_ass_text, format_ass_time,
        hex_to_ass_color, lineas_overlay_activas, parse_ass_time, preset_desde_estilos,
        preset_para_exportar, preset_por_defecto, sanitize_nombre_dialogo, segmentar_por_solape,
        unescape_ass_text,
    };

    // Los casos de este módulo vienen de `src/utils/__tests__/ass.test.ts`, uno
    // por `it`; el `describe` del oráculo se respeta como agrupación.

    fn hablante(id: &str, nombre: &str, tecla: &str, color: &str) -> Hablante {
        Hablante {
            id: id.into(),
            nombre: nombre.into(),
            tecla: tecla.into(),
            color: color.into(),
            preset_id: None,
        }
    }

    fn caption(id: &str, inicio: f64, fin: f64, texto: &str, hablante_id: Option<&str>) -> Caption {
        Caption {
            id: id.into(),
            inicio,
            fin,
            texto: texto.into(),
            hablante_id: hablante_id.map(str::to_string),
        }
    }

    // --- bold/italic en el .ass -------------------------------------------
    // El peso va en los campos Bold/Italic del `Style:`, nunca pegado al
    // Fontname. Tras "Style: H1," los campos son los de ASS_STYLE_FORMAT, así
    // que Bold es el índice 6 y Italic el 7.

    fn campos_del_estilo_h1(preset: &PresetAss) -> Vec<String> {
        let hablantes = vec![hablante("sp1", "Juan", "1", "#E85D4E")];
        let captions = vec![caption("a", 0.0, 3.0, "hola", Some("sp1"))];
        let ass = build_ass(&captions, &hablantes, |_| preset.clone(), 1920, 1080);
        let linea = ass
            .lines()
            .find(|line| line.starts_with("Style: H1,"))
            .expect("el export tiene un estilo H1");
        let campos = &linea["Style: H1,".len()..];
        campos.split(',').map(str::to_string).collect()
    }

    #[test]
    fn a_preset_without_bold_writes_zero_in_both_fields() {
        let campos = campos_del_estilo_h1(&preset_por_defecto());
        assert_eq!(campos[6], "0");
        assert_eq!(campos[7], "0");
    }

    #[test]
    fn bold_writes_minus_one_in_the_bold_field() {
        let preset = PresetAss {
            bold: true,
            ..preset_por_defecto()
        };
        let campos = campos_del_estilo_h1(&preset);
        assert_eq!(campos[6], "-1");
        assert_eq!(campos[7], "0");
    }

    #[test]
    fn italic_writes_minus_one_in_the_italic_field() {
        let preset = PresetAss {
            italic: true,
            ..preset_por_defecto()
        };
        let campos = campos_del_estilo_h1(&preset);
        assert_eq!(campos[6], "0");
        assert_eq!(campos[7], "-1");
    }

    #[test]
    fn bold_and_italic_write_both_fields() {
        let preset = PresetAss {
            bold: true,
            italic: true,
            ..preset_por_defecto()
        };
        let campos = campos_del_estilo_h1(&preset);
        assert_eq!(campos[6], "-1");
        assert_eq!(campos[7], "-1");
    }

    fn fuente_con_estilo(bold: i32, italic: i32) -> String {
        format!(
            "[Script Info]\nScriptType: v4.00+\n\n[V4+ Styles]\nFormat: {ASS_STYLE_FORMAT}\n\
             Style: Default,Arial,48,&H00FFFFFF,&H00FFFFFF,&H00000000,&H00000000,{bold},{italic},0,0,\
             100,100,0,0,1,2,0,2,10,10,40,1\n\n[Events]\nFormat: {ASS_EVENTS_FORMAT}\n\
             Dialogue: 0,0:00:00.00,0:00:03.00,Default,,0,0,0,,hola\n"
        )
    }

    #[test]
    fn parsing_recovers_bold_from_an_imported_style() {
        let estilos = parse_ass(&fuente_con_estilo(-1, -1))
            .expect("ass válido")
            .styles;
        assert!(estilos[0].bold);
        assert!(estilos[0].italic);
    }

    #[test]
    fn parsing_leaves_bold_and_italic_false_when_the_style_has_zero() {
        let estilos = parse_ass(&fuente_con_estilo(0, 0))
            .expect("ass válido")
            .styles;
        assert!(!estilos[0].bold);
        assert!(!estilos[0].italic);
    }

    // --- presetParaExportar ----------------------------------------------

    fn preset(id: &str, nombre: &str) -> PresetAss {
        PresetAss {
            id: id.into(),
            nombre: nombre.into(),
            ..preset_por_defecto()
        }
    }

    fn export_map() -> HashMap<String, PresetAss> {
        let mut map = HashMap::new();
        map.insert("p1".to_string(), preset("p1", "Uno"));
        map.insert("p2".to_string(), preset("p2", "Dos"));
        map
    }

    fn hablantes_con_preset() -> Vec<Hablante> {
        vec![
            Hablante {
                preset_id: Some("p2".into()),
                ..hablante("sp1", "Juan", "1", "#E85D4E")
            },
            hablante("sp2", "María", "2", "#4EA8E8"),
        ]
    }

    fn asignacion(pares: &[(&str, &str)]) -> HashMap<String, String> {
        pares
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn export_preset_prefers_what_the_modal_chose() {
        let elegido = preset_para_exportar(
            &hablantes_con_preset(),
            &export_map(),
            &asignacion(&[("sp1", "p1")]),
            Some("sp1"),
            &preset("p1", "Uno"),
        );
        assert_eq!(elegido.id, "p1");
    }

    #[test]
    fn export_preset_falls_back_to_the_saved_preset_id() {
        let elegido = preset_para_exportar(
            &hablantes_con_preset(),
            &export_map(),
            &asignacion(&[]),
            Some("sp1"),
            &preset("p1", "Uno"),
        );
        assert_eq!(elegido.id, "p2");
    }

    #[test]
    fn export_preset_falls_back_to_the_unassigned_preset() {
        let elegido = preset_para_exportar(
            &hablantes_con_preset(),
            &export_map(),
            &asignacion(&[]),
            Some("sp2"),
            &preset("p1", "Uno"),
        );
        assert_eq!(elegido.id, "p1");
    }

    #[test]
    fn export_preset_uses_the_unassigned_preset_for_captions_without_a_speaker() {
        let elegido = preset_para_exportar(
            &hablantes_con_preset(),
            &export_map(),
            &asignacion(&[("sp1", "p2")]),
            None,
            &preset("p1", "Uno"),
        );
        assert_eq!(elegido.id, "p1");
    }

    #[test]
    fn export_preset_ignores_ids_that_no_longer_exist() {
        // El preset elegido en el modal ya no está: cae al `presetId` guardado.
        let elegido = preset_para_exportar(
            &hablantes_con_preset(),
            &export_map(),
            &asignacion(&[("sp1", "borrado")]),
            Some("sp1"),
            &preset("p1", "Uno"),
        );
        assert_eq!(elegido.id, "p2");

        // Y si tampoco existe el guardado, cae a sinHablante.
        let huerfano = vec![Hablante {
            preset_id: Some("borrado".into()),
            ..hablante("sp1", "Juan", "1", "#E85D4E")
        }];
        let elegido = preset_para_exportar(
            &huerfano,
            &export_map(),
            &asignacion(&[]),
            Some("sp1"),
            &preset("p1", "Uno"),
        );
        assert_eq!(elegido.id, "p1");
    }

    // --- hexToAssColor ---------------------------------------------------

    #[test]
    fn converts_hex_to_inverted_bgr_with_opaque_alpha() {
        assert_eq!(hex_to_ass_color("#FFFFFF"), "&H00FFFFFF");
        assert_eq!(hex_to_ass_color("#E85D4E"), "&H004E5DE8");
    }

    #[test]
    fn respects_the_received_alpha() {
        assert_eq!(
            super::hex_to_ass_color_with_alpha("#000000", "80"),
            "&H80000000"
        );
    }

    #[test]
    fn falls_back_to_black_on_an_invalid_hex() {
        assert_eq!(hex_to_ass_color("nope"), "&H00000000");
        assert_eq!(hex_to_ass_color("#FFF"), "&H00000000");
    }

    // --- assColorToHex ---------------------------------------------------

    #[test]
    fn undoes_the_hex_to_ass_conversion() {
        assert_eq!(ass_color_to_hex("&H004E5DE8"), "#E85D4E");
        assert_eq!(ass_color_to_hex("&H00FFFFFF"), "#FFFFFF");
    }

    #[test]
    fn accepts_colors_without_alpha_and_with_a_trailing_ampersand() {
        assert_eq!(ass_color_to_hex("&H4E5DE8"), "#E85D4E");
        assert_eq!(ass_color_to_hex("&H4E5DE8&"), "#E85D4E");
    }

    #[test]
    fn falls_back_to_white_on_garbage() {
        assert_eq!(ass_color_to_hex("rojo"), "#FFFFFF");
    }

    // --- formatAssTime ---------------------------------------------------

    #[test]
    fn formats_centiseconds_and_does_not_pad_the_hour() {
        assert_eq!(format_ass_time(1.5), "0:00:01.50");
        assert_eq!(format_ass_time(0.0), "0:00:00.00");
        assert_eq!(format_ass_time(3661.5), "1:01:01.50");
    }

    #[test]
    fn rolls_centiseconds_over_into_seconds() {
        assert_eq!(format_ass_time(59.999), "0:01:00.00");
        assert_eq!(format_ass_time(1.999), "0:00:02.00");
    }

    #[test]
    fn clamps_negatives_and_non_finite_values_to_zero() {
        assert_eq!(format_ass_time(-1.0), "0:00:00.00");
        assert_eq!(format_ass_time(f64::NAN), "0:00:00.00");
    }

    // --- parseAssTime ----------------------------------------------------

    #[test]
    fn parses_the_ass_format() {
        assert!((parse_ass_time("0:00:01.50") - 1.5).abs() < 1e-3);
        assert!((parse_ass_time("1:01:01.50") - 3661.5).abs() < 1e-3);
    }

    #[test]
    fn tolerates_a_comma_as_the_fraction_separator() {
        assert!((parse_ass_time("0:00:05,25") - 5.25).abs() < 1e-3);
    }

    #[test]
    fn returns_zero_on_garbage() {
        assert_eq!(parse_ass_time(""), 0.0);
        assert_eq!(parse_ass_time("foo"), 0.0);
    }

    // --- escapeAssText / unescapeAssText ---------------------------------

    #[test]
    fn turns_line_breaks_into_backslash_n_and_back() {
        assert_eq!(escape_ass_text("línea 1\nlínea 2"), "línea 1\\Nlínea 2");
        assert_eq!(unescape_ass_text("línea 1\\Nlínea 2"), "línea 1\nlínea 2");
    }

    #[test]
    fn escapes_braces_so_they_are_not_read_as_overrides() {
        assert_eq!(escape_ass_text("{una} cosa"), "\\{una\\} cosa");
        assert_eq!(unescape_ass_text("\\{una\\} cosa"), "{una} cosa");
    }

    #[test]
    fn strips_override_blocks_when_unescaping() {
        assert_eq!(unescape_ass_text("{\\pos(10,20)}texto"), "texto");
    }

    #[test]
    fn strips_consecutive_overrides_without_leaving_debris() {
        assert_eq!(unescape_ass_text("{\\an8}{\\i1}Hola"), "Hola");
    }

    #[test]
    fn keeps_plain_text_untouched() {
        assert_eq!(unescape_ass_text("texto normal"), "texto normal");
    }

    #[test]
    fn survives_the_escape_round_trip() {
        let original = "antes\ndespués {con llaves} y \\ barra";
        assert_eq!(unescape_ass_text(&escape_ass_text(original)), original);
    }

    // --- sanitizeNombreDialogo -------------------------------------------

    #[test]
    fn removes_commas_that_would_break_the_dialogue_parse() {
        // La coma es el separador de campos: se funde con los espacios que la
        // siguen en UN espacio, no en dos.
        assert_eq!(sanitize_nombre_dialogo("Smith, John"), "Smith John");
        assert_eq!(sanitize_nombre_dialogo("Smith,John"), "Smith John");
    }

    #[test]
    fn leaves_names_without_commas_untouched() {
        assert_eq!(sanitize_nombre_dialogo("Juan"), "Juan");
    }

    // --- buildAss --------------------------------------------------------
    // Un preset por hablante, como elige el modal. H1=rojo, H2=azul, null=blanco.

    fn p_rojo() -> PresetAss {
        PresetAss {
            color: "#E85D4E".into(),
            ..preset_por_defecto()
        }
    }

    fn p_azul() -> PresetAss {
        PresetAss {
            color: "#4EA8E8".into(),
            ..preset_por_defecto()
        }
    }

    fn por_hablante(id: &str) -> PresetAss {
        match id {
            "sp1" => p_rojo(),
            "sp2" => p_azul(),
            _ => preset_por_defecto(),
        }
    }

    fn hablantes_base() -> Vec<Hablante> {
        vec![
            hablante("sp1", "Juan", "1", "#E85D4E"),
            hablante("sp2", "María", "2", "#4EA8E8"),
        ]
    }

    fn cap(id: &str, inicio: f64, fin: f64) -> Caption {
        caption(id, inicio, fin, "x", None)
    }

    fn build(captions: &[Caption]) -> String {
        build_ass(captions, &hablantes_base(), por_hablante, 1920, 1080)
    }

    #[test]
    fn writes_v4_headers_with_the_requested_play_res() {
        let out = build_ass(
            &[cap("a", 0.0, 3.0)],
            &hablantes_base(),
            por_hablante,
            1280,
            720,
        );
        assert!(out.contains("ScriptType: v4.00+"));
        assert!(out.contains("PlayResX: 1280"));
        assert!(out.contains("PlayResY: 720"));
        assert!(out.contains("WrapStyle: 0"));
        assert!(out.contains(&format!("Format: {ASS_STYLE_FORMAT}")));
        assert!(out.contains(&format!("Format: {ASS_EVENTS_FORMAT}")));
    }

    #[test]
    fn the_style_format_has_23_columns_and_the_events_format_10() {
        assert_eq!(ASS_STYLE_FORMAT.split(',').count(), 23);
        assert_eq!(ASS_EVENTS_FORMAT.split(',').count(), 10);
    }

    #[test]
    fn the_color_comes_from_the_speaker_preset_not_the_palette_color() {
        let out = build(&[cap("a", 0.0, 3.0)]);
        let estilos: Vec<&str> = out
            .lines()
            .filter(|line| line.starts_with("Style:"))
            .collect();
        // Se compara contra el nombre del preset, no hardcodeado: si el default
        // cambia de fuente el test sigue valiendo.
        assert!(estilos[0].starts_with(&format!("Style: H1,{},48,&H004E5DE8,", p_rojo().fontname)));
        assert!(estilos[1].starts_with(&format!("Style: H2,{},48,&H00E8A84E,", p_azul().fontname)));
        assert!(estilos[2].starts_with(&format!(
            "Style: Default,{},48,&H00FFFFFF,",
            preset_por_defecto().fontname
        )));
    }

    #[test]
    fn one_style_per_speaker_even_when_they_share_a_preset() {
        let out = build_ass(
            &[cap("a", 0.0, 3.0)],
            &hablantes_base(),
            |_| p_rojo(),
            1920,
            1080,
        );
        assert!(out.contains(&format!("Style: H1,{},48,&H004E5DE8,", p_rojo().fontname)));
        assert!(out.contains(&format!("Style: H2,{},48,&H004E5DE8,", p_rojo().fontname)));
    }

    #[test]
    fn uses_the_cosmetic_name_in_the_name_column_and_the_style_in_the_style_column() {
        let out = build(&[caption("a", 0.0, 3.0, "x", Some("sp2"))]);
        assert!(
            out.contains("Dialogue: 0,0:00:00.00,0:00:03.00,H2,María,0,0,40,,x"),
            "{out}"
        );
    }

    #[test]
    fn captions_without_a_speaker_go_to_the_default_style() {
        assert!(build(&[cap("a", 0.0, 3.0)]).contains(",Default,,0,0,40,,x"));
    }

    #[test]
    fn falls_back_to_the_key_or_id_when_the_speaker_has_no_name() {
        let sin_nombre = vec![hablante("spX", "", "7", "#FFFFFF")];
        let out = build_ass(
            &[caption("a", 0.0, 3.0, "x", Some("spX"))],
            &sin_nombre,
            por_hablante,
            1920,
            1080,
        );
        assert!(out.contains(",H1,7,"), "{out}");
    }

    #[test]
    fn sorts_events_by_start_even_when_they_arrive_out_of_order() {
        let out = build(&[
            caption("b", 5.0, 6.0, "segundo", None),
            caption("a", 1.0, 2.0, "primero", None),
        ]);
        let pos_primero = out.find("primero").expect("primero");
        let pos_segundo = out.find("segundo").expect("segundo");
        assert!(pos_primero < pos_segundo);
    }

    #[test]
    fn escapes_commas_in_the_cosmetic_name_so_the_parse_does_not_break() {
        let con_coma = vec![hablante("sp1", "Smith, John", "1", "#FFFFFF")];
        let out = build_ass(
            &[caption("a", 0.0, 3.0, "x", Some("sp1"))],
            &con_coma,
            por_hablante,
            1920,
            1080,
        );
        assert!(out.contains(",H1,Smith John,0,0,40,,x"), "{out}");
    }

    #[test]
    fn preset_margins_go_to_the_style_and_to_the_event_margin_v() {
        let outro = PresetAss {
            outline: 5.0,
            shadow: 3.0,
            margin_v: 80,
            ..p_rojo()
        };
        let out = build_ass(
            &[cap("a", 0.0, 3.0)],
            &hablantes_base(),
            |_| outro.clone(),
            1920,
            1080,
        );
        assert!(out.contains(",1,5,3,2,10,10,80,1"), "{out}");
        assert!(out.contains(",0,0,80,,"), "{out}");
    }

    #[test]
    fn a_single_speaker_emits_no_override() {
        let out = build(&[
            caption("a", 0.0, 3.0, "hola", Some("sp1")),
            caption("b", 1.0, 2.0, "otro", Some("sp1")),
        ]);
        // El segundo caption está contenido en el primero, así que el segmento
        // los fusiona en un solo evento.
        assert!(out.contains(",H1,Juan,0,0,40,,hola\\N"), "{out}");
    }

    #[test]
    fn merges_two_speakers_into_one_event_with_a_color_override() {
        let out = build(&[
            caption("a", 0.0, 6.0, "linea de Juan", Some("sp1")),
            caption("b", 3.0, 9.0, "linea de Maria", Some("sp2")),
        ]);
        assert!(
            out.contains(
                "Dialogue: 0,0:00:03.00,0:00:06.00,H1,Juan,0,0,40,,linea de Juan\\N{\\c&H00E8A84E&}linea de Maria"
            ),
            "{out}"
        );
    }

    #[test]
    fn emits_fn_and_fs_when_the_second_preset_changes_the_typography() {
        let cine = PresetAss {
            fontname: "Space Grotesk".into(),
            fontsize: 64.0,
            ..p_azul()
        };
        let con_cine = |id: &str| {
            if id == "sp2" { cine.clone() } else { p_rojo() }
        };
        let out = build_ass(
            &[
                caption("a", 0.0, 6.0, "juan", Some("sp1")),
                caption("b", 3.0, 9.0, "maria", Some("sp2")),
            ],
            &hablantes_base(),
            con_cine,
            1920,
            1080,
        );
        assert!(
            out.contains("juan\\N{\\fnSpace Grotesk\\fs64\\c&H00E8A84E&}maria"),
            "{out}"
        );
    }

    // --- fusionarLineas --------------------------------------------------

    fn fusionar(
        captions: &[Caption],
        preset_de: impl Fn(&Caption) -> PresetAss,
        base: &PresetAss,
    ) -> String {
        crate::fusionar_lineas(captions, preset_de, base)
    }

    #[test]
    fn joins_with_backslash_n_and_emits_no_overrides_when_every_line_uses_the_base() {
        let base = preset_por_defecto();
        let texto = fusionar(
            &[
                caption("a", 0.0, 3.0, "uno", None),
                caption("b", 0.0, 3.0, "dos", None),
            ],
            |_| base.clone(),
            &base,
        );
        assert_eq!(texto, "uno\\Ndos");
    }

    #[test]
    fn emits_only_the_color_that_changes() {
        let base = preset_por_defecto();
        let texto = fusionar(
            &[
                caption("a", 0.0, 3.0, "uno", None),
                caption("b", 0.0, 3.0, "dos", None),
            ],
            |c| {
                if c.id == "a" { base.clone() } else { p_rojo() }
            },
            &base,
        );
        assert_eq!(texto, "uno\\N{\\c&H004E5DE8&}dos");
    }

    #[test]
    fn emits_every_field_that_differs_from_the_base() {
        let base = preset_por_defecto();
        let cine = PresetAss {
            fontname: "Space Grotesk".into(),
            fontsize: 64.0,
            outline: 4.0,
            ..p_azul()
        };
        let texto = fusionar(
            &[
                caption("a", 0.0, 3.0, "uno", None),
                caption("b", 0.0, 3.0, "dos", None),
            ],
            |c| {
                if c.id == "a" {
                    base.clone()
                } else {
                    cine.clone()
                }
            },
            &base,
        );
        assert_eq!(
            texto,
            "uno\\N{\\fnSpace Grotesk\\fs64\\c&H00E8A84E&\\bord4}dos"
        );
    }

    // Bold/Italic viven SOLO en el Style: del evento. En un evento fusionado
    // todas las líneas comparten ese Style:, así que una línea con otro preset
    // perdía su negrita/cursiva (quedaba la del base). Igual que \c, \b y \i
    // tienen que viajar en el override de la línea que difiere.
    #[test]
    fn emits_b_and_i_when_the_line_differs_in_weight_or_italic() {
        let base = preset_por_defecto();
        let negrita = PresetAss {
            bold: true,
            italic: true,
            ..base.clone()
        };
        let texto = fusionar(
            &[
                caption("a", 0.0, 3.0, "uno", None),
                caption("b", 0.0, 3.0, "dos", None),
            ],
            |c| {
                if c.id == "a" {
                    base.clone()
                } else {
                    negrita.clone()
                }
            },
            &base,
        );
        assert_eq!(texto, "uno\\N{\\b1\\i1}dos");
    }

    #[test]
    fn turns_off_the_base_b_and_i_on_the_line_that_does_not_have_them() {
        let negrita = PresetAss {
            bold: true,
            italic: true,
            ..preset_por_defecto()
        };
        let texto = fusionar(
            &[
                caption("a", 0.0, 3.0, "uno", None),
                caption("b", 0.0, 3.0, "dos", None),
            ],
            |c| {
                if c.id == "a" {
                    negrita.clone()
                } else {
                    preset_por_defecto()
                }
            },
            &negrita,
        );
        assert_eq!(texto, "uno\\N{\\b0\\i0}dos");
    }

    #[test]
    fn emits_no_override_for_the_first_line_when_it_uses_the_base() {
        let base = preset_por_defecto();
        let texto = fusionar(
            &[
                caption("a", 0.0, 3.0, "uno", None),
                caption("b", 0.0, 3.0, "dos", None),
            ],
            |c| {
                if c.id == "b" { p_rojo() } else { base.clone() }
            },
            &base,
        );
        assert!(texto.starts_with("uno\\N"), "{texto}");
    }

    #[test]
    fn escapes_each_lines_text() {
        let base = preset_por_defecto();
        let texto = fusionar(
            &[
                caption("a", 0.0, 3.0, "l1\nl2", None),
                caption("b", 0.0, 3.0, "{x}", None),
            ],
            |_| base.clone(),
            &base,
        );
        assert_eq!(texto, "l1\\Nl2\\N\\{x\\}");
    }

    // --- parseAss --------------------------------------------------------

    /// El fixture `ASS_MUESTRA` del oráculo: dos estilos (H1 y Default) y tres
    /// eventos, uno de ellos con un `Dialogue` de dos líneas como los escribe
    /// Aegisub y con llaves escapadas dentro del texto.
    fn ass_muestra() -> String {
        format!(
            "[Script Info]\n\
             ScriptType: v4.00+\n\
             PlayResX: 1920\n\
             PlayResY: 1080\n\
             WrapStyle: 0\n\
             \n\
             [V4+ Styles]\n\
             Format: {ASS_STYLE_FORMAT}\n\
             Style: H1,Inter,48,&H004E5DE8,&H004E5DE8,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,1,2,10,10,40,1\n\
             Style: Default,Inter,60,&H00FFFFFF,&H00FFFFFF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,3,2,2,10,10,40,1\n\
             \n\
             [Events]\n\
             Format: {ASS_EVENTS_FORMAT}\n\
             Dialogue: 0,0:00:01.00,0:00:03.50,H1,Juan,0,0,40,,Hola mundo\n\
             Dialogue: 0,0:00:04.00,0:00:06.00,Default,,0,0,40,,Sin hablante\n\
             Dialogue: 0,0:00:05.00,0:00:07.00,H1,Juan,0,0,104,,Con \\{{llaves\\}} y\n\
             salto\n"
        )
    }

    #[test]
    fn rejects_legacy_ssa_script_type_v4_00() {
        assert!(parse_ass("[Script Info]\nScriptType: v4.00\n").is_err());
    }

    #[test]
    fn rejects_a_file_without_script_info() {
        assert!(parse_ass("").is_err());
    }

    #[test]
    fn reads_captions_with_their_times() {
        let r = parse_ass(&ass_muestra()).expect("ass válido");
        assert_eq!(r.captions.len(), 3);
        assert!((r.captions[0].inicio - 1.0).abs() < 0.01);
        assert!((r.captions[0].fin - 3.5).abs() < 0.01);
        assert_eq!(r.captions[0].texto, "Hola mundo");
        assert_eq!(r.captions[2].texto, "Con {llaves} y\nsalto");
    }

    #[test]
    fn creates_one_speaker_per_style_using_the_name_column() {
        let r = parse_ass(&ass_muestra()).expect("ass válido");
        assert_eq!(r.hablantes.len(), 1);
        assert_eq!(r.hablantes[0].id, "H1");
        assert_eq!(r.hablantes[0].nombre, "Juan");
        assert_eq!(r.hablantes[0].color, "#E85D4E");
    }

    #[test]
    fn the_default_style_does_not_become_a_speaker() {
        let r = parse_ass(&ass_muestra()).expect("ass válido");
        assert_eq!(r.captions[1].hablante_id, None);
    }

    #[test]
    fn maps_the_columns_by_format_name_not_by_position() {
        // Format y valores intercambiados juntos: un parser posicional leería
        // "Juan" como estilo y "H1" como nombre. Solo uno que siga el Format
        // acierta.
        let raro = ass_muestra().replace(
            &format!(
                "Format: {ASS_EVENTS_FORMAT}\n\
                 Dialogue: 0,0:00:01.00,0:00:03.50,H1,Juan,0,0,40,,Hola mundo"
            ),
            "Format: Layer, Start, End, Name, Style, MarginL, MarginR, MarginV, Effect, Text\n\
             Dialogue: 0,0:00:01.00,0:00:03.50,Juan,H1,0,0,40,,Hola mundo",
        );
        let r = parse_ass(&raro).expect("ass válido");
        assert_eq!(r.captions[0].hablante_id.as_deref(), Some("H1"));
        assert_eq!(r.hablantes[0].nombre, "Juan");
    }

    #[test]
    fn reads_a_multi_line_dialogue_as_written_by_aegisub() {
        let r = parse_ass(&ass_muestra()).expect("ass válido");
        assert_eq!(r.captions.len(), 3);
        assert_eq!(r.captions[2].texto, "Con {llaves} y\nsalto");
    }

    #[test]
    fn does_not_swallow_a_comment_as_a_continuation() {
        let con_comment = ass_muestra().replace(
            "Dialogue: 0,0:00:04.00,0:00:06.00,Default,,0,0,40,,Sin hablante",
            "Dialogue: 0,0:00:04.00,0:00:06.00,Default,,0,0,40,,Sin hablante\n\
             Comment: 0,0:00:04.00,0:00:06.00,Default,,0,0,40,,soy un comentario",
        );
        let r = parse_ass(&con_comment).expect("ass válido");
        assert_eq!(r.captions.len(), 3);
        assert_eq!(r.captions[1].texto, "Sin hablante");
    }

    // --- segmentarPorSolape ----------------------------------------------

    fn resumen_segmentos(segments: &[super::SegmentoAss]) -> Vec<(f64, f64, Vec<String>)> {
        segments
            .iter()
            .map(|segment| {
                (
                    segment.inicio,
                    segment.fin,
                    segment
                        .captions
                        .iter()
                        .map(|caption| caption.id.clone())
                        .collect(),
                )
            })
            .collect()
    }

    fn ids(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|id| id.to_string()).collect()
    }

    #[test]
    fn without_overlaps_returns_one_segment_per_caption() {
        let s = segmentar_por_solape(&[cap("a", 0.0, 3.0), cap("b", 5.0, 8.0)]);
        assert_eq!(
            resumen_segmentos(&s),
            vec![(0.0, 3.0, ids(&["a"])), (5.0, 8.0, ids(&["b"]))]
        );
    }

    #[test]
    fn splits_whenever_the_active_set_changes() {
        // No se puede poner la línea de B antes de que B hable, ni quitarla
        // antes de que termine: el set de activos cambia de verdad en t=2 y 5.
        let s = segmentar_por_solape(&[cap("a", 0.0, 5.0), cap("b", 2.0, 7.0)]);
        assert_eq!(
            resumen_segmentos(&s),
            vec![
                (0.0, 2.0, ids(&["a"])),
                (2.0, 5.0, ids(&["a", "b"])),
                (5.0, 7.0, ids(&["b"])),
            ]
        );
    }

    #[test]
    fn a_caption_contained_in_another_produces_a_span_with_both_lines() {
        let s = segmentar_por_solape(&[cap("a", 0.0, 10.0), cap("b", 3.0, 5.0)]);
        assert_eq!(
            resumen_segmentos(&s),
            vec![
                (0.0, 3.0, ids(&["a"])),
                (3.0, 5.0, ids(&["a", "b"])),
                (5.0, 10.0, ids(&["a"])),
            ]
        );
    }

    #[test]
    fn cuts_when_the_active_set_changes_without_leaving_text_hanging() {
        // A=[0,10] B=[5,15] C=[12,20]: A y C NO se solapan. Una agrupación en
        // cadena daría un solo evento [0,20] con las tres y dejaría el texto de
        // A 10 s después de que terminó.
        let s = segmentar_por_solape(&[
            cap("a", 0.0, 10.0),
            cap("b", 5.0, 15.0),
            cap("c", 12.0, 20.0),
        ]);
        assert_eq!(
            resumen_segmentos(&s),
            vec![
                (0.0, 5.0, ids(&["a"])),
                (5.0, 10.0, ids(&["a", "b"])),
                (10.0, 12.0, ids(&["b"])),
                (12.0, 15.0, ids(&["b", "c"])),
                (15.0, 20.0, ids(&["c"])),
            ]
        );
    }

    #[test]
    fn does_not_merge_captions_that_merely_touch() {
        let s = segmentar_por_solape(&[cap("a", 0.0, 3.0), cap("b", 3.0, 6.0)]);
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn sorts_the_lines_by_start_inside_the_segment() {
        let s = segmentar_por_solape(&[cap("b", 4.0, 9.0), cap("a", 0.0, 6.0)]);
        let con_ambas = s
            .iter()
            .find(|segment| segment.captions.len() == 2)
            .expect("hay un segmento con las dos");
        let orden: Vec<&str> = con_ambas
            .captions
            .iter()
            .map(|caption| caption.id.as_str())
            .collect();
        assert_eq!(orden, vec!["a", "b"]);
    }

    #[test]
    fn the_same_overlapping_speaker_produces_two_lines_in_the_common_span() {
        let s = segmentar_por_solape(&[
            caption("a", 0.0, 5.0, "x", Some("sp1")),
            caption("b", 2.0, 7.0, "x", Some("sp1")),
        ]);
        let comun = s
            .iter()
            .find(|segment| segment.captions.len() == 2)
            .expect("hay un tramo común");
        assert_eq!(comun.inicio, 2.0);
        assert_eq!(comun.fin, 5.0);
    }

    #[test]
    fn discards_zero_duration_captions() {
        let s = segmentar_por_solape(&[cap("a", 0.0, 0.0), cap("b", 1.0, 3.0)]);
        assert_eq!(resumen_segmentos(&s), vec![(1.0, 3.0, ids(&["b"]))]);
    }

    #[test]
    fn survives_empty_input_and_a_single_caption() {
        assert!(segmentar_por_solape(&[]).is_empty());
        assert_eq!(segmentar_por_solape(&[cap("a", 2.0, 3.0)]).len(), 1);
    }

    // --- round-trip ------------------------------------------------------

    #[test]
    fn reimporting_an_exported_ass_returns_the_same_project() {
        let captions = vec![
            caption("c1", 1.0, 4.0, "Hola mundo", Some("sp1")),
            caption("c2", 5.0, 8.0, "Segunda línea", Some("sp2")),
            caption("c3", 9.0, 12.0, "Sin hablante", None),
        ];
        let r = parse_ass(&build(&captions)).expect("ass válido");

        let textos: Vec<&str> = r.captions.iter().map(|c| c.texto.as_str()).collect();
        assert_eq!(textos, vec!["Hola mundo", "Segunda línea", "Sin hablante"]);
        assert!((r.captions[0].inicio - 1.0).abs() < 0.01);
        assert!((r.captions[2].fin - 12.0).abs() < 0.01);
        let hablantes: Vec<Option<&str>> = r
            .captions
            .iter()
            .map(|c| c.hablante_id.as_deref())
            .collect();
        assert_eq!(hablantes, vec![Some("H1"), Some("H2"), None]);
        let nombres: Vec<&str> = r.hablantes.iter().map(|h| h.nombre.as_str()).collect();
        assert_eq!(nombres, vec!["Juan", "María"]);
        let colores: Vec<&str> = r.hablantes.iter().map(|h| h.color.as_str()).collect();
        assert_eq!(colores, vec!["#E85D4E", "#4EA8E8"]);
    }

    #[test]
    fn re_exporting_what_was_imported_is_stable_h1_stays() {
        let captions = vec![caption("c1", 1.0, 4.0, "Hola", Some("sp1"))];
        let primera = parse_ass(&build(&captions)).expect("ass válido");
        let segunda = build_ass(
            &primera.captions,
            &primera.hablantes,
            por_hablante,
            1920,
            1080,
        );
        let segunda = parse_ass(&segunda).expect("ass válido");
        // buildAss emite un Style por cada hablante del array, usado o no, así
        // que el import reconstruye los dos. H2 no tiene eventos: su nombre cae
        // al nombre del estilo.
        let ids_estilo: Vec<&str> = segunda.hablantes.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids_estilo, vec!["H1", "H2"]);
        assert_eq!(segunda.hablantes[0].nombre, "Juan");
        assert_eq!(segunda.captions[0].texto, "Hola");
        assert_eq!(segunda.captions[0].hablante_id.as_deref(), Some("H1"));
    }

    #[test]
    fn keeps_h1_as_juan_even_when_maria_speaks_first() {
        // El orden es el del array de hablantes, no el de los eventos.
        let captions = vec![
            caption("c1", 1.0, 2.0, "yo soy maría", Some("sp2")),
            caption("c2", 3.0, 4.0, "yo soy juan", Some("sp1")),
        ];
        let r = parse_ass(&build(&captions)).expect("ass válido");
        let ids_estilo: Vec<&str> = r.hablantes.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids_estilo, vec!["H1", "H2"]);
        assert_eq!(r.hablantes[0].nombre, "Juan");
        assert_eq!(r.hablantes[1].nombre, "María");
    }

    #[test]
    fn a_merged_event_comes_back_as_one_caption_with_the_lines_without_the_overrides() {
        // Limitación aceptada: `unescape_ass_text` descarta los bloques {...},
        // así que el color por línea no sobrevive al round-trip. No es info
        // corrupta, es info que no está.
        let captions = vec![
            caption("c1", 0.0, 6.0, "juan", Some("sp1")),
            caption("c2", 3.0, 9.0, "maria", Some("sp2")),
        ];
        let r = parse_ass(&build(&captions)).expect("ass válido");
        let con_dos_lineas: Vec<&Caption> = r
            .captions
            .iter()
            .filter(|c| c.texto.contains('\n'))
            .collect();
        assert_eq!(con_dos_lineas.len(), 1);
        assert_eq!(con_dos_lineas[0].texto, "juan\nmaria");
        assert!(!con_dos_lineas[0].texto.contains('{'));
    }

    // --- presetDesdeEstilos ----------------------------------------------

    #[test]
    fn builds_the_preset_from_the_default_style() {
        let r = parse_ass(&ass_muestra()).expect("ass válido");
        let p = preset_desde_estilos(&r.styles, "Mi estilo", "preset-x").expect("hay Default");
        assert_eq!(p.color, "#FFFFFF");
        assert_eq!(p.fontsize, 60.0);
        assert_eq!(p.outline, 3.0);
        assert_eq!(p.shadow, 2.0);
        assert_eq!(p.nombre, "Mi estilo");
    }

    #[test]
    fn returns_none_when_the_file_has_no_default_style() {
        let sin_default = ass_muestra().replace("Style: Default", "Style: Base");
        let r = parse_ass(&sin_default).expect("ass válido");
        assert_eq!(preset_desde_estilos(&r.styles, "x", "y"), None);
    }

    // --- lineasOverlayActivas --------------------------------------------

    fn preset_uno() -> PresetAss {
        preset("p1", "Uno")
    }

    fn preset_dos() -> PresetAss {
        preset("p2", "Dos")
    }

    fn hablantes_del_overlay() -> Vec<Hablante> {
        vec![
            Hablante {
                preset_id: Some("p2".into()),
                ..hablante("sp1", "Juan", "1", "#E85D4E")
            },
            hablante("sp2", "María", "2", "#4EA8E8"),
        ]
    }

    #[test]
    fn overlay_returns_one_line_per_caption_with_its_resolved_preset() {
        let out = lineas_overlay_activas(
            &[
                caption("a", 0.0, 2.0, "Hola", Some("sp1")),
                caption("b", 1.0, 3.0, "Chau", Some("sp2")),
            ],
            &hablantes_del_overlay(),
            &[preset_uno(), preset_dos()],
        );
        let ids: Vec<&str> = out.iter().map(|l| l.id.as_str()).collect();
        assert_eq!(ids, vec!["a", "b"]);
        assert_eq!(out[0].preset.id, "p2");
        assert_eq!(out[1].preset.id, "p1");
        assert_eq!(out[0].texto, "Hola");
    }

    #[test]
    fn overlay_sorts_by_start_like_the_export() {
        let out = lineas_overlay_activas(
            &[
                caption("b", 5.0, 6.0, "x", Some("sp2")),
                caption("a", 0.0, 2.0, "x", Some("sp1")),
            ],
            &hablantes_del_overlay(),
            &[preset_uno(), preset_dos()],
        );
        let ids: Vec<&str> = out.iter().map(|l| l.id.as_str()).collect();
        assert_eq!(ids, vec!["a", "b"]);
    }

    #[test]
    fn overlay_without_a_speaker_uses_the_first_preset() {
        let out = lineas_overlay_activas(
            &[caption("a", 0.0, 2.0, "x", None)],
            &hablantes_del_overlay(),
            &[preset_uno(), preset_dos()],
        );
        assert_eq!(out[0].preset.id, "p1");
    }

    #[test]
    fn overlay_falls_back_to_the_first_preset_when_the_preset_id_is_gone() {
        let huerfano = vec![Hablante {
            preset_id: Some("muerto".into()),
            ..hablante("sp1", "Juan", "1", "#E85D4E")
        }];
        let out = lineas_overlay_activas(
            &[caption("a", 0.0, 2.0, "x", Some("sp1"))],
            &huerfano,
            &[preset_uno(), preset_dos()],
        );
        assert_eq!(out[0].preset.id, "p1");
    }

    #[test]
    fn overlay_without_presets_returns_nothing() {
        let out = lineas_overlay_activas(
            &[caption("a", 0.0, 2.0, "x", Some("sp1"))],
            &hablantes_del_overlay(),
            &[],
        );
        assert!(out.is_empty());
    }

    // --- build y parse, ida y vuelta mínima ------------------------------

    #[test]
    fn builds_and_parses_styles_speakers_and_dialogues() {
        let preset = PresetAss {
            id: "p".into(),
            nombre: "Default".into(),
            fontname: "Inter".into(),
            fontsize: 48.0,
            color: "#E85D4E".into(),
            outline_color: "#000000".into(),
            outline: 2.0,
            shadow: 1.0,
            alignment: 2,
            margin_l: 10,
            margin_r: 10,
            margin_v: 40,
            bold: false,
            italic: false,
        };
        let speakers = vec![hablante("sp1", "Juan", "1", "#FFFFFF")];
        let captions = vec![caption("c1", 1.0, 3.5, "Hola {mundo}", Some("sp1"))];

        let ass = build_ass(&captions, &speakers, |_| preset.clone(), 1920, 1080);
        assert!(ass.contains("PlayResX: 1920"));
        assert!(ass.contains("Style: H1,Inter,48"));
        assert!(ass.contains("Dialogue: 0,0:00:01.00,0:00:03.50,H1,Juan"));

        let parsed = parse_ass(&ass).expect("generated ASS parses");
        assert_eq!(parsed.hablantes[0].nombre, "Juan");
        assert_eq!(parsed.captions[0].texto, "Hola {mundo}");
        assert_eq!(parsed.captions[0].hablante_id.as_deref(), Some("H1"));
    }
}
