//! Barra de menú dibujada por iced, construida desde el registro de comandos.
//! El menú no decide nada: solo publica el `Comando` que el usuario eligió.

use crate::comandos::{self, Comando, MapaAtajos, Seccion};
use crate::textos::Textos;

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

/// Etiqueta y atajo se devuelven separados: el atajo nunca viaja dentro del
/// texto del botón, para que sea un nodo accesible independiente.
fn texto(comando: Comando, textos: &Textos, atajos: &MapaAtajos) -> (String, String) {
    let etiqueta = textos.comando(comando);
    let atajo = atajos
        .atajo(comando)
        .map(|a| a.to_string())
        .unwrap_or_default();
    (etiqueta.to_owned(), atajo)
}

fn nombre_seccion(seccion: Seccion, textos: &Textos) -> &'static str {
    match seccion {
        Seccion::Archivo => textos.seccion_archivo,
        Seccion::Edicion => textos.seccion_edicion,
        Seccion::Ver => textos.seccion_ver,
        Seccion::Exportar => textos.seccion_exportar,
        Seccion::Estilos => textos.seccion_estilos,
        Seccion::Ayuda => textos.seccion_ayuda,
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
pub fn mostrar(
    ui: &mut egui::Ui,
    en_inicio: bool,
    textos: &Textos,
    atajos: &MapaAtajos,
) -> Option<Comando> {
    use crate::ui::tema;

    let mut elegido = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = tema::ESP_2;
        for seccion in SECCIONES {
            let nombre = nombre_seccion(seccion, textos);
            let etiqueta = egui::RichText::new(nombre)
                .font(tema::fuente(tema::Estilo::Pequeno))
                .color(tema::TEXTO_MENU);
            let _ = egui::containers::menu::MenuButton::from_button(
                egui::Button::new(etiqueta).frame(false),
            )
            .ui(ui, |ui| {
                ui.set_min_width(226.0);
                for comando in comandos_de(seccion) {
                    let habilitado = !en_inicio || comandos::disponible_en_inicio(comando);
                    let (etiqueta, atajo) = texto(comando, textos, atajos);
                    ui.horizontal(|ui| {
                        let boton = ui.add_enabled(
                            habilitado,
                            egui::Button::new(
                                egui::RichText::new(&etiqueta)
                                    .font(tema::fuente(tema::Estilo::Base)),
                            )
                            .frame(false),
                        );
                        if !atajo.is_empty() {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(&atajo)
                                            .font(tema::fuente(tema::Estilo::Mono)),
                                    );
                                },
                            );
                        }
                        if boton.clicked() {
                            elegido = Some(comando);
                            ui.close();
                        }
                    });
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
