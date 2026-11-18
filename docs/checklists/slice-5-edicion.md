# Slice 5 — edición: undo/redo, editor de texto y preview del proyecto

Criterio de salida: editar un subtítulo cambia el preview, y deshacer devuelve
el estado anterior (texto e imagen). Sin GPU y sin copiar el escritorio de nadie.

Hasta aquí la app mostraba un `.ass` de fixture (`basico.ass`) y un historial no
existía. La lista de subtítulos y la selección por id ya venían del slice 2.

## Pasos

- [x] **1. Historial, portado del oráculo.** `crates/core/src/historial.rs`:
  `Historial` con `registrar` / `deshacer` / `rehacer` / `limpiar`,
  `Instantanea` (captions + hablantes, **no** `ruta_video` ni `playhead`) y
  `LIMITE_HISTORIAL = 50` (el `HISTORY_LIMIT` de `useHistory`). Instantáneas
  completas en vez de parches inversos: con unos cientos de captions copiar el
  vector es más barato y no hay parche que pueda quedar inconsistente. 9 tests.

- [x] **2. El editor decide cuándo hay paso.** `EstadoEditor` gana `historial`,
  `registrar_historial`, `deshacer`, `rehacer`, `puede_deshacer`,
  `puede_rehacer` y `aplicar_texto`. `puede_*` es lo que apaga los botones: un
  Ctrl+Z sin pasado no debe gastar un `Message`. `aplicar_texto` ignora el texto
  idéntico (no gasta un paso del historial) y registra **uno** por commit, no
  uno por tecla. La selección no se restaura —como en el oráculo—, solo se poda
  a ids que existan. 4 tests.

- [x] **3. El preview sale del proyecto.** `crates/app/src/preview.rs`:
  `ass_del_proyecto` = `exportar_ass` con los estilos del `.ass` cargado
  (`preset_desde_estilos(.., "Default")`) o `preset_por_defecto` si el proyecto
  vino de un `.srt`. `CANVAS = (1920, 1080)` es una sola constante para el
  buffer y el `PlayRes`. Se borra `ASS_FIXTURE`: el `.ass` del preview es el
  mismo `build_ass` que produciría la exportación. 3 tests.

- [x] **4. `Subtitulos::cargar`.** Cambiar el `.ass` sin rehacer biblioteca ni
  rasterizador: `ass_read_memory` del texto nuevo, `ass_free_track` del viejo y
  `bitmaps.clear()` (apuntaban al track muerto). `refrescar_preview` la llama
  tras cada edición y rasteriza en la posición actual, porque en pausa mpv no
  entrega un frame nuevo que dispare el repintado. 1 test: un `.ass` sin
  eventos no produce bitmaps y con texto sí.

- [x] **5. UI.** `text_input` + botón **Aplicar** (y Enter) bajo la cabecera del
  panel; el texto se edita en un `borrador` y solo toca el proyecto al
  confirmar. Botones ↶/↷ apagados cuando no hay pasado/futuro. Atajos en la
  `subscription`: Ctrl+Z, Ctrl+Shift+Z y Ctrl+Y (`KeyPressed` filtrado por
  `modifiers.command()`; no se puede hacer desde `update` porque el
  `text_input` se queda con los caracteres).

- [x] **6. Verificar.** `cargo fmt --all -- --check`, `cargo clippy --all-targets
  -- -D warnings` y `cargo test --workspace` en 1.99 y 1.100: 24 (app lib) +
  2 (app bin) + 235 (core) + 3 (player lib) + 15 (subs lib) + 3 (oráculo).
  La app arranca y sigue viva a los 6 s con stderr vacío.

## Evidencia observable

`cargo run -p rod-app --example preview` rasteriza el `.ass` del proyecto con
libass y compone sobre negro, sin ventana:

```
ass del preview: 681 bytes
texto del caption 1: Some("Hola mundo")
píxeles con texto antes: 2644
texto tras aplicar: Some("Texto editado a mano")
píxeles con texto después: 4718
la imagen cambió: true
texto tras deshacer: Some("Hola mundo")
deshacer devuelve la imagen exacta: true
```

Los PNG quedan en `docs/checklists/slice-5-preview-antes.png` y
`slice-5-preview-despues.png`.

## Hallazgos

- **`exportar_ass` tenía un parámetro de tipo fantasma.** La firma era
  `exportar_ass<F>(..)` sin usar `F`: compilaba porque nadie la llamaba, y la
  primera llamada real no pudo inferir `F`. Se quitó el genérico; el closure
  `preset_for` ya está dentro de la función.
- **La granularidad del historial es de la app, no del editor.** El editor
  guarda lo que le pidan; la app decide que escribir en el `text_input` no es
  una edición y pulsar Aplicar sí. Eso deja el test del historial en la
  librería y el teclado fuera de él.

## Qué NO hacer todavía

- No meter el preview en el `update` de cada tecla: rasterizar libass por
  pulsación gasta CPU para mostrar un texto que el usuario todavía no confirmó.
- No unir `playhead` a la instantánea: mover el slider no es una edición y
  Ctrl+Z no debe devolver el video a otro segundo.
- **Pendiente del slice 5:** el panel de hablantes (crear, renombrar, color y
  preset por hablante). La lista, la selección y el editor ya están.
