use egui::Ui;

use crate::editor::MAX_HABLANTES;
use crate::editor_app::{CampoHablante, EditorApp, MensajeEditor};
use crate::ui::tema;

pub fn mostrar(ui: &mut Ui, state: &mut EditorApp) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();
    ui.label(egui::RichText::new("Hablantes").strong());

    let hablantes = state.editor.proyecto().hablantes.clone();
    for hablante in &hablantes {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("●").color(tema::color_de_hex(&hablante.color)));
            let borrador = state
                .borradores_hablante
                .entry(hablante.id.clone())
                .or_insert_with(|| hablante.nombre.clone());
            let respuesta = ui.text_edit_singleline(borrador);
            if respuesta.lost_focus() {
                mensajes.push(MensajeEditor::EditarHablante(
                    hablante.id.clone(),
                    CampoHablante::Nombre,
                    borrador.clone(),
                ));
            }
            if ui.button("Asignar").clicked() {
                mensajes.push(MensajeEditor::AsignarHablante(Some(hablante.id.clone())));
            }
            if ui.button("✕").clicked() {
                mensajes.push(MensajeEditor::BorrarHablante(hablante.id.clone()));
            }
        });
    }
    if state.editor.proyecto().hablantes.len() < MAX_HABLANTES
        && ui.button("Añadir hablante").clicked()
    {
        mensajes.push(MensajeEditor::AgregarHablante);
    }
    if ui.button("Quitar hablante de la selección").clicked() {
        mensajes.push(MensajeEditor::AsignarHablante(None));
    }
    mensajes
}
