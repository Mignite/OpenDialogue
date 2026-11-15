use crate::{Caption, Hablante, format_time};
use std::collections::HashSet;

pub const DURACION_MINIMA: f64 = 0.1;
pub const SNAP_THRESHOLD: f64 = 0.15;

#[derive(Debug, Clone, PartialEq)]
pub struct OverlapEntry {
    pub inicio: f64,
    pub fin: f64,
    pub hablante_a: String,
    pub texto_a: String,
    pub hablante_b: String,
    pub texto_b: String,
}

pub fn clamp_caption_tiempos(
    inicio_pedido: f64,
    fin_pedido: f64,
    duracion_video: f64,
    actual: Option<&Caption>,
) -> (f64, f64) {
    let original_inicio = actual.map_or(0.0, |caption| caption.inicio);
    let original_fin = actual.map_or(0.0, |caption| caption.fin);
    let duracion_original = DURACION_MINIMA.max(original_fin - original_inicio);
    let mut inicio = if inicio_pedido.is_finite() {
        inicio_pedido
    } else {
        original_inicio
    }
    .max(0.0);
    let mut fin = if fin_pedido.is_finite() {
        fin_pedido
    } else {
        original_fin
    };

    if inicio >= fin {
        fin = inicio
            + if actual.is_some() {
                duracion_original
            } else {
                DURACION_MINIMA
            };
    }
    if duracion_video > 0.0 && fin > duracion_video {
        fin = duracion_video;
        if inicio >= fin {
            inicio = (fin - DURACION_MINIMA).max(0.0);
        }
    }
    if fin - inicio < DURACION_MINIMA - 1e-9 {
        fin = inicio + DURACION_MINIMA;
        if duracion_video > 0.0 && fin > duracion_video {
            inicio = (duracion_video - DURACION_MINIMA).max(0.0);
            fin = duracion_video;
        }
    }
    (inicio, fin)
}

pub fn find_snap_time(
    time: f64,
    excluded_ids: &[&str],
    captions: &[Caption],
    extra_targets: &[f64],
) -> Option<f64> {
    let excluded: HashSet<_> = excluded_ids.iter().copied().collect();
    let mut best = None;
    let mut best_distance = SNAP_THRESHOLD;
    let mut consider = |target: f64| {
        let distance = (time - target).abs();
        if distance < best_distance {
            best_distance = distance;
            best = Some(target);
        }
    };
    for caption in captions {
        if excluded.contains(caption.id.as_str()) {
            continue;
        }
        consider(caption.inicio);
        consider(caption.fin);
    }
    for &target in extra_targets {
        consider(target);
    }
    best
}

pub fn clamp_delta_bloque(delta: f64, earliest_start: f64) -> f64 {
    delta.max(-earliest_start)
}

pub fn build_overlap_report(captions: &[Caption], speakers: &[Hablante]) -> Vec<OverlapEntry> {
    let name_for = |id: Option<&str>| {
        id.and_then(|id| speakers.iter().find(|speaker| speaker.id == id))
            .map(|speaker| {
                if speaker.nombre.is_empty() {
                    if speaker.tecla.is_empty() {
                        speaker.id.clone()
                    } else {
                        speaker.tecla.clone()
                    }
                } else {
                    speaker.nombre.clone()
                }
            })
            .unwrap_or_else(|| "Sin hablante".into())
    };
    let mut ordered = captions.to_vec();
    ordered.sort_by(|left, right| left.inicio.total_cmp(&right.inicio));
    let mut result = Vec::new();
    for (index, left) in ordered.iter().enumerate() {
        for right in ordered.iter().skip(index + 1) {
            if right.inicio >= left.fin {
                break;
            }
            if left.hablante_id == right.hablante_id {
                continue;
            }
            result.push(OverlapEntry {
                inicio: left.inicio.max(right.inicio),
                fin: left.fin.min(right.fin),
                hablante_a: name_for(left.hablante_id.as_deref()),
                texto_a: left.texto.clone(),
                hablante_b: name_for(right.hablante_id.as_deref()),
                texto_b: right.texto.clone(),
            });
        }
    }
    result
}

pub fn format_overlap_report(entries: &[OverlapEntry]) -> String {
    if entries.is_empty() {
        return "Sin solapes".into();
    }
    entries
        .iter()
        .map(|entry| {
            format!(
                "{} -> {}  |  {} <-> {}\n  \"{}\"\n  \"{}\"\n",
                format_time(entry.inicio),
                format_time(entry.fin),
                entry.hablante_a,
                entry.hablante_b,
                entry.texto_a,
                entry.texto_b
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::{build_overlap_report, clamp_caption_tiempos, find_snap_time};
    use crate::{Caption, Hablante};

    fn caption(id: &str, start: f64, end: f64, speaker: Option<&str>) -> Caption {
        Caption {
            id: id.into(),
            inicio: start,
            fin: end,
            texto: id.into(),
            hablante_id: speaker.map(str::to_string),
        }
    }

    #[test]
    fn clamps_invalid_times_without_inverting_a_caption() {
        let current = caption("a", 2.0, 3.0, None);
        assert_eq!(
            clamp_caption_tiempos(4.0, 1.0, 10.0, Some(&current)),
            (4.0, 5.0)
        );
        assert_eq!(
            clamp_caption_tiempos(f64::NAN, f64::NAN, 4.0, Some(&current)),
            (2.0, 3.0)
        );
        assert_eq!(clamp_caption_tiempos(9.9, 10.0, 10.0, None), (9.9, 10.0));
    }

    #[test]
    fn snaps_to_nearest_non_excluded_boundary() {
        let captions = vec![caption("a", 1.0, 2.0, None), caption("b", 4.0, 5.0, None)];
        assert_eq!(find_snap_time(2.1, &["a"], &captions, &[]), None);
        assert_eq!(find_snap_time(3.9, &[], &captions, &[]), Some(4.0));
        assert_eq!(find_snap_time(8.0, &[], &captions, &[]), None);
    }

    #[test]
    fn reports_only_cross_speaker_overlaps() {
        let speakers = vec![
            Hablante {
                id: "a".into(),
                nombre: "Ana".into(),
                tecla: "1".into(),
                color: "#fff".into(),
                preset_id: None,
            },
            Hablante {
                id: "b".into(),
                nombre: "Beto".into(),
                tecla: "2".into(),
                color: "#fff".into(),
                preset_id: None,
            },
        ];
        let entries = build_overlap_report(
            &[
                caption("one", 0.0, 3.0, Some("a")),
                caption("two", 2.0, 4.0, Some("b")),
                caption("same", 1.0, 2.0, Some("a")),
            ],
            &speakers,
        );
        assert_eq!(entries.len(), 1);
        assert_eq!((entries[0].inicio, entries[0].fin), (2.0, 3.0));
    }
}
