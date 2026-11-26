//! Avisos transitorios abajo al centro, con botón de deshacer opcional.

use std::time::{Duration, Instant};

use egui::RichText;

use crate::editor_app::MensajeEditor;
use crate::textos::Textos;

use super::tema::{self, Estilo};

const VIDA: Duration = Duration::from_secs(6);

#[derive(Default)]
pub struct Toasts {
    avisos: Vec<(String, Option<MensajeEditor>, Instant)>,
    deshacer: Option<MensajeEditor>,
}

impl Toasts {
    pub fn push(&mut self, texto: String, deshacer: bool) {
        let mensaje = deshacer.then_some(MensajeEditor::Deshacer);
        self.avisos.push((texto, mensaje, Instant::now()));
    }

    /// Aviso de preset borrado: «Deshacer» restaura el preset, no toca el
    /// `Historial` de ediciones, para no secuestrar Ctrl+Z.
    pub fn push_restaurar_preset(&mut self, texto: String) {
        self.avisos
            .push((texto, Some(MensajeEditor::RestaurarPreset), Instant::now()));
    }

    /// Dibuja los avisos. Devuelve el mensaje de «Deshacer» pulsado.
    pub fn mostrar(&mut self, ctx: &egui::Context, textos: &Textos) -> Option<MensajeEditor> {
        let ahora = Instant::now();
        self.avisos
            .retain(|(_, _, instante)| ahora.duration_since(*instante) < VIDA);
        if self.avisos.is_empty() {
            return None;
        }
        self.deshacer = None;
        let marco = egui::Frame {
            inner_margin: egui::Margin::symmetric(tema::ESP_12 as i8, tema::ESP_8 as i8),
            fill: tema::PANEL,
            stroke: egui::Stroke::new(1.0_f32, tema::BORDE_CONTROL),
            corner_radius: egui::CornerRadius::same(tema::RADIO_SUPERFICIE as u8),
            ..egui::Frame::NONE
        };
        egui::Area::new(egui::Id::new("toasts"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -16.0))
            .show(ctx, |ui| {
                marco.show(ui, |ui| {
                    for (texto, mensaje, _) in &self.avisos {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(texto)
                                    .font(tema::fuente(Estilo::Base))
                                    .color(tema::TEXTO),
                            );
                            if mensaje.is_some()
                                && ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(
                                                textos.comando(crate::comandos::Comando::Deshacer),
                                            )
                                            .font(tema::fuente(Estilo::Base)),
                                        )
                                        .frame(false),
                                    )
                                    .clicked()
                            {
                                self.deshacer = mensaje.clone();
                            }
                        });
                    }
                });
            });
        if self.deshacer.is_some() {
            self.avisos.clear();
        }
        self.deshacer.take()
    }
}
