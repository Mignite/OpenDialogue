//! Verificación del recovery (slice 7b) sin ventana y sin esperar 15 s.
//!
//! `cargo run -p rod-app --example recovery` hace el recorrido completo sobre
//! un archivo temporal: escribe, lee, caduca, se corrompe y se formatea.
//!
//! `cargo run -p rod-app --example recovery -- sembrar` deja un recovery real
//! (el que lee la app al arrancar) para la captura de pantalla. Es el único
//! modo con efectos fuera del directorio temporal.

use std::path::PathBuf;

use rod_app::recovery_io;
use rod_core::{Caption, Hablante, Proyecto};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match std::env::args().nth(1).as_deref() {
        Some("sembrar") => sembrar(),
        _ => recorrido(),
    }
}

fn recorrido() -> Result<(), Box<dyn std::error::Error>> {
    let ruta: PathBuf =
        std::env::temp_dir().join(format!("rod-ejemplo-recovery-{}.json", std::process::id()));
    let ahora = recovery_io::ms_ahora();
    let copia =
        rod_core::construir_recovery(proyecto(), "/proyectos/mi-video.cdp", ahora - 180_000.0);
    let texto = rod_core::recovery::a_json(&copia)?;
    recovery_io::escribir(&ruta, Some(&texto))?;

    println!(
        "archivo: {} ({} bytes)",
        ruta.display(),
        std::fs::metadata(&ruta)?.len()
    );
    println!("--- contenido ---");
    println!("{texto}");
    println!("--- comprobaciones ---");

    let leido = recovery_io::leer(&ruta, 0.0, ahora).expect("el recovery recién escrito se lee");
    println!("ida y vuelta exacta: {}", leido == copia);
    println!(
        "aviso: {}",
        rod_core::formatear_recovery(&leido, "mi-video.cdp", ahora)
    );

    // Las cuatro decisiones que el autosave toma por segundo.
    let decision = |sucio, n, ultima| rod_core::debe_guardar_recovery(sucio, n, ultima, ahora);
    println!("escribe con cambios: {}", decision(true, 3, None));
    println!("escribe sin cambios: {}", decision(false, 3, None));
    println!("escribe proyecto vacío: {}", decision(true, 0, None));
    println!(
        "escribe 14.9 s después: {}",
        decision(true, 3, Some(ahora - 14_900.0))
    );
    println!(
        "escribe 15 s después: {}",
        decision(true, 3, Some(ahora - 15_000.0))
    );

    // Y los tres descartes que protegen el arranque.
    println!(
        "más viejo que el corte: {:?}",
        recovery_io::leer(&ruta, ahora + 1.0, ahora).map(|_| ())
    );
    let caducado =
        rod_core::construir_recovery(proyecto(), "", ahora - rod_core::VIGENTE_MS - 1000.0);
    let caducado_texto = rod_core::recovery::a_json(&caducado)?;
    recovery_io::escribir(&ruta, Some(&caducado_texto))?;
    println!(
        "más viejo que 7 días: {:?}",
        recovery_io::leer(&ruta, 0.0, ahora).map(|_| ())
    );
    recovery_io::escribir(&ruta, Some("{roto"))?;
    println!(
        "archivo corrupto: {:?}",
        recovery_io::leer(&ruta, 0.0, ahora).map(|_| ())
    );

    recovery_io::escribir(&ruta, None)?;
    println!("tras borrar, existe: {}", ruta.exists());
    assert!(!ruta.exists());
    Ok(())
}

/// Deja un recovery de verdad en la ruta que lee la app, con fecha de hace 3
/// minutos para que el aviso diga algo distinto de «hace un momento».
fn sembrar() -> Result<(), Box<dyn std::error::Error>> {
    let ahora = recovery_io::ms_ahora();
    let copia = rod_core::construir_recovery(
        proyecto(),
        "C:/Dev/RustyOpenDialogue/docs/checklists/mi-video.cdp",
        ahora - 180_000.0,
    );
    let texto = rod_core::recovery::a_json(&copia)?;
    let ruta = recovery_io::ruta();
    recovery_io::escribir(&ruta, Some(&texto))?;
    println!("recovery sembrado en {}", ruta.display());
    println!(
        "aviso esperado: {}",
        rod_core::formatear_recovery(&copia, "mi-video.cdp", ahora)
    );
    Ok(())
}

fn proyecto() -> Proyecto {
    Proyecto {
        ruta_video: "testdata/clip-15s-2audio.mp4".into(),
        hablantes: vec![Hablante {
            id: "sp-1".into(),
            nombre: "Ana".into(),
            tecla: "1".into(),
            color: "#3B82F6".into(),
            preset_id: None,
        }],
        captions: vec![
            Caption {
                id: "cap-1".into(),
                inicio: 1.0,
                fin: 3.0,
                texto: "Recuperado uno".into(),
                hablante_id: Some("sp-1".into()),
            },
            Caption {
                id: "cap-2".into(),
                inicio: 4.0,
                fin: 6.5,
                texto: "Recuperado dos".into(),
                hablante_id: None,
            },
            Caption {
                id: "cap-3".into(),
                inicio: 8.0,
                fin: 10.0,
                texto: "Recuperado tres".into(),
                hablante_id: None,
            },
        ],
        playhead: 9.0,
    }
}
