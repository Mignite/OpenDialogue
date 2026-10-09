import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import type { Caption, Hablante, PresetAss } from "../types";
import { buildAss, presetParaExportar } from "../utils/ass";
import { cargarPresetsAss, guardarPresetsAss } from "../utils/assPresets";
import { t } from "../i18n";

export interface AssEstilosDeps {
  captionsRef: { current: Caption[] };
  hablantesRef: { current: Hablante[] };
  // Mensaje efímero en el statusBar (vive en App).
  notify: (msg: string) => void;
  // Se llama tras exportar con éxito (App cierra el modal).
  alExportar: () => void;
}

// Estilos .ass: presets globales + export con asignación + resolución PlayRes.
// No toca captions/hablantes (eso es de useCaptions/useProyecto): solo lee
// sus refs para armar el .ass. `persistirPresets` es useCallback-estable ([])
// porque se pasa a paneles memo() que re-renderizan con el playhead.
export function useAssEstilos({
  captionsRef,
  hablantesRef,
  notify,
  alExportar,
}: AssEstilosDeps) {
  const [presetsAss, setPresetsAss] = useState<PresetAss[]>([]);
  const [resAss, setResAss] = useState({ x: 1920, y: 1080 });

  const recargarPresets = useCallback(async () => {
    // En navegador (sin Tauri) falla y queda []: los selects muestran "—".
    try {
      setPresetsAss(await cargarPresetsAss());
    } catch {
      /* sin backend: presets vacíos */
    }
  }, []);

  useEffect(() => {
    void recargarPresets();
  }, [recargarPresets]);

  // Timer del auto-guardado (trailing debounce): el write a disco vive acá,
  // no en el panel.
  const presetWriteTimerRef = useRef<number>(0);
  const persistirPresets = useCallback(
    async (presets: PresetAss[], opts?: { ya?: boolean }) => {
      setPresetsAss(presets);
      // El estado en memoria es inmediato (typing fluido); el disco va con
      // trailing debounce: un solo write por ráfaga de keystrokes y siempre
      // con el ÚLTIMO contenido (sin reordenamientos). `ya:true` escribe al
      // instante (import .ass: el mensaje de éxito debe implicar disco).
      window.clearTimeout(presetWriteTimerRef.current);
      const escribir = async () => {
        try {
          await guardarPresetsAss(presets);
        } catch (err) {
          console.error("Error guardando presets .ass:", err);
          notify(t("styles.saveError"));
        }
      };
      if (opts?.ya) {
        await escribir();
        return;
      }
      presetWriteTimerRef.current = window.setTimeout(() => {
        void escribir();
      }, 400);
    },
    [notify],
  );

  const exportarAssConPreset = useCallback(
    async (
      asignacion: Record<string, string>,
      presetSinHablante: string,
    ) => {
      try {
        const path = await save({
          filters: [{ name: t("dialog.filterAss"), extensions: ["ass"] }],
          defaultPath: "subtitulos.ass",
        });
        if (!path) return;
        // La asignacion vive solo en el modal: se resuelve a un preset por
        // hablante y se la pasa al builder. No se persiste en ningun lado.
        // Si el modal no tocó la fila, vale lo guardado en el hablante.
        const porId = new Map(presetsAss.map((p) => [p.id, p]));
        const primero = presetsAss[0];
        const sinHablante = porId.get(presetSinHablante) ?? primero;
        if (!sinHablante) return;
        const presetDe = (hablanteId: string | null) =>
          presetParaExportar(
            hablantesRef.current,
            porId,
            asignacion,
            hablanteId,
            sinHablante,
          );
        // Se arma DESPUÉS del save: si el usuario cancela, no se hace el trabajo.
        const contenido = buildAss(
          captionsRef.current,
          hablantesRef.current,
          presetDe,
          resAss.x,
          resAss.y,
        );
        await invoke("escribir_archivo_texto", { ruta: path, contenido });
        alExportar();
        notify(t("assExport.done", { count: captionsRef.current.length }));
      } catch (err) {
        console.error("Error exportando .ass:", err);
      }
    },
    [presetsAss, resAss, captionsRef, hablantesRef, notify, alExportar],
  );

  return {
    presetsAss,
    resAss,
    setResAss,
    recargarPresets,
    persistirPresets,
    exportarAssConPreset,
  };
}
