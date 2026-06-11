//! Point d'entrée : ouvre la base, construit l'application, sert sur :3000.

use anyhow::Context;

// `#[tokio::main]` transforme ce `async fn main` en un `fn main` classique
// qui démarre le runtime Tokio puis exécute notre future dessus. Sans
// runtime, un `async fn` ne s'exécute jamais : les futures sont paresseuses.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // `mode=rwc` : lecture/écriture, création du fichier si absent.
    // Le schéma est créé au démarrage (CREATE TABLE IF NOT EXISTS),
    // donc le serveur fonctionne dès le premier lancement.
    let pool = notes_api::db::init_db("sqlite:notes.db?mode=rwc")
        .await
        .context("failed to open the database")?;

    let app = notes_api::app(pool);

    // On écoute sur toutes les interfaces, port 3000 — le contrat d'API
    // du cours (le module 6 se connectera à http://localhost:3000).
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .context("failed to bind port 3000")?;

    println!("Notes API listening on http://localhost:3000");

    // `axum::serve` accepte les connexions en boucle et confie chaque
    // requête à une tâche Tokio. C'est notre `app.listen(3000)` d'Express.
    axum::serve(listener, app).await?;

    Ok(())
}
