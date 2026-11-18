# Slice 4 — compositor: escala y letterbox con tests

Criterio de salida: video y subtítulos compuestos con la proporción correcta, y
la geometría que lo decide comprobada **sin GPU**.

El compositor ya existía: `crates/app/src/video/` sube el frame de mpv a una
textura, arma un atlas con las máscaras de libass y dibuja dos veces en el mismo
pase (video, luego capas con blending). Funcionaba a ojo — la captura del slice
0c (`slice-0c-paso-5.png`) muestra el frame con los subtítulos encima —, pero la
aritmética del letterbox y del rectángulo de cada capa vivía **dentro de
`prepare`**, que necesita `device` y `queue`. Esa es justo la parte que se
equivoca en silencio: un video deformado o un texto corrido no fallan, se ven
mal, y un test no puede tocar `prepare` sin una tarjeta.

## Pasos

- [x] **1. La geometría, fuera de la GPU.** `crates/app/src/video/geometria.rs`:
  - `escala_letterbox(video, widget) -> (f32, f32)`: cuánto se encoge el quad por
    eje (`1.0` = ese eje ya encaja).
  - `rect_de_capa(x, y, ancho, alto, video, escala) -> [x0, y0, x1, y1]` en clip
    space, con la `y` invertida y el letterbox aplicado.
  `prepare` y `subir_capas` ahora llaman a esas funciones; la fórmula no cambió,
  se mudó.

- [x] **2. Tests de la geometría, sin tarjeta.** Nueve casos:
  misma proporción no toca nada; 21:9 en 16:9 encoge la altura; 4:3 en 16:9
  encoge la anchura; widget vacío o video sin tamaño no deforman; una capa que
  cubre el canvas llena el quad (`[-1, 1, 1, -1]`); una capa de 2×2 centrada en
  100×100 cae en ±0.02; una capa fuera del canvas sale de clip space (la GPU
  recorta); y **la capa se mueve con el letterbox** (canvas 16:9 en widget 4:3:
  la capa de altura completa queda en ±0.75, igual que el video, para no
  desalinearse).
  *Hecho:* `cargo test -p rod-app --lib` → 14 tests (5 que ya había + 9 nuevos).

- [x] **3. La resolución del canvas es la del proyecto, no la del video.** El
  buffer y el `Subtitulos` son 1920×1080 fijos. No es un descuido: el `.ass`
  exportado usa `PlayResX/Y: 1920 1080` (el oráculo arranca con `{x: 1920,
  y: 1080}`), así que renderizar a otra resolución rompería el invariante del
  proyecto — el preview dejaría de ser el render del `.ass` que se exporta.
  Queda como desviación consciente de la frase "a resolución de video" de la
  spec, anotada allí.

- [x] **4. Cerrar.** `cargo fmt --all -- --check`, `cargo clippy --all-targets
  -- -D warnings`, `cargo test --workspace` y CI.

## Hallazgos

- **Capturar la ventana desde PowerShell miente dos veces.** El primer intento
  (sin `SetProcessDpiAwarenessContext`) dio una imagen desalineada: la ventana
  era de 1000×800 lógicos y 1250×1000 físicos, y `CopyFromScreen` mezcló las
  unidades. El segundo, ya DPI-aware, capturó el escritorio de otra aplicación
  porque `SetForegroundWindow` no puede robar el primer plano a un proceso que no
  lo pidió el usuario. Con el escritorio en uso, la verificación visual se
  pospone: se queda la del slice 0c y los números de la geometría.
- **El letterbox es del widget, no del buffer.** mpv ya entrega el video
  centrado dentro de su canvas de 1920×1080; el shader encoge el quad para que
  ese canvas entre en un widget de otra forma. Por eso las capas de libass se
  escalan con el mismo factor que el video y no se pueden correr respecto a él.

## Qué NO hacer en este slice

- No cambiar el canvas a la resolución del video. Se puede, pero antes hay que
  decidir qué `PlayRes` lleva el `.ass` exportado para videos que no sean 16:9;
  hoy el oráculo exporta 1920×1080 y esa es la referencia.
- No mover la aritmética a un crate aparte. Son dos funciones y su contrato es
  clip space de WGSL; un crate propio sería una capa más que leer.
