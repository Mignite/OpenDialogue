import { describe, it, expect } from "vitest";
import { findSnapTime, findSnapBlockDelta, BuildOverlapReport } from "../captions";
import type { Caption, Hablante } from "../../types";

function makeCap(id: string, inicio: number, fin: number, hablante_id: string | null = null): Caption {
  return { id, inicio, fin, texto: "test", hablante_id };
}

const emptyHablantes: Hablante[] = [];

describe("findSnapTime", () => {
  it("returns null when no match within threshold", () => {
    const caps = [makeCap("c1", 0, 5)];
    expect(findSnapTime(10, "c1", caps)).toBeNull();
  });

  it("snaps to nearest caption edge", () => {
    const caps = [makeCap("c1", 0, 5), makeCap("c2", 10, 15)];
    const snap = findSnapTime(10.1, "c1", caps);
    expect(snap).toBe(10);
  });

  it("ignores the excluded caption id", () => {
    const caps = [makeCap("c1", 0, 5), makeCap("c2", 10, 15)];
    const snap = findSnapTime(10.05, "c1", caps);
    expect(snap).toBe(10);
  });

  it("ignores multiple excluded caption ids (block drag)", () => {
    const caps = [makeCap("c1", 0, 5), makeCap("c2", 10, 15), makeCap("c3", 10.05, 20)];
    // Excluyendo c1 y c2, debe snapear a c3.inicio (10.05).
    const snap = findSnapTime(10.05, ["c1", "c2"], caps);
    expect(snap).toBe(10.05);
    // Excluyendo todos, no hay target.
    const snap2 = findSnapTime(10.05, ["c1", "c2", "c3"], caps);
    expect(snap2).toBeNull();
  });

  it("snaps to an extra target (playhead)", () => {
    const caps = [makeCap("c1", 0, 5)];
    expect(findSnapTime(10.1, "c1", caps, [10])).toBe(10);
    expect(findSnapTime(10.1, "c1", caps)).toBeNull();
  });
});

describe("findSnapBlockDelta", () => {
  // Bloque [2,5] arrastrado; vecino en [10,15].
  const caps = [makeCap("b1", 2, 3), makeCap("b2", 4, 5), makeCap("n", 10, 15)];

  it("snaps block start rightward to the next edge", () => {
    // tMin=9.95 cerca de 10 → delta +0.05.
    expect(findSnapBlockDelta(9.95, 12.95, ["b1", "b2"], caps, null)).toBeCloseTo(0.05, 6);
  });

  it("snaps block end rightward to the next edge", () => {
    // tMax=9.97 cerca de 10 (tMin=6.97 lejos de todo) → delta +0.03.
    expect(findSnapBlockDelta(6.97, 9.97, ["b1", "b2"], caps, null)).toBeCloseTo(0.03, 6);
  });

  it("snaps block end leftward to the previous edge", () => {
    // Bloque [5.03,8.03]: tMax cerca de... tMin=5.03 cerca de 5 (fin de b1)? b1 excluido.
    // Vecino previo: c0=[0,2] fin=2. tMin=2.03 → delta -0.03.
    const caps2 = [makeCap("c0", 0, 2), makeCap("b1", 2, 5)];
    expect(findSnapBlockDelta(2.03, 5.03, ["b1"], caps2, null)).toBeCloseTo(-0.03, 6);
  });

  it("the closest magnet wins", () => {
    // tMin a 0.1 de 10, tMax a 0.05 de 10 → gana el fin (+0.05).
    expect(findSnapBlockDelta(9.9, 9.95, ["b1", "b2"], caps, null)).toBeCloseTo(0.05, 6);
  });

  it("returns 0 when nothing is in range", () => {
    expect(findSnapBlockDelta(20, 25, ["b1", "b2"], caps, null)).toBe(0);
  });

  it("ignores excluded ids and snaps to playhead", () => {
    expect(findSnapBlockDelta(20, 25, ["b1", "b2"], caps, 25.02)).toBeCloseTo(0.02, 6);
    expect(findSnapBlockDelta(20, 25, ["b1", "b2", "n"], caps, 30)).toBe(0);
  });
});

describe("BuildOverlapReport", () => {
  it("returns empty for no overlaps", () => {
    const caps = [makeCap("c1", 0, 5, "s1"), makeCap("c2", 6, 10, "s2")];
    const result = BuildOverlapReport(caps, emptyHablantes);
    expect(result).toHaveLength(0);
  });

  it("detects overlap between different speakers", () => {
    const caps = [makeCap("c1", 0, 10, "s1"), makeCap("c2", 3, 7, "s2")];
    const hablantes: Hablante[] = [
      { id: "s1", nombre: "Alice", tecla: "a", color: "#ff0000" },
      { id: "s2", nombre: "Bob", tecla: "b", color: "#00ff00" },
    ];
    const result = BuildOverlapReport(caps, hablantes);
    expect(result).toHaveLength(1);
    expect(result[0].inicio).toBe(3);
    expect(result[0].fin).toBe(7);
  });

  it("ignores overlap for same speaker", () => {
    const caps = [makeCap("c1", 0, 10, "s1"), makeCap("c2", 3, 7, "s1")];
    const result = BuildOverlapReport(caps, emptyHablantes);
    expect(result).toHaveLength(0);
  });
});
