//! Búsqueda y reemplazo sobre el texto de los subtítulos.
//!
//! Todo case-insensitive por defecto y el matching es por búsqueda literal, no
//! regex: así los caracteres especiales del término (`(`, `)`, `[`…) no hay que
//! escaparlos. Se trabaja sobre `char` y no sobre bytes porque un índice de byte
//! en medio de una `ñ` no es un límite válido y cortaría el texto.

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentoTexto {
    pub texto: String,
    pub coincide: bool,
}

pub fn normalizar_termino(termino: &str) -> &str {
    termino.trim()
}

fn minusculas(caracteres: &[char]) -> Vec<char> {
    caracteres
        .iter()
        .map(|c| c.to_lowercase().next().unwrap_or(*c))
        .collect()
}

/// Índice (en `char`) de la primera aparición del término, o `None`.
fn indice_de(texto: &[char], termino: &[char]) -> Option<usize> {
    if termino.is_empty() || termino.len() > texto.len() {
        return None;
    }
    (0..=texto.len() - termino.len())
        .find(|&inicio| texto[inicio..inicio + termino.len()] == *termino)
}

/// Prepara el texto y el término para comparar según `case_sensitive`.
fn preparar(texto: &str, termino: &str, case_sensitive: bool) -> (Vec<char>, Vec<char>) {
    let texto: Vec<char> = texto.chars().collect();
    let termino: Vec<char> = termino.chars().collect();
    if case_sensitive {
        (texto, termino)
    } else {
        (minusculas(&texto), minusculas(&termino))
    }
}

pub fn coincide(texto: &str, termino: &str, case_sensitive: bool) -> bool {
    let termino = normalizar_termino(termino);
    if termino.is_empty() {
        return false;
    }
    let (texto, termino) = preparar(texto, termino, case_sensitive);
    indice_de(&texto, &termino).is_some()
}

/// Parte el texto en tramos para resaltar con `<mark>`: los que coinciden quedan
/// con `coincide: true`. Coincidencias **no solapadas**, de izquierda a derecha
/// (`"aaa"` con `"aa"` da `["aa", "a"]`).
pub fn partir_resaltado(texto: &str, termino: &str, case_sensitive: bool) -> Vec<SegmentoTexto> {
    let original: Vec<char> = texto.chars().collect();
    let termino = normalizar_termino(termino);
    if termino.is_empty() {
        return vec![SegmentoTexto {
            texto: texto.to_string(),
            coincide: false,
        }];
    }

    let (comparado, buscado) = preparar(texto, termino, case_sensitive);
    let mut segmentos = Vec::new();
    let mut desde = 0;
    while desde <= comparado.len() {
        let Some(relativo) = indice_de(&comparado[desde..], &buscado) else {
            break;
        };
        let absoluto = desde + relativo;
        if absoluto > desde {
            segmentos.push(SegmentoTexto {
                texto: original[desde..absoluto].iter().collect(),
                coincide: false,
            });
        }
        let fin = absoluto + buscado.len();
        segmentos.push(SegmentoTexto {
            texto: original[absoluto..fin].iter().collect(),
            coincide: true,
        });
        desde = fin;
    }
    if desde < original.len() {
        segmentos.push(SegmentoTexto {
            texto: original[desde..].iter().collect(),
            coincide: false,
        });
    }
    if segmentos.is_empty() {
        segmentos.push(SegmentoTexto {
            texto: texto.to_string(),
            coincide: false,
        });
    }
    segmentos
}

/// Reemplaza **todas** las ocurrencias (mismo matching que el resaltado) sin
/// tocar el resto del texto. Con reemplazo vacío las borra.
pub fn reemplazar_todos(
    texto: &str,
    termino: &str,
    reemplazo: &str,
    case_sensitive: bool,
) -> String {
    let original: Vec<char> = texto.chars().collect();
    let termino = normalizar_termino(termino);
    if termino.is_empty() {
        return texto.to_string();
    }

    let (comparado, buscado) = preparar(texto, termino, case_sensitive);
    let mut resultado = String::new();
    let mut desde = 0;
    while desde <= comparado.len() {
        let Some(relativo) = indice_de(&comparado[desde..], &buscado) else {
            break;
        };
        let absoluto = desde + relativo;
        resultado.extend(&original[desde..absoluto]);
        resultado.push_str(reemplazo);
        desde = absoluto + buscado.len();
    }
    resultado.extend(&original[desde.min(original.len())..]);
    resultado
}

#[cfg(test)]
mod tests {
    use super::{SegmentoTexto, coincide, partir_resaltado, reemplazar_todos};

    // Casos portados de `__tests__/busqueda.test.ts`, uno por `it`.

    fn seg(texto: &str, coincide: bool) -> SegmentoTexto {
        SegmentoTexto {
            texto: texto.into(),
            coincide,
        }
    }

    #[test]
    fn an_empty_term_or_only_spaces_never_matches() {
        assert!(!coincide("Hola", "", false));
        assert!(!coincide("Hola", "   ", false));
    }

    #[test]
    fn case_insensitive_by_default() {
        assert!(coincide("Hola Mundo", "mundo", false));
        assert!(coincide("Hola Mundo", "MUNDO", false));
    }

    #[test]
    fn case_sensitive_respects_capitals() {
        assert!(!coincide("Hola Mundo", "mundo", true));
        assert!(coincide("Hola Mundo", "Mundo", true));
    }

    #[test]
    fn the_term_is_trimmed_but_the_text_is_not() {
        assert!(coincide("Hola mundo", " mundo ", false));
    }

    #[test]
    fn highlight_without_a_term_returns_a_single_segment() {
        assert_eq!(
            partir_resaltado("Hola", "", false),
            vec![seg("Hola", false)]
        );
    }

    #[test]
    fn one_match_in_the_middle() {
        assert_eq!(
            partir_resaltado("Hola mundo", "mundo", false),
            vec![seg("Hola ", false), seg("mundo", true)]
        );
    }

    #[test]
    fn several_non_overlapping_matches() {
        assert_eq!(
            partir_resaltado("siso", "s", false),
            vec![
                seg("s", true),
                seg("i", false),
                seg("s", true),
                seg("o", false),
            ]
        );
    }

    #[test]
    fn overlapping_matches_go_left_to_right_without_overlapping() {
        assert_eq!(
            partir_resaltado("aaa", "aa", false),
            vec![seg("aa", true), seg("a", false)]
        );
    }

    #[test]
    fn match_at_the_start_and_at_the_end() {
        assert_eq!(
            partir_resaltado("abca", "a", false),
            vec![seg("a", true), seg("bc", false), seg("a", true)]
        );
    }

    #[test]
    fn the_whole_text_matches() {
        assert_eq!(
            partir_resaltado("Hola", "hola", false),
            vec![seg("Hola", true)]
        );
    }

    #[test]
    fn without_a_match_returns_the_text_intact() {
        assert_eq!(
            partir_resaltado("Hola", "xyz", false),
            vec![seg("Hola", false)]
        );
    }

    #[test]
    fn special_characters_in_the_term_do_not_explode() {
        assert_eq!(
            partir_resaltado("a(b)c", "(b)", false),
            vec![seg("a", false), seg("(b)", true), seg("c", false)]
        );
    }

    #[test]
    fn replaces_every_occurrence_case_insensitively() {
        assert_eq!(
            reemplazar_todos("Ana y ana y ANA", "ana", "Eva", false),
            "Eva y Eva y Eva"
        );
    }

    #[test]
    fn an_empty_replacement_deletes_the_occurrences() {
        assert_eq!(reemplazar_todos("a-b-c", "-", "", false), "abc");
    }

    #[test]
    fn keeps_the_rest_of_the_text_exact() {
        assert_eq!(
            reemplazar_todos("Hola Mundo", "mundo", "mundo cruel", false),
            "Hola mundo cruel"
        );
    }

    #[test]
    fn replacement_case_sensitive_respects_capitals() {
        assert_eq!(
            reemplazar_todos("Ana y ana", "Ana", "Eva", true),
            "Eva y ana"
        );
    }

    #[test]
    fn without_matches_the_text_is_returned_unchanged() {
        assert_eq!(reemplazar_todos("Hola", "xyz", "abc", false), "Hola");
    }

    #[test]
    fn the_replacement_may_contain_the_term_without_an_infinite_loop() {
        assert_eq!(reemplazar_todos("a b a", "a", "aa", false), "aa b aa");
    }
}
