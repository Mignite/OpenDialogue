# Slice 14 — cabecera propia y nombre RustyDialogue

Semáforo (2026-11-23):

- [x] `cargo build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] captura real: `slice-14-inicio-sin-barra.png` (ventana 1600×900, sin barra nativa)

## Cambios

- `main.rs`: `.decorations(false)`, título `RustyDialogue`, cabecera de 30 px con
  marca, menú, zona arrastrable, estado y botones SVG de minimizar/maximizar/
  cerrar. Mensajes `ArrastrarVentana`, `MinimizarVentana`, `MaximizarVentana`
  y `CerrarVentana`, resueltos con `window::latest().and_then(...)`.
- `main.rs`: 10 px de padding lateral en la cabecera; el estado ya no se corta.
- `inicio.rs`: contenido de las tarjetas dentro de un contenedor de 196 px,
  alineado a la izquierda; cabecera y nombre de la app renombrados.
- `editor_app.rs`: marca de la barra de estado renombrada a `RUSTYDIALOGUE`.

## Límites de verificación

- La captura confirma la ausencia de barra nativa y las alineaciones; el arrastre
  y los botones de ventana se apoyan en las acciones documentadas de iced y en
  la compilación, pero no se probaron con clics inyectados (limitación de
  `PrintWindow` explicada en `BITACORA.md`).

## Reversión

El arrastre no funcionó en la práctica. Se restauró la barra nativa
(`.decorations(true)`) y se eliminaron los mensajes y botones de ventana.
Se conservan el nombre `RustyDialogue`, el padding del estado y la alineación
de las tarjetas. Evidencia final: `slice-14-inicio-barra-nativa.png`.
La investigación para retomar CSD con `MouseArea` está en `BITACORA.md`.
