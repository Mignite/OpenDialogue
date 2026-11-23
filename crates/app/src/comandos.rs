//! Registro único de comandos: menú y teclado producen exactamente lo mismo.
//! Portado de `App.tsx:1528-1712` más Ctrl+O, añadido en la versión Rust.

use iced::keyboard::{Key, Modifiers, key::Named};

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

/// No hace falta un guardia de foco: `iced::keyboard::listen()` solo entrega
/// eventos que ningún widget capturó, y `TextInput` captura los suyos. Así
/// Ctrl+Z/Y/C/V quedan al campo de texto y las teclas sueltas no se pierden
/// cuando el foco está en el timeline.
pub fn desde_keyboard(key: &Key, modifiers: &Modifiers) -> Option<Comando> {
    let ctrl = modifiers.command();
    let shift = modifiers.shift();
    let alt = modifiers.alt();

    if ctrl && let Key::Character(c) = key.as_ref() {
        match c.to_ascii_lowercase().as_str() {
            "s" => {
                return Some(if shift {
                    Comando::GuardarComo
                } else {
                    Comando::Guardar
                });
            }
            "f" => return Some(Comando::Buscar),
            _ => {}
        }
    }

    if ctrl {
        if let Key::Character(c) = key.as_ref() {
            return match c.to_ascii_lowercase().as_str() {
                "o" => Some(Comando::AbrirProyecto),
                "z" => Some(if shift {
                    Comando::Rehacer
                } else {
                    Comando::Deshacer
                }),
                "y" => Some(Comando::Rehacer),
                "c" => Some(Comando::Copiar),
                "v" => Some(Comando::Pegar),
                _ => None,
            };
        }
        if shift && matches!(key.as_ref(), Key::Named(Named::ArrowLeft)) {
            return Some(Comando::NudgeIzquierda);
        }
        if shift && matches!(key.as_ref(), Key::Named(Named::ArrowRight)) {
            return Some(Comando::NudgeDerecha);
        }
        return None;
    }

    match key.as_ref() {
        Key::Named(Named::Space) => Some(Comando::ReproducirPausa),
        Key::Named(Named::ArrowLeft) => Some(if shift {
            Comando::RetrocederFino
        } else if alt {
            Comando::CaptionAnterior
        } else {
            Comando::Retroceder
        }),
        Key::Named(Named::ArrowRight) => Some(if shift {
            Comando::AvanzarFino
        } else if alt {
            Comando::CaptionSiguiente
        } else {
            Comando::Avanzar
        }),
        Key::Named(Named::ArrowUp) => Some(Comando::SimultaneoArriba),
        Key::Named(Named::ArrowDown) => Some(Comando::SimultaneoAbajo),
        Key::Named(Named::Delete) => Some(Comando::Eliminar),
        Key::Character(c) => match c.to_ascii_lowercase().as_str() {
            "e" => Some(Comando::EditarTexto),
            "a" => Some(Comando::AgregarFragmento),
            "c" => Some(Comando::Dividir),
            "?" => Some(Comando::Ayuda),
            "+" | "=" => Some(Comando::VelocidadMas),
            "-" | "_" => Some(Comando::VelocidadMenos),
            n => n
                .parse::<u8>()
                .ok()
                .filter(|numero| (1..=9).contains(numero))
                .map(Comando::Hablante),
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(c: &str) -> Key {
        Key::Character(c.into())
    }

    #[test]
    fn ctrl_s_y_ctrl_f_funcionan_dentro_de_un_campo() {
        assert_eq!(
            desde_keyboard(&key("s"), &Modifiers::COMMAND),
            Some(Comando::Guardar)
        );
        assert_eq!(
            desde_keyboard(&key("f"), &Modifiers::COMMAND),
            Some(Comando::Buscar)
        );
    }

    #[test]
    fn ctrl_z_se_mapea_a_deshacer() {
        // Si un text_input está enfocado, iced captura el evento y la
        // suscripción no lo recibe: el guardia vive en el runtime, no aquí.
        assert_eq!(
            desde_keyboard(&key("z"), &Modifiers::COMMAND),
            Some(Comando::Deshacer)
        );
    }

    #[test]
    fn ctrl_shift_s_es_guardar_como() {
        assert_eq!(
            desde_keyboard(&key("s"), &(Modifiers::COMMAND | Modifiers::SHIFT)),
            Some(Comando::GuardarComo)
        );
    }

    #[test]
    fn flechas_con_modificadores() {
        assert_eq!(
            desde_keyboard(&Key::Named(Named::ArrowLeft), &Modifiers::ALT),
            Some(Comando::CaptionAnterior)
        );
        assert_eq!(
            desde_keyboard(&Key::Named(Named::ArrowRight), &Modifiers::SHIFT),
            Some(Comando::AvanzarFino)
        );
        assert_eq!(
            desde_keyboard(&Key::Named(Named::ArrowRight), &Modifiers::NONE),
            Some(Comando::Avanzar)
        );
    }

    #[test]
    fn teclas_de_hablante_y_atajos_simples() {
        assert_eq!(
            desde_keyboard(&key("7"), &Modifiers::NONE),
            Some(Comando::Hablante(7))
        );
        assert_eq!(
            desde_keyboard(&key("c"), &Modifiers::NONE),
            Some(Comando::Dividir)
        );
        assert_eq!(
            desde_keyboard(&key("?"), &Modifiers::NONE),
            Some(Comando::Ayuda)
        );
        assert_eq!(
            desde_keyboard(&Key::Named(Named::Space), &Modifiers::NONE),
            Some(Comando::ReproducirPausa)
        );
    }
}
