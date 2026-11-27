//! Normalización del texto que va y viene del portapapeles del sistema.
//!
//! El acceso asíncrono al portapapeles vive en la capa de la app; acá solo la
//! parte pura: qué texto es usable y qué se copia de un caption.

/// Normaliza lo que devuelve el portapapeles del sistema operativo.
///
/// El SO puede devolver nada (portapapeles vacío), un objeto (si hay una imagen
/// copiada) o texto con CRLF. Lo que no sea texto usable vuelve como `""` para
/// que el llamador decida, en vez de propagar un valor ausente.
pub fn texto_del_portapapeles(valor: Option<&str>) -> String {
    let Some(valor) = valor else {
        return String::new();
    };
    // CRLF y `\r` sueltos → `\n`: un SRT pegado desde Windows no debe traer los
    // CR dentro del texto del subtítulo.
    let normalizado = valor.replace("\r\n", "\n").replace('\r', "");
    // Solo espacios: se trata como vacío (no crear un subtítulo en blanco).
    if normalizado.trim().is_empty() {
        String::new()
    } else {
        normalizado
    }
}

/// Texto que se manda al portapapeles al copiar un caption: su texto tal cual,
/// sin marcas de tiempo. Pegarlo en un editor tiene que dar el subtítulo, no un
/// bloque SRT.
pub fn caption_a_portapapeles(texto: &str) -> &str {
    texto
}

#[cfg(test)]
mod tests {
    use super::{caption_a_portapapeles, texto_del_portapapeles};

    // Casos portados de `__tests__/clipboard.test.ts`, uno por `it`.

    #[test]
    fn copy_returns_the_caption_text() {
        assert_eq!(caption_a_portapapeles("Hola, ¿qué tal?"), "Hola, ¿qué tal?");
    }

    #[test]
    fn copy_does_not_add_timecodes() {
        let texto = caption_a_portapapeles("Prueba");
        assert!(!texto.contains("00:00"));
        assert!(!texto.contains("-->"));
        assert!(!texto.contains("1.5"));
    }

    #[test]
    fn normalizes_crlf_to_lf() {
        assert_eq!(
            texto_del_portapapeles(Some("uno\r\ndos\r\ntres")),
            "uno\ndos\ntres"
        );
    }

    #[test]
    fn trims_loose_carriage_returns() {
        assert_eq!(texto_del_portapapeles(Some("uno\rdos")), "unodos");
    }

    #[test]
    fn returns_empty_for_null_or_absent_values() {
        assert_eq!(texto_del_portapapeles(None), "");
    }

    #[test]
    fn returns_empty_for_text_that_is_only_spaces() {
        assert_eq!(texto_del_portapapeles(Some("   \n\t  ")), "");
    }

    #[test]
    fn keeps_normal_text_as_is() {
        assert_eq!(texto_del_portapapeles(Some("hola")), "hola");
    }

    #[test]
    fn keeps_accents_and_dashes() {
        assert_eq!(texto_del_portapapeles(Some("Canción — sí")), "Canción — sí");
    }
}
