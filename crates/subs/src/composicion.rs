//! Composición en CPU de los bitmaps de libass sobre un lienzo RGB.
//!
//! Existe como módulo de la biblioteca y no como código de un ejemplo porque
//! hay tres consumidores que tienen que componer **igual**: el PNG de depuración,
//! el oráculo del slice 2 (que compara contra mpv) y, más adelante, cualquier
//! ruta sin GPU. Si la fórmula viviera en un ejemplo, el test compararía contra
//! una copia y no contra el código que se usa.

use crate::Bitmap;

/// Región rectangular en píxeles, con origen en la esquina superior izquierda.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub ancho: u32,
    pub alto: u32,
}

/// Imagen RGB de 8 bits por canal, por filas (`ancho * alto * 3` bytes).
pub struct Lienzo {
    ancho: u32,
    alto: u32,
    pixeles: Vec<u8>,
}

impl Lienzo {
    pub fn nuevo(ancho: u32, alto: u32, fondo: [u8; 3]) -> Self {
        let mut pixeles = Vec::new();
        pixeles.reserve_exact(ancho as usize * alto as usize * 3);
        for _ in 0..ancho as usize * alto as usize {
            pixeles.extend_from_slice(&fondo);
        }
        Self {
            ancho,
            alto,
            pixeles,
        }
    }

    /// Envuelve un buffer ya existente (por ejemplo, el frame de fondo cargado
    /// de un PNG). Devuelve `None` si el tamaño no cuadra con `ancho * alto * 3`.
    pub fn desde_rgb(ancho: u32, alto: u32, pixeles: Vec<u8>) -> Option<Self> {
        (pixeles.len() == ancho as usize * alto as usize * 3).then_some(Self {
            ancho,
            alto,
            pixeles,
        })
    }

    pub fn ancho(&self) -> u32 {
        self.ancho
    }

    pub fn alto(&self) -> u32 {
        self.alto
    }

    pub fn pixeles(&self) -> &[u8] {
        &self.pixeles
    }

    pub fn pixeles_mut(&mut self) -> &mut [u8] {
        &mut self.pixeles
    }

    /// Compone los bitmaps en sus propias coordenadas (las del frame).
    pub fn componer(&mut self, bitmaps: &[Bitmap]) {
        self.componer_desplazado(bitmaps, 0, 0);
    }

    /// Igual que `componer`, con un desplazamiento sumado a cada bitmap. Sirve
    /// para componer sobre un recorte: el lienzo empieza en `-dx, -dy` del frame.
    ///
    /// Las partes que caen fuera del lienzo se recortan; un bitmap entero fuera
    /// no hace nada. Eso importa: `dst_x`/`dst_y` pueden ser negativos con un
    /// `\move` que entra desde fuera, y sin recorte el índice de destino
    /// desbordaría.
    pub fn componer_desplazado(&mut self, bitmaps: &[Bitmap], dx: i32, dy: i32) {
        let ancho = i64::from(self.ancho);
        let alto = i64::from(self.alto);

        for bitmap in bitmaps {
            let x0 = i64::from(bitmap.x) + i64::from(dx);
            let y0 = i64::from(bitmap.y) + i64::from(dy);
            let x1 = x0 + i64::from(bitmap.ancho);
            let y1 = y0 + i64::from(bitmap.alto);

            let x_ini = x0.max(0);
            let y_ini = y0.max(0);
            let x_fin = x1.min(ancho);
            let y_fin = y1.min(alto);
            if x_fin <= x_ini || y_fin <= y_ini {
                continue;
            }

            // El alfa de ASS está invertido: 0 = opaco, 255 = invisible.
            let opacidad = 255 - (bitmap.color & 0xFF);
            // 0xRRGGBBAA: los canales van en los bytes altos.
            let capa = [
                (bitmap.color >> 24) as u8,
                ((bitmap.color >> 16) & 0xFF) as u8,
                ((bitmap.color >> 8) & 0xFF) as u8,
            ];
            let mascara = bitmap.alpha();

            for y in y_ini..y_fin {
                let fila = (y - y0) as usize;
                let base = (y as usize) * (self.ancho as usize) * 3;

                for x in x_ini..x_fin {
                    let columna = (x - x0) as usize;
                    let cobertura = u32::from(mascara[fila * bitmap.stride + columna]);
                    // `a · A` en una sola cuenta: la cobertura y la opacidad son
                    // dos alfas que se multiplican.
                    let fuerza = cobertura * opacidad / 255;
                    if fuerza == 0 {
                        continue;
                    }

                    let inicio = base + (x as usize) * 3;
                    for (canal, valor) in capa.iter().enumerate() {
                        let fondo = u32::from(self.pixeles[inicio + canal]);
                        self.pixeles[inicio + canal] =
                            ((u32::from(*valor) * fuerza + fondo * (255 - fuerza)) / 255) as u8;
                    }
                }
            }
        }
    }

    /// Copia de la región pedida. El rectángulo se recorta al lienzo; si no
    /// queda nada, devuelve un lienzo vacío (1×1 transparente no: 0×0).
    pub fn recortar(&self, rect: Rect) -> Lienzo {
        let x_ini = i64::from(rect.x).max(0).min(i64::from(self.ancho));
        let y_ini = i64::from(rect.y).max(0).min(i64::from(self.alto));
        let x_fin = (i64::from(rect.x) + i64::from(rect.ancho))
            .max(0)
            .min(i64::from(self.ancho));
        let y_fin = (i64::from(rect.y) + i64::from(rect.alto))
            .max(0)
            .min(i64::from(self.alto));
        let ancho = (x_fin - x_ini).max(0) as u32;
        let alto = (y_fin - y_ini).max(0) as u32;

        let mut pixeles = Vec::with_capacity(ancho as usize * alto as usize * 3);
        for y in y_ini..y_fin {
            for x in x_ini..x_fin {
                let inicio = (y as usize * self.ancho as usize + x as usize) * 3;
                pixeles.extend_from_slice(&self.pixeles[inicio..inicio + 3]);
            }
        }

        Lienzo {
            ancho,
            alto,
            pixeles,
        }
    }
}

/// Caja que encierra todos los bitmaps, con `margen` píxeles alrededor, recortada
/// al frame de `ancho`×`alto`. `None` si no queda nada visible.
///
/// Es lo que permite guardar como golden solo la franja del subtítulo en vez del
/// frame entero, y sirve para localizar una diferencia: si el render se desvía,
/// el PNG de diferencias ya está recortado a la zona que importa.
pub fn caja(bitmaps: &[Bitmap], margen: u32, ancho: u32, alto: u32) -> Option<Rect> {
    let mut caja: Option<(i64, i64, i64, i64)> = None;

    for bitmap in bitmaps {
        if bitmap.ancho == 0 || bitmap.alto == 0 {
            continue;
        }
        let x0 = i64::from(bitmap.x);
        let y0 = i64::from(bitmap.y);
        let x1 = x0 + i64::from(bitmap.ancho);
        let y1 = y0 + i64::from(bitmap.alto);

        caja = Some(match caja {
            None => (x0, y0, x1, y1),
            Some((cx0, cy0, cx1, cy1)) => (cx0.min(x0), cy0.min(y0), cx1.max(x1), cy1.max(y1)),
        });
    }

    let (x0, y0, x1, y1) = caja?;
    let margen = i64::from(margen);
    let x0 = (x0 - margen).max(0);
    let y0 = (y0 - margen).max(0);
    let x1 = (x1 + margen).min(i64::from(ancho));
    let y1 = (y1 + margen).min(i64::from(alto));
    if x1 <= x0 || y1 <= y0 {
        return None;
    }

    Some(Rect {
        x: x0 as i32,
        y: y0 as i32,
        ancho: (x1 - x0) as u32,
        alto: (y1 - y0) as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bitmap de prueba: `ancho`×`alto` con una sola cobertura para toda la
    /// máscara, para que la cuenta sea a mano.
    fn bitmap(ancho: u32, alto: u32, x: i32, y: i32, cobertura: u8, color: u32) -> Bitmap {
        Bitmap::de_prueba(ancho, alto, x, y, cobertura, color)
    }

    fn pixel(lienzo: &Lienzo, x: u32, y: u32) -> [u8; 3] {
        let inicio = (y as usize * lienzo.ancho() as usize + x as usize) * 3;
        let p = lienzo.pixeles();
        [p[inicio], p[inicio + 1], p[inicio + 2]]
    }

    #[test]
    fn cobertura_total_y_opacidad_total_pinta_el_color() {
        let mut lienzo = Lienzo::nuevo(2, 2, [0, 0, 0]);
        // 0xRRGGBBAA con alfa 0 = opaco.
        lienzo.componer(&[bitmap(2, 2, 0, 0, 255, 0xFF_80_00_00)]);
        assert_eq!(pixel(&lienzo, 0, 0), [255, 128, 0]);
        assert_eq!(pixel(&lienzo, 1, 1), [255, 128, 0]);
    }

    #[test]
    fn cobertura_media_mezcla_mitad() {
        let mut lienzo = Lienzo::nuevo(1, 1, [0, 0, 0]);
        lienzo.componer(&[bitmap(1, 1, 0, 0, 128, 0xFF_FF_FF_00)]);
        // 128/255 de blanco sobre negro: 128, sin desbordar en u8.
        assert_eq!(pixel(&lienzo, 0, 0), [128, 128, 128]);
    }

    #[test]
    fn alfa_del_color_tambien_opera() {
        let mut lienzo = Lienzo::nuevo(1, 1, [0, 0, 0]);
        // Alfa 0x80 = mitad de transparente, cobertura total.
        lienzo.componer(&[bitmap(1, 1, 0, 0, 255, 0xFF_FF_FF_80)]);
        assert_eq!(pixel(&lienzo, 0, 0), [127, 127, 127]);
    }

    #[test]
    fn opacidad_cero_no_toca_el_fondo() {
        let mut lienzo = Lienzo::nuevo(1, 1, [10, 20, 30]);
        // Alfa 0xFF = invisible.
        lienzo.componer(&[bitmap(1, 1, 0, 0, 255, 0xFF_FF_FF_FF)]);
        assert_eq!(pixel(&lienzo, 0, 0), [10, 20, 30]);
    }

    #[test]
    fn bitmap_parcialmente_fuera_por_la_izquierda_se_recorta() {
        // Empieza en x = -1: la columna 0 de la máscara cae fuera y la columna
        // 1 en el píxel 0 del lienzo. Si el recorte ignorara el desfase, se
        // pintaría la columna 0 (transparente) y el píxel quedaría negro.
        let mut lienzo = Lienzo::nuevo(2, 1, [0, 0, 0]);
        lienzo.componer(&[Bitmap::de_prueba_mascara(
            2,
            1,
            -1,
            0,
            &[0, 255],
            0xFF_FF_FF_00,
        )]);
        assert_eq!(pixel(&lienzo, 0, 0), [255, 255, 255]);
        assert_eq!(pixel(&lienzo, 1, 0), [0, 0, 0]);
    }

    #[test]
    fn bitmap_entero_fuera_no_hace_nada() {
        let mut lienzo = Lienzo::nuevo(2, 2, [7, 7, 7]);
        lienzo.componer(&[
            bitmap(4, 4, -10, 0, 255, 0xFF_FF_FF_00),
            bitmap(4, 4, 0, -10, 255, 0xFF_FF_FF_00),
            bitmap(4, 4, 10, 0, 255, 0xFF_FF_FF_00),
        ]);
        for x in 0..2 {
            for y in 0..2 {
                assert_eq!(pixel(&lienzo, x, y), [7, 7, 7]);
            }
        }
    }

    #[test]
    fn desplazamiento_mueve_el_bitmap() {
        let mut lienzo = Lienzo::nuevo(2, 1, [0, 0, 0]);
        // El bitmap dice x=10; el lienzo empieza en x=10 del frame.
        lienzo.componer_desplazado(&[bitmap(1, 1, 10, 0, 255, 0xFF_FF_FF_00)], -10, 0);
        assert_eq!(pixel(&lienzo, 0, 0), [255, 255, 255]);
    }

    #[test]
    fn recortar_copia_la_region_pedida() {
        let mut lienzo = Lienzo::nuevo(3, 2, [0, 0, 0]);
        lienzo.componer(&[bitmap(1, 1, 2, 1, 255, 0xFF_00_00_00)]);

        let recorte = lienzo.recortar(Rect {
            x: 2,
            y: 1,
            ancho: 1,
            alto: 1,
        });
        assert_eq!((recorte.ancho(), recorte.alto()), (1, 1));
        assert_eq!(pixel(&recorte, 0, 0), [255, 0, 0]);
    }

    #[test]
    fn recortar_fuera_del_lienzo_queda_vacio() {
        let lienzo = Lienzo::nuevo(2, 2, [0, 0, 0]);
        let recorte = lienzo.recortar(Rect {
            x: 5,
            y: 5,
            ancho: 4,
            alto: 4,
        });
        assert_eq!((recorte.ancho(), recorte.alto()), (0, 0));
        assert!(recorte.pixeles().is_empty());
    }

    #[test]
    fn caja_une_bitmaps_y_aplica_margen() {
        let bitmaps = [bitmap(4, 2, 10, 10, 255, 0), bitmap(2, 2, 20, 20, 255, 0)];
        let rect = caja(&bitmaps, 1, 100, 100).expect("hay bitmaps");
        assert_eq!(
            rect,
            Rect {
                x: 9,
                y: 9,
                ancho: 14,
                alto: 14
            }
        );
    }

    #[test]
    fn caja_se_recorta_al_frame() {
        let bitmaps = [bitmap(10, 10, -5, -5, 255, 0)];
        let rect = caja(&bitmaps, 0, 100, 100).expect("hay bitmaps");
        assert_eq!(
            rect,
            Rect {
                x: 0,
                y: 0,
                ancho: 5,
                alto: 5
            }
        );
    }

    #[test]
    fn caja_sin_bitmaps_visibles_es_none() {
        assert_eq!(caja(&[], 0, 100, 100), None);
        assert_eq!(caja(&[bitmap(4, 4, 200, 0, 255, 0)], 0, 100, 100), None);
        assert_eq!(caja(&[bitmap(0, 0, 0, 0, 255, 0)], 0, 100, 100), None);
    }
}
