use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::Emitter;
use tauri::Manager;

// Guard de concurrencia: evita dos análisis de volumen del mismo track
// emitiendo chunks duplicados al frontend.
static ANALIZANDO: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashSet<(String, usize)>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashSet::new()));

// Limpia el registro de análisis en vuelo al dropearse (pase lo que pase en el
// cuerpo del análisis), para que un error no bloquee análisis futuros.
struct AnalisisGuard {
    ruta: String,
    idx: usize,
}
impl Drop for AnalisisGuard {
    fn drop(&mut self) {
        if let Ok(mut set) = ANALIZANDO.lock() {
            set.remove(&(self.ruta.clone(), self.idx));
        }
    }
}

// Escritura atómica: escribe a un .tmp y renombra. Un crash/fallo a mitad
// nunca deja el destino final corrupto (archivos de usuario, caches, SRT).
fn escribir_atomico(ruta: &std::path::Path, contenido: &[u8]) -> Result<(), String> {
    let tmp = ruta.with_extension("tmp");
    std::fs::write(&tmp, contenido).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, ruta).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("Error finalizando escritura: {}", e)
    })
}

// El helper `comando_oculto` (CREATE_NO_WINDOW en Windows) se usaba para
// spawnar powershell/reg/fc-list al enumerar fuentes. Con `fontdb` la app ya
// no shellea ningún proceso hijo, así que el helper se eliminó por completo.

fn calcular_cache_key(ruta_video: &str) -> Result<String, String> {
    let metadata = std::fs::metadata(ruta_video).map_err(|e| e.to_string())?;
    let size = metadata.len();
    let modified = metadata
        .modified()
        .map_err(|e| e.to_string())?
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let mut hasher = DefaultHasher::new();
    ruta_video.hash(&mut hasher);
    size.hash(&mut hasher);
    modified.hash(&mut hasher);
    // Dos videos distintos con mismo tamaño y mtime compartirían cache; mezclar
    // los primeros 4KB del contenido elimina la colisión práctica.
    if let Ok(mut f) = std::fs::File::open(ruta_video) {
        use std::io::Read;
        let mut buf = [0u8; 4096];
        let mut total = 0usize;
        while total < buf.len() {
            match f.read(&mut buf[total..]) {
                Ok(0) => break,
                Ok(n) => total += n,
                Err(_) => break,
            }
        }
        buf[..total].hash(&mut hasher);
    }
    let hash = hasher.finish();

    Ok(format!("{:x}", hash))
}

fn ruta_cache_para(
    app: &tauri::AppHandle,
    ruta_video: &str,
    track_index: Option<usize>,
) -> Result<std::path::PathBuf, String> {
    let key = calcular_cache_key(ruta_video)?;
    let track_suffix = track_index.map(|i| format!("_{}", i)).unwrap_or_default();
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?
        .join("audio_cache");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(format!("{}{}.cache", key, track_suffix)))
}

fn guardar_cache_volumen(
    app: &tauri::AppHandle,
    ruta_video: &str,
    track_index: Option<usize>,
    datos: &[f32],
) -> Result<(), String> {
    let path = ruta_cache_para(app, ruta_video, track_index)?;
    let mut bytes = Vec::with_capacity(datos.len() * 4);
    for &v in datos {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    escribir_atomico(&path, &bytes)
}

#[tauri::command]
fn existe_cache_volumen(
    app: tauri::AppHandle,
    ruta_video: String,
    track_index: Option<usize>,
) -> bool {
    match ruta_cache_para(&app, &ruta_video, track_index) {
        Ok(path) => path.exists(),
        Err(_) => false,
    }
}

#[tauri::command]
async fn cargar_cache_volumen(
    app: tauri::AppHandle,
    ruta_video: String,
    track_index: Option<usize>,
) -> Result<Vec<f32>, String> {
    let path = ruta_cache_para(&app, &ruta_video, track_index)?;
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        if bytes.is_empty() || bytes.len() % 4 != 0 {
            // Cache corrupto (crash a mitad de escritura): descartar y re-analizar
            let _ = std::fs::remove_file(&path);
            return Err("Cache de volumen corrupto, se re-analizará".to_string());
        }
        let mut resultado = Vec::with_capacity(bytes.len() / 4);
        for arr in bytes.as_chunks::<4>().0 {
            resultado.push(f32::from_le_bytes(*arr));
        }
        Ok(resultado)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Serialize, Deserialize)]
struct Hablante {
    id: String,
    nombre: String,
    tecla: String,
    color: String,
    // Estilo .ass elegido para el hablante (ver SpeakersPanel). Proyectos
    // viejos no traen la clave → None (mismo patrón que playhead).
    #[serde(default, rename = "presetId")]
    preset_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Caption {
    id: String,
    inicio: f64,
    fin: f64,
    texto: String,
    hablante_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Proyecto {
    ruta_video: String,
    hablantes: Vec<Hablante>,
    captions: Vec<Caption>,
    #[serde(default)]
    playhead: f64,
}

#[tauri::command]
fn guardar_proyecto(ruta: String, proyecto: Proyecto) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&proyecto).map_err(|e| e.to_string())?;
    escribir_atomico(std::path::Path::new(&ruta), json.as_bytes())
}

#[tauri::command]
fn cargar_proyecto(ruta: String) -> Result<Proyecto, String> {
    let contenido = std::fs::read_to_string(&ruta).map_err(|e| e.to_string())?;
    let proyecto: Proyecto = serde_json::from_str(&contenido).map_err(|e| e.to_string())?;
    Ok(proyecto)
}

#[tauri::command]
fn existe_archivo(ruta: String) -> bool {
    std::path::Path::new(&ruta).exists()
}

// SRT en Latin-1 o UTF-16 hacían fallar `read_to_string` (solo UTF-8).
// Orden: BOM (UTF-16LE/BE, UTF-8) → UTF-8 estricto → fallback Windows-1252
// (superconjunto de Latin-1: 0x80-0x9F mapean a caracteres útiles en vez de
// controles). Sin BOM no se intenta UTF-16: sus bytes NUL pasarían como UTF-8.
fn decodificar_texto(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let (texto, _, _) = encoding_rs::UTF_16LE.decode(&bytes[2..]);
        return texto.into_owned();
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let (texto, _, _) = encoding_rs::UTF_16BE.decode(&bytes[2..]);
        return texto.into_owned();
    }
    let sin_bom = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    if let Ok(texto) = std::str::from_utf8(sin_bom) {
        return texto.to_owned();
    }
    let (texto, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
    texto.into_owned()
}

#[tauri::command]
fn leer_archivo_texto(ruta: String) -> Result<String, String> {
    let bytes = std::fs::read(&ruta).map_err(|e| e.to_string())?;
    Ok(decodificar_texto(&bytes))
}

#[tauri::command]
async fn analizar_volumen(
    app: tauri::AppHandle,
    ruta: String,
    track_index: Option<usize>,
) -> Result<Vec<f32>, String> {
    let idx = track_index.unwrap_or(0);
    {
        let mut set = ANALIZANDO
            .lock()
            .map_err(|_| "Error interno de análisis".to_string())?;
        if !set.insert((ruta.clone(), idx)) {
            return Err("Ya se está analizando este video/track".to_string());
        }
    }
    let _guard = AnalisisGuard {
        ruta: ruta.clone(),
        idx,
    };
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<f32>, String> {
        use std::time::Instant;
        use symphonia::core::audio::SampleBuffer;
        use symphonia::core::codecs::DecoderOptions;
        use symphonia::core::errors::Error as SymphoniaError;
        use symphonia::core::formats::FormatOptions;
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let file = std::fs::File::open(&ruta).map_err(|e| e.to_string())?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = std::path::Path::new(&ruta)
            .extension()
            .and_then(|e| e.to_str())
        {
            hint.with_extension(ext);
        }

        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| e.to_string())?;

        let mut format = probed.format;

        let idx = track_index.unwrap_or(0);
        let track = format
            .tracks()
            .iter()
            .filter(|t| t.codec_params.sample_rate.is_some())
            .nth(idx)
            .ok_or("No se encontró el track de audio solicitado")?
            .clone();

        let track_id = track.id;
        let sample_rate = track.codec_params.sample_rate.ok_or("Sin sample rate")? as f64;

        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| e.to_string())?;

        let mut resultados: Vec<f32> = Vec::new();
        let mut acumulador_cuadrados: f64 = 0.0;
        let mut acumulador_cuenta: usize = 0;
        let mut sample_buf: Option<SampleBuffer<f32>> = None;
        let mut ventana_samples: usize = 0;
        let mut ultimo_emit = Instant::now();
        let mut ultimo_flush_idx: usize = 0;

        loop {
            let packet = match format.next_packet() {
                Ok(p) => p,
                Err(SymphoniaError::IoError(e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    break
                }
                Err(e) => return Err(e.to_string()),
            };

            if packet.track_id() != track_id {
                continue;
            }

            let decoded = match decoder.decode(&packet) {
                Ok(d) => d,
                Err(SymphoniaError::DecodeError(_)) => continue,
                Err(e) => return Err(e.to_string()),
            };

            if sample_buf.is_none() {
                let spec = *decoded.spec();
                let duration = decoded.capacity() as u64;
                let channels = spec.channels.count();
                ventana_samples = ((sample_rate / 15.0).round() as usize) * channels;
                sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
            }

            let buf = sample_buf.as_mut().unwrap();
            buf.copy_interleaved_ref(decoded);
            let samples = buf.samples();

            for &s in samples {
                acumulador_cuadrados += (s as f64) * (s as f64);
                acumulador_cuenta += 1;

                if acumulador_cuenta >= ventana_samples {
                    let rms = (acumulador_cuadrados / acumulador_cuenta as f64).sqrt();
                    resultados.push(rms as f32);
                    acumulador_cuadrados = 0.0;
                    acumulador_cuenta = 0;
                }
            }

            if ultimo_emit.elapsed().as_millis() > 250 {
                if ultimo_flush_idx < resultados.len() {
                    let nuevo_chunk = resultados[ultimo_flush_idx..].to_vec();
                    // La ruta viaja con el chunk: si el usuario abre otro video
                    // mientras este analiza, el frontend descarta los chunks
                    // que no son de la ruta que espera (antes, al ser single-track
                    // el track_index siempre era 0 y no filtraba nada).
                    let _ = app.emit("volumen_chunk", (&ruta, track_index, nuevo_chunk));
                    ultimo_flush_idx = resultados.len();
                }
                ultimo_emit = Instant::now();
            }
        }

        if ultimo_flush_idx < resultados.len() {
            let nuevo_chunk = resultados[ultimo_flush_idx..].to_vec();
            let _ = app.emit("volumen_chunk", (&ruta, track_index, nuevo_chunk));
        }
        // Blindaje: un container válido sin audio decodificable daría un
        // .cache de 0 bytes que el loader rechazaría en cada apertura
        // (re-análisis eterno). No se guarda; se informa.
        if resultados.is_empty() {
            return Err("El video no tiene audio decodificable".to_string());
        }
        let _ = guardar_cache_volumen(&app, &ruta, track_index, &resultados);

        Ok(resultados)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn escribir_archivo_texto(ruta: String, contenido: String) -> Result<(), String> {
    escribir_atomico(std::path::Path::new(&ruta), contenido.as_bytes())
}

/// Nombres de las familias instaladas, para el selector de fuente del preset
/// .ass.
///
/// Fuente única de verdad: `fontdb` parsea los archivos de fuente (tabla
/// `name` de cada `.ttf`/`.otf`/`.ttc`) en Rust puro. Ventajas frente a las dos
/// rutas por plataforma anteriores:
///   - Una sola ruta de código, idéntica en Windows/Linux/macOS: sin PowerShell,
///     sin `fc-list`, sin `#[cfg]`, sin dependencia de binarios externos.
///   - Devuelve los nombres de FAMILIA reales ("Bebas Neue"), que es lo que
///     resuelven CSS y libass. El registro de Windows devolvía el nombre con el
///     estilo pegado ("Bebas Neue Regular") y 110 de sus 201 entradas eran
///     estilos, no familias.
///   - MacOS entra gratis (antes devolvía `vec![]`).
///   - ~20 ms con caché caliente (medido por fontdb) frente a los ~400 ms del
///     spawn de powershell.exe.
#[tauri::command]
async fn listar_fuentes_sistema() -> Result<Vec<String>, String> {
    // spawn_blocking: parsear los archivos de fuente toma decenas/cientos de
    // ms y como comando sync congelaba el hilo principal al abrir la app (el
    // panel de Estilos pide las fuentes al montar).
    tauri::async_runtime::spawn_blocking(|| {
        let mut db = fontdb::Database::new();
        // Escanea los directorios estándar de fuentes del SO (Windows:
        // %WINDIR%\Fonts; Linux/macOS: los <dir> de fontconfig). No consulta la
        // API del sistema, así que una fuente instalada fuera de esos
        // directorios (p.ej. una carpeta <dir> propia en fonts.conf) no
        // aparecería; el frontend filtra igual por lo que el webview pinta.
        db.load_system_fonts();
        // Todas las familias declaradas por cada cara (un mismo archivo puede
        // declarar varias). Lo que NO se agrega son las variantes de peso
        // ("Cantarell Light"): fontdb agrupa los pesos bajo la familia base, y
        // en .ass el peso va en el campo Bold del Style:, no pegado al
        // Fontname (ver `lineaStyle` y los campos bold/italic del preset).
        let nombres: Vec<String> = db
            .faces()
            // `families` es Vec<(String, Language)>; el nombre es el primer
            // elemento de cada tupla.
            .flat_map(|cara| cara.families.iter().map(|(n, _)| n.clone()))
            .collect();
        Ok(normalizar_familias(nombres))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Parte pura del listado de fuentes: dedup + orden + descarte de lo que no
/// puede ser una familia CSS/.ass. Una familia no lleva separadores de ruta,
/// ni dos puntos, ni punto y coma (delimitador de los campos `Style:` del
/// .ass). Centralizado aquí y en tests para que el frontend reciba siempre la
/// misma forma de lista venga del SO que venga.
fn normalizar_familias(nombres: Vec<String>) -> Vec<String> {
    let mut vistas: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut familias: Vec<String> = Vec::new();
    for nombre in nombres {
        let limpio = nombre.trim();
        if limpio.is_empty()
            || limpio.contains('/')
            || limpio.contains('\\')
            || limpio.contains(':')
            || limpio.contains(';')
        {
            continue;
        }
        if vistas.insert(limpio.to_string()) {
            familias.push(limpio.to_string());
        }
    }
    familias.sort();
    familias
}

// Un nombre de archivo nunca debe escapar de su carpeta: el frontend
// sanitiza, pero este comando es el choke point y no puede confiar en eso.
// Rechaza `..` como nombre COMPLETO, separadores y rutas absolutas (ambos
// sabores, para que un `C:\x` no cuele en Linux y viceversa). Los puntos
// sueltos dentro del nombre son legales (`Narrador....srt`): la travesía solo
// es posible con un separador, y esos ya están rechazados.
fn nombre_archivo_seguro(nombre: &str) -> Result<String, String> {
    if nombre.is_empty()
        || nombre == "."
        || nombre == ".."
        || nombre.contains('/')
        || nombre.contains('\\')
        || nombre.contains('\0')
        || std::path::Path::new(nombre).is_absolute()
        || nombre.contains(':')
    {
        return Err(format!("Nombre de archivo no válido: {:?}", nombre));
    }
    Ok(nombre.to_string())
}

#[tauri::command]
fn escribir_archivo_en_carpeta(
    carpeta: String,
    nombre_archivo: String,
    contenido: String,
) -> Result<(), String> {
    let seguro = nombre_archivo_seguro(&nombre_archivo)?;
    let ruta = std::path::Path::new(&carpeta).join(&seguro);
    escribir_atomico(&ruta, contenido.as_bytes())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            guardar_proyecto,
            cargar_proyecto,
            existe_archivo,
            analizar_volumen,
            leer_archivo_texto,
            escribir_archivo_texto,
            escribir_archivo_en_carpeta,
            existe_cache_volumen,
            cargar_cache_volumen,
            listar_fuentes_sistema,
        ])
        .setup(|app| {
            let nuevo = MenuItemBuilder::new("Nuevo proyecto")
                .id("nuevo_proyecto")
                .build(app)?;
            let abrir = MenuItemBuilder::new("Abrir proyecto")
                .id("abrir_proyecto")
                .build(app)?;
            let guardar = MenuItemBuilder::new("Guardar proyecto")
                .id("guardar_proyecto")
                .build(app)?;
            let guardar_como = MenuItemBuilder::new("Guardar como...")
                .id("guardar_como")
                .build(app)?;
            let abrir_video = MenuItemBuilder::new("Abrir video")
                .id("abrir_video")
                .build(app)?;
            let cargar_srt = MenuItemBuilder::new("Cargar SRT")
                .id("cargar_srt")
                .build(app)?;
            let cargar_ass = MenuItemBuilder::new("Cargar .ass")
                .id("cargar_ass")
                .build(app)?;
            let importar_autosubs = MenuItemBuilder::new("Importar auto-subs (SRT+TXT)")
                .id("importar_autosubs")
                .build(app)?;

            let menu_archivo = SubmenuBuilder::new(app, "Archivo")
                .item(&nuevo)
                .item(&abrir)
                .item(&guardar)
                .item(&guardar_como)
                .separator()
                .item(&abrir_video)
                .item(&cargar_srt)
                .item(&cargar_ass)
                .item(&importar_autosubs)
                .build()?;

            let exportar_srt = MenuItemBuilder::new("Exportar SRT por hablante")
                .id("exportar_srt_hablantes")
                .build(app)?;
            let exportar_json = MenuItemBuilder::new("Exportar JSON combinado")
                .id("exportar_json")
                .build(app)?;
            let exportar_ass = MenuItemBuilder::new("Exportar .ass...")
                .id("exportar_ass")
                .build(app)?;

            let menu_exportar = SubmenuBuilder::new(app, "Exportar")
                .item(&exportar_srt)
                .item(&exportar_json)
                .item(&exportar_ass)
                .build()?;

            let gestionar_presets = MenuItemBuilder::new("Estilos...")
                .id("gestionar_presets")
                .build(app)?;

            let menu_estilos = SubmenuBuilder::new(app, "Estilos")
                .item(&gestionar_presets)
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&menu_archivo)
                .item(&menu_exportar)
                .item(&menu_estilos)
                .build()?;
            app.set_menu(menu)?;
            Ok(())
        })
        .on_menu_event(|app, event| {
            let _ = app.emit(event.id().as_ref(), ());
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proyecto_value(preset: Option<&str>) -> serde_json::Value {
        let mut hablante =
            serde_json::json!({"id": "h1", "nombre": "A", "tecla": "1", "color": "#fff"});
        if let Some(pid) = preset {
            hablante["presetId"] = pid.into();
        }
        serde_json::json!({"ruta_video": "v.mp4", "captions": [], "hablantes": [hablante]})
    }

    #[test]
    fn preset_id_sobrevive_roundtrip() {
        let p: Proyecto = serde_json::from_value(proyecto_value(Some("p1"))).expect("parse");
        assert_eq!(p.hablantes[0].preset_id.as_deref(), Some("p1"));
        let de_vuelta = serde_json::to_string(&p).expect("serialize");
        assert!(de_vuelta.contains("\"presetId\":\"p1\""));
    }

    #[test]
    fn proyecto_viejo_sin_preset_id_ni_playhead() {
        let p: Proyecto = serde_json::from_value(proyecto_value(None)).expect("parse");
        assert_eq!(p.hablantes[0].preset_id, None);
        assert_eq!(p.playhead, 0.0);
    }

    #[test]
    fn texto_utf16le_con_bom() {
        let original = "1\n00:00:00,000 --> 00:00:01,000\ncanci\u{f3}n\n";
        let mut bytes = vec![0xFF, 0xFE];
        for u in original.encode_utf16() {
            bytes.extend_from_slice(&u.to_le_bytes());
        }
        assert_eq!(decodificar_texto(&bytes), original);
    }

    #[test]
    fn texto_utf16be_con_bom() {
        let original = "1\n00:00:00,000 --> 00:00:01,000\ncanci\u{f3}n\n";
        let mut bytes = vec![0xFE, 0xFF];
        for u in original.encode_utf16() {
            bytes.extend_from_slice(&u.to_be_bytes());
        }
        assert_eq!(decodificar_texto(&bytes), original);
    }

    #[test]
    fn texto_utf8_con_y_sin_bom() {
        let original = "1\n00:00:00,000 --> 00:00:01,000\ncanción\n";
        assert_eq!(decodificar_texto(original.as_bytes()), original);
        let mut con_bom = vec![0xEF, 0xBB, 0xBF];
        con_bom.extend_from_slice(original.as_bytes());
        assert_eq!(decodificar_texto(&con_bom), original);
    }

    #[test]
    fn texto_latin1_con_tildes_cae_a_windows1252() {
        // "canción" en Latin-1 / Windows-1252: ó = 0xF3 (UTF-8 inválido solo).
        let bytes = b"1\n00:00:00,000 --> 00:00:01,000\ncanci\xf3n\n";
        assert_eq!(
            decodificar_texto(bytes),
            "1\n00:00:00,000 --> 00:00:01,000\ncanción\n"
        );
    }

    #[test]
    fn nombre_archivo_normal_ok() {
        assert_eq!(
            nombre_archivo_seguro("subtitulos.srt").as_deref(),
            Ok("subtitulos.srt")
        );
    }

    #[test]
    fn nombre_archivo_parent_err() {
        assert!(nombre_archivo_seguro("../x").is_err());
    }

    #[test]
    fn nombre_archivo_absoluto_err() {
        assert!(nombre_archivo_seguro("/abs").is_err());
    }

    #[test]
    fn nombre_archivo_con_subruta_err() {
        assert!(nombre_archivo_seguro("a/b").is_err());
    }

    // --- Fuentes del sistema ---
    //
    // `normalizar_familias` es la parte pura y testeable del listado: recibe
    // los nombres de familia tal como venga la fuente de verdad y devuelve la
    // lista limpia (dedup + orden). La lectura real del sistema vive en
    // `listar_fuentes_sistema` (fontdb) y NO se testea aquí porque depende del
    // SO. Estos tests fijan el contrato de nombres que el frontend consume.

    #[test]
    fn familias_dedup_y_orden() {
        let crudas = vec!["Zapfino", "Arial", "Arial", "Bebas Neue"];
        let familias = normalizar_familias(crudas.into_iter().map(String::from).collect());
        assert_eq!(familias, vec!["Arial", "Bebas Neue", "Zapfino"]);
    }

    #[test]
    fn familias_descarta_vacias_y_blancos() {
        let crudas = vec!["  ", "", "Arial"];
        let familias = normalizar_familias(crudas.into_iter().map(String::from).collect());
        assert_eq!(familias, vec!["Arial"]);
    }

    #[test]
    fn familias_ignora_nombres_tecnicamente_invalidos() {
        // Una familia no puede llevar un separador de ruta ni un punto y coma
        // (el .ass lo usa como delimitador de campos de Style:). Si el parser
        // de fuentes devolviera algo así, no debe llegar al selector.
        let crudas = vec!["Arial", "C:/Windows/Fonts", "Mala;Sana", "Arial"];
        let familias = normalizar_familias(crudas.into_iter().map(String::from).collect());
        assert_eq!(familias, vec!["Arial"]);
    }

    // Un hablante llamado "Narrador..." producía "Narrador....srt", que el
    // contains("..") de antes rechazaba y abortaba TODO el export por hablante
    // (los archivos ya escritos quedaban, el último no). La travesía real solo
    // puede ocurrir con separadores de ruta, que se rechazan aparte: los puntos
    // sueltos dentro de un nombre de archivo son legales.
    #[test]
    fn nombre_archivo_con_puntos_dentro_ok() {
        assert_eq!(
            nombre_archivo_seguro("Narrador....srt").as_deref(),
            Ok("Narrador....srt")
        );
        assert_eq!(nombre_archivo_seguro("a..b.srt").as_deref(), Ok("a..b.srt"));
    }
}
