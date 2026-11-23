//! Contrato público que el arranque necesita de la librería `rod-app`.

use rod_app::editor_app::{EditorApp, MensajeEditor};
use rod_app::recovery_io::{AvisoRecovery, pendiente};

#[test]
fn editor_y_recovery_son_publicos_en_la_libreria() {
    fn assert_public<T>() {}
    assert_public::<EditorApp>();
    assert_public::<MensajeEditor>();
    assert_public::<AvisoRecovery>();

    let _: fn() -> EditorApp = EditorApp::nuevo;
    let _: fn(&std::path::Path, f64) -> Option<AvisoRecovery> = pendiente;
}
