//! Panel de hablantes: identidad de color, preset de texto y borrado.

use egui::{Color32, RichText, Stroke, Vec2};

use crate::editor::MAX_HABLANTES;
use crate::editor_app::{CampoHablante, EditorApp, MensajeEditor, Panel};
use crate::ui::kit::{self, Icono};
use crate::ui::tema::{self, Estilo};

pub fn mostrar(ui: &mut egui::Ui, state: &mut EditorApp) -> Vec<MensajeEditor> {
    // Hasta que el editor pase `&Textos` a todas las vistas (fase 3).
    let textos = crate::textos::Idioma::default().textos();
    let mut mensajes = Vec::new();

    let hablantes = state.editor.proyecto().hablantes.clone();
    let captions = state.editor.proyecto().captions.clone();
    for (indice, hablante) in hablantes.iter().enumerate() {
        let color = tema::color(rod_core::color_identidad(hablante, indice));
        let (_, _) = kit::tarjeta(ui, |ui| {
            ui.horizontal(|ui| {
                kit::punto(ui, color, tema::ESP_8);
                let borrador = state
                    .borradores_hablante
                    .entry(hablante.id.clone())
                    .or_insert_with(|| hablante.nombre.clone());
                let respuesta =
                    ui.add(egui::TextEdit::singleline(borrador).font(tema::fuente(Estilo::Base)));
                if respuesta.lost_focus() {
                    mensajes.push(MensajeEditor::EditarHablante(
                        hablante.id.clone(),
                        CampoHablante::Nombre,
                        borrador.clone(),
                    ));
                }
                ui.label(
                    RichText::new(&hablante.tecla)
                        .font(tema::fuente(Estilo::Mono))
                        .color(tema::TEXTO_3),
                );
                if kit::icon_button(ui, Icono::Papelera, false, textos.borrar_hablante, None)
                    .clicked()
                {
                    mensajes.push(MensajeEditor::BorrarHablante(hablante.id.clone()));
                }
            });

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(tema::ESP_4);
                for muestra in rod_core::paleta_identidad() {
                    let usado = hablantes
                        .iter()
                        .enumerate()
                        .any(|(otro, h)| otro != indice && h.color.eq_ignore_ascii_case(muestra));
                    let elegido =
                        rod_core::color_identidad(hablante, indice).eq_ignore_ascii_case(muestra);
                    let respuesta = ui.add_enabled(
                        !usado,
                        egui::Button::new(RichText::new(*muestra).color(Color32::TRANSPARENT))
                            .fill(tema::color(muestra))
                            .stroke(Stroke::new(
                                if elegido { 2.0_f32 } else { 1.0_f32 },
                                if elegido {
                                    tema::TEXTO
                                } else {
                                    tema::BORDE_CONTROL
                                },
                            ))
                            .min_size(Vec2::splat(tema::ALTO_CHIP)),
                    );
                    if respuesta.clicked() {
                        mensajes.push(MensajeEditor::EditarHablante(
                            hablante.id.clone(),
                            CampoHablante::Color,
                            (*muestra).to_owned(),
                        ));
                    }
                }
            });

            ui.horizontal(|ui| {
                kit::chip_hablante(ui, &hablante.nombre, color);
                let color_texto = rod_core::color_hablante(hablante, &state.presets);
                ui.label(
                    RichText::new("●")
                        .font(tema::fuente(Estilo::Base))
                        .color(tema::color(&color_texto)),
                );
                ui.label(
                    RichText::new(textos.color_subtitulo)
                        .font(tema::fuente(Estilo::Pequeno))
                        .color(tema::TEXTO_3),
                );
                if let Some(id) = hablante.preset_id.as_deref()
                    && ui
                        .add(egui::Button::new(
                            RichText::new(id).font(tema::fuente(Estilo::Pequeno)),
                        ))
                        .clicked()
                {
                    mensajes.push(MensajeEditor::Panel(Panel::Estilos));
                    mensajes.push(MensajeEditor::SeleccionarPreset(id.to_owned()));
                }
                let lineas = captions
                    .iter()
                    .filter(|caption| caption.hablante_id.as_deref() == Some(&hablante.id))
                    .count();
                ui.label(
                    RichText::new(textos.lineas(lineas))
                        .font(tema::fuente(Estilo::Pequeno))
                        .color(tema::TEXTO_3),
                );
            });
        });
    }

    if state.editor.proyecto().hablantes.len() < MAX_HABLANTES
        && kit::boton_secundario(ui, textos.agregar_hablante, Icono::Mas).clicked()
    {
        mensajes.push(MensajeEditor::AgregarHablante);
    }
    if ui
        .add(
            egui::Button::new(
                RichText::new(textos.quitar_de_la_seleccion).font(tema::fuente(Estilo::Base)),
            )
            .frame(false),
        )
        .clicked()
    {
        mensajes.push(MensajeEditor::AsignarHablante(None));
    }
    mensajes
}
