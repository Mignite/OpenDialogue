//! Paso 4 y 5 del slice 0b: componer los bitmaps de libass sobre el frame
//! limpio y comparar el resultado con lo que compuso mpv.
//!
//! El invariante del proyecto empieza aquí: si nuestro render y el de mpv
//! divergen, es un bug, no un ajuste de estilos.

const W: u32 = 1920;
const H: u32 = 1080;
const INSTANTE_MS: i64 = 1000;

/// Un canal puede diferir de otro por el antialiasing aunque el render sea
/// correcto; por debajo de esto se considera igual.
const TOLERANCIA: u8 = 8;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut lienzo = image::open("testdata/frame-sin-subs.png")?.to_rgb8();
    assert_eq!(
        (lienzo.width(), lienzo.height()),
        (W, H),
        "el frame de fondo no es 1920x1080"
    );

    let library = rod_subs::Library::new();
    let mut renderer = rod_subs::Renderer::new(&library, W, H);
    renderer.usar_fuentes(rod_subs::PROVEEDOR_AUTODETECT);
    let track = rod_subs::Track::parse(&library, include_str!("../tests/fixtures/basico.ass"));

    let bitmaps = renderer.render(&track, INSTANTE_MS);
    println!("{} bitmap(s) a {INSTANTE_MS} ms", bitmaps.len());

    for bitmap in bitmaps {
        let mascara = bitmap.alpha();
        // El alfa de ASS está invertido: 0 = opaco.
        let opacidad = 255 - (bitmap.color & 0xFF);
        // 0xRRGGBBAA: los canales van en los bytes altos.
        let color = [
            bitmap.color >> 24,
            (bitmap.color >> 16) & 0xFF,
            (bitmap.color >> 8) & 0xFF,
        ];

        for fila in 0..bitmap.alto {
            for columna in 0..bitmap.ancho {
                let cobertura = mascara[fila as usize * bitmap.stride + columna as usize] as u32;
                let fuerza = cobertura * opacidad / 255;
                if fuerza == 0 {
                    continue;
                }

                let pixel = lienzo.get_pixel_mut(bitmap.x as u32 + columna, bitmap.y as u32 + fila);
                for (canal, capa) in color.iter().enumerate() {
                    let fondo = pixel.0[canal] as u32;
                    pixel.0[canal] = ((capa * fuerza + fondo * (255 - fuerza)) / 255) as u8;
                }
            }
        }
    }

    lienzo.save("testdata/render-propio.png")?;
    println!("guardado testdata/render-propio.png");

    comparar()
}

/// Diferencias contra el oráculo, y un PNG con los píxeles que no coinciden
/// pintados de rojo: sirve para ver *dónde* falla (borde, posición, color).
fn comparar() -> Result<(), Box<dyn std::error::Error>> {
    let propio = image::open("testdata/render-propio.png")?.to_rgb8();
    let oraculo = image::open("testdata/oracle-subs.png")?.to_rgb8();
    assert_eq!(propio.dimensions(), oraculo.dimensions());

    let mut mascara = image::RgbImage::new(propio.width(), propio.height());
    let mut distintos = 0u64;
    let mut exactos = 0u64;
    let mut maximo = 0u8;

    for (x, y, a) in propio.enumerate_pixels() {
        let b = oraculo.get_pixel(x, y);
        let diferencia = (0..3).map(|c| a.0[c].abs_diff(b.0[c])).max().unwrap();
        maximo = maximo.max(diferencia);

        if diferencia > 0 {
            exactos += 1;
        }
        if diferencia > TOLERANCIA {
            distintos += 1;
            mascara.put_pixel(x, y, image::Rgb([255, 0, 0]));
        }
    }

    let total = u64::from(propio.width()) * u64::from(propio.height());
    let porcentaje = distintos as f64 * 100.0 / total as f64;
    let porcentaje_exacto = exactos as f64 * 100.0 / total as f64;

    println!("distintos (>{TOLERANCIA}): {distintos}/{total} = {porcentaje:.3} % del frame");
    println!("con cualquier diferencia: {exactos}/{total} = {porcentaje_exacto:.3} %");
    println!("diferencia máxima por canal: {maximo}/255");

    mascara.save("testdata/diff.png")?;
    println!("guardado testdata/diff.png");

    Ok(())
}
