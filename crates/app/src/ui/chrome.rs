//! Cabecera de ventana propia: marca, menús, nombre del proyecto y controles.

use crate::comandos::{Comando, MapaAtajos};
use crate::menu;
use crate::textos::Textos;

use super::kit::{self, Icono};
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

/// Color del punto de estado delante de «Saved»/«Unsaved».
pub fn color_punto_estado(sucio: bool) -> egui::Color32 {
    if sucio { tema::AMBAR } else { tema::TEXTO_2 }
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
        stroke: egui::Stroke::new(tema::GROSOR_BORDE, tema::BORDE),
    };
    let mut elegido = None;
    egui::TopBottomPanel::top("cabecera")
        .exact_height(tema::ALTO_CABECERA)
        .frame(marco)
        .show(ctx, |ui| {
            // Align::Center: la marca, los menús y los botones comparten el
            // centro vertical de la cabecera (FilmCraft header.rs).
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = tema::ESP_12;

                kit::logo(ui, tema::TAMANO_LOGO);
                ui.label(
                    egui::RichText::new("OpenDialogue")
                        .font(egui::FontId::proportional(
                            tema::fuente(Estilo::CuerpoFuerte).size,
                        ))
                        .color(tema::TEXTO),
                );

                ui.add_space(tema::ESP_16);
                elegido = menu::mostrar(ui, estado.en_inicio, textos, atajos);
                ui.add_space(tema::ESP_16);

                // Reservar primero el sitio de los controles: si la franja de
                // arrastre toma todo el ancho, los botones se salen del rect.
                let ancho_botones = 3.0 * tema::ANCHO_BOTON_VENTANA + tema::ESP_12;
                let ancho_arrastre = (ui.available_width() - ancho_botones).max(tema::ESP_4);
                let (_franja, respuesta) = ui.allocate_exact_size(
                    egui::vec2(ancho_arrastre, tema::ALTO_CABECERA - tema::ESP_8),
                    egui::Sense::click_and_drag(),
                );
                respuesta.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::Button,
                        true,
                        textos.arrastrar_ventana,
                    )
                });
                let respuesta = respuesta.on_hover_text(textos.arrastrar_ventana);
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
                        format!("{} · {}", tema::acortar(proyecto, 40), textos.sin_guardar)
                    } else {
                        format!("{} · {}", tema::acortar(proyecto, 40), textos.guardado)
                    };
                    // Centro real de la ventana, no de la franja de arrastre:
                    // la franja empieza tras los menús y acababa desplazando
                    // el título hacia la derecha.
                    let centro = ui.max_rect().center();
                    let fuente_titulo = tema::fuente(Estilo::Pequeno);
                    // `len() * size * 0,5` fallaba con títulos no ASCII: la
                    // galley da el ancho real de la cadena ya compuesta.
                    let ancho_titulo = ui
                        .painter()
                        .layout_no_wrap(texto.clone(), fuente_titulo.clone(), tema::TEXTO_2)
                        .size()
                        .x;
                    ui.painter().circle_filled(
                        egui::pos2(centro.x - ancho_titulo * 0.5 - tema::ESP_8, centro.y),
                        tema::ESP_4,
                        color_punto_estado(estado.sucio),
                    );
                    ui.painter().text(
                        centro,
                        egui::Align2::CENTER_CENTER,
                        texto,
                        fuente_titulo,
                        tema::TEXTO_2,
                    );
                }

                ui.spacing_mut().item_spacing.x = 0.0;
                boton_ventana(ui, Icono::Minimizar, textos.minimizar, || {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                });
                boton_ventana(ui, Icono::Maximizar, textos.maximizar, || {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(
                        !ctx.input(|entrada| entrada.viewport().maximized.unwrap_or(false)),
                    ));
                });
                boton_ventana(ui, Icono::Cerrar, textos.cerrar_ventana, || {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                });
            });
        });
    elegido
}

/// Botón de ventana sin caja en reposo: solo el glifo. El realce es un chip
/// interior para que minimizar, maximizar y cerrar no se lean como un bloque
/// de cuadrados pegados.
fn boton_ventana(ui: &mut egui::Ui, icono: Icono, tooltip: &str, accion: impl FnOnce()) {
    let (rect, respuesta) = ui.allocate_exact_size(
        egui::vec2(tema::ANCHO_BOTON_VENTANA, tema::ALTO_CABECERA),
        egui::Sense::click(),
    );
    let cerrar = icono == Icono::Cerrar;
    let realzado = respuesta.hovered() || respuesta.is_pointer_button_down_on();
    if realzado {
        let chip = egui::Rect::from_center_size(
            rect.center(),
            egui::vec2(
                tema::ANCHO_BOTON_VENTANA - tema::ESP_8,
                tema::ALTO_BOTON_CHICO,
            ),
        );
        ui.painter().rect_filled(
            chip,
            tema::radio_control(),
            if cerrar { tema::PELIGRO } else { tema::CONTROL },
        );
    }
    let color = if realzado {
        tema::TEXTO
    } else {
        tema::TEXTO_MENU
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        kit::glifo(icono),
        egui::FontId::proportional(tema::TAMANO_ICONO),
        color,
    );
    if respuesta.has_focus() {
        kit::anillo_foco(ui, &respuesta);
    }
    if respuesta.clicked() {
        accion();
    }
    respuesta.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tooltip));
    let _ = respuesta.on_hover_ui(|ui| {
        ui.label(egui::RichText::new(tooltip).font(tema::fuente(Estilo::Base)));
    });
}
