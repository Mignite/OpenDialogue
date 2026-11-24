use std::path::PathBuf;

use egui_kittest::kittest::Queryable;
use rod_app::inicio::EstadoInicio;
use rod_app::preferencias::Preferencias;
use rod_app::recientes::Reciente;
use rod_app::textos::EN;

type Estado = (EstadoInicio, Preferencias);

fn reciente(nombre: &str, lineas: usize, hablantes: usize) -> Reciente {
    Reciente {
        ruta: PathBuf::from(format!("C:/proyectos/{nombre}")),
        lineas,
        hablantes,
        fecha: Default::default(),
        duracion: 19.5,
        tira: Vec::new(),
    }
}

fn harness(estado: Estado) -> egui_kittest::Harness<'static, Estado> {
    egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, (estado, prefs)| {
                rod_app::ui::inicio::mostrar(ui, estado, &EN, prefs);
            },
            estado,
        )
}

#[test]
fn tres_recientes_dan_tres_tarjetas() {
    let estado = (
        EstadoInicio::nuevo(
            vec![
                reciente("presa.cdp", 19, 3),
                reciente("RobloxTerror.cdp", 543, 7),
                reciente("proyecto.cdp", 543, 7),
            ],
            None,
        ),
        Preferencias::default(),
    );
    let mut harness = harness(estado);
    harness.get_by_label("RobloxTerror.cdp");
    assert_eq!(
        harness
            .query_all_by_label_contains("543 lines · 7 speakers")
            .count(),
        2
    );
}

#[test]
fn sin_recientes_muestra_el_vacio() {
    let estado = (EstadoInicio::nuevo(vec![], None), Preferencias::default());
    let mut harness = harness(estado);
    harness.get_by_label(EN.texto_vacio_recientes);
}

#[test]
fn el_riel_abre_ajustes() {
    let estado = (
        EstadoInicio::nuevo(vec![], None),
        Preferencias::default(),
        false,
    );
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, (estado, prefs, aplicados)| {
                let (_, abrir) = rod_app::ui::inicio::mostrar(ui, estado, &EN, prefs);
                if abrir {
                    *aplicados = true;
                }
            },
            estado,
        );
    h.get_by_label("Preferences").click();
    h.run();
    assert!(h.state().2);
}
