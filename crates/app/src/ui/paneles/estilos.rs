//! Panel de estilos: tarjetas con vista previa viva y editor inline.

use egui::{RichText, Stroke, Vec2};

use crate::editor_app::{EditorApp, MensajeEditor};
use crate::presets_edicion::{CampoPreset, valor_campo};
use crate::textos::Textos;
use crate::ui::kit::{self, Icono};
use crate::ui::tema::{self, Estilo};

pub fn mostrar(ui: &mut egui::Ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();

    state
        .texturas_preset
        .retain(|id, _| state.presets.iter().any(|preset| &preset.id == id));

    if ui
        .add_sized(
            Vec2::new(ui.available_width(), tema::ALTO_NAV),
            egui::Button::new(
                RichText::new(format!(
                    "{}  {}",
                    kit::glifo(Icono::Mas),
                    textos.nuevo_preset
                ))
                .font(tema::fuente(Estilo::Base)),
            ),
        )
        .clicked()
    {
        mensajes.push(MensajeEditor::NuevoPreset);
    }

    let nombres: Vec<String> = state.presets.iter().map(|p| p.fontname.clone()).collect();
    crate::ui::tipografia::registrar_fuentes(ui.ctx(), &nombres);

    let mut abrir: Option<egui::Rect> = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        let presets = state.presets.clone();
        for preset in &presets {
            let abierto = state.preset_seleccionado.as_deref() == Some(&preset.id);
            let altura = tema::ALTO_PREVIEW_STYLES + tema::ALTO_CABECERA_PRESET;
            let (rect, respuesta) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), altura),
                egui::Sense::click(),
            );
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

            let ancho_cabecera = rect.width();
            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                ui.set_width(rect.width());
                ui.vertical(|ui| {
                    vista_previa(ui, state, preset, textos);
                    ui.horizontal(|ui| {
                        ui.set_height(tema::ALTO_CABECERA_PRESET);
                        ui.add_space(tema::ESP_8);
                        if state.preset_renombrando.as_deref() == Some(&preset.id) {
                            let clave = (preset.id.clone(), CampoPreset::Nombre);
                            let valor =
                                valor_campo(preset, CampoPreset::Nombre, &state.borradores_preset);
                            let borrador = state.borradores_preset.entry(clave).or_insert(valor);
                            let respuesta = ui.add_sized(
                                Vec2::new(ancho_cabecera * 0.4, tema::ALTO_CONTROL),
                                egui::TextEdit::singleline(borrador)
                                    .font(tema::fuente_segura(Estilo::CuerpoFuerte)),
                            );
                            if respuesta.lost_focus()
                                || ui.input(|entrada| entrada.key_pressed(egui::Key::Enter))
                            {
                                mensajes.push(MensajeEditor::EditarPreset(
                                    preset.id.clone(),
                                    CampoPreset::Nombre,
                                    borrador.clone(),
                                ));
                                state.preset_renombrando = None;
                            }
                        } else {
                            let nombre = ui.add(
                                egui::Label::new(
                                    RichText::new(&preset.nombre)
                                        .font(tema::fuente_segura(Estilo::CuerpoFuerte))
                                        .color(tema::TEXTO),
                                )
                                .sense(egui::Sense::click()),
                            );
                            if nombre.double_clicked() {
                                state.preset_renombrando = Some(preset.id.clone());
                            }
                        }
                        if !crate::ui::tipografia::fuente_disponible(&preset.fontname) {
                            tema::chip(ui, textos.fuente_no_encontrada, tema::AMBAR);
                        }
                        for (indice, hablante) in
                            state.editor.proyecto().hablantes.iter().enumerate()
                        {
                            if hablante.preset_id.as_deref() == Some(&preset.id) {
                                let (punto, respuesta) = ui.allocate_exact_size(
                                    Vec2::splat(tema::TAMANO_PUNTO_ESTILO),
                                    egui::Sense::hover(),
                                );
                                ui.painter().circle_filled(
                                    punto.center(),
                                    tema::TAMANO_PUNTO_ESTILO * 0.5,
                                    tema::color(rod_core::color_identidad(hablante, indice)),
                                );
                                respuesta.on_hover_text(&hablante.nombre);
                            }
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let giro = ui.ctx().animate_bool_with_time(
                                egui::Id::new(("chevron", &preset.id)),
                                abierto,
                                tema::ANIMACION,
                            );
                            let galley = ui.ctx().fonts_mut(|f| {
                                f.layout_job(egui::text::LayoutJob::simple_singleline(
                                    kit::glifo(Icono::ChevronAbajo).to_owned(),
                                    egui::FontId::proportional(tema::TAMANO_ICONO),
                                    tema::TEXTO_2,
                                ))
                            });
                            let mut forma = egui::epaint::TextShape::new(
                                ui.cursor().min,
                                galley,
                                tema::TEXTO_2,
                            );
                            forma.angle = giro * std::f32::consts::FRAC_PI_2;
                            ui.painter().add(forma);
                        });
                    });
                });
            });

            if respuesta.clicked() {
                state.preset_seleccionado = Some(preset.id.clone());
                abrir = Some(rect);
                mensajes.push(MensajeEditor::SeleccionarPreset(preset.id.clone()));
            }
            if abierto {
                editor_preset(ui, state, preset, textos, &mut mensajes);
            }
        }
    });
    if let Some(rect) = abrir {
        ui.scroll_to_rect(rect, Some(egui::Align::TOP));
        ui.ctx().request_repaint();
    }
    mensajes
}

/// Muestra el mismo raster libass que usa el overlay, en miniatura centrada.
fn vista_previa(
    ui: &mut egui::Ui,
    state: &mut EditorApp,
    preset: &rod_core::PresetAss,
    _textos: &Textos,
) {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), tema::ALTO_PREVIEW_STYLES),
        egui::Sense::hover(),
    );
    let radio = egui::CornerRadius {
        nw: tema::RADIO_SUPERFICIE as u8,
        ne: tema::RADIO_SUPERFICIE as u8,
        sw: 0,
        se: 0,
    };
    ui.painter()
        .rect_filled(rect, radio, tema::COLOR_PREVIEW_STYLES);

    let clave = format!(
        "{:?}",
        (
            &preset.id,
            &preset.fontname,
            preset.fontsize,
            &preset.color,
            &preset.outline_color,
            preset.outline,
            preset.shadow,
            preset.bold,
            preset.italic,
            (
                preset.alignment,
                preset.margin_l,
                preset.margin_r,
                preset.margin_v,
            ),
        )
    );
    let textura = state
        .texturas_preset
        .entry(preset.id.clone())
        .or_insert_with(|| {
            let escala = tema::ALTO_PREVIEW_STYLES / crate::preview::CANVAS.1 as f32;
            let imagen =
                crate::ui::estilos_render::render_subtitle(preset, "Aa Hola mundo", escala);
            let color = egui::ColorImage::from_rgba_unmultiplied(
                [imagen.width() as usize, imagen.height() as usize],
                imagen.as_raw(),
            );
            (
                clave.clone(),
                ui.ctx()
                    .load_texture("preset-preview", color, egui::TextureOptions::LINEAR),
            )
        });
    if textura.0 != clave {
        let escala = tema::ALTO_PREVIEW_STYLES / crate::preview::CANVAS.1 as f32;
        let imagen = crate::ui::estilos_render::render_subtitle(preset, "Aa Hola mundo", escala);
        let color = egui::ColorImage::from_rgba_unmultiplied(
            [imagen.width() as usize, imagen.height() as usize],
            imagen.as_raw(),
        );
        textura.0 = clave;
        textura.1 = ui
            .ctx()
            .load_texture("preset-preview", color, egui::TextureOptions::LINEAR);
    }
    let id = textura.1.id();
    let tamano = textura.1.size_vec2();
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.centered_and_justified(|ui| {
            ui.image((id, tamano));
        });
    });
}

/// La etiqueta es genérica: un preset con `alignment = 8` pone el margen
/// arriba, así que «Bottom margin» mentiría.
pub fn etiqueta_margen(textos: &Textos) -> &'static str {
    textos.margen_v
}

pub fn es_hex_valido(valor: &str) -> bool {
    valor.len() == 7
        && valor.starts_with('#')
        && valor[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn separador(ui: &mut egui::Ui) {
    ui.add_space(tema::ESP_12);
    let y = ui.cursor().top();
    ui.painter().line_segment(
        [
            egui::pos2(ui.max_rect().min.x, y),
            egui::pos2(ui.max_rect().max.x, y),
        ],
        egui::Stroke::new(1.0_f32, tema::BORDE),
    );
    ui.add_space(tema::ESP_12);
}

fn etiqueta(ui: &mut egui::Ui, texto: &str) {
    ui.label(
        RichText::new(texto.to_uppercase())
            .font(tema::fuente_segura(Estilo::Etiqueta))
            .color(tema::TEXTO_2),
    );
}

/// Publica el valor que el widget acaba de producir. Recibir el valor evita
/// leer un borrador viejo: los `DragValue` y el `ComboBox` guardan su estado en
/// una variable local y solo suelta el resultado al terminar el arrastre.
fn aplicar_campo(
    preset: &rod_core::PresetAss,
    campo: CampoPreset,
    valor: String,
    respuesta: egui::Response,
    mensajes: &mut Vec<MensajeEditor>,
) {
    if respuesta.drag_stopped() || respuesta.lost_focus() {
        mensajes.push(MensajeEditor::EditarPreset(preset.id.clone(), campo, valor));
    }
}

fn editor_preset(
    ui: &mut egui::Ui,
    state: &mut EditorApp,
    preset: &rod_core::PresetAss,
    textos: &Textos,
    mensajes: &mut Vec<MensajeEditor>,
) {
    let ancho = ui.available_width();
    ui.add_space(tema::ESP_12);
    let seleccionados = state.seleccion().len();
    let boton = egui::Button::new(
        RichText::new(textos.aplicar_a_lineas(seleccionados)).font(tema::fuente(Estilo::Base)),
    )
    .corner_radius(tema::radio_superficie())
    .min_size(Vec2::new(ancho, tema::ALTO_NAV));
    if ui.add_enabled(seleccionados > 0, boton).clicked() && seleccionados > 0 {
        mensajes.push(MensajeEditor::AplicarPresetSeleccion(preset.id.clone()));
    }
    ui.horizontal(|ui| {
        if ui
            .add_sized(
                Vec2::new(ancho * 0.5, tema::ALTO_CONTROL),
                egui::Button::new(textos.duplicar).frame(false),
            )
            .clicked()
        {
            mensajes.push(MensajeEditor::NuevoPreset);
        }
        // `update` rechaza borrar el último preset: no ofrecer la acción evita
        // un toast que diga «eliminado» cuando no pasó nada.
        if ui
            .add_enabled_ui(state.presets.len() > 1, |ui| {
                ui.add_sized(
                    Vec2::new(ancho * 0.5, tema::ALTO_CONTROL),
                    egui::Button::new(textos.comando(crate::comandos::Comando::Eliminar))
                        .frame(false),
                )
            })
            .inner
            .clicked()
        {
            mensajes.push(MensajeEditor::BorrarPreset(preset.id.clone()));
        }
    });

    separador(ui);
    etiqueta(ui, "FONT");
    let mut fuente = valor_campo(preset, CampoPreset::Fontname, &state.borradores_preset);
    let valor_inicial = fuente.clone();
    // El botón del ComboBox no pide foco en egui 0.33: `lost_focus` no dispara
    // al elegir. La selección se publica desde el propio closure, que es el
    // único sitio donde se sabe qué opción se pulsó.
    let mut elegida: Option<String> = None;
    egui::ComboBox::from_id_salt(("fuente-preset", &preset.id))
        .width(ancho)
        .selected_text(&fuente)
        .show_ui(ui, |ui| {
            ui.add_sized(
                Vec2::new(ancho, tema::ALTO_CONTROL),
                egui::TextEdit::singleline(&mut state.busqueda_fuente_preset)
                    .hint_text(textos.buscar_fuente),
            );
            let busqueda = state.busqueda_fuente_preset.to_lowercase();
            for nombre in crate::ui::tipografia::fuentes_del_sistema() {
                if nombre.to_lowercase().contains(&busqueda) {
                    ui.selectable_value(&mut fuente, nombre.clone(), &nombre);
                    if fuente != valor_inicial {
                        elegida = Some(fuente.clone());
                    }
                }
            }
        });
    if let Some(nombre) = elegida {
        mensajes.push(MensajeEditor::EditarPreset(
            preset.id.clone(),
            CampoPreset::Fontname,
            nombre,
        ));
    }

    ui.horizontal(|ui| {
        let mut tamano = valor_campo(preset, CampoPreset::Fontsize, &state.borradores_preset)
            .parse::<f32>()
            .unwrap_or(preset.fontsize as f32);
        let respuesta = ui.add_sized(
            Vec2::new(tema::ANCHO_DRAG_TAMANO, tema::ALTO_CONTROL),
            egui::DragValue::new(&mut tamano)
                .range(8.0..=200.0)
                .suffix(" px"),
        );
        aplicar_campo(
            preset,
            CampoPreset::Fontsize,
            tamano.to_string(),
            respuesta,
            mensajes,
        );
        for (campo, texto, negrita) in [
            (CampoPreset::Bold, "B", true),
            (CampoPreset::Italic, "I", false),
        ] {
            let activo = valor_campo(preset, campo, &state.borradores_preset) == "true";
            let disponible = crate::ui::tipografia::variante_disponible(&preset.fontname, negrita);
            let boton = egui::Button::new(
                RichText::new(texto).font(tema::fuente_segura(Estilo::CuerpoFuerte)),
            )
            .fill(if activo {
                tema::ACENTO
            } else {
                egui::Color32::TRANSPARENT
            })
            .min_size(Vec2::splat(tema::ANCHO_BOTON_28));
            let respuesta = ui.add_sized(Vec2::splat(tema::ANCHO_BOTON_28), boton);
            if respuesta.clicked() && disponible {
                mensajes.push(MensajeEditor::EditarPreset(
                    preset.id.clone(),
                    campo,
                    (!activo).to_string(),
                ));
            }
            if !disponible {
                respuesta.on_hover_text("Variante no disponible");
            }
        }
    });

    separador(ui);
    etiqueta(ui, "FILL");
    campo_color(
        ui,
        state,
        preset,
        CampoPreset::Color,
        textos,
        ancho,
        mensajes,
    );

    separador(ui);
    etiqueta(ui, "OUTLINE");
    ui.horizontal(|ui| {
        let mut grosor = preset.outline;
        let respuesta = ui.add_sized(
            Vec2::new(tema::ANCHO_DRAG_CONTORNO, tema::ALTO_CONTROL),
            egui::DragValue::new(&mut grosor).suffix(" px"),
        );
        aplicar_campo(
            preset,
            CampoPreset::Outline,
            grosor.to_string(),
            respuesta,
            mensajes,
        );
        campo_color(
            ui,
            state,
            preset,
            CampoPreset::OutlineColor,
            textos,
            ancho - tema::ANCHO_DRAG_CONTORNO,
            mensajes,
        );
    });

    separador(ui);
    etiqueta(ui, "SHADOW");
    let mut sombra = preset.shadow;
    let respuesta = ui.add_sized(
        Vec2::new(tema::ANCHO_DRAG_CONTORNO, tema::ALTO_CONTROL),
        egui::DragValue::new(&mut sombra).suffix(" px"),
    );
    aplicar_campo(
        preset,
        CampoPreset::Shadow,
        sombra.to_string(),
        respuesta,
        mensajes,
    );

    separador(ui);
    etiqueta(ui, etiqueta_margen(textos));
    let mut margen = preset.margin_v;
    let respuesta = ui.add_sized(
        Vec2::new(tema::ANCHO_DRAG_CONTORNO, tema::ALTO_CONTROL),
        egui::DragValue::new(&mut margen).suffix(" px"),
    );
    aplicar_campo(
        preset,
        CampoPreset::MarginV,
        margen.to_string(),
        respuesta,
        mensajes,
    );
    ui.add_space(tema::ESP_12);
}

fn campo_color(
    ui: &mut egui::Ui,
    state: &mut EditorApp,
    preset: &rod_core::PresetAss,
    campo: CampoPreset,
    _textos: &Textos,
    ancho: f32,
    mensajes: &mut Vec<MensajeEditor>,
) {
    let valor = valor_campo(preset, campo, &state.borradores_preset);
    let clave = (preset.id.clone(), campo);
    let color = tema::color_de_hex(&valor);
    let mut borrador = valor;
    let respuesta = ui
        .horizontal(|ui| {
            let boton = ui.add_sized(
                Vec2::splat(tema::ANCHO_BOTON_28),
                egui::Button::new("").fill(color),
            );
            egui::Popup::from_toggle_button_response(&boton).show(|ui| {
                let mut editable = color;
                ui.color_edit_button_srgba(&mut editable);
                if editable != color {
                    borrador = format!(
                        "#{:02X}{:02X}{:02X}",
                        editable.r(),
                        editable.g(),
                        editable.b()
                    );
                    mensajes.push(MensajeEditor::EditarPreset(
                        preset.id.clone(),
                        campo,
                        borrador.clone(),
                    ));
                }
            });
            let respuesta = ui.add_sized(
                Vec2::new(
                    ancho - tema::ANCHO_BOTON_28 - tema::ESP_8,
                    tema::ALTO_CONTROL,
                ),
                egui::TextEdit::singleline(&mut borrador).font(tema::fuente(Estilo::Base)),
            );
            if !es_hex_valido(&borrador) {
                ui.painter().rect_stroke(
                    respuesta.rect,
                    tema::radio_control(),
                    egui::Stroke::new(1.0_f32, tema::PELIGRO),
                    egui::StrokeKind::Inside,
                );
            }
            respuesta
        })
        .inner;
    let valor = borrador.clone();
    state.borradores_preset.insert(clave, borrador);
    // Un hex a medias se conserva como borrador pero no toca el preset.
    if es_hex_valido(&valor) {
        aplicar_campo(preset, campo, valor, respuesta, mensajes);
    }
}
