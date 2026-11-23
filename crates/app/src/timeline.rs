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

use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Program, Text};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme, mouse};
use rod_core::{Caption, Proyecto, agrupar_captions_por_carril};

use crate::tema;

/// Altura de la regla de tiempo (arriba del todo).
pub const ALTO_REGLA: f32 = 22.0;
/// Altura de cada carril.
pub const ALTO_CARRIL: f32 = 34.0;
/// Separación mínima entre marcas de la regla, en píxeles.
const PASO_MINIMO_PX: f32 = 70.0;

/// Cuánto se desplaza un `ScrollDelta::Pixels`: en Windows el wheel entrega
/// líneas, pero un trackpad entrega píxeles y hay que homogeneizarlos.
const PX_POR_LINEA: f32 = 40.0;

// Los tres números del seguimiento vienen del oráculo (`utils/constants.ts`):
// cuando el playhead cruza el 92 % de la ventana, la ventana se recoloca para
// dejarlo al 8 % del borde izquierdo, y el desplazamiento se interpola un 12 %
// por frame para que no salte.
pub const EDGE_TRIGGER: f64 = 0.92;
pub const NEW_MARGIN: f64 = 0.08;
pub const LERP: f64 = 0.12;

const FONDO: Color = tema::SUPERFICIE;
const SEPARADOR: Color = tema::HAIRLINE;
const TEXTO: Color = tema::SECUNDARIO;
const PLAYHEAD: Color = tema::MOSTAZA;
const CARRIL_VACIO: Color = Color::from_rgb(0.28, 0.30, 0.34);
/// Paleta de carriles mientras no hay presets: el color del `.ass` es del
/// hablante (slice 7), aquí es solo una ayuda para distinguir carriles.
const PALETA: [Color; 6] = [
    Color::from_rgb(0.31, 0.49, 0.91),
    Color::from_rgb(0.91, 0.63, 0.31),
    Color::from_rgb(0.31, 0.91, 0.63),
    Color::from_rgb(0.91, 0.36, 0.31),
    Color::from_rgb(0.63, 0.31, 0.91),
    Color::from_rgb(0.31, 0.80, 0.91),
];

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
    Pulsar(Point),
    Arrastrar(Point),
    Soltar(Point),
    /// La rueda giró `lineas` sobre `punto`: desplazar o acercar lo decide la
    /// app (Shift = zoom).
    Rueda {
        lineas: f32,
        punto: Point,
    },
    /// El widget cambió de ancho. La app lo necesita para saber cuántos
    /// segundos caben y hasta dónde puede desplazar la ventana.
    Ancho(f32),
}

pub struct Linea<'a> {
    pub proyecto: &'a Proyecto,
    pub vista: Vista,
    pub duracion: f64,
    pub position: f64,
    pub seleccion: &'a [String],
    /// El arrastre en curso, solo para pintar la marquesina: el resto del
    /// arrastre ya está aplicado al proyecto.
    pub arrastre: Option<&'a Arrastre>,
}

/// Un carril listo para pintar: su etiqueta y sus clips.
struct Carril {
    etiqueta: String,
    color: Color,
    captions: Vec<Caption>,
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

impl Linea<'_> {
    /// Carril 0 = `—` (sin hablante); después, un carril por hablante en el
    /// orden del proyecto, igual que `caption_row_index`.
    fn carriles(&self) -> Vec<Carril> {
        let agrupacion =
            agrupar_captions_por_carril(&self.proyecto.captions, &self.proyecto.hablantes);
        let mut carriles = vec![Carril {
            etiqueta: "—".into(),
            color: CARRIL_VACIO,
            captions: agrupacion.sin_asignar,
        }];
        for (indice, hablante) in self.proyecto.hablantes.iter().enumerate() {
            let etiqueta = if hablante.nombre.is_empty() {
                hablante.tecla.clone()
            } else {
                hablante.nombre.clone()
            };
            carriles.push(Carril {
                etiqueta,
                color: PALETA[indice % PALETA.len()],
                captions: agrupacion
                    .por_hablante
                    .get(&hablante.id)
                    .cloned()
                    .unwrap_or_default(),
            });
        }
        carriles
    }
}

impl Program<Mensaje> for Linea<'_> {
    /// Último ancho publicado. El widget sabe cuánto mide en cada layout; la
    /// app necesita ese número para el pan y el seguimiento, y aquí solo se
    /// avisa cuando cambia (si no, sería un mensaje por frame).
    type State = Option<f32>;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Mensaje>> {
        // Los eventos de ventana (incluido el redibujado) no traen ratón: son
        // el único sitio donde el widget se entera de que cambió de tamaño sin
        // esperar a que el usuario lo toque.
        if matches!(event, canvas::Event::Window(_)) {
            let ancho = bounds.width;
            if *state != Some(ancho) {
                *state = Some(ancho);
                return Some(canvas::Action::publish(Mensaje::Ancho(ancho)));
            }
            return None;
        }

        let canvas::Event::Mouse(evento) = event else {
            return None;
        };

        match evento {
            mouse::Event::ButtonPressed(mouse::Button::Left) => {
                let punto = cursor.position_in(bounds)?;
                Some(canvas::Action::publish(Mensaje::Pulsar(punto)).and_capture())
            }
            mouse::Event::ButtonReleased(mouse::Button::Left) => {
                let punto = cursor.position_in(bounds)?;
                Some(canvas::Action::publish(Mensaje::Soltar(punto)).and_capture())
            }
            // Sin arrastre en curso, mover el ratón no le importa a nadie; con
            // uno, cada movimiento es un frame del arrastre.
            mouse::Event::CursorMoved { .. } => {
                self.arrastre?;
                let punto = cursor.position_in(bounds)?;
                Some(canvas::Action::publish(Mensaje::Arrastrar(punto)))
            }
            // Fuera del widget no hay nada que desplazar: el `?` es la
            // diferencia entre girar la rueda sobre el timeline y girarla
            // encima de los paneles.
            mouse::Event::WheelScrolled { delta } => {
                let lineas = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / PX_POR_LINEA,
                };
                let punto = cursor.position_in(bounds)?;
                Some(canvas::Action::publish(Mensaje::Rueda { lineas, punto }).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), FONDO);

        self.pintar_regla(&mut frame, bounds.size());
        self.pintar_carriles(&mut frame, bounds.size());
        self.pintar_fin(&mut frame, bounds.size());
        self.pintar_playhead(&mut frame, bounds.size());
        self.pintar_marquesina(&mut frame, bounds, cursor);

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        let Some(punto) = cursor.position_in(bounds) else {
            return mouse::Interaction::default();
        };
        let carriles = self.carriles();
        let sobre_clip = carril_en(punto.y)
            .and_then(|carril| carriles.get(carril))
            .is_some_and(|carril| clip_en(&carril.captions, self.vista, punto.x).is_some());
        if sobre_clip {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::Crosshair
        }
    }
}

impl Linea<'_> {
    fn pintar_regla(&self, frame: &mut Frame, tamano: Size) {
        let paso = paso_regla(self.vista.escala);
        let desde = (self.vista.inicio / paso).floor() as i64;
        let hasta = (self.vista.tiempo_de(tamano.width) / paso).ceil() as i64;

        for indice in desde..=hasta {
            let tiempo = indice as f64 * paso;
            let x = self.vista.x_de(tiempo);
            frame.fill_rectangle(
                Point::new(x, ALTO_REGLA - 6.0),
                Size::new(1.0, 6.0),
                SEPARADOR,
            );
            frame.fill_text(Text {
                content: rod_core::format_time(tiempo),
                position: Point::new(x + 4.0, 2.0),
                color: TEXTO,
                size: iced::Pixels(11.0),
                ..Text::default()
            });
        }
        frame.fill_rectangle(
            Point::new(0.0, ALTO_REGLA - 1.0),
            Size::new(tamano.width, 1.0),
            SEPARADOR,
        );
    }

    fn pintar_carriles(&self, frame: &mut Frame, tamano: Size) {
        for (indice, carril) in self.carriles().iter().enumerate() {
            let y = y_de_carril(indice);
            if y > tamano.height {
                break;
            }
            // Fondo del carril y su nombre, fijo a la izquierda.
            frame.fill_rectangle(
                Point::new(0.0, y),
                Size::new(tamano.width, ALTO_CARRIL - 1.0),
                Color {
                    a: 0.04,
                    ..carril.color
                },
            );
            frame.fill_text(Text {
                content: carril.etiqueta.clone(),
                position: Point::new(5.0, y + 9.0),
                color: carril.color,
                size: iced::Pixels(12.0),
                ..Text::default()
            });

            for caption in &carril.captions {
                let x0 = self.vista.x_de(caption.inicio);
                let x1 = self.vista.x_de(caption.fin);
                if x1 < 0.0 || x0 > tamano.width {
                    continue;
                }
                let seleccionado = self.seleccion.iter().any(|id| id == &caption.id);
                let color = if seleccionado {
                    Color::WHITE
                } else {
                    carril.color
                };
                let rect = Rectangle {
                    x: x0,
                    y: y + 4.0,
                    width: (x1 - x0).max(2.0),
                    height: ALTO_CARRIL - 9.0,
                };
                frame.fill_rectangle(rect.position(), rect.size(), color);
                frame.fill_text(Text {
                    content: caption.texto.clone(),
                    position: Point::new(rect.x + 4.0, rect.y + 3.0),
                    color: Color::from_rgb(0.05, 0.06, 0.08),
                    size: iced::Pixels(11.0),
                    ..Text::default()
                });
            }

            frame.fill_rectangle(
                Point::new(0.0, y + ALTO_CARRIL - 1.0),
                Size::new(tamano.width, 1.0),
                SEPARADOR,
            );
        }
    }

    fn pintar_fin(&self, frame: &mut Frame, tamano: Size) {
        let x = self.vista.x_de(self.duracion);
        if !(0.0..=tamano.width).contains(&x) {
            return;
        }
        frame.fill_rectangle(
            Point::new(x, ALTO_REGLA),
            Size::new(1.0, tamano.height - ALTO_REGLA),
            TEXTO,
        );
    }

    /// El rectángulo de la marquesina en curso. Se dibuja del origen al cursor
    /// de ahora mismo: el `Arrastre` solo guarda el origen, y el cursor lo trae
    /// `draw`.
    fn pintar_marquesina(&self, frame: &mut Frame, bounds: Rectangle, cursor: mouse::Cursor) {
        let Some(Modo::Marquesina { origen }) = self.arrastre.map(|arrastre| &arrastre.modo) else {
            return;
        };
        let Some(actual) = cursor.position_in(bounds) else {
            return;
        };
        let rect = Rectangle::new(
            Point::new(origen.x.min(actual.x), origen.y.min(actual.y)),
            Size::new(
                (actual.x - origen.x).abs().max(1.0),
                (actual.y - origen.y).abs().max(1.0),
            ),
        );
        frame.fill_rectangle(
            rect.position(),
            rect.size(),
            Color {
                a: 0.12,
                ..PLAYHEAD
            },
        );
        frame.stroke_rectangle(
            rect.position(),
            rect.size(),
            canvas::Stroke::default()
                .with_color(PLAYHEAD)
                .with_width(1.0),
        );
    }

    fn pintar_playhead(&self, frame: &mut Frame, tamano: Size) {
        let x = self.vista.x_de(self.position);
        if !(0.0..=tamano.width).contains(&x) {
            return;
        }
        frame.fill_rectangle(Point::new(x, 0.0), Size::new(1.5, tamano.height), PLAYHEAD);
        let triangulo = Path::new(|builder| {
            builder.move_to(Point::new(x - 5.0, 0.0));
            builder.line_to(Point::new(x + 6.5, 0.0));
            builder.line_to(Point::new(x + 0.75, 6.0));
            builder.close();
        });
        frame.fill(&triangulo, PLAYHEAD);
    }
}

/// Paso de la regla en segundos: el menor de una lista "redonda" que deje al
/// menos [`PASO_MINIMO_PX`] entre marcas. Sin esto, al hacer zoom las marcas se
/// solaparían o quedarían a un píxel.
pub fn paso_regla(escala: f64) -> f64 {
    const PASOS: [f64; 12] = [
        0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0, 600.0, 1800.0,
    ];
    let minimo = f64::from(PASO_MINIMO_PX) * escala;
    PASOS
        .into_iter()
        .find(|paso| *paso >= minimo)
        .unwrap_or(PASOS[PASOS.len() - 1])
}

/// Atajo para la app: el widget ya configurado para llenar su contenedor.
pub fn widget<'a>(linea: Linea<'a>) -> Canvas<Linea<'a>, Mensaje> {
    Canvas::new(linea)
}

#[cfg(test)]
mod tests {
    use super::{
        ALTO_CARRIL, ALTO_REGLA, Arrastre, EDGE_TRIGGER, Linea, Mensaje, Modo, NEW_MARGIN, Vista,
        acercar, carril_en, clip_en, fila_de_carril, objetivo_de_seguimiento, paso_regla,
    };
    use iced::widget::canvas;
    use iced::{Point, Rectangle, Size, mouse};
    use rod_core::{Caption, Hablante, Proyecto};

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

    fn linea(proyecto: &Proyecto, vista: Vista) -> Linea<'_> {
        Linea {
            proyecto,
            vista,
            duracion: 15.0,
            position: 0.0,
            seleccion: &[],
            arrastre: None,
        }
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
    fn la_regla_usa_pasos_redondos_y_nunca_amontona_marcas() {
        for escala in [0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.5] {
            let paso = paso_regla(escala);
            let separacion = paso / escala;
            assert!(
                separacion >= f64::from(super::PASO_MINIMO_PX),
                "escala {escala}: {separacion} px entre marcas"
            );
        }
        assert_eq!(paso_regla(0.02), 2.0);
        assert_eq!(paso_regla(0.002), 0.5);
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
        let linea = linea(&proyecto, Vista::nueva(0.02));
        let carriles = linea.carriles();

        assert_eq!(carriles.len(), 2, "carril — y el de Ana");
        assert_eq!(carriles[0].etiqueta, "—");
        assert_eq!(carriles[0].captions.len(), 1);
        assert_eq!(carriles[1].etiqueta, "Ana");
        assert_eq!(carriles[1].captions[0].id, "cap-1");
    }

    #[test]
    fn el_clic_encuentra_el_clip_que_esta_debajo() {
        let proyecto = proyecto();
        let vista = Vista::nueva(0.02);
        let linea = linea(&proyecto, vista);
        let carriles = linea.carriles();

        // cap-1 va de 1 s a 3 s: en x, de 50 a 150 px.
        let encontrado = clip_en(&carriles[1].captions, vista, 100.0);
        assert_eq!(encontrado.map(|caption| caption.id.as_str()), Some("cap-1"));
        assert!(clip_en(&carriles[1].captions, vista, 10.0).is_none());
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
        let linea = linea(&proyecto, vista);
        let carriles = linea.carriles();
        let diminuto = carriles[0]
            .captions
            .iter()
            .find(|caption| caption.id == "cap-3")
            .expect("cap-3 está en el carril —");
        let x0 = vista.x_de(diminuto.inicio);
        assert_eq!(
            clip_en(&carriles[0].captions, vista, x0).map(|c| c.id.as_str()),
            Some("cap-3")
        );
    }

    /// Un evento del canvas, empaquetado como lo hace iced. Así se prueba el
    /// camino evento→mensaje sin GPU y sin robar el escritorio.
    fn evento(linea: &Linea<'_>, evento: mouse::Event, x: f32, y: f32) -> Option<Mensaje> {
        use super::Program;

        let mut estado = None;
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 300.0));
        let cursor = mouse::Cursor::Available(Point::new(x, y));
        linea
            .update(&mut estado, &canvas::Event::Mouse(evento), bounds, cursor)
            .and_then(|accion| accion.into_inner().0)
    }

    /// Un evento de ventana, que es donde el widget aprende su ancho.
    fn evento_de_ventana(
        linea: &Linea<'_>,
        estado: &mut Option<f32>,
        ancho: f32,
    ) -> Option<Mensaje> {
        use super::Program;

        let bounds = Rectangle::new(Point::ORIGIN, Size::new(ancho, 300.0));
        let evento = canvas::Event::Window(iced::window::Event::RedrawRequested(
            std::time::Instant::now(),
        ));
        linea
            .update(estado, &evento, bounds, mouse::Cursor::Unavailable)
            .and_then(|accion| accion.into_inner().0)
    }

    #[test]
    fn el_ancho_se_publica_una_vez_por_cambio() {
        let proyecto = proyecto();
        let linea = linea(&proyecto, Vista::nueva(0.02));
        let mut estado = None;

        // Primera vez: la app no sabía cuánto mide.
        assert_eq!(
            evento_de_ventana(&linea, &mut estado, 800.0),
            Some(Mensaje::Ancho(800.0))
        );
        // Sin cambio de ancho no hay mensaje: si no, sería uno por frame.
        assert_eq!(evento_de_ventana(&linea, &mut estado, 800.0), None);
        assert_eq!(
            evento_de_ventana(&linea, &mut estado, 640.0),
            Some(Mensaje::Ancho(640.0))
        );
    }

    #[test]
    fn la_rueda_llega_con_lineas_y_punto_local() {
        let proyecto = proyecto();
        let linea = linea(&proyecto, Vista::nueva(0.02));
        assert_eq!(
            evento(
                &linea,
                mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 }
                },
                300.0,
                40.0
            ),
            Some(Mensaje::Rueda {
                lineas: 1.0,
                punto: Point::new(300.0, 40.0)
            })
        );
        // Un trackpad entrega píxeles; se homogeneizan a líneas.
        assert_eq!(
            evento(
                &linea,
                mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels { x: 0.0, y: 80.0 }
                },
                300.0,
                40.0
            ),
            Some(Mensaje::Rueda {
                lineas: 2.0,
                punto: Point::new(300.0, 40.0)
            })
        );
    }

    #[test]
    fn la_pulsacion_llega_a_la_app_como_un_punto_local() {
        let proyecto = proyecto();
        let linea = linea(&proyecto, Vista::nueva(0.02));
        assert_eq!(
            evento(
                &linea,
                mouse::Event::ButtonPressed(mouse::Button::Left),
                100.0,
                5.0
            ),
            Some(Mensaje::Pulsar(Point::new(100.0, 5.0)))
        );
        assert_eq!(
            evento(
                &linea,
                mouse::Event::ButtonReleased(mouse::Button::Left),
                120.0,
                5.0
            ),
            Some(Mensaje::Soltar(Point::new(120.0, 5.0)))
        );
    }

    #[test]
    fn sin_arrastre_mover_el_raton_no_molesta_a_la_app() {
        let proyecto = proyecto();
        let linea = linea(&proyecto, Vista::nueva(0.02));
        assert_eq!(
            evento(
                &linea,
                mouse::Event::CursorMoved {
                    position: Point::ORIGIN
                },
                50.0,
                50.0
            ),
            None,
            "sin arrastre en curso no hay nada que contar"
        );
    }

    #[test]
    fn con_arrastre_cada_movimiento_se_publica() {
        let proyecto = proyecto();
        let mut linea = linea(&proyecto, Vista::nueva(0.02));
        let arrastre = Arrastre {
            modo: Modo::Marquesina {
                origen: Point::new(40.0, 60.0),
            },
            respaldo: None,
            con_historial: false,
        };
        linea.arrastre = Some(&arrastre);
        assert_eq!(
            evento(
                &linea,
                mouse::Event::CursorMoved {
                    position: Point::ORIGIN
                },
                90.0,
                70.0
            ),
            Some(Mensaje::Arrastrar(Point::new(90.0, 70.0)))
        );
    }

    /// 800 px a 0.02 s/px = 16 s de ventana; el clip de prueba dura 15 s.
    const ANCHO: f32 = 800.0;

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
