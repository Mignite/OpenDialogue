# Rediseño: negro puro + mostaza, 3 tabs y creador de presets con preview — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Aplicar el re-skin de tokens del estudio "Precisión Monolítica", reemplazar los dos acordeones de la columna derecha por 3 tabs, reconstruir el editor de presets `.ass` con preview en vivo que comparte código con el overlay del video, y hacer que el color del hablante se derive del preset en vez de `Hablante.color`.

**Architecture:** Un solo cambio de tokens en `App.css` (Task 1, reversible sola). La derivación de color es una función pura testeable (Task 2). La extracción del render `.ass` a `src/utils/assPreview.ts` con sus tests (Task 4) es la bisagra: sin ella el preview del editor podría divergir del overlay del video. Las Tareas 6-7 son JSX sin cobertura automática — el proyecto no tiene jsdom, así que se verifican manualmente.

**Tech Stack:** React 19 + TypeScript, Tauri 2 (Rust), vitest, CSS custom properties, canvas 2D para el waveform.

**Spec:** `docs/superpowers/specs/2026-10-11-rediseno-estetico-tabs-design.md`

## Global Constraints

- **Token `ASS_EM_PREVIEW = 0.85`** (`src/utils/constants.ts:10`). Calibrado contra mpv el 2026-10-10. **No tocar sin re-comparar contra mpv en el mismo frame.** Un plan que "mejora" el preview cambiando esta constante está roto.
- **Fuentes locales únicamente.** `src/assets/fonts/` tiene SpaceGrotesk 500/600/700 y JetBrainsMono 400/500/600/700. **Nunca Google Fonts CDN** — la app es offline.
- **`--accent` = `#D4A017`, `--warn` = `#E8683C`, `--danger` = `#FF4D4D`.** Los tres se eligen para que el dial cálido sea distinguible. No reintroducir `#E8C34E` como warn.
- **Nombres en español**, `PascalCase` para tipos, `camelCase` para funciones, `UPPER_SNAKE` para constantes.
- **Nada de emojis en la UI** — SVG inline con `currentColor`.
- **El proyecto no tiene jsdom ni testing-library.** Toda Task sin test unitario se verifica manualmente con la app corriendo (§Review Focus).
- **Cualquier cambio en `App.css` se cierra con `npm run build`** (el gate de typecheck).
- **Nada de `git stash` en subagentes** — pisa trabajo. El AGENTS.md del proyecto ya lo registra como incidente.
- Presets y `Hablante.color` se guardan en disco. **No romper la deserialización de proyectos viejos** (`#[serde(default)]` donde aplique).

## Review Focus

Cinco condiciones que el spec implica pero que ningún test unitario va a cubrir — la app se usa con `npm run tauri`, no con vitest. Cada una va atada a su tarea en los pasos.

1. **`preview` y overlay del video no pueden divergir.** Si el texto del preview del editor se ve más grande o más chico que el del video, §5.1 no compartió el código. Es el criterio de aceptación del rediseño.
2. **Editar `preset.fontsize` debe actualizar el preview sin reiniciar** — un `useState` stale o un `key` mal puesto deja el preview congelado.
3. **El tab activo no debe perder la selección al volver.** Volver a Subtítulos después de estar en Estilos tiene que mostrar la misma fila activa.
4. **Los botones de la matriz de alineación no pueden robar el foco.** El guard global de teclado (App.tsx:1510-1516) corta para `INPUT`/`TEXTAREA`/`SELECT` pero no para `BUTTON`, así que un botón focalizado convierte la tecla siguiente en atajo global: `→` hace seek de +5s, `↓` salta de caption. Probado en Task 5 Step 5.
5. **El filtro de búsqueda tiene que funcionar desde el tab de Subtítulos**, que es donde se mueve el `searchBox` — si el input pierde foco al cambiar de tab, el término se queda a medias.
6. **`ASS_EM_PREVIEW` sin tocar** — el preview sobre el video se ve idéntico antes y después del refactor. Cualquier cambio de tamaño es un bug.

---

## File Structure

| Archivo | Acción | Responsabilidad |
|---|---|---|
| `src/App.css` | modificar | Tokens (`:root`), `PanelTabs`, borde de clip, stage de preview, `.videoWrap` |
| `src/App.tsx` | modificar | `paneActivo`, `trackRows` con color derivado, borrar acordeones, evento de menú, guard de teclado |
| `src/components/PanelTabs.tsx` | crear | Barra de 3 tabs (browser-style) |
| `src/components/SpeakersPanel.tsx` | modificar | Quitar acordeón + quitar `<input type="color">` |
| `src/components/StylesPanel.tsx` | modificar | Inspector con matriz de flechas + preview embebido |
| `src/utils/assPreview.ts` | crear | `estiloLineaPreview` / `estiloCajaPreview` puros y compartidos |
| `src/utils/__tests__/assPreview.test.ts` | crear | Tests de las dos funciones puras |
| `src/utils/colorHablante.ts` | crear | `colorHablante(h, presets)` — la derivación |
| `src/utils/__tests__/colorHablante.test.ts` | crear | Test de la derivación |
| `src/hooks/useHablantes.ts` | modificar | Quitar `cambiarColorHablante` y `PALETA` |
| `src/utils/autosubs.ts` | modificar | `hablantesDesdeNombres` sin `PALETA` |
| `src/utils/constants.ts` | modificar | Borrar `PALETA` |
| `src/utils/__tests__/autosubs.test.ts` | modificar | Reescribir el test de convención PALETA |
| `src/i18n/es.json`, `en.json` | modificar | Claves de tabs y del preview |

**Interfaces producidas por este plan** (las consume la Tarea siguiente):

- `colorHablante(h: Hablante, presets: PresetAss[]): string` — devuelve `presets.find(p => p.id === h.presetId)?.color ?? "#FFFFFF"`.
- `estiloLineaPreview(preset: PresetAss, escala: number): CSSProperties`
- `estiloCajaPreview(preset: PresetAss, recuadro: {left:number, top:number, width:number, height:number} | null, escala: number): CSSProperties`
- `PanelTabs({ panes, activo, onCambiar }: { panes: {id: PaneId; label: string; contador: number|null}[], activo: PaneId, onCambiar: (id: PaneId) => void })` — `PaneId = "subs" | "hablantes" | "estilos"`.

---

## Task 1: Tokens del re-skin

El cambio de mayor impacto visual y el más barato de revertir: un commit, un archivo.

**Files:**
- Modify: `src/App.css:46-86` (bloque `:root`)

**Interfaces:**
- Consumes: nada.
- Produces: los tokens que todas las tareas siguientes leen (`--bg`, `--surface*`, `--border*`, `--text*`, `--accent`, `--warn`, `--danger`, `--r-*`).

- [ ] **Step 1: Reemplazar el bloque de tokens**

En `src/App.css`, sustituir el `:root` de las líneas 46-86 por los valores de §3.3 del spec: `--bg: #000000`, `--surface: #08090C`, `--surface2: #101318`, `--surface3: #191D24`, añadir `--surfaceHover: #1F242C`; `--border: #1C212B`, `--borderStrong: #2A313D`, añadir `--borderBright: #3A4353`; `--text: #EDEDF0`, `--textDim: #9AA1AE`, `--textFaint: #737B8A`, añadir `--textHint: #5A6272`; `--accent: #D4A017`, `--accentSoft: rgba(212, 160, 23, 0.14)`, `--warn: #E8683C`, `--danger: #FF4D4D`, `--info` y `--success` **sin cambio**; `--r-sm/md/lg/xl` los cuatro a `2px`.

Conservar sin tocar: `color-scheme: dark` (los selects nativos en WebKitGTK se pintan claros sin él), los tres `@font-face` de Space Grotesk y los cuatro de JetBrains Mono, y `font-family: var(--font-body); font-size: 15px; line-height: 1.5` en la raíz.

- [ ] **Step 2: Verificar el build**

Run: `npm run build`
Expected: PASS. Si falla, es un token mal escrito — no un error de tipos de React.

- [ ] **Step 3: Correr los tests como smoke**

Run: `npm test`
Expected: PASS (los tests no tocan CSS; es solo confirmar que no rompiste el build).

- [ ] **Step 4: Verificar el borde del clip y el video sobre el fondo**

`.videoWrap` (App.css ~231) tiene `background: #000` y el `--bg` ahora también es negro: el video se derrama y pierde su marco. Cambiar a `background: var(--surface); border: 1px solid var(--border);` para que la imagen tenga un asiento visible (§3.5).

Además, `.clip.selected` (App.css:491-493) usa `outline: 1.5px solid var(--text)` — sobre un clip con relleno claro el blanco ya no distingue. Cambiar a `var(--accent)`.

Run: `npm run build` de nuevo.
Expected: PASS.

- [ ] **Step 5: Verificar manualmente (arrancar `npm run tauri`)**

Abrir la app y mirar:
  - El fondo se ve negro puro, no gris azulado.
  - Los timestamps de la lista se leen sin forzar (es el fix de `--textFaint`).
  - El video tiene un marco visible contra el fondo — no se derrama.
  - La barra de resize del dock y los bordes hairline se distinguen.
  - El punto de "sin guardar" en el statusBar es naranja quemado, **no** mostaza — si se ven iguales, `--warn` quedó en `#E8C34E`.

- [ ] **Step 6: Commit**

```bash
git add src/App.css
git commit -m "estilo: tokens del rediseno (negro puro, mostaza, contraste corregido)"
```

---

## Task 2: `colorHablante` — la derivación

La pieza con más lógica del plan y la única con test real de comportamiento de negocio. Es **pura** a propósito: la usan 4 superficies distintas y ninguna necesita saber de dónde viene el color.

**Files:**
- Create: `src/utils/colorHablante.ts`
- Create: `src/utils/__tests__/colorHablante.test.ts`
- Modify: `src/App.tsx:2055-2069` (`trackRows`), `:2405` (chip del editor), `:2664-2665` (borde del clip)
- Modify: `src/components/CaptionList.tsx:106,117`
- Modify: `src/components/SpeakersPanel.tsx` (quitar `<input type="color">`)
- Modify: `src/hooks/useHablantes.ts` (quitar `cambiarColorHablante`)
- Modify: `src/utils/autosubs.ts:78-92`
- Modify: `src/utils/constants.ts:17-27` (borrar `PALETA`)
- Modify: `src/utils/__tests__/autosubs.test.ts:94-109`

**Interfaces:**
- Consumes: `Hablante` y `PresetAss` de `src/types.ts`.
- Produces: `colorHablante(h: Hablante, presets: PresetAss[]): string`, y un `speakerMap` cuyos valores tienen `color` ya resuelto (App.tsx lo construye; `CaptionList` lo consume sin cambios).

- [ ] **Step 1: Escribir el test que falla**

`src/utils/__tests__/colorHablante.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { colorHablante } from "../colorHablante";
import { DEFAULT_PRESET_ASS } from "../constants";
import type { Hablante, PresetAss } from "../../types";

const spk = (over: Partial<Hablante> = {}): Hablante => ({
  id: "sp1", nombre: "Ana", tecla: "1", color: "#E85D4E", ...over,
});
const P_ROJO: PresetAss = { ...DEFAULT_PRESET_ASS, id: "p-rojo", nombre: "Rojo", color: "#E85D4E" };
const P_AZUL: PresetAss = { ...DEFAULT_PRESET_ASS, id: "p-azul", nombre: "Azul", color: "#4EA8E8" };

describe("colorHablante", () => {
  it("sin preset devuelve blanco", () => {
    expect(colorHablante(spk(), [])).toBe("#FFFFFF");
  });
  it("con presetId ausente devuelve blanco aunque haya presets", () => {
    expect(colorHablante(spk({ presetId: undefined }), [P_ROJO, P_AZUL])).toBe("#FFFFFF");
  });
  it("con presetId que no existe devuelve blanco (preset borrado a mano)", () => {
    expect(colorHablante(spk({ presetId: "p-fantasma" }), [P_ROJO])).toBe("#FFFFFF");
  });
  it("con preset devuelve el color del preset, NO el color guardado", () => {
    expect(colorHablante(spk({ presetId: "p-azul" }), [P_ROJO, P_AZUL])).toBe("#4EA8E8");
  });
  it("ignora por completo Hablante.color", () => {
    expect(colorHablante(spk({ color: "#FF00FF", presetId: "p-rojo" }), [P_ROJO])).toBe("#E85D4E");
  });
  it("el preset por defecto (blanco) da blanco", () => {
    expect(colorHablante(spk({ presetId: DEFAULT_PRESET_ASS.id }), [DEFAULT_PRESET_ASS])).toBe("#FFFFFF");
  });
});
```

- [ ] **Step 2: Correr el test para verificar que falla**

Run: `npx vitest run src/utils/__tests__/colorHablante.test.ts`
Expected: FAIL con "Cannot find module '../colorHablante'".

- [ ] **Step 3: Implementar `colorHablante`**

`src/utils/colorHablante.ts`:

```ts
import type { Hablante, PresetAss } from "../types";

/** Color efectivo de un hablante: el de su preset .ass, o blanco si no tiene.
 *  `Hablante.color` existe en el tipo solo para leer proyectos viejos — el
 *  formato .ass no tiene noción de color de hablante, así que usar ese campo
 *  haría que la UI dijera un color que el archivo exportado no tiene. */
export function colorHablante(h: Hablante, presets: PresetAss[]): string {
  return presets.find((p) => p.id === h.presetId)?.color ?? "#FFFFFF";
}
```

- [ ] **Step 4: Correr el test para verificar que pasa**

Run: `npx vitest run src/utils/__tests__/colorHablante.test.ts`
Expected: PASS 6/6.

- [ ] **Step 5: Cablear `trackRows` en App.tsx**

En `App.tsx` línea ~2055, `trackRows` pasa a derivar el color:

```ts
color: colorHablante(h, ass.presetsAss),
```

**El `useMemo` de `trackRows` (deps `[carrilesAgrupados, hablantes]`) debe sumar `ass.presetsAss`.** Sin esto, editar el color de un preset no repinta los carriles hasta recargar — es el mismo bug de deps que ya se corrigió con `proyecto` en el dirty-effect. Review Focus #2 de la spec.

El carril `"—"` (sin asignar, línea ~2059) **no cambia**: sigue en `#4a4853` con su etiqueta `—`. Ese gris es el que ya usan el chip del editor (App.tsx:~2417) y .dropTarget (App.css) — no introduzcas un tono nuevo. Es *nadie asignado*, no *alguien sin preset*; si pasa a blanco se confunde con un hablante real sin preset.

- [ ] **Step 6: Cablear los otros tres consumidores**

- `App.tsx:2405` — el dot del `speakerChip` del editor: `sp ? colorHablante(sp, ass.presetsAss) : "#4a4853"`.
- `CaptionList.tsx:106` y `:117` — `borderLeftColor` y el `captionSpeakerTag`. **Decisión: `App.tsx` les pasa el color YA RESUELTO**, no la lista de presets. `CaptionList` sigue recibiendo `speakerMap` sin cambios, pero el `Map` pasa a ser de hablantes con el color ya calculado (`{...h, color: colorHablante(h, presetsAss)}`), de modo que `sp.color` en las líneas 106 y 117 ya devuelve el valor correcto sin tocar `CaptionList` para nada. La derivación ocurre en un solo lugar — App — y `CaptionList` no necesita saber de dónde sale el color.
- `App.tsx:2664` — el `background` del clip sigue siendo `color-mix(in srgb, ${row.color} 18%, transparent)`. **El `border` cambia a `1px solid #6E7480` fijo**, no derivado (ver Review Focus de la spec §6.6: el borde describe "esto es un clip", no "este es el hablante"). El texto del clip (`--text`, 9px) tiene 11.6:1 contra el relleno `#2E2E2E` — el borde a 2.89:1 no compite con él.

- [ ] **Step 7: Quitar el `<input type="color">` de SpeakersPanel**

En `SpeakersPanel.tsx:92-104`, reemplazar el `<label className="speakerDotWrap">` (con su `<input type="color">` y su `onCambiarColor`) por un `<span className="speakerDot">` decorativo cuyo `backgroundColor` es `colorHablante(h, presets)` — la lista de presets **ya viene por prop**, no hace falta pasarla de más. El `title` pasa a explicar que el color viene del preset.

Borrar la función `normalizarColor` (líneas 18-28) — solo existía para normalizar el valor del `<input type="color">`.

- [ ] **Step 8: `AssExportModal.tsx` — 4ª superficie (ENRUTADO desde la review de Task 2)**

`AssExportModal.tsx:66` sigue haciendo `style={{ background: h.color }}` — el campo muerto. **El modal ya recibe `presets` por prop** (línea 6), así que el fix es de una línea:

```tsx
style={{ background: colorHablante(h, presets) }}
```

Por qué es el peor caso de todos los de §6: el modal es donde el usuario **elige a qué preset asigna cada hablante**. Si muestra el color guardado y el timeline muestra el derivado, ve dos colores distintos para el mismo hablante en dos superficies de la misma pantalla — y el que está en el modal es el que va a gobiernar el `.ass` exportado. La UI miente en la superficie donde la decisión se toma.

Importar `colorHablante` en `AssExportModal.tsx`. Ya tiene `presets` en scope: no hace falta cambiar ninguna prop.

- [ ] **Step 9: Borrar `cambiarColorHablante` y `PALETA`**

- `useHablantes.ts:71-83`: borrar el `useCallback` completo y su línea en el `return`.
- `useHablantes.ts:52`: `color: PALETA[...]` → `color: "#FFFFFF"` (el hablante nuevo nace sin preset, igual que el resto).
- `autosubs.ts:92`: igual, `color: "#FFFFFF"`. Actualizar el comentario de la línea 77 que menciona "PALETA".
- `constants.ts:17-27`: borrar el array `PALETA` completo.
- `App.tsx`: quitar `onCambiarColor={voces.cambiarColorHablante}` de las props de `SpeakersPanel`.

`Hablante.color` **se queda** en `types.ts` y en el `struct Hablante` de Rust (`lib.rs:148`): los `.cdp` guardados lo traen en el JSON y borrarlo rompe la deserialización. Queda como campo muerto, sin leer.

- [ ] **Step 11: Reescribir el test de convención PALETA**

`src/utils/__tests__/autosubs.test.ts:94-109` — los tres tests que verifican que el hablante nuevo tome un color de `PALETA` y que la paleta tenga 9 colores distintos **se borran**. En su lugar, un test que fija el comportamiento nuevo:

```ts
it("el hablante nuevo nace sin preset, y su color guardado es blanco", () => {
  const hs = hablantesDesdeNombres(["1", "2"]);
  expect(hs[0].color).toBe("#FFFFFF");
  expect(hs[0].presetId).toBeUndefined();
});
```

Y borrar el import de `PALETA` de la línea 8 del test.

- [ ] **Step 12: Correr toda la suite**

Run: `npm test`
Expected: PASS. Si `ass.test.ts:444` o `:641` fallan, es porque usan `color` en fixtures — esos **deben seguir pasando** porque el campo sigue viajando en el JSON; solo dejó de leerse en la UI.

- [ ] **Step 13: Verificar manualmente (Review Focus de la spec §6)**

Con la app corriendo y un proyecto con hablantes:
- Editar el color de un preset en el tab Estilos → **los carriles del timeline se repintan sin recargar**.
- Un hablante con preset de color muestra ese color en el carril, en el tag de la lista y en el chip del editor — los tres.
- Un hablante sin preset muestra **blanco** en los tres.
- El carril `"—"` sigue gris y no se confunde con un hablante blanco.
- **Abrir un proyecto viejo** (con colores guardados): los hablantes salen blancos, sin crash.
- Exportar un `.ass`, abrirlo en un editor de texto: el color de cada hablante **coincide con el color de su carril**. Ese es el criterio de aceptación de la sección entera.

- [ ] **Step 14: Commit**

```bash
git add src/utils/colorHablante.ts src/utils/__tests__/colorHablante.test.ts \
        src/utils/__tests__/autosubs.test.ts src/utils/constants.ts src/utils/autosubs.ts \
        src/hooks/useHablantes.ts src/App.tsx src/App.css \
        src/components/CaptionList.tsx src/components/SpeakersPanel.tsx
git commit -m "color: el hablante toma el color de su preset; fin de Hablante.color como fuente"
```

---

## Task 3: `assPreview.ts` — el render compartido

La bisagra del plan. Si esto está mal, el preview del editor miente sobre lo que el `.ass` va a decir. Por eso va **antes** que el componente que lo usa, y tiene tests propios.

Hoy el render vive dentro del cuerpo de `App.tsx` (líneas 1977-2019) cerrado sobre `overlayEscala`. Se extrae a dos funciones puras que reciben la escala como parámetro — el overlay pasa la del video, el editor la de su stage.

**Files:**
- Create: `src/utils/assPreview.ts`
- Create: `src/utils/__tests__/assPreview.test.ts`
- Modify: `src/App.tsx:1977-2019` (reemplazar por llamadas)

**Interfaces:**
- Consumes: `PresetAss` de `src/types.ts`, `ASS_EM_PREVIEW` de `src/utils/constants.ts`.
- Produces: `estiloLineaPreview(preset: PresetAss, escala: number): CSSProperties` y `estiloCajaPreview(preset: PresetAss, recuadro: {left:number;top:number;width:number;height:number} | null, escala: number): CSSProperties`. Ambas las consume la Task 4.

- [ ] **Step 1: Escribir el test que falla**

`src/utils/__tests__/assPreview.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { estiloLineaPreview, estiloCajaPreview } from "../assPreview";
import { DEFAULT_PRESET_ASS, ASS_EM_PREVIEW } from "../constants";
import type { PresetAss } from "../../types";

const P = (over: Partial<PresetAss> = {}): PresetAss => ({ ...DEFAULT_PRESET_ASS, ...over });
const rec = { left: 10, top: 20, width: 600, height: 340 };

describe("estiloLineaPreview", () => {
  it("escala el tamaño con ASS_EM_PREVIEW (celda ASS vs em CSS)", () => {
    const s = estiloLineaPreview(P({ fontsize: 48 }), 1);
    expect(s.fontSize).toBeCloseTo(48 * ASS_EM_PREVIEW);
  });
  it("el borde escala pero nunca baja de 1px", () => {
    expect(estiloLineaPreview(P({ outline: 0 }), 0.1).textShadow).toContain("1px");
  });
});

describe("estiloCajaPreview", () => {
  it("alignment 2 = abajo al centro", () => {
    const s = estiloCajaPreview(P({ alignment: 2 }), rec, 1);
    expect(s.justifyContent).toBe("center");
    expect(s.alignItems).toBe("flex-end");
  });
  it("alignment 7 = arriba al centro", () => {
    const s = estiloCajaPreview(P({ alignment: 7 }), rec, 1);
    expect(s.alignItems).toBe("flex-start");
    expect(s.justifyContent).toBe("center");
  });
  it("alignment 1 = abajo a la izquierda", () => {
    expect(estiloCajaPreview(P({ alignment: 1 }), rec, 1).justifyContent).toBe("flex-start");
  });
  it("alignment 3 = abajo a la derecha", () => {
    expect(estiloCajaPreview(P({ alignment: 3 }), rec, 1).justifyContent).toBe("flex-end");
  });
  it("los márgenes verticales escalan con la escala", () => {
    const s = estiloCajaPreview(P({ alignment: 2, marginV: 40 }), rec, 0.5);
    expect(s.paddingBottom).toBe(20);
    expect(s.paddingTop).toBeUndefined();
  });
  it("alignment 5 (medio) no aplica marginV arriba ni abajo", () => {
    const s = estiloCajaPreview(P({ alignment: 5, marginV: 40 }), rec, 1);
    expect(s.paddingTop).toBeUndefined();
    expect(s.paddingBottom).toBeUndefined();
  });
  it("con recuadro null no fuerza la caja (deja el inset del CSS)", () => {
    const s = estiloCajaPreview(P(), null, 1);
    expect(s.left).toBeUndefined();
    expect(s.width).toBeUndefined();
  });
  it("con recuadro posiciona sobre el video visible, no el elemento", () => {
    const s = estiloCajaPreview(P(), rec, 1);
    expect(s.left).toBe(10);
    expect(s.width).toBe(600);
  });
});
```

- [ ] **Step 2: Correr el test para verificar que falla**

Run: `npx vitest run src/utils/__tests__/assPreview.test.ts`
Expected: FAIL con "Cannot find module '../assPreview'".

- [ ] **Step 3: Implementar `assPreview.ts`**

Dos funciones puras. `estiloLineaPreview` calcula el `fontSize` (con `ASS_EM_PREVIEW`) y el `textShadow` del outline. `estiloCajaPreview` calcula el `justifyContent`/`alignItems` a partir de `alignment` (1-3 abajo, 4-6 medio, 7-9 arriba; columnas 1/4/7 izquierda, 2/5/8 centro, 3/6/9 derecha) y aplica los márgenes: `marginV` solo en `paddingTop` si la alineación es arriba, solo en `paddingBottom` si es abajo, ninguno si es media; `marginL`/`marginR` siempre. Cuando `recuadro` es `null`, no emite `left/top/width/height` (deja el `inset: 0` del CSS).

Es una **extracción**, no una reescritura: la aritmética es idéntica a la que hoy está inline en `App.tsx:1980-2017`. Copiar, no reinterpretar.

- [ ] **Step 4: Correr el test para verificar que pasa**

Run: `npx vitest run src/utils/__tests__/assPreview.test.ts`
Expected: PASS 9/9.

- [ ] **Step 5: Cablear App.tsx**

En `App.tsx:1977-2019`, borrar las definiciones de `estiloLineaOverlay` y `overlayStyle` y reemplazarlas por llamadas:

```ts
const estiloLineaOverlay = (p: PresetAss) => estiloLineaPreview(p, overlayEscala);
const overlayStyle = overlayBase ? estiloCajaPreview(overlayBase, overlayRecuadro, overlayEscala) : undefined;
```

`overlayRecuadro` queda igual — es la geometría del `<video>`, no del render, y ya está testeada por el `ResizeObserver`.

- [ ] **Step 6: Verificar que el preview del video NO cambió**

Run: `npm run build` → PASS.

Con la app corriendo, comparar el preview sobre el video antes y después del commit: **debe ser pixel-idéntico**. `ASS_EM_PREVIEW` no se tocó. Si algo se movió, la extracción cambió aritmética — volver a comparar contra `App.tsx:1980-2017` línea por línea. Este es Review Focus #5 y es el bug más caro del plan si se dispara.

- [ ] **Step 7: Commit**

```bash
git add src/utils/assPreview.ts src/utils/__tests__/assPreview.test.ts src/App.tsx
git commit -m "ass: extrae el render del preview a funciones puras compartidas"
```

---

## Task 4: Estilo del preview embebido en StylesPanel

El editor de presets gana un preview en vivo que usa `estiloLineaPreview` / `estiloCajaPreview` de la Task 3 — **el mismo código que el overlay del video**, no una copia. Un `stage` 16:9 negro que mide su propia caja con `ResizeObserver` y calcula `escala = min(w/1920, h/1080)` (misma fórmula que el video: libass escala por PlayResY, así que manda el mínimo de ambos ejes).

**Files:**
- Create: `src/components/StylePreview.tsx`
- Modify: `src/components/StylesPanel.tsx` (añadir el preview al pie, aceptar `textoInicial`)
- Modify: `src/App.tsx` (pasar `currentCaption?.texto` como `textoInicial`)
- Modify: `src/App.css` (`.stylePreviewStage`)
- Modify: `src/i18n/es.json`, `en.json`

**Interfaces:**
- Consumes: `estiloLineaPreview` y `estiloCajaPreview` de `src/utils/assPreview.ts` (Task 3).
- Produces: `StylePreview({ preset, textoInicial }: { preset: PresetAss; textoInicial: string })`. Lo usa la Task 5.

- [ ] **Step 1: Implementar `StylePreview`**

Componente `memo()`. Props `preset` y `textoInicial`. Mide su propia caja con `ResizeObserver` (patrón ya establecido en `App.tsx:815-840` para el video) y guarda `{w, h}` en un ref + un tick de re-render, **no** en un `useState` que dispare en cada resize frame.

Calcula `escala = Math.min(w / 1920, h / 1080)` y pasa `recuadro = {left:0, top:0, width:w, height:h}`. Renderiza el texto con `estiloLineaPreview(preset, escala)` dentro de un div con `estiloCajaPreview(preset, recuadro, escala)`.

El texto de ejemplo es un `<input>` **editable** cuyo valor inicial es `textoInicial` (el texto del caption bajo el playhead, que App pasa como prop). Cambia el input → cambia el preview. El estado del input vive en `StylePreview` con `useState(textoInicial)`.

- [ ] **Step 2: Estilo `.stylePreviewStage`**

`background: #000; aspect-ratio: 16/9; border: 1px solid var(--border); border-radius: var(--r-md); display: flex;` y el input de texto encima en `--surface`, ancho completo, `font-family: var(--font-mono)` pequeño.

- [ ] **Step 3: Montar en StylesPanel**

Debajo del último campo (`assFields`), fuera del `assPresetList`. Recibe el `preset` seleccionado (`sel`, que ya existe derivado).

`StylesPanel` acepta una prop nueva `textoInicial: string` y la reenvía a `StylePreview`. En `App.tsx`, `<StylesPanel ... textoInicial={currentCaption?.texto ?? t("assExport.sampleText")} />` — `currentCaption` ya existe derivado en App.tsx (~1935) y `styles.previewTextPlaceholder` es el i18n del texto por defecto.

- [ ] **Step 4: i18n**

`es.json` y `en.json`: `styles.previewLabel` ("Vista previa del estilo" / "Style preview"), `styles.previewTextPlaceholder` ("Texto de ejemplo" / "Sample text"), `assExport.sampleText` ("Texto de ejemplo" / "Sample text") — esta última es el fallback en App cuando no hay caption bajo el playhead.

- [ ] **Step 5: Verificar (Review Focus #1 y #2)**

Run: `npm run build` → PASS.

Con la app corriendo, tab Estilos:
- El texto del preview **se ve del mismo tamaño** que el preview sobre el video con el mismo preset y el mismo video de referencia. Si difieren, `escala` o `ASS_EM_PREVIEW` están mal calibrados en el stage.
- Cambiar `fontsize` actualiza el preview **al instante**, sin reiniciar.
- Cambiar `color`, `outline`, `marginV` se refleja en vivo.
- Editar el texto de ejemplo cambia lo que se ve en el stage.

- [ ] **Step 6: Commit**

```bash
git add src/components/StylePreview.tsx src/components/StylesPanel.tsx src/App.css src/i18n/es.json src/i18n/en.json
git commit -m "estilos: preview en vivo del preset con el render compartido"
```

---

## Task 5: Matriz de alineación con flechas

El `<select>` de alineación (StylesPanel.tsx:285-297) se reemplaza por una matriz 3×3 de flechas. Es la forma en que libass piensa la alineación: las bandas 1-3 son abajo, 4-6 medio, 7-9 arriba; las columnas izquierda/centro/derecha.

**Files:**
- Modify: `src/components/StylesPanel.tsx`
- Modify: `src/App.css` (`.assAlignGrid`, `.assAlignBtn`, `.assAlignBtn.activo`)
- Modify: `src/App.tsx` (guard de teclado en 1510-1516, **solo si** el Step 4 lo justifica)

**Interfaces:**
- Consumes: `StylePreview` de la Task 4.
- Produces: nada (reemplaza un control existente).

- [ ] **Step 1: Reemplazar el `<select>` por la matriz de flechas**

Borrar el bloque `assField` de `alignment`. `ALINEACIONES` sigue viva — es el `[1..9]` que mapea las claves i18n; borrar solo si al terminar no quedan referencias.

La grilla, **en el orden de arriba hacia abajo** (fila superior = alineación alta):

```
↖  ↑  ↗        ← alignment 7, 8, 9
←  ●  →        ← alignment 4, 5, 6
↙  ↓  ↘        ← alignment 1, 2, 3
```

Cada botón: `onClick={() => editar("alignment", n)}`, `aria-pressed={sel.alignment === n}`, `aria-label` con el nombre translated (`assExport.align_1..9`, que ya existe en ambos `es.json` y `en.json` — verificado, no hay que crearlas). El glifo es un SVG inline `currentColor`, no un carácter Unicode: los emoji y los glifos de flecha se renderizan distinto según la fuente y rompen la alineación en una grilla.

El centro (`●`) es alignment 5 y **no** lleva flecha: no apunta a ninguna esquina.

- [ ] **Step 2: Estilo**

`.assAlignGrid { display:grid; grid-template-columns: repeat(3, 30px); gap:2px; }` — celdas cuadradas, `--r-sm`, fondo `--surface2`, hover `--surfaceHover`. `.activo { background: var(--accent); color: var(--text); }`.

- [ ] **Step 3: Impedir que el botón robe el foco — CRÍTICO**

Cada botón de la matriz lleva:

```tsx
onMouseDown={(e) => e.preventDefault()}
```

**Por qué es obligatorio y no cosmético.** El guard del teclado global (App.tsx:1510-1516) corta el handler para `INPUT`/`TEXTAREA`/`SELECT` pero **no** para `BUTTON`. Sin el `preventDefault`, hacer click en un botón lo deja focalizado, y la tecla siguiente la lee el handler global:

| Botón | Tecla | Sin el fix |
|---|---|---|
| `↘` (alignment 3) | `→` | **seek +5s** |
| `↓` (alignment 2) | `↓` | **salto de caption** |
| cualquier `1`-`9` anterior | `1`-`9` | **asigna hablante H1-H9** |

Es el mismo patrón que **ya usa** el caret del selector de fuentes (StylesPanel.tsx:182), con un comentario que explica por qué. No es una invención: es la solución que el proyecto ya eligió para este mismo problema.

`preventDefault` en `mousedown` impide que el click mueva el foco; el botón se sigue activando por `onClick`.

**Esto no arregla el caso del tabulador.** `Tab` sí enfoca botones y no pasa por `mousedown`. Un usuario que navegue solo con teclado **sí** puede disparar el atajo global después de enfocar un botón. Ver Review Focus #4 — la defensa completa es el Step 4.

- [ ] **Step 4: Decidir el guard de `BUTTON` (lectura previa, no cambio a ciegas)**

El guard de App.tsx:1510-1516 no cubre `BUTTON`. Agregarlo es la defensa en profundidad, pero tiene un costo que hay que medir antes:

`searchToggleCase` (App.tsx:2499-2504) es un `<button>` que muestra "Aa" y alterna `coincidirMayus`. Si `BUTTON` entra al guard, **el botón deja de responder a la tecla `A`** que hoy muestra como atajo en su label.

Antes de cambiar nada, confirmar en `es.json`/`en.json` si `app.search.matchCase` describe `A` como atajo. Si lo describe como atajo real, **no agregar `BUTTON` al guard** — el `preventDefault` del Step 3 es la solución suficiente para el caso del mouse, y el caso del tabulador se acepta como limitación documentada. Si solo dice "Coincidir mayúsculas", agregar `BUTTON` al guard es gratis y se hace en este step.

Esta decisión se toma leyendo, no asumiendo. El implementer reporta qué encontró en el JSON y qué decidió.

- [ ] **Step 5: Verificar**

Run: `npm run build` → PASS.

Con la app corriendo, tab Estilos:
- Cada una de las 9 posiciones mueve el texto del preview a la esquina correspondiente. **Ese es el test vivo de que Task 3 compartió bien el código** — si el preview y el overlay se mueven distinto, la extracción no se copió fielmente.
- **Click en `↘` (abajo-derecha), después apretar `→`:** el texto NO cambia de esquina y el video **no** hace seek de 5s. Si hace cualquiera de las dos, el `preventDefault` falta.
- **Click en `↓`, después apretar `↓`:** el playhead no salta de caption.

- [ ] **Step 6: Commit**

```bash
git add src/components/StylesPanel.tsx src/App.css src/App.tsx
git commit -m "estilos: matriz de alineacion con flechas; preventDefault para no robar foco"
```

---

## Task 6: `PanelTabs`

Reemplaza los dos acordeones + la lista por 3 tabs. El `CaptionList` se monta solo en su tab → la lista gana la altura que hoy se comen dos cabeceras de acordeón.

**Files:**
- Create: `src/components/PanelTabs.tsx`
- Create: `src/components/__tests__` — **no**: sin jsdom, este componente no tiene test. Verificación manual.
- Modify: `src/App.tsx` (estado `paneActivo`, JSX de la columna derecha, listener del evento de menú)
- Modify: `src/components/SpeakersPanel.tsx` (quitar `panelAbierto`/`onTogglePanel` y la envoltura `.speakersAccordion`)
- Modify: `src/components/StylesPanel.tsx` (idem)
- Modify: `src/App.css` (`.panelTabs`, `.panelTab`, `.panelTab.activo`)
- Modify: `src/i18n/es.json`, `en.json`

**Interfaces:**
- Consumes: nada de otras tareas.
- Produces: `PanelTabs({ panes, activo, onCambiar })`. `PaneId = "subs" | "hablantes" | "estilos"`.

- [ ] **Step 1: Implementar `PanelTabs`**

`memo()`. Props `panes: {id: PaneId; label: string; contador: number | null}[]`, `activo: PaneId`, `onCambiar: (id: PaneId) => void`.

Botón browser-style: `height: 29px`, `border: 1px solid transparent; border-bottom: none; border-radius: var(--r-md) var(--r-md) 0 0; position: relative; top: 1px` (para que el borde activo del activo se coma la línea de la barra). El `.activo`: `background: var(--surface); border-color: var(--border); border-top: 2px solid var(--accent); color: var(--text)`. Los inactivos: `color: var(--textFaint)`, hover `--surfaceHover`.

Cada tab lleva un SVG inline `currentColor` + label + contador en `.tabCounterBadge` mono. Contadores: Subtítulos → `captions.length`, Hablantes → `hablantes.length`, Estilos → `presets.length`. `contador: null` oculta el badge.

- [ ] **Step 2: Estado en App.tsx**

`const [paneActivo, setPaneActivo] = useState<PaneId>("subs")`. **No se persiste** — siempre arranca en `subs`, que es donde se abre el 90% de las sesiones. Reemplaza `panelHablantesAbierto` y `panelEstilosAbierto` (y sus `toggle`).

- [ ] **Step 3: Reestructurar la columna derecha**

```
<PanelTabs panes={[...]} activo={paneActivo} onCambiar={setPaneActivo} />
<div className="tabPaneContainer">
  {paneActivo === "subs"      && <>{searchBox}{rightColHeader}<CaptionList .../></>}
  {paneActivo === "hablantes" && <SpeakersPanel ... />}
  {paneActivo === "estilos"   && <StylesPanel ... />}
</div>
```

El `.searchBox` (App.tsx:2488-2536) **se mueve dentro** del pane de Subtítulos — es donde pertenece. El `.rightColHeader` con el contador y el toggle de búsqueda también.

`CaptionList` solo se monta con `paneActivo === "subs"`. Efecto secundario deseable: los `rowRefs` de la lista virtualizada se liberan al cambiar de tab.

- [ ] **Step 4: Quitar los acordeones**

`SpeakersPanel`: borrar la envoltura `<div className="speakersAccordion">` + el `<button className="speakersAccordionHeader">` + el condicional `{panelAbierto && ...}`; las props `panelAbierto` y `onTogglePanel` desaparecen de la interfaz. Idem `StylesPanel` (misma envoltura, mismo header, mismo `chevronSvg` — los dos archivos declaran un `chevronSvg` local idéntico).

Actualizar el JSX de App.tsx que les pasa esas props.

**Los accordions comparten clase CSS**: `.speakersAccordion` / `.speakersAccordionHeader` / `.speakersPanel` (App.css ~857-1003) están usadas por los dos paneles. Al quitar la envoltura quedan **huérfanas** — borrarlas en Task 7 con la auditoría de tokens (§Task 7 Step 2). No borrarlas en esta tarea: si aparece un uso en un `.tsx` que no leíste, el build lo dice.

- [ ] **Step 5: Evento de menú**

El listener ya existe en `App.tsx:758-764` y hoy hace dos cosas: `ass.recargarPresets()` **y** `setPanelEstilosAbierto(true)`.

Con tabs, la **segunda** pasa a `setPaneActivo("estilos")`. Un tab no se cierra — no hay caso "ya estaba abierto, cerralo", así que la línea desaparece en vez de alternar.

La **primera se queda**: `recargarPresets()` relee `presets_ass.json` desde disco. Si un `.ass` importado desde afuera mientras la app está corriendo grabó presets nuevos, el tab tiene que recogerlos. Borra esa línea solo si también borras el import.

> Nota: `lib.rs` emite `event.id().as_ref()` y el id del item **es** el string `"gestionar_presets"` (lib.rs:590). El nombre del evento no cambia — no se toca `lib.rs` en esta tarea.

- [ ] **Step 6: i18n**

`es.json`/`en.json`: `app.tabs.subs`, `app.tabs.speakers`, `app.tabs.styles`, y los `title` de los tres botones.

- [ ] **Step 7: Build + verificación manual (Review Focus #3 y #4)**

Run: `npm run build` → PASS.

Con la app corriendo:
- Los 3 tabs cambian de contenido. Subtítulos conserva búsqueda + replace + contador.
- Ir a Estilos y volver a Subtítulos: **la fila activa sigue siendo la misma** (el `currentCaptionIdx` no cambió, y el follow-scroll la reposiciona).
- El `searchBox` mantiene su término si se va a Estilos y se vuelve.
- El menú nativo *Estilos…* salta al tab de Estilos.
- Borrar el último hablante: el contador baja a 0 y el badge se oculta.
- `npm test` → PASS (ningún test depende del DOM de estos componentes).

- [ ] **Step 8: Commit**

```bash
git add src/components/PanelTabs.tsx src/components/SpeakersPanel.tsx src/components/StylesPanel.tsx \
        src/App.tsx src/App.css src/i18n/es.json src/i18n/en.json
git commit -m "layout: tabs de navegador para Subtítulos/Hablantes/Estilos"
```

---

## Task 7: Cierre — `DUR` en el editor, tokens huérfanos, gates

Las tres cosas que sobran del spec y que conviene cerrar juntas.

**Files:**
- Modify: `src/App.tsx:2333-2372` (`captionTimeRow`)
- Modify: `src/App.css` (si `--textHint` quedó sin uso, asignarlo o quitarlo)
- Modify: `src/utils/constants.ts` si `PALETA` dejó rastros

**Interfaces:**
- Consumes: nada.
- Produces: nada.

- [ ] **Step 1: `DUR` junto a `IN`/`OUT`**

El `captionTimeRow` (App.tsx:2333-2372) ya tiene los dos inputs numéricos editables. **Se conservan tal cual** — existen desde `ee04b7b` con el motivo "el timing es el 80% del oficio y necesitaba un input numérico". Agregar a la derecha un `<span>` de solo lectura con `DUR: {(currentCaption.fin - currentCaption.inicio).toFixed(2)}s`.

- [ ] **Step 2: Auditar tokens huérfanos y rgba del coral viejo**

**(a) Tokens definidos y nunca usados.** `grep -rn 'var(--textHint)' src/` y `grep -rn 'var(--surfaceHover)' src/` y `grep -rn 'var(--borderBright)' src/`. Si un token no se usa, o se le da un uso (§3 lo pensó para hover de fila, hint de statusBar y anillo de foco) o se borra del `:root`. Un token definido y nunca usado es deuda silenciosa.

**(b) Los 7 `rgba(232, 93, 78, …)` del coral viejo — ENRUTADO desde la review de Task 1.** El acento pasó a mostaza (`#D4A017`) pero siete `box-shadow`/`background` quedaron con el coral hardcodeado. **El caso grave es `.playheadLine`** (App.css:771): tiene `background: var(--accent)` [mostaza] junto a `box-shadow: 0 0 8px rgba(232,93,78,0.8)` [coral] — dos tonos distintos en el mismo elemento, y es el objeto más visible del timeline.

Reemplazar cada `rgba(232, 93, 78, X)` por `color-mix(in srgb, var(--accent) Y%, transparent)` con Y ≈ X × 53 (la conversión de 14% de opacidad `--accentSoft` a porcentaje es 0.14 × 100; los valores existentes van de 0.08 a 0.8, o sea 8% a 80%). El proyecto ya usa `color-mix()` en App.tsx, así que no es una primitiva nueva.

Las 7 ubicaciones (verificadas en App.css post-T1): **334** (`.playBtn` glow), **408** (`select:focus`), **510** y **514** (`.track.dropTarget`), **771** (`.playheadLine` — la prioritaria), **815** (`.marqueeOverlay`), **1302** (`.dropOverlay`).

- [ ] **Step 3: Auditar imports y clases muertos**

`grep -rn 'PALETA\|cambiarColorHablante\|normalizarColor' src/` → debe dar **cero** resultados fuera de comentarios.

`grep -rn 'speakersAccordion' src/` → debe dar **cero**. Las clases que Task 6 dejó huérfanas (`.speakersAccordion`, `.speakersAccordionHeader`, `.speakersPanel`) se borran de `App.css` acá. Ojo: `.speakersPanel` como *string* matchea el nombre del componente `<SpeakersPanel>`; verificar contra el selector CSS completo (`.speakersPanel {`) antes de borrar.

- [ ] **Step 4: Gates**

```bash
npm run build
npm test
cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo fmt --check
```

Expected: todo PASS. Si `cargo clippy` se queja del `struct Hablante` con `color` sin usar — **no lo borres**: los `.cdp` viejos lo traen en el JSON (ver Global Constraints). Si se queja, agregá `#[allow(dead_code)]` con un comentario que explique por qué.

- [ ] **Step 5: Verificación manual final — la lista del spec §8.2 completa**

Con `npm run tauri` corriendo,一遍 los 18 checks del spec. Los que no son evidentes aquí:

- Check 5: `ASS_EM_PREVIEW` sin tocar — el preview sobre el video se ve idéntico al commit previo a Task 3.
- Check 14: los clips blancos **no** tienen borde blanco sólido.
- Check 16: `.clip.selected` se lee en mostaza.
- Check 17: abrir un proyecto viejo — hablantes blancos, sin crash.
- Check 18: exportar `.ass` y abrirlo — el color de cada hablante **coincide con su carril** en el timeline.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "cierre: DUR en el editor, tokens hufranos, gates"
```

---

## Self-Review

**Spec coverage:** §3 tokens → Task 1. §4 tabs → Task 6. §5 preview → Tasks 3-5. §6 color derivado → Task 2. §7 (IN/OUT se conservan, DUR se agrega) → Task 7 Step 1. §8 verificación → distributed. §9 riesgos → cada mitigación es un Step o un Review Focus.

**Type consistency:** `colorHablante` se define en Task 2 y la consumen Tasks 2 y 4 — mismo nombre, misma firma. `estiloLineaPreview`/`estiloCajaPreview` se definen en Task 3, las consumen Task 3 (App) y Task 4 (StylePreview) — mismo nombre. `PaneId` se define en Task 6 y solo se usa ahí. `ASS_EM_PREVIEW` se importa de `constants` en Task 3 y no se re-declara.

**Lo que este plan NO tiene:** tests de los componentes (Task 4, 5, 6). No es omisión — el proyecto no tiene jsdom y `testing-library` no está instalado. Los tres están cubiertos por los checks manuales de sus Steps de verificación, que es la forma en que este proyecto verifica UI hoy.

**Lo que este plan NO toca, deliberadamente:** `Hablante.color` en `types.ts` y en el `struct` de Rust (rompería proyectos viejos). `lib.rs` más allá del listener del menú (el campo `color` sigue siendo parte del contrato de serialización). `ffmpeg` (ya no es requisito desde 2026-10-09).
