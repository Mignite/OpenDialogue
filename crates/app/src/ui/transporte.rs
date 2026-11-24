//! Transporte compacto: play/pause, stepping, reloj y velocidad.

use egui::Ui;

use crate::editor_app::{EditorApp, MensajeEditor, VELOCIDADES};
use crate::ui::iconos;
use crate::ui::tema;

fn boton_transporte(ui: &mut Ui, texto: &str) -> bool {
    ui.add(
        egui::Button::new(
            egui::RichText::new(texto)
                .font(tema::mono_public())
                .size(11.0)
                .color(tema::SECUNDARIO),
        )
        .fill(tema::LATERAL)
        .corner_radius(egui::CornerRadius::same(4))
        .min_size(egui::vec2(30.0, 24.0)),
    )
    .clicked()
}

pub fn mostrar(ui: &mut Ui, state: &mut EditorApp) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let (rect, respuesta) =
            ui.allocate_exact_size(egui::vec2(26.0, 24.0), egui::Sense::click());
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(4), tema::LATERAL);
        iconos::pintar(ui, rect.shrink(4.0), tema::TEXTO, state.paused);
        if respuesta.clicked() {
            mensajes.push(MensajeEditor::PlayPause);
        }

        if boton_transporte(ui, "◀") {
            mensajes.push(MensajeEditor::Step(-5.0));
        }
        if boton_transporte(ui, "▶") {
            mensajes.push(MensajeEditor::Step(5.0));
        }

        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(format!(
                "{} / {}",
                rod_core::format_time(state.position),
                rod_core::format_time(state.duration)
            ))
            .font(tema::mono_public())
            .size(11.0)
            .color(tema::TEXTO),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let velocidad = ui.add(
                egui::Button::new(
                    egui::RichText::new(state.velocidad.to_string())
                        .font(tema::mono_public())
                        .size(10.0)
                        .color(tema::ACENTO),
                )
                .fill(egui::Color32::TRANSPARENT)
                .corner_radius(egui::CornerRadius::same(4))
                .stroke(egui::Stroke::new(1.0_f32, tema::BORDE)),
            );
            if velocidad.clicked() {
                let actual = VELOCIDADES
                    .iter()
                    .position(|candidata| *candidata == state.velocidad)
                    .unwrap_or(2);
                mensajes.push(MensajeEditor::Velocidad(
                    VELOCIDADES[(actual + 1) % VELOCIDADES.len()],
                ));
            }
            tema::etiqueta(ui, "VELOCIDAD");
        });
    });
    mensajes
}
