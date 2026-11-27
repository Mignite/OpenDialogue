//! Envoltorio mínimo sobre libass: cargar un `.ass`, fijar el tamaño del frame y
//! obtener los bitmaps de alfa de un instante. La composición sobre el video y
//! la comparación con el oráculo se hacen fuera, sobre `Bitmap`.

mod composicion;
mod ffi;

pub use composicion::{Lienzo, Rect, caja};

use core::ffi::{c_char, c_int, c_void};
use std::ffi::CString;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Mutex;

/// Versión de libass con la que se enlazó, como `"0.17.5"`.
///
/// Se imprime en los tests de golden: la comparación de píxeles no vale entre
/// versiones de libass, así que el número tiene que quedar en la evidencia.
pub fn version() -> String {
    describir_version(unsafe { ffi::ass_library_version() }.cast_unsigned())
}

/// libass empaqueta la versión en BCD, no en bytes: `0x01705000` es `0.17.5`.
/// Los cinco dígitos hexaltos son `M`, `mm`, `pp` en decimal, y los tres bajos
/// son un extra de parche. Se comprueba con el propio ejemplo del `ass.h`
/// (`0x01300001` = 0.13.0 + extra 1). Un `major` de dos cifras rompería la
/// regla, pero eso queda a más de nueve versiones mayores de distancia.
fn describir_version(empaquetada: u32) -> String {
    let mayor = (empaquetada >> 28) & 0x0F;
    let menor = ((empaquetada >> 24) & 0x0F) * 10 + ((empaquetada >> 20) & 0x0F);
    let parche = ((empaquetada >> 16) & 0x0F) * 10 + ((empaquetada >> 12) & 0x0F);
    format!("{mayor}.{menor}.{parche}")
}

/// Deja que libass elija el proveedor de fuentes de la plataforma
/// (DirectWrite en Windows, fontconfig en Linux).
pub const PROVEEDOR_AUTODETECT: c_int = 1;

/// `ass_library_init` y el log de libass.
///
/// El log importa: ahí es donde libass dice qué proveedor de fuentes eligió, y
/// ese es el punto que se rompe en silencio (sale otra tipografía y la
/// comparación contra mpv falla por la fuente, no por el código).
pub struct Library {
    ptr: *mut ffi::ASS_Library,
    // En una caja para que la dirección no cambie si `Library` se mueve: el
    // callback de C se queda con ese puntero toda la vida de la biblioteca.
    log: Box<Mutex<Vec<String>>>,
}

impl Library {
    pub fn new() -> Self {
        let ptr = unsafe { ffi::ass_library_init() };
        assert!(!ptr.is_null(), "ass_library_init devolvió nulo");

        let log = Box::new(Mutex::new(Vec::new()));
        unsafe {
            ffi::ass_set_message_cb(
                ptr,
                Some(recibir_mensaje),
                &*log as *const Mutex<Vec<String>> as *mut c_void,
            );
        }

        Self { ptr, log }
    }

    /// Directorio extra de fuentes. No hace falta para las del sistema; sí para
    /// repartir tipografías con la app sin instalarlas.
    pub fn anadir_fuentes(&self, dir: &Path) {
        let Ok(dir) = CString::new(dir.to_string_lossy().as_bytes()) else {
            return;
        };
        unsafe { ffi::ass_set_fonts_dir(self.ptr, dir.as_ptr()) };
    }

    /// Copia de las líneas que libass ha escrito hasta ahora.
    pub fn log(&self) -> Vec<String> {
        self.log.lock().map(|log| log.clone()).unwrap_or_default()
    }
}

impl Default for Library {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe { ffi::ass_library_done(self.ptr) };
    }
}

/// Rasterizador. Vive pegado a su `Library` (el `PhantomData` lo garantiza: no
/// se puede soltar la biblioteca y seguir usando el renderer).
pub struct Renderer<'a> {
    ptr: *mut ffi::ASS_Renderer,
    bitmaps: Vec<Bitmap>,
    _library: PhantomData<&'a Library>,
}

impl<'a> Renderer<'a> {
    /// El tamaño del frame es también el sistema de coordenadas de los
    /// `\pos` del `.ass`, así que conviene que coincida con el del video.
    pub fn new(library: &'a Library, ancho: u32, alto: u32) -> Self {
        let ptr = unsafe { ffi::ass_renderer_init(library.ptr) };
        assert!(!ptr.is_null(), "ass_renderer_init devolvió nulo");

        let mut renderer = Self {
            ptr,
            bitmaps: Vec::new(),
            _library: PhantomData,
        };
        renderer.set_frame_size(ancho, alto);
        renderer
    }

    pub fn set_frame_size(&mut self, ancho: u32, alto: u32) {
        unsafe { configurar(self.ptr, ancho, alto) };
    }

    /// Márgenes del video dentro de la ventana. Con `usar = false` libass
    /// ignora los márgenes para colocar los eventos normales (lo que hace mpv
    /// con letterbox).
    pub fn set_margins(&mut self, arriba: i32, abajo: i32, izq: i32, der: i32, usar: bool) {
        unsafe {
            ffi::ass_set_margins(
                self.ptr,
                arriba as c_int,
                abajo as c_int,
                izq as c_int,
                der as c_int,
            );
            ffi::ass_set_use_margins(self.ptr, usar as c_int);
        }
    }

    /// Inicializa el proveedor de fuentes. Las siguientes llamadas a `log()`
    /// dicen cuál eligió.
    pub fn usar_fuentes(&mut self, proveedor: c_int) {
        unsafe { fuentes(self.ptr, proveedor) };
    }

    /// Rasteriza el instante `ms` y devuelve sus bitmaps. Son válidos hasta la
    /// próxima llamada a `render` (el `&mut self` lo hace cumplir).
    pub fn render(&mut self, track: &Track<'_>, ms: i64) -> &[Bitmap] {
        self.bitmaps = unsafe { recolectar(rasterizar(self.ptr, track.ptr, ms)) };
        &self.bitmaps
    }
}

impl Drop for Renderer<'_> {
    fn drop(&mut self) {
        unsafe { ffi::ass_renderer_done(self.ptr) };
    }
}

/// Un trozo de subtítulo ya rasterizado: una máscara de alfa y su color.
pub struct Bitmap {
    pub ancho: u32,
    pub alto: u32,
    /// Bytes por fila. La última fila puede ser más corta: ver `alpha()`.
    pub stride: usize,
    /// Color y alfa empaquetados tal como los da libass.
    pub color: u32,
    /// Esquina superior izquierda dentro del frame.
    pub x: i32,
    pub y: i32,
    alpha: *const u8,
    alpha_len: usize,
}

impl Bitmap {
    /// Cobertura de 0 (transparente) a 255 (opaco), por filas de `stride`
    /// bytes. La última fila solo tiene `ancho` bytes válidos: indexar
    /// `y * stride + x` sin recortar revienta con pánico en vez de leer basura.
    pub fn alpha(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.alpha, self.alpha_len) }
    }

    /// Bitmap artificial para los tests de composición, que no necesitan libass
    /// ni fuentes. La máscara se filtra a propósito: así el test no tiene que
    /// recordar el recorte de la última fila.
    #[cfg(test)]
    pub(crate) fn de_prueba(
        ancho: u32,
        alto: u32,
        x: i32,
        y: i32,
        cobertura: u8,
        color: u32,
    ) -> Self {
        let mascara = vec![cobertura; ancho as usize * alto as usize];
        Self::de_prueba_mascara(ancho, alto, x, y, &mascara, color)
    }

    /// Igual, con una máscara dada fila a fila (`ancho * alto` bytes, sin
    /// relleno). Sirve para comprobar que cada columna de la máscara cae en el
    /// píxel correcto y no solo que se pinta algo.
    #[cfg(test)]
    pub(crate) fn de_prueba_mascara(
        ancho: u32,
        alto: u32,
        x: i32,
        y: i32,
        mascara: &[u8],
        color: u32,
    ) -> Self {
        let stride = ancho as usize;
        let alpha_len = stride * (alto.max(1) as usize - 1) + ancho as usize;
        assert!(
            mascara.len() >= alpha_len,
            "la máscara no cubre {ancho}x{alto}"
        );
        let alpha = Box::leak(mascara.to_vec().into_boxed_slice()).as_ptr();
        Self {
            ancho,
            alto,
            stride,
            color,
            x,
            y,
            alpha,
            alpha_len,
        }
    }
}

/// Un `.ass` ya parseado.
pub struct Track<'a> {
    ptr: *mut ffi::ASS_Track,
    _library: PhantomData<&'a Library>,
}

impl<'a> Track<'a> {
    pub fn parse(library: &'a Library, texto: &str) -> Self {
        // libass pide un buffer mutable; no lo modifica, pero la firma es C.
        let mut buf = texto.as_bytes().to_vec();
        let ptr = unsafe {
            ffi::ass_read_memory(
                library.ptr,
                buf.as_mut_ptr().cast(),
                buf.len(),
                c"UTF-8".as_ptr(),
            )
        };
        assert!(!ptr.is_null(), "ass_read_memory devolvió nulo");

        Self {
            ptr,
            _library: PhantomData,
        }
    }
}

impl Drop for Track<'_> {
    fn drop(&mut self) {
        unsafe { ffi::ass_free_track(self.ptr) };
    }
}

/// Un `.ass` listo para producir bitmaps: biblioteca, rasterizador y track en un
/// solo tipo sin lifetimes, para que la app lo guarde en su estado.
///
/// Es la misma cosa que `Library` + `Renderer` + `Track` armados a mano, pero
/// con el orden de liberación resuelto por dentro (track, rasterizador,
/// biblioteca).
pub struct Subtitulos {
    library: Library,
    renderer: *mut ffi::ASS_Renderer,
    track: *mut ffi::ASS_Track,
    bitmaps: Vec<Bitmap>,
}

impl Subtitulos {
    /// `ancho` y `alto` son los del video: son el sistema de coordenadas de los
    /// `\pos` del `.ass` y el tamaño contra el que libass calcula márgenes.
    pub fn nuevo(ancho: u32, alto: u32, texto: &str) -> Self {
        let library = Library::new();

        let renderer = unsafe { ffi::ass_renderer_init(library.ptr) };
        assert!(!renderer.is_null(), "ass_renderer_init devolvió nulo");

        let mut buf = texto.as_bytes().to_vec();
        let track = unsafe {
            ffi::ass_read_memory(
                library.ptr,
                buf.as_mut_ptr().cast(),
                buf.len(),
                c"UTF-8".as_ptr(),
            )
        };
        assert!(!track.is_null(), "ass_read_memory devolvió nulo");

        let mut subtitulos = Self {
            library,
            renderer,
            track,
            bitmaps: Vec::new(),
        };
        subtitulos.configurar(ancho, alto);
        subtitulos.fuentes(PROVEEDOR_AUTODETECT);
        subtitulos
    }

    /// Al cambiar el tamaño del video (o al entrar a pantalla completa) basta
    /// con este tamaño: las coordenadas del `.ass` se reescalan solas.
    pub fn configurar(&mut self, ancho: u32, alto: u32) {
        unsafe { configurar(self.renderer, ancho, alto) };
    }

    /// Inicializa el proveedor de fuentes que se pida.
    pub fn fuentes(&mut self, proveedor: c_int) {
        unsafe { fuentes(self.renderer, proveedor) };
    }

    /// Cambia el `.ass` sin rehacer biblioteca ni rasterizador. Es la ruta de
    /// la edición: el texto del proyecto se reconstruye y se recarga, y el
    /// siguiente `render` ya rasteriza lo nuevo.
    pub fn cargar(&mut self, texto: &str) {
        let mut buf = texto.as_bytes().to_vec();
        let track = unsafe {
            ffi::ass_read_memory(
                self.library.ptr,
                buf.as_mut_ptr().cast(),
                buf.len(),
                c"UTF-8".as_ptr(),
            )
        };
        assert!(!track.is_null(), "ass_read_memory devolvió nulo");

        unsafe { ffi::ass_free_track(self.track) };
        self.track = track;
        // Los bitmaps apuntan al track viejo; con `&mut self` nadie los tiene
        // prestados, pero el vector quedaría con punteros muertos.
        self.bitmaps.clear();
    }

    pub fn render(&mut self, ms: i64) -> &[Bitmap] {
        self.bitmaps = unsafe { recolectar(rasterizar(self.renderer, self.track, ms)) };
        &self.bitmaps
    }

    pub fn log(&self) -> Vec<String> {
        self.library.log()
    }
}

impl Drop for Subtitulos {
    fn drop(&mut self) {
        unsafe {
            ffi::ass_free_track(self.track);
            ffi::ass_renderer_done(self.renderer);
        }
    }
}

/// Tamaño del frame y del "storage". Sin el storage, `\be`, `\blur` y los
/// transforms 3D se calculan con un aspecto inventado.
unsafe fn configurar(renderer: *mut ffi::ASS_Renderer, ancho: u32, alto: u32) {
    unsafe {
        ffi::ass_set_frame_size(renderer, ancho as c_int, alto as c_int);
        ffi::ass_set_storage_size(renderer, ancho as c_int, alto as c_int);
        ffi::ass_set_pixel_aspect(renderer, 1.0);
    }
}

unsafe fn fuentes(renderer: *mut ffi::ASS_Renderer, proveedor: c_int) {
    let familia = c"Arial";
    unsafe {
        ffi::ass_set_fonts(
            renderer,
            std::ptr::null(), // fuente por defecto: la de la familia
            familia.as_ptr(),
            proveedor,
            std::ptr::null(), // config: solo lo usa fontconfig
            1,
        );
    }
}

unsafe fn rasterizar(
    renderer: *mut ffi::ASS_Renderer,
    track: *mut ffi::ASS_Track,
    ms: i64,
) -> *mut ffi::ASS_Image {
    let mut cambio = 0;
    unsafe { ffi::ass_render_frame(renderer, track, ms, &mut cambio) }
}

/// Recorre la lista enlazada que devuelve `ass_render_frame`. No se puede
/// prestar: los punteros valen hasta el siguiente render, así que `Bitmap` los
/// copia y recorta la última fila.
unsafe fn recolectar(head: *mut ffi::ASS_Image) -> Vec<Bitmap> {
    let mut bitmaps = Vec::new();
    let mut nodo = head;

    while !nodo.is_null() {
        let imagen = unsafe { &*nodo };

        // La última fila no está rellena hasta `stride`: el tamaño garantizado
        // es `stride * (h - 1) + w`. Recortar aquí convierte una lectura fuera
        // de rango en un índice fuera de rango (pánico).
        let filas = imagen.h.max(1) as usize;
        let alpha_len = imagen.stride as usize * (filas - 1) + imagen.w as usize;

        bitmaps.push(Bitmap {
            ancho: imagen.w as u32,
            alto: imagen.h as u32,
            stride: imagen.stride as usize,
            color: imagen.color,
            x: imagen.dst_x,
            y: imagen.dst_y,
            alpha: imagen.bitmap,
            alpha_len,
        });

        nodo = imagen.next;
    }

    bitmaps
}

unsafe extern "C" fn recibir_mensaje(
    _nivel: c_int,
    fmt: *const c_char,
    args: ffi::va_list,
    data: *mut c_void,
) {
    let texto = unsafe { ffi::formatear(fmt, args) };
    if texto.is_empty() {
        return;
    }

    let log = unsafe { &*(data as *const Mutex<Vec<String>>) };
    if let Ok(mut log) = log.lock() {
        log.push(texto);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_version_se_lee_en_bcd() {
        // El valor que trae el port de vcpkg: libass 0.17.5.
        assert_eq!(describir_version(0x0170_5000), "0.17.5");
        // El ejemplo que cita el propio `ass.h` al hablar de 0.13.0.
        assert_eq!(describir_version(0x0130_0001), "0.13.0");
        assert_eq!(describir_version(0x0160_0000), "0.16.0");
    }

    #[test]
    fn la_version_enlazada_se_puede_leer() {
        let version = version();
        assert!(
            version.starts_with("0."),
            "libass reportó una versión inesperada: {version}"
        );
    }

    #[test]
    fn cargar_cambia_el_track_renderizado() {
        // Sin eventos: no hay nada que rasterizar.
        let vacio = "[Script Info]\nScriptType: v4.00+\n\n[V4+ Styles]\n\
                     Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n\
                     Style: Default,Arial,48,&H00FFFFFF,&H00FFFFFF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n\n\
                     [Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";
        let con_texto = format!("{vacio}Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hola");

        let mut subtitulos = Subtitulos::nuevo(1920, 1080, vacio);
        assert!(
            subtitulos.render(1000).is_empty(),
            "un .ass sin eventos no puede producir bitmaps"
        );

        subtitulos.cargar(&con_texto);
        assert!(
            !subtitulos.render(1000).is_empty(),
            "tras cargar un evento debe aparecer texto"
        );
    }
}
