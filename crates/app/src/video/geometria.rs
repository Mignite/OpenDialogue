//! La geometría del compositor, sin GPU.
//!
//! La aritmética del letterbox y del rectángulo de cada capa vive aquí porque
//! `prepare` necesita `device` y `queue`: metida en el `Primitive` no se puede
//! probar sin una tarjeta, y es justo la parte que se equivoca en silencio (un
//! video deformado o unos subtítulos corridos no fallan, se ven mal).
//!
//! Las dos funciones trabajan en el espacio que espera WGSL: clip space, con
//! `x` de -1 (izquierda) a 1 (derecha) y `y` de 1 (arriba) a -1 (abajo).

/// Cuánto se encoge el quad del video dentro del widget, por eje.
///
/// `1.0` en un eje significa que ese eje ya encaja. iced pone un viewport por
/// primitivo, así que basta con encoger el eje que sobra: el quad, centrado en
/// clip space, queda centrado en el widget y el resto son las barras negras.
pub fn escala_letterbox(video: (u32, u32), widget: (f32, f32)) -> (f32, f32) {
    if video.0 == 0 || video.1 == 0 || widget.0 <= 0.0 || widget.1 <= 0.0 {
        return (1.0, 1.0);
    }

    let aspecto_video = video.0 as f32 / video.1 as f32;
    let aspecto_widget = widget.0 / widget.1;

    if aspecto_video > aspecto_widget {
        // El video es más ancho: sobran barras arriba y abajo.
        (1.0, aspecto_widget / aspecto_video)
    } else {
        // El video es más alto: sobran barras a los lados.
        (aspecto_video / aspecto_widget, 1.0)
    }
}

/// Rectángulo de una capa en clip space: `[x0, y0, x1, y1]`.
///
/// `x`/`y`/`ancho`/`alto` están en píxeles del canvas del video, y `escala` es
/// el resultado de [`escala_letterbox`]. La `y` se invierte porque en clip space
/// crece hacia arriba.
pub fn rect_de_capa(
    x: i32,
    y: i32,
    ancho: u32,
    alto: u32,
    video: (u32, u32),
    escala: (f32, f32),
) -> [f32; 4] {
    if video.0 == 0 || video.1 == 0 {
        return [0.0; 4];
    }

    let (video_w, video_h) = (video.0 as f32, video.1 as f32);
    [
        (2.0 * x as f32 / video_w - 1.0) * escala.0,
        (1.0 - 2.0 * y as f32 / video_h) * escala.1,
        (2.0 * (x + ancho as i32) as f32 / video_w - 1.0) * escala.0,
        (1.0 - 2.0 * (y + alto as i32) as f32 / video_h) * escala.1,
    ]
}

#[cfg(test)]
mod tests {
    use super::{escala_letterbox, rect_de_capa};

    const EPSILON: f32 = 1e-6;

    fn cerca(obtenido: (f32, f32), esperado: (f32, f32)) {
        assert!(
            (obtenido.0 - esperado.0).abs() < EPSILON && (obtenido.1 - esperado.1).abs() < EPSILON,
            "esperaba {esperado:?}, salió {obtenido:?}"
        );
    }

    fn rect_cerca(obtenido: [f32; 4], esperado: [f32; 4]) {
        for (valor, esperado) in obtenido.iter().zip(esperado.iter()) {
            assert!(
                (valor - esperado).abs() < EPSILON,
                "esperaba {esperado:?}, salió {obtenido:?}"
            );
        }
    }

    #[test]
    fn misma_proporcion_no_necesita_barras() {
        cerca(escala_letterbox((1920, 1080), (1600.0, 900.0)), (1.0, 1.0));
    }

    #[test]
    fn video_mas_ancho_deja_barras_arriba_y_abajo() {
        // 21:9 dentro de 16:9: se encoge la altura.
        cerca(escala_letterbox((2560, 1080), (1600.0, 900.0)), (1.0, 0.75));
    }

    #[test]
    fn video_mas_alto_deja_barras_a_los_lados() {
        // 4:3 dentro de 16:9: se encoge la anchura.
        cerca(escala_letterbox((1440, 1080), (1600.0, 900.0)), (0.75, 1.0));
    }

    #[test]
    fn un_widget_sin_tamano_no_deforma() {
        cerca(escala_letterbox((1920, 1080), (0.0, 900.0)), (1.0, 1.0));
        cerca(escala_letterbox((0, 0), (1600.0, 900.0)), (1.0, 1.0));
    }

    #[test]
    fn una_capa_que_cubre_el_canvas_llena_el_quad() {
        rect_cerca(
            rect_de_capa(0, 0, 1920, 1080, (1920, 1080), (1.0, 1.0)),
            [-1.0, 1.0, 1.0, -1.0],
        );
    }

    #[test]
    fn una_capa_centrada_queda_centrada() {
        // 2x2 en el centro de 100x100: medio píxel de cada lado en clip space.
        let esperado = -0.02_f32;
        rect_cerca(
            rect_de_capa(49, 49, 2, 2, (100, 100), (1.0, 1.0)),
            [esperado, -esperado, -esperado, esperado],
        );
    }

    #[test]
    fn una_capa_fuera_del_canvas_queda_fuera_del_quad() {
        // A la izquierda del todo: el quad sale de clip space y la GPU recorta.
        rect_cerca(
            rect_de_capa(-50, 0, 10, 10, (100, 100), (1.0, 1.0)),
            [-2.0, 1.0, -1.8, 0.8],
        );
    }

    #[test]
    fn el_letterbox_tambien_mueve_las_capas() {
        // Canvas 16:9 en un widget 4:3: la capa de altura completa queda
        // comprimida a 0.75, igual que el video, para no desalinearse.
        rect_cerca(
            rect_de_capa(0, 0, 1920, 1080, (1920, 1080), (1.0, 0.75)),
            [-1.0, 0.75, 1.0, -0.75],
        );
    }

    #[test]
    fn sin_video_no_hay_rectangulo() {
        let escala = escala_letterbox((0, 1080), (1600.0, 900.0));
        rect_cerca(rect_de_capa(10, 10, 10, 10, (0, 1080), escala), [0.0; 4]);
    }
}
