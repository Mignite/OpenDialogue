# Rediseño: re-skin negro puro + 3 tabs + creador de presets con preview

Fecha: 2026-10-11
Estado: aprobado por el usuario (secciones de diseño presentadas en chat)
Alcance: **diseño + un cambio de modelo de color**. §6 elimina `Hablante.color`
como fuente de verdad — es la única sección que toca lógica de datos, y existe
porque `Hablante.color` nunca llegaba al archivo exportado.

---

## 1. Objetivo

Llevar la identidad visual de OpenDialogue a la dirección del estudio
`design/estudio-cuadrado-precision.html`: negro puro, hairlines de 1px, radios de
2px, tipografía técnica en versalitas mono. Sobre esa base, reemplazar los dos
acordeones de la columna derecha por **3 tabs** y reconstruir el editor de presets
`.ass` como un inspector con **preview en vivo**.

El preview no es una imitación del render del video: es **el mismo código**.

## 2. Fuera de alcance

Decisiones tomadas explícitamente durante el brainstorming. No se implementan, y
no deben colarse "de paso":

| Fuera | Por qué |
|---|---|
| Ruler del timeline con ticks | El usuario no la quiere; el grid del canvas ya cumple |
| Menú HTML en topbar | El menú ya es nativo de Tauri (`app.set_menu`, lib.rs:597-602); el header HTML se borró el 2026-08-15 por ser espacio muerto |
| VU meter | Requiere WebAudio `AnalyserNode`; hoy no hay fuente real del dato |
| Badge de FPS / "SMPTE DROP" | El FPS no está disponible sin ffprobe, que ya no es requisito |
| `LATENCIA: 0 ms` | Dato inventado |
| Métrica de cps | Feature, no diseño — el usuario la descartó |
| Presets `.ass` como estado del proyecto | `--accent` de mostaza **no** se propaga a los hablantes; el color del timeline viene del preset (§6) |
| Migrar los `Hablante.color` guardados a presets | Imposible sin inventar N presets nuevos. Fuera de alcance — ver Riesgo en §9 |

## 3. Tokens

### 3.1 Por qué no es solo cambiar un hex

`--warn` significa **cuatro cosas** en la app hoy:

| Uso | Selector |
|---|---|
| Cambios sin guardar | `.saveState.dirty` (App.css:1147) |
| Video no encontrado | `.missingPath` (:281) |
| Fuente no instalada | `.assFontWarn` (:1932) |
| Resaltado de búsqueda | `.captionMark` (:2123) |

El mostaza elegido (`#D4A017`) está a **ΔE 15.5** del ámbar actual `#E8C34E`.
Si el acento pasa a mostaza sin mover `--warn`, "tenés cambios sin guardar" y
"el foco está acá" se vuelven el mismo color.

### 3.2 Resolución (opción A, aprobada)

El dial cálido queda `[mostaza, naranja quemado, rojo]` en lugar de
`[coral, ámbar, rojo]`. Cada paso baja de tono y sube de calor, así que los tres
son distinguibles de un vistazo.

### 3.3 Valores finales

**Acento y semánticos**

| Token | Antes | **Después** | Razón |
|---|---|---|---|
| `--accent` | `#E85D4E` | **`#D4A017`** | Mostaza. 8.84:1 sobre negro vs 6.11:1 del coral: **+45% de legibilidad** en el texto de los clips del timeline, que se pinta con el color del hablante |
| `--accentSoft` | `rgba(232,93,78,.14)` | **`rgba(212,160,23,.14)`** | Sigue al acento |
| `--warn` | `#E8C34E` | **`#E8683C`** | Naranja quemado. ΔE 25.7 contra el acento; era 15.5 |
| `--danger` | `#F87171` | **`#FF4D4D`** | Rojo puro, se separa de `--warn` |
| `--info` | `#4EA8E8` | `#4EA8E8` | Sin cambio |
| `--success` | `#7ED957` | `#7ED957` | Sin cambio |

**Superficies** (el negro puro es la dirección; los cinco escalones dan lugar a las
capas de superficie que el modelo actual ya usa)

| Token | Actual | **Después** |
|---|---|---|
| `--bg` | `#0e0f11` | **`#000000`** |
| `--surface` | `#131416` | **`#08090C`** |
| `--surface2` | `#16181b` | **`#101318`** |
| `--surface3` | `#1a1c1f` | **`#191D24`** |
| *(nuevo)* `--surfaceHover` | — | **`#1F242C`** |

**Bordes**

| Token | Actual | **Después** |
|---|---|---|
| `--border` | `#1e2024` | **`#1C212B`** |
| `--borderStrong` | `#2c2f36` | **`#2A313D`** |
| *(nuevo)* `--borderBright` | — | **`#3A4353`** |

**Texto** — todos los ratios contra `#000000`

| Token | Uso | Actual | **Después** | Ratio |
|---|---|---|---|---|
| `--text` | Primario | `#e8eaed` | **`#EDEDF0`** | 16.1:1 |
| `--textDim` | Secundario | `#9a9ea6` | **`#9AA1AE`** | 8.1:1 |
| `--textFaint` | Timestamps, etiquetas TC, `#` | `#55585e` (2.94:1) | **`#737B8A`** | **4.93:1** |
| *(nuevo)* `--textHint` | Hints de statusBar, atajos | — | **`#5A6272`** | 3.42:1 |

> `--textFaint` sube de 2.94:1 a 4.93:1. Los timecodes son data que se lee
> constantemente y WCAG pide 4.5:1 para texto normal. El estudio original bajaba
> este valor a 2.59:1 — eso queda corregido.
>
> `--textHint` a 3.42:1 queda **por debajo** de 4.5:1 a propósito: solo se usa para
> hints de teclado en el statusBar, nunca para conveying información. Si en la
> implementación aparece un uso de `color`, sube a `--textFaint`.

**Geometría** — `--r-sm/md/lg/xl` pasan de 5/6/6/8px a **2px** en los cuatro.

### 3.4 Tipografía

Sin cambios. Se conserva la pila actual: Space Grotesk (display), Inter (body),
JetBrains Mono (datos). La dirección "técnica" la produce el **uso** (mayúsculas
con `letter-spacing` en los labels estructurados, mono en todo dato numérico), no un
cambio de familia.

**Decisión sobre densidad:** el estudio dibuja todo a `13px` base; la app está en
`15px`. Bajar la raíz a 13px afecta cada tamaño derivado. La densidad se
consigue **tightening de padding y gap**, no de tamaño de fuente. Se mantiene
`font-size: 15px` en `:root` y se ajustan espaciados.

### 3.5 El video deja de ser negro sobre negro

Con `--bg: #000000`, el `<video>`, su letterbox y el fondo de la app son el mismo
negro y lo único que separa la imagen del resto es un hairline de 1px: el frame
se derrama y desaparece.

`.videoWrap` pasa a `background: #08090C` con `border: 1px solid #1C212B`. El video
recupera un asiento visible.

## 4. Las 3 tabs

### 4.1 Estado

`paneActivo: "subs" | "hablantes" | "estilos"` en `App.tsx`, reemplaza los booleanos
`panelHablantesAbierto` y `panelEstilosAbierto`.

**No se persiste.** Siempre arranca en `subs`: es donde se abre la IMM de las
sesiones, y un tab que aparece en un estado distinto al que dejaste es ruido.

### 4.2 Componente nuevo

`src/components/PanelTabs.tsx`:

```tsx
interface Props {
  activo: PaneId;
  onCambiar: (id: PaneId) => void;
  panes: { id: PaneId; label: string; contador?: number | null }[];
}
```

Botonesbrowser-style: altura 29px, `border-bottom: 2px solid var(--accent)` en el
activo, `top: 1px` para que el borde ativo se coma la línea de la barra. Cada uno
con icono SVG inline `currentColor` (regla del proyecto: **cero emojis**) y un
contador en `.tabCounterBadge` mono.

Contadores: Subtítulos → `captions.length`; Hablantes → `hablantes.length`;
Estilos → `presets.length`. Se ocultan cuando valen 0, no Compiten entre sí.

### 4.3 Paneo Subtítulos

Absorbe el `.rightColHeader` actual (título + contador + toggle de búsqueda) y el
`.searchBox` con sus 4 controles, que hoy viven **sueltos** encima de la lista
(App.tsx:2488-2536). Search, match-case, replace y replace-all entran al tab.
Nada se pierde, nada se mueve de comportamiento.

`CaptionList` solo se monta con `paneActivo === "subs"`. Efecto secundario
deseable: los `rowRefs` de la lista virtualizada se liberan al cambiar de tab.

### 4.4 Paneo Hablantes

`SpeakersPanel` **sin** la envoltura `.speakersAccordion`. Pierde las props
`panelAbierto` y `onTogglePanel`, y con §6 pierde además el `<input type="color">`
— el swatch queda como elemento decorativo leyendo el color derivado.

Se conservan el nombre, la tecla, el select de preset con su `title`, el botón de
eliminar y `onCommit` en `onFocus` (patrón de undo del proyecto).

> El swatch pasa a ser **informativo, no editable**: muestra el color que el
> hablante va a tener de verdad. Como ya no hay nada que editar, un `<input
> type="color">` ahí sería un control que miente sobre lo que hace.

### 4.5 Paneo Estilos

El creador nuevo (§5).

### 4.6 Evento de menú

`lib.rs` emite `gestionar_presets` (menú *Estilos…*, lib.rs:589). Hoy alterna el
acordeón; con tabs debe **setear `paneActivo = "estilos"`**. El caso "ya estaba
abierto, cerralo" desaparece: un tab no se cierra.

### 4.7 i18n

Claves nuevas en `es.json` y `en.json`: `app.tabs.subs`, `app.tabs.speakers`,
`app.tabs.styles`. Los titles de los tabs también necesitan entrada (el proyecto
es bilingüe y la app arranca en `en` por defecto).

## 5. Creador de presets con preview

### 5.1 El principio

El preview del editor y el overlay del video **no pueden divergir**. La
implementación actual ya tiene el render correcto, pero vive **dentro del cuerpo
del componente** en `App.tsx:1977-2019` y está cerrado sobre `overlayEscala`.

Se extrae a un módulo puro compartido:

```
src/utils/assPreview.ts
  estiloLineaPreview(preset, escala): CSSProperties
  estiloCajaPreview(preset, recuadro | null, escala): CSSProperties
```

`recuadro` es `{left, top, width, height}` sobre el área **visible** del video
(calculada con `object-fit: contain` en `overlayRecuadro`, App.tsx:1960-1975);
`null` cuando no hay video cargado.

Los tres efectos de `estiloCajaPreview` son los que hoy están hardcodeados en
`overlayStyle` (App.tsx:1991-2013) y son el bug latente más caro de este refactor:

- `vertical`: `al >= 7 → top`, `al >= 4 → middle`, si no `bottom`
- `horizontal`: `1|4|7 → left`, `3|6|9 → right`, si no `center`
- el padding vertical aplica `marginV` **solo** en la banda correspondiente
  (`paddingTop` si top, `paddingBottom` si bottom); en `middle` no se aplica
  ninguno. Es como funciona libass y es fácil de equivocar al reimplementarlo.

`ASS_EM_PREVIEW = 0.85` se mantiene **intacto**. Calibrado contra mpv el
2026-10-10; la constante está marcada no-tocar-sin-remedir.

`App.tsx` consume las dos funciones. El `estiloCajaPreview` recibe el recuadro del
video; el preview del editor recibe el de su propio stage. **Misma función,
distinto `escala`.**

### 5.2 Stage

`src/components/AssPreviewStage.tsx`. Caja negra 16:9, `ResizeObserver` para
medir y calcular `escala = min(w / 1920, h / 1080)` (misma fórmula que el video:
libass escala por PlayResY, así que manda el mínimo de ambos ejes).

- Fondo `#000000` puro. Es un preview de subtítulo: tiene que ser el frame real.
- Texto de ejemplo **editable**, arranca con el del caption bajo el playhead, o
  `"Texto de ejemplo"` si no hay ninguno.
- Aplica `outline` y `shadow` del preset con la misma escala.
- Responde a resize y a cambios de preset sin `setState` propio: el componente
  padre ya re-renderiza con `presets`.

### 5.3 Layout del inspector

```
┌ ESTILOS ASS                          [+ Nuevo] ─┐
│ ┌ lista ────────┐ ┌ inspector ──────────────────┐ │
│ │ ● Principal   │ │ FUENTE    [Space Grotesk ▾] │ │
│ │ ● Énfasis     │ │ TAMAÑO[24]   BORDE   [2.5]  │ │
│ │ ● Secundario  │ │ ─────────────────────────── │ │
│ │               │ │ Primario [■] #FFFFFF        │ │
│ │               │ │ Borde    [■] #000000        │ │
│ │               │ │ ─────────────────────────── │ │
│ │               │ │   7  8  9                   │ │
│ │               │ │   4  5  6   ← numpad        │ │
│ │               │ │   1 [2] 3                   │ │
│ │               │ │ ─────────────────────────── │ │
│ │               │ │  ┌─ stage 16:9 negro ─────┐ │ │
│ │               │ │  │  Texto de ejemplo      │ │ │
│ │               │ │  └────────────────────────┘ │ │
│ └───────────────┘ └─────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

- **Lista de presets** a la izquierda: swatch + nombre, más `+ Nuevo` y
  `Eliminar`. El `agregarPreset` actual clona del seleccionado (no arranca de
  cero) y autonombra via `nombreLibre` — **se conserva**.
- **Fuente**: el picker con `assFontPicker` actual (input que se pinta en su
  propia fuente + caret + lista filtrada + aviso de "no instalada"). **Se
  conserva entero**: la lógica de `fuenteResuelve` y su medición en DOM ya están
  resueltas y testeadas a mano.
- **Tamaño / Borde**: `fontsize` y `outline` numéricos. `shadow` también numérico.
- **Márgenes**: `marginL`/`marginR`/`marginV` en una fila de 3. Son campos ASS
  reales y el preview nuevo los hace **visibles por primera vez**: moverlos
  mueve el texto en el stage.
- **Numpad 3×3** reemplaza el `<select>` de alineación actual
  (StylesPanel.tsx:285-297). Botones `aria-pressed`, el activo en `--accent`.
  Etiqueta debajo con el nombre de la alineación (sale de i18n, claves
  `assExport.align_*` que ya existen).
- **Preview** al pie del inspector, ancho completo.

### 5.4 El campo de sombra

El estudio dibuja un swatch "Sombra" junto a Primario y Borde. **No se implementa
como color**: `PresetAss` no tiene `shadowColor` y el formato `.ass` tampoco —
V4+ usa `OutlineColour` para el drop shadow. El estudio se equivoca en esto. Solo
`shadow` (la profundidad en píxeles) queda como campo numérico.

### 5.5 Autoguardado

Intacto. `persistirPresets` (useAssEstilos.ts:52-84) con trailing debounce de
400 ms, flush al desmontar, `ya:true` para el import. **No hay draft ni
confirmación de descarte** — no se puede perder trabajo, porque nunca se guarda
tarde.

`StylesPanel` se reescribe pero **no cambia su contrato**: sigue recibiendo
`presets` y `onGuardar`. El debounce y el flush siguen en el hook, no en el panel.

## 6. El color del hablante se deriva del preset (fin de `Hablante.color`)

### 6.1 El hallazgo

`Hablante.color` **nunca llega al archivo `.ass`**. Todo el camino de color en el
export resuelve por preset:

```
presetDe(hablanteId) → lineaStyle()  (ass.ts:193)
                     → overridesDeLinea()  (ass.ts:130-131)
                     → línea overlay  (ass.ts:475)
                     → preset.color  (ass.ts:158, 412)
```

`buildAss` recibe `hablante: Hablante[]` y **solo lee `h.id`**. El campo describe
un color que el formato de salida no tiene concepto.

Consecuencia observable hoy: se puede tener H1 en coral en el panel de hablantes
y que el `.ass` salga blanco. **La UI afirma algo que el archivo no dice.** Es la
misma clase de bug que el flag `dirty`/`saved` desincronizado ya corregido en el
2026-10-10: estado local que miente sobre el destino real.

### 6.2 Resolución

El color de un hablante **se deriva del preset que tiene asignado**. Sin preset,
blanco.

```ts
colorHablante(h: Hablante, presets: PresetAss[]): string {
  return presets.find(p => p.id === h.presetId)?.color ?? "#FFFFFF";
}
```

Una sola fuente de verdad. La UI y el `.ass` no pueden discrepar porque leen lo
mismo.

`Hablante.color` **permanece en `types.ts` y en el `struct` de Rust**, sin leer.
No se borra: los `.cdp` guardados lo traen en el JSON y quitarlo del `struct`
rompería la deserialización de proyectos existentes. Queda como campo muerto,
documentado como tal.

### 6.3 Qué se elimina

| Pieza | Dónde | Nota |
|---|---|---|
| `PALETA` | `constants.ts:17-27` | Muere. Solo la usaban `autosubs.ts:92` y `useHablantes.ts:52` para colorear hablantes nuevos |
| `cambiarColorHablante` | `useHablantes.ts:72` | Sin consumidor |
| `<input type="color">` | `SpeakersPanel.tsx:96-104` | El swatch queda, decorativo, leyendo el color derivado. Ver §4.4 |
| `normalizarColor` | `SpeakersPanel.tsx:18-28` | Era solo para el `<input type="color">` |
| Test de convención PALETA | `autosubs.test.ts:94-109` | Se reescribe: verifica que el hablante nuevo no trae color, no que toma PALETA |

**El test de roundtrip se mantiene** (`ass.test.ts:444,641`): `Hablante.color`
sigue viajando en el JSON, solo dejó de leerse en la UI.

### 6.4 Consecuencia aceptada: instalación nueva monocromática

`DEFAULT_PRESET_ASS.color = "#FFFFFF"` (constants.ts:55). Una instalación nueva
tiene un preset blanco → **todos los carriles blancos**, distinguidos por
posición y por el label del carril, no por color.

El color-coding pasa a ser algo que se **arma a propósito**: un preset por
hablante. Es el mismo trabajo que el `.ass` exige igual, así que no se pierde
capacidad real — pero cambia el flujo de quien hoy marca 9 hablantes con los
atajos 1-9 sin tocar jamás Estilos.

**Decisión explícita del usuario**, consciente del costo.

### 6.5 El color entra en los panes

`trackRows` (App.tsx:2055-2069) cambia de `color: h.color` a
`color: colorHablante(h, presetsAss)`. Como `trackRows` es `useMemo` sobre
`[carrilesAgrupados, hablantes]`, **suma `presetsAss` a las deps**: editar el
color de un preset tiene que repintar los carriles. Es el mismo bug de deps que
se corrigió en el slice 3 con `proyecto` (§ Riesgo del dirty-effect).

Los tres consumidores restantes del color:

| Consumidor | Cambio |
|---|---|
| `CaptionList.tsx:106` `borderLeftColor` | Color derivado |
| `CaptionList.tsx:117` `captionSpeakerTag` | Color derivado |
| `App.tsx:2405` dot del chip del editor | Color derivado |

### 6.6 El borde del clip se tiñe

Hoy: `background: color-mix(in srgb, ${color} 18%, transparent)` +
`border: 1px solid ${color}` (App.tsx:2664-2665).

Con `#FFFFFF` eso deja un **borde blanco sólido de 1px en cada clip** — más
brillante que el playhead sobre negro, y en una UI que se define por monolítica
eso es exactamente lo contrario.

```
background: color-mix(in srgb, ${color} 18%, transparent);
border:     1px solid color-mix(in srgb, ${color} 45%, transparent);
```

El 45% mantiene el borde legible sin que el blanco grite. El borde del clip
seleccionado pasa a `2px solid var(--accent)` para que la selección se lea por
encima del tinte.

### 6.7 El carril "—" no se toca

`{ label: "—", color: "#4a4853" }` (App.tsx:2059) es **nadie asignado**, no
*alguien sin preset*. Si pasa a blanco se vuelve indistinguible de un hablante
real con el preset default. Se queda en gris neutro `#3A3E48`.

### 6.8 Qué se regresa del §8

El riesgo de §8 sobre H4/H8 cerca del mostaza **desaparece**: la `PALETA` no
llega al timeline. Solo quedan H4/H8 como colores de hablante en el panel, que ya
no compiten con nada.

## 7. Corrección sobre el estudio

`IN` / `OUT` se quedan como **inputs numéricos editables**
(App.tsx:2333-2372), no como `<b>` de solo lectura. Existen desde el commit
`ee04b7b` con el motivo explícito: *"el timing es el 80% del oficio y necesitaba un
input numérico"*.

Único agregado: `DUR` como lectura a la derecha de los dos (`fin - inicio`, 2
decimales, vía `formatTime`). Es información, no un tercer input.

## 8. Verificación

**El proyecto no tiene jsdom ni testing-library.** Los ~250 tests vitest pasan
igual si el drag del timeline o el tab se rompen. Eso significa que:

- `npm run build` (typecheck) y `npm test` son **puertas de forma, no de comportamiento**.
- La verificación real de tabs, numpad, preview, contraste y colores derivados es
  **manual, sobre la app corriendo**.

### 8.1 Automático

```bash
npm run build     # typecheck + vite build
npm test          # vitest
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

`assPresets` (nombreLibre, esPresetAss) y `ass` deben seguir verdes sin
modificación: el contrato de `PresetAss` no cambia.

**Tests que sí cambian** (§6.3): `autosubs.test.ts:94-109` — la convención de
PALETA desaparece. Se reescribe para verificar que `agregarHablante` **no** asigna
color.

### 8.2 Manual (obligatorio)

1. **Trampa conocida:** un dev server viejo en el puerto 1420 devuelve HTTP 200
   con el bundle **viejo**. Matar cualquier proceso en 1420 antes de arrancar.
2. Los 3 tabs cambian de contenido; Subtítulos conserva búsqueda + replace.
3. El menú *Estilos…* (nativo) salta al tab de Estilos.
4. En el numpad, cada una de las 9 posiciones mueve el texto en el stage a la
   esquina correspondiente — es la prueba de que §5.1 compartió el código bien.
5. `ASS_EM_PREVIEW` sin tocar: el preview sobre el video se ve **igual** que
   antes del refactor. Cualquier cambio de tamaño es un bug.
6. Margenes mueven el texto en el stage.
7. Cerrar la ventana con un cambio de estilo pendiente (< 400 ms) lo persiste —
   el flush del hook.
8. Contraste: timestamps de la lista legibles, dirty state distinguible del
   acento a simple vista.
9. El video tiene marco visible sobre el fondo.

**Derivación de color (§6) — la que más fácil falla en silencio:**

10. Cambiar `preset.color` repinta **los carriles** sin recargar. Si no, es el
    bug de deps de `useMemo` (§6.5) y está mirando el memo viejo.
11. Un hablante **con** preset de color muestra el color del preset en el carril,
    el tag de la lista y el chip del editor.
12. Un hablante **sin** preset muestra blanco en los tres lugares.
13. El carril `"—"` (sin asignar) sigue gris y **no** se confunde con un hablante
    blanco.
14. Los clips blancos **no** tienen borde blanco sólido: el tinte al 45% se lee
    como superficie, no como contorno.
15. Abrir un proyecto viejo: los hablantes con colores guardados pero sin preset
    salen blancos, sin crash. El campo viaja en el JSON pero no se lee.
16. Exportar un `.ass` y abrirlo: **el color del texto de cada hablante coincide
    con el color de su carril en el timeline**. Ese es el criterio de aceptación
    de toda la sección — la UI y el archivo dicen lo mismo.

## 9. Riesgos

| Riesgo | Mitigación |
|---|---|
| Extraer `estiloCajaPreview` cambia el render del video | Es una **extracción** pura, no una reescritura. El diff se lee contra `App.tsx:1991-2013`. §8.2.5 es el test |
| `CaptionList` desmontada pierde la posición de scroll al volver al tab | Aceptado. El follow-scroll del caption activo lo reposiciona en el primer render |
| Editar `preset.color` no repinta los carriles | `presetsAss` entra a las deps del `useMemo` de `trackRows` (§6.5). Test manual §8.2.10 |
| Un proyecto viejo pierde los colores que el usuario eligió a mano | **Riesgo real, aceptado.** No hay migración posible sin contradecir el modelo: si se preservara el color guardado, la UI volvería a mentir sobre el `.ass`. El color se recupera asignando un preset con ese color |
| Instalar y no saber por qué el timeline es blanco | §6.4 lo documenta y el §8.2.12 lo verifica. El preset con color se crea desde el tab Estilos, que ahora es visible en la tab bar |
| 5 tokens nuevos/renombrados rompen reglas huérfanas | `borderBright`, `surfaceHover`, `textHint` se agregan; `accent`/`warn`/`danger` cambian de valor. Buscar usos al final |
| `PALETA` queda importada en un archivo y `no-unused` lo-take en clippy | TS no falla por imports muertos en el build de Vite, pero `cargo clippy` sí es estricto con el lado Rust. Buscar referencias a `PALETA` antes de borrar |
