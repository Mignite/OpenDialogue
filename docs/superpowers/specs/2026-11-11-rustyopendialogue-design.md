# RustyOpenDialogue — diseño

Fecha: 2026-11-11
Estado: borrador para revisión
Origen: `C:/Dev/OpenDialogue` (Tauri + React, app de referencia y oráculo)

## 1. Objetivo

Reescribir OpenDialogue como aplicación nativa en Rust, para aprender Rust de
verdad. No es un proyecto de producto: es un proyecto de aprendizaje con un
resultado usable.

Criterio de éxito:

1. La app nativa edita subtítulos multi-hablante con la misma fidelidad que la
   actual: cargar SRT/ASS, asignar hablantes, editar textos y tiempos, exportar
   `.ass` y `.srt`.
2. El preview coincide con lo que muestra mpv, porque usa el mismo rasterizador.
3. El código es entendible y mantenible **sin asistencia de IA**: cada slice
   tiene su especificación, sus tests y una razón documentada.

Fuera de alcance: paridad de features con la app Tauri desde el día uno,
multiplataforma de escritorio más allá de Windows/Linux, y cualquier IA local
(se eliminó del proyecto original en 2026-09-20).

## 2. Restricciones

- **Poca asistencia de IA en el código.** La IA escribe specs, diseños y
  revisiones; el código lo escribe el humano. Consecuencia: slices pequeños, con
  criterio de salida verificable y tests antes de implementación.
- **Windows para desarrollar, Linux (CachyOS) para probar.** Ambas plataformas
  deben compilar desde el slice 0, no al final.
- **Tiempo parcial.** El proyecto avanza por slices cerrados; ninguno deja el
  repo en un estado que no compile.
- **La app Tauri no se toca.** Sirve de oráculo: sus 306 tests y sus salidas son
  la especificación del comportamiento a portar.

## 3. Decisiones

### 3.1 Toolkit: iced 0.14

Motivos:

- El corazón de esta app es dibujo custom: waveform por píxel, carriles por
  hablante, playhead, marquee y drag. iced tiene dos capas de primera clase para
  eso: `Canvas` (vectorial) y `shader::Primitive` (acceso a `device`/`queue` en
  `prepare`, draw calls en `render`).
- La arquitectura Elm (`State` / `Message` / `update` / `view`) mapea 1:1 con el
  reducer que la app ya tiene.
- 0.14 trajo tests end-to-end, modo headless y el debugger `comet` con
  time-travel: valioso trabajando sin IA.
- MIT, Rust puro, un binario.

Costos aceptados, verificados:

- La documentación es escasa por decisión del autor y la API cambia en cada
  release. Se ancla a `iced = "0.14"` y no se persigue `master`; los ejemplos se
  leen del tag de la release.
- No hay widget de video, ni menú nativo, ni capa de accesibilidad. Se
  construyen (video) o se usan crates externas (menú, diálogos).

Alternativas descartadas, con motivo:

| Alternativa | Por qué no |
|---|---|
| egui | Techo estético de herramienta; el timeline sí sería fácil. Se conserva como plan B si el video en iced resulta inviable. |
| slint | La UI viviría en un DSL: la mitad del proyecto dejaría de ser Rust. Licencia GPLv3 o royalty-free con atribución. |
| gpui | Docs casi inexistentes y API en movimiento. Buen destino futuro, mal punto de partida. |
| freya / makepad / xilem | Riesgo alto de callejón sin salida. |

### 3.2 Video: libmpv, render por software

`libmpv2` (7.x) cubre decodificación, decodificación por hardware, sincronía A/V,
seek exacto al frame (`hr-seek=yes`), velocidad con corrección de pitch y salida
de audio. Es exactamente lo que no queremos reescribir.

Camino elegido: `MPV_RENDER_API_TYPE_SW` → RGBA en memoria → `queue.write_texture`
→ textura de wgpu.

- El render se hace a la **resolución nativa del video**, sin escalar; la escala
  a la ventana la hace la GPU. El propio `render.h` avisa que el SW render es
  lento justamente porque escala en CPU: evitando la escala, el costo queda en la
  conversión de color.
- Aviso textual de la fuente: *"very slow, because everything, including color
  conversion, scaling, and OSD rendering, is done on the CPU, single-threaded"*.
  Es un riesgo medido en el slice 0, no una opinión.

Reparto de crates, corregido tras revisar el código (ver `BITACORA.md`):

- `libmpv2` (7.0.1) para el ciclo de vida del player, comandos, propiedades y
  eventos.
- Su enum `RenderParam` **solo tiene variantes de OpenGL**, así que el contexto
  de render por software se escribe sobre `libmpv2-sys` (4.0.1: bindings
  pregenerados, sin libclang, enlaza con `cargo:rustc-link-lib=mpv`). Ese módulo
  `mpv_render` es el único `unsafe` del crate, y expone una API segura.
- Ojo: `libmpv-sys` en crates.io es de otro autor y está muerto (2020). El
  correcto es `libmpv2-sys`.

En Windows hace falta un import library: `lib /DEF:mpv.def /OUT:mpv.lib
/MACHINE:X64`, y `libmpv-2.dll` junto al ejecutable.

Descartados:

- **GL render + FBO**: rápido, pero exige compartir contexto OpenGL con el wgpu
  de iced. En Windows wgpu va por DX12/Vulkan y no hay interop de FBO GL sin
  forzar el backend GL (el menos soportado de wgpu).
- **GStreamer** (`iced_video_player`, `gstreamer-rs`): reimplementar el player no
  es el problema; el problema es el runtime. El AppImage de la app Tauri ya
  crasheaba por desajuste de versión entre el core de GStreamer embebido y los
  plugins del host (registrado en su AGENTS.md). No se reintroduce.
- **ffmpeg-next + cpal**: escribir el reloj de audio, el pacing y el seek a mano
  es reimplementar lo general. Va contra la razón de existir del ecosistema de
  crates.

Licencia: libass es ISC. mpv: el build oficial para Windows es GPL, así que
distribuir la app con ese binario la arrastra a GPL. Compilando libmpv se queda
en LGPL y el enlace dinámico es suficiente. Decisión pendiente al momento de
empaquetar; no bloquea el desarrollo.

### 3.3 Subtítulos: libass directo, no mpv

mpv no renderiza ASS: usa libass. Por eso "100 % igual a mpv" se consigue con
libass en proceso, sin cederle el overlay al player.

Invariante central del proyecto:

> **El preview es un render del `.ass` que se exporta.**

Se construye el `.ass` con `buildAss` (portado de `src/utils/ass.ts`), se pasa a
libass con `ass_read_memory` y se rasteriza al tiempo actual. No hay un segundo
motor que reconciliar, no hay aproximación CSS y desaparece `ASS_EM_PREVIEW`
(0.85), que existía solo para calibrar HTML contra una captura de mpv.

El `.ass` del preview se reconstruye **del proyecto** en cada edición
(`crates/app/src/preview.rs`) y se recarga en el `Subtitulos` ya existente
(`Subtitulos::cargar`). Editar texto no toca libass hasta confirmar la edición:
un historial con una entrada por tecla no es lo que el oráculo guarda.

Por qué no dejar que mpv pinte los subtítulos:

- Se pierde el control del overlay: resaltar el clip activo, aislar un hablante,
  mostrar un solape. La única palanca sería reescribir un `.ass` temporal y
  llamar `sub-reload`, que recarga de forma asíncrona y fuera del orden de render.
- El default de mpv no es "tal cual": `--sub-ass-override` vale `scale`, es
  decir que ya aplica `--sub-scale` a los estilos ASS. Haría falta `=no`.

Fidelidad con el default de mpv: `--sub-ass-scale-with-window` es `no` para ASS,
o sea que libass rasteriza al tamaño del video y luego todo se escala junto con
la ventana. Nuestro pipeline (rasterizar a tamaño de video, componer, escalar el
compuesto) es ese mismo comportamiento.

Detalle verificado: libass trae `ass_directwrite.c`. En Windows resuelve fuentes
por DirectWrite y en Linux por fontconfig, lo que reemplaza la heurística de
`src/utils/fuentes.ts` y el `listar_fuentes_sistema` por PowerShell/GDI. El aviso
de "fuente no instalada" pasa a ser el log callback de libass
(`ass_set_message_cb`).

Las crates existentes están abandonadas (`libass` 0.2.0, 2020). Lo que se escribe
es la **declaración** del API: cuatro tipos opacos y unas quince funciones
(`ass_library_init`, `ass_renderer_init`, `ass_set_frame_size`, `ass_set_fonts`,
`ass_read_memory`, `ass_render_frame`, …), a mano y no con bindgen, porque el API
cabe en una pantalla y `extern "C"` a mano es material de aprendizaje real.
Bindgen solo si el API crece. El rasterizador sigue siendo libass: no se
reimplementa nada.

### 3.4 La línea entre crate y código propio

Regla del proyecto:

- **General → crate.** Decodificar, sincronizar, buscar, rasterizar ASS,
  decodificar fuentes, parsear JSON. Si existe y está viva, se usa.
- **Propio → Rust escrito aquí.** La semántica de subtítulos: overrides por línea
  dentro de un evento fusionado, hablante→estilo, segmentación por solape, islas
  de audio, el modelo de edición y su undo. No existe como crate porque es el
  producto.

## 4. Arquitectura

Workspace de cuatro crates. El directorio dice el papel; el nombre del paquete
va prefijado `rod-`, porque un paquete llamado `core` **tapa el `core` del
sysroot** en todo crate que dependa de él (`core::time::Duration` deja de
existir y las macros derivadas fallan al expandir `::core::fmt`):

```
RustyOpenDialogue/          (manifiesto virtual: solo [workspace])
  crates/core/     rod-core     dominio puro: tiempo, SRT, ASS, islas de audio
  crates/subs/     rod-subs     FFI de libass + caché de bitmaps + composición
  crates/player/   rod-player   libmpv: transporte, frames, reloj
  crates/app/      rod-app      iced: update/view, timeline, paneles, IPC
```

### 4.1 `rod-core`

Sin dependencias de UI ni de sistema. Portado desde `src/utils/*.ts`:

- `time`: `formatTime`, `parseTimeInput`.
- `srt`: `parse_srt`, `build_srt`, `format_srt_timestamp` (con rollover de ms).
- `ass`: `hex_to_ass_color` / `ass_color_to_hex` (BGR invertido), tiempos en
  centisegundos, escape/unescape (backslash y llaves primero, `\n` al final),
  `segmentar_por_solape`, `overrides_de_linea`, `fusionar_lineas`, `build_ass`,
  `parse_ass` (hablantes ordenados por las líneas `Style:`, no por los eventos;
  continuaciones de `Dialogue:` unidas; `Comment:`/`Picture:` no las extienden).
- `islands`: `buscar_tramo_audio` (percentil 25 del fondo, onset hacia atrás,
  cola de 0.25 s, sin piso de duración), `pico_en_rango`.
- `model`: `Proyecto`, `Caption`, `Hablante`, `Preset` con `presetId`.

Los 306 tests de vitest son la especificación. Se portan uno a uno; cuando un
comportamiento esté documentado como limitación (por ejemplo, que importar un
evento fusionado pierde los colores por línea), se porta el test que la fija.

### 4.2 `rod-subs`

`Renderer` envuelve el ciclo de libass:

```
ass_library_init → ass_renderer_init
  ass_set_frame_size(video_w, video_h)
  ass_set_storage_size(coded_w, coded_h)   // SAR ≠ 1
  ass_set_pixel_aspect(sar)
  ass_set_margins(0,0,0,0); ass_set_use_margins(0)
  ass_set_fonts(renderer, NULL, "sans-serif", ASS_FONTPROVIDER_AUTODETECT, NULL, 1)
track = ass_read_memory(ass_text)
imgs  = ass_render_frame(track, time_ms, &changed)
```

Cada `ASS_Image` trae `w`, `h`, `stride`, `bitmap` (cobertura 8-bit), `color`
(RGBA, alfa global), `dst_x`, `dst_y`. La composición es
`dst = rgb·(a·A/255) + dst·(1 − a·A/255)`.

Al editar: se reconstruye el `.ass` y se reemplaza el track
(`ass_free_track` + `ass_read_memory`). Es barato porque el render vive sobre un
string en memoria, no sobre un archivo.

El `detect_change` de `ass_render_frame` evita re-subir bitmaps cuando el tiempo
no cambió.

### 4.3 `rod-player`

```rust
pub struct Frame { pub width: u32, pub height: u32, pub rgba: Vec<u8>, pub pts: f64 }

pub trait VideoSource {
    fn poll_frame(&mut self) -> Option<Frame>;  // None si no hay frame nuevo
    fn position(&self) -> f64;                  // reloj autoritativo (time-pos)
    fn seek(&mut self, t: f64);
    fn set_speed(&mut self, s: f64);
    fn play(&mut self);
    fn pause(&mut self);
    fn duration(&self) -> f64;
}
```

El trait existe por **testabilidad**: un `FakeSource` con frames deterministas
permite los tests E2E de iced sin abrir un video. No es un plan para añadir
ffmpeg; si algún día se añade, el trait ya está.

**Audio: lo reproduce mpv, no lo reimplementamos.** Decodificar, sincronizar y
sacar el sonido es exactamente el trabajo que el ecosistema de crates ya resuelve
(§3.2: por eso `ffmpeg-next + cpal` está descartado). El player solo expone:

```rust
pub struct PistaAudio {
    pub id: i64,                   // el número de `aid`; es lo que se escribe
    pub idioma: Option<String>,    // "spa", "eng"…
    pub titulo: Option<String>,    // nombre del track en el contenedor
    pub canales: Option<u32>,
}

fn pistas_audio(&self) -> Vec<PistaAudio>;   // de `track-list`, filtrando type == "audio"
fn pista_audio(&self) -> Option<i64>;        // `aid` efectivo, ya resuelto por mpv
fn set_pista_audio(&self, id: i64);          // `aid`; `auto` lo elige solo
```

Los tres piden `&self`: libmpv muta su propio estado por dentro.

**Una pista a la vez.** `aid` acepta un solo ID (`auto`/`no`/número), así que
"pista seleccionable" es un `enum` de N variantes, no una lista de casillas. Oír
varias a la vez exigiría abandonar `aid` y montar `lavfi-complex` con
`amix`: mientras el grafo existe la selección normal deja de mandar, cada cambio
reconstruye el grafo entero y el volumen por pista pasa a ser un filtro dentro
del grafo. No compensa para subtitular: se descarta (decisión, no pendiente).

**El reloj.** Con audio, mpv sincroniza contra la salida de audio
(`video-sync=audio`), que es lo correcto para lip-sync y lo que queremos al
reproducir. Para stepping frame a frame se pausa y se usa `frame-step`, con el
reloj de video. En ambos casos el playhead de la app sigue siendo `time-pos`.

Configuración de mpv: `vo=libmpv`, `hwdec=auto-safe`, `hr-seek=yes`,
`audio-pitch-correction=yes`, `keep-open=yes`, `config=no`,
`input-default-bindings=no`, `osc=no`, sin pistas de subtítulos. **Sin `ao=null`**:
ese fue un ajuste del slice 0 para aislar la medición de video, no del player.

El playhead de la app es `time-pos` observado, no un temporizador: el snapping y
el seguimiento del timeline se mueven con el reloj real del player.

### 4.4 `rod-app`

- Un `shader::Primitive` compone **video + subtítulos** en un solo pipeline: un
  quad para el frame, N quads para los bitmaps de libass. `prepare` recibe
  `device` y `queue`, así que ahí se suben la textura del video y las de cobertura.
- **El canvas de composición es 1920×1080 fijos, no la resolución del video.**
  El `.ass` exportado lleva `PlayResX/Y: 1920 1080` (el oráculo arranca así), y
  el invariante del proyecto es que el preview sea el render de ese `.ass`. Si
  el buffer siguiera la resolución del video, un clip 4:3 o 4K sacaría los
  subtítulos de sitio respecto al archivo exportado. El letterbox hasta el
  tamaño de la ventana lo hace el quad del shader
  (`crates/app/src/video/geometria.rs`), no el buffer.
- El timeline es un `Canvas` (vectorial): waveform, carriles, clips, playhead,
  marquee. Si el profiling muestra que la waveform cuesta, se mueve a un
  `shader` con una tira de quads; el resto del timeline se queda en `Canvas`.
  Sin `Cache` mientras sean unos cientos de formas: un cache de canvas se
  invalida a mano en cada edición y en cada tick del playhead, y ese riesgo no
  paga el ahorro. La decisión se revisa cuando entre la waveform, con una
  medición en la mano.
- La navegación del eje vive en `Vista` y tiene una sola puerta: `con_inicio`
  acota a `[0, duracion - ancho_visible]`. El canvas no interpreta la rueda,
  solo publica líneas y el punto local (su evento no trae modificadores, así que
  el «Shift = zoom» lo decide la app, que espeja `ModifiersChanged`). El
  seguimiento del playhead interpola el 12 % del camino por frame y se apaga en
  cuanto el usuario mueve el eje a mano.
- Los arrastres del timeline **recalculan desde la posición original** guardada al
  pulsar, en vez de acumular el delta de cada frame: acumular se desvía por los
  `f64`, por los clamps y por el snap, y soltar donde se pulsó dejaría de ser un
  no-op exacto. Las marcas se cuantizan a milisegundos, que es lo que leen SRT y
  ASS. Un arrastre es un solo paso de historial: la instantánea se toma al
  pulsar y se registra en el primer frame que cambia algo.
- Mientras se reproduce, una `Subscription` a los frames de la ventana fuerza el
  redibujado; en pausa, el render es reactivo (0.14) y solo se redibuja al
  cambiar estado.
- Los paneles (lista de subtítulos, hablantes, estilos, editor) son widgets
  normales. La virtualización de la lista no es necesaria al principio: con
  `iced` el costo es menor que en React y se mide antes de optimizar.
- **La barra de transporte se hace de una sola vez**: play/pause, seek y
  selector de pista de audio en un mismo `Row`, encima o debajo del video. Los
  tres leen y escriben el mismo estado del player (`pause`, `time-pos`, `aid`),
  así que probarlos por separado no dice nada: o el conjunto funciona o el
  reproductor no está listo. Los controles restantes (velocidad, volumen,
  stepping) se cuelgan del mismo `Row` después, sin cambiar el diseño.

### 4.5 Flujo de un frame

```
mpv (decodifica, sincroniza, suena)
  └─ poll_frame() → RGBA + pts
       ├─ upload a textura (video)
       └─ time_ms = position()·1000
            └─ libass ass_render_frame → ASS_Image[]
                 └─ upload de coberturas
                      └─ shader::Primitive: video quad + N quads alfa
                           └─ iced compone la ventana (wgpu)
```

## 5. Manejo de errores

- `core`: errores tipados con `thiserror`; nunca `panic!` por entrada del usuario
  (un SRT mal formado devuelve `Err`).
- `subs`: si libass no encuentra una fuente, se reporta por el log callback a la
  barra de estado, no se aborta.
- `player`: fallo de apertura de video o de DLL → error tipado que la UI muestra.
- `app`: todo error de usuario termina en la barra de estado con mensaje; nunca en
  `console.error` silencioso (lección de la app original, donde 7 errores de
  guardado eran invisibles).

## 6. Testing

| Nivel | Estrategia |
|---|---|
| `core` | Port 1:1 de los 306 tests de vitest. Son la especificación. |
| `subs` | Golden frames: se renderiza un `.ass` de fixture y se compara contra un PNG generado por mpv a los mismos tiempos, con tolerancia por porcentaje de píxeles distintos. Se registra la versión de libass: no hay comparación exacta entre versiones. |
| `player` | `FakeSource` para todo lo que no sea decodificar; una prueba de humo con un video real de 3 s generado con ffmpeg en CI. |
| `app` | Tests E2E de iced 0.14 contra `FakeSource`: abrir, editar, undo, exportar. |
| Export | El `.ass` exportado se carga en un mpv real (`mpv --no-config --vo=image --sub-file=x.ass --start=T --frames=1`) y se compara con el render propio. Es el test que sostiene el invariante del preview. |

## 7. Riesgos

| Riesgo | Impacto | Mitigación |
|---|---|---|
| SW render de mpv no da fps suficientes | Alto | Medir en el slice 0 a 1080p; si falla, plan B es ffmpeg + wgpu (el trait ya lo permite) o egui, no GL interop con iced |
| Construir libass en Windows (freetype, fribidi, harfbuzz) | Alto | vcpkg tiene port de libass; alternativa, DLL precompilada. Se valida en el slice 0 antes de escribir UI |
| API de iced cambia entre releases | Medio | Anclar 0.14, no seguir `master`, actualizar por decisión y no por inercia |
| Docs insuficientes de iced | Medio | Leer el código fuente y los ejemplos del tag; los slices pequeños hacen que un bloqueo sea local |
| El proyecto se estanca por tamaño | Alto | Slices cerrados y verificables; la app Tauri sigue funcionando y es el oráculo |

## 8. Slices

Cada slice termina con `cargo build`, `cargo fmt --check`, `cargo clippy -D
warnings` y `cargo test` en verde, en Windows y en Linux.

- **Slice 0 — bootstrap y prueba de fuego.** Workspace con los cuatro crates, CI,
  ventana de iced con un `Canvas` que dibuja algo. libmpv y libass enlazados;
  un frame de video y un `.ass` de fixture renderizados a pantalla. **Criterio de
  salida:** se ve un frame con subtítulos y se mide el costo por frame a 1080p.
  Se descompone en `slice-0z-bootstrap.md` (workspace y ventana), 0a (mpv),
  0b (libass) y 0c (ventana con ambos).
- **Slice 1 — `core`.** Port de `time`, `srt`, `ass`, `islands` con sus tests.
- **Slice 2 — `subs`.** Render estático de un `.ass` a PNG y comparación contra el
  oráculo de mpv.
- **Slice 3 — `player`.** Play/pause/seek/velocidad, selección de pista de audio
  (una a la vez, `aid`), `time-pos` como playhead, frames a textura. La barra de
  transporte de iced (play/pause + seek + pista de audio, juntos) se valida en el
  slice 0c paso 6 con el mismo player, para que el conjunto esté probado antes de
  la UI real.
- **Slice 4 — compositor.** Video + subtítulos con escala y letterbox correctos,
  sobre un canvas de 1920×1080 (el `PlayRes` del `.ass` exportado, §4.4), no el
  tamaño del video.
- **Slice 5 — edición.** Lista de subtítulos, panel de hablantes, editor de texto,
  undo, atajos.
- **Slice 6 — timeline.** Carriles por hablante, clips, drag de bordes y en
  bloque, marquee, snap, zoom y pan.
- **Slice 7 — proyecto.** Guardar/cargar JSON, importar SRT y ASS, exportar SRT y
  `.ass` con presets.
- **Slice 8 — empaquetado.** NSIS en Windows, `.deb`/AppImage en Linux, con las
  DLLs de libmpv y libass.

Orden propuesto: 0 y 1 en paralelo (el 0 cubre el riesgo grande mientras el 1 da
progreso diario sin dependencias), después 2 → 3 → 4, que dan el primer vertical
usable.

## 9. Preguntas abiertas

- Licencia del binario de libmpv en las releases (GPL del build oficial vs LGPL
  compilado). No bloquea; se decide en el slice 8.
- Si el timeline en `Canvas` aguanta 2 h de video con waveform densa, o si la
  waveform se muda a un `shader`. Se responde con medición en el slice 6.
- Tope de resolución para el SW render en videos 4K. Renderizar a resolución
  nativa replica el comportamiento de mpv, pero puede no dar tiempo real; la
  decisión sale de la medición del slice 0, no de una corazonada.
- Formato del proyecto: mantener el JSON actual (compatible con la app Tauri) o
  uno nuevo. Preferencia inicial: mantenerlo, para poder comparar salidas.
