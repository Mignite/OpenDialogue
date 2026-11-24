//! Timeline dibujada con `egui::Painter`: regla, carriles, clips y playhead.
//! El significado de cada gesto sigue en `timeline::pulsar` y en
//! `EditorApp::update`, que recibe los mismos mensajes de antes.

use egui::{Ui, Vec2};

use crate::editor_app::{EditorApp, MensajeEditor};
use crate::timeline::{self, ALTO_CARRIL, ALTO_REGLA, Mensaje};
use crate::ui::tema;

fn punto(rect: egui::Rect, pos: egui::Pos2) -> timeline::Punto {
    timeline::Punto::new(pos.x - rect.min.x, pos.y - rect.min.y)
}

pub fn playhead(state: &EditorApp) -> f64 {
    if state.position.is_finite() && state.duration.is_finite() {
        state.position
    } else {
        0.0
    }
}

pub fn mostrar(ui: &mut Ui, state: &mut EditorApp) -> Option<MensajeEditor> {
    let (rect, respuesta) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), ALTO_REGLA + ALTO_CARRIL * 3.0),
        egui::Sense::click_and_drag(),
    );
    let painter = ui.painter();
    painter.rect_filled(rect, egui::CornerRadius::same(0), tema::SUPERFICIE);

    // Ancho real para que la app sepa cuántos segundos caben.
    let mensaje = Some(MensajeEditor::Timeline(Mensaje::Ancho(rect.width())));

    // Regla y carriles.
    for fila in 0..3 {
        let y = rect.min.y + ALTO_REGLA + fila as f32 * ALTO_CARRIL;
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(rect.min.x, y),
                egui::pos2(rect.max.x, y + ALTO_CARRIL - 1.0),
            ),
            egui::CornerRadius::same(0),
            tema::color_de_hex("#232323"),
        );
    }

    let vista = state.vista;
    for (indice, caption) in state.editor.proyecto().captions.iter().enumerate() {
        let x0 = rect.min.x + vista.x_de(caption.inicio);
        let x1 = rect.min.x + vista.x_de(caption.fin);
        if x1 <= rect.min.x || x0 >= rect.max.x {
            continue;
        }
        let fila = indice % 3;
        let y = rect.min.y + ALTO_REGLA + fila as f32 * ALTO_CARRIL + 4.0;
        let color = caption
            .hablante_id
            .as_deref()
            .and_then(|id| state.editor.hablante(id))
            .map(|hablante| tema::color_de_hex(&hablante.color))
            .unwrap_or(tema::SECUNDARIO);
        let seleccionado = state.editor.seleccion.contains(&caption.id);
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(x0.max(rect.min.x), y),
                egui::pos2(x1.min(rect.max.x), y + ALTO_CARRIL - 10.0),
            ),
            egui::CornerRadius::same(4),
            if seleccionado { tema::ACENTO } else { color },
        );
    }

    let x = rect.min.x + vista.x_de(playhead(state));
    if (rect.min.x..=rect.max.x).contains(&x) {
        painter.line_segment(
            [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
            egui::Stroke::new(1.5_f32, tema::ACENTO),
        );
    }

    if respuesta.drag_started()
        && let Some(pos) = respuesta.interact_pointer_pos()
    {
        return Some(MensajeEditor::Timeline(Mensaje::Pulsar(punto(rect, pos))));
    }
    if respuesta.dragged()
        && let Some(pos) = respuesta.interact_pointer_pos()
    {
        return Some(MensajeEditor::Timeline(Mensaje::Arrastrar(punto(
            rect, pos,
        ))));
    }
    if respuesta.drag_stopped()
        && let Some(pos) = respuesta.interact_pointer_pos()
    {
        return Some(MensajeEditor::Timeline(Mensaje::Soltar(punto(rect, pos))));
    }
    // Una muesca de rueda equivale a ~40 px en el registro del oráculo.
    let lineas = ui.input(|entrada| entrada.raw_scroll_delta.y) / 40.0;
    if lineas != 0.0 {
        return Some(MensajeEditor::Timeline(Mensaje::Rueda {
            lineas,
            punto: punto(rect, rect.min + Vec2::new(rect.width() / 2.0, ALTO_REGLA)),
        }));
    }
    let _ = timeline::UMBRAL_CLIC;
    mensaje
}

#[cfg(test)]
mod tests {
    use crate::timeline::Punto as Point;

    use super::playhead;
    use crate::editor_app::{EditorApp, MensajeEditor, update};
    use crate::timeline::{Mensaje, Vista};

    fn editor() -> EditorApp {
        let video = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../testdata/clip-15s-2audio.mp4"
        );
        let mut editor = EditorApp::desde_video(video).expect("video de prueba");
        editor.editor.agregar(rod_core::Caption {
            id: "cap-arrastre".into(),
            inicio: 1.0,
            fin: 2.0,
            texto: "uno".into(),
            hablante_id: None,
        });
        update(&mut editor, MensajeEditor::Timeline(Mensaje::Ancho(800.0)));
        editor.vista = Vista::nueva(0.02);
        editor
    }

    #[test]
    fn arrastre_termina_en_el_tiempo_soltado_y_es_un_paso() {
        let mut editor = editor();
        let y = super::ALTO_REGLA + crate::timeline::ALTO_CARRIL / 2.0;
        let x_inicio = editor.vista.x_de(1.5);
        let x_fin = editor.vista.x_de(2.5);

        update(
            &mut editor,
            MensajeEditor::Timeline(Mensaje::Pulsar(Point::new(x_inicio, y))),
        );
        update(
            &mut editor,
            MensajeEditor::Timeline(Mensaje::Arrastrar(Point::new(x_fin, y))),
        );
        update(
            &mut editor,
            MensajeEditor::Timeline(Mensaje::Soltar(Point::new(x_fin, y))),
        );

        let caption = &editor.editor.proyecto().captions[0];
        assert!((caption.fin - 3.0).abs() < 0.02, "fin: {}", caption.fin);
        assert!(editor.editor.puede_deshacer());
    }

    #[test]
    fn sin_time_pos_el_playhead_es_cero() {
        let mut editor = editor();
        editor.position = f64::NAN;
        assert_eq!(playhead(&editor), 0.0);
    }
}
