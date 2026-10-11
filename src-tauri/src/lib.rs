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

// Hijos sin flash de consola: en Windows, cada proceso hijo de una app GUI
// abre su propia ventana de consola por un instante (powershell, reg).
// CREATE_NO_WINDOW lo evita; los pipes (.output()) siguen funcionando igual.
#[cfg(target_os = "windows")]
fn comando_oculto(programa: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    let mut cmd = std::process::Command::new(programa);
    cmd.creation_flags(0x08000000);
    cmd
}
#[cfg(not(target_os = "windows"))]
fn comando_oculto(programa: &str) -> std::process::Command {
    std::process::Command::new(programa)
}

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
                    let _ = app.emit("volumen_chunk", (track_index, nuevo_chunk));
                    ultimo_flush_idx = resultados.len();
                }
                ultimo_emit = Instant::now();
            }
        }

        if ultimo_flush_idx < resultados.len() {
            let nuevo_chunk = resultados[ultimo_flush_idx..].to_vec();
            let _ = app.emit("volumen_chunk", (track_index, nuevo_chunk));
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
/// .ass. Sale de GDI+/DirectWrite, que da los nombres de FAMILIA reales
/// ("Bebas Neue"), que es lo que resuelven CSS y libass. Medido: 186 familias
/// limpias en ~400 ms con el spawn de powershell.exe incluido (pwsh tarda 3x).
///
/// El registro de Windows se usa solo de fallback: devuelve el nombre con el
/// estilo pegado ("Bebas Neue Regular") y 110 de sus 201 entradas no son
/// familias sino estilos ("Arial Bold", "Calibri Bold Italic"), que el filtro
/// del frontend descarta midiendo pero no puede renombrar.
#[tauri::command]
async fn listar_fuentes_sistema() -> Result<Vec<String>, String> {
    // Async + spawn_blocking: powershell tarda ~400 ms y como comando
    // sync congelaba el hilo principal al abrir la app (el panel de
    // Estilos pide las fuentes al montar).
    tauri::async_runtime::spawn_blocking(|| {
        // Windows: GDI+ da las familias reales (ver doc arriba).
        #[cfg(target_os = "windows")]
        {
            const PS: &str = "Add-Type -AssemblyName System.Drawing; [System.Drawing.Text.InstalledFontCollection]::new().Families | ForEach-Object { $_.Name }";
            if let Ok(salida) = comando_oculto("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", PS])
                .output()
            {
                let familias: Vec<String> = String::from_utf8_lossy(&salida.stdout)
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
                if !familias.is_empty() {
                    return Ok(familias);
                }
            }
            Ok(familias_del_registro())
        }
        // Linux: fontconfig (`fc-list : family`, una familia por línea; las
        // entradas multi-familia vienen separadas por coma). Sin deps nuevas.
        #[cfg(target_os = "linux")]
        {
            Ok(familias_fc_list())
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Ok(Vec::new())
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(target_os = "linux")]
fn familias_fc_list() -> Vec<String> {
    use std::collections::HashSet;
    let Ok(salida) = comando_oculto("fc-list").args([":", "family"]).output() else {
        return Vec::new();
    };
    let mut vistas: HashSet<String> = HashSet::new();
    let mut todas: Vec<String> = Vec::new();
    for linea in String::from_utf8_lossy(&salida.stdout).lines() {
        for parte in linea.split(',') {
            let nombre = parte.trim().to_string();
            if !nombre.is_empty() && vistas.insert(nombre.clone()) {
                todas.push(nombre);
            }
        }
    }
    todas.sort();
    todas
}

#[cfg(target_os = "windows")]
fn familias_del_registro() -> Vec<String> {
    let claves = [
        r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts",
        r"HKCU\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts",
    ];
    let mut todas: Vec<String> = Vec::new();
    for clave in claves {
        // Si la clave no existe (típico en HKCU sin fuentes de usuario), reg
        // escribe en stderr y devuelve código 1: no es un error.
        let Ok(salida) = comando_oculto("reg").args(["query", clave]).output() else {
            continue;
        };
        for linea in String::from_utf8_lossy(&salida.stdout).lines() {
            let t = linea.trim();
            if t.is_empty() || t.starts_with("HKEY") {
                continue;
            }
            // Cada línea: "    <nombre>    <tipo>    <ruta del archivo>"
            if let Some(nombre) = t.split_whitespace().next() {
                if !nombre.is_empty() {
                    todas.push(nombre.to_string());
                }
            }
        }
    }
    todas
}

// Un nombre de archivo nunca debe escapar de su carpeta: el frontend
// sanitiza, pero este comando es el choke point y no puede confiar en eso.
// Rechaza `..`, separadores y rutas absolutas (ambos sabores, para que un
// `C:\x` no cuele en Linux y viceversa).
fn nombre_archivo_seguro(nombre: &str) -> Result<String, String> {
    if nombre.is_empty()
        || nombre == "."
        || nombre == ".."
        || nombre.contains("..")
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
}
