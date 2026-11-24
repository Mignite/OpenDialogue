//! Fuentes embebidas (Inter y JetBrains Mono) y familias nombradas.
//!
//! Se instalan antes de cualquier pintado: pedir `FontFamily::Name` sin
//! registrar la familia hace que epaint entre en pánito al dibujar el primer
//! texto. El mono queda reservado a cifras, atajos y códigos de tecla.

const INTER_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Inter-Regular.ttf");
const INTER_MEDIUM: &[u8] = include_bytes!("../../assets/fonts/Inter-Medium.ttf");
const INTER_SEMIBOLD: &[u8] = include_bytes!("../../assets/fonts/Inter-SemiBold.ttf");
const MONO_REGULAR: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf");
const MONO_MEDIUM: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Medium.ttf");

/// Pesos disponibles en la interfaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Peso {
    Regular,
    Medium,
    SemiBold,
    Mono,
}

/// Familia de egui correspondiente a cada peso.
pub fn familia(peso: Peso) -> egui::FontFamily {
    match peso {
        Peso::Regular => egui::FontFamily::Proportional,
        Peso::Medium => egui::FontFamily::Name("inter-medium".into()),
        Peso::SemiBold => egui::FontFamily::Name("inter-semibold".into()),
        Peso::Mono => egui::FontFamily::Monospace,
    }
}

/// Registra las cinco fuentes en el contexto. Llamar una sola vez al arrancar.
pub fn instalar(ctx: &egui::Context) {
    let mut defs = egui::FontDefinitions::default();

    defs.font_data.insert(
        "inter-regular".into(),
        egui::FontData::from_static(INTER_REGULAR).into(),
    );
    defs.font_data.insert(
        "inter-medium".into(),
        egui::FontData::from_static(INTER_MEDIUM).into(),
    );
    defs.font_data.insert(
        "inter-semibold".into(),
        egui::FontData::from_static(INTER_SEMIBOLD).into(),
    );
    defs.font_data.insert(
        "jetbrains-mono-regular".into(),
        egui::FontData::from_static(MONO_REGULAR).into(),
    );
    defs.font_data.insert(
        "jetbrains-mono-medium".into(),
        egui::FontData::from_static(MONO_MEDIUM).into(),
    );

    // Inter manda en Proportional; NotoEmoji se conserva como reserva para no
    // dejar tofu en nombres de hablante que incluyan emoji.
    if let Some(fuentes) = defs.families.get_mut(&egui::FontFamily::Proportional) {
        fuentes.insert(0, "inter-regular".into());
    }
    defs.families.insert(
        egui::FontFamily::Name("inter-medium".into()),
        vec!["inter-medium".into(), "inter-regular".into()],
    );
    defs.families.insert(
        egui::FontFamily::Name("inter-semibold".into()),
        vec!["inter-semibold".into(), "inter-regular".into()],
    );
    defs.families.insert(
        egui::FontFamily::Monospace,
        vec![
            "jetbrains-mono-regular".into(),
            "jetbrains-mono-medium".into(),
        ],
    );

    // Los glifos de Phosphor se añaden en la tarea del kit, después de este
    // registro, con la API pública de egui-phosphor.
    ctx.set_fonts(defs);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn las_familias_nombradas_se_registran_y_miden_distinto() {
        let ctx = egui::Context::default();
        instalar(&ctx);
        // epaint solo crea el Fonts view después del primer `run`.
        ctx.run(Default::default(), |_| {});
        let medir = |peso| {
            ctx.fonts_mut(|f| {
                f.layout_no_wrap(
                    "Hablante".into(),
                    egui::FontId::new(13.0, familia(peso)),
                    egui::Color32::WHITE,
                )
                .size()
                .x
            })
        };
        assert!(medir(Peso::Regular) > 0.0);
        assert_ne!(
            medir(Peso::Medium),
            medir(Peso::Regular),
            "Medium no se cargó"
        );
        assert_ne!(
            medir(Peso::Mono),
            medir(Peso::Regular),
            "la mono no se cargó"
        );
        let _ = medir(Peso::SemiBold);
    }
}
