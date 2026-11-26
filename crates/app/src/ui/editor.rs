//! Vista del editor en egui. Compone el preview, la barra de transporte, los
//! paneles laterales y la barra de estado. Devuelve **todos** los mensajes del
//! frame: sobrescribirlos habría dejado los botones inertes.

use std::sync::{Arc, Mutex};

use egui::{Frame, Margin, Ui};

use crate::comandos::MapaAtajos;
use crate::editor_app::{EditorApp, MensajeEditor, Panel};
use crate::preferencias::Preferencias;
use crate::textos::Textos;
use crate::ui::kit;
use crate::ui::paneles::{estilos, hablantes, subtitulos};
use crate::ui::{tema, transporte};
use crate::video::Gpu;

/// Altura de los carriles del timeline, elegible en su barra de herramientas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlturaPista {
    Pequeña,
    Mediana,
    /// Igual que el carril de audio del mockup; es el valor inicial.
    #[default]
    Grande,
}

impl AlturaPista {
    /// Tres alturas sin literales sueltos: todos vienen de `tema`.
    pub fn px(self) -> f32 {
        match self {
            Self::Pequeña => tema::ALTO_CONTROL,
            Self::Mediana => tema::ALTO_FILA_SUB,
            Self::Grande => tema::ALTO_PISTA_AUDIO,
        }
    }
}

/// Medidas del editor. Solo las tres primeras se guardan en `Preferencias`;
/// las de la timeline viven en la memoria de egui mientras dura la sesión.
#[derive(Debug, Clone)]
pub struct MedidasEditor {
    pub ancho_panel: f32,
    pub alto_timeline: f32,
    pub panel_visible: bool,
    /// Ancho del margen de nombres del timeline, arrastrable.
    pub ancho_gutter: f32,
    pub altura_pista: AlturaPista,
    pub ocultar_vacios: bool,
    /// Ids de los carriles contraídos (`""` = el carril de audio).
    pub pistas_colapsadas: Vec<String>,
}

/// `main.rs` compara dos `MedidasEditor` para decidir si escribe preferencias:
/// un cambio de gutter no debe provocar una escritura en disco por frame.
impl PartialEq for MedidasEditor {
    fn eq(&self, otra: &Self) -> bool {
        self.ancho_panel == otra.ancho_panel
            && self.alto_timeline == otra.alto_timeline
            && self.panel_visible == otra.panel_visible
    }
}

impl Default for MedidasEditor {
    fn default() -> Self {
        Self {
            ancho_panel: tema::ANCHO_PANEL_DEFECTO,
            alto_timeline: 260.0,
            panel_visible: true,
            ancho_gutter: tema::ANCHO_GUTTER,
            altura_pista: AlturaPista::default(),
            ocultar_vacios: false,
            pistas_colapsadas: Vec::new(),
        }
    }
}

impl From<&Preferencias> for MedidasEditor {
    fn from(prefs: &Preferencias) -> Self {
        // Un ancho fuera de rango es un valor corrupto (p. ej. el crecido
        // por el bug del lápiz): se descarta en vez de arrastrar el panel.
        let ancho = if !(tema::ANCHO_PANEL_MIN..=tema::ANCHO_PANEL_MAX).contains(&prefs.ancho_panel)
        {
            tema::ANCHO_PANEL_DEFECTO
        } else {
            prefs.ancho_panel
        };
        Self {
            ancho_panel: ancho,
            alto_timeline: prefs.alto_timeline,
            panel_visible: prefs.panel_visible,
            ..Self::default()
        }
    }
}

/// Las medidas sin preferencias se guardan en la memoria de egui: `main.rs`
/// reconstruye el struct desde el disco en cada frame y perdería el gutter, la
/// altura de pista o el colapso si solo vivieran en el struct.
fn id_medidas() -> egui::Id {
    egui::Id::new("rod.medidas-editor")
}

pub(crate) fn medidas_en_ctx(ctx: &egui::Context, semilla: &MedidasEditor) -> MedidasEditor {
    ctx.data(|memoria| memoria.get_temp::<MedidasEditor>(id_medidas()))
        .unwrap_or_else(|| {
            ctx.data_mut(|memoria| memoria.insert_temp(id_medidas(), semilla.clone()));
            semilla.clone()
        })
}

pub(crate) fn guardar_medidas(ctx: &egui::Context, medidas: &MedidasEditor) {
    ctx.data_mut(|memoria| memoria.insert_temp(id_medidas(), medidas.clone()));
}

/// Tope del panel inferior: el video se puede encoger hasta dejarle el 70 %
/// de la ventana al timeline, sin quitarle nunca el suelo de `ALTO_VIDEO_MIN`.
pub fn alto_maximo_timeline(alto_ventana: f32) -> f32 {
    (alto_ventana * 0.7)
        .min(alto_ventana - tema::ALTO_VIDEO_MIN)
        .max(tema::ALTO_PANEL_INFERIOR_MIN)
}

/// Alto de la timeline: el campo de texto puede quitarle altura al crecer.
pub fn alto_timeline(
    state: &EditorApp,
    medidas: &MedidasEditor,
    _animacion: f32,
    ancho: f32,
) -> f32 {
    (medidas.alto_timeline - crate::ui::texto::alto_extra(state, ancho))
        .max(tema::ALTO_PANEL_INFERIOR_MIN)
}

/// Distribución: inspector a la derecha, timeline abajo, monitor al centro.
pub fn show(
    ctx: &egui::Context,
    state: &mut EditorApp,
    formato: wgpu::TextureFormat,
    gpu: Arc<Mutex<Option<Gpu>>>,
    textos: &Textos,
    atajos: &MapaAtajos,
    medidas: &mut MedidasEditor,
) -> Vec<MensajeEditor> {
    let _ = atajos;
    let mut mensajes = Vec::new();
    *medidas = medidas_en_ctx(ctx, medidas);

    if medidas.panel_visible {
        let ancho = medidas
            .ancho_panel
            .clamp(tema::ANCHO_PANEL_MIN, tema::ANCHO_PANEL_MAX);
        medidas.ancho_panel = ancho;
        let mut arrastre_panel: Option<f32> = None;
        egui::SidePanel::right("inspector")
            .resizable(false)
            .exact_width(ancho)
            .frame(Frame {
                inner_margin: Margin::symmetric(tema::ESP_12 as i8, tema::ESP_8 as i8),
                corner_radius: tema::radio_superficie(),
                fill: tema::PANEL,
                stroke: egui::Stroke::new(tema::GROSOR_BORDE, tema::BORDE),
                ..Frame::NONE
            })
            .show(ctx, |ui| {
                // Como en el timeline: el `PanelState` de egui acumula el rect
                // entre frames, así que el ancho sale de `medidas` y el
                // arrastre escribe en ella desde el borde izquierdo del panel.
                let borde = ui.max_rect().min.x;
                let franja = egui::Rect::from_x_y_ranges(
                    (borde - tema::GROSOR_SPLITTER / 2.0)..=(borde + tema::GROSOR_SPLITTER / 2.0),
                    ui.max_rect().y_range(),
                );
                let divisor = ui.interact(
                    franja,
                    egui::Id::new("divisor-panel-lateral"),
                    egui::Sense::drag(),
                );
                if divisor.dragged()
                    && let Some(pointer) = divisor.interact_pointer_pos()
                {
                    arrastre_panel = Some(pointer.x);
                }
                if divisor.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }
                let lineas = state.editor.proyecto().captions.len();
                let hablantes = state.editor.proyecto().hablantes.len();
                if let Some(panel) = kit::tab_bar(
                    ui,
                    &[
                        (Panel::Subtitulos, textos.tab_subtitulos, Some(lineas)),
                        (Panel::Hablantes, textos.tab_hablantes, Some(hablantes)),
                        (Panel::Estilos, textos.tab_estilos, None),
                    ],
                    state.panel,
                ) {
                    mensajes.push(MensajeEditor::Panel(panel));
                }
                match state.panel {
                    Panel::Subtitulos => mensajes.extend(subtitulos::mostrar(ui, state, textos)),
                    Panel::Hablantes => mensajes.extend(hablantes::mostrar(ui, state, textos)),
                    Panel::Estilos => mensajes.extend(estilos::mostrar(ui, state, textos)),
                }
            });
        if let Some(x) = arrastre_panel {
            medidas.ancho_panel =
                (ctx.content_rect().max.x - x).clamp(tema::ANCHO_PANEL_MIN, tema::ANCHO_PANEL_MAX);
        }
    }

    let tope = alto_maximo_timeline(ctx.content_rect().height());
    medidas.alto_timeline = medidas
        .alto_timeline
        .clamp(tema::ALTO_PANEL_INFERIOR_MIN, tope);
    let suelo = tema::ALTO_PANEL_INFERIOR_MIN;
    let id_objetivo = egui::Id::new("rod.splitter.objetivo");
    let id_anim = egui::Id::new("rod.splitter.anim");
    let id_animando = egui::Id::new("rod.splitter.animando");
    let en_tope = ctx.data(|memoria| memoria.get_temp::<bool>(id_objetivo).unwrap_or(false));
    let animando = ctx.data(|memoria| memoria.get_temp::<bool>(id_animando).unwrap_or(false));
    if animando {
        let t = ctx.animate_bool_with_time(id_anim, en_tope, tema::ANIMACION);
        medidas.alto_timeline = suelo + (tope - suelo) * t;
        if (t - f32::from(en_tope)).abs() < 0.001 {
            ctx.data_mut(|memoria| memoria.insert_temp(id_animando, false));
        }
    }
    let alto_abajo = alto_timeline(state, medidas, tema::ANIMACION, ctx.content_rect().width());
    let mut arrastre_divisor: Option<f32> = None;
    let mut alternar_tope = false;
    egui::TopBottomPanel::bottom("timeline")
        .resizable(false)
        .exact_height(alto_abajo)
        .frame(tema::panel(tema::FONDO))
        .show(ctx, |ui| {
            // La franja se registra dentro del Ui del panel, que sí tiene
            // clip rect y capa: un `Ui::new` externo nunca recibía el puntero.
            let borde = ui.max_rect().min.y;
            let franja = egui::Rect::from_x_y_ranges(
                ui.max_rect().x_range(),
                (borde - tema::GROSOR_SPLITTER / 2.0)..=(borde + tema::GROSOR_SPLITTER / 2.0),
            );
            let divisor = ui.interact(
                franja,
                egui::Id::new("divisor-timeline"),
                egui::Sense::click_and_drag(),
            );
            if divisor.drag_started() {
                ctx.data_mut(|memoria| memoria.insert_temp(id_animando, false));
            }
            if divisor.double_clicked() {
                alternar_tope = true;
            }
            if divisor.dragged() {
                // Al arrastrar fuera de la franja, `interact_pointer_pos`
                // devuelve None: se cae al último puntero conocido para que
                // el divisor siga al dedo hasta el borde de la ventana.
                let y = divisor
                    .interact_pointer_pos()
                    .or_else(|| ctx.input(|i| i.pointer.latest_pos()))
                    .map(|p| p.y);
                if let Some(y) = y {
                    arrastre_divisor = Some(y);
                }
            }
            if divisor.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
            }
            mensajes.extend(crate::ui::timeline::toolbar(ui, state, textos, medidas));
            guardar_medidas(ctx, medidas);
            mensajes.extend(crate::ui::timeline::mostrar(ui, state, textos));
        });

    // El `TopBottomPanel` resizable guarda un `PanelState` cuyo rect se
    // acumula entre frames (en una ventana de 700 px llegó a 930 px), así que
    // leer `response.rect.height()` devolvía el panel al tope al arrastrar.
    // Aquí la altura es siempre la de `medidas` y el arrastre escribe en ella.
    if let Some(y) = arrastre_divisor {
        medidas.alto_timeline =
            (ctx.content_rect().max.y - y).clamp(tema::ALTO_PANEL_INFERIOR_MIN, tope);
    }
    if alternar_tope {
        // Doble clic: alterna entre el suelo y el tope con la animación de 140 ms.
        ctx.data_mut(|memoria| {
            memoria.insert_temp(id_objetivo, !en_tope);
            memoria.insert_temp(id_animando, true);
        });
    }
    // La altura calculada aquí es la definitiva: guardarla de nuevo para que
    // el `medidas_en_ctx` del siguiente frame no reviva el valor prearrastre.
    guardar_medidas(ctx, medidas);

    // El monitor va a sangre: sin el frame del `CentralPanel` el video toca
    // los bordes del área que le queda entre el inspector y la timeline.
    egui::CentralPanel::default()
        .frame(Frame::NONE)
        .show(ctx, |ui| {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, tema::COLOR_FONDO_VIDEO);
            mensajes.extend(mostrar(ui, state, formato, gpu, textos));
        });

    mensajes
}

pub fn mostrar(
    ui: &mut Ui,
    state: &mut EditorApp,
    formato: wgpu::TextureFormat,
    gpu: Arc<Mutex<Option<Gpu>>>,
    textos: &Textos,
) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();

    // Preview real: el mismo pipeline de video + libass que antes, ahora a
    // través de un paint callback de egui_wgpu.
    let alto = (ui.available_height()
        - tema::ALTO_TRANSPORTE
        - crate::ui::texto::alto_extra(state, ui.available_width()))
    .max(180.0);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), alto), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::ZERO, tema::COLOR_FONDO_VIDEO);
    if !state.pixels.is_empty() {
        // `state.layers` se sustituye por un Arc nuevo cada vez que mpv entrega
        // un frame. Mientras sea el mismo, reutilizamos el Arc de píxeles: antes
        // se copiaban 8,3 MB en cada repintado de la UI aunque no hubiera frame.
        #[derive(Clone)]
        struct CachePixeles {
            capas: usize,
            longitud: usize,
            pixeles: Arc<Vec<u8>>,
        }
        let id = egui::Id::new("rod.pixeles-preview");
        let marca = Arc::as_ptr(&state.layers) as usize;
        let cache = ui
            .ctx()
            .data(|memoria| memoria.get_temp::<CachePixeles>(id));
        let pixeles = match cache {
            Some(cache) if cache.capas == marca && cache.longitud == state.pixels.len() => {
                cache.pixeles
            }
            _ => {
                let pixeles = Arc::new(state.pixels.clone());
                ui.ctx().data_mut(|memoria| {
                    memoria.insert_temp(
                        id,
                        CachePixeles {
                            capas: marca,
                            longitud: state.pixels.len(),
                            pixeles: Arc::clone(&pixeles),
                        },
                    );
                });
                pixeles
            }
        };
        let frame = crate::video::Frame {
            pixels: pixeles,
            width: crate::preview::CANVAS.0,
            height: crate::preview::CANVAS.1,
            capas: Arc::clone(&state.layers),
        };
        crate::ui::video::pintar(ui, rect, frame, formato, gpu);
    }

    mensajes.extend(transporte::mostrar(
        ui,
        state,
        textos,
        &crate::comandos::MapaAtajos::por_defecto(),
    ));

    transporte::barra_estado(ui, state, textos);
    mensajes
}

#[cfg(test)]
mod tests {
    use super::AlturaPista;
    use crate::editor_app::{EditorApp, MensajeEditor, update};

    fn editor() -> EditorApp {
        let video = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../testdata/clip-15s-2audio.mp4"
        );
        EditorApp::desde_video(video).expect("video de prueba")
    }

    #[test]
    fn aplicar_texto_cambia_el_caption_y_registra_un_paso() {
        let mut editor = editor();
        editor.editor.agregar(rod_core::Caption {
            id: "cap-prueba".into(),
            inicio: 1.0,
            fin: 2.0,
            texto: "texto original".into(),
            hablante_id: None,
        });
        let id = editor.editor.proyecto().captions[0].id.clone();
        editor.editor.seleccionar_uno(&id);
        editor.limpio = editor.editor.proyecto().clone();

        // La edición inline se confirma con un mensaje; el botón que lo emite
        // se prueba en `tests/subtitulos.rs`.
        update(
            &mut editor,
            MensajeEditor::EditarInline(id, "texto nuevo".into()),
        );
        assert_eq!(editor.editor.texto_de("cap-prueba"), Some("texto nuevo"));
        assert!(editor.editor.puede_deshacer());
        assert!(!editor.texto_inline_activo());
    }

    #[test]
    fn las_tres_alturas_de_pista_son_crecientes() {
        assert!(
            AlturaPista::Pequeña.px() < AlturaPista::Mediana.px()
                && AlturaPista::Mediana.px() < AlturaPista::Grande.px()
        );
    }
}
