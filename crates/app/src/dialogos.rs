//! Diálogos nativos fuera del hilo de la interfaz.
//!
//! `rfd` bloquea al esperar al usuario. Llamarlo desde `update` congelaría el
//! frame, así que se lanza en un hilo corto y la respuesta se recoge por un
//! canal al empezar el frame siguiente. Es el sustituto del `Task::future` de
//! iced.

use std::path::PathBuf;
use std::sync::mpsc::Sender;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RespuestaDialogo {
    Video(Option<PathBuf>),
    Proyecto(Option<PathBuf>),
    Carpeta(Option<PathBuf>),
    Guardar(Option<PathBuf>),
    VideoEditor(Option<PathBuf>),
    Srt(Option<PathBuf>),
    Ass(Option<PathBuf>),
    AssGuardar(Option<PathBuf>),
    Json(Option<PathBuf>),
    AutosubsSrt(Option<PathBuf>),
    AutosubsTxt(Option<PathBuf>),
}

pub fn pedir_video(tx: Sender<RespuestaDialogo>) {
    std::thread::spawn(move || {
        let ruta = rfd::FileDialog::new()
            .add_filter("Video", &["mp4", "mkv", "mov", "webm"])
            .pick_file();
        let _ = tx.send(RespuestaDialogo::Video(ruta));
    });
}

pub fn pedir_proyecto(tx: Sender<RespuestaDialogo>) {
    std::thread::spawn(move || {
        let ruta = rfd::FileDialog::new()
            .add_filter("Proyecto", &["cdp"])
            .pick_file();
        let _ = tx.send(RespuestaDialogo::Proyecto(ruta));
    });
}

pub fn pedir_carpeta(tx: Sender<RespuestaDialogo>) {
    std::thread::spawn(move || {
        let ruta = rfd::FileDialog::new().pick_folder();
        let _ = tx.send(RespuestaDialogo::Carpeta(ruta));
    });
}

pub fn pedir_guardar(tx: Sender<RespuestaDialogo>) {
    std::thread::spawn(move || {
        let ruta = rfd::FileDialog::new()
            .add_filter("Proyecto", &["cdp"])
            .save_file();
        let _ = tx.send(RespuestaDialogo::Guardar(ruta));
    });
}

fn abrir(
    filtro: &'static str,
    extensiones: &'static [&'static str],
    guardar: bool,
    nombre: Option<&'static str>,
    tx: Sender<RespuestaDialogo>,
    convertir: fn(Option<PathBuf>) -> RespuestaDialogo,
) {
    std::thread::spawn(move || {
        let mut dialogo = rfd::FileDialog::new();
        if !extensiones.is_empty() {
            dialogo = dialogo.add_filter(filtro, extensiones);
        }
        if let Some(nombre) = nombre {
            dialogo = dialogo.set_file_name(nombre);
        }
        let ruta = if guardar {
            dialogo.save_file()
        } else {
            dialogo.pick_file()
        };
        let _ = tx.send(convertir(ruta));
    });
}

pub fn pedir_video_editor(tx: Sender<RespuestaDialogo>) {
    abrir(
        "Vídeos",
        &["mp4", "mkv", "webm", "mov", "avi"],
        false,
        None,
        tx,
        RespuestaDialogo::VideoEditor,
    );
}

pub fn pedir_srt(tx: Sender<RespuestaDialogo>) {
    abrir(
        "Subtítulos",
        &["srt"],
        false,
        None,
        tx,
        RespuestaDialogo::Srt,
    );
}

pub fn pedir_ass(tx: Sender<RespuestaDialogo>) {
    abrir(
        "Subtítulos",
        &["ass"],
        false,
        None,
        tx,
        RespuestaDialogo::Ass,
    );
}

pub fn pedir_json(tx: Sender<RespuestaDialogo>) {
    abrir(
        "JSON combinado",
        &["json"],
        true,
        Some("subtitulos_combinado.json"),
        tx,
        RespuestaDialogo::Json,
    );
}

pub fn pedir_ass_guardar(tx: Sender<RespuestaDialogo>) {
    abrir(
        "Subtítulos ASS",
        &["ass"],
        true,
        Some("subtitulos.ass"),
        tx,
        RespuestaDialogo::AssGuardar,
    );
}

pub fn pedir_autosubs_srt(tx: Sender<RespuestaDialogo>) {
    abrir(
        "Subtítulos",
        &["srt"],
        false,
        None,
        tx,
        RespuestaDialogo::AutosubsSrt,
    );
}

pub fn pedir_autosubs_txt(tx: Sender<RespuestaDialogo>) {
    abrir(
        "Texto",
        &["txt"],
        false,
        None,
        tx,
        RespuestaDialogo::AutosubsTxt,
    );
}
