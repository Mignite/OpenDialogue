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
fn proyectos_vacios_se_ocultan_hasta_activar_el_filtro() {
    let mut estado = EstadoInicio::nuevo(vec![reciente("test.cdp", 0, 0)], None);
    assert!(!estado.mostrar_vacios);
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, (estado, prefs)| {
                rod_app::ui::inicio::mostrar(ui, estado, &EN, prefs);
            },
            (estado.clone(), Preferencias::default()),
        );
    h.run();
    assert_eq!(h.query_all_by_label("test.cdp").count(), 0);

    estado.mostrar_vacios = true;
    let mut h2 = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, (estado, prefs)| {
                rod_app::ui::inicio::mostrar(ui, estado, &EN, prefs);
            },
            (estado, Preferencias::default()),
        );
    h2.run();
    assert_eq!(h2.query_all_by_label("test.cdp").count(), 1);
}

#[test]
fn recientes_se_deduplican_por_ruta() {
    let estado = EstadoInicio::nuevo(
        vec![reciente("presa.cdp", 19, 3), reciente("presa.cdp", 19, 3)],
        None,
    );
    assert_eq!(estado.recientes.len(), 1);
}

#[test]
fn clic_en_tarjeta_emite_abrir_reciente() {
    type EstadoClic = (
        EstadoInicio,
        Preferencias,
        Option<rod_app::inicio::MensajeInicio>,
    );
    let estado: EstadoClic = (
        EstadoInicio::nuevo(vec![reciente("RobloxTerror.cdp", 543, 7)], None),
        Preferencias::default(),
        None,
    );
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, (estado, prefs, mensaje): &mut EstadoClic| {
                let (intencion, _) = rod_app::ui::inicio::mostrar(ui, estado, &EN, prefs);
                if intencion.is_some() {
                    *mensaje = intencion;
                }
            },
            estado,
        );
    h.get_by_label("RobloxTerror.cdp").click();
    h.run();
    assert!(h.state().2.as_ref().is_some_and(|mensaje| matches!(
        mensaje,
        rod_app::inicio::MensajeInicio::AbrirReciente(_)
    )));
}
