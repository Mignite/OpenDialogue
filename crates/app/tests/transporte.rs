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
    let mut app = EditorApp::desde_proyecto_en(
        rod_core::Proyecto::nuevo(video),
        None,
        0.0,
        &std::env::temp_dir(),
    )
    .expect("abre el clip de prueba");
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

#[test]
fn el_transporte_tiene_selector_de_audio_y_volumen_sin_segundo_follow() {
    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let pista = rod_player::Player::open_sin_audio(video)
        .expect("abre el clip")
        .audio_tracks()
        .first()
        .expect("el clip tiene audio")
        .label
        .clone();
    let app = EditorApp::desde_proyecto_en(
        rod_core::Proyecto::nuevo(video),
        None,
        0.0,
        &std::env::temp_dir(),
    )
    .expect("abre el clip de prueba");
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, editor: &mut EditorApp| {
                rod_app::ui::transporte::mostrar(ui, editor, &ES, &MapaAtajos::por_defecto());
            },
            app,
        );
    h.run();

    // Velocidad + pista de audio: la segunda abre las pistas del clip.
    let combos: Vec<_> = h
        .query_all_by_role(egui::accesskit::Role::ComboBox)
        .collect();
    assert_eq!(combos.len(), 2, "velocidad y pista de audio");
    combos[1].click();
    h.run();
    h.get_by_label(&pista);
    // El volumen ya no ocupa la fila: vive en el desplegable del altavoz.
    assert_eq!(
        h.query_all_by_role(egui::accesskit::Role::Slider).count(),
        0,
        "el slider no debe solaparse con el campo de texto"
    );
    h.get_by_role_and_label(
        egui::accesskit::Role::Button,
        rod_app::ui::kit::glifo(rod_app::ui::kit::Icono::Volumen),
    )
    .click();
    h.run();
    assert!(
        h.query_all_by_role(egui::accesskit::Role::Slider).count() >= 1,
        "el popup del altavoz debe traer el slider vertical"
    );
    // El follow playhead vive en la timeline; aquí solo estaba duplicado.
    assert_eq!(
        h.query_all_by_label(rod_app::ui::kit::glifo(rod_app::ui::kit::Icono::Objetivo))
            .count(),
        0
    );
}
