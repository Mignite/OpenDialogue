//! Timeline vectorial: regla de tiempo, un carril por hablante más el carril
//! `—`, clips, playhead, marquesina y arrastres (slice 6a y 6b).
//!
//! La aritmética tiempo↔x y el zoom anclado viven en [`Vista`], con tests, como
//! la geometría del compositor: es la parte que se equivoca en silencio. El
//! `Program` de canvas solo dibuja y traduce el ratón a puntos; quién se agarra
//! y qué pasa con el proyecto lo decide [`arrastre`], que también tiene tests.
//!
//! Sin waveform todavía (llega con `useWaveform`). El dibujo no usa `Cache`: son
//! unos cientos de formas y un cache que hay que invalidar a mano en cada
//! edición es una fuente de bugs peor que el ahorro. Si el profiling con la
//! waveform lo pide, se añade.

mod arrastre;

pub use arrastre::{Arrastre, Asa, Marco, Modo, Original, Respuesta, UMBRAL_CLIC, pulsar};

use rod_core::Caption;

/// Punto local del widget. Antes era `iced::Point`; lo definimos aquí para que
/// el front-end no arrastre el crate de iced solo por un tipo de dos f32.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Punto {
    pub x: f32,
    pub y: f32,
}

impl Punto {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Altura de la regla de tiempo (arriba del todo).
pub const ALTO_REGLA: f32 = 26.0;
/// Altura de cada carril.
pub const ALTO_CARRIL: f32 = 56.0;
// Los tres números del seguimiento vienen del oráculo (`utils/constants.ts`):
// cuando el playhead cruza el 92 % de la ventana, la ventana se recoloca para
// dejarlo al 8 % del borde izquierdo, y el desplazamiento se interpola un 12 %
// por frame para que no salte.
pub const EDGE_TRIGGER: f64 = 0.92;
pub const NEW_MARGIN: f64 = 0.08;
pub const LERP: f64 = 0.12;

/// Qué ve el usuario del eje de tiempo: dónde empieza y cuánto abarca.
///
/// `escala` es segundos por píxel: crece al alejarse (más segundos en la misma
/// ventana). Se mantiene en un rango porque 0 sería una división por cero y
/// valores absurdos hacen que la regla dibuje millones de marcas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vista {
    pub inicio: f64,
    pub escala: f64,
}

impl Default for Vista {
    fn default() -> Self {
        Self::nueva(0.02)
    }
}

impl Vista {
    pub const ESCALA_MINIMA: f64 = 0.002;
    pub const ESCALA_MAXIMA: f64 = 0.5;

    pub fn nueva(escala: f64) -> Self {
        Self {
            inicio: 0.0,
            escala: escala.clamp(Self::ESCALA_MINIMA, Self::ESCALA_MAXIMA),
        }
    }

    pub fn x_de(&self, tiempo: f64) -> f32 {
        ((tiempo - self.inicio) / self.escala) as f32
    }

    pub fn tiempo_de(&self, x: f32) -> f64 {
        self.inicio + f64::from(x) * self.escala
    }

    pub fn ancho_de(&self, duracion: f64) -> f32 {
        (duracion / self.escala) as f32
    }

    /// Cambia la escala dejando quieto el tiempo que hay bajo `ancla` (en px).
    /// El inicio se acota a 0: no existe un segundo negativo que mostrar.
    pub fn con_escala(&self, escala: f64, ancla: f32) -> Self {
        let escala = escala.clamp(Self::ESCALA_MINIMA, Self::ESCALA_MAXIMA);
        let tiempo_ancla = self.tiempo_de(ancla);
        Self {
            inicio: (tiempo_ancla - f64::from(ancla) * escala).max(0.0),
            escala,
        }
    }

    /// Segundos que abarca la ventana con este ancho de widget.
    pub fn ancho_visible(&self, ancho_px: f32) -> f64 {
        f64::from(ancho_px) * self.escala
    }

    /// Inicio más a la derecha que deja ver algo: si el video es más corto que
    /// la ventana, no hay nada que desplazar y el tope es 0.
    pub fn max_inicio(&self, duracion: f64, ancho_px: f32) -> f64 {
        (duracion - self.ancho_visible(ancho_px)).max(0.0)
    }

    /// Misma escala, otro inicio, acotado a la ventana del video. Es el único
    /// sitio que mueve `inicio`: así no puede quedar uno fuera de rango.
    pub fn con_inicio(&self, inicio: f64, duracion: f64, ancho_px: f32) -> Self {
        Self {
            inicio: inicio.clamp(0.0, self.max_inicio(duracion, ancho_px)),
            escala: self.escala,
        }
    }

    /// Desplaza la ventana `segundos` (negativo = hacia atrás).
    pub fn desplazar(&self, segundos: f64, duracion: f64, ancho_px: f32) -> Self {
        self.con_inicio(self.inicio + segundos, duracion, ancho_px)
    }

    /// Reacota el inicio sin moverlo: necesaria después de alejarse, porque un
    /// inicio que cabía con la escala vieja puede quedar más allá del final con
    /// la nueva.
    pub fn acotar(&self, duracion: f64, ancho_px: f32) -> Self {
        self.con_inicio(self.inicio, duracion, ancho_px)
    }

    /// La ventana justa para un video: lo muestra entero, desde 0. Con un
    /// ancho desconocido (el canvas aún no se midió) devuelve la vista tal cual.
    pub fn encajar(&self, duracion: f64, ancho_px: f32) -> Self {
        let (ancho, duracion) = match (ancho_px, duracion) {
            (ancho, duracion)
                if ancho.is_finite() && ancho > 0.0 && duracion.is_finite() && duracion > 0.0 =>
            {
                (f64::from(ancho), duracion)
            }
            // Ancho sin medir o video sin duración: no hay nada que encajar.
            _ => return *self,
        };
        // `nueva` ya acota al rango válido de escala.
        Self::nueva(duracion / ancho)
    }
}

/// Paso de la regla en segundos para una densidad dada de píxeles por segundo.
pub fn paso_de_regla(px_por_segundo: f64) -> f64 {
    if px_por_segundo >= 500.0 {
        0.5
    } else if px_por_segundo >= 50.0 {
        1.0
    } else if px_por_segundo >= 5.0 {
        10.0
    } else {
        60.0
    }
}

/// Tiempo más cercano dentro de la tolerancia, para ajustar al soltar un clip.
pub fn iman(candidato: f64, puntos: &[f64], tolerancia_s: f64) -> Option<f64> {
    puntos
        .iter()
        .copied()
        .filter_map(|punto| {
            let distancia = (punto - candidato).abs();
            (distancia <= tolerancia_s).then_some((distancia, punto))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, punto)| punto)
}

/// Segundo que hay bajo una x del widget.
pub fn tiempo_bajo_cursor(vista: Vista, x: f32) -> f64 {
    vista.tiempo_de(x)
}

/// Cambia la escala dejando quieto el segundo bajo el cursor.
pub fn escala_anclada(vista: Vista, factor: f64, x: f32, duracion: f64, ancho: f32) -> Vista {
    let escala = (vista.escala * factor).clamp(Vista::ESCALA_MINIMA, Vista::ESCALA_MAXIMA);
    let anclado = tiempo_bajo_cursor(vista, x);
    let max_inicio = (duracion - f64::from(ancho) * escala).max(0.0);
    Vista {
        inicio: (anclado - f64::from(x) * escala).clamp(0.0, max_inicio),
        escala,
    }
}

/// Dónde tiene que empezar la ventana para que el playhead quede a la vista.
///
/// `None` cuando no hay que mover nada: el playhead está dentro de la ventana
/// y todavía no cruzó el disparador del borde derecho. Se recoloca al 8 % del
/// borde izquierdo (no al centro) para dejar sitio a lo que viene.
pub fn objetivo_de_seguimiento(
    vista: Vista,
    playhead: f64,
    duracion: f64,
    ancho_px: f32,
) -> Option<f64> {
    let ancho = vista.ancho_visible(ancho_px);
    let fuera_izquierda = playhead < vista.inicio;
    let pasado_el_borde = playhead > vista.inicio + ancho * EDGE_TRIGGER;
    if !fuera_izquierda && !pasado_el_borde {
        return None;
    }
    Some(
        vista
            .con_inicio(playhead - ancho * NEW_MARGIN, duracion, ancho_px)
            .inicio,
    )
}

/// Un frame del deslizamiento hacia `objetivo`: mueve el inicio una fracción
/// del camino que falta. `false` cuando ya llegó (y lo deja exacto).
///
/// Interpolar 12 % por frame y cortar en 2 ms evita el salto seco sin dejar un
/// residuo que se arrastre para siempre.
pub fn acercar(vista: &mut Vista, objetivo: f64, duracion: f64, ancho_px: f32) -> bool {
    let diff = objetivo - vista.inicio;
    if diff.abs() <= 0.002 {
        *vista = vista.con_inicio(objetivo, duracion, ancho_px);
        return false;
    }
    *vista = vista.con_inicio(vista.inicio + diff * LERP, duracion, ancho_px);
    true
}

/// Lo que el timeline le pide a la app. Son puntos locales del widget y nada
/// más: el significado (buscar, agarrar un borde, mover un bloque) lo decide
/// [`arrastre::pulsar`] con el proyecto delante, y la rueda la interpreta la
/// app, que es la única que sabe si el usuario tiene Shift pulsado.
#[derive(Debug, Clone, PartialEq)]
pub enum Mensaje {
    Pulsar(Punto),
    Arrastrar(Punto),
    Soltar(Punto),
    /// La rueda giró `lineas` sobre `punto`: desplazar o acercar lo decide la
    /// app (Shift = zoom).
    Rueda {
        lineas: f32,
        punto: Punto,
    },
    /// El widget cambió de ancho. La app lo necesita para saber cuántos
    /// segundos caben y hasta dónde puede desplazar la ventana.
    Ancho(f32),
}

/// Y de la primera fila del carril (el borde superior).
pub fn y_de_carril(carril: usize) -> f32 {
    ALTO_REGLA + carril as f32 * ALTO_CARRIL
}

/// Carril bajo una `y` local, si la hay: la regla no es un carril.
pub fn carril_en(y: f32) -> Option<usize> {
    if y < ALTO_REGLA {
        return None;
    }
    Some(((y - ALTO_REGLA) / ALTO_CARRIL) as usize)
}

/// Fila (0 = sin hablante) bajo una `y`, acotada a los carriles que existen.
pub fn fila_de_carril(y: f32, hablantes: usize) -> i32 {
    carril_en(y).unwrap_or(0).min(hablantes) as i32
}

/// Clip bajo `x` en el carril dado. El de arriba (el más reciente en el vector)
/// gana si dos se solapan: es el que se ve.
pub fn clip_en(captions: &[Caption], vista: Vista, x: f32) -> Option<&Caption> {
    captions.iter().rev().find(|caption| {
        let inicio = vista.x_de(caption.inicio);
        let fin = vista.x_de(caption.fin);
        x >= inicio && x <= fin.max(inicio + 2.0)
    })
}

#[cfg(test)]
mod tests {
    use super::{
        ALTO_CARRIL, ALTO_REGLA, EDGE_TRIGGER, NEW_MARGIN, Vista, acercar, carril_en, clip_en,
        fila_de_carril, objetivo_de_seguimiento, y_de_carril,
    };
    use rod_core::{Caption, Hablante, Proyecto, agrupar_captions_por_carril};

    const ANCHO: f32 = 800.0;

    #[test]
    fn altura_de_carril_coincide_con_y_de_carril() {
        assert!((y_de_carril(1) - y_de_carril(0) - ALTO_CARRIL).abs() < 0.001);
        assert_eq!(carril_en(y_de_carril(2) + 1.0), Some(2));
    }

    fn proyecto() -> Proyecto {
        let mut proyecto = Proyecto::nuevo("video.mp4");
        proyecto.hablantes.push(Hablante {
            id: "sp-1".into(),
            nombre: "Ana".into(),
            tecla: "1".into(),
            color: "#FFFFFF".into(),
            preset_id: None,
        });
        proyecto.captions = vec![
            Caption {
                id: "cap-1".into(),
                inicio: 1.0,
                fin: 3.0,
                texto: "Hola".into(),
                hablante_id: Some("sp-1".into()),
            },
            Caption {
                id: "cap-2".into(),
                inicio: 4.0,
                fin: 5.0,
                texto: "Sin nadie".into(),
                hablante_id: None,
            },
        ];
        proyecto
    }

    #[test]
    fn tiempo_y_px_son_inversos() {
        let vista = Vista::nueva(0.02);
        for tiempo in [0.0, 1.5, 12.25, 99.0] {
            let x = vista.x_de(tiempo);
            assert!((vista.tiempo_de(x) - tiempo).abs() < 1e-9);
        }
    }

    #[test]
    fn el_ancho_de_un_clip_es_su_duracion_entre_la_escala() {
        let vista = Vista::nueva(0.02);
        assert!((vista.ancho_de(3.0) - 150.0).abs() < 1e-3);
        assert!((vista.ancho_de(0.0)).abs() < 1e-6);
    }

    #[test]
    fn el_zoom_deja_quieto_el_tiempo_bajo_el_ancla() {
        // Lejos del borde 0 para que el acotado de `inicio` no interfiera: ese
        // caso tiene su propio test.
        let vista = Vista {
            inicio: 10.0,
            escala: 0.01,
        };
        let ancla = 300.0;
        let tiempo_ancla = vista.tiempo_de(ancla);

        let acercado = vista.con_escala(0.005, ancla);
        assert!((acercado.tiempo_de(ancla) - tiempo_ancla).abs() < 1e-9);

        let alejado = acercado.con_escala(0.02, ancla);
        assert!((alejado.tiempo_de(ancla) - tiempo_ancla).abs() < 1e-9);
    }

    #[test]
    fn la_escala_se_acota_a_un_rango_util() {
        let vista = Vista::nueva(1e-9);
        assert_eq!(vista.escala, Vista::ESCALA_MINIMA);
        assert_eq!(Vista::nueva(1e9).escala, Vista::ESCALA_MAXIMA);
        assert_eq!(
            vista.con_escala(1e9, 0.0).escala,
            Vista::ESCALA_MAXIMA,
            "el zoom también se acota"
        );
    }

    #[test]
    fn el_inicio_no_se_va_a_segundos_negativos() {
        // Cerca del borde 0, alejar el zoom empujaría el inicio a negativo.
        let vista = Vista {
            inicio: 0.0,
            escala: 0.005,
        };
        let alejada = vista.con_escala(0.5, 0.0);
        assert_eq!(alejada.inicio, 0.0);
    }

    #[test]
    fn el_carril_sale_de_la_y_y_la_regla_no_es_un_carril() {
        assert_eq!(carril_en(ALTO_REGLA - 1.0), None);
        assert_eq!(carril_en(ALTO_REGLA + 1.0), Some(0));
        assert_eq!(carril_en(ALTO_REGLA + ALTO_CARRIL + 1.0), Some(1));
        // La fila se acota a los carriles que existen: no hay fila 9 con un
        // solo hablante.
        assert_eq!(fila_de_carril(ALTO_REGLA + 40.0 * ALTO_CARRIL, 1), 1);
    }

    #[test]
    fn cada_caption_cae_en_su_carril() {
        let proyecto = proyecto();
        let agrupacion = agrupar_captions_por_carril(&proyecto.captions, &proyecto.hablantes);
        assert_eq!(agrupacion.sin_asignar.len(), 1);
        let del_hablante = &agrupacion.por_hablante["sp-1"];
        assert_eq!(del_hablante[0].id, "cap-1");
    }

    #[test]
    fn el_clic_encuentra_el_clip_que_esta_debajo() {
        let proyecto = proyecto();
        let vista = Vista::nueva(0.02);
        let agrupacion = agrupar_captions_por_carril(&proyecto.captions, &proyecto.hablantes);
        let del_hablante = &agrupacion.por_hablante["sp-1"];

        // cap-1 va de 1 s a 3 s: en x, de 50 a 150 px.
        let encontrado = clip_en(del_hablante, vista, 100.0);
        assert_eq!(encontrado.map(|caption| caption.id.as_str()), Some("cap-1"));
        assert!(clip_en(del_hablante, vista, 10.0).is_none());
    }

    #[test]
    fn un_clip_mas_ancho_que_un_pixel_se_puede_tocar() {
        let mut proyecto = proyecto();
        proyecto.captions.push(Caption {
            id: "cap-3".into(),
            inicio: 6.0,
            fin: 6.5,
            texto: "Diminuto".into(),
            hablante_id: None,
        });
        // 0.1 s/px: un clip de 0.5 s mide 5 px, pero el caso que importa es el
        // de 2 px: el mínimo dibujado tiene que seguir siendo clicable.
        let vista = Vista::nueva(0.1);
        let agrupacion = agrupar_captions_por_carril(&proyecto.captions, &proyecto.hablantes);
        let diminuto = agrupacion
            .sin_asignar
            .iter()
            .find(|caption| caption.id == "cap-3")
            .expect("cap-3 está en el carril —");
        let x0 = vista.x_de(diminuto.inicio);
        assert_eq!(
            clip_en(&agrupacion.sin_asignar, vista, x0).map(|c| c.id.as_str()),
            Some("cap-3")
        );
    }

    /// Un evento del canvas, empaquetado como lo hace iced. Así se prueba el
    /// camino evento→mensaje sin GPU y sin robar el escritorio.

    #[test]
    fn pan_no_saca_la_ventana_del_video() {
        let vista = Vista::nueva(0.02);
        // Hacia atrás, ya en el principio: no hay segundos negativos.
        assert_eq!(vista.desplazar(-30.0, 15.0, ANCHO).inicio, 0.0);
        // Hacia adelante: el máximo deja el final del video pegado al borde.
        // 15 s de video en una ventana de 16 s: no hay nada que desplazar.
        assert_eq!(vista.desplazar(30.0, 15.0, ANCHO).inicio, 0.0);

        // Con más video que ventana (60 s), el tope es duracion - ventana.
        let ancha = vista.desplazar(100.0, 60.0, ANCHO);
        assert!(
            (ancha.inicio - 44.0).abs() < 1e-9,
            "inicio = {}",
            ancha.inicio
        );
        // Y la ventana mide lo mismo: desplazar no cambia la escala.
        assert_eq!(ancha.escala, vista.escala);
    }

    #[test]
    fn el_zoom_de_la_rueda_deja_quieto_el_punto_bajo_el_cursor() {
        let vista = Vista::nueva(0.02);
        let bajo_el_cursor = vista.tiempo_de(400.0);
        let acercada = vista.con_escala(vista.escala / 1.5, 400.0);
        assert!(acercada.escala < vista.escala);
        assert!(
            (acercada.tiempo_de(400.0) - bajo_el_cursor).abs() < 1e-9,
            "el tiempo bajo el cursor se movió: {} -> {}",
            bajo_el_cursor,
            acercada.tiempo_de(400.0)
        );
    }

    #[test]
    fn el_seguimiento_no_mueve_la_vista_mientras_el_playhead_esta_adentro() {
        let vista = Vista::nueva(0.02);
        assert_eq!(objetivo_de_seguimiento(vista, 5.0, 60.0, ANCHO), None);
        // Justo antes del disparador del borde derecho: quieto también.
        let casi = vista.ancho_visible(ANCHO) * EDGE_TRIGGER;
        assert_eq!(
            objetivo_de_seguimiento(vista, casi - 0.1, 60.0, ANCHO),
            None
        );
    }

    #[test]
    fn el_seguimiento_recoloca_el_playhead_al_8_por_ciento() {
        let vista = Vista::nueva(0.02);
        // Se busca hacia atrás, fuera de la ventana: vuelve al principio.
        assert_eq!(objetivo_de_seguimiento(vista, 0.0, 60.0, ANCHO), None);
        let movida = vista.con_inicio(20.0, 60.0, ANCHO);
        let retroceso = objetivo_de_seguimiento(movida, 18.0, 60.0, ANCHO)
            .expect("antes del inicio de la ventana hay que moverse");
        assert!(
            (retroceso - (18.0 - 16.0 * NEW_MARGIN)).abs() < 1e-9,
            "retroceso = {retroceso}"
        );

        // Cruzó el 92 % de la ventana (16 s): la nueva ventana empieza en
        // playhead - 16 * 0.08 = playhead - 1.28.
        let casi = vista.inicio + 16.0 * EDGE_TRIGGER + 0.1;
        let objetivo = objetivo_de_seguimiento(vista, casi, 60.0, ANCHO)
            .expect("pasado el disparador hay que moverse");
        assert!(
            (objetivo - (casi - 16.0 * NEW_MARGIN)).abs() < 1e-9,
            "objetivo = {objetivo}"
        );
    }

    #[test]
    fn alejarse_no_deja_la_ventana_en_el_vacio() {
        // 60 s de video, 16 s de ventana: el inicio máximo es 44.
        let vista = Vista::nueva(0.02).con_inicio(44.0, 60.0, ANCHO);
        assert_eq!(vista.inicio, 44.0);
        // A 0.04 s/px la ventana mide 32 s: el tope baja a 28. El zoom anclado
        // no lo sabe (solo hace la aritmética del ancla), así que la app tiene
        // que acotar después; si no, quedaría mirando el vacío desde 36.
        let alejada = vista.con_escala(0.04, ANCHO / 2.0);
        assert!(alejada.inicio > 28.0, "inicio = {}", alejada.inicio);
        let acotada = alejada.acotar(60.0, ANCHO);
        assert!(
            (acotada.inicio - 28.0).abs() < 1e-9,
            "inicio = {} (tope 28)",
            acotada.inicio
        );
        assert_eq!(acotada.escala, alejada.escala, "acotar no cambia el zoom");
    }

    #[test]
    fn encajar_muestra_el_video_entero() {
        let vista = Vista::nueva(0.02);
        let encajada = vista.encajar(15.0, 1500.0);
        assert!((encajada.escala - 0.01).abs() < 1e-12);
        assert_eq!(encajada.inicio, 0.0);
        assert!(
            (encajada.ancho_visible(1500.0) - 15.0).abs() < 1e-9,
            "la ventana mide {} s",
            encajada.ancho_visible(1500.0)
        );
        // Un ancho o una duración absurdos no rompen la vista.
        assert_eq!(vista.encajar(15.0, 0.0), vista);
        assert_eq!(vista.encajar(0.0, 1500.0), vista);
    }

    #[test]
    fn acercar_llega_exacto_y_para() {
        let mut vista = Vista::nueva(0.02);
        let objetivo = 10.0;
        let mut frames = 0;
        while acercar(&mut vista, objetivo, 60.0, ANCHO) {
            frames += 1;
            assert!(frames < 200, "no converge: inicio = {}", vista.inicio);
        }
        assert!(
            (vista.inicio - objetivo).abs() < 1e-9,
            "inicio = {}",
            vista.inicio
        );
        // 12 % por frame y corte en 2 ms: 10 s de recorrido tardan ~67 frames
        // (1,1 s a 60 Hz). Es el mismo cálculo que hace el oráculo.
        assert!(frames < 90, "tardó {frames} frames");
    }

    #[test]
    fn acercar_no_es_un_salto() {
        let mut vista = Vista::nueva(0.02);
        let objetivo = 10.0;
        acercar(&mut vista, objetivo, 60.0, ANCHO);
        let primer_paso = vista.inicio;
        assert!(
            primer_paso > 0.0 && primer_paso < objetivo * 0.2,
            "el primer frame fue un salto: {primer_paso}"
        );
    }
}
