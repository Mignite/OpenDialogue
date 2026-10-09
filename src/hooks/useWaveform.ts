import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { compactarWaveform } from "../utils/audioIslands";
import { WAVEFORM_MAX_W } from "../utils/constants";

// Waveform: análisis de volumen (symphonia vía IPC) + caché + prerender.
// El mirror bare `volumenRef = volumen` se queda en App (regla transversal:
// un solo mirror); este hook expone el ref para los lectores en rAF.
export function useWaveform({
  waveformPreRenderRef,
}: {
  waveformPreRenderRef: { current: HTMLCanvasElement | null };
}) {
  const [volumen, setVolumen] = useState<number[]>([]);
  const [analizando, setAnalizando] = useState<boolean>(false);
  const volumenRef = useRef<number[]>([]);
  const analisisVolumenRequestRef = useRef(0);
  const waveformCacheRef = useRef<Map<string, number[]>>(new Map());
  // Ruta con análisis en curso (si hay). Reabrirla no duplica el trabajo:
  // se deja el análisis y sus chunks en paz (ver estaEnAnalisis).
  const enCursoRef = useRef<string | null>(null);

  // Descarta el análisis en vuelo y limpia el waveform (al abrir video,
  // proyecto o empezar uno nuevo). No toca duración ni ventana: eso es App.
  const reiniciar = useCallback(() => {
    enCursoRef.current = null;
    analisisVolumenRequestRef.current++;
    setVolumen([]);
    setAnalizando(false);
    waveformCacheRef.current.clear();
  }, []);

  // ¿Sigue corriendo el análisis de esta ruta? Para no pisarlo al reabrir
  // el mismo video a mitad de análisis (el waveform continúa y completa).
  const estaEnAnalisis = useCallback(
    (ruta: string) => enCursoRef.current === ruta,
    [],
  );

  // Cache en memoria con tope: evita que una sesión larga acumule waveforms
  // de decenas de videos sin límite.
  function cachearVolumen(clave: string, datos: number[]) {
    const cache = waveformCacheRef.current;
    if (cache.has(clave)) cache.delete(clave);
    cache.set(clave, datos);
    while (cache.size > 8) {
      const primera = cache.keys().next().value;
      if (primera === undefined) break;
      cache.delete(primera);
    }
  }

  async function analizarVolumenDe(ruta: string) {
    // Single-track: siempre la pista 0 (el video llega pre-editado con solo
    // diálogo). Sin selector de pista ni extracción: el video suena directo.
    const track_index = 0;

    // Blindaje: si esta misma ruta ya se está analizando (reapertura a mitad
    // de análisis), no duplicar ni resetear: los chunks en vuelo siguen
    // llegando y el resultado se acepta con el requestId vigente.
    if (enCursoRef.current === ruta) return;
    enCursoRef.current = ruta;

    const claveCache = `${ruta}::${track_index}`;
    const miRequestId = ++analisisVolumenRequestRef.current;

    const cached = waveformCacheRef.current.get(claveCache);
    if (cached) {
      setVolumen(cached);
      setAnalizando(false);
      enCursoRef.current = null;
      return;
    }

    try {
      const tieneCache = await invoke<boolean>("existe_cache_volumen", {
        rutaVideo: ruta,
        trackIndex: track_index,
      });
      if (analisisVolumenRequestRef.current !== miRequestId) return; // respuesta obsoleta, descartar

      if (tieneCache) {
        const datos = await invoke<number[]>("cargar_cache_volumen", {
          rutaVideo: ruta,
          trackIndex: track_index,
        });
        if (analisisVolumenRequestRef.current !== miRequestId) return;
        cachearVolumen(claveCache, datos);
        setVolumen(datos);
        setAnalizando(false);
        enCursoRef.current = null;
        return;
      }
    } catch (err) {
      console.warn("Error al leer caché de disco:", err);
    }

    setAnalizando(true);
    setVolumen([]);

    const unlistenChunk = await listen<[number | null, number[]]>(
      "volumen_chunk",
      (event) => {
        const [chunkTrack, datos] = event.payload;
        if (chunkTrack !== track_index) return; // descarta chunks de análisis anteriores
        if (analisisVolumenRequestRef.current !== miRequestId) {
          unlistenChunk();
          return;
        }
        setVolumen((prev) => [...prev, ...datos]);
      },
    );

    try {
      const resultado = await invoke<number[]>("analizar_volumen", {
        ruta,
        trackIndex: track_index,
      });
      unlistenChunk();
      if (analisisVolumenRequestRef.current !== miRequestId) return;
      cachearVolumen(claveCache, resultado);
      setVolumen(resultado);
    } catch (err) {
      unlistenChunk();
      console.error("Error analizando volumen:", err);
    } finally {
      if (analisisVolumenRequestRef.current === miRequestId)
        setAnalizando(false);
      // Las salidas obsoletas (stale) no limpian: el análisis vigente es de
      // otra ruta (reiniciar ya limpió) o de esta misma en curso.
      if (enCursoRef.current === ruta) enCursoRef.current = null;
    }
  }

  useEffect(() => {
    const vol = volumen;
    if (vol.length === 0) {
      waveformPreRenderRef.current = null;
      return;
    }
    // Mientras llegan chunks, saltar el prerender: se regenera una sola vez
    // al terminar el análisis (volumen + analizando actualizan en el mismo batch).
    if (analizando) return;
    // El canvas no puede superar WAVEFORM_MAX_W px (los navegadores lo matan
    // sin error): en videos largos se compacta preservando picos. El dibujado
    // mapea con escala (ver drawCanvasFrame), así que el tiempo no se mueve.
    const datos = compactarWaveform(vol, WAVEFORM_MAX_W);
    const PISO_DB = -50;
    const TECHO_DB = 0;
    const height = 90;
    const w = datos.length;
    const canvas = document.createElement("canvas");
    canvas.width = w;
    canvas.height = height;
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      waveformPreRenderRef.current = null;
      return;
    }

    const imageData = ctx.createImageData(w, height);
    const data = imageData.data;
    const barHeight = height * 0.85;

    for (let x = 0; x < w; x++) {
      const amp = datos[x];
      const db = 20 * Math.log10(Math.max(amp, 1e-5));
      const normalizado = Math.max(
        0,
        Math.min(1, (db - PISO_DB) / (TECHO_DB - PISO_DB)),
      );
      const barH = Math.max(1, normalizado * barHeight);
      const y0 = Math.floor((height - barH) / 2);
      const y1 = Math.ceil((height + barH) / 2);

      const r = Math.min(255, Math.floor(normalizado * 2 * 255));
      const g = Math.min(255, Math.floor((2 - normalizado * 2) * 255));
      const b = Math.max(0, Math.floor((1 - normalizado * 1.5) * 255));

      for (let y = y0; y < y1 && y < height; y++) {
        const idx = (y * w + x) * 4;
        data[idx] = r;
        data[idx + 1] = g;
        data[idx + 2] = b;
        data[idx + 3] = 255;
      }
    }

    ctx.putImageData(imageData, 0, 0);
    waveformPreRenderRef.current = canvas;
  }, [volumen, analizando, waveformPreRenderRef]);

  return { volumen, analizando, volumenRef, analizarVolumenDe, reiniciar, estaEnAnalisis };
}
