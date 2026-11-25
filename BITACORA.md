# Bitácora

## 2026-11-24 — Rediseño visual (fases 1–5)

- `Hablante.color` pasa a ser identidad de UI (12 muestras fijas); el texto del
  subtítulo sigue usando el preset con `rod_core::color_hablante`. El `.ass` no
  cambia. Coste: colores arbitrarios de `.cdp` viejos se respetan hasta que se
  elija una muestra.
- Inter y JetBrains Mono se instalan en la app. `tema::fuente_segura` conserva
  el tamaño del estilo con una familia siempre registrada en las vistas que
  prueba kittest, que no puede instalar familias con nombre antes del primer
  paint.
- kittest no ejecuta el `PaintCallback` de mpv: las capturas finales del editor
  son headless con el monitor vacío. El «antes» real con video quedó en
  `docs/checklists/rediseno/antes-editor-1440.png`.
- Panel derecho: ancho común 360/300/520 persistido; sin título duplicado;
  pestañas de 36 px con badges; lista con búsqueda y panel fijo de inspector;
  Speakers con tarjetas y paleta saturada en popover; Styles con vista previa
  y campos rotulados; `Apply` desaparece (edición en vivo).
- Scrubbing: tiempo de UI optimista al arrastrar, redondeado al cuadro, seek
  último-gana y el decoder no sobreescribe mientras se arrastra.
- Pendiente del rediseño del panel: diálogo completo de importación con
  detección de codificación/desfase/mapeo, toast del `.srt` vecino, campos de
  mayúsculas/fondo en estilos, mute/solo reales y scroll automático de la
  selección. Los puntos de entrada (menú, Ctrl+Shift+I y estado vacío) sí están.
- Inicio (pulido): fila de Recientes con etiqueta espaciada, filtro y See all;
  grilla de 3/2/1 columnas con celdas posicionadas a mano (el `Grid` y el
  `horizontal_wrapped` escalonaban las tarjetas); tarjetas con meta a la
  derecha, puntos por hablante, menú ⋯ en hover y barra segmentada; proyectos
  vacíos ocultos con interruptor en el filtro; dedupe por ruta absoluta.
  Drop zone de 96 px con icono y estados de arrastre. Preferences pasó a la
  cabecera como engranaje con Ctrl+,; cerrar solo se pinta rojo en hover.
- Inicio: el título pasa a ser el nombre de la app con subrayado de acento. El
  logo se usa también en la cabecera. Las miniaturas ocupan un rect 16:9 y se
  encajan conservando el aspecto, con barras negras para vídeos no 16:9.
- Inicio: logo del SVG del usuario dibujado con shapes de egui (sin decoder
  SVG). Miniaturas: mpv rellena RGB con alfa basura; se fuerza alfa 255 al
  guardar y al cargar caches viejas antes de subir la textura. Las tarjetas
  llevan un botón real con el nombre: `Frame::show` solo percibía hover y el
  clic no abría el proyecto.
- Minimizar quedó confirmado por el usuario: ya no crashea.
- Menús: reducidos a cuatro (Archivo, Edición, Exportar, Ayuda) y a lo
  esencial; sin columna de atajos y con ancho de 160 px (antes 226 px).
- Cabecera: el usuario aclaró que el problema era el centrado **vertical**, no
  el horizontal. Se usa `horizontal_centered` y la referencia de FilmCraft
  (`storytold/filmcraft`, `crates/ui-egui/src/header.rs`): menú en
  `Align::Center`, 12 px entre menús y 16 px entre marca y menú. La sidebar se
  retiró y Preferences pasó a la vista principal.
- Ajustes de layout tras captura del usuario: margen lateral de 48 px en el
  contenido, rejilla con `horizontal_wrapped` (el `Grid` apilaba las tarjetas
  en la app real), más aire entre bloques y Ajustes al pie sobre la versión.
- Minimizar: se suelta la pipeline wgpu cacheada y no se pinta ni se piden
  frames mientras la superficie tiene tamaño 0.
- Revisión final de rama: se corrigieron foco del editor inline, atajos que se
  ejecutaban con modales abiertos, color de identidad en los clips, pista por
  hablante, seek al seleccionar, zoom con `Ctrl+rueda`, waveform al cambiar de
  video y cadenas sin traducir. Quedan como menores: toast que se congela al
  guardar, tres textos de 9/10 px en la regla, navegación `Home`/`End`/`Tab` y
  tooltips de los botones de ventana.
- Pendiente: marquesina y guía visual de `snapping` en el timeline; se conserva
  la lógica de arrastre ya probada.
- Pendiente: errores de `rod-core` y del sistema de archivos siguen en español
  con prefijo traducido; traducirlos exige un enum de errores.
- Pendiente: `panel_visible` se persiste pero falta el control para colapsar y
  reabrir el panel derecho.
- Evidencia: tests de atajos, ajustes, inicio, editor, subtítulos, inspector,
  timeline, transporte, rendimiento y waveform; capturas `finales-*.png` a
  1100/1440/2560 y una pasada ES a 1100. 543 líneas: <8 ms de media en release
  (23 ms en debug).

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

## 2026-11-17 — `PrintWindow` sí captura una ventana en segundo plano

- **Intentaba:** recuperar la evidencia visual para el slice 6, después de que
  el slice 4 diera la captura por imposible.
- **Error literal:** ninguno; el primer intento funcionó. `PrintWindow(hwnd, hdc,
  PW_RENDERFULLCONTENT)` con `SetProcessDpiAwarenessContext(-4)` devolvió la
  ventana de iced/wgpu completa —157 509 píxeles con contenido en una muestra de
  1/9— con la app detrás de otras ventanas.
- **Qué estaba mal antes:** no era la captura, era el método. `CopyFromScreen`
  copia píxeles de la pantalla, y para eso hay que estar encima; `PrintWindow`
  le pide a la ventana que se dibuje a sí misma en un DC, sin importar qué haya
  arriba. La conclusión del slice 4 («no hay captura honesta con el escritorio
  en uso») valía para `CopyFromScreen`, no para esto.
- **Qué aprendí:** separar «no se puede» de «no lo intenté de la otra forma». La
  captura es barata y repetible; lo que sigue sin poder automatizarse es el
  clic: `PostMessage(WM_LBUTTONDOWN)` no llega a winit, así que la interacción
  se prueba por el `update` del `Program` de canvas con un `Cursor::Available`.
- **Dónde quedé:** `docs/checklists/slice-6-timeline.png` como evidencia del
  slice 6a. El script de captura vive fuera del repo
  (`%TEMP%\opencode\capturar.ps1`) porque es herramienta, no código del proyecto.

## 2026-11-19 — `mouse_event` inyecta el clic en la ventana en primer plano

- **Intentaba:** verificar el botón «Recuperar» del aviso de recovery con un
  clic real, ya que `PostMessage` no llega a winit.
- **Error literal:** ninguno. `SetForegroundWindow(hwnd)` no dio error, pero
  `GetForegroundWindow()` siguió devolviendo otra ventana y los dos clics
  cayeron ahí, no en la app.
- **Qué estaba mal:** `mouse_event`/`SendInput` inyecta en el escritorio, en
  coordenadas de pantalla, y el sistema entrega el botón a quien tenga el foco.
  Windows niega `SetForegroundWindow` a un proceso que no está en primer plano
  (regla anti-robo-de-foco), así que sin `AttachThreadInput` o simular Alt no se
  puede activar nuestra ventana; y activarla le quita el foco a lo que el
  usuario esté haciendo.
- **Qué aprendí:** la evidencia de un clic en una GUI se consigue probando la
  lógica (`update`, `Program::update`) o con un estado sembrado que la app pinte
  al arrancar. Inyectar clics en el escritorio de alguien que está usando la
  máquina no es una opción, ni siquiera «un segundo».
- **Dónde quedé:** `docs/checklists/slice-7b-recovery.md`; el aviso se verifica
  con `slice-7b-recovery-aviso.png` y el camino «Recuperar» con los tests de
  `recovery_pendiente` más `cargar_proyecto`.

## 2026-11-23 - `la_pista_de_audio_cambia_en_mpv` dependía del hardware de audio

- **Intentaba:** verificar en CI que el slice 7d no rompiera nada. El job de
  Ubuntu del run 45186579223 llevaba más de una hora `in_progress` sin logs; se
  canceló y se relanzó.
- **Error literal:** `assertion left == right failed`, `left: None`,
  `right: Some(1)` en `crates/player/tests/controles.rs:96`, junto a
  `couldn't open play stream: No such file or directory` y varios errores ALSA
  (`Unknown PCM default`).
- **Qué estaba mal:** el runner de Ubuntu no tiene dispositivo de audio. mpv
  abre el clip con dos pistas, intenta la salida por defecto, falla y
  deselecciona la pista, así que `aid` pasa a `no` (`None`). El test pasó en el
  run anterior y falló en este: era no determinista, no un regreso de 7d ni de
  8. El test comprueba el mapeo de `set_audio_track`/`audio_track` a la
  propiedad `aid`, no el hardware.
- **Qué aprendí:** un test de propiedades de mpv debe fijar su propio entorno.
  Se llama a `mpv().set_property("ao", "null")` dentro del test antes de cambiar
  de pista; con salida nula la selección no depende de ALSA. El run de 8
  (45196981951) ya había pasado en Ubuntu, así que la corrección se verifica
  con un rerun posterior.
- **Dónde quedó:** `crates/player/tests/controles.rs` y esta entrada.

---

## 2026-11-23 — el menú no mostraba recientes ni recovery en la prueba GUI

- **Intentaba:** verificar con una captura que el menú lee `recientes.json` y
  `recovery.json` de un `%APPDATA%` aislado.
- **Error literal:** ninguno. La app arrancaba, pero la columna de recientes
  estaba vacía y no aparecía el aviso; un `eprintln!` temporal decía
  `recientes=0 aviso=false`.
- **Qué estaba mal:** escribí los fixtures directamente en `%APPDATA%`, pero
  `recovery_io::ruta()` y `recientes::ruta_indice()` añaden el subdirectorio
  `RustyOpenDialogue`. El diagnóstico costó tres capturas porque el síntoma
  parecía un fallo de parseo de serde, no una ruta mal compuesta.
- **Qué aprendí:** antes de depurar el contenido de un archivo de configuración,
  imprime la ruta resuelta. Un `eprintln!` de dos líneas separó «no lee» de
  «no parsea» en un solo arranque. El `eprintln!` se quitó antes del commit.
- **Dónde quedó:** `docs/checklists/slice-9-arranque.md` (evidencia y nota de
  alcance), commit `861042f`.

---

## 2026-11-23 — giro visual: minimalismo corporativo tipo Adobe

- **Decisión del autor:** después de ver el mockup brutalista terminado, pidió un
  estilo «más profesional, algo como Adobe» y pasó dos capturas de PhotoCraft
  (arranque y editor). En la elección guiada eligió **gris Adobe + azul
  corporativo** e **inspector híbrido de 440 px con timeline abajo**.
- **Qué cambia:** la chrome deja de ser casi negra y mostaza; pasa a gris medio
  neutro con acento azul `#1473E6`. La zona de video sigue siendo casi negra.
  El titular display de 58 px, las marcas de registro y el crosshair del
  dropzone desaparecen: el arranque se parece al `Reciente` de PhotoCraft
  (barra lateral estrecha + rejilla de tarjetas `.cdp`).
- **Alternativa descartada:** conservar el mostaza. Era fiel a la spec anterior,
  pero el autor pidió explícitamente el registro corporativo con azul.
- **Riesgo conocido:** el azul de chrome puede confundirse con el color del
  hablante 1 (`#4C8DFF`). Mitigación: el azul de chrome solo aparece en CTA,
  foco y pestaña activa; los colores de hablante siguen mandando en carril,
  punto de la lista y `.ass`, y se mantienen literals.
- **Qué NO cambia:** el flujo (drop → proyecto en memoria → `.cdp`),
  el preview = `.ass` exportado, los seis apartados del menú, el copy funcional
  de `arranque-ux.md`, sin sombras, radios ≤ 3 px y cero emojis.
- **Dónde queda:** plan revisado en
  `docs/superpowers/plans/2026-11-23-port-mockup-inicio-editor.md` y capturas en
  `docs/checklists/slice-12-*`.

## 2026-11-23 — detalles FilmCraft: esquinas y SVG

- **Petición del autor:** «le faltaron ciertos detalles de más redondeados y
  svgs», con la captura de FilmCraft y `github.com/storytold` como referencia
  directa.
- **Problema:** la chrome era completamente cuadrada (radio 3 px) y usaba
  glifos de texto (`＋`, `↶`, `×`) con trazos inconsistentes. FilmCraft usa
  superficies de ~6 px y controles de ~4 px, con iconos de trazo 1,7 px en
  rejilla de 24.
- **Alternativa descartada:** copiar los SVG del repo de ArtCraft. Están bajo
  licencia de marca/asset y el proyecto no debe incorporarlos; se dibujaron
  iconos propios con la misma gramática.
- **Decisión:** `RADIO = 4` para controles y `RADIO_TARJETA = 6` para paneles,
  tarjetas y avisos; test que exige que la tarjeta sea más redondeada que el
  control. Iconos SVG en memoria en `iconos.rs`, teñidos con el filtro del
  widget; se habilitó el feature `svg` de iced (misma dependencia, sin crate
  nueva). Timeline con tira de herramientas funcional (zoom, dividir, buscar).
- **Regla rota a propósito:** la spec antigua fijaba radios ≤ 3 px; ahora el
  techo es 6 px para superficies. Sin sombras ni gradientes.

## 2026-11-23 — cabecera propia y nombre RustyDialogue

- **Petición del autor:** quitar la barra nativa de Windows, hacer la cabecera
  en código, renombrar la app a `RustyDialogue` y arreglar el texto de la
  derecha sin padding y los objetos descentrados.
- **Problema:** `iced` dibujaba la barra del sistema con el título largo; el
  estado de la derecha quedaba pegado al borde, y el contenido de los botones
  de recientes se centraba dentro de la tarjeta en vez de alinearse a la izquierda.
- **Alternativa descartada:** conservar la barra nativa y dejar el título vacío.
  Seguía ocupando 30 px y no permitía los controles SVG de la misma familia que
  el resto del chrome.
- **Decisión:** `.decorations(false)` en el builder; cabecera de 30 px con
  marca, menú, zona arrastrable, estado y botones minimizar/maximizar/cerrar.
  Las acciones usan `window::latest().and_then(...)` (sin `unsafe` ni crates
  nuevas). El contenido de cada tarjeta va dentro de un `container` de 196 px
  para fijar la alineación izquierda; la cabecera gana 10 px de padding lateral.
- **Verificación:** captura `slice-14-inicio-sin-barra.png` (ventana 1600×900,
  sin barra nativa). Los clics en los botones de ventana no se automatizaron:
  `mouse_event` afecta al proceso en primer plano y aquí solo se verifica con
  `PrintWindow`, limitación ya documentada más arriba.

### Reversión del mismo día

- **Resultado:** el arrastre no funcionó. El mensaje `ArrastrarVentana` sí llegaba
  al `update`, pero `window::drag` no movía la ventana. El autor pidió volver a
  la barra nativa; se restauró `.decorations(true)` y se quitaron los mensajes
  y botones de ventana. Se conservan el nombre `RustyDialogue`, el padding del
  estado y la alineación de las tarjetas.
- **Causa probable:** se usó un `button` como zona de arrastre y el id se
  resolvía de forma asíncrona con `window::latest()`. `drag_window()` de winit
  publica `WM_NCLBUTTONDOWN` con el cursor actual y exige que el botón izquierdo
  esté recién presionado: el `button` captura el ratón y la resolución
  asíncrona rompe esa condición. La instrumentación con `mouse_event` tampoco
  fue fiable mientras el autor usaba el equipo, así que la causa quedó como
  hipótesis, no como hecho confirmado.
- **Guía encontrada (para retomarlo con control del ratón):**
  - iced issue #759, comentario de `tsuza`: «Use a `MouseArea` and then use
    `window::drag` when you're inside it» — https://github.com/iced-rs/iced/issues/759
  - iced 0.14.2 ya trae `iced::widget::MouseArea` (en `iced_widget`), con
    `on_press`; es el patrón que falta probar frente al `button`.
  - `iced-window-chrome` tiene el ejemplo `custom_titlebar` y documenta el
    coste real: al quitar el marco se pierden redimensionado y snap de
    Windows, y hay que dibujar handles invisibles — https://lib.rs/crates/iced-window-chrome
  - `toymil` en el mismo issue dice que CSD funciona bien en `master` y avisa
    de un bug con dos `MouseArea` superpuestos (resize debajo de drag).
- **Decisión:** no añadir `iced-window-chrome` ni volver a intentarlo sin poder
  manipular el ratón de forma controlada. Si se retoma: `MouseArea` + id de
  ventana guardado en el estado en el `boot`, y handles de resize.

### Reintento con control del ratón (éxito)

- El autor confirmó que el arrastre funciona con `MouseArea`. La causa del
  fallo anterior era el `button` como zona de arrastre y/o resolver el id de
  forma asíncrona; con el id guardado en `AppState` desde el `boot` y
  `MouseArea::on_press`, `window::drag` se pide en el mismo evento.
- Se añadieron cuatro franjas invisibles con `MouseArea` +
  `window::drag_resize` para recuperar el redimensionado sin marco nativo, y
  botones SVG de minimizar/maximizar/cerrar. Maximizar consulta
  `window::is_maximized` para alternar.
- **Fuente:** se cambió la tipografía por `Segoe UI Variable` (cuerpo, display
  black y seminegrita) y `Cascadia Mono` para etiquetas en Windows. En otros
  sistemas se cae a las fuentes por defecto de iced: no se distribuyen archivos
  ni licencias nuevas. Se aplicó con `.default_font(tema::CUERPO)`.
- Captura: `slice-15-inicio-fuente.png`.

---

## 2026-11-24 — migración del front-end a egui

- **Petición del autor:** iced tomó mucho trabajo para la interfaz; egui ofrece
  mejores capacidades de UI y la base de los proyectos ArtCraft sirve como
  referencia fuerte. Se mantiene el renderer propio con libass.
- **Decisión:** egui solo como front-end. Se descarta `egui-sharkplayer`: usa
  `libmpv2` 6.0 (aquí 7.0.1), crea otro contexto mpv y con `wgpu` hace un
  roundtrip del frame por RAM. `rod-player` y `rod-subs` siguen mandando.
- **Alternativa descartada:** egui 0.35 (wgpu 29) obligaría a reescribir
  `video/mod.rs`. Se ancla egui 0.33 porque `egui-wgpu` 0.33 depende de
  `wgpu 27.0.1`, la versión ya fijada en `Cargo.lock`. Los `.wgsl` no cambian.
- **Decisión:** la composición sigue en GPU mediante un `egui_wgpu::CallbackFn`,
  no en CPU con `Lienzo`: duplicaría el compositor y cambiaría el resultado de
  píxel sin ventaja.
- **Alcance:** `egui_dock` queda fuera; los paneles siguen fijos. El trabajo se
  hace en el worktree `C:/Dev/RustyOpenDialogue-egui` para no tocar los 7
  archivos modificados que quedaron en `main`.
- **Dónde queda:** `docs/superpowers/specs/2026-11-24-egui-migracion-design.md`.

### Cierre del slice 16

- **Problema:** la primera captura falló porque `FontFamily::Name("Segoe UI
  Variable")` no estaba registrado en epaint y el app entraba en pánico al
  dibujar el primer texto.
- **Alternativa descartada:** cargar la fuente del sistema y registrar sus
  bytes; añade un asset y su licencia para un cambio que no afecta a la
  funcionalidad.
- **Decisión:** usar `FontFamily::Proportional` y `Monospace`, que egui ya trae.
  Coste: se pierde la tipografía corporativa específica en Windows; la app
  arranca. Queda como mejora pendiente si se registra la fuente.
- **Evidencia:** `docs/checklists/slice-16-inicio-egui.png`, captura real de la
  ventana de arranque a 1600×900. El editor con video y Linux siguen pendientes.

