import { useCallback, useEffect, useRef, useState } from "react";
import type { Dispatch, SetStateAction } from "react";
import type { Caption, Hablante } from "../types";
import { buscarFinIslaAudio } from "../utils/audioIslands";
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

  function asignarHablante(hablanteId: string) {
    const sel = selectedCaptionIdsRef.current;
    if (sel.length > 0) {
      pushHistorial();
      setCaptions((prev) => {
        const copy = prev.map((c) =>
          sel.includes(c.id) ? { ...c, hablante_id: hablanteId } : c,
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
    // El focus disparará handleEditorFocus; ya se pusheó el snapshot pre-add
    skipEditorHistoryRef.current = true;
    setTimeout(() => textEditorRef.current?.focus(), 30);
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
      const copy = prev.filter((c) => !sel.includes(c.id));
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
      setCaptions((prev) => {
        const copy = prev.map((c) => {
          if (!ids.includes(c.id)) return c;
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
  ) {
    pushHistorial();
    setCaptions((prev) => {
      const copy = prev.map((c) => {
        if (c.id === id) {
          const updated = { ...c, [campo]: Math.max(0, nuevoValor) };
          // Asegurar que inicio < fin
          if (campo === "inicio" && updated.inicio >= updated.fin) {
            updated.inicio = updated.fin - 0.1;
          }
          if (campo === "fin" && updated.fin <= updated.inicio) {
            updated.fin = updated.inicio + 0.1;
          }
          return updated;
        }
        return c;
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
