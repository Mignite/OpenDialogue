//! Panel de hablantes: tarjetas compactas con color, atajo, estilo y mute/solo.

use egui::{RichText, Stroke, Vec2};

use crate::editor::MAX_HABLANTES;
use crate::editor_app::{CampoHablante, EditorApp, MensajeEditor};
use crate::textos::Textos;
use crate::ui::kit::{self, Icono};
use crate::ui::tema::{self, Estilo};

pub fn mostrar(ui: &mut egui::Ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();

    if ui
        .add_enabled(
            state.editor.proyecto().hablantes.len() < MAX_HABLANTES,
            egui::Button::new(
                RichText::new(format!(
                    "{}  {}",
                    kit::glifo(Icono::Mas),
                    textos.agregar_hablante
                ))
                .font(tema::fuente(Estilo::Base)),
            )
            .frame(false),
        )
        .clicked()
    {
        mensajes.push(MensajeEditor::AgregarHablante);
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        let hablantes = state.editor.proyecto().hablantes.clone();
        let captions = state.editor.proyecto().captions.clone();
        for (indice, hablante) in hablantes.iter().enumerate() {
            let color = tema::color(rod_core::color_identidad(hablante, indice));
            let lineas: Vec<_> = captions
                .iter()
                .filter(|caption| caption.hablante_id.as_deref() == Some(&hablante.id))
                .collect();
            let segundos: f32 = lineas
                .iter()
                .map(|caption| (caption.fin - caption.inicio).max(0.0) as f32)
                .sum();
            let minutos = (segundos / 60.0).floor() as u32;
            let resto = (segundos % 60.0).floor() as u32;

            let (rect, respuesta) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), tema::ALTO_TARJETA_HABLANTE),
                egui::Sense::click(),
            );
            let pintor = ui.painter();
            pintor.rect_filled(
                rect,
                tema::radio_superficie(),
                if respuesta.hovered() {
                    tema::CONTROL
                } else {
                    tema::PANEL
                },
            );
            pintor.rect_stroke(
                rect,
                tema::radio_superficie(),
                Stroke::new(1.0_f32, tema::BORDE),
                egui::StrokeKind::Inside,
            );

            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                ui.set_width(rect.width());
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = tema::ESP_8;
                    let _ = egui::containers::menu::MenuButton::from_button(
                        egui::Button::new(
                            RichText::new(rod_core::color_identidad(hablante, indice))
                                .color(egui::Color32::TRANSPARENT),
                        )
                        .fill(color)
                        .stroke(Stroke::NONE)
                        .min_size(Vec2::splat(tema::TAMANO_PUNTO_GRANDE)),
                    )
                    .ui(ui, |ui| {
                        ui.set_min_width(180.0);
                        for muestra in rod_core::paleta_identidad() {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(*muestra).color(egui::Color32::TRANSPARENT),
                                    )
                                    .fill(tema::color(muestra))
                                    .stroke(Stroke::NONE)
                                    .min_size(Vec2::splat(tema::ALTO_CHIP)),
                                )
                                .clicked()
                            {
                                mensajes.push(MensajeEditor::EditarHablante(
                                    hablante.id.clone(),
                                    CampoHablante::Color,
                                    (*muestra).to_owned(),
                                ));
                                ui.close();
                            }
                        }
                    });

                    let borrador = state
                        .borradores_hablante
                        .entry(hablante.id.clone())
                        .or_insert_with(|| hablante.nombre.clone());
                    if ui
                        .add(
                            egui::TextEdit::singleline(borrador)
                                .font(tema::fuente(Estilo::Base))
                                .desired_width(rect.width() * 0.34),
                        )
                        .lost_focus()
                    {
                        mensajes.push(MensajeEditor::EditarHablante(
                            hablante.id.clone(),
                            CampoHablante::Nombre,
                            borrador.clone(),
                        ));
                    }

                    ui.label(
                        RichText::new(format!(
                            "{} · {:02}:{:02}",
                            textos.lineas(lineas.len()),
                            minutos,
                            resto
                        ))
                        .font(tema::fuente(Estilo::Pequeno))
                        .color(tema::TEXTO_3),
                    );

                    ui.label(
                        RichText::new(&hablante.tecla)
                            .font(tema::fuente(Estilo::Mono))
                            .color(tema::TEXTO_2),
                    );

                    if kit::icon_button(
                        ui,
                        Icono::Mute,
                        state.mute_hablantes.contains(&hablante.id),
                        "Mute",
                        None,
                    )
                    .clicked()
                    {
                        mensajes.push(MensajeEditor::AlternarMute(hablante.id.clone()));
                    }
                    if kit::icon_button(
                        ui,
                        Icono::Solo,
                        state.solo_hablantes.contains(&hablante.id),
                        "Solo",
                        None,
                    )
                    .clicked()
                    {
                        mensajes.push(MensajeEditor::AlternarSolo(hablante.id.clone()));
                    }

                    let preset_actual = state
                        .presets
                        .iter()
                        .find(|preset| Some(&preset.id) == hablante.preset_id.as_ref())
                        .map(|preset| preset.nombre.clone())
                        .unwrap_or_else(|| textos.sin_hablante.to_owned());
                    egui::ComboBox::from_id_salt(("estilo", &hablante.id))
                        .selected_text(tema::acortar(&preset_actual, 10))
                        .show_ui(ui, |ui| {
                            for preset in &state.presets {
                                if ui.button(&preset.nombre).clicked() {
                                    mensajes.push(MensajeEditor::AsignarPreset(
                                        hablante.id.clone(),
                                        Some(preset.id.clone()),
                                    ));
                                }
                            }
                        });

                    let _ = egui::containers::menu::MenuButton::from_button(
                        egui::Button::new("···").frame(false),
                    )
                    .ui(ui, |ui| {
                        if ui.button(textos.renombrar).clicked() {
                            ui.close();
                        }
                        if ui.button(textos.duplicar).clicked() {
                            mensajes.push(MensajeEditor::DuplicarHablante(hablante.id.clone()));
                            ui.close();
                        }
                        if ui
                            .button(textos.comando(crate::comandos::Comando::Eliminar))
                            .clicked()
                        {
                            mensajes.push(MensajeEditor::BorrarHablante(hablante.id.clone()));
                            ui.close();
                        }
                    });
                });
            });
        }
    });
    mensajes
}
