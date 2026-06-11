fn main() {
    // Compile la configuration Tauri (tauri.conf.json, capabilities, icônes)
    // dans le binaire. C'est ce qui alimente `tauri::generate_context!()`.
    tauri_build::build()
}
