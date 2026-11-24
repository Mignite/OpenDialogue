//! La pantalla de arranque. Solo dibuja y devuelve la intención del usuario;
//! `rod_app::inicio::update` sigue siendo quien decide qué hacer con ella.

use egui::Ui;

use crate::inicio::{
    EstadoInicio, MensajeInicio, cantidad_visible, resumen_reciente, texto_vacio_recientes,
};
use crate::ui::tema;

pub fn mostrar(ui: &mut Ui, estado: &mut EstadoInicio) -> Option<MensajeInicio> {
    let mut intencion = None;

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("R").color(tema::ACENTO).size(18.0));
        ui.label(egui::RichText::new("RustyDialogue").size(16.0).strong());
    });
    ui.separator();
    ui.label("Arrastra un video aquí, o abre un proyecto");
    ui.horizontal(|ui| {
        if ui.button("Abrir video").clicked() {
            intencion = Some(MensajeInicio::AbrirVideo);
        }
        if ui.button("Abrir proyecto").clicked() {
            intencion = Some(MensajeInicio::AbrirProyecto);
        }
    });

    if let Some(aviso) = &estado.aviso {
        ui.group(|ui| {
            ui.label(&aviso.texto);
            ui.horizontal(|ui| {
                if ui.button("Recuperar").clicked() {
                    intencion = Some(MensajeInicio::Recuperar);
                }
                if ui.button("Descartar").clicked() {
                    intencion = Some(MensajeInicio::DescartarRecovery);
                }
            });
        });
    }
    if let Some(mensaje) = &estado.mensaje {
        ui.colored_label(tema::DIM, mensaje);
    }

    ui.separator();
    ui.label(egui::RichText::new("Recientes").strong());
    if estado.recientes.is_empty() {
        ui.label(texto_vacio_recientes());
    } else {
        let visibles = cantidad_visible(estado.recientes.len(), estado.expandido);
        for reciente in estado.recientes.iter().take(visibles) {
            let etiqueta = format!(
                "{} — {}",
                reciente.ruta.display(),
                resumen_reciente(reciente)
            );
            if ui.button(etiqueta).clicked() {
                intencion = Some(MensajeInicio::AbrirReciente(reciente.ruta.clone()));
            }
        }
        if estado.recientes.len() > 3 && ui.button("Ver todos").clicked() {
            intencion = Some(MensajeInicio::VerTodos);
        }
    }
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
                fecha: "24 nov".into(),
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
        harness.run();
        assert!(harness.query_by_label("Abrir video").is_some());
        assert!(
            harness
                .query_by_label_contains("C:/proyectos/1.cdp")
                .is_some()
        );
        assert!(
            harness
                .query_by_label_contains("C:/proyectos/4.cdp")
                .is_none()
        );
    }
}
