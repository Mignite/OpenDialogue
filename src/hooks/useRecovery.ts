import { useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Proyecto } from "../types";
import {
  construirRecovery,
  formatearRecovery,
  debeGuardarRecovery,
  leerRecovery,
  type Recovery,
} from "../utils/recovery";

export interface RecoveryDeps {
  captionsRef: { current: Proyecto["captions"] };
  hablantesRef: { current: Proyecto["hablantes"] };
  rutaVideoRef: { current: string };
  rutaProyectoRef: { current: string };
  videoRef: { current: HTMLVideoElement | null };
  /** Fuente de verdad del "cambios sin guardar". */
  isDirtyRef: { current: boolean };
  /** Cargar un proyecto recuperado en memoria (sin tocar disco). */
  alRecuperar: (rutaProyecto: string, proyecto: Proyecto) => void;
  /** Traducir la pregunta de recuperación, con el resumen del recovery. */
  preguntarRecuperar: (resumen: string) => Promise<boolean>;
}

// Recovery ante crash: copia automática periódica + oferta al arrancar.
//
// Sin esto, un cierre no limpio (crash del webview, OOM, fin de sesión) pierde
// TODO el trabajo no guardado: la app solo avisaba al cerrar con el diálogo de
// confirmación, que un crash nunca muestra.
//
// Es un seguro, no un backup: escribe en appConfigDir/recovery.json cada
// INTERVALO_AUTOSAVE_MS mientras haya cambios, y al arrancar pregunta si se
// quiere recuperar. Guardar en disco (Ctrl+S) sigue siendo lo que decide la
// versión buena; el recovery no pisa el proyecto.
// El intervalo del poll es corto (5 s) pero la escritura real la frena
// `debeGuardarRecovery`, que exige INTERVALO_AUTOSAVE_MS entre escrituras y
// que no escriba si no hay cambios. Así el coste por tick es una comparación.
const INTERVALO_POLL_MS = 5_000;

export function useRecovery({
  captionsRef,
  hablantesRef,
  rutaVideoRef,
  rutaProyectoRef,
  videoRef,
  isDirtyRef,
  alRecuperar,
  preguntarRecuperar,
}: RecoveryDeps) {
  const ultimaEscrituraRef = useRef<number | null>(null);

  const escribirRecovery = useCallback(async () => {
    // La decisión (¿toca escribir?) vive en la función pura testeada; acá solo
    // se arma el payload y se escribe.
    if (
      !debeGuardarRecovery(
        isDirtyRef.current,
        captionsRef.current.length,
        ultimaEscrituraRef.current,
      )
    ) {
      return;
    }
    const proyecto: Proyecto = {
      ruta_video: rutaVideoRef.current,
      hablantes: hablantesRef.current,
      captions: captionsRef.current,
      playhead: videoRef.current?.currentTime ?? 0,
    };
    try {
      await invoke("guardar_recovery", {
        contenido: JSON.stringify(
          construirRecovery(proyecto, rutaProyectoRef.current, Date.now()),
        ),
      });
      ultimaEscrituraRef.current = Date.now();
    } catch (err) {
      // Falla silenciosa a propósito: el autosave es un extra. Si el disco está
      // lleno o no hay permisos, el aviso al usuario sigue siendo el guardado
      // manual, no esto.
      console.error("Error escribiendo recovery:", err);
    }
  }, [
    captionsRef,
    hablantesRef,
    rutaVideoRef,
    rutaProyectoRef,
    videoRef,
    isDirtyRef,
  ]);

  const borrarRecovery = useCallback(async () => {
    try {
      await invoke("guardar_recovery", { contenido: null });
      ultimaEscrituraRef.current = null;
    } catch {
      // Idem: borrar la copia automática no es crítico.
    }
  }, []);

  // Escritura periódica. Un solo intervalo para toda la app (no uno por cada
  // cambio): decide `debeGuardarRecovery` con el estado real, así que un
  // proyecto limpio o vacío no escribe nada.
  useEffect(() => {
    const id = window.setInterval(() => {
      void escribirRecovery();
    }, INTERVALO_POLL_MS);
    return () => window.clearInterval(id);
  }, [escribirRecovery]);

  // Al arrancar: si hay un recovery válido, preguntar antes de tocar nada. Si
  // el usuario no lo quiere (o el recovery no sirve), se borra para no volver
  // a preguntar en cada inicio.
  useEffect(() => {
    let vivo = true;
    const descartar = () => {
      void invoke("guardar_recovery", { contenido: null }).catch(() => {});
    };
    void (async () => {
      let texto: string | null = null;
      try {
        texto = await invoke<string | null>("leer_recovery");
      } catch {
        return;
      }
      if (!vivo || !texto) return;
      const r: Recovery | null = leerRecovery(texto);
      // Recovery corrupto o de otra versión: se descarta y no bloquea el
      // arranque.
      if (!r) {
        descartar();
        return;
      }
      const nombre = r.rutaProyecto.split(/[\\/]/).pop() ?? "";
      const ok = await preguntarRecuperar(formatearRecovery(r, nombre));
      if (!vivo) return;
      if (!ok) {
        descartar();
        return;
      }
      alRecuperar(r.rutaProyecto, r.proyecto);
      // Recovery aceptado: la app vuelve a estar en línea y el estado actual
      // es el bueno, así que la copia deja de hacer falta.
      descartar();
    })();
    return () => {
      vivo = false;
    };
    // Solo al montar: es la comprobación de arranque.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return { escribirRecovery, borrarRecovery, debeGuardar: debeGuardarRecovery };
}