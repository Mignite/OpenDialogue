//! Barra de menú dibujada por iced, construida desde el registro de comandos.
//! El menú no decide nada: solo publica el `Comando` que el usuario eligió.

use crate::comandos::{self, Comando, MapaAtajos, Seccion};
use crate::textos::Textos;

/// Solo cuatro menús: lo demás está en atajos, pestañas y Ajustes.
const SECCIONES: [Seccion; 4] = [
    Seccion::Archivo,
    Seccion::Edicion,
    Seccion::Exportar,
    Seccion::Ayuda,
];

const TODOS: &[Comando] = &[
    Comando::AbrirVideo,
    Comando::AbrirProyecto,
    Comando::Guardar,
    Comando::GuardarComo,
    Comando::CargarSrt,
    Comando::Deshacer,
    Comando::Rehacer,
    Comando::Dividir,
    Comando::Eliminar,
    Comando::Ajustes,
    Comando::ExportarSrt,
    Comando::ExportarJson,
    Comando::ExportarAss,
    Comando::Ayuda,
];

/// Los comandos por sección, en el orden del registro. Un solo sitio decide
/// qué obras aparecen en el menú, igual que el teclado usa `desde_keyboard`.
pub fn comandos_de(seccion: Seccion) -> Vec<Comando> {
    TODOS
        .iter()
        .copied()
        .filter(|comando| comandos::seccion(*comando) == seccion)
        .collect()
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

/// Barra de menú dibujada con egui. Cuatro secciones con lo esencial.
pub fn mostrar(
    ui: &mut egui::Ui,
    en_inicio: bool,
    textos: &Textos,
    atajos: &MapaAtajos,
) -> Option<Comando> {
    let _ = atajos;
    use crate::ui::tema;

    let mut elegido = None;
    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = tema::ESP_12;
        for seccion in SECCIONES {
            let nombre = nombre_seccion(seccion, textos);
            let fuente = tema::fuente(tema::Estilo::Base);
            let ancho_texto = ui
                .painter()
                .layout_no_wrap(nombre.to_owned(), fuente.clone(), tema::TEXTO_MENU)
                .size()
                .x;
            // Menú de texto: sin relleno en reposo, fondo #2C2C2C en hover.
            ui.scope(|ui| {
                ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
                ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
                ui.visuals_mut().widgets.hovered.bg_fill = tema::CONTROL;
                ui.visuals_mut().widgets.hovered.weak_bg_fill = tema::CONTROL;
                ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::NONE;
                let etiqueta = egui::RichText::new(nombre)
                    .font(fuente)
                    .color(tema::TEXTO_MENU);
                let _ = egui::containers::menu::MenuButton::from_button(
                    egui::Button::new(etiqueta)
                        .corner_radius(tema::radio_superficie())
                        .min_size(egui::vec2(ancho_texto + 20.0, 24.0)),
                )
                .ui(ui, |ui| {
                    // Sin columna de atajos: el panel de Ajustes los muestra con
                    // el valor configurado. Menú compacto, ~160 px.
                    ui.set_min_width(160.0);
                    ui.set_max_width(160.0);
                    for comando in comandos_de(seccion) {
                        let habilitado = !en_inicio || comandos::disponible_en_inicio(comando);
                        let etiqueta = textos.comando(comando);
                        let boton = ui.add_enabled(
                            habilitado,
                            egui::Button::new(
                                egui::RichText::new(tema::acortar(etiqueta, 22))
                                    .font(tema::fuente(tema::Estilo::Base)),
                            )
                            .frame(false),
                        );
                        if boton.clicked() {
                            elegido = Some(comando);
                            ui.close();
                        }
                    }
                });
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
    fn en_inicio_solo_hay_tres_acciones_habilitadas() {
        let habilitadas = TODOS
            .iter()
            .copied()
            .filter(|comando| comandos::disponible_en_inicio(*comando))
            .count();
        assert_eq!(habilitadas, 3);
    }
}
