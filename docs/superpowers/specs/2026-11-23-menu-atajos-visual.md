# Menú, atajos y sistema visual (subproyectos 2 y 3)

Fecha: 2026-11-23
Estado: aprobado para ejecución («completa los 3»)
Especificaciones relacionadas: `2026-11-23-arranque-design.md`, `2026-11-23-arranque-ux.md`
Oráculo: `C:/Dev/OpenDialogue`

## 1. Objetivo

Tres cosas a la vez, con una sola fuente de verdad para las acciones:

1. **Menú** dibujado por la app, visible en arranque y editor, que portea el
   menú del oráculo (`Archivo`, `Exportar`, `Estilos`) y añade `Edición`, `Ver`
   y `Ayuda`. Reduce el clutter del editor porque las acciones viven bajo el
   título, como en PhotoCraft.
2. **Atajos** con paridad del oráculo, mapeados al mismo comando que el menú.
3. **Sistema visual** swiss/brutalista con acento mostaza sobre negro.

Criterio de éxito: menú y teclado ejecutan exactamente la misma acción; los
atajos del oráculo funcionan; la UI se ve con el sistema visual acordado sin
perder los colores semánticos de los hablantes; build, fmt, clippy y tests en
verde en Windows y compilación en Linux.

Fuera de alcance: búsqueda, borrado de proyectos, menú nativo del SO,
asociación de extensión, keybinds globales fuera de la ventana.

## 2. Decisiones

- **Menú dibujado por la app**, no `muda`: el usuario eligió consistencia
  visual y el menú nativo no se puede estilizar. `iced_aw` 0.14.1 con feature
  `menu` aporta `MenuBar`; es la única dependencia nueva. Sin `unsafe`.
- **Un registro de comandos**: `enum Comando` con etiqueta, atajo textual y
  disponibilidad por pantalla. El menú se construye desde él, el teclado
  mapea a él y un único `ejecutar` lo despacha. Motivo: el bug del `seek` se
  produjo por duplicar la apertura en dos caminos.
- **iced posee los atajos**: `keyboard::listen` es el único productor de
  comandos por teclado. El menú solo muestra el atajo como texto; no lleva
  aceleradores nativos (en Win32 exigirían `TranslateAcceleratorW`).
- **Contenido**: porteo del menú del oráculo más `Edición` con los atajos. Se
  mantiene Ctrl+O como añadido de la versión Rust, documentado como desvío.
- **Colores**: el mostaza es el acento de la chrome; los colores de hablante
  son semánticos (identifican track en timeline y ASS) y no se sustituyen.
- **Alcance visual**: arranque + editor, incluida la barra de menú.

## 3. Arquitectura

```
crates/app/src/
  comandos.rs   enum Comando + etiqueta + atajo + disponibilidad + mapeo de teclado
  menu.rs       construye MenuBar desde el registro de comandos
  tema.rs       tokens mostaza/negro/hairline + estilos de widgets
  main.rs       ejecutar(Comando, &mut Pantalla) -> Task<Mensaje>
  editor_app.rs MensajeEditor::Ejecutar(Comando) y mapeo a sus manejadores
  inicio.rs     vista con barra de menú y botón principal destacado
```

`Comando` es la interfaz entre hombre, menú y teclado. Ningún módulo define
atajos por su cuenta.

## 4. Registro de comandos (oráculo + añadido)

Archivo: `Nuevo proyecto`, `Abrir proyecto`, `Guardar proyecto`,
`Guardar como...`, `Abrir video`, `Cargar SRT`, `Cargar .ass`,
`Importar auto-subs (SRT+TXT)`.
Exportar: `Exportar SRT por hablante`, `Exportar JSON combinado`, `Exportar .ass...`.
Estilos: `Estilos...`.
Edición: reproducir/pausa (Space), caption anterior/siguiente (Alt+←/→),
±0,1 s (Shift+←/→), ±5 s (←/→), nudge selección (Ctrl+Shift+←/→), editar
texto (E), caption simultáneo arriba/abajo (↑/↓), añadir fragmento (A),
eliminar (Delete), deshacer/rehacer (Ctrl+Z/Y), copiar/pegar (Ctrl+C/V),
dividir en playhead (C), ajustar velocidad (+/−), buscar (Ctrl+F), ayuda (?),
hablantes 1..9.
Ver: alternar paneles. Ayuda: atajos.

Atajos que el oráculo procesa **antes** del guardia de `INPUT`: Ctrl+S y
Ctrl+F. El resto no debe dispararse mientras se escribe en un `text_input`,
salvo Ctrl+Z/Y/C/V, que se dejan al comportamiento nativo del campo.

## 5. Sistema visual

Tokens: superficie `#0C0C0D`, panel `#141416`, hairline `#FFFFFF26`, texto
`#F3F3F3`, secundario `#9A9A9A`, acento mostaza `#D9A441`. Sin gradientes,
sin sombras, esquinas 3 px, rejilla 2:1 en el arranque, escala de espaciado
8/12/24/48. Display pesado y apretado, cuerpo neutro, etiquetas monoespacio
en mayúsculas. Cero emojis: glifos ASCII o SVG.

El color de cada hablante (`Hablante::color`) sigue mandando en su carril del
timeline, en la lista y en el ASS exportado. El mostaza solo tiñe la chrome:
botón principal, item activo del menú, foco.

## 6. Testing y verificación

- Tests puros: `desde_keyboard` para cada atajo del oráculo; dedup de
  comandos; disponibilidad por pantalla; `Comando` → etiqueta/atajo.
- Test de integración: pulsar un atajo y pulsar el item de menú producen el
  mismo `Comando` y el mismo efecto en `Pantalla`.
- Verificación observable: menú visible en ambas pantallas; Ctrl+S guarda;
  los atajos funcionan; capturas del arranque y del editor con el nuevo
  sistema visual; `cargo fmt --check`, `clippy -D warnings`, `cargo test`.
