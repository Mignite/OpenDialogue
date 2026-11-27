//! Panel de atajos de teclado (`?` / F1).

use egui::{Event, Key, RichText};

use crate::comandos::{Comando, MapaAtajos, Seccion};
use crate::textos::Textos;

use super::kit::{self, Icono};
use super::tema::{self, Estilo};

#[derive(Debug, Default)]
pub struct EstadoAtajos {
    pub abierto: bool,
}

/// Ancho de una cadena con la fuente dada, para alinear columnas sin estimar.
fn ancho_texto(ui: &egui::Ui, texto: &str, fuente: egui::FontId) -> f32 {
    ui.painter()
        .layout_no_wrap(texto.to_owned(), fuente, tema::TEXTO)
        .size()
        .x
}

/// Dibuja el panel. `Esc` lo cierra.
pub fn mostrar(
    ctx: &egui::Context,
    textos: &Textos,
    atajos: &MapaAtajos,
    estado: &mut EstadoAtajos,
) {
    if !estado.abierto {
        return;
    }
    let eventos: Vec<Event> = ctx.input(|entrada| entrada.events.clone());
    if eventos.iter().any(|evento| {
        matches!(
            evento,
            Event::Key {
                key: Key::Escape,
                pressed: true,
                ..
            }
        )
    }) {
        estado.abierto = false;
        return;
    }

    // Tamaño derivado de la ventana, con topes del tema para que no se coma
    // la pantalla ni quede ilegible en las medidas pequeñas.
    let pantalla = ctx.content_rect().size();
    let ancho =
        (pantalla.x * 0.35).clamp(tema::ANCHO_PANEL_DEFECTO * 1.5, tema::ANCHO_FILTRO * 4.0);
    let alto = (pantalla.y - tema::ESP_32 * 2.0).min(tema::ESP_72 * 8.0);
    let marco = egui::Frame {
        inner_margin: egui::Margin::same(tema::ESP_16 as i8),
        fill: tema::PANEL,
        stroke: egui::Stroke::new(tema::GROSOR_BORDE, tema::BORDE_CONTROL),
        corner_radius: egui::CornerRadius::same(tema::RADIO_SUPERFICIE as u8),
        ..egui::Frame::NONE
    };
    egui::Window::new(textos.comando(Comando::Ayuda))
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(ancho, alto))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .frame(marco)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if kit::icon_button(ui, Icono::Cerrar, false, textos.cerrar_ventana, None)
                        .clicked()
                    {
                        estado.abierto = false;
                    }
                });
            });
            let fuente_base = tema::fuente(Estilo::Base);
            let ancho_etiqueta = Comando::todas()
                .iter()
                .map(|comando| ancho_texto(ui, textos.comando(*comando), fuente_base.clone()))
                .fold(0.0, f32::max)
                + tema::ESP_12;
            ui.visuals_mut().widgets.inactive.bg_fill = tema::CONTROL;
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .max_height(alto - tema::ESP_72 * 2.0)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    let mut seccion_actual = None;
                    for comando in Comando::todas() {
                        let seccion = crate::comandos::seccion(*comando);
                        if Some(seccion) != seccion_actual {
                            seccion_actual = Some(seccion);
                            ui.add_space(tema::ESP_8);
                            // Etiqueta de sección con familia siempre registrada:
                            // `set_fonts` solo activa las fuentes nombradas en el
                            // siguiente pass, y este panel se dibuja en el primero.
                            tema::etiqueta(ui, nombre_seccion(seccion, textos));
                        }
                        ui.horizontal(|ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(ancho_etiqueta, tema::ALTO_CONTROL),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        RichText::new(textos.comando(*comando))
                                            .font(fuente_base.clone())
                                            .color(tema::TEXTO),
                                    );
                                },
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let atajo = atajos
                                        .atajo(*comando)
                                        .map(|atajo| atajo.to_string())
                                        .unwrap_or_else(|| textos.sin_atajo.to_owned());
                                    // `?` es ambiguo en la fuente mono: se explicita
                                    // con la clave «None»/«Sin atajo».
                                    let atajo: &str = if atajo == "?" {
                                        textos.sin_atajo
                                    } else {
                                        atajo.as_str()
                                    };
                                    ui.label(
                                        RichText::new(atajo)
                                            .font(tema::fuente(Estilo::Mono))
                                            .color(tema::TEXTO_2),
                                    );
                                },
                            );
                        });
                    }
                });
            let r = ui.button(textos.cancelar);
            if r.has_focus() {
                kit::anillo_foco(ui, &r);
            }
            if r.clicked() {
                estado.abierto = false;
            }
        });
}

fn nombre_seccion(seccion: Seccion, textos: &Textos) -> &'static str {
    match seccion {
        Seccion::Archivo => textos.seccion_archivo,
        Seccion::Edicion => textos.seccion_edicion,
        Seccion::Ver => textos.seccion_ver,
        Seccion::Exportar => textos.seccion_exportar,
        Seccion::Estilos => textos.seccion_estilos,
        Seccion::Ayuda => textos.seccion_ayuda,
    }
}
