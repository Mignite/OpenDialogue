//! Dónde vive la configuración del usuario en cada sistema.
//!
//! Windows mantiene `%APPDATA%\RustyOpenDialogue`, la misma carpeta que el
//! oráculo Tauri, para que presets y proyectos se compartan. Linux usa las
//! rutas XDG: `$XDG_CONFIG_HOME/RustyOpenDialogue` (o `~/.config`) para los
//! archivos y `$XDG_CACHE_HOME` (o `~/.cache`) para las miniaturas.
//!
//! Antes cada módulo repetía `match std::env::var_os("APPDATA")`, así que en
//! Linux todo caía a `/tmp` y se perdía al reiniciar. La resolución vive aquí
//! y recibe las variables como parámetros: los tests no tocan el entorno.

use std::ffi::OsString;
use std::path::PathBuf;

/// `%APPDATA%\RustyOpenDialogue` en Windows; `$XDG_CONFIG_HOME` o
/// `~/.config` en Linux. Sin ninguna variable cae al temporal: perder la
/// configuración es mejor que no arrancar.
pub fn carpeta_config() -> PathBuf {
    config_en(
        std::env::var_os("APPDATA"),
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
        std::env::temp_dir(),
    )
}

/// Un archivo dentro de [`carpeta_config`].
pub fn archivo_config(nombre: &str) -> PathBuf {
    carpeta_config().join(nombre)
}

/// Miniaturas: `%APPDATA%\RustyOpenDialogue\thumbs` en Windows;
/// `$XDG_CACHE_HOME` o `~/.cache` en Linux. Son regenerables, por eso van a
/// la caché y no a la configuración.
pub fn carpeta_cache() -> PathBuf {
    cache_en(
        std::env::var_os("APPDATA"),
        std::env::var_os("XDG_CACHE_HOME"),
        std::env::var_os("HOME"),
        std::env::temp_dir(),
    )
}

/// Una variable vacía es como si no estuviera: un `XDG_CONFIG_HOME=` en el
/// entorno no debe producir una ruta relativa.
fn no_vacio(valor: Option<OsString>) -> Option<PathBuf> {
    valor.filter(|valor| !valor.is_empty()).map(PathBuf::from)
}

fn config_en(
    appdata: Option<OsString>,
    xdg_config: Option<OsString>,
    home: Option<OsString>,
    temporal: PathBuf,
) -> PathBuf {
    if let Some(base) = no_vacio(appdata) {
        return base.join("RustyOpenDialogue");
    }
    if let Some(base) = no_vacio(xdg_config) {
        return base.join("RustyOpenDialogue");
    }
    if let Some(base) = no_vacio(home) {
        return base.join(".config").join("RustyOpenDialogue");
    }
    temporal.join("RustyOpenDialogue")
}

fn cache_en(
    appdata: Option<OsString>,
    xdg_cache: Option<OsString>,
    home: Option<OsString>,
    temporal: PathBuf,
) -> PathBuf {
    if let Some(base) = no_vacio(appdata) {
        return base.join("RustyOpenDialogue").join("thumbs");
    }
    if let Some(base) = no_vacio(xdg_cache) {
        return base.join("RustyOpenDialogue").join("thumbs");
    }
    if let Some(base) = no_vacio(home) {
        return base.join(".cache").join("RustyOpenDialogue").join("thumbs");
    }
    temporal.join("RustyOpenDialogue-thumbs")
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use super::{cache_en, config_en};

    fn os(texto: &str) -> Option<OsString> {
        Some(OsString::from(texto))
    }

    fn temporal() -> PathBuf {
        Path::new("/tmp/rod-test").to_path_buf()
    }

    #[test]
    fn windows_sigue_usando_appdata() {
        let ruta = config_en(
            os("C:/Users/eche/AppData/Roaming"),
            os("/xdg/ignorado"),
            os("/home/ignorado"),
            temporal(),
        );
        assert_eq!(
            ruta,
            Path::new("C:/Users/eche/AppData/Roaming/RustyOpenDialogue")
        );
    }

    #[test]
    fn linux_usa_xdg_config_home() {
        let ruta = config_en(
            None,
            os("/home/eche/.config2"),
            os("/home/eche"),
            temporal(),
        );
        assert_eq!(ruta, Path::new("/home/eche/.config2/RustyOpenDialogue"));
    }

    #[test]
    fn sin_xdg_cae_al_config_del_home() {
        let ruta = config_en(None, None, os("/home/eche"), temporal());
        assert_eq!(ruta, Path::new("/home/eche/.config/RustyOpenDialogue"));
    }

    #[test]
    fn sin_variables_cae_al_temporal() {
        let ruta = config_en(None, None, None, temporal());
        assert_eq!(ruta, Path::new("/tmp/rod-test/RustyOpenDialogue"));
    }

    #[test]
    fn una_variable_vacia_se_ignora() {
        let ruta = config_en(os(""), os(""), os("/home/eche"), temporal());
        assert_eq!(ruta, Path::new("/home/eche/.config/RustyOpenDialogue"));
    }

    #[test]
    fn la_cache_usa_xdg_cache_home() {
        let ruta = cache_en(None, os("/home/eche/.cache2"), os("/home/eche"), temporal());
        assert_eq!(
            ruta,
            Path::new("/home/eche/.cache2/RustyOpenDialogue/thumbs")
        );
    }

    #[test]
    fn la_cache_de_windows_no_se_mueve() {
        let ruta = cache_en(os("C:/Users/eche/AppData/Roaming"), None, None, temporal());
        assert_eq!(
            ruta,
            Path::new("C:/Users/eche/AppData/Roaming/RustyOpenDialogue/thumbs")
        );
    }

    #[test]
    fn la_cache_sin_variables_cae_al_temporal() {
        let ruta = cache_en(None, None, None, temporal());
        assert_eq!(ruta, Path::new("/tmp/rod-test/RustyOpenDialogue-thumbs"));
    }

    /// El bug de campo: en Linux no hay `APPDATA` y todo caía a `/tmp`. Con
    /// `HOME` (o XDG) presente, la ruta real no puede ser temporal.
    #[test]
    fn con_carpeta_de_usuario_la_config_real_no_es_temporal() {
        let hay_base = std::env::var_os("APPDATA").is_some()
            || std::env::var_os("XDG_CONFIG_HOME").is_some()
            || std::env::var_os("HOME").is_some();
        if !hay_base {
            return;
        }
        assert!(!super::carpeta_config().starts_with(std::env::temp_dir()));
        assert!(!super::carpeta_cache().starts_with(std::env::temp_dir()));
    }
}
