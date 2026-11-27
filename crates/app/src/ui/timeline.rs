//! Timeline dibujada con `egui::Painter`: regla con marcas, carriles con
//! nombre, clips con el color del hablante y playhead. El significado de cada
//! gesto sigue en `timeline::pulsar`, que es código probado.

use std::collections::HashSet;

use egui::{Color32, Ui, Vec2};

use crate::comandos::Comando;
use crate::editor_app::{EditorApp, MensajeEditor};
use crate::textos::Textos;
use crate::timeline::{self, ALTO_CARRIL, ALTO_REGLA, Mensaje, Vista};
use crate::ui::editor::{AlturaPista, MedidasEditor};
use crate::ui::kit::{self, Icono};
use crate::ui::tema::{self, Estilo};

/// El slider va en 0–100; `Vista.escala` está en segundos/píxel, así que el
/// mapeo es logarítmico entre los límites de la vista.
pub fn zoom_desde_slider(valor: f32) -> f64 {
    let t = f64::from(valor.clamp(0.0, 100.0)) / 100.0;
    Vista::ESCALA_MINIMA * (Vista::ESCALA_MAXIMA / Vista::ESCALA_MINIMA).powf(t)
}

pub fn slider_desde_zoom(escala: f64) -> f32 {
    let escala = escala.clamp(Vista::ESCALA_MINIMA, Vista::ESCALA_MAXIMA);
    (100.0 * (escala / Vista::ESCALA_MINIMA).ln()
        / (Vista::ESCALA_MAXIMA / Vista::ESCALA_MINIMA).ln()) as f32
}

/// Botón compacto de la barra de herramientas, con tooltip y foco de 2 px.
fn boton_toolbar(ui: &mut Ui, texto: &str, activo: bool) -> egui::Response {
    let color = if activo { tema::ACENTO } else { tema::TEXTO_2 };
    kit::pill(ui, texto, color).on_hover_ui(|ui| {
        ui.label(
            egui::RichText::new(texto)
                .font(tema::fuente(Estilo::Base))
                .color(tema::TEXTO),
        );
    })
}

/// Barra de herramientas de 32 px: etiqueta a la izquierda y acciones a la
/// derecha. Ratón, menú contextual y atajos emiten los mismos comandos.
pub fn toolbar(
    ui: &mut Ui,
    state: &mut EditorApp,
    textos: &Textos,
    medidas: &mut MedidasEditor,
) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), tema::ALTO_TOOLBAR_TIMELINE),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = tema::ESP_4;
            tema::etiqueta(ui, textos.timeline.to_uppercase());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // El inspector se puede cerrar y reabrir desde aquí: el panel
                // derecho desaparece sin perder la pestaña seleccionada.
                if kit::icon_button(
                    ui,
                    Icono::Ojo,
                    medidas.panel_visible,
                    textos.seccion_ver,
                    None,
                )
                .clicked()
                {
                    medidas.panel_visible = !medidas.panel_visible;
                }

                let mut zoom = slider_desde_zoom(state.vista.escala);
                let slider = ui.add(
                    egui::Slider::new(&mut zoom, 0.0..=100.0)
                        .show_value(false)
                        .handle_shape(egui::style::HandleShape::Rect { aspect_ratio: 0.5 }),
                );
                if slider.has_focus() {
                    kit::anillo_foco(ui, &slider);
                }
                let nueva = zoom_desde_slider(zoom);
                if (nueva - state.vista.escala).abs() > 1e-9 {
                    mensajes.push(MensajeEditor::Zoom(state.vista.escala / nueva));
                }
                if kit::icon_button(ui, Icono::Objetivo, state.seguir, textos.seguir, None)
                    .clicked()
                {
                    mensajes.push(MensajeEditor::Seguir);
                }
                if kit::icon_button(
                    ui,
                    Icono::Iman,
                    state.snap,
                    textos.comando(Comando::AlternarSnap),
                    None,
                )
                .clicked()
                {
                    mensajes.push(MensajeEditor::Comando(Comando::AlternarSnap));
                }
                if kit::icon_button(ui, Icono::Unir, false, textos.comando(Comando::Unir), None)
                    .clicked()
                {
                    mensajes.push(MensajeEditor::Comando(Comando::Unir));
                }
                if kit::icon_button(
                    ui,
                    Icono::Dividir,
                    false,
                    textos.comando(Comando::Dividir),
                    None,
                )
                .clicked()
                {
                    mensajes.push(MensajeEditor::Comando(Comando::Dividir));
                }
                if kit::icon_button(
                    ui,
                    Icono::Papelera,
                    false,
                    textos.comando(Comando::Eliminar),
                    None,
                )
                .clicked()
                {
                    mensajes.push(MensajeEditor::Comando(Comando::Eliminar));
                }

                // El ancho publicado por la timeline es el del frame anterior;
                // antes de la primera medida se usa el ancho disponible.
                let ancho = if state.ancho_timeline > 0.0 {
                    state.ancho_timeline
                } else {
                    ui.available_width() - medidas.ancho_gutter - tema::ANCHO_SCROLLBAR
                };
                if boton_toolbar(ui, textos.fit_all, false).clicked() {
                    state.vista = state.vista.encajar(state.duration, ancho);
                    state.scroll_pistas = 0.0;
                }
                if boton_toolbar(ui, textos.hide_empty_tracks, medidas.ocultar_vacios).clicked() {
                    medidas.ocultar_vacios = !medidas.ocultar_vacios;
                    state.scroll_pistas = 0.0;
                }
                // El layout es right_to_left: se inserta al revés para que en
                // pantalla queden S, M, L de izquierda a derecha.
                for (altura, etiqueta) in [
                    (AlturaPista::Grande, "L"),
                    (AlturaPista::Mediana, "M"),
                    (AlturaPista::Pequeña, "S"),
                ] {
                    if boton_toolbar(ui, etiqueta, medidas.altura_pista == altura).clicked() {
                        medidas.altura_pista = altura;
                    }
                }
            });
        },
    );
    mensajes
}

/// Menú contextual compartido por filas y clips.
pub fn menu_contexto(ui: &mut Ui, id: &str, mensajes: &mut Vec<MensajeEditor>, textos: &Textos) {
    // El menú actúa sobre lo que el usuario señaló, no sobre la selección previa.
    mensajes.push(MensajeEditor::Seleccionar(id.to_owned()));
    if ui.button(textos.comando(Comando::Eliminar)).clicked() {
        mensajes.push(MensajeEditor::Comando(Comando::Eliminar));
        ui.close();
    }
    if ui.button(textos.comando(Comando::Dividir)).clicked() {
        mensajes.push(MensajeEditor::Comando(Comando::Dividir));
        ui.close();
    }
    if ui.button(textos.comando(Comando::Unir)).clicked() {
        mensajes.push(MensajeEditor::UnirConSiguiente(id.to_owned()));
        ui.close();
    }
}

/// Carriles que caben bajo la regla en un panel de la altura dada.
pub fn filas_visibles(alto: f32) -> usize {
    (((alto - ALTO_REGLA) / ALTO_CARRIL).floor().max(1.0)) as usize
}

/// Cuántas filas se pueden desplazar: las que sobran tras las visibles.
pub fn max_scroll_pistas(filas: usize, alto: f32) -> f32 {
    filas.saturating_sub(filas_visibles(alto)) as f32
}

/// Nueva posición de scroll tras una muesca de rueda, acotada al rango útil.
pub fn desplazar(scroll: f32, lineas: f32, filas: usize, alto: f32) -> f32 {
    (scroll + lineas).clamp(0.0, max_scroll_pistas(filas, alto))
}

/// Gestos posibles de la rueda sobre el timeline. `Zoom` y `Pan` conservan el
/// sentido de `raw_scroll_delta`; `Pistas` ya usa el convenio nativo (girar
/// hacia abajo avanza por los carriles).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rueda {
    Zoom(f32),
    Pan(f32),
    Pistas(f32),
}

/// Reparte la rueda según el modificador. egui 0.33 mueve la muesca a `.x`
/// cuando Shift está pulsado y deja `.y` a 0, así que el pan lee `.x`.
pub fn gesto_rueda(modificadores: egui::Modifiers, delta: egui::Vec2) -> Rueda {
    if modificadores.ctrl || modificadores.command {
        Rueda::Zoom(delta.y / 40.0)
    } else if modificadores.shift {
        Rueda::Pan(delta.x / 40.0)
    } else {
        Rueda::Pistas(-delta.y / 40.0)
    }
}

/// Scroll que se puede usar este frame, aunque el estado venga de una vista
/// con más hablantes (se borran y el recorrido desaparece).
pub fn scroll_visible(scroll: f32, filas: usize, alto: f32) -> f32 {
    scroll.clamp(0.0, max_scroll_pistas(filas, alto))
}

/// Posición de scroll que corresponde a un punto de la scrollbar.
pub fn scroll_desde_barra(y_rel: f32, alto: f32, filas: usize) -> f32 {
    let max = max_scroll_pistas(filas, alto);
    if alto <= 0.0 || max <= 0.0 {
        return 0.0;
    }
    (y_rel / alto * max).clamp(0.0, max)
}

/// Etiqueta adaptativa: `m:ss` para ticks de 1 s o más, `m:ss.d` si no.
fn etiqueta_regla(segundos: f64, paso: f64) -> String {
    let minutos = (segundos / 60.0).floor() as u64;
    let resto = segundos - minutos as f64 * 60.0;
    if paso >= 1.0 {
        format!("{minutos}:{:02}", resto.floor() as u64)
    } else {
        format!("{minutos}:{:04.1}", resto)
    }
}

/// Coordenada local -> carril absoluto. La regla no se desplaza: un clic ahí
/// sigue pidiendo buscar tiempo, aunque las pistas estén bajadas.
pub fn punto(rel: egui::Vec2, scroll: f32) -> timeline::Punto {
    let y = if rel.y < ALTO_REGLA {
        rel.y
    } else {
        rel.y + scroll * ALTO_CARRIL
    };
    timeline::Punto::new(rel.x - tema::ANCHO_GUTTER, y)
}

/// Punto lógico para `pulsar`: la y sintética conserva la altura fija que la
/// lógica de arrastre espera, aunque en pantalla el carril esté contraído.
fn punto_en_fila(x: f32, fila_modelo: usize) -> timeline::Punto {
    timeline::Punto::new(
        x,
        ALTO_REGLA + fila_modelo as f32 * ALTO_CARRIL + ALTO_CARRIL / 2.0,
    )
}

/// Marcador de borde cuando el tiempo sale del área visible. `true` = izquierda.
pub fn marca_borde(x: f32, min: f32, max: f32) -> Option<(f32, bool)> {
    if x < min {
        Some((min, true))
    } else if x > max {
        Some((max, false))
    } else {
        None
    }
}

pub fn playhead(state: &EditorApp) -> f64 {
    if state.position.is_finite() && state.duration.is_finite() {
        state.tiempo_ui()
    } else {
        0.0
    }
}

fn pintar_borde_punteado(painter: &egui::Painter, rect: egui::Rect, color: Color32) {
    const PASO: f32 = 12.0;
    let mut x = rect.min.x;
    while x < rect.max.x {
        painter.line_segment(
            [
                egui::pos2(x.min(rect.max.x), rect.min.y),
                egui::pos2((x + tema::ESP_2).min(rect.max.x), rect.min.y),
            ],
            egui::Stroke::new(tema::GROSOR_BORDE, color),
        );
        painter.line_segment(
            [
                egui::pos2(x.min(rect.max.x), rect.max.y),
                egui::pos2((x + tema::ESP_2).min(rect.max.x), rect.max.y),
            ],
            egui::Stroke::new(tema::GROSOR_BORDE, color),
        );
        x += PASO;
    }
}

/// Un clip seleccionado siempre gana; sin hablante y sin selección usa el gris.
pub fn color_borde_clip(
    hablante: Option<(usize, &rod_core::Hablante)>,
    seleccionado: bool,
) -> egui::Color32 {
    if seleccionado {
        return tema::ACENTO;
    }
    hablante.map_or(tema::COLOR_SIN_HABLANTE, |(indice, hablante)| {
        tema::color(rod_core::color_identidad(hablante, indice))
    })
}

/// Paso de regla redondo: 0,5 / 1 / 2 / 5 / 10 s según el zoom.
fn paso_de_regla(escala: f64) -> f64 {
    for paso in [0.5, 1.0, 2.0, 5.0, 10.0, 30.0] {
        if paso / escala >= 70.0 {
            return paso;
        }
    }
    60.0
}

/// Un carril tal como se ve: el índice `modelo` es el que entiende `pulsar`.
struct Fila {
    id: String,
    nombre: String,
    color: Color32,
    alto: f32,
    modelo: usize,
}

/// Carril contraído: lo justo para el nombre y el chevron.
fn alto_colapsado() -> f32 {
    tema::ALTO_PILL * 2.0
}

/// y (en coordenadas de pantalla) del carril `indice` de la lista visible.
fn y_de_fila(filas: &[Fila], indice: usize, inicio: f32, scroll: f32) -> f32 {
    let mut y = inicio - scroll;
    for fila in filas.iter().take(indice) {
        y += fila.alto;
    }
    y
}

/// Índice visible bajo una y de pantalla; `None` sobre la regla.
fn fila_en_y(filas: &[Fila], inicio: f32, scroll: f32, y: f32) -> Option<usize> {
    if y < inicio {
        return None;
    }
    let mut tope = inicio - scroll;
    for (indice, fila) in filas.iter().enumerate() {
        tope += fila.alto;
        if y < tope {
            return Some(indice);
        }
    }
    None
}

pub fn mostrar(ui: &mut Ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor> {
    let alto = ui.available_height();
    let mut config = crate::ui::editor::medidas_en_ctx(ui.ctx(), &MedidasEditor::default());
    let gutter_ancho = config
        .ancho_gutter
        .clamp(tema::ALTO_CONTROL, tema::ANCHO_FILTRO);
    let alto_pista = config.altura_pista.px();

    let (rect, respuesta) = ui.allocate_exact_size(
        Vec2::new(ui.available_width() - tema::ANCHO_SCROLLBAR, alto),
        egui::Sense::click_and_drag(),
    );
    let (barra, respuesta_barra) = ui.allocate_exact_size(
        Vec2::new(tema::ANCHO_SCROLLBAR, alto),
        egui::Sense::click_and_drag(),
    );
    let painter = ui.painter();
    painter.rect_filled(rect, egui::CornerRadius::ZERO, tema::SUPERFICIE);

    let gutter =
        egui::Rect::from_min_max(rect.min, egui::pos2(rect.min.x + gutter_ancho, rect.max.y));
    let area = egui::Rect::from_min_max(egui::pos2(gutter.max.x, rect.min.y), rect.max);
    painter.rect_filled(gutter, egui::CornerRadius::ZERO, tema::PANEL);
    painter.line_segment(
        [
            egui::pos2(gutter.max.x, rect.min.y),
            egui::pos2(gutter.max.x, rect.max.y),
        ],
        egui::Stroke::new(tema::GROSOR_BORDE, tema::BORDE),
    );

    // Ancho real del eje. Solo se envía cuando cambia para no sobrescribir
    // otros mensajes del frame.
    let mut mensajes = Vec::new();
    if (state.ancho_timeline - area.width()).abs() > 0.5 {
        mensajes.push(MensajeEditor::Timeline(Mensaje::Ancho(area.width())));
    }

    // El borde del gutter se arrastra. Se registra sobre el Ui del timeline,
    // que es el que tiene clip rect y capa para recibir el puntero.
    let franja_gutter = egui::Rect::from_x_y_ranges(
        (gutter.max.x - tema::GROSOR_SPLITTER / 2.0)..=(gutter.max.x + tema::GROSOR_SPLITTER / 2.0),
        rect.y_range(),
    );
    let divisor_gutter = ui.interact(
        franja_gutter,
        egui::Id::new("divisor-gutter-timeline"),
        egui::Sense::drag(),
    );
    if divisor_gutter.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    if divisor_gutter.dragged()
        && let Some(puntero) = divisor_gutter.interact_pointer_pos()
    {
        config.ancho_gutter =
            (puntero.x - rect.min.x).clamp(tema::ALTO_CONTROL, tema::ANCHO_FILTRO);
        crate::ui::editor::guardar_medidas(ui.ctx(), &config);
    }

    let vista = state.vista;
    let proyecto = state.editor.proyecto();
    let hablantes = &proyecto.hablantes;

    // Fila 0 = audio: siempre bajo la regla, nunca se desplaza.
    let con_captiones: HashSet<&str> = proyecto
        .captions
        .iter()
        .filter_map(|caption| caption.hablante_id.as_deref())
        .collect();
    let mut filas = Vec::with_capacity(hablantes.len() + 1);
    let id_audio = "";
    let push_fila = |id: String,
                     nombre: String,
                     color: Color32,
                     modelo: usize,
                     colapsada: bool,
                     filas: &mut Vec<Fila>| {
        filas.push(Fila {
            id,
            nombre,
            color,
            alto: if colapsada {
                alto_colapsado()
            } else {
                alto_pista
            },
            modelo,
        });
    };
    push_fila(
        id_audio.to_owned(),
        textos.pista_audio.to_owned(),
        tema::TEXTO_3,
        0,
        config.pistas_colapsadas.iter().any(|id| id == id_audio),
        &mut filas,
    );
    for (indice, hablante) in hablantes.iter().enumerate() {
        if config.ocultar_vacios && !con_captiones.contains(hablante.id.as_str()) {
            continue;
        }
        let nombre = if hablante.nombre.is_empty() {
            hablante.tecla.clone()
        } else {
            hablante.nombre.clone()
        };
        push_fila(
            hablante.id.clone(),
            nombre,
            tema::color(rod_core::color_identidad(hablante, indice)),
            indice + 1,
            config.pistas_colapsadas.contains(&hablante.id),
            &mut filas,
        );
    }

    let y_audio = rect.min.y + ALTO_REGLA;
    let y_scroll = y_audio + filas[0].alto;
    let alto_scroll = (rect.max.y - y_scroll).max(0.0);
    let total_scroll: f32 = filas.iter().skip(1).map(|fila| fila.alto).sum();
    let max_scroll = (total_scroll - alto_scroll).max(0.0);
    let scroll = state.scroll_pistas.clamp(0.0, max_scroll);

    // Regla: marcas redondas con etiqueta en mono.
    painter.rect_filled(
        egui::Rect::from_min_max(area.min, egui::pos2(area.max.x, rect.min.y + ALTO_REGLA)),
        egui::CornerRadius::ZERO,
        tema::LATERAL,
    );
    let paso = paso_de_regla(vista.escala);
    let mut t = (vista.inicio / paso).ceil() * paso;
    while t <= vista.tiempo_de(area.width()) + paso {
        let x = area.min.x + vista.x_de(t);
        if (area.min.x..=area.max.x).contains(&x) {
            painter.line_segment(
                [
                    egui::pos2(x, rect.min.y + tema::ESP_16 - tema::ESP_2),
                    egui::pos2(x, rect.min.y + ALTO_REGLA),
                ],
                egui::Stroke::new(tema::GROSOR_BORDE, tema::BORDE),
            );
            painter.text(
                egui::pos2(x + tema::ESP_4, rect.min.y + tema::ESP_8),
                egui::Align2::LEFT_CENTER,
                etiqueta_regla(t, paso),
                tema::fuente(Estilo::Timecode),
                tema::TEXTO_3,
            );
        }
        t += paso;
    }
    painter.text(
        gutter.min + egui::vec2(tema::ESP_12, tema::ESP_8),
        egui::Align2::LEFT_CENTER,
        textos.tiempo.to_uppercase(),
        tema::fuente(Estilo::Pequeno),
        tema::TEXTO_3,
    );

    // Carriles: fondo alterno, nombre y chevron de colapso.
    let mut clique_chevron = false;
    for indice in 0..filas.len() {
        let y = y_de_fila(&filas, indice, y_audio, scroll);
        let fila = &filas[indice];
        if indice > 0 && (y + fila.alto <= y_scroll || y >= rect.max.y) {
            continue;
        }
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(area.min.x, y),
                egui::pos2(area.max.x, y + fila.alto - tema::GROSOR_BORDE),
            ),
            egui::CornerRadius::ZERO,
            if indice % 2 == 0 {
                tema::PANEL
            } else {
                tema::LATERAL
            },
        );
        painter.line_segment(
            [
                egui::pos2(area.min.x, y + fila.alto - tema::GROSOR_BORDE),
                egui::pos2(area.max.x, y + fila.alto - tema::GROSOR_BORDE),
            ],
            egui::Stroke::new(tema::GROSOR_BORDE, tema::BORDE),
        );
        let centro = y + fila.alto / 2.0;
        painter.circle_filled(
            gutter.min + egui::vec2(tema::ESP_16, centro - rect.min.y),
            tema::GROSOR_SPLITTER,
            fila.color,
        );
        painter.text(
            gutter.min + egui::vec2(tema::ESP_24, centro - rect.min.y),
            egui::Align2::LEFT_CENTER,
            tema::acortar(&fila.nombre, 12),
            tema::fuente(Estilo::Pequeno),
            tema::TEXTO_2,
        );

        let angulo = egui::pos2(gutter.min.x + tema::ESP_8, centro);
        let (punta, base) = if config.pistas_colapsadas.contains(&fila.id) {
            (
                egui::pos2(angulo.x - tema::ESP_2, angulo.y - tema::ESP_4),
                [
                    egui::pos2(angulo.x - tema::ESP_2, angulo.y + tema::ESP_4),
                    egui::pos2(angulo.x + tema::ESP_4, angulo.y),
                ],
            )
        } else {
            (
                egui::pos2(angulo.x - tema::ESP_4, angulo.y - tema::ESP_2),
                [
                    egui::pos2(angulo.x + tema::ESP_4, angulo.y - tema::ESP_2),
                    egui::pos2(angulo.x, angulo.y + tema::ESP_4),
                ],
            )
        };
        painter.add(egui::Shape::convex_polygon(
            vec![punta, base[0], base[1]],
            tema::TEXTO_2,
            egui::Stroke::NONE,
        ));
        let zona = egui::Rect::from_center_size(angulo, egui::Vec2::splat(tema::ESP_16));
        let chevron = ui.interact(
            zona,
            egui::Id::new(("chevron-pista", fila.id.clone())),
            egui::Sense::click(),
        );
        let colapsar = chevron.clicked();
        let _ = chevron.on_hover_text(textos.collapse_track);
        if colapsar {
            clique_chevron = true;
            if let Some(pos) = config
                .pistas_colapsadas
                .iter()
                .position(|id| id.as_str() == fila.id)
            {
                config.pistas_colapsadas.remove(pos);
            } else {
                config.pistas_colapsadas.push(fila.id.clone());
            }
            crate::ui::editor::guardar_medidas(ui.ctx(), &config);
        }
    }

    if hablantes.is_empty() {
        painter.text(
            egui::pos2(gutter.max.x + tema::ESP_8, y_scroll + tema::ESP_8),
            egui::Align2::LEFT_TOP,
            textos.pista_sin_hablantes,
            tema::fuente(Estilo::Pequeno),
            tema::TEXTO_2,
        );
    }

    // Waveform del carril de audio: se pintan los picos ya decodificados,
    // nunca se recalculan por frame.
    if let Some(picos) = &state.picos {
        let altura = filas[0].alto - tema::ESP_8;
        let pico = picos
            .max
            .iter()
            .copied()
            .fold(0.0_f32, |a, b| a.max(b))
            .max(0.001);
        for xpx in (0..area.width() as i32).step_by(3) {
            let segundo = vista.tiempo_de(xpx as f32);
            let indice = (segundo * picos.por_segundo as f64) as usize;
            let Some(&valor) = picos.max.get(indice) else {
                continue;
            };
            let alto = ((valor / pico).clamp(0.0, 1.0) * altura * 0.5).max(tema::GROSOR_BORDE);
            let x = area.min.x + xpx as f32;
            let centro = y_audio + filas[0].alto / 2.0;
            painter.line_segment(
                [egui::pos2(x, centro - alto), egui::pos2(x, centro + alto)],
                egui::Stroke::new(tema::GROSOR_SPLITTER / 2.0, tema::COLOR_WAVEFORM),
            );
        }
    }

    // Retícula tenue que baja de cada marca de la regla a los carriles.
    let mut t = (vista.inicio / paso).ceil() * paso;
    while t <= vista.tiempo_de(area.width()) + paso {
        let x = area.min.x + vista.x_de(t);
        if (area.min.x..=area.max.x).contains(&x) {
            painter.line_segment(
                [egui::pos2(x, y_audio), egui::pos2(x, rect.max.y)],
                egui::Stroke::new(tema::GROSOR_BORDE, tema::BORDE),
            );
        }
        t += paso;
    }

    // Clips: el color del hablante manda; el azul marca selección.
    let puntero = ui.ctx().input(|entrada| entrada.pointer.latest_pos());
    let mut id_bajo_puntero: Option<String> = None;
    for caption in proyecto.captions.iter() {
        let indice = match caption.hablante_id.as_deref() {
            Some(id) => filas.iter().position(|fila| fila.id == id),
            None => Some(0),
        };
        let Some(indice) = indice else {
            continue;
        };
        let x0 = area.min.x + vista.x_de(caption.inicio);
        let x1 = area.min.x + vista.x_de(caption.fin);
        if x1 <= area.min.x || x0 >= area.max.x {
            continue;
        }
        let y = y_de_fila(&filas, indice, y_audio, scroll) + tema::ESP_8;
        if indice > 0 && (y + alto_pista <= y_scroll || y >= rect.max.y) {
            continue;
        }
        let clip = egui::Rect::from_min_max(
            egui::pos2(x0.max(area.min.x), y),
            egui::pos2(x1.min(area.max.x), y + filas[indice].alto - tema::ESP_16),
        );
        let identidad = caption.hablante_id.as_deref().and_then(|id| {
            hablantes
                .iter()
                .position(|hablante| hablante.id == id)
                .map(|indice| (indice, &hablantes[indice]))
        });
        let seleccionado = state.editor.seleccion.contains(&caption.id);
        let color = identidad.map_or(tema::DIM, |(indice, hablante)| {
            tema::color(rod_core::color_identidad(hablante, indice))
        });
        let relleno = if identidad.is_some() {
            tema::mezclar(color, tema::SUPERFICIE, 0.78)
        } else {
            tema::mezclar(tema::COLOR_SIN_HABLANTE, tema::SUPERFICIE, 0.85)
        };
        painter.rect_filled(clip, egui::CornerRadius::same(tema::RADIO_CLIP), relleno);
        if identidad.is_none() && !seleccionado && proyecto.captions.len() <= 120 {
            pintar_borde_punteado(painter, clip, tema::COLOR_SIN_HABLANTE);
        } else {
            painter.rect_stroke(
                clip,
                egui::CornerRadius::same(tema::RADIO_CLIP),
                egui::Stroke::new(
                    if seleccionado {
                        tema::GROSOR_FOCO
                    } else {
                        tema::GROSOR_BORDE
                    },
                    color_borde_clip(identidad, seleccionado),
                ),
                egui::StrokeKind::Inside,
            );
        }
        if clip.width() > tema::ANCHO_DRAG_CONTORNO {
            painter.text(
                clip.min + egui::vec2(tema::ESP_4, tema::ESP_4),
                egui::Align2::LEFT_TOP,
                tema::acortar(
                    &caption.texto,
                    (clip.width() / tema::ESP_8).max(4.0) as usize,
                ),
                tema::fuente(Estilo::Pequeno),
                tema::TEXTO,
            );
        }
        if puntero.is_some_and(|punto| clip.contains(punto)) {
            id_bajo_puntero = Some(caption.id.clone());
        }
    }

    // El menú se engancha a la respuesta del timeline, no a un widget por clip:
    // un `interact` bajo el cursor se comía el clic y `pulsar` nunca elegía el
    // caption tocado. Con 543 líneas además costaba varios ms por frame.
    if let Some(id) = id_bajo_puntero {
        respuesta.context_menu(|ui| {
            menu_contexto(ui, &id, &mut mensajes, textos);
        });
    }

    // Playhead con tirador y, si el tiempo salió del área, marcador de borde.
    let x = area.min.x + vista.x_de(playhead(state));
    if (area.min.x..=area.max.x).contains(&x) {
        painter.line_segment(
            [
                egui::pos2(x, rect.min.y + tema::ESP_12),
                egui::pos2(x, rect.max.y),
            ],
            egui::Stroke::new(tema::GROSOR_FOCO, tema::ACENTO),
        );
        painter.add(egui::Shape::closed_line(
            vec![
                egui::pos2(x - tema::ESP_8, rect.min.y + tema::ESP_2),
                egui::pos2(x + tema::ESP_8, rect.min.y + tema::ESP_2),
                egui::pos2(x, rect.min.y + tema::ESP_12),
            ],
            egui::Stroke::new(tema::GROSOR_BORDE, tema::ACENTO),
        ));
    } else if let Some((borde, izquierda)) = marca_borde(x, area.min.x, area.max.x) {
        painter.line_segment(
            [egui::pos2(borde, rect.min.y), egui::pos2(borde, rect.max.y)],
            egui::Stroke::new(tema::GROSOR_BORDE, tema::ACENTO),
        );
        let y = rect.min.y + ALTO_REGLA / 2.0;
        let (punta, base) = if izquierda {
            (
                egui::pos2(borde + tema::GROSOR_BORDE, y),
                [
                    egui::pos2(borde + tema::ESP_8, y - tema::ESP_4),
                    egui::pos2(borde + tema::ESP_8, y + tema::ESP_4),
                ],
            )
        } else {
            (
                egui::pos2(borde - tema::GROSOR_BORDE, y),
                [
                    egui::pos2(borde - tema::ESP_8, y - tema::ESP_4),
                    egui::pos2(borde - tema::ESP_8, y + tema::ESP_4),
                ],
            )
        };
        painter.add(egui::Shape::convex_polygon(
            vec![punta, base[0], base[1]],
            tema::ACENTO,
            egui::Stroke::NONE,
        ));
    }

    // Scrollbar: solo cuando hay pistas que no caben bajo la regla fija.
    if max_scroll > 0.0 {
        painter.rect_filled(barra, egui::CornerRadius::ZERO, tema::LATERAL);
        let pulgar_alto = (barra.height() * alto_scroll / total_scroll)
            .max(tema::ALTO_CONTROL)
            .min(barra.height());
        let pulgar_y = barra.min.y + scroll / max_scroll * (barra.height() - pulgar_alto);
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(barra.min.x, pulgar_y),
                egui::pos2(barra.max.x, pulgar_y + pulgar_alto),
            ),
            egui::CornerRadius::same(tema::RADIO_CONTROL as u8),
            tema::BORDE_CONTROL,
        );
        if (respuesta_barra.clicked() || respuesta_barra.dragged())
            && let Some(pos) = respuesta_barra.interact_pointer_pos()
        {
            mensajes.push(MensajeEditor::ScrollPistas(
                (pos.y - barra.min.y) / barra.height() * max_scroll,
            ));
        }
    }

    // Los eventos llegan en coordenadas locales del rect del timeline: el
    // margen de nombres ya está descontado por la geometría del eje.
    // `punto_en_fila` devuelve la y lógica de `ALTO_CARRIL` que espera `pulsar`.
    let punto_en = |pos: egui::Pos2| -> timeline::Punto {
        let rel = pos - rect.min;
        let fila = fila_en_y(&filas, y_audio - rect.min.y, scroll, rel.y).unwrap_or(0);
        punto_en_fila(rel.x - gutter_ancho, filas[fila].modelo)
    };
    if respuesta.clicked()
        && !clique_chevron
        && let Some(pos) = respuesta.interact_pointer_pos()
    {
        let punto = punto_en(pos);
        mensajes.push(MensajeEditor::Timeline(Mensaje::Pulsar(punto)));
        mensajes.push(MensajeEditor::Timeline(Mensaje::Soltar(punto)));
    }
    if respuesta.drag_started()
        && !divisor_gutter.dragged()
        && let Some(pos) = respuesta.interact_pointer_pos()
    {
        let tiempo = vista.tiempo_de(punto_en(pos).x).max(0.0);
        mensajes.push(MensajeEditor::ScrubInicio(tiempo));
        mensajes.push(MensajeEditor::Timeline(Mensaje::Pulsar(punto_en(pos))));
    }
    if respuesta.dragged()
        && !divisor_gutter.dragged()
        && let Some(pos) = respuesta.interact_pointer_pos()
    {
        let tiempo = vista.tiempo_de(punto_en(pos).x).max(0.0);
        mensajes.push(MensajeEditor::ScrubMover(tiempo));
        mensajes.push(MensajeEditor::Timeline(Mensaje::Arrastrar(punto_en(pos))));
    }
    if respuesta.drag_stopped()
        && !divisor_gutter.dragged()
        && let Some(pos) = respuesta.interact_pointer_pos()
    {
        let tiempo = vista.tiempo_de(punto_en(pos).x).max(0.0);
        mensajes.push(MensajeEditor::ScrubFin(tiempo));
        mensajes.push(MensajeEditor::Timeline(Mensaje::Soltar(punto_en(pos))));
    }
    let (modificadores, delta) = ui.input(|entrada| (entrada.modifiers, entrada.raw_scroll_delta));
    mensajes.push(MensajeEditor::Modificadores(modificadores));
    match gesto_rueda(modificadores, delta) {
        Rueda::Zoom(lineas) if lineas != 0.0 => {
            let cursor = ui
                .input(|entrada| entrada.pointer.latest_pos())
                .unwrap_or_else(|| area.center());
            mensajes.push(MensajeEditor::Timeline(Mensaje::Rueda {
                lineas,
                punto: punto_en(cursor),
            }));
        }
        Rueda::Pan(lineas) if lineas != 0.0 => {
            let cursor = ui
                .input(|entrada| entrada.pointer.latest_pos())
                .unwrap_or_else(|| area.center());
            mensajes.push(MensajeEditor::Timeline(Mensaje::Rueda {
                lineas,
                punto: punto_en(cursor),
            }));
        }
        Rueda::Pistas(lineas) if lineas != 0.0 => {
            mensajes.push(MensajeEditor::ScrollPistas(
                (scroll + lineas * alto_pista).clamp(0.0, max_scroll),
            ));
        }
        _ => {}
    }
    let _ = timeline::UMBRAL_CLIC;
    mensajes
}

#[cfg(test)]
mod tests {
    use crate::timeline::Punto as Point;

    use super::{marca_borde, playhead, punto_en_fila};
    use crate::editor_app::{EditorApp, MensajeEditor, update};
    use crate::timeline::{Mensaje, Vista};

    fn editor() -> EditorApp {
        let video = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../testdata/clip-15s-2audio.mp4"
        );
        let mut editor = EditorApp::desde_proyecto_en(
            rod_core::Proyecto::nuevo(video),
            None,
            0.0,
            &std::env::temp_dir(),
        )
        .expect("video de prueba");
        editor.editor.agregar(rod_core::Caption {
            id: "cap-arrastre".into(),
            inicio: 1.0,
            fin: 2.0,
            texto: "uno".into(),
            hablante_id: None,
        });
        update(&mut editor, MensajeEditor::Timeline(Mensaje::Ancho(800.0)));
        editor.vista = Vista::nueva(0.02);
        editor
    }

    #[test]
    fn arrastre_termina_en_el_tiempo_soltado_y_es_un_paso() {
        let mut editor = editor();
        let y = super::ALTO_REGLA + crate::timeline::ALTO_CARRIL / 2.0;
        let x_inicio = editor.vista.x_de(1.5);
        let x_fin = editor.vista.x_de(2.5);

        update(
            &mut editor,
            MensajeEditor::Timeline(Mensaje::Pulsar(Point::new(x_inicio, y))),
        );
        update(
            &mut editor,
            MensajeEditor::Timeline(Mensaje::Arrastrar(Point::new(x_fin, y))),
        );
        update(
            &mut editor,
            MensajeEditor::Timeline(Mensaje::Soltar(Point::new(x_fin, y))),
        );

        let caption = &editor.editor.proyecto().captions[0];
        assert!((caption.fin - 3.0).abs() < 0.02, "fin: {}", caption.fin);
        assert!(editor.editor.puede_deshacer());
    }

    #[test]
    fn sin_time_pos_el_playhead_es_cero() {
        let mut editor = editor();
        editor.position = f64::NAN;
        assert_eq!(playhead(&editor), 0.0);
    }

    #[test]
    fn un_carril_contraido_sigue_apuntando_a_su_modelo() {
        // Aunque el carril 3 se vea a 32 px, `pulsar` sigue recibiendo la y
        // lógica de ALTO_CARRIL: el hablante es el mismo en los dos casos.
        let punto = punto_en_fila(200.0, 3);
        let esperado = super::ALTO_REGLA + 3.0 * crate::timeline::ALTO_CARRIL;
        assert!((punto.x - 200.0).abs() < 1e-6);
        assert!(punto.y > esperado && punto.y < esperado + crate::timeline::ALTO_CARRIL);
    }

    #[test]
    fn el_marcador_solo_aparece_fuera_del_area() {
        assert_eq!(marca_borde(50.0, 10.0, 100.0), None);
        assert_eq!(marca_borde(10.0, 10.0, 100.0), None);
        assert_eq!(marca_borde(100.0, 10.0, 100.0), None);
        assert_eq!(marca_borde(-1.0, 10.0, 100.0), Some((10.0, true)));
        assert_eq!(marca_borde(120.0, 10.0, 100.0), Some((100.0, false)));
    }
}
