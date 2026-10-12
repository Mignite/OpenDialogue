import { describe, it, expect } from "vitest";
import { urlDeReproduccion } from "../urlVideo";

describe("urlDeReproduccion", () => {
  it("usa la URL del servidor local cuando el backend la ofrece", () => {
    // Linux: el comando devuelve una URL http:// de loopback.
    const local = "http://127.0.0.1:54321/video?t=7";
    expect(urlDeReproduccion("/videos/peli.mp4", local, (_p: string) => "asset://NO")).toBe(
      local,
    );
  });

  it("cae al asset de Tauri cuando no hay URL local", () => {
    // Windows/macOS: urlVideoLocal devuelve null y el asset:// funciona.
    const a = (p: string) => `asset://${p}`;
    expect(urlDeReproduccion("/videos/peli.mp4", null, a)).toBe(a("/videos/peli.mp4"));
  });

  it("ignora una URL local vacia y usa el asset", () => {
    const a = (p: string) => `asset://${p}`;
    expect(urlDeReproduccion("/videos/peli.mp4", "", a)).toBe(a("/videos/peli.mp4"));
  });

  it("prefiere la URL local aunque el asset tambien exista", () => {
    // La local gana porque es la unica que WebKitGTK puede reproducir.
    const a = (_p: string) => "asset://NO-DEBE-GANAR";
    expect(urlDeReproduccion("/p.mp4", "http://127.0.0.1:1/video?t=1", a)).toBe(
      "http://127.0.0.1:1/video?t=1",
    );
  });
});
