# Slice 3 — `player`: velocidad y controles probados solos

Criterio de salida: los controles de la barra de transporte (play/pause, seek,
pista de audio y velocidad) comprobados **automáticamente** contra un mpv real,
no solo a ojo en la ventana.

Al empezar el slice, play/pause, seek exacto (`hr-seek`) y `aid` ya existían del
slice 0c, pero su única prueba era un ejemplo que alguien tenía que correr:
`cargo run -p rod-player --example controles`. Faltaban la velocidad —que iced
podía mostrar pero el `Player` no sabía cambiar— y convertir esa prueba de humo
en algo que corra solo.

## Pasos

- [x] **1. Velocidad en el `Player`.** `set_speed`/`speed` sobre la propiedad
  `speed` de mpv, con `normalizar_velocidad` acotando a `[0.01, 100]` (fuera de
  ese rango mpv rechaza la escritura) y `NaN` → `1.0`. Tres tests unitarios del
  acotado, sin mpv: el rango y el `NaN` son lógica propia, no de mpv.
  *Hecho:* `cargo test -p rod-player --lib` → 3 tests.

- [x] **2. Selector de velocidad en la barra de transporte.** Un `pick_list`
  con `Velocidad(f32)` (tipo propio: `f32` no es `Display` y la etiqueta es lo
  que ve el usuario), `0.5x … 2x`, junto al selector de audio. Al elegir, se
  escribe en mpv y se **relee** la velocidad: mpv es la fuente, igual que con
  `pause` y `aid`. Abrir otro video refresca el espejo.
  *Hecho:* tests del formato (`1x`, no `1.0x`; `1.25x` sí) y de que la lista
  está ordenada y contiene `1.0`. `cargo build -p rod-app` y 7 s de ventana real
  con el clip de 15 s, sin errores en stderr.

- [x] **3. `tests/controles.rs`.** Cuatro pruebas con mpv de verdad: la pista de
  audio cambia y se puede volver, la pausa congela el reloj, el seek cae en el
  instante pedido (no en el keyframe anterior) y 2x avanza el doble.
  *Hecho* (Windows, quitando el gate temporalmente):
  ```
  velocidad en 600 ms: 1x avanzó 0.600 s, 2x avanzó 1.200 s (2.00x)
  seek: pedido 1.800 s, quedó 1.800 s
  pausa: 1.360 -> 1.360 en 500 ms
  4 passed
  ```
  El mismo ejemplo manual sobre el clip de 15 s: `1x 0.833 s`, `2x 1.667 s` en
  800 ms, seek `7.370 → 7.371`, `aid` 1 → 2.

- [x] **4. El reloj se mide en régimen, no al arrancar.** La primera versión
  medía desde el instante de reanudar y daba `1.61x` en vez de `2.0x`: al salir
  de pausa mpv recupera el tiempo perdido, y esa ráfaga no depende de la
  velocidad. Ahora se dejan 150 ms de asentamiento, se toma la posición y se
  mide la ventana. Con eso la razón da 2.00 y el margen del test (1.5–2.5) deja
  de ser un adorno.

- [x] **5. El clip de prueba, generado.** `testdata/` no se versiona; el CI de
  Linux genera un `clip-3s-2audio.mp4` con `lavfi` (`testsrc` 320×240 a 25 fps +
  dos `sine`, H.264/AAC) antes de los tests. Fuera del CI, si no hay clip el
  test se salta con aviso; **en CI falla**, para que un workflow roto no pase
  por "4 tests verdes que no corrieron".

- [x] **6. Solo donde hay libmpv.** `#![cfg(unix)]` en el archivo de test. En
  Windows el import de `libmpv-2.dll` es estático y el runner no tiene la DLL
  (SourceForge bloquea a los agentes), así que el binario de test ni arrancaría.
  Ahí la verificación es el ejemplo `controles` y la app. Los tests unitarios
  del acotado sí corren en Windows: el binario no acaba enlazando el import,
  comprobado ejecutándolo desde `build/rod-player/<hash>/out` sin DLL al lado ni
  en el PATH.

- [x] **7. Cerrar.** `cargo fmt --all -- --check`, `cargo clippy --all-targets
  -- -D warnings` y `cargo test --workspace` en verde con 1.99 y 1.100; CI en
  verde en los dos sistemas.

## Hallazgos

- **Medir velocidad es medir el arranque.** Del primer intento salió `1.61x`
  porque la ventana empezaba al reanudar. mpv, al quitar la pausa, avanza lo que
  el reloj de audio se retrasó. No era la velocidad: era la recuperación. La
  cura fue medir dos tramos estables, y de paso dejó un número más creíble
  (`2.00x`) que cualquier tolerancia generosa.

- **Un test que se salta solo es peor que uno que falla.** El clip del CI vive
  en `testdata/`, que está gitignoreado. Si el paso de `ffmpeg` desaparece del
  workflow, un "skip" silencioso deja el job verde sin haber probado nada. Por
  eso el salto solo se permite fuera de CI (`CI` no definida).

- **En Windows el test automático no puede existir.** No es el código: es el
  enlazado. `libmpv2-sys` enlaza `mpv` con import library, y sin la DLL el
  proceso muere antes de `main` (`STATUS_DLL_NOT_FOUND`). Gate `unix` y a otra
  cosa; el día que el player se pruebe en Windows hará falta una DLL que el
  runner pueda descargar, o el mismo giro a carga dinámica que se propuso para
  libass.

## Qué NO hacer en este slice

- No probar el volumen ni el stepping: no existen todavía en `Player`; entran
  por el mismo `Row` cuando toque, sin rediseñar nada.
- No añadir `--ao=null` ni tocar opciones de mpv para que el test sea
  determinista. El test corre con la salida de audio real (WASAPI en local); si
  un runner sin tarjeta de sonido se queja, se resolverá entonces con evidencia,
  no por si acaso.
- No mover la lógica de los controles a una capa intermedia. `Player` ya es la
  capa: la app solo refleja lo que mpv dice.
