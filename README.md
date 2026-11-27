# OpenDialogue

![Rust](https://img.shields.io/badge/Rust-1.88%2B-CE422B?logo=rust)
![egui](https://img.shields.io/badge/egui-0.33-2D8C6A)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-lightgrey)
![License MIT](https://img.shields.io/badge/License-MIT-green)

Editor visual de subtítulos multihablante y compañero de
[auto-subs](https://github.com/tmoroney/auto-subs). Importa su SRT + TXT, asigna
a cada hablante un color en su carril de la timeline y exporta SRT limpio, ASS
estilizado o JSON.

**Esta rama es la rewrite 100 % Rust.** No hay WebView, React ni Tauri: la
interfaz es egui/eframe nativa, el video se decodifica con libmpv y los
subtítulos se rasterizan con libass. El nombre `RustyDialogue` se usó como
distintivo del experimento; el nombre oficial del producto sigue siendo
**OpenDialogue**.

## Características

- **Timeline por hablante** con clips reales, solapes apilados, selección
  múltiple, arrastre con snap y recorte de bordes.
- **Waveform** a partir del RMS del audio del video, con zoom y rejilla de tiempo.
- **Reproductor** con libmpv: play/pausa, salto, velocidad 0.5×–2× y bucle.
- **Edición** de texto en la barra bajo el video, división en el playhead,
  deshacer/rehacer y búsqueda/filtro en la lista.
- **Estilos ASS** por preset con preview real: lo que ves es el mismo `.ass` que
  se exporta. Fuente, tamaño, color, contorno, sombra, negrita y cursiva.
- **Colores de identidad** del hablante como tinte de UI (barra y chip); nunca
  se mezclan con el render ni con el export.
- **Interfaz en inglés y español**, con atajos configurables y diálogo de
  preferencias.
- **Persistencia** del proyecto `.cdp`, del layout y de los atajos.

## Stack

| Capa | Tecnología |
|---|---|
| UI | egui 0.33 + eframe 0.33 (wgpu) |
| Video | libmpv vía `libmpv2`, frame compuesto en GPU |
| Subtítulos | libass por FFI propio (`rod-subs`) |
| Audio | `symphonia` para el análisis de waveform |
| Modelo | `rod-core` (proyecto, hablantes, captions, ASS, recovery) |

Los crates del workspace son `rod-core`, `rod-subs`, `rod-player` y `rod-app`.
Lo general (decodificar, sincronizar, rasterizar ASS) vive en crates reutilizables;
la semántica propia del editor (solapes, islas de audio, modelo de edición) se
implementa aquí.

## Requisitos

### Windows

- Rust 1.88+ con toolchain MSVC y *Visual Studio Build Tools* (carga de C++).
- libass de vcpkg: `vcpkg install libass:x64-windows` y añadir
  `%VCPKG_ROOT%\installed\x64-windows\bin` al `PATH`.
- `third_party/mpv/mpv.lib` y `mpv.def` están versionados. La DLL
  (`libmpv-2.dll`, ~116 MB) **no** está en el repo: el CI la descarga del
  release de [shinchiro/mpv-winbuild-cmake](https://github.com/shinchiro/mpv-winbuild-cmake/releases).
  Para trabajar en local, coloca `libmpv-2.dll` en `third_party/mpv/`
  (paquete `mpv-dev-x86_64-*.7z` del mismo release).

### Linux (probado en CI Ubuntu)

```bash
sudo apt-get install libxkbcommon-dev libwayland-dev libx11-dev libxcb1-dev \
  libxrandr-dev libxi-dev libxcursor-dev libmpv-dev ffmpeg
```

## Compilación y uso

```bash
cargo build --release
cargo run -p rod-app
```

Abre un video (o arrástralo a la ventana), importa el SRT de auto-subs, edita
en la timeline y guarda el proyecto. `?` muestra la tabla de atajos dentro de
la app.

## Tests

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
# Gate de rendimiento con 543 líneas (objetivo 8 ms/frame):
cargo test --release -p rod-app --test rendimiento
```

La mayoría de los tests no necesita archivos externos. Los que ejercitan el
reproductor usan el clip sintético de `testdata/`; en Linux el CI lo genera con
`ffmpeg` antes de correr.

## Auditoría visual

`ui-audit/` contiene el arnés de capturas (`egui_kittest` + wgpu), el plan de
68 hallazgos y el informe final:

- `ui-audit/HARNESS.md` — cómo regenerar capturas y la desviación de mpv.
- `ui-audit/PLAN.md` — hallazgos, propiedad de archivos y criterios.
- `ui-audit/before/` y `ui-audit/after/` — 91 capturas por fase en cuatro
  resoluciones, con `index.png`.
- `ui-audit/REPORT.md` — punto → arreglado / pendiente / descartado.

```bash
cargo run -p rod-app --features test-video --example ui_audit -- before
cargo run -p rod-app --features test-video --example ui_audit -- after
```

## Transparencia

Proyecto construido con asistencia intensiva de LLM para arquitectura,
implementación y depuración. El autor (estudiante de primer semestre de
Ingeniería de Sistemas e Informática) entiende el flujo general, pero todavía
está estudiando buena parte del código línea por línea. Se publica con honestidad
como proyecto de aprendizaje. Issues y PRs que señalen código confuso o
mejorable son bienvenidos.

## Licencia

MIT — ver [LICENSE](LICENSE). Copyright (c) 2026 Migue Echeverri.
