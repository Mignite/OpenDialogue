//! En Windows el exe de los tests unitarios de `rod-app` acaba en
//! `target/debug/build/rod-app/<hash>/out`, que es este `OUT_DIR`. El
//! build script de `rod-player` copia las DLLs a `target/debug` y `deps`,
//! pero no puede conocer este directorio: sin esta copia el proceso muere
//! con `STATUS_DLL_NOT_FOUND` antes de `main` (lo que en CI aparece como
//! «test failed» sin ninguna salida útil).

use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if !cfg!(windows) {
        return;
    }
    let Ok(out_raw) = std::env::var("OUT_DIR") else {
        return;
    };
    let out = PathBuf::from(out_raw);
    let raiz = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());

    let mut fuentes = vec![raiz.join("../../third_party/mpv")];
    for variable in ["VCPKG_ROOT", "VCPKG_INSTALLATION_ROOT"] {
        if let Ok(valor) = std::env::var(variable) {
            fuentes.push(PathBuf::from(valor).join("installed/x64-windows/bin"));
        }
    }
    if let Ok(valor) = std::env::var("ASS_DIR") {
        fuentes.push(PathBuf::from(valor).join("bin"));
    }

    let mut destinos = vec![out.clone()];
    if let Some(perfil) = out
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
    {
        destinos.push(perfil.join("deps"));
        destinos.push(perfil.to_path_buf());
    }

    for fuente in fuentes {
        let Ok(entradas) = std::fs::read_dir(&fuente) else {
            continue;
        };
        for entrada in entradas.flatten() {
            let ruta = entrada.path();
            if ruta.extension().and_then(|e| e.to_str()) != Some("dll") {
                continue;
            }
            let nombre = match ruta.file_name() {
                Some(n) => n,
                None => continue,
            };
            for destino in &destinos {
                let _ = std::fs::copy(&ruta, destino.join(nombre));
            }
        }
    }
}
