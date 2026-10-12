import { describe, it, expect } from "vitest";
import { colorHablante } from "../colorHablante";
import { DEFAULT_PRESET_ASS } from "../constants";
import type { Hablante, PresetAss } from "../../types";

const spk = (over: Partial<Hablante> = {}): Hablante => ({
  id: "sp1", nombre: "Ana", tecla: "1", color: "#E85D4E", ...over,
});
const P_ROJO: PresetAss = { ...DEFAULT_PRESET_ASS, id: "p-rojo", nombre: "Rojo", color: "#E85D4E" };
const P_AZUL: PresetAss = { ...DEFAULT_PRESET_ASS, id: "p-azul", nombre: "Azul", color: "#4EA8E8" };

describe("colorHablante", () => {
  it("sin preset devuelve blanco", () => {
    expect(colorHablante(spk(), [])).toBe("#FFFFFF");
  });
  it("con presetId ausente devuelve blanco aunque haya presets", () => {
    expect(colorHablante(spk({ presetId: undefined }), [P_ROJO, P_AZUL])).toBe("#FFFFFF");
  });
  it("con presetId que no existe devuelve blanco (preset borrado a mano)", () => {
    expect(colorHablante(spk({ presetId: "p-fantasma" }), [P_ROJO])).toBe("#FFFFFF");
  });
  it("con preset devuelve el color del preset, NO el color guardado", () => {
    expect(colorHablante(spk({ presetId: "p-azul" }), [P_ROJO, P_AZUL])).toBe("#4EA8E8");
  });
  it("ignora por completo Hablante.color", () => {
    expect(colorHablante(spk({ color: "#FF00FF", presetId: "p-rojo" }), [P_ROJO])).toBe("#E85D4E");
  });
  it("el preset por defecto (blanco) da blanco", () => {
    expect(colorHablante(spk({ presetId: DEFAULT_PRESET_ASS.id }), [DEFAULT_PRESET_ASS])).toBe("#FFFFFF");
  });
});
