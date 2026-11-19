use std::collections::HashMap;
use std::path::Path;

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

/// Fallo al leer o escribir un proyecto. Se distinguen los dos casos porque el
/// mensaje que ve el usuario no es el mismo: un disco lleno o un JSON de otra
/// versión son problemas distintos.
#[derive(Debug)]
pub enum ErrorProyecto {
    Io(std::io::Error),
    Formato(serde_json::Error),
}

impl std::fmt::Display for ErrorProyecto {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "no se pudo leer o escribir el archivo: {error}"),
            Self::Formato(error) => write!(formatter, "el proyecto no es un JSON válido: {error}"),
        }
    }
}

impl std::error::Error for ErrorProyecto {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Formato(error) => Some(error),
        }
    }
}

impl From<serde_json::Error> for ErrorProyecto {
    fn from(error: serde_json::Error) -> Self {
        Self::Formato(error)
    }
}

/// Guarda el `.cdp` como lo hace el oráculo: escribe un temporal y lo renombra
/// encima. Si el proceso muere a media escritura, el archivo viejo sigue
/// entero; un `.cdp` truncado es peor que uno ausente.
pub fn guardar_proyecto(ruta: &Path, proyecto: &Proyecto) -> Result<(), ErrorProyecto> {
    let texto = rod_core::a_json(proyecto)?;
    let temporal = ruta.with_extension("tmp");
    std::fs::write(&temporal, texto).map_err(ErrorProyecto::Io)?;
    std::fs::rename(&temporal, ruta).map_err(|error| {
        // El temporal no debe quedar tirado si el rename falla.
        let _ = std::fs::remove_file(&temporal);
        ErrorProyecto::Io(error)
    })
}

pub fn abrir_proyecto(ruta: &Path) -> Result<Proyecto, ErrorProyecto> {
    let texto = std::fs::read_to_string(ruta).map_err(ErrorProyecto::Io)?;
    Ok(rod_core::desde_json(&texto)?)
}

#[cfg(test)]
mod tests {
    use super::{FormatoSubtitulo, abrir_proyecto, cargar, exportar_srt, guardar_proyecto};
    use rod_core::{Caption, Hablante, Proyecto};
    use std::path::PathBuf;

    /// El `.cdp` de prueba, con todo lo que el formato tiene que conservar.
    fn proyecto() -> Proyecto {
        Proyecto {
            ruta_video: "C:/videos/clip.mp4".into(),
            hablantes: vec![Hablante {
                id: "sp-1".into(),
                nombre: "Ana".into(),
                tecla: "1".into(),
                color: "#3B82F6".into(),
                preset_id: Some("p-1".into()),
            }],
            captions: vec![Caption {
                id: "cap-1".into(),
                inicio: 1.0,
                fin: 2.5,
                texto: "Hola".into(),
                hablante_id: Some("sp-1".into()),
            }],
            playhead: 7.25,
        }
    }

    /// Un nombre de archivo propio por test: los tests corren en paralelo y
    /// `temp_dir` es compartido.
    fn ruta_temporal(nombre: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "rod-proyecto-{}-{}.cdp",
            std::process::id(),
            nombre
        ))
    }

    #[test]
    fn guardar_y_abrir_devuelve_el_mismo_proyecto() {
        let ruta = ruta_temporal("ida-vuelta");
        let original = proyecto();
        guardar_proyecto(&ruta, &original).expect("guarda");
        let vuelta = abrir_proyecto(&ruta).expect("abre");
        assert_eq!(vuelta, original);
        // El temporal no queda tirado al lado del proyecto.
        assert!(!ruta.with_extension("tmp").exists());
        std::fs::remove_file(&ruta).expect("limpia");
    }

    #[test]
    fn guardar_dos_veces_reemplaza_el_archivo() {
        let ruta = ruta_temporal("reemplazo");
        let mut original = proyecto();
        guardar_proyecto(&ruta, &original).expect("primera escritura");
        original.captions[0].texto = "Adiós".into();
        guardar_proyecto(&ruta, &original).expect("segunda escritura");
        assert_eq!(
            abrir_proyecto(&ruta).expect("abre").captions[0].texto,
            "Adiós"
        );
        std::fs::remove_file(&ruta).expect("limpia");
    }

    #[test]
    fn un_archivo_que_no_es_un_proyecto_es_un_error_de_formato() {
        let ruta = ruta_temporal("roto");
        std::fs::write(&ruta, "esto no es json").expect("escribe basura");
        let error = abrir_proyecto(&ruta).expect_err("no debería abrir");
        assert!(matches!(error, super::ErrorProyecto::Formato(_)));
        std::fs::remove_file(&ruta).expect("limpia");
    }

    #[test]
    fn un_archivo_que_no_existe_es_un_error_de_io() {
        let ruta = ruta_temporal("inexistente");
        let error = abrir_proyecto(&ruta).expect_err("no debería abrir");
        assert!(matches!(error, super::ErrorProyecto::Io(_)));
    }

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
