use egui::Ui;

use crate::editor_app::{EditorApp, MensajeEditor, etiqueta_caption};

pub fn mostrar(ui: &mut Ui, state: &mut EditorApp) -> Vec<MensajeEditor> {
    let mut mensaje = Vec::new();
    ui.label(egui::RichText::new("Subtítulos").strong());

    let seleccion = state.editor.seleccion.clone();
    for (indice, caption) in state.editor.proyecto().captions.iter().enumerate() {
        let etiqueta = etiqueta_caption(indice, caption.inicio, caption.fin, &caption.texto);
        if ui
            .selectable_label(seleccion.contains(&caption.id), etiqueta)
            .clicked()
        {
            mensaje.push(MensajeEditor::Seleccionar(caption.id.clone()));
        }
    }

    ui.separator();
    let respuesta = ui.text_edit_singleline(&mut state.borrador);
    if ui.button("Aplicar texto").clicked()
        || (respuesta.changed() && ui.input(|entrada| entrada.key_pressed(egui::Key::Enter)))
    {
        mensaje.push(MensajeEditor::AplicarTexto);
    }
    ui.horizontal(|ui| {
        ui.label("Entrada");
        let respuesta = ui.text_edit_singleline(&mut state.borrador_inicio);
        if respuesta.lost_focus() || ui.input(|entrada| entrada.key_pressed(egui::Key::Enter)) {
            let valor = state.borrador_inicio.clone();
            mensaje.push(MensajeEditor::EditarTiempo(
                seleccion.first().cloned().unwrap_or_default(),
                crate::editor_app::CampoTiempo::Entrada,
                valor,
            ));
        }
    });
    ui.horizontal(|ui| {
        ui.label("Salida");
        let respuesta = ui.text_edit_singleline(&mut state.borrador_fin);
        if respuesta.lost_focus() || ui.input(|entrada| entrada.key_pressed(egui::Key::Enter)) {
            let valor = state.borrador_fin.clone();
            mensaje.push(MensajeEditor::EditarTiempo(
                seleccion.first().cloned().unwrap_or_default(),
                crate::editor_app::CampoTiempo::Salida,
                valor,
            ));
        }
    });
    if ui.button("Borrar selección").clicked() {
        mensaje.push(MensajeEditor::BorrarSeleccion);
    }
    mensaje
}
