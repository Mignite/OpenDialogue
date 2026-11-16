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
            // Buscar exacto: sin esto mpv aterriza en el keyframe anterior y el
            // playhead no coincide con lo que se pidió.
            init.set_option("hr-seek", "yes".to_string())?;
            // Al final del archivo, quedarse en el último frame en vez de
            // cerrar el stream: el editor sigue mostrando imagen.
            init.set_option("keep-open", "yes".to_string())?;
            init.set_option("hwdec", "auto-safe".to_string())?;
            init.set_option("audio-pitch-correction", "yes".to_string())?;
            init.set_option("config", "no".to_string())?;
            Ok(())
        })?;

        let render = RenderContext::new(&mpv)?;

        mpv.command("loadfile", &[path, "replace"])?;
        esperar_carga(&mpv)?;

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

    /// La pista que mpv tiene activa de verdad, ya resuelta.
    pub fn audio_track(&self) -> Option<i64> {
        self.mpv.get_property::<i64>("aid").ok()
    }

    /// `true` mientras mpv está buscando. La app lo usa para no pisar la
    /// posición que pidió el usuario con la que todavía no cambió.
    pub fn seeking(&self) -> bool {
        self.mpv.get_property::<bool>("seeking").unwrap_or(false)
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
                // Sin archivo cargado no hay `time-pos` que escribir. Se recarga
                // y se busca otra vez. `esperar_carga` acota la espera: colgar el
                // hilo de la UI para siempre es peor que fallar.
                self.mpv.command("loadfile", &[&self.path, "replace"])?;
                esperar_carga(&self.mpv)?;
                self.mpv.set_property("time-pos", target)
            }
            Err(error) => Err(error),
        }
    }

    pub fn render(&self, buf: &mut [u8], width: usize, height: usize) -> libmpv2::Result<()> {
        self.render.render(buf, width, height)
    }
}

/// Espera a que mpv termine de cargar el archivo actual.
///
/// Acotado a 5 s: si no llega `FileLoaded` se devuelve error en vez de dejar el
/// hilo (que en la app es el de la UI) esperando un evento que puede no venir.
fn esperar_carga(mpv: &Mpv) -> libmpv2::Result<()> {
    loop {
        match mpv.wait_event(5.0) {
            Some(Ok(Event::FileLoaded)) => return Ok(()),
            Some(Ok(Event::EndFile(razon))) => {
                eprintln!("mpv termino antes de cargar: {razon:?}");
                return Err(libmpv2::Error::Raw(libmpv2::mpv_error::LoadingFailed));
            }
            Some(Ok(_)) => {}
            Some(Err(error)) => return Err(error),
            None => {
                eprintln!("mpv no cargo el archivo en 5 s");
                return Err(libmpv2::Error::Raw(libmpv2::mpv_error::LoadingFailed));
            }
        }
    }
}
