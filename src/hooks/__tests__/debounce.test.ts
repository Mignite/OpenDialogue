import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { programarDebounce, descargarDebounce } from "../debounce";

describe("programarDebounce", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("programa la ejecución dentro de la ventana", () => {
    const fn = vi.fn();
    programarDebounce(null, fn, 400);
    vi.advanceTimersByTime(399);
    expect(fn).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(fn).toHaveBeenCalledTimes(1);
  });

  // Ráfaga de edits: solo el último debe escribir (trailing debounce).
  it(" reprogramar cancela el timer anterior", () => {
    const primera = vi.fn();
    const segunda = vi.fn();
    const t1 = programarDebounce(null, primera, 400);
    programarDebounce(t1, segunda, 400);
    vi.advanceTimersByTime(400);
    expect(primera).not.toHaveBeenCalled();
    expect(segunda).toHaveBeenCalledTimes(1);
  });
});

describe("descargarDebounce (flush al desmontar)", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  // El bug que fija: cerrar la app dentro de los 400 ms de debounce cancelaba
  // el timer y el último cambio de preset no llegaba al disco.
  it("ejecuta la escritura pendiente aunque el timer no haya vencido", () => {
    const fn = vi.fn();
    const t = programarDebounce(null, fn, 400);
    vi.advanceTimersByTime(100);
    descargarDebounce(t, fn);
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it("no vuelve a ejecutar algo que ya se escribió", () => {
    const fn = vi.fn();
    const t = programarDebounce(null, fn, 400);
    vi.advanceTimersByTime(400);
    expect(fn).toHaveBeenCalledTimes(1);
    descargarDebounce(t, null); // ya no queda pendiente
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it("es un no-op si no hay nada pendiente", () => {
    expect(descargarDebounce(null, null)).toBeNull();
  });

  it("un flush con callback cancela el timer (no se dispara dos veces)", () => {
    const fn = vi.fn();
    const t = programarDebounce(null, fn, 400);
    descargarDebounce(t, fn);
    vi.advanceTimersByTime(400);
    expect(fn).toHaveBeenCalledTimes(1);
  });
});