# Slice 16 — front-end egui

Semáforo (2026-11-24):

- [x] `cargo build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] captura real del arranque: `slice-16-inicio-egui.png` (1600×900, Windows)
- [ ] captura real del editor y comparación del frame contra el pipeline iced
- [ ] verificación en Linux (CachyOS): pendiente de ejecutar en ese equipo

## Cambios

- `rod-app` ya no depende de iced ni de iced_aw. `eframe 0.33` con backend wgpu
  dibuja la ventana; `Cargo.lock` no contiene iced.
- `Tarea` sustituye a `iced::Task`; los diálogos nativos se resuelven en un hilo
  y se recogen por un canal `mpsc` al empezar el frame.
- `ui/` contiene arranque, editor, paneles, transporte, timeline, tema e iconos
  propios. La lógica del proyecto sigue en los módulos de siempre.
- `ui::video` reutiliza `frame.wgsl`, `subs.wgsl` y `Gpu` mediante un
  `egui_wgpu::CallbackTrait`; el compositor y libass no se reescribieron.
- `timeline::Punto` sustituye a `iced::Point` en la aritmética de arrastres.

## Límites de verificación

- La captura confirma que la app arranca y dibuja con egui. El editor con video
  real y la comparación de píxeles contra el pipeline anterior quedan pendientes
  de una sesión con el video abierto.
- El formato de framebuffer se toma de `preferred_framebuffer_format(&[])`; si la
  comparación muestra desajuste, se pasa el formato real desde `CreationContext`.
- La tipografía usa las familias registradas por egui. Nombrar `Segoe UI
  Variable` sin registrar la fuente hacía entrar en pánico a epaint al dibujar
  el primer texto; se prefiere una app que arranca.
- La ventana conserva la barra nativa y además dibuja una cabecera propia: son
  dos franjas. Se decide en un slice aparte si se quita una.
