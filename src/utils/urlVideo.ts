// URL de reproducción del video, por plataforma.
//
// En Windows/macOS el webview resuelve `asset://` y el `<video>` carga bien
// con `convertFileSrc()`. En Linux NO: WebKitGTK resuelve media con el pipeline
// de GStreamer, que no conoce el handler de esquema custom de Tauri, así que
// `asset://` falla siempre con MEDIA_ERR_SRC_NOT_SUPPORTED (tauri#3725 /
// WebKit#146351). Ahí el backend sirve el archivo por un puerto de loopback y
// manda esa URL en vez de la del asset.
//
// La función queda pura (la URL local entra como dato) para poder testearla sin
// Tauri ni DOM, que es lo que hace el resto de `src/utils`.

/** Elige la URL del `<video>`: la del servidor local si viene, si no la del
 *  asset protocol. `convertirAsset` se inyecta para no depender de Tauri acá. */
export function urlDeReproduccion(
  ruta: string,
  urlLocal: string | null,
  convertirAsset: (ruta: string) => string,
): string {
  return urlLocal && urlLocal.length > 0 ? urlLocal : convertirAsset(ruta);
}
