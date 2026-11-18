use rod_core::{Caption, Hablante, Historial, Instantanea, Proyecto, clamp_caption_tiempos};

use crate::project_io::Documento;

#[derive(Debug, Clone, PartialEq)]
pub struct EstadoEditor {
    pub documento: Documento,
    pub seleccion: Vec<String>,
    /// El historial viaja con el editor: deshacer después de borrar tiene que
    /// recordar lo borrado, así que no puede vivir en la app.
    historial: Historial,
}

impl EstadoEditor {
    pub fn nuevo(documento: Documento) -> Self {
        Self {
            documento,
            seleccion: Vec::new(),
            historial: Historial::default(),
        }
    }

    pub fn proyecto(&self) -> &Proyecto {
        &self.documento.proyecto
    }

    pub fn seleccionar_uno(&mut self, id: &str) {
        self.seleccion.clear();
        if self
            .proyecto()
            .captions
            .iter()
            .any(|caption| caption.id == id)
        {
            self.seleccion.push(id.into());
        }
    }

    pub fn alternar_seleccion(&mut self, id: &str) {
        if let Some(position) = self.seleccion.iter().position(|selected| selected == id) {
            self.seleccion.remove(position);
        } else if self
            .proyecto()
            .captions
            .iter()
            .any(|caption| caption.id == id)
        {
            self.seleccion.push(id.into());
        }
    }

    pub fn texto_de(&self, id: &str) -> Option<&str> {
        self.proyecto()
            .captions
            .iter()
            .find(|caption| caption.id == id)
            .map(|caption| caption.texto.as_str())
    }

    pub fn editar_texto(&mut self, id: &str, texto: impl Into<String>) -> bool {
        let Some(caption) = self
            .documento
            .proyecto
            .captions
            .iter_mut()
            .find(|caption| caption.id == id)
        else {
            return false;
        };
        caption.texto = texto.into();
        true
    }

    /// Aplica el texto del editor como **un** paso de historial. `false` si el
    /// caption no existe o el texto no cambió: un Ctrl+Z sobre un texto idéntico
    /// gastaría un paso del historial sin cambiar la pantalla.
    pub fn aplicar_texto(&mut self, id: &str, texto: impl Into<String>) -> bool {
        let texto = texto.into();
        let Some(actual) = self.texto_de(id) else {
            return false;
        };
        if actual == texto {
            return false;
        }
        self.registrar_historial();
        self.editar_texto(id, texto)
    }

    pub fn editar_tiempos(&mut self, id: &str, inicio: f64, fin: f64, duracion_video: f64) -> bool {
        let Some(position) = self
            .documento
            .proyecto
            .captions
            .iter()
            .position(|caption| caption.id == id)
        else {
            return false;
        };
        let actual = &self.documento.proyecto.captions[position];
        let (inicio, fin) = clamp_caption_tiempos(inicio, fin, duracion_video, Some(actual));
        self.documento.proyecto.captions[position].inicio = inicio;
        self.documento.proyecto.captions[position].fin = fin;
        self.ordenar();
        true
    }

    pub fn asignar_hablante(&mut self, id: &str, hablante_id: Option<String>) -> bool {
        let Some(caption) = self
            .documento
            .proyecto
            .captions
            .iter_mut()
            .find(|caption| caption.id == id)
        else {
            return false;
        };
        caption.hablante_id = hablante_id;
        true
    }

    pub fn agregar(&mut self, caption: Caption) {
        self.documento.proyecto.captions.push(caption);
        self.ordenar();
    }

    pub fn borrar_seleccion(&mut self) -> usize {
        let selected = self
            .seleccion
            .iter()
            .collect::<std::collections::HashSet<_>>();
        let before = self.documento.proyecto.captions.len();
        self.documento
            .proyecto
            .captions
            .retain(|caption| !selected.contains(&caption.id));
        self.seleccion.clear();
        before - self.documento.proyecto.captions.len()
    }

    /// Guarda el estado actual antes de una edición. Se llama **antes** de
    /// tocar el documento; la app decide la granularidad (por ejemplo, una vez
    /// por texto confirmado, no por tecla).
    pub fn registrar_historial(&mut self) {
        self.historial
            .registrar(Instantanea::de_proyecto(self.proyecto()));
    }

    pub fn puede_deshacer(&self) -> bool {
        self.historial.puede_deshacer()
    }

    pub fn puede_rehacer(&self) -> bool {
        self.historial.puede_rehacer()
    }

    /// Restaura el estado anterior. `false` si no había nada que deshacer.
    pub fn deshacer(&mut self) -> bool {
        let actual = Instantanea::de_proyecto(self.proyecto());
        let Some(anterior) = self.historial.deshacer(actual) else {
            return false;
        };
        self.restaurar(anterior);
        true
    }

    pub fn rehacer(&mut self) -> bool {
        let actual = Instantanea::de_proyecto(self.proyecto());
        let Some(siguiente) = self.historial.rehacer(actual) else {
            return false;
        };
        self.restaurar(siguiente);
        true
    }

    fn restaurar(&mut self, instantanea: Instantanea) {
        self.documento.proyecto.captions = instantanea.captions;
        self.documento.proyecto.hablantes = instantanea.hablantes;
        // Deshacer puede devolver al mundo captions que ya no existían o
        // llevarse los que la selección apuntaba. La selección no es parte de
        // la instantánea (como en el oráculo), pero no puede quedar colgando de
        // ids inexistentes.
        self.seleccion
            .retain(|id| self.documento.proyecto.captions.iter().any(|c| &c.id == id));
    }

    fn ordenar(&mut self) {
        self.documento
            .proyecto
            .captions
            .sort_by(|left, right| left.inicio.total_cmp(&right.inicio));
    }
}

pub fn hablante_por_id<'a>(hablantes: &'a [Hablante], id: Option<&str>) -> Option<&'a Hablante> {
    id.and_then(|id| hablantes.iter().find(|speaker| speaker.id == id))
}

#[cfg(test)]
mod tests {
    use super::EstadoEditor;
    use crate::project_io::cargar;
    use rod_core::Caption;

    fn editor() -> EstadoEditor {
        EstadoEditor::nuevo(
            cargar(
                "1\n00:00:02,000 --> 00:00:03,000\nDos\n\n2\n00:00:01,000 --> 00:00:02,000\nUno",
                "video.mp4",
            )
            .expect("document loads"),
        )
    }

    #[test]
    fn edits_and_orders_captions() {
        let mut editor = editor();
        editor.editar_texto("cap-1", "cambiado");
        editor.editar_tiempos("cap-1", 0.0, 0.5, 10.0);
        assert_eq!(editor.proyecto().captions[0].texto, "cambiado");
        assert_eq!(editor.proyecto().captions[0].inicio, 0.0);
    }

    #[test]
    fn selection_and_deletion_are_id_based() {
        let mut editor = editor();
        editor.seleccionar_uno("cap-1");
        editor.alternar_seleccion("cap-2");
        assert_eq!(editor.borrar_seleccion(), 2);
        assert!(editor.proyecto().captions.is_empty());
    }

    #[test]
    fn adding_and_assigning_a_speaker_updates_the_project() {
        let mut editor = editor();
        editor.agregar(Caption {
            id: "cap-3".into(),
            inicio: 4.0,
            fin: 5.0,
            texto: "Tres".into(),
            hablante_id: None,
        });
        assert!(editor.asignar_hablante("cap-3", Some("speaker".into())));
        assert_eq!(
            editor.proyecto().captions[2].hablante_id.as_deref(),
            Some("speaker")
        );
    }

    fn texto_de(editor: &EstadoEditor, id: &str) -> String {
        editor
            .proyecto()
            .captions
            .iter()
            .find(|caption| caption.id == id)
            .map(|caption| caption.texto.clone())
            .expect("el caption existe")
    }

    #[test]
    fn deshacer_devuelve_el_texto_anterior() {
        let mut editor = editor();
        let original = texto_de(&editor, "cap-1");

        editor.registrar_historial();
        assert!(editor.editar_texto("cap-1", "cambiado"));
        assert_eq!(texto_de(&editor, "cap-1"), "cambiado");

        assert!(editor.deshacer());
        assert_eq!(texto_de(&editor, "cap-1"), original);
        assert!(editor.rehacer());
        assert_eq!(texto_de(&editor, "cap-1"), "cambiado");
    }

    #[test]
    fn deshacer_un_borrado_devuelve_los_captions_y_suelta_la_seleccion() {
        let mut editor = editor();
        editor.seleccionar_uno("cap-1");
        editor.alternar_seleccion("cap-2");

        editor.registrar_historial();
        assert_eq!(editor.borrar_seleccion(), 2);
        assert!(editor.proyecto().captions.is_empty());

        assert!(editor.deshacer());
        assert_eq!(editor.proyecto().captions.len(), 2);
        assert!(
            editor.seleccion.is_empty(),
            "la selección no se restaura: solo se poda"
        );
    }

    #[test]
    fn registrar_tras_deshacer_invalida_el_rehacer() {
        let mut editor = editor();
        editor.registrar_historial();
        editor.editar_texto("cap-1", "uno");
        assert!(editor.deshacer());
        assert!(editor.puede_rehacer());

        editor.registrar_historial();
        editor.editar_texto("cap-1", "otro");
        assert!(!editor.puede_rehacer());
        assert!(!editor.rehacer());
    }

    #[test]
    fn deshacer_sin_pasado_no_toca_nada() {
        let mut editor = editor();
        let antes = editor.proyecto().clone();
        assert!(!editor.deshacer());
        assert!(!editor.rehacer());
        assert_eq!(editor.proyecto(), &antes);
    }

    #[test]
    fn texto_de_encuentra_el_caption_o_no_devuelve_nada() {
        let editor = editor();
        assert_eq!(editor.texto_de("cap-1"), Some("Dos"));
        assert_eq!(editor.texto_de("cap-9"), None);
    }

    #[test]
    fn aplicar_el_mismo_texto_no_gasta_un_paso_de_historial() {
        let mut editor = editor();
        assert!(!editor.aplicar_texto("cap-1", "Dos"));
        assert!(!editor.puede_deshacer());
        assert!(!editor.aplicar_texto("cap-9", "fantasma"));
        assert!(!editor.puede_deshacer());
    }

    #[test]
    fn aplicar_texto_es_un_solo_paso_de_historial() {
        let mut editor = editor();
        assert!(editor.aplicar_texto("cap-1", "nuevo"));
        assert_eq!(editor.texto_de("cap-1"), Some("nuevo"));
        assert!(editor.puede_deshacer());

        assert!(editor.deshacer());
        assert_eq!(editor.texto_de("cap-1"), Some("Dos"));
        // Un solo paso: tras deshacer no queda nada más que deshacer.
        assert!(!editor.puede_deshacer());
    }
}
