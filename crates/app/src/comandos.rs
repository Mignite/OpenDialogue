//! Registro único de comandos: menú y teclado producen exactamente lo mismo.
//! Portado de `App.tsx:1528-1712` más Ctrl+O, añadido en la versión Rust.

use egui::{Key, Modifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comando {
    NuevoProyecto,
    AbrirProyecto,
    AbrirVideo,
    Guardar,
    GuardarComo,
    CargarSrt,
    CargarAss,
    ImportarAutosubs,
    ExportarSrt,
    ExportarJson,
    ExportarAss,
    Estilos,
    ReproducirPausa,
    CaptionAnterior,
    CaptionSiguiente,
    RetrocederFino,
    AvanzarFino,
    Retroceder,
    Avanzar,
    NudgeIzquierda,
    NudgeDerecha,
    EditarTexto,
    SimultaneoArriba,
    SimultaneoAbajo,
    AgregarFragmento,
    Eliminar,
    Deshacer,
    Rehacer,
    Copiar,
    Pegar,
    Dividir,
    Buscar,
    Ayuda,
    VelocidadMas,
    VelocidadMenos,
    Hablante(u8),
    PanelSubtitulos,
    PanelHablantes,
    PanelEstilos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seccion {
    Archivo,
    Edicion,
    Ver,
    Exportar,
    Estilos,
    Ayuda,
}

pub fn seccion(comando: Comando) -> Seccion {
    match comando {
        Comando::NuevoProyecto
        | Comando::AbrirProyecto
        | Comando::AbrirVideo
        | Comando::Guardar
        | Comando::GuardarComo
        | Comando::CargarSrt
        | Comando::CargarAss
        | Comando::ImportarAutosubs => Seccion::Archivo,
        Comando::ExportarSrt | Comando::ExportarJson | Comando::ExportarAss => Seccion::Exportar,
        Comando::Estilos => Seccion::Estilos,
        Comando::PanelSubtitulos | Comando::PanelHablantes | Comando::PanelEstilos => Seccion::Ver,
        Comando::Ayuda => Seccion::Ayuda,
        Comando::ReproducirPausa
        | Comando::CaptionAnterior
        | Comando::CaptionSiguiente
        | Comando::RetrocederFino
        | Comando::AvanzarFino
        | Comando::Retroceder
        | Comando::Avanzar
        | Comando::NudgeIzquierda
        | Comando::NudgeDerecha
        | Comando::EditarTexto
        | Comando::SimultaneoArriba
        | Comando::SimultaneoAbajo
        | Comando::AgregarFragmento
        | Comando::Eliminar
        | Comando::Deshacer
        | Comando::Rehacer
        | Comando::Copiar
        | Comando::Pegar
        | Comando::Dividir
        | Comando::Buscar
        | Comando::VelocidadMas
        | Comando::VelocidadMenos
        | Comando::Hablante(_) => Seccion::Edicion,
    }
}

/// En el arranque no hay proyecto ni timeline: esas entradas se muestran
/// deshabilitadas en lugar de desaparecer.
pub fn disponible_en_inicio(comando: Comando) -> bool {
    matches!(
        comando,
        Comando::NuevoProyecto | Comando::AbrirProyecto | Comando::AbrirVideo | Comando::Ayuda
    )
}

pub fn etiqueta(comando: Comando) -> String {
    let texto = match comando {
        Comando::NuevoProyecto => "Nuevo proyecto",
        Comando::AbrirProyecto => "Abrir proyecto",
        Comando::AbrirVideo => "Abrir video",
        Comando::Guardar => "Guardar proyecto",
        Comando::GuardarComo => "Guardar como...",
        Comando::CargarSrt => "Cargar SRT",
        Comando::CargarAss => "Cargar .ass",
        Comando::ImportarAutosubs => "Importar auto-subs (SRT+TXT)",
        Comando::ExportarSrt => "Exportar SRT por hablante",
        Comando::ExportarJson => "Exportar JSON combinado",
        Comando::ExportarAss => "Exportar .ass...",
        Comando::Estilos => "Estilos...",
        Comando::ReproducirPausa => "Reproducir / pausa",
        Comando::CaptionAnterior => "Caption anterior",
        Comando::CaptionSiguiente => "Caption siguiente",
        Comando::RetrocederFino => "Retroceder 0,1 s",
        Comando::AvanzarFino => "Avanzar 0,1 s",
        Comando::Retroceder => "Retroceder 5 s",
        Comando::Avanzar => "Avanzar 5 s",
        Comando::NudgeIzquierda => "Mover caption 0,1 s a la izquierda",
        Comando::NudgeDerecha => "Mover caption 0,1 s a la derecha",
        Comando::EditarTexto => "Editar texto",
        Comando::SimultaneoArriba => "Caption simultáneo anterior",
        Comando::SimultaneoAbajo => "Caption simultáneo siguiente",
        Comando::AgregarFragmento => "Añadir fragmento",
        Comando::Eliminar => "Eliminar",
        Comando::Deshacer => "Deshacer",
        Comando::Rehacer => "Rehacer",
        Comando::Copiar => "Copiar",
        Comando::Pegar => "Pegar",
        Comando::Dividir => "Dividir en el playhead",
        Comando::Buscar => "Buscar",
        Comando::Ayuda => "Atajos de teclado",
        Comando::VelocidadMas => "Aumentar velocidad",
        Comando::VelocidadMenos => "Reducir velocidad",
        Comando::Hablante(n) => return format!("Hablante {n}"),
        Comando::PanelSubtitulos => "Panel de subtítulos",
        Comando::PanelHablantes => "Panel de hablantes",
        Comando::PanelEstilos => "Panel de estilos",
    };
    texto.to_owned()
}

pub fn atajo(comando: Comando) -> String {
    let texto = match comando {
        Comando::Guardar => "Ctrl+S",
        Comando::GuardarComo => "Ctrl+Shift+S",
        Comando::AbrirProyecto => "Ctrl+O",
        Comando::Buscar => "Ctrl+F",
        Comando::ReproducirPausa => "Space",
        Comando::CaptionAnterior => "Alt+←",
        Comando::CaptionSiguiente => "Alt+→",
        Comando::RetrocederFino => "Shift+←",
        Comando::AvanzarFino => "Shift+→",
        Comando::Retroceder => "←",
        Comando::Avanzar => "→",
        Comando::NudgeIzquierda => "Ctrl+Shift+←",
        Comando::NudgeDerecha => "Ctrl+Shift+→",
        Comando::EditarTexto => "E",
        Comando::SimultaneoArriba => "↑",
        Comando::SimultaneoAbajo => "↓",
        Comando::AgregarFragmento => "A",
        Comando::Eliminar => "Delete",
        Comando::Deshacer => "Ctrl+Z",
        Comando::Rehacer => "Ctrl+Y",
        Comando::Copiar => "Ctrl+C",
        Comando::Pegar => "Ctrl+V",
        Comando::Dividir => "C",
        Comando::Ayuda => "?",
        Comando::VelocidadMas => "+",
        Comando::VelocidadMenos => "-",
        Comando::Hablante(n) => return n.to_string(),
        _ => "",
    };
    texto.to_owned()
}

/// Carácter lógico de una tecla de egui, en minúsculas. La rejilla de egui no
/// tiene `Key::Character`: las letras y dígitos son variantes propias.
fn character(key: Key) -> Option<char> {
    match key {
        Key::Equals => Some('='),
        Key::Plus => Some('+'),
        Key::Minus => Some('-'),
        Key::Questionmark => Some('?'),
        _ => {
            let nombre = key.name();
            let mut caracteres = nombre.chars();
            match (caracteres.next(), caracteres.next()) {
                (Some(letra), None) if letra.is_ascii_alphanumeric() => {
                    Some(letra.to_ascii_lowercase())
                }
                _ => None,
            }
        }
    }
}

/// Un único registro de atajos para menú y teclado. `update` de egui entrega
/// la tecla y los modificadores del evento; el foco lo decide egui, igual que
/// lo decidía `iced::keyboard::listen()`.
pub fn desde_input(key: Key, modifiers: Modifiers) -> Option<Comando> {
    let ctrl = modifiers.ctrl || modifiers.command || modifiers.mac_cmd;
    let shift = modifiers.shift;
    let alt = modifiers.alt;

    if ctrl {
        match character(key) {
            Some('s') => {
                return Some(if shift {
                    Comando::GuardarComo
                } else {
                    Comando::Guardar
                });
            }
            Some('f') => return Some(Comando::Buscar),
            Some('o') => return Some(Comando::AbrirProyecto),
            Some('z') => {
                return Some(if shift {
                    Comando::Rehacer
                } else {
                    Comando::Deshacer
                });
            }
            Some('y') => return Some(Comando::Rehacer),
            Some('c') => return Some(Comando::Copiar),
            Some('v') => return Some(Comando::Pegar),
            _ => {}
        }
        if shift && key == Key::ArrowLeft {
            return Some(Comando::NudgeIzquierda);
        }
        if shift && key == Key::ArrowRight {
            return Some(Comando::NudgeDerecha);
        }
        return None;
    }

    match key {
        Key::Space => Some(Comando::ReproducirPausa),
        Key::ArrowLeft => Some(if shift {
            Comando::RetrocederFino
        } else if alt {
            Comando::CaptionAnterior
        } else {
            Comando::Retroceder
        }),
        Key::ArrowRight => Some(if shift {
            Comando::AvanzarFino
        } else if alt {
            Comando::CaptionSiguiente
        } else {
            Comando::Avanzar
        }),
        Key::ArrowUp => Some(Comando::SimultaneoArriba),
        Key::ArrowDown => Some(Comando::SimultaneoAbajo),
        Key::Delete => Some(Comando::Eliminar),
        _ => match character(key)?.to_ascii_lowercase() {
            'e' => Some(Comando::EditarTexto),
            'a' => Some(Comando::AgregarFragmento),
            'c' => Some(Comando::Dividir),
            '?' => Some(Comando::Ayuda),
            '+' | '=' => Some(Comando::VelocidadMas),
            '-' | '_' => Some(Comando::VelocidadMenos),
            numero @ '1'..='9' => Some(Comando::Hablante(numero as u8 - b'0')),
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(c: &str) -> Key {
        egui::Key::from_name(c).expect("tecla conocida")
    }

    fn mods(ctrl: bool, shift: bool, alt: bool) -> Modifiers {
        Modifiers {
            ctrl,
            command: ctrl,
            shift,
            alt,
            ..Default::default()
        }
    }

    #[test]
    fn ctrl_s_es_guardar() {
        let mods = egui::Modifiers {
            ctrl: true,
            ..Default::default()
        };
        assert_eq!(desde_input(egui::Key::S, mods), Some(Comando::Guardar));
    }

    #[test]
    fn ctrl_s_y_ctrl_f_funcionan_dentro_de_un_campo() {
        assert_eq!(
            desde_input(key("s"), mods(true, false, false)),
            Some(Comando::Guardar)
        );
        assert_eq!(
            desde_input(key("f"), mods(true, false, false)),
            Some(Comando::Buscar)
        );
    }

    #[test]
    fn ctrl_z_se_mapea_a_deshacer() {
        assert_eq!(
            desde_input(key("z"), mods(true, false, false)),
            Some(Comando::Deshacer)
        );
    }

    #[test]
    fn ctrl_shift_s_es_guardar_como() {
        assert_eq!(
            desde_input(key("s"), mods(true, true, false)),
            Some(Comando::GuardarComo)
        );
    }

    #[test]
    fn flechas_con_modificadores() {
        assert_eq!(
            desde_input(Key::ArrowLeft, mods(false, false, true)),
            Some(Comando::CaptionAnterior)
        );
        assert_eq!(
            desde_input(Key::ArrowRight, mods(false, true, false)),
            Some(Comando::AvanzarFino)
        );
        assert_eq!(
            desde_input(Key::ArrowRight, mods(false, false, false)),
            Some(Comando::Avanzar)
        );
    }

    #[test]
    fn teclas_de_hablante_y_atajos_simples() {
        assert_eq!(
            desde_input(key("7"), mods(false, false, false)),
            Some(Comando::Hablante(7))
        );
        assert_eq!(
            desde_input(key("c"), mods(false, false, false)),
            Some(Comando::Dividir)
        );
        assert_eq!(
            desde_input(key("?"), mods(false, false, false)),
            Some(Comando::Ayuda)
        );
        assert_eq!(
            desde_input(Key::Space, mods(false, false, false)),
            Some(Comando::ReproducirPausa)
        );
    }
}
