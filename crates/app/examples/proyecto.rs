//! El `.cdp` de ida y vuelta sobre el disco, sin diálogo ni ventana.
//!
//! Es la verificación del slice 7a que no depende de que alguien haga clic en
//! «Guardar»: escribe con `guardar_proyecto`, lo lee con `abrir_proyecto` y
//! compara. El archivo que queda es el que tiene que abrir la app Tauri.
//!
//! `cargo run -p rod-app --example proyecto`

use std::path::Path;

use rod_app::project_io::{abrir_proyecto, guardar_proyecto};
use rod_core::{Caption, Hablante, Proyecto};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ruta = Path::new("docs/checklists/slice-7a-proyecto.cdp");

    let proyecto = Proyecto {
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
                texto: "Primer caption".into(),
                hablante_id: Some("sp-1".into()),
            },
            Caption {
                id: "cap-2".into(),
                inicio: 4.0,
                fin: 6.5,
                texto: "Segundo caption".into(),
                hablante_id: None,
            },
        ],
        playhead: 2.75,
    };

    guardar_proyecto(ruta, &proyecto)?;
    let vuelta = abrir_proyecto(ruta)?;

    println!(
        "archivo: {} ({} bytes)",
        ruta.display(),
        std::fs::metadata(ruta)?.len()
    );
    println!("--- contenido ---");
    print!("{}", std::fs::read_to_string(ruta)?);
    println!("--- comprobaciones ---");
    println!("ida y vuelta exacta: {}", vuelta == proyecto);
    println!("captions: {}", vuelta.captions.len());
    println!("hablantes: {}", vuelta.hablantes.len());
    println!("playhead: {}", vuelta.playhead);
    println!(
        "temporal .tmp al lado: {}",
        ruta.with_extension("tmp").exists()
    );

    assert_eq!(vuelta, proyecto, "el proyecto no sobrevive al disco");
    Ok(())
}
