# Slice 7d — asignar preset por hablante y panel de estilos

Criterio de salida: se puede elegir el estilo `.ass` de cada hablante y editar
los presets que el oráculo guarda en `presets_ass.json`; cada cambio se aplica
al preview y sobrevive al reinicio.

## Pasos

- [x] **1. Asignación por hablante (7d-a).** `EstadoEditor::asignar_preset`
  cambia `preset_id` y registra **un** paso de historial (elegir en una lista es
  un cambio discreto; el historial de tecleo de nombres es otra cosa). El panel
  de hablantes gana un `pick_list` con «Sin preset» y los presets disponibles;
  al cambiar se recarga el preview. Test: asignar, deshacer, repetir el mismo
  valor (sin paso vacío) e id inexistente (sin cambio).

- [x] **2. Tabs en la columna derecha.** `Panel::{Subtitulos,Hablantes,Estilos}`
  (`Estado.panel`, `Message::Panel`): el oráculo usa `PanelTabs` porque los tres
  paneles no caben apilados. Antes los subtítulos y los hablantes se partían la
  altura; ahora cada pestaña recibe la altura completa.

- [x] **3. CRUD de presets en el panel.** Lista con el preset activo, «＋» que
  duplica el abierto (parte del actual, como el oráculo) y renumera el nombre
  con `nombre_libre`, y «Borrar» deshabilitado cuando solo queda uno. La
  selección se cae al primero si el preset elegido desaparece. No se limpia
  `preset_id` de los hablantes al borrar: el oráculo tampoco lo hace, y
  `ass_del_proyecto` ya cae al `Default` cuando el id no existe.

- [x] **4. Campos y borradores.** `rod_app::presets_edicion` es puro e
  independiente de iced: `CampoPreset` (13 campos), `aplicar_campo` y
  `valor_campo`. Los campos de texto (nombre, fuente, color, color de contorno),
  los numéricos (tamaño, contorno, sombra, márgenes), la alineación en cuadrícula
  7-9/4-6/1-3 y los botones B/I. Los borradores se clavean por
  `(id del preset, campo)` para que cambiar de preset no arrastre un texto a
  medias.

- [x] **5. Guardado inmediato y preview.** `guardar_presets` escribe
  `presets_ass.json` con `tmp + rename` en cada cambio aceptado y recarga el
  preview con la misma lista que se acaba de guardar. Así el invariante de la
  spec §1 sigue valiendo también para un preset recién editado. Un error de
  escritura va a la barra de estado (`Estado.mensaje`).

- [x] **6. Verificar.** `cargo fmt --all -- --check`, `cargo clippy
  --all-targets -- -D warnings` y `cargo test --workspace` en 1.99 y 1.100: 97
  (app lib: 37 timeline + 18 editor + 11 project_io + 9 video + 7 recovery_io +
  6 presets_edicion + 5 presets_io + 4 preview) + 8 (app bin) + 256 (core) + 3
  (player) + 15 (subs) + 3 (oráculo). La app arranca con stderr vacío.

## Evidencia

- `cargo run -p rod-app --example estilos` — salida literal (recortada):

  ```
  semilla: Default / Arial / 48
  editados: Cine / Georgia / 72 · Cine 2 / 2.5
  archivo: C:\Users\migue\AppData\Roaming\RustyOpenDialogue\rod-ejemplo-estilos-10776.json
        "fontname": "Georgia",
        "fontsize": 72.0,
        "nombre": "Cine",
  --- .ass del preview ---
  Style: H1,Georgia,72,&H00A8E9FF,&H00A8E9FF,&H00000000,&H00000000,-1,0,0,0,100,100,0,0,1,2.5,1,8,10,10,40,1
  Style: Default,Arial,48,&H00FFFFFF,&H00FFFFFF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,1,2,10,10,40,1
  píxeles con texto en el .ass: 9435
  ```

  El ejemplo comprueba que lo leído del disco es igual a lo escrito, que el
  archivo nuevo se siembra con un preset, que «nuevo» renumera (`Cine 2`), que
  borrar deja uno, y que los valores editados (Georgia 72, crema, negrita,
  contorno 2.5, alineación 8) llegaron al `Style:` que rasteriza libass.

- `docs/checklists/slice-7d-estilos.png`: el `.ass` del preset editado
  rasterizado (Georgia, crema, negrita, arriba). No es una captura de la UI: es
  el archivo, igual que en 7c.

- `docs/checklists/slice-7d-panel.png`: la app con la pestaña «Estilos»
  abierta. Los clics no se pueden inyectar en esta ventana (winit ignora
  `PostMessage`), así que la captura se tomó con la pestaña inicial sembrada en
  `Panel::Estilos` y se revirtió antes de commitear; el código entregado abre en
  «Subtítulos». Muestra lista, campos, cuadrícula de alineación y los botones
  «＋»/«Borrar».

- Tests nuevos: 6 de `presets_edicion` (decimales, texto inválido, punto final,
  alineación 1-9, booleanos, campos de texto, borradores por preset/campo) y 1
  de `editor` (asignar/deshacer/repetir).

## Hallazgos

- **`"4."` es un número válido para Rust, pero no para el campo.** `parse::<f64>`
  devuelve `4.0`: si se aplicara, el borrador se descartaría y el `text_input`
  saltaría de `4.` a `4` antes de escribir el `5`. `aplicar_campo` rechaza el
  punto final y mantiene el borrador. Lo descubrió un test, no una corazonada.
- **`space::horizontal()` dentro de un `scrollable` empuja los botones fuera
  del viewport.** El primer render mostraba la lista y los campos pero no
  «＋»/«Borrar»: el espacio elástico no tiene límite dentro del scroll. Se quitó
  el spacer. La cuadrícula de alineación también repetía glifos porque el
  índice se reiniciaba en cada fila; ahora se empareja valor y glifo antes de
  partir en filas.
- **Sin selector de fuentes ni de color.** El oráculo enumera las fuentes
  instaladas y usa un `<input type="color">`. iced 0.14 no trae un color
  picker y enumerar fuentes de sistema exigiría otra FFI; aquí fuente y color
  son campos de texto (el color se escribe `#RRGGBB`). Un color inválido cae a
  negro en `hex_to_ass_color`, que ya es el comportamiento del oráculo con un
  valor malo. Queda anotado para un slice posterior si se considera necesario.
- **Escritura por tecla, sin debounce.** El oráculo agrupa las pulsaciones con
  un trailing debounce; aquí cada cambio aceptado escribe un JSON de ~1 KB. Es
  más simple y no se nota en la práctica; si el archivo creciera, el lugar
  donde meter el debounce es `guardar_presets`.
- **Los presets no entran en el historial del proyecto.** Viven en el archivo
  compartido, no en el `.cdp`, igual que en el oráculo. Ctrl+Z deshace cambios
  del proyecto (incluida la asignación de preset a un hablante), no ediciones
  de estilos.

## Qué NO hacer todavía

- Nada de modal de asignación al exportar: el oráculo lo tiene, pero aquí la
  asignación se guarda en el hablante y se usa directamente. Es un refinamiento
  opcional, no necesario para que el export respete los estilos.
- Nada de importar/exportar presets como archivo suelto: el oráculo comparte
  `presets_ass.json` y ese es el contrato.
