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
