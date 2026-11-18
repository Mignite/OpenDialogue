//! Preview sin GPU: el mismo `.ass` que exportaría la app, rasterizado con
//! libass y compuesto sobre negro.
//!
//! Es la verificación visual del slice 5 que no depende del escritorio: se ve
//! que el texto editado llega al `.ass` del preview, y que deshacer devuelve
//! exactamente la imagen anterior.
//!
//! `cargo run -p rod-app --example preview`

use rod_app::editor::EstadoEditor;
use rod_app::preview::{CANVAS, ass_del_proyecto};
use rod_app::project_io::cargar;

const INSTANTE_MS: i64 = 2000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let documento = cargar(
        "1\n00:00:00,000 --> 00:00:04,000\nHola mundo\n\n\
         2\n00:00:04,000 --> 00:00:08,000\nSegundo caption\n",
        "testdata/clip-15s-2audio.mp4",
    )?;
    let mut editor = EstadoEditor::nuevo(documento);

    let antes = render(&editor)?;
    antes.save("docs/checklists/slice-5-preview-antes.png")?;

    println!("ass del preview: {} bytes", ass_del_proyecto(&editor).len());
    println!("texto del caption 1: {:?}", editor.texto_de("cap-1"));
    println!("píxeles con texto antes: {}", encendidos(&antes));

    assert!(editor.aplicar_texto("cap-1", "Texto editado a mano"));
    let despues = render(&editor)?;
    despues.save("docs/checklists/slice-5-preview-despues.png")?;

    println!("texto tras aplicar: {:?}", editor.texto_de("cap-1"));
    println!("píxeles con texto después: {}", encendidos(&despues));
    println!("la imagen cambió: {}", antes.as_raw() != despues.as_raw());

    assert!(editor.deshacer());
    let deshecho = render(&editor)?;
    println!("texto tras deshacer: {:?}", editor.texto_de("cap-1"));
    println!(
        "deshacer devuelve la imagen exacta: {}",
        antes.as_raw() == deshecho.as_raw()
    );

    Ok(())
}

/// El `.ass` del proyecto rasterizado por libass y compuesto sobre negro.
fn render(editor: &EstadoEditor) -> Result<image::RgbImage, Box<dyn std::error::Error>> {
    let ass = ass_del_proyecto(editor);
    let mut subtitulos = rod_subs::Subtitulos::nuevo(CANVAS.0, CANVAS.1, &ass);
    let bitmaps = subtitulos.render(INSTANTE_MS);

    let mut lienzo = rod_subs::Lienzo::nuevo(CANVAS.0, CANVAS.1, [0, 0, 0]);
    lienzo.componer(bitmaps);

    Ok(
        image::RgbImage::from_raw(CANVAS.0, CANVAS.1, lienzo.pixeles().to_vec())
            .expect("el lienzo mide CANVAS.0 * CANVAS.1 * 3"),
    )
}

fn encendidos(imagen: &image::RgbImage) -> u64 {
    imagen.pixels().filter(|pixel| pixel.0 != [0, 0, 0]).count() as u64
}
