//! Étape 1 — Le plus petit serveur Axum possible.
//!
//! Complétez les deux `todo!()`, puis :
//!   lancer : `cargo run --example hello`
//!   tester : `curl http://localhost:3000/`  → doit afficher votre message

use axum::routing::get;
use axum::Router;

/// Un handler est une simple fonction async. Ici, pas de paramètre :
/// elle ne lit rien dans la requête. Son retour (`&'static str`)
/// implémente `IntoResponse`, donc Axum sait en faire une réponse 200.
async fn hello() -> &'static str {
    todo!("renvoyez un message de votre choix")
}

#[tokio::main]
async fn main() {
    // TODO : créez un Router avec une seule route : GET / → hello.
    // Indice : `Router::new().route("/", get(hello))`.
    let app: Router = todo!("construire le routeur");

    // Ce qui suit est fourni : réserver le port 3000 puis servir l'app.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind port 3000");

    println!("Hello server on http://localhost:3000");

    axum::serve(listener, app)
        .await
        .expect("server crashed");
}
