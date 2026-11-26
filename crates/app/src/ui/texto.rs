//! Barra única de edición del texto de un subtítulo, bajo el video.

use std::time::Duration;

use egui::{Key, RichText, Stroke, TextEdit, Vec2};

use crate::editor_app::{EditorApp, EstadoTexto, MensajeEditor};
use crate::textos::Textos;
use crate::ui::kit::{self, Icono};
use crate::ui::tema::{self, Estilo};

/// Alto extra que la barra añade al transporte cuando crece a varias líneas.
pub fn alto_extra(state: &EditorApp, _ancho: f32) -> f32 {
    (state.texto.lineas.saturating_sub(1)) as f32 * tema::ALTO_BARRA_TEXTO
}

/// Único formateador de timecode: `hh:mm:ss.cc`. La lista, el transporte y la
/// regla de la timeline muestran el mismo formato; las centésimas se redondean
/// para que la duración reste exactamente los tiempos visibles (C2/C3).
pub fn tiempo(segundos: f64) -> String {
    let centesimas = (segundos.max(0.0) * 100.0).round() as u64;
    format!(
        "{:02}:{:02}:{:02}.{:02}",
        centesimas / 360_000,
        (centesimas / 6_000) % 60,
        (centesimas / 100) % 60,
        centesimas % 100
    )
}

/// Duración calculada sobre las mismas centésimas que `tiempo`.
pub fn duracion(inicio: f64, fin: f64) -> String {
    let inicio = (inicio.max(0.0) * 100.0).round() as u64;
    let fin = (fin.max(0.0) * 100.0).round() as u64;
    let centesimas = fin.saturating_sub(inicio);
    format!("{}.{:02} s", centesimas / 100, centesimas % 100)
}

/// Ancho que el transporte pinta a la derecha del campo: bucle, volumen y
/// velocidad. Se resta **antes** de fijar el ancho del `TextEdit` para que los
/// controles no se pinten encima del texto (C1).
pub fn ancho_reservado_derecha() -> f32 {
    tema::ANCHO_DROPDOWN_VELOCIDAD + tema::ANCHO_BOTON_28 * 2.0 + tema::ESP_4 * 2.0 + tema::ESP_8
}

/// Una sesión de tipeo se cierra sola tras ~1 s sin teclear.
pub fn sesion_vencida(ultimo: Option<std::time::Instant>, ahora: std::time::Instant) -> bool {
    ultimo.is_some_and(|instante| ahora.duration_since(instante) > Duration::from_millis(1000))
}

/// Sincroniza el borrador con la línea activa (selección o playhead). Cuando
/// no hay línea, limpia el campo en vez de conservar el texto de la anterior.
pub fn sincronizar_objetivo(state: &mut EditorApp) {
    let objetivo = EstadoTexto::objetivo(state);
    if state.texto.objetivo == objetivo {
        return;
    }
    let texto = objetivo
        .as_ref()
        .and_then(|id| {
            state
                .editor
                .proyecto()
                .captions
                .iter()
                .find(|caption| &caption.id == id)
        })
        .map(|caption| caption.texto.clone())
        .unwrap_or_default();
    state.texto.objetivo = objetivo;
    state.texto.borrador = texto.clone();
    state.texto.original = texto;
    state.texto.lineas = 1;
}

pub fn color_borde_campo(hay_linea: bool, foco: bool) -> egui::Color32 {
    if foco {
        tema::ACENTO
    } else if hay_linea {
        tema::BORDE
    } else {
        tema::COLOR_SIN_HABLANTE
    }
}

/// Al enfocar el campo, la línea que se edita pasa a estar seleccionada arriba.
pub fn mensaje_al_enfocar(state: &EditorApp) -> Option<MensajeEditor> {
    EstadoTexto::objetivo(state).map(MensajeEditor::SeleccionarSinSeek)
}

fn pintar_borde_rayado(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    /// Paso y longitud de trazo del borde del campo sin línea activa.
    const PASO: f32 = tema::ANCHO_SCROLLBAR;
    const TRAZO: f32 = PASO / 2.0;
    let trazo = Stroke::new(tema::GROSOR_BORDE, color);
    let mut x = rect.min.x;
    while x < rect.max.x {
        painter.line_segment(
            [
                egui::pos2(x, rect.min.y),
                egui::pos2((x + TRAZO).min(rect.max.x), rect.min.y),
            ],
            trazo,
        );
        painter.line_segment(
            [
                egui::pos2(x, rect.max.y),
                egui::pos2((x + TRAZO).min(rect.max.x), rect.max.y),
            ],
            trazo,
        );
        x += PASO;
    }
}

fn cps(state: &EditorApp) -> f32 {
    let Some(id) = EstadoTexto::objetivo(state) else {
        return 0.0;
    };
    let Some(caption) = state
        .editor
        .proyecto()
        .captions
        .iter()
        .find(|caption| caption.id == id)
    else {
        return 0.0;
    };
    let duracion = (caption.fin - caption.inicio).max(0.001) as f32;
    caption.texto.chars().count() as f32 / duracion
}

/// Dibuja la barra. Devuelve mensajes para el bucle del editor.
pub fn mostrar(ui: &mut egui::Ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor> {
    let mut mensajes = Vec::new();
    sincronizar_objetivo(state);
    let objetivo = EstadoTexto::objetivo(state);
    let hablante = objetivo.as_ref().and_then(|id| {
        let hablante_id = state
            .editor
            .proyecto()
            .captions
            .iter()
            .find(|caption| &caption.id == id)
            .and_then(|caption| caption.hablante_id.clone());
        state
            .editor
            .proyecto()
            .hablantes
            .iter()
            .find(|hablante| Some(&hablante.id) == hablante_id.as_ref())
            .cloned()
    });
    let nombre = hablante
        .as_ref()
        .map(|hablante| hablante.nombre.clone())
        .unwrap_or_else(|| textos.sin_hablante.to_owned());
    let color = hablante
        .as_ref()
        .map(|hablante| tema::color_de_hex(&hablante.color))
        .unwrap_or(tema::TEXTO_3);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = tema::ESP_8;
        kit::punto(ui, color, tema::ESP_8);
        // Nombre + chevron en un solo `LayoutJob`: el glifo de Phosphor entra
        // como reserva de Proportional, así que comparte botón sin familia propia.
        let mut etiqueta = egui::text::LayoutJob::default();
        etiqueta.append(
            &nombre,
            0.0,
            egui::text::TextFormat {
                font_id: tema::fuente(Estilo::Pequeno),
                color: tema::TEXTO,
                ..Default::default()
            },
        );
        etiqueta.append(
            kit::glifo(Icono::ChevronAbajo),
            tema::ESP_4,
            egui::text::TextFormat {
                font_id: egui::FontId::proportional(tema::TAMANO_ICONO),
                color: tema::TEXTO_2,
                ..Default::default()
            },
        );
        let chip = egui::containers::menu::MenuButton::from_button(
            egui::Button::new(etiqueta)
                .frame(false)
                .min_size(egui::vec2(tema::ANCHO_BOTON_28, tema::ALTO_BOTON_CHICO)),
        )
        .ui(ui, |ui| {
            if ui.button(textos.sin_hablante).clicked() {
                mensajes.push(MensajeEditor::AsignarHablante(None));
                ui.close();
            }
            for hablante in &state.editor.proyecto().hablantes {
                let nombre = RichText::new(&hablante.nombre).font(tema::fuente(Estilo::Pequeno));
                if ui.button(nombre).clicked() {
                    mensajes.push(MensajeEditor::AsignarHablante(Some(hablante.id.clone())));
                    ui.close();
                }
            }
        });
        let _ = chip;

        let reserva = ancho_reservado_derecha();
        let libre = (ui.available_width() - reserva).max(0.0);
        let mostrar_cps = libre >= tema::ANCHO_MINIMO_TEXTO;
        // Ancho de "00.0 CPS" con la fuente pequeña: dos keycaps y un espacio.
        let ancho_cps = if mostrar_cps {
            tema::ANCHO_BOTON_28 * 2.0 + tema::ESP_8
        } else {
            0.0
        };
        let ancho = (libre - ancho_cps - tema::ESP_8).max(tema::ESP_72);
        // Con una sola línea usamos el TextEdit simple (más barato); si el
        // usuario pide salto, cambiamos a multicampo en este mismo frame para
        // que egui inserte el '\n' en la posición correcta del cursor.
        if ui.input(|entrada| entrada.key_pressed(Key::Enter) && entrada.modifiers.shift) {
            state.texto.lineas = state.texto.lineas.saturating_add(1).min(3);
        }
        let lineas = state.texto.lineas.clamp(1, 3) as usize;
        let pista = if objetivo.is_some() {
            ""
        } else {
            textos.sin_seleccion
        };
        let respuesta = if lineas == 1 {
            ui.add(
                TextEdit::singleline(&mut state.texto.borrador)
                    .hint_text(pista)
                    .font(tema::fuente(Estilo::Cuerpo))
                    .desired_width(ancho)
                    .frame(false),
            )
        } else {
            ui.add(
                TextEdit::multiline(&mut state.texto.borrador)
                    .hint_text(pista)
                    .font(tema::fuente(Estilo::Cuerpo))
                    .desired_rows(lineas)
                    .min_size(Vec2::new(ancho, tema::ALTO_BARRA_TEXTO * lineas as f32))
                    .frame(false),
            )
        };
        let color_borde = color_borde_campo(objetivo.is_some(), false);
        if state.texto.foco || respuesta.has_focus() {
            kit::anillo_foco(ui, &respuesta);
        } else if objetivo.is_some() {
            ui.painter().rect_stroke(
                respuesta.rect.expand(tema::ESP_2),
                tema::radio_control(),
                Stroke::new(tema::GROSOR_BORDE, color_borde),
                egui::StrokeKind::Inside,
            );
        } else {
            pintar_borde_rayado(
                ui.painter(),
                respuesta.rect.expand(tema::ESP_2),
                color_borde,
            );
        }
        if respuesta.clicked()
            && let Some(mensaje) = mensaje_al_enfocar(state)
        {
            mensajes.push(mensaje);
        }
        let solicitar_foco = state.texto.foco && !respuesta.has_focus();
        if solicitar_foco {
            respuesta.request_focus();
        }
        let mut cerrada = false;
        if respuesta.has_focus() {
            if objetivo.is_some() && state.texto.borrador != state.texto.original {
                update_texto(state);
            }
            if respuesta.changed() {
                state.texto.ultimo_tipeo = Some(std::time::Instant::now());
            }
            if sesion_vencida(state.texto.ultimo_tipeo, std::time::Instant::now()) {
                EstadoTexto::cerrar_sesion(state);
                respuesta.surrender_focus();
                cerrada = true;
            } else if state.texto.ultimo_tipeo.is_some() {
                ui.ctx().request_repaint_after(Duration::from_millis(200));
            }
            if !cerrada
                && ui.input(|entrada| entrada.key_pressed(Key::Enter) && !entrada.modifiers.shift)
            {
                if state.texto.borrador.ends_with('\n') {
                    state.texto.borrador.pop();
                }
                EstadoTexto::cerrar_sesion(state);
                respuesta.surrender_focus();
                cerrada = true;
            } else if !cerrada && ui.input(|entrada| entrada.key_pressed(Key::Escape)) {
                EstadoTexto::revertir(state);
                respuesta.surrender_focus();
                cerrada = true;
            } else if !cerrada && ui.input(|entrada| entrada.key_pressed(Key::ArrowUp)) {
                crate::editor_app::ciclar_bajo_playhead(state, -1);
                sincronizar_objetivo(state);
            } else if !cerrada && ui.input(|entrada| entrada.key_pressed(Key::ArrowDown)) {
                crate::editor_app::ciclar_bajo_playhead(state, 1);
                sincronizar_objetivo(state);
            } else {
                state.texto.lineas =
                    (state.texto.borrador.matches('\n').count().clamp(0, 2) + 1) as u8;
            }
        }
        if respuesta.lost_focus() && state.texto.foco {
            EstadoTexto::cerrar_sesion(state);
        }
        if !cerrada && !solicitar_foco && !state.texto.foco {
            state.texto.foco = respuesta.has_focus();
        }
        state.texto.nueva_linea = objetivo.is_none();

        if mostrar_cps {
            let valor = cps(state);
            let color = if valor > tema::CPS_MAXIMO {
                tema::AMBAR
            } else {
                tema::TEXTO_2
            };
            ui.add_space(tema::ESP_4);
            ui.label(
                RichText::new(format!("{:.1} CPS", valor))
                    .font(tema::fuente(Estilo::Pequeno))
                    .color(color),
            );
        }
    });
    mensajes
}

fn update_texto(state: &mut EditorApp) {
    let Some(id) = EstadoTexto::objetivo(state) else {
        return;
    };
    crate::editor_app::update(
        state,
        MensajeEditor::EditarEnVivo(id, state.texto.borrador.clone()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timecode_y_duracion_comparten_base() {
        assert_eq!(tiempo(473.13), "00:07:53.13");
        assert_eq!(tiempo(0.32), "00:00:00.32");
        // Antes la lista mostraba 1.54 s para 12.46 − 10.93.
        assert_eq!(duracion(10.93, 12.46), "1.53 s");
        assert_eq!(duracion(2.0, 2.0), "0.00 s");
    }

    #[test]
    fn la_reserva_deja_hueco_para_bucle_volumen_y_velocidad() {
        assert_eq!(
            ancho_reservado_derecha(),
            tema::ANCHO_DROPDOWN_VELOCIDAD
                + tema::ANCHO_BOTON_28 * 2.0
                + tema::ESP_4 * 2.0
                + tema::ESP_8
        );
    }
}
