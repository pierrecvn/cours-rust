//! Le type d'erreur du domaine et sa traduction en réponse HTTP. (Étape 5)

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// Toutes les façons dont une requête peut échouer dans notre domaine.
///
/// L'enum est fourni ; à vous d'écrire l'`impl IntoResponse` plus bas.
///
/// Le `#[from]` sur la variante `Database` génère
/// `impl From<sqlx::Error> for ApiError` : dans les handlers, l'opérateur `?`
/// convertira automatiquement les erreurs SQLx en `ApiError::Database`.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// La note demandée n'existe pas → 404.
    #[error("note {0} not found")]
    NotFound(i64),

    /// Toute erreur SQLx imprévue → 500.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// La pièce maîtresse : comment une `ApiError` devient une réponse HTTP.
///
/// C'est l'équivalent d'un `@ExceptionHandler` Spring ou d'un middleware
/// d'erreur Express, mais vérifié par le compilateur.
///
/// Indices :
/// 1. choisissez le `StatusCode` avec un `match` sur `&self`
///    (`NotFound` → 404, `Database` → 500) ;
/// 2. construisez un corps JSON, par exemple
///    `Json(json!({ "error": self.to_string() }))` ;
/// 3. un tuple `(StatusCode, Json<...>)` implémente déjà `IntoResponse` :
///    renvoyez `(status, body).into_response()`.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        todo!("Étape 5 : mapper chaque variante vers un statut + corps JSON")
    }
}
