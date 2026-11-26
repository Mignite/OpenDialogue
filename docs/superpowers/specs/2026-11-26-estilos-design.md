# Rediseño de la pestaña Styles (panel derecho)

Fecha: 2026-11-26. Estado: propuesto, pendiente de revisión.

## 1. Intención

Que quien ajusta estilos vea en la tarjeta **exactamente** lo que verá sobre el
video: misma fuente, relleno, contorno y sombra. Hoy la tarjeta dibuja
`"Aa Hola mundo"` con Inter mientras el overlay rasteriza el `.ass` con la
fuente del preset (por ejemplo KiwiSoda), y el editor son diez campos de texto
a media columna con `true`/`false`.

Éxito: preview y overlay idénticos; ningún control a media columna; ningún
`true/false`; ningún texto <11 px; sin overflow horizontal a 300 px de panel.

## 2. Fuera de alcance

- No cambiar el modelo de datos (`PresetAss`, `Hablante`) ni `ass.rs`.
- No cambiar el formato `presets_ass.json` (se comparte con la app Tauri).
- No rediseñar Speakers, Inicio ni el editor de subtítulos.
- No añadir cuadrícula 3×3 de alineación: el modelo no tiene esa propiedad.

## 3. Restricciones

- Todo color, radio, tamaño y espacio sale de `ui/tema.rs`; ningún literal en
  el componente (hoy hay `0x16,0x16,0x16` en `estilos.rs:107`).
- Todo texto nuevo se añade a `textos.rs` en ES y EN.
- La invariante del proyecto se mantiene: el preview es el render del mismo
  `.ass` que se exporta.
- La nueva dependencia `fontdb` se justifica: necesitamos resolver por nombre
  de familia los archivos de fuente instalados en el sistema (Inter y
  JetBrains Mono embebidos no cubren KiwiSoda, Arial, etc.) sin `unsafe` y de
  forma portable. Alternativa descartada: vendorear fuentes en `assets/fonts`
  (no estarían las que el usuario ya tiene instaladas).

## 4. Enfoques (decididos con el autor)

1. **Render compartido sobre libass** (elegido): `render_subtitle(style,
   texto, escala)` envuelve `build_ass` + `Subtitulos::render`. La tarjeta
   construye un `.ass` de una línea con el preset y rasteriza; el video sigue
   usando el mismo pipeline con los captions reales. Idénticos por
   construcción; cuesta un render extra por preset visible y unabitmap por
   tarjeta.
2. **Función de dibujo de egui para ambos**: más simple, pero el video deja de
   ser el render del `.ass` exportado y rompe la invariante. Descartado.
3. **Preview con libass separado del overlay**: funciona pero duplica la
   conversión de preset a estilo; descartado por riesgo de divergencia.

Fuentes: **fuentes del sistema con `fontdb`** (elegido); descartado usar solo
`assets/fonts`.

## 5. Diseño

### 5.1 Render compartido

- `rod-app/src/ui/estilos_render.rs` (nuevo):
  `pub fn render_subtitle(preset: &PresetAss, texto: &str, escala: f32) -> RgbaImage`
  usa `rod_core::build_ass` con un caption sintético y `rod_subs::Subtitulos`
  en memoria; devuelve el bitmap ya compuesto con relleno, contorno y sombra
  del preset. Sin tocar `ass.rs`: se reutiliza el mismo `build_ass` del export.
- Escala: `preset.fontsize * (alto_preview / CANVAS.1)`, con mínimo 20 px
  legibles. `alto_preview = tema::ALTO_PREVIEW_STYLES = 64`.
- El overlay del video pasa a llamar al mismo envoltorio cuando dibuje la
  línea activa; `refrescar_preview` sigue siendo el dueño del `.ass` real.
- Tests: un preset con fuente del sistema y otro con fuente ausente producen
  bitmaps no vacíos; el bitmap del preview y el de una línea equivalente del
  overlay son byte-idénticos.

### 5.2 Tarjeta y acordeón

- Banda de preview de 64 px, fondo `COLOR_PREVIEW_STYLES = #141414`, radio 6
  solo en las esquinas superiores; debajo, cabecera de 36 px.
- Cabecera: nombre en `Estilo::CuerpoFuerte` (13 medium), puntos de los
  hablantes que usan el preset con tooltip de sus nombres (12 px, token
  `TAMANO_PUNTO_GRANDE` o nuevo `TAMANO_PUNTO_ESTILO = 14`), chip ámbar
  `textos.fuente_no_encontrada` si la fuente no se resuelve, y chevron.
- Chevron: mismo glifo, rotación de 90° con `animate_bool_with_time` y
  `tema::ANIMACION` (140 ms), no cambio de icono. Al abrir, `ScrollArea`
  desplaza para que la tarjeta quede visible. Solo un preset abierto.
- Persistencia: `Preferencias::preset_abierto: Option<String>` con
  `#[serde(default)]` en `preferencias.json`; nunca en `presets_ass.json`.

### 5.3 Editor expandido

- Padding 12 px; secciones `FONT`, `FILL`, `OUTLINE`, `SHADOW`, `BOTTOM
  MARGIN`, separadas por 1 px `tema::BORDE` con 12 px de margen. Etiquetas de
  11 px mayúsculas sobre cada control.
- Todos los controles a todo el ancho con
  `ui.add_sized([ui.available_width(), tema::ALTO_CONTROL], ...)`.
- FONT: `ComboBox` buscable a todo el ancho, opciones pintadas con su propia
  fuente (carga perezosa de las visibles); fila con `DragValue` de tamaño 96 px
  con sufijo `" px"`, rango 8–200, y toggles B/I de 28×28 con acento cuando
  activos. Sin variante → toggle deshabilitado con tooltip.
- FILL: swatch 28×28 con picker en popup + campo hex flexible; hex inválido
  con borde rojo, aplica con Enter o al salir.
- OUTLINE: `DragValue` 80 px `" px"` + swatch + hex. Es el segundo «Color»
  actual, renombrado.
- SHADOW: `DragValue " px"`.
- BOTTOM MARGIN: `DragValue " px"` sobre `margin_v`; con `alignment = 2`
  (abajo-centro) corresponde al margen inferior. Sin cuadrícula.
- Sin campo Name: renombrar con doble clic sobre el título de la tarjeta.
- Sin `true`/`false`: los toggles los reemplazan.
- Pie: botón secundario «Apply to N selected lines» a todo el ancho y 32 px,
  deshabilitado sin selección; debajo Duplicate y Delete como botones
  fantasma. Delete se deshace con toast, sin diálogo.
- «+ New preset» en la cabecera de la lista, todo el ancho, 32 px.

### 5.4 Comportamiento e historial

- Cada cambio actualiza en vivo preview y overlay (llamada a
  `refrescar_preview` ya existente).
- `EditarPreset` se escribe en JSON con debounce de ~1 s por control, para no
  escribir cada tecla. Los presets no entran en `Historial` (restricción
  documentada en `slice-7d-estilos.md`); lo que sí es un solo paso de historial
  es `AplicarPresetSeleccion`, que hoy genera uno por hablante: se agrupa con
  `registrar_instantanea`/`registrar_historial` como `asignar_a_seleccion`.

## 6. Pruebas

- `tests/estilos_render.rs`: preview y overlay byte-idénticos, fuente ausente,
  escala mínima 20 px.
- `tests/estilos.rs`: acordeón, persistencia del preset abierto, toggles,
  hex inválido, Apply deshabilitado sin selección, sin overflow a 300 px.
- `capturas.rs` añade fase con `Panel::Estilos` a 1100/1440/2560 px.
- Verificación estándar: `cargo fmt --check`, `cargo clippy -D warnings`,
  `cargo test --workspace`.

## 7. Lo que queda al implementador

Nombres concretos de funciones internas y reparto de archivos; el orden de las
fases del plan; si el picker de color se reutiliza de otro panel o se implementa
uno mínimo en el componente.
