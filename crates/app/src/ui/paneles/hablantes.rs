//! Panel de hablantes: tarjetas compactas con color, atajo, estilo y mute/solo.

use std::collections::HashMap;
use std::sync::Arc;

use egui::{RichText, Stroke, Vec2};

use crate::editor::MAX_HABLANTES;
use crate::editor_app::{CampoHablante, EditorApp, MensajeEditor};
use crate::textos::Textos;
use crate::ui::kit::{self, Icono};
use crate::ui::tema::{self, Estilo};

#[derive(Clone, Copy, Default)]
struct Conteo {
    lineas: usize,
    segundos: f64,
}

#[derive(Clone, Default)]
struct Contadores {
    firma: u64,
    por_hablante: Arc<HashMap<String, Conteo>>,
}

/// Huella de lo que afecta a los contadores. Recorrer los captions para hacer
/// este hash es O(captions), pero evita el producto hablantes × captions y
/// cualquier clonación del proyecto en cada frame.
fn firma_captions(proyecto: &rod_core::Proyecto) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    proyecto.captions.len().hash(&mut hasher);
    for caption in &proyecto.captions {
        caption.hablante_id.hash(&mut hasher);
        caption.inicio.to_bits().hash(&mut hasher);
        caption.fin.to_bits().hash(&mut hasher);
    }
    for hablante in &proyecto.hablantes {
        hablante.id.hash(&mut hasher);
    }
    hasher.finish()
}

/// Contadores agrupados de una pasada, recalculados solo cuando cambia el
/// proyecto. Viven en la memoria de egui porque la vista no puede añadir
/// campos al modelo sin tocar otros archivos.
fn contadores(ui: &egui::Ui, state: &EditorApp) -> Arc<HashMap<String, Conteo>> {
    let firma = firma_captions(state.editor.proyecto());
    let id = egui::Id::new("contadores-hablantes");
    ui.memory_mut(|memoria| {
        let cache = memoria.data.get_temp_mut_or(id, Contadores::default());
        if cache.firma != firma {
            let mut por_hablante: HashMap<String, Conteo> = HashMap::new();
            for caption in &state.editor.proyecto().captions {
                if let Some(id) = &caption.hablante_id {
                    let entrada = por_hablante.entry(id.clone()).or_default();
                    entrada.lineas += 1;
                    entrada.segundos += (caption.fin - caption.inicio).max(0.0);
                }
            }
            cache.firma = firma;
            cache.por_hablante = Arc::new(por_hablante);
        }
        Arc::clone(&cache.por_hablante)
    })
}

pub fn mostrar(ui: &mut egui::Ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();
    let conteos = contadores(ui, state);

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
        for (indice, hablante) in state.editor.proyecto().hablantes.iter().enumerate() {
            let color = tema::color(rod_core::color_identidad(hablante, indice));
            let conteo = conteos.get(&hablante.id).copied().unwrap_or_default();
            let minutos = (conteo.segundos / 60.0).floor() as u32;
            let resto = (conteo.segundos % 60.0).floor() as u32;

            // 60 px de contenido en dos filas + ESP_12 arriba y abajo.
            let altura = tema::ALTO_TARJETA_HABLANTE + tema::ESP_8;
            let (rect, respuesta) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), altura),
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
                Stroke::new(tema::GROSOR_BORDE, tema::BORDE),
                egui::StrokeKind::Inside,
            );

            let interior = rect.shrink(tema::ESP_12);
            ui.scope_builder(egui::UiBuilder::new().max_rect(interior), |ui| {
                ui.set_width(interior.width());
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = tema::ESP_8;

                        // Botón sin texto: el hex sigue siendo su etiqueta
                        // accesible, pero no ensancha el punto circular.
                        let (respuesta_color, _) = egui::containers::menu::MenuButton::from_button(
                            egui::Button::new("")
                                .frame(false)
                                .small()
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(Stroke::NONE)
                                .min_size(Vec2::splat(tema::TAMANO_PUNTO_GRANDE)),
                        )
                        .ui(ui, |ui| {
                            ui.set_min_width(180.0);
                            for muestra in rod_core::paleta_identidad() {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(*muestra)
                                                .color(egui::Color32::TRANSPARENT),
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
                        respuesta_color.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::Button,
                                true,
                                rod_core::color_identidad(hablante, indice),
                            )
                        });
                        let centro = respuesta_color.rect.center();
                        let radio = respuesta_color.rect.width() * 0.5;
                        ui.painter().circle_filled(centro, radio, color);
                        ui.painter().circle_stroke(
                            centro,
                            radio,
                            Stroke::new(tema::GROSOR_BORDE, tema::BORDE_CONTROL),
                        );
                        if respuesta_color.has_focus() {
                            kit::anillo_foco(ui, &respuesta_color);
                        }

                        let borrador = state
                            .borradores_hablante
                            .entry(hablante.id.clone())
                            .or_insert_with(|| hablante.nombre.clone());
                        if ui
                            .add(
                                egui::TextEdit::singleline(borrador)
                                    .font(tema::fuente(Estilo::Base))
                                    .desired_width(interior.width() * 0.28),
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
                                textos.lineas(conteo.lineas),
                                minutos,
                                resto
                            ))
                            .font(tema::fuente(Estilo::Pequeno))
                            .color(tema::TEXTO_3),
                        );

                        kit::keycap(ui, &hablante.tecla);

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
                    // Segunda línea: mute/solo y el selector de estilo a todo
                    // el ancho. Antes compartían fila con nombre y contador y
                    // se salían del panel estrecho.
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = tema::ESP_8;
                        if kit::icon_button(
                            ui,
                            Icono::Mute,
                            state.mute_hablantes.contains(&hablante.id),
                            "Mute",
                            Some(textos.sin_atajo),
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
                            Some(textos.sin_atajo),
                        )
                        .clicked()
                        {
                            mensajes.push(MensajeEditor::AlternarSolo(hablante.id.clone()));
                        }
                        let nombre_preset = state
                            .presets
                            .iter()
                            .find(|preset| Some(&preset.id) == hablante.preset_id.as_ref())
                            .map(|preset| preset.nombre.clone());
                        let texto = match &nombre_preset {
                            Some(nombre) => tema::acortar(nombre, 14),
                            None => textos.sin_estilo.to_owned(),
                        };
                        let respuesta = egui::ComboBox::from_id_salt(("estilo", &hablante.id))
                            .width((ui.available_width() - tema::ANCHO_BOTON_28).max(60.0))
                            .selected_text(texto)
                            .show_ui(ui, |ui| {
                                if ui.button(textos.sin_estilo).clicked() {
                                    mensajes.push(MensajeEditor::AsignarPreset(
                                        hablante.id.clone(),
                                        None,
                                    ));
                                    ui.close();
                                }
                                for preset in &state.presets {
                                    if ui.button(&preset.nombre).clicked() {
                                        mensajes.push(MensajeEditor::AsignarPreset(
                                            hablante.id.clone(),
                                            Some(preset.id.clone()),
                                        ));
                                        ui.close();
                                    }
                                }
                            });
                        if respuesta.response.has_focus() {
                            kit::anillo_foco(ui, &respuesta.response);
                        }
                    });
                });
            });
        }
    });
    mensajes
}
