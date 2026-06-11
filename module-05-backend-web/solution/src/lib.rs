//! API REST de gestion de notes — corrigé du module 5.
//!
//! La bibliothèque expose tout ce qu'il faut pour construire l'application :
//! - `db` : connexion à SQLite et création du schéma ;
//! - `models` : les types `Note` et `NoteInput` ;
//! - `error` : le type d'erreur du domaine, converti en réponse HTTP ;
//! - `handlers` : un handler par route ;
//! - `app` : la fonction qui assemble le routeur.
//!
//! Séparer `lib.rs` et `main.rs` permet aux tests d'intégration de construire
//! l'application sans lancer de vrai serveur (voir `tests/api.rs`).

pub mod db;
pub mod error;
pub mod handlers;
pub mod models;

use axum::routing::get;
use axum::Router;
use sqlx::SqlitePool;

/// Assemble le routeur complet de l'application.
///
/// Le pool de connexions est injecté en paramètre : en production `main` lui
/// donne un fichier `notes.db`, et les tests une base en mémoire. C'est la
/// même idée que l'injection de dépendances de Spring, en plus explicite.
pub fn app(pool: SqlitePool) -> Router {
    Router::new()
        // `get(...).post(...)` enchaîne plusieurs méthodes HTTP sur le même chemin,
        // comme `router.route("/notes").get(...).post(...)` en Express.
        .route(
            "/notes",
            get(handlers::list_notes).post(handlers::create_note),
        )
        // Axum 0.8 note les paramètres de chemin entre accolades : `{id}`.
        .route(
            "/notes/{id}",
            get(handlers::get_note)
                .put(handlers::update_note)
                .delete(handlers::delete_note),
        )
        // `with_state` rend le pool accessible à tous les handlers via
        // l'extracteur `State`. Le pool est déjà un `Arc` en interne :
        // le cloner est bon marché, pas besoin de l'envelopper nous-mêmes.
        .with_state(pool)
}
