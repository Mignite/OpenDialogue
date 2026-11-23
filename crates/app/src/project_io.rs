use std::collections::HashMap;
use std::path::Path;

use rod_core::{
    AssParseResult, Caption, EstiloAss, Hablante, PresetAss, Proyecto, build_ass,
    build_overlap_report, build_srt, format_overlap_report, format_srt_timestamp,
    nombre_archivo_hablante_unico, parse_ass, parse_srt,
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

/// Lo que produce «Exportar SRT por hablante»: un archivo por hablante y el
/// reporte de solapes, más los conteos que el aviso al usuario necesita. Puro:
/// no toca el disco, así que se prueba sin él.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportacionSrt {
    /// `(nombre de archivo, contenido)`, uno por hablante con captions.
    pub archivos: Vec<(String, String)>,
    /// `solapes.txt`, solo si hay solapes entre hablantes distintos.
    pub reporte_solapes: Option<String>,
    /// Captions sin hablante: no van a ningún archivo, pero hay que decirlo.
    pub sin_asignar: usize,
    /// Solapes detectados entre captions de distintos hablantes.
    pub solapes: usize,
}

/// Un SRT por hablante, como el oráculo: cada archivo lleva solo los captions
/// de ese hablante, ordenados por inicio, y el nombre se sanea y se deduplica
/// (dos «Ana» no pueden escribir el mismo archivo).
pub fn exportar_srt_por_hablante(proyecto: &Proyecto) -> ExportacionSrt {
    let mut usados: Vec<String> = Vec::new();
    let mut archivos: Vec<(String, String)> = Vec::new();

    for hablante in &proyecto.hablantes {
        let mut propios: Vec<Caption> = proyecto
            .captions
            .iter()
            .filter(|caption| caption.hablante_id.as_deref() == Some(hablante.id.as_str()))
            .cloned()
            .collect();
        if propios.is_empty() {
            continue;
        }
        propios.sort_by(|left, right| left.inicio.total_cmp(&right.inicio));

        // Nombre -> tecla -> id, el mismo respaldo que el oráculo: un hablante
        // sin nombre no puede quedar sin archivo.
        let base = if !hablante.nombre.is_empty() {
            &hablante.nombre
        } else if !hablante.tecla.is_empty() {
            &hablante.tecla
        } else {
            &hablante.id
        };
        let nombre = nombre_archivo_hablante_unico(base, &usados);
        usados.push(nombre.clone());
        archivos.push((nombre, build_srt(&propios)));
    }

    let solapes = build_overlap_report(&proyecto.captions, &proyecto.hablantes);
    ExportacionSrt {
        archivos,
        reporte_solapes: (!solapes.is_empty()).then(|| format_overlap_report(&solapes)),
        sin_asignar: proyecto
            .captions
            .iter()
            .filter(|caption| caption.hablante_id.is_none())
            .count(),
        solapes: solapes.len(),
    }
}

/// El JSON combinado del oráculo: una lista con los tiempos como texto SRT y el
/// nombre del hablante (`null` si el caption no tiene). Los objetos de JSON no
/// tienen orden, así que `serde_json` puede escribir las claves alfabéticas;
/// lo que importa es el contenido, no el orden del archivo.
pub fn exportar_json_combinado(proyecto: &Proyecto) -> String {
    let hablante_de = |id: Option<&str>| -> serde_json::Value {
        match id.and_then(|id| proyecto.hablantes.iter().find(|hablante| hablante.id == id)) {
            // Nombre, o la tecla si el nombre está vacío: es el texto que el
            // usuario ve en el panel, no el id interno.
            Some(hablante) if !hablante.nombre.is_empty() => {
                serde_json::Value::String(hablante.nombre.clone())
            }
            Some(hablante) => serde_json::Value::String(hablante.tecla.clone()),
            None => serde_json::Value::Null,
        }
    };
    let datos: Vec<serde_json::Value> = proyecto
        .captions
        .iter()
        .map(|caption| {
            serde_json::json!({
                "inicio": format_srt_timestamp(caption.inicio),
                "fin": format_srt_timestamp(caption.fin),
                "texto": caption.texto,
                "hablante": hablante_de(caption.hablante_id.as_deref()),
            })
        })
        .collect();
    serde_json::to_string_pretty(&datos).unwrap_or_default()
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
    use super::{
        Documento, FormatoSubtitulo, abrir_proyecto, cargar, exportar_ass, exportar_json_combinado,
        exportar_srt, exportar_srt_por_hablante, guardar_proyecto,
    };
    use rod_core::{Caption, Hablante, PresetAss, Proyecto, preset_por_defecto};
    use std::collections::HashMap;
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

    fn hablante(id: &str, nombre: &str, tecla: &str) -> Hablante {
        Hablante {
            id: id.into(),
            nombre: nombre.into(),
            tecla: tecla.into(),
            color: "#3B82F6".into(),
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

    /// Tres hablantes (dos homónimos y uno sin nombre) y un caption suelto: el
    /// caso que obliga a deduplicar archivos y a contar los sin asignar.
    fn proyecto_para_exportar() -> Proyecto {
        Proyecto {
            ruta_video: "video.mp4".into(),
            hablantes: vec![
                hablante("sp-1", "Ana", "1"),
                hablante("sp-2", "Ana", "2"),
                hablante("sp-3", "", "3"),
            ],
            captions: vec![
                // Desordenados a propósito: el export ordena por inicio.
                caption("c1", 2.0, 4.0, "dos", Some("sp-1")),
                caption("c2", 1.0, 3.0, "uno", Some("sp-1")),
                caption("c3", 3.5, 5.0, "solapa con dos", Some("sp-2")),
                caption("c4", 6.0, 7.0, "tres", Some("sp-3")),
                caption("c5", 8.0, 9.0, "suelto", None),
            ],
            playhead: 0.0,
        }
    }

    #[test]
    fn exporta_un_srt_por_hablante_con_nombres_unicos_y_sus_captions_ordenados() {
        let exportacion = exportar_srt_por_hablante(&proyecto_para_exportar());
        let nombres: Vec<&str> = exportacion
            .archivos
            .iter()
            .map(|(nombre, _)| nombre.as_str())
            .collect();
        // Los homónimos no se pisan y el hablante sin nombre cae a su tecla.
        assert_eq!(nombres, vec!["Ana.srt", "Ana_2.srt", "3.srt"]);

        let ana = &exportacion.archivos[0].1;
        assert!(ana.contains("00:00:01,000 --> 00:00:03,000"), "{ana}");
        assert!(ana.contains("00:00:02,000 --> 00:00:04,000"), "{ana}");
        let uno = ana.find("uno").expect("está");
        let dos = ana.find("dos").expect("está");
        assert!(uno < dos, "los captions van ordenados por inicio");

        // El solape es entre sp-1 y sp-2 en 3.5–4.0, y el reporte lo dice.
        assert_eq!(exportacion.solapes, 1);
        let reporte = exportacion.reporte_solapes.expect("hay solapes");
        assert!(reporte.contains("Ana"), "{reporte}");
        assert!(reporte.contains("solapa con dos"), "{reporte}");

        // El caption sin hablante no va a ningún archivo, pero se cuenta.
        assert_eq!(exportacion.sin_asignar, 1);
        assert!(!ana.contains("suelto"));
    }

    #[test]
    fn sin_solapes_no_escribe_el_reporte() {
        let mut proyecto = proyecto_para_exportar();
        // sp-2 deja de solaparse con sp-1.
        proyecto.captions[2].inicio = 4.5;
        proyecto.captions[2].fin = 5.0;

        let exportacion = exportar_srt_por_hablante(&proyecto);
        assert_eq!(exportacion.solapes, 0);
        assert_eq!(exportacion.reporte_solapes, None);
    }

    #[test]
    fn un_hablante_sin_captions_no_genera_archivo() {
        let mut proyecto = proyecto_para_exportar();
        proyecto.hablantes.push(hablante("sp-4", "Nadie", "4"));
        let exportacion = exportar_srt_por_hablante(&proyecto);
        assert!(
            exportacion
                .archivos
                .iter()
                .all(|(nombre, _)| nombre != "Nadie.srt")
        );
    }

    #[test]
    fn el_json_combinado_lleva_tiempos_srt_y_el_nombre_del_hablante() {
        let texto = exportar_json_combinado(&proyecto_para_exportar());
        let datos: serde_json::Value = serde_json::from_str(&texto).expect("JSON válido");
        let lista = datos.as_array().expect("es una lista");
        assert_eq!(lista.len(), 5);

        assert_eq!(lista[0]["inicio"], "00:00:02,000");
        assert_eq!(lista[0]["fin"], "00:00:04,000");
        assert_eq!(lista[0]["texto"], "dos");
        assert_eq!(lista[0]["hablante"], "Ana");

        // Sin nombre, la tecla; sin hablante, null.
        assert_eq!(lista[3]["hablante"], "3");
        assert_eq!(lista[4]["hablante"], serde_json::Value::Null);
    }

    #[test]
    fn el_ass_exportado_lleva_el_canvas_y_el_preset_del_hablante() {
        let mut proyecto = proyecto_para_exportar();
        proyecto.hablantes[0].preset_id = Some("p-cine".into());
        let documento = Documento {
            proyecto,
            formato: FormatoSubtitulo::Srt,
            estilos_ass: Vec::new(),
        };
        let cine = PresetAss {
            id: "p-cine".into(),
            nombre: "Cine".into(),
            fontname: "Georgia".into(),
            fontsize: 72.0,
            ..preset_por_defecto()
        };
        let mut presets = HashMap::new();
        presets.insert(cine.id.clone(), cine);

        let ass = exportar_ass(&documento, &presets, &preset_por_defecto(), (1920, 1080));
        assert!(ass.contains("PlayResX: 1920"), "{ass}");
        assert!(ass.contains("PlayResY: 1080"), "{ass}");
        // H1 es sp-1 (el primero del proyecto) y usa su preset.
        assert!(ass.contains("Style: H1,Georgia,72,"), "{ass}");
        // H2 no tiene preset: cae al de fábrica, no al de Cine.
        assert!(ass.contains("Style: H2,Arial,48,"), "{ass}");
    }
}
