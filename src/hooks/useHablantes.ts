import { useCallback, useMemo } from "react";
import type { Dispatch, SetStateAction } from "react";
import type { Caption, Hablante } from "../types";
import { PALETA } from "../utils/constants";

export interface HablantesDeps {
  hablantes: Hablante[];
  setHablantes: Dispatch<SetStateAction<Hablante[]>>;
  hablantesRef: { current: Hablante[] };
  captionsRef: { current: Caption[] };
  setCaptions: Dispatch<SetStateAction<Caption[]>>;
  pushHistorial: () => void;
}

// Hablantes: CRUD + speakerMap memoizado. El ref `hablantesRef` se queda en
// App (lo leen ~15 sitios del drag/editor; su identidad nunca cambia) y el
// mirror bare lo sigue sincronizando ahí. Los callbacks que escriben el ref
// a mano (actualizar/cambiarColor) lo hacen igual que antes, para listeners
// síncronos entre el setState y el effect.
export function useHablantes({
  hablantes,
  setHablantes,
  hablantesRef,
  captionsRef,
  setCaptions,
  pushHistorial,
}: HablantesDeps) {
  const speakerMap = useMemo(() => {
    const map = new Map<string, Hablante>();
    for (const h of hablantes) {
      map.set(h.id, h);
    }
    return map;
  }, [hablantes]);

  const agregarHablante = useCallback(() => {
    pushHistorial();
    setHablantes((prev) => {
      if (prev.length >= 9) return prev;
      const usadas = new Set(prev.map((h) => h.tecla));
      let tecla = "1";
      for (let i = 1; i <= 9; i++) {
        if (!usadas.has(String(i))) {
          tecla = String(i);
          break;
        }
      }
      const nuevo: Hablante = {
        id: `sp-${Date.now()}`,
        nombre: "",
        tecla,
        color: PALETA[prev.length % PALETA.length],
      };
      return [...prev, nuevo];
    });
  }, [pushHistorial, setHablantes]);

  const actualizarHablante = useCallback(
    (id: string, campo: keyof Hablante, valor: string) => {
      setHablantes((prev) => {
        const newHablantes = prev.map((h) =>
          h.id === id ? { ...h, [campo]: valor } : h,
        );
        hablantesRef.current = newHablantes;
        return newHablantes;
      });
    },
    [setHablantes, hablantesRef],
  );

  const cambiarColorHablante = useCallback(
    (id: string, color: string) => {
      pushHistorial();
      setHablantes((prev) => {
        const newHablantes = prev.map((h) =>
          h.id === id ? { ...h, color } : h,
        );
        hablantesRef.current = newHablantes;
        return newHablantes;
      });
    },
    [pushHistorial, hablantesRef],
  );

  const eliminarHablante = useCallback(
    (id: string) => {
      pushHistorial();
      setHablantes((prev) => prev.filter((h) => h.id !== id));
      setCaptions((prev) => {
        const newCaptions = prev.map((c) =>
          c.hablante_id === id ? { ...c, hablante_id: null } : c,
        );
        captionsRef.current = newCaptions;
        return newCaptions;
      });
    },
    [pushHistorial, captionsRef, setCaptions],
  );

  return {
    hablantes,
    setHablantes,
    speakerMap,
    agregarHablante,
    actualizarHablante,
    cambiarColorHablante,
    eliminarHablante,
  };
}
