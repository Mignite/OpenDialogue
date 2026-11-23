use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iced::widget::{
    button, column, container, pick_list, row, scrollable, shader, slider, space, text, text_input,
};
use iced::{Background, Element, Length, Subscription, Task};

use crate::comandos::Comando;
use crate::editor::{EstadoEditor, MAX_HABLANTES, normalizar_tecla};
use crate::presets_edicion::{CampoPreset, aplicar_campo, valor_campo};
use crate::presets_io;
use crate::preview::{CANVAS, ass_del_proyecto};
use crate::project_io::{
    Documento, ExportacionSrt, FormatoSubtitulo, abrir_proyecto, exportar_json_combinado,
    exportar_srt_por_hablante, guardar_proyecto,
};
use crate::recovery_io::{self, AvisoRecovery};
use crate::timeline::{self, Vista};
use crate::video::{Capa, Video};
use crate::{recientes, tema, thumbnail};
use rod_core::{PresetAss, Proyecto, nombre_libre, nuevo_preset_desde};
use rod_player::AudioTrack;

const WIDTH: usize = CANVAS.0 as usize;
const HEIGHT: usize = CANVAS.1 as usize;
const ID_TEXTO_SUBTITULO: iced::widget::Id = iced::widget::Id::new("texto-subtitulo");

/// Velocidades que ofrece la barra de transporte. `pick_list` necesita un tipo
/// propio porque `f32` no es `Display` y el texto de la lista es la etiqueta.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Velocidad(f32);

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

/// Una opción del selector de estilo de un hablante. `pick_list` exige un tipo
/// `Display` y `PresetAss` no lo es; `id: None` es «sin preset» (usa el estilo
/// `Default` del `.ass`).
#[derive(Debug, Clone, PartialEq)]
struct OpcionPreset {
    id: Option<String>,
    nombre: String,
}

impl std::fmt::Display for OpcionPreset {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.nombre)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CampoHablante {
    Nombre,
    Tecla,
}

/// Las tres pestañas de la columna derecha, como en el oráculo (`PanelTabs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Subtitulos,
    Hablantes,
    Estilos,
}

#[derive(Debug, Clone)]
pub enum MensajeEditor {
    /// Un comando del menú o del teclado. Un solo manejador para los dos.
    Comando(Comando),
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
    /// Cambia el preset `.ass` de un hablante (`None` = sin preset).
    AsignarPreset(String, Option<String>),
    /// Pestaña de la columna derecha.
    Panel(Panel),
    /// Preset abierto en el panel de estilos.
    SeleccionarPreset(String),
    /// Duplica el preset seleccionado (parte del actual, como el oráculo).
    NuevoPreset,
    BorrarPreset,
    /// Un campo del preset. El id y el campo son la clave del borrador.
    EditarPreset(String, CampoPreset, String),
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

pub struct EditorApp {
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
    /// Pestaña visible de la columna derecha.
    panel: Panel,
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
    /// Ruta del archivo compartido con el oráculo, resuelta una vez al arrancar.
    ruta_presets: PathBuf,
    /// Preset abierto en el panel de estilos. Si el archivo traía presets, el
    /// primero; `None` solo si el panel no tiene nada que mostrar (no pasa:
    /// siempre hay al menos la semilla).
    preset_seleccionado: Option<String>,
    /// Texto a medias de los campos numéricos, por id de preset y campo. Sin
    /// esto, escribir `72.5` se quedaría en `72` al llegar al punto.
    borradores_preset: HashMap<(String, CampoPreset), String>,
    /// Resultado de la última acción que no tiene otro lugar donde contarse
    /// (exportar, por ahora). iced no tiene toasts; la barra de estado es el
    /// sitio donde el usuario ya mira el estado del proyecto.
    mensaje: Option<String>,
    /// La miniatura del reciente faltaba al abrir: se genera con el primer
    /// frame que rasterice mpv, no con el buffer vacío del constructor.
    miniatura_pendiente: bool,
    /// Portapapeles interno para Ctrl+C/V entre captions.
    portapapeles: String,
}

pub fn subscription(_state: &EditorApp) -> Subscription<MensajeEditor> {
    Subscription::batch([
        iced::window::frames().map(|_| MensajeEditor::Tick),
        // Los modificadores se copian aparte: `WheelScrolled` no los trae y el
        // «Shift + rueda = zoom» necesita saberlos. El teclado ya no vive aquí:
        // `main.rs` mapea las tecas a `Comando`, el mismo registro que el menú.
        iced::keyboard::listen().filter_map(|evento| match evento {
            iced::keyboard::Event::ModifiersChanged(modifiers) => {
                Some(MensajeEditor::Modificadores(modifiers))
            }
            _ => None,
        }),
    ])
}

impl EditorApp {
    /// Proyecto nuevo en memoria: captions vacíos y todavía sin `.cdp`.
    pub fn desde_video(path: &str) -> Result<Self, String> {
        Self::desde_proyecto(Proyecto::nuevo(path), None, 0.0)
    }

    /// Abre un `.cdp` ya cargado. `posicion` es su `playhead` guardado.
    pub fn desde_proyecto(
        proyecto: Proyecto,
        ruta: Option<PathBuf>,
        posicion: f64,
    ) -> Result<Self, String> {
        Self::desde_proyecto_en(proyecto, ruta, posicion, &thumbnail::carpeta_cache())
    }

    pub fn desde_proyecto_en(
        proyecto: Proyecto,
        ruta: Option<PathBuf>,
        posicion: f64,
        base_thumbs: &Path,
    ) -> Result<Self, String> {
        let video_path = proyecto.ruta_video.clone();
        let player = rod_player::Player::open(&video_path)
            .map_err(|error| format!("no se pudo abrir {video_path}: {error}"))?;
        let documento = Documento {
            proyecto,
            formato: FormatoSubtitulo::Srt,
            estilos_ass: Vec::new(),
        };
        let editor = EstadoEditor::nuevo(documento);
        let duration = player.duration();
        let paused = player.paused();
        let audio_tracks = player.audio_tracks();
        let selected_audio = audio_tracks.first().cloned();
        let velocidad = Velocidad(player.speed() as f32);
        // Los presets se leen del archivo que comparte el oráculo. El panel de
        // estilos escribe en esa misma ruta, así que la próxima sesión arranca
        // con lo que el usuario dejó guardado.
        let ruta_presets = presets_io::ruta();
        let presets = presets_io::leer(&ruta_presets);
        let preset_seleccionado = presets.first().map(|preset| preset.id.clone());
        let ass = ass_del_proyecto(&editor, &presets);
        let subtitles = rod_subs::Subtitulos::nuevo(WIDTH as u32, HEIGHT as u32, &ass);
        let limpio = editor.proyecto().clone();
        let miniatura_pendiente = ruta
            .as_ref()
            .is_some_and(|ruta| !thumbnail::ruta_cache_en(base_thumbs, ruta).is_file());

        let mut estado = Self {
            editor,
            player,
            subtitles,
            pixels: vec![0; WIDTH * HEIGHT * 4],
            layers: Arc::new(Vec::new()),
            duration,
            position: posicion,
            arrastrando: false,
            paused,
            video_path,
            audio_tracks,
            selected_audio,
            velocidad,
            borrador: String::new(),
            panel: Panel::Subtitulos,
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
            ruta_proyecto: ruta,
            limpio,
            recovery: None,
            ultima_escritura: None,
            // 0 = todavía no se ha chequeado; el primer `Tick` ya decide con
            // el proyecto limpio y no escribe nada.
            ultimo_chequeo: 0.0,
            presets,
            ruta_presets,
            preset_seleccionado,
            borradores_preset: HashMap::new(),
            mensaje: None,
            miniatura_pendiente,
            portapapeles: String::new(),
        };
        // El playhead del `.cdp` no se copia solo al estado: mpv sigue en 0.
        // Sin este seek, el primer `Tick` sobreescribe `position` con 0 y la
        // reanudación se pierde (y con ella la miniatura del reciente).
        if estado.position > 0.0
            && let Err(error) = estado.player.seek(estado.position)
        {
            eprintln!("no se pudo buscar el playhead guardado: {error}");
        }
        refrescar_preview(&mut estado);
        Ok(estado)
    }

    /// Con el primer frame ya rasterizado se regenera la miniatura que faltaba
    /// al abrir. No se hace en el constructor: allí `pixels` está a ceros.
    fn regenerar_miniatura_en(&mut self, base: &Path) {
        if self.miniatura_pendiente
            && let Some(ruta) = self.ruta_proyecto.clone()
        {
            let _ = thumbnail::guardar_en(base, &ruta, &self.pixels, WIDTH as u32, HEIGHT as u32);
            self.miniatura_pendiente = false;
        }
    }
}

/// Cambia el video de la app sin tocar el proyecto. `position` es dónde queda
/// el playhead: 0 al abrir un video suelto, el guardado al abrir un proyecto.
fn reemplazar_player(
    state: &mut EditorApp,
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

/// ¿El proyecto en memoria difiere del que está en disco? Solo cuentan los
/// captions y los hablantes: el playhead se mueve al reproducir y no es una
/// edición.
fn sucio(state: &EditorApp) -> bool {
    let proyecto = state.editor.proyecto();
    proyecto.captions != state.limpio.captions || proyecto.hablantes != state.limpio.hablantes
}

/// Seguro ante crash: cada 15 s, si hay cambios sin guardar, escribe una copia
/// del proyecto en `recovery.json`. Falla en silencio a propósito: si el disco
/// está lleno o no hay permisos, el aviso al usuario sigue siendo el guardado
/// manual, no esto.
fn autosave(state: &mut EditorApp) {
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
fn recuperar(state: &mut EditorApp) {
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
fn descartar_recovery(state: &mut EditorApp) {
    state.recovery = None;
    borrar_recovery(state);
}

fn borrar_recovery(state: &mut EditorApp) {
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
    state: &mut EditorApp,
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
fn buscar(state: &mut EditorApp, tiempo: f64) {
    state.position = tiempo.clamp(0.0, state.duration);
    if let Err(error) = state.player.seek(state.position) {
        eprintln!("no se pudo hacer seek desde el timeline: {error}");
    }
}

/// Cambia el zoom dejando quieto `ancla` (px) y deja la ventana dentro del
/// video: al alejarse, un inicio que cabía puede quedar más allá del final.
fn aplicar_escala(state: &mut EditorApp, escala: f64, ancla: f32) {
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
fn seguir_playhead(state: &mut EditorApp) {
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
fn refrescar_preview(state: &mut EditorApp) {
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

/// Cada cambio del panel de estilos se guarda al instante, como en el oráculo:
/// no hay botón «Guardar» y cerrar la app no debe perder lo que se acaba de
/// teclear. El preview se recarga con la misma lista, así que lo que se ve es
/// lo que se escribió en el `.ass`.
fn guardar_presets(state: &mut EditorApp) -> Task<MensajeEditor> {
    state.mensaje = match presets_io::escribir(&state.ruta_presets, &state.presets) {
        Ok(()) => None,
        Err(error) => Some(format!("No se pudieron guardar los estilos: {error}")),
    };
    refrescar_preview(state);
    Task::none()
}

/// El `text_input` muestra el texto del caption seleccionado. Sin selección
/// (o con varios) queda vacío: no hay un texto único que editar.
fn sincronizar_borrador(state: &mut EditorApp) {
    let id = match state.editor.seleccion.as_slice() {
        [id] => Some(id.clone()),
        _ => None,
    };
    state.borrador = id
        .and_then(|id| state.editor.texto_de(&id).map(str::to_owned))
        .unwrap_or_default();
}

/// Identificador único para captions nuevos, sin depender de una crate más.
fn id_nuevo() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duracion| duracion.as_nanos())
        .unwrap_or(0);
    format!("cap-{nanos}")
}

/// Un único manejador para el menú y el teclado: los dos entregan un `Comando`.
fn ejecutar_comando(state: &mut EditorApp, comando: Comando) -> Task<MensajeEditor> {
    use Comando as C;
    match comando {
        C::Guardar => update(state, MensajeEditor::GuardarProyecto),
        C::GuardarComo => update(state, MensajeEditor::GuardarProyectoComo),
        C::ExportarSrt => update(state, MensajeEditor::ExportarSrtHablantes),
        C::ExportarJson => update(state, MensajeEditor::ExportarJson),
        C::ExportarAss => update(state, MensajeEditor::ExportarAss),
        C::Estilos => update(state, MensajeEditor::Panel(Panel::Estilos)),
        C::PanelSubtitulos => update(state, MensajeEditor::Panel(Panel::Subtitulos)),
        C::PanelHablantes => update(state, MensajeEditor::Panel(Panel::Hablantes)),
        C::PanelEstilos => update(state, MensajeEditor::Panel(Panel::Estilos)),
        C::Deshacer => update(state, MensajeEditor::Deshacer),
        C::Rehacer => update(state, MensajeEditor::Rehacer),
        C::ReproducirPausa => update(state, MensajeEditor::PlayPause),
        C::Retroceder => update(state, MensajeEditor::Step(-5.0)),
        C::Avanzar => update(state, MensajeEditor::Step(5.0)),
        C::RetrocederFino => update(state, MensajeEditor::Step(-0.1)),
        C::AvanzarFino => update(state, MensajeEditor::Step(0.1)),
        C::Eliminar => update(state, MensajeEditor::BorrarSeleccion),
        C::Hablante(numero) => {
            let tecla = numero.to_string();
            let id = state
                .editor
                .proyecto()
                .hablantes
                .iter()
                .find(|hablante| hablante.tecla == tecla)
                .map(|hablante| hablante.id.clone());
            match id {
                Some(id) => update(state, MensajeEditor::AsignarHablante(Some(id))),
                None => {
                    state.mensaje = Some(format!("No hay hablante con la tecla {numero}"));
                    Task::none()
                }
            }
        }
        C::NudgeIzquierda => nudge_seleccion(state, -0.1),
        C::NudgeDerecha => nudge_seleccion(state, 0.1),
        C::AgregarFragmento => agregar_fragmento(state),
        C::Dividir => dividir_en_playhead(state),
        C::CaptionAnterior => ciclar_caption(state, -1),
        C::CaptionSiguiente => ciclar_caption(state, 1),
        C::SimultaneoArriba => ciclar_simultaneo(state, -1),
        C::SimultaneoAbajo => ciclar_simultaneo(state, 1),
        C::Copiar => copiar(state),
        C::Pegar => pegar(state),
        C::VelocidadMas | C::VelocidadMenos => {
            let actual = VELOCIDADES
                .iter()
                .position(|velocidad| velocidad == &state.velocidad)
                .unwrap_or(2);
            let salto = if comando == C::VelocidadMas { 1 } else { -1 };
            let indice =
                (actual as isize + salto).clamp(0, VELOCIDADES.len() as isize - 1) as usize;
            update(state, MensajeEditor::Velocidad(VELOCIDADES[indice]))
        }
        C::EditarTexto => {
            if state.editor.seleccion.len() > 1 {
                state.mensaje = Some("Selecciona un solo subtítulo para editar su texto".into());
                Task::none()
            } else {
                iced::widget::operation::focus(ID_TEXTO_SUBTITULO)
            }
        }
        C::Buscar => {
            state.mensaje = Some("Buscar: usa el panel de subtítulos para localizar texto".into());
            Task::none()
        }
        C::Ayuda => {
            state.mensaje = Some(
                "Atajos: Space, ←/→, Alt+←/→, Shift+←/→, Ctrl+Shift+←/→, E, A, C, Delete, Ctrl+Z/Y/C/V, ?, +/−".into(),
            );
            Task::none()
        }
        C::ImportarAutosubs => {
            state.mensaje = Some("Importar auto-subs: pendiente en este slice".into());
            Task::none()
        }
        C::CargarSrt | C::CargarAss => cargar_subtitulos(state, comando == C::CargarAss),
        C::NuevoProyecto | C::AbrirProyecto | C::AbrirVideo => {
            state.mensaje = Some("Usa el menú de inicio para abrir un proyecto".into());
            Task::none()
        }
    }
}

fn nudge_seleccion(state: &mut EditorApp, delta: f64) -> Task<MensajeEditor> {
    let duracion = state.duration;
    let ids = state.editor.seleccion.clone();
    for id in ids {
        let Some(caption) = state
            .editor
            .proyecto()
            .captions
            .iter()
            .find(|caption| caption.id == id)
            .cloned()
        else {
            continue;
        };
        state
            .editor
            .editar_tiempos(&id, caption.inicio + delta, caption.fin + delta, duracion);
    }
    refrescar_preview(state);
    Task::none()
}

fn agregar_fragmento(state: &mut EditorApp) -> Task<MensajeEditor> {
    let inicio = state.position;
    let fin = (inicio + 2.0).min(state.duration.max(inicio + 0.5));
    let caption = rod_core::Caption {
        id: id_nuevo(),
        inicio,
        fin,
        texto: String::new(),
        hablante_id: None,
    };
    state.editor.agregar(caption);
    refrescar_preview(state);
    Task::none()
}

fn dividir_en_playhead(state: &mut EditorApp) -> Task<MensajeEditor> {
    let Some(id) = state.editor.seleccion.first().cloned() else {
        return Task::none();
    };
    let Some(caption) = state
        .editor
        .proyecto()
        .captions
        .iter()
        .find(|caption| caption.id == id)
        .cloned()
    else {
        return Task::none();
    };
    let corte = state
        .position
        .clamp(caption.inicio + 0.05, caption.fin - 0.05);
    if corte <= caption.inicio || corte >= caption.fin {
        return Task::none();
    }
    state
        .editor
        .editar_tiempos(&id, caption.inicio, corte, state.duration);
    let segunda = rod_core::Caption {
        id: id_nuevo(),
        inicio: corte,
        fin: caption.fin,
        texto: caption.texto.clone(),
        hablante_id: caption.hablante_id.clone(),
    };
    state.editor.agregar(segunda);
    refrescar_preview(state);
    Task::none()
}

fn ciclar_caption(state: &mut EditorApp, paso: i32) -> Task<MensajeEditor> {
    let captions = &state.editor.proyecto().captions;
    if captions.is_empty() {
        return Task::none();
    }
    let seleccionada = state
        .editor
        .seleccion
        .first()
        .and_then(|id| captions.iter().position(|caption| &caption.id == id));
    let indice = match seleccionada {
        Some(actual) => {
            (actual as isize + paso as isize).rem_euclid(captions.len() as isize) as usize
        }
        None => {
            if paso < 0 {
                captions.len() - 1
            } else {
                0
            }
        }
    };
    let id = captions[indice].id.clone();
    state.editor.seleccionar_uno(&id);
    Task::none()
}

fn ciclar_simultaneo(state: &mut EditorApp, paso: i32) -> Task<MensajeEditor> {
    let simultaneos: Vec<String> = state
        .editor
        .proyecto()
        .captions
        .iter()
        .filter(|caption| caption.inicio <= state.position && caption.fin >= state.position)
        .map(|caption| caption.id.clone())
        .collect();
    if simultaneos.is_empty() {
        return ciclar_caption(state, paso);
    }
    let actual = state
        .editor
        .seleccion
        .first()
        .and_then(|id| simultaneos.iter().position(|candidato| candidato == id));
    let indice = match actual {
        Some(actual) => {
            (actual as isize + paso as isize).rem_euclid(simultaneos.len() as isize) as usize
        }
        None => 0,
    };
    state.editor.seleccionar_uno(&simultaneos[indice]);
    Task::none()
}

fn copiar(state: &mut EditorApp) -> Task<MensajeEditor> {
    if let Some(id) = state.editor.seleccion.first()
        && let Some(caption) = state
            .editor
            .proyecto()
            .captions
            .iter()
            .find(|c| &c.id == id)
    {
        state.portapapeles = format!(
            "{} --> {}\n{}",
            rod_core::format_srt_timestamp(caption.inicio),
            rod_core::format_srt_timestamp(caption.fin),
            caption.texto
        );
    }
    Task::none()
}

fn pegar(state: &mut EditorApp) -> Task<MensajeEditor> {
    if state.portapapeles.trim().is_empty() {
        return Task::none();
    }
    // Insertar fragmentos, nunca reconstruir el editor: `EstadoEditor::nuevo`
    // reinicia el historial y `limpio`, y un Ctrl+V con el portapapeles vacío
    // borraría el proyecto sin posibilidad de deshacer.
    let Ok(documento) = crate::project_io::cargar(&state.portapapeles, &state.video_path) else {
        return Task::none();
    };
    for caption in documento.proyecto.captions {
        state.editor.agregar(caption);
    }
    refrescar_preview(state);
    Task::none()
}

fn cargar_subtitulos(state: &mut EditorApp, ass: bool) -> Task<MensajeEditor> {
    let filtro = if ass { "ass" } else { "srt" };
    let Some(ruta) = rfd::FileDialog::new()
        .add_filter("Subtítulos", &[filtro])
        .pick_file()
    else {
        return Task::none();
    };
    let Ok(texto) = std::fs::read_to_string(ruta) else {
        state.mensaje = Some("No se pudo leer el archivo de subtítulos".into());
        return Task::none();
    };
    match crate::project_io::cargar(&texto, &state.video_path) {
        Ok(documento) => {
            state.editor = EstadoEditor::nuevo(documento);
            state.limpio = state.editor.proyecto().clone();
            refrescar_preview(state);
        }
        Err(error) => {
            state.mensaje = Some(format!("No se pudieron cargar los subtítulos: {error}"))
        }
    }
    Task::none()
}

pub fn update(state: &mut EditorApp, message: MensajeEditor) -> Task<MensajeEditor> {
    match message {
        MensajeEditor::Comando(comando) => return ejecutar_comando(state, comando),
        MensajeEditor::Tick => {
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
                state.regenerar_miniatura_en(&thumbnail::carpeta_cache());
            }
        }
        MensajeEditor::AbrirVideo => {
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
        MensajeEditor::AbrirProyecto => {
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
        MensajeEditor::GuardarProyecto | MensajeEditor::GuardarProyectoComo => {
            let pedir_ruta = matches!(message, MensajeEditor::GuardarProyectoComo)
                || state.ruta_proyecto.is_none();
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
                    if let Err(error) = recientes::registrar_guardado(
                        &ruta,
                        &recientes::ruta_indice(),
                        &thumbnail::carpeta_cache(),
                        &state.pixels,
                        WIDTH as u32,
                        HEIGHT as u32,
                    ) {
                        state.mensaje = Some(format!(
                            "Proyecto guardado, pero no se pudo registrar en recientes: {error}"
                        ));
                    }
                    state.miniatura_pendiente = false;
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
        MensajeEditor::Recuperar => recuperar(state),
        MensajeEditor::DescartarRecovery => descartar_recovery(state),
        MensajeEditor::ExportarSrtHablantes => {
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
        MensajeEditor::ExportarJson => {
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
        MensajeEditor::ExportarAss => {
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
        MensajeEditor::PlayPause => {
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
        MensajeEditor::AudioTrack(track) => match state.player.set_audio_track(track.id) {
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
        MensajeEditor::Velocidad(velocidad) => match state.player.set_speed(velocidad.0 as f64) {
            // Se relee de mpv, que acota la velocidad a su rango.
            Ok(()) => state.velocidad = Velocidad(state.player.speed() as f32),
            Err(error) => eprintln!("no se pudo cambiar la velocidad: {error}"),
        },
        MensajeEditor::Arrastrar(position) => {
            state.arrastrando = true;
            state.position = position as f64;
        }
        MensajeEditor::Soltar => {
            state.arrastrando = false;
            if let Err(error) = state.player.seek(state.position) {
                eprintln!("no se pudo hacer seek: {error}");
            }
        }
        MensajeEditor::Step(delta) => {
            state.position = (state.position + delta).clamp(0.0, state.duration);
            if let Err(error) = state.player.seek(state.position) {
                eprintln!("no se pudo hacer seek: {error}");
            }
        }
        MensajeEditor::Seleccionar(id) => {
            state.editor.seleccionar_uno(&id);
            sincronizar_borrador(state);
        }
        MensajeEditor::Borrador(texto) => state.borrador = texto,
        MensajeEditor::AplicarTexto => {
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
        MensajeEditor::BorrarSeleccion => {
            // Sin selección no hay paso de historial: un Ctrl+Z vacío es peor
            // que un botón apagado.
            if !state.editor.seleccion.is_empty() {
                state.editor.registrar_historial();
                state.editor.borrar_seleccion();
                sincronizar_borrador(state);
                refrescar_preview(state);
            }
        }
        MensajeEditor::Deshacer => {
            state.editando_hablante = None;
            if state.editor.deshacer() {
                sincronizar_borrador(state);
                refrescar_preview(state);
            }
        }
        MensajeEditor::Rehacer => {
            state.editando_hablante = None;
            if state.editor.rehacer() {
                sincronizar_borrador(state);
                refrescar_preview(state);
            }
        }
        MensajeEditor::AgregarHablante => {
            state.editando_hablante = None;
            state.editor.agregar_hablante();
        }
        MensajeEditor::EditarHablante(id, campo, valor) => {
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
        MensajeEditor::BorrarHablante(id) => {
            state.editando_hablante = None;
            if state.editor.borrar_hablante(&id) {
                refrescar_preview(state);
            }
        }
        MensajeEditor::AsignarHablante(hablante_id) => {
            state.editando_hablante = None;
            if state.editor.asignar_a_seleccion(hablante_id) > 0 {
                refrescar_preview(state);
            }
        }
        MensajeEditor::AsignarPreset(hablante_id, preset_id) => {
            state.editando_hablante = None;
            if state.editor.asignar_preset(&hablante_id, preset_id) {
                refrescar_preview(state);
            }
        }
        MensajeEditor::Panel(panel) => {
            state.panel = panel;
            state.editando_hablante = None;
        }
        MensajeEditor::SeleccionarPreset(id) => {
            state.preset_seleccionado = Some(id);
        }
        MensajeEditor::NuevoPreset => {
            // Parte del preset abierto: casi siempre se quiere ajustar lo que
            // ya gusta, no volver de cero. El nombre se renumera solo.
            let base = state
                .presets
                .iter()
                .find(|preset| Some(&preset.id) == state.preset_seleccionado.as_ref())
                .or_else(|| state.presets.first())
                .cloned();
            let Some(base) = base else {
                return Task::none();
            };
            let nombres = state
                .presets
                .iter()
                .map(|preset| preset.nombre.clone())
                .collect::<Vec<_>>();
            let mut nuevo = nuevo_preset_desde(&base);
            nuevo.nombre = nombre_libre(&base.nombre, &nombres);
            state.preset_seleccionado = Some(nuevo.id.clone());
            state.presets.push(nuevo);
            return guardar_presets(state);
        }
        MensajeEditor::BorrarPreset => {
            if state.presets.len() <= 1 {
                return Task::none();
            }
            let Some(id) = state.preset_seleccionado.clone() else {
                return Task::none();
            };
            state.presets.retain(|preset| preset.id != id);
            state
                .borradores_preset
                .retain(|(preset_id, _), _| preset_id != &id);
            state.preset_seleccionado = state.presets.first().map(|preset| preset.id.clone());
            // No se toca `preset_id` de los hablantes: el oráculo tampoco lo
            // limpia, y `ass_del_proyecto` ya cae al Default si el id no existe.
            return guardar_presets(state);
        }
        MensajeEditor::EditarPreset(id, campo, texto) => {
            let aplicado = state
                .presets
                .iter_mut()
                .find(|preset| preset.id == id)
                .is_some_and(|preset| aplicar_campo(preset, campo, &texto));
            if !aplicado {
                // Valor a medias: el preset sigue intacto y el `text_input`
                // muestra este texto hasta que sea aceptable.
                state.borradores_preset.insert((id, campo), texto);
                return Task::none();
            }
            state.borradores_preset.remove(&(id.clone(), campo));
            return guardar_presets(state);
        }
        MensajeEditor::Timeline(mensaje) => match mensaje {
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
        MensajeEditor::Zoom(factor) => {
            // El punto bajo el playhead se queda donde está: es lo que el ojo
            // está mirando cuando cambia el zoom.
            let ancla = state.vista.x_de(state.position).max(0.0);
            aplicar_escala(state, state.vista.escala / factor, ancla);
        }
        MensajeEditor::Seguir => {
            state.seguir = !state.seguir;
            state.objetivo_vista = None;
        }
        MensajeEditor::Modificadores(modificadores) => {
            state.modificadores = modificadores;
        }
    }
    Task::none()
}

pub fn view(state: &EditorApp) -> Element<'_, MensajeEditor> {
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
                button("Abrir video")
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::AbrirVideo),
                button("Abrir .cdp")
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::AbrirProyecto),
                button("Guardar")
                    .style(tema::boton_primario)
                    .on_press(MensajeEditor::GuardarProyecto),
                // Ancho fijo: si no, el botón cambia de tamaño al alternar y la
                // fila entera se corre bajo el cursor.
                button(if state.paused { "▶" } else { "❚❚" })
                    .on_press(MensajeEditor::PlayPause)
                    .width(44),
                button("−5 s")
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::Step(-5.0)),
                button("+5 s")
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::Step(5.0)),
                pick_list(
                    state.audio_tracks.clone(),
                    state.selected_audio.clone(),
                    MensajeEditor::AudioTrack,
                )
                .placeholder("Audio"),
                pick_list(VELOCIDADES, Some(state.velocidad), MensajeEditor::Velocidad,).width(84),
                text(format!(
                    "{} / {}",
                    rod_core::format_time(state.position),
                    rod_core::format_time(state.duration)
                )),
            ]
            .spacing(8),
            row![
                text("Exportar").size(12),
                button("SRT por hablante")
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::ExportarSrtHablantes),
                button("JSON combinado")
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::ExportarJson),
                button(".ass")
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::ExportarAss),
            ]
            .spacing(8)
            .align_y(iced::Center),
            slider(
                0.0..=state.duration.max(0.01) as f32,
                state.position.clamp(0.0, state.duration.max(0.01)) as f32,
                MensajeEditor::Arrastrar,
            )
            .on_release(MensajeEditor::Soltar)
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
    let pestanas = row![
        pestana("Subtítulos", Panel::Subtitulos, state.panel),
        pestana("Hablantes", Panel::Hablantes, state.panel),
        pestana("Estilos", Panel::Estilos, state.panel),
    ]
    .spacing(4)
    .padding(4);
    let contenido = match state.panel {
        Panel::Subtitulos => subtitle_panel(state),
        Panel::Hablantes => speakers_panel(state),
        Panel::Estilos => styles_panel(state),
    };
    let right = column![pestanas, contenido]
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
            .map(MensajeEditor::Timeline),
            column![
                button(if state.seguir { "sigue" } else { "manual" })
                    .on_press(MensajeEditor::Seguir)
                    .width(62)
                    .style(if state.seguir { boton_activo } else { boton }),
                button("+")
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::Zoom(1.5))
                    .width(62),
                button("−")
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::Zoom(1.0 / 1.5))
                    .width(62),
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

fn pestana(
    etiqueta: &'static str,
    destino: Panel,
    actual: Panel,
) -> Element<'static, MensajeEditor> {
    button(etiqueta)
        .style(tema::boton_secundario)
        .on_press(MensajeEditor::Panel(destino))
        .width(Length::Fill)
        .style(if actual == destino {
            boton_activo
        } else {
            boton
        })
        .into()
}

/// Aviso de copia automática al arrancar. `None` (sin recovery pendiente) se
/// convierte en un elemento vacío de altura 0, así que la columna no cambia de
/// forma: es el equivalente a la pregunta que el oráculo lanza al montar, pero
/// sin bloquear la ventana mientras el usuario decide.
fn banner_recovery(state: &EditorApp) -> Option<Element<'_, MensajeEditor>> {
    let aviso = state.recovery.as_ref()?;
    Some(
        container(
            row![
                text("● COPIA AUTOMÁTICA").size(13),
                text(aviso.texto.clone()),
                space::horizontal(),
                button("Recuperar")
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::Recuperar),
                button("Descartar")
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::DescartarRecovery),
            ]
            .spacing(12)
            .padding([6, 12]),
        )
        .width(Length::Fill)
        .style(banner)
        .into(),
    )
}

fn subtitle_panel(state: &EditorApp) -> Element<'_, MensajeEditor> {
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
                .style(tema::boton_secundario)
                .on_press(MensajeEditor::Seleccionar(caption.id.clone()))
                .width(Length::Fill)
                .into()
        })
        .collect::<Vec<Element<'_, MensajeEditor>>>();

    container(
        column![
            row![
                text("Subtitles").size(20),
                space::horizontal(),
                button("↶")
                    .style(tema::boton_secundario)
                    .on_press_maybe(
                        state
                            .editor
                            .puede_deshacer()
                            .then_some(MensajeEditor::Deshacer)
                    )
                    .width(44),
                button("↷")
                    .style(tema::boton_secundario)
                    .on_press_maybe(
                        state
                            .editor
                            .puede_rehacer()
                            .then_some(MensajeEditor::Rehacer)
                    )
                    .width(44),
                button("Borrar")
                    .style(tema::boton_secundario)
                    .on_press_maybe(
                        (!state.editor.seleccion.is_empty())
                            .then_some(MensajeEditor::BorrarSeleccion)
                    ),
            ]
            .spacing(12),
            row![
                text_input("Texto del subtítulo", &state.borrador)
                    .id(ID_TEXTO_SUBTITULO)
                    .style(tema::entrada)
                    .on_input(MensajeEditor::Borrador)
                    .on_submit(MensajeEditor::AplicarTexto),
                button("Aplicar")
                    .style(tema::boton_secundario)
                    .on_press_maybe(
                        (state.editor.seleccion.len() == 1).then_some(MensajeEditor::AplicarTexto)
                    ),
            ]
            .spacing(8),
            text("CAPTIONS").size(14),
            scrollable(column(rows).spacing(8)),
        ]
        .spacing(14)
        .padding(14),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(panel)
    .into()
}

fn speakers_panel(state: &EditorApp) -> Element<'_, MensajeEditor> {
    let hablantes = state.editor.proyecto().hablantes.as_slice();
    // Una sola lista de opciones para todas las filas: el `pick_list` la pide
    // por valor, así que se clona por hablante (son unos pocos presets).
    let mut opciones = vec![OpcionPreset {
        id: None,
        nombre: "Sin preset".into(),
    }];
    opciones.extend(state.presets.iter().map(|preset| OpcionPreset {
        id: Some(preset.id.clone()),
        nombre: preset.nombre.clone(),
    }));

    let rows = hablantes
        .iter()
        .map(|hablante| {
            let id_nombre = hablante.id.clone();
            let id_tecla = hablante.id.clone();
            let id_asignar = hablante.id.clone();
            let id_preset = hablante.id.clone();
            let hay_seleccion = !state.editor.seleccion.is_empty();
            let seleccionado = opciones
                .iter()
                .find(|opcion| opcion.id == hablante.preset_id)
                .cloned();
            row![
                text_input("Nombre", &hablante.nombre).on_input(move |texto| {
                    MensajeEditor::EditarHablante(id_nombre.clone(), CampoHablante::Nombre, texto)
                }),
                text_input("1", &hablante.tecla)
                    .style(tema::entrada)
                    .on_input(move |texto| MensajeEditor::EditarHablante(
                        id_tecla.clone(),
                        CampoHablante::Tecla,
                        texto
                    ))
                    .width(44),
                pick_list(
                    opciones.clone(),
                    seleccionado,
                    move |opcion: OpcionPreset| {
                        MensajeEditor::AsignarPreset(id_preset.clone(), opcion.id)
                    }
                )
                .width(140),
                button("Asignar")
                    .style(tema::boton_secundario)
                    .on_press_maybe(
                        hay_seleccion
                            .then(|| MensajeEditor::AsignarHablante(Some(id_asignar.clone())))
                    ),
                button("×")
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::BorrarHablante(hablante.id.clone()))
                    .width(36),
            ]
            .spacing(6)
            .into()
        })
        .collect::<Vec<Element<'_, MensajeEditor>>>();

    container(
        column![
            row![
                text("Speakers").size(20),
                space::horizontal(),
                button("Sin hablante")
                    .style(tema::boton_secundario)
                    .on_press_maybe(
                        (!state.editor.seleccion.is_empty())
                            .then_some(MensajeEditor::AsignarHablante(None))
                    ),
                button("＋").style(tema::boton_secundario).on_press_maybe(
                    (hablantes.len() < MAX_HABLANTES).then_some(MensajeEditor::AgregarHablante)
                ),
            ]
            .spacing(12),
            // El oráculo pinta aquí el dot con el color del preset `.ass`; aquí
            // el selector de estilo hace visible qué preset usa cada hablante.
            text("SPEAKERS").size(14),
            scrollable(column(rows).spacing(6)),
        ]
        .spacing(10)
        .padding(14),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(panel)
    .into()
}

const ORDEN_ALINEACION: [i32; 9] = [7, 8, 9, 4, 5, 6, 1, 2, 3];
const GLIFOS_ALINEACION: [&str; 9] = ["↖", "↑", "↗", "←", "•", "→", "↙", "↓", "↘"];

/// Un campo de texto del preset. El borrador (si hay uno a medias) tapa al
/// valor guardado; el `update` lo descarta en cuanto `aplicar_campo` lo acepta.
fn campo_texto<'a>(
    preset: &'a PresetAss,
    campo: CampoPreset,
    etiqueta: &'static str,
    borradores: &'a HashMap<(String, CampoPreset), String>,
) -> Element<'a, MensajeEditor> {
    let valor = valor_campo(preset, campo, borradores);
    let id = preset.id.clone();
    row![
        text(etiqueta).width(96),
        text_input(etiqueta, &valor)
            .style(tema::entrada)
            .on_input(move |texto| MensajeEditor::EditarPreset(id.clone(), campo, texto))
            .width(190),
    ]
    .spacing(8)
    .into()
}

/// Panel de estilos: la lista de presets del oráculo, sus campos y la
/// alineación en cuadrícula (7-9 arriba, 1-3 abajo, como libass piensa).
/// Cada cambio escribe `presets_ass.json` y recarga el preview: no hay botón
/// de guardar porque el oráculo tampoco lo tiene.
fn styles_panel(state: &EditorApp) -> Element<'_, MensajeEditor> {
    let seleccionado = state
        .presets
        .iter()
        .find(|preset| Some(&preset.id) == state.preset_seleccionado.as_ref())
        .or_else(|| state.presets.first());

    let Some(preset) = seleccionado else {
        return container(text("Sin presets"))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(panel)
            .into();
    };

    let lista = state
        .presets
        .iter()
        .map(|otro| {
            let activo = otro.id == preset.id;
            button(row![
                text(if activo { "●" } else { " " }).size(12),
                text(otro.nombre.clone()),
            ])
            .on_press(MensajeEditor::SeleccionarPreset(otro.id.clone()))
            .width(Length::Fill)
            .style(if activo { boton_activo } else { boton })
            .into()
        })
        .collect::<Vec<Element<'_, MensajeEditor>>>();

    let alineacion = ORDEN_ALINEACION
        .iter()
        .zip(GLIFOS_ALINEACION.iter())
        .collect::<Vec<_>>()
        .chunks(3)
        .map(|fila| {
            row(fila.iter().map(|(valor, glifo)| {
                let glifo = **glifo;
                let valor = **valor;
                button(text(glifo))
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::EditarPreset(
                        preset.id.clone(),
                        CampoPreset::Alignment,
                        valor.to_string(),
                    ))
                    .width(38)
                    .style(if preset.alignment == valor {
                        boton_activo
                    } else {
                        boton
                    })
                    .into()
            }))
            .spacing(4)
            .into()
        })
        .collect::<Vec<Element<'_, MensajeEditor>>>();

    let borradores = &state.borradores_preset;
    container(scrollable(
        column![
            row![
                text("Styles").size(20),
                // Sin `space::horizontal()`: dentro de un `scrollable` el
                // espacio elástico crece sin límite y empuja los botones
                // fuera del viewport.
                button("＋")
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::NuevoPreset)
                    .width(36),
                button("Borrar")
                    .style(tema::boton_secundario)
                    .on_press_maybe(
                        (state.presets.len() > 1).then_some(MensajeEditor::BorrarPreset)
                    ),
            ]
            .spacing(8),
            text("PRESETS").size(14),
            column(lista).spacing(4),
            text("ESTILO").size(14),
            campo_texto(preset, CampoPreset::Nombre, "Nombre", borradores),
            campo_texto(preset, CampoPreset::Fontname, "Fuente", borradores),
            campo_texto(preset, CampoPreset::Fontsize, "Tamaño", borradores),
            campo_texto(preset, CampoPreset::Outline, "Contorno", borradores),
            campo_texto(preset, CampoPreset::Shadow, "Sombra", borradores),
            campo_texto(preset, CampoPreset::MarginL, "Margen L", borradores),
            campo_texto(preset, CampoPreset::MarginR, "Margen R", borradores),
            campo_texto(preset, CampoPreset::MarginV, "Margen V", borradores),
            campo_texto(preset, CampoPreset::Color, "Color", borradores),
            campo_texto(
                preset,
                CampoPreset::OutlineColor,
                "Color contorno",
                borradores
            ),
            row![text("Alineación").width(96), column(alineacion).spacing(4),].spacing(8),
            row![
                text("Peso").width(96),
                button("B")
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::EditarPreset(
                        preset.id.clone(),
                        CampoPreset::Bold,
                        (!preset.bold).to_string(),
                    ))
                    .width(38)
                    .style(if preset.bold { boton_activo } else { boton }),
                button("I")
                    .style(tema::boton_secundario)
                    .on_press(MensajeEditor::EditarPreset(
                        preset.id.clone(),
                        CampoPreset::Italic,
                        (!preset.italic).to_string(),
                    ))
                    .width(38)
                    .style(if preset.italic { boton_activo } else { boton }),
            ]
            .spacing(8),
            text("Cada cambio se guarda y se aplica al preview.").size(11),
        ]
        .spacing(8)
        .padding(14),
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .style(panel)
    .into()
}

fn status_bar(state: &EditorApp) -> Element<'_, MensajeEditor> {
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

fn black_panel(theme: &iced::Theme) -> container::Style {
    let mut estilo = tema::fondo(theme);
    estilo.background = Some(Background::Color(tema::SUPERFICIE));
    estilo
}

fn panel(theme: &iced::Theme) -> container::Style {
    tema::panel(theme)
}

/// Aviso de recuperación: mostaza sobre fondo oscuro. No es un error (no se
/// perdió nada todavía), pero sí algo que reclama una decisión.
fn banner(theme: &iced::Theme) -> container::Style {
    tema::aviso(theme)
}

/// Toggle encendido: el mostaza marca el estado activo, igual que la acción
/// principal del arranque.
fn boton_activo(theme: &iced::Theme, status: button::Status) -> button::Style {
    tema::boton_primario(theme, status)
}

/// Toggle apagado, con la forma del encendido para que el toggle no baile.
fn boton(theme: &iced::Theme, status: button::Status) -> button::Style {
    tema::boton_secundario(theme, status)
}

#[cfg(test)]
mod tests {
    use image::GenericImageView;

    use super::{VELOCIDADES, Velocidad, resumen_srt};

    #[test]
    fn pegar_inserta_fragmento_y_no_reemplaza_el_proyecto() {
        let ruta = format!(
            "{}/../../testdata/clip-15s-2audio.mp4",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut editor = super::EditorApp::desde_video(&ruta).expect("abre el video de prueba");
        let antes = editor.editor.proyecto().captions.len();
        editor.portapapeles = "1\n00:00:01,000 --> 00:00:02,000\nHola\n".into();
        let _ = super::pegar(&mut editor);
        assert_eq!(editor.editor.proyecto().captions.len(), antes + 1);
        assert_eq!(editor.limpio.captions.len(), antes);
    }

    #[test]
    fn pegar_con_portapapeles_vacio_no_hace_nada() {
        let ruta = format!(
            "{}/../../testdata/clip-15s-2audio.mp4",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut editor = super::EditorApp::desde_video(&ruta).expect("abre el video de prueba");
        editor.portapapeles.clear();
        let _ = super::pegar(&mut editor);
        assert!(editor.editor.proyecto().captions.is_empty());
    }

    #[test]
    fn abrir_reciente_sin_miniatura_regenera_en_el_playhead() {
        let dir = std::env::temp_dir().join(format!("rod-miniatura-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cdp = dir.join("mi.cdp");
        let proyecto = Proyecto {
            ruta_video: format!(
                "{}/../../testdata/clip-15s-2audio.mp4",
                env!("CARGO_MANIFEST_DIR")
            ),
            hablantes: Vec::new(),
            captions: Vec::new(),
            playhead: 3.25,
        };
        let mut editor =
            super::EditorApp::desde_proyecto_en(proyecto, Some(cdp.clone()), 3.25, &dir)
                .expect("abre el video de prueba");
        assert!(editor.miniatura_pendiente);
        assert!((editor.position - 3.25).abs() < 1e-6);
        editor.pixels.fill(9);
        editor.regenerar_miniatura_en(&dir);
        let png = crate::thumbnail::ruta_cache_en(&dir, &cdp);
        assert!(png.is_file(), "no se regeneró la miniatura");
        assert_eq!(image::open(&png).unwrap().dimensions(), (160, 90));
        assert!(!editor.miniatura_pendiente);
    }

    #[test]
    fn desde_proyecto_hace_seek_al_playhead_guardado() {
        let proyecto = Proyecto {
            ruta_video: format!(
                "{}/../../testdata/clip-15s-2audio.mp4",
                env!("CARGO_MANIFEST_DIR")
            ),
            hablantes: Vec::new(),
            captions: Vec::new(),
            playhead: 3.25,
        };
        let editor = super::EditorApp::desde_proyecto(proyecto, None, 3.25)
            .expect("abre el video de prueba");
        assert!(
            (editor.player.position() - 3.25).abs() < 0.5,
            "el player quedó en {}",
            editor.player.position()
        );
    }

    use crate::project_io::ExportacionSrt;
    use crate::recovery_io;
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
        assert!(recovery_io::pendiente(&ruta, 1000.0).is_none());
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

        let aviso = recovery_io::pendiente(&ruta, 1000.0).expect("hay aviso");
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
        assert!(recovery_io::pendiente(&ruta, 1000.0).is_none());
        std::fs::remove_file(&ruta).expect("limpia");
    }
}
