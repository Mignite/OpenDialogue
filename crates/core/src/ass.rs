use crate::Caption;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct Hablante {
    pub id: String,
    pub nombre: String,
    pub tecla: String,
    pub color: String,
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

pub fn sanitize_nombre_dialogo(name: &str) -> String {
    name.replace(',', " ").trim().to_string()
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
    let mut last_event: Option<HashMap<String, String>> = None;
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
                last_event = Some(event.clone());
                events.push(event);
            } else if let Some(event) = last_event.as_mut()
                && !known.iter().any(|prefix| line.starts_with(prefix))
            {
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
    use crate::{Caption, Hablante, PresetAss, build_ass, parse_ass};

    use super::{
        ass_color_to_hex, escape_ass_text, format_ass_time, hex_to_ass_color, parse_ass_time,
        unescape_ass_text,
    };

    #[test]
    fn converts_ass_colors_in_bgr_order() {
        assert_eq!(hex_to_ass_color("#FFFFFF"), "&H00FFFFFF");
        assert_eq!(hex_to_ass_color("#E85D4E"), "&H004E5DE8");
        assert_eq!(hex_to_ass_color("nope"), "&H00000000");
        assert_eq!(
            super::hex_to_ass_color_with_alpha("#000000", "80"),
            "&H80000000"
        );
        assert_eq!(ass_color_to_hex("&H004E5DE8"), "#E85D4E");
        assert_eq!(ass_color_to_hex("&H4E5DE8&"), "#E85D4E");
        assert_eq!(ass_color_to_hex("rojo"), "#FFFFFF");
    }

    #[test]
    fn formats_and_parses_centisecond_times() {
        assert_eq!(format_ass_time(1.5), "0:00:01.50");
        assert_eq!(format_ass_time(59.999), "0:01:00.00");
        assert_eq!(format_ass_time(-1.0), "0:00:00.00");
        assert_eq!(parse_ass_time("1:01:01.50"), 3661.5);
        assert_eq!(parse_ass_time("0:00:05,25"), 5.25);
        assert_eq!(parse_ass_time("foo"), 0.0);
    }

    #[test]
    fn escapes_lines_and_literal_ass_syntax() {
        let original = "línea {literal}\\uno\nlínea 2";
        assert_eq!(
            escape_ass_text(original),
            "línea \\{literal\\}\\\\uno\\Nlínea 2"
        );
        assert_eq!(unescape_ass_text(&escape_ass_text(original)), original);
        assert_eq!(unescape_ass_text("{\\pos(10,20)}texto"), "texto");
        assert_eq!(unescape_ass_text("{\\an8}{\\i1}Hola"), "Hola");
    }

    #[test]
    fn segments_exactly_when_the_active_caption_set_changes() {
        let caption = |id: &str, inicio: f64, fin: f64| Caption {
            id: id.into(),
            inicio,
            fin,
            texto: id.into(),
            hablante_id: None,
        };
        let segments = super::segmentar_por_solape(&[
            caption("a", 0.0, 10.0),
            caption("b", 5.0, 15.0),
            caption("c", 12.0, 20.0),
        ]);
        let summary: Vec<_> = segments
            .iter()
            .map(|segment| {
                (
                    segment.inicio,
                    segment.fin,
                    segment
                        .captions
                        .iter()
                        .map(|caption| caption.id.as_str())
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        assert_eq!(
            summary,
            vec![
                (0.0, 5.0, vec!["a"]),
                (5.0, 10.0, vec!["a", "b"]),
                (10.0, 12.0, vec!["b"]),
                (12.0, 15.0, vec!["b", "c"]),
                (15.0, 20.0, vec!["c"]),
            ]
        );
    }

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
        let speakers = vec![Hablante {
            id: "sp1".into(),
            nombre: "Juan".into(),
            tecla: "1".into(),
            color: "#FFFFFF".into(),
            preset_id: None,
        }];
        let captions = vec![Caption {
            id: "c1".into(),
            inicio: 1.0,
            fin: 3.5,
            texto: "Hola {mundo}".into(),
            hablante_id: Some("sp1".into()),
        }];

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
