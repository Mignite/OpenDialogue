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
    Ajustes,
}

impl Comando {
    /// Todos los comandos, para alimentar el panel de atajos y los textos.
    pub fn todas() -> &'static [Comando] {
        &[
            Comando::NuevoProyecto,
            Comando::AbrirProyecto,
            Comando::AbrirVideo,
            Comando::Guardar,
            Comando::GuardarComo,
            Comando::CargarSrt,
            Comando::CargarAss,
            Comando::ImportarAutosubs,
            Comando::ExportarSrt,
            Comando::ExportarJson,
            Comando::ExportarAss,
            Comando::Estilos,
            Comando::ReproducirPausa,
            Comando::CaptionAnterior,
            Comando::CaptionSiguiente,
            Comando::RetrocederFino,
            Comando::AvanzarFino,
            Comando::Retroceder,
            Comando::Avanzar,
            Comando::NudgeIzquierda,
            Comando::NudgeDerecha,
            Comando::EditarTexto,
            Comando::SimultaneoArriba,
            Comando::SimultaneoAbajo,
            Comando::AgregarFragmento,
            Comando::Eliminar,
            Comando::Deshacer,
            Comando::Rehacer,
            Comando::Copiar,
            Comando::Pegar,
            Comando::Dividir,
            Comando::Buscar,
            Comando::Ayuda,
            Comando::VelocidadMas,
            Comando::VelocidadMenos,
            Comando::Hablante(1),
            Comando::Hablante(2),
            Comando::Hablante(3),
            Comando::Hablante(4),
            Comando::Hablante(5),
            Comando::Hablante(6),
            Comando::Hablante(7),
            Comando::Hablante(8),
            Comando::Hablante(9),
            Comando::PanelSubtitulos,
            Comando::PanelHablantes,
            Comando::PanelEstilos,
            Comando::Ajustes,
        ]
    }
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
        Comando::Ajustes => Seccion::Edicion,
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

/// Error al intentar asignar una combinación que ya tiene otro comando.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conflicto;

/// Combinación de tecla y modificadores.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Atajo {
    pub tecla: Key,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Atajo {
    fn desde(tecla: Key, m: Modifiers) -> Self {
        Self {
            tecla,
            ctrl: m.ctrl || m.command || m.mac_cmd,
            shift: m.shift,
            alt: m.alt,
        }
    }
}

impl std::fmt::Display for Atajo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ctrl {
            write!(f, "Ctrl+")?;
        }
        if self.shift {
            write!(f, "Shift+")?;
        }
        if self.alt {
            write!(f, "Alt+")?;
        }
        write!(f, "{}", self.tecla.name())
    }
}

/// Mapa configurable de comandos a atajos. Puede tener alias (varios atajos
/// para el mismo comando), pero nunca una tecla compartida por dos comandos.
#[derive(Debug, Clone)]
pub struct MapaAtajos {
    filas: Vec<(Comando, Atajo)>,
}

impl Default for MapaAtajos {
    fn default() -> Self {
        Self::por_defecto()
    }
}

impl MapaAtajos {
    fn fila(comando: Comando, tecla: Key, ctrl: bool, shift: bool, alt: bool) -> (Comando, Atajo) {
        (
            comando,
            Atajo {
                tecla,
                ctrl,
                shift,
                alt,
            },
        )
    }

    /// Los atajos que hoy produce `desde_input`, en filas planas.
    pub fn por_defecto() -> Self {
        let mut filas = vec![
            Self::fila(Comando::Guardar, Key::S, true, false, false),
            Self::fila(Comando::GuardarComo, Key::S, true, true, false),
            Self::fila(Comando::Buscar, Key::F, true, false, false),
            Self::fila(Comando::Ajustes, Key::Comma, true, false, false),
            Self::fila(Comando::CargarSrt, Key::I, true, true, false),
            Self::fila(Comando::AbrirProyecto, Key::O, true, false, false),
            Self::fila(Comando::Deshacer, Key::Z, true, false, false),
            Self::fila(Comando::Rehacer, Key::Z, true, true, false),
            Self::fila(Comando::Rehacer, Key::Y, true, false, false),
            Self::fila(Comando::Copiar, Key::C, true, false, false),
            Self::fila(Comando::Pegar, Key::V, true, false, false),
            Self::fila(Comando::NudgeIzquierda, Key::ArrowLeft, true, true, false),
            Self::fila(Comando::NudgeDerecha, Key::ArrowRight, true, true, false),
            Self::fila(Comando::ReproducirPausa, Key::Space, false, false, false),
            Self::fila(Comando::CaptionAnterior, Key::ArrowLeft, false, false, true),
            Self::fila(
                Comando::CaptionSiguiente,
                Key::ArrowRight,
                false,
                false,
                true,
            ),
            Self::fila(Comando::RetrocederFino, Key::ArrowLeft, false, true, false),
            Self::fila(Comando::AvanzarFino, Key::ArrowRight, false, true, false),
            Self::fila(Comando::Retroceder, Key::ArrowLeft, false, false, false),
            Self::fila(Comando::Avanzar, Key::ArrowRight, false, false, false),
            Self::fila(Comando::SimultaneoArriba, Key::ArrowUp, false, false, false),
            Self::fila(
                Comando::SimultaneoAbajo,
                Key::ArrowDown,
                false,
                false,
                false,
            ),
            Self::fila(Comando::Eliminar, Key::Delete, false, false, false),
            Self::fila(Comando::EditarTexto, Key::E, false, false, false),
            Self::fila(Comando::AgregarFragmento, Key::A, false, false, false),
            Self::fila(Comando::Dividir, Key::C, false, false, false),
            Self::fila(Comando::Ayuda, Key::Questionmark, false, false, false),
            Self::fila(Comando::Ayuda, Key::F1, false, false, false),
            Self::fila(Comando::VelocidadMas, Key::Plus, false, false, false),
            Self::fila(Comando::VelocidadMas, Key::Equals, false, true, false),
            Self::fila(Comando::VelocidadMenos, Key::Minus, false, false, false),
        ];
        for n in 1..=9 {
            if let Some(tecla) = Key::from_name(&n.to_string()) {
                filas.push(Self::fila(Comando::Hablante(n), tecla, false, false, false));
            }
        }
        Self { filas }
    }

    /// Primer comando que coincide. Con foco en texto, las teclas desnudas
    /// no disparan: solo pasan las combinaciones con Ctrl o Alt.
    pub fn comando_de(&self, tecla: Key, modif: Modifiers, foco_en_texto: bool) -> Option<Comando> {
        let atajo = Atajo::desde(tecla, modif);
        if foco_en_texto && !atajo.ctrl && !atajo.alt {
            return None;
        }
        self.filas
            .iter()
            .find(|(_, candidato)| *candidato == atajo)
            .map(|(comando, _)| *comando)
    }

    /// Atajo principal de un comando, para menús y panel de atajos.
    pub fn atajo(&self, comando: Comando) -> Option<Atajo> {
        self.filas
            .iter()
            .find(|(c, _)| *c == comando)
            .map(|(_, a)| *a)
    }

    /// Asigna un atajo. Fallar si otra combinación ya lo usa.
    pub fn asignar(&mut self, comando: Comando, atajo: Atajo) -> Result<(), Conflicto> {
        if self.filas.iter().any(|(c, a)| *a == atajo && *c != comando) {
            return Err(Conflicto);
        }
        self.filas.retain(|(c, _)| *c != comando);
        self.filas.push((comando, atajo));
        Ok(())
    }

    /// Intercambia los atajos principales de dos comandos.
    pub fn intercambiar(&mut self, a: Comando, b: Comando) {
        let ia = self.filas.iter().position(|(c, _)| *c == a);
        let ib = self.filas.iter().position(|(c, _)| *c == b);
        if let (Some(ia), Some(ib)) = (ia, ib) {
            let temporal = self.filas[ia].1;
            self.filas[ia].1 = self.filas[ib].1;
            self.filas[ib].1 = temporal;
        }
    }

    pub fn restablecer(&mut self, comando: Comando) {
        let defecto = Self::por_defecto();
        self.filas.retain(|(c, _)| *c != comando);
        self.filas
            .extend(defecto.filas.iter().filter(|(c, _)| *c == comando).cloned());
    }

    pub fn restablecer_todo(&mut self) {
        *self = Self::por_defecto();
    }

    /// Serialización portable: el comando y la tecla se guardan por nombre.
    pub(crate) fn guardadas(&self) -> Vec<(String, crate::preferencias::AtajoGuardado)> {
        self.filas
            .iter()
            .map(|(c, a)| {
                (
                    format!("{c:?}"),
                    crate::preferencias::AtajoGuardado {
                        tecla: a.tecla.name().to_owned(),
                        ctrl: a.ctrl,
                        shift: a.shift,
                        alt: a.alt,
                    },
                )
            })
            .collect()
    }

    pub(crate) fn desde_guardadas(filas: &[(String, crate::preferencias::AtajoGuardado)]) -> Self {
        let mut mapa = Self { filas: Vec::new() };
        for (nombre, guardado) in filas {
            let Some(comando) = Comando::todas()
                .iter()
                .copied()
                .find(|c| format!("{c:?}") == *nombre)
            else {
                continue;
            };
            let Some(tecla) = Key::from_name(&guardado.tecla) else {
                continue;
            };
            mapa.filas.push((
                comando,
                Atajo {
                    tecla,
                    ctrl: guardado.ctrl,
                    shift: guardado.shift,
                    alt: guardado.alt,
                },
            ));
        }
        let defecto = Self::por_defecto();
        for comando in Comando::todas() {
            if !mapa.filas.iter().any(|(c, _)| c == comando) {
                mapa.filas
                    .extend(defecto.filas.iter().filter(|(c, _)| c == comando).cloned());
            }
        }
        mapa
    }
}

/// Un único registro de atajos para menú y teclado. `update` de egui entrega
/// la tecla y los modificadores del evento; el foco lo decide egui, igual que
/// lo decidía `iced::keyboard::listen()`.
pub fn desde_input(key: Key, modifiers: Modifiers) -> Option<Comando> {
    MapaAtajos::por_defecto().comando_de(key, modifiers, false)
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
