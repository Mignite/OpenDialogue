//! Transporte del editor: iconos, timecode grande y velocidad.

use egui::{RichText, Vec2};

use crate::comandos::{Comando, MapaAtajos};
use crate::editor_app::{EditorApp, MensajeEditor, VELOCIDADES, sucio};
use crate::textos::Textos;
use crate::ui::kit::{self, Icono};
use crate::ui::tema::{self, Estilo};

pub fn mostrar(
    ui: &mut egui::Ui,
    state: &mut EditorApp,
    textos: &Textos,
    atajos: &MapaAtajos,
) -> Vec<MensajeEditor> {
    let _ = textos;
    let _ = atajos;
    let mut mensajes = Vec::new();
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = tema::ESP_4;

        if kit::icon_button(ui, Icono::Inicio, false, "Inicio", None).clicked() {
            mensajes.push(MensajeEditor::Step(-state.duration));
        }
        if kit::icon_button(ui, Icono::CuadroAnterior, false, "Cuadro anterior", None).clicked() {
            mensajes.push(MensajeEditor::Step(-0.04));
        }

        let respuesta = ui.add(
            egui::Button::new(
                RichText::new(kit::glifo(if state.paused {
                    Icono::Play
                } else {
                    Icono::Pausa
                }))
                .font(egui::FontId::proportional(20.0))
                .color(tema::TEXTO),
            )
            .fill(tema::CONTROL)
            .corner_radius(egui::CornerRadius::same(tema::RADIO_CONTROL as u8))
            .min_size(Vec2::splat(tema::ALTO_BOTON)),
        );
        if respuesta.clicked() {
            mensajes.push(MensajeEditor::PlayPause);
        }

        if kit::icon_button(ui, Icono::CuadroSiguiente, false, "Cuadro siguiente", None).clicked() {
            mensajes.push(MensajeEditor::Step(0.04));
        }
        if kit::icon_button(ui, Icono::Fin, false, "Fin", None).clicked() {
            mensajes.push(MensajeEditor::Step(state.duration));
        }

        ui.add_space(tema::ESP_8);
        ui.label(
            RichText::new(rod_core::format_time(state.position))
                .font(tema::fuente(Estilo::TimecodeGrande))
                .color(tema::TEXTO),
        );
        ui.label(
            RichText::new(format!("/ {}", rod_core::format_time(state.duration)))
                .font(tema::fuente(Estilo::Pequeno))
                .color(tema::TEXTO_3),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::ComboBox::from_id_salt("velocidad")
                .selected_text(state.velocidad.to_string())
                .show_ui(ui, |ui| {
                    for velocidad in VELOCIDADES {
                        if ui
                            .selectable_value(
                                &mut state.velocidad,
                                velocidad,
                                velocidad.to_string(),
                            )
                            .clicked()
                        {
                            mensajes.push(MensajeEditor::Velocidad(velocidad));
                        }
                    }
                });
            kit::icono(ui, Icono::Volumen);
            kit::icon_button(ui, Icono::Bucle, state.seguir, "Seguir al playhead", None);
        });
    });
    mensajes
}

/// Barra de estado de 24 px bajo el transporte.
pub fn barra_estado(ui: &mut egui::Ui, state: &EditorApp, textos: &Textos) {
    ui.horizontal(|ui| {
        ui.set_height(24.0);
        ui.spacing_mut().item_spacing.x = tema::ESP_8;
        ui.label(
            RichText::new(format!(
                "{} · {} · {}",
                textos.lineas(state.editor.proyecto().captions.len()),
                textos.hablantes(state.editor.proyecto().hablantes.len()),
                textos.comando(Comando::Buscar)
            ))
            .font(tema::fuente(Estilo::Pequeno))
            .color(tema::TEXTO_3),
        );
        ui.label(
            RichText::new(if sucio(state) {
                textos.sin_guardar
            } else {
                textos.guardado
            })
            .font(tema::fuente(Estilo::Pequeno))
            .color(tema::TEXTO_3),
        );
    });
}
