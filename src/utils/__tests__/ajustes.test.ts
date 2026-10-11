import { describe, expect, it } from "vitest";
import { ajustesDesdeJson, ajustesAJson } from "../ajustes";

describe("ajustesDesdeJson", () => {
  it("vacio o nulo devuelve {}", () => {
    expect(ajustesDesdeJson(null)).toEqual({});
    expect(ajustesDesdeJson(undefined)).toEqual({});
    expect(ajustesDesdeJson("")).toEqual({});
  });

  it("locale valido se conserva", () => {
    expect(ajustesDesdeJson('{"locale":"es"}')).toEqual({ locale: "es" });
    expect(ajustesDesdeJson('{"locale":"en"}')).toEqual({ locale: "en" });
  });

  it("locale invalido se descarta", () => {
    expect(ajustesDesdeJson('{"locale":"pt"}')).toEqual({});
    expect(ajustesDesdeJson('{"locale":42}')).toEqual({});
    expect(ajustesDesdeJson("{}")).toEqual({});
  });

  it("json roto devuelve {}", () => {
    expect(ajustesDesdeJson("{no json")).toEqual({});
  });

  it("round-trip", () => {
    expect(ajustesDesdeJson(ajustesAJson({ locale: "es" }))).toEqual({
      locale: "es",
    });
  });
});

describe("ajustesDesdeJson: velocidad", () => {
  it("velocidad válida se conserva junto al locale", () => {
    expect(ajustesDesdeJson('{"locale":"es","velocidad":1.5}')).toEqual({
      locale: "es",
      velocidad: 1.5,
    });
    expect(ajustesDesdeJson('{"velocidad":0.5}')).toEqual({ velocidad: 0.5 });
  });

  it("velocidad fuera del selector se descarta", () => {
    expect(ajustesDesdeJson('{"velocidad":7.3}')).toEqual({});
    expect(ajustesDesdeJson('{"velocidad":"1.5"}')).toEqual({});
    expect(ajustesDesdeJson('{"velocidad":null}')).toEqual({});
  });

  it("round-trip conserva ambos campos", () => {
    expect(ajustesDesdeJson(ajustesAJson({ locale: "en", velocidad: 2 }))).toEqual(
      { locale: "en", velocidad: 2 },
    );
  });
});
