import { describe, it, expect } from "vitest";
import {
  construirRecovery,
  leerRecovery,
  formatearRecovery,
  RECOVERY_VIGENTE_MS,
} from "../recovery";
import type { Proyecto } from "../../types";

const proyecto: Proyecto = {
  ruta_video: "/videos/peli.mkv",
  hablantes: [
    { id: "h1", nombre: "Ana", tecla: "1", color: "#E85D4E" },
  ],
  captions: [
    { id: "c1", inicio: 0, fin: 3, texto: "hola", hablante_id: "h1" },
  ],
  playhead: 12.5,
};

describe("construirRecovery", () => {
  it("envuelve el proyecto con su ruta de origen y el instante", () => {
    const r = construirRecovery(proyecto, "/proyectos/mi.cdp", 1000);
    expect(r.version).toBe(1);
    expect(r.rutaProyecto).toBe("/proyectos/mi.cdp");
    expect(r.proyecto).toEqual(proyecto);
  });

  it("tolera una ruta de proyecto vacía (proyecto nunca guardado)", () => {
    const r = construirRecovery(proyecto, "", 1000);
    expect(r.rutaProyecto).toBe("");
  });
});

describe("leerRecovery", () => {
  it("devuelve null si no hay texto", () => {
    expect(leerRecovery(null)).toBeNull();
    expect(leerRecovery("")).toBeNull();
  });

  it("devuelve null si el JSON está corrupto", () => {
    expect(leerRecovery("{no es json")).toBeNull();
  });

  // Un archivo de recovery escrito por una versión vieja (o corrupto a
  // medias) no debe romper el arranque: se descarta y se sigue.
  it("devuelve null si falta cualquier campo requerido", () => {
    expect(leerRecovery(JSON.stringify({ version: 1 }))).toBeNull();
    expect(
      leerRecovery(JSON.stringify({ version: 1, rutaProyecto: "/a", proyecto: {} })),
    ).toBeNull();
  });

  it("devuelve null si la versión no es la conocida", () => {
    const malo = JSON.stringify({ ...construirRecovery(proyecto, "/a.cdp", 1), version: 99 });
    expect(leerRecovery(malo)).toBeNull();
  });

  it("acepta un recovery bien formado", () => {
    const texto = JSON.stringify(construirRecovery(proyecto, "/a.cdp", Date.now()));
    const r = leerRecovery(texto);
    expect(r?.rutaProyecto).toBe("/a.cdp");
    expect(r?.proyecto.captions).toHaveLength(1);
  });

  // El criterio para ofrecer recuperar: que el recovery sea más nuevo que el
  // último guardado manual. Si el recovery es viejo, el archivo del proyecto
  // está más al día y no hay nada que recuperar.
  it("descarta un recovery más viejo que el cutoff", () => {
    const texto = JSON.stringify(construirRecovery(proyecto, "/a.cdp", 1000));
    expect(leerRecovery(texto, 2000)).toBeNull();
  });

  it("acepta un recovery dentro de la ventana de vigencia", () => {
    const ahora = Date.now();
    const texto = JSON.stringify(construirRecovery(proyecto, "/a.cdp", ahora));
    expect(leerRecovery(texto, ahora - 1000)).not.toBeNull();
  });

  // Si el usuario no abre la app en una semana, la copia automática ya no
  // interesa y solo ocupa espacio.
  it("descarta un recovery mas viejo que la ventana de vigencia", () => {
    const viejo = Date.now() - RECOVERY_VIGENTE_MS - 1000;
    const texto = JSON.stringify(construirRecovery(proyecto, "/a.cdp", viejo));
    expect(leerRecovery(texto)).toBeNull();
  });
});

describe("formatearRecovery", () => {
  it("incluye el nombre del proyecto y el numero de subtitulos", () => {
    const r = construirRecovery(proyecto, "/carpeta/mi-video.cdp", Date.now());
    const texto = formatearRecovery(r, "mi-video");
    expect(texto).toContain("mi-video");
    expect(texto).toContain("1");
  });

  it("no revienta con una ruta vacía", () => {
    const r = construirRecovery(proyecto, "", Date.now());
    expect(() => formatearRecovery(r, "")).not.toThrow();
  });
});