# Bitácora

Una entrada por tropiezo que costó más de 20 minutos. El valor está en el
diagnóstico, no en la solución: el mismo malentendido suele repetirse con otra
cara.

## Formato

```md
## AAAA-MM-DD — título corto

- **Intentaba:**
- **Error literal:**
  ```
  (pegar el mensaje completo, no un resumen)
  ```
- **Qué estaba mal** (concepto, no síntoma):
- **Qué aprendí:**
- **Dónde quedó** (archivo:línea, commit):
```

---

## 2026-11-11 — libmpv2 no expone el render por software

- **Intentaba:** usar `libmpv2` como fachada única para video, incluyendo el
  render a memoria.
- **Error:** ninguno de compilación; el problema es de API. El enum
  `libmpv2::render::RenderParam` solo tiene variantes para OpenGL
  (`ApiType`, `InitParams`, `FBO`, …). No existen `SW_SIZE`, `SW_FORMAT`,
  `SW_STRIDE` ni `SW_POINTER`.
- **Qué estaba mal:** asumí que la crate cubría la API de render completa porque
  el módulo se llama `render`. La cubre a medias: OpenGL sí, software no.
- **Qué aprendí:** `libmpv2-sys` 4.0.1 sí trae los parámetros `SW_*` (bindings
  pregenerados, sin libclang) y enlaza con `cargo:rustc-link-lib=mpv`. El
  contexto de render por software se escribe a mano sobre el crate `-sys`, con
  una API segura encima. De paso: `libmpv-sys` en crates.io es de otro autor y
  está muerto (2020); el bueno es `libmpv2-sys`.
- **Dónde quedó:** spec §3.2 y §4.3; checklist `slice-0a-mpv.md`, paso 3.

## 2026-11-13 - la ventana de iced se queda gris con el frame de mpv

- **Intentaba:** mostrar el frame de mpv (1920x1080, RGBA) en un widget `image`
  con un `Handle::from_rgba` nuevo por frame.
- **Error literal:** ninguno. La ventana se queda en el fondo gris y el contador
  marca 18 fps.
- **Qué estaba mal:** `iced_wgpu` 0.14 no sube imágenes grandes dentro del
  frame. En `image/cache.rs`, `upload_raster` solo sube de forma síncrona si
  `image.len() < 2 * 1024 * 1024`; un frame de 1080p son 8 294 400 bytes, así
  que cae en el camino asíncrono (hilo worker) y esa llamada devuelve `None`,
  o sea que no se añade ninguna instancia y no se dibuja nada. Y como
  `Handle::from_rgba` estrena `Id::unique()` cada vez, `trim()` suelta toda
  imagen que no se haya dibujado en ese frame: la textura que el worker sube
  llega cuando su handle ya no se usa, y se tira. El ciclo sube y tira 8 MiB
  por frame sin pintar un solo píxel, y de ahí los 18 fps.
  El video que sí se vio antes no contradice esto: el clip había terminado
  (timecode 00:00:02.933 de un clip de 3 s), el handle dejó de cambiar y ese
  único frame sí llegó a dibujarse desde la caché.
- **Qué aprendí:** el widget `image` de iced es para iconos, no para video: hay
  un umbral de 2 MiB y la caché va por identidad de handle, no por contenido.
  Para un flujo de frames hay que gestionar la textura uno mismo
  (`shader::Primitive` + `queue.write_texture` sobre una textura creada una
  sola vez).
- **Dónde quedó:** `crates/app/examples/video.rs`; checklist `slice-0c-iced.md`
  paso 3.

## 2026-11-15 - el rojo del subtítulo no coincide con mpv

- **Intentaba:** cerrar el invariante del slice 0b comparando nuestro render
  (libass de vcpkg, composición alfa en CPU) contra el frame de mpv con el mismo
  `.ass` al mismo instante.
- **Error literal:** ninguno. Resultado de la comparación: 0.426 % de píxeles
  distintos, máximo 24/255. El PNG de diferencias dibujaba exactamente
  "Arriba centrado" y nada más.
  ```
  (760,320) propio=(255,0,0)   oraculo=(255,24,0)
  (800,345) propio=(255,241,0) oraculo=(255,241,0)
  ```
- **Qué estaba mal:** no era el render propio. mpv emula por defecto el defecto
  de VSFilter: el color del subtítulo se codifica asumiendo BT.601 y se mezcla
  en un video BT.709. Codificar `(255,0,0)` como BT.601 y decodificarlo como
  BT.709 da `(255,24,0)`, ni más ni menos. Por eso el texto blanco coincidía
  exacto (los grises son invariantes al cambio de matriz) y solo fallaban los
  colores saturados, que es la firma del problema.
- **Qué aprendí:** con `sub-ass-vsfilter-color-compat=no` el oráculo coincide:
  **0.000 %** de píxeles con diferencia > 8/255, 0.064 % con cualquier
  diferencia y máximo 2/255 (el redondeo de la mezcla). Esa es la línea base:
  libass 0.17.5 de vcpkg, proveedor de fuentes **directwrite (with GDI)**,
  Arial → ArialMT. Y la app no va a emular el mangleo: es un defecto ajeno, no
  una decisión de diseño.
- **Dónde quedó:** `crates/subs/examples/componer.rs`,
  `crates/player/examples/oracle_subs.rs`; checklist `slice-0b-libass.md`
  pasos 4-6 y hallazgos.

## 2026-11-15 - los subtitulos hunden el primer segundo a 13 fps

- **Intentaba:** medir el paso 5 del slice 0c (video de mpv + capas de libass
  con blending) y quedarme con los fps sostenidos.
- **Error literal:** ninguno; el contador arrancaba en `12.9 fps - 5 capas -
  mpv 2.4 ms, libass 15.5 ms` y a partir del segundo siguiente saltaba a
  `60.7 fps - mpv 10.7 ms, libass 0.2 ms`.
- **Que estaba mal:** no era el render. La primera llamada a
  `ass_render_frame` hace que libass inicialice el proveedor de fuentes
  (directwrite) y cachee los glifos de la fuente; esos 15 ms se pagaban en el
  primer frame visible, con la ventana ya abierta, asi que el primer segundo
  entero se dibujaba a media cadencia. Al calentar con un `render()` en el
  constructor del `Subtitulos`, el coste se va antes de que haya ventana.
- **Que aprendi:** el coste de inicializacion de libass es de decenas de ms y
  solo aparece una vez, pero cae justo en el peor momento (primer frame). Cualquier
  medicion de fps de los primeros 1-2 s sin warm-up es basura. Y el coste real
  de rasterizar 5 capas por frame es 0.2 ms: despreciable frente a los 12 ms
  de mpv.
- **Donde quede:** `crates/app/examples/video.rs`,
  `crates/subs/src/lib.rs` (`Subtitulos`); checklist `slice-0c-iced.md` paso 5.

## 2026-11-15 - "arregle" una geometria que ya estaba bien

- **Intentaba:** meter la barra de transporte debajo del video (paso 6a del slice
  0c). Di por hecho que el quad del video se dibujaba en NDC del target completo y
  que iced solo recortaba con scissor, asi que al dejar de llenar la ventana el
  video se iria hacia abajo. Cambie `Escala` por un rectangulo en NDC calculado
  desde `bounds` y `viewport`.
- **Error literal:** ninguno; el video aparecio a 461x305 pegado a la esquina
  superior izquierda de un widget de 768x507. Los numeros no cuadraban con el
  uniform (`rect=[-1.0, 0.8956, 0.2, -0.3044]` imprimia `prepare`).
- **Que estaba mal:** el diagnostico. `iced_wgpu` recorre las primitivas y por
  cada una hace `set_viewport(bounds.x, bounds.y, w, h, 0.0, 1.0)` antes de
  dibujar, asi que NDC -1..1 **es el widget**. Al pasarle un rect en NDC de la
  ventana lo escale dos veces: 461/768 = 0.60 = 768/1280 y 305/432 = 0.70 =
  507/720. Esas dos razones fueron la pista.
- **Que aprendi:** para ver la geometria de un quad sin que el contenido del video
  estorbe, pintar el fragment de magenta y medir el bbox. Tardo un minuto y delata
  el rectangulo exacto. Y al leer el codigo de iced, `set_viewport` esta a seis
  lineas de donde estaba mirando (`lib.rs` ~551).
- **Donde quede:** `docs/checklists/slice-0c-iced.md` paso 6a y hallazgos;
  `crates/app/examples/video.rs` con el esqueleto de la ventana.

## 2026-11-15 — el render SW de mpv deja margen de sobra

- **Intentaba:** medir el coste de 100 renders consecutivos a 1080p para decidir
  si el pipeline de libmpv por software podía sostener el plan.
- **Error literal:** ninguno. `cargo run -p rod-player --example measure`
  terminó correctamente y reportó:
  ```
  media 1.80 ms   p95 2.40 ms   fps 556.1
  ```
- **Qué estaba mal:** no había evidencia numérica del coste, solo la impresión
  de que el vídeo iba fluido en la ventana. La medición usa `untimed` y un
  buffer RGBA de 1920×1080, aislando la llamada de render.
- **Qué aprendí:** 1.80 ms está muy por debajo del límite de 33 ms para 30 fps,
  así que el render SW no obliga a cambiar de arquitectura; el techo observado
  en la ventana viene de la cadencia del vídeo, no de esta conversión.
- **Dónde quedó:** `crates/player/examples/measure.rs`,
  `docs/checklists/slice-0a-mpv.md` pasos 6-7.

## 2026-11-15 - una hora persiguiendo una captura que mentia

- **Intentaba:** verificar en la ventana real que pausar congela el timecode y que
  el seek del slider cae donde se pide, sin pedirle a nadie que mirara la pantalla.
- **Error literal:** ninguna excepcion. Dos capturas separadas 800 ms daban la
  misma firma de pixeles (`2458368/1656`) en la region del timecode Y en la del
  video, en tres procesos distintos. Conclui que la app estaba congelada.
- **Que estaba mal:** la herramienta. `PrintWindow(hdc, 2)` devuelve el ultimo
  frame compuesto de la ventana; con wgpu y la ventana ocluida no se recompone
  nada, asi que la captura es una foto vieja. Lo descarte midiendo el proceso:
  vivo y con 11.8 s de CPU en 9 s de reloj, o sea dibujando a 60 fps.
- **Que aprendi:** para verificar movimiento hay que capturar la pantalla real
  (`CopyFromScreen`) con la ventana restaurada y al frente, no la ventana. Y que
  cuando el resultado es "una persona mira y dice si se congela", pedirlo cuesta
  treinta segundos: los tres controles se confirmaron asi al primer intento,
  despues de una hora de scripts. La medicion automatica se queda donde si vale:
  `cargo run -p rod-player --example controles`, que mide sin ventana y sin
  capturas (seek 7.370 -> 7.371, pausa 0.050 -> 0.050, aid 1 -> 2, ao wasapi).
- **Donde quede:** slice 0c cerrado (commit `0535a91`); checklist con las
  desviaciones frente al oraculo y el paso 6c pendiente (selector de velocidad).

## 2026-11-16 — dos búsquedas en mpv no dan el mismo frame

- **Intentaba:** automatizar en `cargo test` la comparación contra mpv que en el
  slice 0b se hacía a ojo, guardando solo el recorte del subtítulo como golden.
- **Error literal:** ninguno; la comparación daba `0.000 %` desde el primer
  intento. Pero el generador imprimía `frame de mpv en time-pos` dos veces con
  valores distintos — `1.08` y `1.04` — para el mismo `seek` a `1.0`. Con el
  `\fad(1000,1000)` del fixture de variantes, medio frame de diferencia es un
  4 % de alfa: el golden y el render propio habrían estado comparando instantes
  distintos, y el fallo habría apuntado al código de composición.
- **Qué estaba mal:** pedía el frame dos veces, en dos sesiones (una sin
  subtítulos y otra con ellos). Cada `seek` aterriza en el frame más cercano y
  no necesariamente en el mismo. Además `time-pos` no es el instante pedido:
  pedir `1.0` da `1.08` con una fuente de 25 fps.
- **Qué aprendí:** para comparar dos renders hay que fijar el instante con el
  que **dice** mpv, no con el que se pidió: se busca una vez, se captura el
  frame limpio, se añade el `.ass` y `sub-add` redibuja el mismo instante. Y un
  golden con desvanecido es un detector de imprecisiones temporales gratis: fue
  el que delató el problema.
- **Dónde quedé:** slice 2 cerrado. `rod-subs` ya tiene `Lienzo`/`caja` como
  biblioteca (12 tests), `examples/golden.rs` para regenerar y
  `tests/oraculo.rs` con dos golden (`basico`, `variantes`): 0 píxeles por
  encima de 8/255 y máximo 2/255, con libass 0.17.5 y Arial por DirectWrite.
  De paso se arregló `version()`: la versión viene en BCD (`0x01705000` =
  0.17.5) y el primer intento imprimía `112.80.0`.

## 2026-11-16 — mi primer test de Windows CI que no podía arrancar

- **Intentaba:** que los tests nuevos de `rod-subs` pasaran en el CI de Windows.
  Corrían bien en local y en Ubuntu; en Windows el job terminó con
  `STATUS_DLL_NOT_FOUND` (0xc0000135) sobre
  `target/debug/build/rod-subs/<hash>/out/rod_subs-<hash>.exe`.
- **Error literal:** `error: test failed, to rerun pass '-p rod-subs --lib'` y,
  debajo, `process didn't exit successfully ... (exit code: 0xc0000135,
  STATUS_DLL_NOT_FOUND)`.
- **Qué estaba mal:** no era el código. Cargo 1.100 mueve los binarios de test de
  un paquete con build script a `build/<paquete>/<hash>/out`, y ese hash no es el
  `OUT_DIR` del build script (otro hash), así que las DLLs de libass que el
  `build.rs` deja junto a `deps/` no están al lado del `.exe`. Como el import de
  `ass-9.dll` es estático, el proceso muere antes de `main`. En local no se veía
  porque la máquina tiene su propio camino feliz.
- **Qué aprendí:** reproducir la versión del CI (`rustup toolchain install
  1.100.0`, `cargo +1.100.0 test`) convierte "no entiendo el log" en un fallo
  local en dos minutos. Mi primer remedio (copiar las DLLs al `OUT_DIR`) no
  servía, y lo delató comparar los dos hashes. El arreglo real es por PATH en el
  CI; el definitivo es enlazar libass estáticamente en el slice 8.
- **Dónde quedé:** `.github/workflows/ci.yml` (paso de dependencias de Windows),
  `crates/subs/build.rs` (comentario con el porqué) y
  `docs/checklists/slice-2-subs.md` (hallazgo).

## 2026-11-16 — el 2x que medía 1.61x

- **Intentaba:** cerrar el slice 3 con un test que demostrara que `speed = 2`
  duplica el ritmo de reproducción, midiendo reloj de pared contra `time-pos`.
- **Error literal:** ninguna aserción falló, y ese fue el problema: daba
  `1x avanzó 0.660 s, 2x avanzó 1.060 s (1.61x)` en una ventana de 600 ms. Con
  un 2x de verdad, 1.060/0.660 no cuadra.
- **Qué estaba mal:** no era la velocidad. La ventana empezaba en el mismo
  instante en que se quitaba la pausa, y mpv aprovecha ese momento para
  recuperar el retraso del reloj de audio: ese salto inicial es igual con 1x y
  con 2x, así que diluía la razón. En el ejemplo manual, con 800 ms, salía
  `0.651` de 0.800 posibles por lo mismo.
- **Qué aprendí:** para medir una tasa hay que medir en régimen, no en el
  transitorio. Dejar 150 ms de asentamiento, tomar la posición ahí y medir la
  ventana siguiente da `0.600` y `1.200` —2.00x— y el margen del test deja de
  ser una apuesta. Lo mismo sirve para cualquier medición de rendimiento sobre
  mpv: el primer tramo después de un cambio de estado no representa la tasa.
- **Dónde quedé:** `crates/player/tests/controles.rs` (4 tests: pausa, seek,
  `aid` y velocidad; `#![cfg(unix)]` porque el CI de Windows no puede cargar
  libmpv), `crates/player/examples/controles.rs` (la versión manual para
  Windows, ahora con velocidad), `set_speed`/`speed` en `Player`, el selector
  `0.5x…2x` en la barra de transporte, y el CI de Linux generando el clip de 3 s
  con `ffmpeg -f lavfi` antes de los tests.

## 2026-11-17 — capturar la ventana de fuera miente (y molesta)

- **Intentaba:** adjuntar al slice 4 una captura del compositor en una ventana
  más alta que 16:9, para ver las barras negras y los subtítulos alineados.
- **Error literal:** ninguna excepción. La primera captura salió desalineada
  (el video ocupaba 720 px de una imagen de 1000) y la segunda salió con el
  escritorio de Minecraft y un vídeo de YouTube de fondo.
- **Qué estaba mal:** dos cosas distintas. Sin
  `SetProcessDpiAwarenessContext`, `SetWindowPos`/`GetWindowRect`/
  `CopyFromScreen` hablan en unidades virtualizadas: pedí 1000×800 lógicos y
  capturé 1000×800 físicos de una ventana de 1250×1000. Y `SetForegroundWindow`
  no puede traer al frente un proceso que el usuario no pidió, así que
  `CopyFromScreen` copió lo que estaba encima.
- **Qué aprendí:** una captura es evidencia solo si el script controla DPI y
  primer plano; si el escritorio está en uso, no hay captura honesta. Mejor un
  test de la aritmética (que sí se puede repetir) y la captura del slice 0c para
  lo visual, que insistir y grabar la pantalla de otra persona.
- **Dónde quedé:** `crates/app/src/video/geometria.rs` con `escala_letterbox` y
  `rect_de_capa` (9 tests nuevos, 14 en `rod-app`), `video/mod.rs` usando la
  biblioteca, y la desviación del canvas fijo 1920×1080 anotada en la spec.
