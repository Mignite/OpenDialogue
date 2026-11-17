//! Declaraciones crudas de libass (`ass.h` de vcpkg 0.17.5), escritas a mano:
//! solo lo que usamos. Sin bindgen.
//!
//! En x64 `va_list` es un puntero, así que el callback de log se puede declarar
//! sin `c_variadic` (que sigue siendo inestable).

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_void};

/// En x64 (MSVC y System V) `va_list` es `char *`.
pub type va_list = *mut c_void;

#[repr(C)]
pub struct ASS_Library {
    _privado: [u8; 0],
}

#[repr(C)]
pub struct ASS_Renderer {
    _privado: [u8; 0],
}

#[repr(C)]
pub struct ASS_Track {
    _privado: [u8; 0],
}

/// Nodo de la lista que devuelve `ass_render_frame`.
#[repr(C)]
pub struct ASS_Image {
    pub w: c_int,
    pub h: c_int,
    pub stride: c_int,
    /// Alfa de 1 byte por píxel. La última fila NO está rellena hasta `stride`:
    /// el tamaño garantizado es `stride * (h - 1) + w`.
    pub bitmap: *mut u8,
    /// Color y alfa del bitmap en formato RGBA empaquetado.
    pub color: u32,
    pub dst_x: c_int,
    pub dst_y: c_int,
    pub next: *mut ASS_Image,
}

#[cfg_attr(windows, link(name = "legacy_stdio_definitions"))]
#[cfg_attr(unix, link(name = "c"))]
unsafe extern "C" {
    /// Del CRT: solo se usa para formatear el `va_list` del callback de log.
    /// En Windows el stub está en `legacy_stdio_definitions.lib`; en Unix
    /// pertenece a la libc.
    fn vsnprintf(buf: *mut c_char, size: usize, format: *const c_char, args: va_list) -> c_int;
}

unsafe extern "C" {
    /// Versión empaquetada como `0xMMmmpp`.
    pub fn ass_library_version() -> c_int;
    pub fn ass_library_init() -> *mut ASS_Library;
    pub fn ass_library_done(privado: *mut ASS_Library);
    pub fn ass_set_fonts_dir(privado: *mut ASS_Library, dir: *const c_char);
    pub fn ass_set_message_cb(
        privado: *mut ASS_Library,
        cb: Option<unsafe extern "C" fn(c_int, *const c_char, va_list, *mut c_void)>,
        data: *mut c_void,
    );

    pub fn ass_renderer_init(privado: *mut ASS_Library) -> *mut ASS_Renderer;
    pub fn ass_renderer_done(privado: *mut ASS_Renderer);
    pub fn ass_set_frame_size(privado: *mut ASS_Renderer, w: c_int, h: c_int);
    pub fn ass_set_storage_size(privado: *mut ASS_Renderer, w: c_int, h: c_int);
    pub fn ass_set_pixel_aspect(privado: *mut ASS_Renderer, par: f64);
    pub fn ass_set_margins(
        privado: *mut ASS_Renderer,
        arriba: c_int,
        abajo: c_int,
        izq: c_int,
        der: c_int,
    );
    pub fn ass_set_use_margins(privado: *mut ASS_Renderer, usar: c_int);
    pub fn ass_set_fonts(
        privado: *mut ASS_Renderer,
        fuente_por_defecto: *const c_char,
        familia_por_defecto: *const c_char,
        proveedor: c_int,
        config: *const c_char,
        actualizar: c_int,
    );

    pub fn ass_read_memory(
        library: *mut ASS_Library,
        buf: *mut c_char,
        tamano: usize,
        codepage: *const c_char,
    ) -> *mut ASS_Track;
    pub fn ass_free_track(track: *mut ASS_Track);

    pub fn ass_render_frame(
        privado: *mut ASS_Renderer,
        track: *mut ASS_Track,
        ahora_ms: i64,
        cambio_detectado: *mut c_int,
    ) -> *mut ASS_Image;
}

/// Formatea un `va_list` con el `vsnprintf` del CRT. Se usa solo desde el
/// callback de log.
pub unsafe fn formatear(fmt: *const c_char, args: va_list) -> String {
    let mut buf = [0u8; 1024];
    let n = unsafe { vsnprintf(buf.as_mut_ptr().cast(), buf.len(), fmt, args) };
    if n <= 0 {
        return String::new();
    }
    let n = (n as usize).min(buf.len() - 1);
    String::from_utf8_lossy(&buf[..n]).trim_end().to_string()
}
