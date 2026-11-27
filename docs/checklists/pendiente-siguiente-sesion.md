# Pendiente entregado por el autor el 2026-11-25

Recibido mientras se cerraba la reorganización del editor. No implementado:
requiere su propio brainstorm/spec/plan (no cabe en el plan ya ejecutado).

## Selección y edición
- El campo de texto debe tener un marco de líneas a su alrededor; sin subtítulo
  seleccionado, grisáceo con borde rayado para expresar que no hay nada que editar.
- Con el playhead corriendo, autoseleccionar el subtítulo debajo; si hay
  varios, el primero de la lista. Cambiar entre ellos con flechas verticales.
- Clic en timeline: sobre un subtítulo lo selecciona; en vacío lleva el tiempo
  del video a donde se clikeó (hoy ambas cosas compiten).

## Fuentes y presets
- Cargar las fuentes reales de los presets (fontdb/font-kit o carpeta del
  proyecto) en `FontDefinitions` con `FontFamily::Name`, una sola vez por lote.
- Cadena de fallback [fuente del preset, Inter, símbolos].
- Un único `render_subtitle(style, texto, escala)` para vista previa, overlay y
  export; contorno por desplazamientos 8–16 direcciones, sombra desplazada.
- El overlay debe usar el preset asignado al hablante de la línea.
- Chip ámbar «Fuente no encontrada»; faux-bold/faux-italic si no hay variante.

## Editor de preset / Speakers / Inicio
- Editor de preset a todo el ancho, secciones de 11 px, ComboBox buscable que
  muestra cada nombre en su fuente, DragValue en px, toggles reales, rename con
  doble clic, campos «Relleno» y «Color del contorno».
- Speakers: tarjeta en 2 líneas, punto de 14 px, nombre editable, keycap, ⋯,
  contador, ComboBox de estilo con valor «No style», mute/solo de 24 px; sin
  desborde a 300 px.
- Inicio: quitar subrayado azul del wordmark, altura igual en tarjetas,
  miniatura a sangre, ⋯ como icono solo en hover, 16 px entre «Most recent» y
  «See all», grilla calculada como (ancho − 2·gap)/3.
- Un solo formato de timecode (redondeo al frame) y un solo idioma en la UI.
