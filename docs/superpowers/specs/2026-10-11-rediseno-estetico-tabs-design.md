# Rediseño: re-skin negro puro + 3 tabs + creador de presets con preview

Fecha: 2026-10-11
Estado: aprobado por el usuario (secciones de diseño presentadas en chat)
Alcance: **solo diseño**. Cero cambios de arquitectura, cero features nuevas.

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
| Presets `.ass` como estado del proyecto | `--accent` de mostaza **no** se propaga a `PALETA` |

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
con `letter-spacing` en labels-structured, mono en todo dato numérico), no un
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
`panelAbierto` y `onTogglePanel`. Se conserva todo lo demás: el color con picker,
el nombre, la tecla, el select de preset con su `title`, el botón de eliminar y
`onCommit` en `onFocus` (patrón de undo del proyecto).

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

## 6. Corrección sobre el estudio

`IN` / `OUT` se quedan como **inputs numéricos editables**
(App.tsx:2333-2372), no como `<b>` de solo lectura. Existen desde el commit
`ee04b7b` con el motivo explícito: *"el timing es el 80% del oficio y necesitaba un
input numérico"*.

Único agregado: `DUR` como lectura a la derecha de los dos (`fin - inicio`, 2
decimales, vía `formatTime`). Es información, no un tercer input.

## 7. Verificación

**El proyecto no tiene jsdom ni testing-library.** Los ~250 tests vitest pasan
igual si el drag del timeline o el tab se rompen. Eso significa que:

- `npm run build` (typecheck) y `npm test` son **puertas de forma, no de comportamiento**.
- La verificación real de tabs, numpad, preview y contraste es **manual, sobre la
  app corriendo**.

### 7.1 Automático

```bash
npm run build     # typecheck + vite build
npm test          # vitest
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Los tests de `assPresets` (nombreLibre, esPresetAss) y `ass` deben seguir verdes
sin modification: el contrato de `PresetAss` no cambia.

### 7.2 Manual (obligatorio)

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

## 8. Riesgos

| Riesgo | Mitigación |
|---|---|
| Extraer `estiloCajaPreview` cambia el render del video | Es una **extracción** pura, no una reescritura. El diff se lee contra `App.tsx:1991-2013`. §7.2.5 es el test |
| `CaptionList` desmontada pierde la posición de scroll al volver al tab | Aceptado. El follow-scroll del caption activo lo reposiciona en el primer render |
| Mostaza sobre clips de hablante | H4 (`#E8C34E`) y H8 (`#F67E2C`) están en la misma familia. Se **documenta** y no se toca `PALETA`: el acento no se propaga a los hablantes |
| 5 tokens nuevos/renombrados rompen reglas huérfanas | `borderBright`, `surfaceHover`, `textHint` se agregan; `accent`/`warn`/`danger` cambian de valor. Buscar usos al final |
