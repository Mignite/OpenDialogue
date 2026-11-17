# Slice 2 — `subs`: del ejemplo al test

Criterio de salida: un `.ass` de fixture renderizado a PNG desde `rod-subs`,
comparado **automáticamente** contra el mismo frame compuesto por mpv, con la
versión de libass registrada (spec §6).

Antes de este slice, la comparación existía pero era manual: `componer.rs`
componía con la fórmula copiada en el propio ejemplo y `oracle_subs.rs` dejaba
un PNG en `testdata/` para mirar a ojo. Nada de eso corría en `cargo test`, y
`testdata/` no está versionado, así que en otra máquina no había contra qué
comparar. Eso es lo que cierra el slice.

Conceptos nuevos: composición como código de biblioteca (no de ejemplo), recorte
por caja envolvente, `\fad` como prueba del alfa del color, y la distinción entre
"el frame que pedí" y "el frame que mpv está mostrando".

## Pasos

- [x] **1. La composición es de la biblioteca.** `Lienzo` (RGB8), `Rect`, `caja`
  y `componer`/`componer_desplazado` en `crates/subs/src/composicion.rs`.
  Con recorte de los bitmaps que caen fuera del lienzo: `dst_x` puede ser
  negativo y sin recorte el índice desborda.
  *Hecho:* 12 tests unitarios, sin libass ni fuentes: cobertura total y media,
  alfa del color, opacidad cero, recorte por los cuatro lados, desplazamiento,
  `recortar`, y `caja` con margen / recortada al frame / vacía.
  `examples/componer.rs` pasó a usarla: mismas cifras que en 0b (0.000 % >8,
  máximo 2/255), así que la fórmula no cambió al mudarse.

- [x] **2. La versión de libass, legible.** `rod_subs::version()` vía
  `ass_library_version()`. Hizo falta un test propio: el primer intento imprimió
  **112.80.0**. Ver hallazgo.

- [x] **3. Un fixture con variantes.** `tests/fixtures/variantes.ass`: cursiva,
  negrita, ajuste automático de línea (`MarginL/R 300` + texto largo), sombra, y
  `{\fad(1000,1000)}`, que a mitad del desvanecido deja el color con alfa 128 y
  ejercita el camino de `color & 0xFF`.
  *Hecho:* 13 bitmaps a 1.08 s.

- [x] **4. El generador del golden.** `crates/subs/examples/golden.rs <caso> <ms>`:
  busca el instante en mpv, captura el frame limpio, añade el `.ass`, captura el
  compuesto, y guarda **los dos recortes** a la caja que calculan nuestros
  bitmaps. Fondo plano con lavfi (`av://lavfi:color=...`), no un video: el
  golden queda en KB y no depende de `testdata/`.
  *Hecho:* `crates/subs/tests/fixtures/golden/{basico,variantes}-{fondo,oraculo}.png`
  (179 KB, 204 KB, 77 KB y 227 KB).

- [x] **5. El test.** `crates/subs/tests/oraculo.rs`: decodifica los PNG con
  `include_bytes!`, compone el render propio sobre el fondo del golden y compara
  píxel a píxel. Tolerancia 8/255 por canal y techo de 0.1 % de píxeles
  distintos (spec §6: "porcentaje de píxeles distintos").
  *Hecho:* **0 píxeles** por encima de 8/255 en los dos casos; con cualquier
  diferencia 499/422904 (0.118 %) en `basico` y 12022/292110 (4.1 %) en
  `variantes` — es el antialiasing del texto y del `\fad` — con máximo 1/255 y
  2/255. El techo de 0.1 % no lo usa nadie: queda de margen para un redondeo
  distinto, no para tolerar un texto movido.
  También se comprueba que el golden **no** sea igual al fondo (si no, el test
  pasaría por no tener subtítulos) y hay dos pruebas estructurales que no
  dependen de la fuente: los bitmaps caen dentro del frame y antes del primer
  evento no hay ninguno.

- [x] **6. Determinismo.** Se regeneraron los cuatro PNG dos veces: `Get-FileHash`
  da el mismo hash. El golden es reproducible en esta máquina, así que un diff
  en el repo significa un cambio real en el render.

- [x] **7. Cerrar.** Números y versión en `BITACORA.md`; `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings` y `cargo test --workspace` en
  verde.

## Hallazgos

- **La versión de libass viene en BCD, no en bytes.** `ass_library_version()`
  devuelve `0x01705000` para 0.17.5; leerla como `major<<16 | minor<<8 | patch`
  da `112.80.0`. Los cinco dígitos hexaltos son `M`, `mm`, `pp` **en decimal**
  (y los tres bajos, un extra de parche: el propio `ass.h` cita `0x01300001` para
  0.13.0). Queda en `describir_version`, con test contra los tres valores
  conocidos.

- **Dos sesiones de mpv no dan el mismo frame.** El primer generador buscaba el
  instante dos veces (con y sin subtítulos). Con la fuente a 25 fps una sesión
  aterrizó en 1.04 s y la otra en 1.08 s: el fondo coincidía por ser plano, pero
  un `\fad` salía con ~4 % de alfa de diferencia entre el golden y el render
  propio. Ahora se busca una vez, se captura el limpio y se añade el `.ass`
  (`sub-add` redibuja el mismo instante).

- **`time-pos` después del seek no es el instante pedido.** Pedir 1.0 s aterriza
  en 1.08 s. Por eso el render propio usa el instante que **reporta** mpv y no el
  que se pidió, y por eso el caso del test dice `1080` donde el comando dice
  `1000`. Al regenerar un golden hay que copiar el `ms` que imprime el
  generador.

- **El fondo plano no se puede suponer.** mpv lo entrega con dithering (un
  536×789 de color plano pesa 179 KB). No importa: el lienzo propio parte del
  píxel que devolvió mpv, y se verificó que el redibujado tras `sub-add` da
  exactamente el mismo fondo (0 píxeles >8 en el caso `basico`, cuyo fondo ocupa
  la mayor parte del recorte).

- **Cargo 1.100 mudó los binarios de test y el primer CI en Windows no
  arrancó.** `STATUS_DLL_NOT_FOUND` en `target/debug/build/rod-subs/<hash>/out/
  rod_subs-<hash>.exe`. Desde 1.100 los binarios de test de un paquete con build
  script viven ahí, y ese `<hash>` **no** es el `OUT_DIR` del build script (otro
  hash), así que las DLLs de libass que `build.rs` copia junto a `deps/`,
  `examples/` y el perfil no quedan al lado. Y el import de `ass-9.dll` es
  estático: el proceso no llega a `main`. Se reproduce en local con
  `rustup toolchain install 1.100.0` y se arregla por PATH
  (`$env:GITHUB_PATH` con `vcpkg/installed/x64-windows/bin` en el CI), no
  copiando a `OUT_DIR` como intenté primero: comprobado que no coincide.
  Anotado en `build.rs`. Es un argumento a favor de enlazar libass **estáticamente**
  (`x64-windows-static-md`) en el slice 8: se lleva por delante las DLLs, el PATH
  y el baile de copias.

## Qué NO hacer en este slice

- No comparar contra el frame de un video real en el test. Eso ya lo hace
  `examples/componer.rs` con `testdata/clip-1080p.mp4` (slice 0b, 0.000 %); aquí
  el objetivo es que la comparación corra sola y sin archivos pesados.
- No bajar la tolerancia a 0. El redondeo de la mezcla da 1-2/255 incluso cuando
  el render es correcto; exigir igualdad exacta convierte el test en un detector
  de compiladores.
- No hacer que el test genere el golden con mpv. Necesitaría libmpv y un video en
  cada `cargo test`; el golden se regenera a mano cuando cambia el fixture o la
  versión de libass, y ese acto queda en el commit.
- No portar el golden a Linux "arreglándolo" con la fuente del sistema: la
  comparación no vale entre fuentes distintas (spec §6). Ahí corren las pruebas
  estructurales, y el día que la app reparta una tipografía propia se podrá
  comparar en los dos sistemas.
