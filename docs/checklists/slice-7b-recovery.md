# Slice 7b — copia automática y recuperación ante crash

Criterio de salida: si la app muere con cambios sin guardar, al volver a
arrancar ofrece recuperarlos; si no hay nada que recuperar (o el archivo está
roto, caducado o es de otra versión), arranca normal sin molestar. Y guardar con
Ctrl+S deja de necesitar el seguro.

## Pasos

- [x] **1. Las decisiones son puras y se prueban sin disco.**
  `rod_core::recovery` porta `src/utils/recovery.ts`: `Recovery { version: 1,
  rutaProyecto, escrito, proyecto }`, `VIGENTE_MS` (7 días),
  `INTERVALO_AUTOSAVE_MS` (15 s), `debe_guardar`, `construir`, `leer`,
  `formatear`. Mismas claves en camelCase que el oráculo, así que un recovery
  escrito por la app Tauri se lee aquí.

- [x] **2. `leer` valida campo por campo, no confía en `serde`.** El oráculo
  comprueba `version === 1`, `rutaProyecto` string, `escrito` finito y que el
  proyecto tenga `captions`/`hablantes` como arrays. `{ "version": 1,
  "rutaProyecto": "/a", "proyecto": {} }` tiene la forma correcta y aun así no
  es un recovery: `serde` solo lo rechazaría por accidente. Cualquier fallo es
  `None`, nunca un pánico: un archivo roto no puede impedir abrir la app.

- [x] **3. `escrito` en epoch ms como `Date.now()`.** `SystemTime` desde
  `UNIX_EPOCH` en `f64`; el reloj entra como parámetro en las funciones puras
  (`ahora`, `corte`) para que la vigencia y el intervalo sean testeables sin
  dormir el test.

- [x] **4. El archivo vive en `%APPDATA%\RustyOpenDialogue\recovery.json`.**
  Equivalente al `app_config_dir` del oráculo, sin traer la API de Tauri. La
  escritura es atómica (`.tmp` + `rename`), y `None` borra (un archivo que ya no
  está no es un error).

- [x] **5. «Sucio» se calcula contra una línea base, no con un flag que hay que
  acordarse de encender.** `Estado.limpio` guarda el proyecto del último
  guardado/apertura/arranque; `sucio` compara captions y hablantes. El
  `playhead` queda fuera a propósito: cambia con el reloj del player y marcaría
  el proyecto sucio 60 veces por segundo (el oráculo lo esquiva con un
  `useEffect` que no depende del playhead).

- [x] **6. El chequeo va a 1 Hz, no por frame.** `autosave` corre en el `Tick`
  pero sale en cuanto no ha pasado un segundo: comparar 500 captions 60 veces
  por segundo para nada es tirar CPU. Dentro, decide `debe_guardar` (cambios,
  algo que recuperar, 15 s desde la última escritura). Falla en silencio: el
  seguro no es el aviso al usuario.

- [x] **7. El aviso es una barra, no un diálogo modal.** iced no tiene modales y
  bloquear la ventana antes del primer frame es peor que dejar la decisión a la
  vista. La barra dice `Recuperado hace 3 min (mi-video.cdp) — 3 subtítulos` con
  «Recuperar» y «Descartar».

- [x] **8. Recuperar carga el proyecto entero y deja el proyecto sucio.**
  Reusa `cargar_proyecto` (editor, video, playhead), así que el video del
  recovery tiene que abrir; si no, el aviso se queda en pantalla y el error va a
  stderr. No se marca limpio: el `.cdp` en disco es más viejo que lo recuperado,
  igual que en el oráculo. Descartar borra el archivo para no volver a
  preguntar.

- [x] **9. Guardar borra el seguro.** Tras un Ctrl+S exitoso no hay nada que
  ofrecer; el oráculo deja el archivo hasta que caduca. Además mueve la línea
  base del «sucio», que es lo que apaga el autosave.

- [x] **10. Verificar.** `cargo fmt --all -- --check`, `cargo clippy
  --all-targets -- -D warnings` y `cargo test --workspace`: 79 (app lib: 37
  timeline + 6 project_io + 7 recovery_io + 29 del editor) + 5 (app bin) + 256
  (core: 16 de `recovery`) + 3 (player lib) + 15 (subs lib) + 3 (oráculo). La app
  arranca, muestra el aviso y sigue viva con stderr vacío.

## Evidencia

- `cargo run -p rod-app --example recovery` — salida literal:

  ```
  archivo: ...\rod-ejemplo-recovery-2956.json (507 bytes)
  {"version":1,"rutaProyecto":"/proyectos/mi-video.cdp","escrito":1795145887624.8477,
   "proyecto":{"ruta_video":"testdata/clip-15s-2audio.mp4", ... ,"playhead":9.0}}
  ida y vuelta exacta: true
  aviso: Recuperado hace 3 min (mi-video.cdp) — 3 subtítulos
  escribe con cambios: true
  escribe sin cambios: false
  escribe proyecto vacío: false
  escribe 14.9 s después: false
  escribe 15 s después: true
  más viejo que el corte: None
  más viejo que 7 días: None
  archivo corrupto: None
  tras borrar, existe: false
  ```

- `docs/checklists/slice-7b-recovery-aviso.png`: la barra ámbar al arrancar,
  con el texto formateado y los dos botones, sobre el proyecto de ejemplo (2
  captions, «sin guardar»). Sembrada con `--example recovery -- sembrar`.

- Los tests cubren lo que la captura no: 16 de `rod_core::recovery` (ida y
  vuelta, claves del oráculo, falta de campos, versión 99, corte, vigencia,
  intervalo de 15 s, antigüedad legible) y 7 de `recovery_io` (escribir/leer,
  borrar inexistente, basura, crear carpeta) más 3 del bin (aviso con nombre de
  archivo, sin archivo, archivo corrupto).

## Hallazgos

- **`SetForegroundWindow` se niega si otra app tiene el foco** y `mouse_event`
  inyecta el clic en la ventana que sí lo tiene. Con el escritorio ocupado no se
  puede automatizar un clic en nuestra ventana sin robarle el foco al usuario:
  la captura de «Recuperar» pulsado no se hizo, y el camino queda cubierto por
  los tests de `recovery_pendiente` y por `cargar_proyecto`, que es el mismo que
  usa «Abrir .cdp» (verificado en 7a).
- **`toLocaleString()` no tiene equivalente en Rust sin zona horaria.** El aviso
  dice la antigüedad («hace 3 min») en vez de la fecha local. Es lo que decide
  si vale la pena recuperar, y evita traer un crate de fechas para una línea.
- **`!(x >= y)` vuelve a ser error de clippy** (`neg_cmp_op_on_partial_ord`) en
  el cálculo de la antigüedad; queda como `!ms.is_finite() || ms < 60_000.0`,
  que además dice la verdad sobre NaN.
- **El oráculo marca sucio con un efecto sobre `captions`/`hablantes`/`ruta`,
  no comparando contra el archivo.** Aquí la línea base (`limpio`) evita el
  flag que hay que acordarse de apagar en cada camino de edición: deshacer hasta
  el estado guardado vuelve a decir «limpio» gratis.

## Qué NO hacer todavía

- Nada de avisar «hay cambios sin guardar» al cerrar: la copia automática ya
  cubre el caso, y un aviso sin diálogo modal solo se puede mostrar como barra.
- Nada de varios recoveries ni de historial de copias: un archivo, el último.
- Nada de autosave en un hilo aparte: el archivo son unos KB y la escritura va a
  1 Hz como mucho.
