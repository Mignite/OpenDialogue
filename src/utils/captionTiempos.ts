import type { Caption } from "../types";

/** Duración mínima de un caption (s). Por debajo el subtítulo no se lee ni
 *  sirve: en .ass el reproductor lo descarta o lo parpadea. */
export const DURACION_MINIMA = 0.1;

/**
 * Calcula inicio/fin válidos al editar un caption a mano.
 *
 * Los tiempos entran de un input de texto, o sea pueden llegar invertidos
 * (inicio > fin), negativos, más allá de la duración del video o ser NaN. Esta
 * función es la única fuente de verdad del clamp, para que el editor numérico
 * y cualquier otra vía (pegar, importar) apliquen las MISMAS reglas.
 *
 * Reglas, en orden:
 *   1. Un NaN se ignora: se conserva el tiempo original (no se destruye el
 *      caption por un typo en el input).
 *   2. El inicio nunca es negativo.
 *   3. Si el inicio pasa al fin, se conserva la DURACIÓN original moviendo
 *      el fin con él (así el borde izquierdo se comporta como el drag, que
 *      mantiene la duración). Nunca se invierte el caption.
 *   4. El fin no pasa la duración del video (si se conoce).
 *   5. Se garantiza una duración mínima.
 */
export function clampCaptionTiempos(
  inicioPedido: number,
  finPedido: number,
  duracionVideo: number,
  actual?: Pick<Caption, "inicio" | "fin">,
): { inicio: number; fin: number } {
  const origInicio = actual?.inicio ?? 0;
  const origFin = actual?.fin ?? 0;
  const duracionOriginal = Math.max(DURACION_MINIMA, origFin - origInicio);

  let inicio = Number.isFinite(inicioPedido) ? inicioPedido : origInicio;
  let fin = Number.isFinite(finPedido) ? finPedido : origFin;

  inicio = Math.max(0, inicio);

  // Si el inicio cruzó al fin: se mantiene la duración (como el drag del
  // borde izquierdo). Si el caption no tiene duración conocida, se usa el
  // mínimo.
  if (inicio >= fin) {
    fin = inicio + (actual ? duracionOriginal : DURACION_MINIMA);
  }

  // Tope por duración del video, solo si se conoce (> 0).
  if (duracionVideo > 0) {
    if (fin > duracionVideo) {
      fin = duracionVideo;
      // Si el inicio quedó fuera, se ancla al final menos la duración mínima.
      if (inicio >= fin) {
        inicio = Math.max(0, fin - DURACION_MINIMA);
      }
    }
  }

  // Duración mínima final. La comparación lleva tolerancia porque en coma
  // flotante `10 + 0.1` da 10.0999999... y sin la tolerancia el caption
  // "ganaba" los 100 ms y quedaba justo por debajo del mínimo (medido).
  if (fin - inicio < DURACION_MINIMA - 1e-9) {
    fin = inicio + DURACION_MINIMA;
    if (duracionVideo > 0 && fin > duracionVideo) {
      inicio = Math.max(0, duracionVideo - DURACION_MINIMA);
      fin = duracionVideo;
    }
  }

  return { inicio, fin };
}