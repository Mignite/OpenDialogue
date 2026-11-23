//! Pantalla de arranque: arrastrar video, abrir proyecto, recientes y recovery.
//!
//! No conoce mpv ni el editor: solo decide qué quiere hacer el usuario.

use std::path::{Path, PathBuf};

use iced::alignment::{Horizontal, Vertical};
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
    let entries = estado
        .recientes
        .iter()
        .take(cantidad)
        .map(|reciente| {
            let miniatura: Element<'_, MensajeInicio> =
                match miniatura_en(&thumbnail::carpeta_cache(), reciente) {
                    Some(ruta) => container(
                        image(iced::widget::image::Handle::from_path(ruta))
                            .width(96)
                            .height(54),
                    )
                    .width(96)
                    .height(54)
                    .style(tema::sunk)
                    .into(),
                    None => container(text("[ > ]").font(tema::MONO).size(18))
                        .width(96)
                        .height(54)
                        .style(tema::sunk)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                        .into(),
                };
            button(
                container(
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
                            text(resumen_reciente(reciente))
                                .font(tema::MONO)
                                .size(10)
                                .color(tema::SECUNDARIO),
                        ]
                        .spacing(4),
                    ]
                    .spacing(8),
                )
                .width(Length::Fill),
            )
            .style(tema::boton_secundario)
            .on_press(MensajeInicio::AbrirReciente(reciente.ruta.clone()))
            .width(Length::Fill)
            .into()
        })
        .collect::<Vec<Element<'_, MensajeInicio>>>();

    let titulo = text("Recientes")
        .font(tema::MONO)
        .size(12)
        .width(Length::Fill);
    let contador = text(format!("{:02}", estado.recientes.len()))
        .font(tema::MONO)
        .size(10)
        .color(tema::MOSTAZA);
    let cabecera = if mostrar_ver_todos(&estado.recientes) {
        row![
            titulo,
            contador,
            button(if estado.expandido {
                "Ver menos"
            } else {
                "Ver todos"
            })
            .style(tema::boton_secundario)
            .on_press(MensajeInicio::VerTodos),
        ]
        .spacing(9)
        .align_y(Vertical::Bottom)
    } else {
        row![titulo, contador].spacing(9).align_y(Vertical::Bottom)
    };

    let mut columna = column![cabecera];
    if entries.is_empty() {
        columna = columna.push(text(texto_vacio_recientes()));
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
    container(
        column![
            row![
                text("QUEDÓ SIN GUARDAR")
                    .font(tema::MONO)
                    .size(9)
                    .color(tema::MOSTAZA),
                container(text("RECOVERY").font(tema::MONO).size(9).color(tema::DIM))
                    .width(Length::Fill)
                    .align_x(Horizontal::Right),
            ],
            text(aviso.texto.clone()).size(12),
            row![
                button("Recuperar")
                    .style(tema::boton_primario)
                    .on_press(MensajeInicio::Recuperar),
                button("Descartar")
                    .style(tema::boton_secundario)
                    .on_press(MensajeInicio::DescartarRecovery),
            ]
            .spacing(7),
            text("Descartar borra la copia de recuperación.")
                .font(tema::MONO)
                .size(9)
                .color(tema::DIM),
        ]
        .spacing(9),
    )
    .style(tema::wash)
    .width(Length::Fill)
    .padding([12, 13])
    .into()
}

fn vista_flujo() -> Element<'static, MensajeInicio> {
    let paso = |numero: &'static str, titulo: &'static str, detalle: &'static str| {
        row![
            text(numero).font(tema::MONO).size(9).color(tema::MOSTAZA),
            column![
                text(titulo).size(11.5),
                text(detalle)
                    .font(tema::MONO)
                    .size(9)
                    .color(tema::SECUNDARIO),
            ]
            .spacing(1),
        ]
        .spacing(8)
    };
    column![
        text("FLUJO").font(tema::MONO).size(9).color(tema::DIM),
        paso("01", "Suelta el video", "MP4, MKV, MOV o WebM"),
        paso(
            "02",
            "Marca y sincroniza",
            "una línea por caption en el timeline"
        ),
        paso(
            "03",
            "Exporta el .ass",
            "el preview es el archivo, no una aproximación"
        ),
    ]
    .spacing(8)
    .into()
}

fn marca_registro() -> Element<'static, MensajeInicio> {
    container("").width(18).height(18).style(tema::marca).into()
}

pub fn view(estado: &EstadoInicio) -> Element<'_, MensajeInicio> {
    let cabecera = row![
        text("01 / ABRIR")
            .font(tema::MONO)
            .size(10)
            .color(tema::MOSTAZA),
        container("")
            .height(1)
            .width(Length::Fill)
            .style(tema::hairline),
        text("Proyecto nuevo en memoria")
            .font(tema::MONO)
            .size(9)
            .color(tema::DIM),
    ]
    .spacing(12)
    .align_y(Vertical::Center);

    let titular = column![
        text("Empezar")
            .font(tema::DISPLAY)
            .size(58)
            .line_height(0.86),
        text("CON UN VIDEO")
            .font(tema::DISPLAY)
            .size(58)
            .line_height(0.86)
            .color(tema::SECUNDARIO),
    ];
    let ayuda = text(
        "Suelta el archivo o ábrelo desde el disco. Nada se escribe hasta el primer Ctrl+S, así que puedes probar sin miedo.",
    )
    .size(12.5)
    .line_height(1.5)
    .color(tema::SECUNDARIO);
    let cabeza = row![titular, ayuda.width(290)]
        .spacing(48)
        .align_y(Vertical::Bottom);

    let meta_superior = row![
        marca_registro(),
        text("TOMA 001 / SIN PROYECTO")
            .font(tema::MONO)
            .size(9)
            .color(tema::DIM),
        container(
            text("00:00:00:00")
                .font(tema::MONO)
                .size(9)
                .color(tema::MOSTAZA)
        )
        .width(Length::Fill)
        .align_x(Horizontal::Right),
        marca_registro(),
    ]
    .spacing(12)
    .align_y(Vertical::Center);

    let meta_inferior = row![
        marca_registro(),
        text("1920 × 1080 · 25 FPS")
            .font(tema::MONO)
            .size(9)
            .color(tema::DIM),
        container(
            text("PREVIEW = .ASS EXPORTADO")
                .font(tema::MONO)
                .size(9)
                .color(tema::MOSTAZA)
        )
        .width(Length::Fill)
        .align_x(Horizontal::Right),
        marca_registro(),
    ]
    .spacing(12)
    .align_y(Vertical::Center);

    let centro = column![
        text("+").size(46).color(tema::MOSTAZA),
        text("ARRASTRA UN VIDEO AQUÍ")
            .font(tema::DISPLAY)
            .size(24)
            .color(tema::TEXTO),
        text("MP4, MKV, MOV o WebM")
            .font(tema::MONO)
            .size(10)
            .color(tema::MOSTAZA),
        // El error ocupa el sitio de la ayuda: nunca hay dos líneas.
        text(
            estado
                .mensaje
                .clone()
                .unwrap_or_else(|| "o abre un proyecto .cdp guardado".into())
        )
        .size(11.5)
        .color(if estado.mensaje.is_some() {
            tema::MOSTAZA
        } else {
            tema::SECUNDARIO
        }),
    ]
    .spacing(9)
    .align_x(Horizontal::Center);

    let zona = container(
        column![
            meta_superior,
            container(centro)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            meta_inferior,
        ]
        .spacing(12)
        .padding(20),
    )
    .style(tema::sunk)
    .width(Length::Fill)
    .height(Length::Fill);

    let acciones = row![
        button("ABRIR VIDEO  →")
            .style(tema::boton_primario)
            .on_press(MensajeInicio::AbrirVideo)
            .height(50),
        button(
            row![
                text("ABRIR PROYECTO").font(tema::MONO).size(11),
                text("Ctrl+O")
                    .font(tema::MONO)
                    .size(9)
                    .color(tema::SECUNDARIO),
            ]
            .spacing(12)
        )
        .style(tema::boton_secundario)
        .on_press(MensajeInicio::AbrirProyecto)
        .height(50),
        container(
            text("El primer Ctrl+S pide dónde guardar\nO suelta un .cdp")
                .font(tema::MONO)
                .size(9)
                .color(tema::DIM)
        )
        .width(Length::Fill)
        .align_x(Horizontal::Right),
    ]
    .spacing(10)
    .align_y(Vertical::Center);

    let principal = container(
        column![cabecera, cabeza, zona, acciones]
            .spacing(14)
            .padding(iced::Padding {
                top: 26.0,
                right: 32.0,
                bottom: 20.0,
                left: 32.0,
            }),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    let mut lateral = column![];
    if let Some(aviso) = estado.aviso.as_ref() {
        lateral = lateral.push(vista_aviso(aviso));
    }
    lateral = lateral
        .push(container(vista_recientes(estado)).height(Length::Fill))
        .push(
            text("La miniatura es el frame del playhead al guardar")
                .font(tema::MONO)
                .size(9)
                .color(tema::DIM),
        )
        .push(vista_flujo());
    let lateral = container(lateral.spacing(16))
        .width(460)
        .height(Length::Fill)
        .padding(iced::Padding {
            top: 20.0,
            right: 24.0,
            bottom: 16.0,
            left: 24.0,
        });

    let rail = container(
        row![
            text("● LISTO · ningún proyecto abierto"),
            text("Suelta un video en cualquier punto de la ventana"),
            text("MPV · LIBASS · ICED 0.14")
                .width(Length::Fill)
                .align_x(Horizontal::Right),
        ]
        .spacing(26)
        .align_y(Vertical::Center)
        .padding([10, 18]),
    )
    .height(60)
    .style(tema::panel);

    column![
        row![
            principal,
            container("")
                .width(1)
                .height(Length::Fill)
                .style(tema::hairline),
            lateral
        ]
        .height(Length::Fill),
        rail,
    ]
    .into()
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
