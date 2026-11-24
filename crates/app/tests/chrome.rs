use egui_kittest::kittest::Queryable;
use rod_app::comandos::MapaAtajos;
use rod_app::textos::ES;

#[test]
fn el_menu_separa_etiqueta_y_atajo() {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui(|ui| {
            rod_app::ui::chrome::mostrar_menu_para_test(ui, &ES, &MapaAtajos::por_defecto());
        });
    harness.get_by_label("Archivo").click();
    harness.run();
    // La etiqueta es solo el comando; el atajo es un nodo aparte.
    harness.get_by_label("Guardar");
    assert!(harness.query_by_label("Guardar    [Ctrl+S]").is_none());
    harness.get_by_label("Ctrl+S");
}
