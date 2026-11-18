use std::collections::HashMap;

use rod_core::{
    AssParseResult, Caption, EstiloAss, Hablante, PresetAss, Proyecto, build_ass, build_srt,
    parse_ass, parse_srt,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatoSubtitulo {
    Srt,
    Ass,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Documento {
    pub proyecto: Proyecto,
    pub formato: FormatoSubtitulo,
    pub estilos_ass: Vec<EstiloAss>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorIo {
    AssInvalido(String),
}

impl std::fmt::Display for ErrorIo {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssInvalido(error) => write!(formatter, "ASS invalido: {error}"),
        }
    }
}

impl std::error::Error for ErrorIo {}

pub fn cargar(texto: &str, ruta_video: impl Into<String>) -> Result<Documento, ErrorIo> {
    if texto.contains("[V4+ Styles]") || texto.contains("[Events]") {
        let AssParseResult {
            captions,
            hablantes,
            styles,
        } = parse_ass(texto).map_err(ErrorIo::AssInvalido)?;
        return Ok(Documento {
            proyecto: Proyecto {
                ruta_video: ruta_video.into(),
                hablantes,
                captions,
                playhead: 0.0,
            },
            formato: FormatoSubtitulo::Ass,
            estilos_ass: styles,
        });
    }

    Ok(Documento {
        proyecto: Proyecto {
            ruta_video: ruta_video.into(),
            hablantes: Vec::new(),
            captions: parse_srt(texto),
            playhead: 0.0,
        },
        formato: FormatoSubtitulo::Srt,
        estilos_ass: Vec::new(),
    })
}

pub fn exportar_srt(documento: &Documento) -> String {
    build_srt(&documento.proyecto.captions)
}

pub fn exportar_ass(
    documento: &Documento,
    presets: &HashMap<String, PresetAss>,
    sin_hablante: &PresetAss,
    resolution: (u32, u32),
) -> String {
    let preset_for = |speaker_id: &str| {
        if speaker_id.is_empty() {
            return sin_hablante.clone();
        }
        documento
            .proyecto
            .hablantes
            .iter()
            .find(|speaker| speaker.id == speaker_id)
            .and_then(|speaker| speaker.preset_id.as_ref())
            .and_then(|preset_id| presets.get(preset_id))
            .cloned()
            .unwrap_or_else(|| sin_hablante.clone())
    };
    build_ass(
        &documento.proyecto.captions,
        &documento.proyecto.hablantes,
        preset_for,
        resolution.0,
        resolution.1,
    )
}

pub fn captions(documento: &Documento) -> &[Caption] {
    &documento.proyecto.captions
}

pub fn hablantes(documento: &Documento) -> &[Hablante] {
    &documento.proyecto.hablantes
}

#[cfg(test)]
mod tests {
    use super::{FormatoSubtitulo, cargar, exportar_srt};

    #[test]
    fn loads_and_exports_srt_without_changing_caption_content() {
        let document =
            cargar("1\n00:00:01,000 --> 00:00:02,500\nHola\n", "video.mp4").expect("SRT loads");
        assert_eq!(document.formato, FormatoSubtitulo::Srt);
        assert_eq!(document.proyecto.ruta_video, "video.mp4");
        assert_eq!(document.proyecto.captions[0].texto, "Hola");
        assert!(exportar_srt(&document).contains("00:00:01,000 --> 00:00:02,500"));
    }

    #[test]
    fn loads_ass_and_preserves_styles_and_speakers() {
        let ass = "[Script Info]\nScriptType: v4.00+\n\n[V4+ Styles]\n\
                   Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n\
                   Style: H1,Arial,48,&H00FFFFFF,&H00FFFFFF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,40,1\n\
                   Style: Default,Arial,48,&H00FFFFFF,&H00FFFFFF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,40,1\n\
                   \n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:02.00,H1,Ana,0,0,40,,Hola";
        let document = cargar(ass, "video.mp4").expect("ASS loads");
        assert_eq!(document.formato, FormatoSubtitulo::Ass);
        assert_eq!(document.proyecto.hablantes[0].nombre, "Ana");
        assert_eq!(document.estilos_ass.len(), 2);
    }
}
