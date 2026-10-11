// Portapapeles real del sistema.
//
// Antes el portapapeles era un ref interno: Ctrl+C copiaba el caption al
// portapapeles de la app y Ctrl+V pegaba desde ahí. Eso imposibilaba lo más
// natural: copiar un subtítulo a un editor de texto externo, o pegar en el
// editor de la app un texto copiado de otro programa.
//
// Estas funciones son puras (normalizan); el acceso asíncrono al portapapeles
// del navegador va en `leerPortapapeles`/`escribirPortapapeles`.

/** Normaliza lo que devuelve el portapapeles del SO.
 *
 *  El SO puede devolver null (clipboard vacío), un objeto (si hay una imagen
 *  copiada) o texto con CRLF. Cualquier cosa que no sea texto usable vuelve
 *  como "" para que el caller decida, en vez de propagar undefined. */
export function textoDelPortapapeles(valor: unknown): string {
  if (typeof valor !== "string") return "";
  // \r\n y \r sueltos -> \n. Un SRT pegado desde Windows no debe traer los CR
  // dentro del texto del subtítulo.
  const normalizado = valor.replace(/\r\n/g, "\n").replace(/\r/g, "");
  // Solo espacios: tratar como vacío (no crear un subtítulo en blanco).
  return normalizado.trim() === "" ? "" : normalizado;
}

/** Texto a mandar al portapapeles al copiar un caption: su texto tal cual,
 *  sin marcas de tiempo. Pegar eso en un editor tiene que dar el subtítulo,
 *  no un bloque SRT. */
export function captionAPortapapeles(
  texto: string,
  _inicio: number,
  _fin: number,
): string {
  return texto;
}

/** Escribe en el portapapeles del SO. Silencioso si el navegador no expone la
 *  API (entornos sin permiso de clipboard). */
export async function escribirPortapapeles(texto: string): Promise<boolean> {
  try {
    if (typeof navigator === "undefined" || !navigator.clipboard) return false;
    await navigator.clipboard.writeText(texto);
    return true;
  } catch {
    return false;
  }
}

/** Lee del portapapeles del SO. Devuelve "" si no hay texto (o no hay API). */
export async function leerPortapapeles(): Promise<string> {
  try {
    if (typeof navigator === "undefined" || !navigator.clipboard) return "";
    const texto = await navigator.clipboard.readText();
    return textoDelPortapapeles(texto);
  } catch {
    // Sin permiso de lectura (o en un contexto no seguro): no es un error
    // para el usuario, simplemente no hay nada que pegar.
    return "";
  }
}