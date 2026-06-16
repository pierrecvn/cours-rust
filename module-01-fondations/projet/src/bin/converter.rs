//! Convertisseur d'unités — partie 2 du projet du module 1.
//!
//! Quatre fonctions de conversion **pures** (mêmes entrées → mêmes sorties,
//! aucun effet de bord), puis un petit menu interactif qui les utilise.
//!
//! Les tests en bas de ce fichier sont déjà écrits : ils échouent tant que les
//! fonctions contiennent `todo!()`. Lance-les avec :
//!     cargo test --bin converter
//!
//! Suis les étapes 8 à 10 de PROJET.md.

// À retirer une fois le projet terminé (voir src/main.rs).
// #![allow(unused)]

use std::io;

/// Convertit des degrés Celsius en degrés Fahrenheit.
///
/// Formule : F = C × 9/5 + 32
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    // TODO (étape 8) : implémente la formule.
    //   Rappel du cours : le corps d'une fonction peut être une simple
    //   expression finale, SANS `;` ni `return`.
    celsius * 9.0 / 5.0 + 32.0
}

/// Convertit des degrés Fahrenheit en degrés Celsius.
///
/// Formule : C = (F − 32) × 5/9
fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    // TODO (étape 8) : implémente la formule.
    //   Attention : écris `5.0 / 9.0`, pas `5 / 9` (division entière = 0 !).
    (fahrenheit - 32.0) * 5.0 / 9.0
}

/// Convertit des kilomètres en miles.
///
/// 1 mile = 1,609344 km exactement.
fn kilometers_to_miles(kilometers: f64) -> f64 {
    // TODO (étape 8) : divise par la constante KM_PER_MILE déclarée plus bas.

    kilometers / KM_PER_MILE
}

/// Convertit des miles en kilomètres.
fn miles_to_kilometers(miles: f64) -> f64 {
    // TODO (étape 8) : multiplie par KM_PER_MILE.
    miles * KM_PER_MILE
}

/// Nombre exact de kilomètres dans un mile (définition internationale).
const KM_PER_MILE: f64 = 1.609344;

fn read_line() -> Option<String> {
    let mut input = String::new();
    let bytes_read: usize = io::stdin()
        .read_line(&mut input)
        .expect("echec de la lecture");

    if bytes_read == 0 {
        None
    } else {
        Some(input.trim().to_string())
    }
}

fn main() {
    println!("=== Convertisseur d'unités ===");

    // TODO (étape 9) : boucle de menu.
    //   1) Affiche les choix :
    //        1. Celsius → Fahrenheit
    //        2. Fahrenheit → Celsius
    //        3. Kilomètres → Miles
    //        4. Miles → Kilomètres
    //        q. Quitter
    //   2) Lis le choix avec `read_line` ; si c'est "q", `break;`.
    //   3) Lis la valeur à convertir, `parse()` en `f64` (un `match`, comme
    //      dans le jeu — `continue;` si l'entrée est invalide).
    //   4) Appelle la bonne fonction et affiche le résultat, par exemple :
    //        println!("{value} °C = {result:.2} °F");
    //      (`:.2` arrondit l'affichage à 2 décimales.)

    loop {
        println!();
        println!("1. Celsius     → Fahrenheit");
        println!("2. Fahrenheit  → Celsius");
        println!("3. Kilomètres  → Miles");
        println!("4. Miles       → Kilomètres");
        println!("q. Quitter");
        println!("Ton choix :");

        let Some(choice) = read_line() else {
            break;
        };

        if choice == "q" {
            break;
        };

        let good_choice: bool = matches!(choice.as_str(), "1" | "2" | "3" | "4");
        if !good_choice {
            println!("Choix inconnu {choice}");
            continue;
        }

        println!("Valeur à convertir : ");
        let Some(value_orignial) = read_line() else {
            break;
        };

        let value: f64 = match value_orignial.parse() {
            Ok(number) => number,
            Err(_) => {
                println!("{value_orignial} n'est pas un nombre valide !");
                continue;
            }
        };

        let output: String = match choice.as_str() {
            "1" => format!("{value} °C = {:.2} °F", celsius_to_fahrenheit(value)),
            "2" => format!("{value} °F = {:.2} °C", fahrenheit_to_celsius(value)),
            "3" => format!("{value} km = {:.2} mi", kilometers_to_miles(value)),
            "4" => format!("{value} mi = {:.2} km", miles_to_kilometers(value)),

            _ => unreachable!("choix deja validé plus haut"),
        };
        println!("{output}");
    }
    println!("Salut !")

    // todo!("suis l'étape 9 de PROJET.md");
}

// ---------------------------------------------------------------------------
// Tests fournis — NE PAS MODIFIER.
// `#[cfg(test)]` : ce module n'est compilé que par `cargo test`.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Compare deux flottants avec une petite tolérance : on ne compare
    /// JAMAIS des `f64` avec `==` (erreurs d'arrondi).
    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "attendu {expected}, obtenu {actual}"
        );
    }

    #[test]
    fn freezing_point_in_fahrenheit() {
        assert_close(celsius_to_fahrenheit(0.0), 32.0);
    }

    #[test]
    fn boiling_point_in_fahrenheit() {
        assert_close(celsius_to_fahrenheit(100.0), 212.0);
    }

    #[test]
    fn negative_celsius_to_fahrenheit() {
        // -40 est le point où les deux échelles se croisent.
        assert_close(celsius_to_fahrenheit(-40.0), -40.0);
    }

    #[test]
    fn body_temperature_in_celsius() {
        assert_close(fahrenheit_to_celsius(98.6), 37.0);
    }

    #[test]
    fn fahrenheit_to_celsius_freezing() {
        assert_close(fahrenheit_to_celsius(32.0), 0.0);
    }

    #[test]
    fn round_trip_temperature() {
        // Convertir dans un sens puis dans l'autre doit redonner la valeur.
        assert_close(fahrenheit_to_celsius(celsius_to_fahrenheit(21.5)), 21.5);
    }

    #[test]
    fn one_mile_in_kilometers() {
        assert_close(miles_to_kilometers(1.0), 1.609344);
    }

    #[test]
    fn marathon_in_miles() {
        assert_close(kilometers_to_miles(42.195), 42.195 / 1.609344);
    }

    #[test]
    fn round_trip_distance() {
        assert_close(kilometers_to_miles(miles_to_kilometers(26.2)), 26.2);
    }
}
