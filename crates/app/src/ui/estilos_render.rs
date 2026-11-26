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

/// Compone los bitmaps de libass sobre un frame transparente en RGBA recto,
/// que es lo que `ColorImage::from_rgba_unmultiplied` espera.
///
/// El shader del video mezcla color recto con `SrcAlpha`. Guardar `RGB` ya
/// multiplicado por el alfa haría que egui lo multiplique otra vez y el
/// contorno se viera más oscuro en la tarjeta que sobre el video.
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
        let frente = [
            (bitmap.color >> 24) as u8,
            ((bitmap.color >> 16) & 0xFF) as u8,
            ((bitmap.color >> 8) & 0xFF) as u8,
        ];
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
                // Source-over con alfa entero: el color de salida es recto,
                // no premultiplicado, aunque haya varias capas solapadas.
                let a_s = u32::from(alfa);
                let a_d = u32::from(fondo[3]);
                let a_out = a_s + a_d * (255 - a_s) / 255;
                for canal in 0..3 {
                    let mezcla = (u32::from(frente[canal]) * a_s
                        + u32::from(fondo[canal]) * a_d * (255 - a_s) / 255)
                        / a_out.max(1);
                    destino.0[canal] = mezcla.min(255) as u8;
                }
                destino.0[3] = a_out.min(255) as u8;
            }
        }
    }
    imagen
}

/// Rasteriza una muestra del preset con el mismo pipeline del `.ass` exportado.
///
/// `escala = 1.0` no toca ningún valor: la imagen es byte-idéntica a la del
/// overlay. En una miniatura (`escala < 1`) el `fontsize` baja con el canvas,
/// pero se frena en 20 px para que la muestra se pueda leer; el resto de
/// medidas se escalan con el mismo factor efectivo y así el texto no queda
/// cortado ni desplazado.
pub fn render_subtitle(preset: &PresetAss, texto: &str, escala: f32) -> RgbaImage {
    let ancho = (CANVAS.0 as f32 * escala).max(1.0) as u32;
    let alto = (CANVAS.1 as f32 * escala).max(1.0) as u32;
    let fuente = if escala >= 1.0 {
        preset.fontsize
    } else {
        f64::from(escala_de_preview(preset.fontsize as f32, alto as f32))
    };
    let factor = fuente / preset.fontsize.max(f64::MIN_POSITIVE);
    let mut escalado = preset.clone();
    escalado.fontsize = fuente;
    escalado.outline = preset.outline * factor;
    escalado.shadow = preset.shadow * factor;
    escalado.margin_l = (preset.margin_l as f64 * factor) as i32;
    escalado.margin_r = (preset.margin_r as f64 * factor) as i32;
    escalado.margin_v = (preset.margin_v as f64 * factor) as i32;

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
    let ass = build_ass(&[caption], &[hablante], |_| escalado.clone(), ancho, alto);
    let mut subs = Subtitulos::nuevo(ancho, alto, &ass);
    let bitmaps = subs.render(100);
    componer(ancho, alto, bitmaps)
}
