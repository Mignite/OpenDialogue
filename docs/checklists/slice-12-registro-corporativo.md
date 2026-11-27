# Slice 12 — registro corporativo (inicio + editor)

Semáforo (2026-11-23):

- [x] `cargo build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo test --workspace` — 435 tests en verde (154 app + 3 bin + 1 integración + 256 core + 3 player + 15 subs + 3 varios)
- [x] captura real del arranque a 1600×900: `slice-12-inicio-corporativo.png`
- [x] captura real del editor con proyecto sembrado: `slice-12-editor-corporativo.png`

## Qué cambió

- `tema.rs`: gris medio corporativo (`#4A4A4A` barra, `#343434` lienzo, `#2C2C2C` panel, `#0E0E0F` preview), texto `#E8E8E8`/`#B8B8B8`/`#8C8C8C`, acento azul `#1473E6`. Sin sombras ni gradientes; radios ≤ 3 px. Tests de invariantes y de `color_de_hex`.
- `inicio.rs`: rejilla de tarjetas `.cdp` con miniatura 16:9, ruta y metadata; CTA `Nuevo archivo`, `Abrir...`, aviso compacto de recovery y tarjeta de drop. Desaparecieron el titular display y las marcas de registro.
- `editor_app.rs`: inspector de 440 px, pestañas con contadores, campos de entrada/salida editables con validación, lista compacta de captions, paneles de hablantes y estilos densos, barra de estado con hablantes y zoom.
- `main.rs`: barra de menú con marca `R`, nombre de app y estado a la derecha.
- `timeline.rs`: carriles con el color semántico del `.ass`, rejilla, clips translúcidos con borde, selección con acento azul y chip del playhead en la regla. Alturas 26/46.

## Notas

- El azul de chrome y el azul del hablante 1 son parecidos; se mantienen separados por uso: CTA/foco/selección frente a carril/lista/.ass.
- El preview de la captura del editor muestra la carta de ajuste del video de prueba; en uso real muestra el frame del video abierto.
- La edición de tiempos se confirma con Enter; un valor inválido o una salida anterior a la entrada se descarta sin mutar el proyecto.

## Evidencia

- Mockup de la dirección: `docs/mockups/2026-11-23-inicio-editor-corporativo.html`.
- Decisiones y desvíos: `BITACORA.md`.
