import { PALETA } from "./constants";
import type { Caption, Hablante } from "../types";

export interface TurnoAutosubs {
  speaker: string;
  texto: string;
}

// Turnos "Speaker N:" del TXT de auto-subs (solo trae hora de inicio por
// turno, sin fin: el alineamiento con los cues es por TEXTO, no por tiempo).
export function parseAutosubsTxt(contenido: string): TurnoAutosubs[] {
  const turnos: TurnoAutosubs[] = [];
  // El TXT puede venir con CRLF de Windows: el regex exige \n.
  const limpio = contenido.replace(/\r/g, "");
  const re = /Speaker (\d+):\n(.+?)(?=\nSpeaker \d+:|\s*$)/gs;
  let m: RegExpExecArray | null;
  while ((m = re.exec(limpio)) !== null) {
    turnos.push({ speaker: m[1], texto: m[2].trim() });
  }
  return turnos;
}

// Minúsculas, sin tildes ni puntuación: SRT y TXT puntúan distinto el mismo
// diálogo ("¡Adiós, muy bueno!" vs "Adiós; muy bueno").
export function normalizarTexto(t: string): string {
  return t
    .toLowerCase()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^\w\s]/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

// El cue debe aparecer como palabras completas dentro del turno: con
// `includes` a secas "si" matcheaba dentro de "así que vamos".
function contienePalabras(turno: string, cue: string): boolean {
  return ` ${turno} `.includes(` ${cue} `);
}

// Cada cue hereda el speaker del turno cuyo texto lo contiene. Búsqueda
// SECUENCIAL desde el último match: ambos archivos salen del mismo transcript
// en orden, así que un cue nunca pertenece a un turno anterior al ya asignado.
export function asignarHablantesPorTexto(
  cues: Caption[],
  turnos: TurnoAutosubs[],
): (string | null)[] {
  const normales = turnos.map((t) => normalizarTexto(t.texto));
  let pos = 0;
  return cues.map((c) => {
    const nc = normalizarTexto(c.texto);
    if (!nc) return null;
    let hit = -1;
    for (let i = pos; i < normales.length; i++) {
      if (contienePalabras(normales[i], nc)) {
        hit = i;
        break;
      }
    }
    if (hit === -1) {
      for (let i = 0; i < normales.length; i++) {
        if (contienePalabras(normales[i], nc)) {
          hit = i;
          break;
        }
      }
    }
    if (hit === -1) return null;
    // Sin +1: el siguiente cue del mismo turno se sigue buscando ahí (con
    // frases repetidas entre turnos, +1 lo empujaba al turno siguiente).
    pos = hit;
    return turnos[hit].speaker;
  });
}

// Hablantes con la convención de la app (ver import de transcripción y
// agregarHablante en App.tsx): nombre "Hablante N", tecla dígito, PALETA.
export function hablantesDesdeNombres(numeros: string[]): Hablante[] {
  const base = Date.now();
  // Orden numérico: el llamador pasa un Set ordenado lexicográficamente
  // ("10" antes que "2"), así que se reordena aquí ("2" antes que "10").
  const ordenados = [...numeros].sort((a, b) => {
    const na = Number(a);
    const nb = Number(b);
    if (!Number.isNaN(na) && !Number.isNaN(nb)) return na - nb;
    return a.localeCompare(b);
  });
  return ordenados.map((n, i) => ({
    id: `sp-autosubs-${base}-${i}`,
    nombre: `Hablante ${n}`,
    tecla: String((i % 9) + 1),
    color: PALETA[i % PALETA.length],
  }));
}
