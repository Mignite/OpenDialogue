//! Pestaña Subtítulos: barra fija, lista acotada e inspector fijo abajo.
//!
//! Cada zona tiene un rect explícito: los `ScrollArea` anidados se comían el
//! alto disponible y hacían que el inspector se pintara sobre la lista.

use egui::{Event, Key, RichText, Sense, Stroke, Vec2};

use crate::comandos::Comando;
use crate::editor_app::{CampoTiempo, EditorApp, MensajeEditor};
use crate::textos::Textos;
use crate::ui::kit::{self, Icono};
use crate::ui::tema::{self, Estilo};

const ALTO_BARRA: f32 = 36.0;
const ALTO_INSPECTOR: f32 = 196.0;

pub fn mostrar(ui: &mut egui::Ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();
    let ancho = ui.available_width();

    // ── Barra superior fija ────────────────────────────────────────────────
    ui.allocate_ui_with_layout(
        Vec2::new(ancho, ALTO_BARRA),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = tema::ESP_8;
            ui.add(
                egui::TextEdit::singleline(&mut state.busqueda_subtitulos)
                    .hint_text(textos.buscar_subtitulos)
                    .font(tema::fuente(Estilo::Base))
                    .desired_width(ui.available_width() - 72.0),
            );
            if kit::icon_button(ui, Icono::Mas, false, textos.anadir_linea, None).clicked() {
                mensajes.push(MensajeEditor::Comando(Comando::AgregarFragmento));
            }
            let _ = egui::containers::menu::MenuButton::from_button(
                egui::Button::new("···").frame(false),
            )
            .ui(ui, |ui| {
                ui.set_min_width(tema::ANCHO_FILTRO);
                if ui.button(textos.importar_subtitulos).clicked() {
                    mensajes.push(MensajeEditor::Comando(Comando::CargarSrt));
                    ui.close();
                }
                if ui.button(textos.exportar_subtitulos).clicked() {
                    mensajes.push(MensajeEditor::Comando(Comando::ExportarSrt));
                    ui.close();
                }
                for hablante in &state.editor.proyecto().hablantes {
                    if ui.button(&hablante.nombre).clicked() {
                        mensajes.push(MensajeEditor::SeleccionarTodosHablante(hablante.id.clone()));
                        ui.close();
                    }
                }
            });
        },
    );
    ui.painter().line_segment(
        [
            egui::pos2(ui.max_rect().min.x, ui.cursor().top()),
            egui::pos2(ui.max_rect().max.x, ui.cursor().top()),
        ],
        Stroke::new(1.0_f32, tema::BORDE),
    );

    // ── Lista: alto calculado, nunca todo el panel ─────────────────────────
    let fondo = ui.max_rect().max.y;
    let y_lista = ui.cursor().top();
    let alto_lista = (fondo - y_lista - ALTO_INSPECTOR - tema::ESP_8).max(80.0);
    let lista = egui::Rect::from_min_max(
        egui::pos2(ui.max_rect().min.x, y_lista),
        egui::pos2(ui.max_rect().max.x, y_lista + alto_lista),
    );
    dibujar_lista(ui, lista, state, textos, &mut mensajes);

    // ── Inspector fijo ─────────────────────────────────────────────────────
    ui.allocate_rect(
        egui::Rect::from_min_max(
            egui::pos2(lista.min.x, lista.max.y),
            egui::pos2(lista.max.x, lista.max.y + tema::ESP_8),
        ),
        Sense::hover(),
    );
    let rect_inspector = egui::Rect::from_min_max(
        egui::pos2(lista.min.x, lista.max.y + tema::ESP_8),
        egui::pos2(lista.max.x, fondo),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect_inspector), |ui| {
        inspector(ui, state, textos, &mut mensajes);
    });

    mensajes
}

fn dibujar_lista(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    state: &mut EditorApp,
    textos: &Textos,
    mensajes: &mut Vec<MensajeEditor>,
) {
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
    let seleccion = state.editor.seleccion.clone();

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        if captions.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(tema::ESP_16);
                ui.label(
                    RichText::new(kit::glifo(Icono::Subir))
                        .font(egui::FontId::proportional(tema::TAMANO_ICONO_GRANDE))
                        .color(tema::TEXTO_2),
                );
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
                    mensajes.push(MensajeEditor::Comando(Comando::CargarSrt));
                }
                if ui
                    .add(egui::Button::new(textos.anadir_linea).frame(false))
                    .clicked()
                {
                    mensajes.push(MensajeEditor::Comando(Comando::AgregarFragmento));
                }
            });
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_rows(ui, tema::ALTO_FILA_SUB, captions.len(), |ui, rango| {
                for indice in rango {
                    let caption = &captions[indice];
                    let seleccionado = seleccion.contains(&caption.id);
                    let color = caption
                        .hablante_id
                        .as_deref()
                        .and_then(|id| hablantes.iter().position(|hablante| hablante.id == id))
                        .map(|i| tema::color(rod_core::color_identidad(&hablantes[i], i)))
                        .unwrap_or(tema::TEXTO_3);
                    let (fila, respuesta) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), tema::ALTO_FILA_SUB),
                        Sense::click(),
                    );
                    let pintor = ui.painter();
                    if seleccionado {
                        pintor.rect_filled(fila, tema::radio_control(), tema::SELECCION_FILA);
                        pintor.rect_stroke(
                            fila,
                            tema::radio_control(),
                            Stroke::new(2.0_f32, tema::ACENTO),
                            egui::StrokeKind::Inside,
                        );
                    }
                    pintor.rect_filled(
                        egui::Rect::from_min_max(
                            fila.min,
                            egui::pos2(fila.min.x + tema::GROSOR_BARRA_HABLANTE, fila.max.y),
                        ),
                        0.0,
                        color,
                    );

                    // El texto no debe pisar la barra de color del hablante.
                    let interior = fila.shrink2(egui::vec2(
                        tema::GROSOR_BARRA_HABLANTE + tema::ESP_4,
                        tema::ESP_2,
                    ));
                    ui.scope_builder(egui::UiBuilder::new().max_rect(interior), |ui| {
                        ui.set_width(interior.width());
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
                                        .desired_width(fila.width() - 8.0),
                                );
                                respuesta.request_focus();
                                if ui.input(|entrada| entrada.key_pressed(Key::Enter)) {
                                    mensajes.push(MensajeEditor::EditarInline(
                                        caption.id.clone(),
                                        borrador.clone(),
                                    ));
                                }
                                if ui.input(|entrada| entrada.key_pressed(Key::Escape)) {
                                    mensajes.push(MensajeEditor::CancelarInline);
                                }
                            } else {
                                let texto = tema::acortar(&caption.texto, 48);
                                let respuesta = ui.add(
                                    egui::Label::new(
                                        RichText::new(&texto)
                                            .font(tema::fuente(Estilo::Base))
                                            .color(tema::TEXTO),
                                    )
                                    .selectable(false),
                                );
                                respuesta.on_hover_text(&caption.texto);
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
                                // El lápiz se coloca en un rect explícito: un
                                // `add_space(available - ancho)` desbordaba el
                                // min_rect del SidePanel 1 px por frame.
                                let boton = egui::Rect::from_min_max(
                                    egui::pos2(
                                        fila.max.x - tema::ALTO_CHIP - tema::ESP_4,
                                        fila.max.y - tema::ALTO_CHIP - tema::ESP_2,
                                    ),
                                    egui::pos2(fila.max.x - tema::ESP_4, fila.max.y - tema::ESP_2),
                                );
                                if kit::icon_button_en(ui, boton, Icono::Editar, textos.editar)
                                    .clicked()
                                {
                                    state.inline =
                                        Some((caption.id.clone(), caption.texto.clone()));
                                }
                            }
                        });
                    });

                    if respuesta.clicked() {
                        mensajes.push(MensajeEditor::Seleccionar(caption.id.clone()));
                    }
                    if respuesta.double_clicked() {
                        state.inline = Some((caption.id.clone(), caption.texto.clone()));
                    }
                }
            });
    });
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

/// Inspector fijo: controles arriba y texto en el espacio libre de abajo.
fn inspector(
    ui: &mut egui::Ui,
    state: &mut EditorApp,
    textos: &Textos,
    mensajes: &mut Vec<MensajeEditor>,
) {
    let Some(id) = state.editor.seleccion.first().cloned() else {
        ui.add_space(tema::ESP_16);
        ui.label(
            RichText::new(textos.sin_seleccion)
                .font(tema::fuente(Estilo::Base))
                .color(tema::TEXTO_3),
        );
        return;
    };

    // Fila de controles, 28 px.
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), tema::ALTO_CONTROL),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = tema::ESP_4;
            let nombre = state
                .editor
                .proyecto()
                .captions
                .iter()
                .find(|caption| caption.id == id)
                .and_then(|caption| caption.hablante_id.clone())
                .and_then(|hablante_id| {
                    state
                        .editor
                        .proyecto()
                        .hablantes
                        .iter()
                        .enumerate()
                        .find(|(_, hablante)| hablante.id == hablante_id)
                        .map(|(indice, hablante)| {
                            (
                                hablante.nombre.clone(),
                                tema::color(rod_core::color_identidad(hablante, indice)),
                            )
                        })
                })
                .unwrap_or_else(|| (textos.sin_hablante.to_owned(), tema::TEXTO_3));
            egui::ComboBox::from_id_salt("hablante-inspector")
                .selected_text(RichText::new(nombre.0).color(nombre.1))
                .show_ui(ui, |ui| {
                    if ui.button(textos.sin_hablante).clicked() {
                        mensajes.push(MensajeEditor::AsignarHablante(None));
                    }
                    for hablante in &state.editor.proyecto().hablantes {
                        if ui.button(&hablante.nombre).clicked() {
                            mensajes
                                .push(MensajeEditor::AsignarHablante(Some(hablante.id.clone())));
                        }
                    }
                });
            if kit::icon_button(
                ui,
                Icono::Papelera,
                false,
                textos.comando(Comando::Eliminar),
                None,
            )
            .clicked()
            {
                mensajes.push(MensajeEditor::BorrarSeleccion);
            }
            if kit::icon_button(ui, Icono::Dividir, false, textos.divisor, None).clicked() {
                mensajes.push(MensajeEditor::Comando(Comando::Dividir));
            }
            if kit::icon_button(ui, Icono::Unir, false, textos.unir_siguiente, None).clicked() {
                mensajes.push(MensajeEditor::UnirConSiguiente(id.clone()));
            }
        },
    );

    // Entrada / salida con botones de un cuadro.
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = tema::ESP_4;
        tema::etiqueta(ui, textos.entrada);
        if kit::campo_tiempo(ui, &mut state.borrador_inicio, false).changed() {
            mensajes.push(MensajeEditor::EditarTiempo(
                id.clone(),
                CampoTiempo::Entrada,
                state.borrador_inicio.clone(),
            ));
        }
        tema::etiqueta(ui, textos.salida);
        if kit::campo_tiempo(ui, &mut state.borrador_fin, false).changed() {
            mensajes.push(MensajeEditor::EditarTiempo(
                id.clone(),
                CampoTiempo::Salida,
                state.borrador_fin.clone(),
            ));
        }
    });

    ui.add_space(tema::ESP_8);
    // Texto: Enter termina la edición, Shift+Enter añade línea.
    let id_texto = egui::Id::new(("inspector-texto", &id));
    let enter_fin = ui.input(|entrada| {
        !entrada.modifiers.shift
            && entrada.events.iter().any(|evento| {
                matches!(
                    evento,
                    Event::Key {
                        key: Key::Enter,
                        pressed: true,
                        ..
                    }
                )
            })
    });
    if enter_fin {
        ui.input_mut(|entrada| {
            entrada.events.retain(|evento| {
                !matches!(
                    evento,
                    Event::Key {
                        key: Key::Enter,
                        ..
                    }
                )
            })
        });
        ui.memory_mut(|memoria| memoria.surrender_focus(id_texto));
    }
    let ancho = ui.available_width();
    let alto = (ui.available_height() - 28.0).max(40.0);
    ui.allocate_ui_with_layout(
        Vec2::new(ancho, alto),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            let respuesta = ui.add(
                egui::TextEdit::multiline(&mut state.borrador)
                    .id(id_texto)
                    .hint_text("Texto del subtítulo")
                    .font(tema::fuente(Estilo::Cuerpo))
                    .desired_width(ancho - 8.0),
            );
            if respuesta.changed() {
                mensajes.push(MensajeEditor::EditarEnVivo(
                    id.clone(),
                    state.borrador.clone(),
                ));
            }
            if respuesta.lost_focus() {
                mensajes.push(MensajeEditor::TerminarEdicionTexto);
            }
        },
    );

    let inicio = rod_core::parse_time_input(&state.borrador_inicio).unwrap_or(0.0);
    let fin = rod_core::parse_time_input(&state.borrador_fin).unwrap_or(inicio);
    let duracion = fin - inicio;
    let caracteres = state.borrador.chars().count();
    let cps = if duracion > 0.0 {
        caracteres as f64 / duracion
    } else {
        0.0
    } as f32;
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
}
