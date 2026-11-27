//! Los presets `.ass` en disco.
//!
//! El formato (`presets_ass.json`: `{ "presets": [...] }` con campos en
//! camelCase) lo define `rod_core::ass_presets`, que es el mismo que escribe la
//! app Tauri. Aquí solo viven la ruta y la E/S; leer nunca falla: un archivo
//! ausente o roto cae a la semilla de un preset por defecto, porque la app
//! tiene que abrir siempre con algo usable.

use std::path::{Path, PathBuf};

use rod_core::{PresetAss, presets_a_json, presets_desde_json};

/// `%APPDATA%\RustyOpenDialogue\presets_ass.json`. El nombre del archivo es el
/// del oráculo (`ASS_PRESETS_ARCHIVO`), para que un usuario pueda copiarlo de
/// una app a la otra. Sin `APPDATA` (sesión rara) cae al temporal: perder los
/// presets es mejor que no arrancar.
pub fn ruta() -> PathBuf {
    match std::env::var_os("APPDATA") {
        Some(appdata) => Path::new(&appdata)
            .join("RustyOpenDialogue")
            .join("presets_ass.json"),
        None => std::env::temp_dir().join("RustyOpenDialogue-presets_ass.json"),
    }
}

/// Presets del archivo. Cualquier problema (no existe, JSON corrupto, lista
/// vacía) devuelve la semilla: `presets_desde_json` ya decide eso.
pub fn leer(ruta: &Path) -> Vec<PresetAss> {
    let texto = std::fs::read_to_string(ruta).ok();
    presets_desde_json(texto.as_deref())
}

/// Escribe los presets con `tmp` + `rename`: un crash a mitad no deja un JSON
/// truncado que se lea como «sin presets» en el arranque siguiente.
pub fn escribir(ruta: &Path, presets: &[PresetAss]) -> std::io::Result<()> {
    if let Some(carpeta) = ruta.parent() {
        std::fs::create_dir_all(carpeta)?;
    }
    let temporal = ruta.with_extension("tmp");
    std::fs::write(&temporal, presets_a_json(presets))?;
    std::fs::rename(&temporal, ruta).inspect_err(|_| {
        let _ = std::fs::remove_file(&temporal);
    })
}

#[cfg(test)]
mod tests {
    use super::{escribir, leer, ruta};
    use rod_core::{PresetAss, preset_por_defecto};

    fn preset(nombre: &str, fontsize: f64) -> PresetAss {
        PresetAss {
            nombre: nombre.into(),
            fontsize,
            ..preset_por_defecto()
        }
    }

    /// Un archivo propio por test: los tests corren en paralelo.
    fn ruta_temporal(nombre: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("rod-presets-{}-{nombre}.json", std::process::id()))
    }

    #[test]
    fn la_ruta_es_el_archivo_del_oraculo() {
        // Sin `APPDATA` (Linux) la ruta cae al temporal pero conserva el
        // nombre: el test no puede asumir la carpeta de Windows.
        let ruta = ruta();
        assert!(
            ruta.to_string_lossy().ends_with("presets_ass.json"),
            "{}",
            ruta.display()
        );
    }

    #[test]
    fn sin_archivo_se_siembra_el_preset_por_defecto() {
        let ruta = ruta_temporal("inexistente");
        let presets = leer(&ruta);
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0], preset_por_defecto());
    }

    #[test]
    fn escribir_y_leer_devuelve_los_mismos_presets() {
        let ruta = ruta_temporal("ida-vuelta");
        let original = vec![preset("Cine", 64.0), preset("Notas", 32.0)];
        escribir(&ruta, &original).expect("escribe");
        assert_eq!(leer(&ruta), original);
        // El temporal no queda tirado al lado del archivo bueno.
        assert!(!ruta.with_extension("tmp").exists());
        std::fs::remove_file(&ruta).expect("limpia");
    }

    #[test]
    fn un_archivo_corrupto_cae_a_la_semilla() {
        let ruta = ruta_temporal("corrupto");
        std::fs::write(&ruta, "{no es json").expect("escribe basura");
        assert_eq!(leer(&ruta), vec![preset_por_defecto()]);
        std::fs::remove_file(&ruta).expect("limpia");
    }

    #[test]
    fn escribir_crea_la_carpeta_si_falta() {
        let ruta = ruta_temporal("carpeta")
            .join("anidada")
            .join("presets_ass.json");
        escribir(&ruta, &[preset_por_defecto()]).expect("crea la carpeta");
        assert!(ruta.exists());
        std::fs::remove_dir_all(
            ruta.parent()
                .expect("tiene padre")
                .parent()
                .expect("abuelo"),
        )
        .expect("limpia");
    }
}
