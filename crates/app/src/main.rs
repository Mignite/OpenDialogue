#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use rod_app::comandos::Comando;
use rod_app::dialogos::{self, RespuestaDialogo};
use rod_app::editor_app::{self, EditorApp, MensajeEditor};
use rod_app::inicio::{self, EstadoInicio, MensajeInicio, Salida};
use rod_app::preferencias::Preferencias;
use rod_app::project_io::abrir_proyecto;
use rod_app::tarea::Tarea;
use rod_app::video::Gpu;
use rod_app::{recientes, recovery_io, ui};

/// Las dos pantallas. El editor solo existe cuando se abrió un video o un
/// proyecto: el arranque no construye mpv.
#[allow(clippy::large_enum_variant)] // el editor es grande por mpv y el preview
enum Pantalla {
    Inicio(EstadoInicio),
    Editor(Box<EditorApp>),
}

struct RustyDialogue {
    pantalla: Pantalla,
    tx_dialogos: Sender<RespuestaDialogo>,
    rx_dialogos: Receiver<RespuestaDialogo>,
    /// Formato real del framebuffer, tomado del `CreationContext`.
    formato: wgpu::TextureFormat,
    /// Un único pipeline de composición para toda la sesión.
    gpu: Arc<Mutex<Option<Gpu>>>,
    preferencias: Preferencias,
    ajustes: ui::ajustes::EstadoAjustes,
    toasts: ui::toast::Toasts,
    panel_atajos: ui::atajos::EstadoAtajos,
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
    fn nuevo(formato: wgpu::TextureFormat) -> Self {
        let (tx_dialogos, rx_dialogos) = std::sync::mpsc::channel();
        Self {
            pantalla: nuevo(),
            tx_dialogos,
            rx_dialogos,
            formato,
            gpu: Arc::new(Mutex::new(None)),
            preferencias: Preferencias::cargar(),
            ajustes: Default::default(),
            toasts: Default::default(),
            panel_atajos: Default::default(),
        }
    }

    /// Ejecuta los efectos que el front-end solo describe.
    fn ejecutar_tarea(&self, tarea: Tarea) {
        let tx = &self.tx_dialogos;
        match tarea {
            Tarea::MostrarEnCarpeta(ruta) => {
                let _ = std::process::Command::new("explorer").arg(&ruta).spawn();
            }
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
            Ok(mut editor) => {
                editor.snap = self.preferencias.snap;
                editor.seguir = self.preferencias.seguir;
                editor.preset_seleccionado = self.preferencias.preset_abierto.clone();
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
                        EditorApp::desde_proyecto(
                            proyecto.clone(),
                            Some(ruta.clone()),
                            proyecto.playhead,
                        )
                    });
                // Abrir un proyecto también lo promueve al tope de recientes:
                // antes solo «Guardar» escribía el índice y la lista quedaba
                // clavada en lo último guardado.
                if self.abrir(editor) {
                    let _ = recientes::registrar(&recientes::ruta_indice(), &ruta);
                }
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
        if comando == Comando::Ajustes {
            self.ajustes.abrir();
            return;
        }
        if comando == Comando::Ayuda {
            self.panel_atajos.abierto = true;
            return;
        }
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
        if comando == Comando::AlternarSnap
            && let Pantalla::Editor(editor) = &self.pantalla
        {
            self.preferencias.snap = editor.snap;
            let _ = self.preferencias.guardar();
        }
    }
}

impl eframe::App for RustyDialogue {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Al minimizar, wgpu recibe una superficie de 0x0 y el renderizador
        // puede fallar. No pintamos ni pedimos frames mientras no haya área.
        let minimizado = ctx.input(|entrada| {
            entrada
                .viewport()
                .inner_rect
                .is_some_and(|rect| rect.width() <= 1.0 || rect.height() <= 1.0)
        });
        if minimizado {
            // La superficie wgpu desaparece al minimizar: soltar la pipeline
            // cacheada evita usar un formato/dispositivo inválidos al volver.
            if let Ok(mut gpu) = self.gpu.lock() {
                gpu.take();
            }
            return;
        }

        // Ventana sin marco: devolver los bordes al SO con WM_NCHITTEST.
        ui::borde::mostrar(ctx, frame);

        // Respuestas de los diálogos que se abrieron en otro hilo.
        let pendientes: Vec<_> = self.rx_dialogos.try_iter().collect();
        for respuesta in pendientes {
            self.responder(respuesta);
        }

        // Un único registro de atajos para menú y teclado. Con un modal
        // abierto o editando una fila, la tecla va al widget, no al proyecto.
        let capturando_teclado = self.ajustes.abierto
            || self.panel_atajos.abierto
            || matches!(&self.pantalla, Pantalla::Editor(editor) if editor.texto_inline_activo());
        // Solo el campo de texto del subtítulo consume las teclas desnudas:
        // el foco en un botón, slider o buscador no debe bloquear Espacio/E/S/M.
        let foco_en_texto = matches!(&self.pantalla, Pantalla::Editor(editor) if editor.texto.foco)
            || capturando_teclado;
        let atajos = &self.preferencias.atajos;
        let pulsaciones: Vec<Comando> = if capturando_teclado {
            Vec::new()
        } else {
            ctx.input(|entrada| {
                entrada
                    .events
                    .iter()
                    .filter_map(|evento| match evento {
                        egui::Event::Key {
                            key,
                            modifiers,
                            pressed: true,
                            ..
                        } => atajos.comando_de(*key, *modifiers, foco_en_texto),
                        _ => None,
                    })
                    .collect()
            })
        };
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
        }
        let repintado = match &self.pantalla {
            // En pausa, mpv puede entregar el frame del seek en un frame
            // posterior: seguimos sondeando hasta que `frame_pendiente` se
            // limpia al renderizar.
            Pantalla::Editor(editor) if editor.frame_pendiente => Some(Duration::from_millis(100)),
            Pantalla::Editor(editor) => ui::repintado::politica(
                !editor_app::pausado(editor),
                editor_app::sucio(editor),
                editor_app::buscando(editor),
                editor.scrubbing(),
            ),
            Pantalla::Inicio(_) => None,
        };
        match repintado {
            Some(Duration::ZERO) => ctx.request_repaint(),
            Some(delay) => ctx.request_repaint_after(delay),
            None => {}
        }

        let proyecto = match &self.pantalla {
            Pantalla::Editor(editor) => Some(editor_app::nombre_proyecto(editor)),
            Pantalla::Inicio(_) => None,
        };
        let sucio = match &self.pantalla {
            Pantalla::Editor(editor) => editor_app::sucio(editor),
            Pantalla::Inicio(_) => false,
        };
        let comando_menu = ui::chrome::mostrar(
            ctx,
            self.preferencias.idioma.textos(),
            &self.preferencias.atajos,
            ui::chrome::EstadoCabecera {
                en_inicio: matches!(self.pantalla, Pantalla::Inicio(_)),
                proyecto,
                sucio,
            },
        );
        if let Some(comando) = comando_menu {
            self.aplicar_comando(comando);
        }
        if ui::ajustes::mostrar(ctx, &mut self.preferencias, &mut self.ajustes)
            && self.preferencias.guardar().is_err()
        {
            // Un archivo de solo lectura no debe impedir usar la app.
        }

        let mut mensaje_inicio = None;
        let mut mensajes_editor = Vec::new();
        let formato = self.formato;
        let gpu = Arc::clone(&self.gpu);
        let textos = self.preferencias.idioma.textos();
        let atajos = &self.preferencias.atajos;
        let mut medidas = ui::editor::MedidasEditor::from(&self.preferencias);
        match &mut self.pantalla {
            Pantalla::Inicio(estado) => {
                egui::CentralPanel::default()
                    .frame(ui::tema::panel(ui::tema::FONDO))
                    .show(ctx, |ui| {
                        let textos = self.preferencias.idioma.textos();
                        let (mensaje, abrir_ajustes) =
                            ui::inicio::mostrar(ui, estado, textos, &mut self.preferencias);
                        mensaje_inicio = mensaje;
                        if abrir_ajustes {
                            self.ajustes.abrir();
                        }
                    });
            }
            Pantalla::Editor(editor) => {
                mensajes_editor =
                    ui::editor::show(ctx, editor, formato, gpu, textos, atajos, &mut medidas);
            }
        }
        if medidas != ui::editor::MedidasEditor::from(&self.preferencias) {
            self.preferencias.ancho_panel = medidas.ancho_panel;
            self.preferencias.alto_timeline = medidas.alto_timeline;
            self.preferencias.panel_visible = medidas.panel_visible;
            let _ = self.preferencias.guardar();
        }
        if let Pantalla::Editor(editor) = &self.pantalla
            && (editor.seguir != self.preferencias.seguir
                || editor.preset_seleccionado != self.preferencias.preset_abierto)
        {
            self.preferencias.seguir = editor.seguir;
            self.preferencias.preset_abierto = editor.preset_seleccionado.clone();
            let _ = self.preferencias.guardar();
        }
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
        let borro = mensajes_editor.iter().any(|mensaje| {
            matches!(
                mensaje,
                MensajeEditor::BorrarSeleccion | MensajeEditor::Comando(Comando::Eliminar)
            )
        });
        let borro_preset = mensajes_editor
            .iter()
            .any(|mensaje| matches!(mensaje, MensajeEditor::BorrarPreset(_)));
        for mensaje in mensajes_editor {
            if let Pantalla::Editor(editor) = &mut self.pantalla {
                let tarea = editor_app::update(editor, mensaje);
                self.ejecutar_tarea(tarea);
            }
        }
        if borro {
            self.toasts.push(textos.linea_eliminada.to_owned(), true);
        }
        if borro_preset {
            self.toasts
                .push_restaurar_preset(textos.preset_eliminado.to_owned());
        }
        ui::atajos::mostrar(
            ctx,
            textos,
            &self.preferencias.atajos,
            &mut self.panel_atajos,
        );
        if let Some(mensaje) = self.toasts.mostrar(ctx, textos)
            && let Pantalla::Editor(editor) = &mut self.pantalla
        {
            let tarea = editor_app::update(editor, mensaje);
            self.ejecutar_tarea(tarea);
        }
    }
}

fn main() -> eframe::Result<()> {
    let opciones = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 900.0])
            .with_decorations(false),
        ..Default::default()
    };
    eframe::run_native(
        "RustyDialogue",
        opciones,
        Box::new(|cc| {
            ui::tema::aplicar(&cc.egui_ctx);
            let formato = cc
                .wgpu_render_state
                .as_ref()
                .map(|estado| estado.target_format)
                .unwrap_or(wgpu::TextureFormat::Bgra8Unorm);
            Ok(Box::new(RustyDialogue::nuevo(formato)))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::{Pantalla, RustyDialogue};
    use rod_app::inicio::Salida;

    fn app_de_prueba() -> RustyDialogue {
        let mut app = RustyDialogue::nuevo(wgpu::TextureFormat::Bgra8Unorm);
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
