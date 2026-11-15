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

    fn silencio(seconds: f64) -> Vec<f64> {
        vec![0.005; (seconds * 15.0).round() as usize]
    }

    fn pico(seconds: f64) -> Vec<f64> {
        vec![0.2; (seconds * 15.0).round() as usize]
    }

    #[test]
    fn finds_an_island_from_silence_and_snaps_inside_speech() {
        let volume = [silencio(1.0), pico(3.0), silencio(2.0)].concat();
        let before = buscar_tramo_audio(&volume, 0.3).expect("speech ahead");
        assert!((before.inicio - 0.3).abs() < 0.01);
        assert!((before.fin - 4.25).abs() < 0.1);

        let inside = buscar_tramo_audio(&volume, 1.4).expect("speech active");
        assert!((inside.inicio - 1.0).abs() < 0.1);
    }

    #[test]
    fn applies_snap_limit_hysteresis_and_duration_cap() {
        let volume = [silencio(2.0), pico(1.0), silencio(0.3), pico(10.0)].concat();
        let tramo = buscar_tramo_audio(&volume, 2.0).expect("speech active");
        assert!(tramo.inicio >= 1.45);
        assert!(tramo.fin >= 5.2);

        let long = [silencio(1.0), pico(10.0)].concat();
        let tramo = buscar_tramo_audio(&long, 0.5).expect("speech ahead");
        assert!(tramo.fin <= 6.0);
    }

    #[test]
    fn returns_none_without_audio_and_aggregates_range_peaks() {
        assert_eq!(buscar_tramo_audio(&[], 1.0), None);
        let volume: Vec<f64> = (0..15).map(|index| index as f64 / 100.0).collect();
        assert!((pico_en_rango(&volume, 0.2, 0.4) - 0.05).abs() < 1e-6);
        assert_eq!(pico_en_rango(&volume, 5.0, 6.0), 0.0);
        assert!((pico_en_rango(&volume, 0.47, 0.475) - 0.07).abs() < 1e-6);
    }
}
