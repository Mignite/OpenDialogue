//! Única fuente de color, radio, espacio y tamaño de la interfaz.
//!
//! Los valores son los del mockup `OpenDialogue UI.html`. Ningún otro módulo de
//! `ui/` debe escribir un hex, un radio o un tamaño literal.

use egui::{Color32, CornerRadius, FontId, Frame, Margin, Stroke, Style, TextStyle, Visuals};

use super::tipografia;

// ── Paleta ──────────────────────────────────────────────────────────────────

pub const FONDO: Color32 = Color32::from_rgb(0x1A, 0x1A, 0x1A);
pub const RIEL: Color32 = Color32::from_rgb(0x1F, 0x1F, 0x1F);
pub const PANEL: Color32 = Color32::from_rgb(0x23, 0x23, 0x23);
pub const ACTIVO: Color32 = Color32::from_rgb(0x26, 0x26, 0x26);
pub const CONTROL: Color32 = Color32::from_rgb(0x2C, 0x2C, 0x2C);
pub const ELEVADO: Color32 = Color32::from_rgb(0x33, 0x33, 0x33);
pub const BORDE: Color32 = Color32::from_rgb(0x2E, 0x2E, 0x2E);
pub const BORDE_CONTROL: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x3A);
pub const BORDE_HOVER: Color32 = Color32::from_rgb(0x4A, 0x4A, 0x4A);
pub const TEXTO: Color32 = Color32::from_rgb(0xE6, 0xE6, 0xE6);
pub const TEXTO_MENU: Color32 = Color32::from_rgb(0xCF, 0xCF, 0xCF);
pub const TEXTO_2: Color32 = Color32::from_rgb(0xA0, 0xA0, 0xA0);
pub const TEXTO_3: Color32 = Color32::from_rgb(0x92, 0x92, 0x92);
/// Solo foco, selección, playhead y marca activa.
pub const ACENTO: Color32 = Color32::from_rgb(0x3B, 0x8E, 0xFF);
pub const PRIMARIO: Color32 = Color32::from_rgb(0x27, 0x66, 0xDB);
pub const PRIMARIO_HOVER: Color32 = Color32::from_rgb(0x2F, 0x72, 0xE8);
pub const PELIGRO: Color32 = Color32::from_rgb(0xC4, 0x2B, 0x1C);

// Alias que usan las vistas todavía sin reescribir. Se retiran cuando cada
// vista pase a los nombres de la spec.
pub const DIM: Color32 = TEXTO_3;
pub const SECUNDARIO: Color32 = TEXTO_2;
pub const LATERAL: Color32 = RIEL;
pub const SUPERFICIE: Color32 = FONDO;
pub const AMBAR: Color32 = Color32::from_rgb(0xE5, 0xA9, 0x3B);

// ── Radios, espacios y alturas ──────────────────────────────────────────────

pub const RADIO_CONTROL: f32 = 4.0;
pub const RADIO_SUPERFICIE: f32 = 6.0;

pub const ESP_2: f32 = 2.0;
pub const ESP_4: f32 = 4.0;
pub const ESP_8: f32 = 8.0;
pub const ESP_12: f32 = 12.0;
pub const ESP_16: f32 = 16.0;
pub const ESP_20: f32 = 20.0;
pub const ESP_24: f32 = 24.0;
pub const ESP_32: f32 = 32.0;
pub const ESP_40: f32 = 40.0;
pub const ESP_48: f32 = 48.0;
/// Espacio extra entre letras, en ems (0.04 em para etiquetas).
pub const LETRA_ESPACIADO: f32 = 0.04;
pub const ANCHO_CONTENEDOR: f32 = 960.0;
pub const ANCHO_TARJETA: f32 = 260.0;
pub const ALTO_ZONA: f32 = 96.0;
pub const ALTO_BARRA_COLOR: f32 = 4.0;
pub const TAMANO_PUNTO: f32 = 8.0;
pub const TAMANO_ICONO_GRANDE: f32 = 20.0;
pub const TAMANO_TITULO: f32 = 32.0;
pub const ANCHO_SUBRAYADO: f32 = 72.0;
pub const ANCHO_FILTRO: f32 = 160.0;
pub const ALTO_BOTON_CHICO: f32 = 28.0;
pub const GROSOR_PUNTEADO: f32 = 1.5;
/// Barra vertical de las pistas del timeline.
pub const ANCHO_SCROLLBAR: f32 = 10.0;
pub const ANCHO_PANEL_DEFECTO: f32 = 320.0;
pub const ANCHO_PANEL_MIN: f32 = 280.0;
pub const ANCHO_PANEL_MAX: f32 = 420.0;
// Editor: medidas del transporte, la barra de texto y la timeline.
pub const ALTO_TRANSPORTE: f32 = 56.0;
pub const ALTO_BARRA_TEXTO: f32 = 32.0;
pub const ANCHO_MINIMO_TEXTO: f32 = 280.0;
pub const ALTO_TOOLBAR_TIMELINE: f32 = 32.0;
pub const ANCHO_BOTON_28: f32 = 28.0;
pub const TAMANO_ICONO_16: f32 = 16.0;
pub const GROSOR_SPLITTER: f32 = 4.0;
pub const ANCHO_DROPDOWN_VELOCIDAD: f32 = 56.0;
pub const ANCHO_SLIDER_ZOOM: f32 = 72.0;
// Estilos: banda de preview, cabecera y controles a todo el ancho.
pub const ALTO_PREVIEW_STYLES: f32 = 64.0;
pub const ALTO_CABECERA_PRESET: f32 = 36.0;
pub const COLOR_PREVIEW_STYLES: Color32 = Color32::from_rgb(0x14, 0x14, 0x14);
pub const TAMANO_PUNTO_ESTILO: f32 = 14.0;
pub const ANCHO_DRAG_TAMANO: f32 = 96.0;
pub const ANCHO_DRAG_CONTORNO: f32 = 80.0;
pub const ALTO_PISTA_AUDIO: f32 = 56.0;
pub const RADIO_CLIP: u8 = 3;
pub const COLOR_FONDO_VIDEO: Color32 = Color32::from_rgb(0x0F, 0x0F, 0x0F);
pub const COLOR_BORDE: Color32 = Color32::from_rgb(0x2E, 0x2E, 0x2E);
pub const COLOR_HOVER: Color32 = Color32::from_rgb(0x2C, 0x2C, 0x2C);
pub const COLOR_SIN_HABLANTE: Color32 = Color32::from_rgb(0x66, 0x66, 0x66);
pub const COLOR_WAVEFORM: Color32 = Color32::from_rgb(0x3A, 0x4A, 0x5A);
pub const ALTO_TABS: f32 = 36.0;
pub const SEPARACION_TABS: f32 = 16.0;
pub const ALTO_TARJETA_HABLANTE: f32 = 76.0;
pub const TAMANO_PUNTO_GRANDE: f32 = 12.0;
pub const GROSOR_BARRA_HABLANTE: f32 = 3.0;
/// Fondo de la fila seleccionada en la lista de subtítulos.
pub const SELECCION_FILA: Color32 = Color32::from_rgb(0x2C, 0x3A, 0x52);
pub const CPS_MAXIMO: f32 = 17.0;
pub const CARACTERES_MAXIMOS: usize = 42;
/// Fondo de la zona de arrastre.
pub const FONDO_ZONA: Color32 = Color32::from_rgb(0x1E, 0x1E, 0x1E);
/// Duración de las transiciones de hover.
pub const ANIMACION: f32 = 0.14;

/// Interpolación lineal de colores para transiciones cortas.
pub fn mezclar(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let canal = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(
        canal(a.r(), b.r()),
        canal(a.g(), b.g()),
        canal(a.b(), b.b()),
        canal(a.a(), b.a()),
    )
}

pub fn radio_superficie() -> egui::CornerRadius {
    egui::CornerRadius::same(RADIO_SUPERFICIE as u8)
}

pub fn radio_control() -> egui::CornerRadius {
    egui::CornerRadius::same(RADIO_CONTROL as u8)
}
pub const ESP_72: f32 = 72.0;

pub const ALTO_CHIP: f32 = 24.0;
pub const ALTO_CONTROL: f32 = 28.0;
pub const ALTO_NAV: f32 = 32.0;
pub const ALTO_BOTON: f32 = 36.0;
pub const ALTO_CABECERA: f32 = 40.0;
pub const ANCHO_BOTON_VENTANA: f32 = 46.0;
pub const ALTO_FILA_SUB: f32 = 44.0;
/// Iconos Phosphor: glifo de 16 px en rejilla de 24.
pub const TAMANO_ICONO: f32 = 16.0;

// ── Tipografía ──────────────────────────────────────────────────────────────

/// Estilos con nombre. Ninguno baja de 11 px.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Estilo {
    Titulo,
    Subtitulo,
    Cuerpo,
    CuerpoFuerte,
    Base,
    Pequeno,
    Etiqueta,
    Mono,
    Timecode,
    TimecodeGrande,
}

impl Estilo {
    pub const TODAS: &'static [Estilo] = &[
        Estilo::Titulo,
        Estilo::Subtitulo,
        Estilo::Cuerpo,
        Estilo::CuerpoFuerte,
        Estilo::Base,
        Estilo::Pequeno,
        Estilo::Etiqueta,
        Estilo::Mono,
        Estilo::Timecode,
        Estilo::TimecodeGrande,
    ];

    /// Clave con la que se registra en `Style::text_styles`.
    pub fn nombre(self) -> &'static str {
        match self {
            Estilo::Titulo => "Titulo",
            Estilo::Subtitulo => "Subtitulo",
            Estilo::Cuerpo => "Cuerpo",
            Estilo::CuerpoFuerte => "CuerpoFuerte",
            Estilo::Base => "Base",
            Estilo::Pequeno => "Pequeno",
            Estilo::Etiqueta => "Etiqueta",
            Estilo::Mono => "Mono",
            Estilo::Timecode => "Timecode",
            Estilo::TimecodeGrande => "TimecodeGrande",
        }
    }
}

/// Fuente correspondiente a cada estilo. La mono es solo para cifras y atajos.
pub fn fuente(estilo: Estilo) -> FontId {
    use tipografia::Peso;
    match estilo {
        Estilo::Titulo => FontId::new(24.0, tipografia::familia(Peso::Medium)),
        Estilo::Subtitulo => FontId::new(16.0, tipografia::familia(Peso::Medium)),
        Estilo::Cuerpo => FontId::new(13.0, tipografia::familia(Peso::Regular)),
        Estilo::CuerpoFuerte => FontId::new(13.0, tipografia::familia(Peso::Medium)),
        Estilo::Base => FontId::new(12.0, tipografia::familia(Peso::Regular)),
        Estilo::Pequeno => FontId::new(11.0, tipografia::familia(Peso::Regular)),
        Estilo::Etiqueta => FontId::new(11.0, tipografia::familia(Peso::Medium)),
        Estilo::Mono => FontId::new(11.0, tipografia::familia(Peso::Mono)),
        Estilo::Timecode => FontId::new(13.0, tipografia::familia(Peso::Mono)),
        Estilo::TimecodeGrande => FontId::new(20.0, tipografia::familia(Peso::Mono)),
    }
}

/// Mono de 11 px, para las vistas que todavía no usan `Estilo::Mono`.
pub fn mono_public() -> FontId {
    fuente(Estilo::Mono)
}

/// Recorta a `max` caracteres con elipsis; para nombres de pista, clips y
/// proyectos que no caben en su rect.
pub fn acortar(texto: &str, max: usize) -> String {
    if texto.chars().count() <= max {
        texto.to_owned()
    } else {
        let corto: String = texto.chars().take(max.saturating_sub(1)).collect();
        format!("{corto}…")
    }
}

/// Misma medida, pero con una familia siempre registrada. La usan las vistas
/// que se pintan en tests sin `aplicar`: las familias con nombre solo existen
/// después de instalar las fuentes.
pub fn fuente_segura(estilo: Estilo) -> FontId {
    let fuente = fuente(estilo);
    let familia = match fuente.family {
        egui::FontFamily::Name(_) => egui::FontFamily::Proportional,
        otra => otra,
    };
    FontId::new(fuente.size, familia)
}

/// Convierte un `#RRGGBB` en color; un valor inválido cae a `TEXTO_2`.
pub fn color(hex: &str) -> Color32 {
    color_de_hex(hex)
}

pub fn color_de_hex(valor: &str) -> Color32 {
    let hexadecimal = valor.strip_prefix('#').unwrap_or(valor);
    if hexadecimal.len() != 6 {
        return TEXTO_2;
    }
    let componentes = [0, 2, 4].map(|inicio| {
        hexadecimal
            .get(inicio..inicio + 2)
            .and_then(|par| u8::from_str_radix(par, 16).ok())
    });
    match componentes {
        [Some(r), Some(g), Some(b)] => Color32::from_rgb(r, g, b),
        _ => TEXTO_2,
    }
}

// ── Estilo y helpers ────────────────────────────────────────────────────────

/// Registra las fuentes y aplica el tema de una sola vez al arrancar.
pub fn aplicar(ctx: &egui::Context) {
    tipografia::instalar(ctx);
    ctx.set_style(estilo());
}

pub fn estilo() -> Style {
    let mut estilo = Style::default();
    let mut visuals = Visuals::dark();

    visuals.panel_fill = FONDO;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = CONTROL;
    visuals.faint_bg_color = RIEL;
    visuals.selection.bg_fill = ACENTO;
    visuals.selection.stroke.color = ACENTO;
    visuals.hyperlink_color = ACENTO;
    visuals.widgets.hovered.bg_fill = ELEVADO;
    visuals.widgets.active.bg_fill = CONTROL;
    for estado in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        estado.corner_radius = CornerRadius::same(RADIO_CONTROL as u8);
        estado.bg_stroke.color = BORDE_CONTROL;
    }
    estilo.visuals = visuals;

    // Los diez estilos propios conviven con los de egui, que también se
    // redimensionan para que ningún texto caiga por debajo de 11 px.
    for estilo_texto in Estilo::TODAS {
        estilo.text_styles.insert(
            TextStyle::Name(estilo_texto.nombre().into()),
            fuente(*estilo_texto),
        );
    }
    let tamaños: [(TextStyle, f32); 5] = [
        (TextStyle::Heading, 20.0),
        (TextStyle::Body, 13.0),
        (TextStyle::Button, 12.0),
        (TextStyle::Small, 11.0),
        (TextStyle::Monospace, 11.0),
    ];
    for (clave, tamano) in tamaños {
        let familia = estilo.text_styles.get(&clave).map(|f| f.family.clone());
        estilo.text_styles.insert(
            clave,
            FontId::new(tamano, familia.unwrap_or(egui::FontFamily::Proportional)),
        );
    }

    estilo.spacing.item_spacing = egui::vec2(ESP_4, ESP_4);
    estilo.spacing.button_padding = egui::vec2(ESP_8, ESP_4);
    estilo.spacing.interact_size = egui::vec2(ALTO_CONTROL, ALTO_CONTROL);
    estilo.spacing.menu_spacing = ESP_2;
    estilo.animation_time = 0.14;
    estilo.interaction.tooltip_delay = 0.4;
    estilo
}

/// Superficie estándar: radio 6, borde de un píxel y margen de 8.
pub fn panel(fill: Color32) -> Frame {
    Frame {
        inner_margin: Margin::same(ESP_8 as i8),
        corner_radius: CornerRadius::same(RADIO_SUPERFICIE as u8),
        fill,
        stroke: Stroke::new(1.0_f32, BORDE),
        ..Frame::NONE
    }
}

/// Texto corto con color semántico (estado de guardado, contadores).
pub fn chip(ui: &mut egui::Ui, texto: &str, color: Color32) {
    ui.label(
        egui::RichText::new(texto)
            .font(fuente(Estilo::Pequeno))
            .color(color),
    );
}

/// Etiqueta de sección: 11 px, medium, gris terciario.
pub fn etiqueta(ui: &mut egui::Ui, texto: impl Into<String>) {
    let fuente_etiqueta = fuente(Estilo::Pequeno);
    let mut trabajo =
        egui::text::LayoutJob::simple_singleline(texto.into(), fuente_etiqueta.clone(), TEXTO_2);
    if let Some(seccion) = trabajo.sections.first_mut() {
        seccion.format.extra_letter_spacing = fuente_etiqueta.size * LETRA_ESPACIADO;
    }
    ui.add(egui::Label::new(trabajo).selectable(false));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luminancia(c: Color32) -> f32 {
        let canal = |v: u8| {
            let x = v as f32 / 255.0;
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * canal(c.r()) + 0.7152 * canal(c.g()) + 0.0722 * canal(c.b())
    }

    fn contraste(a: Color32, b: Color32) -> f32 {
        let (claro, oscuro) = if luminancia(a) > luminancia(b) {
            (a, b)
        } else {
            (b, a)
        };
        (luminancia(claro) + 0.05) / (luminancia(oscuro) + 0.05)
    }

    #[test]
    fn todo_texto_cumple_aa() {
        for (texto, fondo) in [
            (TEXTO, FONDO),
            (TEXTO_2, FONDO),
            (TEXTO_3, FONDO),
            (TEXTO, PANEL),
            (TEXTO_2, PANEL),
            (TEXTO_3, PANEL),
            (TEXTO_2, CONTROL),
            (Color32::WHITE, PRIMARIO),
            (TEXTO_MENU, FONDO),
        ] {
            let c = contraste(texto, fondo);
            assert!(c >= 4.5, "{texto:?} sobre {fondo:?} da {c:.2}");
        }
    }

    #[test]
    fn ningun_estilo_baja_de_once() {
        for estilo in Estilo::TODAS {
            assert!(fuente(*estilo).size >= 11.0, "{estilo:?}");
        }
    }

    #[test]
    fn la_animacion_esta_entre_120_y_160_ms() {
        let t = estilo().animation_time;
        assert!((0.12..=0.16).contains(&t), "animation_time = {t}");
    }
}
