//! Vista del editor en egui. Compone el preview, la barra de transporte, los
//! paneles laterales y la barra de estado. Devuelve **todos** los mensajes del
//! frame: sobrescribirlos habría dejado los botones inertes.

use std::sync::{Arc, Mutex};

use egui::Ui;

use crate::editor_app::{EditorApp, MensajeEditor, Panel, sucio};
use crate::ui::paneles::{estilos, hablantes, subtitulos};
use crate::ui::{tema, transporte};
use crate::video::Gpu;

pub fn mostrar(
    ui: &mut Ui,
    state: &mut EditorApp,
    formato: wgpu::TextureFormat,
    gpu: Arc<Mutex<Option<Gpu>>>,
) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();

    // Preview real: el mismo pipeline de video + libass que antes, ahora a
    // través de un paint callback de egui_wgpu.
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 260.0),
        egui::Sense::hover(),
    );
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(6), tema::SUPERFICIE);
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
    ui.horizontal_top(|ui| {
        let panel = ui.vertical(|ui| {
            ui.horizontal(|ui| {
                for (panel, etiqueta) in [
                    (Panel::Subtitulos, "Subtítulos"),
                    (Panel::Hablantes, "Hablantes"),
                    (Panel::Estilos, "Estilos"),
                ] {
                    if ui
                        .selectable_label(state.panel == panel, etiqueta)
                        .clicked()
                    {
                        mensajes.push(MensajeEditor::Panel(panel));
                    }
                }
            });
            match state.panel {
                Panel::Subtitulos => subtitulos::mostrar(ui, state),
                Panel::Hablantes => hablantes::mostrar(ui, state),
                Panel::Estilos => estilos::mostrar(ui, state),
            }
        });
        mensajes.extend(panel.inner);
    });

    ui.separator();
    mensajes.extend(crate::ui::timeline::mostrar(ui, state));

    ui.separator();
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("RUSTYDIALOGUE")
                .color(tema::DIM)
                .size(9.0),
        );
        ui.label(if sucio(state) {
            "Cambios sin guardar"
        } else {
            "Sin cambios"
        });
        if let Some(texto) = &state.mensaje {
            ui.label(texto);
        }
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
        let mut harness = egui_kittest::Harness::new_ui_state(
            move |ui: &mut egui::Ui, editor: &mut EditorApp| {
                recolector.borrow_mut().extend(super::mostrar(
                    ui,
                    editor,
                    wgpu::TextureFormat::Bgra8Unorm,
                    Arc::new(Mutex::new(None)),
                ));
            },
            editor,
        );
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
