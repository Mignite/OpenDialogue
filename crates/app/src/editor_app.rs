use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::comandos::Comando;
use crate::editor::{EstadoEditor, normalizar_tecla};
use crate::presets_edicion::{CampoPreset, aplicar_campo};
use crate::presets_io;
use crate::preview::{CANVAS, ass_del_proyecto};
use crate::project_io::{
    Documento, ExportacionSrt, FormatoSubtitulo, abrir_proyecto, exportar_json_combinado,
    exportar_srt_por_hablante, guardar_proyecto,
};
use crate::recovery_io::{self, AvisoRecovery};
use crate::tarea::Tarea;
use crate::timeline::{self, Vista};
use crate::video::Capa;
use crate::{recientes, thumbnail};
use rod_core::{PresetAss, Proyecto, nombre_libre, nuevo_preset_desde};
use rod_player::AudioTrack;

const WIDTH: usize = CANVAS.0 as usize;
const HEIGHT: usize = CANVAS.1 as usize;
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

pub const VELOCIDADES: [Velocidad; 6] = [
    Velocidad(0.5),
    Velocidad(0.75),
    Velocidad(1.0),
    Velocidad(1.25),
    Velocidad(1.5),
    Velocidad(2.0),
];

#[derive(Debug, Clone, PartialEq)]
pub enum CampoHablante {
    Nombre,
    Tecla,
    /// Color de identidad de UI, siempre una de las doce muestras.
    Color,
}

/// Campo de tiempo editable en el inspector de subtítulos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CampoTiempo {
    Entrada,
    Salida,
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
    /// Confirmación de la edición inline de una fila (id, texto nuevo).
    EditarInline(String, String),
    CancelarInline,
    /// Aplicación en vivo del inspector (un paso de deshacer por sesión).
    EditarEnVivo(String, String),
    TerminarEdicionTexto,
    UnirConSiguiente(String),
    SeleccionarTodosHablante(String),
    /// Entrada/salida del caption seleccionado. Se aplica al confirmar; un
    /// valor inválido o invertido se descarta sin tocar el proyecto.
    EditarTiempo(String, CampoTiempo, String),
    BorradorInicio(String),
    BorradorFin(String),
    AgregarHablante,
    DuplicarHablante(String),
    AlternarMute(String),
    AlternarSolo(String),
    /// Edición en vivo de un hablante: el campo decide qué se toca.
    EditarHablante(String, CampoHablante, String),
    BorrarHablante(String),
    /// `None` deja los captions seleccionados sin hablante.
    AsignarHablante(Option<String>),
    /// Cambia el preset `.ass` de un hablante (`None` = sin preset).
    AsignarPreset(String, Option<String>),
    /// Aplica un preset a los hablantes de los captions seleccionados.
    AplicarPresetSeleccion(String),
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
    /// Scrubbing optimista: la UI manda y el decoder sigue al último objetivo.
    ScrubInicio(f64),
    ScrubMover(f64),
    ScrubFin(f64),
    /// Modificadores del teclado, para que la rueda sepa si hay Shift.
    Modificadores(egui::Modifiers),
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
    /// Respuestas de los diálogos nativos pedidos desde el editor.
    DialogoVideoEditor(Option<PathBuf>),
    DialogoProyecto(Option<PathBuf>),
    DialogoSubtitulos(Option<PathBuf>, bool),
    DialogoGuardarProyecto(Option<PathBuf>),
    DialogoCarpetaExportar(Option<PathBuf>),
    DialogoJson(Option<PathBuf>),
    DialogoAssExportar(Option<PathBuf>),
    DialogoAutosubsSrt(Option<PathBuf>),
    DialogoAutosubsTxt(Option<PathBuf>),
}

/// Estado de la única barra de edición de texto (bajo el video).
#[derive(Debug)]
pub struct EstadoTexto {
    /// Línea que se edita: selección, o la del playhead si no hay selección.
    pub objetivo: Option<String>,
    pub borrador: String,
    pub original: String,
    pub foco: bool,
    pub ultimo_tipeo: Option<std::time::Instant>,
    pub nueva_linea: bool,
    /// Líneas visibles del campo (1–3).
    pub lineas: u8,
}

impl Default for EstadoTexto {
    fn default() -> Self {
        Self {
            lineas: 1,
            ..Self::vacio()
        }
    }
}

impl EstadoTexto {
    fn vacio() -> Self {
        Self {
            objetivo: None,
            borrador: String::new(),
            original: String::new(),
            foco: false,
            ultimo_tipeo: None,
            nueva_linea: false,
            lineas: 1,
        }
    }

    /// Línea seleccionada o, si no hay, la que contiene el playhead.
    pub fn objetivo(state: &EditorApp) -> Option<String> {
        let captions = &state.editor.proyecto().captions;
        if let Some(id) = state.editor.seleccion.first()
            && captions.iter().any(|caption| &caption.id == id)
        {
            return Some(id.clone());
        }
        captions
            .iter()
            .find(|caption| state.position >= caption.inicio && state.position <= caption.fin)
            .map(|caption| caption.id.clone())
    }

    /// Cierra la sesión: aplica el texto, crea la línea si no había objetivo y
    /// devuelve el foco a la timeline.
    pub fn cerrar_sesion(state: &mut EditorApp) {
        let borrador = state.texto.borrador.clone();
        let original = state.texto.original.clone();
        let objetivo = Self::objetivo(state);
        state.texto.foco = false;
        state.texto.ultimo_tipeo = None;
        if let Some(id) = objetivo {
            if borrador != original {
                update(state, MensajeEditor::EditarEnVivo(id, borrador));
            }
            update(state, MensajeEditor::TerminarEdicionTexto);
        } else if !borrador.is_empty() {
            let caption = rod_core::Caption {
                id: id_nuevo(),
                inicio: state.position,
                fin: (state.position + 2.0).min(state.duration),
                texto: borrador,
                hablante_id: None,
            };
            state.editor.agregar(caption);
            state.editor.registrar_historial();
            refrescar_preview(state);
        }
        state.texto.borrador.clear();
        state.texto.original.clear();
        state.texto.nueva_linea = false;
        state.texto.lineas = 1;
    }

    /// Esc: vuelve al texto que había al enfocar y suelta el foco.
    pub fn revertir(state: &mut EditorApp) {
        let original = state.texto.original.clone();
        if let Some(id) = Self::objetivo(state) {
            update(state, MensajeEditor::EditarEnVivo(id, original.clone()));
            update(state, MensajeEditor::TerminarEdicionTexto);
        }
        state.texto.borrador = original;
        state.texto.foco = false;
        state.texto.ultimo_tipeo = None;
        state.texto.nueva_linea = false;
        state.texto.lineas = 1;
    }
}

pub struct EditorApp {
    pub(crate) editor: EstadoEditor,
    pub texto: EstadoTexto,
    pub(crate) player: rod_player::Player,
    pub(crate) subtitles: rod_subs::Subtitulos,
    pub(crate) pixels: Vec<u8>,
    pub(crate) layers: Arc<Vec<Capa>>,
    pub(crate) duration: f64,
    pub(crate) position: f64,
    /// El usuario tiene el slider tomado: `time-pos` no debe pisar el thumb.
    pub(crate) arrastrando: bool,
    pub(crate) paused: bool,
    pub(crate) video_path: String,
    pub(crate) audio_tracks: Vec<AudioTrack>,
    pub(crate) selected_audio: Option<AudioTrack>,
    pub(crate) velocidad: Velocidad,
    /// Lo que hay en el `text_input` del caption seleccionado.
    pub(crate) borrador: String,
    /// Fila en edición inline: (id del caption, borrador).
    pub inline: Option<(String, String)>,
    /// Tiempo optimista mientras se arrastra el playhead.
    pub scrub_ui_time: Option<f64>,
    /// Texto buscado en la pestaña de subtítulos.
    pub busqueda_subtitulos: String,
    /// Sesión de edición en vivo del inspector (para un solo paso de deshacer).
    pub editando_texto: Option<String>,
    /// Hablantes silenciados / en solo, estado de UI.
    pub mute_hablantes: std::collections::HashSet<String>,
    pub solo_hablantes: std::collections::HashSet<String>,
    /// Picos de la waveform, cuando el análisis en hilo terminó.
    pub picos: Option<rod_player::waveform::Picos>,
    pub rx_picos: Option<
        std::sync::mpsc::Receiver<
            Result<rod_player::waveform::Picos, rod_player::waveform::ErrorWaveform>,
        >,
    >,
    /// Borradores de entrada y salida del caption seleccionado.
    pub(crate) borrador_inicio: String,
    pub(crate) borrador_fin: String,
    /// Pestaña visible de la columna derecha.
    pub(crate) panel: Panel,
    /// Campo de hablante que está recibiendo teclas. iced no da un evento de
    /// foco por widget, así que la sesión se cierra al tocar cualquier otra
    /// cosa: es el equivalente al `onFocus={onCommit}` del oráculo (una sesión
    /// de tecleo = un paso de historial).
    pub(crate) editando_hablante: Option<(String, CampoHablante)>,
    /// Arrastre en curso en el timeline. Vive en la app y no en el canvas
    /// porque el historial y el proyecto también viven aquí.
    pub(crate) arrastre: Option<timeline::Arrastre>,
    /// Ventana visible del timeline (inicio y escala).
    pub(crate) vista: Vista,
    /// Ancho del canvas del timeline, que lo publica el propio canvas. Sin él
    /// no se puede saber cuántos segundos caben (pan, seguimiento, zoom).
    pub(crate) ancho_timeline: f32,
    /// La vista persigue al playhead mientras se reproduce. Se apaga al usar
    /// la rueda o la barra, y se enciende con el botón.
    pub(crate) seguir: bool,
    /// Inicio hacia el que se desliza la vista, si hay un seguimiento en curso.
    pub(crate) objetivo_vista: Option<f64>,
    /// Modificadores del teclado. La rueda no los lleva en su evento, así que
    /// el «Shift para acercar» se decide con este espejo.
    pub(crate) modificadores: egui::Modifiers,
    /// Dónde se guardó el proyecto. `None` = nunca se guardó: «Guardar» pide
    /// ruta y «Guardar como» la vuelve a pedir.
    pub(crate) ruta_proyecto: Option<PathBuf>,
    /// El proyecto tal como está en disco (o como arrancó la app). Es la línea
    /// base del «hay cambios sin guardar»: el playhead no cuenta, porque se
    /// mueve solo al reproducir y marcaría el proyecto sucio 60 veces por
    /// segundo.
    pub(crate) limpio: Proyecto,
    /// Copia automática pendiente de aceptar. Se lee una sola vez, al
    /// arrancar; el texto se formatea ahí mismo para no leer el reloj en cada
    /// frame.
    pub(crate) recovery: Option<AvisoRecovery>,
    /// Epoch ms de la última escritura del recovery (`None` = ninguna todavía)
    /// y del último chequeo del autosave. El chequeo va a 1 Hz: comparar el
    /// proyecto entero en cada frame para nada es tirar CPU.
    pub(crate) ultima_escritura: Option<f64>,
    pub(crate) ultimo_chequeo: f64,
    /// Presets `.ass` cargados al arrancar. Los usa el preview y el export por
    /// igual: son la única fuente de estilos de un caption con hablante.
    pub(crate) presets: Vec<PresetAss>,
    /// Ruta del archivo compartido con el oráculo, resuelta una vez al arrancar.
    pub(crate) ruta_presets: PathBuf,
    /// Preset abierto en el panel de estilos. Si el archivo traía presets, el
    /// primero; `None` solo si el panel no tiene nada que mostrar (no pasa:
    /// siempre hay al menos la semilla).
    pub(crate) preset_seleccionado: Option<String>,
    /// Texto a medias de los campos numéricos, por id de preset y campo. Sin
    /// esto, escribir `72.5` se quedaría en `72` al llegar al punto.
    pub(crate) borradores_preset: HashMap<(String, CampoPreset), String>,
    /// Resultado de la última acción que no tiene otro lugar donde contarse
    /// (exportar, por ahora). iced no tiene toasts; la barra de estado es el
    /// sitio donde el usuario ya mira el estado del proyecto.
    pub(crate) mensaje: Option<String>,
    /// La miniatura del reciente faltaba al abrir: se genera con el primer
    /// frame que rasterice mpv, no con el buffer vacío del constructor.
    pub(crate) miniatura_pendiente: bool,
    /// Portapapeles interno para Ctrl+C/V entre captions.
    pub(crate) portapapeles: String,
    /// Contenido del SRT leído mientras se pide el TXT gemelo del autosubs.
    pub(crate) pendiente_srt: Option<String>,
    /// Borradores de nombre de hablante: la vista no muta el proyecto antes del
    /// historial, así que renombrar sigue siendo deshacible en un paso.
    pub(crate) borradores_hablante: HashMap<String, String>,
}

impl EditorApp {
    /// Acceso a la selección compartida por lista y timeline.
    pub fn editor_mut(&mut self) -> &mut EstadoEditor {
        &mut self.editor
    }

    /// Hay una fila en edición inline.
    pub fn texto_inline_activo(&self) -> bool {
        self.inline.is_some()
    }

    /// Tiempo que debe mostrar la UI (optimista mientras se arrastra).
    pub fn tiempo_ui(&self) -> f64 {
        self.scrub_ui_time.unwrap_or(self.position)
    }

    /// Está arrastrando el playhead: hay que repintar cada frame.
    pub fn scrubbing(&self) -> bool {
        self.scrub_ui_time.is_some()
    }

    /// Posición del playhead, para el transporte y las pruebas.
    pub fn set_position(&mut self, posicion: f64) {
        self.position = posicion;
    }

    /// Proyecto en memoria, solo lectura.
    pub fn proyecto(&self) -> &rod_core::Proyecto {
        self.editor.proyecto()
    }

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

        let ruta_audio = std::path::PathBuf::from(&video_path);
        let mut estado = Self {
            editor,
            texto: EstadoTexto::default(),
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
            inline: None,
            borrador_inicio: String::new(),
            borrador_fin: String::new(),
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
            modificadores: egui::Modifiers::default(),
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
            pendiente_srt: None,
            borradores_hablante: HashMap::new(),
            scrub_ui_time: None,
            busqueda_subtitulos: String::new(),
            editando_texto: None,
            mute_hablantes: std::collections::HashSet::new(),
            solo_hablantes: std::collections::HashSet::new(),
            picos: None,
            rx_picos: Some(rod_player::waveform::analizar_en_hilo(ruta_audio)),
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
    // La waveform del video anterior no sirve para el nuevo.
    state.picos = None;
    state.rx_picos = Some(rod_player::waveform::analizar_en_hilo(
        std::path::PathBuf::from(ruta_video),
    ));
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
pub fn sucio(state: &EditorApp) -> bool {
    let proyecto = state.editor.proyecto();
    proyecto.captions != state.limpio.captions || proyecto.hablantes != state.limpio.hablantes
}

/// Estado de reproducción, para la política de repintado.
pub fn pausado(state: &EditorApp) -> bool {
    state.paused
}

/// mpv todavía está sirviendo un seek.
pub fn buscando(state: &EditorApp) -> bool {
    state.player.seeking()
}

/// Nombre visible en la cabecera: el del video abierto.
pub fn nombre_proyecto(state: &EditorApp) -> &str {
    std::path::Path::new(&state.video_path)
        .file_name()
        .and_then(|nombre| nombre.to_str())
        .unwrap_or(&state.video_path)
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
/// Redondea al cuadro (25 fps por defecto) y acota a la duración.
fn snap_a_frame(tiempo: f64, duracion: f64) -> f64 {
    let cuadro = 1.0 / 25.0;
    ((tiempo / cuadro).round() * cuadro).clamp(0.0, duracion.max(0.0))
}

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
fn guardar_presets(state: &mut EditorApp) -> Tarea {
    state.mensaje = match presets_io::escribir(&state.ruta_presets, &state.presets) {
        Ok(()) => None,
        Err(error) => Some(format!("No se pudieron guardar los estilos: {error}")),
    };
    refrescar_preview(state);
    Tarea::Ninguna
}

/// El `text_input` muestra el texto del caption seleccionado. Sin selección
/// (o con varios) queda vacío: no hay un texto único que editar.
/// Fila compacta de la lista de captions: índice, tiempos y texto.
pub fn etiqueta_caption(indice: usize, inicio: f64, fin: f64, texto: &str) -> String {
    format!(
        "{indice:02}  {} → {}  {texto}",
        rod_core::format_time(inicio),
        rod_core::format_time(fin)
    )
}

/// Duración en segundos con dos decimales, como la muestra el inspector.
pub fn duracion_texto(inicio: f64, fin: f64) -> String {
    format!("{:.2} s", (fin - inicio).max(0.0))
}

fn sincronizar_borrador(state: &mut EditorApp) {
    let id = match state.editor.seleccion.as_slice() {
        [id] => Some(id.clone()),
        _ => None,
    };
    state.borrador = id
        .as_deref()
        .and_then(|id| state.editor.texto_de(id).map(str::to_owned))
        .unwrap_or_default();
    let tiempos = id.and_then(|id| {
        state
            .editor
            .proyecto()
            .captions
            .iter()
            .find(|caption| caption.id == id)
            .map(|caption| (caption.inicio, caption.fin))
    });
    state.borrador_inicio = tiempos
        .map(|(inicio, _)| rod_core::format_time(inicio))
        .unwrap_or_default();
    state.borrador_fin = tiempos
        .map(|(_, fin)| rod_core::format_time(fin))
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

/// Compone la lógica pura de `rod_core::autosubs`: turnos del TXT → números
/// asignados a cada cue → hablantes con ids → captions con `hablante_id`.
/// El orden de los hablantes es el de `hablantes_desde_nombres` (numérico).
fn aplicar_autosubs(
    captions: &[rod_core::Caption],
    turnos: &[rod_core::TurnoAutosubs],
) -> (Vec<rod_core::Hablante>, Vec<rod_core::Caption>) {
    use rod_core::{asignar_hablantes_por_texto, hablantes_desde_nombres};

    let numeros = asignar_hablantes_por_texto(captions, turnos);
    let distintos: Vec<String> = numeros
        .iter()
        .flatten()
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let hablantes = hablantes_desde_nombres(&distintos);
    let id_por_numero: std::collections::HashMap<String, String> = hablantes
        .iter()
        .map(|hablante| {
            (
                hablante
                    .nombre
                    .strip_prefix("Hablante ")
                    .unwrap_or_default()
                    .to_owned(),
                hablante.id.clone(),
            )
        })
        .collect();
    let captions_asignados = captions
        .iter()
        .zip(numeros)
        .map(|(caption, numero)| rod_core::Caption {
            hablante_id: numero.and_then(|numero| id_por_numero.get(&numero).cloned()),
            ..caption.clone()
        })
        .collect();
    (hablantes, captions_asignados)
}

/// Un único manejador para el menú y el teclado: los dos entregan un `Comando`.
fn ejecutar_comando(state: &mut EditorApp, comando: Comando) -> Tarea {
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
                    Tarea::Ninguna
                }
            }
        }
        C::NudgeIzquierda => nudge_seleccion(state, -0.1),
        C::NudgeDerecha => nudge_seleccion(state, 0.1),
        C::AgregarFragmento => agregar_fragmento(state),
        C::Dividir => dividir_en_playhead(state),
        // Estos comandos se conectan a sus mensajes en la fase de timeline.
        C::Unir | C::Entrada | C::Salida | C::AlternarSnap => Tarea::Ninguna,
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
                Tarea::Ninguna
            } else {
                Tarea::EnfocarTexto
            }
        }
        C::Buscar => {
            state.mensaje = Some("Buscar: usa el panel de subtítulos para localizar texto".into());
            Tarea::Ninguna
        }
        C::Ayuda => {
            state.mensaje = Some(
                "Atajos: Space, ←/→, Alt+←/→, Shift+←/→, Ctrl+Shift+←/→, E, A, C, Delete, Ctrl+Z/Y/C/V, ?, +/−".into(),
            );
            Tarea::Ninguna
        }
        C::ImportarAutosubs => importar_autosubs(state),
        C::CargarSrt | C::CargarAss => cargar_subtitulos(state, comando == C::CargarAss),
        C::NuevoProyecto | C::AbrirProyecto | C::AbrirVideo => {
            state.mensaje = Some("Usa el menú de inicio para abrir un proyecto".into());
            Tarea::Ninguna
        }
        // El enrutador abre la capa de Ajustes antes de llegar aquí.
        C::Ajustes => Tarea::Ninguna,
    }
}

/// Par SRT+TXT del oráculo: se pide el SRT, se busca el TXT gemelo en la misma
/// carpeta y, si no está, se pide en un segundo diálogo.
fn importar_autosubs(_state: &mut EditorApp) -> Tarea {
    // El primer diálogo (SRT) se pide al front-end; el resultado llega como
    // `MensajeEditor::DialogoAutosubsSrt`.
    Tarea::ImportarSrt
}

fn importar_autosubs_srt(state: &mut EditorApp, ruta_srt: Option<PathBuf>) -> Tarea {
    use rod_core::parse_srt;

    let Some(ruta_srt) = ruta_srt else {
        return Tarea::Ninguna;
    };
    let Ok(texto_srt) = std::fs::read_to_string(&ruta_srt) else {
        state.mensaje = Some("No se pudo leer el SRT".into());
        return Tarea::Ninguna;
    };
    let captions = parse_srt(&texto_srt);
    if captions.is_empty() {
        state.mensaje = Some("El SRT no tiene subtítulos".into());
        return Tarea::Ninguna;
    }

    let gemelo = ruta_srt.with_extension("txt");
    if gemelo.is_file() {
        return importar_autosubs_txt(state, Some(gemelo));
    }
    state.pendiente_srt = Some(texto_srt);
    Tarea::ImportarTxt
}

fn importar_autosubs_txt(state: &mut EditorApp, ruta_txt: Option<PathBuf>) -> Tarea {
    use rod_core::{parse_autosubs_txt, parse_srt};

    let Some(ruta_txt) = ruta_txt else {
        state.pendiente_srt = None;
        return Tarea::Ninguna;
    };
    let Ok(texto_txt) = std::fs::read_to_string(&ruta_txt) else {
        state.mensaje = Some("No se pudo leer el TXT de turnos".into());
        return Tarea::Ninguna;
    };
    let Some(texto_srt) = state.pendiente_srt.take() else {
        return Tarea::Ninguna;
    };
    let captions = parse_srt(&texto_srt);
    let turnos = parse_autosubs_txt(&texto_txt);
    let (hablantes, captions) = aplicar_autosubs(&captions, &turnos);
    let asignados = captions
        .iter()
        .filter(|caption| caption.hablante_id.is_some())
        .count();

    state.editor.registrar_historial();
    let mut proyecto = state.editor.proyecto().clone();
    proyecto.hablantes = hablantes;
    proyecto.captions = captions;
    state.editor = EstadoEditor::nuevo(Documento {
        proyecto,
        formato: crate::project_io::FormatoSubtitulo::Srt,
        estilos_ass: Vec::new(),
    });
    state.limpio = state.editor.proyecto().clone();
    refrescar_preview(state);
    state.mensaje = Some(format!(
        "Auto-subs importados: {} captions, {} con hablante",
        state.editor.proyecto().captions.len(),
        asignados
    ));
    Tarea::Ninguna
}

fn nudge_seleccion(state: &mut EditorApp, delta: f64) -> Tarea {
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
    Tarea::Ninguna
}

fn agregar_fragmento(state: &mut EditorApp) -> Tarea {
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
    Tarea::Ninguna
}

fn dividir_en_playhead(state: &mut EditorApp) -> Tarea {
    let Some(id) = state.editor.seleccion.first().cloned() else {
        return Tarea::Ninguna;
    };
    let Some(caption) = state
        .editor
        .proyecto()
        .captions
        .iter()
        .find(|caption| caption.id == id)
        .cloned()
    else {
        return Tarea::Ninguna;
    };
    let corte = state
        .position
        .clamp(caption.inicio + 0.05, caption.fin - 0.05);
    if corte <= caption.inicio || corte >= caption.fin {
        return Tarea::Ninguna;
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
    Tarea::Ninguna
}

fn ciclar_caption(state: &mut EditorApp, paso: i32) -> Tarea {
    let captions = &state.editor.proyecto().captions;
    if captions.is_empty() {
        return Tarea::Ninguna;
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
    Tarea::Ninguna
}

fn ciclar_simultaneo(state: &mut EditorApp, paso: i32) -> Tarea {
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
    Tarea::Ninguna
}

fn copiar(state: &mut EditorApp) -> Tarea {
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
    Tarea::Ninguna
}

fn pegar(state: &mut EditorApp) -> Tarea {
    if state.portapapeles.trim().is_empty() {
        return Tarea::Ninguna;
    }
    // Insertar fragmentos, nunca reconstruir el editor: `EstadoEditor::nuevo`
    // reinicia el historial y `limpio`, y un Ctrl+V con el portapapeles vacío
    // borraría el proyecto sin posibilidad de deshacer.
    let Ok(documento) = crate::project_io::cargar(&state.portapapeles, &state.video_path) else {
        return Tarea::Ninguna;
    };
    for caption in documento.proyecto.captions {
        state.editor.agregar(caption);
    }
    refrescar_preview(state);
    Tarea::Ninguna
}

fn cargar_subtitulos(_state: &mut EditorApp, ass: bool) -> Tarea {
    if ass {
        Tarea::CargarAss
    } else {
        Tarea::CargarSrt
    }
}

fn aplicar_subtitulos_cargados(state: &mut EditorApp, ruta: Option<PathBuf>) -> Tarea {
    let Some(ruta) = ruta else {
        return Tarea::Ninguna;
    };
    let Ok(texto) = std::fs::read_to_string(ruta) else {
        state.mensaje = Some("No se pudo leer el archivo de subtítulos".into());
        return Tarea::Ninguna;
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
    Tarea::Ninguna
}

fn guardar_en(state: &mut EditorApp, ruta: PathBuf) {
    // El playhead no es parte de la edición (vive fuera del editor y del
    // historial), pero sí del proyecto: se escribe al guardar y se restaura al
    // abrir.
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
            state.limpio = proyecto;
            state.recovery = None;
            borrar_recovery(state);
        }
        Err(error) => eprintln!("no se pudo guardar el proyecto: {error}"),
    }
}

fn exportar_srt_en(state: &mut EditorApp, carpeta: PathBuf) {
    let exportacion = exportar_srt_por_hablante(state.editor.proyecto());
    let mut escritos = 0;
    for (nombre, contenido) in &exportacion.archivos {
        let ruta = carpeta.join(nombre);
        match std::fs::write(&ruta, contenido) {
            Ok(()) => escritos += 1,
            Err(error) => eprintln!("no se pudo escribir '{}': {error}", ruta.display()),
        }
    }
    if let Some(reporte) = &exportacion.reporte_solapes {
        let ruta = carpeta.join("solapes.txt");
        if let Err(error) = std::fs::write(&ruta, reporte) {
            eprintln!("no se pudo escribir '{}': {error}", ruta.display());
        }
    }
    state.mensaje = Some(resumen_srt(&exportacion, escritos));
}

fn exportar_json_en(state: &mut EditorApp, ruta: PathBuf) {
    let texto = exportar_json_combinado(state.editor.proyecto());
    state.mensaje = Some(match std::fs::write(&ruta, texto) {
        Ok(()) => format!("JSON exportado: {}", ruta.display()),
        Err(error) => {
            eprintln!("no se pudo escribir el JSON: {error}");
            format!("No se pudo exportar el JSON: {error}")
        }
    });
}

fn exportar_ass_en(state: &mut EditorApp, ruta: PathBuf) {
    // La misma cadena que rasteriza el preview: si el archivo y la imagen
    // divergieran, sería un bug (invariante del proyecto).
    let texto = ass_del_proyecto(&state.editor, &state.presets);
    state.mensaje = Some(match std::fs::write(&ruta, texto) {
        Ok(()) => format!("ASS exportado: {}", ruta.display()),
        Err(error) => {
            eprintln!("no se pudo escribir el .ass: {error}");
            format!("No se pudo exportar el .ass: {error}")
        }
    });
}

pub fn update(state: &mut EditorApp, message: MensajeEditor) -> Tarea {
    match message {
        MensajeEditor::Comando(comando) => return ejecutar_comando(state, comando),
        MensajeEditor::Tick => {
            if let Some(receptor) = &state.rx_picos
                && let Ok(resultado) = receptor.try_recv()
            {
                state.picos = resultado.ok();
                state.rx_picos = None;
            }
            let frame_changed = state.player.update() & 1 != 0;

            // `time-pos` sigue en el instante viejo uno o dos frames después de
            // pedir un seek, y no se mueve mientras el usuario arrastra. Si se
            // adopta igual, el thumb salta hacia atrás y pelea con el dedo.
            // Mientras se arrastra, el reloj del decoder NO sobreescribe el
            // tiempo que ve el usuario (llega con retraso tras el seek).
            if state.scrub_ui_time.is_none() && !state.arrastrando && !state.player.seeking() {
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
        MensajeEditor::AbrirVideo => return Tarea::AbrirVideoEditor,
        MensajeEditor::DialogoVideoEditor(path) => {
            let Some(path) = path else {
                return Tarea::Ninguna;
            };

            let Some(path_text) = path.to_str() else {
                eprintln!("la ruta seleccionada no es UTF-8");
                return Tarea::Ninguna;
            };

            match rod_player::Player::open(path_text) {
                Ok(player) => reemplazar_player(state, path_text, player, 0.0),
                Err(error) => {
                    eprintln!("no se pudo abrir el video seleccionado: {error}");
                }
            }
        }
        MensajeEditor::AbrirProyecto => return Tarea::DialogoProyecto,
        MensajeEditor::DialogoProyecto(ruta) => {
            let Some(ruta) = ruta else {
                return Tarea::Ninguna;
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
            match state.ruta_proyecto.clone() {
                Some(ruta) if !pedir_ruta => guardar_en(state, ruta),
                _ => return Tarea::DialogoGuardar,
            }
        }
        MensajeEditor::DialogoGuardarProyecto(ruta) => {
            if let Some(ruta) = ruta {
                guardar_en(state, ruta);
            }
        }
        MensajeEditor::Recuperar => recuperar(state),
        MensajeEditor::DescartarRecovery => descartar_recovery(state),
        MensajeEditor::ExportarSrtHablantes => {
            if state.editor.proyecto().captions.is_empty() {
                state.mensaje = Some("No hay subtítulos para exportar.".into());
                return Tarea::Ninguna;
            }
            return Tarea::DialogoCarpeta;
        }
        MensajeEditor::DialogoCarpetaExportar(carpeta) => {
            if let Some(carpeta) = carpeta {
                exportar_srt_en(state, carpeta);
            }
        }
        MensajeEditor::ExportarJson => {
            if state.editor.proyecto().captions.is_empty() {
                state.mensaje = Some("No hay subtítulos para exportar.".into());
                return Tarea::Ninguna;
            }
            return Tarea::GuardarJson;
        }
        MensajeEditor::DialogoJson(ruta) => {
            if let Some(ruta) = ruta {
                exportar_json_en(state, ruta);
            }
        }
        MensajeEditor::ExportarAss => {
            if state.editor.proyecto().captions.is_empty() {
                state.mensaje = Some("No hay subtítulos para exportar.".into());
                return Tarea::Ninguna;
            }
            return Tarea::GuardarAss;
        }
        MensajeEditor::DialogoAssExportar(ruta) => {
            if let Some(ruta) = ruta {
                exportar_ass_en(state, ruta);
            }
        }
        MensajeEditor::DialogoSubtitulos(ruta, _ass) => {
            aplicar_subtitulos_cargados(state, ruta);
        }
        MensajeEditor::DialogoAutosubsSrt(ruta) => {
            return importar_autosubs_srt(state, ruta);
        }
        MensajeEditor::DialogoAutosubsTxt(ruta) => {
            importar_autosubs_txt(state, ruta);
        }
        MensajeEditor::PlayPause => {
            let destino = !state.player.paused();
            if destino && state.player.position() >= state.duration - 0.05 {
                if let Err(error) = state.player.seek(0.0) {
                    eprintln!("no se pudo reiniciar el video: {error}");
                    return Tarea::Ninguna;
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
            // Un clic en la lista lleva el preview al inicio del caption.
            if let Some(inicio) = state
                .editor
                .proyecto()
                .captions
                .iter()
                .find(|caption| caption.id == id)
                .map(|caption| caption.inicio)
            {
                let _ = state.player.seek(inicio);
            }
        }
        MensajeEditor::Borrador(texto) => state.borrador = texto,
        MensajeEditor::BorradorInicio(texto) => state.borrador_inicio = texto,
        MensajeEditor::BorradorFin(texto) => state.borrador_fin = texto,
        MensajeEditor::AplicarTexto => {
            // El botón está apagado sin selección, pero Enter en el `text_input`
            // puede llegar igual: aquí se vuelve a comprobar.
            let Some(id) = state.editor.seleccion.first().cloned() else {
                return Tarea::Ninguna;
            };
            let texto = state.borrador.clone();
            if state.editor.aplicar_texto(&id, texto) {
                refrescar_preview(state);
            }
        }
        MensajeEditor::UnirConSiguiente(id) => {
            state.editor.registrar_historial();
            let proyecto = state.editor.proyecto_mut();
            if let Some(pos) = proyecto
                .captions
                .iter()
                .position(|caption| caption.id == id)
                && pos + 1 < proyecto.captions.len()
            {
                let fin = proyecto.captions[pos + 1].fin;
                let texto_siguiente = proyecto.captions[pos + 1].texto.clone();
                proyecto.captions[pos].fin = fin;
                proyecto.captions[pos].texto =
                    format!("{} {}", proyecto.captions[pos].texto, texto_siguiente);
                proyecto.captions.remove(pos + 1);
            }
            refrescar_preview(state);
        }
        MensajeEditor::SeleccionarTodosHablante(hablante) => {
            let ids: Vec<_> = state
                .editor
                .proyecto()
                .captions
                .iter()
                .filter(|caption| caption.hablante_id.as_deref() == Some(&hablante))
                .map(|caption| caption.id.clone())
                .collect();
            state.editor.seleccion = ids.into_iter().collect();
        }
        MensajeEditor::EditarEnVivo(id, texto) => {
            if state.editando_texto.as_deref() != Some(&id) {
                state.editor.registrar_historial();
                state.editando_texto = Some(id.clone());
            }
            state.editor.editar_texto(&id, texto);
            refrescar_preview(state);
        }
        MensajeEditor::TerminarEdicionTexto => state.editando_texto = None,
        MensajeEditor::EditarInline(id, texto) => {
            state.inline = None;
            if state.editor.aplicar_texto(&id, texto) {
                refrescar_preview(state);
            }
        }
        MensajeEditor::CancelarInline => {
            state.inline = None;
        }
        MensajeEditor::EditarTiempo(id, campo, valor) => {
            let tiempo = rod_core::parse_time_input(&valor);
            let actuales = state
                .editor
                .proyecto()
                .captions
                .iter()
                .find(|caption| caption.id == id)
                .map(|caption| (caption.inicio, caption.fin));
            if let (Some(nuevo), Some((inicio, fin))) = (tiempo, actuales) {
                let (inicio, fin) = match campo {
                    CampoTiempo::Entrada if nuevo >= 0.0 && nuevo < fin => (nuevo, fin),
                    CampoTiempo::Salida if nuevo > inicio => (inicio, nuevo),
                    _ => (inicio, fin),
                };
                if (inicio, fin) != actuales.unwrap() {
                    state
                        .editor
                        .editar_tiempos(&id, inicio, fin, state.duration);
                    refrescar_preview(state);
                }
            }
            // Válido o no, el campo vuelve a mostrar el valor del proyecto.
            sincronizar_borrador(state);
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
        MensajeEditor::DuplicarHablante(id) => {
            let fuente = state
                .editor
                .proyecto()
                .hablantes
                .iter()
                .find(|h| h.id == id)
                .cloned();
            if let Some(mut hablante) = fuente {
                let nuevo_id = format!("sp-{}", state.editor.proyecto().hablantes.len() + 1);
                hablante.id = nuevo_id;
                hablante.nombre = format!("{} (copia)", hablante.nombre);
                state.editor.proyecto_mut().hablantes.push(hablante);
                refrescar_preview(state);
            }
        }
        MensajeEditor::AlternarMute(id) => {
            if !state.mute_hablantes.remove(&id) {
                state.mute_hablantes.insert(id);
            }
        }
        MensajeEditor::AlternarSolo(id) => {
            if !state.solo_hablantes.remove(&id) {
                state.solo_hablantes.insert(id);
            }
        }
        MensajeEditor::EditarHablante(id, campo, valor) => {
            // Si el texto no cambia no hay nada que registrar: borrar y volver a
            // escribir la misma letra no debe dejar un Ctrl+Z que no hace nada.
            let cambia = match (&campo, state.editor.hablante(&id)) {
                (CampoHablante::Nombre, Some(hablante)) => hablante.nombre != valor,
                (CampoHablante::Tecla, Some(hablante)) => {
                    hablante.tecla != normalizar_tecla(&valor)
                }
                (CampoHablante::Color, Some(hablante)) => hablante.color != valor,
                (_, None) => false,
            };
            if !cambia {
                return Tarea::Ninguna;
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
                CampoHablante::Color => {
                    if let Some(hablante) = state
                        .editor
                        .proyecto_mut()
                        .hablantes
                        .iter_mut()
                        .find(|hablante| hablante.id == id)
                    {
                        hablante.color = valor.clone();
                    }
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
        MensajeEditor::AplicarPresetSeleccion(preset) => {
            let ids: Vec<String> = state
                .editor
                .seleccion
                .iter()
                .filter_map(|caption_id| {
                    state
                        .editor
                        .proyecto()
                        .captions
                        .iter()
                        .find(|caption| caption.id == *caption_id)
                        .and_then(|caption| caption.hablante_id.clone())
                })
                .collect();
            for id in ids {
                state.editor.asignar_preset(&id, Some(preset.clone()));
            }
            refrescar_preview(state);
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
                return Tarea::Ninguna;
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
                return Tarea::Ninguna;
            }
            let Some(id) = state.preset_seleccionado.clone() else {
                return Tarea::Ninguna;
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
                return Tarea::Ninguna;
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
                if state.modificadores.ctrl || state.modificadores.shift {
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
                    return Tarea::Ninguna;
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
        MensajeEditor::ScrubInicio(tiempo) => {
            let objetivo = snap_a_frame(tiempo, state.duration);
            state.scrub_ui_time = Some(objetivo);
            state.position = objetivo;
            let _ = state.player.seek(objetivo);
        }
        MensajeEditor::ScrubMover(tiempo) => {
            // Último gana: no hay cola de seeks, solo el objetivo más reciente.
            let objetivo = snap_a_frame(tiempo, state.duration);
            state.scrub_ui_time = Some(objetivo);
            state.position = objetivo;
            let _ = state.player.seek(objetivo);
        }
        MensajeEditor::ScrubFin(tiempo) => {
            let objetivo = snap_a_frame(tiempo, state.duration);
            state.position = objetivo;
            state.scrub_ui_time = None;
            let _ = state.player.seek(objetivo);
        }
        MensajeEditor::Seguir => {
            state.seguir = !state.seguir;
            state.objetivo_vista = None;
        }
        MensajeEditor::Modificadores(modificadores) => {
            state.modificadores = modificadores;
        }
    }
    Tarea::Ninguna
}

#[cfg(test)]
mod tests {
    use image::GenericImageView;

    use super::{
        CampoTiempo, EditorApp, MensajeEditor, VELOCIDADES, Velocidad, duracion_texto,
        etiqueta_caption, resumen_srt, update,
    };
    use crate::comandos::Comando;
    use crate::tarea::Tarea;

    #[test]
    fn etiqueta_caption_include_indice_tiempo_y_texto() {
        assert_eq!(
            etiqueta_caption(7, 72.5, 75.25, "No, eso es lo que cambia todo."),
            "07  00:01:12.50 → 00:01:15.25  No, eso es lo que cambia todo."
        );
    }

    #[test]
    fn duracion_texto_es_segundos_con_dos_decimales() {
        assert_eq!(duracion_texto(72.5, 75.25), "2.75 s");
    }

    fn editor_con_caption() -> super::EditorApp {
        let proyecto = Proyecto {
            ruta_video: format!(
                "{}/../../testdata/clip-15s-2audio.mp4",
                env!("CARGO_MANIFEST_DIR")
            ),
            hablantes: Vec::new(),
            captions: vec![Caption {
                id: "c1".into(),
                inicio: 1.0,
                fin: 2.0,
                texto: "hola".into(),
                hablante_id: None,
            }],
            playhead: 0.0,
        };
        let mut app =
            super::EditorApp::desde_proyecto(proyecto, None, 0.0).expect("abre el video de prueba");
        let _ = super::update(&mut app, MensajeEditor::Seleccionar("c1".into()));
        app
    }

    #[test]
    fn editar_tiempo_rechaza_valor_invalido() {
        let mut app = editor_con_caption();
        let _ = super::update(
            &mut app,
            MensajeEditor::EditarTiempo("c1".into(), CampoTiempo::Entrada, "basura".into()),
        );
        assert_eq!(app.editor.proyecto().captions[0].inicio, 1.0);
    }

    #[test]
    fn editar_tiempo_rechaza_salida_anterior_a_entrada() {
        let mut app = editor_con_caption();
        let _ = super::update(
            &mut app,
            MensajeEditor::EditarTiempo("c1".into(), CampoTiempo::Salida, "0.5".into()),
        );
        assert_eq!(app.editor.proyecto().captions[0].fin, 2.0);
    }

    #[test]
    fn editar_tiempo_acepta_entrada_valida() {
        let mut app = editor_con_caption();
        let _ = super::update(
            &mut app,
            MensajeEditor::EditarTiempo("c1".into(), CampoTiempo::Entrada, "1.5".into()),
        );
        assert_eq!(app.editor.proyecto().captions[0].inicio, 1.5);
    }

    #[test]
    fn aplicar_autosubs_asigna_hablantes_y_produce_ids() {
        let captions = vec![
            rod_core::Caption {
                id: "c1".into(),
                inicio: 0.0,
                fin: 1.0,
                texto: "¡Hola, mundo!".into(),
                hablante_id: None,
            },
            rod_core::Caption {
                id: "c2".into(),
                inicio: 1.0,
                fin: 2.0,
                texto: "Adiós".into(),
                hablante_id: None,
            },
        ];
        let turnos = vec![
            rod_core::TurnoAutosubs {
                speaker: "1".into(),
                texto: "Hola mundo".into(),
            },
            rod_core::TurnoAutosubs {
                speaker: "2".into(),
                texto: "Adiós".into(),
            },
        ];
        let (hablantes, asignados) = super::aplicar_autosubs(&captions, &turnos);
        assert_eq!(hablantes.len(), 2);
        assert_eq!(hablantes[0].nombre, "Hablante 1");
        assert_eq!(asignados.len(), 2);
        assert_eq!(asignados[0].hablante_id, Some(hablantes[0].id.clone()));
        assert_eq!(asignados[1].hablante_id, Some(hablantes[1].id.clone()));
        // Los ids son distintos: no se reutiliza el mismo para dos hablantes.
        assert_ne!(hablantes[0].id, hablantes[1].id);
    }

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

    #[test]
    fn guardar_como_pide_dialogo() {
        let video = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../testdata/clip-15s-2audio.mp4"
        );
        let mut editor = EditorApp::desde_video(video).expect("video de prueba");
        let tarea = update(&mut editor, MensajeEditor::Comando(Comando::GuardarComo));
        assert_eq!(tarea, Tarea::DialogoGuardar);
    }
}
