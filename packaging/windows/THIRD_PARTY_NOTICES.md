# Terceros incluidos en el paquete de Windows

La app se distribuye con **libmpv LGPL**, no con el build GPL que se usa para
desarrollar (`third_party/mpv/README.md`). Enlace dinámico: el código de
RustyOpenDialogue puede tener la licencia que decida su autor; las obligaciones
de este archivo son las de libmpv y sus dependencias.

| Componente | Licencia | Fuente |
|---|---|---|
| `libmpv-2.dll` (mpv v0.42.0 `x86_64-w64-mingw32-lgpl`, compilado con FFmpeg LGPL) | LGPL-2.1-or-later | https://github.com/mpv-player/mpv/tree/v0.42.0 |
| FFmpeg (enlazado estáticamente dentro de libmpv, variante LGPL) | LGPL-2.1-or-later / LGPL-3.0 según componente | https://ffmpeg.org/download.html |
| `ass-9.dll` (libass, vcpkg) | ISC | https://github.com/libass/libass |
| `freetype.dll` | FTL / GPLv2 dual | https://freetype.org |
| `fribidi-0.dll` | LGPL-2.1-or-later | https://github.com/fribidi/fribidi |
| `harfbuzz.dll` | MIT | https://github.com/harfbuzz/harfbuzz |
| `iconv-2.dll` | LGPL-2.1-or-later | https://www.gnu.org/software/libiconv/ |
| `libpng16.dll` | zlib | http://www.libpng.org/pub/png/libpng.html |
| `brotli*.dll`, `bz2.dll`, `z.dll` | MIT / BSD / zlib | https://github.com/google/brotli, https://sourceware.org/bzip2/, https://zlib.net |

El paquete incluye el texto íntegro de la LGPL-2.1 en
`LICENSE-libmpv-LGPL-2.1.txt`, tal como viene en el ZIP oficial de mpv. Si se
modifica libmpv, hay que ofrecer el código fuente de esa modificación; para el
build sin modificar basta el enlace a la versión etiquetada de arriba.

El clip de ejemplo (`testdata/clip-15s-2audio.mp4`) se genera con ffmpeg para
las pruebas del repo; no es material de terceros con licencia aparte.
