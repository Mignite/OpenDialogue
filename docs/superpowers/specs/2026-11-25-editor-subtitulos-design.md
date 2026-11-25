# Reorganización de la edición de subtítulos (Editor egui)

Fecha: 2026-11-25. Estado: propuesto, pendiente de revisión.
Continúa `2026-11-24-editor-ui-redesign-design.md` (fases 1–3 cerradas, fase 4
parcial). No cambia el modelo de datos, el formato `.cdp`, la exportación ni
`ass.rs`.

## 1. Entendimiento compartido

**Para quién.** Quien subtitula diálogo con varios hablantes y revisa cuadro a
cuadro. El trabajo es texto, tiempos y hablante; la app es un editor, no un
formulario con inspector.

**Qué debe lograr.** Que el texto de una línea se edite desde **un solo sitio**:
la barra bajo el video. Que ninguna acción pierda acceso: ratón, menú
contextual y atajo llegan al mismo comando. Que en 1100–2560 px no haya
superposiciones y ningún texto baje de 11 px.

**Decisiones ya acordadas con el autor.**

1. El campo edita en vivo; el tipeo se agrupa en **un paso de historial**, que
   se cierra al perder foco, con Enter o tras ~1 s sin teclear. Nunca un paso
   por tecla.
2. Enter con foco en lista/timeline **enfoca** el campo. Enter dentro del campo
   **confirma y devuelve el foco a la timeline**; solo crea una línea de 2 s si
   no hay ninguna bajo el playhead. Esc revierte lo escrito desde que se
   enfocó y devuelve el foco. Shift+Enter inserta salto de línea.
3. El overlay de subtítulos sigue siendo el render de `ass.rs`: fuente, relleno,
   contorno y sombra del preset. El color de identidad del hablante es **solo
   UI sobre el video**: barra vertical de 3 px junto al bloque y chip con el
   nombre mientras la línea está seleccionada. Toggle «Mostrar tinte de
   hablante» en el menú Ver, activo por defecto. `ass.rs` no se toca.
4. El icono del grupo derecho que hoy dice «Bucle» controla «Seguir al
   playhead»: se renombra a Seguir, con icono y tooltip correctos. No se
   implementa loop de reproducción en esta spec.
5. Un solo diseño, plan de implementación por fases.

**Fuera de alcance.** Loop de reproducción, volumen real (rod-player no expone
API), cambios en ASS/exportación, guía vertical de snap y chip de timecode al
arrastrar (mejoras futuras).

**Precisiones del autor (2026-11-25).** Con el foco en el campo no se consume
ninguna tecla: los atajos, incluidos `1–9`, solo actúan cuando el foco está en
la timeline o en la lista. `E` se conserva como atajo «Editar texto». La
multiselección entra en alcance: se selecciona en la lista y también en la
timeline con Shift+arrastrar (marquesina), compartiendo la misma selección.

## 2. Arquitectura y componentes

```
ui/texto.rs            barra de edición + chip de hablante + CPS (nuevo)
ui/paneles/subtitulos.rs  búsqueda + lista a todo el alto (sin inspector)
ui/timeline.rs         toolbar de 32 px + pistas + clips + menú contextual
ui/transporte.rs       transporte 56 px + campo flexible + grupo derecho
ui/video.rs / editor.rs  preview a sangre + tinte de hablante (UI)
ui/tema.rs             todos los tokens nuevos
comandos.rs            Comando para In/Out, Eliminar, Dividir, Unir, Seguir, Snap
editor_app.rs          EstadoTexto + manejo único de esos comandos + historial
```

`EditorApp` pasa a tener `texto: EstadoTexto` (borrador, línea objetivo, foco,
instante del último tipeo). La vista `ui/texto.rs` solo dibuja y traduce
eventos; la lógica de aplicar/vivir/revertir sigue en `editor_app.rs`, que ya
tiene `EditarEnVivo` y `TerminarEdicionTexto`. Se elimina el editor inline de la
lista (`state.inline`) y se conecta `Tarea::EnfocarTexto`, hoy ignorada en
`main.rs`.

**Un solo camino para cada acción.** La toolbar, el menú contextual de fila y
clip, y los atajos emiten el mismo `Comando`. `editor_app::update` es el único
que muta el proyecto y empuja un paso de historial por gesto.

## 3. Flujo de foco e historial

- **Objetivo:** selección actual; si no hay, línea bajo el playhead; si no hay
  ninguna, el campo queda en modo «nueva línea» y Enter crea un caption de 2 s
  en el playhead con el texto escrito.
- **Sesión de edición:** al enfocar se guarda `texto_original`. Cada cambio
  actualiza el borrador y reinicia un temporizador de ~1 s. El paso de historial
  se cierra con Enter, pérdida de foco o vencimiento del temporizador, usando
  el mecanismo actual de «una sesión = un paso». Mientras la sesión esté abierta
  se pide repaint diferido para que el cierre ocurra aunque no haya video.
- **Foco:** Enter, doble clic en fila o atajo `E` → campo. Enter en campo →
  confirmar y foco a la timeline (Espacio vuelve a reproducir). Esc → revertir
  y foco a timeline. Mientras el campo tiene foco, ninguna tecla se intercepta:
  `1–9` escriben dígitos en el texto; los comandos solo se disparan desde la
  timeline o la lista.
- **Hablante:** chip a la izquierda del campo con punto de color; clic abre el
  selector existente con «Sin hablante». Atajos 1–9 asignan a la selección
  cuando el foco no está en el campo.
- **CPS:** a la derecha, dentro del campo, 11 px secundario; ámbar si >17,
  reutilizando `CPS_MAXIMO` de `tema.rs`.
- **Altura:** una línea de 32 px; Shift+Enter permite hasta 3 líneas y el
  timeline se desplaza con la animación de `ANIMACION` (140 ms).
- **Responsividad:** si el ancho disponible entre timecode y grupo derecho es
  <280 px (`ANCHO_MINIMO_TEXTO` en `tema.rs`), el campo pasa a su propia fila
  bajo el transporte, que crece a 56 + 32 px.

## 4. Timeline, toolbar y acciones

- La cabecera «Timeline» de 28 px pasa a toolbar de 32 px: etiqueta en 11 px
  mayúsculas a la izquierda; a la derecha, iconos de Eliminar, Dividir, Unir,
  Snap (toggle), Seguir (toggle) y slider de zoom (~72 px). El slider usa el
  `MensajeEditor::Zoom` existente.
- **Multiselección:** la lista admite Ctrl+clic y Shift+clic; la timeline
  añade marquesina con Shift+arrastrar sobre clips vacíos. Ambas vistas leen y
  escriben la misma selección de `rod_core::selection`, así que seleccionar en
  un lado resalta en el otro. Las acciones de borrar, dividir y asignar
  hablante operan sobre toda la selección como un único paso de historial;
  borrar con más de una línea pide confirmación y el toast «Deshacer» revierte
  el grupo entero.
- **In/Out:** se mantienen los arrastres de bordes ya probados; se añade
  edición por doble clic sobre el timecode de la fila de la lista y atajos `[`
  y `]` para fijar entrada/salida en el playhead.
- **Eliminar / Dividir / Unir:** accesibles desde toolbar, menú contextual de
  fila y clip, y atajos `Supr`, `S`, `M`. Eliminar conserva el toast con
  «Deshacer» (ya existe). Dividir y unir usan la lógica actual de
  `editor_app.rs`, que hoy solo se alcanzaba desde el inspector.
- Estado nuevo persistido en `preferencias.rs`: `snap` y `seguir` (zoom y
  desplazamiento quedan para una fase posterior).
- La pestaña Subtitles queda con búsqueda y lista a todo el alto; panel por
  defecto 320 px, rango 280–420, resolviendo el desacuerdo actual entre
  `tema.rs` (360/300/520) y `Preferencias::Datos` (320).

## 5. Pulido visual (todo desde `tema.rs`)

| Zona | Cambio |
|---|---|
| Fila en edición | Un solo contenedor con borde de acento; se retira la caja interior actual. |
| `+` y `⋯` junto a búsqueda | Botones 28 px, iconos 16 px, hover `#2C2C2C`. |
| Pestañas | Contadores como badges `#2C2C2C` 11 px; divisoria 1 px bajo la barra. |
| Grupo derecho transporte | Tres botones fantasma de 28 px; dropdown de velocidad ~56 px. |
| Marco del reproductor | Sin borde redondeado; fondo `#0F0F0F` a sangre. |
| Waveform | Normalizar al pico; barras espejadas de 2 px con 1 px de hueco, gris-azulado; pista 56 px. |
| Clip sin hablante | Relleno gris 15 % y borde punteado `#666`; seleccionado, borde sólido de acento (el acento gana). |
| Zona vacía bajo Audio | «Asigna un hablante para ver su pista» en secundario, u ocultar. |
| Overlay | Texto idéntico al `.ass`; barra de identidad 3 px + chip de nombre sobre la línea seleccionada; toggle en Ver. |
| Barra de título | «Saved»/«Unsaved» con punto de estado (secundario/ámbar) delante del texto. |
| Divisores | 1 px `#2E2E2E`; splitter 4 px entre video y panel con hover azul. |

Ningún valor de color, radio, espacio o tamaño se escribe en los componentes:
nuevas constantes en `ui/tema.rs`. Ningún texto por debajo de 11 px (los tests
de `tema.rs` ya lo verifican y se amplían a los nuevos estilos).

## 6. Fases del plan (una por slice verificable)

1. **Tokens y estado:** constantes en `tema.rs`, `comandos.rs`, `EstadoTexto`,
   preferencias. Tests de comandos y preferencias.
2. **Barra de texto:** componente, foco, historial agrupado, chip, CPS, altura
   1–3 líneas y responsive <280 px. Eliminar inspector e inline de la lista.
   Tests de foco/historial y capturas del transporte.
3. **Timeline y acciones:** toolbar, menús contextuales, atajos `[ ] S M`,
   doble clic en timecode, persistencia de snap/seguir. Tests de comandos y
   de timeline; capturas 1100/1440/2560.
4. **Pulido visual:** todos los puntos de la tabla anterior. Capturas antes/
   después y revisión de superposiciones en los tres anchos.

Cada fase cierra con `cargo fmt --check`, `cargo clippy -D warnings`,
`cargo test --workspace` y una captura observable. La spec anterior exige
también verificar el render real con video para el overlay.

## 7. Riesgos y migas

- El temporizador de 1 s necesita repaints diferidos; si se olvida, el paso de
  historial nunca se cierra en reposo.
- Quitar el inspector deja temporalmente sin acceso a dividir/unir hasta que
  la fase 3 conecte toolbar y atajos: conviene hacerlo en el mismo slice o
  añadir los atajos antes de retirarlo.
- El ancho del panel tiene dos defaults distintos; la fase 1 debe dejar uno.
- La pista «Sin hablante» y el clip sin hablante son conceptos distintos: el
  primero es un carril, el segundo es un caption sin `hablante_id`.
- Los atajos `1–9` solo actúan con el foco en timeline/lista; dentro del campo
  escriben dígitos. Cualquier filtro de atajos que se añada debe comprobar el
  foco antes de despachar el comando.
- La marquesina de la timeline y los clics con modificador de la lista deben
  actualizar la misma selección; si se implementan por separado, las dos vistas
  se desincronizan.

## 8. Pendiente de revisión

Única pregunta abierta: el chip con el nombre del hablante que se dibuja sobre
el video, ¿debe aparecer **solo cuando hay una línea seleccionada**, o también
cuando no hay selección pero hay una línea bajo el playhead? (Hoy la spec
supone lo primero; la barra vertical de color sí aparece para la línea activa
en ambos casos, si el toggle «Mostrar tinte de hablante» está activo.)

Resueltas: `1–9` no consumen teclas con el foco en el campo; `E` enfoca el
campo; borrar con confirmación y multiselección en lista y timeline entran en
alcance.
