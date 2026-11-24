use egui::Ui;

use crate::editor_app::{CampoTiempo, EditorApp, MensajeEditor, etiqueta_caption};
use crate::ui::tema;

pub fn mostrar(ui: &mut Ui, state: &mut EditorApp) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();
    let total = state.editor.proyecto().captions.len();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Subtítulos").strong().size(13.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{total}"))
                    .font(tema::mono_public())
                    .size(10.0)
                    .color(tema::DIM),
            );
        });
    });
    ui.add_space(6.0);

    let seleccion = state.editor.seleccion.clone();
    for (indice, caption) in state.editor.proyecto().captions.iter().enumerate() {
        let seleccionado = seleccion.contains(&caption.id);
        let etiqueta = etiqueta_caption(indice, caption.inicio, caption.fin, &caption.texto);
        let fila = ui.add(
            egui::Button::new(egui::RichText::new(etiqueta).size(11.0))
                .frame(false)
                .truncate()
                .min_size(egui::vec2(ui.available_width(), 26.0))
                .fill(if seleccionado {
                    tema::ACENTO.gamma_multiply(0.22)
                } else {
                    egui::Color32::TRANSPARENT
                })
                .corner_radius(egui::CornerRadius::same(3)),
        );
        if seleccionado {
            ui.painter().rect_stroke(
                fila.rect,
                egui::CornerRadius::same(3),
                egui::Stroke::new(1.0_f32, tema::ACENTO),
                egui::StrokeKind::Inside,
            );
        }
        if fila.clicked() {
            mensajes.push(MensajeEditor::Seleccionar(caption.id.clone()));
        }
    }

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(8.0);
    tema::etiqueta(ui, "Texto");
    ui.add_space(3.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let respuesta = ui.add(
            egui::TextEdit::singleline(&mut state.borrador)
                .desired_width((ui.available_width() - 96.0).max(40.0)),
        );
        if ui.button("Aplicar texto").clicked()
            || (respuesta.changed() && ui.input(|entrada| entrada.key_pressed(egui::Key::Enter)))
        {
            mensajes.push(MensajeEditor::AplicarTexto);
        }
    });

    ui.add_space(10.0);
    ui.columns(2, |columnas| {
        for (columna, (nombre, borrador, campo)) in columnas.iter_mut().zip([
            ("Entrada", &mut state.borrador_inicio, CampoTiempo::Entrada),
            ("Salida", &mut state.borrador_fin, CampoTiempo::Salida),
        ]) {
            tema::etiqueta(columna, nombre);
            let respuesta = columna.text_edit_singleline(borrador);
            if respuesta.lost_focus()
                || columna.input(|entrada| entrada.key_pressed(egui::Key::Enter))
            {
                let valor = borrador.clone();
                mensajes.push(MensajeEditor::EditarTiempo(
                    seleccion.first().cloned().unwrap_or_default(),
                    campo,
                    valor,
                ));
            }
        }
    });

    ui.add_space(12.0);
    if ui.button("Borrar selección").clicked() {
        mensajes.push(MensajeEditor::BorrarSeleccion);
    }
    mensajes
}
