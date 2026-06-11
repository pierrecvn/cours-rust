//! Point d'entrée : ouvre la base, construit l'application, sert sur :3000.
//! (Fourni complet — lisez-le : tout le branchement final est ici.)

use anyhow::Context;

// `#[tokio::main]` démarre le runtime Tokio puis exécute ce `async fn main`
// dessus. Sans runtime, un `async fn` ne s'exécute jamais (future paresseuse).
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Ouvrir la base (votre étape 3). `mode=rwc` = lecture/écriture +
    // création du fichier si absent. Le schéma est créé par `init_db`.
    let pool = notes_api::db::init_db("sqlite:notes.db?mode=rwc")
        .await
        .context("failed to open the database")?;

    // Construire le routeur (votre étape 4, fonction `app` de lib.rs).
    let app = notes_api::app(pool);

    // Écouter sur le port 3000 (le contrat d'API du cours) et servir
    // l'application — l'équivalent de `app.listen(3000)` en Express.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .context("failed to bind port 3000")?;

    println!("Notes API listening on http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
