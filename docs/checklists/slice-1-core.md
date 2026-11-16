# Slice 1 — `crates/core` contra el oráculo

Objetivo: portar la semántica propia de OpenDialogue (tiempo, subtítulos, ASS,
solapes, islas de audio, búsqueda) a `crates/core`, usando **las pruebas de la app
Tauri como especificación**. Al terminar, cada función portada tiene su caso del
oráculo escrito como test de Rust y la suite de `rod-core` debería estar a la
altura de los ~306 tests de `C:/Dev/OpenDialogue`.

## Reglas de este slice

1. **La prueba del oráculo va primero.** Se copia el caso (`input → esperado`) a
   `#[test]` en Rust y se ve fallar. Si el código ya pasa, el test se queda igual:
   congela el comportamiento.
2. **El oráculo se puede ejecutar.** `cd C:/Dev/OpenDialogue && npm test` (vitest
   ya está instalado). Ante una duda de redondeo, regex o caso raro, se escribe un
   test temporal en el oráculo y se le pregunta a él, en vez de adivinar.
3. **Divergencia = bug**, salvo que se anote aquí y en `BITACORA.md` con el motivo.
4. Sin `unwrap`/`panic!` en código de producción; en tests se permite `expect`.
5. Cada paso cierra con `cargo fmt --all -- --check`, `cargo clippy --all-targets
   -- -D warnings` y `cargo test -p rod-core`.

## Inventario

| módulo Rust | oráculo | tests oráculo | estado |
|---|---|---|---|
| `time.rs` | `time.ts` | 7 | portado, faltan 3 asserts literales |
| `srt.rs` | `srt.ts` | 17 | parcial: falta `nombreArchivoHablanteUnico` |
| `editing.rs` | `captions.ts` + `captionTiempos.ts` | 21 | parcial: falta `findSnapBlockDelta` |
| `islands.rs` | `audioIslands.ts` | 19 | funciones completas, cobertura 3/19 |
| `ass.rs` | `ass.ts` | 79 | 16 funciones portadas, cobertura 5/79 |
| — | `busqueda.ts` | 18 | sin portar |
| — | `autosubs.ts` | 13 | sin portar |
| — | `selection.ts` | 19 | sin portar |
| — | `assPresets.ts` | 17 | sin portar (necesita serde/JSON) |
| — | `colorHablante.ts` + `clipboard.ts` | 14 | sin portar |
| — | `export-nombres.test.ts` | 16 | depende de `srt`/`ass` (nombres de archivo) |

**No se portan** (y por qué):
- `assPreview.ts` — calcula CSS para el overlay HTML del `<video>`. Aquí el
  preview es el render de libass (invariante de `AGENTS.md`): si el preview y el
  export divergen es un bug, así que imitar el overlay en CSS sería una segunda
  fuente de verdad.
- `ajustes.ts`, `recovery.ts`, `rutaVideo.ts`, `urlVideo.ts`, `fuentes.ts` — IO de
  Tauri (leer `ajustes.json`, `appDataDir`, `asset://`). En la app nativa eso cae
  en `crates/app`, no en `core`; se abordan cuando exista la app.
- `constants.ts` — son valores; van como constantes donde se usen, no como módulo.

## Pasos

- [x] **1. `time` completo.** Añadidos los asserts que faltaban (`00:01:30` → 90,
      `01:01:01` → 3661). El módulo ya cubría los 7 casos del oráculo; queda como
      plantilla del patrón: el caso del test viaja con un comentario que dice de
      qué `it` viene.

- [x] **2. `srt`.** 17 casos de `srt.test.ts` + 11 de `export-nombres.test.ts`
      portados. Dos bugs reales del port anterior, encontrados por los casos:
      - un bloque con tiempo y sin texto se descartaba; el oráculo lo conserva con
        `texto: ""` (un cue en blanco es un cue, no basura);
      - un bloque de **una** línea sí se descartaba en el oráculo
        (`lineas.length < 2`) y nosotros lo convertíamos en cue vacío.
      Añadido `nombre_archivo_hablante_unico` con las tres reglas que rompen el
      export: caracteres prohibidos de Windows → `_`, dispositivos reservados
      (`CON`, `NUL`, `COM1-9`, `LPT1-9`, con `¹²³`) → prefijo `_`, y
      deduplicación **sin distinguir mayúsculas** (`Ana.srt` y `ana.srt` son el
      mismo archivo).
      *Divergencia conocida*: `parse_srt_time` nuestro exige que el timestamp
      empiece en el primer token; el regex del oráculo acepta basura delante
      (`"x00:01:30,500"`). Somos más estrictos a propósito.

- [x] **3. `editing`.** 21 casos de `captions.test.ts` + `captionTiempos.test.ts`
      más los 6 de `clampDeltaBloque`. Añadido `find_snap_block_delta`. Tres
      divergencias corregidas:
      - `format_overlap_report` usaba `->` y `<->`; el oráculo usa `→` (U+2192) y
        `↔` (U+2194), y el texto del reporte se compara carácter a carácter;
      - el mensaje vacío era `"Sin solapes"`; es
        `"No se encontraron subtítulos solapados.\n"` (`es.json`) — con salto de
        línea incluido;
      - el nombre del hablante caía al **id** cuando nombre y tecla estaban
        vacíos; el oráculo cae a `"Sin hablante"`. Mostrar un id interno al
        usuario no es un fallback, es una fuga.
      *Pendiente de i18n*: las dos cadenas están hardcodeadas en `core`; el
      oráculo las saca de `es.json`. Cuando exista i18n en la app se mueven.


- [ ] **4. `islands`.** Portar los 19 casos de `audioIslands.test.ts`.

- [ ] **5. `ass`, parte pura.** Colores (`hexToAssColor`, `assColorToHex`),
      tiempos (`formatAssTime`, `parseAssTime`), escapado
      (`escapeAssText`/`unescapeAssText`) y `sanitizeNombreDialogo`.

- [ ] **6. `ass`, parte estructural.** `segmentarPorSolape`, `overridesDeLinea`,
      `fusionarLineas`, `buildAss`, `parseAss`, `presetDesdeEstilos`,
      `presetParaExportar`, `lineasOverlayActivas`. Es el bloque grande (79
      casos): si hace falta se parte en dos pasos con la checklist al día.

- [ ] **7. `busqueda`.** 18 casos. Va antes que `selection` porque la lista los
      usa juntos.

- [ ] **8. `autosubs`.** 13 casos: parseo del `.txt` de turnos y asignación de
      hablantes por texto.

- [ ] **9. `selection`.** 19 casos: navegación por filas, carriles y marquesina.

- [ ] **10. `assPresets`.** 17 casos. Decide aquí si entra `serde_json` (está en
      el grafo por iced) o un parser propio.

- [ ] **11. `colorHablante` + `clipboard`.** 14 casos pequeños de formato de texto.

- [ ] **12. Cerrar.** `cargo test -p rod-core` en verde con la cuenta de tests
      portados, diferencia contra los 306 del oráculo anotada aquí, y las
      divergencias (si hay) en `BITACORA.md`.
