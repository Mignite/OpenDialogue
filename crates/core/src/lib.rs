pub mod ass;
pub mod ass_presets;
pub mod autosubs;
pub mod busqueda;
pub mod color_hablante;
pub mod editing;
pub mod islands;
pub mod model;
pub mod portapapeles;
pub mod selection;
pub mod srt;
pub mod time;

pub use ass::{
    ASS_EVENTS_FORMAT, ASS_STYLE_FORMAT, AssParseResult, EstiloAss, Hablante, LineaOverlay,
    PresetAss, SegmentoAss, ass_color_to_hex, build_ass, escape_ass_text, format_ass_time,
    fusionar_lineas, hex_to_ass_color, hex_to_ass_color_with_alpha, lineas_overlay_activas,
    overrides_de_linea, parse_ass, parse_ass_time, preset_desde_estilos, preset_para_exportar,
    preset_por_defecto, sanitize_nombre_dialogo, segmentar_por_solape, unescape_ass_text,
};
pub use ass_presets::{
    nombre_libre, nuevo_preset, nuevo_preset_desde, presets_a_json, presets_desde_json,
};
pub use autosubs::{
    TurnoAutosubs, asignar_hablantes_por_texto, hablantes_desde_nombres, normalizar_texto,
    parse_autosubs_txt,
};
pub use busqueda::{
    SegmentoTexto, coincide, normalizar_termino, partir_resaltado, reemplazar_todos,
};
pub use color_hablante::color_hablante;
pub use editing::{
    DURACION_MINIMA, OverlapEntry, SNAP_THRESHOLD, build_overlap_report, clamp_caption_tiempos,
    clamp_delta_bloque, find_snap_block_delta, find_snap_time, format_overlap_report,
};
pub use islands::{TramoAudio, buscar_tramo_audio, pico_en_rango};
pub use model::{Preset, Proyecto};
pub use portapapeles::{caption_a_portapapeles, texto_del_portapapeles};
pub use selection::{
    AgrupacionCarriles, RectMarquee, agrupar_captions_por_carril, caption_row_index,
    filas_destino_relativas, filtrar_por_marquee,
};
pub use srt::{
    Caption, build_srt, format_srt_timestamp, nombre_archivo_hablante_unico, parse_srt,
    parse_srt_time,
};
pub use time::{format_time, parse_time_input};
