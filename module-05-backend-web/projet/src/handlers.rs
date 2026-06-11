//! Un handler par route — le cœur du projet. (Étapes 4 et 5)
//!
//! Les signatures sont fournies : elles montrent le motif central d'Axum,
//! des **extracteurs** en paramètres (`State`, `Path`, `Json`) et un
//! `Result<_, ApiError>` en retour. À vous d'écrire les corps.
//!
//! Rappel SQLx (liaison à l'exécution, sans macro) :
//! - `sqlx::query_as::<_, Note>("SELECT ... WHERE id = ?").bind(id)` ;
//! - `.fetch_all(&pool)` → `Vec<Note>` ;
//! - `.fetch_one(&pool)` → `Note` (erreur si aucune ligne) ;
//! - `.fetch_optional(&pool)` → `Option<Note>` ;
//! - `sqlx::query("...").execute(&pool)` → infos d'exécution
//!   (dont `.rows_affected()`).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use sqlx::SqlitePool;

use crate::error::ApiError;
use crate::models::{Note, NoteInput};

/// `GET /notes` → 200 + liste de toutes les notes (triées par id).
///
/// Indice : `query_as` + `fetch_all`, puis enveloppez le Vec dans `Json`.
/// Le `?` suffit pour les erreurs : `sqlx::Error` se convertit tout seul
/// en `ApiError` grâce au `#[from]`.
pub async fn list_notes(State(pool): State<SqlitePool>) -> Result<Json<Vec<Note>>, ApiError> {
    todo!("Étape 4 : SELECT id, title, content, created_at FROM notes")
}

/// `POST /notes` → 201 + la note créée.
///
/// Indices :
/// - `INSERT INTO notes (title, content) VALUES (?, ?)
///    RETURNING id, title, content, created_at` : le `RETURNING` renvoie la
///   ligne insérée (id auto-incrémenté et created_at calculé par la base)
///   en une seule requête — combinez-le avec `query_as` + `fetch_one` ;
/// - renvoyez `(StatusCode::CREATED, Json(note))` pour produire un 201.
pub async fn create_note(
    State(pool): State<SqlitePool>,
    // `Json<NoteInput>` désérialise le corps. JSON invalide → 4xx automatique.
    Json(input): Json<NoteInput>,
) -> Result<(StatusCode, Json<Note>), ApiError> {
    todo!("Étape 4 : INSERT ... RETURNING ...")
}

/// `GET /notes/{id}` → 200 + la note, ou 404.
///
/// Indice : `fetch_optional` renvoie `Option<Note>` ;
/// `.ok_or(ApiError::NotFound(id))?` transforme le `None` en 404.
pub async fn get_note(
    State(pool): State<SqlitePool>,
    // `Path<i64>` extrait `{id}` de l'URL. `/notes/abc` → 400 automatique.
    Path(id): Path<i64>,
) -> Result<Json<Note>, ApiError> {
    todo!("Étape 5 : SELECT ... WHERE id = ?, puis None → 404")
}

/// `PUT /notes/{id}` → 200 + la note mise à jour, ou 404.
///
/// Indice : `UPDATE notes SET title = ?, content = ? WHERE id = ?
/// RETURNING ...` + `fetch_optional` : si l'id n'existe pas, aucune ligne
/// ne revient → 404, même motif que `get_note`.
pub async fn update_note(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    // L'extracteur qui consomme le corps (`Json`) doit être le DERNIER.
    Json(input): Json<NoteInput>,
) -> Result<Json<Note>, ApiError> {
    todo!("Étape 5 : UPDATE ... RETURNING ..., puis None → 404")
}

/// `DELETE /notes/{id}` → 204 sans corps, ou 404.
///
/// Indice : `sqlx::query("DELETE ...").execute(&pool)` puis vérifiez
/// `result.rows_affected()` : 0 ligne supprimée → 404, sinon
/// `StatusCode::NO_CONTENT`.
pub async fn delete_note(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    todo!("Étape 5 : DELETE, puis rows_affected() == 0 → 404")
}
