//! Deshacer y rehacer.
//!
//! Portado de `useHistory` de la app Tauri, que guarda instantáneas de
//! `captions + hablantes` (no del video ni del playhead: una edición no los
//! toca). Las instantáneas son copias completas, como allí: con unos cientos de
//! subtítulos copiar el vector es más barato que mantener parches inversos, y
//! no hay forma de que un parche quede inconsistente.
//!
//! El límite es 50 pasos (`HISTORY_LIMIT` en el oráculo). Al llegar, se tira el
//! más viejo; no se avisa al usuario, igual que en el original.

use std::collections::VecDeque;

use crate::{Caption, Hablante, Proyecto};

/// Pasos que se conservan antes de tirar los más viejos.
pub const LIMITE_HISTORIAL: usize = 50;

/// Lo que una edición puede cambiar. `ruta_video` y `playhead` quedan fuera.
#[derive(Debug, Clone, PartialEq)]
pub struct Instantanea {
    pub captions: Vec<Caption>,
    pub hablantes: Vec<Hablante>,
}

impl Instantanea {
    pub fn de_proyecto(proyecto: &Proyecto) -> Self {
        Self {
            captions: proyecto.captions.clone(),
            hablantes: proyecto.hablantes.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Historial {
    pasado: VecDeque<Instantanea>,
    futuro: Vec<Instantanea>,
    limite: usize,
}

impl Default for Historial {
    fn default() -> Self {
        Self::nuevo(LIMITE_HISTORIAL)
    }
}

impl Historial {
    /// `limite` 0 no tiene sentido (no se podría deshacer nada) y haría que
    /// `registrar` tirara la instantánea recién metida; se trata como 1.
    pub fn nuevo(limite: usize) -> Self {
        Self {
            pasado: VecDeque::new(),
            futuro: Vec::new(),
            limite: limite.max(1),
        }
    }

    /// Guarda el estado **anterior** a una edición. Cualquier rehacer pendiente
    /// se descarta: el futuro que había ya no corresponde a este pasado.
    pub fn registrar(&mut self, actual: Instantanea) {
        self.pasado.push_back(actual);
        while self.pasado.len() > self.limite {
            self.pasado.pop_front();
        }
        self.futuro.clear();
    }

    /// Devuelve el estado a restaurar y guarda `actual` para poder rehacer.
    pub fn deshacer(&mut self, actual: Instantanea) -> Option<Instantanea> {
        let anterior = self.pasado.pop_back()?;
        self.futuro.push(actual);
        Some(anterior)
    }

    /// Simétrico de [`Historial::deshacer`].
    pub fn rehacer(&mut self, actual: Instantanea) -> Option<Instantanea> {
        let siguiente = self.futuro.pop()?;
        self.pasado.push_back(actual);
        Some(siguiente)
    }

    /// Al abrir o crear otro proyecto: sus instantáneas no sirven.
    pub fn limpiar(&mut self) {
        self.pasado.clear();
        self.futuro.clear();
    }

    pub fn puede_deshacer(&self) -> bool {
        !self.pasado.is_empty()
    }

    pub fn puede_rehacer(&self) -> bool {
        !self.futuro.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{Historial, Instantanea};
    use crate::Caption;

    fn estado(texto: &str) -> Instantanea {
        Instantanea {
            captions: vec![Caption {
                id: "cap-1".into(),
                inicio: 0.0,
                fin: 1.0,
                texto: texto.into(),
                hablante_id: None,
            }],
            hablantes: Vec::new(),
        }
    }

    fn texto(instantanea: &Instantanea) -> &str {
        &instantanea.captions[0].texto
    }

    #[test]
    fn deshacer_devuelve_el_estado_anterior() {
        let mut historial = Historial::default();
        historial.registrar(estado("uno"));
        let anterior = historial.deshacer(estado("dos")).expect("hay pasado");
        assert_eq!(texto(&anterior), "uno");
        assert!(!historial.puede_deshacer());
        assert!(historial.puede_rehacer());
    }

    #[test]
    fn rehacer_reaplica_lo_deshecho() {
        let mut historial = Historial::default();
        historial.registrar(estado("uno"));
        let anterior = historial.deshacer(estado("dos")).expect("hay pasado");
        let siguiente = historial.rehacer(anterior).expect("hay futuro");
        assert_eq!(texto(&siguiente), "dos");
        assert!(historial.puede_deshacer());
        assert!(!historial.puede_rehacer());
    }

    #[test]
    fn registrar_descarta_el_futuro() {
        let mut historial = Historial::default();
        historial.registrar(estado("uno"));
        let anterior = historial.deshacer(estado("dos")).expect("hay pasado");
        assert!(historial.puede_rehacer());
        // Edición nueva sobre el estado deshecho: el redo ya no aplica.
        historial.registrar(anterior);
        assert!(!historial.puede_rehacer());
    }

    #[test]
    fn sin_pasado_no_hay_deshacer() {
        let mut historial = Historial::default();
        assert!(historial.deshacer(estado("uno")).is_none());
    }

    #[test]
    fn sin_futuro_no_hay_rehacer() {
        let mut historial = Historial::default();
        assert!(historial.rehacer(estado("uno")).is_none());
    }

    #[test]
    fn el_limite_tira_lo_mas_viejo() {
        let mut historial = Historial::nuevo(3);
        for texto in ["uno", "dos", "tres", "cuatro", "cinco"] {
            historial.registrar(estado(texto));
        }

        let mut deshechos = Vec::new();
        let mut actual = estado("seis");
        while let Some(anterior) = historial.deshacer(actual.clone()) {
            actual = anterior;
            deshechos.push(texto(&actual).to_owned());
        }
        // Tres pasos: los tres últimos registrados, del más nuevo al más viejo.
        assert_eq!(deshechos, ["cinco", "cuatro", "tres"]);
    }

    #[test]
    fn limite_cero_se_trata_como_uno() {
        let mut historial = Historial::nuevo(0);
        historial.registrar(estado("uno"));
        assert!(historial.deshacer(estado("dos")).is_some());
    }

    #[test]
    fn limpiar_vacia_los_dos_lados() {
        let mut historial = Historial::default();
        historial.registrar(estado("uno"));
        historial.registrar(estado("dos"));
        let _ = historial.deshacer(estado("tres"));
        assert!(historial.puede_deshacer() && historial.puede_rehacer());

        historial.limpiar();
        assert!(!historial.puede_deshacer());
        assert!(!historial.puede_rehacer());
    }

    #[test]
    fn la_instantanea_copia_el_proyecto() {
        let mut proyecto = crate::Proyecto::nuevo("video.mp4");
        proyecto.captions.push(crate::Caption {
            id: "cap-1".into(),
            inicio: 0.0,
            fin: 1.0,
            texto: "hola".into(),
            hablante_id: Some("speaker".into()),
        });
        let instantanea = Instantanea::de_proyecto(&proyecto);
        proyecto.captions[0].texto = "cambiado".into();
        assert_eq!(instantanea.captions[0].texto, "hola");
    }
}
