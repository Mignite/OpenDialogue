# Arranque — propuesta de UX (fase 1, sin estilo)

Origen: agente de diseño con guía `frontend-design`, 2026-11-23.
Estado: propuesta para implementar después de cerrar los hallazgos de la revisión final.
Fase 2 (paleta, tipografía, forma y movimiento) queda para el subproyecto visual.

## Diagnóstico

1. Cinco botones con el mismo peso: no hay acción principal.
2. La zona de arrastre no está delimitada: es texto suelto en mucho vacío.
3. `Recientes` y `Ver todos` se empujan a extremos opuestos; rótulo y filas no se leen como grupo.
4. El error se dibuja siempre (línea vacía reservada) y describe el formato, no el archivo ni el remedio.
5. Recovery compite con las acciones principales y `Descartar` no dice que borra la copia.
6. Verificar desbordamiento al redimensionar antes de atribuirlo al layout.

## Propuesta

Jerarquía: `Abrir video` (y arrastrar) es la única acción destacada; `Abrir proyecto` visible en el mismo renglón sin destacar; recientes son filas silenciosas; `Ver todos` es texto de control; recovery es aviso contextual **encima** de recientes y solo si existe.

- Proporción izquierda/derecha: 2/3 – 1/3 (se conserva).
- Tamaños relativos: encabezado 1.6× cuerpo; ayuda 1.1×; botón principal 1.3× de alto que el secundario; nombre de reciente 1.1×, metadata 0.85×.
- Bloque de acción centrado en la zona izquierda pero subido ~40–45% de la altura.
- Escala de espaciado: 8 · 12 · 24 · 48; entre entradas 12, entre grupos 48, padding 24.
- Estado vacío: `Todavía no hay proyectos. El primero aparece aquí al guardar.` Sin `Ver todos` si hay 0 o ≤3 entradas.
- Error: reemplaza la línea de ayuda, con nombre de archivo y salida; no reserva espacio cuando no hay error.
- Miniatura: fila `[miniatura 16:9] [nombre] [metadata]` con la misma altura de fila; la ruta PNG se resuelve en boot (sin nuevos mensajes ni estado de carga). Placeholder ASCII cuando falte.

Wireframe (1 u = 1/12 del ancho):

```
┌───────────────────────────────────────────────────────────────────────────┐
│ padding 24                                                                │
├───────────────────────────────────────┬───────────────────────────────────┤
│ ≈2/3 ancho                            │ ≈1/3 ancho                        │
│                                       │ [si hay] Quedó sin guardar        │
│                                       │ entrevista.cdp                    │
│                                       │ [Recuperar]  Descartar            │
│                                       │ ── 48 ──                          │
│ ~40% alto                             │ Recientes        Ver todos →     │
│   Empieza con un video (1.6×)         │ ┌───────────┐                     │
│ ┌───────────────────────────────┐     │ │  16:9     │ slice-7a.cdp        │
│ │ Arrastra un video (1.1×)      │     │ │ ≈1/5 col. │ 2 líneas · 1 habl. │
│ └───────────────────────────────┘     │ └───────────┘                     │
│ MP4, MKV, MOV o WebM → ERROR          │ ── 12 ──                          │
│ [ Abrir video ]  Abrir proyecto       │ (vacío: una línea, sin botón)     │
│  ↑ único destacado, 1.3× alto         │                                   │
└───────────────────────────────────────┴───────────────────────────────────┘
```

## Copy literal

| Elemento | Texto |
|---|---|
| Encabezado | `Empieza con un video` |
| Zona | `Arrastra un video aquí` |
| Ayuda | `MP4, MKV, MOV o WebM` |
| Principal | `Abrir video` |
| Secundario | `Abrir proyecto` |
| Rótulo | `Recientes` |
| Expandir | `Ver todos` / `Ver menos` |
| Metadata | `{líneas} líneas · {hablantes} hablantes · {fecha}` (singular: `1 línea`, `1 hablante`) |
| Vacío | `Todavía no hay proyectos. El primero aparece aquí al guardar.` |
| Error | `No se pudo abrir notas.txt. Usa un video MP4, MKV, MOV o WebM.` |
| Recovery | `Quedó sin guardar entrevista.cdp. Recupera para seguir donde lo dejaste.` |
| Descartar | `Descartar borra la copia de recuperación` |

## Criterio de éxito

- 3/3 personas señalan `Abrir video` o la zona de arrastre como primera acción en 5 s.
- Controles con peso principal: de 5 a 1.
- Tiempo al primer clic ≤2 s o −30% respecto a la captura actual.
- Con `.txt`, el error nombra el archivo y las extensiones válidas.
- Con miniatura, identificar el proyecto entre 3 entradas en ≤2 s; sin ella, el placeholder no bloquea el nombre.
- `Ver todos` ausente con 0 recientes; con recovery sigue habiendo un solo control destacado.

## No cambiar

Arrastrar video = proyecto nuevo en memoria; recientes = rutas de `.cdp`; recovery en el menú; `Ver todos` expande la misma columna; miniatura al guardar + regeneración al abrir; sin emojis ni estilo todavía. Sin búsqueda, borrado ni menús nativos.

## Fase 2 — referencias a consultar

- https://m3.material.io/components/buttons/overview
- https://developer.apple.com/design/human-interface-guidelines/buttons
- https://www.nngroup.com/articles/error-message-guidelines/
- https://www.nngroup.com/articles/ten-usability-heuristics/
