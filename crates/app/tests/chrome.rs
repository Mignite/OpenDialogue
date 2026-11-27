use egui_kittest::kittest::Queryable;
use rod_app::comandos::MapaAtajos;
use rod_app::textos::ES;
use rod_app::ui::chrome::{self, EstadoCabecera};

fn cabecera() -> egui_kittest::Harness<'static, ()> {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_state(
            |ctx, ()| {
                let _ = chrome::mostrar(
                    ctx,
                    &ES,
                    &MapaAtajos::por_defecto(),
                    EstadoCabecera {
                        en_inicio: true,
                        proyecto: None,
                        sucio: false,
                    },
                );
            },
            (),
        );
    rod_app::ui::tema::aplicar(&harness.ctx);
    harness.run();
    harness
}

fn comandos_de_vista(harness: &egui_kittest::Harness<'_, ()>) -> Vec<egui::ViewportCommand> {
    harness
        .output()
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .map(|vista| vista.commands.clone())
        .unwrap_or_default()
}

#[test]
fn estado_guardado_lleva_punto() {
    assert_eq!(
        rod_app::ui::chrome::color_punto_estado(true),
        rod_app::ui::tema::AMBAR
    );
    assert_eq!(
        rod_app::ui::chrome::color_punto_estado(false),
        rod_app::ui::tema::TEXTO_2
    );
}

#[test]
fn el_menu_muestra_los_atajos_en_la_columna_derecha() {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui(|ui| {
            rod_app::ui::chrome::mostrar_menu_para_test(ui, &ES, &MapaAtajos::por_defecto());
        });
    harness.get_by_label("Archivo").click();
    harness.run();
    // Columna de atajos a la derecha, alineada con la etiqueta.
    harness.get_by_label("Guardar");
    assert_eq!(harness.query_all_by_label("Ctrl+S").count(), 1);
    harness.get_by_label("Cargar SRT");
}

#[test]
fn la_cabecera_solo_lleva_los_tres_controles_de_ventana() {
    let harness = cabecera();
    // Ajustes vive en Editar; el engranaje de la cabecera se retiró.
    assert_eq!(
        harness
            .query_all_by_label(rod_app::ui::kit::glifo(rod_app::ui::kit::Icono::Ajustes))
            .count(),
        0
    );
    for tooltip in [ES.minimizar, ES.maximizar, ES.cerrar_ventana] {
        harness.get_by_label(tooltip);
    }
}

#[test]
fn arrastrar_la_cabecera_emite_start_drag() {
    let mut h = cabecera();
    let centro = h.get_by_label(ES.arrastrar_ventana).rect().center();
    let destino = centro + egui::vec2(60.0, 40.0);
    let mut enviados = Vec::new();
    h.input_mut().events.push(egui::Event::PointerMoved(centro));
    h.step();
    h.input_mut().events.push(egui::Event::PointerButton {
        pos: centro,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    h.step();
    h.input_mut()
        .events
        .push(egui::Event::PointerMoved(destino));
    h.step();
    enviados.extend(comandos_de_vista(&h));
    h.input_mut().events.push(egui::Event::PointerButton {
        pos: destino,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::default(),
    });
    h.step();
    assert!(
        enviados.contains(&egui::ViewportCommand::StartDrag),
        "{enviados:?}"
    );
}

#[test]
fn los_controles_de_ventana_emiten_su_comando() {
    let mut harness = cabecera();
    harness.get_by_label(ES.minimizar).click();
    harness.step();
    assert!(
        comandos_de_vista(&harness).contains(&egui::ViewportCommand::Minimized(true)),
        "{:?}",
        comandos_de_vista(&harness)
    );
}
