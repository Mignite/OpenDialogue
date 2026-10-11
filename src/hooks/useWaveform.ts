import { useCallback, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// Waveform: análisis de volumen (symphonia vía IPC) + caché en memoria.
// El dibujado es en tiempo real desde `volumen` (ver drawCanvasFrame): sin
// prerender topado, vale para cualquier duración.
// El mirror bare `volumenRef = volumen` se queda en App (regla transversal:
// un solo mirror); este hook expone el ref para los lectores en rAF.
export function useWaveform() {
  const [volumen, setVolumen] = useState<number[]>([]);
  const [analizando, setAnalizando] = useState<boolean>(false);
  const volumenRef = useRef<number[]>([]);
  const analisisVolumenRequestRef = useRef(0);
  const waveformCacheRef = useRef<Map<string, number[]>>(new Map());
  // Ruta con análisis en curso (si hay). Reabrirla no duplica el trabajo:
  // se deja el análisis y sus chunks en paz (ver estaEnAnalisis).
  const enCursoRef = useRef<string | null>(null);
  // Ruta → requestId de la ÚLTIMA tarea Rust lanzada para ella. El contador
  // global se invalida en cada `reiniciar`, pero la tarea Rust original puede
  // seguir corriendo (el backend deduplica por (ruta, track) y rechaza el
  // duplicado con "Ya se está analizando"). Al reabrir, ese rechazo es la
  // prueba de que la original sigue viva y se re-ancla a ella (ver catch del
  // invoke) en vez de quedarse con el waveform vacío. Solo lo tocan los
  // intentos que llegan a invocar: los atajos de caché no lanzan tarea.
  const rutaRequestRef = useRef<Map<string, number>>(new Map());

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

  // useCallback-estable (solo refs/setters/funciones de módulo): los
  // consumidores (cargarVideo, slice useProyecto) no re-suscriben nada.
  const analizarVolumenDe = useCallback(async (ruta: string) => {
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

    // Registrar el intento ANTES de invocar: si el backend lo rechaza por
    // duplicado, `previoRequestId` identifica la tarea vigente a re-anclar.
    const previoRequestId = rutaRequestRef.current.get(ruta);
    rutaRequestRef.current.set(ruta, miRequestId);
    let reanclado = false;

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
      if (
        String(err).includes("Ya se está analizando") &&
        previoRequestId !== undefined &&
        analisisVolumenRequestRef.current === miRequestId
      ) {
        // La tarea original de esta ruta sigue corriendo (el rechazo lo
        // prueba): re-anclar el request vigente a ella en vez de duplicar.
        // Su continuación pendiente aceptará el resultado con este id y el
        // guard `estaEnAnalisis` vuelve a reconocerla (caso normal). El
        // waveform intermedio se pierde, pero el resultado final es completo.
        rutaRequestRef.current.set(ruta, previoRequestId);
        analisisVolumenRequestRef.current = previoRequestId;
        enCursoRef.current = ruta;
        reanclado = true;
      } else if (analisisVolumenRequestRef.current === miRequestId) {
        console.error("Error analizando volumen:", err);
      }
    } finally {
      if (analisisVolumenRequestRef.current === miRequestId)
        setAnalizando(false);
      // Las salidas obsoletas (stale) no limpian: el análisis vigente es de
      // otra ruta (reiniciar ya limpió) o de esta misma en curso. Una salida
      // re-anclada tampoco: el vigente es el original recién re-anclado.
      if (!reanclado && enCursoRef.current === ruta) enCursoRef.current = null;
    }
  }, []);

  return { volumen, analizando, volumenRef, analizarVolumenDe, reiniciar, estaEnAnalisis };
}
