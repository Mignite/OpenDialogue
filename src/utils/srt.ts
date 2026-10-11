import type { Caption } from "../types";

export function parseSrtTime(t: string): number {
  const m = t.trim().match(/(\d+):(\d+):(\d+)[,.](\d+)/);
  if (!m) return 0;
  const [, h, min, s, ms] = m;
  // La cola fraccional ("5", "50", "500") es decimal: "5" = 0.5 s, no 5 ms
  return +h * 3600 + +min * 60 + +s + Number("0." + ms);
}

export function parseSrt(texto: string): Caption[] {
  // El separador tolera líneas "en blanco" con espacios: igual cortan el bloque
  const bloques = texto.replace(/\r/g, "").trim().split(/\n\s*\n/);
  const resultado: Caption[] = [];
  let contador = 1;

  for (const bloque of bloques) {
    // Ignora líneas compuestas solo de espacios dentro del bloque
    const lineas = bloque.split("\n").filter((l) => l.trim().length > 0);
    if (lineas.length < 2) continue;

    const idxTiempo = lineas.findIndex((l) => l.includes("-->"));
    if (idxTiempo === -1) continue;

    const [inicioStr, finStr] = lineas[idxTiempo].split("-->");
    const textoLineas = lineas.slice(idxTiempo + 1);

    resultado.push({
      id: `cap-${contador++}-${Math.random().toString(36).slice(2, 7)}`,
      inicio: parseSrtTime(inicioStr),
      fin: parseSrtTime(finStr),
      texto: textoLineas.join("\n"),
      hablante_id: null,
    });
  }

  return resultado.sort((a, b) => a.inicio - b.inicio);
}

export function formatSrtTimestamp(sec: number): string {
  if (sec < 0) sec = 0;
  let ms = Math.round((sec - Math.floor(sec)) * 1000);
  let h = Math.floor(sec / 3600);
  let m = Math.floor((sec % 3600) / 60);
  let s = Math.floor(sec % 60);
  if (ms >= 1000) {
    ms = 0;
    s += 1;
    if (s >= 60) {
      s = 0;
      m += 1;
      if (m >= 60) {
        m = 0;
        h += 1;
      }
    }
  }
  const pad = (n: number, len = 2) => String(n).padStart(len, "0");
  return `${pad(h)}:${pad(m)}:${pad(s)},${pad(ms, 3)}`;
}

export function buildSrt(caps: Caption[]): string {
  return caps
    .map(
      (c, i) =>
        `${i + 1}\n${formatSrtTimestamp(c.inicio)} --> ${formatSrtTimestamp(c.fin)}\n${c.texto}\n`,
    )
    .join("\n");
}

// Nombres de dispositivo reservados por Windows: no se debería crear un
// archivo con ninguno de ellos, y añadirles ".srt" NO los vuelve legales
// (NUL.txt es equivalente a NUL). Windows además trata los dígitos
// superíndice ¹²³ como dígitos, así que COM¹ también está reservado.
// El rango documentado es COM1-9 / LPT1-9: COM0 y LPT0 NO están reservados.
// Solo cuenta el nombre completo anterior al primer punto: "CONyecto" o
// "COM10" son legales.
//
// OJO: en Windows 11 reciente (medido en 10.0.26300) escribir "CON.srt" o
// incluso "NUL" con las APIs anchas SÍ funciona; el comportamiento cambió por
// versión y sigue variando según el método de acceso (SMB las rechaza). El
// prefijo "_" es inofensivo donde no hace falta y evita el fallo donde sí.
const RESERVADOS_WINDOWS =
  /^(con|prn|aux|nul|com[1-9¹²³]|lpt[1-9¹²³])$/i;

// Nombre del .srt de un hablante en el export por hablante. Sustituye los
// caracteres prohibidos por "_" y, si el resultado ya se usó, anexa _2, _3...
// Sin esto dos hablantes homónimos (o "A/B" y "A:B", que sanitan al mismo
// texto) escribían el MISMO archivo y el segundo pisaba al primero.
export function nombreArchivoHablanteUnico(
  nombre: string,
  usados: string[],
): string {
  let base = nombre.replace(/[\\/:*?"<>|]/g, "_");
  // Prefijo antes de deduplicar: el nombre reservado debe compararse ya
  // saneado, o "NUL" y "_NUL" no se detectarían como el mismo archivo.
  const antesDelPunto = base.split(".")[0].replace(/[ .]+$/, "");
  if (RESERVADOS_WINDOWS.test(antesDelPunto)) base = `_${base}`;
  // La comparación es case-insensitive: Windows y macOS tratan el filesystem
  // así, así que "Ana.srt" y "ana.srt" son el mismo archivo y la segunda
  // escritura pisaba a la primera. El nombre devuelto conserva las
  // mayúsculas del hablante.
  const usadosLower = usados.map((u) => u.toLowerCase());
  const libre = (n: string) => !usadosLower.includes(n.toLowerCase());
  if (libre(`${base}.srt`)) return `${base}.srt`;
  let n = 2;
  while (!libre(`${base}_${n}.srt`)) n++;
  return `${base}_${n}.srt`;
}