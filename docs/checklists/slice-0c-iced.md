# Slice 0c — iced: componer video y subtítulos en pantalla

Riesgo que cubre: que iced deje subir texturas propias a un `shader::Primitive` y
que el redibujado continuo funcione con la reproducción. Es la prueba de fuego
del slice 0 (spec §8).

Criterio de salida: una ventana que reproduce el video del slice 0a con los
subtítulos del slice 0b encima, y un contador de frames real.

Conceptos nuevos: `shader::Primitive` (`prepare`/`render`), ciclo de vida de una
textura en wgpu, `Subscription` por frame, (re)diseño de `Message`.

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

- [ ] **6. Play/pause y seek.** `enum Message` con `PlayPause`, `Seek(f64)`,
      `Tick`; el video se detiene y arranca sin reiniciar el reloj.
      *Verificación*: pausar y reanudar no salta de frame; el seek cae en el
      frame pedido.

- [x] **7. Medir.** Contador de frames en el título de la ventana durante 30 s de
      reproducción a 1080p.
      *Verificación*: los fps sostenidos. Compáralos con la medición del slice 0a:
      si la diferencia es grande, el costo está en la subida de textura, no en el
      decodificado.
      *Hecho:* 60.0 fps estables a 1920×1080 con un clip fuente de 60 fps (300
      frames). Sin pérdidas: el contador clava la cadencia de la fuente, que es
      el techo real de `mpv_render_context_update`.

- [ ] **8. Cerrar.** Commit con el número de fps y captura de pantalla. Fin del
      slice 0.

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

## Qué NO hacer en este slice

- No empieces con la UI real (paneles, lista, timeline). Este slice es un
  laboratorio: si metes la app aquí, un fallo del video se confunde con un fallo
  de la UI.
- No armes `crates/app` como la app final: un `main.rs` de laboratorio es
  suficiente hasta el slice 4.
- No optimices el redibujado antes del paso 7.
