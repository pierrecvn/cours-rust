//! Étape 1 — Le plus petit serveur Axum possible (corrigé).
//!
//! Lancer : `cargo run --example hello`
//! Tester : `curl http://localhost:3000/`

use axum::routing::get;
use axum::Router;

/// Un handler est une simple fonction async. Ici, pas d'extracteur :
/// elle ne lit rien dans la requête. Son retour (`&'static str`)
/// implémente `IntoResponse`, donc Axum sait en faire un 200 texte.
async fn hello() -> &'static str {
    "Hello from Axum!"
}

#[tokio::main]
async fn main() {
    // Une seule route : GET / → hello.
    let app = Router::new().route("/", get(hello));

    // On réserve le port 3000, puis on sert l'application.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind port 3000");

    println!("Hello server on http://localhost:3000");

    axum::serve(listener, app)
        .await
        .expect("server crashed");
}
