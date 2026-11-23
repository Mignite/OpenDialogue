//! Pantalla de arranque: arrastrar video, abrir proyecto, recientes y recovery.
//!
//! No conoce mpv ni el editor: solo decide qué quiere hacer el usuario.

use std::path::{Path, PathBuf};

use iced::widget::{button, column, container, image, row, scrollable, text};
use iced::{Element, Length, Task};

use crate::recientes::Reciente;
use crate::recovery_io::AvisoRecovery;
use crate::tema;
use crate::thumbnail;

/// Lo que la vista necesita para dibujarse. Los recientes ya vienen resueltos
/// (metadata leída del `.cdp` al arrancar); aquí no se toca ningún archivo.
pub struct EstadoInicio {
    pub recientes: Vec<Reciente>,
    pub expandido: bool,
    pub aviso: Option<AvisoRecovery>,
    pub mensaje: Option<String>,
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
    EstadoInicio {
        recientes,
        expandido: false,
        aviso,
        mensaje: None,
    }
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

pub fn update(
    estado: &mut EstadoInicio,
    mensaje: MensajeInicio,
) -> (Task<MensajeInicio>, Option<Salida>) {
    match mensaje {
        MensajeInicio::ArchivoSoltado(ruta) => {
            if es_video(&ruta) {
                estado.mensaje = None;
                (Task::none(), Some(Salida::Video(ruta)))
            } else {
                estado.mensaje = Some("Ese archivo no es un video (mp4, mkv, mov o webm)".into());
                (Task::none(), None)
            }
        }
        MensajeInicio::AbrirVideo => (
            Task::future(async {
                let ruta = rfd::AsyncFileDialog::new()
                    .add_filter("Video", &["mp4", "mkv", "mov", "webm"])
                    .pick_file()
                    .await
                    .map(|archivo| archivo.path().to_path_buf());
                MensajeInicio::DialogoVideo(ruta)
            }),
            None,
        ),
        MensajeInicio::AbrirProyecto => (
            Task::future(async {
                let ruta = rfd::AsyncFileDialog::new()
                    .add_filter("Proyecto", &["cdp"])
                    .pick_file()
                    .await
                    .map(|archivo| archivo.path().to_path_buf());
                MensajeInicio::DialogoProyecto(ruta)
            }),
            None,
        ),
        MensajeInicio::DialogoVideo(Some(ruta)) => {
            if es_video(&ruta) {
                (Task::none(), Some(Salida::Video(ruta)))
            } else {
                estado.mensaje = Some("Ese archivo no es un video (mp4, mkv, mov o webm)".into());
                (Task::none(), None)
            }
        }
        MensajeInicio::DialogoProyecto(Some(ruta)) => (Task::none(), Some(Salida::Proyecto(ruta))),
        MensajeInicio::DialogoVideo(None) | MensajeInicio::DialogoProyecto(None) => {
            (Task::none(), None)
        }
        MensajeInicio::AbrirReciente(ruta) => (Task::none(), Some(Salida::Proyecto(ruta))),
        MensajeInicio::VerTodos => {
            estado.expandido = !estado.expandido;
            (Task::none(), None)
        }
        MensajeInicio::Recuperar => (Task::none(), Some(Salida::Recuperar)),
        MensajeInicio::DescartarRecovery => {
            estado.aviso = None;
            (Task::none(), Some(Salida::DescartarRecovery))
        }
    }
}

/// La vista solo comprueba si el PNG existe en la ruta que ya calculó
/// `thumbnail`: no abre players ni genera nada al arrancar.
pub fn miniatura_en(base: &Path, reciente: &Reciente) -> Option<PathBuf> {
    let ruta = thumbnail::ruta_cache_en(base, &reciente.ruta);
    ruta.is_file().then_some(ruta)
}

fn vista_recientes(estado: &EstadoInicio) -> Element<'_, MensajeInicio> {
    let cantidad = if estado.expandido {
        estado.recientes.len()
    } else {
        estado.recientes.len().min(3)
    };
    let entries = estado
        .recientes
        .iter()
        .take(cantidad)
        .map(|reciente| {
            let miniatura: Element<'_, MensajeInicio> =
                match miniatura_en(&thumbnail::carpeta_cache(), reciente) {
                    Some(ruta) => image(iced::widget::image::Handle::from_path(ruta))
                        .width(80)
                        .into(),
                    None => text("[ > ]").into(),
                };
            button(
                row![
                    miniatura,
                    column![
                        text(
                            reciente
                                .ruta
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                        ),
                        text(format!(
                            "{} {} · {} {} · {}",
                            reciente.lineas,
                            if reciente.lineas == 1 {
                                "línea"
                            } else {
                                "líneas"
                            },
                            reciente.hablantes,
                            if reciente.hablantes == 1 {
                                "hablante"
                            } else {
                                "hablantes"
                            },
                            reciente.fecha
                        )),
                    ]
                    .spacing(4),
                ]
                .spacing(8),
            )
            .style(tema::boton_secundario)
            .on_press(MensajeInicio::AbrirReciente(reciente.ruta.clone()))
            .into()
        })
        .collect::<Vec<Element<'_, MensajeInicio>>>();

    let cabecera = if mostrar_ver_todos(&estado.recientes) {
        row![
            text("Recientes").width(Length::Fill),
            button(if estado.expandido {
                "Ver menos"
            } else {
                "Ver todos"
            })
            .style(tema::boton_secundario)
            .on_press(MensajeInicio::VerTodos),
        ]
    } else {
        row![text("Recientes").width(Length::Fill)]
    };

    let mut columna = column![cabecera];
    if entries.is_empty() {
        columna = columna.push(text(
            "Todavía no hay proyectos. El primero aparece aquí al guardar.",
        ));
    } else {
        columna = columna.push(scrollable(column(entries).spacing(6)));
    }
    columna.spacing(10).into()
}

/// `Ver todos` aparece solo cuando la lista supera las tres entradas que se
/// muestran por defecto: con cero recientes sería una decisión muerta.
pub fn mostrar_ver_todos(recientes: &[Reciente]) -> bool {
    recientes.len() > 3
}

fn vista_aviso(aviso: &AvisoRecovery) -> Element<'_, MensajeInicio> {
    column![
        text(aviso.texto.clone()),
        row![
            button("Recuperar")
                .style(tema::boton_primario)
                .on_press(MensajeInicio::Recuperar),
            button("Descartar")
                .style(tema::boton_secundario)
                .on_press(MensajeInicio::DescartarRecovery),
        ]
        .spacing(8),
    ]
    .spacing(8)
    .into()
}

pub fn view(estado: &EstadoInicio) -> Element<'_, MensajeInicio> {
    let zona = container(
        column![
            text("[ > ]").size(40),
            text("Arrastra un video aquí").size(22),
            text("MP4, MKV, MOV o WebM"),
            row![
                button("Abrir video")
                    .style(tema::boton_primario)
                    .on_press(MensajeInicio::AbrirVideo),
                button("Abrir proyecto")
                    .style(tema::boton_secundario)
                    .on_press(MensajeInicio::AbrirProyecto),
            ]
            .spacing(10),
            text(estado.mensaje.clone().unwrap_or_default()),
        ]
        .spacing(12)
        .align_x(iced::alignment::Horizontal::Center),
    )
    .style(tema::panel)
    .width(Length::FillPortion(2))
    .padding(24)
    .center_x(Length::Fill)
    .center_y(Length::Fill);

    let derecha = match estado.aviso.as_ref() {
        Some(aviso) => column![vista_recientes(estado), vista_aviso(aviso)],
        None => column![vista_recientes(estado)],
    }
    .width(Length::FillPortion(1))
    .spacing(20);

    row![zona, derecha].spacing(24).padding(24).into()
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
                fecha: "hoy".into(),
            })
            .collect::<Vec<_>>();
        assert!(!super::mostrar_ver_todos(&tres));
        let cuatro = (0..4)
            .map(|_| crate::recientes::Reciente {
                ruta: PathBuf::from("a.cdp"),
                lineas: 1,
                hablantes: 1,
                fecha: "hoy".into(),
            })
            .collect::<Vec<_>>();
        assert!(super::mostrar_ver_todos(&cuatro));
    }

    #[test]
    fn miniatura_ausente_muestra_placeholder_y_presente_devuelve_ruta() {
        let dir = std::env::temp_dir().join(format!("rod-inicio-miniatura-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let reciente = crate::recientes::Reciente {
            ruta: PathBuf::from("/p/mi.cdp"),
            lineas: 1,
            hablantes: 1,
            fecha: "hoy".into(),
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
