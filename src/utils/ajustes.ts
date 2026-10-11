import { invoke } from "@tauri-apps/api/core";
import { appConfigDir, join } from "@tauri-apps/api/path";
import { AJUSTES_ARCHIVO, VELOCIDADES_REPRODUCCION } from "./constants";

export interface Ajustes {
  locale?: "en" | "es";
  velocidad?: number;
}

function esLocale(valor: unknown): valor is "en" | "es" {
  return valor === "en" || valor === "es";
}

// Solo se aceptan las velocidades del selector: un ajustes.json editado a mano
// con 7.3 no debería llegar al <video>.
function esVelocidad(valor: unknown): valor is number {
  return (
    typeof valor === "number" &&
    Number.isFinite(valor) &&
    (VELOCIDADES_REPRODUCCION as readonly number[]).includes(valor)
  );
}

export function ajustesDesdeJson(
  texto: string | null | undefined,
): Ajustes {
  if (!texto) return {};
  try {
    const data = JSON.parse(texto) as { locale?: unknown; velocidad?: unknown };
    const ajustes: Ajustes = {};
    if (esLocale(data?.locale)) ajustes.locale = data.locale;
    if (esVelocidad(data?.velocidad)) ajustes.velocidad = data.velocidad;
    return ajustes;
  } catch {
    return {};
  }
}

export function ajustesAJson(ajustes: Ajustes): string {
  return JSON.stringify(ajustes, null, 2);
}

export async function cargarAjustes(): Promise<Ajustes> {
  try {
    const ruta = await join(await appConfigDir(), AJUSTES_ARCHIVO);
    return ajustesDesdeJson(
      await invoke<string>("leer_archivo_texto", { ruta }),
    );
  } catch {
    // Todavía no hay archivo (primer uso) o no se puede leer.
    return {};
  }
}

export async function guardarAjustes(ajustes: Ajustes): Promise<void> {
  try {
    const ruta = await join(await appConfigDir(), AJUSTES_ARCHIVO);
    await invoke("escribir_archivo_texto", {
      ruta,
      contenido: ajustesAJson(ajustes),
    });
  } catch {
    // Sin backend (navegador/tests): el idioma igual queda en localStorage.
  }
}
