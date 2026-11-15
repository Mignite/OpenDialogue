use rod_core::{Caption, Hablante, Proyecto, clamp_caption_tiempos};

use crate::project_io::Documento;

#[derive(Debug, Clone, PartialEq)]
pub struct EstadoEditor {
    pub documento: Documento,
    pub seleccion: Vec<String>,
}

impl EstadoEditor {
    pub fn nuevo(documento: Documento) -> Self {
        Self {
            documento,
            seleccion: Vec::new(),
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
}
