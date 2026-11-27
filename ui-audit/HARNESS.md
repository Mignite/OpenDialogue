# Arnés visual — Fase 0

## Cómo regenerar

```bash
cargo run -p rod-app --features test-video --example ui_audit -- before
cargo run -p rod-app --features test-video --example ui_audit -- after
cargo run -p rod-app --features test-video --example ui_audit -- hoja before
```

El ejemplo incluye `crates/app/tests/ui_audit/mod.rs` por `#[path]`; no es un
test, es el helper. `guardar()` llama a `Harness::render()` con wgpu y escribe
el PNG.

- `egui_kittest = 0.33.0`, construido con `.wgpu()` y un `pixels_per_point` de 1.
- Tamaños: 1100×700, 1600×900, 1920×1080, 2560×1440.
- Nomenclatura: `pantalla__estado__WxH.png`. `index.png` muestra la serie de
  1600×900 más las dos capturas en español, para que la hoja se pueda mirar.
- Las capturas se generan en inglés salvo `*-es.png`.

## Fixtures

- `make_proyecto(n_lineas, n_hablantes, seed, duracion)` genera proyectos sin
  archivos externos. Líneas cortas, de dos líneas, muy largas, con comillas,
  porcentajes, hablantes ausentes cada 13 y solapes cada 17.
- Grande: 543 líneas / 7 hablantes / 1440 s. Mediano: 19 / 3 / 92 s.
  Pequeño: 1 / 1 / 12 s. Vacío: 0 / 0 / 0 s.
- Cinco recientes sintéticos con miniaturas de degradado escritas en la caché
  de la app. El segundo, `sin-video.cdp`, no tiene miniatura a propósito.
- Banner de recuperación construido con `construir_recovery` sobre el proyecto
  mediano, con texto de fecha relativa fijo.
- Frame de video: patrón 1920×1080 generado por código (barras de color,
  rejilla de tercios, círculo central y bandas diagonales) para que el overlay
  ASS tenga contraste real. Se cachea en un `OnceLock`.

## Desviación documentada: mpv

`EditorApp::desde_proyecto_en` exige un `rod_player::Player` y no existe un
constructor sin reproductor sin tocar la lógica del jugador. El arnés abre el
clip de prueba que ya vive en `testdata/`, pero **no depende de él para el
contenido visible**: `preparar_captura()` (feature `test-video`) sustituye los
píxeles por el frame sintético, fija duración, posición y estado en pausa, y
descarta el análisis de waveform en favor de `picos_fixture()`. El reloj lo fija
el arnés, no mpv. Si en otro equipo no hay mpv, hace falta un `Player` nulo
explícito en `rod-player`; eso queda fuera del alcance de este pase visual.

Tras el primer frame se llama a `encajar_timeline()`, que aplica la misma
`Vista::encajar` de la app con el ancho del eje ya medido. Así la regla de la
captura coincide con la de un video recién abierto.

## Lo que NO cubre el arnés

- No hay diálogo de exportación/importación dentro de la app: `main.rs` usa
  `rfd` nativo. Se captura Preferences (`ajustes__*`) y el panel de atajos
  (`ayuda__atajos__*`) como diálogos representativos.
- No hay menú de engranaje: Ajustes es un comando del menú Edit.
- El foco y el hover se simulan con eventos de egui; el tooltip se espera 32
  frames para pasar los 0.4 s de retardo.
