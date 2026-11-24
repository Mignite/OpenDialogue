use rod_app::comandos::Comando;
use rod_app::textos::{EN, ES, FechaRelativa, Idioma};

#[test]
fn todo_comando_tiene_etiqueta_en_los_dos_idiomas() {
    for comando in Comando::todas() {
        assert!(!ES.comando(*comando).is_empty(), "{comando:?} en ES");
        assert!(!EN.comando(*comando).is_empty(), "{comando:?} en EN");
    }
}

#[test]
fn el_idioma_por_defecto_es_ingles() {
    assert_eq!(Idioma::default(), Idioma::En);
    assert_eq!(EN.inicio_titulo, "Edit subtitles with precision");
    assert_eq!(ES.inicio_titulo, "Edita subtítulos con precisión");
}

#[test]
fn plurales_en_los_dos_idiomas() {
    assert_eq!(EN.lineas(1), "1 line");
    assert_eq!(EN.lineas(543), "543 lines");
    assert_eq!(ES.lineas(1), "1 línea");
    assert_eq!(ES.lineas(543), "543 líneas");
}

#[test]
fn las_fechas_son_relativas_y_se_tradicen() {
    assert_eq!(EN.fecha(&FechaRelativa { dias: 1 }), "yesterday");
    assert_eq!(ES.fecha(&FechaRelativa { dias: 3 }), "hace 3 días");
}
