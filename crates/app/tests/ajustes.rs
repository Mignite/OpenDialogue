use egui::{Key, Modifiers};
use egui_kittest::kittest::Queryable;
use rod_app::comandos::Comando;
use rod_app::preferencias::Preferencias;
use rod_app::textos::Idioma;
use rod_app::ui::ajustes::{EstadoAjustes, Pestana};

type Estado = (Preferencias, EstadoAjustes);

fn harness(estado: Estado) -> egui_kittest::Harness<'static, Estado> {
    egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_state(
            |ctx, (prefs, estado)| {
                rod_app::ui::ajustes::mostrar(ctx, prefs, estado);
            },
            estado,
        )
}

#[test]
fn cambiar_a_espanol_se_ve_al_instante() {
    let estado = (
        Preferencias::default(),
        EstadoAjustes {
            abierto: true,
            ..Default::default()
        },
    );
    let mut harness = harness(estado);
    harness.get_by_label("Español").click();
    harness.run();
    assert_eq!(harness.state().0.idioma, Idioma::Es);
    harness.get_by_label("Atajos");
}

#[test]
fn reasignar_una_tecla_tomada_pide_intercambio() {
    let estado = (
        Preferencias::default(),
        EstadoAjustes {
            abierto: true,
            pestana: Pestana::Atajos,
            busqueda: "Undo".into(),
            ..Default::default()
        },
    );
    let mut harness = harness(estado);
    harness.get_by_label("Change Undo").click();
    harness.run();
    harness.key_press_modifiers(
        Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
        Key::S,
    );
    harness.run();
    harness.get_by_label("Swap");
    harness.get_by_label("Swap").click();
    harness.run();
    let prefs = &harness.state().0;
    assert_eq!(
        prefs.atajo(Comando::Deshacer).unwrap().to_string(),
        "Ctrl+S"
    );
    assert_eq!(prefs.atajo(Comando::Guardar).unwrap().to_string(), "Ctrl+Z");
}
