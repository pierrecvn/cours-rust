//! Le cœur du mini-grep : la recherche d'un motif dans un texte.
//!
//! Tout est écrit en pipeline d'itérateurs : `lines()` → `enumerate()` →
//! `filter()` → `map()` → `collect()`. C'est le style idiomatique en Rust,
//! et le compilateur le transforme en code aussi rapide qu'une boucle `for`
//! écrite à la main (abstraction à coût nul, voir COURS.md §4.5).

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
///
/// # Exemple
///
/// ```
/// use minigrep::{search, SearchOptions};
///
/// let results = search("three", "one\ntwo\nthree", &SearchOptions::default()).unwrap();
/// assert_eq!(results.len(), 1);
/// assert_eq!(results[0].number, 3);
/// ```
pub fn search(
    pattern: &str,
    content: &str,
    options: &SearchOptions,
) -> Result<Vec<MatchedLine>, GrepError> {
    if pattern.is_empty() {
        return Err(GrepError::EmptyPattern);
    }

    // En mode insensible à la casse, on normalise le motif UNE seule fois,
    // avant la boucle — pas à chaque ligne.
    let needle = if options.ignore_case {
        pattern.to_lowercase()
    } else {
        pattern.to_string()
    };

    let results = content
        .lines() // itérateur sur les &str de chaque ligne (paresseux !)
        .enumerate() // ajoute l'index : (0, "ligne 1"), (1, "ligne 2"), ...
        .filter(|(_, line)| {
            let is_match = if options.ignore_case {
                line.to_lowercase().contains(&needle)
            } else {
                line.contains(needle.as_str())
            };
            // `!=` joue le rôle d'un XOR sur les booléens :
            //   invert == false → on garde les lignes qui matchent ;
            //   invert == true  → on garde celles qui ne matchent pas.
            is_match != options.invert
        })
        .map(|(index, line)| MatchedLine {
            number: index + 1, // les humains comptent à partir de 1
            text: line.to_string(),
        })
        .collect(); // matérialise le pipeline en Vec<MatchedLine>

    Ok(results)
}

// ---------------------------------------------------------------------------
// Tests unitaires : dans le même fichier que le code testé, compilés
// uniquement avec `cargo test` grâce à #[cfg(test)].
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
