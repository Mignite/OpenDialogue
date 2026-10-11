// Escrituras debounced que NO se pierden al cerrar la app.
//
// El patrón "setTimeout(() => escribir(), 400)" tiene un fallo silencioso:
// si la ventana se cierra dentro de esos 400 ms, el timer nunca dispara y el
// último cambio del usuario no llega al disco. Con el auto-guardado de estilos
// eso significaba perder la edición de un preset.
//
// Estas dos funciones centralizan la mecánica. El cleanup del unmount llama a
// `descargarDebounce` con null para ejecutar lo pendiente antes de morir.

/** Programa `fn` dentro de `ms`, cancelando cualquier timer previo que le
 *  pasen en `previo`. Devuelve el handle nuevo.
 *
 *  Un solo camino de salida para programar y para hacer flush, de forma que
 *  no se puedan colar dos mecanismos distintos (que es como un `notify` y un
 *  `setTimeout` suelta acabaron peleándose por la misma variable). */
export function programarDebounce(
  previo: ReturnType<typeof setTimeout> | null,
  fn: () => void,
  ms: number,
): ReturnType<typeof setTimeout> {
  if (previo !== null) clearTimeout(previo);
  return setTimeout(fn, ms);
}

/** Flush: ejecuta lo pendiente de inmediato y devuelve null.
 *  - Si `pendiente` es null, no hay nada que ejecutar: no-op.
 *  - Se invoca desde el cleanup del unmount para no perder el último cambio. */
export function descargarDebounce(
  pendiente: ReturnType<typeof setTimeout> | null,
  ejecutar: (() => void) | null,
): null {
  if (pendiente !== null) clearTimeout(pendiente);
  if (ejecutar !== null) ejecutar();
  return null;
}