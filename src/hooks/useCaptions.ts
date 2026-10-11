import { useCallback, useEffect, useRef, useState } from "react";
import type { Dispatch, SetStateAction } from "react";
import type { Caption, Hablante } from "../types";
import { buscarFinIslaAudio } from "../utils/audioIslands";
import { clampCaptionTiempos } from "../utils/captionTiempos";
import { ISLA_FALLBACK } from "../utils/constants";
import { useHistory } from "./useHistory";

export interface CaptionsDeps {
  videoRef: { current: HTMLVideoElement | null };
  ondaVolumenRef: { current: number[] };
  hablantesRef: { current: Hablante[] };
  setHablantes: Dispatch<SetStateAction<Hablante[]>>;
}

// Captions y selección: mutaciones, selección múltiple/marquee, drag de
// bordes (commit), editor de texto y transporte de fragmentos. Dueño de
// captions + selectedCaptionIds (state+refs), sortedByStartRef,
// currentCaptionIdxRef y textEditorRef. App conserva los derivados de render
// (matchingCaptions/currentCaption) y los effects de follow/scroll.
export function useCaptions({
  videoRef,
  ondaVolumenRef,
  hablantesRef,
  setHablantes,
}: CaptionsDeps) {
  const [captions, setCaptions] = useState<Caption[]>([]);
  const [selectedCaptionIds, setSelectedCaptionIds] = useState<string[]>([]);
  const captionsRef = useRef<Caption[]>([]);
  const selectedCaptionIdsRef = useRef<string[]>([]);
  const sortedByStartRef = useRef<Caption[]>([]);
  const currentCaptionIdxRef = useRef<number>(-1);
  const textEditorRef = useRef<HTMLTextAreaElement>(null);
  const wasPlayingBeforeEditRef = useRef(false);
  const skipEditorHistoryRef = useRef(false);
  const editorPushedCaptionsRef = useRef<Caption[] | null>(null);
  const editorPushedHablantesRef = useRef<Hablante[] | null>(null);
  // Timer del focus automático del editor (agregar/pegar fragmento). Rastreado
  // para poder cancelarlo: si el usuario navega dentro de los 30 ms, el focus
  // no debe robarle el input.
  const editorFocusTimerRef = useRef<number>(0);
  // Historial vive acá (sus snapshots son captions+hablantes) y se expone
  // para menú/teclado/paneles: así no hay ciclo de orden con quien lo usa.
  const { pushHistorial, deshacer, rehacer, limpiarHistorial } = useHistory(
    captionsRef,
    hablantesRef,
    setCaptions,
    setHablantes,
  );

  useEffect(() => {
    sortedByStartRef.current = [...captions].sort((a, b) => a.inicio - b.inicio);
  }, [captions]);

  // El focus automático no debe sobrevivir al desmontaje ni quedar pendiente si
  // el usuario navega antes de los 30 ms.
  useEffect(() => {
    return () => window.clearTimeout(editorFocusTimerRef.current);
  }, []);

  function asignarHablante(hablanteId: string) {
    const sel = selectedCaptionIdsRef.current;
    if (sel.length > 0) {
      pushHistorial();
      // Set en vez de sel.includes(): la selección puede ser de cientos de
      // captions (un marquee sobre el timeline entero) y el .map recorre
      // TODOS los captions. Con Array.includes() eso es O(n·k).
      const selSet = new Set(sel);
      setCaptions((prev) => {
        const copy = prev.map((c) =>
          selSet.has(c.id) ? { ...c, hablante_id: hablanteId } : c,
        );
        captionsRef.current = copy;
        return copy;
      });
      return;
    }
    const idx = currentCaptionIdxRef.current;
    if (idx === -1) return;
    pushHistorial();

    setCaptions((prev) => {
      const copy = [...prev];
      copy[idx] = { ...copy[idx], hablante_id: hablanteId };
      captionsRef.current = copy;
      return copy;
    });
  }

  function actualizarTextoCaption(id: string, nuevoTexto: string) {
    setCaptions((prev) => {
      const copy = prev.map((c) =>
        c.id === id ? { ...c, texto: nuevoTexto } : c,
      );
      captionsRef.current = copy;
      return copy;
    });
  }

  // Variante en bloque del actualizarTextoCaption para el buscar-y-reemplazar:
  // un solo setCaptions (un render) y un solo pushHistorial desde el llamador.
  // cambios trae el TEXTO NUEVO ya computado por el caller.
  function actualizarTextos(cambios: Array<{ id: string; texto: string }>) {
    if (cambios.length === 0) return;
    const porId = new Map(cambios.map((c) => [c.id, c.texto]));
    setCaptions((prev) => {
      const copy = prev.map((c) => {
        const nuevo = porId.get(c.id);
        return nuevo === undefined ? c : { ...c, texto: nuevo };
      });
      captionsRef.current = copy;
      return copy;
    });
  }

  // Duración de un fragmento nuevo: usa la isla de audio (fin del diálogo
  // bajo el playhead) si hay análisis de volumen; fallback a ISLA_FALLBACK.
  function duracionFragmento(inicio: number): number {
    const finIsla = buscarFinIslaAudio(ondaVolumenRef.current, inicio);
    return finIsla !== null ? finIsla - inicio : ISLA_FALLBACK;
  }

  function agregarFragmento() {
    pushHistorial();
    const video = videoRef.current;
    const inicio = video ? video.currentTime : 0;
    const duracion = duracionFragmento(inicio);
    const nuevo: Caption = {
      id: `cap-frag-${Date.now()}`,
      inicio,
      fin: inicio + duracion,
      texto: "",
      hablante_id: null,
    };
    setCaptions((prev) => {
      const copy = [...prev, nuevo].sort((a, b) => a.inicio - b.inicio);
      captionsRef.current = copy;
      return copy;
    });
    setSelectedCaptionIds([nuevo.id]);
    // El focus dispara handleEditorFocus, pero ya se pusheó el snapshot pre-add.
    skipEditorHistoryRef.current = true;
    // 30 ms para que el textarea exista con el texto ya en su estado. Si en esa
    // ventana el usuario navega o hace clic en otro sitio, el focus robado
    //ría el input; por eso se cancela al desmontar y al volver a enfocar.
    editorFocusTimerRef.current = window.setTimeout(() => {
      editorFocusTimerRef.current = 0;
      textEditorRef.current?.focus();
    }, 30);
  }

  // Ctrl+V: mismo alta que agregarFragmento pero con texto del portapapeles.
  function pegarFragmento(texto: string) {
    pushHistorial();
    const video = videoRef.current;
    const inicio = video ? video.currentTime : 0;
    const duracion = duracionFragmento(inicio);
    const nuevo: Caption = {
      id: `cap-paste-${Date.now()}`,
      inicio,
      fin: inicio + duracion,
      texto,
      hablante_id: null,
    };
    setCaptions((prev) => {
      const copy = [...prev, nuevo].sort((a, b) => a.inicio - b.inicio);
      captionsRef.current = copy;
      return copy;
    });
    setSelectedCaptionIds([nuevo.id]);
  }

  const eliminarCaption = useCallback((id: string) => {
    pushHistorial();
    setCaptions((prev) => {
      const copy = prev.filter((c) => c.id !== id);
      captionsRef.current = copy;
      return copy;
    });
  }, [pushHistorial]);

  const eliminarSeleccion = useCallback(() => {
    const sel = selectedCaptionIdsRef.current;
    if (sel.length === 0) return;
    pushHistorial();
    setCaptions((prev) => {
      // Set por la misma razón que en asignarHablante: borrar una selección
      // grande recorría todos los captions con O(k) comparación cada uno.
      const selSet = new Set(sel);
      const copy = prev.filter((c) => !selSet.has(c.id));
      captionsRef.current = copy;
      return copy;
    });
    setSelectedCaptionIds([]);
  }, [pushHistorial]);

  // ==== Selección múltiple ====
  const seleccionSola = useCallback((id: string) => {
    setSelectedCaptionIds([id]);
  }, []);

  const toggleSeleccion = useCallback((id: string) => {
    setSelectedCaptionIds((prev) =>
      prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id],
    );
  }, []);

  // Shift+click: rango desde el último seleccionado hasta el clickeado.
  // porTiempo=true usa el orden temporal (track); false usa el orden del array
  // (lista derecha).
  const seleccionRango = useCallback((id: string, porTiempo: boolean) => {
    setSelectedCaptionIds((prev) => {
      const anchor = prev[prev.length - 1];
      const caps = porTiempo ? sortedByStartRef.current : captionsRef.current;
      const iA = anchor ? caps.findIndex((c) => c.id === anchor) : -1;
      const iB = caps.findIndex((c) => c.id === id);
      if (iA === -1 || iB === -1) return [id];
      const [lo, hi] = iA < iB ? [iA, iB] : [iB, iA];
      return caps.slice(lo, hi + 1).map((c) => c.id);
    });
  }, []);

  const handleSelectCaption = useCallback(
    (id: string, shift: boolean, ctrl: boolean) => {
      if (shift) {
        seleccionRango(id, false);
        return;
      }
      if (ctrl) {
        toggleSeleccion(id);
        return;
      }
      seleccionSola(id);
      const cap = captionsRef.current.find((c) => c.id === id);
      const video = videoRef.current;
      if (cap && video) {
        video.currentTime = cap.inicio;
      }
    },
    [seleccionRango, toggleSeleccion, seleccionSola, videoRef],
  );

  // Mueve una selección de captions un deltaT (manteniendo cada duración).
  // cambios reasigna hablantes por id (corrimiento relativo del body drag);
  // ausente o vacío = solo tiempo. pushHistorial UNA vez: Ctrl+Z deshace el bloque.
  const moverCaptions = useCallback(
    (
      ids: string[],
      deltaT: number,
      cambios?: Map<string, string | null>,
    ) => {
      if (ids.length === 0) return;
      if (deltaT === 0 && (!cambios || cambios.size === 0)) return;
      pushHistorial();
      // Un Set por llamada: moverCaptions corre en el mouseup de cada body
      // drag y el .map recorre todos los captions. Con Array.includes() un
      // bloque grande (varios cientos de captions) era O(n·k) en el momento
      // en que el usuario ya soltó y espera ver el resultado.
      const idSet = new Set(ids);
      setCaptions((prev) => {
        const copy = prev.map((c) => {
          if (!idSet.has(c.id)) return c;
          const dur = c.fin - c.inicio;
          const nuevoInicio = Math.max(0, c.inicio + deltaT);
          const updated: Caption = {
            ...c,
            inicio: nuevoInicio,
            fin: nuevoInicio + dur,
          };
          if (cambios && cambios.has(c.id)) {
            updated.hablante_id = cambios.get(c.id) ?? null;
          }
          return updated;
        });
        captionsRef.current = copy;
        return copy;
      });
    },
    [pushHistorial],
  );

  function dividirCaptionEnPlayhead() {
    const video = videoRef.current;
    if (!video) return;
    const t = video.currentTime;
    const idx = currentCaptionIdxRef.current;
    const cap = captionsRef.current[idx];
    if (!cap || t <= cap.inicio || t >= cap.fin) return;
    pushHistorial();
    const izquierda: Caption = {
      ...cap,
      id: `cap-cut-${Date.now()}-l`,
      fin: t,
    };
    const derecha: Caption = {
      ...cap,
      id: `cap-cut-${Date.now()}-r`,
      inicio: t,
    };
    setCaptions((prev) => {
      const copy = prev
        .filter((c) => c.id !== cap.id)
        .concat([izquierda, derecha])
        .sort((a, b) => a.inicio - b.inicio);
      captionsRef.current = copy;
      return copy;
    });
    setSelectedCaptionIds([derecha.id]);
  }

  function actualizarTiempoCaption(
    id: string,
    campo: "inicio" | "fin",
    nuevoValor: number,
    duracionVideo = 0,
  ) {
    // Un solo push aunque se muevan inicio y fin juntos (el editor numérico
    // manda los dos): así un Ctrl+Z revierte el cambio completo.
    pushHistorial();
    setCaptions((prev) => {
      const copy = prev.map((c) => {
        if (c.id !== id) return c;
        // El clamp vive en una función pura testeada y compartida con el
        // editor numérico: no puede haber dos reglas distintas de "qué es un
        // tiempo válido" en la misma app.
        const { inicio, fin } = clampCaptionTiempos(
          campo === "inicio" ? nuevoValor : c.inicio,
          campo === "fin" ? nuevoValor : c.fin,
          duracionVideo,
          c,
        );
        return { ...c, inicio, fin };
      });
      captionsRef.current = copy;
      return copy;
    });
  }

  function handleEditorFocus() {
    if (skipEditorHistoryRef.current) {
      // Focus automático tras "Nuevo fragmento": el snapshot ya se pusheó
      // antes de agregar; no crear un paso de undo duplicado.
      skipEditorHistoryRef.current = false;
    } else if (
      editorPushedCaptionsRef.current !== captionsRef.current ||
      editorPushedHablantesRef.current !== hablantesRef.current
    ) {
      // Snapshot pre-edición (una sola vez por sesión de focus; un segundo
      // focus sin cambios no crea pasos de undo vacíos).
      pushHistorial();
      editorPushedCaptionsRef.current = captionsRef.current;
      editorPushedHablantesRef.current = hablantesRef.current;
    }
    const video = videoRef.current;
    if (!video) return;
    wasPlayingBeforeEditRef.current = !video.paused;
    if (!video.paused) video.pause();
  }

  function handleEditorBlur() {
    const video = videoRef.current;
    if (!video) return;
    if (wasPlayingBeforeEditRef.current) {
      video.play().catch(() => {});
    }
  }

  function handleEditorKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      (e.target as HTMLTextAreaElement).blur();
    }
  }

  return {
    captions,
    setCaptions,
    captionsRef,
    selectedCaptionIds,
    setSelectedCaptionIds,
    selectedCaptionIdsRef,
    sortedByStartRef,
    currentCaptionIdxRef,
    textEditorRef,
    pushHistorial,
    deshacer,
    rehacer,
    limpiarHistorial,
    asignarHablante,
    actualizarTextoCaption,
    actualizarTextos,
    duracionFragmento,
    agregarFragmento,
    pegarFragmento,
    eliminarCaption,
    eliminarSeleccion,
    seleccionSola,
    toggleSeleccion,
    seleccionRango,
    handleSelectCaption,
    moverCaptions,
    dividirCaptionEnPlayhead,
    actualizarTiempoCaption,
    handleEditorFocus,
    handleEditorBlur,
    handleEditorKeyDown,
  };
}
