use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
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
        let Some(time_index) = lines.iter().position(|line| line.contains("-->")) else {
            continue;
        };
        if time_index + 1 >= lines.len() {
            continue;
        }

        let Some((start, end)) = lines[time_index].split_once("-->") else {
            continue;
        };
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

#[cfg(test)]
mod tests {
    use super::{Caption, build_srt, format_srt_timestamp, parse_srt, parse_srt_time};

    #[test]
    fn parses_srt_times_and_decimal_fractions() {
        assert!((parse_srt_time("00:01:30,500") - 90.5).abs() < 0.001);
        assert!((parse_srt_time("00:00:05.250") - 5.25).abs() < 0.001);
        assert!((parse_srt_time("00:00:01,5") - 1.5).abs() < 0.001);
        assert!((parse_srt_time("00:00:01,50") - 1.5).abs() < 0.001);
        assert_eq!(parse_srt_time("foo"), 0.0);
    }

    #[test]
    fn formats_and_rolls_over_srt_timestamps() {
        assert_eq!(format_srt_timestamp(90.5), "00:01:30,500");
        assert_eq!(format_srt_timestamp(-1.0), "00:00:00,000");
        assert_eq!(format_srt_timestamp(1.9995), "00:00:02,000");
        assert_eq!(format_srt_timestamp(59.9996), "00:01:00,000");
        assert_eq!(format_srt_timestamp(1.999), "00:00:01,999");
    }

    #[test]
    fn parses_blocks_with_space_only_separators_and_sorts_them() {
        let input = "1\n00:00:05,000 --> 00:00:06,000\nSecond\n \n\
                     2\n00:00:01,000 --> 00:00:02,000\nFirst";
        let captions = parse_srt(input);
        assert_eq!(captions.len(), 2);
        assert_eq!(captions[0].texto, "First");
        assert_eq!(captions[1].texto, "Second");
        assert_ne!(captions[0].id, captions[1].id);
    }

    #[test]
    fn builds_and_roundtrips_caption_content() {
        let captions = vec![
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
        ];
        let rebuilt = build_srt(&captions);
        let reparsed = parse_srt(&rebuilt);
        assert_eq!(reparsed.len(), captions.len());
        for (parsed, original) in reparsed.iter().zip(&captions) {
            assert!((parsed.inicio - original.inicio).abs() < 0.001);
            assert!((parsed.fin - original.fin).abs() < 0.001);
            assert_eq!(parsed.texto, original.texto);
            assert_eq!(parsed.hablante_id, original.hablante_id);
        }
    }
}
