//! # minigrep — binaire
//!
//! Couche « présentation » du projet : parser la ligne de commande (clap),
//! charger l'éventuel fichier de configuration, appeler la bibliothèque,
//! afficher les résultats. Aucune logique de recherche ici.
//!
//! C'est le chantier des étapes 5 à 7 du PROJET.md :
//!   - étape 5 : la struct `Cli` avec `#[derive(Parser)]` (clap) — motif,
//!     fichiers, drapeaux `-i`, `-n`, `-v`, `-c` et option `--config` ;
//!   - étape 6 : lecture des fichiers, appel de `minigrep::search`,
//!     affichage des résultats (avec `anyhow::Context` pour les erreurs) ;
//!   - étape 7 : chargement du fichier de configuration et fusion avec les
//!     drapeaux de la ligne de commande.
//!
//! Côté erreurs : le binaire utilise `anyhow` (COURS.md §8). La signature
//! `fn main() -> anyhow::Result<()>` est déjà en place : un `Err` qui remonte
//! jusqu'ici est affiché proprement et le programme sort avec le code 1.

use anyhow::Result;

fn main() -> Result<()> {
    todo!("voir PROJET.md, étapes 5 à 7")
}
