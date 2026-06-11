//! Le fichier de configuration optionnel (TOML), désérialisé avec serde.
//!
//! L'utilisateur peut poser un fichier `.minigrep.toml` à côté de son projet
//! pour définir des valeurs par défaut :
//!
//! ```toml
//! ignore_case = true
//! line_numbers = true
//! ```
//!
//! Les drapeaux passés en ligne de commande s'ajoutent ensuite à ces valeurs.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::GrepError;

/// Les valeurs par défaut configurables par fichier.
///
/// - `Deserialize` (serde) : permet de construire la struct depuis du TOML.
/// - `#[serde(default)]` : chaque champ absent prend sa valeur `Default`
///   (ici `false`), donc un fichier partiel — ou vide — est valide.
/// - `#[serde(deny_unknown_fields)]` : une clé inconnue (faute de frappe,
///   par exemple) est une erreur, plutôt qu'un silence trompeur.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
    /// clé inconnue. La conversion depuis `toml::de::Error` est automatique
    /// grâce au `#[from]` déclaré dans `error.rs` : un simple `?` suffit.
    pub fn parse(input: &str) -> Result<Self, GrepError> {
        Ok(toml::from_str(input)?)
    }

    /// Charge la configuration depuis `path`, si le fichier existe.
    ///
    /// - Le fichier n'existe pas → `Ok(None)` : la configuration est
    ///   optionnelle, ce n'est pas une erreur.
    /// - Le fichier existe mais est illisible ou invalide → `Err(...)` :
    ///   l'utilisateur a écrit quelque chose, il faut le prévenir.
    pub fn load_if_exists(path: &Path) -> Result<Option<Self>, GrepError> {
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(path)?; // io::Error → GrepError::Io via #[from]
        Ok(Some(Self::parse(&raw)?))
    }
}

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
