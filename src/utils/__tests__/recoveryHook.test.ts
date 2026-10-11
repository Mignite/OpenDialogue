import { describe, it, expect } from "vitest";
import { debeGuardarRecovery, INTERVALO_AUTOSAVE_MS } from "../recovery";

describe("debeGuardarRecovery", () => {
  it("guarda cuando hay cambios pendientes y corresponde por intervalo", () => {
    // ultimaEscritura viejo (pasó el intervalo) + sucio => escribe.
    const ahora = Date.now();
    expect(debeGuardarRecovery(true, 3, ahora - INTERVALO_AUTOSAVE_MS - 1000, ahora)).toBe(true);
  });

  it("no guarda si no hay cambios", () => {
    expect(debeGuardarRecovery(false, 3, Date.now())).toBe(false);
  });

  // Un proyecto sin captions ni hablantes no es trabajo: escribirlo solo llena
  // el disco y ofrece "recuperar" un proyecto vacío.
  it("no guarda un proyecto vacío", () => {
    expect(debeGuardarRecovery(true, 0, Date.now())).toBe(false);
  });

  // El throttle evita escribir en cada pulsación mientras se edita texto.
  it("respeta el intervalo entre escrituras", () => {
    const ahora = Date.now();
    expect(debeGuardarRecovery(true, 3, ahora - 1000)).toBe(false);
  });

  it("guarda pasado el intervalo", () => {
    const ahora = Date.now();
    expect(debeGuardarRecovery(true, 3, ahora - INTERVALO_AUTOSAVE_MS - 1000)).toBe(true);
  });

  // Sin ultimaEscritura (primera vez) tiene que guardar de inmediato: si no,
  // un crash en los primeros segundos no deja nada.
  it("guarda en la primera pasada", () => {
    expect(debeGuardarRecovery(true, 3, null)).toBe(true);
  });

  it("el proyecto con contenido es el que se serializa", () => {
    expect(debeGuardarRecovery(true, 1, null)).toBe(true);
  });
});