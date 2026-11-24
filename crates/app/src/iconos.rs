//! Iconos del chrome: SVG en memoria, sin archivos ni assets externos.
//!
//! Todos comparten la misma rejilla de 24 px y trazo de 1,7 px. El color se
//! aplica con el filtro del widget, así que el mismo icono sirve para texto,
//! acento y estados deshabilitados.

use iced::widget::svg;
use iced::{Color, Element};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icono {
    Nuevo,
    Abrir,
    Inicio,
    Ayuda,
    Buscar,
    Reproducir,
    Pausa,
    Atrasar,
    Adelantar,
    Deshacer,
    Rehacer,
    Borrar,
    Seleccion,
    Mano,
    Zoom,
    Cortar,
    Texto,
    Imagen,
    Audio,
    Subtitulo,
    Hablante,
    Estilos,
    Ojo,
    Candado,
    Ajustar,
    Mas,
    Menos,
    Cerrar,
    Marcador,
}

impl Icono {
    /// SVG con `currentColor`: el widget lo tiñe del color que pidamos.
    pub fn datos(self) -> &'static str {
        match self {
            Self::Nuevo => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M6 3h8l4 4v14H6z'/><path d='M14 3v5h4M12 11v6M9 14h6'/></svg>"
            }
            Self::Abrir => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M3 7h6l2 2h10v10H3z'/><path d='M3 7V5h6l2 2'/></svg>"
            }
            Self::Inicio => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='m3 11 9-7 9 7'/><path d='M5 10v10h14V10'/><path d='M10 20v-6h4v6'/></svg>"
            }
            Self::Ayuda => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><circle cx='12' cy='12' r='9'/><path d='M9.5 9.5a2.5 2.5 0 1 1 3.5 2.3c-.8.4-1 1-1 1.9'/><path d='M12 17h.01'/></svg>"
            }
            Self::Buscar => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><circle cx='11' cy='11' r='6'/><path d='m15.5 15.5 5 5'/></svg>"
            }
            Self::Reproducir => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='m8 5 11 7-11 7z'/></svg>"
            }
            Self::Pausa => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M9 5v14M15 5v14'/></svg>"
            }
            Self::Atrasar => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M18 6v12L9 12zM6 5v14'/></svg>"
            }
            Self::Adelantar => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M6 6v12l9-6zM18 5v14'/></svg>"
            }
            Self::Deshacer => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M9 8H5V4'/><path d='M5 8a8 8 0 1 1 0 8'/></svg>"
            }
            Self::Rehacer => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M15 8h4V4'/><path d='M19 8a8 8 0 1 0 0 8'/></svg>"
            }
            Self::Borrar => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M5 7h14M9 7V4h6v3M7 7l1 13h8l1-13'/><path d='M10 11v5M14 11v5'/></svg>"
            }
            Self::Seleccion => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='m5 3 13 8-6 1-2 6z'/></svg>"
            }
            Self::Mano => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M7 11V6a1.5 1.5 0 0 1 3 0v4-5a1.5 1.5 0 0 1 3 0v5-3a1.5 1.5 0 0 1 3 0v5-2a1.5 1.5 0 0 1 3 0v7a5 5 0 0 1-5 5h-2a5 5 0 0 1-4-2l-3-4a1.6 1.6 0 0 1 2.4-2z'/></svg>"
            }
            Self::Zoom => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><circle cx='10.5' cy='10.5' r='6'/><path d='m15 15 5 5M10.5 8v5M8 10.5h5'/></svg>"
            }
            Self::Cortar => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><circle cx='6' cy='18' r='2.5'/><circle cx='18' cy='18' r='2.5'/><path d='M8 16 18 4M16 16 6 4'/></svg>"
            }
            Self::Texto => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M4 5h16M12 5v14M9 19h6'/></svg>"
            }
            Self::Imagen => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><rect x='3' y='4' width='18' height='16'/><circle cx='8.5' cy='9' r='1.5'/><path d='m4 19 6-6 4 4 3-3 4 4'/></svg>"
            }
            Self::Audio => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M4 12h2M8 8v8M12 5v14M16 9v6M20 11v2'/></svg>"
            }
            Self::Subtitulo => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><rect x='3' y='5' width='18' height='14'/><path d='M7 13h4M14 13h3M7 16h10'/></svg>"
            }
            Self::Hablante => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><circle cx='12' cy='8' r='3.5'/><path d='M5 20a7 7 0 0 1 14 0'/></svg>"
            }
            Self::Estilos => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M4 7h16M4 12h16M4 17h16'/><path d='M9 5v4M15 10v4M7 15v4'/></svg>"
            }
            Self::Ojo => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M2 12s3.5-6 10-6 10 6 10 6-3.5 6-10 6-10-6-10-6z'/><circle cx='12' cy='12' r='2.5'/></svg>"
            }
            Self::Candado => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><rect x='5' y='10' width='14' height='10'/><path d='M8 10V7a4 4 0 0 1 8 0v3'/></svg>"
            }
            Self::Ajustar => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M4 9V4h5M20 15v5h-5M20 9V4h-5M4 15v5h5'/></svg>"
            }
            Self::Mas => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M12 5v14M5 12h14'/></svg>"
            }
            Self::Menos => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M5 12h14'/></svg>"
            }
            Self::Cerrar => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M6 6l12 12M18 6 6 18'/></svg>"
            }
            Self::Marcador => {
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'><path d='M7 4h10v16l-5-4-5 4z'/></svg>"
            }
        }
    }
}

/// Icono listo para insertar en cualquier vista. El SVG lleva trazo en
/// `currentColor`; el filtro del widget aplica el color pedido.
pub fn vista<'a, Mensaje: Clone + 'a>(
    icono: Icono,
    tamano: u32,
    color: Color,
) -> Element<'a, Mensaje> {
    let svg = svg(svg::Handle::from_memory(icono.datos().as_bytes()))
        .width(tamano)
        .height(tamano)
        .style(move |_: &iced::Theme, _: svg::Status| svg::Style { color: Some(color) });
    svg.into()
}

#[cfg(test)]
mod tests {
    use super::Icono;

    #[test]
    fn todos_los_iconos_son_svg_de_la_misma_rejilla() {
        for icono in [
            Icono::Nuevo,
            Icono::Abrir,
            Icono::Inicio,
            Icono::Ayuda,
            Icono::Buscar,
            Icono::Reproducir,
            Icono::Pausa,
            Icono::Atrasar,
            Icono::Adelantar,
            Icono::Deshacer,
            Icono::Rehacer,
            Icono::Borrar,
            Icono::Seleccion,
            Icono::Mano,
            Icono::Zoom,
            Icono::Cortar,
            Icono::Texto,
            Icono::Imagen,
            Icono::Audio,
            Icono::Subtitulo,
            Icono::Hablante,
            Icono::Estilos,
            Icono::Ojo,
            Icono::Candado,
            Icono::Ajustar,
            Icono::Mas,
            Icono::Menos,
            Icono::Cerrar,
            Icono::Marcador,
        ] {
            let datos = icono.datos();
            assert!(datos.starts_with("<svg"), "{icono:?}");
            assert!(datos.contains("viewBox='0 0 24 24'"), "{icono:?}");
            assert!(datos.contains("currentColor"), "{icono:?}");
        }
    }
}
