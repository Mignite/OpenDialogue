# Slice 13 — detalles FilmCraft: esquinas y SVG

Semáforo (2026-11-23):

- [x] `cargo build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo test --workspace` — 437 tests en verde (156 app + 3 bin + 1 integración + 256 core + 3 player + 15 subs + 3 varios)
- [x] captura real del arranque: `slice-13-inicio-iconos.png`
- [x] captura real del editor con proyecto sembrado: `slice-13-editor-iconos.png`

## Cambios

- `tema.rs`: radios 4 px en controles y 6 px en paneles/tarjetas/avisos. Test
  `las_tarjetas_son_mas_redondeadas_que_los_controles` (rojo antes, verde
  después). Sin sombras ni gradientes.
- `iconos.rs`: 29 iconos SVG en memoria, rejilla 24 px, trazo `currentColor`
  1,7 px; coloreados con el filtro del widget `svg` de iced.
- `Cargo.toml`: feature `svg` de iced (misma dependencia, sin crate nueva).
- Arranque: iconos de archivo, carpeta, casa e imagen en CTA/navegación/tarjetas.
- Editor: iconos de play/pausa, retroceso, avance, zoom, divider, búsqueda,
  deshacer, rehacer y borrar; tira vertical de herramientas en el timeline.

## Notas

- Los SVG son propios, no copiados de ArtCraft: la licencia de marca de ese
  repo no permite reutilizar sus assets.
- Los botones de la tira de herramientas ejecutan comandos reales (Zoom,
  Dividir, Buscar); no hay controles decorativos sin acción.
- Referencia visual: captura de FilmCraft aportada por el autor y
  `https://github.com/storytold/filmcraft`.
