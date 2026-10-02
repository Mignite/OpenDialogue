import { invoke } from "@tauri-apps/api/core";
import { appConfigDir, join } from "@tauri-apps/api/path";
import { AJUSTES_ARCHIVO } from "./constants";

export interface Ajustes {
  locale?: "en" | "es";
}

function esLocale(valor: unknown): valor is "en" | "es" {
  return valor === "en" || valor === "es";
}

export function ajustesDesdeJson(
  texto: string | null | undefined,
): Ajustes {
  if (!texto) return {};
  try {
    const data = JSON.parse(texto) as { locale?: unknown };
    return esLocale(data?.locale) ? { locale: data.locale } : {};
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
