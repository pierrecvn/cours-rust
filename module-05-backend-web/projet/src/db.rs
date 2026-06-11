//! Connexion à SQLite et initialisation du schéma. (Étape 3)

use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

/// Ouvre (ou crée) la base et renvoie un pool de connexions prêt à l'emploi.
///
/// `url` est une URL SQLx, par exemple `sqlite:notes.db`.
///
/// Indices :
/// 1. `SqliteConnectOptions::from_str(url)?` parse l'URL ;
/// 2. `.create_if_missing(true)` crée le fichier au premier lancement
///    (sans cela, SQLx refuse d'ouvrir une base inexistante) ;
/// 3. `SqlitePoolOptions::new().max_connections(5).connect_with(options)`
///    construit le pool (n'oubliez pas le `.await?`) ;
/// 4. appelez `create_schema(&pool)` avant de renvoyer le pool.
pub async fn init_db(url: &str) -> Result<SqlitePool, sqlx::Error> {
    todo!("Étape 3 : ouvrir la base et créer le schéma")
}

/// Crée la table `notes` si elle n'existe pas encore.
///
/// Utilisez `sqlx::query("CREATE TABLE IF NOT EXISTS ...").execute(pool)`.
/// Le `IF NOT EXISTS` rend l'opération idempotente : on peut relancer le
/// serveur autant de fois qu'on veut.
///
/// Colonnes attendues :
/// - `id` : `INTEGER PRIMARY KEY AUTOINCREMENT` ;
/// - `title`, `content` : `TEXT NOT NULL` ;
/// - `created_at` : `TEXT NOT NULL` avec une valeur par défaut ISO 8601
///   calculée par SQLite :
///   `DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))`.
pub async fn create_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    todo!("Étape 3 : CREATE TABLE IF NOT EXISTS notes (...)")
}
