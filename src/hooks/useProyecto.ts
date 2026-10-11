import { useCallback, useRef, useState } from "react";
import type { Dispatch, SetStateAction } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save, ask } from "@tauri-apps/plugin-dialog";
import type { Caption, Hablante, Proyecto, PresetAss } from "../types";
import { parseSrt } from "../utils/srt";
import {
  parseAutosubsTxt,
  asignarHablantesPorTexto,
  hablantesDesdeNombres,
} from "../utils/autosubs";
import {
  parseAss,
  presetDesdeEstilos,
} from "../utils/ass";
import {
  cargarPresetsAss,
  nuevoPreset,
} from "../utils/assPresets";
import { t } from "../i18n";

export interface ProyectoRefs {
  captionsRef: { current: Caption[] };
  hablantesRef: { current: Hablante[] };
  videoPathRef: { current: string };
  videoRef: { current: HTMLVideoElement | null };
  playheadPendienteRef: { current: number | null };
  windowStartRef: { current: number };
  windowTargetRef: { current: number };
  windowSecondsRef: { current: number };
}

export interface ProyectoSetters {
  setCaptions: Dispatch<SetStateAction<Caption[]>>;
  setHablantes: Dispatch<SetStateAction<Hablante[]>>;
  setSelectedCaptionIds: Dispatch<SetStateAction<string[]>>;
  setVideoSrc: Dispatch<SetStateAction<string>>;
  setVideoPath: Dispatch<SetStateAction<string>>;
  setVideoNoEncontrado: Dispatch<SetStateAction<boolean>>;
  setRutaFaltante: Dispatch<SetStateAction<string>>;
  setVideoDuration: Dispatch<SetStateAction<number>>;
}

export interface ProyectoServicios {
  pushHistorial: () => void;
  limpiarHistorial: () => void;
  notify: (msg: string) => void;
  persistirPresetsAss: (
    presets: PresetAss[],
    opts?: { ya?: boolean },
  ) => Promise<void>;
  cargarVideo: (path: string) => void;
  reiniciarOnda: () => void;
  updateScrollbarThumb: (ws: number, wSec: number, dur: number) => void;
}

// Proyecto y archivos: guardar/cargar/nuevo, abrir video/SRT, imports
// (auto-subs, .ass). Dueño de rutaProyecto + flags dirty (isDirty,
// ignoreNextChange, hayCambios): los 5 puntos que los tocan viven acá.
// Todo callback es useCallback-estable (solo refs, setters estables y
// funciones de módulo): el effect del menú nativo (deps []) los captura una
// vez, igual que antes con las function declarations.
export function useProyecto(
  refs: ProyectoRefs,
  setters: ProyectoSetters,
  servicios: ProyectoServicios,
) {
  const {
    captionsRef,
    hablantesRef,
    videoPathRef,
    videoRef,
    playheadPendienteRef,
    windowStartRef,
    windowTargetRef,
    windowSecondsRef,
  } = refs;
  const {
    setCaptions,
    setHablantes,
    setSelectedCaptionIds,
    setVideoSrc,
    setVideoPath,
    setVideoNoEncontrado,
    setRutaFaltante,
    setVideoDuration,
  } = setters;
  const {
    pushHistorial,
    limpiarHistorial,
    notify,
    persistirPresetsAss,
    cargarVideo,
    reiniciarOnda,
    updateScrollbarThumb,
  } = servicios;

  const [rutaProyecto, setRutaProyecto] = useState<string>("");
  const rutaProyectoRef = useRef("");
  const isDirtyRef = useRef(false);
  const [hayCambios, setHayCambios] = useState(false);
  const ignoreNextChangeRef = useRef(true);

  // Marca sucio tras un cambio externo (el effect que observa
  // captions/hablantes/rutaProyecto vive en App). Consume el ignore de un
  // solo uso que dejan guardar/cargar/nuevo/imports.
  const marcarSucio = useCallback((hayContenido: boolean) => {
    if (ignoreNextChangeRef.current) {
      ignoreNextChangeRef.current = false;
      return;
    }
    if (hayContenido) {
      isDirtyRef.current = true;
      setHayCambios(true);
    }
  }, []);

  const limpiarBanderas = useCallback(() => {
    ignoreNextChangeRef.current = true;
    isDirtyRef.current = false;
    setHayCambios(false);
  }, []);

  // Una carga de CONTENIDO (SRT, auto-subs, .ass) no es abrir otro proyecto:
  // reemplaza los captions del proyecto actual, así que la memoria pasa a
  // divergir del archivo en disco. Eso es un cambio sin guardar real, y
  // limpiarlo dejaba dos trampas: cargar un SRT sobre un proyecto guardado
  // descartaba tanto el SRT recién cargado como las ediciones previas sin que
  // al cerrar la app apareciera aviso alguno. El undo recuperaba el estado
  // anterior, pero solo mientras la app siguiera abierta. Marcarlo sucio
  // mantiene ese aviso.
  //
  // Sin proyecto guardado no hay archivo con el cual divergir, así que el
  // proyecto queda limpio (el statusBar ya muestra "Proyecto sin guardar").
  const limpiarBanderasTrasCarga = useCallback(() => {
    ignoreNextChangeRef.current = true;
    if (rutaProyectoRef.current) {
      isDirtyRef.current = true;
      setHayCambios(true);
    } else {
      isDirtyRef.current = false;
      setHayCambios(false);
    }
  }, []);

  const cargarSrtDesdeRuta = useCallback(
    async (path: string) => {
      try {
        const contenido = await invoke<string>("leer_archivo_texto", {
          ruta: path,
        });
        const parsed = parseSrt(contenido);
        pushHistorial();
        limpiarBanderasTrasCarga();
        setCaptions(parsed);
      } catch (err) {
        console.error("Error cargando SRT:", err);
        notify(t("err.loadSrt", { error: String(err) }));
      }
    },
    [pushHistorial, limpiarBanderasTrasCarga, setCaptions, notify],
  );

  const handleAbrirVideo = useCallback(async () => {
    try {
      const path = await open({
        multiple: false,
        filters: [{ name: t("dialog.filterVideo"), extensions: ["mp4", "mov", "mkv"] }],
      });
      if (path) {
        cargarVideo(path as string);
      }
    } catch (err) {
      console.error("[ERROR handleAbrirVideo] Error abriendo diálogo:", err);
      notify(t("err.openVideo", { error: String(err) }));
    }
  }, [cargarVideo, notify]);

  const handleAbrirSrt = useCallback(async () => {
    try {
      const path = await open({
        multiple: false,
        filters: [{ name: t("dialog.filterSubtitles"), extensions: ["srt"] }],
      });
      if (path) await cargarSrtDesdeRuta(path as string);
    } catch (err) {
      console.error("Error abriendo diálogo SRT:", err);
      notify(t("err.openSrtDialog", { error: String(err) }));
    }
  }, [cargarSrtDesdeRuta, notify]);

  // Importa el par SRT+TXT de auto-subs: tiempos del SRT, hablantes de los
  // turnos "Speaker N" del TXT (match secuencial por texto en autosubs.ts).
  // Primero el SRT; el TXT gemelo (misma carpeta, misma base) se busca solo
  // y si no está se pide en un segundo diálogo.
  const handleImportarAutosubs = useCallback(async () => {
    try {
      const srtPath = await open({
        multiple: false,
        filters: [{ name: t("dialog.filterSubtitles"), extensions: ["srt"] }],
      });
      if (!srtPath) return;
      const contenidoSrt = await invoke<string>("leer_archivo_texto", {
        ruta: srtPath,
      });
      const cues = parseSrt(contenidoSrt);
      if (cues.length === 0) return;
      const gemelo = (srtPath as string).replace(/\.srt$/i, ".txt");
      let txtPath: string | null = null;
      try {
        const existe: boolean = await invoke("existe_archivo", { ruta: gemelo });
        if (existe) txtPath = gemelo;
      } catch {
        txtPath = null;
      }
      if (!txtPath) {
        txtPath = (await open({
          multiple: false,
          filters: [{ name: t("dialog.filterText"), extensions: ["txt"] }],
        })) as string | null;
      }
      if (!txtPath) return;
      const contenidoTxt = await invoke<string>("leer_archivo_texto", {
        ruta: txtPath,
      });
      const turnos = parseAutosubsTxt(contenidoTxt);
      const numeros = asignarHablantesPorTexto(cues, turnos);
      const hablantes = hablantesDesdeNombres(
        [...new Set(numeros.filter((n): n is string => n !== null))].sort(),
      );
      const idPorNumero = new Map(
        hablantes.map((h) => [h.nombre.replace(/^Hablante /, ""), h.id]),
      );
      pushHistorial();
      limpiarBanderasTrasCarga();
      setHablantes(hablantes);
      setCaptions(
        cues.map((c, i) => ({
          ...c,
          hablante_id: numeros[i] !== null ? (idPorNumero.get(numeros[i] as string) ?? null) : null,
        })),
      );
      setSelectedCaptionIds([]);
    } catch (err) {
      console.error("Error importando auto-subs:", err);
      notify(t("err.importAutosubs", { error: String(err) }));
    }
  }, [pushHistorial, limpiarBanderasTrasCarga, setHablantes, setCaptions, setSelectedCaptionIds, notify]);

  const guardarProyectoEnRuta = useCallback(
    async (path: string) => {
      const proyecto: Proyecto = {
        ruta_video: videoPathRef.current,
        hablantes: hablantesRef.current,
        captions: captionsRef.current,
        playhead: videoRef.current?.currentTime ?? 0,
      };

      try {
        await invoke("guardar_proyecto", { ruta: path, proyecto });
        if (rutaProyectoRef.current !== path) {
          ignoreNextChangeRef.current = true;
        }
        isDirtyRef.current = false;
        setHayCambios(false);
        setRutaProyecto(path);
        rutaProyectoRef.current = path;
      } catch (err) {
        console.error("ERROR al guardar:", err);
        // Sin esto un guardado fallido (disco lleno, ruta sin permisos) no
        // dejaba ninguna señal: el statusBar seguía diciendo "Guardado".
        notify(t("err.saveProject", { error: String(err) }));
      }
    },
    [captionsRef, hablantesRef, videoPathRef, videoRef, notify],
  );

  const handleGuardarComo = useCallback(async () => {
    try {
      const path = await save({
        filters: [{ name: t("dialog.filterProject"), extensions: ["json"] }],
      });
      if (!path) return;
      await guardarProyectoEnRuta(path as string);
    } catch (err) {
      console.error("Error abriendo diálogo de guardado:", err);
      notify(t("err.saveDialog", { error: String(err) }));
    }
  }, [guardarProyectoEnRuta, notify]);

  const handleGuardar = useCallback(async () => {
    if (rutaProyectoRef.current) {
      await guardarProyectoEnRuta(rutaProyectoRef.current);
    } else {
      await handleGuardarComo();
    }
  }, [guardarProyectoEnRuta, handleGuardarComo]);

  const handleCargarProyecto = useCallback(async () => {
    if (isDirtyRef.current) {
      const ok = await ask(
        t("app.confirm.openProject"),
        { title: t("app.confirm.unsavedTitle"), kind: "warning" },
      );
      if (!ok) return;
    }
    const path = await open({
      multiple: false,
      filters: [{ name: t("dialog.filterProject"), extensions: ["json"] }],
    });
    if (!path) return;

    try {
      const proyecto: Proyecto = await invoke("cargar_proyecto", {
        ruta: path,
      });
      limpiarBanderas();
      // El historial es del proyecto anterior: sus snapshots restauraban
      // captions/hablantes de otro documento sobre estos (con el video y la
      // ruta ya cambiados) y un guardado posterior mezclaba ambos.
      limpiarHistorial();
      setRutaProyecto(path as string);
      setCaptions(proyecto.captions || []);
      setHablantes(proyecto.hablantes || []);
      setSelectedCaptionIds([]);
      // Descartar waveform/duración del proyecto anterior
      reiniciarOnda();
      setVideoDuration(0);

      const existe: boolean = await invoke("existe_archivo", {
        ruta: proyecto.ruta_video,
      });
      playheadPendienteRef.current = proyecto.playhead ?? 0;
      if (existe) {
        cargarVideo(proyecto.ruta_video);
      } else {
        setRutaFaltante(proyecto.ruta_video);
        setVideoNoEncontrado(true);
      }
    } catch (err) {
      console.error("Error cargando proyecto:", err);
      notify(t("err.loadProject", { error: String(err) }));
    }
  }, [
    limpiarBanderas,
    limpiarHistorial,
    setRutaProyecto,
    setCaptions,
    setHablantes,
    setSelectedCaptionIds,
    reiniciarOnda,
    setVideoDuration,
    playheadPendienteRef,
    cargarVideo,
    setRutaFaltante,
    setVideoNoEncontrado,
    notify,
  ]);

  const handleNuevoProyecto = useCallback(async () => {
    if (isDirtyRef.current) {
      const ok = await ask(t("app.confirm.newProject"), {
        title: t("app.confirm.unsavedTitle"),
        kind: "warning",
      });
      if (!ok) return;
    }
    limpiarBanderas();
    limpiarHistorial();
    setVideoSrc("");
    setVideoPath("");
    setRutaProyecto("");
    setVideoNoEncontrado(false);
    setCaptions([]);
    setHablantes([]);
    setSelectedCaptionIds([]);
    playheadPendienteRef.current = null;
    // Descartar análisis en vuelo, waveform y duración del proyecto anterior
    reiniciarOnda();
    setVideoDuration(0);
    windowStartRef.current = 0;
    windowTargetRef.current = 0;
    updateScrollbarThumb(0, windowSecondsRef.current, 0);
  }, [
    limpiarBanderas,
    limpiarHistorial,
    setVideoSrc,
    setVideoPath,
    setVideoNoEncontrado,
    setCaptions,
    setHablantes,
    setSelectedCaptionIds,
    reiniciarOnda,
    setVideoDuration,
    windowStartRef,
    windowTargetRef,
    windowSecondsRef,
    updateScrollbarThumb,
  ]);

  const handleCargarAss = useCallback(async () => {
    try {
      const path = await open({
        multiple: false,
        filters: [{ name: t("dialog.filterAss"), extensions: ["ass"] }],
      });
      if (!path) return;
      const contenido = await invoke<string>("leer_archivo_texto", { ruta: path });
      const resultado = parseAss(contenido);
      if (resultado.captions.length === 0) return;

      // El import crea un preset desde el Style Default del archivo: es la vía
      // para traer estilos de un .ass de Premiere sin tipearlos.
      const nombrePreset =
        (path as string).split(/[\\/]/).pop()?.replace(/\.ass$/i, "") ?? "Importado";
      const nuevo =
        presetDesdeEstilos(
          resultado.styles,
          nombrePreset,
          `preset-import-${Date.now().toString(36)}`,
        ) ?? nuevoPreset({ nombre: nombrePreset });
      const actuales = await cargarPresetsAss();
      await persistirPresetsAss(
        [...actuales.filter((p) => p.id !== nuevo.id), nuevo],
        { ya: true },
      );

      // Mismo patrón que cargarSrtDesdeRuta: la carga deja el proyecto limpio.
      // El push va ANTES de limpiarBanderas para que el snapshot capture el
      // estado previo al import: sin él el Ctrl+Z saltaba a un snapshot más
      // viejo y el import de .ass no era deshacible.
      pushHistorial();
      limpiarBanderasTrasCarga();
      setHablantes(resultado.hablantes);
      setCaptions(resultado.captions);
      setSelectedCaptionIds([]);
      notify(
        t("assExport.importDone", {
          count: resultado.captions.length,
          speakers: resultado.hablantes.length,
        }) +
          " " +
          t("assExport.importPresetCreated", { name: nombrePreset }),
      );
    } catch (err) {
      console.error("Error importando .ass:", err);
      notify(t("assExport.importError", { error: String(err) }));
    }
  }, [
    persistirPresetsAss,
    pushHistorial,
    limpiarBanderasTrasCarga,
    setHablantes,
    setCaptions,
    setSelectedCaptionIds,
    notify,
  ]);

  return {
    rutaProyecto,
    rutaProyectoRef,
    hayCambios,
    isDirtyRef,
    marcarSucio,
    cargarSrtDesdeRuta,
    handleAbrirVideo,
    handleAbrirSrt,
    handleImportarAutosubs,
    guardarProyectoEnRuta,
    handleGuardarComo,
    handleGuardar,
    handleCargarProyecto,
    handleNuevoProyecto,
    handleCargarAss,
  };
}
