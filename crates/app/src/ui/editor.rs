//! Vista del editor en egui. Compone el preview, la barra de transporte, los
//! paneles laterales y la barra de estado. Devuelve **todos** los mensajes del
//! frame: sobrescribirlos habría dejado los botones inertes.

use std::sync::{Arc, Mutex};

use egui::Ui;

use crate::comandos::MapaAtajos;
use crate::editor_app::{EditorApp, MensajeEditor, Panel};
use crate::preferencias::Preferencias;
use crate::textos::Textos;
use crate::ui::kit;
use crate::ui::paneles::{estilos, hablantes, subtitulos};
use crate::ui::{tema, transporte};
use crate::video::Gpu;

/// Medidas del editor, persistidas en `Preferencias`.
#[derive(Debug, Clone, PartialEq)]
pub struct MedidasEditor {
    pub ancho_panel: f32,
    pub alto_timeline: f32,
    pub panel_visible: bool,
}

impl Default for MedidasEditor {
    fn default() -> Self {
        Self {
            ancho_panel: 320.0,
            alto_timeline: 260.0,
            panel_visible: true,
        }
    }
}

impl From<&Preferencias> for MedidasEditor {
    fn from(prefs: &Preferencias) -> Self {
        Self {
            ancho_panel: prefs.ancho_panel,
            alto_timeline: prefs.alto_timeline,
            panel_visible: prefs.panel_visible,
        }
    }
}

/// Distribución: inspector a la derecha, timeline abajo, monitor al centro.
pub fn show(
    ctx: &egui::Context,
    state: &mut EditorApp,
    formato: wgpu::TextureFormat,
    gpu: Arc<Mutex<Option<Gpu>>>,
    textos: &Textos,
    atajos: &MapaAtajos,
    medidas: &mut MedidasEditor,
) -> Vec<MensajeEditor> {
    let _ = atajos;
    let mut mensajes = Vec::new();

    if medidas.panel_visible {
        let lado = egui::SidePanel::right("inspector")
            .resizable(true)
            .default_width(medidas.ancho_panel)
            .min_width(280.0)
            .max_width(420.0)
            .frame(tema::panel(tema::PANEL))
            .show(ctx, |ui| {
                let titulo = match state.panel {
                    Panel::Subtitulos => textos.tab_subtitulos,
                    Panel::Hablantes => textos.tab_hablantes,
                    Panel::Estilos => textos.tab_estilos,
                };
                kit::panel_header(ui, titulo);
                if let Some(panel) = kit::tab_bar(
                    ui,
                    &[
                        (Panel::Subtitulos, textos.tab_subtitulos, None),
                        (Panel::Hablantes, textos.tab_hablantes, None),
                        (Panel::Estilos, textos.tab_estilos, None),
                    ],
                    state.panel,
                ) {
                    mensajes.push(MensajeEditor::Panel(panel));
                }
                egui::ScrollArea::vertical().show(ui, |ui| match state.panel {
                    Panel::Subtitulos => mensajes.extend(subtitulos::mostrar(ui, state, textos)),
                    Panel::Hablantes => mensajes.extend(hablantes::mostrar(ui, state)),
                    Panel::Estilos => mensajes.extend(estilos::mostrar(ui, state)),
                });
            });
        medidas.ancho_panel = lado.response.rect.width();
    }

    let abajo = egui::TopBottomPanel::bottom("timeline")
        .resizable(true)
        .default_height(medidas.alto_timeline)
        .min_height(160.0)
        .frame(tema::panel(tema::FONDO))
        .show(ctx, |ui| {
            kit::panel_header(ui, "Timeline");
            mensajes.extend(crate::ui::timeline::mostrar(ui, state));
        });
    medidas.alto_timeline = abajo.response.rect.height();

    egui::CentralPanel::default()
        .frame(tema::panel(tema::FONDO))
        .show(ctx, |ui| {
            mensajes.extend(mostrar(ui, state, formato, gpu, textos));
        });

    mensajes
}

pub fn mostrar(
    ui: &mut Ui,
    state: &mut EditorApp,
    formato: wgpu::TextureFormat,
    gpu: Arc<Mutex<Option<Gpu>>>,
    textos: &Textos,
) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();

    // Preview real: el mismo pipeline de video + libass que antes, ahora a
    // través de un paint callback de egui_wgpu.
    let alto = (ui.available_height() - 56.0).max(180.0);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), alto), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(6), tema::SUPERFICIE);
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(6),
        egui::Stroke::new(1.0_f32, tema::BORDE),
        egui::StrokeKind::Inside,
    );
    if !state.pixels.is_empty() {
        let frame = crate::video::Frame {
            pixels: Arc::new(state.pixels.clone()),
            width: crate::preview::CANVAS.0,
            height: crate::preview::CANVAS.1,
            capas: Arc::clone(&state.layers),
        };
        crate::ui::video::pintar(ui, rect, frame, formato, gpu);
    }

    mensajes.extend(transporte::mostrar(
        ui,
        state,
        textos,
        &crate::comandos::MapaAtajos::por_defecto(),
    ));

    transporte::barra_estado(ui, state, textos);
    mensajes
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use egui_kittest::kittest::Queryable;

    use super::MedidasEditor;
    use crate::editor_app::{EditorApp, MensajeEditor, update};

    fn editor() -> EditorApp {
        let video = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../testdata/clip-15s-2audio.mp4"
        );
        EditorApp::desde_video(video).expect("video de prueba")
    }

    #[test]
    fn aplicar_texto_cambia_el_caption_y_registra_un_paso() {
        let mut editor = editor();
        editor.editor.agregar(rod_core::Caption {
            id: "cap-prueba".into(),
            inicio: 1.0,
            fin: 2.0,
            texto: "texto original".into(),
            hablante_id: None,
        });
        let id = editor.editor.proyecto().captions[0].id.clone();
        editor.editor.seleccionar_uno(&id);
        editor.limpio = editor.editor.proyecto().clone();

        // La edición inline se confirma con un mensaje; el botón que lo emite
        // se prueba en `tests/subtitulos.rs`.
        update(
            &mut editor,
            MensajeEditor::EditarInline(id, "texto nuevo".into()),
        );
        assert_eq!(editor.editor.texto_de("cap-prueba"), Some("texto nuevo"));
        assert!(editor.editor.puede_deshacer());
        assert!(!editor.texto_inline_activo());
    }
}
