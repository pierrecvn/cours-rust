//! Client HTTP vers l'API REST de notes du module 5.
//!
//! Contrat consommé (base : http://localhost:3000) :
//! - GET    /notes        → 200, liste de notes
//! - POST   /notes        → 201, note créée
//! - PUT    /notes/{id}   → 200 ou 404
//! - DELETE /notes/{id}   → 204 ou 404
//!
//! Ce module ne connaît pas Tauri : c'est du Rust « pur » (reqwest + serde),
//! testable et réutilisable tel quel dans un autre programme.

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
/// être créé une fois et partagé (il est `Clone` et thread-safe).
/// C'est exactement ce que le state managé de Tauri va faire.
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
    pub async fn list_notes(&self) -> Result<Vec<Note>, ApiError> {
        let response = self
            .http
            .get(format!("{}/notes", self.base_url))
            .send()
            .await
            .map_err(ApiError::from_reqwest)?;
        Self::decode(response, StatusCode::OK).await
    }

    /// POST /notes → la note créée (statut 201).
    pub async fn create_note(&self, input: &NoteInput) -> Result<Note, ApiError> {
        let response = self
            .http
            .post(format!("{}/notes", self.base_url))
            .json(input) // sérialise NoteInput en JSON + en-tête Content-Type
            .send()
            .await
            .map_err(ApiError::from_reqwest)?;
        Self::decode(response, StatusCode::CREATED).await
    }

    /// PUT /notes/{id} → la note mise à jour, ou NotFound.
    pub async fn update_note(&self, id: i64, input: &NoteInput) -> Result<Note, ApiError> {
        let response = self
            .http
            .put(format!("{}/notes/{id}", self.base_url))
            .json(input)
            .send()
            .await
            .map_err(ApiError::from_reqwest)?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(ApiError::NotFound(id));
        }
        Self::decode(response, StatusCode::OK).await
    }

    /// DELETE /notes/{id} → rien (204), ou NotFound.
    pub async fn delete_note(&self, id: i64) -> Result<(), ApiError> {
        let response = self
            .http
            .delete(format!("{}/notes/{id}", self.base_url))
            .send()
            .await
            .map_err(ApiError::from_reqwest)?;
        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::NOT_FOUND => Err(ApiError::NotFound(id)),
            other => Err(ApiError::Unexpected(format!("statut {other}"))),
        }
    }

    /// Vérifie le statut puis désérialise le corps JSON vers `T`.
    /// Générique : sert pour `Note` comme pour `Vec<Note>`.
    async fn decode<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
        expected: StatusCode,
    ) -> Result<T, ApiError> {
        let status = response.status();
        if status != expected {
            return Err(ApiError::Unexpected(format!(
                "statut {status} (attendu : {expected})"
            )));
        }
        response
            .json::<T>()
            .await
            .map_err(|err| ApiError::Unexpected(format!("corps JSON invalide : {err}")))
    }
}
