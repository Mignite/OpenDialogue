# Slice 0c — iced: componer video y subtítulos en pantalla

Riesgo que cubre: que iced deje subir texturas propias a un `shader::Primitive` y
que el redibujado continuo funcione con la reproducción. Es la prueba de fuego
del slice 0 (spec §8).

Criterio de salida: una ventana que reproduce el video del slice 0a con los
subtítulos del slice 0b encima, y un contador de frames real.

Conceptos nuevos: `shader::Primitive` (`prepare`/`render`), ciclo de vida de una
textura en wgpu, `Subscription` por frame, (re)diseño de `Message`, y el estado
del player visto desde la UI (`pause`, `time-pos`, `aid`).

## Pasos

- [x] **1. Ventana y `Canvas`.** Ventana de iced con un `Canvas` que dibuja 1000
      barras verticales de alturas pseudoaleatorias con semilla fija.
      *Verificación*: se ven las barras, y al redimensionar se reescalan sin
      deformarse (el layout del canvas se está midiendo bien).
      *Cerrado en el slice 0z.*

- [x] **2. Una textura estática.** Un `shader::Primitive` que dibuja un quad con
      una imagen PNG cargada con `image`.
      *Verificación*: el PNG aparece con las proporciones correctas. Si sale
      volcado, el problema está en las UV, no en la textura.
      *Omitido a propósito:* `image-without-codecs` no trae decodificador de PNG y
      añadir la feature completa no valía para un paso de andamiaje. El paso 4
      valida lo mismo (y más estricto) con el video real, sin dependencias nuevas.

- [x] **3. Textura dinámica.** Reemplazar el PNG por un `Vec<u8>` que cambia cada
      50 ms (un degradado que se desplaza), subido en `prepare` con
      `queue.write_texture`.
      *Verificación*: el degradado se mueve y no parpadea. Si parpadea, se está
      recreando la textura en vez de reescribirla.
      *Hecho:* textura 1920×1080 creada en `Pipeline::new` (recreada solo si
      cambia el tamaño) y reescrita con `write_texture` en `prepare`.

- [x] **4. El frame de mpv.** Sustituir el degradado por `player::poll_frame()`.
      *Verificación*: se ve el video. Compara el instante que muestra la ventana
      con `mpv` en el mismo `time-pos`.
      *Hecho:* el video se ve con el timecode quemado cuadrando. El quad escala
      la posición y no la uv, así que conserva la proporción (letterbox por
      uniform); el alfa se fuerza a 1.0 en el fragment en vez de recorrer 2
      millones de píxeles en CPU.

- [x] **5. Los bitmaps de libass.** Añadir N quads con blending sobre el de video,
      usando el render del slice 0b al `time_pos` actual.
      *Verificación*: los subtítulos coinciden con el PNG de referencia del slice
      0b y con el oráculo de mpv. Es el invariante, ahora en pantalla.
      *Hecho:* un segundo pipeline en el mismo pase dibuja las capas como
      instancias (rect en clip space + uv en atlas + color); el atlas es una
      textura `R8Unorm` de cobertura, así que el color no se duplica por capa.
      El color del `.ass` se convierte de sRGB a lineal en el fragment porque el
      target es sRGB y el blending lo hace el hardware en lineal (ver hallazgos).
      Captura: `slice-0c-paso-5.png` (60.0 fps, 5 capas, mpv 12.0 ms, libass
      0.2 ms, texto y colores correctos, letterbox intacto).

- [x] **6a. El esqueleto de la ventana.** La estructura de la app Tauri, en
      mínimo: columna izquierda (video + transporte), panel derecho, timeline
      abajo, barra de estado. El video ya no llena la ventana, así que el
      letterbox se pone a prueba de verdad.
      *Verificación*: a 1280×720 el video queda 768×432 con barras negras arriba
      y abajo; a 1600×620 queda con barras a los lados; los subtítulos caen
      dentro del video y escalan con él. 60.0 fps, 5 capas.
      *Hecho:* no hizo falta tocar la geometría. `iced_wgpu` pone un **viewport
      por primitivo** (`set_viewport(bounds…)` en `lib.rs`), así que NDC −1..1 ya
      es el rect del widget y el letterbox por uniform era correcto desde el
      paso 4. Lo único que se añadió fue el esqueleto de widgets y un comentario
      en `frame.wgsl` para que nadie "arregle" lo que funciona.

- [x] **6b. Los controles del reproductor, juntos: play/pause, seek y pista de
      audio.** `enum Message { PlayPause, Arrastrar(f32), Soltar, Step(f64),
      AudioTrack(pista), Tick }`, y un `Row` en la barra de transporte que ya
      existe: botón play/pause, slider de posición y `pick_list` de pistas. El
      video se detiene y arranca sin reiniciar el reloj; el seek cae en el frame
      pedido; y el audio suena por la pista elegida, una a la vez (`aid`).
      *Verificación*: pausar y reanudar no salta de frame; el seek cae en el frame
      pedido; cambiar de pista cambia el tono (440 ↔ 880 Hz) sin cortar el video.
      Los tres se prueban en la misma corrida, porque leen el mismo estado del
      player: por separado no dicen nada.
      *Fixture*: `testdata/clip-2audio.mp4`, cinco segundos, dos pistas AAC con
      idioma y nombre distintos (se regenera con
      `ffmpeg -f lavfi -i testsrc2=size=1920x1080:rate=60 -f lavfi -i sine=frequency=440:sample_rate=48000 -f lavfi -i sine=frequency=880:sample_rate=48000 -t 5 -map 0:v -map 1:a -map 2:a -c:v libx264 -pix_fmt yuv420p -c:a aac -metadata:s:a:0 language=eng -metadata:s:a:0 title="Tono 440 Hz" -metadata:s:a:1 language=spa -metadata:s:a:1 title="Tono 880 Hz" testdata/clip-2audio.mp4`).
      *Hecho* (`0535a91`): medido con `cargo run -p rod-player --example
      controles` sobre `testdata/clip-15s-2audio.mp4` —

      | comprobación | medida |
      |---|---|
      | seek exacto, pedido 7.370 s | quedó en 7.371 s |
      | pausa congela el reloj | 0.050 → 0.050 s en 500 ms |
      | cambio de pista | `aid` 1 → 2, mpv lo reporta activo |
      | salida de audio | `wasapi` (antes `ao=null`) |

      Y en la ventana real (rod-app), confirmado a ojo: pausar congela el timecode
      y reanudar sigue desde el mismo instante, `−5 s` retrocede y sigue, y el
      selector `eng` → `spa` cambia el tono sin cortar el video.

      *Lo que hubo que añadir al player*, y el porqué de cada uno:
      - `hr-seek=yes`: sin él mpv aterriza en el keyframe anterior y el playhead
        miente. Es lo único que hace que "el seek cae en el frame pedido" sea
        cierto y no aproximado.
      - `keep-open=yes`: al llegar al final se queda en el último frame en vez de
        cerrar el stream. Un editor sin imagen al final del clip no sirve.
      - `esperar_carga` acotada a 5 s: el `wait_event(-1.0)` anterior podía
        bloquear el hilo de la UI para siempre si no llegaba `FileLoaded`.
      - `seeking()`: entre pedir el seek y que mpv lo ejecute, `time-pos` sigue
        en el instante viejo 1–2 frames. Sin esta guarda el thumb del slider salta
        hacia atrás y pelea con el dedo que arrastra.
      - `audio_track()`: el selector de audio y el botón de pausa releen el
        estado de mpv en vez de invertir un espejo local. mpv es la fuente; el
        espejo solo sirve para pintar.

      *Desviaciones conscientes del oráculo* (`C:/Dev/OpenDialogue`):
      - La app Tauri no tiene slider: el seek viene del timeline o de escribir el
        tiempo en un input. Aquí el slider es del paso 6 y se queda, con
        `on_change` + `on_release` (mover el thumb no busca; soltarlo sí).
      - El timecode del oráculo es `HH:MM:SS.cc` (`utils/time.ts`, centésimas) y
        solo muestra la posición, editable. El nuestro usa el mismo formato y
        añade `posición / duración`, que el oráculo pone en el timeline.
      - **Pendiente 6c**: el selector de velocidad. El oráculo ofrece
        `[0.5, 0.75, 1, 1.25, 1.5, 2]` vía `<select>`; aquí había un `1x` de
        mentira y se quitó antes que dejarlo como control muerto.


- [x] **7. Medir.** Contador de frames en el título de la ventana durante 30 s de
      reproducción a 1080p.
      *Verificación*: los fps sostenidos. Compáralos con la medición del slice 0a:
      si la diferencia es grande, el costo está en la subida de textura, no en el
      decodificado.
      *Hecho:* 60.0 fps estables a 1920×1080 con un clip fuente de 60 fps (300
      frames). Sin pérdidas: el contador clava la cadencia de la fuente, que es
      el techo real de `mpv_render_context_update`.

- [x] **8. Cerrar.** Commits `8154b76` (controles del player), `7537eab`
      (esqueleto de ventana, paso 6a) y `0535a91` (los tres controles juntos en la
      app, paso 6b) dejan el slice 0 cerrado: **60.0 fps** con **5 capas** de libass
      y el reproductor con play/pause, seek exacto y pista de audio. La captura de
      la medición es `docs/checklists/slice-0c-paso-5.png`.

## Hallazgos

- **El widget `image` no sirve para video.** `iced_wgpu` solo sube una imagen de
  forma síncrona si mide menos de 2 MiB (`image/cache.rs`, `MAX_SYNC_SIZE`); un
  frame de 1080p son 8.29 MB y va al hilo worker, que no dibuja nada en ese
  frame. Con un `Handle` nuevo por frame, `trim()` descarta la textura antes de
  que se use: gris y 18 fps. Detalle en `BITACORA.md` (2026-11-13).
- **El techo de fps es la fuente, no el render.** `mpv_render_context_update`
  solo marca frame nuevo cuando hay uno decodificado, así que el contador no
  puede pasar de la cadencia del clip. Para medir holgura hace falta una fuente
  más rápida: `testdata/clip-1080p60.mp4`, generado con
  `ffmpeg -f lavfi -i testsrc2=size=1920x1080:rate=60 -t 5 ...`.
- **El área del letterbox es el fondo del tema.** Para un reproductor debería ser
  negra; pendiente para la UI real (slice 4).
- **libass necesita warm-up.** La primera rasterización inicializa el proveedor de
  fuentes y cachea los glifos: sin una llamada de calentamiento en la
  construcción, el primer segundo cae a ~13 fps con 15.5 ms de media en libass.
  Después queda en 0.1–0.4 ms por frame con 5 capas. Una sola llamada a
  `render(ms)` al crear el `Subtitulos` lo resuelve.
- **El blending de los subtítulos ocurre en espacio lineal.** El target es sRGB y
  la GPU mezcla en lineal, mientras que la composición de CPU del slice 0b y mpv
  mezclan en gamma. Los colores sólidos coinciden; los bordes suavizados (AA)
  difieren por décimas. Es una desviación conocida, no un bug: el invariante
  preview == export es contra el render de libass, que sí es el mismo.
  Convertir el color a lineal en el fragment (en vez de dejar que el hardware lo
  haga) es lo que mantiene los colores puros exactos.
- **El atlas se rehace por frame.** Las máscaras que devuelve libass solo valen
  hasta el siguiente `ass_render_frame`, así que hay que copiarlas. Se empaquetan
  en una sola columna y la textura solo se recrea si cambia el tamaño; el
  contenido se reescribe. Con 5 capas no se nota, pero es el punto a revisar si
  algún día hay decenas de carteles simultáneos.
- **El example todavía clona el frame.** `Arc::new(lab.buf.clone())` copia 8 MB
  por frame; con la GPU yendo a 60 fps no aparece en la medición, pero sobra.
  Pendiente: doble buffer o `Arc` reciclado.
- **El audio es de mpv, pero estaba apagado.** El slice 0 arrancó con `ao=null`
  para no mezclar sonido con la medición de video. El player real no lo lleva:
  quitar esa línea y mpv decodifica y sincroniza sus pistas sin código nuestro.
- **`aid` es de un solo valor** (`auto`/`no`/número), así que el selector es un
  `enum` de N variantes y no una lista de casillas. Para oír varias pistas a la
  vez mpv exige abandonar `aid` y montar `lavfi-complex` con `amix`, que además
  se lleva por delante la selección normal y el volumen por pista. Se descarta
  (decisión en la spec §4.3): para subtitular estorba oír dos diálogos a la vez.
- **NDC es el widget, no la ventana.** `iced_wgpu` recorre las primitivas y, por
  cada una, hace `set_viewport(bounds…, 0.0, 1.0)` + `set_scissor_rect`
  (`iced_wgpu-0.14.0/src/lib.rs`, ~línea 551). O sea que el clip space −1..1 cae
  sobre el rectángulo del widget y no hay que trasladar nada a mano. Darlo por
  hecho al revés (creer que el quad se centraba en la ventana) hace que se
  "arregle" algo que funcionaba: el video sale escalado por
  `widget/ventana` —0.6 en x, 0.7 en y— pegado a la esquina. La comprobación
  barata fue pintar el fragment de magenta: el bbox del quad delata la geometría
  sin depender del contenido del video.
- **`PrintWindow` miente con una ventana wgpu ocluida.** Devuelve el último frame
  compuesto, así que dos capturas separadas 800 ms salen idénticas y parecen una
  app congelada: primero culpé a mi guarda de `seeking()`, después a mpv. La app
  estaba a 11.8 s de CPU en 9 s de reloj, o sea vivita. Para verificar
  reproducción por píxeles sirve `CopyFromScreen` con la ventana al frente
  (`ShowWindow(SW_RESTORE)` + `SetForegroundWindow` + `MoveWindow` a una posición
  conocida); `PrintWindow` solo para capturar la geometría de un frame quieto.
  Y cuando el resultado importa, un ojo humano es más barato: los tres controles
  de 6b se confirmaron así después de una hora peleando con la captura.
- **El transporte del oráculo no tiene slider.** `App.tsx` pone `saltar(-5)` ·
  play/pause · `saltar(+5)` · input de tiempo editable · `<select>` de velocidad
  (`VELOCIDADES_REPRODUCCION = [0.5, 0.75, 1, 1.25, 1.5, 2]`), y el seek real
  viene de arrastrar en el timeline o escribir el tiempo. Nuestro slider es una
  adición del paso 6, no una copia; el formato del timecode (`HH:MM:SS.cc`,
  centésimas) sí es el mismo (`utils/time.ts`).

## Qué NO hacer en este slice

- No empieces con la UI real (paneles, lista, timeline). Este slice es un
  laboratorio: si metes la app aquí, un fallo del video se confunde con un fallo
  de la UI.
- No armes `crates/app` como la app final: un `main.rs` de laboratorio es
  suficiente hasta el slice 4. La única excepción es la barra de transporte del
  paso 6: se queda en el example (aunque su `Row` se pueda copiar tal cual al
  slice 3), no se muda a la app todavía.
- No optimices el redibujado antes del paso 7.
