//! Pantalla de arranque: arrastrar video, abrir proyecto, recientes y recovery.
//!
//! No conoce mpv ni el editor: solo decide qué quiere hacer el usuario.

use std::path::{Path, PathBuf};

use crate::recientes::Reciente;
use crate::recovery_io::AvisoRecovery;
use crate::tarea::Tarea;
use crate::thumbnail;

/// Lo que la vista necesita para dibujarse. Los recientes ya vienen resueltos
/// (metadata leída del `.cdp` al arrancar); aquí no se toca ningún archivo.
/// Criterio de orden de las tarjetas de recientes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrdenRecientes {
    /// Fecha de modificación, de más reciente a más viejo (el orden del índice).
    #[default]
    Recientes,
    Nombre,
    Lineas,
}

pub struct EstadoInicio {
    pub recientes: Vec<Reciente>,
    pub expandido: bool,
    pub aviso: Option<AvisoRecovery>,
    pub mensaje: Option<String>,
    pub orden: OrdenRecientes,
    /// Índice de la tarjeta con el foco del teclado.
    pub destacado: Option<usize>,
}

impl EstadoInicio {
    pub fn nuevo(recientes: Vec<Reciente>, aviso: Option<AvisoRecovery>) -> Self {
        Self {
            recientes,
            expandido: false,
            aviso,
            mensaje: None,
            orden: OrdenRecientes::default(),
            destacado: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MensajeInicio {
    ArchivoSoltado(PathBuf),
    AbrirVideo,
    AbrirProyecto,
    AbrirReciente(PathBuf),
    VerTodos,
    Recuperar,
    DescartarRecovery,
    DialogoVideo(Option<PathBuf>),
    DialogoProyecto(Option<PathBuf>),
}

/// Una intención que el enrutador convierte en editor o en una acción de
/// recovery. `inicio` no construye editores: solo dice qué se pidió.
#[derive(Debug, Clone, PartialEq)]
pub enum Salida {
    Video(PathBuf),
    Proyecto(PathBuf),
    Recuperar,
    DescartarRecovery,
}

pub fn nuevo(recientes: Vec<Reciente>, aviso: Option<AvisoRecovery>) -> EstadoInicio {
    EstadoInicio::nuevo(recientes, aviso)
}

/// El botón «Ver todos» solo aparece cuando hay más de tres recientes.
pub fn mostrar_ver_todos(recientes: &[Reciente]) -> bool {
    recientes.len() > 3
}

pub const MENSAJE_ARCHIVO_INVALIDO: &str =
    "No se pudo abrir notas.txt. Usa un video MP4, MKV, MOV o WebM.";

/// Solo tres recientes hasta que el usuario pide `Ver todos`.
pub fn cantidad_visible(total: usize, expandido: bool) -> usize {
    if expandido { total } else { total.min(3) }
}

/// Extensiones que el menú acepta. Es un filtro de interfaz: mpv decide si el
/// archivo abre de verdad.
fn es_video(ruta: &Path) -> bool {
    ruta.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            ["mp4", "mkv", "mov", "webm"]
                .iter()
                .any(|valida| extension.eq_ignore_ascii_case(valida))
        })
}

pub fn update(estado: &mut EstadoInicio, mensaje: MensajeInicio) -> (Tarea, Option<Salida>) {
    match mensaje {
        MensajeInicio::ArchivoSoltado(ruta) => {
            if es_video(&ruta) {
                estado.mensaje = None;
                (Tarea::Ninguna, Some(Salida::Video(ruta)))
            } else {
                estado.mensaje = Some(MENSAJE_ARCHIVO_INVALIDO.into());
                (Tarea::Ninguna, None)
            }
        }
        MensajeInicio::AbrirVideo => (Tarea::DialogoVideo, None),
        MensajeInicio::AbrirProyecto => (Tarea::DialogoProyecto, None),
        MensajeInicio::DialogoVideo(Some(ruta)) => {
            if es_video(&ruta) {
                (Tarea::Ninguna, Some(Salida::Video(ruta)))
            } else {
                estado.mensaje = Some(MENSAJE_ARCHIVO_INVALIDO.into());
                (Tarea::Ninguna, None)
            }
        }
        MensajeInicio::DialogoProyecto(Some(ruta)) => {
            (Tarea::Ninguna, Some(Salida::Proyecto(ruta)))
        }
        MensajeInicio::DialogoVideo(None) | MensajeInicio::DialogoProyecto(None) => {
            (Tarea::Ninguna, None)
        }
        MensajeInicio::AbrirReciente(ruta) => (Tarea::Ninguna, Some(Salida::Proyecto(ruta))),
        MensajeInicio::VerTodos => {
            estado.expandido = !estado.expandido;
            (Tarea::Ninguna, None)
        }
        MensajeInicio::Recuperar => (Tarea::Ninguna, Some(Salida::Recuperar)),
        MensajeInicio::DescartarRecovery => {
            estado.aviso = None;
            (Tarea::Ninguna, Some(Salida::DescartarRecovery))
        }
    }
}

/// La vista solo comprueba si el PNG existe en la ruta que ya calculó
/// `thumbnail`: no abre players ni genera nada al arrancar.
pub fn miniatura_en(base: &Path, reciente: &Reciente) -> Option<PathBuf> {
    let ruta = thumbnail::ruta_cache_en(base, &reciente.ruta);
    ruta.is_file().then_some(ruta)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn ver_todos_aparece_solo_con_mas_de_tres_recientes() {
        assert!(!super::mostrar_ver_todos(&[]));
        let tres = (0..3)
            .map(|_| crate::recientes::Reciente {
                ruta: PathBuf::from("a.cdp"),
                lineas: 1,
                hablantes: 1,
                fecha: Default::default(),
                duracion: 0.0,
                tira: Vec::new(),
            })
            .collect::<Vec<_>>();
        assert!(!super::mostrar_ver_todos(&tres));
        let cuatro = (0..4)
            .map(|_| crate::recientes::Reciente {
                ruta: PathBuf::from("a.cdp"),
                lineas: 1,
                hablantes: 1,
                fecha: Default::default(),
                duracion: 0.0,
                tira: Vec::new(),
            })
            .collect::<Vec<_>>();
        assert!(super::mostrar_ver_todos(&cuatro));
    }

    #[test]
    fn cantidad_visible_respeta_el_tope_y_el_expandido() {
        assert_eq!(super::cantidad_visible(0, false), 0);
        assert_eq!(super::cantidad_visible(7, false), 3);
        assert_eq!(super::cantidad_visible(7, true), 7);
    }

    #[test]
    fn soltar_extension_no_valida_usa_el_copy_de_la_spec() {
        let mut estado = super::nuevo(Vec::new(), None);
        let (_tarea, salida) = super::update(
            &mut estado,
            super::MensajeInicio::ArchivoSoltado(PathBuf::from("notas.txt")),
        );
        assert!(salida.is_none());
        assert_eq!(
            estado.mensaje.as_deref(),
            Some("No se pudo abrir notas.txt. Usa un video MP4, MKV, MOV o WebM.")
        );
    }

    #[test]
    fn miniatura_ausente_muestra_placeholder_y_presente_devuelve_ruta() {
        let dir = std::env::temp_dir().join(format!("rod-inicio-miniatura-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let reciente = crate::recientes::Reciente {
            ruta: PathBuf::from("/p/mi.cdp"),
            lineas: 1,
            hablantes: 1,
            fecha: Default::default(),
            duracion: 0.0,
            tira: Vec::new(),
        };
        assert!(super::miniatura_en(&dir, &reciente).is_none());
        let png = crate::thumbnail::ruta_cache_en(&dir, &reciente.ruta);
        std::fs::write(&png, b"png").unwrap();
        assert_eq!(super::miniatura_en(&dir, &reciente), Some(png));
    }

    fn aviso() -> crate::recovery_io::AvisoRecovery {
        let proyecto = rod_core::Proyecto::nuevo("clip.mp4");
        let recovery = rod_core::construir_recovery(proyecto, "", 1_000.0);
        let texto = rod_core::formatear_recovery(&recovery, "clip.cdp", 1_000.0);
        crate::recovery_io::AvisoRecovery { recovery, texto }
    }

    #[test]
    fn soltar_video_con_extension_valida_pide_abrir_editor() {
        let mut estado = super::nuevo(Vec::new(), None);
        let (_tarea, salida) = super::update(
            &mut estado,
            super::MensajeInicio::ArchivoSoltado(PathBuf::from("clip.mp4")),
        );
        assert!(matches!(salida, Some(super::Salida::Video(_))));
    }

    #[test]
    fn soltar_extension_no_valida_deja_mensaje_y_no_sale() {
        let mut estado = super::nuevo(Vec::new(), None);
        let (_tarea, salida) = super::update(
            &mut estado,
            super::MensajeInicio::ArchivoSoltado(PathBuf::from("notas.txt")),
        );
        assert!(salida.is_none());
        assert!(estado.mensaje.is_some());
    }

    #[test]
    fn ver_todos_expande_la_lista() {
        let mut estado = super::nuevo(Vec::new(), None);
        assert!(!estado.expandido);
        let _ = super::update(&mut estado, super::MensajeInicio::VerTodos);
        assert!(estado.expandido);
    }

    #[test]
    fn descartar_recovery_quita_el_aviso() {
        let mut estado = super::nuevo(Vec::new(), Some(aviso()));
        let (_tarea, salida) = super::update(&mut estado, super::MensajeInicio::DescartarRecovery);
        assert!(estado.aviso.is_none());
        assert!(matches!(salida, Some(super::Salida::DescartarRecovery)));
    }
}
