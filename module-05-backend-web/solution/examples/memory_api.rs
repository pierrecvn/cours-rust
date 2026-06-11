//! Étape 2 — API de notes EN MÉMOIRE (corrigé).
//!
//! Objectif pédagogique : pratiquer les extracteurs (`State`, `Json`) et le
//! state partagé avec les outils du module 4 (`Arc<Mutex<...>>`), avant
//! d'introduire SQLx. Les notes disparaissent à l'arrêt du serveur.
//!
//! Lancer : `cargo run --example memory_api`
//! Tester :
//!   curl http://localhost:3000/notes
//!   curl -X POST http://localhost:3000/notes \
//!        -H 'Content-Type: application/json' \
//!        -d '{"title":"Essai","content":"En mémoire"}'

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
/// toutes les requêtes par un Arc. Exactement le motif du module 4 —
/// plusieurs tâches Tokio peuvent toucher le Vec en même temps, le Mutex
/// sérialise les accès.
type Db = Arc<Mutex<Vec<Note>>>;

/// GET /notes → la liste complète.
async fn list_notes(State(db): State<Db>) -> Json<Vec<Note>> {
    // On clone les données POUR SORTIR du verrou immédiatement : il ne faut
    // jamais garder un MutexGuard de `std` à travers un `.await`.
    let notes = db.lock().expect("mutex poisoned").clone();
    Json(notes)
}

/// POST /notes → 201 + la note créée.
async fn create_note(State(db): State<Db>, Json(input): Json<NoteInput>) -> (StatusCode, Json<Note>) {
    let mut notes = db.lock().expect("mutex poisoned");

    // Id naïf : le prochain entier. Suffisant pour cette étape jetable ;
    // SQLite fera mieux avec AUTOINCREMENT.
    let note = Note {
        id: notes.len() as i64 + 1,
        title: input.title,
        content: input.content,
    };
    notes.push(note.clone());

    (StatusCode::CREATED, Json(note))
}

#[tokio::main]
async fn main() {
    let db: Db = Arc::new(Mutex::new(Vec::new()));

    let app = Router::new()
        .route("/notes", get(list_notes).post(create_note))
        // `with_state` exige un type `Clone` : Arc se clone à bas coût
        // (un compteur incrémenté), les données ne sont pas dupliquées.
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind port 3000");

    println!("In-memory notes API on http://localhost:3000");

    axum::serve(listener, app).await.expect("server crashed");
}
