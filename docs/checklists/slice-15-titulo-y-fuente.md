# Slice 15 — cabecera CSD y tipografía

Semáforo (2026-11-23):

- [x] `cargo build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] captura real: `slice-15-inicio-fuente.png`
- [x] arrastre verificado por el autor en la app real

## Cambios

- `main.rs`: `AppState` guarda el `window::Id` recibido en el `boot`. La cabecera
  usa `MouseArea::on_press` para `window::drag`, con franjas invisibles de
  `window::drag_resize` en los cuatro bordes y botones SVG de minimizar,
  maximizar (conmuta según `window::is_maximized`) y cerrar.
- Iconos SVG+texto alineados al centro vertical en CTA, navegación y transporte.
- `tema.rs`: `Segoe UI Variable` para cuerpo, display y seminegrita;
  `Cascadia Mono` para etiquetas en Windows. `.default_font(tema::CUERPO)` en
  el builder. Sin assets de fuente nuevos: en otros sistemas usa los defaults de iced.

## Notas

- `iced-window-chrome` sigue siendo el plan B si aparecen problemas de snap o
  de handles; no se añadió la dependencia.
- El diagnóstico con `mouse_event` resultó poco fiable; la validación definitiva
  fue el uso real por parte del autor.
