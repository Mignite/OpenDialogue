//! Genera la matriz de capturas de la auditoría visual.
//!
//! Uso:
//!   cargo run -p rod-app --features test-video --example ui_audit -- before
//!   cargo run -p rod-app --features test-video --example ui_audit -- after
//!   cargo run -p rod-app --features test-video --example ui_audit -- hoja before
//!
//! Escribe en `ui-audit/<fase>/` y compone `index.png`.

#[path = "../tests/ui_audit/mod.rs"]
mod arnes;

use arnes::*;
use rod_app::editor_app::Panel;
use rod_app::inicio;
use rod_app::textos::Idioma;

const DURACION_GRANDE: f64 = 1440.0;
const DURACION_MEDIANA: f64 = 92.0;
const DURACION_PEQUENA: f64 = 12.0;

fn proyecto_grande() -> rod_core::Proyecto {
    make_proyecto(543, 7, 3, DURACION_GRANDE)
}

fn proyecto_mediano() -> rod_core::Proyecto {
    make_proyecto(19, 3, 1, DURACION_MEDIANA)
}

fn proyecto_pequeno() -> rod_core::Proyecto {
    make_proyecto(1, 1, 0, DURACION_PEQUENA)
}

fn proyecto_vacio() -> rod_core::Proyecto {
    make_proyecto(0, 0, 0, 0.0)
}

fn main() {
    let fase = std::env::args().nth(1).unwrap_or_else(|| "before".into());
    if fase == "hoja" {
        let destino = std::env::args().nth(2).unwrap_or_else(|| "before".into());
        hoja_contacto(&destino);
        return;
    }
    let en = Idioma::En;

    for (ancho, alto) in TAMANOS {
        let t = etiqueta_tamano(ancho, alto);

        // ── Inicio ────────────────────────────────────────────────────────
        capturar_inicio(
            &fase,
            &nombre_captura("inicio", "vacio", ancho, alto),
            ancho,
            alto,
            inicio::nuevo(Vec::new(), None),
            en,
            |_| {},
        );
        capturar_inicio(
            &fase,
            &nombre_captura("inicio", "recientes", ancho, alto),
            ancho,
            alto,
            inicio::nuevo(recientes_fixture(), None),
            en,
            |_| {},
        );
        capturar_inicio(
            &fase,
            &nombre_captura("inicio", "recuperacion", ancho, alto),
            ancho,
            alto,
            inicio::nuevo(recientes_fixture(), Some(aviso_recovery())),
            en,
            |_| {},
        );
        capturar_inicio(
            &fase,
            &nombre_captura("inicio", "arrastrando", ancho, alto),
            ancho,
            alto,
            inicio::nuevo(recientes_fixture(), None),
            en,
            |h| arrastrando_archivo(h),
        );
        capturar_inicio(
            &fase,
            &nombre_captura("inicio", "foco", ancho, alto),
            ancho,
            alto,
            {
                let mut estado = inicio::nuevo(recientes_fixture(), None);
                estado.destacado = Some(0);
                estado
            },
            en,
            |_| {},
        );

        // ── Editor: pestañas y proyectos ─────────────────────────────────
        capturar_editor(
            &fase,
            &nombre_captura("editor", "subtitulos-grande", ancho, alto),
            ancho,
            alto,
            proyecto_grande(),
            Panel::Subtitulos,
            DURACION_GRANDE,
            en,
            |_| {},
        );
        capturar_editor(
            &fase,
            &nombre_captura("editor", "subtitulos-mediano", ancho, alto),
            ancho,
            alto,
            proyecto_mediano(),
            Panel::Subtitulos,
            DURACION_MEDIANA,
            en,
            |_| {},
        );
        capturar_editor(
            &fase,
            &nombre_captura("editor", "subtitulos-pequeno", ancho, alto),
            ancho,
            alto,
            proyecto_pequeno(),
            Panel::Subtitulos,
            DURACION_PEQUENA,
            en,
            |_| {},
        );
        capturar_editor(
            &fase,
            &nombre_captura("editor", "hablantes", ancho, alto),
            ancho,
            alto,
            proyecto_grande(),
            Panel::Hablantes,
            DURACION_GRANDE,
            en,
            |_| {},
        );
        capturar_editor(
            &fase,
            &nombre_captura("editor", "estilos", ancho, alto),
            ancho,
            alto,
            proyecto_grande(),
            Panel::Estilos,
            DURACION_GRANDE,
            en,
            |_| {},
        );
        capturar_editor(
            &fase,
            &nombre_captura("editor", "vacio", ancho, alto),
            ancho,
            alto,
            proyecto_vacio(),
            Panel::Subtitulos,
            0.0,
            en,
            |_| {},
        );

        // ── Menús de la cabecera ─────────────────────────────────────────
        for (clave, etiqueta) in [
            ("archivo", "File"),
            ("edicion", "Edit"),
            ("exportar", "Export"),
            ("ayuda", "Help"),
        ] {
            capturar_editor(
                &fase,
                &nombre_captura("menu", clave, ancho, alto),
                ancho,
                alto,
                proyecto_grande(),
                Panel::Subtitulos,
                DURACION_GRANDE,
                en,
                |h| abrir_menu_editor(h, etiqueta),
            );
        }

        // ── Capas y estados ──────────────────────────────────────────────
        capturar_ajustes(
            &fase,
            &nombre_captura("ajustes", "idioma", ancho, alto),
            ancho,
            alto,
            inicio::nuevo(recientes_fixture(), None),
            en,
            rod_app::ui::ajustes::Pestana::Idioma,
        );
        capturar_ajustes(
            &fase,
            &nombre_captura("ajustes", "atajos", ancho, alto),
            ancho,
            alto,
            inicio::nuevo(recientes_fixture(), None),
            en,
            rod_app::ui::ajustes::Pestana::Atajos,
        );
        capturar_atajos(
            &fase,
            &nombre_captura("ayuda", "atajos", ancho, alto),
            ancho,
            alto,
            inicio::nuevo(recientes_fixture(), None),
            en,
        );
        capturar_toast(
            &fase,
            &nombre_captura("editor", "toast", ancho, alto),
            ancho,
            alto,
            proyecto_grande(),
            en,
            "Line deleted",
        );

        // Punto de estado guardado: el contraste con el ámbar de «Unsaved».
        capturar_editor(
            &fase,
            &nombre_captura("editor", "guardado", ancho, alto),
            ancho,
            alto,
            proyecto_grande(),
            Panel::Subtitulos,
            DURACION_GRANDE,
            en,
            |h| {
                h.state_mut().sucio = false;
                h.run();
            },
        );

        // Hover y foco dentro del editor: el puntero sobre el timeline y el
        // campo de texto enfocado.
        capturar_editor(
            &fase,
            &nombre_captura("editor", "foco-texto", ancho, alto),
            ancho,
            alto,
            proyecto_grande(),
            Panel::Subtitulos,
            DURACION_GRANDE,
            en,
            |h| {
                h.state_mut().editor.texto.foco = true;
                h.run();
            },
        );
        capturar_editor(
            &fase,
            &nombre_captura("editor", "hover-timeline", ancho, alto),
            ancho,
            alto,
            proyecto_grande(),
            Panel::Subtitulos,
            DURACION_GRANDE,
            en,
            |h| apuntar_y_esperar(h, ancho * 0.45, alto - 90.0),
        );

        println!("capturas {t} listas");
    }

    // Español: una captura para detectar mezclas de idioma.
    capturar_inicio(
        &fase,
        &nombre_simple("inicio", "recientes-es"),
        1600.0,
        900.0,
        inicio::nuevo(recientes_fixture(), None),
        Idioma::Es,
        |_| {},
    );
    capturar_editor(
        &fase,
        &nombre_simple("editor", "subtitulos-es"),
        1600.0,
        900.0,
        proyecto_mediano(),
        Panel::Subtitulos,
        DURACION_MEDIANA,
        Idioma::Es,
        |_| {},
    );

    hoja_contacto(&fase);
}
