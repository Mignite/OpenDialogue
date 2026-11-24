use egui::Ui;

use crate::editor::MAX_HABLANTES;
use crate::editor_app::{CampoHablante, EditorApp, MensajeEditor};
use crate::ui::tema;

pub fn mostrar(ui: &mut Ui, state: &mut EditorApp) -> Option<MensajeEditor> {
    let mut mensaje = None;
    ui.label(egui::RichText::new("Hablantes").strong());

    let hablantes = state.editor.proyecto().hablantes.clone();
    for hablante in &hablantes {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("●").color(tema::color_de_hex(&hablante.color)));
            let respuesta = ui.text_edit_singleline(
                &mut state
                    .editor
                    .proyecto_mut()
                    .hablantes
                    .iter_mut()
                    .find(|candidato| candidato.id == hablante.id)
                    .expect("el hablante sigue ahí")
                    .nombre,
            );
            if respuesta.lost_focus() {
                mensaje = Some(MensajeEditor::EditarHablante(
                    hablante.id.clone(),
                    CampoHablante::Nombre,
                    respuesta_lost(&state.editor.proyecto().hablantes, &hablante.id),
                ));
            }
            if ui.button("Asignar").clicked() {
                mensaje = Some(MensajeEditor::AsignarHablante(Some(hablante.id.clone())));
            }
            if ui.button("✕").clicked() {
                mensaje = Some(MensajeEditor::BorrarHablante(hablante.id.clone()));
            }
        });
    }
    if state.editor.proyecto().hablantes.len() < MAX_HABLANTES
        && ui.button("Añadir hablante").clicked()
    {
        mensaje = Some(MensajeEditor::AgregarHablante);
    }
    if ui.button("Quitar hablante de la selección").clicked() {
        mensaje = Some(MensajeEditor::AsignarHablante(None));
    }
    mensaje
}

fn respuesta_lost(hablantes: &[rod_core::Hablante], id: &str) -> String {
    hablantes
        .iter()
        .find(|hablante| hablante.id == id)
        .map(|hablante| hablante.nombre.clone())
        .unwrap_or_default()
}
