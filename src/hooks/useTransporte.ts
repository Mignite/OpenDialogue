import { useCallback, useEffect, useRef, useState } from "react";
import type { Caption } from "../types";
import { formatTime, parseTimeInput } from "../utils/time";
import { VELOCIDADES_REPRODUCCION } from "../utils/constants";
import type { Ajustes } from "../utils/ajustes";

interface UseTransporteParams {
  videoRef: React.RefObject<HTMLVideoElement | null>;
  videoSrc: string;
  // Timeline-vista: togglePlay re-activa el follow y limpia el scroll manual.
  setAutoFollowing: React.Dispatch<React.SetStateAction<boolean>>;
  isScrollingManuallyRef: React.MutableRefObject<boolean>;
  // De useCaptions: navegación por caption y ciclado de simultáneos.
  sortedByStartRef: React.MutableRefObject<Caption[]>;
  selectedCaptionIdsRef: React.MutableRefObject<string[]>;
  setSelectedCaptionIds: React.Dispatch<React.SetStateAction<string[]>>;
  // Persistencia de la velocidad en ajustes.json (merge del lado de App).
  persistirAjustes: (parcial: Ajustes) => void;
}

// Slice 6 del monolito: transporte del playhead. Dueño del estado de
// reproducción (playheadTime/reproduciendo/velocidad/videoDuration) y de los
// saltos (togglePlay/saltar/saltarCaption/ciclado). Todos los callbacks que
// consume el keydown global (deps []) son estables: leen refs, nunca state.
export function useTransporte({
  videoRef,
  videoSrc,
  setAutoFollowing,
  isScrollingManuallyRef,
  sortedByStartRef,
  selectedCaptionIdsRef,
  setSelectedCaptionIds,
  persistirAjustes,
}: UseTransporteParams) {
  const [playheadTime, setPlayheadTime] = useState<number>(0);
  const [reproduciendo, setReproduciendo] = useState<boolean>(false);
  const [videoDuration, setVideoDuration] = useState<number>(0);
  const [timeInputValue, setTimeInputValue] = useState<string>("");
  const [editandoTiempo, setEditandoTiempo] = useState<boolean>(false);

  // Dual ref+state (patrón #1): el keydown global (deps []) lee la ref, el
  // <select> renderiza el state. playbackRate se re-aplica en loadedmetadata
  // porque el <video> lo resetea a 1 al cargar otro archivo.
  const [velocidad, setVelocidad] = useState(1);
  const velocidadRef = useRef(1);
  const wasPlayingBeforeSeekRef = useRef<boolean>(false);
  // Captions activos bajo el playhead: lo escribe el effect follow de App y
  // lo lee ciclarCaptionSimultaneo (ambos via refs, sin re-render).
  const matchingCaptionsRef = useRef<Caption[]>([]);

  // Solo se refleja play/pause en el botón: el video suena directo.
  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    const onPlay = () => setReproduciendo(true);
    const onPause = () => setReproduciendo(false);
    video.addEventListener("play", onPlay);
    video.addEventListener("pause", onPause);
    return () => {
      video.removeEventListener("play", onPlay);
      video.removeEventListener("pause", onPause);
    };
  }, [videoSrc]);

  const cambiarVelocidad = useCallback(
    (v: number) => {
      velocidadRef.current = v;
      setVelocidad(v);
      const video = videoRef.current;
      if (video) video.playbackRate = v;
      persistirAjustes({ velocidad: v });
    },
    [videoRef, persistirAjustes],
  );

  const pasoVelocidad = useCallback(
    (dir: 1 | -1) => {
      const idx = VELOCIDADES_REPRODUCCION.findIndex(
        (v) => Math.abs(v - velocidadRef.current) < 0.001,
      );
      const actual = idx === -1 ? VELOCIDADES_REPRODUCCION.indexOf(1) : idx;
      const nuevo = Math.min(
        VELOCIDADES_REPRODUCCION.length - 1,
        Math.max(0, actual + dir),
      );
      cambiarVelocidad(VELOCIDADES_REPRODUCCION[nuevo]);
    },
    [cambiarVelocidad],
  );

  // El <video> resetea playbackRate a 1 con cada archivo nuevo (loadedmetadata).
  const reaplicarVelocidad = useCallback(() => {
    const video = videoRef.current;
    if (video) video.playbackRate = velocidadRef.current;
  }, [videoRef]);

  const togglePlay = useCallback(() => {
    const video = videoRef.current;
    if (!video) return;
    if (video.paused) {
      video.play().catch(() => {});
      setAutoFollowing(true);
      isScrollingManuallyRef.current = false;
    } else {
      video.pause();
    }
  }, [videoRef, setAutoFollowing, isScrollingManuallyRef]);

  const saltar = useCallback(
    (delta: number) => {
      const video = videoRef.current;
      if (!video) return;
      const newTime = Math.max(
        0,
        Math.min(video.duration || Infinity, video.currentTime + delta),
      );
      video.currentTime = newTime;
      isScrollingManuallyRef.current = false;
    },
    [videoRef, isScrollingManuallyRef],
  );

  const saltarCaption = useCallback(
    (direccion: 1 | -1) => {
      const video = videoRef.current;
      if (!video) return;
      const caps = sortedByStartRef.current;
      if (caps.length === 0) return;
      const t = video.currentTime;

      let newTime: number;
      if (direccion === 1) {
        let lo = 0;
        let hi = caps.length;
        while (lo < hi) {
          const mid = (lo + hi) >> 1;
          if (caps[mid].inicio > t + 0.05) hi = mid;
          else lo = mid + 1;
        }
        newTime = lo < caps.length ? caps[lo].inicio + 0.01 : t;
      } else {
        let lo = 0;
        let hi = caps.length;
        while (lo < hi) {
          const mid = (lo + hi) >> 1;
          if (caps[mid].inicio < t - 0.05) lo = mid + 1;
          else hi = mid;
        }
        // Si el playhead está dentro o después del inicio del candidato, ese no
        // es el "anterior": retroceder uno más (Alt+← dentro de un caption va al
        // caption anterior, no reinicia el actual).
        if (lo > 0 && t >= caps[lo - 1].inicio) lo -= 1;
        newTime = lo > 0 ? caps[lo - 1].inicio + 0.01 : caps[0].inicio + 0.01;
      }
      video.currentTime = newTime;
      isScrollingManuallyRef.current = false;
    },
    [videoRef, sortedByStartRef, isScrollingManuallyRef],
  );

  const ciclarCaptionSimultaneo = useCallback(
    (direccion: 1 | -1) => {
      const actual = matchingCaptionsRef.current;
      if (actual.length <= 1) return;

      const sel = selectedCaptionIdsRef.current;
      const currentId = sel.length > 0 ? sel[sel.length - 1] : "";
      const idxActual = actual.findIndex((c) => c.id === currentId);
      const siguienteIdx =
        (idxActual + direccion + actual.length) % actual.length;
      setSelectedCaptionIds([actual[siguienteIdx].id]);
    },
    [matchingCaptionsRef, selectedCaptionIdsRef, setSelectedCaptionIds],
  );

  function handleTimeInputFocus() {
    setEditandoTiempo(true);
    setTimeInputValue(formatTime(playheadTime));
  }

  function handleTimeInputBlur() {
    setEditandoTiempo(false);
  }

  function handleTimeInputKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key !== "Enter") return;
    const secs = parseTimeInput(timeInputValue);
    const video = videoRef.current;
    if (secs !== null && video) {
      // Guardar estado de reproducción
      wasPlayingBeforeSeekRef.current = !video.paused;
      // Pausar si estaba reproduciendo
      if (!video.paused) {
        video.pause();
      }
      video.currentTime = Math.max(0, Math.min(video.duration || Infinity, secs));
      // Restaurar reproducción
      if (wasPlayingBeforeSeekRef.current) {
        setTimeout(() => {
          // El video pudo cambiar dentro de estos 50 ms (proyecto nuevo): sin
          // esta guarda, `video` sigue siendo el element del video anterior y
          // lo reanuda por detrás del nuevo.
          if (videoRef.current !== video) return;
          video.play().catch(() => {});
        }, 50);
      }
    }
    (e.target as HTMLInputElement).blur();
  }

  return {
    playheadTime,
    setPlayheadTime,
    reproduciendo,
    videoDuration,
    setVideoDuration,
    velocidad,
    velocidadRef,
    wasPlayingBeforeSeekRef,
    cambiarVelocidad,
    pasoVelocidad,
    reaplicarVelocidad,
    togglePlay,
    saltar,
    saltarCaption,
    ciclarCaptionSimultaneo,
    matchingCaptionsRef,
    timeInputValue,
    setTimeInputValue,
    editandoTiempo,
    handleTimeInputFocus,
    handleTimeInputBlur,
    handleTimeInputKeyDown,
  };
}
