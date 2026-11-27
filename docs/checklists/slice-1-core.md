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
| `time.rs` | `time.ts` | 7 | portado, 7/7 |
| `srt.rs` | `srt.ts` | 17 | portado, 17/17 |
| `editing.rs` | `captions.ts` + `captionTiempos.ts` | 21 | portado, 21/21 |
| `islands.rs` | `audioIslands.ts` | 19 | portado, 19/19 |
| `ass.rs` | `ass.ts` | 80 | portado, 80/80 |
| — | `busqueda.ts` | 18 | portado, 18/18 |
| — | `autosubs.ts` | 13 | portado, 13/13 |
| — | `selection.ts` | 19 | portado, 18/19 (la última prueba identidad de referencias) |
| — | `assPresets.ts` | 17 | portado, 17/17 (con `serde_json`) |
| — | `colorHablante.ts` + `clipboard.ts` | 14 | portado, 14/14 |
| — | `export-nombres.test.ts` | 16 | portado (repartido entre `srt` y `editing`) |

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


- [x] **4. `islands`.** Los 19 casos de `audioIslands.test.ts` portados. El port
      previo era **fiel**: los 18 casos de comportamiento pasan sin tocar el
      código, lo que confirma que las constantes (`FACTOR_UMBRAL` 2.5, `PISO`
      0.01, `HISTERESIS_MUESTRAS` 8, `MAX_DURACION` 5 s, `COLA_SEG` 0.25,
      `LEAD_MAX_SEG` 0.3, `SNAP_ATRAS_SEG` 0.5) estaban bien copiadas.

- [x] **5. `ass`, parte pura.** Colores, tiempos, escapado, `sanitize_nombre_dialogo`,
      bold/italic y `preset_para_exportar`. Dos cosas que hubo que añadir:
      - `preset_por_defecto()`, el `DEFAULT_PRESET_ASS` del oráculo, en `ass.rs`:
        un proyecto nuevo necesita un preset base y las pruebas del oráculo lo
        usan como fixture. Es la misma decisión que `constants.ts` allá.
      - **bug real** en `sanitize_nombre_dialogo`: reemplazar la coma por un
        espacio dejaba doble espacio (`"Smith, John"` → `"Smith  John"`). El
        oráculo colapsa `,\s*` en **un** espacio. La coma es el separador de
        campos del `Dialogue:`, así que esto se ve en el reproductor.

- [x] **6. `ass`, parte estructural.** Los 8 bloques del oráculo (`buildAss`,
      `parseAss`, `segmentarPorSolape`, `fusionarLineas`, `round-trip`,
      `presetDesdeEstilos`, `lineasOverlayActivas`). Solo un bug, y gordo:
      - **`parse_ass` perdía las líneas 2..n de un `Dialogue` multilínea.** La
        continuación se pegaba a un **clon** del evento (el que hacía de
        "último"), no al evento guardado en `events`. El `.ass` de Aegisub, que
        es el formato normal de entrada, entraba truncado a la primera línea.
        Ahora `last_event` es un índice dentro de `events`.
      *Desviación consciente*: `fusionar_lineas` con `\N` entre líneas y los
      overrides solo por campos que difieren del base; comparar `f64` con `!=`
      es seguro aquí porque los presets vienen de los mismos valores, no de
      aritmética acumulada.

- [x] **7. `busqueda`.** 18 casos. Se implementa sobre `Vec<char>` y no sobre
      bytes: un índice de byte en medio de una `ñ` no es límite válido y cortaría
      el texto al resaltar. El matching sigue siendo literal (`contains`), no
      regex, para que `(` o `[` en el término no exploten.

- [x] **8. `autosubs`.** 13 casos. Dos decisiones:
      - `normalizar_texto` reproduce `NFD` + borrar diacríticos con una tabla de
        letras latinas en vez de traer una tabla Unicode entera. Cubre el caso
        real (español y demás lenguas romances) sin dependencia nueva.
      - `hablantes_desde_nombres` genera el id con milisegundos **más un
        contador atómico**: dos altas en el mismo milisegundo darían el mismo id
        y una pisaría a la otra (`nuevo_preset` hace lo mismo).

- [x] **9. `selection`.** 18 de 19 casos. El que falta
      (*"no colapsa captions mutados: usa las referencias del array recibido"*)
      prueba **identidad de referencias** de JS: en Rust la agrupación devuelve
      copias y la prueba no tiene equivalente. En su lugar, `agrupar_captions_por_carril`
      conserva el orden de entrada de cada carril, que es el requisito que la
      prueba original protegía.

- [x] **10. `assPresets`.** 17 casos. **Decisión: entra `serde_json`** (una sola
      dependencia nueva en `rod-core`). El archivo `ass-presets.json` lo comparte
      la app Tauri; un parser propio sería una segunda fuente de desacuerdo sobre
      el formato, justo lo contrario de lo que pide el invariante de paridad.
      Se valida a mano con `Value` en vez de derivar `Deserialize` porque una
      entrada rota debe **descartarse sin tirar el archivo entero**, y un `bold:
      "sí"` tiene que caer a `false`, no invalidar el preset.

- [x] **11. `colorHablante` + `clipboard`.** 14 casos, todos puros. El
      portapapeles real (leer/escribir) es de la app, no de `core`.

- [x] **12. Cerrar.** `cargo test -p rod-core`: **226 tests** contra los 310 del
      oráculo (304 contados por archivo). Los 240 casos de los módulos que sí se
      portan están cubiertos; los 64 restantes son los siete archivos de la lista
      de "no se portan" (`ajustes` 8, `assPreview` 14, `fuentes` 8, `recovery`
      12, `recoveryHook` 7, `rutaVideo` 11, `urlVideo` 4). La diferencia 240 →
      226 son casos del oráculo que comparten un `it` con varios `assert` o que
      no tienen equivalente en Rust (identidad de referencias, `undefined`), no
      casos sin portar.
