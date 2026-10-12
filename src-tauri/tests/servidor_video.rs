//! Pruebas de integracion del servidor de video contra el socket real.
//!
//! Los tests de `interpretar_rango` verifican la matematica; estos verifican el
//! cableado completo: socket -> parseo de peticion -> respuesta HTTP con los
//! bytes correctos y los headers que GStreamer necesita (Accept-Ranges,
//! Content-Range, Content-Length). Es la capa donde un error tipico (header mal
//! escrito, body desalineado, 206 sin Content-Range) rompe la reproduccion sin
//! que ningun test unitario lo note.
//!
//! Solo corre en Linux: el servidor esta gated con `#[cfg(target_os = "linux")]`.

#![cfg(target_os = "linux")]

use opendialogue_lib::servidor_video::{arrancar, estado_nuevo, fijar};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Levanta el servidor con un archivo de `tamano` bytes y devuelve
/// (puerto, ruta temporal, token).
async fn servidor_de_prueba(tamano: usize) -> (u16, std::path::PathBuf) {
    let ruta = std::env::temp_dir().join(format!("od-test-{}-{}.bin", std::process::id(), tamano));
    let datos: Vec<u8> = (0..tamano).map(|i| (i % 251) as u8).collect();
    std::fs::write(&ruta, datos).expect("escribir archivo de prueba");

    let estado = estado_nuevo();
    fijar(&estado, 7, ruta.clone());
    let puerto = arrancar(estado).await.expect("arrancar servidor");
    (puerto, ruta)
}

/// Envia una peticion cruda y devuelve (linea_estado, headers, cuerpo).
async fn pedir(puerto: u16, peticion: &str) -> (String, Vec<(String, String)>, Vec<u8>) {
    let mut socket = tokio::net::TcpStream::connect(("127.0.0.1", puerto))
        .await
        .expect("conectar");
    socket.write_all(peticion.as_bytes()).await.expect("enviar");

    let mut todo = Vec::new();
    socket.read_to_end(&mut todo).await.expect("leer respuesta");

    let corte = todo
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("respuesta con cabeceras");
    let cabeceras = String::from_utf8_lossy(&todo[..corte]).into_owned();
    let cuerpo = todo[corte + 4..].to_vec();

    let mut lineas = cabeceras.split("\r\n");
    let estado = lineas.next().unwrap_or("").to_string();
    let headers = lineas
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
        .collect();
    (estado, headers, cuerpo)
}

fn valor(headers: &[(String, String)], clave: &str) -> Option<String> {
    headers
        .iter()
        .find(|(k, _)| k == clave)
        .map(|(_, v)| v.clone())
}

#[tokio::test]
async fn sin_range_devuelve_el_archivo_entero() {
    let (puerto, _ruta) = servidor_de_prueba(1000).await;
    let (estado, headers, cuerpo) =
        pedir(puerto, "GET /video?t=7 HTTP/1.1\r\nHost: x\r\n\r\n").await;

    assert!(estado.contains("200"), "estado fue {estado}");
    assert_eq!(cuerpo.len(), 1000);
    assert_eq!(valor(&headers, "content-length").as_deref(), Some("1000"));
    assert_eq!(
        valor(&headers, "accept-ranges").as_deref(),
        Some("bytes"),
        "GStreamer necesita Accept-Ranges para pedir rangos"
    );
}

#[tokio::test]
async fn range_parcial_devuelve_206_con_los_bytes_pedidos() {
    let (puerto, _ruta) = servidor_de_prueba(1000).await;
    let (estado, headers, cuerpo) = pedir(
        puerto,
        "GET /video?t=7 HTTP/1.1\r\nRange: bytes=100-199\r\n\r\n",
    )
    .await;

    assert!(estado.contains("206"), "estado fue {estado}");
    assert_eq!(cuerpo.len(), 100, "deben ser 100 bytes (100..=199)");
    assert_eq!(
        valor(&headers, "content-range").as_deref(),
        Some("bytes 100-199/1000")
    );
    assert_eq!(valor(&headers, "content-length").as_deref(), Some("100"));

    // El contenido tiene que ser el del archivo en ese offset, no el inicio.
    let esperado: Vec<u8> = (100..200).map(|i| (i % 251) as u8).collect();
    assert_eq!(cuerpo, esperado, "los bytes no corresponden al rango");
}

#[tokio::test]
async fn range_abierto_cubre_hasta_el_final() {
    let (puerto, _ruta) = servidor_de_prueba(1000).await;
    let (estado, _h, cuerpo) = pedir(
        puerto,
        "GET /video?t=7 HTTP/1.1\r\nRange: bytes=900-\r\n\r\n",
    )
    .await;

    assert!(estado.contains("206"));
    assert_eq!(cuerpo.len(), 100, "900..=999 son 100 bytes");
}

#[tokio::test]
async fn token_invalido_no_sirve_el_archivo() {
    // Un <video> del proyecto anterior no puede leer el archivo nuevo.
    let (puerto, _ruta) = servidor_de_prueba(1000).await;
    let (estado, _h, cuerpo) = pedir(puerto, "GET /video?t=999 HTTP/1.1\r\n\r\n").await;

    assert!(estado.contains("404"), "estado fue {estado}");
    assert!(cuerpo.is_empty(), "no debe filtrar bytes del video");
}

#[tokio::test]
async fn sin_token_no_sirve_el_archivo() {
    let (puerto, _ruta) = servidor_de_prueba(1000).await;
    let (estado, _h, _cuerpo) = pedir(puerto, "GET /video HTTP/1.1\r\n\r\n").await;
    assert!(estado.contains("403"), "estado fue {estado}");
}

#[tokio::test]
async fn otra_ruta_es_404() {
    let (puerto, _ruta) = servidor_de_prueba(1000).await;
    let (estado, _h, _cuerpo) = pedir(puerto, "GET /etc/passwd?t=7 HTTP/1.1\r\n\r\n").await;
    assert!(estado.contains("404"), "estado fue {estado}");
}

#[tokio::test]
async fn head_anuncia_el_tamano_sin_cuerpo() {
    let (puerto, _ruta) = servidor_de_prueba(1000).await;
    let (estado, headers, cuerpo) = pedir(
        puerto,
        "HEAD /video?t=7 HTTP/1.1\r\nRange: bytes=0-99\r\n\r\n",
    )
    .await;

    assert!(estado.contains("206"), "estado fue {estado}");
    assert!(cuerpo.is_empty(), "HEAD no lleva cuerpo");
    assert_eq!(valor(&headers, "content-length").as_deref(), Some("100"));
}

#[tokio::test]
async fn post_es_rechazado() {
    let (puerto, _ruta) = servidor_de_prueba(1000).await;
    let (estado, _h, _cuerpo) = pedir(puerto, "POST /video?t=7 HTTP/1.1\r\n\r\n").await;
    assert!(estado.contains("405"), "estado fue {estado}");
}

#[tokio::test]
async fn content_type_segun_extension() {
    let ruta = std::env::temp_dir().join(format!("od-test-mime-{}.mkv", std::process::id()));
    std::fs::write(&ruta, b"x").expect("escribir");
    let estado = estado_nuevo();
    fijar(&estado, 3, ruta.clone());
    let puerto = arrancar(estado).await.expect("arrancar");
    let (_e, headers, _c) = pedir(puerto, "GET /video?t=3 HTTP/1.1\r\n\r\n").await;

    assert_eq!(
        valor(&headers, "content-type").as_deref(),
        Some("video/x-matroska")
    );
    let _ = std::fs::remove_file(&ruta);
}
