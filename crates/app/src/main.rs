#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

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
        let mut pantalla = pantalla_de_prueba();
        let _ = super::actualizar(
            &mut pantalla,
            super::Mensaje::ArchivoSoltado("x.txt".into()),
        );
        assert!(matches!(pantalla, super::Pantalla::Inicio(_)));
    }
}
