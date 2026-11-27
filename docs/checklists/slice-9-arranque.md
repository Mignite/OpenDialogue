# Slice 9 — arranque con menú principal

Especificación: `docs/superpowers/specs/2026-11-23-arranque-design.md`
Plan: `docs/superpowers/plans/2026-11-23-arranque.md`

Criterio de salida: la app arranca en un menú sin abrir mpv ni leer
`testdata/`; desde ahí se abre un video nuevo o un `.cdp` reciente; el recovery
se ofrece en el menú; el ejecutable release no abre consola en Windows.

## Semáforo

- [x] `cargo build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo test --workspace` — 403 tests en verde (122 lib + 3 bin + 1
  integración + 256 core + 15 subs + 3 player + 3 varios; el resto 0).
  Incluye los seis tests añadidos en la pasada de corrección de la revisión
  final: seek al playhead, recovery conservado en fallo, miniatura/placeholder,
  caché fallida, regeneración al abrir y aislamiento de `%APPDATA%`.

## Verificación observable

### 1. Arranque sin `testdata/`

```powershell
Start-Process -FilePath '...\target\debug\rod-app.exe' `
  -WorkingDirectory "$env:TEMP\rod-arranque-smoke" -PassThru
# tras 20 s:
pid=16988 exited=False title=[Rusty Open Dialogue]
```

El código anterior habría panicado en
`Player::open("testdata/clip-15s-2audio.mp4").expect(...)`. Captura:
`docs/checklists/slice-9-inicio.png`. La zona de arrastre y los dos botones
aparecen sin estilo, con el glifo ASCII `[ > ]`.

### 2. Recientes y recovery

Con un `%APPDATA%` aislado y un `recientes.json` que apunta a
`docs/checklists/slice-7a-proyecto.cdp`, más un `recovery.json` de fixture, la
app arranca con:

```
DEPURACION indice="...\rod-arranque-appdata\RustyOpenDialogue\recientes.json" recientes=1 aviso=true
```

y la captura `docs/checklists/slice-9-recientes.png` muestra:

- entrada «slice-7a-proyecto.cdp» con «2 líneas · 1 hablantes · hace 4 días»;
- aviso «Recuperado hace un momento (entrevista.cdp) — 1 subtítulo» con
  «Recuperar» y «Descartar».

El `eprintln!` de depuración se quitó antes del commit final; la línea de arriba
es evidencia del procedimiento, no código entregado.

**Nota de alcance:** el arrastre real de un archivo y el ciclo guardar → borrar
PNG → regenerar están cubiertos por tests unitarios (`inicio.rs`,
`recientes.rs`, `thumbnail.rs`) y por la lógica cableada en `editor_app.rs`,
pero no se automatizaron con un gesto de ratón. La verificación GUI cubre el
arranque, el listado de recientes y el aviso de recovery.

### 3. Consola solo en release

El PE de los dos binarios dice:

```
debug subsystem=3   (CUI, con consola para desarrollo)
release subsystem=2 (GUI, sin consola)
2=GUI (sin consola), 3=CUI (consola)
```

Viene de `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
en `crates/app/src/main.rs`, igual que el oráculo. El acceso directo del
instalador ya apuntaba al `.exe`, sin `cmd`.

## Decisiones y desvíos

- El recovery pasó de barra en el editor a aviso en el menú (spec §2.4).
- La miniatura se genera al guardar, en el segundo del `playhead`, y se
  regenera al abrir si falta el PNG. Desvío documentado: usa `state.pixels`
  (frame de video), porque los subtítulos se componen en GPU como quads y no
  están en ese buffer. El spec §2.6 decía «video + subtítulos».
- `DescartarRecovery` solo limpia el estado en `inicio`; la escritura en disco
  la hace el enrutador, para que los tests no borren el recovery real.
- `Pantalla::Editor` va en `Box` para evitar el aviso de variante grande de
  clippy; es indirección, no cambio de comportamiento.
- Sin estilo, sin emojis y a 1600×900: el sistema visual es otro subproyecto.
