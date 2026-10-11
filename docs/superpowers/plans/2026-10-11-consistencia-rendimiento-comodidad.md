# Plan — Consistencia, rendimiento y comodidad (OpenDialogue) — COMPLETADO

Objetivo del usuario: "que sea una app útil, cómoda y rápida"; prioridad a consistencia,
todo el código en el repo, que compile sólido, más beneficios que pérdidas.

Estado: **las 5 fases completas**. 276 vitest + 14 Rust, clippy/fmt/build en verde.

## Fase 1 — Fuentes con `fontdb` (consistencia multiplataforma) — HECHO (88a0f56)
- `fontdb = "0.24"`; `normalizar_familias` (puro, 3 tests); `listar_fuentes_sistema`
  sin `#[cfg]` por plataforma.
- Fuera `familias_fc_list`, `familias_del_registro`, la rama PowerShell y `comando_oculto`.
- Bold/Italic agregados a los presets (antes se escribían fijos en 0 = imposibles).
- Medido: 136 familias canónicas vs 265 de `fc-list` (la diferencia son variantes de
  peso, que en .ass van en el campo Bold). La app ya no shellea ningún proceso hijo.

## Fase 2 — Rendimiento (6f8041c)
- `agruparCaptionsPorCarril` (5 tests): una pasada en vez de S+1 filter.
  Medido 1.04 ms → 0.34 ms por render (2000 captions, 9 hablantes).
- `matchingCaptions` en useMemo.
- `Set` en vez de `Array.includes()` en 4 mutaciones (O(n·k) → O(n)).
- ResizeObserver cachea medidas del <video> en un ref (fin del layout thrash).
- Color del waveform cacheado por cubo (48k template-literals/s → ~256/frame).

## Fase 3 — Estabilidad (90e828c)
- `hooks/debounce.ts` (6 tests): flush del auto-guardado de presets al desmontar
  (se perdía el último cambio al cerrar dentro de los 400 ms).
- Los 5 `setTimeout` crudos → `notify` (un solo mecanismo de mensaje en statusBar).
- `playheadPendienteRef` solo se setea si el video existe (contaminaba el flujo
  "seguir sin video").
- El `setTimeout` de 50 ms al reanudar comprueba que el <video> sea el mismo.
- Timer del focus automático del editor rastreado y limpiado.

## Fase 4 — Comodidad (ee04b7b)
- Portapapeles real del SO en Ctrl+C/V (antes era interno: no se podía copiar a
  un editor externo ni pegar de otro programa). `utils/clipboard.ts`, 8 tests.
- Editor numérico de inicio/fin del caption. `clampCaptionTiempos` (7 tests) es
  ahora la ÚNICA regla de clamp (antes había dos distintas).
- Shift+←/→ = 0,1 s; Ctrl+Shift+←/→ mueve el caption seleccionado.

## Fase 5 — Portabilidad + pulido (177d0a4)
- `ruta_video` relativa al .cdp cuando el video cuelga de su carpeta → el proyecto
  cruza máquinas y SO. Backward-compatible (`utils/rutaVideo.ts`, 11 tests).
- i18n: 9 etiquetas de alineación + 3 errores que estaban hardcodeados en español.
  151 claves es/en con paridad.

## Commits
88a0f56 · 6f8041c · 90e828c · e3224b8 · ee04b7b · 177d0a4

## Ideas que quedaron fuera (a propósito)
- `useTimelineVista` / `useTimelineGestos`: App.tsx sigue grande (~2900 líneas) pero
  partirlos es refactor puro, sin ganancia medible; la matemática del body-drag ya
  está extraída y testeada (`findSnapBlockDelta`, `filasDestinoRelativas`,
  `agruparCaptionsPorCarril`, `clampCaptionTiempos`).
- `Ctrl+A` / `Ctrl+D` / `Tab` entre captions: atajos válidos, pero sin hueco medido
  frente a los que sí se agregaron.
- Snap a picos de audio: sigue en la lista de ideas futuras (no era parte de este
  encargo).