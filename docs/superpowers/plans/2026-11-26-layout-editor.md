# Layout del editor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Hacer que el video se pueda encoger arrastrando el divisor y que las pistas del timeline se puedan recorrer cuando no caben, sin tocar la lógica pura del timeline.

**Architecture:** El tope del `TopBottomPanel::bottom("timeline")` pasa a un helpers puro dependiente de la altura de la ventana. El scroll se guarda en `EditorApp` como filas (`f32`); la UI suma `scroll * ALTO_CARRIL` a la `y` del `Punto` y dibuja solo las filas visibles, de modo que `pulsar`/`arrastrar` siguen recibiendo coordenadas de carril absoluto. `ui/tema.rs` aporta el ancho de la scrollbar; `textos.rs` no cambia.

**Tech Stack:** Rust, egui 0.33, eframe, kittest para pruebas de UI.

**Spec:** `docs/superpowers/specs/2026-11-26-layout-editor-design.md`

## Global Constraints

- No cambiar `crates/app/src/timeline.rs` ni `crates/app/src/timeline/arrastre.rs` (lógica pura y sus tests).
- `ALTO_REGLA = 26.0`, `ALTO_CARRIL = 56.0`, `min_height = 160.0`, tope del timeline = `0.7 * alto de contenido`.
- Rueda simple → pistas; Ctrl+rueda → zoom anclado; Shift+rueda → pan horizontal; slider de zoom intacto.
- Regla de tiempo y etiqueta «TIEMPO» fijas; etiquetas de carril se desplazan.
- Scrollbar de 10 px a la derecha, solo cuando `filas > filas_visibles`.
- Sin texto nuevo; valores visuales desde `ui/tema.rs`.
- Verificación por tarea: `cargo fmt --all -- --check`, `cargo clippy -p rod-app -- -D warnings`, `cargo test --workspace`.

## Review Focus

- Arrancar el timeline con `Preferencias::alto_timeline` corrupto (mayor que la ventana) no debe dejar el video en 0 px → Task 1, `tope_respeta_la_ventana_y_el_suelo`.
- Con el campo de texto expandido (`alto_extra > 0`) el divisor no debe guardar una altura hinchada → Task 1, `alto_extra_no_guarda_altura_hinchada`.
- Tras desplazar las pistas, un clic en un clip debe seleccionar **ese** clip, no el de la misma posición visual sin scroll → Task 2, `clic_con_scroll_selecciona_el_carril_visible`.
- Un clic en la regla con pistas desplazadas debe seguir buscando tiempo, no seleccionar un clip → Task 2, `regla_sigue_busca_tiempo_con_scroll`.
- Con pocas pistas no debe aparecer scrollbar ni debe poder desplazarse → Task 3, `sin_scrollbar_cuando_todas_caben`.

---

## Task 1: Divisor libre y persistencia

**Files:**
- Modify: `crates/app/src/ui/editor.rs:36-62,112-125`
- Test: `crates/app/tests/layout.rs` (nuevo)

**Interfaces:**
- Consumes: `MedidasEditor`, `Preferencias`, `crate::ui::texto::alto_extra`, `egui::Context::content_rect`.
- Produces: `pub fn alto_maximo_timeline(alto_ventana: f32) -> f32` en `ui/editor.rs`; `MedidasEditor::alto_timeline` se limita a `[160.0, alto_maximo_timeline(...)]` dentro de `show`.

- [ ] **Step 1: Write the failing test**

En `tests/layout.rs`:

```rust
#[test]
fn tope_respeta_la_ventana_y_el_suelo() {
    assert_eq!(rod_app::ui::editor::alto_maximo_timeline(900.0), 630.0);
    assert_eq!(rod_app::ui::editor::alto_maximo_timeline(200.0), 160.0);
    let prefs = rod_app::preferencias::Preferencias {
        alto_timeline: 50_000.0,
        ..Default::default()
    };
    let medidas = rod_app::ui::editor::MedidasEditor::from(&prefs);
    assert_eq!(medidas.alto_timeline, 50_000.0); // `from` no conoce la ventana
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p rod-app --test layout`
Expected: FAIL, `alto_maximo_timeline` no existe.

- [ ] **Step 3: Implement `alto_maximo_timeline` y el divisor**

`pub fn alto_maximo_timeline(alto_ventana: f32) -> f32 { (alto_ventana * 0.7).max(160.0) }`. En `show`, antes de crear el panel, limitar `medidas.alto_timeline` a `(160.0..=alto_maximo_timeline(ctx.content_rect().height())).clamp(...)`. Cambiar `.max_height(alto_abajo)` por `.max_height(alto_maximo_timeline(ctx.content_rect().height()))`, conservando `.min_height(160.0)` y `.resizable(true)`. Mantener la condición actual de guardado (`alto_extra == 0`).

- [ ] **Step 4: Add the `alto_extra` test and make it pass**

```rust
#[test]
fn alto_extra_no_guarda_altura_hinchada() {
    use rod_app::editor_app::EditorApp;
    use rod_app::ui::editor::MedidasEditor;
    use rod_app::video::Gpu;
    use std::sync::{Arc, Mutex};

    let video = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/clip-15s-2audio.mp4"
    );
    let mut editor = EditorApp::desde_video(video).expect("abre el clip de prueba");
    editor.texto.lineas = 3;
    let estado = (
        editor,
        MedidasEditor {
            alto_timeline: 300.0,
            ..Default::default()
        },
        Arc::new(Mutex::new(None)),
    );
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1200.0, 700.0))
        .build_state(
            |ctx, (e, m, gpu): &mut (EditorApp, MedidasEditor, Arc<Mutex<Option<Gpu>>>)| {
                let _ = rod_app::ui::editor::show(
                    ctx,
                    e,
                    wgpu::TextureFormat::Rgba8Unorm,
                    gpu.clone(),
                    &rod_app::textos::EN,
                    &rod_app::comandos::MapaAtajos::por_defecto(),
                    m,
                );
            },
            estado,
        );
    h.run();
    assert_eq!(h.state().1.alto_timeline, 300.0);
}
```

El test comprueba que con el campo a 3 líneas no se escribe una altura
hinchada: la condición `alto_extra == 0` de `show` sigue gobernando el guardado.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p rod-app --test layout`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/app/src/ui/editor.rs crates/app/tests/layout.rs
git commit -m "feat(layout): divisor libre hasta el 70% de la ventana"
```

---

## Task 2: Scroll de pistas y desplazamiento de eventos

**Files:**
- Modify: `crates/app/src/editor_app.rs` (campo, mensaje, handler)
- Modify: `crates/app/src/ui/timeline.rs:127-128,184-188,251-311,358-375,487-498`
- Test: `crates/app/tests/layout.rs`

**Interfaces:**
- Consumes: `EditorApp.scroll_pistas`, `crate::timeline::{ALTO_REGLA, ALTO_CARRIL, Punto}`, `MensajeEditor::Modificadores`.
- Produces: `EditorApp::scroll_pistas: f32`; `MensajeEditor::ScrollPistas(f32)`; en `ui/timeline.rs`: `pub fn filas_visibles(alto: f32) -> usize`, `pub fn max_scroll_pistas(filas: usize, alto: f32) -> f32`, `pub fn desplazar(scroll: f32, lineas: f32, filas: usize, alto: f32) -> f32`, `pub fn punto(rel: egui::Vec2, scroll: f32) -> crate::timeline::Punto`.

- [ ] **Step 1: Write the failing tests**

En `tests/layout.rs`:

```rust
#[test]
fn limites_de_scroll_por_pistas() {
    use rod_app::ui::timeline::{filas_visibles, max_scroll_pistas, desplazar};
    // 700 px de alto del panel => regla 26 + 12 carriles de 56 = 698.
    assert_eq!(filas_visibles(700.0), 12);
    assert_eq!(max_scroll_pistas(15, 700.0), 3.0);
    assert_eq!(max_scroll_pistas(4, 700.0), 0.0);
    assert_eq!(desplazar(2.0, 5.0, 15, 700.0), 3.0);
    assert_eq!(desplazar(0.0, -1.0, 15, 700.0), 0.0);
}

#[test]
fn clic_con_scroll_selecciona_el_carril_visible() {
    use rod_app::ui::timeline::punto;
    let p = punto(egui::vec2(300.0, ALTO_REGLA + 20.0), 2.0);
    // Carril 2 absoluto: el que se ve en la primera fila tras bajar dos.
    assert_eq!(p.y, ALTO_REGLA + 2.0 * ALTO_CARRIL + 20.0);
}

#[test]
fn regla_sigue_busca_tiempo_con_scroll() {
    let p = punto(egui::vec2(300.0, 8.0), 2.0);
    assert!(p.y < ALTO_REGLA);
}
```

Importar `ALTO_REGLA`/`ALTO_CARRIL` desde `rod_app::timeline`.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p rod-app --test layout`
Expected: FAIL por funciones y campo inexistentes.

- [ ] **Step 3: Implement estado y helpers**

`filas_visibles(alto) = ((alto - ALTO_REGLA) / ALTO_CARRIL).floor().max(1.0) as usize`. `max_scroll_pistas` = `filas.saturating_sub(filas_visibles(alto)) as f32`. `desplazar` = `(scroll + lineas).clamp(0.0, max_scroll_pistas(...))`. `punto(rel, scroll)` = `Punto::new(rel.x - GUTTER, if rel.y < ALTO_REGLA { rel.y } else { rel.y + scroll * ALTO_CARRIL })`. Añadir `scroll_pistas: f32` a `EditorApp` (default 0.0) y la variante `MensajeEditor::ScrollPistas(f32)`, cuyo handler asigna el valor recibido.

- [ ] **Step 4: Conectar la rueda y el dibujo**

En `ui/timeline.rs`: leer `(modificadores, lineas)` como hoy; si `ctrl || command` emitir `Mensaje::Rueda` (zoom, igual que ahora), si `shift` emitir `Mensaje::Rueda` (pan), si no emitir `MensajeEditor::ScrollPistas(desplazar(state.scroll_pistas, lineas, filas, rect.height()))`. Pasar `state.scroll_pistas` a todos los `punto(...)`. En el dibujo de carriles, onda y clips, calcular `y = rect.min.y + ALTO_REGLA + (fila as f32 - scroll) * ALTO_CARRIL` y saltar filas fuera de `[rect.min.y + ALTO_REGLA, rect.max.y]`. La regla y «TIEMPO» se pintan sin offset.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p rod-app --test layout && cargo test --workspace`
Expected: PASS, sin cambios en los tests de `crates/app/src/timeline.rs`.

- [ ] **Step 6: Commit**

```bash
git add crates/app/src/editor_app.rs crates/app/src/ui/timeline.rs crates/app/tests/layout.rs
git commit -m "feat(layout): scroll de pistas con regla fija"
```

---

## Task 3: Scrollbar visible y capturas

**Files:**
- Modify: `crates/app/src/ui/tema.rs` (constante `ANCHO_SCROLLBAR: f32 = 10.0`)
- Modify: `crates/app/src/ui/timeline.rs:184-193,440-500`
- Modify: `crates/app/examples/capturas.rs`
- Test: `crates/app/tests/layout.rs`

**Interfaces:**
- Consumes: `max_scroll_pistas`, `filas_visibles`, `EditorApp.scroll_pistas`, `MensajeEditor::ScrollPistas`.
- Produces: `pub fn scroll_desde_barra(y_rel: f32, alto: f32, filas: usize) -> f32` en `ui/timeline.rs`.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn sin_scrollbar_cuando_todas_caben() {
    assert_eq!(rod_app::ui::timeline::max_scroll_pistas(4, 700.0), 0.0);
    // scroll_desde_barra devuelve 0 con todas las pistas visibles.
    assert_eq!(rod_app::ui::timeline::scroll_desde_barra(350.0, 700.0, 4), 0.0);
}

#[test]
fn scrollbar_mapa_el_tamanio_de_la_barra() {
    // Barra de 700 px, 15 filas, 12 visibles: el recorrido son 3 filas.
    assert_eq!(rod_app::ui::timeline::scroll_desde_barra(0.0, 700.0, 15), 0.0);
    assert_eq!(rod_app::ui::timeline::scroll_desde_barra(700.0, 700.0, 15), 3.0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p rod-app --test layout sin_scrollbar`
Expected: FAIL por `scroll_desde_barra` inexistente.

- [ ] **Step 3: Implement `scroll_desde_barra` y la barra**

`scroll_desde_barra(y_rel, alto, filas)` = `(y_rel / alto * max_scroll_pistas(filas, alto)).clamp(0.0, max)`, con `alto <= 0` → 0. En `mostrar`, estrechar el rect principal `ANCHO_SCROLLBAR` px y reservar esa franja. Solo si `max_scroll_pistas(filas, rect.height()) > 0.0`: pintar riel con `tema::LATERAL`, pulgar con altura `alto * filas_visibles / filas` en `tema::BORDE_CONTROL`, y `ui.interact` con `egui::Id::new("scroll-pistas")` y `Sense::click_and_drag()`; al arrastrar o hacer clic emitir `MensajeEditor::ScrollPistas(scroll_desde_barra(pos.y - rect.min.y, rect.height(), filas))`. La rueda sobre la barra sigue controlando pistas por el camino de Task 2.

- [ ] **Step 4: Añadir capturas y revisarlas**

En `capturas.rs`, fase `revision`, añadir `capturar_editor` con 12 hablantes y `Panel::Subtitulos` a 1100/1440/2560 px (`revision-pistas-{ancho}.png`). Verificar con `read` que la barra aparece, la regla sigue arriba y no hay desborde.

- [ ] **Step 5: Run full verification**

Run: `cargo fmt --all -- --check && cargo clippy -p rod-app -- -D warnings && cargo test --workspace`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/app docs/checklists/rediseno
git commit -m "feat(layout): scrollbar de pistas y capturas"
```

---

## Self-review

- Spec §divisor: Task 1 cubre tope, suelo y persistencia; `alto_extra` se comprueba en su propio test.
- Spec §rueda: Task 2 reparte rueda/Ctrl/Shift y mantiene el slider; no hay test de Ctrl+rueda porque el camino es el código existente sin cambios.
- Spec §regla fija: Task 2, `regla_sigue_busca_tiempo_con_scroll`.
- Spec §scrollbar: Task 3, barra de 10 px solo cuando hace falta.
- Spec §no tocar lógica pura: ninguna tarea edita `timeline.rs` ni `timeline/arrastre.rs`; los offsets se aplican en `ui/timeline.rs`.
- Tipos: `filas_visibles -> usize`, `max_scroll_pistas -> f32`, `desplazar(f32, f32, usize, f32) -> f32`, `punto(Vec2, f32) -> Punto`, `scroll_desde_barra(f32, f32, usize) -> f32` se definen una vez y se consumen igual en todas las tareas.
