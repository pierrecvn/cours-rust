//! Étape 2 — API de notes EN MÉMOIRE.
//!
//! Objectif : pratiquer les extracteurs (`State`, `Json`) et le state partagé
//! avec les outils du module 4 (`Arc<Mutex<...>>`), avant d'introduire SQLx.
//! Les notes disparaissent à l'arrêt du serveur — c'est volontaire.
//!
//! Complétez les `todo!()`, puis :
//!   lancer : `cargo run --example memory_api`
//!   tester :
//!     curl http://localhost:3000/notes
//!     curl -X POST http://localhost:3000/notes \
//!          -H 'Content-Type: application/json' \
//!          -d '{"title":"Essai","content":"En mémoire"}'

#![allow(unused_variables)]

use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

/// Version simplifiée de la note : pas encore de `created_at`,
/// il arrivera avec SQLite qui le calculera pour nous.
#[derive(Debug, Clone, Serialize)]
struct Note {
    id: i64,
    title: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct NoteInput {
    title: String,
    content: String,
}

/// Notre « base de données » : un Vec protégé par un Mutex, partagé entre
/// toutes les requêtes par un Arc — le motif du module 4.
type Db = Arc<Mutex<Vec<Note>>>;

/// GET /notes → la liste complète.
///
/// Indices :
/// - `db.lock().expect("mutex poisoned")` donne accès au Vec ;
/// - clonez les données pour sortir du verrou tout de suite (ne JAMAIS
///   garder un MutexGuard de `std` à travers un `.await`) ;
/// - renvoyez `Json(notes)`.
async fn list_notes(State(db): State<Db>) -> Json<Vec<Note>> {
    todo!("verrouiller, cloner, renvoyer Json(...)")
}

/// POST /notes → 201 + la note créée.
///
/// Indices :
/// - id naïf : `notes.len() as i64 + 1` (SQLite fera mieux à l'étape 3) ;
/// - poussez un clone de la note dans le Vec ;
/// - renvoyez `(StatusCode::CREATED, Json(note))`.
async fn create_note(State(db): State<Db>, Json(input): Json<NoteInput>) -> (StatusCode, Json<Note>) {
    todo!("créer la note, la stocker, répondre 201")
}

#[tokio::main]
async fn main() {
    let db: Db = Arc::new(Mutex::new(Vec::new()));

    // TODO : routez GET et POST /notes vers les deux handlers, puis donnez
    // le state au routeur. Indices : `get(...).post(...)`, `.with_state(db)`.
    let app: Router = todo!("construire le routeur avec son state");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind port 3000");

    println!("In-memory notes API on http://localhost:3000");

    axum::serve(listener, app).await.expect("server crashed");
}
