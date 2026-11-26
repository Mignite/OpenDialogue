# Plan de pulido visual

> Estado: Fase 0 cerrada (arnés + `before/`). Fase 1 en curso: seis auditores
> de solo lectura (A–F). Este documento se completa con sus hallazgos.

## Reglas compartidas

- No cambiar lógica ni modelo de datos. No tocar `crates/core` (incluido `ass.rs`).
- Todo valor visual (color, spacing, radio, tipografía, duración) sale de
  `crates/app/src/ui/tema.rs`; todo texto visible sale de `crates/app/src/textos.rs`.
- Invariante: el preview es el mismo ASS que se exporta. El color de identidad
  del hablante es tinte de UI (barra + chip), nunca entra al render ni al export.
- Animaciones entre 120 y 160 ms con `animate_bool_with_time`.
- Sin dependencias nuevas sin justificación; sin `unwrap` nuevo en rutas de UI.
- Nada se da por terminado sin captura `after` que lo demuestre.

## Criterios de aceptación

- Contraste AA; ningún texto < 11 px; foco de 2 px en color de acento.
- Estados hover / active / disabled / focus en cada control interactivo.
- Sin desbordes ni vacíos absurdos entre 1100 y 2560 px.
- Tooltips a 0.4 s con atalo incluido cuando exista.
- Un solo formato de timecode y un solo idioma de UI por pantalla.
- 60 fps con el proyecto grande (543 líneas / 7 hablantes).

## Orden y propiedad de archivos

1. **Fundación** (exclusiva): `ui/tema.rs`, `ui/kit.rs`. Compila antes de que
   arranquen las áreas.
2. **A — Inicio**: `ui/inicio.rs`.
3. **B — Chrome**: `ui/chrome.rs`, `menu.rs`, `ui/ajustes.rs`, `ui/atajos.rs`,
   `ui/toast.rs`.
4. **C — Subtítulos**: `ui/paneles/subtitulos.rs`, `ui/texto.rs`,
   `ui/transporte.rs`.
5. **D — Speakers y Styles**: `ui/paneles/hablantes.rs`,
   `ui/paneles/estilos.rs`.
6. **E — Timeline**: `ui/timeline.rs`, `ui/editor.rs`, `src/timeline.rs`.
7. **F — Transversal**: `textos.rs` y cualquier archivo no asignado arriba.

Un archivo pertenece a un solo subagente. Si un área necesita un cambio en
archivo ajeno, lo pide al orquestador.

## Hallazgos

_Pendiente de la Fase 1._

## Matriz de correspondencia

_Pendiente: punto → captura before → captura after → estado._
