use egui::Ui;

use crate::editor_app::{EditorApp, MensajeEditor};
use crate::presets_edicion::{CampoPreset, valor_campo};

pub fn mostrar(ui: &mut Ui, state: &mut EditorApp) -> Option<MensajeEditor> {
    let mut mensaje = None;
    ui.label(egui::RichText::new("Estilos").strong());

    let presets = state.presets.clone();
    for preset in &presets {
        ui.collapsing(preset.nombre.clone(), |ui| {
            for campo in [
                CampoPreset::Nombre,
                CampoPreset::Fontname,
                CampoPreset::Fontsize,
            ] {
                let valor = valor_campo(preset, campo, &state.borradores_preset);
                let clave = (preset.id.clone(), campo);
                let borrador = state.borradores_preset.entry(clave).or_insert(valor);
                let respuesta = ui.text_edit_singleline(borrador);
                if respuesta.lost_focus() {
                    mensaje = Some(MensajeEditor::EditarPreset(
                        preset.id.clone(),
                        campo,
                        borrador.clone(),
                    ));
                }
            }
            if ui.button("Asignar a la selección").clicked() {
                mensaje = Some(MensajeEditor::AsignarPreset(
                    preset.id.clone(),
                    Some(preset.id.clone()),
                ));
            }
        });
    }
    if ui.button("Nuevo preset").clicked() {
        mensaje = Some(MensajeEditor::NuevoPreset);
    }
    mensaje
}
