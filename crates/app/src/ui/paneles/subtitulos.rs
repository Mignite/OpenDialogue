//! Lista virtualizada de subtítulos con edición inline.

use egui::{RichText, Sense, Vec2};

use crate::editor_app::{CampoTiempo, EditorApp, MensajeEditor};
use crate::textos::Textos;
use crate::ui::kit;
use crate::ui::tema::{self, Estilo};

/// Solo la lista; el inspector se añade en la tarea siguiente.
pub fn mostrar(ui: &mut egui::Ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor> {
    let _ = textos;
    let mut mensajes = Vec::new();
    let total = state.editor.proyecto().captions.len();
    let seleccion = state.editor.seleccion.clone();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, tema::ALTO_FILA_SUB, total, |ui, rango| {
            for indice in rango {
                let caption = state.editor.proyecto().captions[indice].clone();
                let seleccionado = seleccion.contains(&caption.id);
                let color = caption
                    .hablante_id
                    .as_deref()
                    .and_then(|id| {
                        state
                            .editor
                            .proyecto()
                            .hablantes
                            .iter()
                            .position(|h| h.id == id)
                    })
                    .map(|i| {
                        tema::color(rod_core::color_identidad(
                            &state.editor.proyecto().hablantes[i],
                            i,
                        ))
                    })
                    .unwrap_or(tema::TEXTO_3);

                let (rect, _) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), tema::ALTO_FILA_SUB),
                    Sense::click(),
                );
                let pintor = ui.painter();
                if seleccionado {
                    pintor.rect_filled(
                        rect,
                        egui::CornerRadius::same(tema::RADIO_CONTROL as u8),
                        tema::ACENTO.gamma_multiply(0.18),
                    );
                }
                pintor.rect_filled(
                    egui::Rect::from_min_max(rect.min, egui::pos2(rect.min.x + 3.0, rect.max.y)),
                    0.0,
                    color,
                );

                ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                    ui.set_width(rect.width());
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = tema::ESP_2;
                        if let Some((id, borrador)) = state.inline.as_mut()
                            && *id == caption.id
                        {
                            let respuesta = ui.add(
                                egui::TextEdit::singleline(borrador)
                                    .font(tema::fuente(Estilo::Base))
                                    .desired_width(rect.width() - 8.0),
                            );
                            if ui.input(|entrada| entrada.key_pressed(egui::Key::Enter)) {
                                mensajes.push(MensajeEditor::EditarInline(
                                    caption.id.clone(),
                                    borrador.clone(),
                                ));
                            }
                            let escape = ui.input(|entrada| {
                                entrada.events.iter().any(|evento| {
                                    matches!(
                                        evento,
                                        egui::Event::Key {
                                            key: egui::Key::Escape,
                                            pressed: true,
                                            ..
                                        }
                                    )
                                })
                            });
                            if escape || respuesta.lost_focus() {
                                mensajes.push(MensajeEditor::CancelarInline);
                            }
                        } else {
                            ui.label(
                                RichText::new(&caption.texto)
                                    .font(tema::fuente(Estilo::Cuerpo))
                                    .color(tema::TEXTO),
                            );
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = tema::ESP_4;
                                ui.label(
                                    RichText::new(rod_core::format_time(caption.inicio))
                                        .font(tema::fuente(Estilo::Timecode))
                                        .color(tema::TEXTO_3),
                                );
                                ui.label(
                                    RichText::new(crate::editor_app::duracion_texto(
                                        caption.inicio,
                                        caption.fin,
                                    ))
                                    .font(tema::fuente(Estilo::Pequeno))
                                    .color(tema::TEXTO_3),
                                );
                            });
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Editar").font(tema::fuente(Estilo::Pequeno)),
                                    )
                                    .frame(false),
                                )
                                .clicked()
                            {
                                state.inline = Some((caption.id.clone(), caption.texto.clone()));
                            }
                        }
                    });
                });

                if state.inline.is_none() {
                    let sobre = rect.contains(
                        ui.ctx()
                            .input(|entrada| entrada.pointer.latest_pos())
                            .unwrap_or_default(),
                    );
                    if sobre {
                        let doble = ui.ctx().input(|entrada| {
                            entrada
                                .pointer
                                .button_double_clicked(egui::PointerButton::Primary)
                        });
                        if doble {
                            state.inline = Some((caption.id.clone(), caption.texto.clone()));
                        } else if ui.ctx().input(|entrada| {
                            entrada.pointer.button_clicked(egui::PointerButton::Primary)
                        }) {
                            mensajes.push(MensajeEditor::Seleccionar(caption.id.clone()));
                        }
                    }
                }
            }
        });

    ui.add_space(tema::ESP_8);
    ui.separator();
    mensajes.extend(inspector(ui, state, textos));
    mensajes
}

/// Inspector del caption seleccionado, bajo la lista.
pub fn inspector(ui: &mut egui::Ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();
    let Some(id) = state.editor.seleccion.first().cloned() else {
        kit::recuadro_punteado(ui, 72.0, textos.sin_seleccion);
        return mensajes;
    };

    ui.label(
        RichText::new(textos.nombre)
            .font(tema::fuente(Estilo::Pequeno))
            .color(tema::TEXTO_3),
    );
    let respuesta = kit::campo_texto(ui, &mut state.borrador, 160.0);
    if ui
        .add(
            egui::Button::new(RichText::new(textos.aplicar).font(tema::fuente(Estilo::Base)))
                .frame(false),
        )
        .clicked()
        || (respuesta.changed() && ui.input(|entrada| entrada.key_pressed(egui::Key::Enter)))
    {
        mensajes.push(MensajeEditor::AplicarTexto);
    }

    ui.horizontal(|ui| {
        let hablantes = state.editor.proyecto().hablantes.clone();
        let actual = state
            .editor
            .proyecto()
            .captions
            .iter()
            .find(|caption| caption.id == id)
            .and_then(|caption| caption.hablante_id.clone());
        let nombre = actual
            .as_deref()
            .and_then(|aid| hablantes.iter().find(|h| h.id == aid))
            .map(|h| h.nombre.clone())
            .unwrap_or_else(|| textos.sin_hablante.to_owned());
        egui::ComboBox::from_id_salt("hablante-inspector")
            .selected_text(nombre)
            .show_ui(ui, |ui| {
                if ui.button(textos.sin_hablante).clicked() {
                    mensajes.push(MensajeEditor::AsignarHablante(None));
                }
                for hablante in &hablantes {
                    if ui.button(&hablante.nombre).clicked() {
                        mensajes.push(MensajeEditor::AsignarHablante(Some(hablante.id.clone())));
                    }
                }
            });
        if ui
            .add(
                egui::Button::new(
                    RichText::new(textos.comando(crate::comandos::Comando::Eliminar))
                        .font(tema::fuente(Estilo::Base)),
                )
                .frame(false),
            )
            .clicked()
        {
            mensajes.push(MensajeEditor::BorrarSeleccion);
        }
    });

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = tema::ESP_4;
        ui.label(RichText::new(textos.entrada).font(tema::fuente(Estilo::Pequeno)));
        if kit::campo_tiempo(ui, &mut state.borrador_inicio, false).changed() {
            let valor = state.borrador_inicio.clone();
            mensajes.push(MensajeEditor::EditarTiempo(
                id.clone(),
                CampoTiempo::Entrada,
                valor,
            ));
        }
        ui.label(RichText::new(textos.salida).font(tema::fuente(Estilo::Pequeno)));
        if kit::campo_tiempo(ui, &mut state.borrador_fin, false).changed() {
            let valor = state.borrador_fin.clone();
            mensajes.push(MensajeEditor::EditarTiempo(
                id.clone(),
                CampoTiempo::Salida,
                valor,
            ));
        }
    });
    mensajes
}
