//! Barra de menú dibujada por egui, construida desde el registro de comandos.
//! El menú no decide nada: solo publica el `Comando` que el usuario eligió.

use crate::comandos::{self, Comando, MapaAtajos, Seccion};
use crate::textos::Textos;

use crate::ui::kit;
use crate::ui::tema::{self, Estilo};

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

/// Ancho de una cadena con la fuente dada, para alinear columnas sin estimar.
fn ancho_texto(ui: &egui::Ui, texto: &str, fuente: egui::FontId) -> f32 {
    ui.painter()
        .layout_no_wrap(texto.to_owned(), fuente, tema::TEXTO)
        .size()
        .x
}

/// Barra de menú dibujada con egui. Cuatro secciones con lo esencial.
pub fn mostrar(
    ui: &mut egui::Ui,
    en_inicio: bool,
    textos: &Textos,
    atajos: &MapaAtajos,
) -> Option<Comando> {
    let mut elegido = None;
    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = tema::ESP_12;
        for seccion in SECCIONES {
            let nombre = nombre_seccion(seccion, textos);
            let fuente = tema::fuente(Estilo::Base);
            let ancho_nombre = ui
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
                        .min_size(egui::vec2(ancho_nombre + tema::ESP_20, tema::ALTO_CHIP)),
                )
                .ui(ui, |ui| {
                    let comandos = comandos_de(seccion);
                    let fuente_base = tema::fuente(Estilo::Base);
                    let fuente_mono = tema::fuente(Estilo::Mono);
                    let ancho_etiqueta = comandos
                        .iter()
                        .map(|comando| {
                            ancho_texto(
                                ui,
                                &tema::acortar(textos.comando(*comando), 22),
                                fuente_base.clone(),
                            )
                        })
                        .fold(0.0, f32::max)
                        + tema::ESP_20;
                    let ancho_atajo = comandos
                        .iter()
                        .filter_map(|comando| atajos.atajo(*comando))
                        .map(|atajo| ancho_texto(ui, &atajo.to_string(), fuente_mono.clone()))
                        .fold(0.0, f32::max);
                    let ancho_menu =
                        (ancho_etiqueta + ancho_atajo + tema::ESP_12 + tema::ESP_20 + tema::ESP_8)
                            .max(tema::ANCHO_FILTRO);
                    ui.set_min_width(ancho_menu);
                    ui.set_max_width(ancho_menu);

                    // Hover y active de cada entrada, con el atajo en una
                    // segunda columna alineada a la derecha.
                    ui.scope(|ui| {
                        ui.visuals_mut().widgets.hovered.bg_fill = tema::CONTROL;
                        ui.visuals_mut().widgets.active.bg_fill = tema::ELEVADO;
                        egui::Grid::new(nombre)
                            .num_columns(2)
                            .spacing(egui::vec2(tema::ESP_12, tema::ESP_2))
                            .show(ui, |ui| {
                                for comando in comandos {
                                    let habilitado =
                                        !en_inicio || comandos::disponible_en_inicio(comando);
                                    let etiqueta = egui::RichText::new(tema::acortar(
                                        textos.comando(comando),
                                        22,
                                    ))
                                    .font(tema::fuente(Estilo::Base));
                                    let respuesta = ui.add_enabled(
                                        habilitado,
                                        egui::Button::new(etiqueta).min_size(egui::vec2(
                                            ancho_etiqueta,
                                            tema::ALTO_CONTROL,
                                        )),
                                    );
                                    if respuesta.has_focus() {
                                        kit::anillo_foco(ui, &respuesta);
                                    }
                                    if respuesta.clicked() {
                                        elegido = Some(comando);
                                        ui.close();
                                    }
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if let Some(atajo) = atajos.atajo(comando) {
                                                ui.label(
                                                    egui::RichText::new(atajo.to_string())
                                                        .font(tema::fuente(Estilo::Mono))
                                                        .color(tema::TEXTO_2),
                                                );
                                            }
                                        },
                                    );
                                    ui.end_row();
                                }
                            });
                    });
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
    fn en_inicio_hay_acciones_de_archivo_ayuda_y_ajustes() {
        let habilitadas: Vec<_> = TODOS
            .iter()
            .copied()
            .filter(|comando| comandos::disponible_en_inicio(*comando))
            .collect();
        assert_eq!(habilitadas.len(), 4);
        assert!(habilitadas.contains(&Comando::Ajustes));
        assert!(habilitadas.contains(&Comando::AbrirProyecto));
    }
}
