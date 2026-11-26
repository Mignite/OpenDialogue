//! Pestaña Subtítulos: búsqueda y lista a todo el alto.
//! El texto se edita en la barra bajo el video (`ui/texto.rs`).

use egui::{RichText, Sense, Stroke, Vec2};

use crate::comandos::Comando;
use crate::editor_app::{EditorApp, MensajeEditor};
use crate::textos::Textos;
use crate::ui::kit::{self, Icono};
use crate::ui::tema::{self, Estilo};

const ALTO_BARRA: f32 = 36.0;

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
                egui::Button::new("···")
                    .frame(false)
                    .min_size(Vec2::splat(tema::ANCHO_BOTON_28)),
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

    // ── Lista a todo el alto restante ──────────────────────────────────────
    let fondo = ui.max_rect().max.y;
    let y_lista = ui.cursor().top();
    let lista = egui::Rect::from_min_max(
        egui::pos2(ui.max_rect().min.x, y_lista),
        egui::pos2(ui.max_rect().max.x, fondo.max(y_lista + 80.0)),
    );
    dibujar_lista(ui, lista, state, textos, &mut mensajes);

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
                            let texto = tema::acortar(&caption.texto, 48);
                            let respuesta = ui.add(
                                egui::Label::new(
                                    RichText::new(&texto)
                                        .font(tema::fuente(Estilo::Base))
                                        .color(tema::TEXTO),
                                )
                                .selectable(false),
                            );
                            if respuesta.hovered() {
                                respuesta.on_hover_text(&caption.texto);
                            }
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = tema::ESP_4;
                                let tiempos = ui
                                    .add(
                                        egui::Label::new(
                                            RichText::new(format!(
                                                "{} → {}",
                                                tiempo_corto(caption.inicio),
                                                tiempo_corto(caption.fin)
                                            ))
                                            .font(tema::fuente(Estilo::Timecode))
                                            .color(tema::TEXTO_3),
                                        )
                                        .sense(Sense::click()),
                                    )
                                    .on_hover_text("Doble clic: fijar entrada");
                                let id = caption.id.clone();
                                if ui.rect_contains_pointer(tiempos.rect) {
                                    tiempos.context_menu(|ui| {
                                        if ui.button(textos.comando(Comando::Entrada)).clicked() {
                                            mensajes.push(MensajeEditor::Seleccionar(id.clone()));
                                            mensajes.push(MensajeEditor::Comando(Comando::Entrada));
                                            ui.close();
                                        }
                                        if ui.button(textos.comando(Comando::Salida)).clicked() {
                                            mensajes.push(MensajeEditor::Seleccionar(id.clone()));
                                            mensajes.push(MensajeEditor::Comando(Comando::Salida));
                                            ui.close();
                                        }
                                    });
                                }
                                ui.label(
                                    RichText::new(crate::editor_app::duracion_texto(
                                        caption.inicio,
                                        caption.fin,
                                    ))
                                    .font(tema::fuente(Estilo::Pequeno))
                                    .color(tema::TEXTO_3),
                                );
                            });
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
                                mensajes.push(MensajeEditor::Seleccionar(caption.id.clone()));
                                mensajes.push(MensajeEditor::Comando(Comando::EditarTexto));
                            }
                            // Basura junto al lápiz: borra esta línea sin
                            // cambiar la selección ni pedir confirmación.
                            let x1 = fila.max.x - tema::ALTO_CHIP - tema::ESP_4 - tema::ESP_2;
                            let basura = egui::Rect::from_min_max(
                                egui::pos2(
                                    x1 - tema::ALTO_CHIP,
                                    fila.max.y - tema::ALTO_CHIP - tema::ESP_2,
                                ),
                                egui::pos2(x1, fila.max.y - tema::ESP_2),
                            );
                            if kit::icon_button_en(
                                ui,
                                basura,
                                Icono::Papelera,
                                textos.comando(Comando::Eliminar),
                            )
                            .clicked()
                            {
                                mensajes.push(MensajeEditor::BorrarCaption(caption.id.clone()));
                            }
                        });
                    });

                    if respuesta.clicked() {
                        let modif = ui.input(|entrada| entrada.modifiers);
                        if modif.ctrl || modif.shift {
                            mensajes.push(MensajeEditor::SeleccionarConModificador {
                                id: caption.id.clone(),
                                ctrl: modif.ctrl,
                                shift: modif.shift,
                            });
                        } else {
                            mensajes.push(MensajeEditor::Seleccionar(caption.id.clone()));
                        }
                    }
                    if respuesta.double_clicked() {
                        mensajes.push(MensajeEditor::Seleccionar(caption.id.clone()));
                        mensajes.push(MensajeEditor::Comando(Comando::EditarTexto));
                    }
                    let id = caption.id.clone();
                    if ui.rect_contains_pointer(fila) {
                        respuesta.context_menu(|ui| {
                            crate::ui::timeline::menu_contexto(ui, &id, mensajes, textos);
                        });
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
