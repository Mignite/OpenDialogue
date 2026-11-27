# Layout del editor: divisor libre y pistas scrolleables

## Intención

El video ocupa casi toda la ventana y el timeline solo deja ver una pista y
media. El divisor del timeline **ya es arrastrable**, pero `editor.rs` le pone
`max_height(alto_abajo)` — igual a su altura actual, unos 260 px — así que
arrastrar hacia arriba no hace nada: el video tiene un mínimo efectivo de
~580 px. Además, las pistas que no caben no se dibujan: el bucle de
`timeline.rs` rompe al llegar al borde del rect.

## Objetivos

- Poder encoger el video arrastrando el divisor hasta que el timeline ocupe
  como máximo el 70 % de la ventana; el alto queda guardado en `Preferencias`,
  igual que el ancho del panel lateral.
- Ver todas las pistas cuando haya más de las que caben, con scroll vertical y
  la regla de tiempo fija arriba.

## No objetivos

- Cambiar la lógica pura de `crates/app/src/timeline.rs` ni sus tests.
- Tocar el ancho del panel lateral ni el pipeline de video.
- Persistir el desplazamiento vertical de pistas (es estado de vista, no de
  documento; el zoom temporal tampoco se guarda).

## Restricciones

- Todo valor visual desde `ui/tema.rs`; ningún texto nuevo por debajo de 11 px.
- El ancho del eje se sigue comunicando con `Mensaje::Ancho` solo cuando cambia.
- Verificación: `cargo fmt --all -- --check`, `cargo clippy -p rod-app -- -D
  warnings`, `cargo test --workspace`, y capturas a 1100/1440/2560 px.

## Approaches

1. **Quitar el tope del divisor + scroll con offset en la frontera (elegido).**
   `max_height` pasa a `0.7 * alto_de_la_ventana` con `min_height` 160. El
   scroll se lleva en `EditorApp` como `scroll_pistas: f32` (filas, no píxeles).
   La UI suma `scroll_pistas * ALTO_CARRIL` a la `y` del `Punto` antes de
   despachar los mensajes, así que `pulsar`, `arrastrar` y `clip_en` siguen
   recibiendo coordenadas de carril absolutos y no cambian. Se dibuja una
   scrollbar delgada a la derecha del timeline, solo cuando hay más pistas que
   caben. **Trade-off:** ~80 líneas de UI para la scrollbar y su interacción;
   a cambio la regla queda fija y el modelo de arrastre intacto.
2. **`ScrollArea::vertical()` de egui envolviendo el timeline.** Menos código y
   scrollbar gratis, pero la regla se desplaza con las pistas y el rect que
   recibe los eventos deja de ser el mismo, así que habría que reescribir la
   geometría de clics. Descartado.
3. **Altura automática según número de pistas.** El timeline crece solo hasta
   caberlas todas. Descartado: el usuario pidió *más* control del layout, no
   menos.

## Decisiones cerradas

- **Rueda:** la rueda simple desplaza pistas; Ctrl+rueda mantiene el zoom anclado
  al cursor; Shift+rueda mantiene el pan horizontal de tiempo. El slider de zoom
  de la toolbar sigue funcionando. Cambiar la rueda simple de pan a pistas es
  la única regresión de gesto aceptada y es lo que pidió el autor.
- **Divisor:** tope del 70 % de la altura de contenido, suelo 160 px. Se guarda
  `medidas.alto_timeline` como hoy (solo cuando el campo de texto no está
  expandido), y se limita al rango válido al cargar.
- **Regla y gutter:** la regla de tiempo y la etiqueta «TIEMPO» quedan fijas;
  las etiquetas de carril se desplazan con sus pistas.
- **Scrollbar:** 10 px a la derecha del timeline; el rect interactivo del
  timeline se estrecha ese mismo ancho para que los clics en la barra no
  lleguen al despacho de `Pulsar`.

## Lo que queda al implementador

- Reparto exacto de `show_viewport` frente a painter manual, y si el offset se
  guarda en filas (`f32`) o en píxeles; la spec solo fija el comportamiento.
- Si el scroll se agrupa en un solo paso de historial: no, es vista, no entra
  en `Historial`.
- Nombre concreto del campo y de los helpers de límites.
