# RustyOpenDialogue

Reescritura nativa de OpenDialogue en Rust. Slices pequeños y verificables; el
agente implementa y el autor del proyecto tiene que poder **explicar** cada
decisión (ver «Modo de trabajo»).

- **Diseño y motivos**: `docs/superpowers/specs/2026-11-11-rustyopendialogue-design.md`.
  Leerlo antes de proponer cambios de arquitectura.
- **Stack**: iced 0.14 + libmpv (render SW → textura wgpu) + libass por FFI propio.
- **Workspace**: manifiesto virtual; directorios `crates/{core,subs,player,app}`,
  paquetes `rod-core`, `rod-subs`, `rod-player`, `rod-app`. Nunca un paquete
  llamado `core`: tapa el `core` del sysroot y rompe `core::time` y las macros
  derivadas que expanden `::core` (spec §4).
- **No reintroducir GStreamer**: ya rompió el AppImage de la app Tauri
  (`C:/Dev/OpenDialogue`) por desajuste de plugins core/host.
- **Invariante**: el preview es un render del `.ass` que se exporta (libass). Si
  el preview y el export divergen, es un bug, no un ajuste de estilos.
- **Regla crate vs propio**: lo general (decodificar, sincronizar, buscar,
  rasterizar ASS) va en crates vivas; lo propio (semántica de subtítulos,
  solapes, islas de audio, modelo de edición) se escribe aquí.
- **Oráculo**: la app Tauri sigue viva. Sus 306 tests y sus salidas son la
  especificación al portar `src/utils/*.ts` a `crates/core`.

## Modo de trabajo

Decidido el 2026-11-15. El agente implementa; el criterio del proyecto es que
Eche pueda **explicar el problema y la solución** (qué fallaba, qué alternativas
había, por qué esta), no que escriba cada línea. El modo tutor se retiró.

1. El agente escribe el código de producción, las pruebas y la documentación.
2. Cada paso cierra con evidencia, no con una afirmación: build, `cargo fmt
   --check`, `cargo clippy -D warnings`, tests, y una verificación observable
   (captura, número medido, salida de un comando).
3. Las decisiones no obvias se explican al tomarlas: problema → alternativa
   descartada → por qué esta. Corto, en el momento, no en un ensayo final.
4. Nada de cajas negras: si una pieza no se puede explicar en dos frases, está
   demasiado enrevesada. Código explícito antes que genérico; ni traits ni macros
   que ahorren cinco líneas y cuesten una tarde de lectura.
5. Al portar de la app Tauri (`C:/Dev/OpenDialogue`) su comportamiento es la
   especificación. Si nos desviamos, el motivo queda escrito en la spec o en
   `BITACORA.md`.
6. No usar `unsafe`, macros complejas ni crates fuera de la spec sin justificar
   antes qué problema resuelve.
7. Los archivos de `docs/checklists/` son el plan del día; la spec es el plan
   macro. Cada tropiezo de más de 20 minutos va a `BITACORA.md`.
