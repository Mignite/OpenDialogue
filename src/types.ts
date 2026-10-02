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

export interface TrackInfo {
  index: number;
  nombre: string;
  sample_rate: number;
  canales: number;
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
}
