import { describe, expect, it } from "vitest";
import {
  asignarHablantesPorTexto,
  hablantesDesdeNombres,
  normalizarTexto,
  parseAutosubsTxt,
} from "../autosubs";
import { PALETA } from "../constants";
import type { Caption } from "../../types";

function cue(inicio: number, fin: number, texto: string): Caption {
  return { id: "x", inicio, fin, texto, hablante_id: null };
}

describe("parseAutosubsTxt", () => {
  it("extrae turnos Speaker N con su texto", () => {
    const turnos = parseAutosubsTxt(
      "Speaker 1:\nHola mundo.\n\nSpeaker 2:\nAdiós.\n",
    );
    expect(turnos).toEqual([
      { speaker: "1", texto: "Hola mundo." },
      { speaker: "2", texto: "Adiós." },
    ]);
  });

  it("texto vacío no da turnos", () => {
    expect(parseAutosubsTxt("")).toEqual([]);
  });

  it("acepta TXT con CRLF de Windows", () => {
    // Regresión: el regex exigía \n y un TXT con \r\n daba 0 turnos.
    const turnos = parseAutosubsTxt(
      "Speaker 1:\r\nHola mundo.\r\n\r\nSpeaker 2:\r\nAdiós.\r\n",
    );
    expect(turnos).toEqual([
      { speaker: "1", texto: "Hola mundo." },
      { speaker: "2", texto: "Adiós." },
    ]);
  });
});

describe("normalizarTexto", () => {
  it("minúsculas, sin tildes ni puntuación, espacios simples", () => {
    expect(normalizarTexto("¡Adiós,  MUY bueno!")).toBe("adios muy bueno");
  });
});

describe("asignarHablantesPorTexto", () => {
  it("asigna por orden secuencial de turnos", () => {
    const cues = [cue(0, 1, "Hola"), cue(1, 2, "Adiós"), cue(2, 3, "Hola otra vez")];
    const turnos = [
      { speaker: "1", texto: "Hola" },
      { speaker: "2", texto: "Adiós. Hola otra vez" },
    ];
    expect(asignarHablantesPorTexto(cues, turnos)).toEqual(["1", "2", "2"]);
  });

  it("tolera tildes y puntuación distinta entre SRT y TXT", () => {
    const cues = [cue(0, 1, "¡Adios, muy bueno!")];
    const turnos = [{ speaker: "3", texto: "Adiós; muy bueno" }];
    expect(asignarHablantesPorTexto(cues, turnos)).toEqual(["3"]);
  });

  it("cue sin match queda null", () => {
    const cues = [cue(0, 1, "zzz inexistente")];
    const turnos = [{ speaker: "1", texto: "Hola" }];
    expect(asignarHablantesPorTexto(cues, turnos)).toEqual([null]);
  });

  it("sin turnos todo queda null", () => {
    expect(asignarHablantesPorTexto([cue(0, 1, "Hola")], [])).toEqual([null]);
  });

  it("dos cues del mismo turno no saltan al turno siguiente repetido", () => {
    // Regresión: con `pos = hit + 1` el segundo cue ya no se buscaba en su
    // propio turno y caía en el turno repetido siguiente.
    const cues = [cue(0, 1, "hola"), cue(1, 2, "mundo")];
    const turnos = [
      { speaker: "1", texto: "hola mundo" },
      { speaker: "2", texto: "mundo cruel" },
    ];
    expect(asignarHablantesPorTexto(cues, turnos)).toEqual(["1", "1"]);
  });

  it("no matchea subcadenas dentro de otra palabra", () => {
    // Regresión: "Sí." normaliza a "si" y con `includes` matcheaba dentro de "así".
    const cues = [cue(0, 1, "Sí.")];
    const turnos = [{ speaker: "1", texto: "Así que vamos" }];
    expect(asignarHablantesPorTexto(cues, turnos)).toEqual([null]);
  });
});

describe("hablantesDesdeNombres", () => {
  it("convención de la app: nombre, tecla dígito, color PALETA", () => {
    const hs = hablantesDesdeNombres(["1", "2"]);
    expect(hs.map((h) => h.nombre)).toEqual(["Hablante 1", "Hablante 2"]);
    expect(hs.map((h) => h.tecla)).toEqual(["1", "2"]);
    expect(hs[0].color).toBe("#E85D4E");
    expect(hs[0].id).toMatch(/^sp-autosubs-/);
  });

  // Repro bug 1: PALETA debe cubrir los 9 hablantes (teclas 1-9) sin repetir.
  it("9 hablantes reciben 9 colores distintos", () => {
    expect(PALETA.length).toBeGreaterThanOrEqual(9);
    expect(new Set(PALETA).size).toBe(PALETA.length);
    const hs = hablantesDesdeNombres(
      ["1", "2", "3", "4", "5", "6", "7", "8", "9"],
    );
    expect(new Set(hs.map((h) => h.color)).size).toBe(9);
  });

  // Repro bug 2: el orden debe ser numérico, no lexicográfico ("2" < "10").
  it("ordena numéricamente aunque lleguen desordenados", () => {
    const hs = hablantesDesdeNombres(["10", "2", "1"]);
    expect(hs.map((h) => h.nombre)).toEqual([
      "Hablante 1",
      "Hablante 2",
      "Hablante 10",
    ]);
    expect(hs.map((h) => h.tecla)).toEqual(["1", "2", "3"]);
  });
});
