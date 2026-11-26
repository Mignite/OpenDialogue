//! Textos visibles en los dos idiomas soportados.
//!
//! Una tabla por idioma, ambas del mismo tipo: si falta una frase, no compila.
//! No hay claves de cadena que admitan erratas silenciosas.

use crate::comandos::Comando;

/// Días completos entre la modificación de un reciente y ahora.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FechaRelativa {
    pub dias: u64,
    pub horas: u64,
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
    pub aplicar: &'static str,
    pub sin_hablante: &'static str,
    pub entrada: &'static str,
    pub salida: &'static str,
    pub sin_seleccion: &'static str,
    pub linea_eliminada: &'static str,
    pub ver_todos: &'static str,
    pub editar: &'static str,
    pub timeline: &'static str,
    pub seguir: &'static str,
    pub pista_sin_hablantes: &'static str,
    pub mostrar_vacios: &'static str,
    pub abrir: &'static str,
    pub mostrar_en_carpeta: &'static str,
    pub quitar_de_recientes: &'static str,
    pub drop_titulo: &'static str,
    pub drop_subtitulo: &'static str,
    pub drop_activo: &'static str,
    pub buscar_subtitulos: &'static str,
    pub anadir_linea: &'static str,
    pub importar_subtitulos: &'static str,
    pub exportar_subtitulos: &'static str,
    pub seleccionar_todas: &'static str,
    pub divisor: &'static str,
    pub unir_siguiente: &'static str,
    pub aplicar_a_seleccion: &'static str,
    pub nuevo_preset: &'static str,
    pub duplicar: &'static str,
    pub renombrar: &'static str,
    pub mostrar_vacios_toggle: &'static str,
    pub suelta_subtitulos: &'static str,
    pub encontrar_subtitulo: &'static str,
    pub fuente: &'static str,
    pub tamano: &'static str,
    pub color: &'static str,
    pub contorno: &'static str,
    pub sombra: &'static str,
    pub fondo: &'static str,
    pub posicion: &'static str,
    pub negrita: &'static str,
    pub cursiva: &'static str,
    pub mayusculas: &'static str,
    pub estilo: &'static str,
    pub fuente_no_encontrada: &'static str,
    pub margen_v: &'static str,
    pub buscar_fuente: &'static str,
    pub preset_eliminado: &'static str,
    pub sin_subtitulos: &'static str,
    pub importar_archivo: &'static str,
    // Fundación: claves compartidas por las áreas A–E.
    pub tiempo: &'static str,
    pub pista_audio: &'static str,
    pub sin_estilo: &'static str,
    pub fit_all: &'static str,
    pub hide_empty_tracks: &'static str,
    pub collapse_track: &'static str,
    pub restore: &'static str,
    pub discard: &'static str,
    pub video_not_found: &'static str,
    pub locate_video: &'static str,
    pub sin_atajo: &'static str,
    pub preview_texto: &'static str,
    pub variante_no_disponible: &'static str,
    pub minimizar: &'static str,
    pub maximizar: &'static str,
    pub cerrar_ventana: &'static str,
    pub arrastrar_ventana: &'static str,
    pub atajo_abrir_proyecto: &'static str,
    pub inicio_transporte: &'static str,
    pub cuadro_anterior: &'static str,
    pub cuadro_siguiente: &'static str,
    pub fin_transporte: &'static str,
    pub volumen: &'static str,
}

pub const ES: Textos = Textos {
    es: true,
    inicio_titulo: "OpenDialogue",
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
    aplicar: "Aplicar",
    sin_hablante: "Sin hablante",
    entrada: "Entrada",
    salida: "Salida",
    sin_seleccion: "Selecciona una línea para editarla",
    linea_eliminada: "Línea eliminada",
    ver_todos: "Ver todos",
    editar: "Editar",
    timeline: "Línea de tiempo",
    seguir: "Seguir al playhead",
    pista_sin_hablantes: "Asigna un hablante para ver su pista",
    mostrar_vacios: "Mostrar vacíos",
    abrir: "Abrir",
    mostrar_en_carpeta: "Mostrar en carpeta",
    quitar_de_recientes: "Quitar de recientes",
    drop_titulo: "Suelta un video aquí",
    drop_subtitulo: "MP4, MKV, MOV o WebM",
    drop_activo: "Suelta para abrir",
    buscar_subtitulos: "Buscar texto o hablante",
    anadir_linea: "Añadir línea",
    importar_subtitulos: "Importar subtítulos…",
    exportar_subtitulos: "Exportar…",
    seleccionar_todas: "Seleccionar todas las líneas del hablante",
    divisor: "Dividir en el playhead",
    unir_siguiente: "Unir con la siguiente",
    aplicar_a_seleccion: "Aplicar a la selección",
    nuevo_preset: "Nuevo preset",
    duplicar: "Duplicar",
    renombrar: "Renombrar",
    mostrar_vacios_toggle: "Mostrar vacíos",
    suelta_subtitulos: "Suelta para importar subtítulos",
    encontrar_subtitulo: "Se encontró {} · Importar",
    fuente: "Fuente",
    tamano: "Tamaño",
    color: "Color",
    contorno: "Contorno",
    sombra: "Sombra",
    fondo: "Fondo",
    posicion: "Posición",
    negrita: "Negrita",
    cursiva: "Cursiva",
    mayusculas: "Mayúsculas",
    estilo: "Estilo",
    fuente_no_encontrada: "Fuente no encontrada",
    margen_v: "Margen V",
    buscar_fuente: "Buscar fuente…",
    preset_eliminado: "Estilo eliminado",
    sin_subtitulos: "Aún no hay subtítulos",
    importar_archivo: "Importar archivo",
    tiempo: "Tiempo",
    pista_audio: "Audio",
    sin_estilo: "Estilo / Sin estilo",
    fit_all: "Ajustar todo",
    hide_empty_tracks: "Ocultar pistas vacías",
    collapse_track: "Contraer pista",
    restore: "Restaurar",
    discard: "Descartar",
    video_not_found: "Video no encontrado",
    locate_video: "Localizar video…",
    sin_atajo: "Sin atajo",
    preview_texto: "Vista previa",
    variante_no_disponible: "Variante no disponible",
    minimizar: "Minimizar",
    maximizar: "Maximizar",
    cerrar_ventana: "Cerrar ventana",
    arrastrar_ventana: "Arrastrar ventana",
    atajo_abrir_proyecto: "Ctrl+O",
    inicio_transporte: "Inicio",
    cuadro_anterior: "Cuadro anterior",
    cuadro_siguiente: "Cuadro siguiente",
    fin_transporte: "Fin",
    volumen: "Volumen",
};

pub const EN: Textos = Textos {
    es: false,
    inicio_titulo: "OpenDialogue",
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
    aplicar: "Apply",
    sin_hablante: "No speaker",
    entrada: "In",
    salida: "Out",
    sin_seleccion: "Select a line to edit it",
    linea_eliminada: "Line deleted",
    ver_todos: "See all",
    editar: "Edit",
    timeline: "Timeline",
    seguir: "Follow playhead",
    pista_sin_hablantes: "Assign a speaker to see their track",
    mostrar_vacios: "Show empty",
    abrir: "Open",
    mostrar_en_carpeta: "Show in folder",
    quitar_de_recientes: "Remove from recent",
    drop_titulo: "Drop a video here",
    drop_subtitulo: "MP4, MKV, MOV or WebM",
    drop_activo: "Drop to open",
    buscar_subtitulos: "Find text or speaker",
    anadir_linea: "Add line",
    importar_subtitulos: "Import subtitles…",
    exportar_subtitulos: "Export…",
    seleccionar_todas: "Select all lines from speaker",
    divisor: "Split at playhead",
    unir_siguiente: "Join with next",
    aplicar_a_seleccion: "Apply to selection",
    nuevo_preset: "New preset",
    duplicar: "Duplicate",
    renombrar: "Rename",
    mostrar_vacios_toggle: "Show empty",
    suelta_subtitulos: "Drop to import subtitles",
    encontrar_subtitulo: "Found {} · Import",
    fuente: "Font",
    tamano: "Size",
    color: "Color",
    contorno: "Outline",
    sombra: "Shadow",
    fondo: "Background",
    posicion: "Position",
    negrita: "Bold",
    cursiva: "Italic",
    mayusculas: "Uppercase",
    estilo: "Style",
    fuente_no_encontrada: "Font not found",
    margen_v: "Margin V",
    buscar_fuente: "Search font…",
    preset_eliminado: "Style deleted",
    sin_subtitulos: "No subtitles yet",
    importar_archivo: "Import file",
    tiempo: "Time",
    pista_audio: "Audio",
    sin_estilo: "Style / No style",
    fit_all: "Fit all",
    hide_empty_tracks: "Hide empty tracks",
    collapse_track: "Collapse track",
    restore: "Restore",
    discard: "Discard",
    video_not_found: "Video not found",
    locate_video: "Locate video…",
    sin_atajo: "None",
    preview_texto: "Preview",
    variante_no_disponible: "Variant not available",
    minimizar: "Minimize",
    maximizar: "Maximize",
    cerrar_ventana: "Close window",
    arrastrar_ventana: "Drag window",
    atajo_abrir_proyecto: "Ctrl+O",
    inicio_transporte: "Start",
    cuadro_anterior: "Previous frame",
    cuadro_siguiente: "Next frame",
    fin_transporte: "End",
    volumen: "Volume",
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
            Comando::Unir => "Unir con la siguiente",
            Comando::Entrada => "Fijar entrada",
            Comando::Salida => "Fijar salida",
            Comando::AlternarSnap => "Ajustar al imán",
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

    /// Etiqueta del botón que aplica el preset a las líneas seleccionadas.
    pub fn aplicar_a_lineas(&self, n: usize) -> String {
        if self.es {
            if n == 1 {
                "Aplicar a 1 línea seleccionada".into()
            } else {
                format!("Aplicar a {n} líneas seleccionadas")
            }
        } else if n == 1 {
            "Apply to 1 selected line".into()
        } else {
            format!("Apply to {n} selected lines")
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
            if n == 0 && fecha.horas > 0 {
                format!("hace {} h", fecha.horas)
            } else {
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
            }
        } else if n == 0 && fecha.horas > 0 {
            format!("{} h ago", fecha.horas)
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

    /// Contador de solapes, con singular y plural en los dos idiomas.
    pub fn solapes(&self, n: usize) -> String {
        if self.es {
            if n == 1 {
                "1 solape".into()
            } else {
                format!("{n} solapes")
            }
        } else if n == 1 {
            "1 overlap".into()
        } else {
            format!("{n} overlaps")
        }
    }

    /// Aviso de recuperación reconstruido en la capa app: el texto del core es
    /// español fijo y no se puede traducir allí. `fecha` ya viene formateada y
    /// `lineas` es `recovery.proyecto.captions.len()`.
    pub fn fecha_recuperacion(&self, fecha: &str, lineas: usize) -> String {
        let subtitulos = if self.es {
            if lineas == 1 {
                "subtítulo"
            } else {
                "subtítulos"
            }
        } else if lineas == 1 {
            "subtitle"
        } else {
            "subtitles"
        };
        if self.es {
            format!("Copia automática de {fecha} · {lineas} {subtitulos}")
        } else {
            format!("Autosaved {fecha} · {lineas} {subtitulos}")
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
            Comando::Unir => "Join with next line",
            Comando::Entrada => "Set in point",
            Comando::Salida => "Set out point",
            Comando::AlternarSnap => "Toggle snapping",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn las_tablas_traen_las_claves_nuevas_y_los_plurales() {
        for tabla in [&ES, &EN] {
            assert_eq!(tabla.atajo_abrir_proyecto, "Ctrl+O");
            assert!(!tabla.sin_estilo.is_empty());
            assert!(!tabla.tiempo.is_empty());
            assert!(!tabla.pista_audio.is_empty());
            assert!(tabla.solapes(1).starts_with('1'));
            assert!(tabla.fecha_recuperacion("hace 3 min", 2).contains("2"));
        }
        assert_eq!(ES.sin_estilo, "Estilo / Sin estilo");
        assert_eq!(EN.sin_estilo, "Style / No style");
        assert!(ES.solapes(2).contains("solapes"));
        assert!(EN.solapes(2).contains("overlaps"));
        assert!(ES.fecha_recuperacion("hace 3 min", 1).contains("subtítulo"));
        assert!(EN.fecha_recuperacion("3 min ago", 1).contains("subtitle"));
    }
}
