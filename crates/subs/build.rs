use std::path::{Path, PathBuf};

/// Enlaza libass de vcpkg. El FFI está declarado a mano; esto solo resuelve
/// dónde están `ass.lib` y las DLLs de sus dependencias.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=ASS_DIR");

    if cfg!(windows) {
        let dir = buscar().unwrap_or_else(|| {
            panic!(
                "no encuentro libass. Instálalo con `vcpkg install libass:x64-windows` \
                 o define ASS_DIR apuntando a installed/x64-windows"
            )
        });

        println!(
            "cargo:rustc-link-search=native={}",
            dir.join("lib").display()
        );
        println!("cargo:rustc-link-lib=ass");
        copiar_dlls(&dir.join("bin"));
    } else {
        pkg_config::Config::new()
            .probe("libass")
            .expect("no encuentro libass. Instala el paquete de desarrollo libass");
    }
}

/// Directorios candidatos, en orden: variable explícita, vcpkg del usuario, y
/// las rutas típicas. El primero que tenga `lib/ass.lib` gana.
fn buscar() -> Option<PathBuf> {
    let mut candidatos: Vec<PathBuf> = Vec::new();

    if let Ok(dir) = std::env::var("ASS_DIR") {
        candidatos.push(PathBuf::from(dir));
    }
    if let Ok(root) = std::env::var("VCPKG_ROOT") {
        candidatos.push(Path::new(&root).join("installed/x64-windows"));
    }
    if let Ok(root) = std::env::var("VCPKG_INSTALLATION_ROOT") {
        candidatos.push(Path::new(&root).join("installed/x64-windows"));
    }
    if let Ok(home) = std::env::var("USERPROFILE") {
        candidatos.push(Path::new(&home).join("vcpkg/installed/x64-windows"));
    }
    candidatos.push(PathBuf::from("C:/vcpkg/installed/x64-windows"));

    candidatos
        .into_iter()
        .find(|dir| dir.join("lib/ass.lib").exists())
}

/// Copia las DLLs junto a los ejecutables. En Windows la búsqueda de DLLs
/// empieza por el directorio del `.exe`, así que sin esto cada `cargo run` o
/// `cargo test` exige añadir `vcpkg/installed/x64-windows/bin` al PATH.
///
/// Cargo 1.100 cambió dónde deja los binarios de test de un paquete con build
/// script: pasan a `build/<paquete>/<hash>/out`, un directorio que el build
/// script no puede conocer (su propio `OUT_DIR` tiene otro hash). Ahí la DLL no
/// queda al lado y el proceso no arranca (`STATUS_DLL_NOT_FOUND`), así que la
/// única salida sin tocar el enlazado es el PATH: el CI lo añade, y en local
/// `$env:PATH += ';%USERPROFILE%\vcpkg\installed\x64-windows\bin'` lo resuelve.
fn copiar_dlls(origen: &Path) {
    const DLLS: &[&str] = &[
        "ass-9.dll",
        "freetype.dll",
        "fribidi-0.dll",
        "harfbuzz.dll",
        "iconv-2.dll",
        "libpng16.dll",
        "brotlicommon.dll",
        "brotlidec.dll",
        "bz2.dll",
        "z.dll",
    ];

    // OUT_DIR = <target>/<perfil>/build/<paquete>/<hash>/out (cargo 1.100) o
    // <target>/<perfil>/build/<paquete>-<hash>/out (anteriores). Subir tres
    // niveles da el directorio del perfil en los dos formatos.
    let Some(perfil) = std::env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .and_then(|out| out.parent()?.parent()?.parent().map(Path::to_path_buf))
    else {
        return;
    };

    for destino in [perfil.clone(), perfil.join("examples"), perfil.join("deps")] {
        if std::fs::create_dir_all(&destino).is_err() {
            continue;
        }
        for dll in DLLS {
            let desde = origen.join(dll);
            if desde.exists() {
                let _ = std::fs::copy(&desde, destino.join(dll));
            }
        }
    }
}
