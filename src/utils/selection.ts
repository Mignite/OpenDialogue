import type { Caption, Hablante } from "../types";

// Fila del carril de un caption: 0 = sin asignar ("—"), luego 1 por hablante.
export function captionRowIndex(
  hablanteId: string | null | undefined,
  hablantes: Hablante[],
): number {
  if (!hablanteId) return 0;
  const idx = hablantes.findIndex((h) => h.id === hablanteId);
  return idx === -1 ? 0 : idx + 1;
}

// Corrimiento relativo de carriles para el body drag en bloque: cada clip
// conserva su offset respecto al lead. shift = filaDestinoLead -
// filaOrigenLead; el destino se clampéa a [0, maxFila] (0 = sin asignar).
// Solo cede la relación en los bordes, donde el clip que se saldría se queda.
export function filasDestinoRelativas(
  filasOrigen: Map<string, number>,
  shift: number,
  maxFila: number,
): Map<string, number> {
  const out = new Map<string, number>();
  for (const [id, fila] of filasOrigen) {
    out.set(id, Math.min(maxFila, Math.max(0, fila + shift)));
  }
  return out;
}

export interface RectMarquee {
  t1: number;
  t2: number;
  fila1: number;
  fila2: number;
}

// Captions cuyo rango [inicio, fin] intersecta [t1, t2] y cuyo carril cae en
// [fila1, fila2] (bounds normalizados: soporta rectángulos invertidos).
export function filtrarPorMarquee(
  captions: Caption[],
  hablantes: Hablante[],
  rect: RectMarquee,
): string[] {
  const t1 = Math.min(rect.t1, rect.t2);
  const t2 = Math.max(rect.t1, rect.t2);
  const fila1 = Math.min(rect.fila1, rect.fila2);
  const fila2 = Math.max(rect.fila1, rect.fila2);
  const ids: string[] = [];
  for (const c of captions) {
    const row = captionRowIndex(c.hablante_id, hablantes);
    if (row < fila1 || row > fila2) continue;
    if (c.inicio > t2 || c.fin < t1) continue;
    ids.push(c.id);
  }
  return ids;
}
