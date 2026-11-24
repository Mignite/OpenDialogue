//! Vista del editor en egui. Compone el preview, la barra de transporte, los
//! paneles laterales y la barra de estado. Devuelve **todos** los mensajes del
//! frame: sobrescribirlos habría dejado los botones inertes.

use std::sync::{Arc, Mutex};

use egui::Ui;

use crate::editor_app::{EditorApp, MensajeEditor, Panel, sucio};
use crate::ui::paneles::{estilos, hablantes, subtitulos};
use crate::ui::{tema, transporte};
use crate::video::Gpu;

/// Distribución real: inspector a la derecha, timeline abajo, monitor al centro.
pub fn show(
    ctx: &egui::Context,
    state: &mut EditorApp,
    formato: wgpu::TextureFormat,
    gpu: Arc<Mutex<Option<Gpu>>>,
) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();

    egui::SidePanel::right("inspector")
        .resizable(true)
        .default_width(300.0)
        .min_width(240.0)
        .frame(tema::panel(tema::PANEL))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (panel, etiqueta) in [
                    (Panel::Subtitulos, "SUBTÍTULOS"),
                    (Panel::Hablantes, "HABLANTES"),
                    (Panel::Estilos, "ESTILOS"),
                ] {
                    let activo = state.panel == panel;
                    let boton = ui.add(
                        egui::Button::new(
                            egui::RichText::new(etiqueta)
                                .font(tema::mono_public())
                                .size(9.0)
                                .color(if activo {
                                    egui::Color32::WHITE
                                } else {
                                    tema::DIM
                                }),
                        )
                        .frame(false)
                        .fill(if activo {
                            tema::ACENTO
                        } else {
                            egui::Color32::TRANSPARENT
                        })
                        .corner_radius(egui::CornerRadius::same(3)),
                    );
                    if boton.clicked() {
                        mensajes.push(MensajeEditor::Panel(panel));
                    }
                }
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| match state.panel {
                Panel::Subtitulos => mensajes.extend(subtitulos::mostrar(ui, state)),
                Panel::Hablantes => mensajes.extend(hablantes::mostrar(ui, state)),
                Panel::Estilos => mensajes.extend(estilos::mostrar(ui, state)),
            });
        });

    egui::TopBottomPanel::bottom("timeline")
        .resizable(true)
        .default_height(224.0)
        .min_height(120.0)
        .frame(tema::panel(tema::SUPERFICIE))
        .show(ctx, |ui| {
            mensajes.extend(crate::ui::timeline::mostrar(ui, state));
        });

    egui::CentralPanel::default()
        .frame(tema::panel(tema::FONDO))
        .show(ctx, |ui| {
            mensajes.extend(mostrar(ui, state, formato, gpu));
        });

    mensajes
}

pub fn mostrar(
    ui: &mut Ui,
    state: &mut EditorApp,
    formato: wgpu::TextureFormat,
    gpu: Arc<Mutex<Option<Gpu>>>,
) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();

    // Preview real: el mismo pipeline de video + libass que antes, ahora a
    // través de un paint callback de egui_wgpu.
    let alto = (ui.available_height() - 56.0).max(180.0);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), alto), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(6), tema::SUPERFICIE);
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(6),
        egui::Stroke::new(1.0_f32, tema::BORDE),
        egui::StrokeKind::Inside,
    );
    if !state.pixels.is_empty() {
        let frame = crate::video::Frame {
            pixels: Arc::new(state.pixels.clone()),
            width: crate::preview::CANVAS.0,
            height: crate::preview::CANVAS.1,
            capas: Arc::clone(&state.layers),
        };
        crate::ui::video::pintar(ui, rect, frame, formato, gpu);
    }

    mensajes.extend(transporte::mostrar(ui, state));

    ui.separator();
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        ui.label(
            egui::RichText::new("RUSTYDIALOGUE")
                .font(tema::mono_public())
                .color(tema::DIM)
                .size(9.0),
        );
        let sucio = sucio(state);
        ui.label(egui::RichText::new("●").size(9.0).color(if sucio {
            tema::AMBAR
        } else {
            tema::DIM
        }));
        ui.label(
            egui::RichText::new(if sucio {
                "Cambios sin guardar"
            } else {
                "Guardado"
            })
            .font(tema::mono_public())
            .size(10.0)
            .color(if sucio { tema::AMBAR } else { tema::SECUNDARIO }),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(texto) = &state.mensaje {
                ui.label(
                    egui::RichText::new(texto)
                        .size(11.0)
                        .color(tema::SECUNDARIO),
                );
            }
        });
    });
    mensajes
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use egui_kittest::kittest::Queryable;

    use crate::editor_app::{EditorApp, MensajeEditor, update};

    fn editor() -> EditorApp {
        let video = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../testdata/clip-15s-2audio.mp4"
        );
        EditorApp::desde_video(video).expect("video de prueba")
    }

    #[test]
    fn aplicar_texto_cambia_el_caption_y_registra_un_paso() {
        let mut editor = editor();
        editor.editor.agregar(rod_core::Caption {
            id: "cap-prueba".into(),
            inicio: 1.0,
            fin: 2.0,
            texto: "texto original".into(),
            hablante_id: None,
        });
        let id = editor.editor.proyecto().captions[0].id.clone();
        editor.editor.seleccionar_uno(&id);
        editor.borrador = "texto nuevo".into();
        editor.limpio = editor.editor.proyecto().clone();

        let mensajes = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let recolector = std::rc::Rc::clone(&mensajes);
        let mut harness = egui_kittest::Harness::new_state(
            move |ctx: &egui::Context, editor: &mut EditorApp| {
                recolector.borrow_mut().extend(super::show(
                    ctx,
                    editor,
                    wgpu::TextureFormat::Bgra8Unorm,
                    Arc::new(Mutex::new(None)),
                ));
            },
            editor,
        );
        harness.run();
        harness.get_by_label("Aplicar texto").click();
        harness.run();
        let mensajes = mensajes.borrow().clone();
        assert!(
            mensajes
                .iter()
                .any(|mensaje| matches!(mensaje, MensajeEditor::AplicarTexto))
        );

        for mensaje in mensajes {
            update(harness.state_mut(), mensaje);
        }
        let editor = harness.state();
        assert_eq!(editor.editor.texto_de("cap-prueba"), Some("texto nuevo"));
        assert!(editor.editor.puede_deshacer());
    }
}
