use crate::{Caption, Hablante, PresetAss};

#[derive(Debug, Clone, PartialEq)]
pub struct Proyecto {
    pub ruta_video: String,
    pub hablantes: Vec<Hablante>,
    pub captions: Vec<Caption>,
    pub playhead: f64,
}

pub type Preset = PresetAss;

impl Proyecto {
    pub fn nuevo(ruta_video: impl Into<String>) -> Self {
        Self {
            ruta_video: ruta_video.into(),
            hablantes: Vec::new(),
            captions: Vec::new(),
            playhead: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Proyecto;

    #[test]
    fn starts_with_an_empty_editable_project() {
        let project = Proyecto::nuevo("video.mp4");
        assert_eq!(project.ruta_video, "video.mp4");
        assert!(project.hablantes.is_empty());
        assert!(project.captions.is_empty());
        assert_eq!(project.playhead, 0.0);
    }
}
