// Búsqueda y reemplazo sobre el texto de los subtítulos. Todo case-insensitive
// por defecto; el matching es por indexOf (no regex) para que los caracteres
// especiales del término no haya que escaparlos.

export interface SegmentoTexto {
  texto: string;
  coincide: boolean;
}

export function normalizarTermino(termino: string): string {
  return termino.trim();
}

function indiceDe(
  texto: string,
  termino: string,
  caseSensitive: boolean,
): number {
  return caseSensitive
    ? texto.indexOf(termino)
    : texto.toLowerCase().indexOf(termino.toLowerCase());
}

export function coincide(
  texto: string,
  termino: string,
  caseSensitive: boolean,
): boolean {
  const t = normalizarTermino(termino);
  if (t.length === 0) return false;
  return indiceDe(texto, t, caseSensitive) !== -1;
}

// Parte el texto en segmentos para resaltar con <mark>: los tramos que
// matchean quedan con coincide=true. Matches no solapados, de izquierda a
// derecha ("aaa" con "aa" da ["aa","a"]).
export function partirResaltado(
  texto: string,
  termino: string,
  caseSensitive: boolean,
): SegmentoTexto[] {
  const t = normalizarTermino(termino);
  if (t.length === 0) return [{ texto, coincide: false }];

  const segmentos: SegmentoTexto[] = [];
  let desde = 0;
  for (;;) {
    const idx = indiceDe(texto.slice(desde), t, caseSensitive);
    if (idx === -1) break;
    const abs = desde + idx;
    if (abs > desde) segmentos.push({ texto: texto.slice(desde, abs), coincide: false });
    segmentos.push({ texto: texto.slice(abs, abs + t.length), coincide: true });
    desde = abs + t.length;
  }
  if (desde < texto.length) segmentos.push({ texto: texto.slice(desde), coincide: false });
  return segmentos.length > 0 ? segmentos : [{ texto, coincide: false }];
}

// Reemplaza TODAS las ocurrencias del término (mismo matching que el resaltado)
// sin tocar el resto del texto. Con reemplazo vacío borra las ocurrencias.
export function reemplazarTodos(
  texto: string,
  termino: string,
  reemplazo: string,
  caseSensitive: boolean,
): string {
  const t = normalizarTermino(termino);
  if (t.length === 0) return texto;

  let resultado = "";
  let desde = 0;
  for (;;) {
    const idx = indiceDe(texto.slice(desde), t, caseSensitive);
    if (idx === -1) break;
    const abs = desde + idx;
    resultado += texto.slice(desde, abs) + reemplazo;
    desde = abs + t.length;
  }
  return resultado + texto.slice(desde);
}
