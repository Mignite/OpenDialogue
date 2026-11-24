//! Cabecera de ventana propia: marca, menús, nombre del proyecto y controles.

use crate::comandos::{Comando, MapaAtajos};
use crate::menu;
use crate::textos::Textos;

use super::kit::Icono;
use super::tema::{self, Estilo};

/// Lo que la cabecera necesita saber del estado de la app.
pub struct EstadoCabecera<'a> {
    pub en_inicio: bool,
    pub proyecto: Option<&'a str>,
    pub sucio: bool,
}

/// Solo los menús, para pruebas de accesibilidad sin ventana.
pub fn mostrar_menu_para_test(ui: &mut egui::Ui, textos: &Textos, atajos: &MapaAtajos) {
    menu::mostrar(ui, false, textos, atajos);
}

/// Dibuja la cabecera y devuelve el comando elegido en los menús.
pub fn mostrar(
    ctx: &egui::Context,
    textos: &Textos,
    atajos: &MapaAtajos,
    estado: EstadoCabecera<'_>,
) -> Option<Comando> {
    let marco = egui::Frame {
        inner_margin: egui::Margin::symmetric(tema::ESP_12 as i8, 0),
        outer_margin: egui::Margin::ZERO,
        corner_radius: egui::CornerRadius::ZERO,
        shadow: egui::Shadow::NONE,
        fill: tema::FONDO,
        stroke: egui::Stroke::new(1.0_f32, tema::BORDE),
    };
    let mut elegido = None;
    egui::TopBottomPanel::top("cabecera")
        .exact_height(tema::ALTO_CABECERA)
        .frame(marco)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = tema::ESP_8;

                monograma(ui);
                ui.label(
                    egui::RichText::new("OpenDialogue")
                        .font(egui::FontId::proportional(
                            tema::fuente(Estilo::CuerpoFuerte).size,
                        ))
                        .color(tema::TEXTO),
                );

                elegido = menu::mostrar(ui, estado.en_inicio, textos, atajos);

                // Reservar primero el sitio de los controles: si la franja de
                // arrastre toma todo el ancho, los botones se salen del rect.
                let ancho_botones = 3.0 * tema::ANCHO_BOTON_VENTANA;
                let ancho_arrastre = (ui.available_width() - ancho_botones).max(4.0);
                let (franja, respuesta) = ui.allocate_exact_size(
                    egui::vec2(ancho_arrastre, tema::ALTO_CABECERA - 8.0),
                    egui::Sense::click_and_drag(),
                );
                if respuesta.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if respuesta.double_clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(
                        !ctx.input(|entrada| entrada.viewport().maximized.unwrap_or(false)),
                    ));
                }
                if let Some(proyecto) = estado.proyecto {
                    let texto = if estado.sucio {
                        format!("{proyecto} · {}", textos.sin_guardar)
                    } else {
                        format!("{proyecto} · {}", textos.guardado)
                    };
                    ui.painter().text(
                        franja.center(),
                        egui::Align2::CENTER_CENTER,
                        texto,
                        tema::fuente(Estilo::Pequeno),
                        tema::TEXTO_2,
                    );
                }

                ui.spacing_mut().item_spacing.x = 0.0;
                boton_ventana(ui, Icono::Minimizar, tema::TEXTO_MENU, "minimizar", || {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                });
                boton_ventana(ui, Icono::Maximizar, tema::TEXTO_MENU, "maximizar", || {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(
                        !ctx.input(|entrada| entrada.viewport().maximized.unwrap_or(false)),
                    ));
                });
                boton_ventana(ui, Icono::Cerrar, tema::PELIGRO, "cerrar", || {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                });
            });
        });
    elegido
}

fn monograma(ui: &mut egui::Ui) {
    let (rect, _) =
        ui.allocate_exact_size(egui::Vec2::splat(tema::ALTO_CHIP), egui::Sense::hover());
    ui.painter().rect_filled(
        rect,
        egui::CornerRadius::same(tema::RADIO_CONTROL as u8),
        tema::PANEL,
    );
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(tema::RADIO_CONTROL as u8),
        egui::Stroke::new(1.0_f32, tema::BORDE_CONTROL),
        egui::StrokeKind::Inside,
    );
    let colores = [
        tema::color("#F2B84B"),
        tema::color("#B39BFF"),
        tema::color("#3FC4B8"),
    ];
    for (i, color) in colores.into_iter().enumerate() {
        let y = rect.min.y + 6.0 + i as f32 * 5.0;
        ui.painter().line_segment(
            [
                egui::pos2(rect.min.x + 5.0, y),
                egui::pos2(rect.max.x - 5.0, y),
            ],
            egui::Stroke::new(2.0_f32, color),
        );
    }
}

fn boton_ventana(
    ui: &mut egui::Ui,
    icono: Icono,
    color: egui::Color32,
    tooltip: &str,
    accion: impl FnOnce(),
) {
    let respuesta = ui.add(
        egui::Button::new(
            egui::RichText::new(super::kit::glifo(icono))
                .font(egui::FontId::proportional(tema::TAMANO_ICONO))
                .color(color),
        )
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::NONE)
        .min_size(egui::vec2(tema::ANCHO_BOTON_VENTANA, tema::ALTO_CABECERA)),
    );
    if respuesta.hovered() {
        ui.painter().rect_filled(respuesta.rect, 0.0, tema::CONTROL);
    }
    if respuesta.clicked() {
        accion();
    }
    let _ = tooltip;
}
