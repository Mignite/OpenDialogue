import type { Caption, OverlapEntry, Hablante } from "../types";
import { SNAP_THRESHOLD } from "./constants";
import { formatTime } from "./time";
import { t } from "../i18n";

export function BuildOverlapReport(
  caps: Caption[],
  hablantes: Hablante[],
): OverlapEntry[] {
  const nombreDe = (id: string | null) =>
    hablantes.find((h) => h.id === id)?.nombre ||
    hablantes.find((h) => h.id === id)?.tecla ||
    t("overlap.unassigned");

  const ordenados = [...caps].sort((a, b) => a.inicio - b.inicio);
  const resultado: OverlapEntry[] = [];

  for (let i = 0; i < ordenados.length; i++) {
    for (let j = i + 1; j < ordenados.length; j++) {
      const a = ordenados[i];
      const b = ordenados[j];
      if (b.inicio >= a.fin) break;
      if (a.hablante_id === b.hablante_id) continue;
      resultado.push({
        inicio: Math.max(a.inicio, b.inicio),
        fin: Math.min(a.fin, b.fin),
        hablanteA: nombreDe(a.hablante_id),
        textoA: a.texto,
        hablanteB: nombreDe(b.hablante_id),
        textoB: b.texto,
      });
    }
  }
  return resultado;
}

export function FormatOverlapReport(entries: OverlapEntry[]): string {
  if (entries.length === 0) return t("overlap.noOverlaps");
  return entries
    .map(
      (e) =>
        `${formatTime(e.inicio)} \u2192 ${formatTime(e.fin)}  |  ${e.hablanteA} \u2194 ${e.hablanteB}\n  "${e.textoA}"\n  "${e.textoB}"\n`,
    )
    .join("\n");
}

export function findSnapTime(
  time: number,
  excludeIds: string | string[],
  caps: Caption[],
  extraTargets: number[] = [],
): number | null {
  const exclude =
    typeof excludeIds === "string" ? new Set([excludeIds]) : new Set(excludeIds);
  let best: number | null = null;
  let bestDist = SNAP_THRESHOLD;
  const considerar = (t: number) => {
    const dist = Math.abs(time - t);
    if (dist < bestDist) {
      bestDist = dist;
      best = t;
    }
  };
  for (const cap of caps) {
    if (exclude.has(cap.id)) continue;
    considerar(cap.inicio);
    considerar(cap.fin);
  }
  for (const t of extraTargets) considerar(t);
  return best;
}

// Clamp del body drag contra el borde izquierdo (t=0). `tMin` es el inicio
// MÁS TEMPRANO de la selección, NO el del clip lead (el que se agarró): si
// otro clip del bloque empieza antes, el clamp por lead dejaba pasar un
// deltaT que empujaba ese clip por debajo de 0, y `moverCaptions` lo recortaba a
// 0 por su cuenta — la selección se deformaba (el desfase interno entre
// clips no se conservaba) y el preview no coincidía con lo guardado.
export function clampDeltaBloque(deltaT: number, tMin: number): number {
  // El clamp de t=0. Devuelve -0 cuando tMin es 0 (inofensivo: `-0 === 0`,
  // así que los guards `deltaT === 0` de moverCaptions siguen disparando).
  return Math.max(deltaT, -tMin);
}

// Snap de bloque (body drag, estilo Premiere): prueba AMBOS extremos del
// bloque ya desplazado (tMin/tMax) contra bordes vecinos + playhead y
// devuelve el delta del imán más cercano (0 si ninguno). Con un clip,
// tMin == lead.inicio y equivale al snap simple de antes.
// `excludeIds` acepta un Set ya armado: el body drag lo reutiliza en cada
// mousemove en vez de construir uno nuevo del array de ids.
export function findSnapBlockDelta(
  tMin: number,
  tMax: number,
  excludeIds: string | string[] | Set<string>,
  caps: Caption[],
  playhead: number | null,
): number {
  const exclude =
    excludeIds instanceof Set
      ? excludeIds
      : typeof excludeIds === "string"
        ? new Set([excludeIds])
        : new Set(excludeIds);
  let bestDelta = 0;
  let bestDist = SNAP_THRESHOLD;
  const considerar = (t: number) => {
    for (const bound of [tMin, tMax]) {
      const dist = Math.abs(bound - t);
      if (dist < bestDist) {
        bestDist = dist;
        bestDelta = t - bound;
      }
    }
  };
  for (const cap of caps) {
    if (exclude.has(cap.id)) continue;
    considerar(cap.inicio);
    considerar(cap.fin);
  }
  if (playhead !== null) considerar(playhead);
  return bestDelta;
}