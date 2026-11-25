use egui_kittest::kittest::Queryable;
use rod_app::comandos::MapaAtajos;
use rod_app::editor_app::EditorApp;
use rod_app::textos::ES;

#[test]
fn dropdown_de_velocidad_tiene_ancho_definido() {
    assert_eq!(
        rod_app::ui::transporte::ancho_dropdown(),
        rod_app::ui::tema::ANCHO_DROPDOWN_VELOCIDAD
    );
    assert_eq!(rod_app::ui::transporte::ancho_dropdown(), 56.0);
}

#[test]
fn el_timecode_sale_de_format_time_y_hay_controles() {
    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let mut app = EditorApp::desde_video(video).expect("abre el clip de prueba");
    app.set_position(19.48);
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, editor: &mut EditorApp| {
                rod_app::ui::transporte::mostrar(ui, editor, &ES, &MapaAtajos::por_defecto());
            },
            app,
        );
    h.run();
    h.get_by_label(&rod_core::format_time(19.48));
    let play = rod_app::ui::kit::glifo(rod_app::ui::kit::Icono::Play);
    let pausa = rod_app::ui::kit::glifo(rod_app::ui::kit::Icono::Pausa);
    assert!(h.query_all_by_label(play).count() + h.query_all_by_label(pausa).count() >= 1);
}
