//! Le fichier de configuration optionnel (TOML), désérialisé avec serde.
//!
//! C'est l'étape 4 du PROJET.md. L'utilisateur peut poser un fichier
//! `.minigrep.toml` à côté de son projet pour définir des valeurs par défaut :
//!
//! ```toml
//! ignore_case = true
//! line_numbers = true
//! ```
//!
//! Les drapeaux passés en ligne de commande s'ajoutent ensuite à ces valeurs.

use std::path::Path;

use crate::error::GrepError;

/// Les valeurs par défaut configurables par fichier.
///
/// TODO — étape 4 : pour que serde puisse construire cette struct depuis du
/// TOML, il manque ici un derive et deux attributs `#[serde(...)]` :
///   - un fichier partiel (ou vide) doit être valide, les champs absents
///     prenant leur valeur `Default` ;
///   - une clé inconnue (faute de frappe) doit être une erreur.
/// Indices dans PROJET.md, théorie dans COURS.md §6.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileConfig {
    /// Recherche insensible à la casse par défaut.
    pub ignore_case: bool,
    /// Affichage des numéros de ligne par défaut.
    pub line_numbers: bool,
}

impl FileConfig {
    /// Désérialise une configuration depuis une chaîne TOML.
    ///
    /// Séparer le parsing (`parse`) de la lecture du fichier (`load_if_exists`)
    /// rend cette logique testable sans toucher au disque.
    ///
    /// # Erreurs
    ///
    /// [`GrepError::InvalidConfig`] si le TOML est invalide ou contient une
    /// clé inconnue.
    pub fn parse(input: &str) -> Result<Self, GrepError> {
        // TODO — étape 4 : une seule ligne suffit (indice : `toml::from_str`
        // et l'opérateur `?` — la conversion d'erreur est déjà gérée par le
        // `#[from]` de `error.rs`).
        let _ = input;
        todo!("voir PROJET.md, étape 4")
    }

    /// Charge la configuration depuis `path`, si le fichier existe.
    ///
    /// - Le fichier n'existe pas → `Ok(None)` : la configuration est
    ///   optionnelle, ce n'est pas une erreur.
    /// - Le fichier existe mais est illisible ou invalide → `Err(...)` :
    ///   l'utilisateur a écrit quelque chose, il faut le prévenir.
    pub fn load_if_exists(path: &Path) -> Result<Option<Self>, GrepError> {
        // TODO — étape 4 : `path.exists()`, `std::fs::read_to_string`, puis
        // `Self::parse`. Pensez aux `?` (les conversions sont automatiques).
        let _ = path;
        todo!("voir PROJET.md, étape 4")
    }
}

// ---------------------------------------------------------------------------
// Tests unitaires FOURNIS — ne les modifiez pas, faites-les passer.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_full_config() {
        let config = FileConfig::parse("ignore_case = true\nline_numbers = false").unwrap();
        assert_eq!(
            config,
            FileConfig {
                ignore_case: true,
                line_numbers: false,
            }
        );
    }

    #[test]
    fn missing_fields_fall_back_to_default() {
        let config = FileConfig::parse("line_numbers = true").unwrap();
        assert_eq!(
            config,
            FileConfig {
                ignore_case: false,
                line_numbers: true,
            }
        );
    }

    #[test]
    fn empty_input_gives_the_default_config() {
        let config = FileConfig::parse("").unwrap();
        assert_eq!(config, FileConfig::default());
    }

    #[test]
    fn invalid_toml_is_an_error() {
        let error = FileConfig::parse("ignore_case = ").unwrap_err();
        assert!(matches!(error, GrepError::InvalidConfig(_)));
    }

    #[test]
    fn unknown_key_is_an_error() {
        let error = FileConfig::parse("colour = true").unwrap_err();
        assert!(matches!(error, GrepError::InvalidConfig(_)));
    }
}
