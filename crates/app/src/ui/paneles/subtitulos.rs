//! Lista virtualizada de subtítulos con edición inline.

use egui::{RichText, Sense, Vec2};

use crate::editor_app::{CampoTiempo, EditorApp, MensajeEditor};
use crate::textos::Textos;
use crate::ui::kit::{self, Icono};
use crate::ui::tema::{self, Estilo};

/// Solo la lista; el inspector se añade en la tarea siguiente.
pub fn mostrar(ui: &mut egui::Ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();
    let filtro = state.busqueda_subtitulos.to_lowercase();
    let hablantes = state.editor.proyecto().hablantes.clone();
    let captions: Vec<_> = state
        .editor
        .proyecto()
        .captions
        .iter()
        .filter(|caption| {
            filtro.is_empty()
                || caption.texto.to_lowercase().contains(&filtro)
                || hablantes
                    .iter()
                    .find(|hablante| Some(&hablante.id) == caption.hablante_id.as_ref())
                    .is_some_and(|hablante| hablante.nombre.to_lowercase().contains(&filtro))
        })
        .cloned()
        .collect();
    let total = captions.len();
    let seleccion = state.editor.seleccion.clone();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = tema::ESP_8;
        ui.add(
            egui::TextEdit::singleline(&mut state.busqueda_subtitulos)
                .hint_text(textos.buscar_subtitulos)
                .font(tema::fuente(Estilo::Base))
                .desired_width(ui.available_width() - 72.0),
        );
        if kit::icon_button(ui, Icono::Mas, false, textos.anadir_linea, None).clicked() {
            mensajes.push(MensajeEditor::Comando(
                crate::comandos::Comando::AgregarFragmento,
            ));
        }
        let _ =
            egui::containers::menu::MenuButton::from_button(egui::Button::new("···").frame(false))
                .ui(ui, |ui| {
                    if ui.button(textos.importar_subtitulos).clicked() {
                        mensajes.push(MensajeEditor::Comando(crate::comandos::Comando::CargarSrt));
                        ui.close();
                    }
                    if ui.button(textos.exportar_subtitulos).clicked() {
                        mensajes.push(MensajeEditor::Comando(
                            crate::comandos::Comando::ExportarSrt,
                        ));
                        ui.close();
                    }
                    for hablante in &state.editor.proyecto().hablantes {
                        if ui.button(&hablante.nombre).clicked() {
                            mensajes
                                .push(MensajeEditor::SeleccionarTodosHablante(hablante.id.clone()));
                            ui.close();
                        }
                    }
                });
    });

    if total == 0 && filtro.is_empty() {
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(kit::glifo(Icono::Subir))
                    .font(egui::FontId::proportional(tema::TAMANO_ICONO_GRANDE))
                    .color(tema::TEXTO_2),
            );
            ui.add_space(tema::ESP_8);
            ui.label(RichText::new(textos.sin_subtitulos).font(tema::fuente(Estilo::Base)));
            ui.add_space(tema::ESP_8);
            if ui
                .add(
                    egui::Button::new(textos.importar_archivo)
                        .fill(tema::PRIMARIO)
                        .corner_radius(tema::radio_superficie()),
                )
                .clicked()
            {
                mensajes.push(MensajeEditor::Comando(crate::comandos::Comando::CargarSrt));
            }
            if ui
                .add(egui::Button::new(textos.anadir_linea).frame(false))
                .clicked()
            {
                mensajes.push(MensajeEditor::Comando(
                    crate::comandos::Comando::AgregarFragmento,
                ));
            }
        });
        return mensajes;
    }

    let alto_lista = (ui.available_height() * 0.55).max(160.0);
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), alto_lista),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(ui, tema::ALTO_FILA_SUB, total, |ui, rango| {
                    for indice in rango {
                        let caption = captions[indice].clone();
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
                                tema::SELECCION_FILA,
                            );
                        }
                        pintor.rect_filled(
                            egui::Rect::from_min_max(
                                rect.min,
                                egui::pos2(rect.min.x + tema::GROSOR_BARRA_HABLANTE, rect.max.y),
                            ),
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
                                    let id_campo = egui::Id::new(("inline", &caption.id));
                                    let respuesta = ui.add(
                                        egui::TextEdit::singleline(borrador)
                                            .id(id_campo)
                                            .font(tema::fuente(Estilo::Base))
                                            .desired_width(rect.width() - 8.0),
                                    );
                                    // Sin foco, las teclas desnudas las consume el
                                    // enrutador de atajos y no llegan al texto.
                                    respuesta.request_focus();
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
                                    if escape
                                        || ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new(textos.cancelar)
                                                        .font(tema::fuente(Estilo::Pequeno)),
                                                )
                                                .frame(false),
                                            )
                                            .clicked()
                                    {
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
                                            RichText::new(format!(
                                                "{} → {}",
                                                tiempo_corto(caption.inicio),
                                                tiempo_corto(caption.fin)
                                            ))
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
                                                RichText::new(textos.editar)
                                                    .font(tema::fuente(Estilo::Pequeno)),
                                            )
                                            .frame(false),
                                        )
                                        .clicked()
                                    {
                                        state.inline =
                                            Some((caption.id.clone(), caption.texto.clone()));
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
                                    state.inline =
                                        Some((caption.id.clone(), caption.texto.clone()));
                                } else if ui.ctx().input(|entrada| {
                                    entrada.pointer.button_clicked(egui::PointerButton::Primary)
                                }) {
                                    mensajes.push(MensajeEditor::Seleccionar(caption.id.clone()));
                                }
                            }
                        }
                    }
                });
        },
    );
    ui.add_space(tema::ESP_8);
    ui.separator();
    mensajes.extend(inspector(ui, state, textos));
    mensajes
}

/// `mm:ss.cc` para la segunda línea de cada fila.
fn tiempo_corto(segundos: f64) -> String {
    let centesimas = (segundos * 100.0).round().max(0.0) as u64;
    format!(
        "{:02}:{:02}.{:02}",
        centesimas / 6000,
        (centesimas / 100) % 60,
        centesimas % 100
    )
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
    if respuesta.changed() {
        mensajes.push(MensajeEditor::EditarEnVivo(
            id.clone(),
            state.borrador.clone(),
        ));
    }
    if respuesta.lost_focus() {
        mensajes.push(MensajeEditor::TerminarEdicionTexto);
    }
    let duracion = (state.borrador_fin.parse::<f64>().unwrap_or(0.0)
        - state.borrador_inicio.parse::<f64>().unwrap_or(0.0))
    .max(0.001);
    let caracteres = state.borrador.chars().count();
    let cps = (caracteres as f64 / duracion) as f32;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{caracteres} · {cps:.1} CPS"))
                .font(tema::fuente(Estilo::Pequeno))
                .color(
                    if cps > tema::CPS_MAXIMO || caracteres > tema::CARACTERES_MAXIMOS {
                        tema::AMBAR
                    } else {
                        tema::TEXTO_3
                    },
                ),
        );
    });

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
        if kit::icon_button(
            ui,
            Icono::Papelera,
            false,
            textos.comando(crate::comandos::Comando::Eliminar),
            None,
        )
        .clicked()
        {
            mensajes.push(MensajeEditor::BorrarSeleccion);
        }
        if kit::icon_button(ui, Icono::Dividir, false, textos.divisor, None).clicked() {
            mensajes.push(MensajeEditor::Comando(crate::comandos::Comando::Dividir));
        }
        if kit::icon_button(ui, Icono::Unir, false, textos.unir_siguiente, None).clicked() {
            mensajes.push(MensajeEditor::UnirConSiguiente(id.clone()));
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
