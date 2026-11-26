//! Panel de estilos: tarjetas con vista previa viva y editor inline.

use egui::{RichText, Stroke, Vec2};

use crate::editor_app::{EditorApp, MensajeEditor};
use crate::presets_edicion::{CampoPreset, valor_campo};
use crate::textos::Textos;
use crate::ui::kit::{self, Icono};
use crate::ui::tema::{self, Estilo};

pub fn mostrar(ui: &mut egui::Ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();

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

    let mut abrir: Option<String> = None;
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

            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                ui.set_width(rect.width());
                ui.vertical(|ui| {
                    vista_previa(ui, state, preset, textos);
                    ui.horizontal(|ui| {
                        ui.set_height(tema::ALTO_CABECERA_PRESET);
                        ui.add_space(tema::ESP_8);
                        ui.label(
                            RichText::new(&preset.nombre)
                                .font(tema::fuente_segura(Estilo::CuerpoFuerte))
                                .color(tema::TEXTO),
                        );
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
                abrir = Some(preset.id.clone());
                mensajes.push(MensajeEditor::SeleccionarPreset(preset.id.clone()));
            }
            if abierto {
                editor_preset(ui, state, preset, textos, &mut mensajes);
            }
        }
    });
    if let Some(id) = abrir {
        ui.ctx().request_repaint();
        let _ = id;
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
        )
    );
    let textura = state.texturas_preset.entry(clave).or_insert_with(|| {
        let escala = tema::ALTO_PREVIEW_STYLES / crate::preview::CANVAS.1 as f32;
        let imagen = crate::ui::estilos_render::render_subtitle(preset, "Aa Hola mundo", escala);
        let color = egui::ColorImage::from_rgba_unmultiplied(
            [imagen.width() as usize, imagen.height() as usize],
            imagen.as_raw(),
        );
        ui.ctx()
            .load_texture("preset-preview", color, egui::TextureOptions::LINEAR)
    });
    let tamano = textura.size_vec2();
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.centered_and_justified(|ui| {
            ui.image((textura.id(), tamano));
        });
    });
}

fn editor_preset(
    ui: &mut egui::Ui,
    state: &mut EditorApp,
    preset: &rod_core::PresetAss,
    textos: &Textos,
    mensajes: &mut Vec<MensajeEditor>,
) {
    ui.add_space(tema::ESP_8);
    if ui
        .add(
            egui::Button::new(textos.aplicar_a_seleccion)
                .frame(true)
                .corner_radius(tema::radio_superficie()),
        )
        .clicked()
    {
        mensajes.push(MensajeEditor::AplicarPresetSeleccion(preset.id.clone()));
    }

    let campos = [
        (CampoPreset::Nombre, textos.nombre),
        (CampoPreset::Fontname, textos.fuente),
        (CampoPreset::Fontsize, textos.tamano),
        (CampoPreset::Color, textos.color),
        (CampoPreset::Outline, textos.contorno),
        (CampoPreset::OutlineColor, textos.color),
        (CampoPreset::Shadow, textos.sombra),
        (CampoPreset::MarginL, textos.posicion),
        (CampoPreset::Bold, textos.negrita),
        (CampoPreset::Italic, textos.cursiva),
    ];
    for (campo, etiqueta) in campos {
        tema::etiqueta(ui, etiqueta);
        let valor = valor_campo(preset, campo, &state.borradores_preset);
        let clave = (preset.id.clone(), campo);
        let borrador = state.borradores_preset.entry(clave).or_insert(valor);
        if ui
            .add(egui::TextEdit::singleline(borrador).font(tema::fuente(Estilo::Base)))
            .lost_focus()
        {
            mensajes.push(MensajeEditor::EditarPreset(
                preset.id.clone(),
                campo,
                borrador.clone(),
            ));
        }
    }
    ui.add_space(tema::ESP_12);
}
