# Arranque — menú principal (subproyecto 1)

Fecha: 2026-11-23
Estado: borrador para revisión
Spec padre: `2026-11-11-rustyopendialogue-design.md`
Oráculo: `C:/Dev/OpenDialogue` (la app Tauri sigue siendo la referencia de comportamiento)

## 1. Objetivo

Sustituir el arranque sembrado por una pantalla de inicio dentro de la app:
arrastrar o elegir un video para empezar un proyecto, abrir un proyecto
existente, ver proyectos recientes y atender un recovery pendiente. Además, el
ejecutable instalado en Windows no debe abrir consola.

Criterio de éxito:

1. `rod-app.exe` arranca mostrando el menú **sin abrir mpv ni leer `testdata/`**.
2. Soltar un video abre el editor con captions vacíos; el primer Ctrl+S pide la
   ruta `.cdp` y la escribe junto con la miniatura.
3. Los recientes muestran nombre, líneas, hablantes y fecha relativa derivados
   del propio `.cdp`; la miniatura se lee de caché y se regenera al abrir el
   proyecto si no está.
4. El aviso de recovery se ofrece en el menú y "Recuperar" abre el editor con
   ese proyecto.
5. En release, no aparece ventana de terminal al ejecutar el instalado.

Fuera de alcance: keybinds, sistema visual/estilos, empaquetado Linux,
asociación de la extensión en el sistema operativo, borrado de archivos desde
el menú.

## 2. Decisiones

Cada decisión es problema → alternativa descartada → por qué esta.

### 2.1 Menú como pantalla del mismo binario

Problema: hoy `Estado::nuevo()` (`crates/app/src/main.rs:280`) carga un SRT
sembrado y hace `Player::open("testdata/clip-15s-2audio.mp4").expect(...)`; el
menú exige que el arranque no dependa de ese archivo.

Descartado: dos binarios (`rod-inicio` + `rod-app`, ruta por `argv`) dejan dos
exes que empaquetar, dos procesos y el recovery repartido. Un menú como capa
sobre el editor obliga a abrir mpv en cada arranque, que es lo que se quiere
quitar.

Elegido: `enum Pantalla { Inicio, Editor(EditorApp) }` dentro de `rod-app`.
`main.rs` pasa a ser un enrutador delgado. El coste es mover ~1800 líneas de
`main.rs` a `editor_app.rs`; se hace en dos pasos verificables: primero mover
sin cambiar comportamiento (el sembrado sigue ahí), después sustituir el
sembrado por los constructores y añadir el menú. Cada paso con build y tests
en verde.

### 2.2 Proyecto nuevo en memoria

Al soltar un video se crea un `Proyecto` con captions vacíos y se entra al
editor; no se escribe nada en disco hasta el primer Ctrl+S.

Descartado: guardar el `.cdp` de inmediato escribe antes de saber si el video
sirve y obliga a elegir ruta en mitad del gesto de arrastre.

### 2.3 Recientes: índice de rutas + metadata leída del `.cdp`

`recientes.json` guarda rutas y orden. Líneas, hablantes y fecha relativa se
derivan al arrancar leyendo cada `.cdp` y su `mtime`.

Descartado: cachear la metadata en el índice introduce una segunda fuente de
verdad que puede mentir si el archivo se edita fuera de la app. Escanear una
carpeta fija no encaja con el oráculo, que deja elegir la ruta al guardar.

El oráculo no tiene recientes: esto es producto nuevo, no un porteo. El
mockup los dibuja con extensión `.json`, pero el formato del proyecto es `.cdp`
(JSON bonito con claves del oráculo, `rod_core::proyecto_json`); se mantiene
`.cdp` por compatibilidad y para distinguirlos de `subtitulos_combinado.json`.

### 2.4 Recovery en el menú

Hoy el aviso es una barra dentro del editor (spec padre §4.4 y §8). Con menú
inicial, ese sitio no existe hasta que se abre un proyecto, y quien entra a
trabajar en otro proyecto vería una oferta vieja. El aviso pasa al menú;
"Recuperar" abre el editor con el proyecto recuperado y borra `recovery.json`.
**Desvío documentado respecto al spec padre**, no un cambio de semántica del
recovery: la lógica pura de `rod_core::recovery` y `recovery_io` no se toca.

### 2.5 "Ver todos" expande la misma columna

De 3 entradas a la lista completa con scroll. Descartado: una vista separada
con buscador añade estado y acciones que el mockup no pide.

### 2.6 Miniatura en caché, generada al guardar, en el segundo del playhead

Al guardar se escribe un PNG en caché a partir del frame que ya tiene el
editor en memoria, **en el instante exacto del `playhead` que se está
guardando**: la imagen muestra el segundo en que el usuario dejó el proyecto y
coincide con el `playhead` que se restaura al reabrir. No se abre un player
nuevo por reciente. El frame compuesto (video + subtítulos) se reduce a 160×90
antes de codificarlo: a 1920×1080 cada miniatura pesaría varios MB por proyecto.

No hace falta un campo nuevo para el segundo: `rod_core::proyecto_json` ya
escribe `playhead` y la app ya lo restaura con `seek` al abrir
(`main.rs:422`, `:721`, `:523`, `:372`). "No perder dónde vamos entre sesiones"
ya funciona para los `.cdp`; el añadido es que la miniatura corresponda a ese
mismo punto, no a un frame arbitrario.

Si el PNG no existe, el menú dibuja un placeholder y la miniatura se regenera
**al abrir el proyecto**, nunca al arrancar.

Descartado: generar las miniaturas al dibujar el menú abre N players y retrasa
la pantalla de inicio. El placeholder es un rectángulo con un glifo de video
dibujado con `Canvas` o texto ASCII.

### 2.7 Consola solo en release

`#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`, igual
que el oráculo (`src-tauri/src/main.rs:2`). En debug se conserva la consola
para ver errores durante el desarrollo; el instalado no la abre. El `.lnk`
del instalador ya apunta directo a `rod-app.exe`, así que no hay que tocar
NSIS.

### 2.8 Sin estilo, cero emojis

Los widgets son los que trae iced y el tamaño de ventana se mantiene en
1600×900. Los iconos son formas vectoriales de `Canvas` o ASCII; no se usan
emojis ni se añade un crate de SVG. El sistema visual es otro subproyecto.

## 3. Arquitectura

```
crates/app/src/
  main.rs          main() + enum Pantalla + Message/update/view/subscription que delegan
  inicio.rs        EstadoInicio, MensajeInicio, view: arrastre, botones, recientes, recovery
  editor_app.rs    Estado y Message actuales de main.rs, sin cambio de comportamiento
  recientes.rs     índice, dedup, tope, metadata derivada del .cdp
  thumbnail.rs     ruta de caché, escritura del PNG, ausencia → placeholder
  editor.rs        modelo de edición (documento + historial): no se cambia
  project_io.rs    lectura/escritura atómica del .cdp: se reutiliza
  recovery_io.rs   sigue igual; cambia solo quién muestra el aviso
```

`main.rs` expone los constructores del editor que sustituyen al sembrado:

- `EditorApp::desde_video(path) -> Result<Self, Error>`: `Proyecto::nuevo(path)`
  con captions vacíos, sin ruta de proyecto todavía.
- `EditorApp::desde_proyecto(proyecto, ruta) -> Result<Self, Error>`: valida que
  el video abre antes de construir el editor (spec padre §4.4).

El arrastre usa `iced::window::events()`, que devuelve `(Id, Event)`, y
`window::Event::FileDropped(PathBuf)` (verificado en iced 0.14). La app tiene
una sola ventana, así que no se filtra por `Id`.

`image` pasa de `dev-dependencies` a `dependencies` de `rod-app` (ya está en el
árbol para los PNG del preview); `rfd` ya está disponible para los diálogos.

## 4. Flujo

1. **Arranque.** `boot` lee `recientes.json` y el recovery pendiente y
   construye `Pantalla::Inicio`. No crea player, no lee presets hasta que se
   entra al editor (como hoy) y no toca `testdata/`.
2. **Video nuevo.** `FileDropped` o *Abrir video* (`rfd`) → comprobar que la
   extensión es `mp4`, `mkv`, `mov` o `webm` → `EditorApp::desde_video` →
   `Pantalla::Editor`. Es un filtro de interfaz, no una garantía: mpv decide si
   el archivo abre. Un video que no abre produce un mensaje en la propia
   pantalla de inicio, sin `eprintln!`.
3. **Abrir proyecto.** Botón o entrada de recientes → `project_io::leer` →
   `EditorApp::desde_proyecto`. Si el `.cdp` no abre o su video no existe, se
   muestra el error y se permanece en el menú: no se abre un editor a medias.
   Si falta su miniatura, se genera con el frame ya disponible al entrar.
4. **Recovery.** "Recuperar" usa el recovery leído en el arranque y llama a
   `desde_proyecto` con su proyecto; "Descartar" borra el archivo y se queda en
   el menú.
5. **Guardar.** El `.cdp` ya escribe `playhead = state.position` con escritura
   atómica. A esa operación se añaden: registrar la ruta en `recientes.json`
   (dedup, tope 10) y refrescar el PNG en caché con el frame compuesto en ese
   mismo instante, reducido a 160×90. Imagen y punto de reanudación salen de
   la misma variable, no de dos lecturas separadas del reloj.

## 5. Datos en disco

- `%APPDATA%\RustyOpenDialogue\recientes.json`: array de rutas absolutas a
  `.cdp`, más reciente primero, tope 10, dedup por ruta canónica. Un archivo
  roto o inválido da una lista vacía, igual que `presets_ass.json`.
- Metadata derivada: `captions.len()`, `hablantes.len()` y fecha relativa desde
  el `mtime` del `.cdp`. Nada de esto se persiste.
- `%APPDATA%\RustyOpenDialogue\thumbs\<hash>.png`: `hash` de la ruta canónica
  con `std::collections::hash_map::DefaultHasher`. No es criptográfico ni un
  contrato entre versiones: si cambia, la miniatura se regenera. El segundo al
  que corresponde no se guarda aparte: es el `playhead` del `.cdp`.
- Si un `.cdp` del índice ya no existe o no parsea, se omite al mostrar y se
  elimina del índice.

## 6. Errores

| Situación | Comportamiento |
|---|---|
| `recientes.json` roto | Lista vacía, sin crash |
| `.cdp` referenciado ausente o inválido | Se omite y se limpia del índice |
| PNG de caché ausente | Placeholder; regeneración al abrir el proyecto |
| Video que no abre al soltarlo | Mensaje en el menú, se permanece en él |
| Fallo al escribir caché | Se ignora en silencio; es una caché, no el proyecto |

## 7. Testing y verificación

Tests puros en `recientes.rs` (dedup, tope, orden, omisión de inexistentes,
metadata desde un `.cdp` fixture) y `thumbnail.rs` (ruta determinista, ausencia
→ `None`). Las decisiones de pantalla se prueban a nivel de `update`: abrir un
reciente cambia a `Pantalla::Editor`; descartar el recovery borra el archivo.

Semáforo: `cargo build`, `cargo fmt --all -- --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test --workspace`.

Verificación observable: ejecutar la app desde una carpeta **sin `testdata/`**
y comprobar que el menú aparece igual; arrastrar un video real → captions
vacíos → situar el playhead en un segundo conocido → Ctrl+S → `.cdp` con ese
`playhead` y PNG; reabrir y comprobar que el player hace `seek` a ese segundo y
que la miniatura mostrada corresponde a ese instante; borrar el PNG, ver el
placeholder y confirmar que se regenera al abrir; capturar la ventana y
comprobar que el ejecutable release no abre consola.

## 8. Desvíos del spec padre

- Recovery: de barra en el editor a aviso en el menú (§2.4).
- Arranque: de proyecto sembrado a menú sin mpv (§2.1).
- Persistencia nueva: `recientes.json` y `thumbs/`, sin equivalente en el
  oráculo.
- Dependencia `image` de dev a normal en `rod-app`.
- `subscription` añade `window::events()` para `FileDropped`.

## 9. Riesgos

- El evento de arrastre de iced no trae modificadores ni validación de tipo:
  la extensión se comprueba en la app antes de abrir mpv.
- `DefaultHasher` puede cambiar entre versiones de Rust: el efecto es solo una
  miniatura huérfana, no pérdida de proyectos.
- Partir `main.rs` es el paso de mayor riesgo mecánico; se aísla en un primer
  slice que no cambia ningún comportamiento y se verifica con las capturas
  existentes de los slices 7 y 8.
