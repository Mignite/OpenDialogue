import { describe, it, expect } from "vitest";
import { buscarFinIslaAudio, picoEnRango } from "../audioIslands";

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

describe("picoEnRango", () => {
  // vol de 1s exacto a 15 vps: vol[i] = i/100.
  const vol = Array.from({ length: 15 }, (_, i) => i / 100);

  it("toma el pico de las ventanas intersectadas", () => {
    // [0.2s, 0.4s) cubre ventanas 3..6 → pico 0.05.
    expect(picoEnRango(vol, 0.2, 0.4)).toBeCloseTo(0.05, 6);
  });

  it("devuelve 0 fuera del audio o con entrada vacía", () => {
    expect(picoEnRango(vol, 5.0, 6.0)).toBe(0);
    expect(picoEnRango([], 0.0, 1.0)).toBe(0);
  });

  it("un rango menor a una ventana evalúa esa ventana", () => {
    // 0.005s dentro de la ventana 7 (0.07).
    expect(picoEnRango(vol, 0.47, 0.475)).toBeCloseTo(0.07, 6);
  });
});
