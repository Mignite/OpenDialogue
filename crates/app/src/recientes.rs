//! Índice de proyectos recientes y metadata derivada del `.cdp`.
//!
//! Solo `rod-app` toca el disco: `rod-core` se queda con la lógica pura del
//! formato de proyecto.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// Cuántas rutas caben en el menú. Las más viejas se descartan al registrar.
const TOPE: usize = 10;
const MS_POR_DIA: f64 = 86_400_000.0;

/// Una entrada ya resuelta, lista para dibujar. Nada de esto se persiste:
/// las líneas, los hablantes y la fecha se derivan del `.cdp` en cada arranque.
#[derive(Debug, Clone, PartialEq)]
pub struct Reciente {
    pub ruta: PathBuf,
    pub lineas: usize,
    pub hablantes: usize,
    pub fecha: String,
}

/// `%APPDATA%\RustyOpenDialogue\recientes.json`, con el mismo respaldo al
/// directorio temporal que `recovery_io::ruta()`.
pub fn ruta_indice() -> PathBuf {
    match std::env::var_os("APPDATA") {
        Some(appdata) => Path::new(&appdata)
            .join("RustyOpenDialogue")
            .join("recientes.json"),
        None => std::env::temp_dir().join("RustyOpenDialogue-recientes.json"),
    }
}

/// Un índice roto, vacío o de otra versión da una lista vacía: el menú tiene
/// que abrir siempre, igual que pasa con `presets_ass.json`.
pub fn leer_indice(ruta: &Path) -> Vec<PathBuf> {
    let Ok(texto) = std::fs::read_to_string(ruta) else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<PathBuf>>(&texto).unwrap_or_default()
}

/// Escritura atómica: un corte a media escritura no debe dejar el índice
/// truncado y perder los diez proyectos de golpe.
pub fn escribir_indice(ruta: &Path, rutas: &[PathBuf]) -> std::io::Result<()> {
    let texto = serde_json::to_string_pretty(rutas)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    let temporal = ruta.with_extension("tmp");
    std::fs::write(&temporal, texto)?;
    if let Err(error) = std::fs::rename(&temporal, ruta) {
        let _ = std::fs::remove_file(&temporal);
        return Err(error);
    }
    Ok(())
}

/// Pone `proyecto` delante, quita duplicados y recorta a [`TOPE`].
pub fn registrar(ruta_indice: &Path, proyecto: &Path) -> std::io::Result<()> {
    let mut rutas = leer_indice(ruta_indice);
    rutas.retain(|ruta| ruta != proyecto);
    rutas.insert(0, proyecto.to_path_buf());
    rutas.truncate(TOPE);
    escribir_indice(ruta_indice, &rutas)
}

/// Días completos entre dos instantes epoch-ms. Un reloj adelantado se toma
/// como «hoy» en vez de dar una fecha negativa.
pub fn fecha_relativa(mtime_ms: f64, ahora_ms: f64) -> String {
    let dias = ((ahora_ms - mtime_ms) / MS_POR_DIA).floor().max(0.0) as u64;
    match dias {
        0 => "hoy".to_owned(),
        1 => "ayer".to_owned(),
        n if n < 7 => format!("hace {n} días"),
        n => {
            let semanas = n / 7;
            if semanas == 1 {
                "hace 1 semana".to_owned()
            } else {
                format!("hace {semanas} semanas")
            }
        }
    }
}

/// Lee un `.cdp` y deriva lo que el menú muestra. Cualquier fallo (archivo
/// movido, JSON roto, sin permisos) hace que la entrada se omita.
pub fn metadata(ruta_cdp: &Path, ahora_ms: f64) -> Option<Reciente> {
    let texto = std::fs::read_to_string(ruta_cdp).ok()?;
    let proyecto = rod_core::desde_json(&texto).ok()?;
    let mtime_ms = std::fs::metadata(ruta_cdp)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs_f64()
        * 1000.0;
    Some(Reciente {
        ruta: ruta_cdp.to_path_buf(),
        lineas: proyecto.captions.len(),
        hablantes: proyecto.hablantes.len(),
        fecha: fecha_relativa(mtime_ms, ahora_ms),
    })
}

/// Justo después de guardar el `.cdp`: mete la ruta en el índice y refresca
/// su miniatura con el frame que el editor tiene en memoria en ese instante.
/// Imagen y `playhead` salen del mismo momento del guardado.
pub fn registrar_guardado(
    proyecto: &Path,
    ruta_indice: &Path,
    ruta_thumbs: &Path,
    frame: &[u8],
    ancho: u32,
    alto: u32,
) -> std::io::Result<()> {
    crate::thumbnail::guardar_en(ruta_thumbs, proyecto, frame, ancho, alto)?;
    registrar(ruta_indice, proyecto)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CONTADOR: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir_de_test(nombre: &str) -> PathBuf {
        let n = CONTADOR.fetch_add(1, Ordering::Relaxed);
        let ruta =
            std::env::temp_dir().join(format!("rod-recientes-{}-{n}-{nombre}", std::process::id()));
        std::fs::create_dir_all(&ruta).expect("crea directorio de test");
        ruta
    }

    fn texto_fixture() -> String {
        use rod_core::{Caption, Hablante, Proyecto};

        let proyecto = Proyecto {
            ruta_video: "clip.mp4".into(),
            hablantes: vec![
                Hablante {
                    id: "sp-1".into(),
                    nombre: "Ana".into(),
                    tecla: "1".into(),
                    color: "#3B82F6".into(),
                    preset_id: None,
                },
                Hablante {
                    id: "sp-2".into(),
                    nombre: "Beto".into(),
                    tecla: "2".into(),
                    color: "#EF4444".into(),
                    preset_id: None,
                },
            ],
            captions: vec![
                Caption {
                    id: "cap-1".into(),
                    inicio: 1.0,
                    fin: 2.0,
                    texto: "Hola".into(),
                    hablante_id: Some("sp-1".into()),
                },
                Caption {
                    id: "cap-2".into(),
                    inicio: 3.0,
                    fin: 4.0,
                    texto: "Adiós".into(),
                    hablante_id: Some("sp-2".into()),
                },
            ],
            playhead: 0.0,
        };
        rod_core::a_json(&proyecto).expect("serializa fixture")
    }

    #[test]
    fn indice_roto_da_lista_vacia() {
        let dir = temp_dir_de_test("indice-roto");
        let ruta = dir.join("recientes.json");
        std::fs::write(&ruta, "{ esto no es json").unwrap();
        assert!(super::leer_indice(&ruta).is_empty());
    }

    #[test]
    fn registrar_dedup_y_tope() {
        let dir = temp_dir_de_test("tope");
        let indice = dir.join("recientes.json");
        for n in 0..12 {
            super::registrar(&indice, &PathBuf::from(format!("/p/proyecto-{n}.cdp"))).unwrap();
        }
        super::registrar(&indice, &PathBuf::from("/p/proyecto-11.cdp")).unwrap();
        let rutas = super::leer_indice(&indice);
        assert_eq!(rutas.len(), 10);
        assert_eq!(rutas[0], PathBuf::from("/p/proyecto-11.cdp"));
        assert_eq!(
            rutas
                .iter()
                .filter(|r| *r == &PathBuf::from("/p/proyecto-11.cdp"))
                .count(),
            1
        );
    }

    #[test]
    fn guardar_registra_imagen_en_el_playhead() {
        let dir = temp_dir_de_test("guardar");
        let cdp = dir.join("mi.cdp");
        let indice = dir.join("recientes.json");
        let frame = vec![7u8; 1920 * 1080 * 4];
        super::registrar_guardado(&cdp, &indice, &dir.join("thumbs"), &frame, 1920, 1080).unwrap();
        assert_eq!(crate::recientes::leer_indice(&indice), vec![cdp.clone()]);
        assert_eq!(dir.join("thumbs").read_dir().unwrap().count(), 1);
    }

    #[test]
    fn metadata_omite_proyecto_inexistente() {
        assert!(super::metadata(Path::new("/no/existe.cdp"), 1_000.0).is_none());
    }

    #[test]
    fn metadata_cuenta_lineas_y_hablantes() {
        let dir = temp_dir_de_test("metadata");
        let cdp = dir.join("mi.cdp");
        std::fs::write(&cdp, texto_fixture()).unwrap();
        let r = super::metadata(&cdp, 0.0).unwrap();
        assert_eq!(r.lineas, 2);
        assert_eq!(r.hablantes, 2);
    }

    #[test]
    fn fecha_relativa_es_hoy_y_hace_tres_dias() {
        assert_eq!(super::fecha_relativa(1_000.0, 1_000.0), "hoy");
        assert_eq!(
            super::fecha_relativa(1_000.0, 1_000.0 + 3.0 * 86_400_000.0),
            "hace 3 días"
        );
    }
}
