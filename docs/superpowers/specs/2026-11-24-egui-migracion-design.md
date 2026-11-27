# RustyDialogue — migración del front-end a egui

Fecha: 2026-11-24
Estado: aprobado en charla, pendiente de revisión del documento
Origen: decisión del autor tras comparar con FilmCraft (`storytold/filmcraft`)

## 1. Objetivo

Sustituir el front-end de iced por egui para mejorar las capacidades de UI y
reducir la fricción de desarrollo, **sin tocar el motor**. FilmCraft y los demás
proyectos ArtCraft se usan como referencia de patrones de interfaz, no como
dependencia ni como fuente de código copiado.

Criterio de éxito:

1. La app abre, edita y exporta igual que hoy (mismo oráculo: app Tauri).
2. El preview sigue siendo el render del `.ass` que se exporta.
3. El video y los subtítulos se componen con el mismo pipeline GPU y el mismo
   resultado de píxel que antes de la migración.
4. Cada slice compila y pasa build + fmt + clippy + tests en Windows y Linux.

Fuera de alcance: `egui_dock`, paneles acoplables, cambio de formato de
proyecto, cambio de reproceso de audio, y adoptar un widget de video de terceros.

## 2. Decisión y motivos

### 2.1 egui solo como front-end

Problema: iced exigió mucho trabajo para acercarse al resultado visual deseado
(slices 12–15: cabecera propia, iconos, tipografía) y el autor lo considera un
crate menos maduro para esta interfaz.

Alternativa descartada: `egui-sharkplayer` (0.5.1), que es el crate de video
egui+libmpv que motivó inicialmente el cambio. Se descarta porque usa
`libmpv2` 6.0 (aquí usamos 7.0.1), crea su propio contexto mpv, y recomienda el
backend `glow`; con `wgpu` hace un roundtrip del frame por memoria RAM. Aporta
un segundo player que no necesitamos y no resuelve la composición con libass.

Decisión: egui se encarga solo de widgets, eventos y estilo. `rod-player` y
`rod-subs` siguen siendo el player y el rasterizador.

### 2.2 Versión anclada: egui 0.33

Problema: la versión más reciente de egui (0.35) depende de `wgpu 29`, mientras
que `Cargo.lock` fija `wgpu 27.0.1` a través de iced 0.14.

Alternativa descartada: actualizar a egui 0.35 y migrar `video/mod.rs` a la API
de wgpu 29. Es trabajo extra sin beneficio para la app.

Decisión: `eframe = "=0.33.0"` con feature `wgpu`, que trae `egui 0.33` y
`egui-wgpu 0.33`, ambos sobre `wgpu 27.0.1`. Se declara `wgpu = "=27.0.1"` en
`rod-app` para que el pipeline y `egui-wgpu` comparten exactamente el mismo tipo.
Los archivos `frame.wgsl` y `subs.wgsl` no se modifican.

### 2.3 Composición GPU, no CPU

Problema: egui pinta texturas RGBA con `Painter::image`, que no ofrece los dos
draw calls con blending alfa que hoy componen video + bitmaps de libass.

Alternativa descartada: componer en CPU con `rod_subs::composicion::Lienzo` y
subir un único buffer RGBA a una textura de egui. Duplica un compositor que ya
existe, cuesta ~8 MB por frame a 1080p y cambia el resultado de píxel respecto
al pipeline actual.

Decisión: portar `Gpu` a un `egui_wgpu::CallbackFn`. `prepare` recibe
`device`/`queue` y sube la textura del frame, el uniform de letterbox y el atlas
de máscaras; `paint` recibe el `RenderPass` y ejecuta los mismos draw calls.
Las structs `Frame` y `Capa` y `video/geometria.rs` se conservan.

### 2.4 Referencia, no copia

Los proyectos ArtCraft están bajo licencias MIT/Apache-2.0. Se estudian sus
patrones (distribución de paneles, tema, iconos, atajos) y se escriben aquí con
nuestra propia gramática visual. No se incorporan sus assets ni su código.

## 3. Arquitectura

### 3.1 Crates

El workspace sigue siendo el mismo, con `rod-app` como único front-end.

Intactos: `rod-core`, `rod-subs`, `rod-player`.
Intactos dentro de `rod-app`: `preview.rs`, `project_io.rs`, `presets_io.rs`,
`recientes.rs`, `recovery_io.rs`, `comandos.rs` y `video/geometria.rs`.

Reescritos: `main.rs`, `editor_app.rs`, `inicio.rs`, `menu.rs`, `tema.rs`,
`timeline.rs`, `video/mod.rs`. Se crea `crates/app/src/ui/` para los view, de
modo que la lógica pura siga en los módulos actuales y ningún archivo vuelva a
concentrar 2500 líneas de interfaz.

### 3.2 Estado y flujo

Hoy: `Mensaje` → `actualizar(state, mensaje) -> Task<Mensaje>` → `view(state)`.

Mañana: `impl eframe::App for RustyDialogue`, con `AppState { pantalla: Inicio |
Editor }`. `update()` lee el input de egui, llama a la misma lógica pura
(`EstadoEditor`, `inicio::update`, `comandos::desde_keyboard`) y pinta con
`egui::Ui`. No hay `Task` ni `Subscription`.

- Diálogos y E/S: `rfd` síncrono congelaría el frame. Se abre en un hilo corto
  y el resultado se recoge por un canal `std::sync::mpsc` que se drena al
  empezar el frame siguiente. Es el sustituto del `Task` de iced.
- Player: `poll_frame()` se llama una vez por frame; si hay frame nuevo se
  actualiza el `Frame` guardado en el estado. Mientras haya video activo se
  pide `ctx.request_repaint()`, sustituyendo la `Subscription` de frames.
- El reloj sigue siendo `time-pos` de mpv. El snapping y el seguimiento del
  playhead usan el mismo valor.

### 3.3 Video y subtítulos

`preview::ass_del_proyecto(editor, presets)` sigue siendo la única función que
produce el `.ass`; la exportación escribe esa misma cadena. `rod-subs` rasteriza
ese texto a bitmaps y `video::Capa` copia las máscaras. El callback de
`egui_wgpu` las compone sobre el frame con el mismo blending.

Único cambio matemático: iced aplicaba un viewport por primitivo, así que el
quad centrado en clip space ya quedaba dentro del widget. En egui el callback
recibe el rect del widget y debe convertir a píxeles de pantalla usando
`pixels_per_point` y `screen_size_px` de `PaintCallbackInfo`. La escala de
letterbox la sigue dando `geometria::escala_letterbox`.

### 3.4 Timeline y paneles

- Timeline: widget inmediato con `egui::Painter`: waveform, carriles por
  hablante, clips, playhead, marquee y eje con zoom/pan. El estado del arrastre
  vive en el app y recalcula desde la posición guardada al pulsar, como en
  `timeline/arrastre.rs`. Se conservan `Vista::con_inicio` y el snap de `rod-core`.
- Paneles: `TopPanel` para marca, menú, estado y botones de ventana;
  `SidePanel` derecha con las pestañas subítulos/hablantes/estilos;
  `CentralPanel` para video y transporte; timeline abajo.
- `egui_dock` no entra en este diseño. Es un cambio de alcance y la app actual
  usa paneles fijos.
- Menú: egui no trae barra de menú nativa en todas las plataformas; se dibuja
  con los widgets de menú de egui. Los items siguen produciendo `Comando`.

### 3.5 Estilo

`tema.rs` pasa a construir `egui::Style` y `egui::Visuals`: gris Adobe neutro,
acento azul `#1473E6` solo en CTA/foco/pestaña activa, radios 4 px en controles
y 6 px en superficies, sin sombras ni gradientes. Tipografías: `Segoe UI
Variable` para cuerpo/display y `Cascadia Mono` para etiquetas en Windows, con
caída a las fuentes por defecto de egui en otros sistemas (sin assets nuevos).
Los iconos se dibujan con `Painter` en rejilla de 24 y trazo de 1,7 px, con la
misma gramática que los SVG actuales.

## 4. Manejo de errores

Igual que hoy: todo error de usuario termina en la barra de estado con mensaje;
nunca en `panic!` ni en un log silencioso. Los errores de `rod-player` y
`rod-subs` se convierten a texto en el borde entre motor y UI.

## 5. Testing

| Nivel | Estrategia |
|---|---|
| `core` | Los tests actuales se conservan sin cambios. |
| `subs` | Los golden contra mpv se conservan sin cambios. |
| `player` | `FakeSource` y la prueba de humo se conservan. |
| `app` | Los E2E de iced se sustituyen por `egui_kittest`: arranque → abrir → editar → exportar. `geometria`, `arrastre` y `tema` tienen tests puros sin ventana. |
| Visual | Captura real por slice, en Windows y Linux. El slice 2 compara el frame compuesto contra la captura anterior del pipeline iced. |

Cada slice cierra con `cargo build`, `cargo fmt --all -- --check`,
`cargo clippy --all-targets -- -D warnings` y `cargo test --workspace` en verde.

## 6. Slices

0. Bootstrap: workspace con eframe 0.33, estilo y pantalla de arranque. Salida:
   se ve el inicio con el estilo nuevo.
1. Editor: paneles, menú y barra de estado sobre estado real, sin video. Salida:
   se puede crear/abrir un proyecto y editar captions.
2. Video + subtítulos: callback `egui_wgpu` reutilizando los `.wgsl`. Salida: un
   frame con subtítulos y medición del coste por frame a 1080p.
3. Timeline: carriles, clips, arrastre, snap, zoom y pan. Salida: arrastrar y
   dividir captions en la app real.
4. Proyecto: guardar/cargar, importar, exportar y recovery. Son llamadas a
   módulos existentes; se conectan al nuevo front-end.
5. Cierre: borrar `iced` e `iced_aw` del workspace, revisar `Cargo.lock` y
   fusionar la rama.

## 7. Riesgos

| Riesgo | Impacto | Mitigación |
|---|---|---|
| `egui_wgpu` 0.33 no expone lo suficiente para el callback | Alto | Verificar la API al abrir el slice 2; si el callback no da `RenderPass`, se aísla el compositor en un módulo propio y se decide antes de seguir. |
| El rect del widget no coincide con el frame | Medio | Test de `geometria` con rects no cuadrados y comparación de captura en el slice 2. |
| `rfd` en hilo + canal cambia el orden de eventos | Medio | El resultado se aplica en el frame siguiente, nunca dentro de `update()`; test que comprueba que dos clics rápidos no se pierden. |
| egui 0.33 queda viejo | Bajo | Anclado a propósito. La actualización se decide por separado, con medición del coste de migrar a wgpu 29. |
| Trabajo sin commitear en `main` | Medio | El trabajo ocurre en el worktree `C:/Dev/RustyOpenDialogue-egui` desde `HEAD`; los 7 archivos modificados de `main` no se tocan. |

## 8. Decisiones registradas en `BITACORA.md`

- egui solo para UI; se descarta `egui-sharkplayer`.
- egui 0.33 para conservar `wgpu 27.0.1`.
- Composición GPU reutilizada, no compositor CPU nuevo.
- `egui_dock` pospuesto.
