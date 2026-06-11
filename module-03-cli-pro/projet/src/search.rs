//! Le cœur du mini-grep : la recherche d'un motif dans un texte.
//!
//! À vous de jouer : c'est ici que se passent les étapes 1 à 3 du PROJET.md.
//! Objectif de style : un pipeline d'itérateurs (`lines()` → `enumerate()` →
//! `filter()` → `map()` → `collect()`), pas de boucle `for` avec `push`.

use crate::error::GrepError;

/// Les options qui modifient le comportement de la recherche.
///
/// `Default` donne `false` partout : par défaut, la recherche est sensible
/// à la casse et sélectionne les lignes qui contiennent le motif.
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    /// Ignorer la casse (`-i` / `--ignore-case`).
    pub ignore_case: bool,
    /// Inverser la sélection : garder les lignes qui ne contiennent PAS
    /// le motif (`-v` / `--invert`).
    pub invert: bool,
}

/// Une ligne retenue par la recherche.
///
/// On stocke le numéro de ligne (à partir de 1, comme dans tous les éditeurs)
/// et une copie du texte. `PartialEq`/`Eq` servent aux assertions des tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchedLine {
    /// Numéro de la ligne dans le fichier, en partant de 1.
    pub number: usize,
    /// Contenu de la ligne (sans le `\n` final).
    pub text: String,
}

/// Recherche `pattern` dans `content`, ligne par ligne.
///
/// Retourne la liste des lignes retenues, dans l'ordre du texte.
///
/// # Erreurs
///
/// Retourne [`GrepError::EmptyPattern`] si le motif est vide.
pub fn search(
    pattern: &str,
    content: &str,
    options: &SearchOptions,
) -> Result<Vec<MatchedLine>, GrepError> {
    // TODO — étape 1 : recherche de base (sensible à la casse), motif vide
    //                  → Err(GrepError::EmptyPattern).
    // TODO — étape 2 : option `ignore_case` (indice : `to_lowercase`).
    // TODO — étape 3 : option `invert` (indice : un seul `!=` suffit).
    //
    // La ligne suivante évite les avertissements « unused variable » tant que
    // la fonction n'est pas écrite : supprimez-la quand vous commencez.
    let _ = (pattern, content, options);
    todo!("voir PROJET.md, étapes 1 à 3")
}

// ---------------------------------------------------------------------------
// Tests unitaires FOURNIS : dans le même fichier que le code testé, compilés
// uniquement avec `cargo test` grâce à #[cfg(test)]. Ne les modifiez pas —
// faites-les passer, étape par étape (voir les points de contrôle).
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Petit texte de travail, emprunté au Rust Book.
    const CONTENT: &str = "safe, fast, productive.\nPick three.\nDuct tape.";

    #[test]
    fn finds_a_simple_match() {
        let results = search("duct", CONTENT, &SearchOptions::default()).unwrap();
        assert_eq!(
            results,
            vec![MatchedLine {
                number: 1,
                text: String::from("safe, fast, productive."),
            }]
        );
    }

    #[test]
    fn is_case_sensitive_by_default() {
        // "Duct" (majuscule) ne doit matcher que la ligne 3, pas "productive".
        let results = search("Duct", CONTENT, &SearchOptions::default()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].number, 3);
    }

    #[test]
    fn ignore_case_finds_more_lines() {
        let options = SearchOptions {
            ignore_case: true,
            ..SearchOptions::default()
        };
        let results = search("rUsT", "Rust:\nTrust me.", &options).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn invert_selects_non_matching_lines() {
        let options = SearchOptions {
            invert: true,
            ..SearchOptions::default()
        };
        let results = search("safe", CONTENT, &options).unwrap();
        let numbers: Vec<usize> = results.iter().map(|m| m.number).collect();
        assert_eq!(numbers, vec![2, 3]);
    }

    #[test]
    fn line_numbers_start_at_one() {
        let results = search("three", CONTENT, &SearchOptions::default()).unwrap();
        assert_eq!(results[0].number, 2);
        assert_eq!(results[0].text, "Pick three.");
    }

    #[test]
    fn empty_pattern_is_an_error() {
        let error = search("", CONTENT, &SearchOptions::default()).unwrap_err();
        assert!(matches!(error, GrepError::EmptyPattern));
    }

    #[test]
    fn no_match_returns_an_empty_vec() {
        let results = search("zebra", CONTENT, &SearchOptions::default()).unwrap();
        assert!(results.is_empty());
    }
}
