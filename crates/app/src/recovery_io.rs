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

use crate::textos::Textos;

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

/// `recovery.json` dentro de la carpeta de configuración del sistema, que es
/// el equivalente del `app_config_dir` del oráculo (`rutas::carpeta_config`).
pub fn ruta() -> PathBuf {
    crate::rutas::archivo_config("recovery.json")
}

/// Lee y valida el recovery de `ruta`. Cualquier fallo es `None`: un archivo
/// corrupto, de otra versión o sin permisos no puede impedir abrir la app.
pub fn leer(ruta: &Path, corte: f64, ahora: f64) -> Option<Recovery> {
    let texto = std::fs::read_to_string(ruta).ok()?;
    rod_core::leer_recovery(Some(&texto), corte, ahora)
}

/// Lo que el aviso de recuperación necesita: la copia y el texto ya formateado.
#[derive(Debug, Clone)]
pub struct AvisoRecovery {
    pub recovery: Recovery,
    /// Texto del core, siempre en español; se conserva por compatibilidad con
    /// quien construye el aviso. La vista usa [`Self::texto`], localizado.
    pub texto: String,
}

impl AvisoRecovery {
    /// Antigüedad del autosave en el idioma de la interfaz. El core imprime la
    /// fecha en español fijo y su formateador es privado, así que aquí se
    /// calcula de `escrito` con los mismos umbrales (minuto, hora, día).
    pub fn fecha(&self, textos: &Textos) -> String {
        let minutos = ((ms_ahora() - self.recovery.escrito) / 60_000.0)
            .floor()
            .max(0.0) as u64;
        let horas = minutos / 60;
        if textos.es {
            match minutos {
                0 => "hace un momento".into(),
                m if m < 60 => format!("hace {m} min"),
                _ if horas < 24 => format!("hace {horas} h"),
                _ => format!("hace {} días", horas / 24),
            }
        } else {
            match minutos {
                0 => "just now".into(),
                m if m < 60 => format!("{m} min ago"),
                _ if horas < 24 => format!("{horas} h ago"),
                _ => format!("{} days ago", horas / 24),
            }
        }
    }

    /// Aviso completo en el idioma de la UI: fecha localizada + número de
    /// subtítulos, con el nombre del proyecto cuando se conoce.
    pub fn texto(&self, textos: &Textos) -> String {
        let aviso =
            textos.fecha_recuperacion(&self.fecha(textos), self.recovery.proyecto.captions.len());
        let nombre = nombre_proyecto(&self.recovery);
        if nombre.is_empty() {
            aviso
        } else {
            format!("{aviso} ({nombre})")
        }
    }
}

/// Nombre con el que se reconoce la copia: el del archivo si hay ruta guardada.
fn nombre_proyecto(recovery: &Recovery) -> String {
    Path::new(&recovery.ruta_proyecto)
        .file_name()
        .and_then(|nombre| nombre.to_str())
        .unwrap_or_default()
        .to_owned()
}

/// Copia automática que quedó de la sesión anterior, si la hay y sirve.
///
/// La ruta y el reloj entran como parámetros para poder probar esto sin tocar
/// el `%APPDATA%` real ni depender de la hora.
pub fn pendiente(ruta: &Path, ahora: f64) -> Option<AvisoRecovery> {
    let recovery = leer(ruta, 0.0, ahora)?;
    // El nombre es el del archivo (`mi-video.cdp`), como en el oráculo: con la
    // ruta entera el aviso no cabe en una barra.
    let nombre = nombre_proyecto(&recovery);
    let texto = rod_core::formatear_recovery(&recovery, &nombre, ahora);
    Some(AvisoRecovery { recovery, texto })
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
    use super::{AvisoRecovery, escribir, leer, ms_ahora, ruta};
    use crate::textos::{EN, ES};
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
    fn el_aviso_se_formatea_en_el_idioma_de_la_ui() {
        let aviso = AvisoRecovery {
            recovery: construir(
                Proyecto::nuevo("clip.mp4"),
                "C:/proyectos/mi.cdp",
                ms_ahora() - 180_000.0,
            ),
            texto: "Recuperado hace 3 min (mi.cdp) — 0 subtítulos".into(),
        };
        let es = aviso.texto(&ES);
        let en = aviso.texto(&EN);
        assert!(es.starts_with("Copia automática de hace 3 min"), "{es}");
        assert!(en.starts_with("Autosaved 3 min ago"), "{en}");
        assert!(en.contains("mi.cdp"), "{en}");
        assert!(!en.contains("hace"), "{en}");
    }

    #[test]
    fn la_ruta_es_un_recovery_json() {
        // La carpeta depende del sistema (rutas::carpeta_config); el nombre
        // es el contrato con el oráculo.
        let ruta = ruta();
        assert!(
            ruta.to_string_lossy().ends_with("recovery.json"),
            "{}",
            ruta.display()
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
