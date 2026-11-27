//! Paso 1-3 del slice 0b: que libass enlace, resuelva fuentes y rasterice.
//! Imprime el log (para ver el proveedor) y la geometría de cada bitmap, que es
//! lo que después hay que volcar sobre el frame.

fn main() {
    let library = rod_subs::Library::new();
    let mut renderer = rod_subs::Renderer::new(&library, 1920, 1080);
    renderer.usar_fuentes(rod_subs::PROVEEDOR_AUTODETECT);

    let track = rod_subs::Track::parse(&library, include_str!("../tests/fixtures/basico.ass"));

    for ms in [0, 1000, 2500] {
        let bitmaps = renderer.render(&track, ms);
        println!("t = {ms} ms -> {} bitmap(s)", bitmaps.len());
        for b in bitmaps {
            println!(
                "  {}x{} stride {} en ({}, {}) color {:#010x} alfa {} bytes",
                b.ancho,
                b.alto,
                b.stride,
                b.x,
                b.y,
                b.color,
                b.alpha().len()
            );
        }
    }

    println!("--- log de libass ---");
    for linea in library.log() {
        println!("{linea}");
    }
}
