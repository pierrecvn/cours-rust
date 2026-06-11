//! Les types du domaine : ce que l'API reçoit et renvoie.

use serde::{Deserialize, Serialize};

/// Une note telle qu'elle existe en base et telle que l'API la renvoie.
///
/// Trois derives travaillent ensemble :
/// - `Serialize` (serde) : la note peut devenir du JSON dans les réponses ;
/// - `FromRow` (sqlx) : la note peut être construite depuis une ligne SQL,
///   colonne par colonne, en faisant correspondre les noms de champs ;
/// - `Deserialize` sert aux tests pour relire le JSON renvoyé par l'API.
///
/// Équivalent Java : une entité JPA annotée `@Entity` + un DTO Jackson,
/// fusionnés en un seul type car notre domaine est simple.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub content: String,
    /// Date de création au format ISO 8601 (ex. `2026-06-11T14:30:00Z`).
    /// On la stocke et la transporte comme une `String` : SQLite n'a pas de
    /// vrai type date, et le contrat d'API demande une chaîne ISO 8601.
    pub created_at: String,
}

/// Le corps attendu par `POST /notes` et `PUT /notes/{id}`.
///
/// Pas d'`id` ni de `created_at` : c'est le serveur qui les attribue.
/// Avoir un type distinct pour l'entrée évite qu'un client n'envoie un `id`
/// forgé — le système de types fait office de validation structurelle.
#[derive(Debug, Deserialize)]
pub struct NoteInput {
    pub title: String,
    pub content: String,
}
