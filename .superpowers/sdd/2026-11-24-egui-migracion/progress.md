# SDD ledger — plan: docs/superpowers/plans/2026-11-24-egui-migracion.md

Setup: worktree verificado en `C:/Dev/RustyOpenDialogue-egui` (rama `egui-ui`, HEAD 13412f0). `sdd-workspace` no produjo salida ni directorio en este Windows; se crea el workspace manualmente en la ruta que el script habría devuelto. Coste si está mal: solo thrash de directorio scratch.

Pre-flight: Task 1 produce `Tarea` / `inicio::update -> (Tarea, Option<Salida>)`; Task 2 y Task 8 lo consumen. Sin divergencia.

Pre-flight: Task 2 produce `editor_app::update -> Tarea`; Task 8 lo consume. Sin divergencia.

Pre-flight: Task 3 produce `ui::tema`; Tasks 4, 5 y 7 lo consumen. Sin divergencia.

Pre-flight: Task 5 dibuja un rectángulo oscuro en el lugar del video y un timeline placeholders; Task 6 y Task 7 sustituyen esos huecos. Ruling: se mantiene el orden del plan para que cada task cierre con build verde; el coste es un frame sin video ni timeline durante los tasks 5 a 7, que no se entrega al usuario.

Pre-flight: Task 6 produce `ui::video::pintar` y `Gpu::{de_prueba, subir}`; Task 8 lo consume. Sin divergencia.

Pre-flight: Task 7 produce `ui::timeline::mostrar` y `playhead`; Task 8 lo consume. Sin divergencia.
Task 1: complete (commits 13412f0..95d037f, tests: cargo test -p rod-app tarea::tests ? 4/4 PASS; workspace 160+256+15+3+3+1+1 PASS; fmt y clippy OK)
Task 2: Ruling: el plan listing de MensajeEditor no cubria Json/Ass/autosubs ni el dialogo de proyecto del editor; se anadieron variantes y Tarea::{AbrirVideoEditor,CargarSrt,CargarAss,GuardarJson,GuardarAss,ImportarSrt,ImportarTxt} porque sin ellas las exportaciones no podian salir del frame. Coste: mas variantes, mismo flujo. - Ruling: el test de GuardarComo usa ruta absoluta con CARGO_MANIFEST_DIR porque la ruta relativa falla con mpv -13 bajo cargo test; coste: ninguno en produccion. - complete (commits 95d037f..1e4d030, tests: cargo test --workspace ? 162+256+15+3+3+3+1+1 PASS; fmt y clippy OK)
Task 3: complete (commits 1e4d030..73f49e1, tests: cargo test -p rod-app ui::tema ? 3/3 PASS; workspace green; fmt y clippy OK)
Task 3: se corrigio assertions_on_constants en el test del tema (const assert) y clippy volvio a estar verde.
Task 4: complete (commits c52c8de..c8a75c5, tests: cargo test -p rod-app arranque_muestra ? 1/1 PASS; workspace green; fmt y clippy OK)
Task 4: fmt aplicado antes del commit final.
Task 5: Ruling: los campos de EditorApp se hicieron pub(crate) y VELOCIDADES pub porque los view de ui/ viven fuera de editor_app.rs; coste: mayor superficie visible dentro del crate, ninguna API publica nueva. - complete (commits 534955d..78bac9c, tests: cargo test -p rod-app teclear_texto_y_confirmar ? 1/1 PASS; workspace green; fmt y clippy OK)
Task 6: Ruling: ScreenDescriptor de egui-wgpu 0.33 no expone texture_format; el formato se toma de preferred_framebuffer_format(&[]) y debe sustituirse por el real de CreationContext en el task 8 si la captura muestra desajuste. Coste: riesgo de formato hasta verificar en pantalla. - Ruling: el test de cambio de resolucion se mide sobre escala_letterbox (sin device), no sobre Gpu: crear un Gpu exige un adapter real y haria el test dependiente de la GPU. - complete (commits 78bac9c..89bd8a0, tests: cargo test -p rod-app ui::video ? 2/2 PASS; workspace green; fmt y clippy OK)
Task 7: Ruling: la timeline de egui usa iced::Point como puente mientras el binario sigue siendo iced; el task 8 debe sustituirlo por un Punto propio al retirar el crate. Coste: un tipo prestado hasta el cierre. - complete (commits 89bd8a0..8686ea4, tests: cargo test -p rod-app ui::timeline ? 2/2 PASS; workspace green; fmt y clippy OK)
