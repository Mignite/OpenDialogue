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

/// Altura de la regla de tiempo (arriba del todo).
pub const ALTO_REGLA: f32 = 22.0;
/// Altura de cada carril.
pub const ALTO_CARRIL: f32 = 34.0;
/// Separación mínima entre marcas de la regla, en píxeles.
const PASO_MINIMO_PX: f32 = 70.0;

const FONDO: Color = Color::from_rgb(0.03, 0.04, 0.06);
const SEPARADOR: Color = Color::from_rgb(0.14, 0.16, 0.20);
const TEXTO: Color = Color::from_rgb(0.62, 0.66, 0.74);
const PLAYHEAD: Color = Color::from_rgb(0.95, 0.36, 0.32);
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
}

/// Lo que el timeline le pide a la app. Son puntos locales del widget y nada
/// más: el significado (buscar, agarrar un borde, mover un bloque) lo decide
/// [`arrastre::pulsar`] con el proyecto delante.
#[derive(Debug, Clone, PartialEq)]
pub enum Mensaje {
    Pulsar(Point),
    Arrastrar(Point),
    Soltar(Point),
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
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Mensaje>> {
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
        ALTO_CARRIL, ALTO_REGLA, Arrastre, Linea, Mensaje, Modo, Vista, carril_en, clip_en,
        fila_de_carril, paso_regla,
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

        let mut estado = ();
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 300.0));
        let cursor = mouse::Cursor::Available(Point::new(x, y));
        linea
            .update(&mut estado, &canvas::Event::Mouse(evento), bounds, cursor)
            .and_then(|accion| accion.into_inner().0)
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
}
