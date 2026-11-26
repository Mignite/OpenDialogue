//! Capa modal de Ajustes: idioma y atajos reasignables.

use egui::{Event, Key, RichText, Vec2};

use crate::comandos::{Atajo, Comando, Conflicto};
use crate::preferencias::Preferencias;
use crate::textos::Idioma;

use super::kit::{self, Icono};
use super::tema::{self, Estilo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pestana {
    Atajos,
    #[default]
    Idioma,
}

#[derive(Debug, Default)]
pub struct EstadoAjustes {
    pub abierto: bool,
    pub pestana: Pestana,
    pub capturando: Option<Comando>,
    pub busqueda: String,
    /// Comando que ya tiene la tecla recién pulsada.
    pub conflicto: Option<(Comando, Atajo)>,
    /// La capa se abrió en este frame.
    recien_abierto: bool,
    /// El clic que la abrió sigue en curso: esperar a que el ratón se suelte
    /// antes de aceptar «clic fuera». Con el menú, el comando llega al pulsar y
    /// el botón se suelta en un frame posterior.
    ignorar_clic: bool,
}

impl EstadoAjustes {
    /// Abre la capa. El clic que la abre no debe cerrarla.
    pub fn abrir(&mut self) {
        self.abierto = true;
        self.recien_abierto = true;
        self.ignorar_clic = false;
    }
}

/// Dibuja la capa. Devuelve `true` si cambió algo y hay que guardar.
fn cerrar(estado: &mut EstadoAjustes) {
    estado.abierto = false;
    estado.recien_abierto = false;
    estado.ignorar_clic = false;
    estado.capturando = None;
    estado.conflicto = None;
}

/// Ancho de una cadena con la fuente dada, para alinear columnas sin estimar.
fn ancho_texto(ui: &egui::Ui, texto: &str, fuente: egui::FontId) -> f32 {
    ui.painter()
        .layout_no_wrap(texto.to_owned(), fuente, tema::TEXTO)
        .size()
        .x
}

pub fn mostrar(ctx: &egui::Context, prefs: &mut Preferencias, estado: &mut EstadoAjustes) -> bool {
    if !estado.abierto {
        return false;
    }
    if std::mem::take(&mut estado.recien_abierto) {
        estado.ignorar_clic = true;
    }
    let idioma_inicial = prefs.idioma;
    let textos = idioma_inicial.textos();
    let mut cambiado = false;

    // Velo oscuro: la capa no es una pantalla, el proyecto sigue ahí detrás.
    ctx.layer_painter(egui::LayerId::background()).rect_filled(
        ctx.content_rect(),
        0.0,
        egui::Color32::from_black_alpha(115),
    );

    // Captura de teclado antes de pintar: una pulsación async no se pierde.
    let eventos: Vec<Event> = ctx.input(|entrada| entrada.events.clone());
    if let Some(comando) = estado.capturando {
        for evento in &eventos {
            if let Event::Key {
                key,
                modifiers,
                pressed: true,
                ..
            } = evento
            {
                if *key == Key::Escape {
                    estado.capturando = None;
                    continue;
                }
                let atajo = Atajo {
                    tecla: *key,
                    ctrl: modifiers.ctrl || modifiers.command || modifiers.mac_cmd,
                    shift: modifiers.shift,
                    alt: modifiers.alt,
                };
                match prefs.atajos.asignar(comando, atajo) {
                    Ok(()) => {
                        estado.capturando = None;
                        estado.conflicto = None;
                        cambiado = true;
                    }
                    Err(Conflicto) => {
                        let otro = Comando::todas()
                            .iter()
                            .copied()
                            .find(|c| prefs.atajos.atajo(*c) == Some(atajo));
                        estado.conflicto = Some((otro.unwrap_or(comando), atajo));
                    }
                }
                break;
            }
        }
    }
    if eventos.iter().any(|evento| {
        matches!(
            evento,
            Event::Key {
                key: Key::Escape,
                pressed: true,
                ..
            }
        )
    }) && estado.capturando.is_none()
    {
        cerrar(estado);
    }

    // El modal escala con la ventana: nunca más ancho que la pantalla ni más
    // estrecho que un panel cómodo, con los topes tomados de `tema`.
    let pantalla = ctx.content_rect().size();
    let ancho = (pantalla.x * 0.45).clamp(tema::ANCHO_PANEL_DEFECTO * 2.0, tema::ANCHO_CONTENEDOR);
    let alto = (pantalla.y * 0.62).clamp(tema::ALTO_VIDEO_MIN * 4.0, tema::ESP_72 * 10.0);
    let marco = egui::Frame {
        inner_margin: egui::Margin::same(tema::ESP_16 as i8),
        outer_margin: egui::Margin::ZERO,
        corner_radius: egui::CornerRadius::same(tema::RADIO_SUPERFICIE as u8),
        shadow: egui::Shadow::NONE,
        fill: tema::PANEL,
        stroke: egui::Stroke::new(tema::GROSOR_BORDE, tema::BORDE_CONTROL),
    };
    let respuesta = egui::Window::new(textos.ajustes)
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .fixed_size(Vec2::new(ancho, alto))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(marco)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (pestana, etiqueta) in [
                    (Pestana::Atajos, textos.pestana_atajos),
                    (Pestana::Idioma, textos.pestana_idioma),
                ] {
                    let activo = estado.pestana == pestana;
                    let r = ui.add(
                        egui::Button::new(RichText::new(etiqueta).font(tema::fuente(Estilo::Base)))
                            .frame(false),
                    );
                    if activo {
                        ui.painter().rect_filled(
                            egui::Rect::from_min_max(
                                egui::pos2(r.rect.min.x, r.rect.max.y - tema::ESP_2),
                                r.rect.max,
                            ),
                            0.0,
                            tema::ACENTO,
                        );
                    }
                    if r.has_focus() {
                        kit::anillo_foco(ui, &r);
                    }
                    if r.clicked() {
                        estado.pestana = pestana;
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if kit::icon_button(ui, Icono::Cerrar, false, textos.cerrar_ventana, None)
                        .clicked()
                    {
                        cerrar(estado);
                    }
                });
            });
            ui.separator();

            match estado.pestana {
                Pestana::Idioma => {
                    // `radio_value` usa el estilo Body de egui, distinto del
                    // Base del resto de la capa: se dibuja el widget con
                    // RichText para que comparta tipografía.
                    for (idioma, etiqueta) in [(Idioma::Es, "Español"), (Idioma::En, "English")] {
                        let r = ui.add(egui::RadioButton::new(
                            prefs.idioma == idioma,
                            RichText::new(etiqueta).font(tema::fuente(Estilo::Base)),
                        ));
                        if r.has_focus() {
                            kit::anillo_foco(ui, &r);
                        }
                        if r.clicked() {
                            prefs.idioma = idioma;
                        }
                    }
                    if prefs.idioma != idioma_inicial {
                        cambiado = true;
                    }
                }
                Pestana::Atajos => {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(textos.buscar).font(tema::fuente(Estilo::Pequeno)));
                        ui.add(
                            egui::TextEdit::singleline(&mut estado.busqueda)
                                .font(tema::fuente(Estilo::Base)),
                        );
                    });
                    if let Some((otro, atajo)) = estado.conflicto {
                        ui.label(
                            RichText::new(format!("{} {otro:?} · {atajo}", textos.intercambiar))
                                .font(tema::fuente(Estilo::Pequeno))
                                .color(tema::ERROR_TEXTO),
                        );
                        let r = ui.add(
                            egui::Button::new(
                                RichText::new(textos.intercambiar).font(tema::fuente(Estilo::Base)),
                            )
                            .frame(false),
                        );
                        if r.has_focus() {
                            kit::anillo_foco(ui, &r);
                        }
                        if r.clicked() {
                            if let Some(comando) = estado.capturando {
                                prefs.atajos.intercambiar(comando, otro);
                                cambiado = true;
                            }
                            estado.conflicto = None;
                            estado.capturando = None;
                        }
                        let r = ui.add(
                            egui::Button::new(
                                RichText::new(textos.cancelar).font(tema::fuente(Estilo::Base)),
                            )
                            .frame(false),
                        );
                        if r.has_focus() {
                            kit::anillo_foco(ui, &r);
                        }
                        if r.clicked() {
                            estado.conflicto = None;
                            estado.capturando = None;
                        }
                    }
                    let fuente_base = tema::fuente(Estilo::Base);
                    let fuente_mono = tema::fuente(Estilo::Mono);
                    let busqueda = estado.busqueda.to_lowercase();
                    let visibles: Vec<_> = Comando::todas()
                        .iter()
                        .copied()
                        .filter(|comando| {
                            textos.comando(*comando).to_lowercase().contains(&busqueda)
                        })
                        .collect();
                    // Columnas medidas sobre la fuente real: la etiqueta no se
                    // repite en el botón y el atajo queda a la derecha.
                    let ancho_etiqueta = visibles
                        .iter()
                        .map(|comando| {
                            ancho_texto(ui, textos.comando(*comando), fuente_base.clone())
                        })
                        .fold(0.0, f32::max)
                        + tema::ESP_12;
                    let ancho_atajo = visibles
                        .iter()
                        .filter_map(|comando| prefs.atajos.atajo(*comando))
                        .map(|atajo| ancho_texto(ui, &atajo.to_string(), fuente_mono.clone()))
                        .fold(0.0, f32::max)
                        .max(tema::ANCHO_KEYCAP_MIN);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("atajos-ajustes")
                            .num_columns(4)
                            .spacing(egui::vec2(tema::ESP_12, tema::ESP_4))
                            .show(ui, |ui| {
                                for comando in visibles {
                                    let etiqueta = textos.comando(comando);
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(ancho_etiqueta, tema::ALTO_CONTROL),
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(etiqueta).font(fuente_base.clone()),
                                            );
                                        },
                                    );
                                    let atajo = prefs
                                        .atajos
                                        .atajo(comando)
                                        .map(|atajo| atajo.to_string())
                                        .unwrap_or_default();
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(ancho_atajo, tema::ALTO_CONTROL),
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(atajo)
                                                    .font(fuente_mono.clone())
                                                    .color(tema::TEXTO_2),
                                            );
                                        },
                                    );
                                    let r = ui.add(
                                        egui::Button::new(
                                            RichText::new(textos.cambiar).font(fuente_base.clone()),
                                        )
                                        .min_size(
                                            egui::vec2(
                                                tema::ANCHO_BOTON_MIN / 2.0,
                                                tema::ALTO_CONTROL,
                                            ),
                                        ),
                                    );
                                    if r.has_focus() {
                                        kit::anillo_foco(ui, &r);
                                    }
                                    if r.clicked() {
                                        estado.capturando = Some(comando);
                                        estado.conflicto = None;
                                    }
                                    let r = ui.add(
                                        egui::Button::new(
                                            RichText::new(textos.restablecer)
                                                .font(fuente_base.clone()),
                                        )
                                        .min_size(
                                            egui::vec2(
                                                tema::ANCHO_BOTON_MIN / 2.0,
                                                tema::ALTO_CONTROL,
                                            ),
                                        ),
                                    );
                                    if r.has_focus() {
                                        kit::anillo_foco(ui, &r);
                                    }
                                    if r.clicked() {
                                        prefs.atajos.restablecer(comando);
                                        cambiado = true;
                                    }
                                    ui.end_row();
                                }
                            });
                    });
                }
            }

            if estado.capturando.is_some() {
                ui.label(
                    RichText::new(textos.pulsa_una_tecla)
                        .font(tema::fuente(Estilo::Pequeno))
                        .color(tema::ACENTO),
                );
            }
        });

    // Clic fuera para cerrar. Se descarta mientras siga en curso el clic que
    // abrió la capa: si no, abrir desde el menú con el ratón la cerraba al
    // soltar el botón.
    if estado.ignorar_clic {
        estado.ignorar_clic =
            ctx.input(|entrada| entrada.pointer.any_down() || entrada.pointer.any_click());
    }
    if let Some(respuesta) = respuesta {
        let clic_fuera = !estado.ignorar_clic
            && ctx.input(|entrada| {
                entrada.pointer.button_clicked(egui::PointerButton::Primary)
                    && entrada
                        .pointer
                        .latest_pos()
                        .is_some_and(|pos| !respuesta.response.rect.contains(pos))
            });
        if clic_fuera {
            cerrar(estado);
        }
    }
    cambiado
}
