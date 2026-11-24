//! Iconos propios dibujados con `Painter`, en la misma gramática que los SVG
//! anteriores: rejilla de 24 y trazo de 1,7 px. No se importan assets de
//! ArtCraft.

use egui::{Color32, Rect, Ui};

pub const TRAZO: f32 = 1.7;

/// Un icono sencillo de "play" o "pausa", dibujado en el rect dado.
pub fn pintar(ui: &mut Ui, rect: Rect, color: Color32, pausa: bool) {
    let painter = ui.painter();
    if pausa {
        let tercio = rect.width() / 3.0;
        painter.rect_filled(
            Rect::from_min_max(
                rect.min + egui::vec2(tercio, 2.0),
                rect.min + egui::vec2(2.0 * tercio, rect.height() - 2.0),
            ),
            egui::CornerRadius::same(1),
            color,
        );
    } else {
        painter.add(egui::Shape::closed_line(
            vec![
                rect.min + egui::vec2(4.0, 2.0),
                rect.min + egui::vec2(rect.width() - 3.0, rect.height() / 2.0),
                rect.min + egui::vec2(4.0, rect.height() - 2.0),
            ],
            egui::Stroke::new(TRAZO, color),
        ));
    }
}
