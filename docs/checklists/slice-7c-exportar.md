# Slice 7c — exportar SRT por hablante, JSON combinado y `.ass` con presets

Criterio de salida: desde la app se pueden producir los tres archivos que
exporta el oráculo, y el `.ass` exportado es **exactamente** el que rasteriza el
preview, con los presets del usuario. Nada de dos generadores que se puedan
desincronizar.

## Pasos

- [x] **1. Los presets se leen del archivo del oráculo.** `rod_app::presets_io`
  lee `%APPDATA%\RustyOpenDialogue\presets_ass.json` (el nombre de archivo del
  oráculo, `ASS_PRESETS_ARCHIVO`) con `rod_core::presets_desde_json`, así que un
  archivo ausente, roto o vacío cae a la semilla de un preset por defecto.
  `escribir` (tmp + rename) queda listo para el CRUD del slice 7d; todavía no
  hay UI que lo llame.

- [x] **2. El preview y el export son la misma función.** `ass_del_proyecto`
  pasó a recibir `&[PresetAss]` y arma el mapa `id -> preset` que consume
  `project_io::exportar_ass`. El handler de «Exportar .ass» escribe la cadena
  que devuelve `ass_del_proyecto`: el invariante de la spec §1 deja de ser una
  promesa y pasa a ser el tipo de la función.

- [x] **3. El estilo sin hablante sigue al `.ass` cargado.** Un caption sin
  hablante (o cuyo hablante no tiene `preset_id`) usa el `Default` del `.ass`
  importado, o el de fábrica si el proyecto vino de un `.srt`. Es la misma regla
  que ya usaba el preview, ahora también en el archivo.

- [x] **4. SRT por hablante: puro y con nombres únicos.**
  `project_io::exportar_srt_por_hablante(&Proyecto) -> ExportacionSrt` devuelve
  los `(nombre, contenido)` y, si hay solapes entre hablantes distintos,
  `solapes.txt` aparte. Cada archivo lleva solo los captions de su hablante,
  ordenados por inicio; el nombre sale de `nombre_archivo_hablante_unico`, que
  sanea los caracteres prohibidos de Windows y desambigua dos «Ana» (`Ana.srt`,
  `Ana_2.srt`). Un hablante sin captions no genera archivo; los captions sin
  hablante no van a ninguno pero se cuentan.

- [x] **5. JSON combinado: la lista del oráculo.** `exportar_json_combinado`
  produce `[{ inicio, fin, texto, hablante }]` con los tiempos como texto SRT y
  el nombre del hablante (la tecla si el nombre está vacío, `null` si el caption
  no tiene). El texto se genera **después** del diálogo de guardado: si el
  usuario cancela, no se hizo el trabajo.

- [x] **6. La app gana la fila «Exportar».** Tres botones en la barra de
  transporte (SRT por hablante pide carpeta; JSON y `.ass` piden archivo, con
  `subtitulos_combinado.json` y `subtitulos.ass` como nombres por defecto). El
  resultado o el error van a la barra de estado (`Estado.mensaje`): iced no
  tiene toasts y un modal para decir «listo» estorba más que ayuda.

- [x] **7. Verificar.** `cargo fmt --all -- --check`, `cargo clippy
  --all-targets -- -D warnings` y `cargo test --workspace` en 1.99 y 1.100: 90
  (app lib: 37 timeline + 11 project_io + 9 video + 7 recovery_io + 5 presets_io
  + 4 preview + 17 editor) + 8 (app bin) + 256 (core) + 3 (player) + 15 (subs) +
  3 (oráculo). La app arranca con la fila nueva y stderr vacío.

## Evidencia

- `cargo run -p rod-app --example exportar` — salida literal (recortada):

  ```
  carpeta: C:\Users\migue\AppData\Local\Temp\rod-ejemplo-exportar-15276
  --- SRT por hablante ---
  Ana.srt: 48 bytes
  Beto.srt: 48 bytes
  sin hablante: 1, solapes: 0
  --- Ana.srt ---
  1
  00:00:01,000 --> 00:00:03,000
  Hola desde Cine

  --- .ass ---
  PlayResX: 1920
  PlayResY: 1080
  Style: H1,Georgia,72,&H00A8E9FF,...   <- preset «Cine» del hablante
  Style: H2,Arial,48,&H00FFFFFF,...     <- sin preset: de fábrica
  Style: Default,Arial,48,&H00FFFFFF,...
  el archivo y el preview son idénticos: true
  píxeles con texto en el .ass exportado: 6991
  ```

  El ejemplo además escribe los archivos en la carpeta temporal (para
  inspeccionarlos) y comprueba con `assert` que el archivo leído del disco es la
  misma cadena que el preview.

- `docs/checklists/slice-7c-preset.png`: el `.ass` exportado rasterizado por
  libass. «Hola desde Cine» sale en Georgia (serif) y en el color crema del
  preset, no en Arial blanco: el preset llegó al archivo, y libass lo acepta.

- `docs/checklists/slice-7c-exportar.png`: la app con la fila «Exportar» debajo
  del transporte.

- Tests: 5 de `presets_io` (ruta, semilla, ida y vuelta, corrupto, crear
  carpeta), 5 nuevos de `project_io` (archivos y nombres únicos, sin solapes,
  hablante sin captions, JSON combinado, preset en el `.ass`) y 3 del bin para
  el resumen de la barra de estado.

## Hallazgos

- **El oráculo no tiene «exportar un SRT con todo».** Su menú «Exportar» ofrece
  SRT **por hablante**, JSON combinado y `.ass`. `exportar_srt` (todos los
  captions) se queda como primitiva de la librería y de sus tests, pero no hay
  botón: agregar una cuarta exportación que el oráculo no tiene sería inventar
  producto.
- **El orden de las claves del JSON combinado no coincide.** `serde_json` sin
  la feature `preserve_order` usa un `BTreeMap`, así que escribe
  `fin, hablante, inicio, texto`; el oráculo escribe
  `inicio, fin, texto, hablante`. Un objeto JSON no tiene orden y quien lo lee
  parsea: el contenido es idéntico. Traer `indexmap` para que el archivo se vea
  como el del oráculo no paga la dependencia.
- **El modal de estilos del oráculo no entra en 7c.** Allí se elige la
  asignación de preset por hablante *al exportar*; aquí se exporta con el
  `preset_id` guardado en el proyecto. El modal (CRUD de presets, asignación y
  persistencia en `presets_ass.json`) es el slice 7d.
- **`exportar_ass` sigue recibiendo un `HashMap`.** Es la firma que ya tenía y
  la que usan los tests; el mapa se arma en `ass_del_proyecto`, que es el único
  llamador de producción. Cambiarla a `&[PresetAss]` sería otro barrido de
  firmas sin ganar nada.
- **La CI de 7b falló en Linux por un test de Windows.** `la_ruta_es_un_recovery_json`
  comparaba el nombre exacto del archivo, pero sin `APPDATA` (Linux) la ruta cae
  al temporal con otro prefijo (`RustyOpenDialogue-recovery.json`). Se cambió a
  «la ruta termina en `recovery.json`», que vale en las dos ramas, y el test
  nuevo de presets nace con la misma forma. Windows pasaba y Ubuntu no: el test
  era el bug, no la ruta.

## Qué NO hacer todavía

- Nada de panel de estilos ni de asignar presets desde la UI (slice 7d).
- Nada de escribir `presets_ass.json` desde la app: sin CRUD no hay nada que
  guardar, y sembrar el archivo en el primer arranque solo agrega una escritura
  que el oráculo tampoco hace.
- Nada de «exportar todo el proyecto» (video + subtítulos): no está en el
  oráculo.
