import { describe, it, expect } from "vitest";
import { buscarFinIslaAudio, compactarWaveform } from "../audioIslands";

const VPS = 15;

function silencio(seg: number): number[] {
  return new Array(Math.round(seg * VPS)).fill(0.005);
}

function pico(seg: number, v = 0.2): number[] {
  return new Array(Math.round(seg * VPS)).fill(v);
}

describe("buscarFinIslaAudio", () => {
  it("extiende el subtítulo hasta el fin de la isla cuando el playhead está sobre diálogo", () => {
    // 2s silencio, 3s diálogo, silencio. Playhead al inicio del diálogo.
    const vol = [...silencio(2), ...pico(3), ...silencio(2)];
    const fin = buscarFinIslaAudio(vol, 2.0);
    expect(fin).not.toBeNull();
    expect(fin!).toBeCloseTo(5.0, 1);
  });

  it("devuelve null cuando el playhead está en silencio (sin isla)", () => {
    const vol = [...silencio(5), ...pico(2), ...silencio(2)];
    expect(buscarFinIslaAudio(vol, 1.0)).toBeNull();
  });

  it("devuelve null con volumen vacío", () => {
    expect(buscarFinIslaAudio([], 1.0)).toBeNull();
  });

  it("devuelve null si el playhead está fuera del rango del análisis", () => {
    const vol = [...silencio(1), ...pico(2)];
    expect(buscarFinIslaAudio(vol, 10.0)).toBeNull();
  });

  it("respeta el mínimo de 1.5s para islas más cortas", () => {
    // 1s silencio, 0.5s diálogo, silencio → isla de 0.5s se extiende a 1.5s
    const vol = [...silencio(1), ...pico(0.5), ...silencio(2)];
    const fin = buscarFinIslaAudio(vol, 1.0);
    expect(fin).not.toBeNull();
    expect(fin!).toBeGreaterThanOrEqual(2.5 - 1e-6);
  });

  it("no supera los 5 segundos de duración", () => {
    // 1s silencio, 10s diálogo → fin recortado a inicio + 5
    const vol = [...silencio(1), ...pico(10)];
    const fin = buscarFinIslaAudio(vol, 1.0);
    expect(fin).not.toBeNull();
    expect(fin!).toBeLessThanOrEqual(6.0 + 1e-6);
  });

  it("salva micro-pausas dentro de la frase (histéresis)", () => {
    // Diálogo 1s, silencio 0.3s, diálogo 2s: la frase no se corta en la pausa
    const vol = [
      ...silencio(2),
      ...pico(1),
      ...silencio(0.3),
      ...pico(2),
      ...silencio(2),
    ];
    const fin = buscarFinIslaAudio(vol, 2.0);
    expect(fin).not.toBeNull();
    // Debe cubrir toda la frase (2 + 1 + 0.3 + 2 = 5.3s)
    expect(fin!).toBeGreaterThanOrEqual(5.2);
  });

  it("funciona con playhead en t=0 (sin ventana previa de fondo)", () => {
    const vol = [...pico(3), ...silencio(2)];
    const fin = buscarFinIslaAudio(vol, 0.0);
    expect(fin).not.toBeNull();
    expect(fin!).toBeCloseTo(3.0, 1);
  });
});

describe("compactarWaveform", () => {
  it("devuelve el mismo array cuando cabe en el máximo", () => {
    const vol = [0.1, 0.5, 0.3];
    expect(compactarWaveform(vol, 4096)).toEqual([0.1, 0.5, 0.3]);
  });

  it("topa el largo al máximo preservando picos", () => {
    // 100 ventanas, 10 de salida: cada píxel cubre 10 ventanas con un pico.
    const vol = new Array(100).fill(0.01);
    vol[5] = 0.9;
    vol[95] = 0.8;
    const out = compactarWaveform(vol, 10);
    expect(out).toHaveLength(10);
    expect(out[0]).toBe(0.9);
    expect(out[9]).toBe(0.8);
  });

  it("devuelve vacío con entrada vacía", () => {
    expect(compactarWaveform([], 4096)).toEqual([]);
  });

  it("un maxW de 1 colapsa todo al pico global", () => {
    expect(compactarWaveform([0.1, 0.9, 0.3], 1)).toEqual([0.9]);
  });
});
