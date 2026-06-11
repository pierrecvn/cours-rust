//! Les erreurs de la bibliothèque, définies avec `thiserror`.
//!
//! ⚠️ Ce fichier est FOURNI : rien à compléter ici. Lisez-le attentivement,
//! il est votre premier contact concret avec `thiserror` (COURS.md §8) et
//! l'étape 3 du PROJET.md vous demandera d'expliquer ce qu'il fait.
//!
//! Règle du module 3 : une **bibliothèque** expose des erreurs typées et
//! précises (`thiserror`), un **binaire** peut se contenter d'`anyhow`.

use thiserror::Error;

/// Toutes les erreurs que la bibliothèque peut produire.
///
/// `#[derive(Error)]` génère l'implémentation du trait `std::error::Error` ;
/// l'attribut `#[error("...")]` fournit le message du trait `Display`.
#[derive(Debug, Error)]
pub enum GrepError {
    /// Chercher une chaîne vide n'a pas de sens : chaque ligne « contiendrait »
    /// le motif. On préfère le signaler clairement à l'utilisateur.
    #[error("the search pattern is empty")]
    EmptyPattern,

    /// Le fichier de configuration existe mais son TOML est invalide.
    /// `#[from]` génère `impl From<toml::de::Error> for GrepError`, ce qui
    /// permet d'utiliser l'opérateur `?` directement sur `toml::from_str`.
    #[error("invalid configuration file: {0}")]
    InvalidConfig(#[from] toml::de::Error),

    /// Erreur d'entrée/sortie (lecture d'un fichier, etc.).
    /// Là encore, `#[from]` rend `?` utilisable sur les fonctions de `std::fs`.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
