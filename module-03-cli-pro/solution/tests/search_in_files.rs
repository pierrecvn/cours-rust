//! Tests d'intégration : ce fichier vit dans `tests/`, il est compilé comme
//! une crate À PART qui utilise `minigrep` exactement comme le ferait un
//! projet tiers — uniquement via son API publique (`use minigrep::...`).
//!
//! Les fichiers d'exemple sont dans `tests/fixtures/`. Cargo lance toujours
//! les tests d'intégration depuis la racine de la crate, les chemins relatifs
//! ci-dessous sont donc fiables.

use std::fs;
use std::path::Path;

use minigrep::{FileConfig, SearchOptions, search};

/// Lit le poème d'exemple (Emily Dickinson, domaine public).
fn poem() -> String {
    fs::read_to_string("tests/fixtures/poem.txt").expect("tests/fixtures/poem.txt is missing")
}

#[test]
fn finds_nobody_twice_in_the_poem() {
    let results = search("nobody", &poem(), &SearchOptions::default()).unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].number, 1);
    assert_eq!(results[0].text, "I'm nobody! Who are you?");
    assert_eq!(results[1].number, 2);
}

#[test]
fn ignore_case_matches_capitalized_words() {
    let options = SearchOptions {
        ignore_case: true,
        ..SearchOptions::default()
    };
    // « WHO » ne matche que grâce à l'insensibilité à la casse (« Who »).
    let results = search("WHO", &poem(), &options).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].number, 1);
}

#[test]
fn case_sensitive_search_misses_capitalized_words() {
    // Sans -i, « who » (minuscule) ne trouve rien : le poème écrit « Who ».
    let results = search("who", &poem(), &SearchOptions::default()).unwrap();
    assert!(results.is_empty());
}

#[test]
fn invert_keeps_only_lines_without_the_pattern() {
    let options = SearchOptions {
        invert: true,
        ..SearchOptions::default()
    };
    // Toutes les lignes du poème contiennent un « o »... sauf la ligne vide.
    let results = search("o", &poem(), &options).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].number, 5);
    assert_eq!(results[0].text, "");
}

#[test]
fn count_is_just_the_length_of_the_results() {
    let results = search("us", &poem(), &SearchOptions::default()).unwrap();
    // « a pair of us » (ligne 3) et « banish us » (ligne 4).
    assert_eq!(results.len(), 2);
}

#[test]
fn loads_the_config_fixture() {
    let config = FileConfig::load_if_exists(Path::new("tests/fixtures/config.toml"))
        .unwrap()
        .expect("the fixture file exists");

    assert_eq!(
        config,
        FileConfig {
            ignore_case: true,
            line_numbers: true,
        }
    );
}

#[test]
fn missing_config_file_is_not_an_error() {
    let config = FileConfig::load_if_exists(Path::new("tests/fixtures/does-not-exist.toml"));
    assert!(matches!(config, Ok(None)));
}
