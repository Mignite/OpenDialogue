# Rediseño visual de la UI (Inicio + Editor)

Fecha: 2026-11-24. Estado: propuesto, pendiente de revisión.
Complementa `2026-11-24-egui-migracion-design.md` (que cubre la migración técnica
iced→egui). Esta spec cubre **solo** lo visual y lo interactivo: no cambia el
modelo de datos, ni el formato `.cdp`, ni la exportación.

## 1. Entendimiento compartido

**Para quién.** Para quien subtitula diálogo con varios hablantes y revisa el
resultado cuadro a cuadro. No es una herramienta de karaoke ni un editor de
video: el trabajo es texto, tiempos y a quién pertenece cada línea.

**Qué debe lograr.** Que la app se vea y se sienta como una herramienta de
estudio (Premiere, Resolve) en vez de un formulario: densa, oscura, con un solo
acento, y donde el hablante se reconoce de un vistazo en las tres vistas (lista,
timeline, preview) porque el color es el mismo y viene del mismo sitio.

**Referencia obligatoria.** `OpenDialogue UI.html` (mockup del usuario, pantalla
de Inicio, 1440×900) fija la paleta, la tipografía, las densidades y el chrome.
Donde el mockup y el brief original se contradicen, **gana el mockup**; las tres
contradicciones y su motivo están en §2.3. Lo que el mockup no cubre (inspector,
timeline, transporte, teclado) se diseña aquí heredando sus principios.

**Éxito medible.**
1. Cero glifos tofu y cero texto por debajo de 11 px.
2. Todo par texto/fondo ≥ 4.5:1 (WCAG AA), medido en un test, no a ojo.
3. El color de un hablante es el mismo en lista, timeline, chips y preview, y
   sale de un único campo del proyecto.
4. Ningún valor de color, radio, espacio o tamaño fuera de `theme.rs`.
5. 543 líneas y 7 hablantes se mueven fluido y sin repintar en reposo.
6. Funciona de 1100 a 2560 px de ancho.
7. Cada fase cierra con capturas antes/después generadas sin usar la pantalla.

**Fuera de alcance.** Modelo de subtítulos, `.ass`, preview libass, exportación,
`.cdp`, recovery, autosubs, presets. Se tocan solo para leer de ellos.

## 2. Sistema visual

### 2.1 Tokens (`crates/app/src/ui/tema.rs`, reescrito)

Única fuente. Ni `Color32::from_rgb`, ni `.size(...)`, ni `CornerRadius` literales
en el resto de `ui/`. Los valores del mockup, literales:

| Token | Valor | Uso |
|---|---|---|
| `FONDO` | `#1A1A1A` | ventana, monitor, fondo de timeline |
| `RIEL` | `#1F1F1F` | riel izquierdo de Inicio |
| `PANEL` | `#232323` | paneles, tarjetas, cabecera de pistas |
| `CONTROL` | `#2C2C2C` | inputs, botón secundario, hover de menú |
| `ELEVADO` | `#333333` | hover de icono y de secundario |
| `ACTIVO` | `#262626` | ítem activo del riel |
| `BORDE` | `#2E2E2E` | divisoria de un píxel, borde de tarjeta |
| `BORDE_CONTROL` | `#3A3A3A` | borde de input y de botón secundario |
| `BORDE_HOVER` | `#4A4A4A` | hover de tarjeta y de fantasma |
| `TEXTO` | `#E6E6E6` | títulos y contenido |
| `TEXTO_MENU` | `#CFCFCF` | ítems de menú y controles de ventana |
| `TEXTO_2` | `#A0A0A0` | secundario |
| `TEXTO_3` | `#929292` | terciario y mono |
| `ACENTO` | `#3B8EFF` | **solo** foco, selección, playhead y marca activa |
| `PRIMARIO` | `#2766DB` | fondo del botón primario |
| `PRIMARIO_HOVER` | `#2F72E8` | hover del primario |
| `PELIGRO` | `#C42B1C` | hover de cerrar, solape de un mismo hablante |

Radios: `RADIO_CONTROL = 4`, `RADIO_SUPERFICIE = 6`. Nada intermedio.

Espacios: 2, 4, 8, 12, 16, 20, 24, 32, 40, 72 (múltiplos de 4 salvo el 2 de
separación entre ítems de menú, que el mockup usa).

Alturas: `ALTO_CHIP = 24`, `ALTO_CONTROL = 28`, `ALTO_NAV = 32`,
`ALTO_BOTON = 36`, `ALTO_CABECERA = 40`, `ANCHO_BOTON_VENTANA = 46`,
`ALTO_FILA_SUB = 44`.

Iconos: 16×16, trazo 1.5, juntas redondas, color heredado del texto.

### 2.2 Tipografía (`ui/tipografia.rs`, nuevo)

Inter (UI) y JetBrains Mono (solo cifras y atajos), embebidas con
`include_bytes!` y licencia OFL junto a los archivos en
`crates/app/assets/fonts/`. Se subconjuntos con `fonttools` a
Latin-1 + Latin-Ext-A + puntuación (Python 3.14 está disponible), lo que deja
~40 KB por peso; si el subconjunto falla, se usan los TTF completos y se anota
en `BITACORA.md`.

`FontDefinitions` con familias nombradas, registradas **antes** de cualquier
pintado (la spec anterior ya documentó el pánico de `FontFamily::Name` sin
cargar):

- `FontFamily::Proportional` → Inter Regular, con `NotoEmoji` de egui como
  reserva para no dejar tofu en nombres de hablante con emoji.
- `FontFamily::Name("inter-medium")` → Inter Medium.
- `FontFamily::Name("inter-semibold")` → Inter SemiBold.
- `FontFamily::Monospace` → JetBrains Mono, con reserva a Inter.

`TextStyle`s:

| Estilo | Fuente | Uso |
|---|---|---|
| `Titulo` | 24/32 · Medium · −0.01 em | H1 de Inicio |
| `Subtitulo` | 16 · Medium | cabecera de inspector |
| `Cuerpo` | 13 · Regular | texto de subtítulo |
| `CuerpoFuerte` | 13 · Medium | nombre de proyecto, hablante |
| `Base` | 12 · Regular | cuerpo de UI |
| `Pequeno` | 11 · Regular | meta, fechas |
| `Etiqueta` | 11 · Medium · mayúsculas · 0.04 em | cabeceras de sección |
| `Mono` | 11 · Mono | atajos, versión, chips de duración |
| `Timecode` | 13 · Mono | tiempos de fila, campos |
| `TimecodeGrande` | 20 · Mono | timecode del transporte |

Nada por debajo de 11. La mono va **solo** en cifras, atajos, códigos de tecla y
contadores técnicos; nunca en prosa. `egui::RichText` con familia explícita
donde importe el ancho fijo.

### 2.3 Donde el mockup corrige el brief

| Brief | Mockup | Se adopta | Motivo |
|---|---|---|---|
| primario = acento `#3B8EFF` | `#2766DB` | mockup | el acento queda libre para foco/selección/playhead; si el botón usara el mismo azul, competiría con el cursor |
| terciario `#666666` | `#929292` | mockup | `#666` sobre `#1A1A1A` da 2.5:1 e incumple el propio AA del brief |
| paleta de 10–12 colores | 8 colores | 12 (8 + 4) | 7 hablantes en `RobloxTerror` y margen para proyectos mayores |
| cabecera de ventana 32 px | 40 px | mockup | 40 deja respirar 24 px de menú con 8 de margen |

### 2.4 Paleta de identidad de hablante

Doce colores pastel sobre oscuro, ordenados por tono, con el 8 del mockup
marcado con `*`:

`#FF8A6B`*, `#F2B84B`*, `#C9D45C`*, `#A8E06B`, `#7BD17F`*, `#3FC4B8`*,
`#7FE0E8`, `#6BB6F0`*, `#8FA9FF`, `#B39BFF`*, `#E0A8F0`, `#F27BB5`*

Reglas:
- La paleta vive en `rod_core::color_hablante` como `paleta_identidad() -> [&str; 12]`.
- `color_identidad(hablante, indice) -> Color32`: si `hablante.color` es un hex
  válido **de la paleta**, ese; si no, `paleta[indice % 12]` con `indice` = su
  posición en `proyecto.hablantes`. Determinista y sin campo nuevo en el `.cdp`
  (`Hablante.color` ya existe y ya se serializa). El color por defecto depende
  de la posición, así que reordenar hablantes a mano puede cambiar los colores
  por defecto; en la práctica el orden solo cambia al importar autosubs, que
  reemplaza el proyecto entero. Un color ya elegido por el usuario no se mueve.
- Al crear un hablante se elige el primer color de la paleta no usado.
- El selector del panel Hablantes son las 12 muestras: no hay rueda de color
  libre, así que nunca se puede guardar un color inaccesible.

**Invariante que no se toca.** El color del *texto* del subtítulo sigue siendo
`rod_core::color_hablante(&hablante, &presets)` (color del preset, blanco si no
tiene). `Hablante.color` pasa a ser identidad de UI y **no** se usa para pintar
subtítulos ni para exportar. El preview y el `.ass` exportado no cambian.

En el panel Hablantes, junto al selector de identidad, va un chip de solo lectura
con el color efectivo del preset y un enlace que cambia a la pestaña Estilos con
ese preset seleccionado: el usuario ve los dos colores y sabe cuál manda dónde.

## 3. Componentes (`ui/kit.rs`, nuevo)

Un componente por concepto, sin `egui::Button` suelto en las vistas:

- `icon_button(ui, Icono, activo: bool, tooltip: &str, atajo: Option<&str>) -> Response`
  — 24×24 en filas y menús, 28×28 en cabeceras y transporte; fondo
  transparente, hover `ELEVADO`, activo `CONTROL` con texto `TEXTO`; el tooltip
  muestra `texto` y el atajo en mono separado.
- `boton_primario(ui, texto, Icono) -> Response` y `boton_secundario(...)` —
  36 de alto, radio 6, padding 16, separación 8.
- `panel_header(ui, titulo, acciones: &[AccionCabecera])` — 28 de alto, título en
  `Etiqueta`, acciones a la derecha; el borde inferior lo pone el panel.
- `tab_bar(ui, tabs: &[(Panel, &str, Option<usize>)], activo) -> Option<Panel>` —
  32 de alto, subrayado de 2 px `ACENTO` en el activo, contador opcional en
  `Pequeno` con `CONTROL` de fondo.
- `chip_hablante(ui, nombre, color) -> Response` — punto de 8 px, texto 11,
  fondo `CONTROL`, radio 4.
- `punto_hablante(ui, color, tamano)` — el punto suelto, para listas y pistas.
- `campo_tiempo(ui, texto, invalido: bool) -> Response` — ancho fijo al de la
  mono (`14 ch`), alineado a la izquierda, borde `BORDE_CONTROL`, inválido
  `PELIGRO`.
- `campo_texto(ui, texto, alto_min)` — crece con el contenido hasta un máximo.
- `recuadro_punteado(ui, alto)` — el estado vacío de Inicio, reutilizado en
  timeline vacío e inspector sin selección.
- `tarjeta(ui, ...)` — superficie `PANEL`, borde `BORDE`, radio 6, hover
  `BORDE_HOVER` + elevación de 1 px en 140 ms.
- `separador_vertical(ui)` — 1×16 en `BORDE`.

Tooltips: retardo global 0.4 s vía `style.interaction.tooltip_delay` (una sola
vez en `theme.rs`), no por widget.

Animaciones: 120–160 ms (`style.animation_time = 0.14`), solo color de fondo,
borde y opacidad. Nada que mueva texto.

Foco: anillo de 2 px `ACENTO` con 1 px de separación, dibujado por el kit, no por
egui; `style.visuals.selection` también `ACENTO`.

## 4. Chrome de ventana (`ui/chrome.rs`, nuevo; sale de `main.rs`)

`decorations(false)` se queda. La cabecera pasa de 40 px con tres zonas:

- **Izquierda:** monograma de 24 px (cuadrado redondeado 6, `PANEL`, borde
  `BORDE_CONTROL`) con las tres barras del mockup en tres colores de paleta;
  «OpenDialogue» en 13 SemiBold; separador; menús Archivo, Edición, Ver,
  Exportar, Estilos, Ayuda (24 de alto, radio 4, hover `CONTROL`).
- **Centro:** nombre del proyecto y estado de guardado en una sola línea
  («`RobloxTerror.cdp` · Sin guardar»), centrado de verdad (rect calculado, no
  `ui.horizontal`), en `Pequeno`. En Inicio: vacío.
- **Derecha:** minimizar, maximizar, cerrar de 46×40 con iconos de 16; hover
  `CONTROL`, salvo cerrar que usa `PELIGRO` con icono blanco.

Arrastre: franja `Sense::click_and_drag()` en el hueco entre menús y centro;
`StartDrag` al arrastrar, doble clic maximiza. Los botones se reservan antes que
la franja (el bug actual de desborde ya está resuelto así y se conserva).

Menús: mismo `Comando` de `comandos.rs`. Cada ítem es una fila con etiqueta a la
izquierda y atajo en mono a la derecha, alineados a una columna común; los
separadores son `BORDE`. El atajo **no** va dentro del texto de la etiqueta
(hoy sí: `"Guardar   Ctrl+S"`).

El nombre visible pasa a «OpenDialogue» en todo lo que el usuario lee: título de
ventana, marca de la cabecera, `Cargo.toml` del binario si aplica. La ruta de
datos `%APPDATA%\RustyOpenDialogue` **no** cambia: moverla huérfana recientes,
presets y recovery.

## 5. Inicio (`ui/inicio.rs`, reescrito)

- Riel de 220 px (`RIEL`, borde derecho `BORDE`): Inicio, Recientes y Ajustes
  con icono de 16 + etiqueta 12, 32 de alto, activo `ACTIVO` + barra de 2 px
  `ACENTO` separada 6 px arriba y abajo; versión al pie en mono 11 `TEXTO_3`.
  Ajustes no es una pantalla del enrutador: abre la capa de §11.3 y se marca
  como activo mientras esté abierta.
- Contenido: columna de 960 centrada, 72 px de aire arriba.
- Título 24/32 y subtítulo 13 `TEXTO_2`; botones primario «Abrir video» y
  secundario «Abrir proyecto» con sus iconos; el atajo `Ctrl + O` en mono 11 a
  la derecha, no pegado al botón.
- Zona de arrastre de 104 px, borde discontinuo de 1.5 px, fondo `#1E1E1E`;
  reacciona al arrastre de archivos (`hovering_files` + extensión válida):
  borde y texto pasan a `ACENTO` y el icono sube 2 px. Es la misma zona que
  acepta el drop de verdad, no un texto.
- «Recientes» con etiqueta de sección y selector de orden («Más recientes»,
  «Nombre», «Líneas»).
- Rejilla de tarjetas de mínimo 260 px (`repeat(auto-fill, minmax(260px, 1fr))`
  hecho a mano con el ancho disponible), separación 20.
  - Miniatura 16:9: la caché de `thumbnail.rs` si existe; si no, degradado
    derivado del color del primer hablante. Encima: chip de duración en mono 11
    sobre negro al 62 %, y al pie la tira de distribución de hablantes de 4 px
    (proporcional a la duración de sus líneas)..
  - Cuerpo: nombre 13 Medium con elipsis y tooltip con la ruta completa; meta
    `543 líneas · 7 hablantes`; puntos de hablante de 8 px; fecha relativa 11
    `TEXTO_3`.
  - Botón «···» que abre menú contextual: Abrir, Mostrar en carpeta, Quitar de
    recientes.
  - Hover: borde `BORDE_HOVER`, `translateY(-1px)` y ligera sombra, 140 ms.
  - Tarjeta fantasma punteada «Abrir otro proyecto» al final.
- Vacío: recuadro punteado con icono y la frase de `texto_vacio_recientes()`.
- Error: banda con `PELIGRO` al 12 % y el mensaje de `EstadoInicio::mensaje`,
  descartable.
- `EstadoInicio` gana `orden: OrdenRecientes` y `destacado: Option<usize>` (para
  teclado). `Reciente` gana dos campos derivados del `.cdp` que ya se lee, sin
  I/O extra: `duracion: f64` (fin del último caption) y
  `tira: Vec<(String, f32)>` (color de identidad y segundos acumulados de cada
  hablante que tiene líneas, en el orden del proyecto, usada por el degradado y
  por la tira de 4 px).

## 6. Editor

### 6.1 Distribución (`ui/editor.rs`, reescrito)

```
cabecera 40
┌───────────────────────────────┬──────────────┐
│ monitor (16:9, letterbox)     │ panel derecho│
│                               │ 320 (280–420)│
├───────────────────────────────┤              │
│ transporte 56                 │              │
├───────────────────────────────┴──────────────┤
│ timeline 260 (160–60 %)                      │
├──────────────────────────────────────────────┤
│ estado 24                                    │
└──────────────────────────────────────────────┘
```

`SidePanel` derecho con `resizable`, `TopBottomPanel` inferior con `resizable`,
las dos colapsables (doble clic en el divisor, o el icono de la cabecera). Las
medidas se persisten en `%APPDATA%\RustyOpenDialogue\preferencias.json`
(`ui/preferencias.rs`, escritura atómica como `recientes.rs`, lectura tolerante:
un archivo roto da valores por defecto). Se guarda: ancho del panel, alto del
timeline, panel visible, `seguir`, zoom y desplazamiento del timeline, idioma y
atajos (§11).

### 6.2 Panel derecho

`tab_bar` con Subtítulos (contador de líneas), Hablantes (contador) y Estilos.
`panel_header` de 28 con título y acciones por pestaña. El contenido va en un
`ScrollArea` propio que solo repinta si se mueve.

**Subtítulos.** Lista virtualizada (`show_rows` con alto fijo 44) con:
- Barra de 3 px del color de identidad a la izquierda (más ancho si está
  seleccionada).
- Dos líneas: texto 13 con elipsis y tooltip completo; debajo, timecode de
  entrada en mono 13 + duración en `Pequeno` + `chip_hablante`.
- Selección: fondo `ACENTO` al 18 % y borde izquierdo a 3 px.
- Solape del mismo hablante: texto del chip en `PELIGRO`.
- Doble clic: editor inline en la fila (Enter confirma, Esc cancela, pierde foco
  confirma). Un clic selecciona y busca el playhead al inicio del caption.
- `Ctrl+clic` multiselección, `Shift+clic` rango, flechas mueven la selección y
  desplazan la lista para mantenerla visible.

**Inspector bajo la lista** (sustituye el bloque TEXTO/ENTRADA/SALIDA/Aplicar/
Borrar, que se elimina): solo con una selección, si no `recuadro_punteado`.
- Textarea de 3 líneas mínimas que crece hasta 8, con el borrador existente
  (`Borrador` + `AplicarTexto`); `Ctrl+Enter` aplica, `Esc` revierte.
- Fila de hablante: `chip_hablante` + selector desplegable con los 12 puntos y
  «Sin hablante»; `AsignarHablante`.
- Entrada y salida con `campo_tiempo`, botones `−1`/`+1` cuadro a cada lado
  (usa el frame del video si está disponible; si no, 1/25 s acotado por
  `duration`), y el texto resultante de `duracion_texto`.
- `icon_button` de papelera que borra la selección y ofrece toast con
  «Deshacer» durante 5 s (`MensajeEditor::Deshacer`).
- Todo aplica en vivo con el historial ya existente: un gesto, un paso de
  deshacer.

**Hablantes.** Tarjeta por hablante: nombre editable en línea, tecla en mono,
12 muestras de identidad, `chip_hablante` de muestra, chip de color del preset
con enlace a Estilos, contador de líneas y papelera con confirmación.
«Agregar hablante» como botón secundario.

**Estilos.** Se conserva la funcionalidad y se re-estiliza con el kit: lista de
presets a la izquierda, campos a la derecha, sin cambios de comportamiento.

### 6.3 Monitor y transporte

Monitor: el frame de mpv con la relación de aspecto real, centrado, sobre
`FONDO`, sin bordes ni sombra; si no hay frame, recuadro punteado con «Sin
video». Opción «superponer subtítulos» ya existente. El área se mide y se
publica para que el preview no se deforme (ya lo hace `ui/video.rs`).

Transporte (56 de alto, fondo `PANEL`, borde superior `BORDE`):
- Iconos: inicio, cuadro anterior, play/pausa de 36, cuadro siguiente, fin.
- Timecode grande en mono 20 (`00:00:19:12`) y total en `Pequeno` a la derecha,
  ambos de `rod_core::format_time`; el actual es clicable y hace seek.
- Velocidad: `ComboBox` de 28 con `VELOCIDADES`. Volumen: icono + slider de
  100 px. Bucle de selección: icono con estado activo.
- Todos con tooltip y atajo (`Space`, `,`/`.`, `J`/`K`/`L` si ya existen en
  `comandos.rs`; no se inventan atajos nuevos sin registrarlos ahí).

### 6.4 Estado

Barra de 24 al pie: `N líneas · M hablantes · S solapes`, ruta del proyecto y
`Guardando…`/`Guardado`/`Sin guardar`. Sustituye a los mensajes sueltos: lo
transitorio (exportación, errores de diálogo) va a un toast abajo al centro, con
icono por tono y descarte a los 6 s. El conteo de solapes reutiliza el que ya
calcula `exportar_srt_por_hablante`, sin una segunda implementación.

### 6.5 Panel de atajos (`ui/atajos.rs`, nuevo)

`?` (o F1) abre una capa centrada de 560 px sobre el editor: fondo `PANEL`, borde
`BORDE`, radio 6, dos columnas de filas «acción / atajo en mono», agrupadas por
contexto (Reproducción, Edición, Selección, Proyecto). `Esc` o clic fuera
cierra. Ni la etiqueta ni el atajo se escriben a mano: se leen de `Textos` y de
`MapaAtajos` (§11), así que el panel muestra los atajos que el usuario tenga
puestos, no los de fábrica. Un test comprueba que todo `Comando` que
`MapaAtajos` puede producir aparece en el panel.

## 7. Timeline (`ui/timeline.rs`, reescrito)

Pintado a mano con `Painter`; la lógica de geometría y arrastre sigue en
`timeline.rs` y `timeline/arrastre.rs` (ya probada, no se reescribe).

- Cabeceras fijas de 120 px a la izquierda: `punto_hablante`, nombre editable,
  mute, solo y candado (24×24). Sin fila «—»: los captions sin hablante van a
  una pista «Sin hablante» con clips de relleno `CONTROL`, borde discontinuo
  `BORDE_CONTROL` y etiqueta `TEXTO_2` (sin transparencia: el texto tiene que
  seguir cumpliendo AA).
- Una pista de audio de 44 px arriba con la waveform en azul grisáceo apagado
  (`#3A4A5A`), nunca en color de hablante.
- Regla de 28 px con marcas adaptativas (1 s / 5 s / 10 s / 30 s / 1 min según
  píxeles por segundo) y etiquetas en mono 11 para arriba; los píxeles por
  segundo salen de `Vista`, que ya existe.
- Clip: relleno del color de identidad al 22 %, borde sólido de 1 px al 100 %,
  texto 11 recortado con elipsis y tooltip; seleccionado con borde de 2 px
  `ACENTO`; handles de 6 px con `ResizeHorizontal`.
- Solape del mismo hablante: borde y tinte `PELIGRO`. Simultaneidad de
  hablantes distintos (islas) no es error: tinte neutro y línea de unión.
- Playhead: 1 px `ACENTO`, cabeza triangular de 9 px arriba, y chip flotante con
  el timecode mientras se arrastra.
- Interacción: rueda = desplazamiento horizontal; `Ctrl+rueda` = zoom anclado
  al cursor (no al playhead: el ancla actual es el playhead y el brief pide el
  cursor); `Shift+rueda` = desplazamiento vertical de pistas; arrastrar en vacío
  = marquesina; `Ctrl` = suma a la selección; arrastrar clip = mover con
  ajuste a bordes de otros clips, playhead y 0 (guía vertical visible a 6 px).
- Seguimiento del playhead con el botón ya existente; se apaga al mover a mano.
- Rendimiento: solo se pintan los clips visibles (búsqueda binaria por inicio),
  el texto se cachea en `Galley` por (texto, ancho, estilo) y la waveform se
  agrega por nivel de zoom una vez por cambio de vista, no por frame.

## 8. Waveform (`crates/player/src/waveform.rs`, nuevo)

`rod-player` decodifica con `symphonia` (`isomp4`, `aac`, `mp3`, `wav`, `flac`)
la **primera** pista de audio, en un hilo propio, y publica picos por `mpsc`:
`Picos { por_segundo: 100, max: Vec<f32>, rms: Vec<f32> }`. La UI pide el
análisis al abrir el video y pinta lo que haya: sin waveform, la pista de audio
se dibuja vacía y el resto del timeline funciona igual. Un video de 10 min son
~120 KB en memoria; no se escribe a disco.

Motivo de `symphonia` y no `ffmpeg`: el instalador ya no puede depender de un
binario externo (el AppImage del oráculo se rompió justo por eso). Coste: ~8
crates nuevos en `rod-player`.

## 9. Rendimiento y repintado

`main.rs` deja de pedir repintado incondicional:

- Reproduciendo → `request_repaint()` (mpv entrega frames).
- En pausa, con cambios sin guardar → `request_repaint_after(1 s)`: el autosave
  vive en `Tick` y sin frames no correría nunca.
- En pausa, tras un seek → `request_repaint_after(100 ms)` hasta que mpv deje de
  informar `seeking`.
- En pausa y limpio → nada: la entrada despierta la app.
- Durante una animación o el deslizamiento de la vista → repintado hasta que
  terminen.

`Tick` se sigue llamando en cada frame que ocurra; lo que se elimina es el frame
gratuito a 60 Hz en reposo.

Objetivo medible: con el fixture de 543 líneas, `Context::run` sin GPU por debajo
de 8 ms de media y sin asignaciones por frame en el camino de pintado del
timeline (los buffers se reutilizan).

## 10. Verificación sin pantalla

El usuario necesita el PC, así que **no se captura la pantalla**. Se añade
`egui_kittest = { version = "0.33", features = ["wgpu", "snapshot", "eframe"] }`
como `dev-dependency` de `rod-app` y un binario de desarrollo
`cargo run -p rod-app --example capturas` que:

1. Construye el estado real de Inicio (`EstadoInicio` con 0, 1 y 3 recientes) y
   de Editor (fixture de 543 líneas y 7 hablantes, `gpu = None`).
2. Renderiza a 1100×700, 1440×900 y 2560×1440 con `Harness::render()`. En el
   idioma por defecto, y una pasada en español a 1100 px: las cadenas españolas
   son las largas, así que ahí es donde se ve un desborde.
3. Escribe PNG en `docs/checklists/rediseno/` con nombre estable por pantalla,
   tamaño y fase (`editor-1440-fase3.png`, etc.).

Los mismos `Harness` sirven de tests de interacción headless: doble clic en una
fila abre el editor inline; `Esc` lo cierra sin cambios; arrastrar un clip
cambia sus tiempos; `Ctrl+rueda` cambia la escala manteniendo el tiempo bajo el
cursor. Nada de esto toca la pantalla ni abre una ventana.

Tests unitarios nuevos:
- `theme.rs`: cada par texto/fondo de §2.1 ≥ 4.5:1; todo `TextStyle` ≥ 11;
  `RADIO_CONTROL < RADIO_SUPERFICIE`.
- `color_hablante.rs`: la paleta tiene 12 colores únicos, ninguno es `ACENTO`;
  `color_identidad` es estable ante reordenamientos relativos y cae a la paleta
  ante hex inválido; `color_hablante` (preset) no cambia de comportamiento.
- `recientes.rs`: `duracion` = fin del último caption; `tira` suma los segundos
  de cada hablante con líneas y no incluye a quien no tiene ninguna.
- `ui/timeline.rs`: marcas adaptativas para 10/100/1000 px/s; ajuste (snap) a
  bordes y playhead.
- Regresión de tofu: para cada constante de `Icono`, el glifo existe en la
  fuente de Phosphor (comparación contra la tabla `cmap` leída con `ttf-parser`,
  añadido como dev-dependency; ya está en el árbol vía `ab_glyph`).
- Atajos, idioma y `preferencias.json`: los de §11.4, que son los que sostienen
  la capa de Ajustes.

Cierre de cada fase: `cargo fmt --check`, `cargo clippy -p rod-app -D warnings`,
`cargo test --workspace`, el ejemplo de capturas, y las imágenes revisadas por el
agente antes de decir «listo».

## 11. Ajustes: idioma y atajos

Lo que el oráculo tenía y esta versión no portó. Vuelve como capa modal, no como
pantalla: se abre desde el riel de Inicio y desde el menú Edición del editor, y
no interrumpe el proyecto abierto.

### 11.1 Textos (`crates/app/src/textos.rs`, nuevo)

Sin dependencia nueva y con el compilador como red:

```rust
pub struct Textos { pub inicio_titulo: &'static str, /* … */ }
pub const ES: Textos = Textos { inicio_titulo: "Edita subtítulos con precisión", /* … */ };
pub const EN: Textos = Textos { inicio_titulo: "Edit subtitles with precision", /* … */ };
```

Una tabla por idioma, las dos del mismo tipo: si una frase falta en una, no
compila. No hay claves de cadena (`t("inicio.titulo")`), que admiten erratas
silenciosas, ni un `match` de 200 brazos en el punto de dibujo.

- `pub enum Idioma { Es, En }` con `fn textos(self) -> &'static Textos`.
- Los plurales y la interpolación son métodos de `Textos`
  (`fn lineas(n: usize) -> String`), el único sitio con `format!`.
- `comandos::etiqueta` se sustituye por `textos.comando(Comando) -> &'static str`:
  un `match` exhaustivo por tabla, que el compilador obliga a completar.
- Se pasan por parámetro (`&Textos`) a las vistas; no hay estado global.
- Qué se traduce: todo lo que dibuja la UI, incluidos menús, tooltips, estados
  vacíos y las fechas relativas de los recientes. `recientes::fecha_relativa`
  pasa a devolver `FechaRelativa { dias: u64 }` y el texto lo pone `Textos`; lo
  mismo con `resumen_reciente`, `texto_vacio_recientes` y
  `MENSAJE_ARCHIVO_INVALIDO`, que hoy son `String` en español dentro de la capa
  de lógica.
- Qué no: captions, nombres de hablante y nombres de preset (son datos), y los
  mensajes de error que nacen en `rod-core` o en el sistema de archivos. Esos se
  muestran con un prefijo traducido («No se pudo abrir el proyecto: …»); para
  traducirlos de verdad habría que convertir los `String` de error en un enum,
  que no es de esta spec. Queda como deuda anotada.
- Idioma por defecto: **inglés**. `preferencias.idioma` manda desde el primer
  arranque en que exista el archivo; no se adivina por la configuración de
  Windows. El producto es global y la primera pantalla no debe obligar a cambiar
  de idioma para entenderla; quien subtitule en español lo pone en Ajustes una
  vez y queda guardado.

### 11.2 Atajos (`comandos.rs` y `atajos.rs`)

Hoy `desde_input` es un `match` fijo y `atajo()` devuelve cadenas sueltas; las
dos se sustituyen por datos.

- `Atajo { tecla: Key, ctrl: bool, shift: bool, alt: bool }` con `Display` para
  el chip en mono (`Ctrl+Shift+S`, `Space`, `?`).
- `MapaAtajos { filas: Vec<(Comando, Atajo)> }` con `por_defecto()`, que
  reproduce **exactamente** el comportamiento actual (un test recorre la tabla
  de casos de `desde_input` y compara). Las variantes de flecha son filas
  distintas (`Retroceder` = `←`, `RetrocederFino` = `Shift+←`), no una fila con
  modificadores: así el mapa es plano y una tecla nueva nunca cambia el sentido
  de otra.
- `Comando::todas() -> &'static [Comando]` alimenta el panel: los comandos sin
  atajo de fábrica (`NuevoProyecto`, los de panel…) aparecen con «—» y se les
  puede asignar uno. Un test comprueba que `todas()` cubre el enum.
- `comando_de(tecla, modificadores, foco_en_texto) -> Option<Comando>`: primer
  acierto gana; sin duplicados.
  - **Con foco en un campo de texto, solo pasan los combos con Ctrl o Alt** (y
    `Escape`). Las teclas desnudas se ignoran mientras se escribe. Hoy no es así
    y es un bug real: escribir un espacio en el textarea del caption pausa el
    video, y una `c` divide el caption. Esto se arregla aquí, porque un editor
    de atajos sobre un mapeo roto no se puede probar.
- Rebinding en la capa de Ajustes: una fila por comando, agrupadas por
  `Seccion`, con el atajo actual en mono y un `icon_button` de cambiar. Al
  pulsarlo la fila entra en «pulsa una tecla»; la siguiente pulsación con
  modificadores se asigna, `Esc` cancela. Si la tecla ya está tomada se ofrece
  **intercambiar** o cancelar, nunca dos comandos con la misma. «Restablecer»
  por fila y «restablecer todo» al pie con confirmación.
  - Aviso en la fila cuando el atajo es una tecla desnuda: no funcionará
    mientras se escribe en un campo. No se prohíbe (`E`, `A`, `C` son legítimos).
  - `Escape` no es asignable: es el cancelar universal.
  - Los `Hablante(1..9)` son nueve filas normales.
- Persistencia en `preferencias.json` junto al resto (§6.1), con `serde(default)`
  por campo: un archivo viejo o roto cae a los valores de fábrica sin perder el
  resto.
- Los tooltips y el panel de atajos (§6.5) leen del mapa; `atajo()` desaparece.

### 11.3 La capa (`ui/ajustes.rs`, nuevo)

Modal de 720×560 centrado, fondo `PANEL`, borde `BORDE`, radio 6, sobre un velo
negro al 45 %. Cabecera con título, `icon_button` de cerrar, y dos pestañas con
el `tab_bar` del kit: **Atajos** e **Idioma**.

- Idioma: dos filas de radio con el nombre nativo («Español», «English»); el
  cambio se aplica al siguiente frame, sin reiniciar.
- Atajos: lista con scroll y buscador por nombre; el contador de conflictos va
  en la cabecera de la pestaña y bloquea el cierre si quedara alguno (no puede
  quedar: se resuelven al asignar).
- Cada cambio se persiste al instante, como el panel de estilos: no hay botón
  «Guardar».
- `Esc` cierra; clic fuera cierra; el foco vuelve al widget anterior.

### 11.4 Pruebas

- `MapaAtajos::por_defecto()` reproduce `desde_input` actual, caso por caso.
- Sin duplicados en los valores por defecto; asignar un atajo tomado exige
  intercambio y el mapa nunca queda con dos comandos iguales.
- Round-trip de `preferencias.json`; archivo roto o de una versión anterior da
  los valores de fábrica.
- Con `foco_en_texto`: `Space` no pausa, `Ctrl+S` sí guarda.
- `Textos`: los dos `const` compilan (cobertura garantizada por el tipo) y un
  test recorre `Comando` comprobando que `comando()` no da cadena vacía en
  ningún idioma.
- kittest: con el idioma por defecto, comprobar que el título de Inicio está en
  inglés; abrir Ajustes, cambiar a Español, cerrar y verlo traducido, y comprobar
  que al reabrir la app sigue en español (`preferencias.json`).

## 12. Fases

1. **Tokens, tipografía y base de configuración.** `tema.rs` reescrito,
   `tipografia.rs`, iconos, `kit.rs` mínimo, `preferencias.rs`, política de
   repintado, `textos.rs` con las frases de Inicio y `comandos.rs` convertido en
   `MapaAtajos` (mismo comportamiento, con el arreglo del foco en texto). Cierre:
   Inicio actual re-estilizado sin cambiar todavía su distribución + capturas.
2. **Chrome, Inicio y Ajustes.** `chrome.rs`, menús con atajos alineados,
   monograma, Inicio completo con riel, drop zone, tarjetas, vacío/error,
   `Reciente` ampliado, y la capa de Ajustes con las pestañas de Idioma y
   Atajos funcionando (es donde el usuario ve por primera vez las dos cosas).
3. **Lista e inspector.** Filas virtualizadas, edición inline, inspector,
   selección sincronizada con el playhead, panel Hablantes con paleta y enlace a
   Estilos, toasts.
4. **Timeline.** Painter nuevo, waveform de `rod-player`, zoom anclado, snapping,
   marquesina, pistas con mute/solo/lock, playhead con timecode flotante.
5. **Pulido.** Animaciones, anillo de foco, navegación por teclado completa,
   panel de atajos con `?`, estados de carga/guardado, 1100–2560 px, y la prueba
   de 543 líneas.

Cada fase es un commit con sus capturas. **Cada fase añade sus frases a las dos
tablas de `Textos` en el mismo commit**: traducir al final obligaría a tocar dos
veces cada vista. La fase 4 necesita `symphonia`; si el decodificador falla con
el mp4 de prueba, la waveform se queda apagada y se anota en `BITACORA.md` sin
bloquear la fase.

## 13. Riesgos y migas

- **`egui_kittest` con wgpu** necesita adaptador. Si el render falla, el ejemplo
  escribe un aviso y sale con código 0 (no rompe el flujo); las capturas se
  generan entonces con la ventana real cuando el usuario lo permita.
- **Fuentes**: si el subconjunto cambia métricas respecto al TTF completo, las
  capturas de referencia se regeneran una vez y se congelan.
- **Phosphor ≠ Lucide**: trazo y proporciones difieren del mockup; se acepta la
  diferencia y se documenta. Si molesta, se sustituye el módulo `iconos.rs` por
  paths SVG propios sin tocar el resto.
- **`Hablante.color` cambia de significado** para la UI. Los `.cdp` viejos con
  colores arbitrarios muestran esos colores hasta que el usuario elija uno de la
  paleta; no se migran datos.
- **Tests existentes**: `ui/editor.rs` espera la etiqueta «Aplicar texto» y el
  bloque que se elimina; se actualizan en la fase 3 junto con el inspector.
- **Inglés a medias**: los errores que nacen en `rod-core` y en el sistema de
  archivos siguen en español con prefijo traducido. Se ve sobre todo al abrir un
  proyecto roto. Arreglarlo de verdad pide un enum de errores en `rod-core`;
  queda como deuda, no como sorpresa.

## 14. Archivos

Nuevos: `textos.rs`, `ui/tipografia.rs`, `ui/kit.rs`, `ui/chrome.rs`,
`ui/atajos.rs`, `ui/ajustes.rs`, `ui/preferencias.rs`, `ui/toast.rs`,
`crates/player/src/waveform.rs`, `crates/app/assets/fonts/` (+ OFL),
`crates/app/examples/capturas.rs`.

Reescritos: `ui/tema.rs`, `ui/iconos.rs`, `ui/inicio.rs`, `ui/editor.rs`,
`ui/timeline.rs`, `ui/transporte.rs`, `ui/paneles/{subtitulos,hablantes,estilos}.rs`,
`comandos.rs` (`MapaAtajos` en lugar del `match` fijo), `menu.rs` (presentación y
etiquetas), `main.rs` (usa `chrome`, repintado condicional).

Tocados: `recientes.rs` (`Reciente` ampliado y `FechaRelativa`), `inicio.rs`
(`orden`, `destacado`, textos fuera), `crates/core/src/color_hablante.rs` (paleta
e identidad), `crates/app/Cargo.toml` (egui-phosphor, dev-deps), `Cargo.toml`
(symphonia en `rod-player`).
