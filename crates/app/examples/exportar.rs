//! Verificación del slice 7c sin ventana: las tres exportaciones del oráculo.
//!
//! `cargo run -p rod-app --example exportar`
//!
//! Escribe en una carpeta temporal (deja la ruta en pantalla para revisarla) y
//! guarda `docs/checklists/slice-7c-preset.png`, que es el `.ass` exportado
//! rasterizado por libass: si el preset no llegara al archivo, la imagen no
//! tendría la fuente ni el tamaño esperados.

use std::path::Path;

use rod_app::editor::EstadoEditor;
use rod_app::preview::{CANVAS, ass_del_proyecto};
use rod_app::project_io::{
    Documento, FormatoSubtitulo, exportar_json_combinado, exportar_srt_por_hablante,
};
use rod_core::{Caption, Hablante, PresetAss, Proyecto, preset_por_defecto};

const INSTANTE_MS: i64 = 2000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let carpeta = std::env::temp_dir().join(format!("rod-ejemplo-exportar-{}", std::process::id()));
    std::fs::create_dir_all(&carpeta)?;

    let proyecto = proyecto();
    let presets = presets();
    let documento = Documento {
        proyecto: proyecto.clone(),
        formato: FormatoSubtitulo::Srt,
        estilos_ass: Vec::new(),
    };
    let editor = EstadoEditor::nuevo(documento);
    let preview = ass_del_proyecto(&editor, &presets);

    println!("carpeta: {}", carpeta.display());

    // 1. SRT por hablante (y solapes.txt, si los hay).
    let srt = exportar_srt_por_hablante(&proyecto);
    println!("--- SRT por hablante ---");
    for (nombre, contenido) in &srt.archivos {
        std::fs::write(carpeta.join(nombre), contenido)?;
        println!("{nombre}: {} bytes", contenido.len());
    }
    if let Some(reporte) = &srt.reporte_solapes {
        std::fs::write(carpeta.join("solapes.txt"), reporte)?;
        println!("solapes.txt: {} bytes", reporte.len());
    }
    println!(
        "sin hablante: {}, solapes: {}",
        srt.sin_asignar, srt.solapes
    );
    let ana = &srt.archivos[0].1;
    println!("--- Ana.srt ---\n{ana}");

    // 2. JSON combinado.
    let json = exportar_json_combinado(&proyecto);
    std::fs::write(carpeta.join("subtitulos_combinado.json"), &json)?;
    println!("--- JSON combinado ---\n{json}");

    // 3. El `.ass` del preview, que es el que se exporta.
    std::fs::write(carpeta.join("subtitulos.ass"), &preview)?;
    println!("--- .ass ---");
    for linea in preview
        .lines()
        .filter(|linea| linea.starts_with("PlayRes") || linea.starts_with("Style:"))
    {
        println!("{linea}");
    }

    // Comprobaciones: el archivo y el preview son la misma cadena, el canvas es
    // el del proyecto y el preset de Ana llegó a su estilo.
    assert_eq!(
        std::fs::read_to_string(carpeta.join("subtitulos.ass"))?,
        preview
    );
    assert!(preview.contains("PlayResX: 1920"), "{preview}");
    assert!(preview.contains("Style: H1,Georgia,72,"), "{preview}");
    println!("el archivo y el preview son idénticos: true");

    // Y libass lo acepta: rasterizar el `.ass` exportado da texto en pantalla.
    let mut subtitulos = rod_subs::Subtitulos::nuevo(CANVAS.0, CANVAS.1, &preview);
    let mut lienzo = rod_subs::Lienzo::nuevo(CANVAS.0, CANVAS.1, [0, 0, 0]);
    lienzo.componer(subtitulos.render(INSTANTE_MS));
    let imagen = image::RgbImage::from_raw(CANVAS.0, CANVAS.1, lienzo.pixeles().to_vec())
        .expect("el lienzo mide CANVAS.0 * CANVAS.1 * 3");
    let encendidos = imagen.pixels().filter(|pixel| pixel.0 != [0, 0, 0]).count();
    println!("píxeles con texto en el .ass exportado: {encendidos}");
    assert!(encendidos > 0, "el .ass exportado no se rasteriza");
    imagen.save(Path::new("docs/checklists/slice-7c-preset.png"))?;

    Ok(())
}

/// Dos hablantes, un preset propio y un caption suelto: cubre el estilo por
/// preset, el respaldo sin hablante y el caption que no va a ningún SRT.
fn proyecto() -> Proyecto {
    Proyecto {
        ruta_video: "testdata/clip-15s-2audio.mp4".into(),
        hablantes: vec![
            Hablante {
                id: "sp-1".into(),
                nombre: "Ana".into(),
                tecla: "1".into(),
                color: "#3B82F6".into(),
                preset_id: Some("p-cine".into()),
            },
            Hablante {
                id: "sp-2".into(),
                nombre: "Beto".into(),
                tecla: "2".into(),
                color: "#F59E0B".into(),
                preset_id: None,
            },
        ],
        captions: vec![
            Caption {
                id: "cap-1".into(),
                inicio: 1.0,
                fin: 3.0,
                texto: "Hola desde Cine".into(),
                hablante_id: Some("sp-1".into()),
            },
            Caption {
                id: "cap-2".into(),
                inicio: 4.0,
                fin: 6.0,
                texto: "Beto sin preset".into(),
                hablante_id: Some("sp-2".into()),
            },
            Caption {
                id: "cap-3".into(),
                inicio: 8.0,
                fin: 10.0,
                texto: "Suelto".into(),
                hablante_id: None,
            },
        ],
        playhead: 2.0,
    }
}

fn presets() -> Vec<PresetAss> {
    vec![
        PresetAss {
            id: "p-cine".into(),
            nombre: "Cine".into(),
            fontname: "Georgia".into(),
            fontsize: 72.0,
            color: "#FFE9A8".into(),
            ..preset_por_defecto()
        },
        preset_por_defecto(),
    ]
}
