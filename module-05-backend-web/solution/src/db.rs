//! Connexion à SQLite et initialisation du schéma.

use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

/// Ouvre (ou crée) la base et renvoie un pool de connexions prêt à l'emploi.
///
/// `url` est une URL SQLx, par exemple `sqlite:notes.db`.
/// `create_if_missing(true)` crée le fichier au premier lancement — sans cela,
/// SQLx refuse d'ouvrir une base inexistante.
pub async fn init_db(url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(url)?.create_if_missing(true);

    // Un pool de 5 connexions suffit largement pour SQLite : c'est une base
    // embarquée, un seul écrivain à la fois. L'important est le principe :
    // on réutilise les connexions au lieu d'en ouvrir une par requête HTTP.
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    create_schema(&pool).await?;
    Ok(pool)
}

/// Crée la table `notes` si elle n'existe pas encore.
///
/// `CREATE TABLE IF NOT EXISTS` rend l'opération idempotente : on peut
/// relancer le serveur autant de fois qu'on veut. C'est la version artisanale
/// des migrations (Flyway/Liquibase en Java, `sqlx migrate` en Rust).
///
/// `created_at` reçoit une valeur par défaut calculée par SQLite elle-même :
/// `strftime` produit directement le format ISO 8601 exigé par le contrat.
pub async fn create_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS notes (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            title       TEXT NOT NULL,
            content     TEXT NOT NULL,
            created_at  TEXT NOT NULL
                        DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}
