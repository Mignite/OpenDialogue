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
