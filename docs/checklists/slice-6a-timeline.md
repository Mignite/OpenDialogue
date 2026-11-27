# Slice 6a — timeline vectorial: regla, carriles, clips y playhead

Criterio de salida: el timeline deja de ser una fila de caracteres `━` y pasa a
dibujar la regla de tiempo, un carril por hablante (más `—`), los clips con su
texto y el playhead; un clic busca o selecciona.

Sin arrastres todavía: mover bordes y bloques, marquesina, pan y waveform son
6b. Este slice deja la aritmética probada, que es lo que los arrastres van a
usar.

## Pasos

- [x] **1. `Vista` con tests.** `crates/app/src/timeline.rs`: `inicio` (s) y
  `escala` (segundos por píxel) con `x_de`, `tiempo_de`, `ancho_de` y
  `con_escala(escala, ancla)`. El zoom deja quieto el tiempo bajo el playhead y
  el `inicio` se acota a 0 (no hay segundos negativos); la escala se acota a
  `[2 ms/px, 500 ms/px]` para que la regla no dibuje millones de marcas ni una
  división por cero. 6 tests.

- [x] **2. Paso de la regla.** `paso_regla(escala)` elige el menor paso de una
  lista redonda (0.5 s … 30 min) que deje al menos 70 px entre marcas. Sin eso,
  al hacer zoom las etiquetas se amontonan o desaparecen. 1 test con 7 escalas.

- [x] **3. Carriles.** Un carril `—` (sin hablante) y uno por hablante, en el
  orden del proyecto: es exactamente lo que ya hace
  `rod_core::caption_row_index`, y la agrupación reutiliza
  `agrupar_captions_por_carril` (una pasada, O(n)). Color fijo por índice de
  carril: es una ayuda para distinguirlos, no el color del `.ass` (eso llega con
  presets, slice 7).

- [x] **4. Clic → mensaje.** `Program::update` publica
  `Mensaje::Seleccionar(id)` si el clic cae sobre un clip y `Mensaje::Buscar(t)`
  en la regla, en un hueco del carril o fuera de la regla. Un clip de menos de
  2 px se dibuja con 2 px y sigue siendo clicable (`clip_en` usa el mismo
  mínimo). 3 tests que llaman a `update` con un `mouse::Cursor::Available` —
  el camino de interacción se prueba sin GPU, sin ventana y sin escritorio.

- [x] **5. En la app.** El placeholder de "Timeline" pasa a
  `canvas(timeline::widget(..))` de 200 px con botones `+`/`−` de zoom anclado al
  playhead. Arriba de los carriles, una línea marca el fin del video.

- [x] **6. Verificar.** `fmt --check`, `clippy -D warnings` y `test --workspace`
  en 1.99 y 1.100: 44 (app lib) + 2 (app bin) + 235 (core) + 3 (player lib) +
  15 (subs lib) + 3 (oráculo). La app arranca y sigue viva.

## Evidencia

`docs/checklists/slice-6-timeline.png`: la ventana real, 1616×939, con la regla
en 2 s, el carril `—` con los dos captions del fixture, el playhead rojo en
14,5 s y los botones de zoom.

## Hallazgos

- **La captura de la ventana sí se puede.** El slice 4 concluyó que capturar
  exigía robar el primer plano; era cierto para `CopyFromScreen`, no para
  `PrintWindow(hwnd, hdc, PW_RENDERFULLCONTENT)`, que pide a la ventana su
  propia composición y funciona con la ventana en segundo plano. Con
  `SetProcessDpiAwarenessContext(-4)` las coordenadas coinciden con las físicas.
  Queda como el método de evidencia visual: `docs/checklists/slice-6-timeline.png`
  salió sin tocar el escritorio. Los **clics** no se pueden inyectar así
  (`PostMessage` no llega a winit), por eso el clic se prueba por el `update`
  del canvas.
- **Sin `Cache` a propósito.** El timeline redibuja sus formas cada frame. Un
  `Cache` de canvas hay que invalidarlo a mano en cada edición y en cada tick del
  playhead; con unos cientos de rectángulos, el ahorro no paga ese riesgo. Se
  mide en 6b, cuando entre la waveform.
- **`agrupar_captions_por_carril` clona los captions.** Para el timeline eso es
  otra copia por frame encima de la que ya hace el grupo. Está bien para el
  fixture; si el profiling de 6b lo señala, se pasa a préstamos.

## Qué NO hacer todavía

- Nada de arrastres, marquesina, pan ni auto-scroll del playhead: son 6b y
  necesitan el estado de arrastre que este slice no tiene.
- No inventar colores de hablante desde `Hablante.color`: el color del `.ass`
  sale del preset (`color_hablante`), y sin presets todo es blanco.
- No usar la resolución del video para nada del timeline: el eje es de tiempo.
