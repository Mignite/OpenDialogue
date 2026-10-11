import type { Proyecto } from "../types";

// Recuperación ante crash.
//
// Hoy la app solo avisa al cerrar con cambios sin guardar: si el proceso
// muere (crash del webview, OOM, cierre forzado de sesión) todo lo no
// guardado se pierde y no queda rastro. Este módulo es la pieza de datos:
// el autosave escribe un "recovery" a disco y al arrancar se lee para
// ofrecer recuperarlo.
//
// Decisiones:
// - El recovery envuelve el proyecto CON la ruta de origen, no lo reemplaza:
//   al recuperar, la app abre el proyecto guardado y solo avisa de que hay
//   una versión más nueva.
// - Todo lo de E/S (invoke, rutas de appConfigDir) vive en el hook; aquí solo
//   hay funciones puras, que son las que se testean.
// - `version` permite descartar archivos de una versión futura o de una app
//   vieja sin romper el arranque.

export interface Recovery {
  version: 1;
  /** Ruta del .cdp original ("" si nunca se guardó). */
  rutaProyecto: string;
  /** Fecha de escritura (epoch ms). */
  escrito: number;
  proyecto: Proyecto;
}

/** Un recovery más viejo que esto se considera muerto. Si el usuario no abrió
 *  la app en 7 días, la copia automática ya no interesa (y ocupa espacio). */
export const RECOVERY_VIGENTE_MS = 7 * 24 * 60 * 60 * 1000;

/** Cada cuánto se escribe la copia automática mientras se edita. Es un
 *  seguro ante crash, no un backup: 15 s es suficiente para acotar la pérdida
 *  sin escribir en cada pulsación. */
export const INTERVALO_AUTOSAVE_MS = 15_000;

/** Decide si toca escribir el recovery ahora.
 *
 *  `sucio` = hay cambios sin guardar, `nCaptions` = tamaño del proyecto,
 *  `ultimaEscritura` = epoch ms de la última escritura (null = nunca). */
export function debeGuardarRecovery(
  sucio: boolean,
  nCaptions: number,
  ultimaEscritura: number | null,
  ahora: number = Date.now(),
): boolean {
  if (!sucio) return false;
  if (nCaptions === 0) return false;
  if (ultimaEscritura === null) return true;
  return ahora - ultimaEscritura >= INTERVALO_AUTOSAVE_MS;
}

export function construirRecovery(
  proyecto: Proyecto,
  rutaProyecto: string,
  escrito: number,
): Recovery {
  return { version: 1, rutaProyecto, escrito, proyecto };
}

function esProyecto(v: unknown): v is Proyecto {
  if (typeof v !== "object" || v === null) return false;
  const p = v as Record<string, unknown>;
  return Array.isArray(p.captions) && Array.isArray(p.hablantes);
}

/** Lee y valida un recovery. Devuelve null ante cualquier cosa que no sea un
 *  recovery válido y vigente; el objetivo es que un archivo corrupto NUNCA
 *  impida abrir la app.
 *
 *  `corte` (epoch ms) descarta lo más viejo: si el proyecto ya se guardó
 *  después de ese instante, el recovery ya no aporta nada. */
export function leerRecovery(
  texto: string | null | undefined,
  corte = 0,
): Recovery | null {
  if (!texto) return null;
  let data: unknown;
  try {
    data = JSON.parse(texto);
  } catch {
    return null;
  }
  if (typeof data !== "object" || data === null) return null;
  const r = data as Record<string, unknown>;
  if (r.version !== 1) return null;
  if (typeof r.rutaProyecto !== "string") return null;
  if (typeof r.escrito !== "number" || !Number.isFinite(r.escrito)) return null;
  if (!esProyecto(r.proyecto)) return null;
  if (r.escrito < corte) return null;
  if (r.escrito < Date.now() - RECOVERY_VIGENTE_MS) return null;
  return r as unknown as Recovery;
}

/** Texto para el diálogo de recuperación. El nombre del proyecto se usa tal
 *  cual venga: si no hay ruta guardada, es un proyecto nuevo y solo tenemos
 *  este texto para que el usuario lo reconozca. */
export function formatearRecovery(r: Recovery, nombreProyecto: string): string {
  const cuando = new Date(r.escrito).toLocaleString();
  const n = r.proyecto.captions.length;
  const subtitulos = n === 1 ? "1 subtítulo" : `${n} subtítulos`;
  const donde = r.rutaProyecto
    ? ` (${nombreProyecto || r.rutaProyecto})`
    : " (sin guardar)";
  return `Recuperado ${cuando}${donde} — ${subtitulos}`;
}