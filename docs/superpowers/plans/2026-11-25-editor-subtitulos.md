# Reorganización de la edición de subtítulos — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Dejar un único lugar para editar el texto de un subtítulo (barra bajo el video), mover las acciones del inspector a timeline/atajos/menús y cerrar el pulido visual, sin perder accesos ni romper el render ASS.

**Architecture:** `EditorApp` pasa a tener `texto: EstadoTexto`; el componente nuevo `ui/texto.rs` solo dibuja y devuelve mensajes, y `editor_app::update` aplica el historial. La toolbar, el menú contextual y los atajos emiten los mismos comandos. Todos los valores visuales nuevos viven en `ui/tema.rs`.

**Tech Stack:** Rust, egui 0.33, eframe, wgpu, kittest; workspace `rod-core/rod-subs/rod-player/rod-app`.

**Spec:** `docs/superpowers/specs/2026-11-25-editor-subtitulos-design.md`

## Global Constraints

- Ningún texto por debajo de 11 px (`tema::aplicar` ya lo fuerza; no relajar).
- Ningún valor de color, radio, espacio, grosor o tamaño fuera de `crates/app/src/ui/tema.rs`.
- El preview sigue siendo el render del `.ass` exportado: **no tocar `crates/core/src/ass.rs`** ni el color del texto en el overlay.
- Con el foco en el campo de texto no se consume ninguna tecla: `1–9` escriben dígitos.
- Un gesto de edición = un paso de historial; el tipeo se agrupa y se cierra con Enter, pérdida de foco o ~1 s de inactividad.
- Todo comando nuevo debe aparecer en `MapaAtajos::por_defecto()` y en el panel de atajos.
- Anchos a verificar: 1100, 1440 y 2560 px; sin superposiciones.
- Cada fase cierra con `cargo fmt --all -- --check`, `cargo clippy -p rod-app -- -D warnings`, `cargo test --workspace` y una captura observable.
- Idiomas ES/EN: todo texto nuevo se añade a `textos.rs` en ambos.

## Review Focus

- Tipeo con pausa >1 s dentro del campo: el historial debe registrar **un** paso, no uno por tecla, y no debe quedar abierto en reposo. → Task 2, test `cerrar_sesion_por_inactividad_es_un_solo_paso`.
- Shift+Enter hasta 3 líneas en una ventana estrecha: el campo crece, la timeline baja y no se solapa con el panel. → Task 2, test `campo_crece_a_tres_lineas_y_empuja_timeline`.
- Marquesina con Shift+arrastrar en la timeline cuando la lista ya tenía selección: ambas vistas comparten `state.editor.seleccion`. → Task 3, test `marquesina_y_lista_comparten_seleccion`.
- Borrar con multiselección desde menú contextual: pide confirmación y el toast «Deshacer» revierte el grupo entero en un paso. → Task 3, test `borrar_multiples_confirma_y_deshace_grupo`.
- Clip sin hablante seleccionado: el borde sólido de acento gana sobre el relleno gris punteado. → Task 4, test `clip_sin_hablante_seleccionado_usa_acento`.

---

## Task 1: Tokens, comandos y estado

**Files:**
- Modify: `crates/app/src/ui/tema.rs`
- Modify: `crates/app/src/comandos.rs`
- Modify: `crates/app/src/editor_app.rs`
- Modify: `crates/app/src/preferencias.rs`
- Modify: `crates/app/src/main.rs`
- Test: `crates/app/tests/atajos.rs`

**Interfaces:**
- Consumes: `Comando`, `MapaAtajos::por_defecto()`, `EditorApp`, `MedidasEditor`.
- Produces: constantes `ALTO_TRANSPORTE`, `ALTO_BARRA_TEXTO`, `ANCHO_MINIMO_TEXTO`, `ALTO_TOOLBAR_TIMELINE`, `ANCHO_PANEL_DEFECTO/MIN/MAX` ajustados, `ANCHO_BOTON_28`, `TAMANO_ICONO_16`, `GROSOR_SPLITTER`, `ANCHO_DROPDOWN_VELOCIDAD`, `COLOR_FONDO_VIDEO`, `COLOR_BORDE`, `COLOR_HOVER`, `COLOR_SIN_HABLANTE`, `ALTO_PISTA_AUDIO`, `RADIO_CLIP`; variantes `Comando::Entrada`, `Comando::Salida`, `Comando::Unir`, `Comando::AlternarSnap` y `Dividir` con atajo `S`; `struct EstadoTexto` con `objetivo: Option<String>, borrador: String, original: String, foco: bool, ultimo_tipeo: Option<std::time::Instant>, nueva_linea: bool`; campo `EditorApp::texto: EstadoTexto`; campos `snap: bool` y `seguir: bool` en `Preferencias::Datos`.

- [ ] **Step 1: Write the failing test**

En `crates/app/tests/atajos.rs`, test `atajos_de_edicion_registrados`: `por_defecto()` debe producir `Comando::Entrada` con `Key::OpenBracket`, `Comando::Salida` con `Key::CloseBracket`, `Comando::Dividir` con `Key::S`, `Comando::Unir` con `Key::M`, y `Comando::Eliminar` con `Key::Delete`; además ningún par (tecla, ctrl, shift, alt) puede estar duplicado.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p rod-app --test atajos atajos_de_edicion_registrados -- --nocapture`
Expected: FAIL por variantes inexistentes o atajo de `Dividir` igual a `C`.

- [ ] **Step 3: Implement tokens, comandos y estado**

Añadir las constantes a `tema.rs` con los valores exactos del spec (56, 32, 280, 320/280/420, 28, 16, 4, 56, `#0F0F0F`, `#2E2E2E`, `#2C2C2C`, `#666666`). Añadir las variantes de `Comando`, sus filas en `por_defecto()` y sus textos en `Textos` (ES/EN). Mover `borrador`, `editando_texto` e `inline` de `EditorApp` a `EstadoTexto`, dejando `EditorApp::texto`. Añadir `snap`/`seguir` a `Preferencias::Datos` con valores `true`/`false` y la lectura tolerante existente. Cambiar `Tarea::EnfocarTexto` para que en `main.rs` marque `state.texto.foco = true` en vez de ignorarse.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p rod-app --test atajos`
Expected: PASS, sin duplicados.

- [ ] **Step 5: Commit**

```bash
git add crates/app/src/ui/tema.rs crates/app/src/comandos.rs crates/app/src/editor_app.rs crates/app/src/preferencias.rs crates/app/src/main.rs crates/app/src/textos.rs crates/app/tests/atajos.rs
git commit -m "feat(editor): tokens, comandos y estado de edición"
```

---

## Task 2: Barra de texto y foco

**Files:**
- Create: `crates/app/src/ui/texto.rs`
- Modify: `crates/app/src/ui/mod.rs`
- Modify: `crates/app/src/ui/transporte.rs`
- Modify: `crates/app/src/ui/editor.rs`
- Modify: `crates/app/src/ui/paneles/subtitulos.rs`
- Modify: `crates/app/src/editor_app.rs`
- Test: `crates/app/tests/texto.rs` (nuevo), `crates/app/tests/inspector.rs`, `crates/app/tests/subtitulos.rs`

**Interfaces:**
- Consumes: `EstadoTexto`, `Comando`, constantes de Task 1, `EditorApp::editor.seleccion`.
- Produces: `pub fn mostrar(ui: &mut egui::Ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor>` en `ui/texto.rs`; `EstadoTexto::objetivo(state: &EditorApp) -> Option<String>`; `EstadoTexto::cerrar_sesion(&mut self, state: &mut EditorApp)`.

- [ ] **Step 1: Write the failing tests**

En `crates/app/tests/texto.rs`:
1. `objetivo_es_seleccion_o_playhead`: con dos captions y selección en el segundo, `objetivo` devuelve el segundo; sin selección y playhead dentro del primero, devuelve el primero; sin ambos, `None`.
2. `cerrar_sesion_por_inactividad_es_un_solo_paso`: simular 5 caracteres con un salto >1 s; `EditorApp` debe tener exactamente un paso nuevo en el historial.
3. `enter_crea_linea_de_dos_segundos_sin_objetivo`: sin selección ni caption bajo el playhead, confirmar «hola» crea un caption `[playhead, playhead+2.0]` con ese texto.
4. `esc_revierte_y_devuelve_foco_a_timeline`: editar, pulsar Esc, el texto vuelve a `original` y `foco == false`.
5. `campo_crece_a_tres_lineas_y_empuja_timeline`: dos Shift+Enter hacen que `EstadoTexto.lineas == 3` y el rect de la timeline se desplace `ALTO_BARRA_TEXTO * 2` tras `tema::ANIMACION` frames.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p rod-app --test texto`
Expected: FAIL por módulo `ui::texto` inexistente.

- [ ] **Step 3: Implement the field and remove the old editors**

`mostrar` dibuja: chip de hablante (`kit::chip_hablante`, clic abre el selector con «Sin hablante»), `TextEdit` de una línea con borde de acento cuando `foco`, CPS a la derecha en `Estilo::Pequeno` con `AMBAR` si >`CPS_MAXIMO`, y altura 1→3 líneas. Resolver `objetivo` con `EstadoTexto::objetivo`. Filtrar teclas: `Enter` sin Shift cierra sesión y quita foco; `Shift+Enter` deja pasar; las demás teclas no se interceptan. Si `ui.available_width() < ANCHO_MINIMO_TEXTO`, dibujar el campo en una fila propia (la función se llama desde `transporte::mostrar`, que añade el alto extra). Borrar `inspector` y el `TextEdit` inline de `subtitulos.rs`; la lista conserva doble clic → enfocar campo (mensaje `EnfocarTexto`).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p rod-app --test texto --test inspector --test subtitulos`
Expected: PASS. Los tests viejos que asumían inspector/inline se actualizan a la nueva ruta.

- [ ] **Step 5: Commit**

```bash
git add crates/app/src/ui crates/app/src/editor_app.rs crates/app/tests
git commit -m "feat(editor): barra única de edición de texto"
```

---

## Task 3: Timeline, toolbar y acciones

**Files:**
- Modify: `crates/app/src/ui/timeline.rs`
- Modify: `crates/app/src/ui/paneles/subtitulos.rs`
- Modify: `crates/app/src/editor_app.rs`
- Modify: `crates/app/src/main.rs`
- Modify: `crates/app/src/textos.rs`
- Test: `crates/app/tests/timeline.rs`, `crates/app/tests/editor.rs`

**Interfaces:**
- Consumes: `Comando::Entrada/Salida/Dividir/Unir/Eliminar/AlternarSnap`, `EstadoTexto`, `state.editor.seleccion`.
- Produces: `pub fn toolbar(ui: &mut egui::Ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor>`; `pub fn menu_contexto(ui: &mut egui::Ui, id: &str, state: &mut EditorApp, textos: &Textos)`; `MensajeEditor::MarcarEntrada`, `MarcarSalida`, `Unir`, `AlternarSnap`.

- [ ] **Step 1: Write the failing tests**

En `crates/app/tests/timeline.rs`:
1. `marquesina_y_lista_comparten_seleccion`: `state.editor.seleccionar_uno("c1")`, simular Shift+arrastrar sobre `["c2","c3"]`; la selección final debe contener `c1,c2,c3` y el panel de subtítulos debe resaltar los tres.
2. `borrar_multiples_confirma_y_deshace_grupo`: con dos líneas seleccionadas, `Comando::Eliminar` pide confirmación; tras confirmar, un solo `Deshacer` restaura ambas.
3. `atajos_entrada_salida_en_playhead`: `[` y `]` producen `MensajeEditor::MarcarEntrada/MarcarSalida` con el tiempo del playhead.
En `crates/app/tests/subtitulos.rs`: doble clic sobre el timecode de una fila abre edición de entrada; el doble clic sobre el texto enfoca la barra y ya no abre editor inline.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p rod-app --test timeline --test subtitulos`
Expected: FAIL por mensajes y toolbar inexistentes.

- [ ] **Step 3: Implement toolbar, menús y atajos**

`toolbar` pinta 32 px con etiqueta 11 px mayúsculas a la izquierda y, a la derecha, iconos de Eliminar/Dividir/Unir/Snap/Seguir y slider de zoom que emite `MensajeEditor::Zoom` (ya existente). Conectar los nuevos `Comando` en `editor_app::update` a los handlers existentes de dividir/unir y a entrada/salida en el playhead. Añadir `context_menu` en fila y clip con los mismos comandos. Doble clic en el timecode de la fila edita inicio/fin. `snap` y `seguir` se persisten vía `Preferencias`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p rod-app --test timeline --test subtitulos --test atajos`
Expected: PASS; ningún comando queda sin acceso por ratón y atajo.

- [ ] **Step 5: Commit**

```bash
git add crates/app/src crates/app/tests
git commit -m "feat(timeline): toolbar, menús y atajos de edición"
```

---

## Task 4: Pulido visual y responsive

**Files:**
- Modify: `crates/app/src/ui/tema.rs`
- Modify: `crates/app/src/ui/kit.rs`
- Modify: `crates/app/src/ui/editor.rs`
- Modify: `crates/app/src/ui/transporte.rs`
- Modify: `crates/app/src/ui/chrome.rs`
- Modify: `crates/app/src/ui/timeline.rs`
- Modify: `crates/app/examples/capturas.rs`
- Test: `crates/app/tests/editor.rs`, `crates/app/tests/transporte.rs`, `crates/app/tests/kit.rs`

**Interfaces:**
- Consumes: constantes de Task 1 y componentes de Tasks 2–3.
- Produces: capturas `revision-editor-{1100,1440,2560}.png` y `revision-transporte.png` en `docs/checklists/rediseno/`.

- [ ] **Step 1: Write the failing tests**

1. `clip_sin_hablante_seleccionado_usa_acento` en `tests/timeline.rs`:clip sin `hablante_id` seleccionado debe pintar borde de `tema::ACENTO`, no punteado gris.
2. `dropdown_velocidad_tiene_ancho_definido` en `tests/transporte.rs`: el ancho del dropdown es `ANCHO_DROPDOWN_VELOCIDAD` (56), y los tres botones del grupo miden `ANCHO_BOTON_28`.
3. `estado_guardado_lleva_punto` en `tests/chrome.rs`: la barra muestra el punto de estado antes del texto «Saved»/«Unsaved».
4. `sin_texto_menor_de_11px` en `tema.rs`: ampliar el test existente a los nuevos estilos de toolbar, badges y CPS.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p rod-app --test timeline --test transporte --test chrome --lib tema::tests`
Expected: FAIL por colores literales/anchos antiguos.

- [ ] **Step 3: Apply the visual changes**

Sustituir los literales detectados por constantes de `tema.rs`: quitar el marco redondeado del preview (`editor.rs:133-143`), `+`/`⋯` de 28 px con iconos de 16 px y hover `#2C2C2C`, badges de pestañas y divisoria 1 px, tres botones fantasma en el transporte y dropdown de 56 px, waveform normalizada al pico con barras espejadas de 2 px/1 px y pista de 56 px, clip sin hablante con relleno gris 15 % y borde punteado `#666` (el acento gana si está seleccionado), texto vacío bajo Audio, punto de estado en la barra de título, divisores `#2E2E2E` y splitter de 4 px con hover azul. Unificar `ANCHO_PANEL_DEFECTO` a 320 y borrar defaults duplicados.

- [ ] **Step 4: Run tests and inspect captures**

Run: `cargo test --workspace && cargo run -p rod-app --example capturas -- revision`
Expected: todos los tests PASS; las capturas nuevas no muestran superposición a 1100/1440/2560 px. Revisar `revision-editor-1100.png` y `revision-editor-2560.png` con `read`.

- [ ] **Step 5: Commit**

```bash
git add crates/app docs/checklists/rediseno
git commit -m "feat(editor): pulido visual y responsive"
```

---

## Self-review

- Spec §1 (barra de texto): Task 2 cubre campo, foco, historial, chip, CPS, altura y responsive. El toggle de hablante sobre video se eliminó por decisión del autor y no aparece en el plan.
- Spec §2 (dónde va lo demás): Task 3 cubre toolbar, In/Out, menús, atajos y multiselección; Task 2 elimina inspector e inline.
- Spec §3 (pulido): Task 4 cubre la tabla completa; Task 1 fija los tokens.
- Criterios de aceptación: un solo lugar de edición (Task 2), ninguna función sin acceso (Task 3), ≥11 px (Task 4), sin superposiciones 1100–2560 (Task 4 con capturas).
- Tipos: `EstadoTexto`, `Comando::{Entrada,Salida,Unir,AlternarSnap}`, `MensajeEditor::{MarcarEntrada,MarcarSalida,Unir,AlternarSnap}` se definen en Task 1 y se consumen igual en Tasks 2–4.
