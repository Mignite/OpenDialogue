//! Timeline dibujada con `egui::Painter`: regla con marcas, carriles con
//! nombre, clips con el color del hablante y playhead. El significado de cada
//! gesto sigue en `timeline::pulsar`, que es código probado.

use egui::{Ui, Vec2};

use crate::editor_app::{EditorApp, MensajeEditor};
use crate::timeline::{self, ALTO_CARRIL, ALTO_REGLA, Mensaje};
use crate::ui::tema::{self, Estilo};

/// Margen izquierdo para los nombres de carril. El eje empieza después.
const GUTTER: f32 = 104.0;

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

fn punto(rel: egui::Vec2) -> timeline::Punto {
    timeline::Punto::new(rel.x - GUTTER, rel.y)
}

pub fn playhead(state: &EditorApp) -> f64 {
    if state.position.is_finite() && state.duration.is_finite() {
        state.tiempo_ui()
    } else {
        0.0
    }
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

pub fn mostrar(ui: &mut Ui, state: &mut EditorApp) -> Vec<MensajeEditor> {
    let (rect, respuesta) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), ui.available_height()),
        egui::Sense::click_and_drag(),
    );
    let painter = ui.painter();
    painter.rect_filled(rect, egui::CornerRadius::ZERO, tema::SUPERFICIE);

    let gutter = egui::Rect::from_min_max(rect.min, egui::pos2(rect.min.x + GUTTER, rect.max.y));
    let area = egui::Rect::from_min_max(egui::pos2(gutter.max.x, rect.min.y), rect.max);
    painter.rect_filled(gutter, egui::CornerRadius::ZERO, tema::PANEL);
    painter.line_segment(
        [
            egui::pos2(gutter.max.x, rect.min.y),
            egui::pos2(gutter.max.x, rect.max.y),
        ],
        egui::Stroke::new(1.0_f32, tema::BORDE),
    );

    // Ancho real del eje. Solo se envía cuando cambia para no sobrescribir
    // otros mensajes del frame.
    let mut mensajes = Vec::new();
    if (state.ancho_timeline - area.width()).abs() > 0.5 {
        mensajes.push(MensajeEditor::Timeline(Mensaje::Ancho(area.width())));
    }

    let vista = state.vista;
    let hablantes = &state.editor.proyecto().hablantes;
    // Una pista por hablante; el bucle de dibujo ya para al llegar al borde.
    let filas = hablantes.len() + 1;

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
                    egui::pos2(x, rect.min.y + 15.0),
                    egui::pos2(x, rect.min.y + ALTO_REGLA),
                ],
                egui::Stroke::new(1.0_f32, tema::BORDE),
            );
            painter.text(
                egui::pos2(x + 4.0, rect.min.y + 8.0),
                egui::Align2::LEFT_CENTER,
                etiqueta_regla(t, paso),
                tema::fuente(Estilo::Timecode),
                tema::TEXTO_3,
            );
        }
        t += paso;
    }
    painter.text(
        gutter.min + egui::vec2(10.0, 8.0),
        egui::Align2::LEFT_CENTER,
        "TIEMPO",
        tema::fuente(Estilo::Pequeno),
        tema::TEXTO_3,
    );

    // Carriles: fondo alterno y nombre con el punto semántico del hablante.
    for fila in 0..filas {
        let y = rect.min.y + ALTO_REGLA + fila as f32 * ALTO_CARRIL;
        if y + ALTO_CARRIL > rect.max.y {
            break;
        }
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(area.min.x, y),
                egui::pos2(area.max.x, y + ALTO_CARRIL - 1.0),
            ),
            egui::CornerRadius::ZERO,
            if fila % 2 == 0 {
                tema::PANEL
            } else {
                tema::LATERAL
            },
        );
        painter.line_segment(
            [
                egui::pos2(area.min.x, y + ALTO_CARRIL - 1.0),
                egui::pos2(area.max.x, y + ALTO_CARRIL - 1.0),
            ],
            egui::Stroke::new(1.0_f32, tema::BORDE),
        );
        let (nombre, color) = if fila == 0 {
            ("Audio".to_string(), tema::TEXTO_3)
        } else {
            let hablante = &hablantes[fila - 1];
            let nombre = if hablante.nombre.is_empty() {
                hablante.tecla.clone()
            } else {
                hablante.nombre.clone()
            };
            (
                tema::acortar(&nombre, 12),
                tema::color(rod_core::color_identidad(hablante, fila - 1)),
            )
        };
        let centro = ALTO_REGLA + fila as f32 * ALTO_CARRIL + ALTO_CARRIL / 2.0;
        painter.circle_filled(gutter.min + egui::vec2(12.0, centro), 3.0, color);
        painter.text(
            gutter.min + egui::vec2(22.0, centro),
            egui::Align2::LEFT_CENTER,
            tema::acortar(&nombre, 12),
            tema::fuente(Estilo::Pequeno),
            tema::TEXTO_2,
        );
    }

    // Waveform apagada en el carril de audio: se pinta de los picos ya
    // decodificados, nunca se recalcula por frame.
    if let Some(picos) = &state.picos {
        let base = rect.min.y + ALTO_REGLA;
        let altura = ALTO_CARRIL - 6.0;
        let color_onda = tema::color("#3A4A5A");
        for xpx in (0..area.width() as i32).step_by(2) {
            let segundo = vista.tiempo_de(xpx as f32);
            let indice = (segundo * picos.por_segundo as f64) as usize;
            let Some(&valor) = picos.max.get(indice) else {
                continue;
            };
            let alto = (valor.clamp(0.0, 1.0) * altura * 0.5).max(1.0);
            let x = area.min.x + xpx as f32;
            let centro = base + ALTO_CARRIL / 2.0;
            painter.line_segment(
                [egui::pos2(x, centro - alto), egui::pos2(x, centro + alto)],
                egui::Stroke::new(2.0_f32, color_onda),
            );
        }
    }

    // Retícula tenue que baja de cada marca de la regla a los carriles.
    let mut t = (vista.inicio / paso).ceil() * paso;
    while t <= vista.tiempo_de(area.width()) + paso {
        let x = area.min.x + vista.x_de(t);
        if (area.min.x..=area.max.x).contains(&x) {
            painter.line_segment(
                [
                    egui::pos2(x, rect.min.y + ALTO_REGLA),
                    egui::pos2(x, rect.max.y),
                ],
                egui::Stroke::new(1.0_f32, tema::BORDE.gamma_multiply(0.45)),
            );
        }
        t += paso;
    }

    // Clips: el color del hablante manda; el azul marca selección.
    for caption in state.editor.proyecto().captions.iter() {
        let fila = rod_core::caption_row_index(caption.hablante_id.as_deref(), hablantes);
        if fila >= filas {
            continue;
        }
        let x0 = area.min.x + vista.x_de(caption.inicio);
        let x1 = area.min.x + vista.x_de(caption.fin);
        if x1 <= area.min.x || x0 >= area.max.x {
            continue;
        }
        let y = rect.min.y + ALTO_REGLA + fila as f32 * ALTO_CARRIL + 7.0;
        let clip = egui::Rect::from_min_max(
            egui::pos2(x0.max(area.min.x), y),
            egui::pos2(x1.min(area.max.x), y + ALTO_CARRIL - 14.0),
        );
        let color = caption
            .hablante_id
            .as_deref()
            .and_then(|id| {
                hablantes
                    .iter()
                    .position(|hablante| hablante.id == id)
                    .map(|indice| (indice, &hablantes[indice]))
            })
            .map(|(indice, hablante)| tema::color(rod_core::color_identidad(hablante, indice)))
            .unwrap_or(tema::DIM);
        let seleccionado = state.editor.seleccion.contains(&caption.id);
        painter.rect_filled(
            clip,
            egui::CornerRadius::same(3),
            color.gamma_multiply(0.42),
        );
        painter.rect_stroke(
            clip,
            egui::CornerRadius::same(3),
            egui::Stroke::new(
                if seleccionado { 2.0_f32 } else { 1.0_f32 },
                if seleccionado { tema::ACENTO } else { color },
            ),
            egui::StrokeKind::Inside,
        );
        if clip.width() > 56.0 {
            painter.text(
                clip.min + egui::vec2(5.0, 4.0),
                egui::Align2::LEFT_TOP,
                tema::acortar(&caption.texto, (clip.width() / 7.0).max(4.0) as usize),
                tema::fuente(Estilo::Pequeno),
                tema::TEXTO,
            );
        }
    }

    // Playhead con tirador.
    let x = area.min.x + vista.x_de(playhead(state));
    if (area.min.x..=area.max.x).contains(&x) {
        painter.line_segment(
            [egui::pos2(x, rect.min.y + 10.0), egui::pos2(x, rect.max.y)],
            egui::Stroke::new(1.5_f32, tema::ACENTO),
        );
        painter.add(egui::Shape::closed_line(
            vec![
                egui::pos2(x - 5.0, rect.min.y + 2.0),
                egui::pos2(x + 5.0, rect.min.y + 2.0),
                egui::pos2(x, rect.min.y + 10.0),
            ],
            egui::Stroke::new(1.0_f32, tema::ACENTO),
        ));
    }

    // Los eventos llegan en coordenadas locales del rect del timeline: el
    // margen de nombres ya está descontado por la geometría del eje.
    if respuesta.drag_started()
        && let Some(pos) = respuesta.interact_pointer_pos()
    {
        let tiempo = vista.tiempo_de(punto(pos - rect.min).x).max(0.0);
        mensajes.push(MensajeEditor::ScrubInicio(tiempo));
        mensajes.push(MensajeEditor::Timeline(Mensaje::Pulsar(punto(
            pos - rect.min,
        ))));
    }
    if respuesta.dragged()
        && let Some(pos) = respuesta.interact_pointer_pos()
    {
        let tiempo = vista.tiempo_de(punto(pos - rect.min).x).max(0.0);
        mensajes.push(MensajeEditor::ScrubMover(tiempo));
        mensajes.push(MensajeEditor::Timeline(Mensaje::Arrastrar(punto(
            pos - rect.min,
        ))));
    }
    if respuesta.drag_stopped()
        && let Some(pos) = respuesta.interact_pointer_pos()
    {
        let tiempo = vista.tiempo_de(punto(pos - rect.min).x).max(0.0);
        mensajes.push(MensajeEditor::ScrubFin(tiempo));
        mensajes.push(MensajeEditor::Timeline(Mensaje::Soltar(punto(
            pos - rect.min,
        ))));
    }
    let (modificadores, lineas) =
        ui.input(|entrada| (entrada.modifiers, entrada.raw_scroll_delta.y / 40.0));
    mensajes.push(MensajeEditor::Modificadores(modificadores));
    if lineas != 0.0 {
        let cursor = ui
            .input(|entrada| entrada.pointer.latest_pos())
            .unwrap_or_else(|| area.center());
        mensajes.push(MensajeEditor::Timeline(Mensaje::Rueda {
            lineas,
            punto: punto(cursor - rect.min),
        }));
    }
    let _ = timeline::UMBRAL_CLIC;
    mensajes
}

#[cfg(test)]
mod tests {
    use crate::timeline::Punto as Point;

    use super::playhead;
    use crate::editor_app::{EditorApp, MensajeEditor, update};
    use crate::timeline::{Mensaje, Vista};

    fn editor() -> EditorApp {
        let video = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../testdata/clip-15s-2audio.mp4"
        );
        let mut editor = EditorApp::desde_video(video).expect("video de prueba");
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
}
