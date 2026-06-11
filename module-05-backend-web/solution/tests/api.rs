//! Tests d'intégration : ils vérifient le CONTRAT D'API du cours.
//! Le module 6 (app desktop Tauri) consommera exactement cette API —
//! si ces tests passent, le module 6 fonctionnera.
//!
//! Technique : `tower::ServiceExt::oneshot` envoie une requête directement
//! au `Router`, sans ouvrir de port ni lancer de serveur. C'est l'équivalent
//! de `supertest` en Node ou de `MockMvc` en Spring : rapide et isolé.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

/// Construit une application branchée sur une base SQLite EN MÉMOIRE.
///
/// `max_connections(1)` est important : avec `sqlite::memory:`, chaque
/// connexion ouvrirait sa PROPRE base vide. Une seule connexion garantit
/// que tous les tests d'une même application voient les mêmes données.
async fn test_app() -> Router {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("failed to open in-memory database");

    notes_api::db::create_schema(&pool)
        .await
        .expect("failed to create schema");

    notes_api::app(pool)
}

/// Envoie une requête à l'application et renvoie (statut, corps JSON).
/// Le corps est `Value::Null` si la réponse est vide (cas du 204).
async fn send(app: &Router, request: Request<Body>) -> (StatusCode, Value) {
    // `oneshot` consomme le service : on travaille sur un clone du Router
    // (un Router se clone à très bas coût, comme tout ce qui est partagé).
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request failed");

    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("failed to read body")
        .to_bytes();

    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("body is not valid JSON")
    };

    (status, body)
}

/// Petit raccourci pour fabriquer une requête avec un corps JSON.
fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("failed to build request")
}

/// Requête sans corps (GET, DELETE).
fn empty_request(method: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .expect("failed to build request")
}

/// Crée une note via l'API et renvoie son JSON. Utilitaire des autres tests.
async fn create_note(app: &Router, title: &str, content: &str) -> Value {
    let (status, body) = send(
        app,
        json_request("POST", "/notes", json!({ "title": title, "content": content })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "POST /notes doit répondre 201");
    body
}

// ---------------------------------------------------------------------------
// GET /notes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_notes_empty_returns_200_and_empty_array() {
    let app = test_app().await;

    let (status, body) = send(&app, empty_request("GET", "/notes")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!([]));
}

#[tokio::test]
async fn list_notes_returns_created_notes_in_order() {
    let app = test_app().await;
    create_note(&app, "Première", "contenu 1").await;
    create_note(&app, "Deuxième", "contenu 2").await;

    let (status, body) = send(&app, empty_request("GET", "/notes")).await;

    assert_eq!(status, StatusCode::OK);
    let notes = body.as_array().expect("la réponse doit être un tableau");
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0]["title"], "Première");
    assert_eq!(notes[1]["title"], "Deuxième");
}

// ---------------------------------------------------------------------------
// POST /notes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_note_returns_201_and_full_note() {
    let app = test_app().await;

    let note = create_note(&app, "Courses", "Acheter du pain").await;

    assert!(note["id"].is_i64(), "id doit être un entier");
    assert_eq!(note["title"], "Courses");
    assert_eq!(note["content"], "Acheter du pain");

    // `created_at` doit être une chaîne ISO 8601 (au minimum : non vide,
    // avec le 'T' séparant date et heure).
    let created_at = note["created_at"]
        .as_str()
        .expect("created_at doit être une chaîne");
    assert!(
        created_at.contains('T'),
        "created_at doit être au format ISO 8601, reçu : {created_at}"
    );
}

#[tokio::test]
async fn create_note_with_invalid_body_returns_client_error() {
    let app = test_app().await;

    // Corps qui n'est pas du JSON valide.
    let request = Request::builder()
        .method("POST")
        .uri("/notes")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("ceci n'est pas du JSON"))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert!(
        response.status().is_client_error(),
        "un corps invalide doit produire une erreur 4xx, reçu : {}",
        response.status()
    );
}

// ---------------------------------------------------------------------------
// GET /notes/{id}
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_note_returns_200_with_the_note() {
    let app = test_app().await;
    let created = create_note(&app, "Idée", "Apprendre Axum").await;
    let id = created["id"].as_i64().unwrap();

    let (status, body) = send(&app, empty_request("GET", &format!("/notes/{id}"))).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], id);
    assert_eq!(body["title"], "Idée");
    assert_eq!(body["content"], "Apprendre Axum");
}

#[tokio::test]
async fn get_unknown_note_returns_404() {
    let app = test_app().await;

    let (status, _body) = send(&app, empty_request("GET", "/notes/9999")).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// PUT /notes/{id}
// ---------------------------------------------------------------------------

#[tokio::test]
async fn update_note_returns_200_with_updated_note() {
    let app = test_app().await;
    let created = create_note(&app, "Brouillon", "v1").await;
    let id = created["id"].as_i64().unwrap();

    let (status, body) = send(
        &app,
        json_request(
            "PUT",
            &format!("/notes/{id}"),
            json!({ "title": "Final", "content": "v2" }),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], id, "l'id ne doit pas changer");
    assert_eq!(body["title"], "Final");
    assert_eq!(body["content"], "v2");

    // La mise à jour doit être persistée : on relit la note.
    let (_, reread) = send(&app, empty_request("GET", &format!("/notes/{id}"))).await;
    assert_eq!(reread["title"], "Final");
}

#[tokio::test]
async fn update_unknown_note_returns_404() {
    let app = test_app().await;

    let (status, _body) = send(
        &app,
        json_request("PUT", "/notes/9999", json!({ "title": "x", "content": "y" })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// DELETE /notes/{id}
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_note_returns_204_then_note_is_gone() {
    let app = test_app().await;
    let created = create_note(&app, "À supprimer", "...").await;
    let id = created["id"].as_i64().unwrap();

    let (status, body) = send(&app, empty_request("DELETE", &format!("/notes/{id}"))).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(body, Value::Null, "un 204 ne doit pas avoir de corps");

    // La note ne doit plus exister.
    let (status, _) = send(&app, empty_request("GET", &format!("/notes/{id}"))).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_unknown_note_returns_404() {
    let app = test_app().await;

    let (status, _body) = send(&app, empty_request("DELETE", "/notes/9999")).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
