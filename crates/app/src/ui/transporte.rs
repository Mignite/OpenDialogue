//! Barra de transporte: play/pause, stepping, tiempo y velocidad.

use egui::Ui;

use crate::editor_app::{EditorApp, MensajeEditor, VELOCIDADES};
use crate::ui::iconos;

pub fn mostrar(ui: &mut Ui, state: &mut EditorApp) -> Option<MensajeEditor> {
    let mut mensaje = None;
    ui.horizontal(|ui| {
        let (rect, respuesta) =
            ui.allocate_exact_size(egui::vec2(22.0, 18.0), egui::Sense::click());
        iconos::pintar(ui, rect, crate::ui::tema::TEXTO, state.paused);
        if respuesta.clicked() {
            mensaje = Some(MensajeEditor::PlayPause);
        }

        if ui.button("◀").clicked() {
            mensaje = Some(MensajeEditor::Step(-5.0));
        }
        if ui.button("▶").clicked() {
            mensaje = Some(MensajeEditor::Step(5.0));
        }
        ui.label(format!(
            "{} / {}",
            rod_core::format_time(state.position),
            rod_core::format_time(state.duration)
        ));

        if ui.button(state.velocidad.to_string()).clicked() {
            let actual = VELOCIDADES
                .iter()
                .position(|velocidad| *velocidad == state.velocidad)
                .unwrap_or(2);
            let siguiente = VELOCIDADES[(actual + 1) % VELOCIDADES.len()];
            mensaje = Some(MensajeEditor::Velocidad(siguiente));
        }
    });
    mensaje
}
