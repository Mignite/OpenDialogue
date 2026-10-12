import type { CSSProperties } from "react";
import type { PresetAss } from "../types";
import { ASS_EM_PREVIEW } from "./constants";

export interface RecuadroVideo {
  left: number;
  top: number;
  width: number;
  height: number;
}

export function estiloLineaPreview(preset: PresetAss, escala: number): CSSProperties {
  // Sin clamp: en preview chico o grande el tamaño debe ser el real
  // escalado, como en el render. ASS_EM compensa celda ASS vs em CSS.
  const px = preset.fontsize * escala * ASS_EM_PREVIEW;
  const borde = Math.max(1, preset.outline * escala);
  return {
    fontFamily: `"${preset.fontname}", sans-serif`,
    fontSize: px,
    color: preset.color,
    textShadow: `0 0 ${borde}px ${preset.outlineColor}, 0 1px 2px rgba(0,0,0,.8)`,
  };
}

export function estiloCajaPreview(
  preset: PresetAss,
  recuadro: RecuadroVideo | null,
  escala: number,
): CSSProperties {
  const al = preset.alignment;
  const vertical = al >= 7 ? "top" : al >= 4 ? "middle" : "bottom";
  const horizontal =
    al === 1 || al === 4 || al === 7 ? "left"
    : al === 3 || al === 6 || al === 9 ? "right"
    : "center";
  return {
    // Caja = video visible (no el elemento con letterbox); sin recuadro,
    // vale el inset:0 del CSS. Márgenes del preset escalados, no % fijos.
    ...(recuadro
      ? {
          left: recuadro.left,
          top: recuadro.top,
          width: recuadro.width,
          height: recuadro.height,
        }
      : {}),
    justifyContent:
      horizontal === "left" ? "flex-start"
      : horizontal === "right" ? "flex-end"
      : "center",
    textAlign: horizontal as "left" | "center" | "right",
    alignItems: vertical === "top" ? "flex-start" : vertical === "middle" ? "center" : "flex-end",
    paddingLeft: preset.marginL * escala,
    paddingRight: preset.marginR * escala,
    paddingBottom: vertical === "bottom" ? preset.marginV * escala : undefined,
    paddingTop: vertical === "top" ? preset.marginV * escala : undefined,
  };
}
