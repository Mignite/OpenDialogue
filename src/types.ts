export interface Hablante {
  id: string;
  nombre: string;
  tecla: string;
  color: string;
  // Preset .ass asignado al hablante (dato del proyecto, se guarda en el .cdp).
  // undefined = sin asignar → se usa el primer preset al exportar.
  presetId?: string;
}

export interface Caption {
  id: string;
  inicio: number;
  fin: number;
  texto: string;
  hablante_id: string | null;
}

export interface Proyecto {
  ruta_video: string;
  hablantes: Hablante[];
  captions: Caption[];
  playhead: number;
}

export interface OverlapEntry {
  inicio: number;
  fin: number;
  hablanteA: string;
  textoA: string;
  hablanteB: string;
  textoB: string;
}

export interface PresetAss {
  id: string;
  nombre: string;
  fontname: string;
  fontsize: number;
  color: string;
  outlineColor: string;
  outline: number;
  shadow: number;
  alignment: number;
  marginL: number;
  marginR: number;
  marginV: number;
  // Peso y cursiva del estilo .ass (campos Bold/Italic del Style:). El
  // selector de fuente lista solo familias canónicas: el peso NO va pegado al
  // nombre ("Cantarell Light" no es una familia, es "Cantarell" en negrita),
  // va en su propio campo, que es donde libass lo lee.
  // Presets viejos no traen estas claves → ausentes → false (ver esPresetAss).
  bold?: boolean;
  italic?: boolean;
}
