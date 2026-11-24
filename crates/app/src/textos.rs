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
    pub nombre: &'static str,
    pub tecla: &'static str,
    pub color_identidad: &'static str,
    pub color_subtitulo: &'static str,
    pub agregar_hablante: &'static str,
    pub quitar_de_la_seleccion: &'static str,
    pub borrar_hablante: &'static str,
    pub ajustes: &'static str,
    pub pestana_atajos: &'static str,
    pub pestana_idioma: &'static str,
    pub cambiar: &'static str,
    pub intercambiar: &'static str,
    pub cancelar: &'static str,
    pub restablecer: &'static str,
    pub buscar: &'static str,
    pub pulsa_una_tecla: &'static str,
    pub arrastrar_video: &'static str,
    pub mas_recientes: &'static str,
    pub orden_nombre: &'static str,
    pub orden_lineas: &'static str,
    pub abrir_otro: &'static str,
    pub inicio: &'static str,
    pub tab_subtitulos: &'static str,
    pub tab_hablantes: &'static str,
    pub tab_estilos: &'static str,
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
    nombre: "Nombre",
    tecla: "Tecla",
    color_identidad: "Color de identidad",
    color_subtitulo: "Color del subtítulo (preset)",
    agregar_hablante: "Agregar hablante",
    quitar_de_la_seleccion: "Quitar de la selección",
    borrar_hablante: "Borrar hablante",
    ajustes: "Ajustes",
    pestana_atajos: "Atajos",
    pestana_idioma: "Idioma",
    cambiar: "Cambiar",
    intercambiar: "Intercambiar",
    cancelar: "Cancelar",
    restablecer: "Restablecer",
    buscar: "Buscar",
    pulsa_una_tecla: "Pulsa una tecla…",
    arrastrar_video: "Suelta aquí un MP4, MKV, MOV o WebM",
    mas_recientes: "Más recientes",
    orden_nombre: "Nombre",
    orden_lineas: "Líneas",
    abrir_otro: "Abrir otro proyecto",
    inicio: "Inicio",
    tab_subtitulos: "Subtítulos",
    tab_hablantes: "Hablantes",
    tab_estilos: "Estilos",
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
    nombre: "Name",
    tecla: "Key",
    color_identidad: "Identity color",
    color_subtitulo: "Subtitle color (preset)",
    agregar_hablante: "Add speaker",
    quitar_de_la_seleccion: "Clear speaker of selection",
    borrar_hablante: "Delete speaker",
    ajustes: "Preferences",
    pestana_atajos: "Shortcuts",
    pestana_idioma: "Language",
    cambiar: "Change",
    intercambiar: "Swap",
    cancelar: "Cancel",
    restablecer: "Reset",
    buscar: "Search",
    pulsa_una_tecla: "Press a key…",
    arrastrar_video: "Drop an MP4, MKV, MOV or WebM here",
    mas_recientes: "Most recent",
    orden_nombre: "Name",
    orden_lineas: "Lines",
    abrir_otro: "Open another project",
    inicio: "Home",
    tab_subtitulos: "Subtitles",
    tab_hablantes: "Speakers",
    tab_estilos: "Styles",
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
            Comando::Ajustes => "Ajustes",
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
            Comando::Ajustes => "Preferences",
        }
    }
}
