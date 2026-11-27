fn main() {
    if cfg!(windows) {
        // `env!` se resuelve al compilar el build script, así que un build
        // desde otro directorio (un worktree, por ejemplo) deja la ruta vieja
        // en la caché y el linker no encuentra `mpv.lib`. Leer la variable en
        // runtime siempre apunta al checkout que se está compilando.
        let raiz = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
        let dir = std::path::Path::new(&raiz).join("../../third_party/mpv");
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
