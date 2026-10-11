import { describe, it, expect } from "vitest";
import {
  captionRowIndex,
  filasDestinoRelativas,
  filtrarPorMarquee,
  agruparCaptionsPorCarril,
} from "../selection";
import type { Caption, Hablante } from "../../types";

function makeCap(id: string, inicio: number, fin: number, hablante_id: string | null = null): Caption {
  return { id, inicio, fin, texto: "test", hablante_id };
}

const hablantes: Hablante[] = [
  { id: "s1", nombre: "Alice", tecla: "a", color: "#ff0000" },
  { id: "s2", nombre: "Bob", tecla: "b", color: "#00ff00" },
];

describe("captionRowIndex", () => {
  it("maps unassigned captions to row 0", () => {
    expect(captionRowIndex(null, hablantes)).toBe(0);
  });

  it("maps each speaker to its position + 1", () => {
    expect(captionRowIndex("s1", hablantes)).toBe(1);
    expect(captionRowIndex("s2", hablantes)).toBe(2);
  });

  it("falls back to row 0 for unknown speaker", () => {
    expect(captionRowIndex("nope", hablantes)).toBe(0);
  });
});

describe("filtrarPorMarquee", () => {
  it("returns empty for empty captions", () => {
    expect(filtrarPorMarquee([], hablantes, { t1: 0, t2: 10, fila1: 0, fila2: 2 })).toEqual([]);
  });

  it("selects captions intersecting the time range", () => {
    const caps = [
      makeCap("c1", 0, 5, "s1"),
      makeCap("c2", 6, 10, "s1"),
      makeCap("c3", 12, 15, "s1"),
    ];
    const ids = filtrarPorMarquee(caps, hablantes, { t1: 4, t2: 13, fila1: 1, fila2: 1 });
    expect(ids).toEqual(["c1", "c2", "c3"]);
  });

  it("filters by speaker row", () => {
    const caps = [
      makeCap("c1", 0, 5, "s1"),
      makeCap("c2", 1, 3, "s2"),
      makeCap("c3", 2, 4, null),
    ];
    const ids = filtrarPorMarquee(caps, hablantes, { t1: 0, t2: 10, fila1: 2, fila2: 2 });
    expect(ids).toEqual(["c2"]);
  });

  it("normalizes inverted rectangle bounds", () => {
    const caps = [makeCap("c1", 0, 5, "s1")];
    const ids = filtrarPorMarquee(caps, hablantes, { t1: 5, t2: 0, fila1: 1, fila2: 0 });
    expect(ids).toEqual(["c1"]);
  });

  it("requires overlap, not containment", () => {
    const caps = [makeCap("c1", 0, 1, "s1"), makeCap("c2", 10, 12, "s1")];
    const ids = filtrarPorMarquee(caps, hablantes, { t1: 2, t2: 9, fila1: 1, fila2: 1 });
    expect(ids).toEqual([]);
  });
});

describe("filasDestinoRelativas", () => {
  it("shifts every row by the same delta, preserving the pattern", () => {
    const origen = new Map([["a", 1], ["b", 2], ["c", 3]]);
    expect(filasDestinoRelativas(origen, 1, 5)).toEqual(new Map([["a", 2], ["b", 3], ["c", 4]]));
  });

  it("is identity with shift 0", () => {
    const origen = new Map([["a", 0], ["b", 2]]);
    expect(filasDestinoRelativas(origen, 0, 5)).toEqual(new Map([["a", 0], ["b", 2]]));
  });

  it("shifts negatively", () => {
    const origen = new Map([["a", 2], ["b", 3]]);
    expect(filasDestinoRelativas(origen, -2, 5)).toEqual(new Map([["a", 0], ["b", 1]]));
  });

  it("clamps at row 0 instead of going negative", () => {
    const origen = new Map([["a", 0], ["b", 1]]);
    expect(filasDestinoRelativas(origen, -1, 5)).toEqual(new Map([["a", 0], ["b", 0]]));
  });

  it("clamps at maxFila instead of overflowing", () => {
    const origen = new Map([["a", 3], ["b", 2]]);
    expect(filasDestinoRelativas(origen, 1, 3)).toEqual(new Map([["a", 3], ["b", 3]]));
  });

  it("returns empty for empty input", () => {
    expect(filasDestinoRelativas(new Map(), 2, 5)).toEqual(new Map());
  });
});

// Agrupar por carril en UNA pasada. Antes, `trackRows` hacía
// `captions.filter()` por cada hablante en cada render: O(n × (S+1)). Medido
// con 2000 captions y 9 hablantes = 1.04 ms por render vs 0.34 ms de una
// pasada (3.1×), y como el playhead fuerza renders a 60 Hz eso son ~62 ms/s
// de CPU quemada en algo que no cambia.
describe("agruparCaptionsPorCarril", () => {
  it("separa los captions sin hablante de los asignados", () => {
    const caps = [makeCap("a", 0, 1, null), makeCap("b", 1, 2, "s1")];
    const g = agruparCaptionsPorCarril(caps, hablantes);
    expect(g.porHablante.get("s1")?.map((c) => c.id)).toEqual(["b"]);
    expect(g.sinAsignar.map((c) => c.id)).toEqual(["a"]);
  });

  it("un hablante sin captions devuelve la entrada con array vacío", () => {
    const g = agruparCaptionsPorCarril([makeCap("a", 0, 1, "s1")], hablantes);
    // s2 no tiene nada: la clave existe para que el render no tenga que
    // distinguir "sin captions" de "hablante que no está".
    expect(g.porHablante.has("s2")).toBe(true);
    expect(g.porHablante.get("s2")).toEqual([]);
  });

  it("un hablante_id desconocido no aparece en el mapa ni en sinAsignar", () => {
    // Un caption asignado a un hablante que ya no existe: no tiene carril.
    // No va al carril "—" porque ese es "sin asignar", no "hablante fantasma".
    const g = agruparCaptionsPorCarril([makeCap("a", 0, 1, "fantasma")], hablantes);
    expect(g.porHablante.has("fantasma")).toBe(false);
    expect(g.sinAsignar).toEqual([]);
  });

  it("conserva el orden de entrada dentro de cada carril", () => {
    const caps = [
      makeCap("a", 0, 1, "s1"),
      makeCap("b", 2, 3, "s2"),
      makeCap("c", 4, 5, "s1"),
    ];
    const g = agruparCaptionsPorCarril(caps, hablantes);
    expect(g.porHablante.get("s1")?.map((c) => c.id)).toEqual(["a", "c"]);
  });

  it("no colapsa captions mutados: usa las referencias del array recibido", () => {
    const cap = makeCap("a", 0, 1, "s1");
    const g = agruparCaptionsPorCarril([cap], hablantes);
    expect(g.porHablante.get("s1")?.[0]).toBe(cap);
  });
});
