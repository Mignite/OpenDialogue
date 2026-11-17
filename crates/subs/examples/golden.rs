//! Regenera los golden del slice 2: el mismo `.ass` y el mismo instante,
//! renderizado por el libass que trae libmpv (el oráculo), más el frame limpio
//! sobre el que hay que componer para que la comparación de píxeles tenga
//! sentido.
//!
//! Uso: `cargo run -p rod-subs --example golden -- <caso> <ms>`, con el fixture
//! en `crates/subs/tests/fixtures/<caso>.ass`. Escribe
//! `crates/subs/tests/fixtures/golden/<caso>-fondo.png` y `<caso>-oraculo.png`.
//!
//! El fondo es un color plano generado con lavfi en vez de un video: el golden
//! queda en unos pocos KB y no depende de ningún archivo en `testdata/`. El
//! color exacto no importa — se toma del frame que devuelve mpv, así que los dos
//! lados parten del mismo píxel de fondo.
//!
//! Dos detalles que costaron un intento fallido:
//!
//! - **Un solo seek para los dos frames.** Con dos sesiones, cada una aterriza
//!   en un frame distinto (1.04 y 1.08 con la fuente a 25 fps) y un `\fad` sale
//!   con distinta opacidad en el oráculo que en el render propio. Aquí se busca
//!   una vez, se captura el frame limpio y después se añade el `.ass`: el
//!   redibujado es del mismo instante.
//! - **Nuestro render usa el `time-pos` que reporta mpv**, no el que se pidió:
//!   el seek cae en el frame más cercano, y con un desvanecido ese margen se ve.

const W: u32 = 1920;
const H: u32 = 1080;

/// Aire alrededor del texto en el recorte: deja ver si el borde se corta.
const MARGEN: u32 = 12;

/// 5 s de color plano, 25 fps. `s=` en formato `ANCHOxALTO` porque `w` y `h`
/// chocan con la sintaxis de opciones de lavfi.
const FONDO: &str = "av://lavfi:color=c=#404040:s=1920x1080:d=5:r=25";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let caso = args.next().ok_or("uso: golden <caso> <ms>")?;
    let ms: i64 = args.next().ok_or("uso: golden <caso> <ms>")?.parse()?;
    let fixture = format!("crates/subs/tests/fixtures/{caso}.ass");
    let texto = std::fs::read_to_string(&fixture)?;

    let player = rod_player::Player::open(FONDO)?;
    let mpv = player.mpv();

    // No dejar que mpv reinterpreta los estilos del `.ass`.
    mpv.set_property("sub-ass-override", "no")?;
    // VSFilter codificaba el color del subtítulo asumiendo BT.601 y lo mezclaba
    // en un video BT.709: el rojo (255,0,0) vuelve como (255,24,0). Aquí no
    // emulamos ese defecto, así que el oráculo tampoco debe hacerlo.
    mpv.set_property("sub-ass-vsfilter-color-compat", "no")?;
    mpv.set_property("pause", "yes")?;

    let instante = ms as f64 / 1000.0;
    buscar(&player, instante)?;
    let (limpio, t_limpio) = esperar_frame(&player)?;

    mpv.command("sub-add", &[&fixture, "select"])?;
    let (compuesto, t_compuesto) = esperar_frame(&player)?;

    // Dos milisegundos de margen: es el redondeo de `time-pos`, no un frame.
    if (t_compuesto - t_limpio).abs() > 0.002 {
        return Err(
            format!("el redibujado cambió de instante: {t_limpio} s y {t_compuesto} s").into(),
        );
    }

    // Nuestros bitmaps, al instante que mpv dice que está mostrando.
    let nuestro_ms = (t_limpio * 1000.0).round() as i64;
    let libreria = rod_subs::Library::new();
    let mut renderer = rod_subs::Renderer::new(&libreria, W, H);
    renderer.usar_fuentes(rod_subs::PROVEEDOR_AUTODETECT);
    let track = rod_subs::Track::parse(&libreria, &texto);
    let bitmaps = renderer.render(&track, nuestro_ms);

    // El recorte sale de los bitmaps propios: el golden y el test miran la misma
    // región sin repetir números a mano.
    let caja = rod_subs::caja(bitmaps, MARGEN, W, H).ok_or("no hay subtítulos en ese instante")?;
    println!(
        "{caso} en {t_limpio} s ({nuestro_ms} ms): {} bitmap(s), recorte {}x{} en ({}, {})",
        bitmaps.len(),
        caja.ancho,
        caja.alto,
        caja.x,
        caja.y
    );
    println!("libass {}", rod_subs::version());
    // El seek cae en el frame más cercano, no en el instante pedido. Si se
    // regenera el golden, el caso de `tests/oraculo.rs` tiene que decir este
    // número: renderizar a otro milisegundo cambia el `\fad`.
    println!("anota ms = {nuestro_ms} en tests/oraculo.rs");

    guardar(
        &limpio,
        caja,
        &format!("crates/subs/tests/fixtures/golden/{caso}-fondo.png"),
    )?;
    guardar(
        &compuesto,
        caja,
        &format!("crates/subs/tests/fixtures/golden/{caso}-oraculo.png"),
    )?;
    println!("guardados los golden de {caso}");

    Ok(())
}

/// Busca el instante y espera a que mpv entregue ese frame.
///
/// Los flags pendientes se drenan **antes** del seek: si no, el primero que
/// llega describe el frame anterior y el render devuelve ese. Y no basta con
/// mirar `time-pos` (se actualiza al pedir el seek): el aviso de que la búsqueda
/// terminó es el evento `playback-restart`.
fn buscar(player: &rod_player::Player, instante: f64) -> Result<(), Box<dyn std::error::Error>> {
    player
        .mpv()
        .command("seek", &[&instante.to_string(), "absolute", "exact"])?;

    while player.update() & 1 != 0 {}

    let mut reiniciado = false;
    for _ in 0..500 {
        while let Some(Ok(evento)) = player.mpv().wait_event(0.0) {
            if matches!(evento, libmpv2::events::Event::PlaybackRestart) {
                reiniciado = true;
            }
        }

        let posicion = player.mpv().get_property::<f64>("time-pos").unwrap_or(-1.0);
        if reiniciado && posicion >= instante - 0.01 {
            return Ok(());
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Err("mpv no llegó al instante pedido".into())
}

/// Espera el siguiente redibujado y devuelve el frame en RGB y su `time-pos`.
fn esperar_frame(
    player: &rod_player::Player,
) -> Result<(Vec<u8>, f64), Box<dyn std::error::Error>> {
    let mut buf = vec![0u8; (W * H * 4) as usize];

    for _ in 0..500 {
        if player.update() & 1 != 0 {
            player.render(&mut buf, W as usize, H as usize)?;
            let posicion = player.mpv().get_property::<f64>("time-pos").unwrap_or(-1.0);

            // El render context entrega RGBA; se descarta el alfa.
            let rgb = buf
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|px| [px[0], px[1], px[2]])
                .collect();
            return Ok((rgb, posicion));
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Err("mpv no entregó un frame nuevo".into())
}

fn guardar(
    rgb: &[u8],
    caja: rod_subs::Rect,
    destino: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = image::RgbImage::from_raw(W, H, rgb.to_vec()).ok_or("el frame no mide W*H*3")?;
    let recorte =
        image::imageops::crop_imm(&frame, caja.x as u32, caja.y as u32, caja.ancho, caja.alto);
    if let Some(dir) = std::path::Path::new(destino).parent() {
        std::fs::create_dir_all(dir)?;
    }
    recorte.to_image().save(destino)?;
    Ok(())
}
