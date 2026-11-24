//! Componentes compartidos. Ningún widget suelto en las vistas: tamaños,
//! colores y tipografía vienen siempre de `tema`.

use egui::{Color32, CornerRadius, FontId, Response, RichText, Sense, Stroke, Ui, Vec2};

use super::tema::{self, Estilo};
use crate::editor_app::Panel;

/// Iconos Phosphor (Regular) usados por la interfaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icono {
    Play,
    Pausa,
    Inicio,
    Fin,
    CuadroAnterior,
    CuadroSiguiente,
    Video,
    Carpeta,
    Mas,
    Menos,
    Cerrar,
    Minimizar,
    Maximizar,
    Ajustes,
    Recientes,
    Inicio2,
    Buscar,
    Papelera,
    Deshacer,
    Rehacer,
    Volumen,
    Bucle,
    Candado,
    Ojo,
    Mute,
    Solo,
    ChevronAbajo,
    Punto,
    Reloj,
    Subir,
    Flecha,
    Descargar,
    Guardar,
    Rayo,
}

impl Icono {
    pub const TODAS: &'static [Icono] = &[
        Icono::Play,
        Icono::Pausa,
        Icono::Inicio,
        Icono::Fin,
        Icono::CuadroAnterior,
        Icono::CuadroSiguiente,
        Icono::Video,
        Icono::Carpeta,
        Icono::Mas,
        Icono::Menos,
        Icono::Cerrar,
        Icono::Minimizar,
        Icono::Maximizar,
        Icono::Ajustes,
        Icono::Recientes,
        Icono::Inicio2,
        Icono::Buscar,
        Icono::Papelera,
        Icono::Deshacer,
        Icono::Rehacer,
        Icono::Volumen,
        Icono::Bucle,
        Icono::Candado,
        Icono::Ojo,
        Icono::Mute,
        Icono::Solo,
        Icono::ChevronAbajo,
        Icono::Punto,
        Icono::Reloj,
        Icono::Subir,
        Icono::Flecha,
        Icono::Descargar,
        Icono::Guardar,
        Icono::Rayo,
    ];
}

/// Glifo de la fuente Phosphor. Nunca un carácter unicode escrito a mano.
pub fn glifo(icono: Icono) -> &'static str {
    use egui_phosphor::regular as r;
    match icono {
        Icono::Play => r::PLAY,
        Icono::Pausa => r::PAUSE,
        Icono::Inicio => r::SKIP_BACK,
        Icono::Fin => r::SKIP_FORWARD,
        Icono::CuadroAnterior => r::REWIND,
        Icono::CuadroSiguiente => r::FAST_FORWARD,
        Icono::Video => r::VIDEO,
        Icono::Carpeta => r::FOLDER_SIMPLE,
        Icono::Mas => r::PLUS,
        Icono::Menos => r::MINUS,
        Icono::Cerrar => r::X,
        Icono::Minimizar => r::MINUS,
        Icono::Maximizar => r::SQUARE,
        Icono::Ajustes => r::GEAR,
        Icono::Recientes => r::CLOCK_COUNTER_CLOCKWISE,
        Icono::Inicio2 => r::HOUSE,
        Icono::Buscar => r::MAGNIFYING_GLASS,
        Icono::Papelera => r::TRASH,
        Icono::Deshacer => r::ARROW_COUNTER_CLOCKWISE,
        Icono::Rehacer => r::ARROW_CLOCKWISE,
        Icono::Volumen => r::SPEAKER_HIGH,
        Icono::Bucle => r::REPEAT,
        Icono::Candado => r::LOCK,
        Icono::Ojo => r::EYE,
        Icono::Mute => r::SPEAKER_SLASH,
        Icono::Solo => r::HEADPHONES,
        Icono::ChevronAbajo => r::CARET_DOWN,
        Icono::Punto => r::DOT,
        Icono::Reloj => r::CLOCK,
        Icono::Subir => r::UPLOAD,
        Icono::Flecha => r::ARROW_RIGHT,
        Icono::Descargar => r::DOWNLOAD,
        Icono::Guardar => r::FLOPPY_DISK,
        Icono::Rayo => r::LIGHTNING,
    }
}

/// Glifo suelto, sin caja ni fondo.
pub fn icono(ui: &mut Ui, icono: Icono) -> Response {
    ui.label(
        RichText::new(glifo(icono))
            .font(FontId::proportional(tema::TAMANO_ICONO))
            .color(tema::TEXTO_MENU),
    )
}

/// Botón de icono con hover, estado activo y tooltip con atajo opcional.
pub fn icon_button(
    ui: &mut Ui,
    icono: Icono,
    activo: bool,
    tooltip: &str,
    atajo: Option<&str>,
) -> Response {
    let tamaño = Vec2::splat(tema::ALTO_CHIP);
    let respuesta = ui.add(
        egui::Button::new(
            RichText::new(glifo(icono))
                .font(FontId::proportional(tema::TAMANO_ICONO))
                .color(if activo { tema::TEXTO } else { tema::TEXTO_2 }),
        )
        .fill(if activo {
            tema::CONTROL
        } else {
            Color32::TRANSPARENT
        })
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(tema::RADIO_CONTROL as u8))
        .min_size(tamaño),
    );
    if respuesta.hovered() && !activo {
        ui.painter().rect_filled(
            respuesta.rect,
            CornerRadius::same(tema::RADIO_CONTROL as u8),
            tema::ELEVADO,
        );
    }
    if respuesta.has_focus() {
        anillo_foco(ui, &respuesta);
    }
    respuesta.on_hover_ui(|ui| {
        ui.label(RichText::new(tooltip).font(tema::fuente(Estilo::Base)));
        if let Some(atajo) = atajo {
            ui.label(RichText::new(atajo).font(tema::fuente(Estilo::Mono)));
        }
    })
}

fn anillo_foco(ui: &mut Ui, respuesta: &Response) {
    ui.painter().rect_stroke(
        respuesta.rect.expand(1.0),
        CornerRadius::same(tema::RADIO_CONTROL as u8),
        Stroke::new(2.0_f32, tema::ACENTO),
        egui::StrokeKind::Outside,
    );
}

/// Botón principal: fondo primario, 36 de alto.
pub fn boton_primario(ui: &mut Ui, texto: &str, icono: Icono) -> Response {
    let respuesta = ui.add(
        egui::Button::new(
            RichText::new(format!("{}  {texto}", glifo(icono)))
                .font(tema::fuente(Estilo::Base))
                .color(Color32::WHITE),
        )
        .fill(tema::PRIMARIO)
        .corner_radius(CornerRadius::same(tema::RADIO_SUPERFICIE as u8))
        .min_size(Vec2::new(120.0, tema::ALTO_BOTON)),
    );
    if respuesta.has_focus() {
        anillo_foco(ui, &respuesta);
    }
    respuesta
}

/// Botón secundario: fondo de control y borde de un píxel.
pub fn boton_secundario(ui: &mut Ui, texto: &str, icono: Icono) -> Response {
    let respuesta = ui.add(
        egui::Button::new(
            RichText::new(format!("{}  {texto}", glifo(icono)))
                .font(tema::fuente(Estilo::Base))
                .color(tema::TEXTO),
        )
        .fill(tema::CONTROL)
        .stroke(Stroke::new(1.0_f32, tema::BORDE_CONTROL))
        .corner_radius(CornerRadius::same(tema::RADIO_SUPERFICIE as u8))
        .min_size(Vec2::new(120.0, tema::ALTO_BOTON)),
    );
    if respuesta.has_focus() {
        anillo_foco(ui, &respuesta);
    }
    respuesta
}

/// Cabecera de panel de 28 px con el título a la izquierda.
pub fn panel_header(ui: &mut Ui, titulo: &str) -> Response {
    ui.horizontal(|ui| {
        ui.set_height(tema::ALTO_CONTROL);
        ui.label(
            RichText::new(titulo)
                .font(tema::fuente(Estilo::Base))
                .color(tema::TEXTO_2),
        );
    })
    .response
}

/// Pestañas con subrayado de 2 px en la activa y contador opcional.
pub fn tab_bar(
    ui: &mut Ui,
    pestanas: &[(Panel, &str, Option<usize>)],
    activa: Panel,
) -> Option<Panel> {
    let mut elegida = None;
    ui.horizontal(|ui| {
        ui.set_height(tema::ALTO_NAV);
        ui.spacing_mut().item_spacing.x = tema::ESP_4;
        for (panel, texto, contador) in pestanas {
            let es_activa = *panel == activa;
            let respuesta = ui.add(
                egui::Button::new(
                    RichText::new(*texto)
                        .font(tema::fuente(Estilo::Base))
                        .color(if es_activa {
                            tema::TEXTO
                        } else {
                            tema::TEXTO_2
                        }),
                )
                .frame(false),
            );
            if let Some(n) = contador {
                ui.label(RichText::new(n.to_string()).font(tema::fuente(Estilo::Pequeno)));
            }
            if es_activa {
                let subrayado = egui::Rect::from_min_max(
                    egui::pos2(respuesta.rect.min.x, respuesta.rect.max.y - 2.0),
                    respuesta.rect.max,
                );
                ui.painter().rect_filled(subrayado, 0.0, tema::ACENTO);
            }
            if respuesta.has_focus() {
                anillo_foco(ui, &respuesta);
            }
            if respuesta.clicked() {
                elegida = Some(*panel);
            }
        }
    });
    elegida
}

/// Punto de color de hablante.
pub fn punto(ui: &mut Ui, color: Color32, diametro: f32) {
    let (_id, rect) = ui.allocate_space(Vec2::splat(diametro));
    ui.painter()
        .circle_filled(rect.center(), diametro / 2.0, color);
}

/// Chip con punto de color y nombre de hablante.
pub fn chip_hablante(ui: &mut Ui, nombre: &str, color: Color32) -> Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = tema::ESP_4;
        punto(ui, color, tema::ESP_8);
        ui.label(
            RichText::new(nombre)
                .font(tema::fuente(Estilo::Pequeno))
                .color(tema::TEXTO_2),
        );
    })
    .response
}

/// Campo de timecode de ancho fijo; en rojo si el valor no es válido.
pub fn campo_tiempo(ui: &mut Ui, valor: &mut String, invalido: bool) -> Response {
    let fuente = tema::fuente(Estilo::Timecode);
    let ancho = ui.fonts_mut(|f| f.glyph_width(&fuente, '0')) * 14.0;
    let respuesta = ui.add(
        egui::TextEdit::singleline(valor)
            .font(fuente)
            .desired_width(ancho)
            .text_color(if invalido { tema::PELIGRO } else { tema::TEXTO }),
    );
    if invalido {
        ui.painter().rect_stroke(
            respuesta.rect,
            CornerRadius::same(tema::RADIO_CONTROL as u8),
            Stroke::new(1.0_f32, tema::PELIGRO),
            egui::StrokeKind::Inside,
        );
    }
    respuesta
}

/// Campo de texto que crece con el contenido hasta `alto_max`.
pub fn campo_texto(ui: &mut Ui, valor: &mut String, alto_max: f32) -> Response {
    ui.set_max_height(alto_max);
    ui.add(
        egui::TextEdit::multiline(valor)
            .font(tema::fuente(Estilo::Cuerpo))
            .desired_rows(3),
    )
}

/// Recuadro punteado para estados vacíos.
pub fn recuadro_punteado(ui: &mut Ui, alto: f32, texto: &str) {
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), alto), egui::Sense::hover());
    // egui 0.33 no tiene trazo discontinuo: se dibuja a segmentos.
    let trazo = Stroke::new(1.5_f32, tema::BORDE_HOVER);
    let pintor = ui.painter();
    let dash = tema::ESP_8;
    let hueco = tema::ESP_4;
    let mut barrido = 0.0;
    while barrido < rect.width() {
        let fin = (barrido + dash).min(rect.width());
        pintor.line_segment(
            [
                egui::pos2(rect.min.x + barrido, rect.min.y),
                egui::pos2(rect.min.x + fin, rect.min.y),
            ],
            trazo,
        );
        pintor.line_segment(
            [
                egui::pos2(rect.min.x + barrido, rect.max.y),
                egui::pos2(rect.min.x + fin, rect.max.y),
            ],
            trazo,
        );
        barrido += dash + hueco;
    }
    barrido = 0.0;
    while barrido < rect.height() {
        let fin = (barrido + dash).min(rect.height());
        pintor.line_segment(
            [
                egui::pos2(rect.min.x, rect.min.y + barrido),
                egui::pos2(rect.min.x, rect.min.y + fin),
            ],
            trazo,
        );
        pintor.line_segment(
            [
                egui::pos2(rect.max.x, rect.min.y + barrido),
                egui::pos2(rect.max.x, rect.min.y + fin),
            ],
            trazo,
        );
        barrido += dash + hueco;
    }
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.centered_and_justified(|ui| {
            ui.label(
                egui::RichText::new(texto)
                    .font(tema::fuente(Estilo::Base))
                    .color(tema::TEXTO_3),
            );
        });
    });
}

/// Tarjeta con superficie, borde y radio del tema.
pub fn tarjeta<R>(ui: &mut Ui, contenido: impl FnOnce(&mut Ui) -> R) -> (Response, R) {
    let marco = tema::panel(tema::PANEL).show(ui, contenido);
    (marco.response, marco.inner)
}

/// Logotipo OpenDialogue: dos burbujas superpuestas, calcado del SVG que
/// aportó el usuario. Se dibuja con shapes para no depender de un decoder SVG.
pub fn logo(ui: &mut Ui, tamaño: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(tamaño), Sense::hover());
    let pintor = ui.painter();
    let escala = tamaño / 512.0;
    let punto = |x: f32, y: f32| egui::pos2(rect.min.x + x * escala, rect.min.y + y * escala);
    let fondo = tema::color("#232323");
    let ambar = tema::color("#FFB020");
    let violeta = tema::color("#8F6BFF");

    pintor.circle_filled(rect.center(), tamaño * 0.5, fondo);
    let radio_ambar = egui::CornerRadius::same((64.0 * escala).round() as u8);
    pintor.rect_filled(
        egui::Rect::from_min_max(punto(85.0, 96.0), punto(330.0, 277.0)),
        radio_ambar,
        ambar,
    );
    pintor.add(egui::Shape::convex_polygon(
        vec![
            punto(128.0, 256.0),
            punto(107.0, 336.0),
            punto(197.0, 267.0),
        ],
        ambar,
        egui::Stroke::NONE,
    ));
    let radio_violeta = egui::CornerRadius::same((64.0 * escala).round() as u8);
    let borde = egui::Stroke::new(16.0 * escala, fondo);
    pintor.rect(
        egui::Rect::from_min_max(punto(181.0, 192.0), punto(426.0, 373.0)),
        radio_violeta,
        violeta,
        borde,
        egui::StrokeKind::Inside,
    );
    pintor.add(egui::Shape::convex_polygon(
        vec![
            punto(384.0, 352.0),
            punto(405.0, 432.0),
            punto(315.0, 363.0),
        ],
        violeta,
        borde,
    ));
}

/// Separador de un píxel.
pub fn separador(ui: &mut Ui) {
    ui.add(egui::Separator::default().spacing(tema::ESP_4));
}
