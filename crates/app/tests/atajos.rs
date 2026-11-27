use egui::{Key, Modifiers};
use rod_app::comandos::{Atajo, Comando, MapaAtajos};
use rod_app::preferencias::Preferencias;
use rod_app::textos::Idioma;

fn mods(ctrl: bool, shift: bool, alt: bool) -> Modifiers {
    Modifiers {
        ctrl,
        command: ctrl,
        shift,
        alt,
        ..Default::default()
    }
}

#[test]
fn el_mapa_por_defecto_reproduce_los_atajos_de_hoy() {
    let mapa = MapaAtajos::por_defecto();
    for (tecla, ctrl, shift, alt, esperado) in [
        (Key::S, true, false, false, Comando::Guardar),
        (Key::S, true, true, false, Comando::GuardarComo),
        (Key::Z, true, false, false, Comando::Deshacer),
        (Key::Space, false, false, false, Comando::ReproducirPausa),
        (Key::ArrowLeft, false, false, true, Comando::CaptionAnterior),
        (Key::ArrowLeft, false, true, false, Comando::RetrocederFino),
        (Key::S, false, false, false, Comando::Dividir),
        (Key::Num7, false, false, false, Comando::Hablante(7)),
    ] {
        assert_eq!(
            mapa.comando_de(tecla, mods(ctrl, shift, alt), false),
            Some(esperado),
            "{tecla:?}"
        );
    }
}

#[test]
fn atajos_de_edicion_registrados() {
    let mapa = MapaAtajos::por_defecto();
    for (tecla, esperado) in [
        (Key::OpenBracket, Comando::Entrada),
        (Key::CloseBracket, Comando::Salida),
        (Key::S, Comando::Dividir),
        (Key::M, Comando::Unir),
    ] {
        assert_eq!(
            mapa.comando_de(tecla, Modifiers::default(), false),
            Some(esperado),
            "{tecla:?}"
        );
    }
    assert_eq!(
        mapa.comando_de(Key::Delete, Modifiers::default(), false),
        Some(Comando::Eliminar)
    );

    let mut vistos = std::collections::HashSet::new();
    for (_, atajo) in mapa.filas() {
        let clave = (atajo.tecla, atajo.ctrl, atajo.shift, atajo.alt);
        assert!(vistos.insert(clave), "atajo duplicado: {clave:?}");
    }
}

#[test]
fn ctrl_comma_abre_ajustes() {
    let mapa = MapaAtajos::por_defecto();
    assert_eq!(
        mapa.comando_de(Key::Comma, mods(true, false, false), false),
        Some(Comando::Ajustes)
    );
}

#[test]
fn con_foco_en_un_campo_las_teclas_desnudas_no_disparan() {
    let mapa = MapaAtajos::por_defecto();
    assert_eq!(
        mapa.comando_de(Key::Space, Modifiers::default(), true),
        None
    );
    assert_eq!(mapa.comando_de(Key::C, Modifiers::default(), true), None);
    assert_eq!(mapa.comando_de(Key::Num1, Modifiers::default(), true), None);
    assert_eq!(
        mapa.comando_de(Key::S, mods(true, false, false), true),
        Some(Comando::Guardar)
    );
}

#[test]
fn no_hay_atajos_repetidos_y_asignar_uno_tomado_falla() {
    let mut mapa = MapaAtajos::por_defecto();
    assert_eq!(
        mapa.comando_de(Key::S, mods(true, false, false), false),
        Some(Comando::Guardar)
    );
    assert!(
        mapa.asignar(
            Comando::Deshacer,
            Atajo {
                tecla: Key::S,
                ctrl: true,
                shift: false,
                alt: false,
            }
        )
        .is_err()
    );
    mapa.intercambiar(Comando::Guardar, Comando::Deshacer);
    assert_eq!(mapa.atajo(Comando::Deshacer).unwrap().to_string(), "Ctrl+S");
}

#[test]
fn preferencias_rotas_dan_los_valores_de_fabrica() {
    let dir = std::env::temp_dir().join(format!("rod-prefs-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let ruta = dir.join("preferencias.json");
    std::fs::write(&ruta, "{ no es json").unwrap();
    let prefs = Preferencias::cargar_en(&ruta);
    assert_eq!(prefs.idioma, Idioma::En);
    assert_eq!(prefs.atajo(Comando::Guardar).unwrap().to_string(), "Ctrl+S");
}

#[test]
fn un_archivo_parcial_no_deja_los_demas_atajos_sin_asignar() {
    let dir = std::env::temp_dir().join(format!("rod-prefs-parcial-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let ruta = dir.join("preferencias.json");
    std::fs::write(
        &ruta,
        r#"{"atajos":[["Guardar",{"tecla":"S","ctrl":true,"shift":false,"alt":false}]]}"#,
    )
    .unwrap();
    let prefs = Preferencias::cargar_en(&ruta);
    assert_eq!(prefs.atajo(Comando::Guardar).unwrap().to_string(), "Ctrl+S");
    assert_eq!(
        prefs.atajo(Comando::ReproducirPausa).unwrap().to_string(),
        "Space"
    );
    assert_eq!(
        prefs.atajo(Comando::Deshacer).unwrap().to_string(),
        "Ctrl+Z"
    );
}

#[test]
fn preferencias_hacen_round_trip() {
    let dir = std::env::temp_dir().join(format!("rod-prefs-rt-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let ruta = dir.join("preferencias.json");
    let prefs = Preferencias {
        idioma: Idioma::Es,
        ancho_panel: 348.0,
        atajos: MapaAtajos::por_defecto(),
        ..Default::default()
    };
    prefs.guardar_en(&ruta).unwrap();
    let cargadas = Preferencias::cargar_en(&ruta);
    assert_eq!(cargadas.idioma, Idioma::Es);
    assert_eq!(cargadas.ancho_panel, 348.0);
    assert_eq!(
        cargadas.atajo(Comando::Guardar).unwrap().to_string(),
        "Ctrl+S"
    );
}
