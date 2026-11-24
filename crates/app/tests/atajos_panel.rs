use egui::Key;
use egui_kittest::kittest::Queryable;
use rod_app::comandos::{Atajo, Comando};
use rod_app::preferencias::Preferencias;
use rod_app::ui::atajos::EstadoAtajos;

#[test]
fn el_panel_lista_el_atajo_vigente_y_escape_lo_cierra() {
    let mut prefs = Preferencias::default();
    prefs
        .atajos
        .asignar(
            Comando::Dividir,
            Atajo {
                tecla: Key::D,
                ctrl: true,
                shift: false,
                alt: false,
            },
        )
        .unwrap();
    let mut estado = EstadoAtajos {
        abierto: true,
        ..Default::default()
    };
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_state(
            |ctx, (prefs, estado): &mut (Preferencias, EstadoAtajos)| {
                rod_app::ui::atajos::mostrar(ctx, prefs.idioma.textos(), &prefs.atajos, estado);
            },
            (prefs, estado),
        );
    h.get_by_label("Split at playhead");
    h.get_by_label("Ctrl+D");
    h.key_press(Key::Escape);
    h.run();
    assert!(!h.state().1.abierto);
}
