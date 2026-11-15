use std::cell::Cell;
use std::fmt;

pub mod mpv_render;
use libmpv2::{Mpv, events::Event};
use mpv_render::RenderContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioTrack {
    pub id: i64,
    pub label: String,
}

impl fmt::Display for AudioTrack {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label)
    }
}

pub struct Player {
    render: RenderContext,
    mpv: Mpv,
    path: String,
    ended: Cell<bool>,
}

impl Player {
    pub fn open(path: &str) -> libmpv2::Result<Self> {
        let mpv = Mpv::with_initializer(|init| {
            init.set_option("vo", "libmpv".to_string())?;
            init.set_option("config", "no".to_string())?;
            Ok(())
        })?;

        let render = RenderContext::new(&mpv)?;

        mpv.command("loadfile", &[path, "replace"])?;

        loop {
            match mpv.wait_event(-1.0) {
                Some(Ok(Event::FileLoaded)) => break,
                Some(Ok(Event::EndFile(reason))) => {
                    eprintln!("Se acabo antes de cargar: {reason:?}");
                    break;
                }
                Some(Ok(otro)) => println!("Evento inesperado: {otro:?}"),
                Some(Err(e)) => {
                    eprintln!("Error al esperar evento: {e:?}");
                    break;
                }
                None => {
                    eprintln!("Se acabo el tiempo de espera antes de cargar");
                    break;
                }
            }
        }

        Ok(Self {
            render,
            mpv,
            path: path.to_owned(),
            ended: Cell::new(false),
        })
    }

    pub fn update(&self) -> u64 {
        self.drain_events();
        self.render.update()
    }

    fn drain_events(&self) {
        while let Some(event) = self.mpv.wait_event(0.0) {
            match event {
                Ok(Event::EndFile(_)) => {
                    self.ended.set(true);
                }
                Ok(Event::FileLoaded) => {
                    self.ended.set(false);
                }
                Ok(Event::StartFile)
                | Ok(Event::Seek)
                | Ok(Event::PlaybackRestart)
                | Ok(Event::VideoReconfig)
                | Ok(Event::AudioReconfig)
                | Ok(Event::Deprecated(_)) => {}
                Ok(Event::QueueOverflow) => {
                    eprintln!("la cola de eventos de mpv se desbordo");
                }
                Ok(_) => {}
                Err(error) => {
                    eprintln!("error de evento mpv: {error}");
                }
            }
        }
    }

    pub fn mpv(&self) -> &Mpv {
        &self.mpv
    }

    pub fn audio_tracks(&self) -> Vec<AudioTrack> {
        let mut tracks = Vec::new();

        for index in 0..32 {
            let prefix = format!("track-list/{index}");
            let Ok(track_type) = self.mpv.get_property::<String>(&format!("{prefix}/type")) else {
                break;
            };
            if track_type != "audio" {
                continue;
            }

            let Ok(id) = self.mpv.get_property::<i64>(&format!("{prefix}/id")) else {
                continue;
            };
            let language = self
                .mpv
                .get_property::<String>(&format!("{prefix}/lang"))
                .unwrap_or_default();
            let title = self
                .mpv
                .get_property::<String>(&format!("{prefix}/title"))
                .unwrap_or_default();
            let label = match (language.is_empty(), title.is_empty()) {
                (false, false) => format!("{language} — {title}"),
                (false, true) => language,
                (true, false) => title,
                (true, true) => format!("Audio {id}"),
            };
            tracks.push(AudioTrack { id, label });
        }

        tracks
    }

    pub fn set_audio_track(&self, id: i64) -> libmpv2::Result<()> {
        self.drain_events();
        self.mpv.set_property("aid", id)
    }

    pub fn duration(&self) -> f64 {
        self.mpv.get_property::<f64>("duration").unwrap_or(0.0)
    }

    pub fn position(&self) -> f64 {
        if self.ended.get() {
            return self.duration();
        }
        self.mpv
            .get_property::<f64>("time-pos")
            .ok()
            .filter(|position| position.is_finite())
            .unwrap_or_else(|| self.duration())
    }

    pub fn paused(&self) -> bool {
        self.mpv.get_property::<bool>("pause").unwrap_or(false)
    }

    pub fn set_paused(&self, paused: bool) -> libmpv2::Result<()> {
        self.drain_events();
        self.mpv.set_property("pause", paused)
    }

    pub fn seek(&self, seconds: f64) -> libmpv2::Result<()> {
        self.drain_events();
        let target = seconds.max(0.0).min(self.duration());
        self.ended.set(false);
        match self.mpv.set_property("time-pos", target) {
            Ok(()) => Ok(()),
            Err(libmpv2::Error::Raw(code)) if code == libmpv2::mpv_error::PropertyUnavailable => {
                self.mpv.command("loadfile", &[&self.path, "replace"])?;
                loop {
                    match self.mpv.wait_event(-1.0) {
                        Some(Ok(Event::FileLoaded)) => break,
                        Some(Ok(Event::EndFile(_))) | Some(Ok(Event::StartFile)) => {}
                        Some(Ok(_)) => {}
                        Some(Err(error)) => return Err(error),
                        None => continue,
                    }
                }
                self.mpv.set_property("time-pos", target)
            }
            Err(error) => Err(error),
        }
    }

    pub fn render(&self, buf: &mut [u8], width: usize, height: usize) -> libmpv2::Result<()> {
        self.render.render(buf, width, height)
    }
}
