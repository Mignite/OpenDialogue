use std::time::Duration;

use rod_app::ui::repintado::politica;

#[test]
fn en_reposo_limpio_no_pide_frames() {
    assert_eq!(politica(false, false, false), None);
}

#[test]
fn con_cambios_sin_guardar_pide_un_segundo() {
    assert_eq!(politica(false, true, false), Some(Duration::from_secs(1)));
}

#[test]
fn buscando_pide_rapido_y_reproduciendo_cada_frame() {
    assert_eq!(
        politica(false, false, true),
        Some(Duration::from_millis(100))
    );
    assert_eq!(politica(true, false, false), Some(Duration::ZERO));
}
