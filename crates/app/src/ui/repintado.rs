//! Política de repintado: solo se piden frames cuando hay algo que ver.

use std::time::Duration;

/// Cuándo debe la app pedir otro frame.
///
/// - Reproduciendo: mpv entrega frames, hace falta cada frame.
/// - Buscando: se sondea `seeking` hasta que termina.
/// - Con cambios sin guardar: el autosave vive en `Tick` y necesita un
///   repintado periódico; en reposo limpio, la entrada despierta la app.
pub fn politica(
    reproduciendo: bool,
    sucio: bool,
    buscando: bool,
    scrubbing: bool,
) -> Option<Duration> {
    if reproduciendo || scrubbing {
        Some(Duration::ZERO)
    } else if buscando {
        Some(Duration::from_millis(100))
    } else if sucio {
        Some(Duration::from_secs(1))
    } else {
        None
    }
}
