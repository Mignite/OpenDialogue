# Menú, atajos y sistema visual — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Menú dibujado por iced en ambas pantallas, atajos del oráculo con un único despachador y sistema visual swiss/brutalista con acento mostaza.

**Architecture:** `comandos.rs` es la única fuente de verdad: etiqueta, atajo y disponibilidad. `menu.rs` construye el `MenuBar` de `iced_aw` desde ese registro; `keyboard::listen` mapea al mismo `Comando`; `main.rs::ejecutar` lo despacha a la pantalla correspondiente. `tema.rs` centraliza los tokens visuales.

**Tech Stack:** Rust 2024, iced 0.14, iced_aw 0.14.1 (feature `menu`), `rfd`, `rod-core`.

**Spec:** `docs/superpowers/specs/2026-11-23-menu-atajos-visual.md`

## Global Constraints

- Sin `unsafe`, sin macros complejas, sin crates nuevos salvo `iced_aw`.
- Colores de hablante son semánticos: no se sustituyen por el mostaza.
- Cero emojis: ASCII o SVG. Menú dibujado por la app, no nativo.
- Cada tarea cierra con build, fmt --check, clippy -D warnings y tests.

## Tasks

### Task 1: registro de comandos y mapeo de teclado

- Create `crates/app/src/comandos.rs`; Modify `lib.rs`.
- `pub enum Comando` con las acciones del spec; `pub fn desde_keyboard(key: &Key, modifiers: Modifiers) -> Option<Comando>`; `pub fn etiqueta(Comando) -> &'static str`; `pub fn atajo(Comando) -> &'static str`; `pub fn disponible_en_inicio(Comando) -> bool`.
- Tests: cada atajo del oráculo mapea a su comando; Ctrl+S y Ctrl+F se reconocen aunque haya texto enfocado; las teclas de hablante 1..9 mapean a `AsignarHablante`; sin duplicados.
- Commit.

### Task 2: menú en ambas pantallas y despachador único

- Modify `Cargo.toml` (+ `iced_aw` feature `menu`), Create `menu.rs`, Modify `main.rs`, `inicio.rs`, `editor_app.rs`.
- `menu::barra(estado) -> Element<Mensaje>` construye `MenuBar` desde `comandos`, con `Archivo`, `Edición`, `Ver`, `Exportar`, `Estilos`, `Ayuda`; en inicio las acciones no aplicables salen deshabilitadas.
- `main.rs::ejecutar(Comando, &mut Pantalla) -> Task<Mensaje>`: comandos de app en el enrutador; comandos de edición mapeados a `MensajeEditor` existentes o a variantes nuevas mínimas.
- `subscription` delega el teclado a `comandos::desde_keyboard` y elimina el listen actual duplicado.
- Tests: menú y teclado producen el mismo `Comando`; ejecutar Guardar desde menú y desde Ctrl+S coincide.
- Commit.

### Task 3: sistema visual swiss/brutalista

- Create `tema.rs`; Modify `inicio.rs`, `menu.rs`, `editor_app.rs` (estilos).
- Tokens: superficie `#0C0C0D`, panel `#141416`, hairline `#FFFFFF26`, texto `#F3F3F3`, secundario `#9A9A9A`, mostaza `#D9A441`, esquinas 3 px.
- Aplicar a barra de menú, botones y contenedores del arranque y del editor; rejilla 2:1, jerarquía tipográfica, sin gradientes ni sombras. No tocar `Hablante::color`.
- Commit.

### Task 4: verificación y evidencia

- Capturas del arranque y del editor con menú; comprobar atajos en ejecución real; checklist en `docs/checklists/slice-10-menu-atajos-visual.md`.
- Semáforo completo.
- Commit.
