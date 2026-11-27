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
    let harness = harness(estado);
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
    let harness = harness(estado);
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

type EstadoClic = (
    EstadoInicio,
    Preferencias,
    Option<rod_app::inicio::MensajeInicio>,
);

/// Una sola tarjeta reciente, para probar qué abre y qué no.
fn una_tarjeta() -> egui_kittest::Harness<'static, EstadoClic> {
    let estado: EstadoClic = (
        EstadoInicio::nuevo(vec![reciente("RobloxTerror.cdp", 543, 7)], None),
        Preferencias::default(),
        None,
    );
    egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, (estado, prefs, mensaje): &mut EstadoClic| {
                let (intencion, _) = rod_app::ui::inicio::mostrar(ui, estado, &EN, prefs);
                if intencion.is_some() {
                    *mensaje = intencion;
                }
            },
            estado,
        )
}

#[test]
fn clic_en_tarjeta_emite_abrir_reciente() {
    let mut h = una_tarjeta();
    h.get_by_label("RobloxTerror.cdp").click();
    h.run();
    assert!(h.state().2.as_ref().is_some_and(|mensaje| matches!(
        mensaje,
        rod_app::inicio::MensajeInicio::AbrirReciente(_)
    )));
}

#[test]
fn clic_en_los_metadatos_de_la_tarjeta_abre_el_proyecto() {
    let mut h = una_tarjeta();
    // Toda la burbuja abre el proyecto, no solo el margen del marco.
    h.get_by_label_contains("543 lines").click();
    h.run();
    assert!(
        h.state().2.as_ref().is_some_and(|mensaje| matches!(
            mensaje,
            rod_app::inicio::MensajeInicio::AbrirReciente(_)
        )),
        "el clic en los metadatos no abrió la tarjeta: {:?}",
        h.state().2
    );
}

#[test]
fn clic_en_la_zona_baja_de_la_tarjeta_abre_el_proyecto() {
    let mut h = una_tarjeta();
    let titulo = h.get_by_label("RobloxTerror.cdp").rect();
    // Bajo los metadatos, donde antes quedaba la celda de altura fija y el
    // Frame de la tarjeta seguía pintando. Un clic ahí debe abrir igual.
    let pos = egui::pos2(titulo.center().x, titulo.bottom() + 40.0);
    for pressed in [true, false] {
        h.input_mut().events.push(egui::Event::PointerMoved(pos));
        h.input_mut().events.push(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        });
        h.step();
    }
    h.run();
    assert!(
        matches!(
            h.state().2,
            Some(rod_app::inicio::MensajeInicio::AbrirReciente(_))
        ),
        "el clic en la parte baja no abrió la tarjeta: {:?}",
        h.state().2
    );
}

#[test]
fn con_muchos_recientes_expandidos_se_puede_desplazar() {
    let recientes: Vec<_> = (0..10)
        .map(|i| reciente(&format!("proyecto-{i}.cdp"), 1, 1))
        .collect();
    let mut estado = EstadoInicio::nuevo(recientes, None);
    estado.expandido = true;
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(800.0, 300.0))
        .build_ui_state(
            |ui, estado: &mut EstadoInicio| {
                let mut prefs = Preferencias::default();
                let _ = rod_app::ui::inicio::mostrar(ui, estado, &EN, &mut prefs);
            },
            estado,
        );
    h.run();
    let antes = h.get_by_label("proyecto-9.cdp").rect().min.y;
    assert!(
        antes > 300.0,
        "el último proyecto debería quedar fuera: {antes}"
    );

    h.input_mut()
        .events
        .push(egui::Event::PointerMoved(egui::pos2(400.0, 150.0)));
    h.input_mut().events.push(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, -600.0),
        modifiers: egui::Modifiers::default(),
    });
    h.step();
    h.run();
    let despues = h.get_by_label("proyecto-9.cdp").rect().min.y;
    assert!(
        despues < antes - 100.0,
        "la lista no se desplazó: antes={antes} después={despues}"
    );
}

#[test]
fn el_menu_de_la_tarjeta_no_abre_el_proyecto() {
    let mut h = una_tarjeta();
    // El menú de puntos solo aparece cuando la tarjeta está bajo el puntero.
    h.get_by_label("RobloxTerror.cdp").hover();
    h.run();
    h.get_by_label(rod_app::ui::kit::glifo(rod_app::ui::kit::Icono::Puntos))
        .click();
    h.run();
    assert!(
        h.state().2.is_none(),
        "abrir el menú abrió el proyecto: {:?}",
        h.state().2
    );
    // El menú ofrece las tres acciones sin abrir la tarjeta.
    h.get_by_label(EN.mostrar_en_carpeta);
    h.get_by_label(EN.quitar_de_recientes);

    h.get_by_label(EN.abrir).click();
    h.run();
    assert!(matches!(
        h.state().2,
        Some(rod_app::inicio::MensajeInicio::AbrirReciente(_))
    ));
}
