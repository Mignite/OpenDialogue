import { describe, it, expect } from "vitest";
import { textoDelPortapapeles, captionAPortapapeles } from "../clipboard";

describe("captionAPortapapeles", () => {
  // El texto del caption tal cual: pegarlo en un editor externo tiene que dar
  // exactamente lo que se ve en el subtítulo, sin metadatos que ensucien.
  it("devuelve el texto del caption", () => {
    expect(captionAPortapapeles("Hola, ¿qué tal?", 1.5, 4.2)).toBe("Hola, ¿qué tal?");
  });

  it("no mete marcas de tiempo en el texto", () => {
    const t = captionAPortapapeles("Prueba", 1.5, 4.2);
    expect(t).not.toContain("00:00");
    expect(t).not.toContain("-->");
    expect(t).not.toContain("1.5");
  });
});

describe("textoDelPortapapeles", () => {
  // El SO puede devolver basura: clipboard con formato raro, una imagen, o
  // texto con \r\n. Lo que no sea texto usable tiene que llegar como cadena
  // vacía para que el caller decida, no como "undefined" o un objeto.
  it("normaliza CRLF a LF", () => {
    expect(textoDelPortapapeles("uno\r\ndos\r\ntres")).toBe("uno\ndos\ntres");
  });

  it("recorta CR sueltos", () => {
    expect(textoDelPortapapeles("uno\rdos")).toBe("unodos");
  });

  it("devuelve vacio para null/undefined", () => {
    expect(textoDelPortapapeles(null)).toBe("");
    expect(textoDelPortapapeles(undefined)).toBe("");
  });

  it("devuelve vacio para texto que solo son espacios", () => {
    expect(textoDelPortapapeles("   \n\t  ")).toBe("");
  });

  it("conserva el texto normal tal cual", () => {
    expect(textoDelPortapapeles("hola")).toBe("hola");
  });

  it("conserva acentos y eme", () => {
    expect(textoDelPortapapeles("Canción — sí")).toBe("Canción — sí");
  });
});