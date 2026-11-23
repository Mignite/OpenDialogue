use std::path::PathBuf;
use std::sync::Arc;

use iced::widget::{
    button, column, container, pick_list, row, scrollable, shader, slider, space, text, text_input,
};
use iced::{Background, Color, Element, Length, Subscription, Task};

use rod_app::editor::{EstadoEditor, MAX_HABLANTES, normalizar_tecla};
use rod_app::presets_io;
use rod_app::preview::{CANVAS, ass_del_proyecto};
use rod_app::project_io::{
    Documento, ExportacionSrt, FormatoSubtitulo, abrir_proyecto, cargar, exportar_json_combinado,
    exportar_srt_por_hablante, guardar_proyecto,
};
use rod_app::recovery_io;
use rod_app::timeline::{self, Vista};
use rod_app::video::{Capa, Video};
use rod_core::{PresetAss, Proyecto};
use rod_player::AudioTrack;

const WIDTH: usize = CANVAS.0 as usize;
const HEIGHT: usize = CANVAS.1 as usize;

/// Velocidades que ofrece la barra de transporte. `pick_list` necesita un tipo
/// propio porque `f32` no es `Display` y el texto de la lista es la etiqueta.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Velocidad(f32);

impl std::fmt::Display for Velocidad {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let texto = if self.0.fract() == 0.0 {
            format!("{}x", self.0 as i64)
        } else {
            format!("{}x", self.0)
        };
        formatter.write_str(&texto)
    }
}

const VELOCIDADES: [Velocidad; 6] = [
    Velocidad(0.5),
    Velocidad(0.75),
    Velocidad(1.0),
    Velocidad(1.25),
    Velocidad(1.5),
    Velocidad(2.0),
];

#[derive(Debug, Clone, PartialEq)]
enum CampoHablante {
    Nombre,
    Tecla,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    AbrirVideo,
    AudioTrack(AudioTrack),
    Velocidad(Velocidad),
    PlayPause,
    /// El usuario mueve el slider pero todavía no suelta: solo se mueve el
    /// thumb, no se le pide nada a mpv.
    Arrastrar(f32),
    /// Soltó el slider: aquí sí se busca.
    Soltar,
    Step(f64),
    Seleccionar(String),
    BorrarSeleccion,
    Deshacer,
    Rehacer,
    /// Texto que el usuario escribe. No toca el proyecto hasta `AplicarTexto`:
    /// así un Ctrl+Z deshace el texto confirmado, no cada tecla.
    Borrador(String),
    AplicarTexto,
    AgregarHablante,
    /// Edición en vivo de un hablante: el campo decide qué se toca.
    EditarHablante(String, CampoHablante, String),
    BorrarHablante(String),
    /// `None` deja los captions seleccionados sin hablante.
    AsignarHablante(Option<String>),
    Timeline(timeline::Mensaje),
    /// Zoom del timeline: `factor > 1` acerca. El ancla es el playhead.
    Zoom(f64),
    /// El usuario enciende o apaga el seguimiento del playhead.
    Seguir,
    /// Modificadores del teclado, para que la rueda sepa si hay Shift.
    Modificadores(iced::keyboard::Modifiers),
    /// Abre un `.cdp`: proyecto, video y playhead.
    AbrirProyecto,
    /// Guarda en la ruta actual o, si no hay, pregunta una.
    GuardarProyecto,
    /// Pregunta ruta siempre.
    GuardarProyectoComo,
    /// Acepta la copia automática que se ofreció al arrancar.
    Recuperar,
    /// Rechaza la copia automática y la borra.
    DescartarRecovery,
    /// Un `.srt` por hablante (y `solapes.txt`) en una carpeta elegida.
    ExportarSrtHablantes,
    /// `subtitulos_combinado.json` con tiempos, texto y hablante.
    ExportarJson,
    /// El `.ass` del preview, con los presets guardados.
    ExportarAss,
}

struct Estado {
    editor: EstadoEditor,
    player: rod_player::Player,
    subtitles: rod_subs::Subtitulos,
    pixels: Vec<u8>,
    layers: Arc<Vec<Capa>>,
    duration: f64,
    position: f64,
    /// El usuario tiene el slider tomado: `time-pos` no debe pisar el thumb.
    arrastrando: bool,
    paused: bool,
    video_path: String,
    audio_tracks: Vec<AudioTrack>,
    selected_audio: Option<AudioTrack>,
    velocidad: Velocidad,
    /// Lo que hay en el `text_input` del caption seleccionado.
    borrador: String,
    /// Campo de hablante que está recibiendo teclas. iced no da un evento de
    /// foco por widget, así que la sesión se cierra al tocar cualquier otra
    /// cosa: es el equivalente al `onFocus={onCommit}` del oráculo (una sesión
    /// de tecleo = un paso de historial).
    editando_hablante: Option<(String, CampoHablante)>,
    /// Arrastre en curso en el timeline. Vive en la app y no en el canvas
    /// porque el historial y el proyecto también viven aquí.
    arrastre: Option<timeline::Arrastre>,
    /// Ventana visible del timeline (inicio y escala).
    vista: Vista,
    /// Ancho del canvas del timeline, que lo publica el propio canvas. Sin él
    /// no se puede saber cuántos segundos caben (pan, seguimiento, zoom).
    ancho_timeline: f32,
    /// La vista persigue al playhead mientras se reproduce. Se apaga al usar
    /// la rueda o la barra, y se enciende con el botón.
    seguir: bool,
    /// Inicio hacia el que se desliza la vista, si hay un seguimiento en curso.
    objetivo_vista: Option<f64>,
    /// Modificadores del teclado. La rueda no los lleva en su evento, así que
    /// el «Shift para acercar» se decide con este espejo.
    modificadores: iced::keyboard::Modifiers,
    /// Dónde se guardó el proyecto. `None` = nunca se guardó: «Guardar» pide
    /// ruta y «Guardar como» la vuelve a pedir.
    ruta_proyecto: Option<PathBuf>,
    /// El proyecto tal como está en disco (o como arrancó la app). Es la línea
    /// base del «hay cambios sin guardar»: el playhead no cuenta, porque se
    /// mueve solo al reproducir y marcaría el proyecto sucio 60 veces por
    /// segundo.
    limpio: Proyecto,
    /// Copia automática pendiente de aceptar. Se lee una sola vez, al
    /// arrancar; el texto se formatea ahí mismo para no leer el reloj en cada
    /// frame.
    recovery: Option<AvisoRecovery>,
    /// Epoch ms de la última escritura del recovery (`None` = ninguna todavía)
    /// y del último chequeo del autosave. El chequeo va a 1 Hz: comparar el
    /// proyecto entero en cada frame para nada es tirar CPU.
    ultima_escritura: Option<f64>,
    ultimo_chequeo: f64,
    /// Presets `.ass` cargados al arrancar. Los usa el preview y el export por
    /// igual: son la única fuente de estilos de un caption con hablante.
    presets: Vec<PresetAss>,
    /// Resultado de la última acción que no tiene otro lugar donde contarse
    /// (exportar, por ahora). iced no tiene toasts; la barra de estado es el
    /// sitio donde el usuario ya mira el estado del proyecto.
    mensaje: Option<String>,
}

/// Lo que el aviso de recuperación necesita: la copia y el texto ya formateado.
struct AvisoRecovery {
    recovery: rod_core::Recovery,
    texto: String,
}

fn main() -> iced::Result {
    iced::application(Estado::nuevo, update, view)
        .title("Rusty Open Dialogue")
        .window_size([1600.0, 900.0])
        .subscription(subscription)
        .run()
}

fn subscription(_state: &Estado) -> Subscription<Message> {
    Subscription::batch([
        iced::window::frames().map(|_| Message::Tick),
        // Ctrl+Z / Ctrl+Shift+Z (y Ctrl+Y, que es el redo de Windows). El
        // atajo se decide aquí y no en el `update` porque la app no recibe el
        // teclado de otra forma: los `text_input` se comen los caracteres.
        // Los modificadores se copian aparte: `WheelScrolled` no los trae y el
        // «Shift + rueda = zoom» necesita saberlos.
        iced::keyboard::listen().filter_map(|evento| match evento {
            iced::keyboard::Event::ModifiersChanged(modifiers) => {
                Some(Message::Modificadores(modifiers))
            }
            iced::keyboard::Event::KeyPressed { key, modifiers, .. } => {
                if !modifiers.command() {
                    return None;
                }
                let iced::keyboard::Key::Character(texto) = key.as_ref() else {
                    return None;
                };
                if texto.eq_ignore_ascii_case("z") {
                    return Some(if modifiers.shift() {
                        Message::Rehacer
                    } else {
                        Message::Deshacer
                    });
                }
                if texto.eq_ignore_ascii_case("y") && !modifiers.shift() {
                    return Some(Message::Rehacer);
                }
                if texto.eq_ignore_ascii_case("s") {
                    return Some(if modifiers.shift() {
                        Message::GuardarProyectoComo
                    } else {
                        Message::GuardarProyecto
                    });
                }
                if texto.eq_ignore_ascii_case("o") && !modifiers.shift() {
                    return Some(Message::AbrirProyecto);
                }
                None
            }
            _ => None,
        }),
    ])
}

impl Estado {
    fn nuevo() -> Self {
        let documento = cargar(
            "1\n00:00:01,000 --> 00:00:03,000\nPrimer caption\n\n\
             2\n00:00:04,000 --> 00:00:06,500\nSegundo caption",
            "testdata/clip-15s-2audio.mp4",
        )
        .expect("el fixture inicial debe ser SRT valido");
        let editor = EstadoEditor::nuevo(documento);
        let video_path = "testdata/clip-15s-2audio.mp4".to_owned();
        let player = rod_player::Player::open(&video_path)
            .expect("no se pudo abrir testdata/clip-15s-2audio.mp4");
        let duration = player.duration();
        let paused = player.paused();
        let audio_tracks = player.audio_tracks();
        let selected_audio = audio_tracks.first().cloned();
        let velocidad = Velocidad(player.speed() as f32);
        // Los presets se leen una sola vez: el panel para editarlos llega en
        // el slice 7d, así que nada los cambia en caliente todavía.
        let presets = presets_io::leer(&presets_io::ruta());
        let ass = ass_del_proyecto(&editor, &presets);
        let subtitles = rod_subs::Subtitulos::nuevo(WIDTH as u32, HEIGHT as u32, &ass);
        let limpio = editor.proyecto().clone();

        let mut estado = Self {
            editor,
            player,
            subtitles,
            pixels: vec![0; WIDTH * HEIGHT * 4],
            layers: Arc::new(Vec::new()),
            duration,
            position: 0.0,
            arrastrando: false,
            paused,
            video_path,
            audio_tracks,
            selected_audio,
            velocidad,
            borrador: String::new(),
            editando_hablante: None,
            arrastre: None,
            // 15 s caben en ~750 px: el clip de prueba entra completo.
            vista: Vista::default(),
            // Ancho 0 = el canvas todavía no se midió. Con 0 el pan no tiene
            // sentido y el seguimiento espera; el primer `Ancho` lo arregla y,
            // de paso, ajusta la ventana al video.
            ancho_timeline: 0.0,
            seguir: true,
            objetivo_vista: None,
            modificadores: iced::keyboard::Modifiers::default(),
            ruta_proyecto: None,
            limpio,
            recovery: recovery_pendiente(&recovery_io::ruta(), recovery_io::ms_ahora()),
            ultima_escritura: None,
            // 0 = todavía no se ha chequeado; el primer `Tick` ya decide con
            // el proyecto limpio y no escribe nada.
            ultimo_chequeo: 0.0,
            presets,
            mensaje: None,
        };
        refrescar_preview(&mut estado);
        estado
    }
}

/// Cambia el video de la app sin tocar el proyecto. `position` es dónde queda
/// el playhead: 0 al abrir un video suelto, el guardado al abrir un proyecto.
fn reemplazar_player(
    state: &mut Estado,
    ruta_video: &str,
    player: rod_player::Player,
    position: f64,
) {
    state.duration = player.duration();
    state.position = position.clamp(0.0, state.duration.max(0.0));
    state.paused = player.paused();
    state.video_path = ruta_video.to_owned();
    state.audio_tracks = player.audio_tracks();
    state.selected_audio = state.audio_tracks.first().cloned();
    state.velocidad = Velocidad(player.speed() as f32);
    state.player = player;
    state.layers = Arc::new(Vec::new());
    // Otro video, otra escala: se encaja entero en la ventana.
    state.vista = state.vista.encajar(state.duration, state.ancho_timeline);
    state.objetivo_vista = None;
    if state.position > 0.0
        && let Err(error) = state.player.seek(state.position)
    {
        eprintln!("no se pudo buscar el playhead guardado: {error}");
    }
}

/// Copia automática que quedó de la sesión anterior, si la hay y sirve. Se lee
/// una sola vez, al construir el estado: el archivo se descarta en cuanto el
/// usuario decide, así que no hay nada que volver a mirar.
///
/// La ruta y el reloj entran como parámetros para poder probar esto sin tocar
/// el `%APPDATA%` real ni depender de la hora.
fn recovery_pendiente(ruta: &std::path::Path, ahora: f64) -> Option<AvisoRecovery> {
    let recovery = recovery_io::leer(ruta, 0.0, ahora)?;
    // El nombre es el del archivo (`mi-video.cdp`), como en el oráculo: con la
    // ruta entera el aviso no cabe en una barra.
    let nombre = std::path::Path::new(&recovery.ruta_proyecto)
        .file_name()
        .and_then(|nombre| nombre.to_str())
        .unwrap_or_default()
        .to_owned();
    let texto = rod_core::formatear_recovery(&recovery, &nombre, ahora);
    Some(AvisoRecovery { recovery, texto })
}

/// ¿El proyecto en memoria difiere del que está en disco? Solo cuentan los
/// captions y los hablantes: el playhead se mueve al reproducir y no es una
/// edición.
fn sucio(state: &Estado) -> bool {
    let proyecto = state.editor.proyecto();
    proyecto.captions != state.limpio.captions || proyecto.hablantes != state.limpio.hablantes
}

/// Seguro ante crash: cada 15 s, si hay cambios sin guardar, escribe una copia
/// del proyecto en `recovery.json`. Falla en silencio a propósito: si el disco
/// está lleno o no hay permisos, el aviso al usuario sigue siendo el guardado
/// manual, no esto.
fn autosave(state: &mut Estado) {
    let ahora = recovery_io::ms_ahora();
    // La decisión (¿toca escribir?) se toma una vez por segundo, no por frame:
    // comparar el proyecto entero 60 veces por segundo para nada es tirar CPU.
    if ahora - state.ultimo_chequeo < 1000.0 {
        return;
    }
    state.ultimo_chequeo = ahora;
    let n_captions = state.editor.proyecto().captions.len();
    if !rod_core::debe_guardar_recovery(sucio(state), n_captions, state.ultima_escritura, ahora) {
        return;
    }
    let mut proyecto = state.editor.proyecto().clone();
    proyecto.playhead = state.position;
    let ruta_proyecto = state
        .ruta_proyecto
        .as_ref()
        .map(|ruta| ruta.to_string_lossy().into_owned())
        .unwrap_or_default();
    let copia = rod_core::construir_recovery(proyecto, ruta_proyecto, ahora);
    let texto = match rod_core::recovery::a_json(&copia) {
        Ok(texto) => texto,
        Err(error) => {
            eprintln!("no se pudo serializar la copia automática: {error}");
            return;
        }
    };
    match recovery_io::escribir(&recovery_io::ruta(), Some(&texto)) {
        Ok(()) => state.ultima_escritura = Some(ahora),
        Err(error) => eprintln!("no se pudo escribir la copia automática: {error}"),
    }
}

/// Carga la copia automática ofrecida al arrancar. Es un proyecto completo con
/// su ruta de origen; si el video no abre no se toca nada y el aviso sigue en
/// pantalla, porque no hay otra forma de arreglarlo desde aquí.
fn recuperar(state: &mut Estado) {
    let Some(aviso) = state.recovery.take() else {
        return;
    };
    let ruta = if aviso.recovery.ruta_proyecto.is_empty() {
        None
    } else {
        Some(PathBuf::from(&aviso.recovery.ruta_proyecto))
    };
    if let Err(error) = cargar_proyecto(state, aviso.recovery.proyecto.clone(), ruta) {
        eprintln!("no se pudo recuperar el proyecto: {error}");
        state.recovery = Some(aviso);
        return;
    }
    // Recuperar no limpia el proyecto (`limpio` sigue siendo el de arranque):
    // el `.cdp` en disco es más viejo que lo recuperado, así que sigue
    // habiendo cambios sin guardar. Es lo que hace el oráculo, y además
    // protege el caso de un video que sí abrió pero cuyo proyecto estaba sin
    // guardar: lo recuperado solo existe en memoria.
    borrar_recovery(state);
}

/// Rechaza la copia automática: se borra para no volver a preguntar en cada
/// arranque.
fn descartar_recovery(state: &mut Estado) {
    state.recovery = None;
    borrar_recovery(state);
}

fn borrar_recovery(state: &mut Estado) {
    if let Err(error) = recovery_io::escribir(&recovery_io::ruta(), None) {
        eprintln!("no se pudo borrar la copia automática: {error}");
    }
    // Sin archivo no hay nada escrito: la próxima edición guarda al instante
    // en vez de esperar los 15 s del intervalo.
    state.ultima_escritura = None;
}

/// Resumen de «Exportar SRT por hablante» para la barra de estado. Dice lo que
/// el usuario no ve: cuántos captions quedaron sin hablante y si hubo solapes.
fn resumen_srt(exportacion: &ExportacionSrt, escritos: usize) -> String {
    if escritos == 0 {
        return "No se exportó ningún SRT: ningún caption tiene hablante.".into();
    }
    let mut texto = format!("Exportados {escritos} SRT");
    if exportacion.sin_asignar > 0 {
        texto.push_str(&format!(
            "; {} caption(s) sin hablante",
            exportacion.sin_asignar
        ));
    }
    if exportacion.solapes > 0 {
        texto.push_str(&format!(
            "; {} solape(s) en solapes.txt",
            exportacion.solapes
        ));
    }
    texto
}

/// Reemplaza el proyecto abierto entero: editor (con su historial), video y
/// playhead. Si el video del proyecto no abre, no se toca nada: esta app no
/// sabe trabajar un proyecto sin video (el oráculo ofrece seguir sin él; aquí
/// eso sería un estado que ninguna de las otras piezas entiende).
///
/// `ruta` es `None` cuando el proyecto nunca se guardó (un recovery de un
/// proyecto nuevo): se puede editar, pero «Guardar» volverá a preguntar dónde.
fn cargar_proyecto(
    state: &mut Estado,
    proyecto: Proyecto,
    ruta: Option<PathBuf>,
) -> Result<(), String> {
    let player = rod_player::Player::open(&proyecto.ruta_video).map_err(|error| {
        format!(
            "no se pudo abrir el video '{}': {error}",
            proyecto.ruta_video
        )
    })?;
    let playhead = proyecto.playhead;
    let ruta_video = proyecto.ruta_video.clone();
    state.editor = EstadoEditor::nuevo(Documento {
        proyecto,
        // El `.cdp` no recuerda de qué formato venían los subtítulos; el
        // proyecto ya tiene los captions, así que da igual mientras no se
        // exporte.
        formato: FormatoSubtitulo::Srt,
        estilos_ass: Vec::new(),
    });
    state.editando_hablante = None;
    state.arrastre = None;
    state.borrador = String::new();
    reemplazar_player(state, &ruta_video, player, playhead);
    state.ruta_proyecto = ruta;
    refrescar_preview(state);
    sincronizar_borrador(state);
    Ok(())
}

/// Mueve el reproductor al segundo pedido. Un clic en la regla o en un hueco
/// del carril no es una edición: no toca el proyecto ni el historial.
fn buscar(state: &mut Estado, tiempo: f64) {
    state.position = tiempo.clamp(0.0, state.duration);
    if let Err(error) = state.player.seek(state.position) {
        eprintln!("no se pudo hacer seek desde el timeline: {error}");
    }
}

/// Cambia el zoom dejando quieto `ancla` (px) y deja la ventana dentro del
/// video: al alejarse, un inicio que cabía puede quedar más allá del final.
fn aplicar_escala(state: &mut Estado, escala: f64, ancla: f32) {
    let vista = state.vista.con_escala(escala, ancla);
    state.vista = vista.acotar(state.duration, state.ancho_timeline);
}

/// La ventana del timeline persigue al playhead mientras el usuario no la haya
/// tomado a mano (rueda o botón «manual»).
///
/// Se hace en el `Tick` y no al cambiar `position` porque el desplazamiento es
/// animado: cada frame avanza un 12 % del camino que falta, así un seek grande
/// no da un salto seco. Durante un arrastre no se toca la vista: el usuario
/// tiene el ratón en un punto concreto y moverle el mundo bajo el dedo es un
/// bug de sensación.
fn seguir_playhead(state: &mut Estado) {
    if !state.seguir || state.ancho_timeline <= 0.0 || state.arrastre.is_some() || state.arrastrando
    {
        return;
    }
    if let Some(objetivo) = timeline::objetivo_de_seguimiento(
        state.vista,
        state.position,
        state.duration,
        state.ancho_timeline,
    ) {
        state.objetivo_vista = Some(objetivo);
    }
    let Some(objetivo) = state.objetivo_vista else {
        return;
    };
    let (duracion, ancho) = (state.duration, state.ancho_timeline);
    if !timeline::acercar(&mut state.vista, objetivo, duracion, ancho) {
        state.objetivo_vista = None;
    }
}

/// Reconstruye el `.ass` del proyecto y rasteriza las capas de la posición
/// actual. Se llama tras cada edición: el video no cambia, pero los subtítulos
/// sí, y en pausa mpv no entrega un frame nuevo para disparar el repintado.
fn refrescar_preview(state: &mut Estado) {
    let ass = ass_del_proyecto(&state.editor, &state.presets);
    state.subtitles.cargar(&ass);
    let position = state.position;
    state.layers = Arc::new(
        state
            .subtitles
            .render((position * 1000.0) as i64)
            .iter()
            .map(Capa::desde)
            .collect(),
    );
}

/// El `text_input` muestra el texto del caption seleccionado. Sin selección
/// (o con varios) queda vacío: no hay un texto único que editar.
fn sincronizar_borrador(state: &mut Estado) {
    let id = match state.editor.seleccion.as_slice() {
        [id] => Some(id.clone()),
        _ => None,
    };
    state.borrador = id
        .and_then(|id| state.editor.texto_de(&id).map(str::to_owned))
        .unwrap_or_default();
}

fn update(state: &mut Estado, message: Message) -> Task<Message> {
    match message {
        Message::Tick => {
            let frame_changed = state.player.update() & 1 != 0;

            // `time-pos` sigue en el instante viejo uno o dos frames después de
            // pedir un seek, y no se mueve mientras el usuario arrastra. Si se
            // adopta igual, el thumb salta hacia atrás y pelea con el dedo.
            if !state.arrastrando && !state.player.seeking() {
                state.position = state.player.position();
            }
            state.paused = state.player.paused();
            seguir_playhead(state);
            autosave(state);
            if frame_changed
                && state
                    .player
                    .render(&mut state.pixels, WIDTH, HEIGHT)
                    .is_ok()
            {
                let position = state.position;
                state.layers = Arc::new(
                    state
                        .subtitles
                        .render((position * 1000.0) as i64)
                        .iter()
                        .map(Capa::desde)
                        .collect(),
                );
            }
        }
        Message::AbrirVideo => {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("Vídeos", &["mp4", "mkv", "webm", "mov", "avi"])
                .pick_file()
            else {
                return Task::none();
            };

            let Some(path_text) = path.to_str() else {
                eprintln!("la ruta seleccionada no es UTF-8");
                return Task::none();
            };

            match rod_player::Player::open(path_text) {
                Ok(player) => reemplazar_player(state, path_text, player, 0.0),
                Err(error) => {
                    eprintln!("no se pudo abrir el video seleccionado: {error}");
                }
            }
        }
        Message::AbrirProyecto => {
            let Some(ruta) = rfd::FileDialog::new()
                .add_filter("Proyecto", &["cdp"])
                .pick_file()
            else {
                return Task::none();
            };
            match abrir_proyecto(&ruta) {
                Ok(proyecto) => match cargar_proyecto(state, proyecto, Some(ruta)) {
                    Ok(()) => {
                        // Abrir es empezar de cero con un archivo que ya está
                        // en disco: no hay cambios sin guardar.
                        state.limpio = state.editor.proyecto().clone();
                    }
                    Err(error) => eprintln!("no se pudo abrir el proyecto: {error}"),
                },
                Err(error) => eprintln!("no se pudo leer el proyecto: {error}"),
            }
        }
        Message::GuardarProyecto | Message::GuardarProyectoComo => {
            let pedir_ruta =
                matches!(message, Message::GuardarProyectoComo) || state.ruta_proyecto.is_none();
            let ruta = match state.ruta_proyecto.clone() {
                Some(ruta) if !pedir_ruta => ruta,
                _ => {
                    let Some(ruta) = rfd::FileDialog::new()
                        .add_filter("Proyecto", &["cdp"])
                        .set_file_name("proyecto.cdp")
                        .save_file()
                    else {
                        return Task::none();
                    };
                    ruta
                }
            };
            // El playhead no es parte de la edición (vive fuera del editor y
            // del historial), pero sí del proyecto: se escribe al guardar y se
            // restaura al abrir.
            let mut proyecto = state.editor.proyecto().clone();
            proyecto.playhead = state.position;
            match guardar_proyecto(&ruta, &proyecto) {
                Ok(()) => {
                    state.ruta_proyecto = Some(ruta);
                    // Lo que hay en disco es lo que hay en memoria: la línea
                    // base del «sucio» se mueve aquí.
                    state.limpio = proyecto;
                    // Y el seguro ya no hace falta: al arrancar no hay nada
                    // que ofrecer (el oráculo deja el archivo hasta que
                    // caduca; borrarlo es más honesto).
                    state.recovery = None;
                    borrar_recovery(state);
                }
                Err(error) => eprintln!("no se pudo guardar el proyecto: {error}"),
            }
        }
        Message::Recuperar => recuperar(state),
        Message::DescartarRecovery => descartar_recovery(state),
        Message::ExportarSrtHablantes => {
            if state.editor.proyecto().captions.is_empty() {
                state.mensaje = Some("No hay subtítulos para exportar.".into());
                return Task::none();
            }
            let Some(carpeta) = rfd::FileDialog::new().pick_folder() else {
                return Task::none();
            };
            let exportacion = exportar_srt_por_hablante(state.editor.proyecto());
            let mut escritos = 0;
            for (nombre, contenido) in &exportacion.archivos {
                let ruta = carpeta.join(nombre);
                match std::fs::write(&ruta, contenido) {
                    Ok(()) => escritos += 1,
                    Err(error) => eprintln!("no se pudo escribir '{}': {error}", ruta.display()),
                }
            }
            // El reporte va aparte porque no es un SRT ni cuenta como archivo
            // exportado en el resumen, igual que en el oráculo.
            if let Some(reporte) = &exportacion.reporte_solapes {
                let ruta = carpeta.join("solapes.txt");
                if let Err(error) = std::fs::write(&ruta, reporte) {
                    eprintln!("no se pudo escribir '{}': {error}", ruta.display());
                }
            }
            state.mensaje = Some(resumen_srt(&exportacion, escritos));
        }
        Message::ExportarJson => {
            if state.editor.proyecto().captions.is_empty() {
                state.mensaje = Some("No hay subtítulos para exportar.".into());
                return Task::none();
            }
            let Some(ruta) = rfd::FileDialog::new()
                .add_filter("JSON combinado", &["json"])
                .set_file_name("subtitulos_combinado.json")
                .save_file()
            else {
                return Task::none();
            };
            let texto = exportar_json_combinado(state.editor.proyecto());
            state.mensaje = Some(match std::fs::write(&ruta, texto) {
                Ok(()) => format!("JSON exportado: {}", ruta.display()),
                Err(error) => {
                    eprintln!("no se pudo escribir el JSON: {error}");
                    format!("No se pudo exportar el JSON: {error}")
                }
            });
        }
        Message::ExportarAss => {
            if state.editor.proyecto().captions.is_empty() {
                state.mensaje = Some("No hay subtítulos para exportar.".into());
                return Task::none();
            }
            let Some(ruta) = rfd::FileDialog::new()
                .add_filter("Subtítulos ASS", &["ass"])
                .set_file_name("subtitulos.ass")
                .save_file()
            else {
                return Task::none();
            };
            // La misma cadena que rasteriza el preview: si el archivo y la
            // imagen divergieran, sería un bug (invariante del proyecto).
            let texto = ass_del_proyecto(&state.editor, &state.presets);
            state.mensaje = Some(match std::fs::write(&ruta, texto) {
                Ok(()) => format!("ASS exportado: {}", ruta.display()),
                Err(error) => {
                    eprintln!("no se pudo escribir el .ass: {error}");
                    format!("No se pudo exportar el .ass: {error}")
                }
            });
        }
        Message::PlayPause => {
            let destino = !state.player.paused();
            if destino && state.player.position() >= state.duration - 0.05 {
                if let Err(error) = state.player.seek(0.0) {
                    eprintln!("no se pudo reiniciar el video: {error}");
                    return Task::none();
                }
                state.position = 0.0;
            }
            match state.player.set_paused(destino) {
                // Se relee en vez de confiar en el espejo: mpv es la fuente.
                Ok(()) => state.paused = state.player.paused(),
                Err(error) => eprintln!("no se pudo cambiar pausa: {error}"),
            }
        }
        Message::AudioTrack(track) => match state.player.set_audio_track(track.id) {
            Ok(()) => {
                let activa = state.player.audio_track();
                state.selected_audio = state
                    .audio_tracks
                    .iter()
                    .find(|pista| Some(pista.id) == activa)
                    .cloned();
            }
            Err(error) => eprintln!("no se pudo cambiar la pista de audio: {error}"),
        },
        Message::Velocidad(velocidad) => match state.player.set_speed(velocidad.0 as f64) {
            // Se relee de mpv, que acota la velocidad a su rango.
            Ok(()) => state.velocidad = Velocidad(state.player.speed() as f32),
            Err(error) => eprintln!("no se pudo cambiar la velocidad: {error}"),
        },
        Message::Arrastrar(position) => {
            state.arrastrando = true;
            state.position = position as f64;
        }
        Message::Soltar => {
            state.arrastrando = false;
            if let Err(error) = state.player.seek(state.position) {
                eprintln!("no se pudo hacer seek: {error}");
            }
        }
        Message::Step(delta) => {
            state.position = (state.position + delta).clamp(0.0, state.duration);
            if let Err(error) = state.player.seek(state.position) {
                eprintln!("no se pudo hacer seek: {error}");
            }
        }
        Message::Seleccionar(id) => {
            state.editor.seleccionar_uno(&id);
            sincronizar_borrador(state);
        }
        Message::Borrador(texto) => state.borrador = texto,
        Message::AplicarTexto => {
            // El botón está apagado sin selección, pero Enter en el `text_input`
            // puede llegar igual: aquí se vuelve a comprobar.
            let Some(id) = state.editor.seleccion.first().cloned() else {
                return Task::none();
            };
            let texto = state.borrador.clone();
            if state.editor.aplicar_texto(&id, texto) {
                refrescar_preview(state);
            }
        }
        Message::BorrarSeleccion => {
            // Sin selección no hay paso de historial: un Ctrl+Z vacío es peor
            // que un botón apagado.
            if !state.editor.seleccion.is_empty() {
                state.editor.registrar_historial();
                state.editor.borrar_seleccion();
                sincronizar_borrador(state);
                refrescar_preview(state);
            }
        }
        Message::Deshacer => {
            state.editando_hablante = None;
            if state.editor.deshacer() {
                sincronizar_borrador(state);
                refrescar_preview(state);
            }
        }
        Message::Rehacer => {
            state.editando_hablante = None;
            if state.editor.rehacer() {
                sincronizar_borrador(state);
                refrescar_preview(state);
            }
        }
        Message::AgregarHablante => {
            state.editando_hablante = None;
            state.editor.agregar_hablante();
        }
        Message::EditarHablante(id, campo, valor) => {
            // Si el texto no cambia no hay nada que registrar: borrar y volver a
            // escribir la misma letra no debe dejar un Ctrl+Z que no hace nada.
            let cambia = match (&campo, state.editor.hablante(&id)) {
                (CampoHablante::Nombre, Some(hablante)) => hablante.nombre != valor,
                (CampoHablante::Tecla, Some(hablante)) => {
                    hablante.tecla != normalizar_tecla(&valor)
                }
                (_, None) => false,
            };
            if !cambia {
                return Task::none();
            }
            let clave = (id.clone(), campo.clone());
            if state.editando_hablante.as_ref() != Some(&clave) {
                state.editor.registrar_historial();
                state.editando_hablante = Some(clave);
            }
            match campo {
                CampoHablante::Nombre => {
                    state.editor.editar_nombre_hablante(&id, valor);
                }
                CampoHablante::Tecla => {
                    state.editor.editar_tecla_hablante(&id, &valor);
                }
            }
            refrescar_preview(state);
        }
        Message::BorrarHablante(id) => {
            state.editando_hablante = None;
            if state.editor.borrar_hablante(&id) {
                refrescar_preview(state);
            }
        }
        Message::AsignarHablante(hablante_id) => {
            state.editando_hablante = None;
            if state.editor.asignar_a_seleccion(hablante_id) > 0 {
                refrescar_preview(state);
            }
        }
        Message::Timeline(mensaje) => match mensaje {
            timeline::Mensaje::Ancho(ancho) => {
                let primera_medida = state.ancho_timeline <= 0.0;
                state.ancho_timeline = ancho;
                if primera_medida {
                    // Antes de medir no se puede saber cuánto abarca la
                    // ventana. La primera medida ajusta la vista al video: con
                    // la escala por defecto un clip de 15 s ocupaba media
                    // pantalla y el resto era vacío.
                    state.vista = state.vista.encajar(state.duration, ancho);
                }
            }
            timeline::Mensaje::Rueda { lineas, punto } => {
                // Girar la rueda es tomar el eje: si la vista siguiera al
                // playhead, el siguiente frame desharía el gesto.
                state.seguir = false;
                state.objetivo_vista = None;
                if state.modificadores.shift() {
                    // Zoom anclado al cursor: el tiempo bajo el ratón no se
                    // mueve. 1.15 por muesca, el mismo paso que el oráculo.
                    let factor = 1.15_f64.powf(f64::from(lineas));
                    aplicar_escala(state, state.vista.escala * factor, punto.x);
                } else {
                    // Pan proporcional: una muesca mueve un 10 % de lo que se
                    // ve. Con segundos fijos, el gesto sería un salto al zoom
                    // de cerca y un suspiro al de lejos.
                    let visible = state.vista.ancho_visible(state.ancho_timeline);
                    let paso = visible * 0.1 * f64::from(lineas);
                    state.vista = state
                        .vista
                        .desplazar(paso, state.duration, state.ancho_timeline);
                }
            }
            timeline::Mensaje::Pulsar(punto) => {
                let respuesta = timeline::pulsar(
                    state.editor.proyecto(),
                    state.vista,
                    punto,
                    &state.editor.seleccion,
                );
                match respuesta {
                    timeline::Respuesta::Buscar(tiempo) => buscar(state, tiempo),
                    timeline::Respuesta::Empezar {
                        arrastre,
                        seleccion,
                    } => {
                        state.editando_hablante = None;
                        state.editor.reemplazar_seleccion(seleccion);
                        sincronizar_borrador(state);
                        state.arrastre = Some(arrastre);
                    }
                }
            }
            timeline::Mensaje::Arrastrar(punto) => {
                let respaldo = match state.arrastre.as_mut() {
                    Some(arrastre) if !arrastre.es_marquesina() => {
                        let marco = timeline::Marco::nuevo(
                            state.vista,
                            punto,
                            state.duration,
                            state.position,
                        );
                        let cambio = arrastre.aplicar(state.editor.proyecto_mut(), marco);
                        // Un arrastre es **uno** en el historial, no uno por
                        // frame: el respaldo es el proyecto de antes de pulsar.
                        if cambio && !arrastre.con_historial {
                            arrastre.con_historial = true;
                            arrastre.respaldo.clone()
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                let cambiado = respaldo.is_some();
                if let Some(respaldo) = respaldo {
                    state.editor.registrar_instantanea(respaldo);
                }
                if cambiado {
                    refrescar_preview(state);
                }
            }
            timeline::Mensaje::Soltar(punto) => {
                let Some(arrastre) = state.arrastre.take() else {
                    return Task::none();
                };
                if arrastre.es_marquesina() {
                    let origen = arrastre.origen().unwrap_or(punto);
                    if (punto.x - origen.x).abs() < timeline::UMBRAL_CLIC
                        && (punto.y - origen.y).abs() < timeline::UMBRAL_CLIC
                    {
                        // Fue un clic: en un hueco del carril se busca, como en
                        // el oráculo.
                        buscar(state, state.vista.tiempo_de(punto.x));
                    } else {
                        let marco = timeline::Marco::nuevo(
                            state.vista,
                            punto,
                            state.duration,
                            state.position,
                        );
                        let ids = arrastre.ids_de_marquesina(state.editor.proyecto(), marco);
                        state.editor.reemplazar_seleccion(ids);
                        sincronizar_borrador(state);
                    }
                }
            }
        },
        Message::Zoom(factor) => {
            // El punto bajo el playhead se queda donde está: es lo que el ojo
            // está mirando cuando cambia el zoom.
            let ancla = state.vista.x_de(state.position).max(0.0);
            aplicar_escala(state, state.vista.escala / factor, ancla);
        }
        Message::Seguir => {
            state.seguir = !state.seguir;
            state.objetivo_vista = None;
        }
        Message::Modificadores(modificadores) => {
            state.modificadores = modificadores;
        }
    }
    Task::none()
}

fn view(state: &Estado) -> Element<'_, Message> {
    let video = container(
        shader(Video {
            pixels: Arc::new(state.pixels.clone()),
            width: WIDTH as u32,
            height: HEIGHT as u32,
            capas: Arc::clone(&state.layers),
        })
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(black_panel);

    let transport = container(
        column![
            row![
                button("Abrir video").on_press(Message::AbrirVideo),
                button("Abrir .cdp").on_press(Message::AbrirProyecto),
                button("Guardar").on_press(Message::GuardarProyecto),
                // Ancho fijo: si no, el botón cambia de tamaño al alternar y la
                // fila entera se corre bajo el cursor.
                button(if state.paused { "▶" } else { "❚❚" })
                    .on_press(Message::PlayPause)
                    .width(44),
                button("−5 s").on_press(Message::Step(-5.0)),
                button("+5 s").on_press(Message::Step(5.0)),
                pick_list(
                    state.audio_tracks.clone(),
                    state.selected_audio.clone(),
                    Message::AudioTrack,
                )
                .placeholder("Audio"),
                pick_list(VELOCIDADES, Some(state.velocidad), Message::Velocidad,).width(84),
                text(format!(
                    "{} / {}",
                    rod_core::format_time(state.position),
                    rod_core::format_time(state.duration)
                )),
            ]
            .spacing(8),
            row![
                text("Exportar").size(12),
                button("SRT por hablante").on_press(Message::ExportarSrtHablantes),
                button("JSON combinado").on_press(Message::ExportarJson),
                button(".ass").on_press(Message::ExportarAss),
            ]
            .spacing(8)
            .align_y(iced::Center),
            slider(
                0.0..=state.duration.max(0.01) as f32,
                state.position.clamp(0.0, state.duration.max(0.01)) as f32,
                Message::Arrastrar,
            )
            .on_release(Message::Soltar)
            .step(0.01_f32),
        ]
        .spacing(6)
        .padding(8),
    )
    .width(Length::Fill)
    .style(panel);

    let left = column![video, transport]
        .width(Length::FillPortion(3))
        .height(Length::Fill);
    let right = column![subtitle_panel(state), speakers_panel(state),]
        .width(Length::FillPortion(2))
        .height(Length::Fill);
    let timeline = container(
        row![
            Element::from(
                timeline::widget(timeline::Linea {
                    proyecto: state.editor.proyecto(),
                    vista: state.vista,
                    duracion: state.duration,
                    position: state.position,
                    seleccion: &state.editor.seleccion,
                    arrastre: state.arrastre.as_ref(),
                })
                .width(Length::Fill)
                .height(Length::Fill)
            )
            .map(Message::Timeline),
            column![
                button(if state.seguir { "sigue" } else { "manual" })
                    .on_press(Message::Seguir)
                    .width(62)
                    .style(if state.seguir { boton_activo } else { boton }),
                button("+").on_press(Message::Zoom(1.5)).width(62),
                button("−").on_press(Message::Zoom(1.0 / 1.5)).width(62),
            ]
            .spacing(4)
            .padding(4),
        ]
        .spacing(4),
    )
    .width(Length::Fill)
    .height(200)
    .style(panel);

    column![
        banner_recovery(state),
        row![left, right].height(Length::Fill),
        timeline,
        status_bar(state),
    ]
    .into()
}

/// Aviso de copia automática al arrancar. `None` (sin recovery pendiente) se
/// convierte en un elemento vacío de altura 0, así que la columna no cambia de
/// forma: es el equivalente a la pregunta que el oráculo lanza al montar, pero
/// sin bloquear la ventana mientras el usuario decide.
fn banner_recovery(state: &Estado) -> Option<Element<'_, Message>> {
    let aviso = state.recovery.as_ref()?;
    Some(
        container(
            row![
                text("● COPIA AUTOMÁTICA").size(13),
                text(aviso.texto.clone()),
                space::horizontal(),
                button("Recuperar").on_press(Message::Recuperar),
                button("Descartar").on_press(Message::DescartarRecovery),
            ]
            .spacing(12)
            .padding([6, 12]),
        )
        .width(Length::Fill)
        .style(banner)
        .into(),
    )
}

fn subtitle_panel(state: &Estado) -> Element<'_, Message> {
    let rows = state
        .editor
        .proyecto()
        .captions
        .iter()
        .map(|caption| {
            let selected = state.editor.seleccion.iter().any(|id| id == &caption.id);
            let label = format!(
                "{} {} -> {}\n{}",
                if selected { ">" } else { " " },
                rod_core::format_time(caption.inicio),
                rod_core::format_time(caption.fin),
                caption.texto
            );
            button(text(label))
                .on_press(Message::Seleccionar(caption.id.clone()))
                .width(Length::Fill)
                .into()
        })
        .collect::<Vec<Element<'_, Message>>>();

    container(
        column![
            row![
                text("Subtitles").size(20),
                space::horizontal(),
                button("↶")
                    .on_press_maybe(state.editor.puede_deshacer().then_some(Message::Deshacer))
                    .width(44),
                button("↷")
                    .on_press_maybe(state.editor.puede_rehacer().then_some(Message::Rehacer))
                    .width(44),
                button("Borrar").on_press_maybe(
                    (!state.editor.seleccion.is_empty()).then_some(Message::BorrarSeleccion)
                ),
            ]
            .spacing(12),
            row![
                text_input("Texto del subtítulo", &state.borrador)
                    .on_input(Message::Borrador)
                    .on_submit(Message::AplicarTexto),
                button("Aplicar").on_press_maybe(
                    (state.editor.seleccion.len() == 1).then_some(Message::AplicarTexto)
                ),
            ]
            .spacing(8),
            text("CAPTIONS").size(14),
            scrollable(column(rows).spacing(8)),
        ]
        .spacing(14)
        .padding(14),
    )
    .width(Length::FillPortion(2))
    .height(Length::Fill)
    .style(panel)
    .into()
}

fn speakers_panel(state: &Estado) -> Element<'_, Message> {
    let hablantes = state.editor.proyecto().hablantes.as_slice();
    let rows = hablantes
        .iter()
        .map(|hablante| {
            let id_nombre = hablante.id.clone();
            let id_tecla = hablante.id.clone();
            let id_asignar = hablante.id.clone();
            let hay_seleccion = !state.editor.seleccion.is_empty();
            row![
                text_input("Nombre", &hablante.nombre).on_input(move |texto| {
                    Message::EditarHablante(id_nombre.clone(), CampoHablante::Nombre, texto)
                }),
                text_input("1", &hablante.tecla)
                    .on_input(move |texto| Message::EditarHablante(
                        id_tecla.clone(),
                        CampoHablante::Tecla,
                        texto
                    ))
                    .width(44),
                button("Asignar").on_press_maybe(
                    hay_seleccion.then(|| Message::AsignarHablante(Some(id_asignar.clone())))
                ),
                button("×")
                    .on_press(Message::BorrarHablante(hablante.id.clone()))
                    .width(36),
            ]
            .spacing(6)
            .into()
        })
        .collect::<Vec<Element<'_, Message>>>();

    container(
        column![
            row![
                text("Speakers").size(20),
                space::horizontal(),
                button("Sin hablante").on_press_maybe(
                    (!state.editor.seleccion.is_empty()).then_some(Message::AsignarHablante(None))
                ),
                button("＋").on_press_maybe(
                    (hablantes.len() < MAX_HABLANTES).then_some(Message::AgregarHablante)
                ),
            ]
            .spacing(12),
            // El oráculo pinta aquí el dot con el color del preset `.ass`; sin
            // panel de estilos no hay preset que mostrar y `Hablante.color`
            // sería un color que el `.ass` exportado no usa.
            text("SPEAKERS").size(14),
            scrollable(column(rows).spacing(6)),
        ]
        .spacing(10)
        .padding(14),
    )
    .width(Length::Fill)
    .height(240)
    .style(panel)
    .into()
}

fn status_bar(state: &Estado) -> Element<'_, Message> {
    let proyecto = match &state.ruta_proyecto {
        Some(ruta) => ruta.display().to_string(),
        None => "sin guardar".to_owned(),
    };
    container(
        row![
            text("● RUSTYOPENDIALOGUE"),
            text(state.video_path.clone()),
            text(proyecto),
            // Último resultado de exportar (o nada). Va aquí y no en un modal
            // porque iced no tiene toasts y la barra de estado ya es el lugar
            // donde el usuario mira qué pasó.
            state.mensaje.clone().map(text),
            space::horizontal(),
            text(format!(
                "ventana {:.1} s · {} captions · {} capas libass",
                state.vista.ancho_visible(state.ancho_timeline),
                state.editor.proyecto().captions.len(),
                state.layers.len()
            )),
        ]
        .spacing(16)
        .padding([6, 12]),
    )
    .width(Length::Fill)
    .style(panel)
    .into()
}

fn black_panel(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::BLACK)),
        ..container::Style::default()
    }
}

fn panel(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.04, 0.05, 0.07))),
        border: iced::Border {
            color: Color::from_rgb(0.16, 0.18, 0.22),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// Aviso de recuperación: ámbar sobre el fondo oscuro. No es un error (no se
/// perdió nada todavía), pero sí algo que reclama una decisión.
fn banner(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.22, 0.16, 0.05))),
        text_color: Some(Color::from_rgb(0.95, 0.80, 0.55)),
        border: iced::Border {
            color: Color::from_rgb(0.55, 0.40, 0.12),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// Botón encendido: el estilo por defecto no distingue un toggle de un botón
/// cualquiera, y «sigue»/«manual» es un estado, no una acción.
fn boton_activo(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb(0.16, 0.30, 0.52))),
        text_color: Color::WHITE,
        border: iced::Border {
            color: Color::from_rgb(0.31, 0.49, 0.91),
            width: 1.0,
            radius: 2.0.into(),
        },
        ..button::Style::default()
    }
}

/// Botón apagado, con la forma del encendido para que el toggle no baile.
fn boton(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb(0.07, 0.08, 0.11))),
        text_color: Color::from_rgb(0.62, 0.66, 0.74),
        border: iced::Border {
            color: Color::from_rgb(0.16, 0.18, 0.22),
            width: 1.0,
            radius: 2.0.into(),
        },
        ..button::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{VELOCIDADES, Velocidad, recovery_pendiente, resumen_srt};
    use rod_app::project_io::ExportacionSrt;
    use rod_core::{Caption, Proyecto};

    #[test]
    fn la_etiqueta_no_arrastra_decimales_inutiles() {
        assert_eq!(Velocidad(1.0).to_string(), "1x");
        assert_eq!(Velocidad(2.0).to_string(), "2x");
        assert_eq!(Velocidad(0.5).to_string(), "0.5x");
        assert_eq!(Velocidad(1.25).to_string(), "1.25x");
    }

    #[test]
    fn la_lista_esta_ordenada_y_contiene_la_normal() {
        assert!(VELOCIDADES.windows(2).all(|par| par[0].0 < par[1].0));
        assert!(VELOCIDADES.contains(&Velocidad(1.0)));
    }

    fn exportacion(sin_asignar: usize, solapes: usize) -> ExportacionSrt {
        ExportacionSrt {
            archivos: vec![("Ana.srt".into(), String::new())],
            reporte_solapes: (solapes > 0).then(|| "reporte".into()),
            sin_asignar,
            solapes,
        }
    }

    #[test]
    fn el_resumen_nombra_lo_que_quedo_fuera() {
        let texto = resumen_srt(&exportacion(2, 1), 1);
        assert!(texto.contains("Exportados 1 SRT"), "{texto}");
        assert!(texto.contains("2 caption(s) sin hablante"), "{texto}");
        assert!(texto.contains("1 solape(s)"), "{texto}");
    }

    #[test]
    fn el_resumen_no_menciona_lo_que_no_paso() {
        let texto = resumen_srt(&exportacion(0, 0), 3);
        assert_eq!(texto, "Exportados 3 SRT");
    }

    #[test]
    fn sin_hablantes_el_resumen_lo_dice() {
        let texto = resumen_srt(&exportacion(4, 0), 0);
        assert!(texto.contains("ningún caption tiene hablante"), "{texto}");
    }

    /// Un recovery propio por test: los tests corren en paralelo.
    fn ruta_temporal(nombre: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("rod-aviso-{}-{nombre}.json", std::process::id()))
    }

    #[test]
    fn sin_archivo_no_hay_aviso() {
        let ruta = ruta_temporal("inexistente");
        assert!(recovery_pendiente(&ruta, 1000.0).is_none());
    }

    #[test]
    fn un_recovery_valido_se_ofrece_con_el_nombre_del_archivo() {
        let ruta = ruta_temporal("valido");
        let proyecto = Proyecto {
            ruta_video: "testdata/clip-15s-2audio.mp4".into(),
            hablantes: Vec::new(),
            captions: vec![Caption {
                id: "cap-1".into(),
                inicio: 1.0,
                fin: 2.0,
                texto: "Hola".into(),
                hablante_id: None,
            }],
            playhead: 1.5,
        };
        let copia = rod_core::construir_recovery(proyecto, "C:/proyectos/mi-video.cdp", 1000.0);
        let texto = rod_core::recovery::a_json(&copia).expect("serializa");
        std::fs::write(&ruta, texto).expect("escribe");

        let aviso = recovery_pendiente(&ruta, 1000.0).expect("hay aviso");
        // El aviso usa el nombre del archivo, no la ruta entera, y cuenta los
        // subtítulos: es lo que el usuario necesita para decidir.
        assert!(aviso.texto.contains("mi-video.cdp"), "{}", aviso.texto);
        assert!(aviso.texto.contains("1 subtítulo"), "{}", aviso.texto);
        assert!(!aviso.texto.contains("C:/proyectos"), "{}", aviso.texto);
        std::fs::remove_file(&ruta).expect("limpia");
    }

    #[test]
    fn un_archivo_corrupto_no_ofrece_nada() {
        let ruta = ruta_temporal("corrupto");
        std::fs::write(&ruta, "{roto").expect("escribe");
        assert!(recovery_pendiente(&ruta, 1000.0).is_none());
        std::fs::remove_file(&ruta).expect("limpia");
    }
}
