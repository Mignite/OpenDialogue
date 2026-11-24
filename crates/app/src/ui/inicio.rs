//! La pantalla de arranque. Solo dibuja y devuelve la intención del usuario;
//! `rod_app::inicio::update` sigue siendo quien decide qué hacer con ella.

use egui::Ui;

use crate::inicio::{EstadoInicio, MensajeInicio, cantidad_visible};
use crate::ui::tema;

pub fn mostrar(ui: &mut Ui, estado: &mut EstadoInicio) -> Option<MensajeInicio> {
    // Hasta la tarea 9, la vista toma el idioma por defecto.
    let textos = crate::textos::Idioma::default().textos();
    let mut intencion = None;
    let altura = ui.available_height();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        // Rail estrecho: identidad y navegación, sin repetir la cabecera.
        ui.allocate_ui_with_layout(
            egui::vec2(88.0, altura),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.add_space(28.0);
                let (marca, _) =
                    ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(marca, egui::CornerRadius::same(5), tema::ACENTO);
                ui.painter().text(
                    marca.center(),
                    egui::Align2::CENTER_CENTER,
                    "R",
                    egui::FontId::proportional(16.0),
                    egui::Color32::WHITE,
                );
                ui.add_space(24.0);
                tema::etiqueta(ui, "INICIO");
                ui.add_space(10.0);
                tema::etiqueta(ui, "RECIENTES");
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("v0.9")
                            .font(tema::mono_public())
                            .size(9.0)
                            .color(tema::DIM),
                    );
                });
            },
        );
        ui.separator();

        // Una sola columna con ancho máximo, centrada: el contenido manda, no
        // el lienzo vacío de una ventana de 1600 px.
        let disponible = ui.available_width();
        let ancho = disponible.min(1040.0);
        ui.add_space(((disponible - ancho) / 2.0).max(0.0));
        ui.allocate_ui_with_layout(
            egui::vec2(ancho, altura),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.add_space(70.0);
                ui.label(
                    egui::RichText::new("Edita subtítulos con precisión")
                        .size(30.0)
                        .strong()
                        .color(tema::TEXTO),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("Abre un video o continúa un proyecto guardado.")
                        .size(14.0)
                        .color(tema::SECUNDARIO),
                );
                ui.add_space(26.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    let principal = ui.add(
                        egui::Button::new(
                            egui::RichText::new("Abrir video")
                                .size(13.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(tema::ACENTO)
                        .corner_radius(egui::CornerRadius::same(5))
                        .min_size(egui::vec2(152.0, 38.0)),
                    );
                    if principal.clicked() {
                        intencion = Some(MensajeInicio::AbrirVideo);
                    }
                    let secundario = ui.add(
                        egui::Button::new(egui::RichText::new("Abrir proyecto").size(13.0))
                            .fill(tema::PANEL)
                            .corner_radius(egui::CornerRadius::same(5))
                            .stroke(egui::Stroke::new(1.0_f32, tema::BORDE))
                            .min_size(egui::vec2(140.0, 38.0)),
                    );
                    if secundario.clicked() {
                        intencion = Some(MensajeInicio::AbrirProyecto);
                    }
                });
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("También puedes soltar un MP4, MKV, MOV o WebM aquí.")
                        .size(11.0)
                        .color(tema::DIM),
                );

                if let Some(aviso) = &estado.aviso {
                    ui.add_space(20.0);
                    let marco = egui::Frame {
                        inner_margin: egui::Margin::symmetric(14, 12),
                        corner_radius: egui::CornerRadius::same(6),
                        stroke: egui::Stroke::new(1.0_f32, tema::AMBAR),
                        ..tema::panel(tema::LATERAL)
                    };
                    marco.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("●").size(12.0).color(tema::AMBAR));
                            ui.label(egui::RichText::new(&aviso.texto).size(12.0));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("Descartar").clicked() {
                                        intencion = Some(MensajeInicio::DescartarRecovery);
                                    }
                                    if ui.button("Recuperar").clicked() {
                                        intencion = Some(MensajeInicio::Recuperar);
                                    }
                                },
                            );
                        });
                    });
                }
                if let Some(mensaje) = &estado.mensaje {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(mensaje).size(12.0).color(tema::AMBAR));
                }

                ui.add_space(40.0);
                tema::etiqueta(ui, "Recientes");
                ui.add_space(10.0);
                if estado.recientes.is_empty() {
                    ui.label(
                        egui::RichText::new(textos.texto_vacio_recientes)
                            .size(12.0)
                            .color(tema::DIM),
                    );
                } else {
                    let visibles = cantidad_visible(estado.recientes.len(), estado.expandido);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(12.0, 10.0);
                        for reciente in estado.recientes.iter().take(visibles) {
                            let respuesta = tema::panel(tema::PANEL).show(ui, |ui| {
                                ui.set_min_size(egui::vec2(244.0, 104.0));
                                ui.vertical(|ui| {
                                    ui.add_space(2.0);
                                    ui.label(
                                        egui::RichText::new("R")
                                            .font(tema::mono_public())
                                            .size(11.0)
                                            .color(tema::ACENTO),
                                    );
                                    ui.add_space(12.0);
                                    ui.label(
                                        egui::RichText::new(
                                            reciente
                                                .ruta
                                                .file_name()
                                                .map(|nombre| nombre.to_string_lossy().into_owned())
                                                .unwrap_or_else(|| {
                                                    reciente.ruta.display().to_string()
                                                }),
                                        )
                                        .strong()
                                        .size(13.0),
                                    );
                                    ui.add_space(4.0);
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} · {} · {}",
                                            textos.lineas(reciente.lineas),
                                            textos.hablantes(reciente.hablantes),
                                            textos.fecha(&reciente.fecha)
                                        ))
                                        .font(tema::mono_public())
                                        .size(10.0)
                                        .color(tema::DIM),
                                    );
                                });
                            });
                            if respuesta.response.clicked() {
                                intencion =
                                    Some(MensajeInicio::AbrirReciente(reciente.ruta.clone()));
                            }
                        }
                    });
                    if estado.recientes.len() > 3 && ui.button("Ver todos").clicked() {
                        intencion = Some(MensajeInicio::VerTodos);
                    }
                }
            },
        );
    });
    intencion
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::inicio;
    use crate::recientes::Reciente;
    use egui_kittest::kittest::Queryable;

    fn tres_recientes() -> Vec<Reciente> {
        (1..=3)
            .map(|n| Reciente {
                ruta: PathBuf::from(format!("C:/proyectos/{n}.cdp")),
                lineas: n,
                hablantes: 1,
                fecha: Default::default(),
                duracion: 0.0,
                tira: Vec::new(),
            })
            .collect()
    }

    #[test]
    fn arranque_muestra_tres_recientes_y_boton_abrir() {
        let estado = inicio::nuevo(tres_recientes(), None);
        let mut harness = egui_kittest::Harness::new_ui_state(
            |ui, estado: &mut inicio::EstadoInicio| {
                super::mostrar(ui, estado);
            },
            estado,
        );
        assert!(harness.query_by_label("Abrir video").is_some());
        assert!(harness.query_by_label_contains("1.cdp").is_some());
        assert!(harness.query_by_label_contains("4.cdp").is_none());
    }
}
