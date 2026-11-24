//! Picos de audio para la pista de waveform. Se decodifica una sola vez con
//! symphonia, sin ffmpeg, y se publica por canal desde un hilo propio.

use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};

use symphonia::core::audio::AudioBufferRef;
use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Picos por segundo.
pub const POR_SEGUNDO: u32 = 100;

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorWaveform(String);

impl fmt::Display for ErrorWaveform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ErrorWaveform {}

#[derive(Debug, Clone)]
pub struct Picos {
    pub por_segundo: u32,
    /// Valor absoluto máximo por ventana de 10 ms.
    pub max: Vec<f32>,
    /// RMS por ventana de 10 ms.
    pub rms: Vec<f32>,
}

/// Decodifica la primera pista de audio y acumula máximo y RMS por décima de
/// segundo. Un archivo sin audio, roto o con códec no soportado da error.
pub fn analizar(ruta: &Path) -> Result<Picos, ErrorWaveform> {
    let archivo = File::open(ruta).map_err(|e| ErrorWaveform(e.to_string()))?;
    let fuente = MediaSourceStream::new(Box::new(archivo), Default::default());
    let mut hint = Hint::new();
    if let Some(extension) = ruta.extension().and_then(|e| e.to_str()) {
        hint.with_extension(extension);
    }
    let sondeo = symphonia::default::get_probe()
        .format(
            &hint,
            fuente,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| ErrorWaveform(e.to_string()))?;
    let mut formato = sondeo.format;
    let (id, tasa, parametros) = {
        let pista_audio = formato
            .tracks()
            .iter()
            .find(|pista| pista.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| ErrorWaveform("sin pista de audio".into()))?;
        let tasa = pista_audio
            .codec_params
            .sample_rate
            .ok_or_else(|| ErrorWaveform("sin frecuencia de muestreo".into()))?;
        (pista_audio.id, tasa, pista_audio.codec_params.clone())
    };
    let mut decodificador = symphonia::default::get_codecs()
        .make(&parametros, &DecoderOptions::default())
        .map_err(|e| ErrorWaveform(e.to_string()))?;

    let por_ventana = (tasa / POR_SEGUNDO).max(1) as u64;
    let mut max: Vec<f32> = Vec::new();
    let mut sumas: Vec<f64> = Vec::new();
    let mut cuentas: Vec<u64> = Vec::new();
    let mut indice: u64 = 0;

    while let Ok(paquete) = formato.next_packet() {
        if paquete.track_id() != id {
            continue;
        }
        let decodificado = match decodificador.decode(&paquete) {
            Ok(buffer) => buffer,
            Err(_) => continue,
        };
        let muestras: Vec<Vec<f32>> = match &decodificado {
            AudioBufferRef::F32(buffer) => buffer
                .planes()
                .planes()
                .iter()
                .map(|plano| plano.as_ref().to_vec())
                .collect(),
            AudioBufferRef::S16(buffer) => buffer
                .planes()
                .planes()
                .iter()
                .map(|plano| plano.as_ref().iter().map(|v| *v as f32 / 32768.0).collect())
                .collect(),
            AudioBufferRef::S32(buffer) => buffer
                .planes()
                .planes()
                .iter()
                .map(|plano| {
                    plano
                        .as_ref()
                        .iter()
                        .map(|v| *v as f32 / 2147483648.0)
                        .collect()
                })
                .collect(),
            _ => continue,
        };
        for plano in &muestras {
            for &valor in plano {
                let ventana = (indice / por_ventana) as usize;
                while max.len() <= ventana {
                    max.push(0.0);
                    sumas.push(0.0);
                    cuentas.push(0);
                }
                let abs = valor.abs();
                if abs > max[ventana] {
                    max[ventana] = abs;
                }
                sumas[ventana] += valor as f64 * valor as f64;
                cuentas[ventana] += 1;
                indice += 1;
            }
        }
    }

    if max.is_empty() {
        return Err(ErrorWaveform("sin muestras de audio".into()));
    }
    let rms = sumas
        .iter()
        .zip(cuentas.iter())
        .map(|(suma, cuenta)| {
            if *cuenta == 0 {
                0.0
            } else {
                (*suma / *cuenta as f64).sqrt() as f32
            }
        })
        .collect();
    Ok(Picos {
        por_segundo: POR_SEGUNDO,
        max,
        rms,
    })
}

/// Lanza el análisis en un hilo y devuelve el canal de resultados.
pub fn analizar_en_hilo(ruta: PathBuf) -> Receiver<Result<Picos, ErrorWaveform>> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let _ = tx.send(analizar(&ruta));
    });
    rx
}
