//! Golden del slice 2: se renderiza un `.ass` de fixture con `rod-subs` y se
//! compara contra el PNG que produjo mpv para el mismo `.ass` y el mismo
//! instante, con tolerancia por porcentaje de píxeles distintos (spec §6).
//!
//! Los PNG se regeneran con `cargo run -p rod-subs --example golden -- <caso>
//! <ms>` y se versionan aquí. Son recortes alrededor del texto, no el frame
//! entero: si el render se desvía, el recorte ya está sobre la zona que importa.
//!
//! La comparación **no vale entre versiones de libass** (spec §6): el número que
//! se imprime en el mensaje de fallo es la evidencia que hay que anotar al
//! cambiar de versión. En Linux el golden no aplica: la fuente del sistema no es
//! la misma (Arial/DirectWrite en Windows, fontconfig en Linux), así que el test
//! de píxeles se salta y quedan las comprobaciones de estructura.

use rod_subs::Lienzo;

const W: u32 = 1920;
const H: u32 = 1080;

/// El mismo margen que usó `examples/golden.rs` para recortar.
const MARGEN: u32 = 12;

/// Un canal puede diferir por el antialiasing y por la mezcla en punto flotante
/// de mpv; por debajo de esto se considera igual.
const TOLERANCIA: u8 = 8;

/// Techo de la comparación. Con libass 0.17.5 y Arial la diferencia real es
/// 0.000 %, así que esto es margen para el redondeo, no para desviaciones de
/// posición o de color: un texto movido un píxel se pasa de largo.
const LIMITE_PORCENTAJE: f64 = 0.1;

struct Caso {
    nombre: &'static str,
    ass: &'static str,
    ms: i64,
    fondo_png: &'static [u8],
    oraculo_png: &'static [u8],
}

const CASOS: &[Caso] = &[
    Caso {
        // Dos estilos, contorno, sombra al 50 %, rojo con `\pos`, negrita.
        nombre: "basico",
        ass: include_str!("fixtures/basico.ass"),
        ms: 1080,
        fondo_png: include_bytes!("fixtures/golden/basico-fondo.png"),
        oraculo_png: include_bytes!("fixtures/golden/basico-oraculo.png"),
    },
    Caso {
        // Cursiva, negrita, ajuste de línea, sombra y un `\fad` a mitad.
        nombre: "variantes",
        ass: include_str!("fixtures/variantes.ass"),
        ms: 1080,
        fondo_png: include_bytes!("fixtures/golden/variantes-fondo.png"),
        oraculo_png: include_bytes!("fixtures/golden/variantes-oraculo.png"),
    },
];

/// Estadística de la comparación de dos imágenes del mismo tamaño.
struct Diferencias {
    /// Píxeles que difieren en más de `TOLERANCIA` en algún canal.
    distintos: u64,
    /// Píxeles que difieren en algo.
    con_diferencia: u64,
    /// Diferencia máxima en un canal.
    maxima: u8,
    total: u64,
}

impl Diferencias {
    fn porcentaje(&self) -> f64 {
        self.distintos as f64 * 100.0 / self.total as f64
    }
}

fn comparar(a: &Lienzo, b: &Lienzo) -> Diferencias {
    assert_eq!(
        (a.ancho(), a.alto()),
        (b.ancho(), b.alto()),
        "las dos imágenes no miden lo mismo"
    );

    let mut diferencias = Diferencias {
        distintos: 0,
        con_diferencia: 0,
        maxima: 0,
        total: u64::from(a.ancho()) * u64::from(a.alto()),
    };

    for (pa, pb) in a
        .pixeles()
        .as_chunks::<3>()
        .0
        .iter()
        .zip(b.pixeles().as_chunks::<3>().0)
    {
        let diferencia = (0..3).map(|c| pa[c].abs_diff(pb[c])).max().unwrap_or(0);
        diferencias.maxima = diferencias.maxima.max(diferencia);
        if diferencia > 0 {
            diferencias.con_diferencia += 1;
        }
        if diferencia > TOLERANCIA {
            diferencias.distintos += 1;
        }
    }

    diferencias
}

fn png(bytes: &[u8]) -> Lienzo {
    let imagen = image::load_from_memory(bytes)
        .expect("el golden no es un PNG válido")
        .to_rgb8();
    let (ancho, alto) = imagen.dimensions();
    Lienzo::desde_rgb(ancho, alto, imagen.into_raw()).expect("el golden no mide ancho*alto*3")
}

/// Renderiza el caso y compone sobre el frame limpio del golden.
fn render_propio(caso: &Caso) -> Lienzo {
    let libreria = rod_subs::Library::new();
    let mut renderer = rod_subs::Renderer::new(&libreria, W, H);
    renderer.usar_fuentes(rod_subs::PROVEEDOR_AUTODETECT);
    let track = rod_subs::Track::parse(&libreria, caso.ass);
    let bitmaps = renderer.render(&track, caso.ms);

    let caja = rod_subs::caja(bitmaps, MARGEN, W, H)
        .unwrap_or_else(|| panic!("{}: no hay subtítulos a {} ms", caso.nombre, caso.ms));

    let mut lienzo = png(caso.fondo_png);
    assert_eq!(
        (lienzo.ancho(), lienzo.alto()),
        (caja.ancho, caja.alto),
        "{}: el recorte del golden no coincide con el de los bitmaps",
        caso.nombre
    );

    lienzo.componer_desplazado(bitmaps, -caja.x, -caja.y);
    lienzo
}

#[test]
#[cfg_attr(
    not(windows),
    ignore = "golden generado con Arial por DirectWrite; en Linux la fuente del sistema es otra"
)]
fn el_render_coincide_con_el_oraculo() {
    for caso in CASOS {
        let propio = render_propio(caso);
        let oraculo = png(caso.oraculo_png);

        // Si el golden no tuviera subtítulos, la comparación pasaría por la
        // razón equivocada: se exige que el oráculo sí pinte algo.
        let limpio = png(caso.fondo_png);
        let tinta = comparar(&oraculo, &limpio);
        assert!(
            tinta.con_diferencia > 0,
            "{}: el golden es idéntico al fondo, no tiene subtítulos",
            caso.nombre
        );

        let diferencias = comparar(&propio, &oraculo);
        eprintln!(
            "{}: libass {} — {} px distintos (>{TOLERANCIA}) de {}, {:.3} %; \
             con cualquier diferencia {}, máximo {}/255",
            caso.nombre,
            rod_subs::version(),
            diferencias.distintos,
            diferencias.total,
            diferencias.porcentaje(),
            diferencias.con_diferencia,
            diferencias.maxima
        );

        if diferencias.porcentaje() > LIMITE_PORCENTAJE {
            guardar_diff(caso.nombre, &propio, &oraculo);
        }
        assert!(
            diferencias.porcentaje() <= LIMITE_PORCENTAJE,
            "{}: {:.3} % de píxeles distintos (>{TOLERANCIA}) supera el límite de \
             {LIMITE_PORCENTAJE} % con libass {}; diferencia máxima {}/255",
            caso.nombre,
            diferencias.porcentaje(),
            rod_subs::version(),
            diferencias.maxima
        );
    }
}

/// Deja el mapa de diferencias fuera del repo, para mirar *dónde* falla.
fn guardar_diff(nombre: &str, propio: &Lienzo, oraculo: &Lienzo) {
    let mut mascara = image::RgbImage::new(propio.ancho(), propio.alto());

    for y in 0..propio.alto() {
        for x in 0..propio.ancho() {
            let inicio = (y as usize * propio.ancho() as usize + x as usize) * 3;
            let diferencia = (0..3)
                .map(|c| propio.pixeles()[inicio + c].abs_diff(oraculo.pixeles()[inicio + c]))
                .max()
                .unwrap_or(0);
            if diferencia > TOLERANCIA {
                mascara.put_pixel(x, y, image::Rgb([255, 0, 0]));
            }
        }
    }

    let destino = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("oraculo-{nombre}-diff.png"));
    let _ = mascara.save(&destino);
    eprintln!("diferencias pintadas en {}", destino.display());
}

/// Comprobaciones que no dependen de la fuente ni de la versión de libass: son
/// las que siguen corriendo en Linux.
#[test]
fn los_bitmaps_caen_dentro_del_frame() {
    let libreria = rod_subs::Library::new();
    let mut renderer = rod_subs::Renderer::new(&libreria, W, H);
    renderer.usar_fuentes(rod_subs::PROVEEDOR_AUTODETECT);

    for caso in CASOS {
        let track = rod_subs::Track::parse(&libreria, caso.ass);
        let bitmaps = renderer.render(&track, caso.ms);
        assert!(!bitmaps.is_empty(), "{}: sin bitmaps", caso.nombre);

        for bitmap in bitmaps {
            assert!(
                bitmap.ancho > 0 && bitmap.alto > 0,
                "{}: bitmap vacío",
                caso.nombre
            );
            assert!(
                bitmap.alpha().len()
                    >= bitmap.stride * (bitmap.alto as usize - 1) + bitmap.ancho as usize,
                "{}: la máscara no cubre el bitmap",
                caso.nombre
            );
            assert!(
                bitmap.x >= 0
                    && bitmap.y >= 0
                    && bitmap.x as u32 + bitmap.ancho <= W
                    && bitmap.y as u32 + bitmap.alto <= H,
                "{}: bitmap fuera del frame en ({}, {}) de {}x{}",
                caso.nombre,
                bitmap.x,
                bitmap.y,
                bitmap.ancho,
                bitmap.alto
            );
        }
    }
}

/// El instante sin eventos no pinta nada: separa "hay subtítulos" de "hay algo".
#[test]
fn antes_del_primer_evento_no_hay_bitmaps() {
    let libreria = rod_subs::Library::new();
    let mut renderer = rod_subs::Renderer::new(&libreria, W, H);
    renderer.usar_fuentes(rod_subs::PROVEEDOR_AUTODETECT);

    for caso in CASOS {
        let track = rod_subs::Track::parse(&libreria, caso.ass);
        assert!(
            renderer.render(&track, 0).is_empty(),
            "{}: hay bitmaps antes del primer evento",
            caso.nombre
        );
        assert_eq!(
            rod_subs::caja(renderer.render(&track, 0), MARGEN, W, H),
            None
        );
    }
}
