//! Le type d'erreur du domaine et sa traduction en réponse HTTP.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// Toutes les façons dont une requête peut échouer dans notre domaine.
///
/// C'est le pendant HTTP de ce que vous avez fait avec `thiserror` au
/// module 3 : un enum qui énumère les cas d'échec, avec un message pour
/// chacun. La nouveauté est l'implémentation d'`IntoResponse` plus bas,
/// qui associe un statut HTTP à chaque variante.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// La note demandée n'existe pas → 404.
    #[error("note {0} not found")]
    NotFound(i64),

    /// Toute erreur SQLx imprévue → 500.
    ///
    /// `#[from]` génère `impl From<sqlx::Error> for ApiError`, ce qui permet
    /// d'utiliser l'opérateur `?` sur les appels SQLx dans les handlers :
    /// l'erreur est convertie automatiquement, comme une exception qui
    /// remonte en Java — mais visible dans la signature.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// La pièce maîtresse : comment une `ApiError` devient une réponse HTTP.
///
/// Grâce à ce `impl`, un handler peut renvoyer `Result<Json<Note>, ApiError>`
/// et Axum sait quoi faire dans les deux cas. C'est l'équivalent d'un
/// `@ControllerAdvice` + `@ExceptionHandler` de Spring, ou d'un middleware
/// d'erreur Express (`app.use((err, req, res, next) => ...)`), mais vérifié
/// par le compilateur : impossible d'oublier de gérer un cas.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        // En production, on logguerait le détail côté serveur et on ne
        // renverrait au client qu'un message générique pour les erreurs 500.
        // Ici, le message aide à déboguer pendant l'apprentissage.
        let body = Json(json!({ "error": self.to_string() }));

        (status, body).into_response()
    }
}
