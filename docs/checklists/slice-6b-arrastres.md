# Slice 6b — arrastres del timeline: bordes, bloque y marquesina

Criterio de salida: un clip se puede estirar por sus bordes, mover en el tiempo
y cambiar de carril arrastrándolo, y una marquesina selecciona lo que toca. Todo
ello sin tocar la GPU y con la aritmética probada.

## Pasos

- [x] **1. El canvas solo traduce el ratón.** `timeline::Mensaje` pasa a
  `Pulsar(punto)`, `Arrastrar(punto)` y `Soltar(punto)`, en coordenadas locales
  del widget. Antes el canvas decidía "esto es un clip, selecciónalo"; ahora el
  dibujo y la interacción no comparten responsabilidad: quién se agarra lo
  decide `arrastre::pulsar` con el proyecto delante.

- [x] **2. Agarre, sin ambigüedad de píxel.** `AGARRE_BORDE = 6 px` a cada lado
  del borde dibujado (1 px). Un clip más estrecho que dos agarres cede al
  izquierdo; el cuerpo se agarra en el resto. `Asa::{Izquierda, Derecha}`.

- [x] **3. Recalcular desde el original, no acumular delta.** Cada `Arrastre`
  guarda `Original { inicio, fin, fila }` de lo que agarró y cada frame
  recalcula desde ahí. Acumular deltas se desvía (los `f64` no son exactos, el
  clamp recorta, el snap se pega al último imán), y soltar donde se pulsó
  dejaría de ser un no-op. Las marcas se cuantizan a milisegundos
  (`a_milisegundos`), que es lo que entienden SRT y ASS.

- [x] **4. Bordes.** El clamp es explícito por asa en vez de usar
  `clamp_caption_tiempos`: esa función empuja el borde contrario cuando la
  duración mínima no cabe, y al estirar por la izquierda eso movería el fin que
  el usuario no está tocando. `DURACION_MINIMA` (0.1 s) se respeta, y el fin no
  pasa de la duración del video.

- [x] **5. Bloque.** Mover un clip mueve toda la selección (si el clip pulsado
  ya estaba seleccionado), con `clamp_delta_bloque` contra t=0 y
  `find_snap_block_delta` probando los dos extremos del bloque contra vecinos y
  playhead. En vertical, `filas_destino_relativas` conserva el desfase entre
  carriles y cede en los bordes (nadie se va a la fila -1 ni más allá del último
  hablante). Cambiar de carril no mueve el tiempo.

- [x] **6. Marquesina.** Pulsar en un hueco arranca un rectángulo;
  `filtrar_por_marquee` (ya en `rod_core`) devuelve los ids por solape de tiempo
  y de fila, y el dibujo la pinta translúcida mientras se arrastra. Si el
  puntero no se movió más de `UMBRAL_CLIC = 3 px`, fue un clic: se busca, como
  en el oráculo.

- [x] **7. Un arrastre = un paso de historial.** El `Arrastre` lleva la
  instantánea del proyecto al pulsar; la app la registra
  (`registrar_instantanea`) en el primer frame que cambia algo, no al pulsar.
  Así un clic sin movimiento no deja un Ctrl+Z que no hace nada, y un arrastre
  de 60 frames no deja 60 pasos.

- [x] **8. Verificar.** `cargo fmt --all -- --check`, `cargo clippy --all-targets
  -- -D warnings` y `cargo test --workspace` en 1.99 y 1.100: 58 (app lib,
  27 del timeline: 14 de arrastre + 13 de dibujo/aritmética) + 2 (app bin) +
  235 (core) + 3 (player lib) + 15 (subs lib) + 3 (oráculo). La app arranca y
  sigue viva a los 7 s con stderr vacío.

## Evidencia

- `docs/checklists/slice-6b-timeline.png`: la ventana real con la regla, el
  carril `—` con los dos captions, el playhead en 11,69 s, la línea del fin del
  video en 15 s y los botones de zoom.
- Los tests son la evidencia del arrastre: cubren agarrar borde vs cuerpo,
  estirar sin mover el borde contrario, el mínimo de 0.1 s, el bloque sin
  deformarse, el tope en 0, el cambio de carril con desfase, el snap al vecino,
  la marquesina por fila y que soltar donde se pulsó no cambia nada.

## Hallazgos

- **`clamp_caption_tiempos` no sirve para estirar un borde.** Está pensada para
  "dame estos tiempos y arregla lo que esté mal", y cuando la duración mínima no
  cabe alarga el fin. En un arrastre del borde izquierdo eso mueve el borde que
  el usuario no está tocando: el clamp por asa es más corto y más honesto.
- **El snap de borde puede dejar solapes.** `find_snap_time` prueba contra
  *todos* los bordes ajenos, así que estirar un clip puede pegarle el fin al
  inicio de un vecino o al fin de otro que ya solapa. Es lo mismo que hace el
  oráculo (el solape no se prohíbe, se reporta con `build_overlap_report`), y
  aquí se deja igual a propósito.
- **`Action::publish(..).capture()` no existe**; el método es `and_capture()`
  (`iced_widget::Action`). El compilador lo dijo con un `help` muy claro.
- **`Marco` evitó una firma de ocho argumentos.** `aplicar` necesita vista,
  punto, duración y playhead además del proyecto; agrupar los cuatro en un
  `struct Copy` dejó las firmas legibles y quitó el `allow(too_many_arguments)`.

## Qué NO hacer todavía

- Nada de pan del eje ni auto-scroll del playhead: son el resto de 6 y necesitan
  decidir cuándo la vista sigue al reproductor (si el usuario arrastró el eje, no
  debe seguirlo).
- Nada de waveform: es lo que justifica (o no) el `Cache` y el `shader`.
- El arrastre no tiene "cancelar con Esc": soltar siempre deja el último frame.
  El oráculo tampoco lo tiene, y añadirlo sin pedirlo complica el estado.
- No hay imán visual (línea guía) cuando el snap muerde: se ve el resultado, no
  el porqué. Va con el pulido de 6.
