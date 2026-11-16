const VENTANAS_POR_SEGUNDO: f64 = 15.0;
const VENTANA_FONDO_SEG: f64 = 2.0;
const FACTOR_UMBRAL: f64 = 2.5;
const PISO: f64 = 0.01;
const HISTERESIS_MUESTRAS: usize = 8;
const MAX_DURACION: f64 = 5.0;
const COLA_SEG: f64 = 0.25;
const FONDO_PERCENTIL: f64 = 0.25;
const BUSQUEDA_ONSET_SEG: f64 = 2.0;
const LEAD_MAX_SEG: f64 = 0.3;
const SNAP_ATRAS_SEG: f64 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TramoAudio {
    pub inicio: f64,
    pub fin: f64,
}

fn umbral_local(volumen: &[f64], i0: usize) -> f64 {
    let ventana = (VENTANA_FONDO_SEG * VENTANAS_POR_SEGUNDO).round() as usize;
    let desde = i0.saturating_sub(ventana);
    let mut fondo = PISO;
    if desde < i0 {
        let mut anteriores = volumen[desde..i0].to_vec();
        anteriores.sort_by(f64::total_cmp);
        let index = ((FONDO_PERCENTIL * anteriores.len() as f64).floor() as usize)
            .min(anteriores.len() - 1);
        fondo = anteriores[index].max(PISO);
    }
    (fondo * FACTOR_UMBRAL).max(PISO)
}

pub fn buscar_tramo_audio(volumen: &[f64], playhead: f64) -> Option<TramoAudio> {
    if volumen.is_empty() {
        return None;
    }
    let i0 = (playhead * VENTANAS_POR_SEGUNDO).floor().max(0.0) as usize;
    if i0 >= volumen.len() {
        return None;
    }

    let threshold = umbral_local(volumen, i0);
    let high = |index: usize| index < volumen.len() && volumen[index] >= threshold;
    let (onset, inicio) = if high(i0) {
        let mut index = i0;
        let limit = i0.saturating_sub((SNAP_ATRAS_SEG * VENTANAS_POR_SEGUNDO).round() as usize);
        while index > limit && high(index - 1) {
            index -= 1;
        }
        (index, index as f64 / VENTANAS_POR_SEGUNDO)
    } else {
        let mut index = i0;
        let limit =
            (i0 + (BUSQUEDA_ONSET_SEG * VENTANAS_POR_SEGUNDO).round() as usize).min(volumen.len());
        while index < limit && !high(index) {
            index += 1;
        }
        if index >= limit {
            return None;
        }
        (
            index,
            playhead.min((index as f64 / VENTANAS_POR_SEGUNDO - LEAD_MAX_SEG).max(0.0)),
        )
    };

    let mut last_high = onset;
    let mut tolerance = HISTERESIS_MUESTRAS;
    for (index, value) in volumen.iter().enumerate().skip(onset + 1) {
        if *value >= threshold {
            last_high = index;
            tolerance = HISTERESIS_MUESTRAS;
        } else {
            tolerance = tolerance.saturating_sub(1);
            if tolerance == 0 {
                break;
            }
        }
    }

    let end = ((last_high + 1) as f64 / VENTANAS_POR_SEGUNDO + COLA_SEG)
        .min(onset as f64 / VENTANAS_POR_SEGUNDO + MAX_DURACION);
    Some(TramoAudio { inicio, fin: end })
}

pub fn pico_en_rango(volumen: &[f64], t0: f64, t1: f64) -> f64 {
    let start = (t0 * VENTANAS_POR_SEGUNDO).floor().max(0.0) as usize;
    let end = (t1 * VENTANAS_POR_SEGUNDO).ceil().max((start + 1) as f64) as usize;
    volumen
        .get(start..end.min(volumen.len()))
        .unwrap_or_default()
        .iter()
        .copied()
        .fold(0.0, f64::max)
}

#[cfg(test)]
mod tests {
    use super::{buscar_tramo_audio, pico_en_rango};

    // Casos portados de `__tests__/audioIslands.test.ts`, uno por `it`.

    const VPS: f64 = 15.0;

    fn silencio(segundos: f64) -> Vec<f64> {
        vec![0.005; (segundos * VPS).round() as usize]
    }

    fn pico(segundos: f64) -> Vec<f64> {
        vec![0.2; (segundos * VPS).round() as usize]
    }

    fn frase() -> Vec<f64> {
        [silencio(1.0), pico(3.0), silencio(2.0)].concat()
    }

    #[test]
    fn covers_the_whole_phrase_from_the_previous_silence() {
        // Playhead 0.7 s antes de la voz: arranca donde está, cierra en voz + 0.25.
        let tramo = buscar_tramo_audio(&frase(), 0.3).expect("hay voz adelante");
        assert!((tramo.inicio - 0.3).abs() < 0.01, "inicio {}", tramo.inicio);
        assert!((tramo.fin - 4.25).abs() < 0.1, "fin {}", tramo.fin);
    }

    #[test]
    fn leaves_at_most_300ms_of_air_before_the_onset() {
        let tramo = buscar_tramo_audio(&frase(), 0.75).expect("hay voz adelante");
        assert!((tramo.inicio - 0.7).abs() < 0.01, "inicio {}", tramo.inicio);
    }

    #[test]
    fn never_starts_after_the_playhead() {
        let tramo = buscar_tramo_audio(&frase(), 0.5).expect("hay voz adelante");
        assert!(tramo.inicio <= 0.5, "inicio {}", tramo.inicio);
    }

    #[test]
    fn snaps_back_to_the_onset_when_the_playhead_is_inside_speech() {
        let tramo = buscar_tramo_audio(&frase(), 1.4).expect("hay voz");
        assert!((tramo.inicio - 1.0).abs() < 0.1, "inicio {}", tramo.inicio);
        assert!((tramo.fin - 4.25).abs() < 0.1, "fin {}", tramo.fin);
    }

    #[test]
    fn does_not_snap_back_more_than_500ms() {
        let tramo = buscar_tramo_audio(&frase(), 2.0).expect("hay voz");
        assert!(tramo.inicio >= 1.45, "inicio {}", tramo.inicio);
        assert!(tramo.inicio <= 1.6, "inicio {}", tramo.inicio);
        assert!((tramo.fin - 4.25).abs() < 0.1, "fin {}", tramo.fin);
    }

    #[test]
    fn detects_the_island_even_when_the_recent_background_is_speech() {
        // Playhead 1.2 s dentro de la frase: la media de los 2 s previos se
        // inflaría con la voz y taparía el habla; el percentil 25 no.
        let volumen = [silencio(1.5), pico(3.0), silencio(2.0)].concat();
        let tramo = buscar_tramo_audio(&volumen, 2.7).expect("hay voz");
        assert!((tramo.fin - 4.75).abs() < 0.1, "fin {}", tramo.fin);
    }

    #[test]
    fn returns_none_without_an_onset_in_the_next_two_seconds() {
        let volumen = [silencio(1.0), pico(2.0), silencio(5.0), pico(2.0)].concat();
        assert_eq!(buscar_tramo_audio(&volumen, 4.0), None);
    }

    #[test]
    fn returns_none_for_an_empty_analysis() {
        assert_eq!(buscar_tramo_audio(&[], 1.0), None);
    }

    #[test]
    fn returns_none_when_the_playhead_is_outside_the_analysis() {
        let volumen = [silencio(1.0), pico(2.0)].concat();
        assert_eq!(buscar_tramo_audio(&volumen, 10.0), None);
    }

    #[test]
    fn a_short_fragment_measures_the_phrase_plus_the_tail() {
        // 0.53 s de habla → 0.78 s de fragmento: sin piso de duración.
        let volumen = [silencio(1.0), pico(0.5), silencio(2.0)].concat();
        let tramo = buscar_tramo_audio(&volumen, 0.5).expect("hay voz adelante");
        assert!((tramo.fin - 1.783).abs() < 0.01, "fin {}", tramo.fin);
    }

    #[test]
    fn a_single_window_blip_is_not_inflated() {
        let mut volumen = silencio(1.0);
        volumen.push(0.2);
        volumen.extend(silencio(2.0));
        let tramo = buscar_tramo_audio(&volumen, 1.0).expect("hay voz");
        assert!((tramo.inicio - 1.0).abs() < 0.01, "inicio {}", tramo.inicio);
        assert!((tramo.fin - 1.317).abs() < 0.01, "fin {}", tramo.fin);
    }

    #[test]
    fn closes_with_the_tail_after_the_last_sound() {
        let volumen = [silencio(1.0), pico(0.6), silencio(2.0)].concat();
        let tramo = buscar_tramo_audio(&volumen, 1.0).expect("hay voz");
        assert!((tramo.inicio - 1.0).abs() < 0.01, "inicio {}", tramo.inicio);
        assert!((tramo.fin - 1.85).abs() < 0.01, "fin {}", tramo.fin);
        assert!(tramo.fin - 1.6 <= 0.3, "cola {}", tramo.fin - 1.6);
    }

    #[test]
    fn a_short_phrase_does_not_leave_silence_at_the_end() {
        // Caso reportado: el mínimo de 1.5 s estiraba el fragmento 0.9 s detrás
        // de una voz de 0.6 s.
        let volumen = [silencio(2.0), pico(0.6), silencio(3.0)].concat();
        let tramo = buscar_tramo_audio(&volumen, 2.0).expect("hay voz");
        assert!((tramo.inicio - 2.0).abs() < 0.01, "inicio {}", tramo.inicio);
        assert!(tramo.fin - 2.6 <= 0.3, "cola {}", tramo.fin - 2.6);
        assert!(tramo.fin - tramo.inicio < 1.0);
    }

    #[test]
    fn never_covers_more_than_five_seconds_of_speech() {
        let volumen = [silencio(1.0), pico(10.0)].concat();
        let tramo = buscar_tramo_audio(&volumen, 0.5).expect("hay voz adelante");
        assert!(tramo.fin <= 6.0 + 1e-6, "fin {}", tramo.fin);
    }

    #[test]
    fn bridges_micro_pauses_inside_the_phrase() {
        let volumen = [
            silencio(2.0),
            pico(1.0),
            silencio(0.3),
            pico(2.0),
            silencio(2.0),
        ]
        .concat();
        let tramo = buscar_tramo_audio(&volumen, 2.0).expect("hay voz");
        assert!(tramo.fin >= 5.2, "fin {}", tramo.fin);
    }

    #[test]
    fn works_with_the_playhead_at_zero() {
        let volumen = [pico(3.0), silencio(2.0)].concat();
        let tramo = buscar_tramo_audio(&volumen, 0.0).expect("hay voz");
        assert_eq!(tramo.inicio, 0.0);
        assert!((tramo.fin - 3.25).abs() < 0.1, "fin {}", tramo.fin);
    }

    fn volumen_lineal() -> Vec<f64> {
        (0..15).map(|index| index as f64 / 100.0).collect()
    }

    #[test]
    fn peaks_over_the_intersected_windows() {
        // [0.2 s, 0.4 s) cubre las ventanas 3..6 → pico 0.05.
        let volumen = volumen_lineal();
        assert!((pico_en_rango(&volumen, 0.2, 0.4) - 0.05).abs() < 1e-6);
    }

    #[test]
    fn peak_is_zero_outside_the_audio_or_with_empty_input() {
        let volumen = volumen_lineal();
        assert_eq!(pico_en_rango(&volumen, 5.0, 6.0), 0.0);
        assert_eq!(pico_en_rango(&[], 0.0, 1.0), 0.0);
    }

    #[test]
    fn a_range_smaller_than_one_window_evaluates_that_window() {
        let volumen = volumen_lineal();
        assert!((pico_en_rango(&volumen, 0.47, 0.475) - 0.07).abs() < 1e-6);
    }
}
