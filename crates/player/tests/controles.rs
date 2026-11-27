//! Los controles del player, sin ventana: pausa, seek exacto, pista de audio y
//! velocidad.
//!
//! Son las cosas que la UI promete y una captura no puede demostrar. El test
//! abre un mpv real, así que solo corre donde hay libmpv: Linux (el CI lo
//! instala con `libmpv-dev`). En Windows el import de `libmpv-2.dll` es estático
//! y el runner no tiene la DLL, así que allí el binario de test ni arrancaría;
//! la verificación equivalente es `cargo run -p rod-player --example controles`.
//!
//! El clip no se versiona: el CI lo genera con ffmpeg (3 s, dos pistas de
//! audio). Si falta fuera del CI, el test se salta con un aviso en vez de
//! fallar; dentro del CI falla, porque un test que se salta solo no avisa de
//! nada.

#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const TOLERANCIA_SEEK: f64 = 0.05;
/// Ventana de reloj de pared para medir la velocidad. Corta, porque el clip del
/// CI dura 3 s.
const VENTANA_VELOCIDAD: Duration = Duration::from_millis(600);
const ANCHO: usize = 320;
const ALTO: usize = 180;

/// El clip del CI primero; si no está, el de 15 s que sí está versionado.
fn clip() -> Option<PathBuf> {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    [
        "testdata/clip-3s-2audio.mp4",
        "testdata/clip-15s-2audio.mp4",
    ]
    .iter()
    .map(|relativa| raiz.join(relativa))
    .find(|ruta| ruta.is_file())
}

/// `None` cuando no hay clip y no estamos en CI (se omite el test).
fn abrir() -> Option<rod_player::Player> {
    let Some(ruta) = clip() else {
        let mensaje = "falta testdata/clip-3s-2audio.mp4; genéralo con ffmpeg \
                       (ver .github/workflows/ci.yml)";
        assert!(
            std::env::var_os("CI").is_none(),
            "en CI no se puede omitir: {mensaje}"
        );
        eprintln!("test omitido: {mensaje}");
        return None;
    };
    let texto = ruta.to_str().expect("la ruta del clip debe ser UTF-8");
    Some(rod_player::Player::open(texto).expect("no se pudo abrir el clip"))
}

/// Deja correr el reloj como lo hace la app: bombear eventos y dibujar.
fn avanzar(player: &rod_player::Player, duracion: Duration) {
    let mut buf = vec![0u8; ANCHO * ALTO * 4];
    let inicio = Instant::now();
    while inicio.elapsed() < duracion {
        let _ = player.update();
        let _ = player.render(&mut buf, ANCHO, ALTO);
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Pausa, busca y espera a que mpv diga que llegó.
fn buscar(player: &rod_player::Player, objetivo: f64) {
    player.set_paused(true).expect("pausar antes de buscar");
    player.seek(objetivo).expect("seek");
    let inicio = Instant::now();
    while inicio.elapsed() < Duration::from_secs(3) {
        let _ = player.update();
        if !player.seeking() && (player.position() - objetivo).abs() <= TOLERANCIA_SEEK {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn la_pista_de_audio_cambia_en_mpv() {
    let Some(player) = abrir() else {
        return;
    };
    // En un runner sin dispositivo ALSA, mpv no abre la salida por defecto y
    // deselecciona la pista (`aid` vuelve a "no"): el test pasaría o no según
    // el hardware del runner. Aquí se comprueba el mapeo de `set_audio_track` y
    // `audio_track` a la propiedad `aid`, no el audio en sí, así que se fuerza
    // una salida nula para que el resultado sea el mismo en todas partes.
    player
        .mpv()
        .set_property("ao", "null")
        .expect("salida de audio nula para el test");
    let pistas = player.audio_tracks();
    assert!(
        pistas.len() >= 2,
        "el clip de prueba debe tener dos pistas de audio, tiene {}",
        pistas.len()
    );

    player.set_audio_track(pistas[1].id).expect("cambiar aid");
    assert_eq!(player.audio_track(), Some(pistas[1].id));

    player.set_audio_track(pistas[0].id).expect("volver al aid");
    assert_eq!(player.audio_track(), Some(pistas[0].id));
}

#[test]
fn la_pausa_congela_el_reloj() {
    let Some(player) = abrir() else {
        return;
    };
    let inicio = (player.duration() * 0.3).max(0.2);
    buscar(&player, inicio);
    player.set_paused(false).expect("reanudar");
    avanzar(&player, Duration::from_millis(400));

    // La primera lectura puede caer antes de que la salida de audio consuma su
    // último paquete, así que se mide después de asentar.
    player.set_paused(true).expect("pausar");
    std::thread::sleep(Duration::from_millis(300));

    let antes = player.position();
    std::thread::sleep(Duration::from_millis(500));
    let despues = player.position();

    assert!(player.paused(), "mpv no se quedó en pausa");
    assert!(antes > inicio, "no avanzó antes de pausar: {antes:.3} s");
    println!("pausa: {antes:.3} -> {despues:.3} en 500 ms");
    assert!(
        (despues - antes).abs() < 0.02,
        "el reloj se movió en pausa: {antes:.3} -> {despues:.3}"
    );
}

#[test]
fn el_seek_cae_en_el_instante_pedido() {
    let Some(player) = abrir() else {
        return;
    };
    // A mitad del clip: lejos de los extremos y de cualquier keyframe.
    let objetivo = (player.duration() * 0.6).max(0.2);
    buscar(&player, objetivo);

    let posicion = player.position();
    println!("seek: pedido {objetivo:.3} s, quedó {posicion:.3} s");
    assert!(
        (posicion - objetivo).abs() <= TOLERANCIA_SEEK,
        "se pidió {objetivo:.3} s y quedó en {posicion:.3} s"
    );
}

#[test]
fn el_doble_de_velocidad_avanza_el_doble() {
    let Some(player) = abrir() else {
        return;
    };
    let principio = (player.duration() * 0.1).max(0.1);

    // Se mide el tramo estable: después de reanudar hay una ráfaga de
    // recuperación que no depende de la velocidad y ensuciaría la razón.
    let medir = |velocidad: f64| -> f64 {
        buscar(&player, principio);
        player.set_speed(velocidad).expect("cambiar velocidad");
        player.set_paused(false).expect("reanudar");
        avanzar(&player, Duration::from_millis(150));
        let desde = player.position();
        avanzar(&player, VENTANA_VELOCIDAD);
        let hasta = player.position();
        player.set_paused(true).expect("pausar");
        hasta - desde
    };

    let normal = medir(1.0);
    let doble = medir(2.0);
    assert_eq!(player.speed(), 2.0, "mpv no tomó la velocidad");

    // La ventana incluye el arranque (unos 50-150 ms que no dependen de la
    // velocidad), así que el doble no da 2.0 exacto: el margen es amplio a
    // propósito.
    println!(
        "velocidad en {} ms: 1x avanzó {normal:.3} s, 2x avanzó {doble:.3} s ({:.2}x)",
        VENTANA_VELOCIDAD.as_millis(),
        doble / normal
    );
    assert!(
        normal > 0.3,
        "a velocidad normal el reloj no avanzó: {normal:.3} s"
    );
    assert!(
        doble > normal * 1.5 && doble < normal * 2.5,
        "2x avanzó {doble:.3} s frente a {normal:.3} s en la misma ventana"
    );
}
