fn main() {
    // SAFETY: mpv_client_api_version no recibe punteros ni toca estado global
    // mutable; solo devuelve una constante de la biblioteca.
    let version = unsafe { libmpv2_sys::mpv_client_api_version() };

    println!("mpv_client_api_version = 0x{version:04X}");
}
