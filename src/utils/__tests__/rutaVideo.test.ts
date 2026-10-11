import { describe, it, expect } from "vitest";
import { rutaRelativaAlProyecto, resolverRutaVideo } from "../rutaVideo";

describe("rutaRelativaAlProyecto", () => {
  it("devuelve la ruta tal cual si el video esta en la misma carpeta", () => {
    // "/p/proyecto.cdp" + "/p/video.mp4" -> mismo directorio.
    expect(rutaRelativaAlProyecto("/p/video.mp4", "/p/proyecto.cdp")).toBe("video.mp4");
  });

  it("devuelve la ruta relativa si el video esta en una subcarpeta", () => {
    expect(rutaRelativaAlProyecto("/p/videos/video.mp4", "/p/proyecto.cdp")).toBe(
      "videos/video.mp4",
    );
  });

  // Un proyecto en /p no puede guardar una ruta relativa a un video de /q:
  // al abrirlo en otra maquina, el video no esta ahi y la ruta relativa
  // mentiria mas que la absoluta.
  it("devuelve la ruta ABSOLUTA si el video esta fuera de la carpeta del proyecto", () => {
    expect(rutaRelativaAlProyecto("/otro/video.mp4", "/p/proyecto.cdp")).toBe(
      "/otro/video.mp4",
    );
  });

  it("tolera separadores mixtos (Windows guardado, leido en Linux)", () => {
    expect(rutaRelativaAlProyecto("/p/videos/a.mp4", "/p/proyecto.cdp")).toBe(
      "videos/a.mp4",
    );
  });

  it("devuelve la ruta tal cual si falta alguno de los dos datos", () => {
    expect(rutaRelativaAlProyecto("", "/p/p.cdp")).toBe("");
    expect(rutaRelativaAlProyecto("/p/v.mp4", "")).toBe("/p/v.mp4");
  });
});

describe("resolverRutaVideo", () => {
  // Al abrir un proyecto: si la ruta guardada es relativa, resolverla contra
  // la carpeta del .cdp. Si es absoluta, se prueba tal cual (proyectos viejos).
  it("resuelve una ruta relativa contra la carpeta del proyecto", () => {
    expect(resolverRutaVideo("video.mp4", "/p/proyecto.cdp")).toBe("/p/video.mp4");
  });

  it("resuelve una ruta relativa con subcarpeta", () => {
    expect(resolverRutaVideo("videos/a.mp4", "/p/proyecto.cdp")).toBe(
      "/p/videos/a.mp4",
    );
  });

  it("devuelve la ruta absoluta sin tocarla", () => {
    expect(resolverRutaVideo("/otro/video.mp4", "/p/proyecto.cdp")).toBe(
      "/otro/video.mp4",
    );
  });

  it("reconoce una ruta absoluta de Windows como absoluta", () => {
    // "C:\\videos\\a.mp4" no es relativa aunque en Linux no exista.
    expect(resolverRutaVideo("C:\\videos\\a.mp4", "/p/proyecto.cdp")).toBe(
      "C:\\videos\\a.mp4",
    );
  });

  it("devuelve cadena vacia si no hay ruta", () => {
    expect(resolverRutaVideo("", "/p/proyecto.cdp")).toBe("");
  });

  it("normaliza barras para una ruta guardada en Windows y abierta en Linux", () => {
    expect(resolverRutaVideo("videos\\a.mp4", "/p/proyecto.cdp")).toBe(
      "/p/videos/a.mp4",
    );
  });
});