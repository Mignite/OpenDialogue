//! Selección sobre la línea de tiempo: fila (carril) de un caption, corrimiento
//! de carriles en el arrastre de bloque, marquesina y agrupación por carril.

use std::collections::{BTreeMap, HashMap};

use crate::{Caption, Hablante};

/// Fila del carril de un caption: 0 = sin asignar (`—`), luego 1 por hablante.
pub fn caption_row_index(hablante_id: Option<&str>, hablantes: &[Hablante]) -> usize {
    let Some(hablante_id) = hablante_id else {
        return 0;
    };
    hablantes
        .iter()
        .position(|hablante| hablante.id == hablante_id)
        .map_or(0, |indice| indice + 1)
}

/// Corrimiento relativo de carriles para el arrastre de bloque: cada clip
/// conserva su offset respecto al lead y el destino se acota a `[0, max_fila]`
/// (0 = sin asignar). Solo cede la relación en los bordes, donde el clip que se
/// saldría se queda.
pub fn filas_destino_relativas(
    filas_origen: &BTreeMap<String, i32>,
    shift: i32,
    max_fila: i32,
) -> BTreeMap<String, i32> {
    filas_origen
        .iter()
        .map(|(id, fila)| (id.clone(), (fila + shift).clamp(0, max_fila)))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectMarquee {
    pub t1: f64,
    pub t2: f64,
    pub fila1: i32,
    pub fila2: i32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AgrupacionCarriles {
    /// Captions del carril `—` (sin hablante asignado).
    pub sin_asignar: Vec<Caption>,
    /// Por id de hablante. Todo hablante de la lista tiene entrada (puede ser
    /// vacía), para que el render no tenga que distinguir los dos casos.
    pub por_hablante: HashMap<String, Vec<Caption>>,
}

/// Reparte los captions en sus carriles en **una pasada** (O(n)), en vez de un
/// filtrado por hablante (O(n × (S+1))). Importa porque esto se recalcula en
/// cada render y el playhead fuerza renders a 60 Hz.
///
/// Un caption con `hablante_id` que no está en la lista no aparece en ningún
/// carril: no es lo mismo "sin asignar" que "asignado a alguien que ya no
/// existe". No se pierde (sigue en el array de captions), pero queda fuera del
/// timeline hasta que se reasigne.
pub fn agrupar_captions_por_carril(
    captions: &[Caption],
    hablantes: &[Hablante],
) -> AgrupacionCarriles {
    let mut por_hablante: HashMap<String, Vec<Caption>> = hablantes
        .iter()
        .map(|hablante| (hablante.id.clone(), Vec::new()))
        .collect();
    let mut sin_asignar = Vec::new();

    for caption in captions {
        let Some(hablante_id) = caption.hablante_id.as_deref() else {
            sin_asignar.push(caption.clone());
            continue;
        };
        // Id desconocido: no es un carril válido, se ignora.
        if let Some(destino) = por_hablante.get_mut(hablante_id) {
            destino.push(caption.clone());
        }
    }

    AgrupacionCarriles {
        sin_asignar,
        por_hablante,
    }
}

/// Captions cuyo rango `[inicio, fin]` intersecta `[t1, t2]` y cuyo carril cae
/// en `[fila1, fila2]`. Los límites se normalizan, así que un rectángulo
/// dibujado de abajo-derecha a arriba-izquierda también selecciona.
pub fn filtrar_por_marquee(
    captions: &[Caption],
    hablantes: &[Hablante],
    rect: RectMarquee,
) -> Vec<String> {
    let t1 = rect.t1.min(rect.t2);
    let t2 = rect.t1.max(rect.t2);
    let fila1 = rect.fila1.min(rect.fila2);
    let fila2 = rect.fila1.max(rect.fila2);

    captions
        .iter()
        .filter(|caption| {
            let fila = caption_row_index(caption.hablante_id.as_deref(), hablantes) as i32;
            if fila < fila1 || fila > fila2 {
                return false;
            }
            // Solape, no contención: un caption enorme que cubre el rectángulo
            // también entra, y uno que asoma por un borde también.
            !(caption.inicio > t2 || caption.fin < t1)
        })
        .map(|caption| caption.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        RectMarquee, agrupar_captions_por_carril, caption_row_index, filas_destino_relativas,
        filtrar_por_marquee,
    };
    use crate::{Caption, Hablante};

    // Casos portados de `__tests__/selection.test.ts`, uno por `it`.

    fn caption(id: &str, inicio: f64, fin: f64, hablante_id: Option<&str>) -> Caption {
        Caption {
            id: id.into(),
            inicio,
            fin,
            texto: "test".into(),
            hablante_id: hablante_id.map(str::to_string),
        }
    }

    fn hablantes() -> Vec<Hablante> {
        vec![
            Hablante {
                id: "s1".into(),
                nombre: "Alice".into(),
                tecla: "a".into(),
                color: "#ff0000".into(),
                preset_id: None,
            },
            Hablante {
                id: "s2".into(),
                nombre: "Bob".into(),
                tecla: "b".into(),
                color: "#00ff00".into(),
                preset_id: None,
            },
        ]
    }

    fn filas(pares: &[(&str, i32)]) -> BTreeMap<String, i32> {
        pares
            .iter()
            .map(|(id, fila)| (id.to_string(), *fila))
            .collect()
    }

    fn ids(captions: &[Caption]) -> Vec<String> {
        captions.iter().map(|c| c.id.clone()).collect()
    }

    #[test]
    fn maps_unassigned_captions_to_row_zero() {
        assert_eq!(caption_row_index(None, &hablantes()), 0);
    }

    #[test]
    fn maps_each_speaker_to_its_position_plus_one() {
        let hablantes = hablantes();
        assert_eq!(caption_row_index(Some("s1"), &hablantes), 1);
        assert_eq!(caption_row_index(Some("s2"), &hablantes), 2);
    }

    #[test]
    fn falls_back_to_row_zero_for_an_unknown_speaker() {
        assert_eq!(caption_row_index(Some("nope"), &hablantes()), 0);
    }

    #[test]
    fn marquee_returns_empty_for_empty_captions() {
        let rect = RectMarquee {
            t1: 0.0,
            t2: 10.0,
            fila1: 0,
            fila2: 2,
        };
        assert!(filtrar_por_marquee(&[], &hablantes(), rect).is_empty());
    }

    #[test]
    fn marquee_selects_captions_intersecting_the_time_range() {
        let captions = [
            caption("c1", 0.0, 5.0, Some("s1")),
            caption("c2", 6.0, 10.0, Some("s1")),
            caption("c3", 12.0, 15.0, Some("s1")),
        ];
        let rect = RectMarquee {
            t1: 4.0,
            t2: 13.0,
            fila1: 1,
            fila2: 1,
        };
        assert_eq!(
            filtrar_por_marquee(&captions, &hablantes(), rect),
            vec!["c1", "c2", "c3"]
        );
    }

    #[test]
    fn marquee_filters_by_speaker_row() {
        let captions = [
            caption("c1", 0.0, 5.0, Some("s1")),
            caption("c2", 1.0, 3.0, Some("s2")),
            caption("c3", 2.0, 4.0, None),
        ];
        let rect = RectMarquee {
            t1: 0.0,
            t2: 10.0,
            fila1: 2,
            fila2: 2,
        };
        assert_eq!(
            filtrar_por_marquee(&captions, &hablantes(), rect),
            vec!["c2"]
        );
    }

    #[test]
    fn marquee_normalizes_inverted_rectangle_bounds() {
        let captions = [caption("c1", 0.0, 5.0, Some("s1"))];
        let rect = RectMarquee {
            t1: 5.0,
            t2: 0.0,
            fila1: 1,
            fila2: 0,
        };
        assert_eq!(
            filtrar_por_marquee(&captions, &hablantes(), rect),
            vec!["c1"]
        );
    }

    #[test]
    fn marquee_requires_overlap_not_containment() {
        let captions = [
            caption("c1", 0.0, 1.0, Some("s1")),
            caption("c2", 10.0, 12.0, Some("s1")),
        ];
        let rect = RectMarquee {
            t1: 2.0,
            t2: 9.0,
            fila1: 1,
            fila2: 1,
        };
        assert!(filtrar_por_marquee(&captions, &hablantes(), rect).is_empty());
    }

    #[test]
    fn shifts_every_row_by_the_same_delta_preserving_the_pattern() {
        let origen = filas(&[("a", 1), ("b", 2), ("c", 3)]);
        assert_eq!(
            filas_destino_relativas(&origen, 1, 5),
            filas(&[("a", 2), ("b", 3), ("c", 4)])
        );
    }

    #[test]
    fn is_identity_with_shift_zero() {
        let origen = filas(&[("a", 0), ("b", 2)]);
        assert_eq!(
            filas_destino_relativas(&origen, 0, 5),
            filas(&[("a", 0), ("b", 2)])
        );
    }

    #[test]
    fn shifts_negatively() {
        let origen = filas(&[("a", 2), ("b", 3)]);
        assert_eq!(
            filas_destino_relativas(&origen, -2, 5),
            filas(&[("a", 0), ("b", 1)])
        );
    }

    #[test]
    fn clamps_at_row_zero_instead_of_going_negative() {
        let origen = filas(&[("a", 0), ("b", 1)]);
        assert_eq!(
            filas_destino_relativas(&origen, -1, 5),
            filas(&[("a", 0), ("b", 0)])
        );
    }

    #[test]
    fn clamps_at_max_row_instead_of_overflowing() {
        let origen = filas(&[("a", 3), ("b", 2)]);
        assert_eq!(
            filas_destino_relativas(&origen, 1, 3),
            filas(&[("a", 3), ("b", 3)])
        );
    }

    #[test]
    fn returns_empty_for_empty_input() {
        assert!(filas_destino_relativas(&BTreeMap::new(), 2, 5).is_empty());
    }

    #[test]
    fn separates_captions_without_a_speaker_from_the_assigned_ones() {
        let captions = [
            caption("a", 0.0, 1.0, None),
            caption("b", 1.0, 2.0, Some("s1")),
        ];
        let agrupacion = agrupar_captions_por_carril(&captions, &hablantes());
        assert_eq!(
            ids(agrupacion.por_hablante.get("s1").expect("s1")),
            vec!["b"]
        );
        assert_eq!(ids(&agrupacion.sin_asignar), vec!["a"]);
    }

    #[test]
    fn a_speaker_without_captions_returns_the_entry_with_an_empty_array() {
        let captions = [caption("a", 0.0, 1.0, Some("s1"))];
        let agrupacion = agrupar_captions_por_carril(&captions, &hablantes());
        // s2 no tiene nada: la clave existe para que el render no tenga que
        // distinguir "sin captions" de "hablante que no está".
        let s2 = agrupacion.por_hablante.get("s2").expect("s2 existe");
        assert!(s2.is_empty());
    }

    #[test]
    fn an_unknown_speaker_id_appears_in_neither_map_nor_unassigned() {
        let captions = [caption("a", 0.0, 1.0, Some("fantasma"))];
        let agrupacion = agrupar_captions_por_carril(&captions, &hablantes());
        assert!(!agrupacion.por_hablante.contains_key("fantasma"));
        assert!(agrupacion.sin_asignar.is_empty());
    }

    #[test]
    fn keeps_the_input_order_inside_each_lane() {
        let captions = [
            caption("a", 0.0, 1.0, Some("s1")),
            caption("b", 2.0, 3.0, Some("s2")),
            caption("c", 4.0, 5.0, Some("s1")),
        ];
        let agrupacion = agrupar_captions_por_carril(&captions, &hablantes());
        assert_eq!(
            ids(agrupacion.por_hablante.get("s1").expect("s1")),
            vec!["a", "c"]
        );
    }
}
