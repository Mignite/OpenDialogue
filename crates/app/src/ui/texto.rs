//! Barra única de edición del texto de un subtítulo, bajo el video.

use egui::{Key, RichText, TextEdit, Vec2};

use crate::editor_app::{EditorApp, EstadoTexto, MensajeEditor};
use crate::textos::Textos;
use crate::ui::kit;
use crate::ui::tema::{self, Estilo};

/// Alto extra que la barra añade al transporte cuando crece a varias líneas.
pub fn alto_extra(state: &EditorApp, _ancho: f32) -> f32 {
    (state.texto.lineas.saturating_sub(1)) as f32 * tema::ALTO_BARRA_TEXTO
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
    let objetivo = EstadoTexto::objetivo(state);
    if objetivo.is_some() && state.texto.objetivo != objetivo {
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
        state.texto.objetivo = objetivo.clone();
        state.texto.borrador = texto.clone();
        state.texto.original = texto;
        state.texto.lineas = 1;
    }
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
        .unwrap_or_else(|| "Sin hablante".into());
    let color = hablante
        .as_ref()
        .map(|hablante| tema::color_de_hex(&hablante.color))
        .unwrap_or(tema::TEXTO_3);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = tema::ESP_8;
        kit::punto(ui, color, tema::ESP_8);
        let chip = egui::containers::menu::MenuButton::from_button(
            egui::Button::new(RichText::new(&nombre).font(tema::fuente(Estilo::Pequeno)))
                .frame(false)
                .min_size(egui::vec2(tema::ANCHO_BOTON_28, tema::ALTO_BOTON_CHICO)),
        )
        .ui(ui, |ui| {
            if ui.button("Sin hablante").clicked() {
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

        let mostrar_cps = ui.available_width() >= tema::ANCHO_MINIMO_TEXTO;
        let ancho_cps = if mostrar_cps { 64.0 } else { 0.0 };
        let ancho = (ui.available_width() - ancho_cps - tema::ESP_8).max(tema::ESP_72);
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
        let solicitar_foco = state.texto.foco && !respuesta.has_focus();
        if solicitar_foco {
            respuesta.request_focus();
        }
        if respuesta.has_focus() {
            if state.texto.original.is_empty() && state.texto.ultimo_tipeo.is_none() {
                state.texto.original = state.texto.borrador.clone();
            }
            if objetivo.is_some() && state.texto.borrador != state.texto.original {
                update_texto(state);
            }
            if !state.texto.borrador.is_empty() {
                state.texto.ultimo_tipeo = Some(std::time::Instant::now());
            }
            if ui.input(|entrada| entrada.key_pressed(Key::Enter) && !entrada.modifiers.shift) {
                if state.texto.borrador.ends_with('\n') {
                    state.texto.borrador.pop();
                }
                EstadoTexto::cerrar_sesion(state);
                respuesta.surrender_focus();
            } else if ui.input(|entrada| entrada.key_pressed(Key::Escape)) {
                EstadoTexto::revertir(state);
                respuesta.surrender_focus();
            } else {
                state.texto.lineas =
                    (state.texto.borrador.matches('\n').count().clamp(0, 2) + 1) as u8;
            }
        }
        if respuesta.lost_focus() && state.texto.foco {
            EstadoTexto::cerrar_sesion(state);
        }
        if !solicitar_foco && !state.texto.foco {
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
