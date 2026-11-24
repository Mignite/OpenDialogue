//! Panel de atajos de teclado (`?` / F1).

use egui::{Event, Key, RichText};

use crate::comandos::{Comando, MapaAtajos, Seccion};
use crate::textos::Textos;

use super::tema::{self, Estilo};

#[derive(Debug, Default)]
pub struct EstadoAtajos {
    pub abierto: bool,
}

/// Dibuja el panel. `Esc` lo cierra.
pub fn mostrar(
    ctx: &egui::Context,
    textos: &Textos,
    atajos: &MapaAtajos,
    estado: &mut EstadoAtajos,
) {
    if !estado.abierto {
        return;
    }
    let eventos: Vec<Event> = ctx.input(|entrada| entrada.events.clone());
    if eventos.iter().any(|evento| {
        matches!(
            evento,
            Event::Key {
                key: Key::Escape,
                pressed: true,
                ..
            }
        )
    }) {
        estado.abierto = false;
        return;
    }

    let marco = egui::Frame {
        inner_margin: egui::Margin::same(tema::ESP_16 as i8),
        fill: tema::PANEL,
        stroke: egui::Stroke::new(1.0_f32, tema::BORDE_CONTROL),
        corner_radius: egui::CornerRadius::same(tema::RADIO_SUPERFICIE as u8),
        ..egui::Frame::NONE
    };
    egui::Window::new(textos.comando(Comando::Ayuda))
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(560.0, 480.0))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .frame(marco)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut seccion_actual = None;
                for comando in Comando::todas() {
                    let seccion = crate::comandos::seccion(*comando);
                    if Some(seccion) != seccion_actual {
                        seccion_actual = Some(seccion);
                        ui.add_space(tema::ESP_8);
                        ui.label(
                            RichText::new(nombre_seccion(seccion, textos))
                                .font(tema::fuente(Estilo::Pequeno))
                                .color(tema::TEXTO_3),
                        );
                    }
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(textos.comando(*comando))
                                .font(tema::fuente(Estilo::Base))
                                .color(tema::TEXTO),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let atajo = atajos
                                .atajo(*comando)
                                .map(|atajo| atajo.to_string())
                                .unwrap_or_else(|| "—".into());
                            ui.label(
                                RichText::new(atajo)
                                    .font(tema::fuente(Estilo::Mono))
                                    .color(tema::TEXTO_2),
                            );
                        });
                    });
                }
            });
            if ui.button(textos.cancelar).clicked() {
                estado.abierto = false;
            }
        });
}

fn nombre_seccion(seccion: Seccion, textos: &Textos) -> &'static str {
    match seccion {
        Seccion::Archivo => textos.seccion_archivo,
        Seccion::Edicion => textos.seccion_edicion,
        Seccion::Ver => textos.seccion_ver,
        Seccion::Exportar => textos.seccion_exportar,
        Seccion::Estilos => textos.seccion_estilos,
        Seccion::Ayuda => textos.seccion_ayuda,
    }
}
