//! Arrastres del timeline (slice 6b): bordes, bloque (tiempo y carril) y
//! marquesina.
//!
//! Cada frame se recalcula desde las posiciones **originales** guardadas al
//! pulsar, no sumando el delta del frame anterior. Acumular delta se desvía:
//! los `f64` no son exactos, el clamp recorta y el snap se queda pegado al
//! último imán tocado, así que soltar donde se pulsó dejaría de ser un no-op
//! exacto. Recalcular desde el original hace que el arrastre sea reversible sin
//! histéresis.
//!
//! La app es la dueña del paso de historial: aquí se muta el proyecto y se
//! devuelve si cambió; la instantánea `respaldo` viaja en el arrastre para que
//! la app la registre en el primer frame que cambie algo (no al pulsar: un clic
//! sin movimiento no es una edición).

use std::collections::BTreeMap;

use crate::timeline::Punto;
use rod_core::{
    Caption, DURACION_MINIMA, Hablante, Instantanea, Proyecto, RectMarquee, caption_row_index,
    clamp_delta_bloque, filas_destino_relativas, filtrar_por_marquee, find_snap_block_delta,
    find_snap_time,
};

use crate::timeline::{Vista, carril_en, clip_en, fila_de_carril};

/// Ancho de la zona de agarre de un borde. El borde se dibuja de 1 px; agarrar
/// 1 px con el ratón es una lotería, así que se aceptan 6 a cada lado.
pub const AGARRE_BORDE: f32 = 6.0;

/// Cuántos píxeles hay que moverse para que un arrastre de marquesina no
/// cuente como un clic.
pub const UMBRAL_CLIC: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asa {
    Izquierda,
    Derecha,
}

/// Dónde estaba un caption al pulsar. Es la base del recálculo por frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Original {
    pub id: String,
    pub inicio: f64,
    pub fin: f64,
    pub fila: i32,
}

impl Original {
    fn de(caption: &Caption, hablantes: &[Hablante]) -> Self {
        Self {
            id: caption.id.clone(),
            inicio: caption.inicio,
            fin: caption.fin,
            fila: caption_row_index(caption.hablante_id.as_deref(), hablantes) as i32,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Modo {
    Borde {
        id: String,
        asa: Asa,
        original: Original,
    },
    Bloque {
        originales: Vec<Original>,
        /// Punto donde se pulsó: el delta se mide contra él, no contra el frame
        /// anterior.
        origen: Punto,
        fila_origen: i32,
    },
    Marquesina {
        origen: Punto,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arrastre {
    pub modo: Modo,
    /// Estado del proyecto al pulsar. Solo los arrastres que pueden cambiar el
    /// proyecto lo llevan; la marquesina solo toca la selección.
    pub respaldo: Option<Instantanea>,
    /// La app ya registró el paso de historial de este arrastre.
    pub con_historial: bool,
}

/// Lo que un arrastre necesita saber del mundo además del proyecto: dónde está
/// el ratón, cómo se traduce a tiempo y cuáles son los topes. Agruparlo evita
/// una firma de ocho argumentos que nadie lee bien en las llamadas.
#[derive(Debug, Clone, Copy)]
pub struct Marco {
    pub vista: Vista,
    pub punto: Punto,
    pub duracion: f64,
    pub playhead: f64,
}

impl Marco {
    pub fn nuevo(vista: Vista, punto: Punto, duracion: f64, playhead: f64) -> Self {
        Self {
            vista,
            punto,
            duracion,
            playhead,
        }
    }
}

impl Arrastre {
    pub fn es_marquesina(&self) -> bool {
        matches!(self.modo, Modo::Marquesina { .. })
    }

    pub fn origen(&self) -> Option<Punto> {
        match &self.modo {
            Modo::Bloque { origen, .. } | Modo::Marquesina { origen } => Some(*origen),
            Modo::Borde { .. } => None,
        }
    }

    /// Recalcula el proyecto con el ratón en `marco.punto`. `false` si nada
    /// cambió.
    ///
    /// El `playhead` entra como imán: arrastrar un borde hasta el cursor del
    /// reproductor es la forma habitual de pegar dos captions.
    pub fn aplicar(&self, proyecto: &mut Proyecto, marco: Marco) -> bool {
        match &self.modo {
            Modo::Borde { id, asa, original } => aplicar_borde(proyecto, marco, id, *asa, original),
            Modo::Bloque {
                originales,
                origen,
                fila_origen,
            } => aplicar_bloque(proyecto, marco, originales, *origen, *fila_origen),
            Modo::Marquesina { .. } => false,
        }
    }

    /// Ids que caen dentro del rectángulo de la marquesina.
    pub fn ids_de_marquesina(&self, proyecto: &Proyecto, marco: Marco) -> Vec<String> {
        let Modo::Marquesina { origen } = &self.modo else {
            return Vec::new();
        };
        filtrar_por_marquee(
            &proyecto.captions,
            &proyecto.hablantes,
            RectMarquee {
                t1: marco.vista.tiempo_de(origen.x),
                t2: marco.vista.tiempo_de(marco.punto.x),
                fila1: fila_de_carril(origen.y, proyecto.hablantes.len()),
                fila2: fila_de_carril(marco.punto.y, proyecto.hablantes.len()),
            },
        )
    }
}

/// Qué hacer con la pulsación.
#[derive(Debug, Clone, PartialEq)]
pub enum Respuesta {
    /// Regla o fuera de los carriles: mover el playhead.
    Buscar(f64),
    /// Empezar un arrastre; la selección que la app debe dejar puesta.
    Empezar {
        arrastre: Arrastre,
        seleccion: Vec<String>,
    },
}

pub fn pulsar(proyecto: &Proyecto, vista: Vista, punto: Punto, seleccion: &[String]) -> Respuesta {
    let Some(carril) = carril_en(punto.y) else {
        return Respuesta::Buscar(vista.tiempo_de(punto.x).max(0.0));
    };

    let agrupacion = rod_core::agrupar_captions_por_carril(&proyecto.captions, &proyecto.hablantes);
    let captions = if carril == 0 {
        agrupacion.sin_asignar
    } else {
        proyecto
            .hablantes
            .get(carril - 1)
            .and_then(|hablante| agrupacion.por_hablante.get(&hablante.id).cloned())
            .unwrap_or_default()
    };

    let Some(caption) = clip_en(&captions, vista, punto.x) else {
        // Hueco del carril: marquesina. Si se suelta sin moverse, la app busca.
        return Respuesta::Empezar {
            arrastre: Arrastre {
                modo: Modo::Marquesina { origen: punto },
                respaldo: None,
                con_historial: false,
            },
            seleccion: seleccion.to_vec(),
        };
    };

    let original = Original::de(caption, &proyecto.hablantes);
    let inicio = vista.x_de(caption.inicio);
    let fin = vista.x_de(caption.fin).max(inicio + 2.0);
    let asa = if (punto.x - inicio).abs() <= AGARRE_BORDE {
        // En un clip estrecho los dos agarres se solapan: gana el izquierdo,
        // que es el que la vista dibuja primero.
        Some(Asa::Izquierda)
    } else if (fin - punto.x).abs() <= AGARRE_BORDE {
        Some(Asa::Derecha)
    } else {
        None
    };

    if let Some(asa) = asa {
        return Respuesta::Empezar {
            arrastre: Arrastre {
                modo: Modo::Borde {
                    id: caption.id.clone(),
                    asa,
                    original,
                },
                respaldo: Some(Instantanea::de_proyecto(proyecto)),
                con_historial: false,
            },
            seleccion: vec![caption.id.clone()],
        };
    }

    // Bloque: si el clip pulsado ya está en la selección se mueve todo, como en
    // el oráculo; si no, la selección pasa a ser ese clip.
    let ids: Vec<String> = if seleccion.iter().any(|id| id == &caption.id) {
        seleccion.to_vec()
    } else {
        vec![caption.id.clone()]
    };
    let originales: Vec<Original> = ids
        .iter()
        .filter_map(|id| {
            proyecto
                .captions
                .iter()
                .find(|caption| &caption.id == id)
                .map(|caption| Original::de(caption, &proyecto.hablantes))
        })
        .collect();

    Respuesta::Empezar {
        arrastre: Arrastre {
            modo: Modo::Bloque {
                originales,
                origen: punto,
                fila_origen: carril as i32,
            },
            respaldo: Some(Instantanea::de_proyecto(proyecto)),
            con_historial: false,
        },
        seleccion: ids,
    }
}

fn aplicar_borde(
    proyecto: &mut Proyecto,
    marco: Marco,
    id: &str,
    asa: Asa,
    original: &Original,
) -> bool {
    if !proyecto.captions.iter().any(|caption| caption.id == id) {
        return false;
    }

    let pedido = marco.vista.tiempo_de(marco.punto.x);
    let id_ajeno = [id];
    let imanes = [marco.playhead];
    let tiempo = find_snap_time(pedido, &id_ajeno, &proyecto.captions, &imanes).unwrap_or(pedido);

    // El clamp es explícito por asa: `clamp_caption_tiempos` empuja el borde
    // contrario cuando la duración mínima no cabe, y aquí eso movería el borde
    // que el usuario no está tocando.
    let (inicio, fin) = match asa {
        Asa::Izquierda => (
            a_milisegundos(tiempo.clamp(0.0, original.fin - DURACION_MINIMA)),
            original.fin,
        ),
        Asa::Derecha => {
            let tope = if marco.duracion > 0.0 {
                marco.duracion.max(original.inicio + DURACION_MINIMA)
            } else {
                f64::INFINITY
            };
            (
                original.inicio,
                a_milisegundos(tiempo.clamp(original.inicio + DURACION_MINIMA, tope)),
            )
        }
    };

    let Some(caption) = proyecto
        .captions
        .iter_mut()
        .find(|caption| caption.id == id)
    else {
        return false;
    };
    if (caption.inicio - inicio).abs() < 1e-9 && (caption.fin - fin).abs() < 1e-9 {
        return false;
    }
    caption.inicio = inicio;
    caption.fin = fin;
    true
}

fn aplicar_bloque(
    proyecto: &mut Proyecto,
    marco: Marco,
    originales: &[Original],
    origen: Punto,
    fila_origen: i32,
) -> bool {
    let Some(inicio_min) = originales.iter().map(|o| o.inicio).reduce(f64::min) else {
        return false;
    };
    let fin_max = originales.iter().map(|o| o.fin).fold(f64::MIN, f64::max);

    let mut delta = marco.vista.tiempo_de(marco.punto.x) - marco.vista.tiempo_de(origen.x);
    delta = clamp_delta_bloque(delta, inicio_min);
    if marco.duracion > 0.0 {
        // El bloque tampoco puede salirse por la derecha.
        delta = delta.min(marco.duracion - fin_max);
    }

    let ids: Vec<&str> = originales.iter().map(|o| o.id.as_str()).collect();
    let snap = find_snap_block_delta(
        inicio_min + delta,
        fin_max + delta,
        &ids,
        &proyecto.captions,
        Some(marco.playhead),
    );
    let delta = a_milisegundos(delta + snap);

    let shift = fila_de_carril(marco.punto.y, proyecto.hablantes.len()) - fila_origen;
    let filas: BTreeMap<String, i32> = originales
        .iter()
        .map(|original| (original.id.clone(), original.fila))
        .collect();
    let destinos = filas_destino_relativas(&filas, shift, proyecto.hablantes.len() as i32);
    // Id por fila, resuelto antes de tocar los captions: mientras se muta
    // `captions` no se puede leer `hablantes` del mismo proyecto.
    let hablante_por_fila: Vec<Option<String>> = std::iter::once(None)
        .chain(proyecto.hablantes.iter().map(|h| Some(h.id.clone())))
        .collect();

    let mut cambio = false;
    for original in originales {
        let destino = destinos.get(&original.id).copied().unwrap_or(original.fila);
        let hablante_id = hablante_por_fila
            .get(destino.max(0) as usize)
            .cloned()
            .flatten();
        let inicio = a_milisegundos(original.inicio + delta);
        let fin = a_milisegundos(original.fin + delta);
        let Some(caption) = proyecto
            .captions
            .iter_mut()
            .find(|caption| caption.id == original.id)
        else {
            continue;
        };
        if (caption.inicio - inicio).abs() < 1e-9
            && (caption.fin - fin).abs() < 1e-9
            && caption.hablante_id == hablante_id
        {
            continue;
        }
        caption.inicio = inicio;
        caption.fin = fin;
        caption.hablante_id = hablante_id;
        cambio = true;
    }
    cambio
}

/// Las marcas de tiempo del `.srt` y el `.ass` son de milisegundos: sin
/// cuantizar, un arrastre deja `inicio: 1.2000000000000002` en el proyecto y en
/// el archivo exportado.
fn a_milisegundos(tiempo: f64) -> f64 {
    (tiempo * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::{AGARRE_BORDE, Arrastre, Asa, Marco, Modo, Respuesta, pulsar};
    use crate::timeline::Punto;
    use crate::timeline::{ALTO_CARRIL, ALTO_REGLA, Vista};
    use rod_core::{Caption, Hablante, Proyecto};

    fn proyecto() -> Proyecto {
        let mut proyecto = Proyecto::nuevo("video.mp4");
        proyecto.hablantes = vec![
            Hablante {
                id: "sp-1".into(),
                nombre: "Ana".into(),
                tecla: "1".into(),
                color: "#FFFFFF".into(),
                preset_id: None,
            },
            Hablante {
                id: "sp-2".into(),
                nombre: "Beto".into(),
                tecla: "2".into(),
                color: "#FFFFFF".into(),
                preset_id: None,
            },
        ];
        proyecto.captions = vec![
            Caption {
                id: "cap-1".into(),
                inicio: 1.0,
                fin: 3.0,
                texto: "Hola".into(),
                hablante_id: None,
            },
            Caption {
                id: "cap-2".into(),
                inicio: 4.0,
                fin: 5.0,
                texto: "Adios".into(),
                hablante_id: Some("sp-1".into()),
            },
        ];
        proyecto
    }

    /// 0.02 s/px: cap-1 va de x=50 a x=150, cap-2 de x=200 a x=250.
    fn vista() -> Vista {
        Vista::nueva(0.02)
    }

    fn y_de(fila: usize) -> f32 {
        ALTO_REGLA + fila as f32 * ALTO_CARRIL + ALTO_CARRIL / 2.0
    }

    /// Un frame del arrastre con el ratón en (x, y), sobre 15 s de video y el
    /// playhead en 9 s (fuera del alcance de estos tests salvo el de snap).
    fn marco(x: f32, y: f32) -> Marco {
        Marco::nuevo(vista(), Punto::new(x, y), 15.0, 9.0)
    }

    fn empezar(proyecto: &Proyecto, p: Punto, seleccion: &[String]) -> (Arrastre, Vec<String>) {
        match pulsar(proyecto, vista(), p, seleccion) {
            Respuesta::Empezar {
                arrastre,
                seleccion,
            } => (arrastre, seleccion),
            Respuesta::Buscar(t) => panic!("se esperaba arrastre, se buscó a {t}"),
        }
    }

    fn caption(proyecto: &Proyecto, id: &str) -> Caption {
        proyecto
            .captions
            .iter()
            .find(|caption| caption.id == id)
            .cloned()
            .expect("el caption existe")
    }

    #[test]
    fn pulsar_en_la_regla_pide_buscar() {
        match pulsar(&proyecto(), vista(), Punto::new(100.0, 4.0), &[]) {
            Respuesta::Buscar(t) => assert!((t - 2.0).abs() < 1e-9),
            otro => panic!("se esperaba Buscar, llegó {otro:?}"),
        }
    }

    #[test]
    fn pulsar_el_cuerpo_de_un_clip_prepara_un_bloque() {
        let proyecto = proyecto();
        let (arrastre, seleccion) = empezar(&proyecto, Punto::new(100.0, y_de(0)), &[]);
        assert!(matches!(arrastre.modo, Modo::Bloque { .. }));
        assert_eq!(seleccion, vec!["cap-1".to_string()]);
        assert!(
            arrastre.respaldo.is_some(),
            "sin respaldo no se puede deshacer"
        );
    }

    #[test]
    fn pulsar_cerca_de_un_borde_agarra_ese_borde() {
        let proyecto = proyecto();
        let (izquierdo, _) = empezar(
            &proyecto,
            Punto::new(50.0 + AGARRE_BORDE - 1.0, y_de(0)),
            &[],
        );
        match izquierdo.modo {
            Modo::Borde { asa, .. } => assert_eq!(asa, Asa::Izquierda),
            otro => panic!("se esperaba un borde, llegó {otro:?}"),
        }

        let (derecho, _) = empezar(
            &proyecto,
            Punto::new(150.0 - AGARRE_BORDE + 1.0, y_de(0)),
            &[],
        );
        match derecho.modo {
            Modo::Borde { asa, .. } => assert_eq!(asa, Asa::Derecha),
            otro => panic!("se esperaba un borde, llegó {otro:?}"),
        }
    }

    #[test]
    fn pulsar_un_clip_ya_seleccionado_mueve_toda_la_seleccion() {
        let proyecto = proyecto();
        let seleccion = vec!["cap-1".to_string(), "cap-2".to_string()];
        let (arrastre, ids) = empezar(&proyecto, Punto::new(100.0, y_de(0)), &seleccion);
        match arrastre.modo {
            Modo::Bloque { originales, .. } => assert_eq!(originales.len(), 2),
            otro => panic!("se esperaba un bloque, llegó {otro:?}"),
        }
        assert_eq!(ids, seleccion);
    }

    #[test]
    fn arrastrar_el_borde_derecho_alarga_sin_tocar_el_inicio() {
        let mut proyecto = proyecto();
        let (arrastre, _) = empezar(&proyecto, Punto::new(150.0, y_de(0)), &[]);

        // Hasta x=250 -> 5 s.
        assert!(arrastre.aplicar(&mut proyecto, marco(250.0, y_de(0))));
        let editado = caption(&proyecto, "cap-1");
        assert!((editado.fin - 5.0).abs() < 1e-6, "fin = {}", editado.fin);
        assert!((editado.inicio - 1.0).abs() < 1e-6);
    }

    #[test]
    fn el_borde_no_puede_cruzar_el_borde_contrario() {
        let mut proyecto = proyecto();
        let (arrastre, _) = empezar(&proyecto, Punto::new(50.0, y_de(0)), &[]);

        // Muy a la derecha: el inicio se queda a DURACION_MINIMA del fin.
        assert!(arrastre.aplicar(&mut proyecto, marco(400.0, y_de(0))));
        let editado = caption(&proyecto, "cap-1");
        assert!((editado.inicio - (editado.fin - 0.1)).abs() < 1e-6);
        assert!(editado.inicio <= editado.fin);
    }

    #[test]
    fn soltar_donde_se_pulso_no_cambia_nada() {
        let mut proyecto = proyecto();
        let antes = proyecto.clone();
        let (arrastre, _) = empezar(&proyecto, Punto::new(100.0, y_de(0)), &[]);
        assert!(!arrastre.aplicar(&mut proyecto, marco(100.0, y_de(0))));
        assert_eq!(proyecto.captions, antes.captions);
    }

    #[test]
    fn el_bloque_se_mueve_sin_deformarse() {
        let mut proyecto = proyecto();
        let seleccion = vec!["cap-1".to_string(), "cap-2".to_string()];
        let (arrastre, _) = empezar(&proyecto, Punto::new(100.0, y_de(0)), &seleccion);

        // +50 px = +1 s.
        assert!(arrastre.aplicar(&mut proyecto, marco(150.0, y_de(0))));
        let primero = caption(&proyecto, "cap-1");
        let segundo = caption(&proyecto, "cap-2");
        assert!((primero.inicio - 2.0).abs() < 1e-6);
        assert!(
            (segundo.inicio - 5.0).abs() < 1e-6,
            "el desfase se conserva"
        );
        assert!((primero.fin - 4.0).abs() < 1e-6);
        assert!((segundo.fin - 6.0).abs() < 1e-6);
    }

    #[test]
    fn el_bloque_no_puede_irse_a_tiempo_negativo() {
        let mut proyecto = proyecto();
        let (arrastre, _) = empezar(&proyecto, Punto::new(100.0, y_de(0)), &[]);

        assert!(arrastre.aplicar(&mut proyecto, marco(0.0, y_de(0))));
        assert!(caption(&proyecto, "cap-1").inicio >= 0.0);
        assert!((caption(&proyecto, "cap-1").inicio - 0.0).abs() < 1e-6);
    }

    #[test]
    fn arrastrar_en_vertical_cambia_de_carril() {
        let mut proyecto = proyecto();
        let (arrastre, _) = empezar(&proyecto, Punto::new(100.0, y_de(0)), &[]);

        // Del carril `—` al carril de Beto (fila 2).
        assert!(arrastre.aplicar(&mut proyecto, marco(100.0, y_de(2))));
        let editado = caption(&proyecto, "cap-1");
        assert_eq!(editado.hablante_id.as_deref(), Some("sp-2"));
        assert!(
            (editado.inicio - 1.0).abs() < 1e-6,
            "cambiar de carril no mueve el tiempo"
        );
    }

    #[test]
    fn el_bloque_cede_en_el_borde_de_los_carriles() {
        let mut proyecto = proyecto();
        let seleccion = vec!["cap-1".to_string(), "cap-2".to_string()];
        // Se agarra cap-2 (fila 1, de x=200 a 250) con los dos seleccionados.
        let (arrastre, _) = empezar(&proyecto, Punto::new(225.0, y_de(1)), &seleccion);

        // Subir al carril `—`: cap-1 ya está en la fila 0, así que se queda y
        // cap-2 baja a la 0; el desfase relativo se conserva.
        assert!(arrastre.aplicar(&mut proyecto, marco(225.0, y_de(0))));
        assert!(caption(&proyecto, "cap-1").hablante_id.is_none());
        assert!(
            caption(&proyecto, "cap-2").hablante_id.is_none(),
            "el bloque cedió en la fila 0, no se fue a la -1"
        );
    }

    #[test]
    fn el_snap_pega_el_bloque_al_vecino() {
        let mut proyecto = proyecto();
        let (arrastre, _) = empezar(&proyecto, Punto::new(100.0, y_de(0)), &[]);

        // cap-1 acaba en 2-4 s y cap-2 empieza en 4 s: el fin del bloque queda
        // a 0.03 s del vecino y el imán lo pega.
        let destino = 2.0 + 1.03;
        assert!(arrastre.aplicar(&mut proyecto, marco(vista().x_de(destino), y_de(0))));
        let editado = caption(&proyecto, "cap-1");
        assert!(
            (editado.fin - 4.0).abs() < 1e-6,
            "fin = {} (debería pegarse a los 4 s del vecino)",
            editado.fin
        );
        assert!((editado.inicio - 2.0).abs() < 1e-6);
    }

    #[test]
    fn la_marquesina_devuelve_los_captions_que_toca() {
        let proyecto = proyecto();
        let (arrastre, _) = empezar(&proyecto, Punto::new(40.0, y_de(0)), &[]);
        assert!(arrastre.es_marquesina());

        // De 0.8 s a 4.5 s cubre cap-1 (1-3) y el arranque de cap-2 (4-5), pero
        // cap-2 está en otra fila.
        let ids = arrastre.ids_de_marquesina(&proyecto, marco(240.0, y_de(0)));
        assert_eq!(ids, vec!["cap-1".to_string()]);

        let ids = arrastre.ids_de_marquesina(&proyecto, marco(240.0, y_de(1)));
        assert_eq!(ids, vec!["cap-1".to_string(), "cap-2".to_string()]);
    }

    #[test]
    fn un_clic_sin_moverse_no_es_una_marquesina() {
        let proyecto = proyecto();
        let (arrastre, _) = empezar(&proyecto, Punto::new(40.0, y_de(0)), &[]);
        let origen = arrastre.origen().expect("la marquesina guarda su origen");
        assert!((origen.x - 40.0).abs() < 1e-6);
        // La app decide clic vs marquesina comparando el punto de soltar con
        // este origen; aquí solo se comprueba que el origen se conservó.
        assert!(arrastre.es_marquesina());
    }
}
