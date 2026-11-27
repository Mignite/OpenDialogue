//! Oráculo del slice 0b: el mismo `.ass` y el mismo instante, pero compuesto por
//! el libass que trae libmpv dentro. Su PNG es la referencia contra la que se
//! compara el render propio, así que aquí no se escribe ninguna lógica de
//! subtítulos: solo se le pide a mpv su versión de la verdad.
//!
//! Con `--sin-subs` guarda el mismo frame sin subtítulos: es el fondo sobre el
//! que hay que componer para que la comparación de píxeles tenga sentido.

const W: usize = 1920;
const H: usize = 1080;
const INSTANTE: f64 = 1.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sin_subs = std::env::args().any(|arg| arg == "--sin-subs");
    let salida = if sin_subs {
        "testdata/frame-sin-subs.png"
    } else {
        "testdata/oracle-subs.png"
    };

    let player = rod_player::Player::open_sin_audio("testdata/clip-1080p.mp4")?;

    // hr-seek: buscar por tiempo y no por keyframe.
    player.mpv().set_property("hr-seek", "yes")?;
    // No dejar que mpv reinterpreta los estilos del .ass.
    player.mpv().set_property("sub-ass-override", "no")?;
    // VSFilter codificaba el color del subtítulo asumiendo BT.601 y lo mezclaba
    // en un video BT.709: el rojo (255,0,0) vuelve como (255,24,0). Aquí no
    // emulamos ese defecto, así que el oráculo tampoco debe hacerlo.
    player
        .mpv()
        .set_property("sub-ass-vsfilter-color-compat", "no")?;
    player.mpv().set_property("pause", "yes")?;
    if !sin_subs {
        player.mpv().command(
            "sub-add",
            &["crates/subs/tests/fixtures/basico.ass", "select"],
        )?;
    }
    player
        .mpv()
        .command("seek", &[&INSTANTE.to_string(), "absolute", "exact"])?;

    // Tirar los flags pendientes antes de buscar: si no, el primero que llegue
    // describe el frame anterior al seek y el render devuelve ese.
    while player.update() & 1 != 0 {}

    let mut buf = vec![0u8; W * H * 4];
    let mut ultimo = -1.0f64;
    // `time-pos` se actualiza al pedir el seek, pero el frame que mpv entrega
    // todavía puede ser el de antes: el aviso de que el seek terminó es
    // `playback-restart`, no la propiedad.
    let mut reiniciado = false;

    for _ in 0..500 {
        while let Some(Ok(evento)) = player.mpv().wait_event(0.0) {
            if matches!(evento, libmpv2::events::Event::PlaybackRestart) {
                reiniciado = true;
            }
        }

        if player.update() & 1 != 0 {
            player.render(&mut buf, W, H)?;
            ultimo = player.mpv().get_property::<f64>("time-pos").unwrap_or(-1.0);

            if reiniciado && ultimo >= INSTANTE - 0.01 {
                break;
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    println!("frame renderizado en time-pos = {ultimo}");

    if ultimo < INSTANTE - 0.01 {
        return Err("mpv no entregó el frame del instante pedido".into());
    }

    let rgb: Vec<u8> = buf
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|px| [px[0], px[1], px[2]])
        .collect();

    image::RgbImage::from_raw(W as u32, H as u32, rgb)
        .expect("el buffer no mide W*H*3")
        .save(salida)?;

    println!("guardado {salida}");

    Ok(())
}
