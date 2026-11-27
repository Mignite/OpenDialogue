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

/// Abrir Ajustes con el clic del menú Editar: ese clic está fuera de la
/// ventana y no debe cerrarla en el mismo frame.
#[test]
fn abrir_ajustes_desde_el_menu_no_cierra_la_capa() {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_state(
            |ctx, (prefs, ajustes): &mut Estado| {
                let elegido = rod_app::ui::chrome::mostrar(
                    ctx,
                    &rod_app::textos::ES,
                    &rod_app::comandos::MapaAtajos::por_defecto(),
                    rod_app::ui::chrome::EstadoCabecera {
                        en_inicio: true,
                        proyecto: None,
                        sucio: false,
                    },
                );
                if elegido == Some(Comando::Ajustes) {
                    ajustes.abrir();
                }
                rod_app::ui::ajustes::mostrar(ctx, prefs, ajustes);
            },
            (Preferencias::default(), EstadoAjustes::default()),
        );
    rod_app::ui::tema::aplicar(&harness.ctx);
    harness
        .get_by_label(rod_app::textos::ES.seccion_edicion)
        .click();
    harness.run();
    harness
        .get_by_label(rod_app::textos::ES.comando(Comando::Ajustes))
        .click();
    harness.run();
    assert!(
        harness.state().1.abierto,
        "el clic del menú cerró Ajustes en el mismo frame"
    );
}

#[test]
fn cambiar_a_espanol_se_ve_al_instante() {
    let mut abierto = EstadoAjustes::default();
    abierto.abrir();
    let estado = (Preferencias::default(), abierto);
    let mut harness = harness(estado);
    harness.get_by_label("Español").click();
    harness.run();
    assert_eq!(harness.state().0.idioma, Idioma::Es);
    harness.get_by_label("Atajos");
}

#[test]
fn reasignar_una_tecla_tomada_pide_intercambio() {
    let mut abierto = EstadoAjustes::default();
    abierto.abrir();
    abierto.pestana = Pestana::Atajos;
    abierto.busqueda = "Undo".into();
    let estado = (Preferencias::default(), abierto);
    let mut harness = harness(estado);
    harness.get_by_label("Change").click();
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
