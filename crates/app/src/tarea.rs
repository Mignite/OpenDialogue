/// Efecto que el front-end debe ejecutar fuera del dibujado.
///
/// Sustituye al `iced::Task` que usaban `inicio` y `editor_app`: aquí solo se
/// describen efectos, nunca se ejecutan dentro de la vista.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tarea {
    Ninguna,
    DialogoVideo,
    DialogoProyecto,
    DialogoCarpeta,
    DialogoGuardar,
    EnfocarTexto,
    /// Los diálogos del editor que no caben en las cuatro formas de arranque.
    AbrirVideoEditor,
    CargarSrt,
    CargarAss,
    GuardarJson,
    GuardarAss,
    ImportarSrt,
    ImportarTxt,
}

impl Tarea {
    pub fn none() -> Self {
        Self::Ninguna
    }
}

#[cfg(test)]
mod tests {
    use super::Tarea;
    use crate::dialogos::RespuestaDialogo;
    use crate::inicio::{self, MensajeInicio};

    #[test]
    fn none_es_la_tarea_vacia() {
        assert_eq!(Tarea::none(), Tarea::Ninguna);
    }

    #[test]
    fn abrir_video_pide_dialogo() {
        let mut estado = inicio::nuevo(Vec::new(), None);
        let (tarea, salida) = inicio::update(&mut estado, MensajeInicio::AbrirVideo);
        assert_eq!(tarea, Tarea::DialogoVideo);
        assert!(salida.is_none());
    }

    #[test]
    fn dialogo_cancelado_no_abre_nada() {
        let mut estado = inicio::nuevo(Vec::new(), None);
        let (tarea, salida) = inicio::update(&mut estado, MensajeInicio::DialogoVideo(None));
        assert_eq!(tarea, Tarea::Ninguna);
        assert!(salida.is_none());
    }

    #[test]
    fn dos_respuestas_se_drenan_en_orden() {
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(RespuestaDialogo::Video(None)).unwrap();
        tx.send(RespuestaDialogo::Proyecto(None)).unwrap();
        assert!(matches!(rx.try_recv(), Ok(RespuestaDialogo::Video(None))));
        assert!(matches!(
            rx.try_recv(),
            Ok(RespuestaDialogo::Proyecto(None))
        ));
    }
}
