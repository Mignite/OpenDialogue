#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use iced::{Element, Subscription, Task};

use rod_app::editor_app::{self, EditorApp, MensajeEditor};
use rod_app::inicio::{self, EstadoInicio, MensajeInicio, Salida};
use rod_app::project_io::abrir_proyecto;
use rod_app::{recientes, recovery_io};

/// Las dos pantallas de la app. El editor solo existe cuando se abrió un
/// video o un proyecto: el arranque no construye mpv.
enum Pantalla {
    Inicio(EstadoInicio),
    Editor(Box<EditorApp>),
}

#[derive(Debug, Clone)]
enum Mensaje {
    Inicio(MensajeInicio),
    Editor(MensajeEditor),
    ArchivoSoltado(PathBuf),
}

fn nuevo() -> Pantalla {
    let ahora = recovery_io::ms_ahora();
    let ruta_indice = recientes::ruta_indice();
    let rutas = recientes::leer_indice(&ruta_indice);
    let total = rutas.len();
    // Las entradas cuyo `.cdp` ya no abre se omiten y se quitan del índice:
    // no se arrastra una lista de proyectos muertos entre sesiones.
    let (visibles, validas): (Vec<_>, Vec<_>) = rutas
        .into_iter()
        .filter_map(|ruta| recientes::metadata(&ruta, ahora).map(|meta| (meta, ruta)))
        .unzip();
    if validas.len() != total {
        let _ = recientes::escribir_indice(&ruta_indice, &validas);
    }
    let aviso = recovery_io::pendiente(&recovery_io::ruta(), ahora);
    Pantalla::Inicio(inicio::nuevo(visibles, aviso))
}

/// Abre el editor o devuelve al menú con el motivo del fallo. Nunca deja un
/// editor a medias: si el video no abre, el usuario sigue en el menú.
fn abrir(pantalla: &mut Pantalla, editor: Result<EditorApp, String>) {
    match editor {
        Ok(editor) => *pantalla = Pantalla::Editor(Box::new(editor)),
        Err(error) => {
            if let Pantalla::Inicio(estado) = pantalla {
                estado.mensaje = Some(error);
            }
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
            let aviso = match pantalla {
                Pantalla::Inicio(estado) => estado.aviso.take(),
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
                abrir(
                    pantalla,
                    EditorApp::desde_proyecto(recovery.proyecto, ruta, posicion),
                );
                let _ = recovery_io::escribir(&recovery_io::ruta(), None);
            }
            Task::none()
        }
        Salida::DescartarRecovery => {
            let _ = recovery_io::escribir(&recovery_io::ruta(), None);
            Task::none()
        }
    }
}

fn actualizar(pantalla: &mut Pantalla, mensaje: Mensaje) -> Task<Mensaje> {
    match mensaje {
        Mensaje::ArchivoSoltado(ruta) => {
            let (tarea, salida) = match pantalla {
                Pantalla::Inicio(estado) => {
                    inicio::update(estado, MensajeInicio::ArchivoSoltado(ruta))
                }
                Pantalla::Editor(_) => (Task::none(), None),
            };
            if let Some(salida) = salida {
                return despachar_salida(pantalla, salida);
            }
            tarea.map(Mensaje::Inicio)
        }
        Mensaje::Inicio(mensaje) => {
            let (tarea, salida) = match pantalla {
                Pantalla::Inicio(estado) => inicio::update(estado, mensaje),
                Pantalla::Editor(_) => (Task::none(), None),
            };
            if let Some(salida) = salida {
                return despachar_salida(pantalla, salida);
            }
            tarea.map(Mensaje::Inicio)
        }
        Mensaje::Editor(mensaje) => match pantalla {
            Pantalla::Editor(editor) => editor_app::update(editor, mensaje).map(Mensaje::Editor),
            Pantalla::Inicio(_) => Task::none(),
        },
    }
}

fn vista(pantalla: &Pantalla) -> Element<'_, Mensaje> {
    match pantalla {
        Pantalla::Inicio(estado) => inicio::view(estado).map(Mensaje::Inicio),
        Pantalla::Editor(editor) => editor_app::view(editor).map(Mensaje::Editor),
    }
}

fn suscripcion(pantalla: &Pantalla) -> Subscription<Mensaje> {
    let arrastre = iced::window::events().filter_map(|(_id, evento)| match evento {
        iced::window::Event::FileDropped(ruta) => Some(Mensaje::ArchivoSoltado(ruta)),
        _ => None,
    });
    match pantalla {
        Pantalla::Inicio(_) => arrastre,
        Pantalla::Editor(editor) => Subscription::batch([
            arrastre,
            editor_app::subscription(editor).map(Mensaje::Editor),
        ]),
    }
}

fn main() -> iced::Result {
    iced::application(nuevo, actualizar, vista)
        .title("Rusty Open Dialogue")
        .window_size([1600.0, 900.0])
        .subscription(suscripcion)
        .run()
}

#[cfg(test)]
mod tests {
    #[test]
    fn arranca_en_inicio_sin_abrir_player() {
        assert!(matches!(super::nuevo(), super::Pantalla::Inicio(_)));
    }

    #[test]
    fn video_invalido_no_cambia_de_pantalla() {
        let mut pantalla = super::nuevo();
        let _ = super::actualizar(
            &mut pantalla,
            super::Mensaje::ArchivoSoltado("x.txt".into()),
        );
        assert!(matches!(pantalla, super::Pantalla::Inicio(_)));
    }
}
