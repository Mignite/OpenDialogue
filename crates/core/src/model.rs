use crate::{Caption, Hablante, PresetAss};
use serde::{Deserialize, Serialize};

/// El proyecto tal como lo guarda el oráculo (`Proyecto` de `src-tauri`): las
/// claves son las mismas para que un `.cdp` guardado por la app Tauri abra aquí
/// y al revés. `playhead` es lo único que puede faltar en un archivo viejo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proyecto {
    pub ruta_video: String,
    pub hablantes: Vec<Hablante>,
    pub captions: Vec<Caption>,
    #[serde(default)]
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
