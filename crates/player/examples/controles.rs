//! Prueba de humo de los controles del player, sin ventana.
//!
//! Verifica lo que la UI promete y no se puede ver en una captura: que pausar
//! congela el reloj, que el seek cae en el instante pedido (exacto, no en el
//! keyframe anterior) y que la pista de audio y la velocidad cambian de verdad
//! en mpv.
//!
//! Es la versión para Windows de `tests/controles.rs`: el test automático solo
//! corre donde hay libmpv (el CI de Linux), porque en Windows el import de la
//! DLL es estático y el runner no la tiene.
//!
//! `cargo run -p rod-player --example controles` desde la raíz del repo, con
//! `third_party/mpv` en el PATH. Sale con código 1 si alguna comprobación falla.

use std::time::{Duration, Instant};

const CLIP: &str = "testdata/clip-15s-2audio.mp4";
/// Instante que casi seguro no cae en un keyframe.
const OBJETIVO: f64 = 7.37;
const TOLERANCIA: f64 = 0.05;

fn main() {
    let player = rod_player::Player::open_sin_audio(CLIP).expect("no se pudo abrir el clip");
    let mut fallos = 0;

    println!("duracion: {:.3} s", player.duration());

    // --- Pistas de audio -------------------------------------------------
    let pistas = player.audio_tracks();
    for pista in &pistas {
        println!("pista {}: {}", pista.id, pista.label);
    }
    if pistas.len() < 2 {
        println!(
            "FALLO: el clip debe tener 2 pistas de audio, hay {}",
            pistas.len()
        );
        fallos += 1;
    }

    if let Some(segunda) = pistas.get(1) {
        player.set_audio_track(segunda.id).expect("set aid");
        let activa = player.audio_track();
        println!("aid tras cambiar a {}: {activa:?}", segunda.id);
        if activa != Some(segunda.id) {
            println!("FALLO: mpv no tomo la pista {}", segunda.id);
            fallos += 1;
        }
    }

    let salida = player
        .mpv()
        .get_property::<String>("current-ao")
        .or_else(|_| {
            player
                .mpv()
                .get_property::<String>("audio-out-params/format")
        })
        .unwrap_or_else(|_| "?".to_owned());
    println!("salida de audio activa: {salida}");

    // --- Pausa ------------------------------------------------------------
    // La primera lectura puede caer antes de que la salida de audio arranque y
    // consuma su primer paquete (~33 ms), así que se mide después de asentar.
    player.set_paused(true).expect("pausar");
    std::thread::sleep(Duration::from_millis(400));
    let antes = player.position();
    std::thread::sleep(Duration::from_millis(500));
    let despues = player.position();
    println!("en pausa: {antes:.3} -> {despues:.3}");
    if player.paused() && (despues - antes).abs() < 0.02 {
        println!("OK pausa congela el reloj");
    } else {
        println!("FALLO: el reloj se movio en pausa");
        fallos += 1;
    }

    // --- Seek exacto ------------------------------------------------------
    player.seek(OBJETIVO).expect("seek");
    let inicio = Instant::now();
    let mut alcanzado = None;
    while inicio.elapsed() < Duration::from_secs(3) {
        let posicion = player.position();
        if (posicion - OBJETIVO).abs() <= TOLERANCIA && !player.seeking() {
            alcanzado = Some(posicion);
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    match alcanzado {
        Some(posicion) => println!("OK seek exacto: pedido {OBJETIVO:.3}, quedo {posicion:.3}"),
        None => {
            println!(
                "FALLO: seek a {OBJETIVO:.3} quedo en {:.3} (keyframe?)",
                player.position()
            );
            fallos += 1;
        }
    }

    // --- Reproduccion -----------------------------------------------------
    player.set_paused(false).expect("reanudar");
    std::thread::sleep(Duration::from_millis(400));
    let posicion = player.position();
    println!("tras reanudar: {posicion:.3}");
    if posicion > OBJETIVO + 0.1 {
        println!("OK reanuda y avanza");
    } else {
        println!("FALLO: no avanzo tras reanudar");
        fallos += 1;
    }

    // --- Velocidad --------------------------------------------------------
    // Se mide reloj de pared contra reloj del medio, como en el test automático
    // pero con una ventana más larga.
    let medir = |velocidad: f64| -> f64 {
        player.set_speed(velocidad).expect("cambiar velocidad");
        player.seek(OBJETIVO).expect("seek antes de medir");
        std::thread::sleep(Duration::from_millis(300));
        player.set_paused(false).expect("reanudar");
        // Tramo estable: al reanudar hay una ráfaga de recuperación que no
        // depende de la velocidad.
        std::thread::sleep(Duration::from_millis(150));
        let inicio = player.position();
        std::thread::sleep(Duration::from_millis(800));
        let fin = player.position();
        player.set_paused(true).expect("pausar");
        fin - inicio
    };
    let normal = medir(1.0);
    let doble = medir(2.0);
    println!("avance en 800 ms: 1x {normal:.3} s, 2x {doble:.3} s");
    if normal > 0.3 && doble > normal * 1.4 && doble < normal * 2.8 {
        println!("OK la velocidad cambia el ritmo");
    } else {
        println!("FALLO: 2x no avanzo aproximadamente el doble");
        fallos += 1;
    }
    if player.speed() != 2.0 {
        println!("FALLO: mpv no reporta la velocidad pedida");
        fallos += 1;
    }

    if fallos == 0 {
        println!("\nTODO OK");
    } else {
        println!("\n{fallos} comprobaciones fallaron");
        std::process::exit(1);
    }
}
