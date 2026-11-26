//! Pestaña Subtítulos: búsqueda y lista a todo el alto.
//! El texto se edita en la barra bajo el video (`ui/texto.rs`).

use egui::{RichText, Sense, Stroke, Vec2};

use crate::comandos::{self, Comando};
use crate::editor_app::{EditorApp, MensajeEditor};
use crate::textos::Textos;
use crate::ui::kit::{self, Icono};
use crate::ui::tema::{self, Estilo};
use crate::ui::texto;

/// Resultado del filtro, guardado entre frames en la memoria de egui (C12).
#[derive(Clone)]
struct CacheFiltro {
    firma: u64,
    filtro: String,
    indices: Vec<usize>,
}

/// Firma barata de lo que puede afectar al filtro. Recorrer los bytes sale
/// más barato que clonar 543 `Caption` (con su `String`) y bajar cada texto a
/// minúsculas en cada frame.
fn firma(proyecto: &rod_core::Proyecto) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64; // FNV-1a
    let mut alimentar = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for hablante in &proyecto.hablantes {
        alimentar(hablante.id.as_bytes());
        alimentar(hablante.nombre.as_bytes());
    }
    for caption in &proyecto.captions {
        alimentar(caption.id.as_bytes());
        alimentar(caption.texto.as_bytes());
        alimentar(&caption.inicio.to_bits().to_le_bytes());
        alimentar(&caption.fin.to_bits().to_le_bytes());
        if let Some(id) = &caption.hablante_id {
            alimentar(id.as_bytes());
        }
    }
    hash
}

/// Índices de las captions que pasan el filtro. Sin filtro son todas; con
/// filtro se calculan una vez y se reutilizan hasta que cambia el proyecto.
fn indices_filtrados(ui: &egui::Ui, proyecto: &rod_core::Proyecto, filtro: &str) -> Vec<usize> {
    let id = egui::Id::new("filtro-lista-subtitulos");
    let firma_actual = firma(proyecto);
    if let Some(cache) = ui.data(|datos| datos.get_temp::<CacheFiltro>(id))
        && cache.firma == firma_actual
        && cache.filtro == filtro
    {
        return cache.indices;
    }

    let indices: Vec<usize> = if filtro.is_empty() {
        (0..proyecto.captions.len()).collect()
    } else {
        proyecto
            .captions
            .iter()
            .enumerate()
            .filter(|(_, caption)| {
                caption.texto.to_lowercase().contains(filtro)
                    || caption
                        .hablante_id
                        .as_deref()
                        .and_then(|id| proyecto.hablantes.iter().find(|hablante| hablante.id == id))
                        .is_some_and(|hablante| hablante.nombre.to_lowercase().contains(filtro))
            })
            .map(|(indice, _)| indice)
            .collect()
    };
    ui.data_mut(|datos| {
        datos.insert_temp(
            id,
            CacheFiltro {
                firma: firma_actual,
                filtro: filtro.to_owned(),
                indices: indices.clone(),
            },
        );
    });
    indices
}

fn tooltip_con_atajo(texto: &str, atajo: String) -> String {
    if atajo.is_empty() {
        texto.to_owned()
    } else {
        format!("{texto} ({atajo})")
    }
}

pub fn mostrar(ui: &mut egui::Ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();
    let ancho = ui.available_width();

    // ── Barra superior fija ────────────────────────────────────────────────
    ui.allocate_ui_with_layout(
        Vec2::new(ancho, tema::ALTO_TABS),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = tema::ESP_8;
            ui.add(
                egui::TextEdit::singleline(&mut state.busqueda_subtitulos)
                    .hint_text(textos.buscar_subtitulos)
                    .font(tema::fuente(Estilo::Base))
                    .desired_width(ui.available_width() - tema::ESP_72),
            );
            if kit::icon_button(ui, Icono::Mas, false, textos.anadir_linea, None).clicked() {
                mensajes.push(MensajeEditor::Comando(Comando::AgregarFragmento));
            }
            let _ = egui::containers::menu::MenuButton::from_button(
                egui::Button::new(
                    RichText::new(kit::glifo(Icono::Puntos))
                        .font(egui::FontId::proportional(tema::TAMANO_ICONO))
                        .color(tema::TEXTO_2),
                )
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
        Stroke::new(tema::GROSOR_BORDE, tema::BORDE),
    );

    // ── Lista a todo el alto restante ──────────────────────────────────────
    let fondo = ui.max_rect().max.y;
    let y_lista = ui.cursor().top();
    let lista = egui::Rect::from_min_max(
        egui::pos2(ui.max_rect().min.x, y_lista),
        egui::pos2(
            ui.max_rect().max.x,
            fondo.max(y_lista + tema::ALTO_FILA_SUB * 2.0),
        ),
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
    // Préstamo por referencia: ni las captions ni los hablantes se clonan.
    let proyecto = state.editor.proyecto();
    let filtro = state.busqueda_subtitulos.trim().to_lowercase();
    let indices = indices_filtrados(ui, proyecto, &filtro);
    let seleccion = &state.editor.seleccion;

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        if indices.is_empty() {
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
            .show_rows(ui, tema::ALTO_FILA_SUB, indices.len(), |ui, rango| {
                for fila_indice in rango {
                    let caption = &proyecto.captions[indices[fila_indice]];
                    let seleccionado = seleccion.contains(&caption.id);
                    let color = caption
                        .hablante_id
                        .as_deref()
                        .and_then(|id| proyecto.hablantes.iter().position(|h| h.id == id))
                        .map(|i| tema::color(rod_core::color_identidad(&proyecto.hablantes[i], i)))
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
                            Stroke::new(tema::GROSOR_FOCO, tema::ACENTO),
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

                    // 12 px a ambos lados y hueco para la scrollbar: el lápiz
                    // deja de quedar recortado contra el borde del inspector.
                    let margen = tema::ESP_12;
                    let borde_derecho = fila.max.x - margen - tema::ANCHO_SCROLLBAR;
                    let ancho_iconos = tema::ALTO_CHIP * 2.0 + tema::ESP_4;
                    let interior = egui::Rect::from_min_max(
                        egui::pos2(
                            fila.min.x + tema::GROSOR_BARRA_HABLANTE + margen,
                            fila.min.y + tema::ESP_2,
                        ),
                        egui::pos2(
                            borde_derecho - ancho_iconos - tema::ESP_4,
                            fila.max.y - tema::ESP_2,
                        ),
                    );
                    ui.scope_builder(egui::UiBuilder::new().max_rect(interior), |ui| {
                        ui.set_width(interior.width());
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = tema::ESP_2;
                            let texto_fila =
                                tema::acortar(&caption.texto, tema::CARACTERES_MAXIMOS - 8);
                            let respuesta = ui.add(
                                egui::Label::new(
                                    RichText::new(&texto_fila)
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
                                let atajo_entrada = comandos::atajo(Comando::Entrada);
                                let tiempos = ui
                                    .add(
                                        egui::Label::new(
                                            RichText::new(format!(
                                                "{} → {}",
                                                texto::tiempo(caption.inicio),
                                                texto::tiempo(caption.fin)
                                            ))
                                            .font(tema::fuente(Estilo::Mono))
                                            .color(tema::TEXTO_3),
                                        )
                                        .sense(Sense::click()),
                                    )
                                    .on_hover_ui(move |ui| {
                                        ui.label(
                                            RichText::new(textos.comando(Comando::Entrada))
                                                .font(tema::fuente(Estilo::Base)),
                                        );
                                        ui.label(
                                            RichText::new(&atajo_entrada)
                                                .font(tema::fuente(Estilo::Mono)),
                                        );
                                    });
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
                                    RichText::new(texto::duracion(caption.inicio, caption.fin))
                                        .font(tema::fuente(Estilo::Pequeno))
                                        .color(tema::TEXTO_3),
                                );
                            });

                            let y = fila.max.y - tema::ALTO_CHIP - tema::ESP_2;
                            let lapiz = egui::Rect::from_min_max(
                                egui::pos2(borde_derecho - tema::ALTO_CHIP, y),
                                egui::pos2(borde_derecho, y + tema::ALTO_CHIP),
                            );
                            let editar = tooltip_con_atajo(
                                textos.editar,
                                comandos::atajo(Comando::EditarTexto),
                            );
                            if kit::icon_button_en(ui, lapiz, Icono::Editar, &editar).clicked() {
                                mensajes.push(MensajeEditor::Seleccionar(caption.id.clone()));
                                mensajes.push(MensajeEditor::Comando(Comando::EditarTexto));
                            }
                            // Basura junto al lápiz: borra esta línea sin
                            // cambiar la selección ni pedir confirmación.
                            let x1 = lapiz.min.x - tema::ESP_4;
                            let basura = egui::Rect::from_min_max(
                                egui::pos2(x1 - tema::ALTO_CHIP, y),
                                egui::pos2(x1, y + tema::ALTO_CHIP),
                            );
                            let borrar = tooltip_con_atajo(
                                textos.comando(Comando::Eliminar),
                                comandos::atajo(Comando::Eliminar),
                            );
                            if kit::icon_button_en(ui, basura, Icono::Papelera, &borrar).clicked() {
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
