# Migración a egui — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Sustituir el front-end de iced por egui 0.33 en `rod-app`, conservando el motor (`rod-core`, `rod-subs`, `rod-player`) y el pipeline GPU de video + libass.

**Architecture:** Los módulos egui se construyen primero dentro de `rod-app` mientras el binario iced sigue compilando. Se sustituye `iced::Task` por un enum `Tarea` propio y los diálogos por un canal `mpsc`. El pin de wgpu 27.0.1 compartido con `egui-wgpu 0.33` permite portar `video::Gpu` a un `CallbackTrait` sin tocar los `.wgsl`. El último task cambia el binario a `eframe` y retira iced.

**Tech Stack:** Rust 2024, eframe/egui/egui-wgpu 0.33.0, wgpu 27.0.1, libmpv2 7.0.1, libass propio, egui_kittest.

**Spec:** `docs/superpowers/specs/2026-11-24-egui-migracion-design.md`

## Global Constraints

- Versiones exactas: `eframe = "=0.33.0"`, `egui = "=0.33.0"`, `egui-wgpu = "=0.33.0"`, `wgpu = "=27.0.1"`.
- `edition = "2024"`, `rust-version = "1.88"`.
- Sin `unsafe` nuevo fuera del FFI ya existente en `rod-subs`/`rod-player`.
- Sin crates nuevas fuera de la spec. No se añade `egui_dock`.
- Sin `egui-sharkplayer`: `rod-player` sigue siendo el único player.
- Invariante: `preview::ass_del_proyecto` es la única fuente del `.ass`; canvas `CANVAS = (1920, 1080)`.
- `frame.wgsl` y `subs.wgsl` no se modifican.
- Todo slice cierra con `cargo build`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` y `cargo test --workspace` en verde, en Windows y Linux.
- El trabajo ocurre en `C:/Dev/RustyOpenDialogue-egui` (rama `egui-ui`). Los 7 archivos modificados en `main` no se tocan.
- Un commit por task.

## Review Focus

| Entrada / condición | Comportamiento esperado | Test que lo fija |
|---|---|---|
| Rect de video no cuadrado y ventana redimensionada | Letterbox sin deformar y subtítulos dentro del rect | Task 6: aserción de `escala_letterbox` para frame 1920×1080 en rect 1000×400 + captura comparada |
| Cambio de resolución entre dos frames | La textura se rehace y no hay pánico | Task 6: dos `Frame` de tamaños distintos en el mismo `Gpu` |
| Diálogo cancelado (`None`) | No se abre editor, no cambia el mensaje de error | Task 1: `RespuestaDialogo::Video(None)` vía `inicio::update` |
| Dos respuestas de diálogo seguidas | Se drenan las dos, en orden, sin perder la primera | Task 1: channel con dos `RespuestaDialogo` |
| `time-pos` todavía no disponible al abrir | Timeline y transporte no pánico; playhead en 0 | Task 7: `EditorApp` con posición `None` |

---

### Task 1: Sustituto de `Task` y diálogos por canal

**Files:**
- Modify: `Cargo.toml`, `crates/app/Cargo.toml`, `crates/app/src/lib.rs`, `crates/app/src/inicio.rs`, `crates/app/src/main.rs`
- Create: `crates/app/src/tarea.rs`, `crates/app/src/dialogos.rs`

**Interfaces:**
- Consumes: `inicio::{nuevo, update, EstadoInicio, MensajeInicio, Salida}` (ya existen).
- Produces:
  - `tarea::Tarea` — variantes `Ninguna`, `DialogoVideo`, `DialogoProyecto`, `DialogoCarpeta`, `DialogoGuardar`, `EnfocarTexto`.
  - `tarea::Tarea::none() -> Tarea`.
  - `dialogos::RespuestaDialogo` — `Video(Option<PathBuf>)`, `Proyecto(Option<PathBuf>)`, `Carpeta(Option<PathBuf>)`, `Guardar(Option<PathBuf>)`.
  - `dialogos::pedir_video(Sender<RespuestaDialogo>)`, `pedir_proyecto`, `pedir_carpeta`, `pedir_guardar`.
  - `inicio::update(&mut EstadoInicio, MensajeInicio) -> (Tarea, Option<Salida>)`.

- [ ] **Step 1: Añadir dependencias egui sin quitar iced todavía**

En `[workspace.dependencies]`: `egui = "=0.33.0"`, `eframe = "=0.33.0"`, `wgpu = "=27.0.1"`.
En `crates/app/Cargo.toml`: `egui`, `eframe = { workspace = true, features = ["wgpu"] }`, `wgpu`, y `egui_kittest = { version = "0.33.0", features = ["eframe", "wgpu"] }` en `[dev-dependencies]`.

- [ ] **Step 2: Escribir los tests que fallan**

```rust
// crates/app/src/tarea.rs
#[cfg(test)]
mod tests {
    use super::Tarea;
    use crate::inicio::{self, MensajeInicio};

    #[test]
    fn none_es_la_tarea_vacia() { assert_eq!(Tarea::none(), Tarea::Ninguna); }

    #[test]
    fn abrir_video_pide_dialogo() {
        let mut estado = inicio::nuevo(Vec::new(), None);
        let (tarea, salida) = inicio::update(&mut estado, MensajeInicio::AbrirVideo);
        assert_eq!(tarea, Tarea::DialogoVideo);
        assert!(salida.is_none());
    }

    #[test]
    fn dialogo_cancelado_no_abre_nada() {
        let mut estado = inicio::nuevo(Vec::new(), None);
        let (tarea, salida) = inicio::update(&mut estado, MensajeInicio::DialogoVideo(None));
        assert_eq!(tarea, Tarea::Ninguna);
        assert!(salida.is_none());
    }

    #[test]
    fn dos_respuestas_se_drenan_en_orden() {
        use crate::dialogos::RespuestaDialogo;
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(RespuestaDialogo::Video(None)).unwrap();
        tx.send(RespuestaDialogo::Proyecto(None)).unwrap();
        assert!(matches!(rx.try_recv(), Ok(RespuestaDialogo::Video(None))));
        assert!(matches!(rx.try_recv(), Ok(RespuestaDialogo::Proyecto(None))));
    }
}
```

- [ ] **Step 3: Verificar que falla**

Run: `cargo test -p rod-app tarea::tests`
Expected: FAIL — `no module named tarea`.

- [ ] **Step 4: Implementar `Tarea` y `dialogos`**

`Tarea` es un `enum` derivado de `Debug, Clone, PartialEq, Eq`. `dialogos` crea el canal y lanza `std::thread::spawn` con `rfd::FileDialog` síncrono dentro del hilo; el hilo envía la `RespuestaDialogo` por el `Sender`. Nunca se llama a `rfd` en el frame principal.

- [ ] **Step 5: Migrar `inicio::update` y `main.rs`**

Cambiar la firma a `(Tarea, Option<Salida>)`. Sustituir los dos `Task::future(async { ... })` por `Tarea::DialogoVideo` y `Tarea::DialogoProyecto`. En `main.rs`, quitar `.map(Mensaje::Inicio)` y manejar la `Tarea` devuelta directamente; al recibir `DialogoVideo`/`DialogoProyecto` llamar a `dialogos::pedir_*` con el `Sender` guardado en `AppState`.

- [ ] **Step 6: Verificar que pasa**

Run: `cargo test -p rod-app tarea::tests`
Expected: 4 tests PASS.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml crates/app/Cargo.toml crates/app/src/lib.rs crates/app/src/tarea.rs crates/app/src/dialogos.rs crates/app/src/inicio.rs crates/app/src/main.rs
git commit -m "feat: sustituir Task por Tarea y dialogos por canal"
```

---

### Task 2: Editor sin `Task` ni `rfd` en el frame

**Files:**
- Modify: `crates/app/src/editor_app.rs`, `crates/app/src/comandos.rs`, `crates/app/src/main.rs`

**Interfaces:**
- Consumes: `tarea::Tarea`, `dialogos::RespuestaDialogo`.
- Produces:
  - `editor_app::update(&mut EditorApp, MensajeEditor) -> Tarea`.
  - `MensajeEditor` gana las variantes `DialogoVideo(Option<PathBuf>)`, `DialogoProyecto(Option<PathBuf>)`, `DialogoSrt(Option<PathBuf>)`, `DialogoAss(Option<PathBuf>)`, `DialogoCarpeta(Option<PathBuf>)`, `DialogoGuardar(Option<PathBuf>)`.
  - `comandos::desde_input(key: egui::Key, modifiers: egui::Modifiers) -> Option<Comando>`.

- [ ] **Step 1: Escribir los tests que fallan**

```rust
// crates/app/src/editor_app.rs, módulo tests
#[test]
fn guardar_como_pide_dialogo() {
    let mut editor = EditorApp::desde_video("testdata/video.mp4").expect("video de prueba");
    let tarea = update(&mut editor, MensajeEditor::Comando(Comando::GuardarComo));
    assert_eq!(tarea, Tarea::DialogoGuardar);
}

// crates/app/src/comandos.rs, módulo tests
#[test]
fn ctrl_s_es_guardar() {
    let mods = egui::Modifiers { ctrl: true, ..Default::default() };
    assert_eq!(desde_input(egui::Key::S, mods), Some(Comando::Guardar));
}
```

- [ ] **Step 2: Verificar que falla**

Run: `cargo test -p rod-app guardar_como_pide_dialogo ctrl_s_es_guardar`
Expected: FAIL — `Tarea` no es el tipo devuelto; `desde_input` no existe.

- [ ] **Step 3: Convertir `editor_app`**

Cambiar el tipo de retorno de `update` y de los ayudantes que devuelven `Task<MensajeEditor>` a `Tarea`; `Task::none()` → `Tarea::Ninguna`; `iced::widget::operation::focus(ID_TEXTO_SUBTITULO)` → `Tarea::EnfocarTexto`. Sustituir las llamadas inline a `rfd::FileDialog` en las líneas 808, 828, 1029, 1086, 1106, 1130, 1180, 1207 y 1228 por devolver la variante `Tarea::Dialogo*` correspondiente; el resultado llega después como `MensajeEditor::Dialogo*(ruta)`. Borrar `subscription` (el tick lo hará `request_repaint`).

- [ ] **Step 4: Portar `desde_keyboard` a `desde_input`**

Mismo mapa de teclas, con `egui::Key` y `egui::Modifiers`. `Modificadores` en `EditorApp` pasa a ser `egui::Modifiers` (o un struct propio con los mismos campos si egui no es `Default`).

- [ ] **Step 5: Verificar que pasa**

Run: `cargo test -p rod-app`
Expected: PASS (todos los tests anteriores siguen verde).

- [ ] **Step 6: Commit**

```bash
git add crates/app/src/editor_app.rs crates/app/src/comandos.rs crates/app/src/main.rs
git commit -m "feat: editor usa Tarea y dialogos asincronos"
```

---

### Task 3: Tema egui

**Files:**
- Create: `crates/app/src/ui/mod.rs`, `crates/app/src/ui/tema.rs`
- Modify: `crates/app/src/lib.rs`

**Interfaces:**
- Consumes: valores de `tema.rs` actual (`FONDO`, `SUPERFICIE`, `DIM`, `TEXTO`, `SECUNDARIO`, radios).
- Produces:
  - `ui::tema::FONDO: egui::Color32`, `SUPERFICIE`, `ACENTO`, `TEXTO`, `DIM`.
  - `ui::tema::RADIO_CONTROL: f32 = 4.0`, `RADIO_SUPERFICIE: f32 = 6.0`.
  - `ui::tema::estilo() -> egui::Style`.

- [ ] **Step 1: Test**

```rust
#[test]
fn acento_es_azul_corporativo() {
    assert_eq!(ui::tema::ACENTO, egui::Color32::from_hex("#1473E6").unwrap());
}
#[test]
fn la_superficie_es_mas_redondeada_que_el_control() {
    assert!(ui::tema::RADIO_SUPERFICIE > ui::tema::RADIO_CONTROL);
}
```

- [ ] **Step 2: Verificar que falla** — `cargo test -p rod-app tema` → FAIL, módulo `ui` inexistente.

- [ ] **Step 3: Implementar** `estilo()` con `Visuals` gris neutro, acento solo en CTA/foco/pestaña activa, radios 4/6, sin sombras ni gradientes. Fuentes: `Segoe UI Variable` y `Cascadia Mono` en Windows, defaults de egui en otros sistemas.

- [ ] **Step 4: Verificar que pasa** — `cargo test -p rod-app tema` → PASS.

- [ ] **Step 5: Commit** — `git commit -m "feat: tema egui"`

---

### Task 4: Pantalla de arranque en egui

**Files:**
- Create: `crates/app/src/ui/inicio.rs`
- Modify: `crates/app/src/ui/mod.rs`

**Interfaces:**
- Consumes: `inicio::{EstadoInicio, MensajeInicio, cantidad_visible, resumen_reciente, texto_vacio_recientes}`.
- Produces: `ui::inicio::mostrar(ui: &mut egui::Ui, estado: &mut EstadoInicio) -> Option<MensajeInicio>`.

- [ ] **Step 1: Test con `egui_kittest`**

```rust
#[test]
fn arranque_muestra_tres_recientes_y_boton_abrir() {
    let mut estado = inicio::nuevo(tres_recientes(), None);
    let mut harness = egui_kittest::Harness::new_ui(|ui| {
        ui::inicio::mostrar(ui, &mut estado);
    });
    harness.run();
    assert!(harness.get_by_label("Abrir video").is_some());
}
```

- [ ] **Step 2: Verificar que falla** — `cargo test -p rod-app arranque_muestra` → FAIL.

- [ ] **Step 3: Implementar `mostrar`** — dropzone, recientes (3 o todos con `Ver todos`), aviso de recovery con «Recuperar»/«Descartar», mensaje de error. Cada control devuelve su `MensajeInicio`; la vista no escribe archivos ni construye editores.

- [ ] **Step 4: Verificar que pasa** — `cargo test -p rod-app arranque_muestra` → PASS.

- [ ] **Step 5: Commit** — `git commit -m "feat: arranque en egui"`

---

### Task 5: Editor y paneles en egui

**Files:**
- Create: `crates/app/src/ui/editor.rs`, `crates/app/src/ui/paneles/mod.rs`, `crates/app/src/ui/paneles/subtitulos.rs`, `crates/app/src/ui/paneles/hablantes.rs`, `crates/app/src/ui/paneles/estilos.rs`, `crates/app/src/ui/transporte.rs`, `crates/app/src/ui/iconos.rs`
- Modify: `crates/app/src/ui/mod.rs`

**Interfaces:**
- Consumes: `editor_app::{EditorApp, MensajeEditor}`, `EstadoEditor`, `Comando`, `ui::tema`.
- Produces: `ui::editor::mostrar(ui: &mut egui::Ui, state: &mut EditorApp) -> Option<MensajeEditor>`; `ui::iconos::pintar(ui: &mut egui::Ui, rect: egui::Rect, icono: Icono, color: egui::Color32)`; y `editor_app::sucio(&EditorApp) -> bool` (se hace público; hoy es privado).

- [ ] **Step 1: Test**

```rust
#[test]
fn teclear_texto_y_confirmar_es_un_solo_paso_de_historial() {
    let mut editor = EditorApp::desde_video("testdata/video.mp4").unwrap();
    // simular: seleccionar caption, escribir, confirmar con Enter
    let mut harness = egui_kittest::Harness::new_ui(|ui| { ui::editor::mostrar(ui, &mut editor); });
    harness.run();
    assert!(editor_app::sucio(&editor));
}
```

- [ ] **Step 2: Verificar que falla** — `cargo test -p rod-app teclear_texto` → FAIL.

- [ ] **Step 3: Implementar paneles** — `TopPanel` (marca, menú, estado, controles de ventana), `SidePanel` derecha con tres pestañas, `CentralPanel` para video y transporte, timeline abajo. La edición de texto confirma al perder foco o Enter, nunca por tecla, para conservar la granularidad del historial. Iconos propios con `Painter`, rejilla 24 y trazo 1,7 px.

- [ ] **Step 4: Verificar que pasa** — `cargo test -p rod-app teclear_texto` → PASS.

- [ ] **Step 5: Commit** — `git commit -m "feat: editor y paneles en egui"`

---

### Task 6: Video y subtítulos con `egui_wgpu`

**Files:**
- Modify: `crates/app/src/video/mod.rs`
- Create: `crates/app/src/ui/video.rs`

**Interfaces:**
- Consumes: `video::{Frame, Capa, Gpu}`, `video::geometria::escala_letterbox`, `rod_subs::Bitmap`.
- Produces:
  - `struct VideoCallback { frame: Frame, capas: Vec<Capa>, rect_px: [u32; 2] }` que implementa `egui_wgpu::CallbackTrait`.
  - `ui::video::pintar(ui: &mut egui::Ui, rect: egui::Rect, frame: Frame, capas: Vec<Capa>)`.
  - `Gpu::de_prueba(format: wgpu::TextureFormat) -> Gpu` y `Gpu::subir(&mut self, frame: &Frame, capas: &[Capa])` (nueva interfaz pública; hoy `ensure_texture` y `subir_capas` son privados).

- [ ] **Step 1: Tests**

```rust
#[test]
fn letterbox_en_rect_no_cuadrado() {
    let (x, y) = geometria::escala_letterbox((1920, 1080), (1000.0, 400.0));
    assert!((x - 400.0 / 1080.0).abs() < 1e-6 || (y - 400.0 / 1080.0).abs() < 1e-6);
}

#[test]
fn cambio_de_resolucion_no_panica() {
    let mut gpu = Gpu::de_prueba(wgpu_format());
    gpu.subir(frame_de(1920, 1080), &[]);
    gpu.subir(frame_de(1280, 720), &[]);
}
```

- [ ] **Step 2: Verificar que falla** — `cargo test -p rod-app letterbox_en_rect` → FAIL.

- [ ] **Step 3: Portar `Gpu` a `CallbackTrait`**

`prepare(&self, device, queue, screen_descriptor, encoder, resources) -> Vec<CommandBuffer>`: obtiene o crea `Gpu` en `resources` (type-map), sube textura, uniform y atlas con el mismo código de `video/mod.rs`. `paint(&self, info, render_pass, resources)`: usa `info.viewport_in_pixels()` como viewport y ejecuta los dos draw calls con el mismo blending. `ui.painter().add(egui_wgpu::Callback::new_paint_callback(rect, callback))`. Los `.wgsl` no cambian.

- [ ] **Step 4: Verificar que pasa**

Run: `cargo test -p rod-app letterbox_en_rect cambio_de_resolucion`
Expected: PASS. Además, captura real: el frame compuesto contra `docs/checklists/slice-5-preview-despues.png`, diferencia ≤ 1 % de píxeles.

- [ ] **Step 5: Commit** — `git commit -m "feat: compositor video+libass sobre egui_wgpu"`

---

### Task 7: Timeline en egui

**Files:**
- Create: `crates/app/src/ui/timeline.rs`
- Modify: `crates/app/src/ui/mod.rs`, `crates/app/src/timeline/arrastre.rs` (hacer públicos los tipos que falten)

**Interfaces:**
- Consumes: `EstadoEditor`, `Vista`, `timeline::arrastre` y el snap de `rod-core`.
- Produces: `ui::timeline::mostrar(ui: &mut egui::Ui, state: &mut EditorApp) -> Option<MensajeEditor>`; `ui::timeline::playhead(&EditorApp) -> f64`; `EditorApp::desde_video_sin_posicion(ruta: &str) -> Result<EditorApp, String>` (helper de test: construye el editor con `time-pos` ausente).

- [ ] **Step 1: Tests**

```rust
#[test]
fn arrastre_termina_en_el_tiempo_soltado_y_es_un_paso() {
    // pulsar en t=1.0, soltar en t=2.5: el caption termina en 2.5 y puede_deshacer()
}

#[test]
fn sin_time_pos_el_playhead_es_cero() {
    let editor = EditorApp::desde_video_sin_posicion("testdata/video.mp4");
    assert_eq!(ui::timeline::playhead(&editor), 0.0);
}
```

- [ ] **Step 2: Verificar que falla** — `cargo test -p rod-app arrastre_termina` → FAIL.

- [ ] **Step 3: Implementar widget** con `Painter`: waveform, carriles por hablante, clips, playhead, marquee, eje con zoom/pan y `Vista::con_inicio`. El arrastre recalcula desde la posición guardada al pulsar y registra una sola instantánea en el primer frame que cambia.

- [ ] **Step 4: Verificar que pasa** — `cargo test -p rod-app timeline` → PASS. Captura real: `docs/checklists/slice-16-timeline-egui.png`.

- [ ] **Step 5: Commit** — `git commit -m "feat: timeline en egui"`

---

### Task 8: Binario eframe y retirada de iced

**Files:**
- Rewrite: `crates/app/src/main.rs`
- Modify: `crates/app/src/lib.rs`, `crates/app/src/editor_app.rs`, `crates/app/src/inicio.rs`, `crates/app/src/timeline.rs`, `crates/app/src/menu.rs`, `crates/app/src/tema.rs`, `crates/app/src/iconos.rs`, `Cargo.toml`, `crates/app/Cargo.toml`, `crates/app/tests/arranque.rs`

**Interfaces:**
- Consumes: todos los `ui::*` de los tasks anteriores, `tarea::Tarea`, `dialogos::RespuestaDialogo`.
- Produces: `struct RustyDialogue` que implementa `eframe::App` con `fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame)`.

- [ ] **Step 1: Test**

```rust
#[test]
fn arranca_en_inicio_sin_abrir_player() {
    let state = estado_de_prueba();
    assert!(matches!(state.pantalla, Pantalla::Inicio(_)));
}
```

- [ ] **Step 2: Verificar que falla** — `cargo test -p rod-app --test arranque` → FAIL (el test aún importa `iced`).

- [ ] **Step 3: Escribir `main.rs`**

`eframe::run_native("RustyDialogue", NativeOptions::default(), Box::new(...))`. `AppState` guarda `Pantalla`, el `Receiver<RespuestaDialogo>` y el `Frame` del player. En cada `update`: drenar el canal, despachar mensajes de UI, `poll_frame()`, `request_repaint()` si hay video. Quitar los view de iced de `editor_app.rs`, `inicio.rs`, `timeline.rs`, `menu.rs`, `tema.rs`, `iconos.rs`. Borrar `iced` e `iced_aw` de los manifiestos y regenerar `Cargo.lock`.

- [ ] **Step 4: Verificar que pasa**

Run: `cargo build && cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test --workspace`
Expected: todo PASS. Captura real en Windows y Linux: arranque y editor con subtítulos.

- [ ] **Step 5: Commit** — `git commit -m "feat: binario eframe, retirada de iced"`

---

### Task 9: Cierre y verificación

**Files:**
- Create: `docs/checklists/slice-16-egui.md`
- Modify: `BITACORA.md`

**Interfaces:** ninguna.

- [ ] **Step 1: Checklist con semáforo** — build, fmt, clippy, tests, capturas de Windows y Linux, comparación del frame contra el pipeline iced.

- [ ] **Step 2: BITACORA** — registrar el coste por frame medido a 1080p y cualquier desviación del spec.

- [ ] **Step 3: Verificar** — `git status --short` limpio en el worktree; `git log --oneline -9` con un commit por task.

- [ ] **Step 4: Commit** — `git commit -m "docs: cierre de la migracion a egui"`
