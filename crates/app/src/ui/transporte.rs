//! Transporte del editor: iconos, timecode grande y velocidad.

use egui::{RichText, Vec2};

use crate::comandos::{Comando, MapaAtajos};
use crate::editor_app::{EditorApp, MensajeEditor, VELOCIDADES, sucio};
use crate::textos::Textos;
use crate::ui::kit::{self, Icono};
use crate::ui::tema::{self, Estilo};

/// Ancho del selector de velocidad; el dropdown por defecto de egui era ~85 px.
pub fn ancho_dropdown() -> f32 {
    tema::ANCHO_DROPDOWN_VELOCIDAD
}

pub fn mostrar(
    ui: &mut egui::Ui,
    state: &mut EditorApp,
    textos: &Textos,
    atajos: &MapaAtajos,
) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();
    let atajo = |comando| {
        atajos
            .atajo(comando)
            .map(|atajo| atajo.to_string())
            .unwrap_or_else(|| textos.sin_atajo.to_owned())
    };
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = tema::ESP_4;

        let inicio = atajo(Comando::NudgeIzquierda);
        if kit::icon_button(
            ui,
            Icono::Inicio,
            false,
            textos.inicio_transporte,
            Some(&inicio),
        )
        .clicked()
        {
            mensajes.push(MensajeEditor::Step(-state.duration));
        }
        let anterior = atajo(Comando::RetrocederFino);
        if kit::icon_button(
            ui,
            Icono::CuadroAnterior,
            false,
            textos.cuadro_anterior,
            Some(&anterior),
        )
        .clicked()
        {
            mensajes.push(MensajeEditor::Step(-0.04));
        }

        let reproducir = atajo(Comando::ReproducirPausa);
        let respuesta = ui
            .add(
                egui::Button::new(
                    RichText::new(kit::glifo(if state.paused {
                        Icono::Play
                    } else {
                        Icono::Pausa
                    }))
                    .font(egui::FontId::proportional(20.0))
                    .color(tema::TEXTO),
                )
                .fill(tema::CONTROL)
                .corner_radius(egui::CornerRadius::same(tema::RADIO_CONTROL as u8))
                .min_size(Vec2::splat(tema::ALTO_BOTON)),
            )
            .on_hover_ui(move |ui| {
                ui.label(
                    RichText::new(textos.comando(Comando::ReproducirPausa))
                        .font(tema::fuente(Estilo::Base)),
                );
                ui.label(RichText::new(&reproducir).font(tema::fuente(Estilo::Mono)));
            });
        if respuesta.clicked() {
            mensajes.push(MensajeEditor::PlayPause);
        }

        let siguiente = atajo(Comando::AvanzarFino);
        if kit::icon_button(
            ui,
            Icono::CuadroSiguiente,
            false,
            textos.cuadro_siguiente,
            Some(&siguiente),
        )
        .clicked()
        {
            mensajes.push(MensajeEditor::Step(0.04));
        }
        let fin = atajo(Comando::NudgeDerecha);
        if kit::icon_button(ui, Icono::Fin, false, textos.fin_transporte, Some(&fin)).clicked() {
            mensajes.push(MensajeEditor::Step(state.duration));
        }

        ui.add_space(tema::ESP_8);
        ui.label(
            RichText::new(crate::ui::texto::tiempo(state.tiempo_ui()))
                .font(tema::fuente(Estilo::TimecodeGrande))
                .color(tema::TEXTO),
        );
        ui.label(
            RichText::new(format!("/ {}", crate::ui::texto::tiempo(state.duration)))
                .font(tema::fuente(Estilo::Pequeno))
                .color(tema::TEXTO_3),
        );

        ui.add_space(tema::ESP_8);
        mensajes.extend(crate::ui::texto::mostrar(ui, state, textos));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let velocidad = egui::ComboBox::from_id_salt("velocidad")
                .width(ancho_dropdown())
                .selected_text(state.velocidad.to_string())
                .show_ui(ui, |ui| {
                    for valor in VELOCIDADES {
                        if ui
                            .selectable_value(&mut state.velocidad, valor, valor.to_string())
                            .clicked()
                        {
                            mensajes.push(MensajeEditor::Velocidad(valor));
                        }
                    }
                })
                .response;
            if velocidad.has_focus() {
                kit::anillo_foco(ui, &velocidad);
            }
            // Pista de audio: la lista y la activa viven en el editor, pero
            // mpv manda. El mensaje se reescribe desde mpv al volver.
            let pista_actual = state
                .selected_audio
                .as_ref()
                .map(|pista| pista.label.clone())
                .unwrap_or_else(|| textos.sin_audio.to_owned());
            let audio = egui::ComboBox::from_id_salt("pista-audio")
                .width(ancho_dropdown())
                .selected_text(pista_actual)
                .show_ui(ui, |ui| {
                    for pista in &state.audio_tracks {
                        let activa = state
                            .selected_audio
                            .as_ref()
                            .is_some_and(|seleccion| seleccion.id == pista.id);
                        if ui.selectable_label(activa, &pista.label).clicked() {
                            mensajes.push(MensajeEditor::AudioTrack(pista.clone()));
                        }
                    }
                })
                .response;
            if audio.has_focus() {
                kit::anillo_foco(ui, &audio);
            }

            // El volumen vive dentro del altavoz: un desplegable vertical
            // evita que el slider se pinte encima del campo de texto.
            let (respuesta, _) = egui::containers::menu::MenuButton::from_button(
                egui::Button::new(
                    RichText::new(kit::glifo(Icono::Volumen))
                        .font(egui::FontId::proportional(tema::TAMANO_ICONO))
                        .color(if state.silenciado {
                            tema::TEXTO_3
                        } else {
                            tema::TEXTO
                        }),
                )
                .frame(false)
                .min_size(Vec2::splat(tema::ANCHO_BOTON_28)),
            )
            .ui(ui, |ui| {
                ui.vertical(|ui| {
                    let mut volumen = state.volumen;
                    let slider = ui.add_sized(
                        Vec2::new(24.0, 112.0),
                        egui::Slider::new(&mut volumen, 0.0..=100.0)
                            .vertical()
                            .show_value(false),
                    );
                    if slider.changed() {
                        mensajes.push(MensajeEditor::Volumen(volumen));
                    }
                    ui.label(
                        RichText::new(format!("{}%", volumen as i32))
                            .font(tema::fuente(Estilo::Pequeno)),
                    );
                    let rotulo = if state.silenciado {
                        textos.activar_sonido
                    } else {
                        textos.silenciar
                    };
                    if ui.button(rotulo).clicked() {
                        mensajes.push(MensajeEditor::Silenciar);
                        ui.close();
                    }
                });
            });
            if respuesta.has_focus() {
                kit::anillo_foco(ui, &respuesta);
            }
        });
    });
    mensajes
}

/// Barra de estado bajo el transporte.
pub fn barra_estado(ui: &mut egui::Ui, state: &EditorApp, textos: &Textos) {
    ui.horizontal(|ui| {
        ui.set_height(tema::ALTO_CHIP);
        ui.spacing_mut().item_spacing.x = tema::ESP_8;
        ui.label(
            RichText::new(format!(
                "{} · {} · {}",
                textos.lineas(state.editor.proyecto().captions.len()),
                textos.hablantes(state.editor.proyecto().hablantes.len()),
                {
                    let captions = &state.editor.proyecto().captions;
                    let solapes = captions
                        .windows(2)
                        .filter(|par| par[1].inicio < par[0].fin)
                        .count();
                    textos.solapes(solapes)
                }
            ))
            .font(tema::fuente(Estilo::Pequeno))
            .color(tema::TEXTO_3),
        );
        ui.label(
            RichText::new(if sucio(state) {
                textos.sin_guardar
            } else {
                textos.guardado
            })
            .font(tema::fuente(Estilo::Pequeno))
            .color(tema::TEXTO_3),
        );
        if let Some(mensaje) = &state.mensaje {
            ui.label(
                RichText::new(mensaje)
                    .font(tema::fuente(Estilo::Pequeno))
                    .color(tema::ERROR_TEXTO),
            );
        }
    });
}
