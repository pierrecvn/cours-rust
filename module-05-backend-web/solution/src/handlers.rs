//! Un handler par route. C'est l'équivalent des méthodes d'un
//! `@RestController` Spring ou des callbacks `app.get(...)` d'Express.
//!
//! Remarquez le motif commun : les paramètres sont des **extracteurs**
//! (`State`, `Path`, `Json`) et le retour est un `Result<_, ApiError>`.
//! Axum se charge de tout le reste — désérialisation, sérialisation,
//! statuts HTTP — à partir des types.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use sqlx::SqlitePool;

use crate::error::ApiError;
use crate::models::{Note, NoteInput};

/// `GET /notes` → 200 + liste de toutes les notes.
pub async fn list_notes(State(pool): State<SqlitePool>) -> Result<Json<Vec<Note>>, ApiError> {
    // `query_as::<_, Note>` mappe chaque ligne vers une `Note` grâce au
    // derive `FromRow`. `fetch_all` rassemble toutes les lignes dans un Vec.
    // Le `?` final convertit une éventuelle `sqlx::Error` en `ApiError`
    // (variante `Database`, donc réponse 500) via le `#[from]` de l'enum.
    let notes = sqlx::query_as::<_, Note>("SELECT id, title, content, created_at FROM notes ORDER BY id")
        .fetch_all(&pool)
        .await?;

    Ok(Json(notes))
}

/// `POST /notes` → 201 + la note créée.
pub async fn create_note(
    State(pool): State<SqlitePool>,
    // `Json<NoteInput>` désérialise le corps de la requête. Si le JSON est
    // invalide ou qu'un champ manque, Axum répond 4xx tout seul : le handler
    // n'est même pas appelé. Comparable à `@Valid @RequestBody` en Spring.
    Json(input): Json<NoteInput>,
) -> Result<(StatusCode, Json<Note>), ApiError> {
    // `RETURNING` (SQLite ≥ 3.35) renvoie la ligne insérée en une seule
    // requête : on récupère ainsi l'`id` auto-incrémenté et le `created_at`
    // calculé par la base, sans second SELECT.
    let note = sqlx::query_as::<_, Note>(
        "INSERT INTO notes (title, content) VALUES (?, ?)
         RETURNING id, title, content, created_at",
    )
    // `bind` lie les valeurs aux `?` à l'exécution — paramètres préparés,
    // donc aucune injection SQL possible, comme un PreparedStatement JDBC.
    .bind(&input.title)
    .bind(&input.content)
    .fetch_one(&pool)
    .await?;

    // Renvoyer un tuple `(StatusCode, Json<...>)` est la façon idiomatique
    // de choisir le statut : ici 201 Created au lieu du 200 par défaut.
    Ok((StatusCode::CREATED, Json(note)))
}

/// `GET /notes/{id}` → 200 + la note, ou 404.
pub async fn get_note(
    State(pool): State<SqlitePool>,
    // `Path<i64>` extrait `{id}` de l'URL et le parse en i64.
    // `/notes/abc` produit un 400 automatique, sans toucher au handler.
    Path(id): Path<i64>,
) -> Result<Json<Note>, ApiError> {
    // `fetch_optional` renvoie `Option<Note>` : `None` si aucune ligne.
    // `ok_or` transforme ce `None` en notre erreur 404 — tout le mapping
    // « absent → 404 » tient en une ligne.
    let note = sqlx::query_as::<_, Note>(
        "SELECT id, title, content, created_at FROM notes WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or(ApiError::NotFound(id))?;

    Ok(Json(note))
}

/// `PUT /notes/{id}` → 200 + la note mise à jour, ou 404.
pub async fn update_note(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    // Attention à l'ordre des extracteurs : celui qui consomme le corps de
    // la requête (`Json`) doit toujours être le DERNIER paramètre.
    Json(input): Json<NoteInput>,
) -> Result<Json<Note>, ApiError> {
    // Même astuce que pour l'insertion : `UPDATE ... RETURNING` met à jour
    // et relit en une requête. Si l'id n'existe pas, aucune ligne ne revient
    // → `fetch_optional` donne `None` → 404.
    let note = sqlx::query_as::<_, Note>(
        "UPDATE notes SET title = ?, content = ? WHERE id = ?
         RETURNING id, title, content, created_at",
    )
    .bind(&input.title)
    .bind(&input.content)
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or(ApiError::NotFound(id))?;

    Ok(Json(note))
}

/// `DELETE /notes/{id}` → 204 sans corps, ou 404.
pub async fn delete_note(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;

    // Un DELETE qui ne supprime rien n'est pas une erreur SQL : il faut
    // vérifier `rows_affected` soi-même pour distinguer 204 de 404.
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound(id));
    }

    Ok(StatusCode::NO_CONTENT)
}
