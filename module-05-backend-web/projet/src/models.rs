//! Les types du domaine : ce que l'API reçoit et renvoie.
//!
//! Ces deux types sont fournis complets : ils définissent le contrat d'API
//! (la forme du JSON) et les tests en dépendent. Lisez-les attentivement,
//! les derives font une grande partie du travail du module.

use serde::{Deserialize, Serialize};

/// Une note telle qu'elle existe en base et telle que l'API la renvoie.
///
/// - `Serialize` : la note peut devenir du JSON dans les réponses ;
/// - `sqlx::FromRow` : la note peut être construite depuis une ligne SQL
///   (correspondance par nom de colonne) ;
/// - `Deserialize` sert aux tests pour relire le JSON renvoyé.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub content: String,
    /// Date de création au format ISO 8601 (ex. `2026-06-11T14:30:00Z`),
    /// stockée et transportée comme une simple chaîne.
    pub created_at: String,
}

/// Le corps attendu par `POST /notes` et `PUT /notes/{id}`.
///
/// Pas d'`id` ni de `created_at` : c'est le serveur qui les attribue.
#[derive(Debug, Deserialize)]
pub struct NoteInput {
    pub title: String,
    pub content: String,
}
