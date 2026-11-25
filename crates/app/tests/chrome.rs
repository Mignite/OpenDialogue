use egui_kittest::kittest::Queryable;
use rod_app::comandos::MapaAtajos;
use rod_app::textos::ES;

#[test]
fn el_menu_es_compacto_y_solo_muestra_etiquetas() {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui(|ui| {
            rod_app::ui::chrome::mostrar_menu_para_test(ui, &ES, &MapaAtajos::por_defecto());
        });
    harness.get_by_label("Archivo").click();
    harness.run();
    // Menú corto: sin columna de atajos (ésos están en Ajustes).
    harness.get_by_label("Guardar");
    assert_eq!(harness.query_all_by_label("Ctrl+S").count(), 0);
    harness.get_by_label("Cargar SRT");
}
