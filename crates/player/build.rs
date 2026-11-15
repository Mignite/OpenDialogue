fn main() {
    if cfg!(windows) {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../third_party/mpv");
        println!("cargo:rustc-link-search=native={}", dir.display());
        copiar_dll(&dir.join("libmpv-2.dll"));
    }
    println!("cargo:rerun-if-changed=build.rs");
}

fn copiar_dll(origen: &std::path::Path) {
    let Some(perfil) = std::env::var_os("OUT_DIR")
        .map(std::path::PathBuf::from)
        .and_then(|out| {
            out.parent()?
                .parent()?
                .parent()
                .map(std::path::PathBuf::from)
        })
    else {
        return;
    };
    for destino in [perfil.clone(), perfil.join("examples"), perfil.join("deps")] {
        if std::fs::create_dir_all(&destino).is_ok() {
            let _ = std::fs::copy(origen, destino.join("libmpv-2.dll"));
        }
    }
}
