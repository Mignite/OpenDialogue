import { describe, expect, it } from "vitest";
import { coincide, partirResaltado, reemplazarTodos } from "../busqueda";

describe("coincide", () => {
  it("término vacío o de espacios nunca matchea", () => {
    expect(coincide("Hola", "", false)).toBe(false);
    expect(coincide("Hola", "   ", false)).toBe(false);
  });

  it("case-insensitive por defecto", () => {
    expect(coincide("Hola Mundo", "mundo", false)).toBe(true);
    expect(coincide("Hola Mundo", "MUNDO", false)).toBe(true);
  });

  it("case-sensitive respeta mayúsculas", () => {
    expect(coincide("Hola Mundo", "mundo", true)).toBe(false);
    expect(coincide("Hola Mundo", "Mundo", true)).toBe(true);
  });

  it("el término se recorta, el texto no", () => {
    expect(coincide("Hola mundo", " mundo ", false)).toBe(true);
  });
});

describe("partirResaltado", () => {
  it("sin término devuelve un solo segmento", () => {
    expect(partirResaltado("Hola", "", false)).toEqual([
      { texto: "Hola", coincide: false },
    ]);
  });

  it("una coincidencia al medio", () => {
    expect(partirResaltado("Hola mundo", "mundo", false)).toEqual([
      { texto: "Hola ", coincide: false },
      { texto: "mundo", coincide: true },
    ]);
  });

  it("varias coincidencias no solapadas", () => {
    expect(partirResaltado("siso", "s", false)).toEqual([
      { texto: "s", coincide: true },
      { texto: "i", coincide: false },
      { texto: "s", coincide: true },
      { texto: "o", coincide: false },
    ]);
  });

  it("matches solapados van de izquierda a derecha sin solapar", () => {
    expect(partirResaltado("aaa", "aa", false)).toEqual([
      { texto: "aa", coincide: true },
      { texto: "a", coincide: false },
    ]);
  });

  it("coincidencia al inicio y al final", () => {
    expect(partirResaltado("abca", "a", false)).toEqual([
      { texto: "a", coincide: true },
      { texto: "bc", coincide: false },
      { texto: "a", coincide: true },
    ]);
  });

  it("coincide el texto completo", () => {
    expect(partirResaltado("Hola", "hola", false)).toEqual([
      { texto: "Hola", coincide: true },
    ]);
  });

  it("sin coincidencia devuelve el texto intacto", () => {
    expect(partirResaltado("Hola", "xyz", false)).toEqual([
      { texto: "Hola", coincide: false },
    ]);
  });

  it("caracteres especiales del término no explotan (indexOf, no regex)", () => {
    expect(partirResaltado("a(b)c", "(b)", false)).toEqual([
      { texto: "a", coincide: false },
      { texto: "(b)", coincide: true },
      { texto: "c", coincide: false },
    ]);
  });
});

describe("reemplazarTodos", () => {
  it("reemplaza todas las ocurrencias case-insensitive", () => {
    expect(reemplazarTodos("Ana y ana y ANA", "ana", "Eva", false)).toBe(
      "Eva y Eva y Eva",
    );
  });

  it("reemplazo vacío borra las ocurrencias", () => {
    expect(reemplazarTodos("a-b-c", "-", "", false)).toBe("abc");
  });

  it("conserva el resto del texto exacto", () => {
    expect(reemplazarTodos("Hola Mundo", "mundo", "mundo cruel", false)).toBe(
      "Hola mundo cruel",
    );
  });

  it("case-sensitive respeta mayúsculas", () => {
    expect(reemplazarTodos("Ana y ana", "Ana", "Eva", true)).toBe("Eva y ana");
  });

  it("sin coincidencias devuelve el texto igual", () => {
    expect(reemplazarTodos("Hola", "xyz", "abc", false)).toBe("Hola");
  });

  it("el reemplazo puede contener el término sin loop infinito", () => {
    expect(reemplazarTodos("a b a", "a", "aa", false)).toBe("aa b aa");
  });
});
