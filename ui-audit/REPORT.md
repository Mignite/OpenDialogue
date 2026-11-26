# Informe de pulido visual

> Pendiente de Fase 4. Esta plantilla se completa con evidencia de `after/`.

## Resumen

- Arnés: `egui_kittest` + wgpu, 91 capturas por fase, 4 tamaños
  (1100×700, 1600×900, 1920×1080, 2560×1440) y hoja de contacto.
- Hallazgos de auditoría: 68, fusionados en `PLAN.md`.
- Verificación final: `cargo build`, `cargo clippy -D warnings`, tests y
  regeneración completa de `after/` con el verificador de contexto limpio.

## Punto → arreglado / pendiente / descartado (motivo)

| Punto | Estado | Evidencia |
|---|---|---|
| _Completar tras la verificación_ | | |

## Pendientes

_Ninguno / listar lo que quede con motivo y captura._

## Resumen de commits

- `f079e4c` — arnés de capturas con frame sintético.
- `dc54aef` — ancho medido del eje y `before/` regenerado.
- `3f033fb` — plan de pulido y propiedad de archivos.
- _Fundación y áreas A–E: completar._
- _Verificación: completar._

## Invariantes comprobadas

- Overlay y export comparten el mismo pipeline ASS (`build_ass`), sin tocar `crates/core`.
- El color de identidad del hablante es solo tinte de UI; no entra al render ni al export.

## Desviaciones

- El arnés construye `EditorApp` con el clip de prueba de `testdata/` porque no
  existe constructor sin `Player`; el contenido visible se sustituye por un frame
  sintético inyectado y el reloj lo fija el arnés. Ver `HARNESS.md`.
- No hay diálogo in-app de export/import (`rfd` nativo) ni menú de engranaje;
  se capturan Preferences y el panel de atajos.
