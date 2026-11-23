//! Barra de menú dibujada por iced, construida desde el registro de comandos.
//! El menú no decide nada: solo publica el `Comando` que el usuario eligió.

use iced::widget::button;
use iced_aw::menu::{Item, Menu, MenuBar};

use crate::comandos::{self, Comando, Seccion};
use crate::tema;

const SECCIONES: [Seccion; 6] = [
    Seccion::Archivo,
    Seccion::Edicion,
    Seccion::Ver,
    Seccion::Exportar,
    Seccion::Estilos,
    Seccion::Ayuda,
];

const TODOS: &[Comando] = &[
    Comando::NuevoProyecto,
    Comando::AbrirProyecto,
    Comando::AbrirVideo,
    Comando::Guardar,
    Comando::GuardarComo,
    Comando::CargarSrt,
    Comando::CargarAss,
    Comando::ImportarAutosubs,
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
    Comando::PanelSubtitulos,
    Comando::PanelHablantes,
    Comando::PanelEstilos,
    Comando::ExportarSrt,
    Comando::ExportarJson,
    Comando::ExportarAss,
    Comando::Estilos,
    Comando::Ayuda,
];

/// En el arranque no hay proyecto ni timeline: esas entradas se muestran
/// deshabilitadas en lugar de desaparecer, para que el menú sea el mismo sitio
/// en ambas pantallas.
fn disponible_en_inicio(comando: Comando) -> bool {
    matches!(
        comando,
        Comando::NuevoProyecto | Comando::AbrirProyecto | Comando::AbrirVideo | Comando::Ayuda
    )
}

fn texto(comando: Comando) -> String {
    let etiqueta = comandos::etiqueta(comando);
    let atajo = comandos::atajo(comando);
    if atajo.is_empty() {
        etiqueta
    } else {
        format!("{etiqueta}    [{atajo}]")
    }
}

/// Los comandos por sección, en el orden del registro. Un solo sitio decide
/// qué obras aparecen en el menú, igual que el teclado usa `desde_keyboard`.
pub fn comandos_de(seccion: Seccion) -> Vec<Comando> {
    TODOS
        .iter()
        .copied()
        .filter(|comando| comandos::seccion(*comando) == seccion)
        .collect()
}

pub fn barra<'a, Message>(
    en_inicio: bool,
    on_comando: impl Fn(Comando) -> Message + 'a,
) -> MenuBar<'a, Message, iced::Theme, iced::Renderer>
where
    Message: Clone + 'a,
{
    let raices = SECCIONES
        .into_iter()
        .map(|seccion| {
            let entradas = comandos_de(seccion)
                .into_iter()
                .map(|comando| {
                    let habilitado = !en_inicio || disponible_en_inicio(comando);
                    let mut boton =
                        button(iced::widget::text(texto(comando))).style(tema::boton_menu);
                    if habilitado {
                        boton = boton.on_press(on_comando(comando));
                    }
                    Item::new(boton)
                })
                .collect::<Vec<_>>();
            let nombre = match seccion {
                Seccion::Archivo => "Archivo",
                Seccion::Edicion => "Edición",
                Seccion::Ver => "Ver",
                Seccion::Exportar => "Exportar",
                Seccion::Estilos => "Estilos",
                Seccion::Ayuda => "Ayuda",
            };
            Item::with_menu(
                button(nombre).style(tema::boton_menu),
                Menu::new(entradas)
                    .max_width(280.0)
                    .padding(8.0)
                    .offset(2.0),
            )
        })
        .collect::<Vec<_>>();
    MenuBar::new(raices).style(tema::barra_menu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todos_los_comandos_aparecen_en_una_seccion() {
        let total: usize = SECCIONES
            .iter()
            .map(|seccion| comandos_de(*seccion).len())
            .sum();
        assert_eq!(total, TODOS.len());
    }

    #[test]
    fn el_registro_no_repite_comandos() {
        for (i, a) in TODOS.iter().enumerate() {
            assert!(!TODOS[i + 1..].contains(a), "comando repetido: {a:?}");
        }
    }

    #[test]
    fn menu_y_teclado_producen_el_mismo_comando() {
        let en_menu = comandos_de(Seccion::Archivo).contains(&Comando::Guardar);
        let por_teclado = crate::comandos::desde_keyboard(
            &iced::keyboard::Key::Character("s".into()),
            &iced::keyboard::Modifiers::COMMAND,
            true,
        );
        assert!(en_menu);
        assert_eq!(por_teclado, Some(Comando::Guardar));
    }

    #[test]
    fn en_inicio_solo_hay_cuatro_acciones_habilitadas() {
        let habilitadas = TODOS
            .iter()
            .copied()
            .filter(|comando| disponible_en_inicio(*comando))
            .count();
        assert_eq!(habilitadas, 4);
    }
}
