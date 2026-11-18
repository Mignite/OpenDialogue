use rod_core::{Caption, Hablante, Historial, Instantanea, Proyecto, clamp_caption_tiempos};

use crate::project_io::Documento;

/// El oráculo tope a 9 hablantes porque `9` es la última tecla de acceso
/// directo (`useHablantes.ts`).
pub const MAX_HABLANTES: usize = 9;

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

    /// Deja la selección exactamente en `ids`, descartando los que no existen
    /// (un arrastre o una marquesina pueden citar captions ya borrados).
    pub fn reemplazar_seleccion(&mut self, ids: Vec<String>) {
        self.seleccion = ids
            .into_iter()
            .filter(|id| {
                self.documento
                    .proyecto
                    .captions
                    .iter()
                    .any(|caption| &caption.id == id)
            })
            .collect();
    }

    /// El proyecto mutable, para los arrastres del timeline: mueven muchos
    /// captions por frame y la app ya decidió cuándo hay paso de historial.
    pub fn proyecto_mut(&mut self) -> &mut Proyecto {
        &mut self.documento.proyecto
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

    pub fn hablante(&self, id: &str) -> Option<&Hablante> {
        self.proyecto()
            .hablantes
            .iter()
            .find(|hablante| hablante.id == id)
    }

    /// Añade un hablante con la primera tecla libre. `None` si ya hay 9: el
    /// oráculo ignora el clic en ese caso, aquí además se apaga el botón.
    ///
    /// El id es `sp-N` secuencial en vez del `sp-<Date.now()>` del oráculo: con
    /// proyectos guardados en JSON (slice 7) un id reproducible hace que el
    /// archivo no cambie de una sesión a otra sin que el usuario toque nada.
    pub fn agregar_hablante(&mut self) -> Option<String> {
        if self.proyecto().hablantes.len() >= MAX_HABLANTES {
            return None;
        }
        let tecla = self.tecla_libre()?;
        let id = self.id_hablante_libre();
        self.registrar_historial();
        self.documento.proyecto.hablantes.push(Hablante {
            id: id.clone(),
            nombre: String::new(),
            tecla,
            color: "#FFFFFF".into(),
            preset_id: None,
        });
        Some(id)
    }

    /// Renombrar no registra historial: la app decide la granularidad (una
    /// sesión de tecleo = un paso), como con el texto de un caption.
    /// `false` si el hablante no existe o el nombre no cambió.
    pub fn editar_nombre_hablante(&mut self, id: &str, nombre: impl Into<String>) -> bool {
        let nombre = nombre.into();
        let Some(hablante) = self
            .documento
            .proyecto
            .hablantes
            .iter_mut()
            .find(|hablante| hablante.id == id)
        else {
            return false;
        };
        if hablante.nombre == nombre {
            return false;
        }
        hablante.nombre = nombre;
        true
    }

    /// Misma regla que [`EstadoEditor::editar_nombre_hablante`]. La tecla es un
    /// solo carácter en el oráculo (`maxLength={1}`); aquí se recorta con
    /// [`normalizar_tecla`].
    pub fn editar_tecla_hablante(&mut self, id: &str, tecla: &str) -> bool {
        let tecla = normalizar_tecla(tecla);
        let Some(hablante) = self
            .documento
            .proyecto
            .hablantes
            .iter_mut()
            .find(|hablante| hablante.id == id)
        else {
            return false;
        };
        if hablante.tecla == tecla {
            return false;
        }
        hablante.tecla = tecla;
        true
    }

    /// Borra el hablante y suelta sus captions (`hablante_id = None`), como el
    /// oráculo. Un solo paso de historial: sin eso, deshacer devolvería el
    /// hablante pero los captions quedarían huérfanos.
    pub fn borrar_hablante(&mut self, id: &str) -> bool {
        if self.hablante(id).is_none() {
            return false;
        }
        self.registrar_historial();
        self.documento.proyecto.hablantes.retain(|h| h.id != id);
        for caption in &mut self.documento.proyecto.captions {
            if caption.hablante_id.as_deref() == Some(id) {
                caption.hablante_id = None;
            }
        }
        true
    }

    /// Asigna (o quita, con `None`) el hablante a todos los captions
    /// seleccionados. Devuelve cuántos cambiaron; sin cambios no hay paso de
    /// historial.
    pub fn asignar_a_seleccion(&mut self, hablante_id: Option<String>) -> usize {
        let cambian = self
            .seleccion
            .iter()
            .filter(|id| {
                self.proyecto()
                    .captions
                    .iter()
                    .find(|caption| &caption.id == *id)
                    .is_some_and(|caption| caption.hablante_id != hablante_id)
            })
            .count();
        if cambian == 0 {
            return 0;
        }
        self.registrar_historial();
        for id in self.seleccion.clone() {
            self.asignar_hablante(&id, hablante_id.clone());
        }
        cambian
    }

    fn tecla_libre(&self) -> Option<String> {
        (1..=MAX_HABLANTES).map(|n| n.to_string()).find(|tecla| {
            !self
                .proyecto()
                .hablantes
                .iter()
                .any(|hablante| &hablante.tecla == tecla)
        })
    }

    fn id_hablante_libre(&self) -> String {
        let mut n = 1;
        loop {
            let id = format!("sp-{n}");
            if self.hablante(&id).is_none() {
                return id;
            }
            n += 1;
        }
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

    /// Registra un estado capturado **antes** de la edición. Lo usa el
    /// arrastre del timeline: la instantánea se toma al pulsar, pero el paso
    /// solo entra cuando el primer frame cambia algo de verdad.
    pub fn registrar_instantanea(&mut self, instantanea: Instantanea) {
        self.historial.registrar(instantanea);
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

/// La tecla de un hablante es un solo carácter (`maxLength={1}` en el oráculo).
pub fn normalizar_tecla(texto: &str) -> String {
    texto.chars().next().map(String::from).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{EstadoEditor, MAX_HABLANTES};
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

    #[test]
    fn agregar_hablante_toma_la_primera_tecla_libre() {
        let mut editor = editor();
        let primero = editor.agregar_hablante().expect("hay espacio");
        let segundo = editor.agregar_hablante().expect("hay espacio");
        assert_ne!(primero, segundo);
        assert_eq!(editor.proyecto().hablantes[0].tecla, "1");
        assert_eq!(editor.proyecto().hablantes[1].tecla, "2");
        assert!(editor.puede_deshacer(), "agregar sí es un paso");
    }

    #[test]
    fn con_nueve_hablantes_no_se_puede_agregar() {
        let mut editor = editor();
        for _ in 0..MAX_HABLANTES {
            assert!(editor.agregar_hablante().is_some());
        }
        assert!(editor.agregar_hablante().is_none());
        assert_eq!(editor.proyecto().hablantes.len(), MAX_HABLANTES);
    }

    #[test]
    fn renombrar_y_cambiar_tecla_no_registran_por_si_solos() {
        let mut editor = editor();
        let id = editor.agregar_hablante().expect("hay espacio");
        editor.registrar_historial(); // lo que hace la app al empezar a teclear

        assert!(editor.editar_nombre_hablante(&id, "A"));
        assert!(editor.editar_nombre_hablante(&id, "Ana"));
        assert_eq!(editor.hablante(&id).map(|h| h.nombre.as_str()), Some("Ana"));

        assert!(editor.deshacer());
        assert_eq!(
            editor.hablante(&id).map(|h| h.nombre.as_str()),
            Some(""),
            "una sesión de tecleo es un solo paso, no uno por letra"
        );
    }

    #[test]
    fn la_tecla_se_recorta_a_un_caracter() {
        let mut editor = editor();
        let id = editor.agregar_hablante().expect("hay espacio");
        assert!(editor.editar_tecla_hablante(&id, "42"));
        assert_eq!(editor.hablante(&id).map(|h| h.tecla.as_str()), Some("4"));
        assert!(!editor.editar_tecla_hablante(&id, "4"), "sin cambio");
    }

    #[test]
    fn borrar_hablante_suelta_sus_captions_y_deshacer_lo_deshace() {
        let mut editor = editor();
        let id = editor.agregar_hablante().expect("hay espacio");
        assert!(editor.asignar_hablante("cap-1", Some(id.clone())));

        assert!(editor.borrar_hablante(&id));
        assert!(editor.proyecto().hablantes.is_empty());
        assert!(
            editor
                .proyecto()
                .captions
                .iter()
                .all(|caption| caption.hablante_id.is_none())
        );

        assert!(editor.deshacer());
        assert_eq!(editor.proyecto().hablantes.len(), 1);
        assert_eq!(
            editor
                .proyecto()
                .captions
                .iter()
                .filter(|caption| caption.hablante_id.as_deref() == Some(id.as_str()))
                .count(),
            1,
            "el hablante vuelve con el caption que tenía"
        );
    }

    #[test]
    fn asignar_a_la_seleccion_es_un_solo_paso() {
        let mut editor = editor();
        let id = editor.agregar_hablante().expect("hay espacio");
        editor.seleccionar_uno("cap-1");
        editor.alternar_seleccion("cap-2");

        assert_eq!(editor.asignar_a_seleccion(Some(id.clone())), 2);
        assert_eq!(
            editor.asignar_a_seleccion(Some(id.clone())),
            0,
            "sin cambios no hay paso de historial"
        );
        assert!(editor.deshacer());
        assert!(
            editor
                .proyecto()
                .captions
                .iter()
                .all(|caption| caption.hablante_id.is_none())
        );
    }

    #[test]
    fn quitar_el_hablante_de_la_seleccion_tambien_es_una_edicion() {
        let mut editor = editor();
        let id = editor.agregar_hablante().expect("hay espacio");
        editor.seleccionar_uno("cap-1");
        assert_eq!(editor.asignar_a_seleccion(Some(id.clone())), 1);

        assert_eq!(editor.asignar_a_seleccion(None), 1);
        assert!(
            editor.proyecto().captions[0].hablante_id.is_none()
                || editor.proyecto().captions[1].hablante_id.is_none()
        );

        assert!(editor.deshacer());
        assert!(
            editor
                .proyecto()
                .captions
                .iter()
                .any(|caption| caption.hablante_id.is_some())
        );
    }
}
