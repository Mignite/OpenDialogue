//! Caché de miniaturas de los proyectos recientes.
//!
//! El PNG se genera al guardar, con el frame que ya tiene el editor en memoria.
//! Aquí solo está la ruta y la codificación: no se abre ningún player.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Tamaño de la miniatura. El frame del editor es 1920×1080: guardarlo tal
/// cual costaría varios MB por proyecto en la caché.
pub const ANCHO: u32 = 160;
pub const ALTO: u32 = 90;

/// `%APPDATA%\RustyOpenDialogue\thumbs`, con el mismo respaldo que las demás
/// rutas de la app.
pub fn carpeta_cache() -> PathBuf {
    match std::env::var_os("APPDATA") {
        Some(appdata) => Path::new(&appdata).join("RustyOpenDialogue").join("thumbs"),
        None => std::env::temp_dir().join("RustyOpenDialogue-thumbs"),
    }
}

/// El hash no es criptográfico ni un contrato entre versiones de Rust: si
/// cambia, la miniatura vieja queda huérfana y se regenera al abrir.
fn hash_proyecto(proyecto: &Path) -> u64 {
    let clave = std::fs::canonicalize(proyecto).unwrap_or_else(|_| proyecto.to_path_buf());
    let mut hasher = DefaultHasher::new();
    clave.hash(&mut hasher);
    hasher.finish()
}

pub fn ruta_cache_en(base: &Path, proyecto: &Path) -> PathBuf {
    base.join(format!("{:016x}.png", hash_proyecto(proyecto)))
}

pub fn ruta_cache(proyecto: &Path) -> PathBuf {
    ruta_cache_en(&carpeta_cache(), proyecto)
}

/// Codifica el frame RGBA ya compuesto (video + subtítulos) del editor.
pub fn guardar_en(
    base: &Path,
    proyecto: &Path,
    frame: &[u8],
    ancho: u32,
    alto: u32,
) -> std::io::Result<()> {
    let original = image::RgbaImage::from_raw(ancho, alto, frame.to_vec()).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("el frame no mide {ancho}x{alto}×4 bytes"),
        )
    })?;
    let pequeno = image::imageops::resize(
        &original,
        ANCHO,
        ALTO,
        image::imageops::FilterType::Triangle,
    );
    std::fs::create_dir_all(base)?;
    image::DynamicImage::ImageRgba8(pequeno)
        .save(ruta_cache_en(base, proyecto))
        .map_err(std::io::Error::other)
}

pub fn guardar(proyecto: &Path, frame: &[u8], ancho: u32, alto: u32) -> std::io::Result<()> {
    guardar_en(&carpeta_cache(), proyecto, frame, ancho, alto)
}

/// `None` cuando el PNG no está: el menú dibuja el placeholder en ese caso.
pub fn leer(proyecto: &Path) -> Option<PathBuf> {
    let ruta = ruta_cache(proyecto);
    ruta.is_file().then_some(ruta)
}

#[cfg(test)]
mod tests {
    use image::GenericImageView;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CONTADOR: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir_de_test(nombre: &str) -> PathBuf {
        let n = CONTADOR.fetch_add(1, Ordering::Relaxed);
        let ruta =
            std::env::temp_dir().join(format!("rod-thumb-{}-{n}-{nombre}", std::process::id()));
        std::fs::create_dir_all(&ruta).expect("crea directorio de test");
        ruta
    }

    #[test]
    fn ruta_cache_es_determinista_y_png() {
        let a = super::ruta_cache(Path::new("/p/mi.cdp"));
        let b = super::ruta_cache(Path::new("/p/mi.cdp"));
        assert_eq!(a, b);
        assert_eq!(a.extension().unwrap(), "png");
    }

    #[test]
    fn leer_devuelve_none_si_falta() {
        assert!(super::leer(Path::new("/no/existe.cdp")).is_none());
    }

    #[test]
    fn guardar_escribe_un_png_de_160x90() {
        let dir = temp_dir_de_test("thumb");
        let cdp = dir.join("mi.cdp");
        let frame = vec![40u8; 1920 * 1080 * 4];
        super::guardar_en(&dir, &cdp, &frame, 1920, 1080).unwrap();
        let img = image::open(super::ruta_cache_en(&dir, &cdp)).unwrap();
        assert_eq!(img.dimensions(), (160, 90));
    }
}
