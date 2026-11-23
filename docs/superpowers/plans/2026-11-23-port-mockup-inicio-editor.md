# Port del mockup inicio + editor a la app iced

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (el autor delegó la ejecución en esta sesión). Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Llevar la composición, densidad y sistema visual de `docs/mockups/2026-11-23-inicio-editor.html` a las pantallas reales de iced sin romper el comportamiento del oráculo.

**Architecture:** Los tokens y estilos viven en `tema.rs`; las vistas solo los consumen. Toda la lógica que antes estaba inline en las vistas se extrae a funciones puras pequeñas, testeables sin renderer. El timeline sigue siendo un `canvas`: se le cambian colores, alturas y decoración, no su aritmética tiempo↔píxel.

**Tech Stack:** Rust, iced 0.14 (`canvas`, `shader`, `iced_aw::MenuBar`), `rod-core::format_time` / `parse_time_input`.

**Spec:** `docs/mockups/2026-11-23-inicio-editor.html` (composición y medidas) + `docs/superpowers/specs/2026-11-23-arranque-ux.md` (copy y comportamiento) + `docs/superpowers/specs/2026-11-23-menu-atajos-visual.md` (tokens y reglas).

## Global Constraints

- Sin `unsafe`, sin crates nuevas, sin features nuevas de iced. Sin gradientes ni sombras; radios ≤ 3 px; glifos ASCII/SVG, cero emojis.
- Tokens vigentes: superficie `#0C0C0D`, panel `#141416`, hairline `#FFFFFF26`, texto `#F3F3F3`, secundario `#9A9A9A`, mostaza `#D9A441`.
- El color del hablante (`Hablante::color`, hexadecimal) manda en carril y `.ass`; el mostaza solo tiñe chrome, foco y selección.
- Copy funcional literal de `arranque-ux.md` (`Empieza con un video`, `MP4, MKV, MOV o WebM`, mensajes de error y recovery). El titular display usa `Empezar / con un video` porque es la decisión visual aprobada; el motivo va en `BITACORA.md`.
- Medidas del mockup a 1600×900: menú 46, sidebar inicio 460, sidebar editor 620, transporte 82, pestañas 44, timeline 196, barra de estado 30, rail 60.
- Verificación de cada tarea: `cargo build`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace`, y una captura o salida observable.

## Review Focus

1. **Hablante con color hexadecimal inválido o vacío:** no puede caer en mostaza (el mostaza es chrome). → test `color_de_hex_invalido_no_devuelve_mostaza` (Tarea 1).
2. **Cero recientes:** no aparece `Ver todos` y el texto vacío es el literal de la spec. → test `ver_todos_aparece_solo_con_mas_de_tres_recientes` (ya existe) y `texto_vacio_recientes` (Tarea 2).
3. **Archivo soltado inválido:** mensaje exacto de la spec y la ayuda desaparece en vez de reservar línea. → test `soltar_extension_no_valida_usa_el_copy_de_la_spec` (Tarea 2).
4. **Entrada/salida editadas a mano:** valor inválido o invertido no muta el proyecto. → tests `editar_tiempo_rechaza_valor_invalido` y `editar_tiempo_rechaza_salida_anterior_a_entrada` (Tarea 4).
5. **Cambio de altura de carril:** hit-testing y límites siguen alineados con lo dibujado. → tests existentes de `timeline.rs`/`arrastre.rs` + `altura_de_carril_coincide_con_y_de_carril` (Tarea 6).

---

### Tarea 1: Tokens, fuentes y estilos

**Files:**
- Modify: `crates/app/src/tema.rs`
- Test: `crates/app/src/tema.rs` (`mod tests`)

**Interfaces:**
- Produce: `SUNK`, `LINE2`, `MUTE`, `DIM`, `MOSTAZA_HI`, `MOSTAZA_LINE`, `MOSTAZA_WASH`, `DISPLAY: Font`, `MONO: Font`, `color_de_hex(&str) -> Color`, `sunk(theme) -> container::Style`, `hairline(theme) -> container::Style`, `wash(theme) -> container::Style`.

- [ ] **Step 1: tests que fallan** — `color_de_hex_lee_seis_digitos` (`"#4C8DFF"` → `Color { r ≈ 0.298, g ≈ 0.553, b ≈ 1.0 }`; `"ffffff"` sin almohadilla también funciona); `color_de_hex_invalido_no_devuelve_mostaza` (`"", "rojo", "#123"` → `SECUNDARIO`); `ningun_estilo_tiene_sombra_ni_radio_mayor_a_tres` (recorre `fondo`, `panel`, `aviso`, `boton_primario`, `boton_secundario`, `entrada`, `barra_menu` con statuses representativos y comprueba `shadow == Shadow::default()` y los cuatro radios ≤ 3).
- [ ] **Step 2:** `cargo test -p rod-app tema::tests` → FAIL por funciones/constantes inexistentes.
- [ ] **Step 3:** implementar constantes, `color_de_hex` (parseo manual de 6 hex, sin crates), `DISPLAY`/`MONO` vía `iced::Font { family, weight, .. }`, y los tres estilos de contenedor. Ajustar `SECUNDARIO` a 0.604 (`#9A9A9A`). No tocar el resto de estilos salvo lo que exija el test.
- [ ] **Step 4:** `cargo test --workspace` → PASS (sin regresiones).
- [ ] **Step 5:** commit `feat: tokens y estilos del sistema visual definitivo`.

### Tarea 2: Pantalla de inicio

**Files:**
- Modify: `crates/app/src/inicio.rs`
- Test: `crates/app/src/inicio.rs` (`mod tests`)

**Interfaces:**
- Consumes: `tema::{DISPLAY, MONO, MOSTAZA, MUTE, DIM, SUNK, LINE2, sunk}`.
- Produce: `cantidad_visible(total: usize, expandido: bool) -> usize`, `resumen_reciente(&Reciente) -> String`, `texto_vacio_recientes() -> &'static str`, `MENSAJE_ARCHIVO_INVALIDO: &str`.

- [ ] **Step 1: tests que fallan** — `cantidad_visible` devuelve 3 con 7 recientes cerrado, 7 abierto, 0 con vacío; `resumen_reciente` produce `"52 líneas · 2 hablantes · hace 2 h"` y `"1 línea · 1 hablante · hoy"`; `texto_vacio_recientes` es el literal de la spec; `soltar_extension_no_valida_usa_el_copy_de_la_spec` comprueba el mensaje exacto `No se pudo abrir notas.txt. Usa un video MP4, MKV, MOV o WebM.`.
- [ ] **Step 2:** `cargo test -p rod-app inicio::tests` → FAIL.
- [ ] **Step 3:** extraer las cuatro piezas y usarlas en `update`/`vista_recientes`. Reconstruir `view`: eyebrow `01 / ABRIR`, titular display en dos líneas, dropzone con marcas de registro y cuatro etiquetas mono (cruz de dos líneas de 1 px, sin patrones), botón principal `Abrir video →` con ghost `Abrir proyecto` y `Ctrl+O`, sidebar recovery con `tema::wash` + `Descartar borra la copia de recuperación.`, recents con miniatura 96 px y pie `La miniatura es el frame del playhead al guardar`, bloque `Flujo` de tres pasos y rail inferior de 60 px. La ayuda/el error se muestran en el mismo sitio (nunca dos líneas).
- [ ] **Step 4:** `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace` → PASS; captura de la ventana a 1600×900 en `docs/checklists/slice-11-inicio-definitivo.png`.
- [ ] **Step 5:** commit `feat: arranque con composición del mockup definitivo`.

### Tarea 3: Estructura del editor, transporte y barra de estado

**Files:**
- Modify: `crates/app/src/editor_app.rs` (`view`, `pestana`, `status_bar`), `crates/app/src/main.rs` (`vista`), `crates/app/src/menu.rs` (`barra`)
- Test: `crates/app/src/editor_app.rs` (`mod tests`)

**Interfaces:**
- Consumes: `tema::{DISPLAY, MONO, LINE2, MOSTAZA, MUTE, DIM}`.
- Produce: `resumen_estado(captions: usize, hablantes: usize, capas: usize, ventana: f64, zoom: f64) -> String`.

- [ ] **Step 1: test que falla** — `resumen_estado(42, 3, 6, 24.0, 1.0) == "42 captions · 3 hablantes · 6 capas libass · Ventana 24.0 s · zoom 1.00×"`.
- [ ] **Step 2:** `cargo test -p rod-app editor_app::tests` → FAIL.
- [ ] **Step 3:** implementar el formateador y usarlo en `status_bar` (altura 30, mono 8.5, nombre y ruta separados). En `view`: `left.width(Fill)`, `right.width(620)`, transporte `height(82)` con play mostaza 36×36 y timecode mono 18, pestañas `height(44)` con contadores, timeline `height(196)`. En `main.rs`: envolver la barra de menú en `row` con marca `R`, nombre, estado (`Sin proyecto` / `Guardado`) y versión; `height(46)` y hairline inferior. La barra de menú conserva sus seis raíces y atajos.
- [ ] **Step 4:** build + fmt + clippy + tests + captura `docs/checklists/slice-11-editor-estructura.png` → PASS.
- [ ] **Step 5:** commit `feat: estructura del editor a 980/620 con timeline de 196 px`.

### Tarea 4: Panel de subtítulos denso

**Files:**
- Modify: `crates/app/src/editor_app.rs` (`MensajeEditor`, `EditorApp`, `update`, `subtitle_panel`)
- Test: `crates/app/src/editor_app.rs` (`mod tests`)

**Interfaces:**
- Consumes: `rod_core::{format_time, parse_time_input}`, `Editor::editar_tiempos(&str, f64, f64, f64) -> bool`.
- Produce: `etiqueta_caption(indice: usize, inicio: f64, fin: f64, texto: &str) -> String`, `duracion_texto(inicio: f64, fin: f64) -> String`, mensaje `MensajeEditor::EditarTiempo(String, CampoTiempo, String)` con `CampoTiempo::{Entrada, Salida}`.

- [ ] **Step 1: tests que fallan** — `etiqueta_caption(7, 72.48, 75.96, "No, eso es lo que cambia todo.")` → `"07  00:01:12.48 → 00:01:15.96  No, eso es lo que cambia todo."` (usar el formato real de `format_time` como literal, comprobado a mano); `duracion_texto(72.48, 75.96) == "3.48 s"`; `editar_tiempo_rechaza_valor_invalido` y `editar_tiempo_rechaza_salida_anterior_a_entrada` sobre `EditorApp::desde_video` con un caption sembrado.
- [ ] **Step 2:** `cargo test -p rod-app editor_app::tests` → FAIL.
- [ ] **Step 3:** implementar formateadores y mensaje; `EditorApp` gana dos borradores (`borrador_inicio`, `borrador_fin`) que se sincronizan al seleccionar caption; `on_submit` parsea y llama a `editar_tiempos`, con entrada inválida/invertida se descarta el borrador sin tocar el proyecto. Rehacer `subtitle_panel`: cabecera `Caption NN / TOTAL` con ↶ ↷ ⧉ ×, rejilla Entrada/Salida/Duración, chip de hablante, área de texto con contador, `Aplicar`/`Dividir`, lista compacta con índice, punto del color semántico y tiempo.
- [ ] **Step 4:** fmt + clippy + tests + captura `docs/checklists/slice-11-inspector-subtitulos.png` → PASS.
- [ ] **Step 5:** commit `feat: inspector de subtítulos denso con edición de tiempos`.

### Tarea 5: Paneles Hablantes y Estilos

**Files:**
- Modify: `crates/app/src/editor_app.rs` (`speakers_panel`, `styles_panel`, `campo_texto`)
- Test: sin tests nuevos de lógica (solo presentación); se apoya en `tema::color_de_hex` de la Tarea 1.

- [ ] **Step 1:** sin RED: esta tarea solo reordena widgets existentes; la lógica (alta/baja/asignación/presets) ya está testeada en `editor.rs` y `editor_app.rs`.
- [ ] **Step 2:** reformatear Hablantes: contador `N de 9`, swatch de 8 px con `color_de_hex(&hablante.color)` (Tarea 1), tecla, preset, `Asignar`, `×`, nota «El color es semántico…». Estilos: presets en chips horizontales, campos en rejilla de dos columnas con etiqueta de 64 px, alineación 3×30 px y bloque de vista previa al pie. Conservar todos los `on_press` actuales.
- [ ] **Step 3:** fmt + clippy + tests + captura de las dos pestañas → PASS.
- [ ] **Step 4:** commit `feat: paneles de hablantes y estilos según el mockup`.

### Tarea 6: Timeline: colores semánticos y decoración

**Files:**
- Modify: `crates/app/src/timeline.rs`
- Test: `crates/app/src/timeline.rs` (`mod tests`)

**Interfaces:**
- Consumes: `Hablante.color`, `tema::{color_de_hex, MOSTAZA, DIM}`.
- Produce: `color_clip(color: Color, seleccionado: bool) -> Color` (si seleccionado devuelve el borde mostaza; el relleno sigue siendo el color del hablante).

- [ ] **Step 1: tests que fallan** — `color_clip` con hablante azul y `seleccionado=false` conserva el azul; con `true` devuelve `MOSTAZA`; `altura_de_carril_coincide_con_y_de_carril` comprueba `y_de_carril(1) - y_de_carril(0) == ALTO_CARRIL` y que `carril_en(y_de_carril(2) + 1.0) == Some(2)`.
- [ ] **Step 2:** `cargo test -p rod-app timeline::tests` → FAIL.
- [ ] **Step 3:** implementar `color_clip`; en `carriles()` usar `color_de_hex(&hablante.color)` con fallback a la paleta actual; subir `ALTO_REGLA` a 26 y `ALTO_CARRIL` a 46; `pintar_carriles` dibuja relleno translúcido del color del hablante, borde de 1 px, texto claro, rejilla vertical y selección con outline mostaza + tiras laterales; `pintar_regla` añade chip mostaza con el tiempo del playhead. No cambiar `y_de_carril`, `carril_en` ni las funciones de arrastre.
- [ ] **Step 4:** fmt + clippy + `cargo test --workspace` (los 23 tests de timeline y 14 de arrastre en verde) + captura `docs/checklists/slice-11-timeline.png` → PASS.
- [ ] **Step 5:** commit `feat: timeline con carriles semánticos del mockup`.

### Tarea 7: Cierre, desvíos y evidencia

**Files:**
- Modify: `BITACORA.md`, `docs/checklists/slice-11-mockup-definitivo.md`
- [ ] **Step 1:** escribir en `BITACORA.md` los desvíos decididos: titular display vs copy literal; sin gradientes/trama diagonal (spec los prohíbe; el still del mockup es un SVG de relleno y la app muestra el frame real); sin timecode en recientes (el modelo no lo guarda); recovery usa el texto estructurado que ya genera `rod-core`; carril «—» se mantiene porque captions sin hablante necesitan sitio.
- [ ] **Step 2:** checklist con semáforo: build, fmt, clippy, `cargo test --workspace` (número total), capturas de inicio y editor a 1600×900.
- [ ] **Step 3:** verificación final: `cargo build && cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test --workspace` y revisión visual de las capturas.
- [ ] **Step 4:** commit `docs: cierre del port del mockup definitivo`.
