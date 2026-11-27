# libmpv para desarrollo (Windows x64)

No se versiona nada de aquí salvo este README y `mpv.def`: la DLL pesa 116 MB y
el `.lib` es derivado.

## Qué hay

| Archivo | Origen | ¿Al repo? |
|---|---|---|
| `libmpv-2.dll` | paquete `mpv-dev` de shinchiro | no |
| `mpv.lib` | generado con `lib.exe` desde `mpv.def` | **sí** (13 KB, lo necesita el CI de Windows) |
| `mpv.def` | generado desde los exports de la DLL | sí |
| `include/mpv/*.h` | paquete `mpv-dev` | sí (son la referencia del FFI) |

## Procedencia

`mpv-dev-x86_64-20261108-git-55d3feb45c.7z` de
`https://sourceforge.net/projects/mpv-player-windows/files/libmpv/`
(31.7 MB, extraído ~116 MB). Build GPL de shinchiro: sirve para desarrollar, no
para distribuir (spec §3.2, decisión en el slice 8).

## Regenerar el import library

El paquete **no** trae `mpv.def` (trae `libmpv.dll.a`, que es de MinGW y MSVC no
usa). El `.def` se saca de los exports de la DLL, filtrando solo `mpv_*`: la DLL
también exporta símbolos de sus dependencias estáticas (`opus_*`, `sixel_*`,
`Java_java_awt_*`), que no se llaman.

```powershell
$msvc = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64"
& "$msvc\dumpbin.exe" /exports libmpv-2.dll > exports.txt
# de exports.txt, quedarse con las líneas '^\s+\d+\s+[0-9A-F]+\s+[0-9A-F]+\s+(mpv_\S+)$'
# y escribir: LIBRARY libmpv-2.dll / EXPORTS / "  mpv_..."
& "$msvc\lib.exe" /DEF:mpv.def /OUT:mpv.lib /MACHINE:X64
```

Verificar que el import library sirve:

```powershell
& "$msvc\dumpbin.exe" /linkermember:2 mpv.lib | Select-String mpv_render_context_create
# debe aparecer __imp_mpv_render_context_create
```

## Linux

No se usa este directorio: `pacman -S mpv` deja `libmpv.so` y los headers, y
`pkg-config --libs mpv` resuelve el enlace.
