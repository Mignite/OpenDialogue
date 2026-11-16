use std::sync::Arc;

use iced::widget::{
    button, column, container, pick_list, row, scrollable, shader, slider, space, text,
};
use iced::{Background, Color, Element, Length, Subscription, Task};

use rod_app::editor::EstadoEditor;
use rod_app::project_io::cargar;
use rod_app::video::{Capa, Video};
use rod_player::AudioTrack;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const ASS_FIXTURE: &str = include_str!("../../subs/tests/fixtures/basico.ass");

#[derive(Debug, Clone)]
enum Message {
    Tick,
    AbrirVideo,
    AudioTrack(AudioTrack),
    PlayPause,
    /// El usuario mueve el slider pero todavía no suelta: solo se mueve el
    /// thumb, no se le pide nada a mpv.
    Arrastrar(f32),
    /// Soltó el slider: aquí sí se busca.
    Soltar,
    Step(f64),
    Seleccionar(String),
    BorrarSeleccion,
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
}

fn main() -> iced::Result {
    iced::application(Estado::nuevo, update, view)
        .title("Rusty Open Dialogue")
        .window_size([1600.0, 900.0])
        .subscription(subscription)
        .run()
}

fn subscription(_state: &Estado) -> Subscription<Message> {
    iced::window::frames().map(|_| Message::Tick)
}

impl Estado {
    fn nuevo() -> Self {
        let documento = cargar(
            "1\n00:00:01,000 --> 00:00:03,000\nPrimer caption\n\n\
             2\n00:00:04,000 --> 00:00:06,500\nSegundo caption",
            "testdata/clip-15s-2audio.mp4",
        )
        .expect("el fixture inicial debe ser SRT valido");
        let video_path = "testdata/clip-15s-2audio.mp4".to_owned();
        let player = rod_player::Player::open(&video_path)
            .expect("no se pudo abrir testdata/clip-15s-2audio.mp4");
        let duration = player.duration();
        let paused = player.paused();
        let audio_tracks = player.audio_tracks();
        let selected_audio = audio_tracks.first().cloned();
        let mut subtitles = rod_subs::Subtitulos::nuevo(WIDTH as u32, HEIGHT as u32, ASS_FIXTURE);
        subtitles.render(0);

        Self {
            editor: EstadoEditor::nuevo(documento),
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
        }
    }
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
        Message::Seleccionar(id) => state.editor.seleccionar_uno(&id),
        Message::BorrarSeleccion => {
            state.editor.borrar_seleccion();
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
                button("Borrar").on_press(Message::BorrarSeleccion),
            ]
            .spacing(12),
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
