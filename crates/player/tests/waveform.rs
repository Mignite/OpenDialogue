use std::path::Path;

use rod_player::waveform::analizar;

#[test]
fn el_clip_de_prueba_da_picos_proporcionales_a_su_duracion() {
    let ruta = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/clip-15s-2audio.mp4");
    let picos = analizar(&ruta).expect("decodifica");
    assert_eq!(picos.por_segundo, 100);
    assert!(picos.max.len() > 10 * picos.por_segundo as usize);
    assert!(picos.max.iter().any(|v| *v > 0.05), "todo silencio");
}

#[test]
fn un_archivo_que_no_es_audio_da_error_sin_panico() {
    let dir = std::env::temp_dir().join("rod-waveform");
    std::fs::create_dir_all(&dir).unwrap();
    let ruta = dir.join("no-es-audio.mp4");
    std::fs::write(&ruta, b"basura").unwrap();
    assert!(analizar(&ruta).is_err());
}
