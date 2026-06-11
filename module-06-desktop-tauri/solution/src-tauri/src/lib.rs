//! Côté Rust de l'application : les commandes exposées au frontend.
//!
//! Chaque fonction marquée `#[tauri::command]` devient appelable depuis
//! TypeScript via `invoke("nom_de_la_commande", { ...args })`.
//! Les arguments arrivent en JSON et sont désérialisés par serde ;
//! la valeur de retour repart en JSON dans l'autre sens.

pub mod api;
pub mod error;

use api::{ApiClient, Note, NoteInput};
use error::ApiError;
use tauri::State;

/// Commande de démonstration : prouve que le pont TS ↔ Rust fonctionne.
/// Synchrone, pas d'erreur possible → simple `String` en retour.
#[tauri::command]
fn greet(name: String) -> String {
    format!("Bonjour, {name} ! Le pont TypeScript ↔ Rust fonctionne.")
}

/// Liste toutes les notes.
///
/// - `async` : Tauri exécute la commande sur son runtime async, la webview
///   n'est jamais bloquée pendant l'appel HTTP.
/// - `State<'_, ApiClient>` : Tauri injecte le client enregistré par
///   `.manage()` plus bas — même principe que le `State` d'Axum au module 5.
/// - `Result<_, ApiError>` : un `Err` devient une Promise rejetée côté TS,
///   avec le message sérialisé de `ApiError`.
#[tauri::command]
async fn list_notes(client: State<'_, ApiClient>) -> Result<Vec<Note>, ApiError> {
    client.list_notes().await
}

/// Crée une note. Les clés `{ title, content }` passées à `invoke` côté TS
/// sont désérialisées vers ces deux paramètres, par nom.
#[tauri::command]
async fn create_note(
    client: State<'_, ApiClient>,
    title: String,
    content: String,
) -> Result<Note, ApiError> {
    client.create_note(&NoteInput { title, content }).await
}

/// Met à jour une note existante (404 de l'API → ApiError::NotFound).
#[tauri::command]
async fn update_note(
    client: State<'_, ApiClient>,
    id: i64,
    title: String,
    content: String,
) -> Result<Note, ApiError> {
    client.update_note(id, &NoteInput { title, content }).await
}

/// Supprime une note. Rien à renvoyer : `Ok(())` devient `undefined`
/// côté TS (la Promise se résout sans valeur).
#[tauri::command]
async fn delete_note(client: State<'_, ApiClient>, id: i64) -> Result<(), ApiError> {
    client.delete_note(id).await
}

/// Point d'entrée de l'application, appelé par `main.rs`.
/// (L'attribut `mobile_entry_point` ne sert que pour les cibles iOS/Android ;
/// il est inactif sur desktop mais fait partie du squelette standard.)
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // State managé : une seule instance d'ApiClient pour toute l'app,
        // injectable dans n'importe quelle commande via State<'_, ApiClient>.
        .manage(ApiClient::new("http://localhost:3000"))
        // Enregistrement des commandes : toute commande absente de cette
        // liste est invisible pour `invoke` (erreur à l'exécution).
        .invoke_handler(tauri::generate_handler![
            greet,
            list_notes,
            create_note,
            update_note,
            delete_note
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de l'application Tauri");
}
