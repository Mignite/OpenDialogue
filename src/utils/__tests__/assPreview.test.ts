import { describe, it, expect } from "vitest";
import { estiloLineaPreview, estiloCajaPreview } from "../assPreview";
import { DEFAULT_PRESET_ASS, ASS_EM_PREVIEW } from "../constants";
import type { PresetAss } from "../../types";

const P = (over: Partial<PresetAss> = {}): PresetAss => ({ ...DEFAULT_PRESET_ASS, ...over });
const rec = { left: 10, top: 20, width: 600, height: 340 };

describe("estiloLineaPreview", () => {
  it("escala el tamaño con ASS_EM_PREVIEW (celda ASS vs em CSS)", () => {
    const s = estiloLineaPreview(P({ fontsize: 48 }), 1);
    expect(s.fontSize).toBeCloseTo(48 * ASS_EM_PREVIEW);
  });
  it("el borde escala pero nunca baja de 1px", () => {
    expect(estiloLineaPreview(P({ outline: 0 }), 0.1).textShadow).toContain("1px");
  });
});

describe("estiloCajaPreview", () => {
  it("alignment 2 = abajo al centro", () => {
    const s = estiloCajaPreview(P({ alignment: 2 }), rec, 1);
    expect(s.justifyContent).toBe("center");
    expect(s.alignItems).toBe("flex-end");
  });
  it("alignment 8 = arriba al centro", () => {
    const s = estiloCajaPreview(P({ alignment: 8 }), rec, 1);
    expect(s.alignItems).toBe("flex-start");
    expect(s.justifyContent).toBe("center");
  });
  it("alignment 1 = abajo a la izquierda", () => {
    expect(estiloCajaPreview(P({ alignment: 1 }), rec, 1).justifyContent).toBe("flex-start");
  });
  it("alignment 3 = abajo a la derecha", () => {
    expect(estiloCajaPreview(P({ alignment: 3 }), rec, 1).justifyContent).toBe("flex-end");
  });
  it("los márgenes verticales escalan con la escala", () => {
    const s = estiloCajaPreview(P({ alignment: 2, marginV: 40 }), rec, 0.5);
    expect(s.paddingBottom).toBe(20);
    expect(s.paddingTop).toBeUndefined();
  });
  it("alignment 5 (medio) no aplica marginV arriba ni abajo", () => {
    const s = estiloCajaPreview(P({ alignment: 5, marginV: 40 }), rec, 1);
    expect(s.paddingTop).toBeUndefined();
    expect(s.paddingBottom).toBeUndefined();
  });
  it("con recuadro null no fuerza la caja (deja el inset del CSS)", () => {
    const s = estiloCajaPreview(P(), null, 1);
    expect(s.left).toBeUndefined();
    expect(s.width).toBeUndefined();
  });
  it("con recuadro posiciona sobre el video visible, no el elemento", () => {
    const s = estiloCajaPreview(P(), rec, 1);
    expect(s.left).toBe(10);
    expect(s.width).toBe(600);
  });
});
