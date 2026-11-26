//! Render compartido de subtítulos: la tarjeta de Estilos y el overlay del
//! video pasan por el mismo `build_ass` + libass, así que no pueden divergir.

use image::RgbaImage;
use rod_core::{Caption, Hablante, PresetAss, build_ass};
use rod_subs::{Bitmap, Subtitulos};

use crate::preview::CANVAS;

/// Factor de escala de una muestra de 64 px sobre el frame completo.
pub fn escala_de_preview(fontsize: f32, alto_preview: f32) -> f32 {
    (fontsize * alto_preview / CANVAS.1 as f32).max(20.0)
}

/// Compone los bitmaps de libass sobre un frame transparente.
pub fn componer(ancho: u32, alto: u32, bitmaps: &[Bitmap]) -> RgbaImage {
    let mut imagen = RgbaImage::new(ancho, alto);
    for bitmap in bitmaps {
        let x0 = bitmap.x.max(0) as u32;
        let y0 = bitmap.y.max(0) as u32;
        let x1 = (bitmap.x + bitmap.ancho as i32).min(ancho as i32).max(0) as u32;
        let y1 = (bitmap.y + bitmap.alto as i32).min(alto as i32).max(0) as u32;
        let mascara = bitmap.alpha();
        // ASS guarda el alfa invertido y el color en 0xRRGGBBAA.
        let opacidad = 255_u32 - (bitmap.color & 0xFF);
        let rojo = (bitmap.color >> 24) as u8;
        let verde = ((bitmap.color >> 16) & 0xFF) as u8;
        let azul = ((bitmap.color >> 8) & 0xFF) as u8;
        for y in y0..y1 {
            for x in x0..x1 {
                let indice = (y - y0) as usize * bitmap.stride + (x - x0) as usize;
                let Some(&cobertura) = mascara.get(indice) else {
                    continue;
                };
                let alfa = (u32::from(cobertura) * opacidad / 255) as u8;
                if alfa == 0 {
                    continue;
                }
                let destino = imagen.get_pixel_mut(x, y);
                let fondo = destino.0;
                destino.0 = [
                    mezclar(fondo[0], rojo, alfa),
                    mezclar(fondo[1], verde, alfa),
                    mezclar(fondo[2], azul, alfa),
                    fondo[3].max(alfa),
                ];
            }
        }
    }
    imagen
}

fn mezclar(fondo: u8, frente: u8, alfa: u8) -> u8 {
    let t = f32::from(alfa) / 255.0;
    (f32::from(fondo) * (1.0 - t) + f32::from(frente) * t).round() as u8
}

/// Rasteriza una muestra del preset con el mismo pipeline del `.ass` exportado.
pub fn render_subtitle(preset: &PresetAss, texto: &str, escala: f32) -> RgbaImage {
    let ancho = (CANVAS.0 as f32 * escala).max(1.0) as u32;
    let alto = (CANVAS.1 as f32 * escala).max(1.0) as u32;
    let caption = Caption {
        id: "preview".into(),
        inicio: 0.0,
        fin: 1.0,
        texto: texto.into(),
        hablante_id: Some("preview".into()),
    };
    let hablante = Hablante {
        id: "preview".into(),
        nombre: "Preview".into(),
        tecla: "1".into(),
        color: String::new(),
        preset_id: Some(preset.id.clone()),
    };
    let ass = build_ass(&[caption], &[hablante], |_| preset.clone(), ancho, alto);
    let mut subs = Subtitulos::nuevo(ancho, alto, &ass);
    let bitmaps = subs.render(100);
    componer(ancho, alto, bitmaps)
}
