pub mod ass;
pub mod editing;
pub mod islands;
pub mod model;
pub mod srt;
pub mod time;

pub use ass::{
    ASS_EVENTS_FORMAT, ASS_STYLE_FORMAT, AssParseResult, EstiloAss, Hablante, LineaOverlay,
    PresetAss, SegmentoAss, ass_color_to_hex, build_ass, escape_ass_text, format_ass_time,
    fusionar_lineas, hex_to_ass_color, hex_to_ass_color_with_alpha, lineas_overlay_activas,
    overrides_de_linea, parse_ass, parse_ass_time, preset_desde_estilos, preset_para_exportar,
    sanitize_nombre_dialogo, segmentar_por_solape, unescape_ass_text,
};
pub use editing::{
    DURACION_MINIMA, OverlapEntry, SNAP_THRESHOLD, build_overlap_report, clamp_caption_tiempos,
    clamp_delta_bloque, find_snap_time, format_overlap_report,
};
pub use islands::{TramoAudio, buscar_tramo_audio, pico_en_rango};
pub use model::{Preset, Proyecto};
pub use srt::{Caption, build_srt, format_srt_timestamp, parse_srt, parse_srt_time};
pub use time::{format_time, parse_time_input};
