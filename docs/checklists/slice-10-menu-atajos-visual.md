# Slice 10 — menú, atajos y sistema visual

Specs: `docs/superpowers/specs/2026-11-23-menu-atajos-visual.md` y
`docs/superpowers/specs/2026-11-23-arranque-ux.md`.
Plan: `docs/superpowers/plans/2026-11-23-menu-atajos-visual.md`.

## Semáforo

- [x] `cargo build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo test --workspace` — 415 tests en verde (134 lib + 3 bin + 1
  integración + 256 core + 15 subs + 3 player + 3 varios).

## Qué quedó

- Barra de menú dibujada por iced (`iced_aw::MenuBar`) visible en arranque y
  editor, con `Archivo`, `Edición`, `Ver`, `Exportar`, `Estilos` y `Ayuda`.
  Las entradas que no aplican en el arranque salen deshabilitadas.
- `comandos.rs`: un solo registro con etiqueta, atajo y sección. El menú se
  construye desde él y `keyboard::listen` mapea a él; un solo `ejecutar`.
- Atajos porteados del oráculo en `App.tsx:1528-1712`, más Ctrl+O (añadido de
  la versión Rust, documentado como desvío). Ctrl+S y Ctrl+F se reconocen
  incluso con el foco en un campo de texto, igual que en el oráculo.
- Sistema visual swiss/brutalista: negro `#0C0C0D`, panel `#141416`, hairlines
  al 15 %, esquinas 3 px, acento mostaza `#D9A441` en el botón principal, el
  foco del menú y el aviso de recovery. Cero emojis. Los colores de hablante
  siguen mandando en timeline y ASS; el mostaza solo tiñe la chrome.
- Estado vacío: `Todavía no hay proyectos. El primero aparece aquí al guardar.`
  `Ver todos` solo aparece con más de tres recientes.

## Evidencia

- `docs/checklists/slice-10-inicio-visual.png`: arranque con barra de menú,
  botón principal mostaza, estado vacío y sin `Ver todos`.
- `docs/checklists/slice-10-menu-abierto.png`: desplegable `Archivo` con las
  etiquetas y sus atajos alineados a la derecha.

El menú se abrió con un clic real en `Archivo` sobre la ventana en ejecución;
las capturas son de la app, no mockups.

## Verificación que NO se hizo

- No se capturó el editor con el menú: abrir un video requiere el diálogo del
  SO y no se automatizó el gesto.
- No se pulsaron los atajos en la GUI real; la cobertura es de las funciones
  puras de mapeo y del despachador. Queda pendiente una pasada manual de
  Ctrl+S, Ctrl+O, Space y las flechas.
- Algunos comandos del registro muestran un estado en la barra en lugar de
  ejecutar una acción completa: `Buscar`, `ImportarAutosubs`, `EditarTexto` y
  los comandos de apertura dentro del editor. Están documentados como
  pendientes en `ejecutar_comando`; el menú y el teclado siguen enviando el
  mismo `Comando`.

## Decisiones

- `iced_aw` es la única dependencia nueva; `muda` se descartó porque el menú
  nativo no se puede estilizar y exigiría `unsafe` para enganchar el HWND.
- El acento mostaza no sustituye a los colores semánticos de los hablantes.
- El menú se construye desde el mismo registro que el teclado para evitar la
  divergencia que causó el bug del `seek` en el slice anterior.
