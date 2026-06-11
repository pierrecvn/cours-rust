//! Type d'erreur des commandes Tauri.
//!
//! Pour traverser le pont Rust → TypeScript, une erreur doit être
//! sérialisable : Tauri convertit le `Err(...)` d'une commande en
//! rejet de la `Promise` côté frontend. On implémente donc `Serialize`
//! à la main pour envoyer un message lisible (une simple chaîne).

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Le serveur ne répond pas : connexion refusée ou délai dépassé.
    /// C'est le cas « l'API du module 5 n'est pas démarrée ».
    /// Le message est destiné à l'utilisateur : il sera affiché tel quel
    /// dans le bandeau d'erreur du frontend.
    #[error(
        "Impossible de joindre l'API ({0}). Le serveur du module 5 est-il démarré ? \
         Lancez `cargo run` dans le projet du module 5, puis réessayez."
    )]
    Unreachable(String),

    /// L'API a répondu 404 pour cet identifiant.
    #[error("note introuvable (id {0})")]
    NotFound(i64),

    /// Tout le reste : statut inattendu, JSON invalide…
    #[error("réponse inattendue de l'API : {0}")]
    Unexpected(String),
}

impl ApiError {
    /// Classe une erreur reqwest : panne de connexion ou autre problème.
    pub fn from_reqwest(err: reqwest::Error) -> Self {
        if err.is_connect() || err.is_timeout() {
            ApiError::Unreachable(err.to_string())
        } else {
            ApiError::Unexpected(err.to_string())
        }
    }
}

// Sérialisation vers une chaîne : côté TS, `catch (err)` reçoit
// directement le message produit par `#[error("...")]` (via Display).
impl Serialize for ApiError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
