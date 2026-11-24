//! Avisos transitorios abajo al centro, con botón de deshacer opcional.

use std::time::{Duration, Instant};

use egui::RichText;

use crate::textos::Textos;

use super::tema::{self, Estilo};

const VIDA: Duration = Duration::from_secs(6);

#[derive(Default)]
pub struct Toasts {
    avisos: Vec<(String, bool, Instant)>,
    deshacer: bool,
}

impl Toasts {
    pub fn push(&mut self, texto: String, deshacer: bool) {
        self.avisos.push((texto, deshacer, Instant::now()));
    }

    /// Dibuja los avisos. Devuelve `true` si se pulsó «Deshacer».
    pub fn mostrar(&mut self, ctx: &egui::Context, textos: &Textos) -> bool {
        let ahora = Instant::now();
        self.avisos
            .retain(|(_, _, instante)| ahora.duration_since(*instante) < VIDA);
        if self.avisos.is_empty() {
            return false;
        }
        self.deshacer = false;
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
                    for (texto, con_deshacer, _) in &self.avisos {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(texto)
                                    .font(tema::fuente(Estilo::Base))
                                    .color(tema::TEXTO),
                            );
                            if *con_deshacer
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
                                self.deshacer = true;
                            }
                        });
                    }
                });
            });
        if self.deshacer {
            self.avisos.clear();
        }
        self.deshacer
    }
}
