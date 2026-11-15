//! Laboratorio del slice 0c: video de mpv con los subtítulos de libass encima.
//!
//! Los subtítulos se rasterizan en cada frame al `time-pos` actual y se pasan
//! como capas al widget; el widget las sube a un atlas y las dibuja con
//! blending sobre el video.

use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::widget::{column, shader, text};
use iced::{Element, Length};

use rod_app::video::{Capa, Video};

const W: usize = 1920;
const H: usize = 1080;

/// El fixture del slice 0b: dos estilos, un `\pos` y una línea normal.
const FIXTURE: &str = include_str!("../../subs/tests/fixtures/basico.ass");

struct Lab {
    player: rod_player::Player,
    buf: Vec<u8>,
    subtitulos: rod_subs::Subtitulos,
    capas: Arc<Vec<Capa>>,
    frames: u64,
    desde: Instant,
    fps: f32,
    /// Acumuladores del segundo en curso.
    acumulado_mpv: f64,
    acumulado_libass: f64,
    /// Medias del último segundo completo, para mostrar.
    ms_mpv: f64,
    ms_libass: f64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick,
}

fn main() -> iced::Result {
    iced::application(Lab::nuevo, update, view)
        .title("lab: video + subtitulos")
        .window_size([1280.0, 720.0])
        .subscription(|_| iced::window::frames().map(|_| Message::Tick))
        .run()
}

impl Lab {
    fn nuevo() -> Self {
        let player = rod_player::Player::open("testdata/clip-1080p60.mp4")
            .expect("no se pudo abrir el video");
        player
            .mpv()
            .set_property("loop-file", "inf")
            .expect("loop-file");

        let mut subtitulos = rod_subs::Subtitulos::nuevo(W as u32, H as u32, FIXTURE);
        // Calentar: la primera rasterización inicializa el proveedor de fuentes
        // y cachea los glifos. Si se deja para el primer frame visible, el
        // primer segundo se cae a ~13 fps.
        subtitulos.render(1000);

        Self {
            player,
            buf: vec![0u8; W * H * 4],
            subtitulos,
            capas: Arc::new(Vec::new()),
            frames: 0,
            desde: Instant::now(),
            fps: 0.0,
            acumulado_mpv: 0.0,
            acumulado_libass: 0.0,
            ms_mpv: 0.0,
            ms_libass: 0.0,
        }
    }
}

fn update(lab: &mut Lab, _mensaje: Message) {
    if lab.player.update() & 1 == 0 {
        return;
    }

    let antes_render = Instant::now();
    if lab.player.render(&mut lab.buf, W, H).is_err() {
        return;
    }
    let despues_render = Instant::now();

    // Los subtítulos se piden al reloj del video, no al del sistema: en pausa
    // o tras un seek el texto que toca es el del frame, no el del reloj real.
    let posicion = lab
        .player
        .mpv()
        .get_property::<f64>("time-pos")
        .unwrap_or(0.0);

    lab.capas = Arc::new(
        lab.subtitulos
            .render((posicion * 1000.0) as i64)
            .iter()
            .map(Capa::desde)
            .collect(),
    );
    let despues_capas = Instant::now();

    lab.acumulado_mpv += despues_render.duration_since(antes_render).as_secs_f64() * 1000.0;
    lab.acumulado_libass += despues_capas.duration_since(despues_render).as_secs_f64() * 1000.0;

    lab.frames += 1;

    let transcurrido = lab.desde.elapsed();
    if transcurrido >= Duration::from_secs(1) {
        lab.fps = lab.frames as f32 / transcurrido.as_secs_f32();
        lab.ms_mpv = lab.acumulado_mpv / lab.frames as f64;
        lab.ms_libass = lab.acumulado_libass / lab.frames as f64;
        lab.frames = 0;
        lab.acumulado_mpv = 0.0;
        lab.acumulado_libass = 0.0;
        lab.desde = Instant::now();
    }
}

fn view(lab: &Lab) -> Element<'_, Message> {
    column![
        text(format!(
            "{:.1} fps — {} capas — mpv {:.1} ms, libass {:.1} ms",
            lab.fps,
            lab.capas.len(),
            lab.ms_mpv,
            lab.ms_libass
        )),
        shader(Video {
            pixels: Arc::new(lab.buf.clone()),
            width: W as u32,
            height: H as u32,
            capas: Arc::clone(&lab.capas),
        })
        .width(Length::Fill)
        .height(Length::Fill),
    ]
    .into()
}
