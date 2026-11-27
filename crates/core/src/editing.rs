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
    // Clamp del body drag contra t=0. `earliest_start` es el inicio MÁS
    // TEMPRANO de la selección, no el del clip que se agarró: si otro clip del
    // bloque empieza antes, el clamp por lead dejaba pasar un delta que lo
    // empujaba por debajo de 0 y el guardado lo recortaba, deformando la
    // selección (el desfase interno entre clips se perdía).
    delta.max(-earliest_start)
}

/// Snap de bloque (body drag, estilo Premiere): prueba AMBOS extremos del
/// bloque ya desplazado contra los bordes de los vecinos y el playhead, y
/// devuelve el delta del imán más cercano (0 si ninguno entra en el umbral).
///
/// Con un solo clip, `t_min` es su inicio y esto equivale al snap simple.
pub fn find_snap_block_delta(
    t_min: f64,
    t_max: f64,
    excluded_ids: &[&str],
    captions: &[Caption],
    playhead: Option<f64>,
) -> f64 {
    let excluded: HashSet<_> = excluded_ids.iter().copied().collect();
    let mut best_delta = 0.0;
    let mut best_distance = SNAP_THRESHOLD;
    let mut consider = |target: f64| {
        for boundary in [t_min, t_max] {
            let distance = (boundary - target).abs();
            if distance < best_distance {
                best_distance = distance;
                best_delta = target - boundary;
            }
        }
    };
    for caption in captions {
        if excluded.contains(caption.id.as_str()) {
            continue;
        }
        consider(caption.inicio);
        consider(caption.fin);
    }
    if let Some(playhead) = playhead {
        consider(playhead);
    }
    best_delta
}

pub fn build_overlap_report(captions: &[Caption], speakers: &[Hablante]) -> Vec<OverlapEntry> {
    // Nombre → tecla → "Sin hablante", en ese orden: es lo que muestra el
    // reporte del oráculo (`captions.ts` + `es.json`). Caer al id dejaba un
    // identificador interno a la vista del usuario.
    let name_for = |id: Option<&str>| {
        let Some(speaker) = id.and_then(|id| speakers.iter().find(|speaker| speaker.id == id))
        else {
            return "Sin hablante".to_string();
        };
        if !speaker.nombre.is_empty() {
            speaker.nombre.clone()
        } else if !speaker.tecla.is_empty() {
            speaker.tecla.clone()
        } else {
            "Sin hablante".to_string()
        }
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
        // Literal de `es.json` (`overlap.noOverlaps`), con el salto final.
        return "No se encontraron subtítulos solapados.\n".into();
    }
    entries
        .iter()
        .map(|entry| {
            // Las flechas son de `captions.ts`: "→" (U+2192) entre tiempos y
            // "↔" (U+2194) entre hablantes. El reporte se compara con el del
            // oráculo, así que el texto tiene que ser idéntico.
            format!(
                "{} → {}  |  {} ↔ {}\n  \"{}\"\n  \"{}\"\n",
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
    use super::{
        build_overlap_report, clamp_caption_tiempos, clamp_delta_bloque, find_snap_block_delta,
        find_snap_time, format_overlap_report,
    };
    use crate::{Caption, Hablante};

    // Casos portados de `__tests__/captions.test.ts`, `captionTiempos.test.ts` y
    // la parte de `clampDeltaBloque` de `export-nombres.test.ts`.

    fn caption(id: &str, start: f64, end: f64, speaker: Option<&str>) -> Caption {
        Caption {
            id: id.into(),
            inicio: start,
            fin: end,
            texto: id.into(),
            hablante_id: speaker.map(str::to_string),
        }
    }

    fn tiempos(inicio: f64, fin: f64) -> (f64, f64) {
        (inicio, fin)
    }

    #[test]
    fn keeps_valid_times_untouched() {
        let base = caption("a", 10.0, 13.0, None);
        assert_eq!(
            clamp_caption_tiempos(10.0, 13.0, 0.0, Some(&base)),
            tiempos(10.0, 13.0)
        );
        // Sin duración de video conocida (0) tampoco se toca.
        assert_eq!(
            clamp_caption_tiempos(10.0, 13.0, 0.0, Some(&base)),
            tiempos(10.0, 13.0)
        );
    }

    #[test]
    fn never_leaves_a_negative_start() {
        assert_eq!(
            clamp_caption_tiempos(-5.0, 13.0, 0.0, None),
            tiempos(0.0, 13.0)
        );
    }

    #[test]
    fn keeps_the_duration_when_the_start_crosses_the_end() {
        let base = caption("a", 10.0, 13.0, None);
        let (inicio, fin) = clamp_caption_tiempos(20.0, 13.0, 0.0, Some(&base));
        assert_eq!(inicio, 20.0);
        assert!(
            (fin - inicio - 3.0).abs() < 1e-9,
            "duración {fin} - {inicio}"
        );
    }

    #[test]
    fn enforces_a_minimum_duration() {
        let base = caption("a", 10.0, 13.0, None);
        let (inicio, fin) = clamp_caption_tiempos(10.0, 10.1, 0.0, Some(&base));
        assert!(
            fin - inicio >= 0.1 - 1e-9,
            "duración {} por debajo del mínimo",
            fin - inicio
        );
    }

    #[test]
    fn respects_the_video_duration_as_the_end_cap() {
        let base = caption("a", 10.0, 13.0, None);
        let (inicio, fin) = clamp_caption_tiempos(10.0, 99.0, 60.0, Some(&base));
        assert!(fin <= 60.0);
        assert!(inicio <= fin);
    }

    #[test]
    fn discards_nan_and_keeps_the_original_time() {
        let base = caption("a", 10.0, 13.0, None);
        assert_eq!(
            clamp_caption_tiempos(f64::NAN, 13.0, 0.0, Some(&base)),
            tiempos(10.0, 13.0)
        );
        assert_eq!(
            clamp_caption_tiempos(10.0, f64::NAN, 0.0, Some(&base)),
            tiempos(10.0, 13.0)
        );
    }

    #[test]
    fn snaps_to_the_nearest_edge_inside_the_threshold() {
        // Caso 1 del oráculo: con un solo caption [0,5], 10 s no tiene imán.
        let solo_uno = vec![caption("c1", 0.0, 5.0, None)];
        assert_eq!(find_snap_time(10.0, &["c1"], &solo_uno, &[]), None);

        let captions = vec![
            caption("c1", 0.0, 5.0, None),
            caption("c2", 10.0, 15.0, None),
        ];
        assert_eq!(find_snap_time(10.1, &["c1"], &captions, &[]), Some(10.0));
        // 10.05 está a 0.05 de c2.inicio: entra.
        assert_eq!(find_snap_time(10.05, &["c1"], &captions, &[]), Some(10.0));
    }

    #[test]
    fn snap_ignores_every_excluded_id() {
        let captions = vec![
            caption("c1", 0.0, 5.0, None),
            caption("c2", 10.0, 15.0, None),
            caption("c3", 10.05, 20.0, None),
        ];
        // Excluyendo c1 y c2, el imán que queda es c3.inicio.
        assert_eq!(
            find_snap_time(10.05, &["c1", "c2"], &captions, &[]),
            Some(10.05)
        );
        assert_eq!(
            find_snap_time(10.05, &["c1", "c2", "c3"], &captions, &[]),
            None
        );
    }

    #[test]
    fn snap_accepts_extra_targets_like_the_playhead() {
        let captions = vec![caption("c1", 0.0, 5.0, None)];
        assert_eq!(
            find_snap_time(10.1, &["c1"], &captions, &[10.0]),
            Some(10.0)
        );
        assert_eq!(find_snap_time(10.1, &["c1"], &captions, &[]), None);
    }

    fn bloque() -> Vec<Caption> {
        vec![
            caption("b1", 2.0, 3.0, None),
            caption("b2", 4.0, 5.0, None),
            caption("n", 10.0, 15.0, None),
        ]
    }

    #[test]
    fn block_snap_pulls_the_start_to_the_next_edge() {
        // tMin = 9.95, a 0.05 de n.inicio = 10 → delta +0.05.
        let delta = find_snap_block_delta(9.95, 12.95, &["b1", "b2"], &bloque(), None);
        assert!((delta - 0.05).abs() < 1e-6, "delta {delta}");
    }

    #[test]
    fn block_snap_pulls_the_end_to_the_next_edge() {
        // tMax = 9.97, a 0.03 de 10; tMin = 6.97 lejos de todo.
        let delta = find_snap_block_delta(6.97, 9.97, &["b1", "b2"], &bloque(), None);
        assert!((delta - 0.03).abs() < 1e-6, "delta {delta}");
    }

    #[test]
    fn block_snap_pulls_backwards_to_the_previous_edge() {
        let captions = vec![caption("c0", 0.0, 2.0, None), caption("b1", 2.0, 5.0, None)];
        // tMin = 2.03, a 0.03 de c0.fin = 2 → delta -0.03.
        let delta = find_snap_block_delta(2.03, 5.03, &["b1"], &captions, None);
        assert!((delta + 0.03).abs() < 1e-6, "delta {delta}");
    }

    #[test]
    fn block_snap_takes_the_closest_magnet() {
        // tMin a 0.1 de 10 y tMax a 0.05: gana el fin del bloque.
        let delta = find_snap_block_delta(9.9, 9.95, &["b1", "b2"], &bloque(), None);
        assert!((delta - 0.05).abs() < 1e-6, "delta {delta}");
    }

    #[test]
    fn block_snap_returns_zero_when_nothing_is_in_range() {
        assert_eq!(
            find_snap_block_delta(20.0, 25.0, &["b1", "b2"], &bloque(), None),
            0.0
        );
    }

    #[test]
    fn block_snap_ignores_excluded_ids_and_snaps_to_the_playhead() {
        let delta = find_snap_block_delta(20.0, 25.0, &["b1", "b2"], &bloque(), Some(25.02));
        assert!((delta - 0.02).abs() < 1e-6, "delta {delta}");
        let sin_iman = find_snap_block_delta(20.0, 25.0, &["b1", "b2", "n"], &bloque(), Some(30.0));
        assert_eq!(sin_iman, 0.0);
    }

    #[test]
    fn block_clamp_uses_the_earliest_start_of_the_selection() {
        // El bug original: con el lead en 5 s y otro clip en 2 s, arrastrar a -4
        // empujaba el clip de 2 s por debajo de 0.
        assert_eq!(clamp_delta_bloque(10.0, 5.0), 10.0);
        assert_eq!(clamp_delta_bloque(-2.0, 2.0), -2.0);
        assert_eq!(clamp_delta_bloque(-4.0, 2.0), -2.0);
        assert_eq!(clamp_delta_bloque(-4.0, 5.0), -4.0);
        assert_eq!(clamp_delta_bloque(-5.0, 5.0), -5.0);
        assert_eq!(clamp_delta_bloque(-9.0, 5.0), -5.0);
        assert_eq!(clamp_delta_bloque(0.0, 0.0), 0.0);
        assert_eq!(clamp_delta_bloque(-1.0, 0.0), 0.0);
    }

    #[test]
    fn block_clamp_keeps_the_internal_offset_of_the_selection() {
        let caps = [caption("A", 5.0, 7.0, None), caption("B", 2.0, 4.0, None)];
        let t_min = caps.iter().map(|c| c.inicio).fold(f64::INFINITY, f64::min);
        assert_eq!(t_min, 2.0);

        for pedido in [-4.0, -5.0, -2.0, -1.0, 0.0, 3.0, 100.0] {
            let delta = clamp_delta_bloque(pedido, t_min);
            for cap in &caps {
                let guardado = (cap.inicio + delta).max(0.0);
                // El preview no miente: nada queda por debajo de 0.
                assert!(
                    (guardado - (cap.inicio + delta)).abs() < 1e-10,
                    "delta {delta} recorta {:?}",
                    cap.id
                );
            }
        }

        let delta = clamp_delta_bloque(-4.0, t_min);
        let nuevo_a = (caps[0].inicio + delta).max(0.0);
        let nuevo_b = (caps[1].inicio + delta).max(0.0);
        assert!((nuevo_a - nuevo_b - (caps[0].inicio - caps[1].inicio)).abs() < 1e-10);
        assert!(nuevo_b.abs() < 1e-10);
    }

    fn hablante(id: &str, nombre: &str, tecla: &str) -> Hablante {
        Hablante {
            id: id.into(),
            nombre: nombre.into(),
            tecla: tecla.into(),
            color: "#fff".into(),
            preset_id: None,
        }
    }

    #[test]
    fn reports_no_overlaps_when_captions_do_not_touch() {
        let captions = vec![
            caption("c1", 0.0, 5.0, Some("s1")),
            caption("c2", 6.0, 10.0, Some("s2")),
        ];
        assert!(build_overlap_report(&captions, &[]).is_empty());
    }

    #[test]
    fn detects_overlap_between_different_speakers() {
        let speakers = vec![hablante("s1", "Alice", "a"), hablante("s2", "Bob", "b")];
        let captions = vec![
            caption("c1", 0.0, 10.0, Some("s1")),
            caption("c2", 3.0, 7.0, Some("s2")),
        ];
        let entries = build_overlap_report(&captions, &speakers);
        assert_eq!(entries.len(), 1);
        assert_eq!((entries[0].inicio, entries[0].fin), (3.0, 7.0));
        assert_eq!(entries[0].hablante_a, "Alice");
        assert_eq!(entries[0].hablante_b, "Bob");
    }

    #[test]
    fn ignores_overlap_between_the_same_speaker() {
        let captions = vec![
            caption("c1", 0.0, 10.0, Some("s1")),
            caption("c2", 3.0, 7.0, Some("s1")),
        ];
        assert!(build_overlap_report(&captions, &[]).is_empty());
    }

    #[test]
    fn overlap_report_falls_back_from_name_to_key_to_unassigned() {
        let speakers = vec![
            hablante("s1", "", "a"),
            Hablante {
                id: "s2".into(),
                nombre: String::new(),
                tecla: String::new(),
                color: "#fff".into(),
                preset_id: None,
            },
        ];
        let captions = vec![
            caption("c1", 0.0, 10.0, Some("s1")),
            caption("c2", 3.0, 7.0, Some("s2")),
        ];
        let entries = build_overlap_report(&captions, &speakers);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].hablante_a, "a");
        assert_eq!(entries[0].hablante_b, "Sin hablante");
    }

    #[test]
    fn formats_the_overlap_report_like_the_oracle() {
        let entries = vec![super::OverlapEntry {
            inicio: 3.0,
            fin: 7.0,
            hablante_a: "Alice".into(),
            texto_a: "hola".into(),
            hablante_b: "Bob".into(),
            texto_b: "chau".into(),
        }];
        assert_eq!(
            format_overlap_report(&entries),
            "00:00:03.00 → 00:00:07.00  |  Alice ↔ Bob\n  \"hola\"\n  \"chau\"\n"
        );
        assert_eq!(
            format_overlap_report(&[]),
            "No se encontraron subtítulos solapados.\n"
        );
    }
}
