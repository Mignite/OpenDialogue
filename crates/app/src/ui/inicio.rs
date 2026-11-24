//! Pantalla de arranque: riel, acciones, zona de arrastre y recientes.

use egui::{Align2, Color32, RichText, Sense, Stroke, Vec2};

use crate::inicio::{EstadoInicio, MensajeInicio, OrdenRecientes, cantidad_visible};
use crate::preferencias::Preferencias;
use crate::recientes::Reciente;
use crate::textos::Textos;

use super::kit::{self, Icono};
use super::tema::{self, Estilo};

/// Dibuja la pantalla. El booleano indica que se pidió abrir Ajustes.
pub fn mostrar(
    ui: &mut egui::Ui,
    estado: &mut EstadoInicio,
    textos: &Textos,
    prefs: &mut Preferencias,
) -> (Option<MensajeInicio>, bool) {
    let _ = &prefs;
    let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, tema::FONDO);

    let ancho_riel = 220.0;
    let riel = egui::Rect::from_min_size(rect.min, Vec2::new(ancho_riel, rect.height()));
    // Margen lateral real: sin él el título y las tarjetas se pegan al riel.
    let contenido = egui::Rect::from_min_max(
        egui::pos2(rect.min.x + ancho_riel + tema::ESP_48, rect.min.y),
        egui::pos2(rect.max.x - tema::ESP_48, rect.max.y),
    );

    let abrir_ajustes = riel_de_navegacion(ui, riel, textos);

    let mut intencion = None;
    ui.scope_builder(egui::UiBuilder::new().max_rect(contenido), |ui| {
        ui.set_width(contenido.width());
        ui.add_space(tema::ESP_72);

        ui.label(
            RichText::new(textos.inicio_titulo)
                .font(tema::fuente_segura(Estilo::Titulo))
                .color(tema::TEXTO),
        );
        ui.add_space(tema::ESP_12);
        ui.label(
            RichText::new(textos.inicio_subtitulo)
                .font(tema::fuente_segura(Estilo::Cuerpo))
                .color(tema::TEXTO_2),
        );
        ui.add_space(tema::ESP_24);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = tema::ESP_8;
            if kit::boton_primario(ui, textos.abrir_video, Icono::Video).clicked() {
                intencion = Some(MensajeInicio::AbrirVideo);
            }
            if kit::boton_secundario(ui, textos.abrir_proyecto, Icono::Carpeta).clicked() {
                intencion = Some(MensajeInicio::AbrirProyecto);
            }
            ui.label(
                RichText::new("Ctrl+O")
                    .font(tema::fuente(Estilo::Mono))
                    .color(tema::TEXTO_3),
            );
        });

        ui.add_space(tema::ESP_16);
        zona_de_arrastre(ui, textos);

        if let Some(aviso) = &estado.aviso {
            ui.add_space(tema::ESP_16);
            let aviso_color = Color32::from_rgba_premultiplied(
                tema::PELIGRO.r() as u16 as u8,
                tema::PELIGRO.g() as u16 as u8,
                tema::PELIGRO.b() as u16 as u8,
                30,
            );
            let marco = egui::Frame {
                inner_margin: egui::Margin::symmetric(tema::ESP_12 as i8, tema::ESP_8 as i8),
                fill: aviso_color,
                ..egui::Frame::NONE
            };
            marco.show(ui, |ui| {
                ui.label(
                    RichText::new(&aviso.texto)
                        .font(tema::fuente(Estilo::Base))
                        .color(tema::TEXTO),
                );
            });
        }
        if let Some(mensaje) = &estado.mensaje {
            ui.add_space(tema::ESP_8);
            ui.label(
                RichText::new(mensaje)
                    .font(tema::fuente(Estilo::Base))
                    .color(tema::PELIGRO),
            );
        }

        ui.add_space(tema::ESP_48);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(textos.recientes)
                    .font(tema::fuente(Estilo::Pequeno))
                    .color(tema::TEXTO_2),
            );
            let orden_actual = match estado.orden {
                OrdenRecientes::Recientes => textos.mas_recientes,
                OrdenRecientes::Nombre => textos.orden_nombre,
                OrdenRecientes::Lineas => textos.orden_lineas,
            };
            egui::ComboBox::from_id_salt("orden-recientes")
                .selected_text(orden_actual)
                .show_ui(ui, |ui| {
                    for (orden, etiqueta) in [
                        (OrdenRecientes::Recientes, textos.mas_recientes),
                        (OrdenRecientes::Nombre, textos.orden_nombre),
                        (OrdenRecientes::Lineas, textos.orden_lineas),
                    ] {
                        ui.selectable_value(&mut estado.orden, orden, etiqueta);
                    }
                });
        });
        ui.add_space(tema::ESP_12);

        if estado.recientes.is_empty() {
            kit::recuadro_punteado(ui, 104.0, textos.texto_vacio_recientes);
        } else {
            let destacado = estado.destacado;
            let mut recientes: Vec<_> = estado.recientes.iter().collect();
            match estado.orden {
                OrdenRecientes::Recientes => {}
                OrdenRecientes::Nombre => {
                    recientes.sort_by_key(|r| r.ruta.to_string_lossy().into_owned())
                }
                OrdenRecientes::Lineas => recientes.sort_by_key(|r| std::cmp::Reverse(r.lineas)),
            }
            let visibles = cantidad_visible(recientes.len(), estado.expandido);
            let ancho_tarjeta = 260.0;
            // horizontal_wrapped decide las columnas con el ancho realmente
            // disponible; el Grid anterior se apilaba en la app real.
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(tema::ESP_24);
                for (indice, reciente) in recientes.into_iter().take(visibles).enumerate() {
                    ui.allocate_ui_with_layout(
                        Vec2::new(ancho_tarjeta, 210.0),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.set_width(ancho_tarjeta);
                            tarjeta_reciente(
                                ui,
                                reciente,
                                textos,
                                ancho_tarjeta,
                                Some(indice) == destacado,
                                &mut intencion,
                            );
                        },
                    );
                }
            });
            if estado.recientes.len() > 3 && ui.button(textos.ver_todos).clicked() {
                intencion = Some(MensajeInicio::VerTodos);
            }
        }
    });

    (intencion, abrir_ajustes)
}

fn riel_de_navegacion(ui: &mut egui::Ui, riel: egui::Rect, textos: &Textos) -> bool {
    let painter = ui.painter();
    painter.rect_filled(riel, 0.0, tema::RIEL);
    painter.line_segment(
        [
            egui::pos2(riel.max.x, riel.min.y),
            egui::pos2(riel.max.x, riel.max.y),
        ],
        Stroke::new(1.0_f32, tema::BORDE),
    );

    let mut abrir_ajustes = false;
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(riel.shrink(tema::ESP_12)),
        |ui| {
            ui.vertical(|ui| {
                ui.add_space(tema::ESP_32);
                for (etiqueta, icono, activo) in [
                    (textos.inicio, Icono::Inicio2, true),
                    (textos.recientes, Icono::Reloj, false),
                ] {
                    let respuesta = ui.add(
                        egui::Button::new(
                            RichText::new(format!("{}  {etiqueta}", kit::glifo(icono)))
                                .font(tema::fuente(Estilo::Base))
                                .color(if activo { tema::TEXTO } else { tema::TEXTO_2 }),
                        )
                        .frame(false)
                        .min_size(Vec2::new(ui.available_width(), tema::ALTO_NAV)),
                    );
                    if activo {
                        let barra = egui::Rect::from_min_max(
                            egui::pos2(respuesta.rect.min.x - 6.0, respuesta.rect.min.y + 6.0),
                            egui::pos2(respuesta.rect.min.x - 4.0, respuesta.rect.max.y - 6.0),
                        );
                        ui.painter().rect_filled(barra, 0.0, tema::ACENTO);
                    }
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    // En bottom_up el primer widget va abajo del todo: la
                    // versión queda al pie y Ajustes justo encima.
                    ui.label(
                        RichText::new("v0.9")
                            .font(tema::fuente(Estilo::Mono))
                            .color(tema::TEXTO_3),
                    );
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(textos.ajustes)
                                    .font(tema::fuente(Estilo::Base))
                                    .color(tema::TEXTO_2),
                            )
                            .frame(false)
                            .min_size(Vec2::new(ui.available_width(), tema::ALTO_NAV)),
                        )
                        .clicked()
                    {
                        abrir_ajustes = true;
                    }
                });
            });
        },
    );
    abrir_ajustes
}

fn zona_de_arrastre(ui: &mut egui::Ui, textos: &Textos) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 104.0), Sense::hover());
    let sobre = ui.ctx().input(|entrada| {
        entrada.raw.hovered_files.iter().any(|archivo| {
            archivo
                .path
                .as_ref()
                .and_then(|ruta| ruta.extension())
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    ["mp4", "mkv", "mov", "webm"]
                        .iter()
                        .any(|valida| extension.eq_ignore_ascii_case(valida))
                })
        })
    });
    let color = if sobre {
        tema::ACENTO
    } else {
        tema::BORDE_HOVER
    };
    let pintor = ui.painter();
    let trazo = Stroke::new(1.5_f32, color);
    let dash = tema::ESP_8;
    let mut x = rect.min.x;
    while x < rect.max.x {
        pintor.line_segment(
            [
                egui::pos2(x, rect.min.y),
                egui::pos2((x + dash).min(rect.max.x), rect.min.y),
            ],
            trazo,
        );
        pintor.line_segment(
            [
                egui::pos2(x, rect.max.y),
                egui::pos2((x + dash).min(rect.max.x), rect.max.y),
            ],
            trazo,
        );
        x += dash + tema::ESP_4;
    }
    let mut y = rect.min.y;
    while y < rect.max.y {
        pintor.line_segment(
            [
                egui::pos2(rect.min.x, y),
                egui::pos2(rect.min.x, (y + dash).min(rect.max.y)),
            ],
            trazo,
        );
        pintor.line_segment(
            [
                egui::pos2(rect.max.x, y),
                egui::pos2(rect.max.x, (y + dash).min(rect.max.y)),
            ],
            trazo,
        );
        y += dash + tema::ESP_4;
    }
    pintor.text(
        rect.center(),
        Align2::CENTER_CENTER,
        textos.arrastrar_video,
        tema::fuente(Estilo::Base),
        if sobre { tema::ACENTO } else { tema::TEXTO_2 },
    );
}

fn tarjeta_reciente(
    ui: &mut egui::Ui,
    reciente: &Reciente,
    textos: &Textos,
    ancho: f32,
    destacada: bool,
    intencion: &mut Option<MensajeInicio>,
) {
    let color = reciente
        .tira
        .first()
        .map(|(color, _)| tema::color(color))
        .unwrap_or(tema::CONTROL);
    let (respuesta, _) = kit::tarjeta(ui, |ui| {
        // El Grid celda usa layout horizontal: sin este vertical los labels
        // caen al lado de la miniatura y se solapan con ella.
        ui.vertical(|ui| {
            // El ancho de la tarjeta ya viene del Grid; forzarlo aquí rompía el
            // layout y los labels se solapaban con la miniatura.
            let ancho_util = (ancho - tema::ESP_16 * 2.0).max(120.0);
            let (miniatura, _) =
                ui.allocate_exact_size(Vec2::new(ancho_util, 96.0), Sense::hover());
            ui.painter().rect_filled(miniatura, 0.0, tema::CONTROL);
            ui.painter().rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(miniatura.min.x, miniatura.max.y - 4.0),
                    miniatura.max,
                ),
                0.0,
                color,
            );
            ui.set_width(ancho_util);
            ui.label(
                RichText::new(tema::acortar(
                    &reciente
                        .ruta
                        .file_name()
                        .map(|nombre| nombre.to_string_lossy().into_owned())
                        .unwrap_or_else(|| reciente.ruta.display().to_string()),
                    28,
                ))
                .font(tema::fuente_segura(Estilo::CuerpoFuerte))
                .color(tema::TEXTO),
            );
            ui.label(
                RichText::new(format!(
                    "{} · {} · {}",
                    textos.lineas(reciente.lineas),
                    textos.hablantes(reciente.hablantes),
                    textos.fecha(&reciente.fecha)
                ))
                .font(tema::fuente(Estilo::Pequeno))
                .color(tema::TEXTO_3),
            );
        });
    });
    if destacada {
        ui.painter().rect_stroke(
            respuesta.rect,
            egui::CornerRadius::same(tema::RADIO_SUPERFICIE as u8),
            Stroke::new(2.0_f32, tema::ACENTO),
            egui::StrokeKind::Outside,
        );
    }
    if respuesta.clicked() {
        *intencion = Some(MensajeInicio::AbrirReciente(reciente.ruta.clone()));
    }
    respuesta.on_hover_ui(|ui| {
        ui.label(reciente.ruta.display().to_string());
    });
}
