//! Recuperación ante crash: la copia automática del proyecto y las decisiones
//! que la rodean.
//!
//! Portado de `src/utils/recovery.ts`. Igual que allí, aquí solo hay datos y
//! funciones puras: el disco lo toca `rod-app` (`recovery_io`), que es quien
//! sabe dónde vive el archivo. El autosave es un seguro, no un backup: guardar
//! con Ctrl+S sigue siendo lo que decide la versión buena.
//!
//! El recovery *envuelve* el proyecto con la ruta de origen en vez de
//! reemplazarlo: al recuperar, el usuario ve de qué proyecto se trata.

use serde::{Deserialize, Serialize};

use crate::Proyecto;

/// Un recovery más viejo que esto se considera muerto: si el usuario no abrió
/// la app en 7 días, la copia automática ya no interesa (y ocupa espacio).
pub const VIGENTE_MS: f64 = 7.0 * 24.0 * 60.0 * 60.0 * 1000.0;

/// Cada cuánto se escribe la copia automática mientras haya cambios. Es un
/// seguro ante crash, no un backup: 15 s acotan la pérdida sin escribir en
/// cada pulsación.
pub const INTERVALO_AUTOSAVE_MS: f64 = 15_000.0;

/// La copia automática. `version` permite descartar archivos de otra versión de
/// la app sin romper el arranque, y las claves son las del oráculo
/// (`rutaProyecto` en camelCase) para que un recovery escrito por la app Tauri
/// se pueda leer aquí y al revés.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recovery {
    pub version: u32,
    #[serde(rename = "rutaProyecto")]
    pub ruta_proyecto: String,
    /// Fecha de escritura en epoch ms (como `Date.now()`).
    pub escrito: f64,
    pub proyecto: Proyecto,
}

/// Decide si toca escribir el recovery ahora.
///
/// `sucio` = hay cambios sin guardar, `n_captions` = tamaño del proyecto,
/// `ultima_escritura` = epoch ms de la última escritura (`None` = nunca).
/// Un proyecto vacío no se escribe: no hay nada que recuperar.
pub fn debe_guardar(
    sucio: bool,
    n_captions: usize,
    ultima_escritura: Option<f64>,
    ahora: f64,
) -> bool {
    if !sucio {
        return false;
    }
    if n_captions == 0 {
        return false;
    }
    match ultima_escritura {
        None => true,
        Some(ultima) => ahora - ultima >= INTERVALO_AUTOSAVE_MS,
    }
}

pub fn construir(proyecto: Proyecto, ruta_proyecto: impl Into<String>, escrito: f64) -> Recovery {
    Recovery {
        version: 1,
        ruta_proyecto: ruta_proyecto.into(),
        escrito,
        proyecto,
    }
}

pub fn a_json(recovery: &Recovery) -> Result<String, serde_json::Error> {
    // Compacto, como el `JSON.stringify` del oráculo: es un archivo de trabajo
    // que se reescribe cada 15 s, no un documento para leer a mano.
    serde_json::to_string(recovery)
}

/// Lee y valida un recovery. `None` ante cualquier cosa que no sea un recovery
/// válido y vigente: un archivo corrupto NUNCA debe impedir abrir la app.
///
/// `corte` (epoch ms) descarta lo más viejo: si el proyecto ya se guardó
/// después de ese instante, el recovery no aporta nada. `ahora` entra como
/// parámetro (y no se lee el reloj aquí) para que la vigencia sea testeable.
///
/// La validación sigue al oráculo campo por campo en vez de confiar en
/// `serde`: `{ "version": 1, "rutaProyecto": "/a", "proyecto": {} }` tiene la
/// forma correcta y aun así no es un recovery, y un error de `serde` no
/// distingue «archivo de otra versión» de «archivo roto».
pub fn leer(texto: Option<&str>, corte: f64, ahora: f64) -> Option<Recovery> {
    let texto = texto.filter(|texto| !texto.is_empty())?;
    let data: serde_json::Value = serde_json::from_str(texto).ok()?;
    let objeto = data.as_object()?;
    if objeto.get("version")?.as_u64()? != 1 {
        return None;
    }
    objeto.get("rutaProyecto")?.as_str()?;
    let escrito = objeto.get("escrito")?.as_f64()?;
    if !escrito.is_finite() {
        return None;
    }
    let proyecto = objeto.get("proyecto")?.as_object()?;
    if !proyecto.get("captions")?.is_array() || !proyecto.get("hablantes")?.is_array() {
        return None;
    }
    if escrito < corte {
        return None;
    }
    if escrito < ahora - VIGENTE_MS {
        return None;
    }
    serde_json::from_value(data).ok()
}

/// Texto para el aviso de recuperación. El nombre del proyecto se usa tal cual
/// venga: si no hay ruta guardada, es un proyecto nuevo y solo queda este texto
/// para que el usuario lo reconozca.
///
/// Desviación del oráculo: allí la fecha se imprime con `toLocaleString()`;
/// aquí se imprime la antigüedad («hace 3 min») porque Rust no trae zona
/// horaria ni locale, y la antigüedad es justo lo que decide si vale la pena
/// recuperar. Un recovery con fecha futura (reloj movido) cae en «hace un
/// momento» en vez de imprimir un número negativo.
pub fn formatear(recovery: &Recovery, nombre_proyecto: &str, ahora: f64) -> String {
    let cuando = hace(recovery.escrito, ahora);
    let n = recovery.proyecto.captions.len();
    let subtitulos = if n == 1 {
        "1 subtítulo".to_owned()
    } else {
        format!("{n} subtítulos")
    };
    let donde = if recovery.ruta_proyecto.is_empty() {
        " (sin guardar)".to_owned()
    } else if nombre_proyecto.is_empty() {
        format!(" ({})", recovery.ruta_proyecto)
    } else {
        format!(" ({nombre_proyecto})")
    };
    format!("Recuperado {cuando}{donde} — {subtitulos}")
}

fn hace(escrito: f64, ahora: f64) -> String {
    let ms = ahora - escrito;
    // Un instante no finito (reloj raro) o reciente cae en «hace un momento»:
    // es lo que el usuario puede entender sin números negativos.
    if !ms.is_finite() || ms < 60_000.0 {
        return "hace un momento".to_owned();
    }
    let minutos = (ms / 60_000.0).floor();
    if minutos < 60.0 {
        return format!("hace {minutos} min");
    }
    let horas = (minutos / 60.0).floor();
    if horas < 24.0 {
        return format!("hace {horas} h");
    }
    let dias = (horas / 24.0).floor();
    format!("hace {dias} días")
}

#[cfg(test)]
mod tests {
    use super::{
        INTERVALO_AUTOSAVE_MS, Recovery, VIGENTE_MS, a_json, construir, debe_guardar, formatear,
        leer,
    };
    use crate::{Caption, Hablante, Proyecto};

    fn proyecto() -> Proyecto {
        Proyecto {
            ruta_video: "/videos/peli.mkv".into(),
            hablantes: vec![Hablante {
                id: "h1".into(),
                nombre: "Ana".into(),
                tecla: "1".into(),
                color: "#E85D4E".into(),
                preset_id: None,
            }],
            captions: vec![Caption {
                id: "c1".into(),
                inicio: 0.0,
                fin: 3.0,
                texto: "hola".into(),
                hablante_id: Some("h1".into()),
            }],
            playhead: 12.5,
        }
    }

    #[test]
    fn construir_envuelve_el_proyecto_con_su_ruta_y_el_instante() {
        let recovery = construir(proyecto(), "/proyectos/mi.cdp", 1000.0);
        assert_eq!(recovery.version, 1);
        assert_eq!(recovery.ruta_proyecto, "/proyectos/mi.cdp");
        assert_eq!(recovery.escrito, 1000.0);
        assert_eq!(recovery.proyecto, proyecto());
    }

    #[test]
    fn tolera_una_ruta_de_proyecto_vacia() {
        let recovery = construir(proyecto(), "", 1000.0);
        assert_eq!(recovery.ruta_proyecto, "");
    }

    #[test]
    fn sin_texto_no_hay_recovery() {
        assert_eq!(leer(None, 0.0, 2000.0), None);
        assert_eq!(leer(Some(""), 0.0, 2000.0), None);
    }

    #[test]
    fn un_json_corrupto_no_es_un_recovery() {
        assert_eq!(leer(Some("{no es json"), 0.0, 2000.0), None);
    }

    #[test]
    fn si_falta_cualquier_campo_no_hay_recovery() {
        assert_eq!(leer(Some(r#"{"version": 1}"#), 0.0, 2000.0), None);
        assert_eq!(
            leer(
                Some(r#"{"version": 1, "rutaProyecto": "/a", "proyecto": {}}"#),
                0.0,
                2000.0
            ),
            None
        );
    }

    #[test]
    fn una_version_desconocida_no_es_un_recovery() {
        let texto = a_json(&construir(proyecto(), "/a.cdp", 1.0)).expect("serializa");
        let malo = texto.replace("\"version\":1", "\"version\":99");
        assert_eq!(leer(Some(&malo), 0.0, 2000.0), None);
    }

    #[test]
    fn acepta_un_recovery_bien_formado() {
        let texto = a_json(&construir(proyecto(), "/a.cdp", 2000.0)).expect("serializa");
        let recovery = leer(Some(&texto), 0.0, 2000.0).expect("es válido");
        assert_eq!(recovery.ruta_proyecto, "/a.cdp");
        assert_eq!(recovery.proyecto.captions.len(), 1);
    }

    #[test]
    fn descarta_un_recovery_mas_viejo_que_el_corte() {
        let texto = a_json(&construir(proyecto(), "/a.cdp", 1000.0)).expect("serializa");
        assert_eq!(leer(Some(&texto), 2000.0, 2000.0), None);
    }

    #[test]
    fn acepta_un_recovery_dentro_de_la_ventana_de_vigencia() {
        let ahora = 10_000_000.0;
        let texto = a_json(&construir(proyecto(), "/a.cdp", ahora)).expect("serializa");
        assert!(leer(Some(&texto), ahora - 1000.0, ahora).is_some());
    }

    #[test]
    fn descarta_un_recovery_mas_viejo_que_la_ventana_de_vigencia() {
        let ahora = 10_000_000.0;
        let viejo = ahora - VIGENTE_MS - 1000.0;
        let texto = a_json(&construir(proyecto(), "/a.cdp", viejo)).expect("serializa");
        assert_eq!(leer(Some(&texto), 0.0, ahora), None);
    }

    #[test]
    fn el_json_usa_las_claves_del_oraculo() {
        let texto = a_json(&construir(proyecto(), "/a.cdp", 1.0)).expect("serializa");
        assert!(texto.starts_with("{\"version\":1,\"rutaProyecto\":\"/a.cdp\""));
        // snake_case sería un archivo que la app Tauri no sabe leer.
        assert!(!texto.contains("ruta_proyecto"));
        // Y la ida y vuelta conserva el proyecto entero.
        let vuelta: Recovery = serde_json::from_str(&texto).expect("deserializa");
        assert_eq!(vuelta.proyecto, proyecto());
    }

    #[test]
    fn no_se_escribe_sin_cambios_ni_con_el_proyecto_vacio() {
        let ahora = 100_000.0;
        assert!(!debe_guardar(false, 3, None, ahora), "limpio");
        assert!(!debe_guardar(true, 0, None, ahora), "vacío");
        assert!(debe_guardar(true, 3, None, ahora), "sucio y nunca escrito");
    }

    #[test]
    fn entre_escrituras_pasa_el_intervalo() {
        let ahora = 100_000.0;
        let casi = ahora - INTERVALO_AUTOSAVE_MS + 1.0;
        assert!(!debe_guardar(true, 3, Some(casi), ahora));
        let justo = ahora - INTERVALO_AUTOSAVE_MS;
        assert!(debe_guardar(true, 3, Some(justo), ahora));
    }

    #[test]
    fn formatear_incluye_el_nombre_y_el_numero_de_subtitulos() {
        let ahora = 100_000_000.0;
        let recovery = construir(proyecto(), "/carpeta/mi-video.cdp", ahora);
        let texto = formatear(&recovery, "mi-video", ahora);
        assert!(texto.contains("mi-video"), "{texto}");
        assert!(texto.contains('1'), "{texto}");
        assert!(texto.starts_with("Recuperado hace un momento"), "{texto}");
    }

    #[test]
    fn formatear_no_revienta_con_una_ruta_vacia() {
        let ahora = 100_000_000.0;
        let recovery = construir(proyecto(), "", ahora);
        let texto = formatear(&recovery, "", ahora);
        assert!(texto.contains("(sin guardar)"), "{texto}");
        // Sin nombre propio, se usa la ruta guardada.
        let con_ruta = construir(proyecto(), "/carpeta/x.cdp", ahora);
        assert!(formatear(&con_ruta, "", ahora).contains("/carpeta/x.cdp"));
    }

    #[test]
    fn la_antiguedad_se_dice_en_unidades_legibles() {
        let ahora = 10.0 * 24.0 * 60.0 * 60.0 * 1000.0;
        let con = |hace_ms: f64| {
            let recovery = construir(proyecto(), "", ahora - hace_ms);
            formatear(&recovery, "", ahora)
        };
        assert!(con(30_000.0).contains("hace un momento"));
        assert!(con(5.0 * 60_000.0).contains("hace 5 min"));
        assert!(con(3.0 * 3_600_000.0).contains("hace 3 h"));
        assert!(con(2.0 * 86_400_000.0).contains("hace 2 días"));
        // Un reloj que se movió hacia adelante no imprime negativos.
        assert!(con(-60_000.0).contains("hace un momento"));
    }
}
