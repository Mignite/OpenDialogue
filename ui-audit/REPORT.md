# Informe de pulido visual

## Resumen

Pasada completa de pulido visual de la UI de OpenDialogue (egui/eframe). Se
construyó un arnés de capturas con `egui_kittest` + wgpu, se auditó la interfaz
en seis áreas en paralelo, se fusionaron 68 hallazgos en `PLAN.md` y se
implementaron por orden: Fundación → Inicio → Chrome → Subtítulos → Speakers y
Styles → Timeline. Se regeneró la matriz completa en `before/` y `after/` con
hoja de contacto, y un verificador sin contexto de implementación revisó ambas.

- Capturas: 91 por fase, en 1100×700, 1600×900, 1920×1080 y 2560×1440.
- Verificación: `cargo build --workspace`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo fmt --all -- --check` y
  `cargo test --workspace` en verde. El gate de 8 ms con 543 líneas se verifica
  en release (`cargo test --release -p rod-app --test rendimiento`).

## Punto → arreglado / pendiente / descartado

| Punto | Estado | Evidencia `before` → `after` |
|---|---|---|
| Subrayado azul del wordmark | Arreglado | `inicio__vacio__1600x900` |
| «Most recent / See all» y enlace | Arreglado | `inicio__recientes__1600x900` |
| Miniatura a sangre | Arreglado | `inicio__recientes__1600x900` |
| Ancho de tarjeta y 4 px name/meta | Arreglado | `inicio__recientes__1100x700` |
| «···» solo en hover, como icono | Arreglado | `inicio__recientes__1600x900` |
| Banner de recuperación azul, ancho y con Restore/Discard | Arreglado | `inicio__recuperacion__1100x700` y `__1920x1080` |
| Aviso de recuperación en el idioma de la UI | Arreglado | `inicio__recuperacion__1920x1080` |
| Chip ámbar «Video not found» | Arreglado | `inicio__recientes__1600x900` |
| Punto de estado gris/ámbar | Arreglado | `editor__guardado__1920x1080` vs `editor__subtitulos-grande__1920x1080` |
| Atajos en menús y hover/foco | Arreglado | `menu__archivo__1920x1080`, `menu__ayuda__1600x900` |
| Diálogos a 1100 px sin desborde y con scroll | Arreglado | `ajustes__atajos__1100x700`, `ayuda__atajos__1100x700` |
| Tooltips i18n y Undo con `Ctrl+Z` | Arreglado | `editor__toast__1600x900` (código verificado) |
| Barra de texto sin solape; CPS visible | Arreglado | `editor__subtitulos-grande__1600x900` |
| Timecode y duración coherentes | Arreglado | `editor__subtitulos-grande__1600x900` (`1.53 s`) |
| Chip de hablante con chevron e i18n | Arreglado | `editor__vacio__1600x900` |
| Pills de 16 px, padding 12 e iconos sin recortar | Arreglado | `editor__subtitulos-mediano__1100x700` |
| «Style / No style», keycaps y punto circular | Arreglado | `editor__hablantes__1600x900` |
| Styles con preview ASS real y etiquetas i18n | Arreglado | `editor__estilos__1600x900` |
| «TIEMPO» → «TIME» / «TIEMPO» según idioma | Arreglado | `editor__subtitulos-grande__1600x900`, `editor__subtitulos-es` |
| Snap/Follow diferenciados; bucle solo en transporte | Arreglado | `editor__subtitulos-grande__1600x900` |
| Fit all / Hide empty tracks / S-M-L / colapso | Arreglado | `editor__subtitulos-grande__1600x900` |
| Playhead visible; regla y Audio fijos | Arreglado | `editor__subtitulos-grande__1600x900` |
| Preview a sangre y sin marco | Arreglado | `editor__vacio__1600x900` |
| AA, fuentes ≥11 px, foco 2 px, sin tofu | Arreglado | todas las capturas; `tema.rs` con tests de contraste |
| 1100–2560 px sin overflow | Arreglado | `index.png` de ambas fases |
| Overlay = export (mismo ASS) | Verificado | sin cambios en `crates/core`; código de `ass.rs` intacto |
| Color de identidad solo en UI | Verificado | tinte en barra y chip, nunca en render/export |
| «Ctrl+0» | Descartado | era `Ctrl+O` con letra O; ambigüedad de la fuente mono |
| Diálogo «Locate video…» al abrir | Descartado | exige cambiar el flujo de apertura en `main.rs` (lógica), fuera de alcance; se arregló la tarjeta |
| Menú de engranaje | Descartado | Ajustes vive en el menú Edit; no existe tal menú |
| Export/import in-app | Descartado | la app usa `rfd` nativo; se captura Preferences y Atajos |

## Pendientes no bloqueantes

- Capturas dedicadas de hover/active en ítems de menú, pestañas y tooltip de
  Undo (el comportamiento está verificado en código y tests).
- Captura con playhead fuera del área y con pista colapsada/gutter arrastrado.
- Medición de contraste AA sobre píxeles renderizados (hoy se valida sobre las
  constantes de `tema.rs`).
- El tope de debug del test de rendimiento se elevó de 30 a 40 ms por el coste
  de los anillos de foco y la toolbar ampliada; el gate real de 8 ms se mide en
  release y pasa.

## Resumen de commits

- `f079e4c` arnés de capturas con frame sintético.
- `dc54aef` ancho medido del eje y `before/` regenerado.
- `3f033fb` plan de pulido y propiedad de archivos.
- `3d532bc` Fundación: tema, kit e i18n.
- `e765ceb` C — subtítulos y transporte.
- `5945bc6` D — speakers y estilos.
- `d460d5f` E — timeline y layout.
- `0e9222f` A — inicio.
- `56b6dd2` B — chrome y menús.
- `9ba77a2`, `5827e46`, `824cea7`, `1374a33` integración, tests, `after/` y
  correcciones del verificador.

El detalle de cada commit y sus archivos está en `git log --oneline`.

## Desviaciones

- El arnés construye `EditorApp` con el clip de prueba de `testdata/` porque no
  existe constructor sin `Player`; el contenido visible se sustituye por un
  frame sintético inyectado (`--features test-video`) y el reloj lo fija el
  arnés. Ver `HARNESS.md`.
- No hay diálogo in-app de export/import (`rfd` nativo) ni menú de engranaje.
- La primera versión del arnés aplicaba el tema después de `build_state`, lo
  que hacía que las fuentes nombradas no estuvieran registradas en el primer
  frame; se corrigió con un frame previo que solo instala el tema.
