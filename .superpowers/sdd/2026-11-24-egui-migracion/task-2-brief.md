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

