# Slice 6c — navegar el eje: rueda, zoom y seguimiento del playhead

Criterio de salida: con el video largo, el usuario puede moverse por el eje con
la rueda, acercar y alejar, y la vista persigue al playhead sola mientras no la
tome a mano. Todo con la aritmética probada y sin GPU.

## Pasos

- [x] **1. Una sola puerta para mover el inicio.** `Vista::con_inicio` acota a
  `[0, duracion - ancho_visible]`; `desplazar` y `objetivo_de_seguimiento` pasan
  por ella. Antes solo existía `con_escala` (zoom), que también acota su propio
  inicio a 0: no había forma de dejar la ventana fuera del video porque no se
  podía desplazar.

- [x] **2. La rueda la interpreta la app, no el canvas.** El canvas publica
  `Mensaje::Rueda { lineas, punto }` y nada más; la app decide si es pan o zoom.
  Motivo: `mouse::Event::WheelScrolled` **no trae modificadores**, así que el
  «Shift + rueda = zoom» solo se puede decidir donde vive el espejo del teclado.
  El canvas traduce `ScrollDelta::Pixels` a líneas (`PX_POR_LINEA = 40`) porque
  un trackpad entrega píxeles y el resto del mundo, muescas.

- [x] **3. El ancho lo publica el canvas.** La app necesita saber cuántos
  segundos caben (pan, seguimiento, «encajar el video»), y el canvas es el único
  que lo sabe. `Mensaje::Ancho(f32)` se publica desde los eventos de ventana y
  solo cuando cambia (`State = Option<f32>`), si no sería un mensaje por frame.

- [x] **4. Pan proporcional.** Una muesca mueve un 10 % de lo que se ve. El
  oráculo usa segundos fijos (`deltaY * 0.03`, ~3 s por muesca): con una ventana
  de 20 s eso es un salto y con una de 240 s es un suspiro. Proporcional se
  siente igual a cualquier zoom.

- [x] **5. Zoom anclado al cursor.** `Vista::con_escala` con `punto.x` deja
  quieto el tiempo que hay bajo el ratón; 1.15 por muesca, el mismo paso que el
  oráculo. Los botones `+`/`−` siguen anclados al playhead (6a): son otra cosa
  —«acerca de lo que estoy viendo»— y no cambian.

- [x] **6. Seguimiento con LERP.** Los tres números del oráculo
  (`utils/constants.ts`): disparador al 92 % de la ventana (`EDGE_TRIGGER`),
  recolocación al 8 % (`NEW_MARGIN`) e interpolación del 12 % por frame
  (`LERP`). Un seek grande no da un salto seco; el bucle corta a 2 ms de
  distancia y deja el inicio exacto. Se calcula en el `Tick` y se salta durante
  un arrastre: mover el mundo bajo el dedo es un bug de sensación.

- [x] **7. Un booleano, no dos.** El oráculo tiene `autoFollowing` (botón) y
  `isScrollingManuallyRef`, pero el pan pone `autoFollowing = false` a la vez
  que el flag manual, así que el segundo solo acompaña al primero. Aquí es un
  `seguir: bool`, visible en el botón «sigue»/«manual». La rueda (pan o zoom) lo
  apaga: si siguiera encendido, el frame siguiente desharía el gesto.

- [x] **8. Encajar el video al medirlo.** Con la escala por defecto (0,02 s/px)
  un clip de 15 s ocupaba media pantalla y el resto era vacío. La primera medida
  del canvas (y cada `Abrir video`) hace `Vista::encajar`: la ventana mide justo
  la duración. Es una desviación del oráculo, que usa 10 s fijos; aquí el ancho
  no es fijo y encajar es lo que el ojo espera.

- [x] **9. Verificar.** `cargo fmt --all -- --check`, `cargo clippy --all-targets
  -- -D warnings` y `cargo test --workspace`: 68 (app lib, 37 del timeline) + 2
  (app bin) + 235 (core) + 3 (player lib) + 15 (subs lib) + 3 (oráculo). La app
  arranca y sigue viva a los 7 s con stderr vacío.

## Evidencia

- `docs/checklists/slice-6c-eje.png`: la barra de estado dice «ventana 15.0 s»,
  que es la prueba de que `Ancho` llegó a la app y de que la vista se encajó al
  video (15,04 s de duración en 1526 px de canvas: 9,83 ms/px). El botón «sigue»
  se ve encendido y el playhead en 11,37 s dentro de la ventana.
- Los tests cubren lo que la captura no puede: pan acotado por los dos extremos,
  zoom que deja quieto el punto bajo el cursor, el disparador del 92 %, la
  recolocación al 8 %, que `acercar` converge exacto y que el primer frame no es
  un salto.

## Hallazgos

- **`WheelScrolled` no lleva modificadores.** Ni en `mouse::Event` ni en
  `iced::event::listen_with`. El «Shift para acercar» obliga a espejar
  `ModifiersChanged` en el estado y a que la rueda la interprete la app; por eso
  el canvas solo publica líneas y punto.
- **`Event::Window(..)` sí llega al `Program` del canvas.** Es lo que permite
  publicar el ancho sin inventar un `Responsive`: el widget recibe el
  `RedrawRequested` de cada frame y lo usa para avisar una sola vez.
- **`!(x > 0.0)` es error de clippy** (`neg_cmp_op_on_partial_ord`). El guard de
  `encajar` quedó como un `match` con `is_finite`, que además dice la verdad:
  NaN y anchos sin medir son el mismo caso.
- **Convergencia del LERP:** 12 % por frame y corte en 2 ms son ~67 frames para
  recorrer 10 s (1,1 s a 60 Hz). Está en el test para que nadie lo «arregle» a
  100 frames sin saber lo que tarda.

## Qué NO hacer todavía

- Nada de barra de desplazamiento arrastrable: el oráculo la tiene y es el
  siguiente paso natural, pero necesita su propio estado de arrastre y una zona
  fuera del canvas. El pan con rueda ya cubre el caso de uso.
- Nada de Ctrl + rueda para desplazar los carriles en vertical: con 9 hablantes
  como máximo el timeline de 200 px no da para más de 5 carriles, pero el scroll
  vertical merece su propio slice junto con la waveform.
- Nada de waveform todavía: es lo que decide `Cache`/`shader`, y el oráculo la
  analiza con `AudioContext`, que no existe aquí (haría falta decodificar audio
  en Rust, otra dependencia).
