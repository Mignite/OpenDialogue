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

    // Phosphor entra como reserva de Proportional en el índice 1: los glifos
    // se pintan con cualquier texto sin familia explícita.
    egui_phosphor::add_to_fonts(&mut defs, egui_phosphor::Variant::Regular);
    ctx.set_fonts(defs);
}

/// Base de fuentes del sistema, cargada una sola vez.
fn base_del_sistema() -> &'static fontdb::Database {
    static BASE: std::sync::OnceLock<fontdb::Database> = std::sync::OnceLock::new();
    BASE.get_or_init(|| {
        let mut base = fontdb::Database::new();
        base.load_system_fonts();
        base
    })
}

/// ¿La familia está instalada en el sistema?
pub fn fuente_disponible(nombre: &str) -> bool {
    if nombre.is_empty() || nombre.eq_ignore_ascii_case("arial") {
        return true; // libass la resuelve siempre como fuente por defecto.
    }
    base_del_sistema()
        .query(&fontdb::Query {
            families: &[fontdb::Family::Name(nombre)],
            ..Default::default()
        })
        .is_some()
}

/// ¿La familia instalada tiene cara bold/italic?
pub fn variante_disponible(nombre: &str, negrita: bool) -> bool {
    if !fuente_disponible(nombre) {
        return false;
    }
    let mut consulta = fontdb::Query {
        families: &[fontdb::Family::Name(nombre)],
        ..Default::default()
    };
    if negrita {
        consulta.weight = fontdb::Weight::BOLD;
    } else {
        consulta.style = fontdb::Style::Italic;
    }
    base_del_sistema().query(&consulta).is_some()
}

/// Registra en egui las familias visibles del panel, una sola vez por lote.
pub fn registrar_fuentes(ctx: &egui::Context, nombres: &[String]) {
    static REGISTRADAS: std::sync::OnceLock<std::sync::Mutex<std::collections::HashSet<String>>> =
        std::sync::OnceLock::new();
    let registradas =
        REGISTRADAS.get_or_init(|| std::sync::Mutex::new(std::collections::HashSet::new()));
    let nuevas: Vec<String> = {
        let mut vistas = registradas.lock().unwrap();
        nombres
            .iter()
            .filter(|nombre| !nombre.is_empty() && vistas.insert((*nombre).clone()))
            .cloned()
            .collect()
    };
    if nuevas.is_empty() {
        return;
    }
    let mut defs = ctx.fonts(|fuentes| fuentes.definitions().clone());
    for nombre in &nuevas {
        let Some(id) = base_del_sistema().query(&fontdb::Query {
            families: &[fontdb::Family::Name(nombre)],
            ..Default::default()
        }) else {
            continue;
        };
        let Some(fuente) = base_del_sistema().face(id) else {
            continue;
        };
        let fontdb::Source::File(ruta) = &fuente.source else {
            continue;
        };
        let Ok(bytes) = std::fs::read(ruta) else {
            continue;
        };
        let clave = format!("preset-{}", nombre);
        defs.font_data
            .insert(clave.clone(), egui::FontData::from_owned(bytes).into());
        defs.families.insert(
            egui::FontFamily::Name(nombre.clone().into()),
            vec![clave, "inter-regular".into(), "phosphor".into()],
        );
    }
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
