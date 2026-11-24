//! Barra de menú dibujada por iced, construida desde el registro de comandos.
//! El menú no decide nada: solo publica el `Comando` que el usuario eligió.

use crate::comandos::{self, Comando, Seccion};

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

/// Barra de menú dibujada con egui. Un `MenuButton` por sección, con los
/// comandos del registro; en el arranque solo aparecen los cuatro válidos.
pub fn mostrar(ui: &mut egui::Ui, en_inicio: bool) -> Option<Comando> {
    use crate::ui::tema;

    let mut elegido = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for (indice, seccion) in SECCIONES.into_iter().enumerate() {
            if indice > 0 {
                ui.separator();
            }
            let nombre = match seccion {
                Seccion::Archivo => "Archivo",
                Seccion::Edicion => "Edición",
                Seccion::Ver => "Ver",
                Seccion::Exportar => "Exportar",
                Seccion::Estilos => "Estilos",
                Seccion::Ayuda => "Ayuda",
            };
            let etiqueta = egui::RichText::new(nombre)
                .size(11.0)
                .color(tema::SECUNDARIO);
            ui.menu_button(etiqueta, |ui| {
                ui.set_min_width(226.0);
                for comando in comandos_de(seccion) {
                    let habilitado = !en_inicio || comandos::disponible_en_inicio(comando);
                    if ui
                        .add_enabled(habilitado, egui::Button::new(texto(comando)).frame(false))
                        .clicked()
                    {
                        elegido = Some(comando);
                        ui.close();
                    }
                }
            });
        }
    });
    elegido
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
        let por_teclado = crate::comandos::desde_input(
            egui::Key::S,
            egui::Modifiers {
                ctrl: true,
                ..Default::default()
            },
        );
        assert!(en_menu);
        assert_eq!(por_teclado, Some(Comando::Guardar));
    }

    #[test]
    fn en_inicio_solo_hay_cuatro_acciones_habilitadas() {
        let habilitadas = TODOS
            .iter()
            .copied()
            .filter(|comando| comandos::disponible_en_inicio(*comando))
            .count();
        assert_eq!(habilitadas, 4);
    }
}
