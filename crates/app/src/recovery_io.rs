//! El archivo de recuperación en disco.
//!
//! La lógica (qué es válido, cuándo toca escribir) vive en
//! `rod_core::recovery`; aquí solo están la ruta, el reloj y la E/S. Igual que
//! en el oráculo (`guardar_recovery`/`leer_recovery`), el archivo es uno solo y
//! `None` significa «bórralo».

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rod_core::Recovery;

/// Reloj de pared en epoch ms, que es la unidad del `escrito` del recovery
/// (la misma que `Date.now()` en el oráculo).
///
/// Un reloj anterior a 1970 no existe en la práctica; si pasara, 0 es un valor
/// tan bueno como cualquiera para un archivo que se va a descartar por viejo.
pub fn ms_ahora() -> f64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(desde) => desde.as_secs_f64() * 1000.0,
        Err(_) => 0.0,
    }
}

/// `%APPDATA%\RustyOpenDialogue\recovery.json`, que es el equivalente del
/// `app_config_dir` del oráculo. Sin `APPDATA` (sesión rara) cae al directorio
/// temporal: perder el seguro es mejor que no arrancar.
pub fn ruta() -> PathBuf {
    match std::env::var_os("APPDATA") {
        Some(appdata) => Path::new(&appdata)
            .join("RustyOpenDialogue")
            .join("recovery.json"),
        None => std::env::temp_dir().join("RustyOpenDialogue-recovery.json"),
    }
}

/// Lee y valida el recovery de `ruta`. Cualquier fallo es `None`: un archivo
/// corrupto, de otra versión o sin permisos no puede impedir abrir la app.
pub fn leer(ruta: &Path, corte: f64, ahora: f64) -> Option<Recovery> {
    let texto = std::fs::read_to_string(ruta).ok()?;
    rod_core::leer_recovery(Some(&texto), corte, ahora)
}

/// Escribe `contenido` (o borra el archivo con `None`). Atómico como el
/// guardado del proyecto: un crash a mitad no deja un recovery truncado que
/// rompa el arranque siguiente. Que el archivo ya no esté al borrar no es un
/// error.
pub fn escribir(ruta: &Path, contenido: Option<&str>) -> std::io::Result<()> {
    let Some(texto) = contenido else {
        return match std::fs::remove_file(ruta) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        };
    };
    if let Some(carpeta) = ruta.parent() {
        std::fs::create_dir_all(carpeta)?;
    }
    let temporal = ruta.with_extension("tmp");
    std::fs::write(&temporal, texto)?;
    std::fs::rename(&temporal, ruta).inspect_err(|_| {
        let _ = std::fs::remove_file(&temporal);
    })
}

#[cfg(test)]
mod tests {
    use super::{escribir, leer, ruta};
    use rod_core::recovery::{a_json, construir};
    use rod_core::{Caption, Proyecto};

    fn proyecto() -> Proyecto {
        let mut proyecto = Proyecto::nuevo("testdata/clip-15s-2audio.mp4");
        proyecto.captions.push(Caption {
            id: "cap-1".into(),
            inicio: 1.0,
            fin: 2.0,
            texto: "Hola".into(),
            hablante_id: None,
        });
        proyecto
    }

    /// Un archivo propio por test: los tests corren en paralelo y el
    /// directorio temporal es compartido.
    fn ruta_temporal(nombre: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("rod-recovery-{}-{nombre}.json", std::process::id()))
    }

    #[test]
    fn la_ruta_es_un_recovery_json() {
        let ruta = ruta();
        assert_eq!(
            ruta.file_name().and_then(|nombre| nombre.to_str()),
            Some("recovery.json")
        );
    }

    #[test]
    fn escribir_y_leer_devuelve_el_mismo_proyecto() {
        let ruta = ruta_temporal("ida-vuelta");
        let recovery = construir(proyecto(), "/proyectos/mi.cdp", 5_000.0);
        let texto = a_json(&recovery).expect("serializa");
        escribir(&ruta, Some(&texto)).expect("escribe");
        let vuelta = leer(&ruta, 0.0, 5_000.0).expect("lee");
        assert_eq!(vuelta, recovery);
        // El temporal no queda tirado.
        assert!(!ruta.with_extension("tmp").exists());
        std::fs::remove_file(&ruta).expect("limpia");
    }

    #[test]
    fn borrar_un_archivo_que_no_existe_no_es_un_error() {
        let ruta = ruta_temporal("borrado-inexistente");
        assert!(!ruta.exists());
        escribir(&ruta, None).expect("borrar lo que no está es un no-op");
    }

    #[test]
    fn borrar_quita_el_recovery() {
        let ruta = ruta_temporal("borrado");
        escribir(&ruta, Some("{}")).expect("escribe");
        escribir(&ruta, None).expect("borra");
        assert!(!ruta.exists());
    }

    #[test]
    fn leer_un_archivo_que_no_existe_es_none() {
        let ruta = ruta_temporal("inexistente");
        assert_eq!(leer(&ruta, 0.0, 5_000.0), None);
    }

    #[test]
    fn leer_un_archivo_con_basura_es_none() {
        let ruta = ruta_temporal("basura");
        escribir(&ruta, Some("esto no es json")).expect("escribe");
        assert_eq!(leer(&ruta, 0.0, 5_000.0), None);
        std::fs::remove_file(&ruta).expect("limpia");
    }

    #[test]
    fn escribir_crea_la_carpeta_si_falta() {
        let ruta = ruta_temporal("carpeta")
            .join("anidada")
            .join("recovery.json");
        escribir(&ruta, Some("{}")).expect("crea la carpeta");
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
