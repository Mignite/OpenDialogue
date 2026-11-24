#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};

use rod_app::comandos::{self, Comando};
use rod_app::dialogos::{self, RespuestaDialogo};
use rod_app::editor_app::{self, EditorApp, MensajeEditor};
use rod_app::inicio::{self, EstadoInicio, MensajeInicio, Salida};
use rod_app::project_io::abrir_proyecto;
use rod_app::tarea::Tarea;
use rod_app::{recientes, recovery_io, ui};

/// Las dos pantallas. El editor solo existe cuando se abrió un video o un
/// proyecto: el arranque no construye mpv.
enum Pantalla {
    Inicio(EstadoInicio),
    Editor(Box<EditorApp>),
}

struct RustyDialogue {
    pantalla: Pantalla,
    tx_dialogos: Sender<RespuestaDialogo>,
    rx_dialogos: Receiver<RespuestaDialogo>,
}

fn nuevo() -> Pantalla {
    nuevo_con(&recientes::ruta_indice(), &recovery_io::ruta())
}

fn nuevo_con(ruta_indice: &Path, ruta_recovery: &Path) -> Pantalla {
    let ahora = recovery_io::ms_ahora();
    let rutas = recientes::leer_indice(ruta_indice);
    let total = rutas.len();
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

impl RustyDialogue {
    fn nuevo() -> Self {
        let (tx_dialogos, rx_dialogos) = std::sync::mpsc::channel();
        Self {
            pantalla: nuevo(),
            tx_dialogos,
            rx_dialogos,
        }
    }

    /// Ejecuta los efectos que el front-end solo describe.
    fn ejecutar_tarea(&self, tarea: Tarea) {
        let tx = &self.tx_dialogos;
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
    }

    fn abrir(&mut self, editor: Result<EditorApp, String>) -> bool {
        match editor {
            Ok(editor) => {
                self.pantalla = Pantalla::Editor(Box::new(editor));
                true
            }
            Err(error) => {
                if let Pantalla::Inicio(estado) = &mut self.pantalla {
                    estado.mensaje = Some(error);
                }
                false
            }
        }
    }

    fn despachar_salida(&mut self, salida: Salida) {
        match salida {
            Salida::Video(ruta) => {
                let ruta = ruta.to_string_lossy().into_owned();
                self.abrir(EditorApp::desde_video(&ruta));
            }
            Salida::Proyecto(ruta) => {
                let editor = abrir_proyecto(&ruta)
                    .map_err(|error| error.to_string())
                    .and_then(|proyecto| {
                        EditorApp::desde_proyecto(proyecto.clone(), Some(ruta), proyecto.playhead)
                    });
                self.abrir(editor);
            }
            Salida::Recuperar => {
                let aviso = match &self.pantalla {
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
                    if self.abrir(EditorApp::desde_proyecto(recovery.proyecto, ruta, posicion)) {
                        if let Pantalla::Inicio(estado) = &mut self.pantalla {
                            estado.aviso = None;
                        }
                        let _ = recovery_io::escribir(&recovery_io::ruta(), None);
                    }
                }
            }
            Salida::DescartarRecovery => {
                let _ = recovery_io::escribir(&recovery_io::ruta(), None);
            }
        }
    }

    fn responder(&mut self, respuesta: RespuestaDialogo) {
        use rod_app::editor_app::MensajeEditor as E;
        match &mut self.pantalla {
            Pantalla::Inicio(estado) => {
                let mensaje = match respuesta {
                    RespuestaDialogo::Video(ruta) => Some(MensajeInicio::DialogoVideo(ruta)),
                    RespuestaDialogo::Proyecto(ruta) => Some(MensajeInicio::DialogoProyecto(ruta)),
                    _ => None,
                };
                if let Some(mensaje) = mensaje {
                    let (tarea, salida) = inicio::update(estado, mensaje);
                    self.ejecutar_tarea(tarea);
                    if let Some(salida) = salida {
                        self.despachar_salida(salida);
                    }
                }
            }
            Pantalla::Editor(editor) => {
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
                    RespuestaDialogo::Video(_) => return,
                };
                let tarea = editor_app::update(editor, mensaje);
                self.ejecutar_tarea(tarea);
            }
        }
    }

    fn aplicar_comando(&mut self, comando: Comando) {
        match &mut self.pantalla {
            Pantalla::Inicio(estado) => {
                let mensaje = match comando {
                    Comando::NuevoProyecto | Comando::AbrirVideo => Some(MensajeInicio::AbrirVideo),
                    Comando::AbrirProyecto => Some(MensajeInicio::AbrirProyecto),
                    Comando::Ayuda => {
                        estado.mensaje = Some(
                            "Atajos: Ctrl+S guardar, Ctrl+O abrir, Space reproducir, ←/→ saltar, ? ayuda"
                                .into(),
                        );
                        None
                    }
                    _ => None,
                };
                if let Some(mensaje) = mensaje {
                    let (tarea, salida) = inicio::update(estado, mensaje);
                    self.ejecutar_tarea(tarea);
                    if let Some(salida) = salida {
                        self.despachar_salida(salida);
                    }
                }
            }
            Pantalla::Editor(editor) => {
                let mensaje = match comando {
                    Comando::NuevoProyecto | Comando::AbrirVideo => MensajeEditor::AbrirVideo,
                    Comando::AbrirProyecto => MensajeEditor::AbrirProyecto,
                    Comando::Guardar => MensajeEditor::GuardarProyecto,
                    Comando::GuardarComo => MensajeEditor::GuardarProyectoComo,
                    _ => MensajeEditor::Comando(comando),
                };
                let tarea = editor_app::update(editor, mensaje);
                self.ejecutar_tarea(tarea);
            }
        }
    }
}

impl eframe::App for RustyDialogue {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Respuestas de los diálogos que se abrieron en otro hilo.
        let pendientes: Vec<_> = self.rx_dialogos.try_iter().collect();
        for respuesta in pendientes {
            self.responder(respuesta);
        }

        // Un único registro de atajos para menú y teclado.
        let pulsaciones: Vec<Comando> = ctx.input(|entrada| {
            entrada
                .events
                .iter()
                .filter_map(|evento| match evento {
                    egui::Event::Key {
                        key,
                        modifiers,
                        pressed: true,
                        ..
                    } => comandos::desde_input(*key, *modifiers),
                    _ => None,
                })
                .collect()
        });
        for comando in pulsaciones {
            self.aplicar_comando(comando);
        }

        let soltados: Vec<PathBuf> = ctx.input(|entrada| {
            entrada
                .raw
                .dropped_files
                .iter()
                .filter_map(|archivo| archivo.path.clone())
                .collect()
        });
        for ruta in soltados {
            if matches!(self.pantalla, Pantalla::Inicio(_)) {
                let (tarea, salida) = match &mut self.pantalla {
                    Pantalla::Inicio(estado) => {
                        inicio::update(estado, MensajeInicio::ArchivoSoltado(ruta))
                    }
                    Pantalla::Editor(_) => (Tarea::Ninguna, None),
                };
                self.ejecutar_tarea(tarea);
                if let Some(salida) = salida {
                    self.despachar_salida(salida);
                }
            }
        }

        // mpv entrega frames por eventos: se sondea una vez por frame y se pide
        // repintado mientras haya video abierto.
        if let Pantalla::Editor(editor) = &mut self.pantalla {
            let tarea = editor_app::update(editor, MensajeEditor::Tick);
            self.ejecutar_tarea(tarea);
            ctx.request_repaint();
        }

        let mut comando_menu = None;
        egui::TopBottomPanel::top("cabecera").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("R").color(ui::tema::ACENTO).strong());
                ui.label(egui::RichText::new("RustyDialogue").size(12.0));
                let en_inicio = matches!(self.pantalla, Pantalla::Inicio(_));
                comando_menu = rod_app::menu::mostrar(ui, en_inicio);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("□").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(
                            !ctx.input(|entrada| entrada.viewport().maximized.unwrap_or(false)),
                        ));
                    }
                    if ui.button("—").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                });
            });
        });
        if let Some(comando) = comando_menu {
            self.aplicar_comando(comando);
        }

        let mut mensaje_inicio = None;
        let mut mensaje_editor = None;
        egui::CentralPanel::default().show(ctx, |ui| match &mut self.pantalla {
            Pantalla::Inicio(estado) => mensaje_inicio = ui::inicio::mostrar(ui, estado),
            Pantalla::Editor(editor) => mensaje_editor = ui::editor::mostrar(ui, editor),
        });
        if let Some(mensaje) = mensaje_inicio {
            let (tarea, salida) = match &mut self.pantalla {
                Pantalla::Inicio(estado) => inicio::update(estado, mensaje),
                Pantalla::Editor(_) => (Tarea::Ninguna, None),
            };
            self.ejecutar_tarea(tarea);
            if let Some(salida) = salida {
                self.despachar_salida(salida);
            }
        }
        if let Some(mensaje) = mensaje_editor
            && let Pantalla::Editor(editor) = &mut self.pantalla
        {
            let tarea = editor_app::update(editor, mensaje);
            self.ejecutar_tarea(tarea);
        }
    }
}

fn main() -> eframe::Result<()> {
    let opciones = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native(
        "RustyDialogue",
        opciones,
        Box::new(|cc| {
            cc.egui_ctx.set_style(ui::tema::estilo());
            Ok(Box::new(RustyDialogue::nuevo()))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::{Pantalla, RustyDialogue};
    use rod_app::inicio::Salida;

    fn app_de_prueba() -> RustyDialogue {
        let mut app = RustyDialogue::nuevo();
        app.pantalla = super::nuevo_con(
            &std::env::temp_dir().join(format!("rod-router-{}", std::process::id())),
            &std::env::temp_dir().join(format!("rod-recovery-{}", std::process::id())),
        );
        app
    }

    #[test]
    fn arranca_en_inicio_sin_abrir_player() {
        assert!(matches!(app_de_prueba().pantalla, Pantalla::Inicio(_)));
    }

    #[test]
    fn recuperar_con_video_inexistente_conserva_el_aviso() {
        let mut app = app_de_prueba();
        let proyecto = rod_core::Proyecto::nuevo("C:/no/existe/clip.mp4");
        let recovery = rod_core::construir_recovery(proyecto, "C:/no/existe.cdp", 1_000.0);
        let texto = rod_core::formatear_recovery(&recovery, "existe.cdp", 1_000.0);
        let aviso = rod_app::recovery_io::AvisoRecovery { recovery, texto };
        if let Pantalla::Inicio(estado) = &mut app.pantalla {
            estado.aviso = Some(aviso);
        }
        app.despachar_salida(Salida::Recuperar);
        assert!(matches!(app.pantalla, Pantalla::Inicio(_)));
    }
}
