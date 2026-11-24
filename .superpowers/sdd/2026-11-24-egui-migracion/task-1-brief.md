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

