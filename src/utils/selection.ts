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

export interface AgrupacionCarriles {
  /** Caption del carril "—" (sin hablante asignado). */
  sinAsignar: Caption[];
  /** Por id de hablante. Todo hablante de la lista tiene entrada (puede ser
   *  vacía), para que el render no tenga que distinguir los dos casos. */
  porHablante: Map<string, Caption[]>;
}

/** Reparte los captions en sus carriles en UNA pasada (O(n)), en vez de un
 *  `filter` por hablante (O(n × (S+1))).
 *
 *  Motivo medido: `trackRows` se recalculaba en cada render y el playhead
 *  fuerza renders a 60 Hz. Con 2000 captions y 9 hablantes son 1.04 ms por
 *  render contra 0.34 ms de una pasada (3.1×), más 10 arrays nuevos por frame
 *  para el GC. Va en `useMemo([captions, hablantes])`, así que solo corre
 *  cuando los datos cambian de verdad.
 *
 *  Un caption con `hablante_id` que no está en la lista no aparece en ningún
 *  carril: no es lo mismo "sin asignar" que "asignado a alguien que ya no
 *  existe". Queda solo en el array de captions (no se pierde), pero fuera del
 *  timeline hasta que se reasigne. */
export function agruparCaptionsPorCarril(
  captions: Caption[],
  hablantes: Hablante[],
): AgrupacionCarriles {
  const porHablante = new Map<string, Caption[]>();
  // Se presuestan todos los carriles para que un hablante sin captions
  // devuelva [] y no undefined.
  for (const h of hablantes) porHablante.set(h.id, []);
  const sinAsignar: Caption[] = [];
  for (const c of captions) {
    if (!c.hablante_id) {
      sinAsignar.push(c);
      continue;
    }
    const destino = porHablante.get(c.hablante_id);
    // Id desconocido: no es un carril válido, se ignora (ver doc arriba).
    if (destino) destino.push(c);
  }
  return { sinAsignar, porHablante };
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
