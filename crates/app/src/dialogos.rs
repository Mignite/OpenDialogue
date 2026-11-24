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
