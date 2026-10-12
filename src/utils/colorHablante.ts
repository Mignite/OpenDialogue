import type { Hablante, PresetAss } from "../types";

/** Color efectivo de un hablante: el de su preset .ass, o blanco si no tiene.
 *  `Hablante.color` existe en el tipo solo para leer proyectos viejos — el
 *  formato .ass no tiene noción de color de hablante, así que usar ese campo
 *  haría que la UI dijera un color que el archivo exportado no tiene. */
export function colorHablante(h: Hablante, presets: PresetAss[]): string {
  return presets.find((p) => p.id === h.presetId)?.color ?? "#FFFFFF";
}
