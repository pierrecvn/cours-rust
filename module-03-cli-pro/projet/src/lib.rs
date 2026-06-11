//! # minigrep — bibliothèque
//!
//! Toute la logique métier du mini-grep vit ici, dans la **bibliothèque**
//! (`src/lib.rs`). Le binaire (`src/main.rs`) ne fait que parser la ligne de
//! commande, appeler ces fonctions et afficher les résultats.
//!
//! Cette séparation lib + bin est la clé de la testabilité : les tests
//! unitaires et d'intégration s'appuient sur la bibliothèque, sans jamais
//! avoir besoin de lancer le binaire. (Voir COURS.md §9.3.)

pub mod config;
pub mod error;
pub mod search;

// Ré-exports : les utilisateurs de la crate (dont notre propre binaire)
// écrivent `minigrep::search(...)` au lieu de `minigrep::search::search(...)`.
pub use config::FileConfig;
pub use error::GrepError;
pub use search::{MatchedLine, SearchOptions, search};
