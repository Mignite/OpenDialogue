//! El `.ass` que se rasteriza para el preview.
//!
//! Es el mismo `build_ass` que produce la exportación: si el preview y el
//! archivo divergen, es un bug (invariante del proyecto, spec §1). Vive en la
//! librería y no en `main.rs` para poder probarlo y para que un ejemplo lo
//! rasterice sin abrir la ventana.

use std::collections::HashMap;

use rod_core::{preset_desde_estilos, preset_por_defecto};

use crate::editor::EstadoEditor;
use crate::project_io::exportar_ass;

/// El canvas de composición: el `PlayResX/Y` del `.ass` exportado, no la
/// resolución del video (spec §4.4).
pub const CANVAS: (u32, u32) = (1920, 1080);

pub fn ass_del_proyecto(editor: &EstadoEditor) -> String {
    let documento = &editor.documento;
    // Sin presets propios todavía, todos los hablantes usan el estilo `Default`
    // del `.ass` cargado; un proyecto que vino de un `.srt` usa el de fábrica.
    let sin_hablante = preset_desde_estilos(&documento.estilos_ass, "Preview", "preview")
        .unwrap_or_else(preset_por_defecto);
    exportar_ass(documento, &HashMap::new(), &sin_hablante, CANVAS)
}

#[cfg(test)]
mod tests {
    use super::{CANVAS, ass_del_proyecto};
    use crate::editor::EstadoEditor;
    use crate::project_io::cargar;

    fn editor() -> EstadoEditor {
        EstadoEditor::nuevo(
            cargar(
                "1\n00:00:01,000 --> 00:00:02,000\nHola mundo\n",
                "video.mp4",
            )
            .expect("el SRT carga"),
        )
    }

    #[test]
    fn el_canvas_es_el_playres_del_ass_exportado() {
        assert_eq!(CANVAS, (1920, 1080));
        let ass = ass_del_proyecto(&editor());
        assert!(ass.contains("PlayResX: 1920"));
        assert!(ass.contains("PlayResY: 1080"));
    }

    #[test]
    fn el_texto_del_proyecto_llega_al_ass() {
        let ass = ass_del_proyecto(&editor());
        assert!(ass.contains("Hola mundo"));
    }

    #[test]
    fn el_ass_sigue_al_texto_editado() {
        let mut editor = editor();
        editor.registrar_historial();
        editor.editar_texto("cap-1", "Después");

        let ass = ass_del_proyecto(&editor);
        assert!(ass.contains("Después"));
        assert!(
            !ass.contains("Hola mundo"),
            "el .ass no debe arrastrar el texto viejo"
        );
    }
}
