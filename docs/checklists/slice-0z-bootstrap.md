# Slice 0z — bootstrap: workspace de cuatro crates y una ventana

Riesgo que cubre: que la estructura del repo se convierta en un problema
después (un crate mal nombrado, dependencias cruzadas, un binario que no es el
que crees). Es barato ahora y caro en el slice 5.

Criterio de salida: `cargo run -p rod-app` abre una ventana de iced con una
figura dibujada en `Canvas`; `cargo fmt --check`, `cargo clippy --all-targets
-- -D warnings` y `cargo test` en verde. Nada de libmpv ni libass: eso es 0a y
0b.

Decisiones ya tomadas (spec §4):

- Manifiesto virtual en la raíz: `Cargo.toml` solo tiene `[workspace]`; el
  binario vive en `crates/app`.
- Paquetes `rod-core`, `rod-subs`, `rod-player`, `rod-app`; directorios
  `crates/core`, `crates/subs`, `crates/player`, `crates/app`.
- El paquete **no** puede llamarse `core`: tapa el `core` del sysroot y rompe
  `core::time::…` y toda macro derivada que expanda `::core::…` (comprobado con
  `thiserror::Error`; ver spec §4).

Conceptos nuevos, uno por paso: paquete vs workspace, herencia de metadatos,
dependencia por ruta, arquitectura Elm de iced, `canvas::Program`.

## Paso 0 — Árbol limpio

- [x] La raíz hoy arrastra cambios sin commitear: `Cargo.toml` con `iced`
  añadido, `Cargo.lock` sin seguimiento y `src/main.rs` con el `Hello, world!`
  de la plantilla. Commitea ese estado (o descártalo) antes de reestructurar:
  si no, el diff del slice mezcla lo viejo con lo nuevo y no se puede leer.

  *Verificación*: `git status --short` vacío.

## Paso 1 — Manifiesto virtual y cuatro crates vacíos

- [x] Borra el `[package]` de la raíz y deja solo `[workspace]` con `members`
  (puedes usar el glob `"crates/*"`) y `resolver = "3"`.
  **Concepto**: un *virtual manifest* no es un paquete, es la lista de miembros.
  **Trampa**: `resolver` no se hereda del `edition` de los miembros; si lo
  omites, un workspace virtual se comporta como resolver 1 y las features se
  unifican como en 2015.
- [x] Crea los cuatro crates con `cargo new --lib crates/<dir> --name rod-<x>`
  (`--name` es lo que desacopla el nombre del directorio). `crates/app` es el
  único sin `--lib`.
- [x] Borra `src/main.rs` y el directorio `src/` de la raíz.

*Verificación*: `cargo build` compila los cuatro sin warnings; `cargo
metadata --no-deps --format-version 1` lista exactamente `rod-core`, `rod-subs`,
`rod-player`, `rod-app` con `edition: "2024"`.

## Paso 2 — Una sola fuente para versión y edition

- [x] Pasa `version`, `edition` y `rust-version` a `[workspace.package]`, y cada
  crate los hereda con `edition.workspace = true` (etc.).

  **Concepto**: `[workspace.package]` es una plantilla, no un padre. Ningún
  miembro hereda nada sin pedirlo con `campo.workspace = true`; es explícito a
  propósito, porque cada crate es un paquete independiente para Cargo.
  **Trampa 1**: solo ciertas claves se pueden heredar (`version`, `edition`,
  `rust-version`, `license`, `authors`, `repository`, …). `name` no: el nombre
  de un crate nunca se hereda.
  **Trampa 2**: si pides `license.workspace = true` y la clave no existe en
  `[workspace.package]`, Cargo falla con `` `workspace.package.license` was not
  defined ``. Heredar una clave inexistente no es "nada", es error.
  **Trade-off**: compartir `version` obliga a versionar los cuatro crates juntos.
  Es lo correcto ahora; si algún día `rod-core` se publica solo, deja de heredar.

*Verificación*: `cargo metadata --no-deps` muestra `rust_version: "1.88"` en los
cuatro; borra temporalmente `rust-version` del workspace y comprueba el error,
luego devuélvela.

## Paso 3 — `rod-app` es el único binario

- [x] `crates/app/src/main.rs` con un `fn main()` que imprime algo. Declara
  `rod-core` en `[workspace.dependencies]` como dependencia por ruta
  (`{ path = "crates/core" }`), y en `rod-app` pídelo con
  `rod-core.workspace = true`. Desde `main`, llama a una función pública de
  `rod-core` (un `&'static str` basta; se borra en el paso siguiente).

  **Concepto**: `[workspace.dependencies]` no es solo para crates.io; también
  sirve para dependencias por ruta. La ruta se escribe relativa a la **raíz**
  del workspace, no al crate que la usa.
  **Concepto 2**: `rod-core` → `rod_core` en `use` (guion a guion bajo).
  **Trampa**: no añadas todavía `rod-subs` ni `rod-player` a `rod-app` "por si
  acaso". Una arista que no se usa cuesta compilación y arrastra las features
  del otro crate.

*Verificación*: `cargo run -p rod-app` imprime la cadena que viene de
`rod-core`; `cargo tree -p rod-app` muestra `rod-core` y nada más.

## Paso 4 — La ventana

- [x] Declara `iced = "0.14.0"` en `[workspace.dependencies]` y pídelo en
  `rod-app` con `iced.workspace = true`. Sustituye el `println!` por una
  aplicación de iced: `fn main() -> iced::Result`,
  `iced::application(boot, update, view).run()`, un `struct` de estado y un
  `enum Message` (aunque no haya mensajes todavía).

  **Concepto**: arquitectura Elm. `update(&mut State, Message)` es la única
  función que muta el estado; `view(&State) -> Element` es pura. El estado es
  tuyo, no de iced.
  **Trampa 1**: `Message` debe ser `Send + 'static`; y si `update` devuelve `()`
  en vez de `Task`, no puedes encadenar trabajo asíncrono. Empieza con `()`,
  cámbialo cuando lo necesites (slice 3).
  **Trampa 2**: con `iced.workspace = true` **no** puedes escribir
  `default-features = false` en el miembro; eso solo se decide en
  `[workspace.dependencies]`. Las `features` sí puedes añadirlas en el miembro.

*Verificación*: `cargo run -p rod-app` abre una ventana con el título que
pusiste y se cierra sin error (código de salida 0).

## Paso 5 — `Canvas`

- [x] Activa la feature `canvas` en la dependencia de iced (en
  `[workspace.dependencies]`: `features = ["canvas"]`). Implementa
  `canvas::Program<Message>` para una struct propia y dibuja una figura
  (un círculo o una línea) dentro de `draw`.

  **Concepto**: `draw` recibe `bounds: Rectangle` y devuelve geometría; el
  tamaño del lienzo llega *por argumento*, no se guarda en el estado.
  **Trampa 1**: `canvas` **no** es una feature por defecto de iced; sin ella el
  import falla con `E0432 unresolved import`.
  **Trampa 2**: la firma de `draw` incluye `renderer`, `theme` y `cursor`
  aunque no los uses; si te sobra alguno, el trait no compila. Prefija con `_`
  y listo.

*Verificación*: la ventana muestra la figura; al redimensionar, la figura se
recalcula con el nuevo tamaño (si se deforma o se sale, estás guardando
coordenadas en el estado).

## Paso 6 — El semáforo del slice

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo test --workspace`

*Verificación*: los tres terminan en 0. `--all-targets` incluye tests y
ejemplos: clippy que solo mira la lib deja pasar lo que más se rompe. Sin
`--all-features`: activaría las features experimentales de iced (`debug`, `hot`)
y estarías depurando el toolkit, no tu código.

## Paso 7 — CI

- [x] `.github/workflows/ci.yml`: matriz `windows-latest` / `ubuntu-latest`,
  pasos `fmt --check`, `clippy -D warnings`, `test`.

  *Hecho*: primer run 43243883199 en verde, windows 3m41s y ubuntu 2m28s. Los
  paquetes de sistema para compilar iced en Linux fueron
  `libxkbcommon-dev libwayland-dev libx11-dev libxcb1-dev libxrandr-dev
  libxi-dev libxcursor-dev`.

  **Aviso**: el repo aún no tiene remoto (`git remote -v` está vacío), así que
  el workflow no corre hasta que exista un repositorio en GitHub. El archivo se
  escribe igual; la primera ejecución es la verificación.
  **Nota para 0a/0b**: cuando entren libmpv y libass, el job de Linux necesitará
  `sudo apt install libmpv-dev libass-dev` (o los paquetes equivalentes) y el de
  Windows las DLLs de `third_party/`. Eso se toca en esos slices, no aquí.

*Verificación*: el archivo es YAML válido y los tres comandos que ejecuta son
exactamente los del paso 6.

## Qué NO hacer en este slice

- No enlaces libmpv ni libass: 0a y 0b.
- No añadas `thiserror`, `serde` ni `anyhow` "para dejarlo listo". Cada crate
  estrena dependencia cuando el paso la necesita.
- No pongas `#![deny(warnings)]` en el código: eso es del CI, donde se puede
  actualizar el toolchain sin romper tu build local.
- No diseñes todavía el `VideoSource` de la spec §4.3: en 0z no hay player.
