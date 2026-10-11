import { describe, it, expect } from "vitest";
import { clampCaptionTiempos } from "../captionTiempos";

describe("clampCaptionTiempos", () => {
  const base = { inicio: 10, fin: 13 };

  it("deja los tiempos validos sin cambios", () => {
    expect(clampCaptionTiempos(10, 13, 0)).toEqual({ inicio: 10, fin: 13 });
  });

  it("nunca deja el inicio en negativo", () => {
    expect(clampCaptionTiempos(-5, 13, 0)).toEqual({ inicio: 0, fin: 13 });
  });

  it("mantiene la duracion al mover el inicio mas alla del fin", () => {
    // Mover el inicio a 20 con fin en 13 no puede quedar inicio=20 fin=13
    // (duracion negativa). Lo que el usuario quiere al arrastrar el borde
    // izquierdo mas alla del derecho es conservar la duracion del caption.
    const r = clampCaptionTiempos(20, 13, 0, base);
    expect(r.fin - r.inicio).toBeCloseTo(3);
    expect(r.inicio).toBe(20);
  });

  it("impone una duracion minima para que el caption siga siendo usable", () => {
    const r = clampCaptionTiempos(10, 10.1, 0, base);
    // Comparacion con tolerancia: en coma flotante 10 + 0.1 puede dar
    // 10.09999..., y el test debe fijar el minimo, no la representacion.
    expect(r.fin - r.inicio).toBeGreaterThanOrEqual(0.1 - 1e-9);
  });

  it("respeta la duracion del video como tope del fin", () => {
    const r = clampCaptionTiempos(10, 99, 60, base);
    expect(r.fin).toBeLessThanOrEqual(60);
    expect(r.inicio).toBeLessThanOrEqual(r.fin);
  });

  it("no rompe con duracion de video desconocida (0)", () => {
    const r = clampCaptionTiempos(10, 13, 0, base);
    expect(r).toEqual({ inicio: 10, fin: 13 });
  });

  // Un NaN viene de un input a medio escribir ("1." o el campo vacio): no
  // debe destruir el caption, se conserva el tiempo que ya tenia.
  it("descarta NaN conservando el tiempo original del caption", () => {
    expect(clampCaptionTiempos(NaN, 13, 0, base)).toEqual({ inicio: 10, fin: 13 });
    expect(clampCaptionTiempos(10, NaN, 0, base)).toEqual({ inicio: 10, fin: 13 });
  });
});