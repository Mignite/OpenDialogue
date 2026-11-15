# Slice 0b — libass: bitmaps de subtítulo y fidelidad con mpv

Riesgo que cubre: construir/enlazar libass en Windows, que resuelva fuentes, y
que el render coincida con mpv. Es el slice donde nace el invariante del
proyecto (spec §3.3).

Criterio de salida: un `.ass` de fixture rasterizado a PNG desde Rust, y una
comparación numérica contra el mismo frame renderizado por mpv.

Conceptos nuevos: FFI declarado a mano, `#[repr(C)]`, punteros opacos, strings C,
libertad de memoria (`ass_free_track`), composición alfa.

## Preparación

- [x] **Windows**: `vcpkg install libass:x64-windows`. (El port existe: libass
      0.17.5, con freetype, fribidi y harfbuzz como dependencias.)
      *Hecho:* 4.2 min. `crates/subs/build.rs` busca `lib/ass.lib` en `ASS_DIR`,
      `VCPKG_ROOT/installed/x64-windows`, `%USERPROFILE%/vcpkg/...` y
      `C:/vcpkg/...`, y copia las DLLs junto a los ejecutables (en Windows la
      búsqueda empieza por el directorio del `.exe`, así que sin eso cada
      `cargo run` exigiría el PATH a mano).
- [ ] **Linux**: `sudo pacman -S libass`, y comprobar con
      `pkg-config --modversion libass`.
- [x] **El oráculo sin `mpv.exe`.** No hace falta instalar mpv: la DLL de libmpv
      que ya enlazamos trae su propio libass y compone los subtítulos en el
      frame. `crates/player/examples/oracle_subs.rs` carga el fixture con
      `sub-add`, busca el instante con `hr-seek` y guarda
      `testdata/oracle-subs.png`.
      *Ojo con el orden*: hay que **drenar** los flags de
      `mpv_render_context_update` antes del `seek`. El flag pendiente de la
      carga describe el frame anterior, y si se consume primero el render
      devuelve el frame 0 con el `time-pos` ya en 1.0. Cuesta 15 minutos y el
      player real va a repetirlo.
      *Y no basta con mirar `time-pos`*: se actualiza al pedir el seek, mucho
      antes de que llegue el frame. El aviso de que el seek terminó es el evento
      `playback-restart`. Sin esperarlo, el PNG sale a veces con el frame 0 (se
      vio en la captura: timecode quemado `00:00:00.000`, caso 0). Con el evento,
      timecode `00:00:01.000`, caso 30.
      *Dos salidas*: `oracle_subs` guarda `testdata/oracle-subs.png` y, con
      `--sin-subs`, `testdata/frame-sin-subs.png`: el mismo frame sin
      subtítulos, que es el fondo sobre el que componer para comparar píxeles.
- [x] **Verificar el proveedor de fuentes**, que es el punto delicado en Windows:
      inicializar libass con `ASS_FONTPROVIDER_AUTODETECT` y leer el log. Debe
      decir que usa *directwrite* en Windows y *fontconfig* en Linux.
      *Verificación*: aparece el nombre del proveedor en el log. Si no aparece,
      libass se compiló sin ese proveedor y hay que resolverlo **antes** de
      seguir (es el motivo por el que existen los avisos de "fuente no
      instalada").
      *Hecho:* `Using font provider directwrite (with GDI)`, y
      `fontselect: (Arial, 400, 0) -> ArialMT`. El port de vcpkg **no** arrastra
      fontconfig, así que no hay `fonts.conf` que configurar. El log se lee por
      `ass_set_message_cb` (ver el hallazgo de `vsnprintf`).

## Pasos

- [x] **1. FFI a mano, sin bindgen.** Declarar solo lo necesario: cuatro tipos
      opacos (`AssLibrary`, `AssRenderer`, `AssTrack`, `AssFontProvider`) y unas
      quince funciones (`ass_library_init`, `ass_renderer_init`,
      `ass_set_frame_size`, `ass_set_storage_size`, `ass_set_pixel_aspect`,
      `ass_set_margins`, `ass_set_use_margins`, `ass_set_fonts`,
      `ass_read_memory`, `ass_free_track`, `ass_render_frame`,
      `ass_renderer_done`, `ass_library_done`, `ass_set_message_cb`).
      *Verificación*: compila y `ass_library_init` no devuelve nulo. Bindgen
      queda como plan B solo si el API crece.
      *Hecho:* `crates/subs/src/ffi.rs` (crudo) y `lib.rs` (`Library`,
      `Renderer`, `Track`, `Bitmap`). `ASS_FontProvider` sobró: el proveedor se
      pide con un `int` (`ASS_DefaultFontProvider`), no con un puntero.

- [x] **2. Leer un `.ass` desde memoria.** Un fixture de tres líneas con dos
      estilos distintos.
      *Verificación*: `ass_read_memory` devuelve un track y el log de libass no
      reporta error de parseo.
      *Hecho:* el log muestra los dos estilos y `Added subtitle file: <memory>
      (3 styles, 2 events)` (3 porque libass añade `Default`).

- [x] **3. Rasterizar un instante.** `ass_render_frame(track, 1000 ms, &changed)`
      y recorrer la lista de `ASS_Image`.
      *Verificación*: imprime cuántas imágenes hay, y para cada una `w`, `h`,
      `stride`, `dst_x`, `dst_y`. Compara esas coordenadas con lo que esperas de
      tu fixture: ahí se ve si el tamaño de frame está bien configurado.
      *Hecho:* `cargo run -p rod-subs --example smoke`. A 1000 ms: 5 bitmaps —
      "Subtitulo de prueba" en `(711,963)` y `(714,966)` (contorno negro y
      relleno blanco), y "Arriba centrado" en `(719,275)`, `(717,273)`,
      `(720,276)` (sombra, contorno, relleno rojo). El `\pos(960,300)` con
      alineación 5 cae centrado en y=300: el frame está a 1920×1080 y `stride`
      sale 512 porque libass alinea las máscaras.

- [x] **4. Componer en CPU.** Volcar los bitmaps sobre un buffer RGBA con
      `dst = rgb·(a·A/255) + dst·(1 − a·A/255)` y escribirlo a PNG.
      *Verificación*: se lee el texto, con su borde y su color, en la posición
      correcta. Sin wgpu todavía: primero la matemática, después la GPU.
      *Hecho:* `crates/subs/examples/componer.rs`, que compone sobre
      `testdata/frame-sin-subs.png` (el frame del oráculo sin subtítulos) y
      guarda `testdata/render-propio.png`. En la fórmula, `a` es la máscara
      (`Bitmap::alpha()`) y `A` es la opacidad del color,
      `255 - (color & 0xFF)`; se hace en `u32` porque en `u8` los productos se
      desbordan.

- [x] **5. El oráculo.** El mismo `.ass`, el mismo instante, renderizado por mpv:
      `mpv --no-config --vo=image --sub-file=fixture.ass --sub-ass-override=no
      --start=1 --frames=1`. Comparar los dos PNG.
      *Verificación*: porcentaje de píxeles distintos. Anotar el número, la
      versión de libass y la fuente usada. Si difiere en más de un 1 %, averiguar
      si es escala, fuente o versión **antes** de dar el paso por bueno.
      *Hecho:* `crates/player/examples/oracle_subs.rs` (sin `mpv.exe`: usa el
      libass que trae la DLL de libmpv) contra `componer`. **0.000 %** de
      píxeles con diferencia > 8/255; 0.064 % con cualquier diferencia y máximo
      2/255, que es el redondeo de la mezcla. El PNG de diferencias
      (`testdata/diff.png`) sale negro.
      *Con la compatibilidad de color de VSFilter encendida* (el default de
      mpv) el rojo `(255,0,0)` volvía como `(255,24,0)` y la diferencia subía a
      0.426 %. Ver el hallazgo: no es un defecto del render propio.

- [x] **6. Cerrar.** Escribir en `BITACORA.md` la versión de libass, el proveedor
      de fuentes y el porcentaje de coincidencia. Ese número es la línea base
      del proyecto.
      *Hecho:* entrada del 2026-11-15.

## Hallazgos

- **El color de `ASS_Image` es `0xRRGGBBAA`, con el alfa al revés.** No es
  `0xAABBGGRR` como los colores del `.ass`. En el código de libass
  (`ass_parse.h`): `_r(c) = c >> 24`, `_g = (c >> 16) & 0xFF`,
  `_b = (c >> 8) & 0xFF`, `_a = c & 0xFF`; y ese `_a` es **transparencia**
  (0 = opaco, 255 = invisible), la convención de ASS. Comprobado con el fixture:
  relleno blanco `0xffffff00`, contorno negro `0x00000000`, sombra al 50 %
  `0x00000080`, relleno rojo `0xff000000`.
- **La lista viene en orden de pintado**: sombra, contorno, relleno. Se compone
  en ese orden, sin ordenar.
- **`vsnprintf` no enlaza con `msvcrt.lib`.** El callback de `ass_set_message_cb`
  recibe un `va_list`, así que hay que formatearlo en C: `#[link(name =
  "legacy_stdio_definitions")]`, que es donde vive el stub que exporta UCRT.
- **`ass-9.dll` no basta.** Depende de `iconv-2.dll`, y freetype de
  `z`/`bz2`/`libpng16`/`brotlidec`: la lista completa está en el `build.rs`.
- **La máscara viene alineada.** `stride` (512) no es `w` (490), y la última
  fila puede ser más corta: `stride * (h - 1) + w` es el tamaño garantizado.
  `Bitmap::alpha()` recorta a eso a propósito.
- **mpv manglea los colores de subtítulo por defecto, y no es un bug propio.**
  Con `--sub-ass-override=no` pero sin tocar `--sub-ass-vsfilter-color-compat`,
  el rojo `(255,0,0)` del `.ass` sale `(255,24,0)`: VSFilter codificaba el color
  asumiendo BT.601 y lo mezclaba en un video BT.709, y mpv lo emula. Los grises
  no se enteran (el texto blanco coincidía exacto), así que el síntoma es
  "solo fallan los colores saturados". En el oráculo se apaga con
  `sub-ass-vsfilter-color-compat=no`; en la app no se emula: sería adoptar un
  defecto ajeno. Es la primera decisión de fidelidad del proyecto, y va en la
  misma familia que "el preview es el export".
- **La comparación RGB contra mpv solo vale con esa compatibilidad apagada.**
  mpv mezcla los subtítulos en el espacio del video (YUV), no en RGB; cualquier
  comparación píxel a píxel contra su frame pasa por su conversión de color.

## Qué NO hacer en este slice

- No dibujes en la ventana: el PNG es suficiente para decidir.
- No implementes caché de bitmaps ni `detect_change` todavía. La primera versión
  vuelve a rasterizar cada vez; la optimización se mide en el slice 4.
- No adaptes el `.ass` para que "se vea igual". Si difiere de mpv, la diferencia
  es información, no un estorbo.
