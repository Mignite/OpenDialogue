//! Import de una transcripción de auto-subs: turnos `Speaker N:` de un `.txt`
//! y asignación de esos hablantes a los cues por **texto**, no por tiempo (el
//! TXT solo trae la hora de inicio de cada turno).

use crate::{Caption, Hablante};

#[derive(Debug, Clone, PartialEq)]
pub struct TurnoAutosubs {
    pub speaker: String,
    pub texto: String,
}

struct Encabezado {
    inicio: usize,
    longitud: usize,
    speaker: String,
}

/// Busca el próximo `Speaker N:\n` que empiece una línea a partir de `desde`.
fn buscar_encabezado(texto: &str, desde: usize) -> Option<Encabezado> {
    let bytes = texto.as_bytes();
    for (relativo, _) in texto[desde..].char_indices() {
        let inicio = desde + relativo;
        if inicio != 0 && bytes[inicio - 1] != b'\n' {
            continue;
        }
        let Some(resto) = texto[inicio..].strip_prefix("Speaker ") else {
            continue;
        };
        let digitos = resto.bytes().take_while(u8::is_ascii_digit).count();
        if digitos == 0 {
            continue;
        }
        let speaker = &resto[..digitos];
        let Some(despues) = resto[digitos..].strip_prefix(":\n") else {
            continue;
        };
        let longitud = texto[inicio..].len() - despues.len();
        return Some(Encabezado {
            inicio,
            longitud,
            speaker: speaker.to_string(),
        });
    }
    None
}

/// Turnos `Speaker N:` del TXT de auto-subs. Un texto de cada turno va hasta el
/// siguiente encabezado, sin el salto que los separa.
pub fn parse_autosubs_txt(contenido: &str) -> Vec<TurnoAutosubs> {
    // El TXT puede venir con CRLF de Windows: el formato pide `\n`.
    let limpio = contenido.replace('\r', "");
    let mut turnos = Vec::new();
    let mut desde = 0;
    while let Some(encabezado) = buscar_encabezado(&limpio, desde) {
        let inicio_texto = encabezado.inicio + encabezado.longitud;
        let fin = buscar_encabezado(&limpio, inicio_texto)
            .map_or(limpio.len(), |siguiente| siguiente.inicio);
        turnos.push(TurnoAutosubs {
            speaker: encabezado.speaker,
            texto: limpio[inicio_texto..fin].trim().to_string(),
        });
        desde = inicio_texto;
    }
    turnos
}

/// Quita las tildes de las letras latinas que usa el español (y las demás
/// romances). Es el equivalente de `normalize("NFD")` + borrar los diacríticos
/// sin traer una tabla Unicode entera.
fn sin_tildes(caracter: char) -> char {
    match caracter {
        'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' | 'ā' | 'ă' | 'ą' => 'a',
        'é' | 'è' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => 'e',
        'í' | 'ì' | 'î' | 'ï' | 'ī' | 'ĭ' | 'į' => 'i',
        'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ø' | 'ō' | 'ŏ' | 'ő' => 'o',
        'ú' | 'ù' | 'û' | 'ü' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => 'u',
        'ý' | 'ÿ' | 'ŷ' => 'y',
        'ñ' | 'ń' | 'ņ' | 'ň' => 'n',
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => 'c',
        'š' | 'ś' | 'ŝ' | 'ş' => 's',
        'ž' | 'ź' | 'ż' => 'z',
        'ĝ' | 'ğ' | 'ġ' | 'ģ' => 'g',
        'ĥ' | 'ħ' => 'h',
        'ĵ' => 'j',
        'ĺ' | 'ļ' | 'ľ' | 'ł' => 'l',
        'ŕ' | 'ŗ' | 'ř' => 'r',
        'ţ' | 'ť' | 'ŧ' => 't',
        'ď' | 'đ' => 'd',
        'þ' => 't',
        'æ' => 'a',
        'œ' => 'o',
        otro => otro,
    }
}

/// Minúsculas, sin tildes ni puntuación: el SRT y el TXT puntúan distinto el
/// mismo diálogo ("¡Adiós, muy bueno!" vs "Adiós; muy bueno").
pub fn normalizar_texto(texto: &str) -> String {
    let plano: String = texto
        .to_lowercase()
        .chars()
        .map(sin_tildes)
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || c.is_whitespace())
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect();

    plano.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// El cue tiene que aparecer como **palabras completas** dentro del turno: con
/// una búsqueda a secas, `"si"` matchea dentro de `"así que vamos"`.
fn contiene_palabras(turno: &str, cue: &str) -> bool {
    format!(" {turno} ").contains(&format!(" {cue} "))
}

/// Cada cue hereda el speaker del turno cuyo texto lo contiene. La búsqueda es
/// **secuencial** desde el último match: ambos archivos salen del mismo
/// transcript en orden, así que un cue no pertenece a un turno anterior al ya
/// asignado. `+1` en la posición rompía los turnos con frases repetidas.
pub fn asignar_hablantes_por_texto(
    cues: &[Caption],
    turnos: &[TurnoAutosubs],
) -> Vec<Option<String>> {
    let normalizados: Vec<String> = turnos
        .iter()
        .map(|turno| normalizar_texto(&turno.texto))
        .collect();
    let mut posicion = 0;

    cues.iter()
        .map(|cue| {
            let buscado = normalizar_texto(&cue.texto);
            if buscado.is_empty() {
                return None;
            }

            let desde_posicion = (posicion..normalizados.len())
                .find(|&indice| contiene_palabras(&normalizados[indice], &buscado));
            let encontrado = desde_posicion.or_else(|| {
                (0..normalizados.len())
                    .find(|&indice| contiene_palabras(&normalizados[indice], &buscado))
            })?;
            posicion = encontrado;
            Some(turnos[encontrado].speaker.clone())
        })
        .collect()
}

/// Hablantes con la convención de la app: nombre `Hablante N`, tecla dígito y
/// color blanco (el color efectivo sale del preset `.ass`). El orden es
/// **numérico**: el llamador suele pasar un set ordenado lexicográficamente,
/// donde `"10"` va antes que `"2"`.
pub fn hablantes_desde_nombres(numeros: &[String]) -> Vec<Hablante> {
    let marca = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duracion| duracion.as_millis());

    let mut ordenados = numeros.to_vec();
    ordenados.sort_by(|a, b| match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(a), Ok(b)) => a.total_cmp(&b),
        _ => a.cmp(b),
    });

    ordenados
        .iter()
        .enumerate()
        .map(|(indice, numero)| Hablante {
            id: format!("sp-autosubs-{marca}-{indice}"),
            nombre: format!("Hablante {numero}"),
            tecla: ((indice % 9) + 1).to_string(),
            color: "#FFFFFF".into(),
            preset_id: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        TurnoAutosubs, asignar_hablantes_por_texto, hablantes_desde_nombres, normalizar_texto,
        parse_autosubs_txt,
    };
    use crate::Caption;

    // Casos portados de `__tests__/autosubs.test.ts`, uno por `it`.

    fn cue(inicio: f64, fin: f64, texto: &str) -> Caption {
        Caption {
            id: "x".into(),
            inicio,
            fin,
            texto: texto.into(),
            hablante_id: None,
        }
    }

    fn turno(speaker: &str, texto: &str) -> TurnoAutosubs {
        TurnoAutosubs {
            speaker: speaker.into(),
            texto: texto.into(),
        }
    }

    fn numeros(lista: &[&str]) -> Vec<String> {
        lista.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn extracts_speaker_turns_with_their_text() {
        let turnos = parse_autosubs_txt("Speaker 1:\nHola mundo.\n\nSpeaker 2:\nAdiós.\n");
        assert_eq!(
            turnos,
            vec![turno("1", "Hola mundo."), turno("2", "Adiós.")]
        );
    }

    #[test]
    fn empty_text_yields_no_turns() {
        assert!(parse_autosubs_txt("").is_empty());
    }

    #[test]
    fn accepts_windows_crlf_text() {
        // Regresión: exigir `\n` daba 0 turnos con un TXT de `\r\n`.
        let turnos =
            parse_autosubs_txt("Speaker 1:\r\nHola mundo.\r\n\r\nSpeaker 2:\r\nAdiós.\r\n");
        assert_eq!(
            turnos,
            vec![turno("1", "Hola mundo."), turno("2", "Adiós.")]
        );
    }

    #[test]
    fn normalizes_case_accents_punctuation_and_whitespace() {
        assert_eq!(normalizar_texto("¡Adiós,  MUY bueno!"), "adios muy bueno");
    }

    #[test]
    fn assigns_by_sequential_turn_order() {
        let cues = [
            cue(0.0, 1.0, "Hola"),
            cue(1.0, 2.0, "Adiós"),
            cue(2.0, 3.0, "Hola otra vez"),
        ];
        let turnos = [turno("1", "Hola"), turno("2", "Adiós. Hola otra vez")];
        assert_eq!(
            asignar_hablantes_por_texto(&cues, &turnos),
            vec![
                Some("1".to_string()),
                Some("2".to_string()),
                Some("2".to_string())
            ]
        );
    }

    #[test]
    fn tolerates_different_accents_and_punctuation_between_srt_and_txt() {
        let cues = [cue(0.0, 1.0, "¡Adios, muy bueno!")];
        let turnos = [turno("3", "Adiós; muy bueno")];
        assert_eq!(
            asignar_hablantes_por_texto(&cues, &turnos),
            vec![Some("3".to_string())]
        );
    }

    #[test]
    fn a_cue_without_a_match_stays_null() {
        let cues = [cue(0.0, 1.0, "zzz inexistente")];
        let turnos = [turno("1", "Hola")];
        assert_eq!(asignar_hablantes_por_texto(&cues, &turnos), vec![None]);
    }

    #[test]
    fn without_turns_everything_stays_null() {
        assert_eq!(
            asignar_hablantes_por_texto(&[cue(0.0, 1.0, "Hola")], &[]),
            vec![None]
        );
    }

    #[test]
    fn two_cues_of_the_same_turn_do_not_jump_to_the_next_repeated_turn() {
        // Regresión: con `pos = hit + 1` el segundo cue ya no se buscaba en su
        // propio turno y caía en el turno repetido siguiente.
        let cues = [cue(0.0, 1.0, "hola"), cue(1.0, 2.0, "mundo")];
        let turnos = [turno("1", "hola mundo"), turno("2", "mundo cruel")];
        assert_eq!(
            asignar_hablantes_por_texto(&cues, &turnos),
            vec![Some("1".to_string()), Some("1".to_string())]
        );
    }

    #[test]
    fn does_not_match_substrings_inside_another_word() {
        // Regresión: "Sí." normaliza a "si" y con `includes` matcheaba dentro
        // de "así".
        let cues = [cue(0.0, 1.0, "Sí.")];
        let turnos = [turno("1", "Así que vamos")];
        assert_eq!(asignar_hablantes_por_texto(&cues, &turnos), vec![None]);
    }

    #[test]
    fn speakers_follow_the_app_convention_name_and_digit_key() {
        let hablantes = hablantes_desde_nombres(&numeros(&["1", "2"]));
        let nombres: Vec<&str> = hablantes.iter().map(|h| h.nombre.as_str()).collect();
        assert_eq!(nombres, vec!["Hablante 1", "Hablante 2"]);
        let teclas: Vec<&str> = hablantes.iter().map(|h| h.tecla.as_str()).collect();
        assert_eq!(teclas, vec!["1", "2"]);
        assert!(hablantes[0].id.starts_with("sp-autosubs-"));
    }

    #[test]
    fn a_new_speaker_has_no_preset_and_its_saved_color_is_white() {
        let hablantes = hablantes_desde_nombres(&numeros(&["1", "2"]));
        assert_eq!(hablantes[0].color, "#FFFFFF");
        assert_eq!(hablantes[0].preset_id, None);
    }

    #[test]
    fn sorts_numerically_even_when_they_arrive_out_of_order() {
        // "2" < "10": ordenar como texto pondría 10 antes que 2.
        let hablantes = hablantes_desde_nombres(&numeros(&["10", "2", "1"]));
        let nombres: Vec<&str> = hablantes.iter().map(|h| h.nombre.as_str()).collect();
        assert_eq!(nombres, vec!["Hablante 1", "Hablante 2", "Hablante 10"]);
        let teclas: Vec<&str> = hablantes.iter().map(|h| h.tecla.as_str()).collect();
        assert_eq!(teclas, vec!["1", "2", "3"]);
    }
}
