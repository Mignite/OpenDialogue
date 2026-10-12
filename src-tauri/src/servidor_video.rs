//! Servidor HTTP local para reproducir el video en Linux.
//!
//! Por qué existe: WebKitGTK resuelve `<video>` con el pipeline de media de
//! GStreamer, que NO consulta el handler de esquema custom que Tauri registra
//! para `asset://`. En Linux, por lo tanto, `convertFileSrc()` produce una URL
//! que GStreamer no sabe abrir y el `<video>` dispara `error` con
//! MEDIA_ERR_SRC_NOT_SUPPORTED (tauri#3725 / WebKit#146351). En Windows y
//! macOS el webview usa WebView2/WebKit y sí lo resuelve.
//!
//! El rodeo es servir el archivo por un puerto loopback real, que GStreamer sí
//! entiende (protocolo `http://` nativo). Solo se sirve el video que la app
//! tiene abierto: el comando `url_video_local` fija la ruta en un estado de
//! un solo slot, así el servidor no es un generador de archivos arbitrarios.

/// Qué bytes servir para una petición, según el header `Range`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RespuestaRango {
    /// Sin `Range` (o ilegible): el archivo entero, respuesta 200. Lleva el
    /// tamano porque es justamente lo que hay que escribir.
    Completo { longitud: u64 },
    /// `Range` válido: bytes `[inicio, fin]` **inclusivos**, respuesta 206.
    Parcial { inicio: u64, fin: u64 },
    /// `Range` fuera del archivo: respuesta 416, cero bytes.
    NoSatisfacible,
}

impl RespuestaRango {
    /// Cantidad de bytes a escribir (`fin` es inclusivo, por eso el `+1`).
    pub fn longitud(&self) -> u64 {
        match *self {
            RespuestaRango::Completo { longitud } => longitud,
            RespuestaRango::Parcial { inicio, fin } => fin.saturating_sub(inicio) + 1,
            RespuestaRango::NoSatisfacible => 0,
        }
    }
}

/// Traduce el header `Range` a los bytes a servir. Solo se acepta la forma
/// `bytes=<inicio>-<fin>` (con `fin` opcional); cualquier otra cosa se ignora
/// y se sirve el archivo completo, porque cortar el video en el primer byte
/// por un header raro es peor que ignorarlo.
pub fn interpretar_rango(header: Option<&str>, tamano: u64) -> RespuestaRango {
    if tamano == 0 {
        return RespuestaRango::Completo { longitud: tamano };
    }
    let Some(header) = header else {
        return RespuestaRango::Completo { longitud: tamano };
    };
    let Some(especificacion) = header.trim().strip_prefix("bytes=") else {
        return RespuestaRango::Completo { longitud: tamano };
    };
    // "bytes=0-1,4-5" son rangos multiples: no lo soportamos, va completo.
    if especificacion.contains(',') {
        return RespuestaRango::Completo { longitud: tamano };
    }
    let Some((inicio, fin)) = especificacion.trim().split_once('-') else {
        return RespuestaRango::Completo { longitud: tamano };
    };
    let inicio = inicio.trim();
    let fin = fin.trim();

    // Sufijo "bytes=-N": los ultimos N bytes.
    if inicio.is_empty() {
        let n: u64 = match fin.parse() {
            Ok(n) if n > 0 => n,
            _ => return RespuestaRango::Completo { longitud: tamano },
        };
        return RespuestaRango::Parcial {
            inicio: tamano.saturating_sub(n),
            fin: tamano - 1,
        };
    }

    let inicio: u64 = match inicio.parse() {
        Ok(i) => i,
        Err(_) => return RespuestaRango::Completo { longitud: tamano },
    };
    if inicio >= tamano {
        return RespuestaRango::NoSatisfacible;
    }
    let fin: u64 = if fin.is_empty() {
        tamano - 1
    } else {
        match fin.parse::<u64>() {
            Ok(f) => f.min(tamano - 1),
            // "bytes=10-abc": el inicio si se entiende. Servir hasta el final
            // es lo que espera el player.
            Err(_) => tamano - 1,
        }
    };
    if fin < inicio {
        return RespuestaRango::Completo { longitud: tamano };
    }
    RespuestaRango::Parcial { inicio, fin }
}

/// Content-Type por extension. Los contenedores que la app abre con symphonia
/// estan cubiertos; lo demas cae a `video/mp4` porque GStreamer necesita un
/// tipo concreto para aceptar el flujo.
pub fn tipo_mime(ruta: &str) -> &'static str {
    let extension = ruta
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .unwrap_or_default();
    match extension.as_str() {
        "mkv" => "video/x-matroska",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "m4v" => "video/x-m4v",
        "avi" => "video/x-msvideo",
        "mpg" | "mpeg" => "video/mpeg",
        "ogv" => "video/ogg",
        _ => "video/mp4",
    }
}

/// Video que el servidor está autorizado a servir ahora mismo. Un solo slot:
/// abrir otro video reemplaza la entrada, asi que el servidor nunca sirve un
/// archivo arbitrario pedido por la red.
///
/// Es un `Arc<Mutex<...>>` para poder clonar el handle y compartirlo entre la
/// tarea del servidor TCP y el comando de Tauri sin dances de ownership.
pub type EstadoVideo = std::sync::Arc<std::sync::Mutex<Option<(u64, std::path::PathBuf)>>>;

pub fn estado_nuevo() -> EstadoVideo {
    std::sync::Arc::new(std::sync::Mutex::new(None))
}

pub fn fijar(estado: &EstadoVideo, token: u64, ruta: std::path::PathBuf) {
    if let Ok(mut guard) = estado.lock() {
        *guard = Some((token, ruta));
    }
}

pub fn resolver(estado: &EstadoVideo, token: u64) -> Option<std::path::PathBuf> {
    let guard = estado.lock().ok()?;
    let (token_guardado, ruta) = guard.as_ref()?;
    (*token_guardado == token).then(|| ruta.clone())
}

/// Token monotónico por video abierto. El frontend lo recibe en la URL y el
/// servidor solo sirve si coincide: un `<video>` del proyecto anterior que
/// llega tarde recibe 404 en vez de los bytes del video nuevo.
#[cfg(target_os = "linux")]
static TOKEN_VIDEO: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Arranca el servidor en un puerto efimero de loopback y devuelve el puerto.
/// Escucha solo en 127.0.0.1: no es alcanzable desde la red.
#[cfg(target_os = "linux")]
pub async fn arrancar(estado: EstadoVideo) -> std::io::Result<u16> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let puerto = listener.local_addr()?.port();
    tokio::spawn(async move {
        loop {
            let Ok((socket, _)) = listener.accept().await else {
                continue;
            };
            let estado = estado.clone();
            tokio::spawn(async move {
                let _ = atender(socket, estado).await;
            });
        }
    });
    Ok(puerto)
}

/// URL de reproduccion para `ruta`, o `None` en plataformas donde `asset://`
/// ya funciona (Windows/macOS) y este servidor no debe usarse.
#[tauri::command]
pub fn url_video_local(
    estado: tauri::State<'_, EstadoVideo>,
    ruta: String,
) -> Result<Option<String>, String> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (estado, ruta);
        Ok(None)
    }
    #[cfg(target_os = "linux")]
    {
        let puerto = *PUERTO_VIDEO
            .lock()
            .map_err(|_| "servidor de video no inicializado")?;
        let token = TOKEN_VIDEO.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        fijar(&estado, token, std::path::PathBuf::from(ruta));
        Ok(Some(format!("http://127.0.0.1:{puerto}/video?t={token}")))
    }
}

/// Guarda el puerto asignado al arrancar. Lo consulta el comando por request.
#[cfg(target_os = "linux")]
pub fn fijar_puerto(puerto: u16) {
    if let Ok(mut guard) = PUERTO_VIDEO.lock() {
        *guard = puerto;
    }
}

#[cfg(target_os = "linux")]
static PUERTO_VIDEO: std::sync::LazyLock<std::sync::Mutex<u16>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(0));

/// Extrae el `t=` de la query. `None` si no viene o no parsea.
pub fn token_de_query(query: &str) -> Option<u64> {
    query
        .split('&')
        .find_map(|par| par.strip_prefix("t="))
        .and_then(|v| v.parse().ok())
}

/// Escribe solo las cabeceras, con `Content-Length` explicito (no derivado de
/// un body en memoria: el cuerpo se transmite despues, por chunks).
#[cfg(target_os = "linux")]
async fn escribir_cabeceras(
    socket: &mut tokio::net::TcpStream,
    estado_linea: &str,
    headers: &[(String, String)],
    longitud: u64,
) -> std::io::Result<()> {
    use tokio::io::AsyncWriteExt;
    let mut respuesta = format!("HTTP/1.1 {estado_linea}\r\n");
    for (clave, valor) in headers {
        respuesta.push_str(&format!("{clave}: {valor}\r\n"));
    }
    respuesta.push_str(&format!("Content-Length: {longitud}\r\n"));
    respuesta.push_str("Connection: close\r\n\r\n");
    socket.write_all(respuesta.as_bytes()).await?;
    socket.flush().await
}

/// Lee la peticion, resuelve el video y responde con `Range` si lo piden.
#[cfg(target_os = "linux")]
async fn atender(mut socket: tokio::net::TcpStream, estado: EstadoVideo) -> std::io::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    // Cabecera: hasta 8 KB o hasta el blank line.
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];
    let cabecera = loop {
        let leidos = socket.read(&mut chunk).await?;
        if leidos == 0 {
            break String::new();
        }
        buffer.extend_from_slice(&chunk[..leidos]);
        if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
            break String::from_utf8_lossy(&buffer[..pos]).into_owned();
        }
        if buffer.len() > 8192 {
            break String::new();
        }
    };
    if cabecera.is_empty() {
        let _ = escribir_cabeceras(
            &mut socket,
            "400 Bad Request",
            &[("Content-Type".into(), "text/plain".into())],
            0,
        )
        .await;
        return Ok(());
    }

    let mut lineas = cabecera.split("\r\n");
    let Some(peticion) = lineas.next() else {
        return Ok(());
    };
    let mut partes = peticion.split_whitespace();
    let metodo = partes.next().unwrap_or("");
    let objetivo = partes.next().unwrap_or("");

    // Solo GET/HEAD a /video; el resto 404 (no es un servidor de archivos).
    let Some(query) = objetivo.strip_prefix("/video") else {
        let _ = escribir_cabeceras(
            &mut socket,
            "404 Not Found",
            &[("Content-Type".into(), "text/plain".into())],
            0,
        )
        .await;
        return Ok(());
    };
    let solo_head = metodo == "HEAD";
    if metodo != "GET" && !solo_head {
        let _ = escribir_cabeceras(
            &mut socket,
            "405 Method Not Allowed",
            &[("Content-Type".into(), "text/plain".into())],
            0,
        )
        .await;
        return Ok(());
    }
    // Sin query no hay token: es una peticion sin autenticar, no un metodo
    // incorrecto, asi que responde 403 y no 405.
    if !query.starts_with('?') {
        let _ = escribir_cabeceras(
            &mut socket,
            "403 Forbidden",
            &[("Content-Type".into(), "text/plain".into())],
            0,
        )
        .await;
        return Ok(());
    }

    let Some(token) = token_de_query(&query[1..]) else {
        let _ = escribir_cabeceras(
            &mut socket,
            "403 Forbidden",
            &[("Content-Type".into(), "text/plain".into())],
            0,
        )
        .await;
        return Ok(());
    };
    let Some(ruta) = resolver(&estado, token) else {
        let _ = escribir_cabeceras(
            &mut socket,
            "404 Not Found",
            &[("Content-Type".into(), "text/plain".into())],
            0,
        )
        .await;
        return Ok(());
    };

    let Ok(metadatos) = tokio::fs::metadata(&ruta).await else {
        let _ = escribir_cabeceras(
            &mut socket,
            "404 Not Found",
            &[("Content-Type".into(), "text/plain".into())],
            0,
        )
        .await;
        return Ok(());
    };
    let tamano = metadatos.len();
    let mime = tipo_mime(&ruta.to_string_lossy());

    // Header Range (case-insensitive en el nombre).
    let mut header_rango = None;
    for linea in lineas {
        if let Some((clave, valor)) = linea.split_once(':') {
            if clave.trim().eq_ignore_ascii_case("range") {
                header_rango = Some(valor.trim().to_string());
            }
        }
    }

    let rango = interpretar_rango(header_rango.as_deref(), tamano);

    // Content-Length SIEMPRE es la longitud del rango, incluso en HEAD (que no
    // lleva cuerpo pero debe prometer el tamaño real) y en 416 (0 bytes). Por
    // eso el enum lleva la longitud en `Completo`: aca se lee de el.
    let longitud = rango.longitud();
    let inicio = match rango {
        RespuestaRango::Parcial { inicio, .. } => inicio,
        _ => 0,
    };

    let mut cabeceras = vec![
        ("Content-Type".to_string(), mime.to_string()),
        ("Accept-Ranges".to_string(), "bytes".to_string()),
    ];
    let estado_linea = match rango {
        RespuestaRango::Parcial { inicio, fin } => {
            cabeceras.push((
                "Content-Range".to_string(),
                format!("bytes {inicio}-{fin}/{tamano}"),
            ));
            "206 Partial Content"
        }
        RespuestaRango::Completo { .. } => "200 OK",
        RespuestaRango::NoSatisfacible => {
            cabeceras.push(("Content-Range".to_string(), format!("bytes */{tamano}")));
            "416 Range Not Satisfiable"
        }
    };

    escribir_cabeceras(&mut socket, estado_linea, &cabeceras, longitud).await?;

    // HEAD y 416 no llevan cuerpo. El resto se transmite por chunks: un video
    // de 2 h no cabe en memoria, y el player va pidiendo rangos chicos igual.
    if !solo_head && longitud > 0 {
        let mut archivo = tokio::fs::File::open(&ruta).await?;
        use tokio::io::AsyncSeekExt;
        // Un error de seek/read se propaga: el archivo se truncó o no es
        // legible mientras se servia, y es preferible cortar la conexion a
        // mandar bytes basura al decoder.
        archivo.seek(std::io::SeekFrom::Start(inicio)).await?;
        let mut restantes = longitud;
        let mut buffer = vec![0u8; 64 * 1024];
        while restantes > 0 {
            let trozo = std::cmp::min(restantes as usize, buffer.len());
            archivo.read_exact(&mut buffer[..trozo]).await?;
            socket.write_all(&buffer[..trozo]).await?;
            restantes -= trozo as u64;
        }
        socket.flush().await?;
    }

    let _ = socket.shutdown().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rango_ausente_sirve_archivo_completo() {
        assert_eq!(
            interpretar_rango(None, 100),
            RespuestaRango::Completo { longitud: 100 }
        );
    }

    #[test]
    fn rango_desde_cero_hasta_el_final() {
        assert_eq!(
            interpretar_rango(Some("bytes=0-"), 100),
            RespuestaRango::Parcial { inicio: 0, fin: 99 }
        );
    }

    #[test]
    fn rango_abierto_desde_offset() {
        assert_eq!(
            interpretar_rango(Some("bytes=10-"), 100),
            RespuestaRango::Parcial {
                inicio: 10,
                fin: 99
            }
        );
    }

    #[test]
    fn rango_cerrado_explicito() {
        assert_eq!(
            interpretar_rango(Some("bytes=10-20"), 100),
            RespuestaRango::Parcial {
                inicio: 10,
                fin: 20
            }
        );
    }

    #[test]
    fn rango_sufijo_pide_ultimos_bytes() {
        assert_eq!(
            interpretar_rango(Some("bytes=-20"), 100),
            RespuestaRango::Parcial {
                inicio: 80,
                fin: 99
            }
        );
    }

    #[test]
    fn rango_sufijo_mayor_que_el_archivo_toma_todo() {
        assert_eq!(
            interpretar_rango(Some("bytes=-500"), 100),
            RespuestaRango::Parcial { inicio: 0, fin: 99 }
        );
    }

    #[test]
    fn rango_que_empieza_despues_del_final_no_satisfacible() {
        assert_eq!(
            interpretar_rango(Some("bytes=200-"), 100),
            RespuestaRango::NoSatisfacible
        );
    }

    #[test]
    fn rango_que_termina_muy_alla_se_ajusta_al_tamanio() {
        assert_eq!(
            interpretar_rango(Some("bytes=50-9999"), 100),
            RespuestaRango::Parcial {
                inicio: 50,
                fin: 99
            }
        );
    }

    #[test]
    fn rango_de_un_solo_byte() {
        assert_eq!(
            interpretar_rango(Some("bytes=0-0"), 100),
            RespuestaRango::Parcial { inicio: 0, fin: 0 }
        );
    }

    #[test]
    fn rango_ilexible_cae_a_archivo_completo() {
        // Un Range que no se entiende no debe romper la carga: se sirve el
        // archivo entero (200) en vez de cortar el video en el primer byte.
        assert_eq!(
            interpretar_rango(Some("bytes=abc"), 100),
            RespuestaRango::Completo { longitud: 100 }
        );
    }

    #[test]
    fn rango_con_espacios_se_tolera() {
        assert_eq!(
            interpretar_rango(Some("bytes= 0-9 "), 100),
            RespuestaRango::Parcial { inicio: 0, fin: 9 }
        );
    }

    #[test]
    fn archivo_vacio_nunca_es_rango_parcial() {
        assert_eq!(
            interpretar_rango(Some("bytes=0-"), 0),
            RespuestaRango::Completo { longitud: 0 }
        );
    }

    #[test]
    fn mime_segun_extension() {
        assert_eq!(tipo_mime("a.mp4"), "video/mp4");
        assert_eq!(tipo_mime("a.MKV"), "video/x-matroska");
        assert_eq!(tipo_mime("a.webm"), "video/webm");
        assert_eq!(tipo_mime("a.mov"), "video/quicktime");
        assert_eq!(tipo_mime("a.m4v"), "video/x-m4v");
        assert_eq!(tipo_mime("a.avi"), "video/x-msvideo");
    }

    #[test]
    fn mime_desconocido_cae_a_mp4() {
        // GStreamer necesita un tipo concreto para abrir el flujo; antes que
        // inventar uno, el contenedor que la app maneja de verdad.
        assert_eq!(tipo_mime("a.xyz"), "video/mp4");
        assert_eq!(tipo_mime("sin_extension"), "video/mp4");
    }

    #[test]
    fn token_se_lectura_de_la_query() {
        assert_eq!(token_de_query("t=7"), Some(7));
        assert_eq!(token_de_query("a=1&t=42&b=2"), Some(42));
    }

    #[test]
    fn token_ausente_o_invalido_es_none() {
        assert_eq!(token_de_query(""), None);
        assert_eq!(token_de_query("a=1"), None);
        assert_eq!(token_de_query("t=abc"), None);
        assert_eq!(token_de_query("t="), None);
    }

    #[test]
    fn token_no_se_confunde_con_otro_parametro() {
        // "xt=5" no es el token: el prefijo debe ser exacto.
        assert_eq!(token_de_query("xt=5"), None);
    }

    #[test]
    fn longitud_de_rango_es_inclusiva() {
        let r = interpretar_rango(Some("bytes=0-9"), 100);
        assert_eq!(r.longitud(), 10);
        assert_eq!(interpretar_rango(Some("bytes=0-"), 100).longitud(), 100);
        assert_eq!(interpretar_rango(None, 100).longitud(), 100);
    }
}
