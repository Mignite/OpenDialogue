//! Panel de estilos: tarjetas con vista previa viva y editor inline.

use egui::{Color32, RichText, Stroke, Vec2};

use crate::editor_app::{EditorApp, MensajeEditor};
use crate::presets_edicion::{CampoPreset, valor_campo};
use crate::textos::Textos;
use crate::ui::kit::{self, Icono};
use crate::ui::tema::{self, Estilo};

pub fn mostrar(ui: &mut egui::Ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();

    if ui
        .add(
            egui::Button::new(
                RichText::new(format!(
                    "{}  {}",
                    kit::glifo(Icono::Mas),
                    textos.nuevo_preset
                ))
                .font(tema::fuente(Estilo::Base)),
            )
            .frame(false),
        )
        .clicked()
    {
        mensajes.push(MensajeEditor::NuevoPreset);
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        let presets = state.presets.clone();
        for preset in &presets {
            let abierto = state.preset_seleccionado.as_deref() == Some(&preset.id);
            let (rect, respuesta) = ui
                .allocate_exact_size(Vec2::new(ui.available_width(), 104.0), egui::Sense::click());
            let pintor = ui.painter();
            pintor.rect_filled(
                rect,
                tema::radio_superficie(),
                if abierto { tema::CONTROL } else { tema::PANEL },
            );
            pintor.rect_stroke(
                rect,
                tema::radio_superficie(),
                Stroke::new(1.0_f32, tema::BORDE),
                egui::StrokeKind::Inside,
            );

            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                ui.set_width(rect.width());
                ui.vertical(|ui| {
                    ui.add_space(tema::ESP_4);
                    vista_previa(ui, preset);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&preset.nombre)
                                .font(tema::fuente(Estilo::Base))
                                .color(tema::TEXTO),
                        );
                        for (indice, hablante) in
                            state.editor.proyecto().hablantes.iter().enumerate()
                        {
                            if hablante.preset_id.as_deref() == Some(&preset.id) {
                                kit::punto(
                                    ui,
                                    tema::color(rod_core::color_identidad(hablante, indice)),
                                    tema::TAMANO_PUNTO,
                                );
                            }
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let giro = ui.ctx().animate_bool_with_time(
                                egui::Id::new(("chevron", &preset.id)),
                                abierto,
                                tema::ANIMACION,
                            );
                            ui.label(
                                RichText::new(kit::glifo(Icono::ChevronAbajo))
                                    .font(egui::FontId::proportional(
                                        tema::TAMANO_ICONO * (1.0 + giro * 0.15),
                                    ))
                                    .color(tema::TEXTO_2),
                            );
                        });
                    });
                });
            });

            if respuesta.clicked() {
                state.preset_seleccionado = Some(preset.id.clone());
                mensajes.push(MensajeEditor::SeleccionarPreset(preset.id.clone()));
            }
            if abierto {
                editor_preset(ui, state, preset, textos, &mut mensajes);
            }
        }
    });
    mensajes
}

/// Texto de muestra con los atributos visuales del preset.
fn vista_previa(ui: &mut egui::Ui, preset: &rod_core::PresetAss) {
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 44.0), egui::Sense::hover());
    let pintor = ui.painter();
    pintor.rect_filled(rect, 0.0, Color32::from_rgb(0x16, 0x16, 0x16));
    let fuente = egui::FontId::new(
        (preset.fontsize as f32).clamp(11.0, 36.0),
        egui::FontFamily::Proportional,
    );
    let centro = rect.center();
    if preset.shadow > 0.0 {
        pintor.text(
            centro + Vec2::new(preset.shadow as f32, preset.shadow as f32),
            egui::Align2::CENTER_CENTER,
            "Aa Hola mundo",
            fuente.clone(),
            Color32::from_black_alpha(160),
        );
    }
    if preset.outline > 0.0 {
        for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
            pintor.text(
                centro + Vec2::new(dx, dy) * preset.outline as f32,
                egui::Align2::CENTER_CENTER,
                "Aa Hola mundo",
                fuente.clone(),
                tema::color(&preset.outline_color),
            );
        }
    }
    pintor.text(
        centro,
        egui::Align2::CENTER_CENTER,
        "Aa Hola mundo",
        fuente,
        tema::color(&preset.color),
    );
}

fn editor_preset(
    ui: &mut egui::Ui,
    state: &mut EditorApp,
    preset: &rod_core::PresetAss,
    textos: &Textos,
    mensajes: &mut Vec<MensajeEditor>,
) {
    ui.add_space(tema::ESP_8);
    if ui
        .add(
            egui::Button::new(textos.aplicar_a_seleccion)
                .frame(true)
                .corner_radius(tema::radio_superficie()),
        )
        .clicked()
    {
        mensajes.push(MensajeEditor::AplicarPresetSeleccion(preset.id.clone()));
    }

    let campos = [
        (CampoPreset::Nombre, textos.nombre),
        (CampoPreset::Fontname, textos.fuente),
        (CampoPreset::Fontsize, textos.tamano),
        (CampoPreset::Color, textos.color),
        (CampoPreset::Outline, textos.contorno),
        (CampoPreset::OutlineColor, textos.color),
        (CampoPreset::Shadow, textos.sombra),
        (CampoPreset::MarginL, textos.posicion),
        (CampoPreset::Bold, textos.negrita),
        (CampoPreset::Italic, textos.cursiva),
    ];
    for (campo, etiqueta) in campos {
        tema::etiqueta(ui, etiqueta);
        let valor = valor_campo(preset, campo, &state.borradores_preset);
        let clave = (preset.id.clone(), campo);
        let borrador = state.borradores_preset.entry(clave).or_insert(valor);
        if ui
            .add(egui::TextEdit::singleline(borrador).font(tema::fuente(Estilo::Base)))
            .lost_focus()
        {
            mensajes.push(MensajeEditor::EditarPreset(
                preset.id.clone(),
                campo,
                borrador.clone(),
            ));
        }
    }
    ui.add_space(tema::ESP_12);
}
