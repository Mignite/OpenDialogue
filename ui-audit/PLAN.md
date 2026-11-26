# Plan de pulido visual

Fase 0 cerrada: arnés egui_kittest + wgpu, 91 capturas `before/` e `index.png`.
Fase 1 cerrada: seis auditores de solo lectura (A–F). Fase 2: este plan.
Fase 3: Fundación → áreas A–E en paralelo. Fase 4: integración + verificador.

## Reglas compartidas

- No cambiar lógica ni modelo de datos. No tocar `crates/core` (incluido `ass.rs`).
- Todo valor visual sale de `crates/app/src/ui/tema.rs`. Todo texto visible sale
  de `crates/app/src/textos.rs`. Un archivo pertenece a un solo subagente.
- Invariante: el preview es el mismo ASS que se exporta; el color de identidad
  del hablante es tinte de UI (barra + chip), nunca entra al render ni al export.
- Animaciones de 120–160 ms con `animate_bool_with_time`.
- Sin dependencias nuevas; sin `unwrap` nuevo en rutas de UI.
- Nada se da por terminado sin captura `after` que lo demuestre.

## Criterios de aceptación

Contraste AA; ningún texto < 11 px; foco de 2 px en acento; estados hover,
active, disabled y focus en cada control; sin desbordes de 1100 a 2560 px;
tooltips a 0.4 s con atajo; un solo formato de timecode y un solo idioma por
pantalla; 60 fps con el proyecto grande.

## Notas sobre las capturas `before`

- El playhead ausente que reportó la auditoría E era **artefacto del arnés**:
  `ancho_timeline` quedaba en 0 porque el helper descartaba el mensaje `Ancho`
  que publica el widget. Corregido en `dc54aef`; `before/` se regeneró. El
  playhead ahora se ve en `editor__subtitulos-grande__1600x900.png`. Queda como
  mejora E1b el marcador de borde cuando el tiempo sale del área.
- `menu__ayuda__*` sí abre el desplegable: tiene un solo elemento
  («Keyboard shortcuts») y el botón va sin marco, así que parece un tooltip.
- No hay diálogo de export/import in-app (`main.rs` usa `rfd` nativo) ni menú de
  engranaje (Ajustes está en Edit). Se capturan Preferences y el panel de atajos.

## Fundación — `ui/tema.rs`, `ui/kit.rs`, `textos.rs`, `ui/tipografia.rs`

Propiedad exclusiva del subagente de Fundación. Debe compilar antes de que
arranquen las áreas. Entregará la firma exacta de los helpers nuevos.

| ID | Sev. | Archivo | Arreglo |
|---|---|---|---|
| F1 | Alta | `tema.rs:271`, `:29`, `:377-392` | `selection.bg_fill = ACENTO` con `TEXTO` da 2,59:1; `PELIGRO` como texto da 3,07:1 sobre `FONDO`. Añadir `SELECCION_FILA`, `ERROR_TEXTO` con AA ≥ 4,5, `BANNER_INFO` azul tenue y ampliar el test de pares. |
| F2 | Media | `tema.rs:24` | `TEXTO_3` sobre `CONTROL` da 4,49:1: subir a #969696 o documentar el par correcto. |
| F3 | Media | `tema.rs` | Constantes para los valores repetidos por áreas: alto de pill 16, separadores, grosores, radios, tamaños de keycap/punto, duración de animación 120–160 ms. |
| F4 | Media | `kit.rs:451-472`, `:227`, `:246`, `:293`, `:385` | Hex del logo y tamaños sueltos a `tema.rs`; medición real del ancho del título en `chrome.rs` la hace B con `galley`. |
| F5 | Media | `kit.rs:170-215`, `anillo_foco` | Hacer público `anillo_foco`; `icon_button_en` debe tener hover/active/focus como `icon_button`. |
| F6 | Media | `kit.rs` | Añadir helpers: `keycap`, `pill` (16 px), `etiqueta_seccion`, `banner_informativo`, `campo_color` (swatch + hex), `toggle`, `drag_value_con_sufijo`. |
| F7 | Media | `kit.rs:glifo / Icono` | Glifos Phosphor para «···», chevron, imán, objetivo/crosshair y cierre de modal; sustituir los textos sueltos en las áreas. |
| F8 | Media | `textos.rs` | Claves nuevas para todas las áreas: `tiempo`, `pista_audio`, `sin_estilo`, `fit_all`, `hide_empty_tracks`, `collapse_track`, `restore`, `discard`, `video_not_found`, `locate_video`, `solapes(n)`, `fecha_recuperacion(...)`, tooltips de transporte y ventana, `sin_atajo`, `preview_texto`, `variante_no_disponible`, `editar`, `seguir`. Una sola tabla EN/ES. |
| F9 | Baja | `ui/tipografia.rs:190` | `let _ = ctx.run(...)` para que `clippy -D warnings` pase sin warnings preexistentes. |

## A — Inicio — `ui/inicio.rs`, `recovery_io.rs`

| ID | Sev. | Captura | Arreglo |
|---|---|---|---|
| A1 | Alta | `inicio__recuperacion__1100x700.png` | Banner rojo saturado por canales premultiplicados mal usados: usar `BANNER_INFO` de `tema` y `from_rgba_unmultiplied`; contraste ≥ 4,5. |
| A2 | Media | `inicio__recuperacion__*` | Añadir botones «Restore» / «Discard» dentro del banner que emitan los mensajes existentes en `inicio.rs:74-75`. |
| A3 | Media | `inicio__recuperacion__1100x700.png` | Banner de ~356 px solapando la drop zone: ancho del contenedor y separación real de `ESP_16`. |
| A4 | Media | `inicio__recuperacion__1920x1080.png` | Texto en español del core con UI en inglés: reconstruir el aviso en la capa app con la clave `fecha_recuperacion` de F8. |
| A5 | Media | `inicio__vacio__1100x700.png` | Quitar el subrayado azul suelto bajo el wordmark. |
| A6 | Media | `inicio__recientes__1600x900.png` | «Most recent⌄See all» sin separación: `ESP_16`, «See all» como enlace con color/subrayado del tema. |
| A7 | Media | `inicio__recientes__1600x900.png` | Miniatura con ~9 px de margen: sangre total arriba/laterales; 8 px solo en el texto. |
| A8 | Media | `inicio__recientes__1100x700.png` | Tarjetas de 292 px y huecos de 34 px: ancho = (disponible − 2·gap)/3 con gap exacto de `tema`. |
| A9 | Media | `inicio__recientes__1600x900.png` | ~17 px entre nombre y meta, ~35 px vacíos abajo: `item_spacing.y = 0` y sin `ESP_12` final. |
| A10 | Media | `inicio__recientes__1100x700.png` | `···` literal siempre visible: glifo de `Icono` solo en hover/foco. |
| A11 | Media | `inicio__recientes__1600x900.png` | `sin-video.cdp` sin chip: placeholder + chip ámbar «Video not found» con clave de F8. |
| A12 | Baja | `inicio__recientes-es.png` | Hex del logo y `Color32::WHITE/BLACK`/`from_black_alpha(150)` a constantes de `tema`. |
| A13 | Media | `inicio__arrastrando__*` | Mantener foco de 2 px y estado de arrastre en acento (verificado; solo regresión). |
| — | — | `inicio__vacio__1600x900.png` | Descartado: el atajo es `Ctrl+O` con letra O, no `Ctrl+0`; fue ambigüedad de la fuente mono. |

## B — Chrome — `ui/chrome.rs`, `menu.rs`, `ui/ajustes.rs`, `ui/atajos.rs`, `ui/toast.rs`

| ID | Sev. | Captura | Arreglo |
|---|---|---|---|
| B1 | Alta | `menu__archivo__1920x1080.png` | Los desplegables no muestran atajos; `menu.rs:60` descarta `atajos`. Columna derecha con `atajos.atajo(comando)` en mono. |
| B2 | Media | `ajustes__atajos__1920x1080.png` | Sin anillo de foco de 2 px en pestañas, botones Change/Reset y toast Undo: usar `kit::anillo_foco` / `visuals.widgets.focused`. |
| B3 | Media | `menu__edicion__2560x1440.png` | Ítems de menú sin hover/active: scope con `hovered.bg_fill = CONTROL`, `active.bg_fill = ELEVADO`. |
| B4 | Media | `ajustes__atajos__1100x700.png` | Columnas desalineadas y botón que repite la etiqueta: ancho fijo, atajo a la derecha, botón «Change» solo. |
| B5 | Media | `ajustes__atajos__1920x1080.png` | `PELIGRO` sobre `PANEL` da 2,8:1: usar `ERROR_TEXTO` de F1. |
| B6 | Media | `ajustes__atajos__2560x1440.png`, `ayuda__atajos__2560x1440.png` | Diálogos fijos 720×560 y 560×480: derivar el tamaño de `ctx.screen_rect()` con límites de `tema`. |
| B7 | Media | `editor__toast__1100x700.png` | Tooltip de Undo sin `Ctrl+Z`: `on_hover_ui` con texto + atajo mono. |
| B8 | Baja | `menu__archivo__1920x1080.png` | Tooltips «minimizar/maximizar/cerrar/arrastrar-ventana» en español dentro de UI EN: claves de F8. |
| B9 | Baja | `ajustes__idioma__1920x1080.png` | RadioButtons con tipografía distinta: `Estilo::Base`. |
| B10 | Baja | `ajustes__idioma__1920x1080.png` | Modal sin cierre visible: botón X que llame a `cerrar`. |
| B11 | Baja | `menu__archivo__1920x1080.png`, `editor__toast__1920x1080.png` | 20/24/160 px, logo 26, offset −16 y VIDA 6 s a constantes de `tema`. |
| B12 | Baja | `ayuda__atajos__1920x1080.png` | `?` como atajo ambiguo: texto «None»/«Sin atajo» de F8. |
| B13 | Media | `menu__archivo__1920x1080.png` | Estimar el ancho del título con `galley` real en vez de `len() * size * 0,5` (falla con no ASCII). |
| — | — | `editor__guardado__1920x1080.png` | Verificado correcto: punto gris/ámbar, tooltip 0,4 s y atajos a la derecha ya funcionan. |

## C — Subtítulos, barra de texto y transporte — `ui/paneles/subtitulos.rs`, `ui/texto.rs`, `ui/transporte.rs`

| ID | Sev. | Captura | Arreglo |
|---|---|---|---|
| C1 | Alta | `editor__subtitulos-grande__1600x900.png`, `_mediano__1100x700.png` | `TextEdit` a todo el ancho y controles encima: reservar el ancho de bucle/volumen/velocidad antes de calcular el campo. |
| C2 | Media | `editor__subtitulos-grande__1600x900.png` | `1.54 s` frente a `12.46−10.93 = 1.53`: calcular la duración con la misma base de centésimas que `tiempo_corto`. |
| C3 | Media | `editor__subtitulos-grande__1600x900.png` | Dos formatos de timecode (lista `00:00.32`, transporte `00:07:53.13`): un único formateador para lista, transporte y timeline. |
| C4 | Media | `editor__vacio__1600x900.png` | «Sin hablante» hardcodeado en UI EN: `textos.sin_hablante`. |
| C5 | Media | `editor__subtitulos-grande__1600x900.png` | Tooltips y «solapes» en español: claves de F8 / `textos.comando(...)`. |
| C6 | Media | `editor__vacio__1600x900.png` | Chip de hablante sin chevron: glifo de F7 en el `MenuButton`. |
| C7 | Media | `editor__foco-texto__1600x900.png` | Foco del campo de 1 px: `Stroke::new(2.0, ACENTO)` con `StrokeKind::Outside`. |
| C8 | Media | `editor__subtitulos-grande__1600x900.png` | Pills de pestaña de ~30×29 px: usar `kit::pill` de 16 px. |
| C9 | Media | `editor__subtitulos-mediano__1920x1080.png` | Padding de 8 px y lápiz recortado a la derecha: 12 px a ambos lados y reservar la scrollbar (coordinar con E en `editor.rs`). |
| C10 | Baja | `editor__subtitulos-grande__1600x900.png` | Iconos de lápiz/papelera por fila en vez de «Edit» compacto con tooltip. |
| C11 | Baja | `subtitulos.rs:28`, `texto.rs:155`, `transporte.rs:102` | Literales 72/64/10/5/24/260 a constantes de `tema`. |
| C12 | Media | `editor__subtitulos-mediano__1100x700.png` | Clonar y `to_lowercase()` 543 captions por frame: filtrar por referencia y cachear el resultado del filtro. |

## D — Speakers y Styles — `ui/paneles/hablantes.rs`, `ui/paneles/estilos.rs`

| ID | Sev. | Captura | Arreglo |
|---|---|---|---|
| D1 | Alta | `editor__hablantes__1100x700.png` | Selector dice «No speaker»: clave `sin_estilo` («Style / No style») de F8 y opción para limpiar preset. |
| D2 | Media | `editor__hablantes__*` | Tecla 1…7 como texto suelto: `kit::keycap`; tooltips Mute/Solo con atajo. |
| D3 | Media | `editor__hablantes__1100x700.png` | Barra de color de ~70×24 px: punto circular con `Button::new("")` y `min_size`. |
| D4 | Media | `editor__estilos__1600x900.png` | «FONT/FILL/OUTLINE/SHADOW» hardcodeados en inglés: `textos.fuente/color/contorno/sombra`. |
| D5 | Baja | `editor__estilos__1100x700.png` | «Aa Hola mundo» y «Variante no disponible» en español en UI EN: claves de F8. |
| D6 | Media | `editor__estilos__1920x1080.png` | ComboBox, DragValues, swatch y hex sin foco de 2 px: `kit::anillo_foco`. |
| D7 | Baja | `editor__estilos__1100x700.png` | Swatch cuadrado de 28 px: punto circular de `tema` dentro del toggle. |
| D8 | Baja | `editor__hablantes__1100x700.png` | Padding de 8 px en tarjetas: `ESP_12`. |
| D9 | Media | `hablantes.rs:33-44` | Clonar hablantes + captions y O(hablantes×captions) por frame: contadores precalculados al cambiar el proyecto. |
| — | — | `editor__estilos__1600x900.png` | Verificado correcto: la vista previa usa `render_subtitle`/libass igual que el overlay; DragValue con « px» y sin campo Name. |

## E — Timeline y layout del editor — `ui/timeline.rs`, `ui/editor.rs`, `src/timeline.rs`, `ui/transporte.rs`

`ui/transporte.rs` está en propiedad compartida: C hace i18n y tooltips, E solo
cambia el icono de bucle. C no toca el archivo; E lo cierra en su commit.

| ID | Sev. | Captura | Arreglo |
|---|---|---|---|
| E1 | Baja | `editor__subtitulos-grande__1600x900.png` | Playhead fuera del rect: era artefacto del arnés, ya corregido en `before/`. Queda E1b: marcador de borde en la regla cuando el tiempo sale del área. |
| E2 | Media | `editor__subtitulos-es.png` | «TIEMPO» y «Audio» hardcodeados: claves de F8. |
| E3 | Media | `editor__subtitulos-grande__1600x900.png` | Bucle duplicado e iconos poco distintivos: icono propio para Follow, imán para Snap, bucle solo en el transporte. |
| E4 | Media | `editor__subtitulos-grande__1600x900.png` | Sin «Fit all» ni «Hide empty tracks»: botones en la toolbar que llamen a `Vista::encajar` y filtren carriles vacíos. |
| E5 | Media | `editor__subtitulos-grande__1600x900.png` | Sin alturas S/M/L ni colapso de pista: `alto_pista` en `MedidasEditor` y chevron que reduzca la fila. |
| E6 | Media | `editor__subtitulos-grande__1600x900.png` | El carril Audio se desplaza con el scroll: fijar fila 0 bajo la regla. |
| E7 | Media | `editor__subtitulos-grande__1100x700.png` | Gutter fijo de 104 px: guardar `ancho_gutter` en `MedidasEditor` y arrastrar el borde. |
| E8 | Media | `editor__subtitulos-grande__1100x700.png` | Splitter sin doble clic y suelos de 180/160 px: doble clic alterna topes, video mínimo 140 px, usar `GROSOR_SPLITTER`. |
| E9 | Baja | `editor__vacio__1600x900.png` | Preview no a sangre por el frame del `CentralPanel`: `Frame::NONE` + fondo `COLOR_FONDO_VIDEO`. |
| E10 | Baja | `editor__subtitulos-grande__1600x900.png` | `gamma_multiply`, `from_white_alpha`, 104/56 a constantes de `tema`. |
| E11 | Media | `editor__subtitulos-grande__1600x900.png` | Foco del slider de zoom sin anillo: `kit::anillo_foco`. |
| E12 | Media | `ui/editor.rs:211` | `Arc::new(state.pixels.clone())` copia 8,3 MB por frame: reutilizar el `Arc` y copiar solo con frame nuevo. |
| E13 | Media | `editor.rs:92`, `:117-123` | Padding del inspector y toggle de colapsar el panel (compartido con C9). |

## Descartados / fuera de alcance

- Diálogo «Locate video…» al abrir un reciente sin video: exige cambiar el
  flujo de apertura en `main.rs` (lógica), prohibido en este pase. Se arregla
  solo la tarjeta (A11).
- Cambios en `crates/core` / `ass.rs`: invariante de export; no se tocan.
- Menú de settings con engranaje: no existe en esta app; Ajustes está en Edit.
- `menu__ayuda__*` parecía sin dropdown: es un menú de un solo elemento sin
  marco, no un fallo.

## Resumen de propiedad

| Subagente | Archivos |
|---|---|
| Fundación | `ui/tema.rs`, `ui/kit.rs`, `textos.rs`, `ui/tipografia.rs` |
| A | `ui/inicio.rs`, `recovery_io.rs` |
| B | `ui/chrome.rs`, `menu.rs`, `ui/ajustes.rs`, `ui/atajos.rs`, `ui/toast.rs` |
| C | `ui/paneles/subtitulos.rs`, `ui/texto.rs`, `ui/transporte.rs` (i18n) |
| D | `ui/paneles/hablantes.rs`, `ui/paneles/estilos.rs` |
| E | `ui/timeline.rs`, `ui/editor.rs`, `src/timeline.rs`, `ui/transporte.rs` (icono) |

Cada uno pasa `cargo check -p rod-app --all-targets` y
`cargo clippy -p rod-app --all-targets -- -D warnings`, vuelve a renderizar sus
capturas y hace un commit. El verificador de Fase 4 no implementa nada.
