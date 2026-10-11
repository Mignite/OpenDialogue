import { describe, it, expect } from "vitest";
import { clampDeltaBloque } from "../captions";
import { nombreArchivoHablanteUnico } from "../srt";
import type { Caption } from "../../types";

// ---------------------------------------------------------------- #2 body drag
// El clamp del body drag usaba el inicio del clip Lead (el que se agarró) en
// vez del inicio MÁS TEMPRANO de la selección. Si otro clip del bloque empieza
// antes, moverCaptions lo recortaba a 0 por su cuenta y la selección se
// deformaba: el desfase interno entre clips no se conservaba.

describe("clampDeltaBloque", () => {
  it("permite mover a la derecha sin límite", () => {
    expect(clampDeltaBloque(10, 5)).toBe(10);
  });

  it("permite ir hasta t=0 exacto del clip más temprano", () => {
    // Bloque con lead en 5s y otro clip en 2s: el más temprano es 2.
    // deltaT = -2 lleva el clip de 2s justo a 0: se permite.
    expect(clampDeltaBloque(-2, 2)).toBe(-2);
  });

  it("clampa con el inicio más temprano de la selección, no con el del lead", () => {
    // El bug: con lead en 5s y otro clip en 2s, arrastrar a -4 empujaba
    // más allá de 0 para el clip de 2s. El clamp correcto es -2 (su inicio).
    expect(clampDeltaBloque(-4, 2)).toBe(-2);
  });

  it("no recorta cuando el lead es el más temprano", () => {
    // Un solo clip en 5s: comportamiento idéntico al clamp por lead.
    expect(clampDeltaBloque(-4, 5)).toBe(-4);
    expect(clampDeltaBloque(-5, 5)).toBe(-5);
    expect(clampDeltaBloque(-9, 5)).toBe(-5);
  });

  it("delta 0 con un clip en 0 se queda en 0", () => {
    // toBeCloseTo y no toBe: el clamp devuelve -0 (indistinto de 0 para ===,
    // que es lo que protege moverCaptions), pero Object.is los separa.
    expect(clampDeltaBloque(0, 0)).toBeCloseTo(0, 10);
    expect(clampDeltaBloque(-1, 0)).toBeCloseTo(0, 10);
  });

  // El preview (transform del DOM) usa el deltaT del clamp; lo que se guarda
  // pasa por `Math.max(0, inicio + deltaT)` en moverCaptions. Si ese max
  // tocara algún clip, el bloque se deformaría al soltar y el preview
  // mentía. Con el clamp por tMin la propiedad se cumple siempre.
  it("ningún clip del bloque cae por debajo de 0 (preview == guardado)", () => {
    const cap = (id: string, inicio: number, fin: number): Caption => ({
      id,
      inicio,
      fin,
      texto: "",
      hablante_id: null,
    });
    // El caso del reporte: lead en 5s, otro clip en 2s → desfase inicial 3s.
    const caps = [cap("A", 5, 7), cap("B", 2, 4)];
    const tMin = Math.min(...caps.map((c) => c.inicio)); // 2 (NO el 5 del lead)

    for (const deltaPedido of [-4, -5, -2, -1, 0, 3, 100]) {
      const deltaT = clampDeltaBloque(deltaPedido, tMin);
      for (const c of caps) {
        const guardado = Math.max(0, c.inicio + deltaT);
        // Lo que se ve en el preview es `inicio + deltaT` sin recortar.
        expect(guardado).toBeCloseTo(c.inicio + deltaT, 10);
      }
    }

    // Y el desfase interno entre A y B se conserva (era el síntoma visible:
    // 3s -> 1s).
    const deltaT = clampDeltaBloque(-4, tMin);
    const nuevoA = Math.max(0, caps[0].inicio + deltaT);
    const nuevoB = Math.max(0, caps[1].inicio + deltaT);
    expect(nuevoA - nuevoB).toBeCloseTo(
      caps[0].inicio - caps[1].inicio,
      10,
    );
    expect(nuevoB).toBeCloseTo(0, 10);
  });
});

// ---------------------------------------------------------------- #3 export SRT
// El nombre del archivo sale del nombre del hablante con los caracteres
// prohibidos de Windows reemplazados por "_". Sin deduplicar, dos hablantes
// homónimos (o "A/B" y "A:B") producían el MISMO archivo y el segundo
// borraba al primero.

describe("nombreArchivoHablanteUnico", () => {
  it("devuelve el nombre con extensión en el primer uso", () => {
    expect(nombreArchivoHablanteUnico("Ana", [])).toBe("Ana.srt");
  });

  it("sufija _2, _3... cuando el nombre ya se usó", () => {
    expect(nombreArchivoHablanteUnico("Ana", ["Ana.srt"])).toBe("Ana_2.srt");
    expect(nombreArchivoHablanteUnico("Ana", ["Ana.srt", "Ana_2.srt"])).toBe(
      "Ana_3.srt",
    );
  });

  it("sufija igual si el nombre base ya traía sufijo", () => {
    // "Ana_2" y "Ana" colisionan tras el sufijo automático.
    expect(nombreArchivoHablanteUnico("Ana_2", [])).toBe("Ana_2.srt");
    expect(nombreArchivoHablanteUnico("Ana_2", ["Ana.srt", "Ana_2.srt"])).toBe(
      "Ana_2_2.srt",
    );
  });

  it("dos nombres que se sanitizan al mismo texto colisionan", () => {
    // "A/B" y "A:B" -> ambos "A_B"
    const usados: string[] = [];
    const a = nombreArchivoHablanteUnico("A/B", usados);
    usados.push(a);
    const b = nombreArchivoHablanteUnico("A:B", usados);
    expect(a).toBe("A_B.srt");
    expect(b).toBe("A_B_2.srt");
  });

  it("no confunde nombres que solo se parecen", () => {
    expect(nombreArchivoHablanteUnico("Ana", ["Ana_2.srt"])).toBe("Ana.srt");
  });

  // Windows (y macOS por defecto) tratan el filesystem como case-insensitive:
  // "Ana.srt" y "ana.srt" son el MISMO archivo. Comparar con includes() las
  // daba por distintas y la segunda pisaba a la primera.
  it("colisiona cuando solo difieren en mayúsculas", () => {
    const usados: string[] = [];
    const a = nombreArchivoHablanteUnico("Ana", usados);
    usados.push(a);
    const b = nombreArchivoHablanteUnico("ana", usados);
    expect(a).toBe("Ana.srt");
    expect(b).toBe("ana_2.srt");
  });

  it("la comparación sin distinguir mayúsculas cubre también el sufijo", () => {
    expect(nombreArchivoHablanteUnico("ana", ["ANA.srt", "ana_2.srt"])).toBe(
      "ana_3.srt",
    );
  });

  // Nombres de dispositivo reservados: CON, PRN, AUX, NUL, COM1-9, LPT1-9
  // (más los dígitos superíndice ¹²³, que Windows trata como dígitos). No se
  // puede crear un archivo con ninguno de ellos, y añadirlos ".srt" tampoco
  // los vuelve legales. Se prefijan con "_" para que el export no falle.
  it("prefija con _ los nombres reservados de Windows", () => {
    for (const base of [
      "CON", "PRN", "AUX", "NUL",
      "COM1", "COM9", "LPT1", "LPT9",
      "con", "nul", "com1",
      "COM¹", "LPT²",
    ]) {
      expect(nombreArchivoHablanteUnico(base, [])).toBe(`_${base}.srt`);
    }
  });

  it("NO reserva nombres que solo empiezan como uno reservado", () => {
    // Solo es reservado el nombre completo antes del primer punto.
    for (const base of ["CONyecto", "NULO", "COM10", "LPT0", "CONSORCIO"]) {
      expect(nombreArchivoHablanteUnico(base, [])).toBe(`${base}.srt`);
    }
  });

  it("el prefijo _ también pasa por la deduplicación", () => {
    const usados: string[] = [];
    usados.push(nombreArchivoHablanteUnico("NUL", usados));
    usados.push(nombreArchivoHablanteUnico("NUL", usados));
    expect(usados).toEqual(["_NUL.srt", "_NUL_2.srt"]);
  });
});
