#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Stack, button, column, container, mouse_area, row, text};
use iced::window::Direction;
use iced::{Element, Length, Subscription, Task};

use rod_app::comandos::{self, Comando};
use rod_app::dialogos::{self, RespuestaDialogo};
use rod_app::editor_app::{self, EditorApp, MensajeEditor};
use rod_app::iconos::{self, Icono};
use rod_app::inicio::{self, EstadoInicio, MensajeInicio, Salida};
use rod_app::menu;
use rod_app::project_io::abrir_proyecto;
use rod_app::tarea::Tarea;
use rod_app::tema;
use rod_app::{recientes, recovery_io};

/// Las dos pantallas de la app. El editor solo existe cuando se abrió un
/// video o un proyecto: el arranque no construye mpv.
enum Pantalla {
    Inicio(EstadoInicio),
    Editor(Box<EditorApp>),
}

/// Estado de la ventana además de la pantalla visible. El id se guarda en el
/// `boot` para que `window::drag` se pueda pedir en el mismo evento del ratón.
struct AppState {
    pantalla: Pantalla,
    ventana: Option<iced::window::Id>,
    /// Diálogos nativos resueltos en otro hilo; se drenan en el update.
    tx_dialogos: std::sync::mpsc::Sender<RespuestaDialogo>,
    rx_dialogos: std::sync::mpsc::Receiver<RespuestaDialogo>,
}

#[derive(Debug, Clone)]
enum Mensaje {
    Inicio(MensajeInicio),
    Editor(MensajeEditor),
    Comando(Comando),
    CargarRespuesta,
    RespuestaDialogo(RespuestaDialogo),
    ArchivoSoltado(PathBuf),
    VentanaId(Option<iced::window::Id>),
    ArrastrarVentana,
    MinimizarVentana,
    MaximizarVentana,
    CerrarVentana,
    Redimensionar(Direction),
}

fn boot() -> (AppState, Task<Mensaje>) {
    let (tx_dialogos, rx_dialogos) = std::sync::mpsc::channel();
    (
        AppState {
            pantalla: nuevo(),
            ventana: None,
            tx_dialogos,
            rx_dialogos,
        },
        iced::window::latest().map(Mensaje::VentanaId),
    )
}

/// Ejecuta los efectos descritos por `Tarea` fuera de la vista.
fn despachar_tarea(tarea: Tarea, tx: &std::sync::mpsc::Sender<RespuestaDialogo>) -> Task<Mensaje> {
    match tarea {
        Tarea::DialogoVideo => dialogos::pedir_video(tx.clone()),
        Tarea::DialogoProyecto => dialogos::pedir_proyecto(tx.clone()),
        Tarea::DialogoCarpeta => dialogos::pedir_carpeta(tx.clone()),
        Tarea::DialogoGuardar => dialogos::pedir_guardar(tx.clone()),
        Tarea::AbrirVideoEditor => dialogos::pedir_video_editor(tx.clone()),
        Tarea::CargarSrt => dialogos::pedir_srt(tx.clone()),
        Tarea::CargarAss => dialogos::pedir_ass(tx.clone()),
        Tarea::GuardarJson => dialogos::pedir_json(tx.clone()),
        Tarea::GuardarAss => dialogos::pedir_ass_guardar(tx.clone()),
        Tarea::ImportarSrt => dialogos::pedir_autosubs_srt(tx.clone()),
        Tarea::ImportarTxt => dialogos::pedir_autosubs_txt(tx.clone()),
        Tarea::EnfocarTexto | Tarea::Ninguna => {}
    }
    Task::none()
}

fn nuevo() -> Pantalla {
    nuevo_con(&recientes::ruta_indice(), &recovery_io::ruta())
}

/// Las rutas entran como parámetro para que los tests no toquen el índice ni
/// el recovery reales del usuario.
fn nuevo_con(ruta_indice: &Path, ruta_recovery: &Path) -> Pantalla {
    let ahora = recovery_io::ms_ahora();
    let rutas = recientes::leer_indice(ruta_indice);
    let total = rutas.len();
    // Las entradas cuyo `.cdp` ya no abre se omiten y se quitan del índice:
    // no se arrastra una lista de proyectos muertos entre sesiones.
    let (visibles, validas): (Vec<_>, Vec<_>) = rutas
        .into_iter()
        .filter_map(|ruta| recientes::metadata(&ruta, ahora).map(|meta| (meta, ruta)))
        .unzip();
    if validas.len() != total {
        let _ = recientes::escribir_indice(ruta_indice, &validas);
    }
    let aviso = recovery_io::pendiente(ruta_recovery, ahora);
    Pantalla::Inicio(inicio::nuevo(visibles, aviso))
}

/// Abre el editor o devuelve al menú con el motivo del fallo. Nunca deja un
/// editor a medias: si el video no abre, el usuario sigue en el menú.
fn abrir(pantalla: &mut Pantalla, editor: Result<EditorApp, String>) -> bool {
    match editor {
        Ok(editor) => {
            *pantalla = Pantalla::Editor(Box::new(editor));
            true
        }
        Err(error) => {
            if let Pantalla::Inicio(estado) = pantalla {
                estado.mensaje = Some(error);
            }
            false
        }
    }
}

fn despachar_salida(pantalla: &mut Pantalla, salida: Salida) -> Task<Mensaje> {
    match salida {
        Salida::Video(ruta) => {
            let ruta = ruta.to_string_lossy().into_owned();
            abrir(pantalla, EditorApp::desde_video(&ruta));
            Task::none()
        }
        Salida::Proyecto(ruta) => {
            let editor = abrir_proyecto(&ruta)
                .map_err(|error| error.to_string())
                .and_then(|proyecto| {
                    EditorApp::desde_proyecto(proyecto.clone(), Some(ruta), proyecto.playhead)
                });
            abrir(pantalla, editor);
            Task::none()
        }
        Salida::Recuperar => {
            // Clonar: si el video no abre, el aviso y el archivo tienen que
            // seguir ahí para que el usuario pueda intentarlo de nuevo.
            let aviso = match pantalla {
                Pantalla::Inicio(estado) => estado.aviso.clone(),
                Pantalla::Editor(_) => None,
            };
            if let Some(aviso) = aviso {
                let recovery = aviso.recovery;
                let ruta = if recovery.ruta_proyecto.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(&recovery.ruta_proyecto))
                };
                let posicion = recovery.proyecto.playhead;
                let abrio = abrir(
                    pantalla,
                    EditorApp::desde_proyecto(recovery.proyecto, ruta, posicion),
                );
                if abrio {
                    if let Pantalla::Inicio(estado) = pantalla {
                        estado.aviso = None;
                    }
                    let _ = recovery_io::escribir(&recovery_io::ruta(), None);
                }
            }
            Task::none()
        }
        Salida::DescartarRecovery => {
            let _ = recovery_io::escribir(&recovery_io::ruta(), None);
            Task::none()
        }
    }
}

fn actualizar(state: &mut AppState, mensaje: Mensaje) -> Task<Mensaje> {
    let pantalla = &mut state.pantalla;
    match mensaje {
        Mensaje::CargarRespuesta => {
            let pendientes: Vec<_> = state.rx_dialogos.try_iter().collect();
            let mut tarea_total = Task::none();
            for respuesta in pendientes {
                tarea_total = Task::batch([
                    tarea_total,
                    actualizar(state, Mensaje::RespuestaDialogo(respuesta)),
                ]);
            }
            tarea_total
        }
        Mensaje::RespuestaDialogo(respuesta) => match pantalla {
            Pantalla::Inicio(_) => {
                let mensaje = match respuesta {
                    RespuestaDialogo::Video(ruta) => Some(MensajeInicio::DialogoVideo(ruta)),
                    RespuestaDialogo::Proyecto(ruta) => Some(MensajeInicio::DialogoProyecto(ruta)),
                    _ => None,
                };
                match mensaje {
                    Some(mensaje) => actualizar(state, Mensaje::Inicio(mensaje)),
                    None => Task::none(),
                }
            }
            Pantalla::Editor(_) => {
                use rod_app::editor_app::MensajeEditor as E;
                let mensaje = match respuesta {
                    RespuestaDialogo::VideoEditor(ruta) => E::DialogoVideoEditor(ruta),
                    RespuestaDialogo::Proyecto(ruta) => E::DialogoProyecto(ruta),
                    RespuestaDialogo::Srt(ruta) => E::DialogoSubtitulos(ruta, false),
                    RespuestaDialogo::Ass(ruta) => E::DialogoSubtitulos(ruta, true),
                    RespuestaDialogo::AssGuardar(ruta) => E::DialogoAssExportar(ruta),
                    RespuestaDialogo::Guardar(ruta) => E::DialogoGuardarProyecto(ruta),
                    RespuestaDialogo::Carpeta(ruta) => E::DialogoCarpetaExportar(ruta),
                    RespuestaDialogo::Json(ruta) => E::DialogoJson(ruta),
                    RespuestaDialogo::AutosubsSrt(ruta) => E::DialogoAutosubsSrt(ruta),
                    RespuestaDialogo::AutosubsTxt(ruta) => E::DialogoAutosubsTxt(ruta),
                    RespuestaDialogo::Video(_) => return Task::none(),
                };
                actualizar(state, Mensaje::Editor(mensaje))
            }
        },
        Mensaje::VentanaId(id) => {
            state.ventana = id;
            Task::none()
        }
        Mensaje::ArrastrarVentana => match state.ventana {
            Some(id) => iced::window::drag(id),
            None => Task::none(),
        },
        Mensaje::Redimensionar(direccion) => match state.ventana {
            Some(id) => iced::window::drag_resize(id, direccion),
            None => Task::none(),
        },
        Mensaje::MinimizarVentana => match state.ventana {
            Some(id) => iced::window::minimize(id, true),
            None => Task::none(),
        },
        Mensaje::MaximizarVentana => match state.ventana {
            Some(id) => iced::window::is_maximized(id)
                .then(move |maximizada| iced::window::maximize(id, !maximizada)),
            None => Task::none(),
        },
        Mensaje::CerrarVentana => match state.ventana {
            Some(id) => iced::window::close(id),
            None => Task::none(),
        },
        Mensaje::Comando(comando) => match pantalla {
            Pantalla::Editor(editor) => match comando {
                Comando::NuevoProyecto | Comando::AbrirVideo => despachar_tarea(
                    editor_app::update(editor, MensajeEditor::AbrirVideo),
                    &state.tx_dialogos,
                ),
                Comando::AbrirProyecto => despachar_tarea(
                    editor_app::update(editor, MensajeEditor::AbrirProyecto),
                    &state.tx_dialogos,
                ),
                Comando::Guardar | Comando::GuardarComo => {
                    let mensaje = if comando == Comando::GuardarComo {
                        MensajeEditor::GuardarProyectoComo
                    } else {
                        MensajeEditor::GuardarProyecto
                    };
                    despachar_tarea(editor_app::update(editor, mensaje), &state.tx_dialogos)
                }
                _ => despachar_tarea(
                    editor_app::update(editor, MensajeEditor::Comando(comando)),
                    &state.tx_dialogos,
                ),
            },
            Pantalla::Inicio(estado) => {
                if comando == Comando::Ayuda {
                    estado.mensaje = Some(
                        "Atajos: Ctrl+S guardar, Ctrl+O abrir, Space reproducir, ←/→ saltar, ? ayuda"
                            .into(),
                    );
                    return Task::none();
                }
                let mensaje_inicio = match comando {
                    Comando::NuevoProyecto | Comando::AbrirVideo => MensajeInicio::AbrirVideo,
                    Comando::AbrirProyecto => MensajeInicio::AbrirProyecto,
                    _ => return Task::none(),
                };
                let (tarea, salida) = inicio::update(estado, mensaje_inicio);
                if let Some(salida) = salida {
                    return despachar_salida(pantalla, salida);
                }
                despachar_tarea(tarea, &state.tx_dialogos)
            }
        },
        Mensaje::ArchivoSoltado(ruta) => {
            let (tarea, salida) = match pantalla {
                Pantalla::Inicio(estado) => {
                    inicio::update(estado, MensajeInicio::ArchivoSoltado(ruta))
                }
                Pantalla::Editor(_) => (Tarea::Ninguna, None),
            };
            if let Some(salida) = salida {
                return despachar_salida(pantalla, salida);
            }
            despachar_tarea(tarea, &state.tx_dialogos)
        }
        Mensaje::Inicio(mensaje) => {
            let (tarea, salida) = match pantalla {
                Pantalla::Inicio(estado) => inicio::update(estado, mensaje),
                Pantalla::Editor(_) => (Tarea::Ninguna, None),
            };
            if let Some(salida) = salida {
                return despachar_salida(pantalla, salida);
            }
            despachar_tarea(tarea, &state.tx_dialogos)
        }
        Mensaje::Editor(mensaje) => match pantalla {
            Pantalla::Editor(editor) => {
                despachar_tarea(editor_app::update(editor, mensaje), &state.tx_dialogos)
            }
            Pantalla::Inicio(_) => Task::none(),
        },
    }
}

/// Franja invisible que redimensiona la ventana sin marco nativo.
fn borde(direccion: Direction, horizontal: bool, al_final: bool) -> Element<'static, Mensaje> {
    let area = mouse_area(
        container("")
            .width(if horizontal {
                Length::Fill
            } else {
                Length::from(4.0)
            })
            .height(if horizontal {
                Length::from(4.0)
            } else {
                Length::Fill
            }),
    )
    .on_press(Mensaje::Redimensionar(direccion));
    container(area)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(if horizontal {
            iced::alignment::Horizontal::Left
        } else if al_final {
            iced::alignment::Horizontal::Right
        } else {
            iced::alignment::Horizontal::Left
        })
        .align_y(if horizontal && al_final {
            iced::alignment::Vertical::Bottom
        } else {
            iced::alignment::Vertical::Top
        })
        .into()
}

fn vista(state: &AppState) -> Element<'_, Mensaje> {
    let pantalla = &state.pantalla;
    let (en_inicio, estado, contenido) = match pantalla {
        Pantalla::Inicio(estado) => (
            true,
            "Sin proyecto · v0.9",
            inicio::view(estado).map(Mensaje::Inicio),
        ),
        Pantalla::Editor(editor) => (
            false,
            "Proyecto abierto",
            editor_app::view(editor).map(Mensaje::Editor),
        ),
    };
    let marca = container(text("R").font(tema::DISPLAY).size(11))
        .width(18)
        .height(18)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .style(tema::marca_solida);
    let barra = container(
        row![
            mouse_area(
                row![marca, text("RustyDialogue").font(tema::MONO).size(10)]
                    .spacing(8)
                    .align_y(Vertical::Center)
            )
            .on_press(Mensaje::ArrastrarVentana),
            menu::barra(en_inicio, Mensaje::Comando),
            mouse_area(container("").width(Length::Fill).height(30))
                .on_press(Mensaje::ArrastrarVentana),
            text(estado).font(tema::MONO).size(9).color(tema::DIM),
            button(iconos::vista(Icono::Menos, 12, tema::SECUNDARIO))
                .style(tema::boton_menu)
                .on_press(Mensaje::MinimizarVentana)
                .width(28)
                .height(30),
            button(iconos::vista(Icono::Ajustar, 12, tema::SECUNDARIO))
                .style(tema::boton_menu)
                .on_press(Mensaje::MaximizarVentana)
                .width(28)
                .height(30),
            button(iconos::vista(Icono::Cerrar, 12, tema::TEXTO))
                .style(tema::boton_menu)
                .on_press(Mensaje::CerrarVentana)
                .width(28)
                .height(30),
        ]
        .spacing(8)
        .align_y(Vertical::Center),
    )
    .height(30)
    .padding([0, 10])
    .style(tema::fondo);

    let base = container(column![barra, contenido])
        .style(tema::fondo)
        .width(Length::Fill)
        .height(Length::Fill);
    Stack::new()
        .push(base)
        .push(borde(Direction::North, true, false))
        .push(borde(Direction::South, true, true))
        .push(borde(Direction::West, false, false))
        .push(borde(Direction::East, false, true))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Puente temporal mientras el binario sigue siendo de iced: traduce la tecla
/// y los modificadores al tipo que espera el registro de comandos de egui.
fn tecla_egui(key: iced::keyboard::Key) -> Option<egui::Key> {
    use iced::keyboard::{Key, key::Named};
    match key {
        Key::Character(c) => egui::Key::from_name(&c),
        Key::Named(Named::Space) => Some(egui::Key::Space),
        Key::Named(Named::ArrowLeft) => Some(egui::Key::ArrowLeft),
        Key::Named(Named::ArrowRight) => Some(egui::Key::ArrowRight),
        Key::Named(Named::ArrowUp) => Some(egui::Key::ArrowUp),
        Key::Named(Named::ArrowDown) => Some(egui::Key::ArrowDown),
        Key::Named(Named::Delete) => Some(egui::Key::Delete),
        Key::Named(Named::Enter) => Some(egui::Key::Enter),
        _ => None,
    }
}

fn mods_egui(mods: iced::keyboard::Modifiers) -> egui::Modifiers {
    use iced::keyboard::Modifiers as M;
    egui::Modifiers {
        ctrl: mods.contains(M::CTRL),
        command: mods.contains(M::LOGO),
        mac_cmd: cfg!(target_os = "macos") && mods.contains(M::LOGO),
        shift: mods.shift(),
        alt: mods.alt(),
    }
}

fn suscripcion(state: &AppState) -> Subscription<Mensaje> {
    let pantalla = &state.pantalla;
    let arrastre = iced::window::events().filter_map(|(_id, evento)| match evento {
        iced::window::Event::FileDropped(ruta) => Some(Mensaje::ArchivoSoltado(ruta)),
        _ => None,
    });
    let teclado = iced::keyboard::listen().filter_map(|evento| match evento {
        iced::keyboard::Event::KeyPressed { key, modifiers, .. } => {
            let (key, mods) = (tecla_egui(key)?, mods_egui(modifiers));
            comandos::desde_input(key, mods).map(Mensaje::Comando)
        }
        _ => None,
    });
    // Un tick por frame drena el canal de diálogos. `window::frames` ya está
    // en el árbol de dependencias; `iced::time::every` exigiría añadir tokio.
    let respuestas = iced::window::frames().map(|_| Mensaje::CargarRespuesta);
    match pantalla {
        Pantalla::Inicio(_) => Subscription::batch([arrastre, teclado, respuestas]),
        Pantalla::Editor(editor) => Subscription::batch([
            arrastre,
            teclado,
            respuestas,
            editor_app::subscription(editor).map(Mensaje::Editor),
        ]),
    }
}

fn main() -> iced::Result {
    iced::application(boot, actualizar, vista)
        .title("RustyDialogue")
        .default_font(tema::CUERPO)
        .decorations(false)
        .window_size([1600.0, 900.0])
        .subscription(suscripcion)
        .run()
}

#[cfg(test)]
mod tests {
    use rod_app::inicio::{self, Salida};

    fn pantalla_de_prueba() -> super::Pantalla {
        let dir = std::env::temp_dir().join(format!("rod-router-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        super::nuevo_con(&dir.join("recientes.json"), &dir.join("recovery.json"))
    }

    #[test]
    fn recuperar_con_video_inexistente_conserva_el_aviso() {
        let proyecto = rod_core::Proyecto::nuevo("C:/no/existe/clip.mp4");
        let recovery = rod_core::construir_recovery(proyecto, "C:/no/existe.cdp", 1_000.0);
        let texto = rod_core::formatear_recovery(&recovery, "existe.cdp", 1_000.0);
        let aviso = rod_app::recovery_io::AvisoRecovery { recovery, texto };
        let mut pantalla = super::Pantalla::Inicio(inicio::nuevo(Vec::new(), Some(aviso)));
        let _ = super::despachar_salida(&mut pantalla, Salida::Recuperar);
        match pantalla {
            super::Pantalla::Inicio(estado) => assert!(estado.aviso.is_some()),
            super::Pantalla::Editor(_) => panic!("abrió un editor con video inexistente"),
        }
    }

    #[test]
    fn arranca_en_inicio_sin_abrir_player() {
        assert!(matches!(pantalla_de_prueba(), super::Pantalla::Inicio(_)));
    }

    #[test]
    fn video_invalido_no_cambia_de_pantalla() {
        let (tx_dialogos, rx_dialogos) = std::sync::mpsc::channel();
        let mut state = super::AppState {
            pantalla: pantalla_de_prueba(),
            ventana: None,
            tx_dialogos,
            rx_dialogos,
        };
        let _ = super::actualizar(&mut state, super::Mensaje::ArchivoSoltado("x.txt".into()));
        assert!(matches!(state.pantalla, super::Pantalla::Inicio(_)));
    }
}
