//! Vista del editor en egui. Compone el preview, la barra de transporte, los
//! paneles laterales y la barra de estado. El timeline se enchufa en el task 7.

use egui::Ui;

use crate::editor_app::{EditorApp, MensajeEditor, Panel, sucio};
use crate::ui::paneles::{estilos, hablantes, subtitulos};
use crate::ui::{tema, transporte};

pub fn mostrar(ui: &mut Ui, state: &mut EditorApp) -> Option<MensajeEditor> {
    let mut mensaje = None;

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
            pixels: std::sync::Arc::new(state.pixels.clone()),
            width: crate::preview::CANVAS.0,
            height: crate::preview::CANVAS.1,
            capas: std::sync::Arc::clone(&state.layers),
        };
        crate::ui::video::pintar(ui, rect, frame);
    }

    if let Some(mensaje_transporte) = transporte::mostrar(ui, state) {
        mensaje = Some(mensaje_transporte);
    }

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
                        mensaje = Some(MensajeEditor::Panel(panel));
                    }
                }
            });
            match state.panel {
                Panel::Subtitulos => subtitulos::mostrar(ui, state),
                Panel::Hablantes => hablantes::mostrar(ui, state),
                Panel::Estilos => estilos::mostrar(ui, state),
            }
        });
        if let Some(mensaje_panel) = panel.inner {
            mensaje = Some(mensaje_panel);
        }
    });

    ui.separator();
    if let Some(mensaje_timeline) = crate::ui::timeline::mostrar(ui, state) {
        mensaje = Some(mensaje_timeline);
    }

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
    mensaje
}

#[cfg(test)]
mod tests {
    use egui_kittest::kittest::Queryable;

    use crate::editor_app::{EditorApp, sucio};

    fn editor() -> EditorApp {
        let video = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../testdata/clip-15s-2audio.mp4"
        );
        EditorApp::desde_video(video).expect("video de prueba")
    }

    #[test]
    fn teclear_texto_y_confirmar_es_un_solo_paso_de_historial() {
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

        let mut harness = egui_kittest::Harness::new_ui_state(
            |ui, editor: &mut EditorApp| {
                super::mostrar(ui, editor);
            },
            editor,
        );
        harness.get_by_label("Aplicar texto").click();
        harness.run();

        assert!(sucio(harness.state()));
    }
}
