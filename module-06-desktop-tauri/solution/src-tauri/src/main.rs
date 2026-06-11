// Sous Windows, empêche l'ouverture d'une console à côté de la fenêtre
// en mode release. Sans effet sur Linux et macOS.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Toute la logique vit dans la bibliothèque (src/lib.rs) :
    // le binaire ne fait que l'appeler. C'est la structure standard
    // d'un projet Tauri 2 (elle permet aussi les cibles mobiles).
    notes_desktop_lib::run()
}
