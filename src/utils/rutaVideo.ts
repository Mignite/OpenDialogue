// Portabilidad del proyecto entre máquinas.
//
// `ruta_video` se guardaba ABSOLUTA. Eso funciona en la máquina donde se creó
// y en ninguna otra: un proyecto con `C:\Videos\peli.mkv` no abre en Linux ni
// en otro equipo, aunque el .cdp y el videoViajen juntos en un pendrive.
//
// Formato: si el video está en la misma carpeta que el proyecto (o debajo), se
// guarda la ruta RELATIVA al .cdp. Si está fuera, se guarda la absoluta, que es
// la única forma honesta de dejarlo. Al abrir, una ruta relativa se resuelve
// contra la carpeta del proyecto.
//
// Compatibilidad: proyectos viejos tienen la ruta absoluta y se siguen
// abriendo sin cambios (una ruta absoluta se devuelve tal cual).

/** Normaliza \ a / para poder comparar rutas guardadas en Windows con las
 *  leídas en Linux. */
function aBarras(ruta: string): string {
  return ruta.replace(/\\/g, "/");
}

/** ¿Es una ruta absoluta? Cubre Windows (C:/ o \\UNC) y POSIX (/). */
function esAbsoluta(ruta: string): boolean {
  const n = aBarras(ruta);
  return n.startsWith("/") || /^[a-zA-Z]:\//.test(n);
}

/** Ruta del video tal como se guarda en el proyecto: relativa si puede, absoluta
 *  si no. `rutaProyecto` es la ruta del .cdp. */
export function rutaRelativaAlProyecto(
  rutaVideo: string,
  rutaProyecto: string,
): string {
  if (!rutaVideo || !rutaProyecto) return rutaVideo;

  const video = aBarras(rutaVideo);
  const proyecto = aBarras(rutaProyecto);

  // Carpeta del proyecto: todo antes del último separador.
  const corte = proyecto.lastIndexOf("/");
  const carpetaProyecto = corte === -1 ? "" : proyecto.slice(0, corte + 1);

  // Solo tiene sentido una ruta relativa si el video cuelga de la carpeta del
  // proyecto (o es la misma carpeta). Con ".." no: al mover solo el proyecto,
  // la ruta relativa dejaría de apuntar al video.
  if (!video.startsWith(carpetaProyecto)) {
    return rutaVideo;
  }
  return video.slice(carpetaProyecto.length);
}

/** Resuelve la ruta guardada al abrir un proyecto. Las relativas se anclan a la
 *  carpeta del .cdp; las absolutas se devuelven tal cual (proyectos viejos). */
export function resolverRutaVideo(
  rutaGuardada: string,
  rutaProyecto: string,
): string {
  if (!rutaGuardada) return "";
  if (esAbsoluta(rutaGuardada)) return rutaGuardada;
  if (!rutaProyecto) return rutaGuardada;

  const proyecto = aBarras(rutaProyecto);
  const corte = proyecto.lastIndexOf("/");
  const carpetaProyecto = corte === -1 ? "" : proyecto.slice(0, corte + 1);
  return carpetaProyecto + aBarras(rutaGuardada);
}