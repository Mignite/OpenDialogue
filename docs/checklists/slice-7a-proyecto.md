# Slice 7a — guardar y abrir el `.cdp`

Criterio de salida: el proyecto abierto se puede guardar en un `.cdp` y volver a
abrir sin perder nada, con el mismo formato que la app Tauri, de modo que
cualquiera de las dos apps abra el archivo de la otra.

## Pasos

- [x] **1. El formato es el del oráculo, no uno nuevo.** `crates/core/src/proyecto_json.rs`
  escribe `{ ruta_video, hablantes, captions, playhead }` con `presetId` en el
  hablante y `hablante_id` en el caption: exactamente los nombres de
  `src-tauri/src/lib.rs` (`guardar_proyecto`/`cargar_proyecto`). Renombrar
  `preset_id` a snake_case habría producido un archivo que la Tauri no lee.

- [x] **2. `serde` con `derive` en `rod-core`.** Es la única dependencia extra del
  crate (más `serde_json`, ya en el árbol). Escribir a mano el
  (de)serializador de `Proyecto`/`Hablante`/`Caption` es más código y más
  frágil que la biblioteca, y el JSON es un contrato con la otra app: tiene que
  coincidir byte a byte en las claves. Justificado en la spec §4.1.

- [x] **3. Compatibilidad hacia atrás.** `#[serde(default)]` en `playhead` y en
  `Hablante::preset_id`: un `.cdp` de antes de los presets de estilo carga con
  `playhead = 0` y `preset_id = None`, igual que en el oráculo.

- [x] **4. Escritura atómica.** `rod_app::project_io::guardar_proyecto` escribe
  un `.tmp` al lado y lo renombra encima. Un corte a media escritura deja el
  archivo anterior entero; un `.cdp` truncado es peor que uno ausente. Si el
  `rename` falla, el temporal se borra.

- [x] **5. El playhead se guarda pero vive fuera del editor.** No es una edición
  (no entra al historial ni al undo): se escribe desde `state.position` al
  guardar y se restaura con un `seek` al abrir.

- [x] **6. Atajos y botones.** Ctrl+S guarda (pide ruta si no hay), Ctrl+Shift+S
  guarda como, Ctrl+O abre. Botones «Abrir .cdp» y «Guardar» en la barra de
  transporte, y la barra de estado muestra la ruta o «sin guardar».

- [x] **7. Abrir reemplaza el estado entero.** `cargar_proyecto` cambia editor
  (con historial nuevo), video, playhead y ruta. Si el video del proyecto no
  abre, no se toca nada: media app en un estado que nadie entiende es peor que
  un error claro. El oráculo sí ofrece seguir sin video; aquí se anota como
  limitación.

- [x] **8. Verificar.** `cargo fmt --all -- --check`, `cargo clippy --all-targets
  -- -D warnings` y `cargo test --workspace`: 72 (app lib: 37 timeline + 6
  project_io) + 2 (app bin) + 240 (core: 5 de `proyecto_json`) + 3 (player lib)
  + 15 (subs lib) + 3 (oráculo). La app arranca y sigue viva a los 7 s con
  stderr vacío.

## Evidencia

- `docs/checklists/slice-7a-proyecto.cdp`: 504 bytes escritos por
  `cargo run -p rod-app --example proyecto`, releídos y comparados. Ida y
  vuelta exacta (`vuelta == proyecto`), `presetId: null` presente, sin `.tmp`
  residual al lado.
- `docs/checklists/slice-7a-proyecto.png`: la barra de transporte con «Abrir
  .cdp» y «Guardar», y la barra de estado en «sin guardar» (proyecto recién
  arrancado, sin ruta todavía).
- Los tests cubren lo que la captura no puede: ida y vuelta, claves exactas,
  JSON bonito con `\n` final, proyecto viejo sin `presetId`/`playhead`, JSON
  roto, archivo inexistente, doble guardado que reemplaza.

## Hallazgos

- **En Windows `std::fs::rename` sí reemplaza** un archivo existente (usa
  `MoveFileEx` con `MOVEFILE_REPLACE_EXISTING`). No hace falta borrar antes,
  que además abriría una ventana en la que el `.cdp` no existe. El test
  `guardar_dos_veces_reemplaza_el_archivo` fija el comportamiento.
- **`to_string_pretty` no termina en `\n`.** El oráculo escribe el archivo sin
  salto final; aquí se añade uno porque el `.cdp` se versiona con git y los
  editores lo esperan. Es una diferencia de un byte que ninguna de las dos apps
  nota al leer.
- **Un `r#"..."#` se rompe con `"#fff"` dentro**: el `"#` cierra el raw string.
  El JSON de un proyecto viejo con un color `#fff` necesita `r##"..."##`. Costó
  un error de compilación confuso («suffixes on string literals are invalid»).
- **`"[]"` no es un `Proyecto`**: `serde_json` da error, no pánico. Es la prueba
  de que un archivo con la forma equivocada se rechaza en vez de producir un
  proyecto vacío silencioso.
- **`presetId: null` se escribe explícito.** El oráculo hace lo mismo (su
  `Option<String>` con `serde` no lleva `skip_serializing_if`), así que el
  archivo es idéntico en las dos apps en vez de «casi idéntico».

## Qué NO hacer todavía

- Nada de recuperación ante crash (autosave + `recovery.json`): es el slice 7b,
  y necesita el hook `useRecovery.ts` del oráculo como especificación.
- Nada de recordar el último proyecto abierto en la configuración: hoy el
  arranque siempre empieza con el proyecto de ejemplo y «sin guardar».
- Nada de avisar «hay cambios sin guardar» al cerrar: sin recovery, ese aviso
  solo molesta.
