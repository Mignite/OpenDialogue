use rod_app::timeline::{Vista, escala_anclada, iman, paso_de_regla, tiempo_bajo_cursor};
use rod_app::ui::tema;

#[test]
fn clip_sin_hablante_seleccionado_usa_acento() {
    assert_eq!(
        rod_app::ui::timeline::color_borde_clip(None, true),
        tema::ACENTO
    );
    assert_eq!(
        rod_app::ui::timeline::color_borde_clip(None, false),
        tema::COLOR_SIN_HABLANTE
    );
}

#[test]
fn la_regla_cambia_de_paso_segun_el_zoom() {
    assert_eq!(paso_de_regla(10.0), 10.0);
    assert_eq!(paso_de_regla(100.0), 1.0);
    assert_eq!(paso_de_regla(1000.0), 0.5);
}

#[test]
fn el_iman_pega_al_borde_mas_cercano_dentro_de_la_tolerancia() {
    assert_eq!(iman(4.98, &[0.0, 5.0, 9.0], 0.06), Some(5.0));
    assert_eq!(iman(4.80, &[0.0, 5.0, 9.0], 0.06), None);
}

#[test]
fn el_zoom_conserva_el_tiempo_bajo_el_cursor() {
    let vista = Vista {
        inicio: 10.0,
        escala: 0.1,
    };
    let antes = tiempo_bajo_cursor(vista, 100.0);
    let despues = escala_anclada(vista, 2.0, 100.0, 200.0, 800.0);
    assert!((tiempo_bajo_cursor(despues, 100.0) - antes).abs() < 1e-6);
}
