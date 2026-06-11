//! Client HTTP vers l'API REST de notes du module 5.
//!
//! Contrat consommé (base : http://localhost:3000) :
//! - GET    /notes        → 200, liste de notes
//! - POST   /notes        → 201, note créée
//! - PUT    /notes/{id}   → 200 ou 404
//! - DELETE /notes/{id}   → 204 ou 404
//!
//! Ce module ne connaît pas Tauri : c'est du Rust « pur » (reqwest + serde).
//! C'est vous qui implémentez les quatre méthodes (étapes 3 à 6 du PROJET).

use std::time::Duration;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

/// Une note, telle que l'API la renvoie.
/// Miroir exact de l'interface `Note` côté TypeScript : c'est serde qui
/// garantit la correspondance champ à champ à travers le pont.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub content: String,
    /// Date ISO 8601 ; on la garde en `String`, le frontend la formatera.
    pub created_at: String,
}

/// Corps envoyé à POST /notes et PUT /notes/{id}.
#[derive(Debug, Serialize)]
pub struct NoteInput {
    pub title: String,
    pub content: String,
}

/// Le client : une URL de base et un `reqwest::Client` réutilisé.
/// `reqwest::Client` contient un pool de connexions ; il est conçu pour
/// être créé une fois et partagé. C'est exactement ce que le state managé
/// de Tauri fera (étape 3).
pub struct ApiClient {
    http: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            // Sans timeout, une API injoignable peut bloquer ~30 s :
            // 5 s suffisent largement pour du localhost.
            .timeout(Duration::from_secs(5))
            .build()
            .expect("configuration du client HTTP invalide");
        Self {
            http,
            base_url: base_url.into(),
        }
    }

    /// GET /notes → Vec<Note>
    ///
    /// Indices :
    /// - `self.http.get(format!("{}/notes", self.base_url)).send().await`
    /// - convertir l'erreur réseau avec `.map_err(ApiError::from_reqwest)?`
    /// - vérifier que le statut est `StatusCode::OK`
    /// - désérialiser le corps avec `.json::<Vec<Note>>().await`
    pub async fn list_notes(&self) -> Result<Vec<Note>, ApiError> {
        todo!("étape 3 : appeler GET /notes et désérialiser la réponse")
    }

    /// POST /notes → la note créée (statut attendu : 201 CREATED).
    ///
    /// Indice : `.post(...).json(input).send().await` — `.json(...)`
    /// sérialise `NoteInput` et pose l'en-tête Content-Type.
    pub async fn create_note(&self, input: &NoteInput) -> Result<Note, ApiError> {
        todo!("étape 5 : appeler POST /notes avec le corps JSON")
    }

    /// PUT /notes/{id} → la note mise à jour (200), ou `ApiError::NotFound`
    /// si l'API répond 404.
    pub async fn update_note(&self, id: i64, input: &NoteInput) -> Result<Note, ApiError> {
        todo!("étape 6 : appeler PUT /notes/{{id}} et gérer le 404")
    }

    /// DELETE /notes/{id} → `Ok(())` si 204, `ApiError::NotFound` si 404,
    /// `ApiError::Unexpected` pour tout autre statut.
    pub async fn delete_note(&self, id: i64) -> Result<(), ApiError> {
        todo!("étape 6 : appeler DELETE /notes/{{id}} et examiner le statut")
    }
}
