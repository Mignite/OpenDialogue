//! Verificación del slice 7d sin ventana: editar presets, guardarlos en el
//! archivo compartido y comprobar que el `.ass` del preview usa lo guardado.
//!
//! `cargo run -p rod-app --example estilos`
//!
//! No abre la app: usa la misma `ruta()` que la app y un archivo temporal
//! propio. Deja `docs/checklists/slice-7d-estilos.png`, el raster del `.ass`
//! con el preset editado.

use std::error::Error;
use std::path::Path;

use rod_app::editor::EstadoEditor;
use rod_app::presets_edicion::{CampoPreset, aplicar_campo};
use rod_app::presets_io;
use rod_app::preview::{CANVAS, ass_del_proyecto};
use rod_app::project_io::{Documento, FormatoSubtitulo};
use rod_core::{Caption, Hablante, Proyecto, nuevo_preset_desde};

fn main() -> Result<(), Box<dyn Error>> {
    let ruta = presets_io::ruta()
        .with_file_name(format!("rod-ejemplo-estilos-{}.json", std::process::id()));
    let mut presets = presets_io::leer(&ruta);
    let semilla = presets.clone();
    assert_eq!(
        presets.len(),
        1,
        "el archivo nuevo se siembra con un preset"
    );
    println!(
        "semilla: {} / {} / {}",
        presets[0].nombre, presets[0].fontname, presets[0].fontsize
    );

    let id = presets[0].id.clone();
    // Los mismos `Message::EditarPreset` que manda el panel, aplicados a la
    // lista que la app tendría en memoria.
    for (campo, texto) in [
        (CampoPreset::Nombre, "Cine"),
        (CampoPreset::Fontname, "Georgia"),
        (CampoPreset::Fontsize, "72"),
        (CampoPreset::Outline, "2.5"),
        (CampoPreset::Alignment, "8"),
        (CampoPreset::Color, "#FFE9A8"),
        (CampoPreset::Bold, "true"),
    ] {
        assert!(aplicar_campo(&mut presets[0], campo, texto), "{texto}");
    }
    // Un valor fuera de rango no se aplica: el campo se quedaría en borrador.
    assert!(!aplicar_campo(
        &mut presets[0],
        CampoPreset::Alignment,
        "99"
    ));
    assert!(!aplicar_campo(&mut presets[0], CampoPreset::Fontsize, "4."));

    // «Nuevo» parte del seleccionado y renumera el nombre.
    let mut segundo = nuevo_preset_desde(&presets[0]);
    segundo.nombre = rod_core::nombre_libre(&segundo.nombre, &[presets[0].nombre.clone()]);
    let segundo_id = segundo.id.clone();
    presets.push(segundo);
    println!(
        "editados: {} / {} / {} · {} / {}",
        presets[0].nombre,
        presets[0].fontname,
        presets[0].fontsize,
        presets[1].nombre,
        presets[0].outline
    );

    presets_io::escribir(&ruta, &presets)?;
    let leidos = presets_io::leer(&ruta);
    assert_eq!(leidos, presets, "lo que se guarda es lo que se lee");
    assert_ne!(leidos, semilla, "los cambios llegaron al disco");
    let contenido = std::fs::read_to_string(&ruta)?;
    println!("archivo: {}", ruta.display());
    for linea in contenido.lines().filter(|linea| {
        linea.contains("\"nombre\"")
            || linea.contains("\"fontname\"")
            || linea.contains("\"fontsize\"")
    }) {
        println!("{linea}");
    }

    // «Borrar» deja un solo preset: la app impide borrar el último.
    let mut resto = leidos.clone();
    resto.retain(|preset| preset.id != segundo_id);
    presets_io::escribir(&ruta, &resto)?;
    let leidos = presets_io::leer(&ruta);
    assert_eq!(leidos.len(), 1);
    assert_eq!(leidos[0].id, id);

    let editor = EstadoEditor::nuevo(Documento {
        proyecto: Proyecto {
            ruta_video: "testdata/clip-15s-2audio.mp4".into(),
            hablantes: vec![Hablante {
                id: "sp-1".into(),
                nombre: "Ana".into(),
                tecla: "1".into(),
                color: "#3B82F6".into(),
                preset_id: Some(id.clone()),
            }],
            captions: vec![Caption {
                id: "cap-1".into(),
                inicio: 1.0,
                fin: 5.0,
                texto: "Estilo editado".into(),
                hablante_id: Some("sp-1".into()),
            }],
            playhead: 2.0,
        },
        formato: FormatoSubtitulo::Srt,
        estilos_ass: Vec::new(),
    });
    let ass = ass_del_proyecto(&editor, &leidos);
    println!("--- .ass del preview ---");
    for linea in ass.lines().filter(|linea| linea.starts_with("Style:")) {
        println!("{linea}");
    }
    assert!(ass.contains("Style: H1,Georgia,72,"), "{ass}");
    assert!(ass.contains("&H00A8E9FF"), "el color editado llegó al .ass");

    let mut subtitulos = rod_subs::Subtitulos::nuevo(CANVAS.0, CANVAS.1, &ass);
    let mut lienzo = rod_subs::Lienzo::nuevo(CANVAS.0, CANVAS.1, [0, 0, 0]);
    lienzo.componer(subtitulos.render(2000));
    let imagen = image::RgbImage::from_raw(CANVAS.0, CANVAS.1, lienzo.pixeles().to_vec())
        .expect("el lienzo mide CANVAS.0 * CANVAS.1 * 3");
    let encendidos = imagen.pixels().filter(|pixel| pixel.0 != [0, 0, 0]).count();
    println!("píxeles con texto en el .ass: {encendidos}");
    assert!(encendidos > 0, "el .ass editado no se rasteriza");
    imagen.save(Path::new("docs/checklists/slice-7d-estilos.png"))?;
    std::fs::remove_file(&ruta)?;

    Ok(())
}
