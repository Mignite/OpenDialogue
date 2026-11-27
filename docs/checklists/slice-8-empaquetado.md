# Slice 8 (parcial) — paquete de Windows con libmpv LGPL

Criterio de salida: existe un instalador que se puede ejecutar en una máquina
limpia (sin vcpkg ni mpv instalados) y la app arranca, abre el video de ejemplo
y no escribe nada en stderr. **Solo Windows**; Linux queda pendiente.

## Decisiones

- **Primer build con LGPL.** El DLL de desarrollo (`third_party/mpv`, shinchiro
  GPL) no se distribuye. El paquete lleva `libmpv-2.dll` del release oficial
  `v0.42.0 x86_64-w64-mingw32-lgpl` de mpv. Enlace dinámico: esto no obliga a
  publicar el código de RustyOpenDialogue bajo GPL.
- **El hash es el contrato.** `packaging/windows/libmpv-lgpl.sha256` fija el
  SHA256 del DLL; `empaquetar.ps1` descarga el ZIP oficial, extrae el DLL,
  compara el hash y falla si no coincide. Si el release cambia, se revisa el
  binario antes de actualizar el hash.
- **Licencias visibles.** El paquete incluye `LICENSE-libmpv-LGPL-2.1.txt`
  (texto íntegro del ZIP) y `THIRD_PARTY_NOTICES.md` con las licencias y
  fuentes de libmpv, FFmpeg LGPL enlazado estáticamente, libass y el resto de
  DLLs. El NSIS muestra la licencia LGPL durante la instalación.

## Pasos

- [x] **1. Comprobar que el DLL LGPL sirve al código actual.** `link_smoke`
  reporta `mpv_client_api_version = 0x20006` con el DLL GPL y con el LGPL: la
  ABI es la misma. `open_video` rasteriza frames no nulos con el LGPL, y la app
  abre el clip y reproduce hasta el final.

- [x] **2. Script de empaquetado.** `packaging/windows/empaquetar.ps1` parte del
  `cargo build --release -p rod-app`, descarta el DLL GPL que copia `build.rs`,
  mete el LGPL verificado, las diez DLLs de libass y sus dependencias, el clip
  de ejemplo, los avisos de licencia y un `LEEME.txt`. Sale un ZIP en
  `target/release/`.

- [x] **3. Instalador NSIS.** `packaging/windows/instalar.nsi` instala en
  `$LOCALAPPDATA\RustyOpenDialogue` sin pedir administrador (mismo
  `installMode: currentUser` que el oráculo), crea el acceso directo y el
  desinstalador. Se compila con:

  ```powershell
  makensis /DSTAGING=<staging> /DOUTFILE=<setup.exe> packaging\windows\instalar.nsi
  ```

- [x] **4. Probar la instalación de verdad.** Instalación silenciosa con
  `/S /D=<carpeta temporal>`, PATH sin vcpkg ni `third_party/mpv`, ejecutar
  `rod-app.exe` desde la carpeta instalada, esperar a que aparezca el título,
  capturar la ventana y desinstalar.

## Evidencia

- `empaquetar.ps1` (recortado):

  ```
  libmpv LGPL verificado: 94fb87247732c59915cd59808bc582e843a71c22214b7b6c1e5563aa8149cef4
  staging: C:\Users\migue\AppData\Local\Temp\rod-paquete\staging
  paquete: C:\Dev\RustyOpenDialogue\target\release\RustyOpenDialogue-0.1.0-win-x64.zip
  tamano: 73.6 MB
  ```

- `makensis` (recortado):

  ```
  Install data: 156531162 bytes
  Compressed data: 51550055 / 156542114 bytes
  Total size: 51587435 / 156581030 bytes (32.9%)
  ```

  El instalador pesa 49,2 MB (el ZIP, 73,6 MB: NSIS comprime mejor).

- Instalación silenciosa:

  ```
  installer exit: 0
  testdata, ass-9.dll, …, libmpv-2.dll, LICENSE-libmpv-LGPL-2.1.txt,
  rod-app.exe, THIRD_PARTY_NOTICES.md, Uninstall.exe, z.dll
  libmpv hash: 94fb87247732c59915cd59808bc582e843a71c22214b7b6c1e5563aa8149cef4
  viva (id 21616) ventana=2425622 titulo=Rusty Open Dialogue
  ```

  stderr vacío. Tras `Uninstall.exe /S`: carpeta borrada y acceso directo del
  menú de inicio borrado.

- `docs/checklists/slice-8-instalado.png` y `slice-8-instalador.png`: la app
  ejecutada desde el ZIP extraído y desde el instalador, con video, carriles y
  controles visibles. La pestaña inicial es «Subtítulos»: el binario de
  release está compilado del código entregado (el binario de debug seguía con
  la pestaña sembrada de 7d y por eso la primera prueba mostraba «Estilos»).

## Hallazgos

- **El DLL GPL y el LGPL tienen la misma API pero no son el mismo binario.**
  `libmpv-2.dll` pasó de 121.627.648 a 125.790.720 bytes. El `mpv.lib` del repo
  sirve para los dos (solo exporta `mpv_*`), así que no hay que regenerar el
  import library ni tocar el FFI: el cambio es del DLL que se empaqueta.
- **La ventana tarda en aparecer.** La primera captura, a los 8-10 s, mostró la
  ventana oculta de 16×16 de winit y el título vacío; con ~20 s ya estaba la
  ventana completa. El chequeo espera al título, no a un `Sleep` fijo corto.
- **`choco install nsis` falló por permisos** (`Acceso denegado a
  'C:\ProgramData\chocolatey\lib-bad'`). No se instaló nada en el sistema: se
  bajó el paquete `nsis.portable` como nupkg y se extrajo `makensis.exe` en
  `%TEMP%`. El NSIS no forma parte del repo ni del PATH.
- **El repo sigue sin `LICENSE` propio.** Elegir LGPL para libmpv no elige la
  licencia del código de RustyOpenDialogue; eso queda para cuando se publique
  el primer release.

## Qué NO hacer todavía

- Nada de `.deb`/AppImage: no hay forma de probarlos desde esta máquina y el
  spec los pide con DLLs incluidas.
- Nada de firmar el instalador, subirlo a GitHub Releases ni añadir un job de
  empaquetado al CI: primero hay que decidir el canal de distribución.
- Nada de «instalar mpv»: el paquete lleva su libmpv LGPL y el usuario no debe
  instalar nada más.
