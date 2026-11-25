//! Preferencias persistentes: distribución, idioma y atajos.
//!
//! Se guardan en `%APPDATA%\RustyOpenDialogue\preferencias.json` con
//! escritura atómica. Un archivo roto o de otra versión cae a los valores de
//! fábrica sin impedir que la app arranque.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::comandos::{Atajo, Comando, MapaAtajos};
use crate::textos::Idioma;

/// Representación serializada de una combinación (sin `egui::Key`, que no
/// tiene serde activado en este workspace).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtajoGuardado {
    pub tecla: String,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct Datos {
    ancho_panel: f32,
    alto_timeline: f32,
    panel_visible: bool,
    snap: bool,
    seguir: bool,
    idioma: String,
    atajos: Vec<(String, AtajoGuardado)>,
}

impl Default for Datos {
    fn default() -> Self {
        Self {
            ancho_panel: 320.0,
            alto_timeline: 260.0,
            panel_visible: true,
            snap: true,
            seguir: false,
            idioma: "en".into(),
            atajos: MapaAtajos::por_defecto().guardadas(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Preferencias {
    pub ancho_panel: f32,
    pub alto_timeline: f32,
    pub panel_visible: bool,
    pub snap: bool,
    pub seguir: bool,
    pub idioma: Idioma,
    pub atajos: MapaAtajos,
}

impl Default for Preferencias {
    fn default() -> Self {
        Self {
            ancho_panel: Datos::default().ancho_panel,
            alto_timeline: Datos::default().alto_timeline,
            panel_visible: true,
            snap: true,
            seguir: false,
            idioma: Idioma::default(),
            atajos: MapaAtajos::por_defecto(),
        }
    }
}

impl Preferencias {
    /// Ruta canónica. No se cambia: moverla huérfanaría recientes y presets.
    pub fn ruta() -> PathBuf {
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        base.join("RustyOpenDialogue").join("preferencias.json")
    }

    pub fn cargar() -> Self {
        Self::cargar_en(&Self::ruta())
    }

    pub fn cargar_en(ruta: &Path) -> Self {
        let Ok(texto) = std::fs::read_to_string(ruta) else {
            return Self::default();
        };
        let Ok(datos) = serde_json::from_str::<Datos>(&texto) else {
            return Self::default();
        };
        Self {
            ancho_panel: datos.ancho_panel,
            alto_timeline: datos.alto_timeline,
            panel_visible: datos.panel_visible,
            snap: datos.snap,
            seguir: datos.seguir,
            idioma: if datos.idioma == "es" {
                Idioma::Es
            } else {
                Idioma::En
            },
            atajos: MapaAtajos::desde_guardadas(&datos.atajos),
        }
    }

    pub fn guardar(&self) -> std::io::Result<()> {
        self.guardar_en(&Self::ruta())
    }

    pub fn guardar_en(&self, ruta: &Path) -> std::io::Result<()> {
        if let Some(carpeta) = ruta.parent() {
            std::fs::create_dir_all(carpeta)?;
        }
        let datos = Datos {
            ancho_panel: self.ancho_panel,
            alto_timeline: self.alto_timeline,
            panel_visible: self.panel_visible,
            snap: self.snap,
            seguir: self.seguir,
            idioma: match self.idioma {
                Idioma::Es => "es".into(),
                Idioma::En => "en".into(),
            },
            atajos: self.atajos.guardadas(),
        };
        let texto = serde_json::to_string_pretty(&datos)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let temporal = ruta.with_extension("tmp");
        std::fs::write(&temporal, texto)?;
        if ruta.exists() {
            std::fs::remove_file(ruta)?;
        }
        std::fs::rename(&temporal, ruta)
    }

    pub fn atajo(&self, comando: Comando) -> Option<Atajo> {
        self.atajos.atajo(comando)
    }
}
