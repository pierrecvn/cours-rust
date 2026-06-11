//! API REST de gestion de notes — squelette du module 5.
//!
//! Votre mission : remplacer tous les `todo!()` pour faire passer les tests
//! de `tests/api.rs`. Suivez les étapes de PROJET.md.
//!
//! Pourquoi une `lib.rs` ET un `main.rs` ? Pour que les tests d'intégration
//! puissent construire l'application (`notes_api::app(...)`) sans lancer de
//! vrai serveur. `main.rs` ne fait que le branchement final.

// TODO (fin du projet) : retirez cette ligne et corrigez les avertissements.
#![allow(unused_variables, unused_imports)]

pub mod db;
pub mod error;
pub mod handlers;
pub mod models;

use axum::routing::get;
use axum::Router;
use sqlx::SqlitePool;

/// Assemble le routeur complet de l'application.
///
/// Le pool est injecté en paramètre : `main` lui donnera un fichier
/// `notes.db`, les tests une base en mémoire (injection de dépendances).
///
/// Routes à déclarer (le contrat d'API du cours) :
/// - `GET  /notes`        → `handlers::list_notes`
/// - `POST /notes`        → `handlers::create_note`
/// - `GET    /notes/{id}` → `handlers::get_note`
/// - `PUT    /notes/{id}` → `handlers::update_note`
/// - `DELETE /notes/{id}` → `handlers::delete_note`
///
/// Indices :
/// - `Router::new().route("/notes", get(...).post(...))` enchaîne plusieurs
///   méthodes HTTP sur un même chemin, comme en Express ;
/// - en Axum 0.8, un paramètre de chemin s'écrit `{id}` (accolades) ;
/// - terminez par `.with_state(pool)` pour donner le pool aux handlers.
pub fn app(pool: SqlitePool) -> Router {
    todo!("Étape 4 : construire le Router et lui donner le pool en state")
}
