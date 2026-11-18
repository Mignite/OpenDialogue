use std::sync::Arc;

use iced::widget::{
    button, column, container, pick_list, row, scrollable, shader, slider, space, text, text_input,
};
use iced::{Background, Color, Element, Length, Subscription, Task};

use rod_app::editor::EstadoEditor;
use rod_app::preview::{CANVAS, ass_del_proyecto};
use rod_app::project_io::cargar;
use rod_app::video::{Capa, Video};
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
        iced::keyboard::listen().filter_map(|evento| {
            let iced::keyboard::Event::KeyPressed { key, modifiers, .. } = evento else {
                return None;
            };
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
            None
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
        let ass = ass_del_proyecto(&editor);
        let subtitles = rod_subs::Subtitulos::nuevo(WIDTH as u32, HEIGHT as u32, &ass);

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
        };
        refrescar_preview(&mut estado);
        estado
    }
}

/// Reconstruye el `.ass` del proyecto y rasteriza las capas de la posición
/// actual. Se llama tras cada edición: el video no cambia, pero los subtítulos
/// sí, y en pausa mpv no entrega un frame nuevo para disparar el repintado.
fn refrescar_preview(state: &mut Estado) {
    let ass = ass_del_proyecto(&state.editor);
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
                Ok(player) => {
                    state.duration = player.duration();
                    state.position = 0.0;
                    state.paused = player.paused();
                    state.video_path = path_text.to_owned();
                    state.audio_tracks = player.audio_tracks();
                    state.selected_audio = state.audio_tracks.first().cloned();
                    state.velocidad = Velocidad(player.speed() as f32);
                    state.player = player;
                    state.layers = Arc::new(Vec::new());
                }
                Err(error) => {
                    eprintln!("no se pudo abrir el video seleccionado: {error}");
                }
            }
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
            if state.editor.deshacer() {
                sincronizar_borrador(state);
                refrescar_preview(state);
            }
        }
        Message::Rehacer => {
            if state.editor.rehacer() {
                sincronizar_borrador(state);
                refrescar_preview(state);
            }
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
    .height(48)
    .style(panel);

    let left = column![video, transport]
        .width(Length::FillPortion(3))
        .height(Length::Fill);
    let right = subtitle_panel(state);
    let timeline = container(
        column![
            text("Timeline").size(18),
            text("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"),
            text("duración total: ".to_owned() + &rod_core::format_time(state.duration)),
        ]
        .spacing(8)
        .padding(12),
    )
    .width(Length::Fill)
    .height(150)
    .style(panel);

    column![
        row![left, right].height(Length::Fill),
        timeline,
        status_bar(state),
    ]
    .into()
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
                text("Speakers"),
                text("Styles"),
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

fn status_bar(state: &Estado) -> Element<'_, Message> {
    container(
        row![
            text("● RUSTYOPENDIALOGUE"),
            text(state.video_path.clone()),
            space::horizontal(),
            text(format!(
                "{} captions · {} capas libass",
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

#[cfg(test)]
mod tests {
    use super::{VELOCIDADES, Velocidad};

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
}
