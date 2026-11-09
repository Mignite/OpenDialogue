import { describe, it, expect } from "vitest";
import { buscarTramoAudio, picoEnRango } from "../audioIslands";

const VPS = 15;

function silencio(seg: number): number[] {
  return new Array(Math.round(seg * VPS)).fill(0.005);
}

function pico(seg: number, v = 0.2): number[] {
  return new Array(Math.round(seg * VPS)).fill(v);
}

describe("buscarTramoAudio", () => {
  it("cubre la frase entera cuando el playhead está en el silencio previo", () => {
    // 1s silencio, 3s habla, silencio. Playhead 0.7s antes de la voz.
    const vol = [...silencio(1), ...pico(3), ...silencio(2)];
    const t = buscarTramoAudio(vol, 0.3);
    expect(t).not.toBeNull();
    expect(t!.inicio).toBeCloseTo(0.3, 2); // arranca donde está el playhead
    expect(t!.fin).toBeCloseTo(4.25, 1); // voz (4.0) + 0.25s de cola
  });

  it("deja como máximo 0.3s de aire antes del onset", () => {
    const vol = [...silencio(1), ...pico(3), ...silencio(2)];
    const t = buscarTramoAudio(vol, 0.75); // onset en 1.0
    expect(t!.inicio).toBeCloseTo(0.7, 2);
  });

  it("nunca arranca después del playhead", () => {
    const vol = [...silencio(1), ...pico(3), ...silencio(2)];
    const t = buscarTramoAudio(vol, 0.5);
    expect(t!.inicio).toBeLessThanOrEqual(0.5);
  });

  it("retrocede al onset cuando el playhead cae dentro de la frase", () => {
    const vol = [...silencio(1), ...pico(3), ...silencio(2)];
    const t = buscarTramoAudio(vol, 1.4);
    expect(t!.inicio).toBeCloseTo(1.0, 1);
    expect(t!.fin).toBeCloseTo(4.25, 1);
  });

  it("no retrocede más de 0.5s al onset", () => {
    const vol = [...silencio(1), ...pico(3), ...silencio(2)];
    const t = buscarTramoAudio(vol, 2.0);
    expect(t!.inicio).toBeGreaterThanOrEqual(1.45);
    expect(t!.inicio).toBeLessThanOrEqual(1.6);
    expect(t!.fin).toBeCloseTo(4.25, 1);
  });

  it("detecta la isla aunque el fondo reciente sea habla", () => {
    // Playhead 1.2s dentro de la frase: la media de los 2s previos se infla
    // con la voz y el umbral tapa el habla. El fondo por percentil 25 no.
    const vol = [...silencio(1.5), ...pico(3), ...silencio(2)];
    const t = buscarTramoAudio(vol, 2.7);
    expect(t).not.toBeNull();
    expect(t!.fin).toBeCloseTo(4.75, 1);
  });

  it("devuelve null cuando no hay onset en los 2s siguientes", () => {
    const vol = [...silencio(1), ...pico(2), ...silencio(5), ...pico(2)];
    expect(buscarTramoAudio(vol, 4.0)).toBeNull();
  });

  it("devuelve null con volumen vacío", () => {
    expect(buscarTramoAudio([], 1.0)).toBeNull();
  });

  it("devuelve null si el playhead está fuera del rango del análisis", () => {
    const vol = [...silencio(1), ...pico(2)];
    expect(buscarTramoAudio(vol, 10.0)).toBeNull();
  });

  it("un fragmento corto mide la frase + 0.25s (sin piso de duración)", () => {
    // 0.53s de habla → 0.78s de fragmento. Antes el mínimo lo estiraba a 1.5s.
    const vol = [...silencio(1), ...pico(0.5), ...silencio(2)];
    const t = buscarTramoAudio(vol, 0.5);
    expect(t).not.toBeNull();
    expect(t!.fin).toBeCloseTo(1.783, 2);
  });

  it("un blip de una sola ventana no se infla", () => {
    const vol = [...silencio(1), 0.2, ...silencio(2)];
    const t = buscarTramoAudio(vol, 1.0);
    expect(t!.inicio).toBeCloseTo(1.0, 2);
    expect(t!.fin).toBeCloseTo(1.317, 2);
  });

  it("cierra con 0.25s de cola tras el último sonido", () => {
    // Voz de 0.6s (1.0 → 1.6): 1.6 + 0.25 = 1.85, por encima del mínimo.
    const vol = [...silencio(1), ...pico(0.6), ...silencio(2)];
    const t = buscarTramoAudio(vol, 1.0);
    expect(t!.inicio).toBeCloseTo(1.0, 2);
    expect(t!.fin).toBeCloseTo(1.85, 2);
    expect(t!.fin - 1.6).toBeLessThanOrEqual(0.3);
  });

  it("una frase corta no deja 0.9s de silencio al final (caso reportado)", () => {
    // Voz de 0.6s con el playhead justo en el onset: antes el mínimo de 1.5s
    // estiraba el fragmento hasta 0.9s de silencio detrás de la voz.
    const vol = [...silencio(2), ...pico(0.6), ...silencio(3)];
    const t = buscarTramoAudio(vol, 2.0);
    expect(t!.inicio).toBeCloseTo(2.0, 2);
    expect(t!.fin - 2.6).toBeLessThanOrEqual(0.3);
    expect(t!.fin - t!.inicio).toBeLessThan(1.0);
  });

  it("no cubre más de 5s de habla desde el onset", () => {
    const vol = [...silencio(1), ...pico(10)];
    const t = buscarTramoAudio(vol, 0.5);
    expect(t).not.toBeNull();
    expect(t!.fin).toBeLessThanOrEqual(6.0 + 1e-6);
  });

  it("salva micro-pausas dentro de la frase (histéresis)", () => {
    const vol = [
      ...silencio(2),
      ...pico(1),
      ...silencio(0.3),
      ...pico(2),
      ...silencio(2),
    ];
    const t = buscarTramoAudio(vol, 2.0);
    expect(t).not.toBeNull();
    expect(t!.fin).toBeGreaterThanOrEqual(5.2);
  });

  it("funciona con playhead en t=0 (sin ventana previa de fondo)", () => {
    const vol = [...pico(3), ...silencio(2)];
    const t = buscarTramoAudio(vol, 0.0);
    expect(t).not.toBeNull();
    expect(t!.inicio).toBe(0);
    expect(t!.fin).toBeCloseTo(3.25, 1);
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
