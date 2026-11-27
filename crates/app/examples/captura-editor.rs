//! Abre el editor con el clip de prueba para capturas reales de pantalla.
//! Es una herramienta de desarrollo: no forma parte de la app.

use std::sync::{Arc, Mutex};

use rod_app::editor_app::{self, EditorApp, MensajeEditor};
use rod_app::video::Gpu;

struct CapturaEditor {
    editor: EditorApp,
    formato: wgpu::TextureFormat,
    gpu: Arc<Mutex<Option<Gpu>>>,
    medidas: rod_app::ui::editor::MedidasEditor,
}

impl eframe::App for CapturaEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        editor_app::update(&mut self.editor, MensajeEditor::Tick);
        let mensajes = rod_app::ui::editor::show(
            ctx,
            &mut self.editor,
            self.formato,
            Arc::clone(&self.gpu),
            &rod_app::textos::ES,
            &rod_app::comandos::MapaAtajos::por_defecto(),
            &mut self.medidas,
        );
        for mensaje in mensajes {
            editor_app::update(&mut self.editor, mensaje);
        }
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let opciones = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_decorations(false),
        ..Default::default()
    };
    eframe::run_native(
        "OpenDialogue (capturas)",
        opciones,
        Box::new(|cc| {
            rod_app::ui::tema::aplicar(&cc.egui_ctx);
            let formato = cc
                .wgpu_render_state
                .as_ref()
                .map(|estado| estado.target_format)
                .unwrap_or(wgpu::TextureFormat::Bgra8Unorm);

            let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            let video = raiz.join("testdata/clip-15s-2audio.mp4");
            let mut proyecto = rod_core::Proyecto::nuevo(video.to_string_lossy().into_owned());
            for i in 0..7 {
                proyecto.hablantes.push(rod_core::Hablante {
                    id: format!("h{i}"),
                    nombre: format!("Hablante {i}"),
                    tecla: String::new(),
                    color: String::new(),
                    preset_id: None,
                });
            }
            for n in 0..40 {
                let inicio = n as f64 * 0.4;
                proyecto.captions.push(rod_core::Caption {
                    id: format!("c{n}"),
                    inicio,
                    fin: inicio + 0.32,
                    texto: format!("Línea {n} · hablante {}", n % 7),
                    hablante_id: Some(format!("h{}", n % 7)),
                });
            }
            let base = std::env::temp_dir().join("rod-captura-editor");
            std::fs::create_dir_all(&base).expect("crea la carpeta de miniaturas");
            let editor = EditorApp::desde_proyecto_en(proyecto, None, 0.0, &base)
                .expect("abre el clip de prueba");

            Ok(Box::new(CapturaEditor {
                editor,
                formato,
                gpu: Arc::new(Mutex::new(None)),
                medidas: rod_app::ui::editor::MedidasEditor::default(),
            }))
        }),
    )
}
