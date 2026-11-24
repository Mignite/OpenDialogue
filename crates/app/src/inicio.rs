//! Pantalla de arranque: arrastrar video, abrir proyecto, recientes y recovery.
//!
//! No conoce mpv ni el editor: solo decide qué quiere hacer el usuario.

use std::path::{Path, PathBuf};

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, image, row, scrollable, text};
use iced::{Color, Element, Length, Task};

use crate::iconos::{self, Icono};
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

pub const MENSAJE_ARCHIVO_INVALIDO: &str =
    "No se pudo abrir notas.txt. Usa un video MP4, MKV, MOV o WebM.";

/// Solo tres recientes hasta que el usuario pide `Ver todos`.
pub fn cantidad_visible(total: usize, expandido: bool) -> usize {
    if expandido { total } else { total.min(3) }
}

pub fn resumen_reciente(reciente: &Reciente) -> String {
    format!(
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
    )
}

pub fn texto_vacio_recientes() -> &'static str {
    "Todavía no hay proyectos. El primero aparece aquí al guardar."
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
                estado.mensaje = Some(MENSAJE_ARCHIVO_INVALIDO.into());
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
                estado.mensaje = Some(MENSAJE_ARCHIVO_INVALIDO.into());
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
    let cantidad = cantidad_visible(estado.recientes.len(), estado.expandido);
    let mut tarjetas = estado
        .recientes
        .iter()
        .take(cantidad)
        .map(|reciente| {
            let miniatura: Element<'_, MensajeInicio> =
                match miniatura_en(&thumbnail::carpeta_cache(), reciente) {
                    Some(ruta) => container(
                        image(iced::widget::image::Handle::from_path(ruta))
                            .width(196)
                            .height(110),
                    )
                    .width(196)
                    .height(110)
                    .style(tema::sunk)
                    .into(),
                    None => container(iconos::vista(Icono::Imagen, 30, tema::DIM))
                        .width(196)
                        .height(110)
                        .style(tema::sunk)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                        .into(),
                };
            let ruta = reciente.ruta.display().to_string();
            let ruta = if ruta.chars().count() > 30 {
                format!("{}…", ruta.chars().take(29).collect::<String>())
            } else {
                ruta
            };
            button(column![
                miniatura,
                container(
                    column![
                        text(
                            reciente
                                .ruta
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string()
                        )
                        .size(12),
                        text(ruta).font(tema::MONO).size(9).color(tema::DIM),
                        text(resumen_reciente(reciente))
                            .font(tema::MONO)
                            .size(9)
                            .color(tema::SECUNDARIO),
                    ]
                    .spacing(4),
                )
                .padding([8, 10]),
            ])
            .style(tema::boton_secundario)
            .on_press(MensajeInicio::AbrirReciente(reciente.ruta.clone()))
            .width(196)
            .into()
        })
        .collect::<Vec<Element<'_, MensajeInicio>>>();

    if tarjetas.is_empty() {
        return column![text(texto_vacio_recientes())].spacing(10).into();
    }

    tarjetas.push(
        button(
            container(
                column![
                    iconos::vista(Icono::Mas, 22, tema::SECUNDARIO),
                    text("Suelta un video en cualquier lugar para abrirlo")
                        .size(11)
                        .color(tema::SECUNDARIO),
                    text("MP4 · MKV · MOV · WEBM")
                        .font(tema::MONO)
                        .size(9)
                        .color(tema::DIM),
                ]
                .spacing(10)
                .align_x(Horizontal::Center),
            )
            .width(196)
            .height(196)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center),
        )
        .style(tema::boton_secundario)
        .on_press(MensajeInicio::AbrirVideo)
        .width(196)
        .into(),
    );

    let mut filas: Vec<Element<'_, MensajeInicio>> = Vec::new();
    let mut actual: Vec<Element<'_, MensajeInicio>> = Vec::new();
    for tarjeta in tarjetas {
        actual.push(tarjeta);
        if actual.len() == 4 {
            filas.push(row(actual).spacing(12).into());
            actual = Vec::new();
        }
    }
    if !actual.is_empty() {
        filas.push(row(actual).spacing(12).into());
    }
    scrollable(column(filas).spacing(12))
        .height(Length::Fill)
        .into()
}

/// `Ver todos` aparece solo cuando la lista supera las tres entradas que se
/// muestran por defecto: con cero recientes sería una decisión muerta.
pub fn mostrar_ver_todos(recientes: &[Reciente]) -> bool {
    recientes.len() > 3
}

fn vista_aviso(aviso: &AvisoRecovery) -> Element<'_, MensajeInicio> {
    container(
        row![
            container("").width(7).height(7).style(tema::marca_solida),
            column![
                text(aviso.texto.clone()).size(12),
                text("Descartar borra la copia de recuperación.")
                    .font(tema::MONO)
                    .size(9)
                    .color(tema::DIM),
            ]
            .spacing(3),
            container("").width(Length::Fill),
            button("Recuperar")
                .style(tema::boton_primario)
                .on_press(MensajeInicio::Recuperar),
            button("Descartar")
                .style(tema::boton_secundario)
                .on_press(MensajeInicio::DescartarRecovery),
        ]
        .spacing(14)
        .align_y(Vertical::Center),
    )
    .style(tema::wash)
    .width(Length::Fill)
    .padding([12, 14])
    .into()
}

pub fn view(estado: &EstadoInicio) -> Element<'_, MensajeInicio> {
    let lateral = container(
        column![
            button(
                row![
                    iconos::vista(Icono::Nuevo, 16, Color::WHITE),
                    text("Nuevo archivo"),
                ]
                .spacing(8),
            )
            .style(tema::boton_primario)
            .on_press(MensajeInicio::AbrirVideo)
            .width(Length::Fill)
            .height(38),
            button(
                row![
                    iconos::vista(Icono::Abrir, 16, tema::TEXTO),
                    text("Abrir..."),
                ]
                .spacing(8),
            )
            .style(tema::boton_secundario)
            .on_press(MensajeInicio::AbrirProyecto)
            .width(Length::Fill)
            .height(34),
            container("")
                .height(1)
                .width(Length::Fill)
                .style(tema::hairline),
            text("ESPACIO").font(tema::MONO).size(9).color(tema::DIM),
            container(
                row![
                    iconos::vista(Icono::Inicio, 15, tema::SECUNDARIO),
                    text("Inicio").size(12),
                ]
                .spacing(9),
            )
            .width(Length::Fill)
            .padding([8, 10])
            .style(tema::sunk),
            text("El espacio de trabajo se abre al soltar un video.")
                .size(10)
                .color(tema::DIM),
            container("").height(Length::Fill),
            container("")
                .height(1)
                .width(Length::Fill)
                .style(tema::hairline),
            text("Sitio web · GitHub")
                .font(tema::MONO)
                .size(9)
                .color(tema::DIM),
        ]
        .spacing(10),
    )
    .width(216)
    .height(Length::Fill)
    .padding(iced::Padding {
        top: 20.0,
        right: 14.0,
        bottom: 16.0,
        left: 14.0,
    })
    .style(tema::lateral);

    let cabecera = row![
        container(text("01").font(tema::MONO).size(11))
            .width(24)
            .height(24)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .style(tema::marca_solida),
        text("Rusty Open Dialogue").size(18),
        container("")
            .height(1)
            .width(Length::Fill)
            .style(tema::hairline),
        text("v0.9 · 1600×900")
            .font(tema::MONO)
            .size(9)
            .color(tema::DIM),
    ]
    .spacing(12)
    .align_y(Vertical::Center);

    let mut principal = column![cabecera].spacing(0);
    if let Some(aviso) = estado.aviso.as_ref() {
        principal = principal.push(container(vista_aviso(aviso)).padding([16, 0]));
    }
    if let Some(mensaje) = estado.mensaje.as_ref() {
        principal = principal
            .push(container(text(mensaje.clone()).size(11).color(tema::ACENTO)).padding([12, 0]));
    }
    let titulo = row![
        text("Reciente").size(15),
        text(format!("{:02}", estado.recientes.len()))
            .font(tema::MONO)
            .size(10)
            .color(tema::DIM),
        button(
            text(if estado.expandido {
                "Ver menos"
            } else {
                "Ver todos"
            })
            .size(11)
            .color(tema::ACENTO)
        )
        .style(tema::boton_menu)
        .on_press_maybe(mostrar_ver_todos(&estado.recientes).then_some(MensajeInicio::VerTodos)),
    ]
    .spacing(9)
    .align_y(Vertical::Bottom);
    principal = principal
        .push(container(titulo).padding(iced::Padding {
            top: 24.0,
            right: 0.0,
            bottom: 12.0,
            left: 0.0,
        }))
        .push(vista_recientes(estado))
        .push(
            container(
                text("Los recientes son rutas .cdp. La miniatura se guarda con el proyecto.")
                    .font(tema::MONO)
                    .size(9)
                    .color(tema::DIM),
            )
            .padding([12, 0]),
        );

    let main = container(principal)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::Padding {
            top: 30.0,
            right: 38.0,
            bottom: 20.0,
            left: 38.0,
        });

    row![lateral, main].height(Length::Fill).into()
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
    fn cantidad_visible_respeta_el_tope_y_el_expandido() {
        assert_eq!(super::cantidad_visible(0, false), 0);
        assert_eq!(super::cantidad_visible(7, false), 3);
        assert_eq!(super::cantidad_visible(7, true), 7);
    }

    #[test]
    fn resumen_reciente_usa_singular_y_plural() {
        let reciente = |lineas, hablantes, fecha| crate::recientes::Reciente {
            ruta: PathBuf::from("a.cdp"),
            lineas,
            hablantes,
            fecha,
        };
        assert_eq!(
            super::resumen_reciente(&reciente(52, 2, "hace 2 h".into())),
            "52 líneas · 2 hablantes · hace 2 h"
        );
        assert_eq!(
            super::resumen_reciente(&reciente(1, 1, "hoy".into())),
            "1 línea · 1 hablante · hoy"
        );
    }

    #[test]
    fn texto_vacio_recientes_es_el_literal_de_la_spec() {
        assert_eq!(
            super::texto_vacio_recientes(),
            "Todavía no hay proyectos. El primero aparece aquí al guardar."
        );
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
