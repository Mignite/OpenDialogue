//! Textos visibles en los dos idiomas soportados.
//!
//! Una tabla por idioma, ambas del mismo tipo: si falta una frase, no compila.
//! No hay claves de cadena que admitan erratas silenciosas.

use crate::comandos::Comando;

/// Días completos entre la modificación de un reciente y ahora.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FechaRelativa {
    pub dias: u64,
}

/// Idioma de la interfaz. Por defecto, inglés: el producto es global.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Idioma {
    Es,
    #[default]
    En,
}

impl Idioma {
    pub fn textos(self) -> &'static Textos {
        match self {
            Idioma::Es => &ES,
            Idioma::En => &EN,
        }
    }
}

/// Todas las frases de la interfaz. Los plurales van en métodos.
pub struct Textos {
    /// Discriminador para los métodos con `match` por tabla.
    pub es: bool,
    pub inicio_titulo: &'static str,
    pub inicio_subtitulo: &'static str,
    pub abrir_video: &'static str,
    pub abrir_proyecto: &'static str,
    pub recientes: &'static str,
    pub texto_vacio_recientes: &'static str,
    pub guardado: &'static str,
    pub sin_guardar: &'static str,
    pub seccion_archivo: &'static str,
    pub seccion_edicion: &'static str,
    pub seccion_ver: &'static str,
    pub seccion_exportar: &'static str,
    pub seccion_estilos: &'static str,
    pub seccion_ayuda: &'static str,
}

pub const ES: Textos = Textos {
    es: true,
    inicio_titulo: "Edita subtítulos con precisión",
    inicio_subtitulo: "Abre un video o continúa un proyecto guardado.",
    abrir_video: "Abrir video",
    abrir_proyecto: "Abrir proyecto",
    recientes: "Recientes",
    texto_vacio_recientes: "Todavía no hay proyectos. El primero aparece aquí al guardar.",
    guardado: "Guardado",
    sin_guardar: "Sin guardar",
    seccion_archivo: "Archivo",
    seccion_edicion: "Edición",
    seccion_ver: "Ver",
    seccion_exportar: "Exportar",
    seccion_estilos: "Estilos",
    seccion_ayuda: "Ayuda",
};

pub const EN: Textos = Textos {
    es: false,
    inicio_titulo: "Edit subtitles with precision",
    inicio_subtitulo: "Open a video or continue a saved project.",
    abrir_video: "Open video",
    abrir_proyecto: "Open project",
    recientes: "Recent",
    texto_vacio_recientes: "No projects yet. The first one appears here when you save.",
    guardado: "Saved",
    sin_guardar: "Unsaved",
    seccion_archivo: "File",
    seccion_edicion: "Edit",
    seccion_ver: "View",
    seccion_exportar: "Export",
    seccion_estilos: "Styles",
    seccion_ayuda: "Help",
};

impl Textos {
    /// Etiqueta de un comando, para menús y tooltips.
    pub fn comando(&self, comando: Comando) -> &'static str {
        if self.es {
            Self::comando_es(comando)
        } else {
            Self::comando_en(comando)
        }
    }

    fn comando_es(comando: Comando) -> &'static str {
        match comando {
            Comando::NuevoProyecto => "Nuevo proyecto",
            Comando::AbrirProyecto => "Abrir proyecto",
            Comando::AbrirVideo => "Abrir video",
            Comando::Guardar => "Guardar",
            Comando::GuardarComo => "Guardar como…",
            Comando::CargarSrt => "Cargar SRT",
            Comando::CargarAss => "Cargar ASS",
            Comando::ImportarAutosubs => "Importar autosubs",
            Comando::ExportarSrt => "Exportar SRT",
            Comando::ExportarJson => "Exportar JSON",
            Comando::ExportarAss => "Exportar ASS",
            Comando::Estilos => "Estilos",
            Comando::ReproducirPausa => "Reproducir o pausar",
            Comando::CaptionAnterior => "Subtítulo anterior",
            Comando::CaptionSiguiente => "Subtítulo siguiente",
            Comando::RetrocederFino => "Retroceder un cuadro",
            Comando::AvanzarFino => "Avanzar un cuadro",
            Comando::Retroceder => "Retroceder",
            Comando::Avanzar => "Avanzar",
            Comando::NudgeIzquierda => "Mover al inicio",
            Comando::NudgeDerecha => "Mover al final",
            Comando::EditarTexto => "Editar texto",
            Comando::SimultaneoArriba => "Subir simultáneo",
            Comando::SimultaneoAbajo => "Bajar simultáneo",
            Comando::AgregarFragmento => "Agregar fragmento",
            Comando::Eliminar => "Eliminar",
            Comando::Deshacer => "Deshacer",
            Comando::Rehacer => "Rehacer",
            Comando::Copiar => "Copiar",
            Comando::Pegar => "Pegar",
            Comando::Dividir => "Dividir en el playhead",
            Comando::Buscar => "Buscar",
            Comando::Ayuda => "Atajos de teclado",
            Comando::VelocidadMas => "Aumentar velocidad",
            Comando::VelocidadMenos => "Reducir velocidad",
            Comando::Hablante(_) => "Hablante",
            Comando::PanelSubtitulos => "Panel de subtítulos",
            Comando::PanelHablantes => "Panel de hablantes",
            Comando::PanelEstilos => "Panel de estilos",
        }
    }

    pub fn lineas(&self, n: usize) -> String {
        if self.es {
            if n == 1 {
                "1 línea".into()
            } else {
                format!("{n} líneas")
            }
        } else if n == 1 {
            "1 line".into()
        } else {
            format!("{n} lines")
        }
    }

    pub fn hablantes(&self, n: usize) -> String {
        if self.es {
            if n == 1 {
                "1 hablante".into()
            } else {
                format!("{n} hablantes")
            }
        } else if n == 1 {
            "1 speaker".into()
        } else {
            format!("{n} speakers")
        }
    }

    pub fn fecha(&self, fecha: &FechaRelativa) -> String {
        let n = fecha.dias;
        if self.es {
            match n {
                0 => "hoy".into(),
                1 => "ayer".into(),
                n if n < 7 => format!("hace {n} días"),
                n => {
                    let semanas = n / 7;
                    if semanas == 1 {
                        "hace 1 semana".into()
                    } else {
                        format!("hace {semanas} semanas")
                    }
                }
            }
        } else {
            match n {
                0 => "today".into(),
                1 => "yesterday".into(),
                n if n < 7 => format!("{n} days ago"),
                n => {
                    let semanas = n / 7;
                    if semanas == 1 {
                        "last week".into()
                    } else {
                        format!("{semanas} weeks ago")
                    }
                }
            }
        }
    }
}

impl Textos {
    /// Tabla inglesa, exhaustiva e independiente de la española.
    fn comando_en(comando: Comando) -> &'static str {
        match comando {
            Comando::NuevoProyecto => "New project",
            Comando::AbrirProyecto => "Open project",
            Comando::AbrirVideo => "Open video",
            Comando::Guardar => "Save",
            Comando::GuardarComo => "Save as…",
            Comando::CargarSrt => "Load SRT",
            Comando::CargarAss => "Load ASS",
            Comando::ImportarAutosubs => "Import autosubs",
            Comando::ExportarSrt => "Export SRT",
            Comando::ExportarJson => "Export JSON",
            Comando::ExportarAss => "Export ASS",
            Comando::Estilos => "Styles",
            Comando::ReproducirPausa => "Play or pause",
            Comando::CaptionAnterior => "Previous subtitle",
            Comando::CaptionSiguiente => "Next subtitle",
            Comando::RetrocederFino => "Step back one frame",
            Comando::AvanzarFino => "Step forward one frame",
            Comando::Retroceder => "Rewind",
            Comando::Avanzar => "Fast forward",
            Comando::NudgeIzquierda => "Move to start",
            Comando::NudgeDerecha => "Move to end",
            Comando::EditarTexto => "Edit text",
            Comando::SimultaneoArriba => "Move overlap up",
            Comando::SimultaneoAbajo => "Move overlap down",
            Comando::AgregarFragmento => "Add fragment",
            Comando::Eliminar => "Delete",
            Comando::Deshacer => "Undo",
            Comando::Rehacer => "Redo",
            Comando::Copiar => "Copy",
            Comando::Pegar => "Paste",
            Comando::Dividir => "Split at playhead",
            Comando::Buscar => "Find",
            Comando::Ayuda => "Keyboard shortcuts",
            Comando::VelocidadMas => "Increase speed",
            Comando::VelocidadMenos => "Decrease speed",
            Comando::Hablante(_) => "Speaker",
            Comando::PanelSubtitulos => "Subtitles panel",
            Comando::PanelHablantes => "Speakers panel",
            Comando::PanelEstilos => "Styles panel",
        }
    }
}
