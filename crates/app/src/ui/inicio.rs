//! Pantalla de arranque: logo, acciones, zona de arrastre y recientes.

use std::collections::HashMap;
use std::path::PathBuf;

use egui::{Color32, RichText, Sense, Stroke, Vec2};

use crate::inicio::{EstadoInicio, MensajeInicio, OrdenRecientes, cantidad_visible};
use crate::preferencias::Preferencias;
use crate::recientes::Reciente;
use crate::textos::Textos;

use super::kit::{self, Icono};
use super::tema::{self, Estilo};

/// Dibuja la pantalla. El booleano indica que se pidió abrir Ajustes.
/// (Ajustes vive ahora en la cabecera; se conserva por compatibilidad.)
pub fn mostrar(
    ui: &mut egui::Ui,
    estado: &mut EstadoInicio,
    textos: &Textos,
    _prefs: &mut Preferencias,
) -> (Option<MensajeInicio>, bool) {
    // El fondo cubre la ventana; el contenido puede crecerse (Ver todos) y
    // sale en un ScrollArea en vez de dibujarse fuera de pantalla.
    let rect = ui.available_rect_before_wrap();
    ui.painter().rect_filled(rect, 0.0, tema::FONDO);

    let ancho_columna = (rect.width() - tema::ESP_48 * 2.0)
        .clamp(320.0, tema::ANCHO_CONTENEDOR)
        .min(tema::ANCHO_CONTENEDOR);
    let centro_x = rect.center().x;
    let contenido = egui::Rect::from_min_max(
        egui::pos2(centro_x - ancho_columna / 2.0, rect.min.y),
        egui::pos2(centro_x + ancho_columna / 2.0, rect.max.y),
    );

    let mut intencion = None;
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            // Sin `max_rect` absoluto: el ScrollArea traslada el Ui hijo al
            // desplazarse y fijar el rect aquí dejaría el contenido quieto.
            // El margen a mano conserva el centrado sin fijar coordenadas.
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let margen = (ui.available_width() - contenido.width()).max(0.0) * 0.5;
                ui.add_space(margen);
                ui.vertical(|ui| {
                    ui.set_width(contenido.width());
                    ui.add_space(tema::ESP_40);
                    kit::logo(ui, 48.0);
                    ui.add_space(tema::ESP_24);

                    let titulo = ui.add(
                        egui::Label::new(
                            RichText::new(textos.inicio_titulo)
                                .font(egui::FontId::proportional(tema::TAMANO_TITULO))
                                .color(Color32::WHITE),
                        )
                        .selectable(false),
                    );
                    ui.painter().rect_filled(
                        egui::Rect::from_min_max(
                            egui::pos2(titulo.rect.min.x, titulo.rect.max.y + 6.0),
                            egui::pos2(
                                titulo.rect.min.x + tema::ANCHO_SUBRAYADO,
                                titulo.rect.max.y + 8.0,
                            ),
                        ),
                        0.0,
                        tema::ACENTO,
                    );
                    ui.add_space(tema::ESP_24);
                    ui.label(
                        RichText::new(textos.inicio_subtitulo)
                            .font(tema::fuente(Estilo::Cuerpo))
                            .color(tema::TEXTO_2),
                    );
                    ui.add_space(tema::ESP_24);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = tema::ESP_8;
                        if kit::boton_primario(ui, textos.abrir_video, Icono::Video).clicked() {
                            intencion = Some(MensajeInicio::AbrirVideo);
                        }
                        if kit::boton_secundario(ui, textos.abrir_proyecto, Icono::Carpeta)
                            .clicked()
                        {
                            intencion = Some(MensajeInicio::AbrirProyecto);
                        }
                        ui.label(
                            RichText::new("Ctrl+O")
                                .font(tema::fuente(Estilo::Mono))
                                .color(tema::TEXTO_3),
                        );
                    });

                    ui.add_space(tema::ESP_24);
                    if zona_de_arrastre(ui, textos) {
                        intencion = Some(MensajeInicio::AbrirVideo);
                    }

                    if let Some(aviso) = &estado.aviso {
                        ui.add_space(tema::ESP_16);
                        let aviso_color = Color32::from_rgba_premultiplied(
                            tema::PELIGRO.r(),
                            tema::PELIGRO.g(),
                            tema::PELIGRO.b(),
                            30,
                        );
                        let marco = egui::Frame {
                            inner_margin: egui::Margin::symmetric(
                                tema::ESP_12 as i8,
                                tema::ESP_8 as i8,
                            ),
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

                    ui.add_space(tema::ESP_40);
                    fila_recientes(ui, estado, textos, &mut intencion);
                    ui.add_space(tema::ESP_12);

                    let mut recientes: Vec<_> = estado.recientes.iter().collect();
                    if !estado.mostrar_vacios {
                        recientes.retain(|reciente| reciente.lineas > 0);
                    }
                    match estado.orden {
                        OrdenRecientes::Recientes => {}
                        OrdenRecientes::Nombre => {
                            recientes.sort_by_key(|r| r.ruta.to_string_lossy().into_owned())
                        }
                        OrdenRecientes::Lineas => {
                            recientes.sort_by_key(|r| std::cmp::Reverse(r.lineas))
                        }
                    }
                    if recientes.is_empty() {
                        kit::recuadro_punteado(ui, 104.0, textos.texto_vacio_recientes);
                    } else {
                        let gap = tema::ESP_20;
                        let columnas = if contenido.width() >= 780.0 {
                            3
                        } else if contenido.width() >= 520.0 {
                            2
                        } else {
                            1
                        };
                        let ancho_tarjeta =
                            (contenido.width() - gap * (columnas - 1) as f32) / columnas as f32;
                        let visibles = cantidad_visible(recientes.len(), estado.expandido);
                        let destacado = estado.destacado;
                        let tarjetas: Vec<_> =
                            recientes.into_iter().take(visibles).enumerate().collect();
                        for (fila, grupo) in tarjetas.chunks(columnas).enumerate() {
                            if fila > 0 {
                                ui.add_space(gap);
                            }
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = gap;
                                for (indice, reciente) in grupo {
                                    // Cada tarjeta necesita su propio Ui: `set_width` en el Ui
                                    // de la fila limitaría el ancho de toda la fila y solo
                                    // cabria la primera. Sin celda fija, el Frame decide el
                                    // alto según la miniatura y el texto.
                                    ui.scope_builder(egui::UiBuilder::new(), |ui| {
                                        ui.set_width(ancho_tarjeta);
                                        tarjeta_reciente(
                                            ui,
                                            reciente,
                                            textos,
                                            ancho_tarjeta,
                                            Some(*indice) == destacado,
                                            &mut intencion,
                                            &mut estado.miniaturas,
                                        );
                                    });
                                }
                            });
                        }
                        ui.add_space(tema::ESP_24);
                    }
                });
            });
        });

    (intencion, false)
}

/// Etiqueta de sección con letter-spacing de 0.04 em.
fn etiqueta_espaciada(ui: &mut egui::Ui, texto: &str, color: Color32) {
    let fuente = tema::fuente(Estilo::Pequeno);
    let mut trabajo =
        egui::text::LayoutJob::simple_singleline(texto.to_owned(), fuente.clone(), color);
    if let Some(seccion) = trabajo.sections.first_mut() {
        seccion.format.extra_letter_spacing = fuente.size * tema::LETRA_ESPACIADO;
    }
    ui.add(egui::Label::new(trabajo).selectable(false));
}

/// Fila «Recientes / filtro / See all», con todo alineado al centro vertical.
fn fila_recientes(
    ui: &mut egui::Ui,
    estado: &mut EstadoInicio,
    textos: &Textos,
    intencion: &mut Option<MensajeInicio>,
) {
    // Altura acotada: un `with_layout` sin límite se come todo el alto restante.
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), tema::ALTO_NAV),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            etiqueta_espaciada(ui, &textos.recientes.to_uppercase(), tema::TEXTO_2);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if estado.recientes.len() > 3 {
                    let respuesta = ui.add(
                        egui::Button::new(
                            RichText::new(textos.ver_todos)
                                .font(tema::fuente(Estilo::Base))
                                .color(tema::TEXTO_2),
                        )
                        .frame(false),
                    );
                    if respuesta.hovered() {
                        ui.painter().rect_filled(
                            egui::Rect::from_min_max(
                                egui::pos2(respuesta.rect.min.x, respuesta.rect.max.y - 1.0),
                                respuesta.rect.max,
                            ),
                            0.0,
                            tema::ACENTO,
                        );
                    }
                    if respuesta.clicked() {
                        *intencion = Some(MensajeInicio::VerTodos);
                    }
                }

                let actual = match estado.orden {
                    OrdenRecientes::Recientes => textos.mas_recientes,
                    OrdenRecientes::Nombre => textos.orden_nombre,
                    OrdenRecientes::Lineas => textos.orden_lineas,
                };
                let _ = egui::containers::menu::MenuButton::from_button(
                    egui::Button::new(
                        RichText::new(format!("{actual} {}", kit::glifo(Icono::ChevronAbajo)))
                            .font(tema::fuente(Estilo::Base))
                            .color(tema::TEXTO_2),
                    )
                    .frame(false),
                )
                .ui(ui, |ui| {
                    ui.set_min_width(tema::ANCHO_FILTRO);
                    for (orden, etiqueta) in [
                        (OrdenRecientes::Recientes, textos.mas_recientes),
                        (OrdenRecientes::Nombre, textos.orden_nombre),
                        (OrdenRecientes::Lineas, textos.orden_lineas),
                    ] {
                        if ui.button(etiqueta).clicked() {
                            estado.orden = orden;
                        }
                    }
                    ui.separator();
                    ui.checkbox(&mut estado.mostrar_vacios, textos.mostrar_vacios);
                });
            });
        },
    );
}

/// Zona de arrastre de 96 px, clicable para abrir el selector de vídeo.
fn zona_de_arrastre(ui: &mut egui::Ui, textos: &Textos) -> bool {
    let (rect, respuesta) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), tema::ALTO_ZONA),
        Sense::click(),
    );
    let sobre = ui
        .ctx()
        .input(|entrada| !entrada.raw.hovered_files.is_empty());
    let activo =
        ui.ctx()
            .animate_bool_with_time(egui::Id::new("zona-arrastre"), sobre, tema::ANIMACION);
    let borde = tema::mezclar(tema::BORDE_CONTROL, tema::ACENTO, activo);
    let fondo = tema::mezclar(tema::FONDO_ZONA, tema::ACENTO, activo * 0.08);
    let color_texto = tema::mezclar(tema::TEXTO_2, tema::ACENTO, activo);

    ui.painter()
        .rect_filled(rect, tema::radio_superficie(), fondo);
    let trazo = Stroke::new(tema::GROSOR_PUNTEADO, borde);
    let dash = tema::ESP_8;
    let mut x = rect.min.x;
    while x < rect.max.x {
        for y in [rect.min.y, rect.max.y] {
            ui.painter().line_segment(
                [egui::pos2(x, y), egui::pos2((x + dash).min(rect.max.x), y)],
                trazo,
            );
        }
        x += dash + tema::ESP_4;
    }
    let mut y = rect.min.y;
    while y < rect.max.y {
        for x in [rect.min.x, rect.max.x] {
            ui.painter().line_segment(
                [egui::pos2(x, y), egui::pos2(x, (y + dash).min(rect.max.y))],
                trazo,
            );
        }
        y += dash + tema::ESP_4;
    }

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        // Sin `centered_and_justified`: expande el hijo hasta el fondo de la
        // ventana y empuja la grilla fuera de pantalla.
        ui.vertical_centered(|ui| {
            ui.add_space(tema::ESP_12);
            ui.label(
                RichText::new(kit::glifo(Icono::Subir))
                    .font(egui::FontId::proportional(tema::TAMANO_ICONO_GRANDE))
                    .color(color_texto),
            );
            ui.label(
                RichText::new(if activo > 0.5 {
                    textos.drop_activo
                } else {
                    textos.drop_titulo
                })
                .font(tema::fuente(Estilo::Base))
                .color(color_texto),
            );
            ui.label(
                RichText::new(textos.drop_subtitulo)
                    .font(tema::fuente(Estilo::Pequeno))
                    .color(tema::TEXTO_2),
            );
        });
    });
    respuesta.clicked()
}

fn tarjeta_reciente(
    ui: &mut egui::Ui,
    reciente: &Reciente,
    textos: &Textos,
    ancho: f32,
    destacada: bool,
    intencion: &mut Option<MensajeInicio>,
    miniaturas: &mut HashMap<PathBuf, egui::TextureHandle>,
) {
    // El Frame mide la tarjeta: crece con la miniatura y el texto. Reservar
    // una celda de altura fija dejaba el borde y el clic por encima de la
    // burbuja real (210 px reservados frente a ~273 px pintados).
    let mut miniatura = egui::Rect::NOTHING;
    let (marco, _) = kit::tarjeta(ui, |ui| {
        ui.vertical(|ui| {
            let ancho_util = (ancho - tema::ESP_16 * 2.0).max(120.0);
            let (rect_miniatura, _) = ui.allocate_exact_size(
                Vec2::new(ancho_util, ancho_util * 9.0 / 16.0),
                Sense::hover(),
            );
            miniatura = rect_miniatura;
            pintar_miniatura(ui, reciente, rect_miniatura, miniaturas);
            pintar_barra_hablantes(ui, reciente, rect_miniatura);

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
            ui.add_space(tema::ESP_4);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} · {}",
                        textos.lineas(reciente.lineas),
                        textos.hablantes(reciente.hablantes)
                    ))
                    .font(tema::fuente(Estilo::Base))
                    .color(tema::TEXTO_2),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(textos.fecha(&reciente.fecha))
                            .font(tema::fuente(Estilo::Pequeno))
                            .color(tema::TEXTO_3),
                    );
                });
            });

            ui.add_space(tema::ESP_4);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = tema::ESP_4;
                for (color, _) in reciente.tira.iter().take(7) {
                    kit::punto(ui, tema::color(color), tema::TAMANO_PUNTO);
                }
                if reciente.tira.len() > 7 {
                    ui.label(
                        RichText::new(format!("+{}", reciente.tira.len() - 7))
                            .font(tema::fuente(Estilo::Pequeno))
                            .color(tema::TEXTO_3),
                    );
                }
            });
            ui.add_space(tema::ESP_12);
        });
    });

    // Un solo rect para borde, hover y clic: el del Frame pintado. Antes se
    // usaba la celda de altura fija y la zona interactiva no coincidía con la
    // burbuja.
    let rect = marco.rect;
    let borde = if ui.rect_contains_pointer(rect) {
        tema::BORDE_HOVER
    } else {
        tema::BORDE
    };
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(tema::RADIO_SUPERFICIE as u8),
        Stroke::new(1.0_f32, borde),
        egui::StrokeKind::Inside,
    );
    if destacada {
        ui.painter().rect_stroke(
            rect,
            egui::CornerRadius::same(tema::RADIO_SUPERFICIE as u8),
            Stroke::new(2.0_f32, tema::ACENTO),
            egui::StrokeKind::Outside,
        );
    }

    let respuesta = ui.interact(
        rect,
        ui.make_persistent_id(("tarjeta", &reciente.ruta)),
        Sense::click(),
    );
    if respuesta.clicked() {
        *intencion = Some(MensajeInicio::AbrirReciente(reciente.ruta.clone()));
    }
    respuesta.on_hover_ui(|ui| {
        ui.label(reciente.ruta.display().to_string());
    });

    // «···» va después de la interacción de la tarjeta para que el menú siga
    // siendo clicable. Antes se dibujaba al final del contenido, fuera de la
    // tarjeta.
    if miniatura != egui::Rect::NOTHING {
        let zona = egui::Rect::from_min_size(
            egui::pos2(
                miniatura.max.x - tema::ESP_24 - tema::ESP_8,
                miniatura.min.y + tema::ESP_8,
            ),
            Vec2::splat(tema::ESP_24),
        );
        ui.scope_builder(egui::UiBuilder::new().max_rect(zona), |ui| {
            ui.painter()
                .rect_filled(zona, tema::radio_control(), Color32::from_black_alpha(150));
            let _ = egui::containers::menu::MenuButton::from_button(
                egui::Button::new(
                    RichText::new("···")
                        .font(tema::fuente(Estilo::Base))
                        .color(tema::TEXTO),
                )
                .frame(false)
                .min_size(zona.size()),
            )
            .ui(ui, |ui| {
                if ui.button(textos.abrir).clicked() {
                    *intencion = Some(MensajeInicio::AbrirReciente(reciente.ruta.clone()));
                    ui.close();
                }
                if ui.button(textos.mostrar_en_carpeta).clicked() {
                    *intencion = Some(MensajeInicio::MostrarEnCarpeta(reciente.ruta.clone()));
                    ui.close();
                }
                if ui.button(textos.quitar_de_recientes).clicked() {
                    *intencion = Some(MensajeInicio::QuitarDeRecientes(reciente.ruta.clone()));
                    ui.close();
                }
            });
        });
    }
}

fn pintar_miniatura(
    ui: &mut egui::Ui,
    reciente: &Reciente,
    miniatura: egui::Rect,
    miniaturas: &mut HashMap<PathBuf, egui::TextureHandle>,
) {
    let ruta = crate::thumbnail::ruta_cache(&reciente.ruta);
    if !ruta.is_file() {
        let color = reciente
            .tira
            .first()
            .map(|(color, _)| tema::color(color))
            .unwrap_or(tema::CONTROL);
        ui.painter().rect_filled(miniatura, 0.0, tema::CONTROL);
        ui.painter().rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(miniatura.min.x, miniatura.max.y - tema::ALTO_BARRA_COLOR),
                miniatura.max,
            ),
            0.0,
            color,
        );
        return;
    }
    let textura = miniaturas.entry(reciente.ruta.clone()).or_insert_with(|| {
        let mut rgba = image::open(&ruta)
            .map(|imagen| imagen.to_rgba8())
            .unwrap_or_else(|_| {
                image::RgbaImage::from_pixel(160, 90, image::Rgba([38, 38, 38, 255]))
            });
        for pixel in rgba.as_mut().as_chunks_mut::<4>().0.iter_mut() {
            pixel[3] = 255;
        }
        let (ancho, alto) = rgba.dimensions();
        ui.ctx().load_texture(
            format!("miniatura-{:?}", reciente.ruta),
            egui::ColorImage::from_rgba_unmultiplied([ancho as usize, alto as usize], &rgba),
            egui::TextureOptions::default(),
        )
    });
    ui.painter().rect_filled(miniatura, 0.0, Color32::BLACK);
    let tamaño = textura.size();
    let aspecto = tamaño[0] as f32 / tamaño[1].max(1) as f32;
    let (ancho_img, alto_img) = if aspecto > miniatura.width() / miniatura.height() {
        (miniatura.width(), miniatura.width() / aspecto)
    } else {
        (miniatura.height() * aspecto, miniatura.height())
    };
    ui.painter().image(
        textura.id(),
        egui::Rect::from_center_size(miniatura.center(), Vec2::new(ancho_img, alto_img)),
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        Color32::WHITE,
    );
}

/// Barra de 4 px segmentada por el tiempo que habla cada hablante.
fn pintar_barra_hablantes(ui: &mut egui::Ui, reciente: &Reciente, miniatura: egui::Rect) {
    let alto = tema::ALTO_BARRA_COLOR;
    let rect = egui::Rect::from_min_max(
        egui::pos2(miniatura.min.x, miniatura.max.y - alto),
        miniatura.max,
    );
    let total: f32 = reciente.tira.iter().map(|(_, segundos)| *segundos).sum();
    if total <= 0.0 {
        ui.painter().rect_filled(rect, 0.0, tema::BORDE_CONTROL);
        return;
    }
    let mut x = rect.min.x;
    for (color, segundos) in &reciente.tira {
        let ancho = rect.width() * (*segundos / total);
        ui.painter().rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(x, rect.min.y),
                egui::pos2((x + ancho).min(rect.max.x), rect.max.y),
            ),
            0.0,
            tema::color(color),
        );
        x += ancho;
    }
}
