import {
  VENTANAS_POR_SEGUNDO,
  ISLA_VENTANA_FONDO_SEG,
  ISLA_FACTOR_UMBRAL,
  ISLA_PISO,
  ISLA_HISTERESIS_MUESTRAS,
  ISLA_MAX_DURACION,
  ISLA_COLA_SEG,
  ISLA_FONDO_PERCENTIL,
  ISLA_BUSQUEDA_ONSET_SEG,
  ISLA_SNAP_ATRAS_SEG,
  ISLA_LEAD_MAX_SEG,
} from "./constants";

// Umbral de "hay voz" en el playhead: percentil bajo de los 2s previos (el
// percentil, no la media: la media se infla con el habla misma y tapa la
// frase cuando el playhead cae dentro) por ISLA_FACTOR_UMBRAL, con piso.
function umbralLocal(volumen: number[], i0: number): number {
  const vps = VENTANAS_POR_SEGUNDO;
  const ventana = Math.round(ISLA_VENTANA_FONDO_SEG * vps);
  const desde = Math.max(0, i0 - ventana);
  let fondo = ISLA_PISO;
  if (desde < i0) {
    const previas = volumen.slice(desde, i0).sort((a, b) => a - b);
    const idx = Math.min(
      previas.length - 1,
      Math.floor(ISLA_FONDO_PERCENTIL * previas.length),
    );
    fondo = Math.max(previas[idx], ISLA_PISO);
  }
  return Math.max(fondo * ISLA_FACTOR_UMBRAL, ISLA_PISO);
}

export interface TramoAudio {
  inicio: number;
  fin: number;
}

// Tramo de audio que debe cubrir un fragmento nuevo creado en `playhead`:
//  - playhead sobre habla → retrocede al onset (máx. ISLA_SNAP_ATRAS_SEG).
//  - playhead en silencio → busca el próximo onset hasta ISLA_BUSQUEDA_ONSET_SEG;
//    arranca en el playhead, o hasta ISLA_LEAD_MAX_SEG antes de la voz (nunca
//    después del playhead: el seguimiento del playhead perdería la selección).
//  - fin: último sonido + ISLA_COLA_SEG, con tope ISLA_MAX_DURACION de habla.
//    Sin piso de duración: un mínimo estiraba las frases cortas con silencio
//    muerto y el fragmento dejaba de parecerse al pico.
// null si no hay habla alrededor (el caller usa el fallback).
export function buscarTramoAudio(
  volumen: number[],
  playhead: number,
): TramoAudio | null {
  const vps = VENTANAS_POR_SEGUNDO;
  const n = volumen.length;
  if (n === 0) return null;
  const i0 = Math.max(0, Math.floor(playhead * vps));
  if (i0 >= n) return null;

  const umbral = umbralLocal(volumen, i0);
  const alto = (i: number) => i >= 0 && i < n && volumen[i] >= umbral;

  let onset: number;
  let inicio: number;
  if (alto(i0)) {
    let j = i0;
    const lim = Math.max(0, i0 - Math.round(ISLA_SNAP_ATRAS_SEG * vps));
    while (j > lim && alto(j - 1)) j--;
    onset = j;
    inicio = j / vps;
  } else {
    let j = i0;
    const lim = Math.min(n, i0 + Math.round(ISLA_BUSQUEDA_ONSET_SEG * vps));
    while (j < lim && !alto(j)) j++;
    if (j >= lim) return null;
    onset = j;
    inicio = Math.min(playhead, Math.max(0, j / vps - ISLA_LEAD_MAX_SEG));
  }

  // Barrido con histéresis: tolera micro-pausas dentro de la frase
  // (ISLA_HISTERESIS_MUESTRAS muestras por debajo del umbral antes de cerrar).
  let ultimoAlto = onset;
  let tolerancia = ISLA_HISTERESIS_MUESTRAS;
  for (let j = onset + 1; j < n; j++) {
    if (volumen[j] >= umbral) {
      ultimoAlto = j;
      tolerancia = ISLA_HISTERESIS_MUESTRAS;
    } else if (--tolerancia <= 0) {
      break;
    }
  }

  const fin = Math.min(
    (ultimoAlto + 1) / vps + ISLA_COLA_SEG,
    onset / vps + ISLA_MAX_DURACION,
  );
  return { inicio, fin };
}

// Pico del RMS en [t0, t1): agregación por píxel para el dibujado en tiempo
// real (sin prerender topado). Fuera del audio o vacío → 0.
export function picoEnRango(vol: number[], t0: number, t1: number): number {
  const vps = VENTANAS_POR_SEGUNDO;
  const a = Math.max(0, Math.floor(t0 * vps));
  const b = Math.min(vol.length, Math.max(a + 1, Math.ceil(t1 * vps)));
  let pico = 0;
  for (let i = a; i < b; i++) {
    if (vol[i] > pico) pico = vol[i];
  }
  return pico;
}
