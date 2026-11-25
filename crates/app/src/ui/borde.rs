//! Bordes redimensionables de la ventana sin decoraciones.
//!
//! `with_decorations(false)` quita `WS_SIZEBOX` y Windows deja de ofrecer
//! bordes al ratón. Igual que PhotoCraft (craft-win32), instalamos un
//! subclass del window procedure que responde `WM_NCHITTEST` con el borde
//! correspondiente; el propio Windows gestiona entonces el arrastre, el
//! snap y el maximizado, sin enviar comandos `InnerSize` desde egui.
//!
//! `unsafe` justificado: winit 0.30 no expone `WM_NCHITTEST` ni permite
//! sustituir el window procedure desde la API segura. Se aísla en un único
//! `extern "system"` que solo lee el rect de la ventana y delega el resto.

use egui::{Pos2, Rect};

/// Grosor de la zona sensible, en píxeles físicos; se actualiza cada frame.
#[cfg(target_os = "windows")]
static GROSOR_PX: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(8);
#[cfg(target_os = "windows")]
static WNDPROC_ANTERIOR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Borde {
    Izquierdo,
    Derecho,
    Superior,
    Inferior,
    SupIzquierdo,
    SupDerecho,
    InfIzquierdo,
    InfDerecho,
}

impl Borde {
    #[cfg(target_os = "windows")]
    fn codigo_nc(self) -> isize {
        match self {
            Self::Izquierdo => 10,    // HTLEFT
            Self::Derecho => 11,      // HTRIGHT
            Self::Superior => 12,     // HTTOP
            Self::SupIzquierdo => 13, // HTTOPLEFT
            Self::SupDerecho => 14,   // HTTOPRIGHT
            Self::Inferior => 15,     // HTBOTTOM
            Self::InfIzquierdo => 16, // HTBOTTOMLEFT
            Self::InfDerecho => 17,   // HTBOTTOMRIGHT
        }
    }
}

/// Hit-test puro: qué borde está bajo `punto` dentro de `ventana`.
/// El orden importa: las esquinas ganan a los lados.
fn borde_en(ventana: Rect, punto: Pos2, grosor: f32) -> Option<Borde> {
    if !ventana.contains(punto) || grosor <= 0.0 {
        return None;
    }
    let izquierda = punto.x - ventana.min.x < grosor;
    let derecha = ventana.max.x - punto.x < grosor;
    let arriba = punto.y - ventana.min.y < grosor;
    let abajo = ventana.max.y - punto.y < grosor;
    match (arriba, abajo, izquierda, derecha) {
        (true, _, true, _) => Some(Borde::SupIzquierdo),
        (true, _, _, true) => Some(Borde::SupDerecho),
        (_, true, true, _) => Some(Borde::InfIzquierdo),
        (_, true, _, true) => Some(Borde::InfDerecho),
        (true, _, _, _) => Some(Borde::Superior),
        (_, true, _, _) => Some(Borde::Inferior),
        (_, _, true, _) => Some(Borde::Izquierdo),
        (_, _, _, true) => Some(Borde::Derecho),
        _ => None,
    }
}

/// Instala el subclass una vez. En el resto de plataformas no hace nada:
/// el marco del sistema ya es redimensionable.
pub fn mostrar(ctx: &egui::Context, frame: &mut eframe::Frame) {
    #[cfg(target_os = "windows")]
    {
        use raw_window_handle::{HasWindowHandle as _, RawWindowHandle};
        use std::sync::atomic::Ordering;

        GROSOR_PX.store(
            (8.0 * ctx.pixels_per_point()).round() as i32,
            Ordering::Relaxed,
        );
        if WNDPROC_ANTERIOR.load(Ordering::Relaxed) == 0
            && let Ok(handle) = frame.window_handle()
            && let RawWindowHandle::Win32(win) = handle.as_raw()
        {
            unsafe {
                let hwnd = win.hwnd.get() as *mut std::ffi::c_void;
                win32::instalar(hwnd);
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (ctx, frame);
    }
}

#[cfg(target_os = "windows")]
mod win32 {
    use super::{GROSOR_PX, WNDPROC_ANTERIOR, borde_en};
    use egui::{Pos2, Rect, Vec2};
    use std::ffi::c_void;
    use std::sync::atomic::Ordering;

    type Hwnd = *mut c_void;
    type Lresult = isize;

    #[repr(C)]
    #[derive(Default)]
    struct RectWin {
        izquierda: i32,
        arriba: i32,
        derecha: i32,
        abajo: i32,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn SetWindowLongPtrW(hwnd: Hwnd, indice: i32, nuevo: usize) -> usize;
        fn CallWindowProcW(
            anterior: isize,
            hwnd: Hwnd,
            mensaje: u32,
            wparam: usize,
            lparam: isize,
        ) -> Lresult;
        fn GetWindowRect(hwnd: Hwnd, rect: *mut RectWin) -> i32;
    }

    /// Instala el subclass. El cast del puntero de función a `isize` es
    /// obligatorio para `SetWindowLongPtrW`; no hay alternativa segura en
    /// winit 0.30 y el procédure original se guarda para delegar.
    #[allow(clippy::fn_to_numeric_cast)]
    pub(super) unsafe fn instalar(hwnd: Hwnd) {
        let anterior = unsafe { SetWindowLongPtrW(hwnd, -4, wndproc as *const () as usize) };
        WNDPROC_ANTERIOR.store(anterior, Ordering::Relaxed);
    }

    unsafe extern "system" fn wndproc(
        hwnd: Hwnd,
        mensaje: u32,
        wparam: usize,
        lparam: isize,
    ) -> Lresult {
        const WM_NCHITTEST: u32 = 0x0084;
        if mensaje == WM_NCHITTEST {
            let x = (lparam as i32 & 0xFFFF) as i16 as i32;
            let y = ((lparam >> 16) as i32 & 0xFFFF) as i16 as i32;
            let mut nativa = RectWin::default();
            if unsafe { GetWindowRect(hwnd, &mut nativa) } != 0 {
                let rect = Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(
                        (nativa.derecha - nativa.izquierda) as f32,
                        (nativa.abajo - nativa.arriba) as f32,
                    ),
                );
                let punto = Pos2::new((x - nativa.izquierda) as f32, (y - nativa.arriba) as f32);
                let grosor = GROSOR_PX.load(Ordering::Relaxed) as f32;
                if let Some(borde) = borde_en(rect, punto, grosor) {
                    return borde.codigo_nc();
                }
            }
        }
        let anterior = WNDPROC_ANTERIOR.load(Ordering::Relaxed) as isize;
        unsafe { CallWindowProcW(anterior, hwnd, mensaje, wparam, lparam) }
    }
}

#[cfg(test)]
mod tests {
    use super::{Borde, borde_en};
    use egui::{Rect, pos2, vec2};

    fn ventana() -> Rect {
        Rect::from_min_size(pos2(100.0, 50.0), vec2(800.0, 600.0))
    }

    #[test]
    fn el_centro_no_es_borde() {
        assert_eq!(borde_en(ventana(), pos2(500.0, 350.0), 8.0), None);
    }

    #[test]
    fn cada_lado_devuelve_su_borde() {
        let v = ventana();
        assert_eq!(borde_en(v, pos2(101.0, 300.0), 8.0), Some(Borde::Izquierdo));
        assert_eq!(borde_en(v, pos2(898.0, 300.0), 8.0), Some(Borde::Derecho));
        assert_eq!(borde_en(v, pos2(500.0, 51.0), 8.0), Some(Borde::Superior));
        assert_eq!(borde_en(v, pos2(500.0, 648.0), 8.0), Some(Borde::Inferior));
    }

    #[test]
    fn las_esquinas_ganan_a_los_lados() {
        let v = ventana();
        assert_eq!(
            borde_en(v, pos2(101.0, 51.0), 8.0),
            Some(Borde::SupIzquierdo)
        );
        assert_eq!(borde_en(v, pos2(898.0, 51.0), 8.0), Some(Borde::SupDerecho));
        assert_eq!(
            borde_en(v, pos2(101.0, 648.0), 8.0),
            Some(Borde::InfIzquierdo)
        );
        assert_eq!(
            borde_en(v, pos2(898.0, 648.0), 8.0),
            Some(Borde::InfDerecho)
        );
    }
}
