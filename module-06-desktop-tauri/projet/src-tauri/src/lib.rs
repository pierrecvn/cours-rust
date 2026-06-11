//! Côté Rust de l'application : les commandes exposées au frontend.
//!
//! Chaque fonction marquée `#[tauri::command]` devient appelable depuis
//! TypeScript via `invoke("nom_de_la_commande", { ...args })`.
//! Les arguments arrivent en JSON et sont désérialisés par serde ;
//! la valeur de retour repart en JSON dans l'autre sens.

// TODO : retirez cette ligne à la fin du module (elle masque les
// avertissements sur le code pas encore utilisé du squelette).
#![allow(unused_variables, unused_imports, dead_code)]

pub mod api;
pub mod error;

use api::{ApiClient, Note, NoteInput};
use error::ApiError;
use tauri::State;

// TODO (étape 2) : écrivez ici la commande `greet`.
//
// Indices :
// - une fonction `fn greet(name: String) -> String` qui renvoie un salut ;
// - décorée de `#[tauri::command]` ;
// - enregistrée plus bas dans `generate_handler![...]`.

// TODO (étape 4) : la commande `list_notes`.
//
// Signature indicative :
//   #[tauri::command]
//   async fn list_notes(client: State<'_, ApiClient>) -> Result<Vec<Note>, ApiError>
//
// Le corps tient en une ligne : déléguer à `client.list_notes().await`.
// Attention : pour une commande async qui renvoie un Result avec un State,
// le lifetime anonyme `'_` est obligatoire dans `State<'_, ApiClient>`.

// TODO (étape 5) : la commande `create_note(client, title, content)`.

// TODO (étape 6) : les commandes `update_note(client, id, title, content)`
// et `delete_note(client, id)`.

/// Point d'entrée de l'application, appelé par `main.rs`.
/// (L'attribut `mobile_entry_point` ne sert que pour les cibles iOS/Android ;
/// il est inactif sur desktop mais fait partie du squelette standard.)
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // TODO (étape 3) : enregistrer le state managé —
        //   .manage(ApiClient::new("http://localhost:3000"))
        //
        // TODO (étape 2, puis 4 à 6) : lister vos commandes dans
        // generate_handler![...] au fur et à mesure que vous les écrivez.
        // Une commande non listée ici est invisible pour `invoke`.
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de l'application Tauri");
}
