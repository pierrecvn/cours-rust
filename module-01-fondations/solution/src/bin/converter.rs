//! Convertisseur d'unités — corrigé commenté du module 1.
//!
//! Lance-le avec `cargo run --bin converter`,
//! teste-le avec `cargo test --bin converter`.

use std::io;

/// Nombre exact de kilomètres dans un mile (définition internationale).
/// `const` : valeur fixée à la compilation, type obligatoire, nom en
/// SCREAMING_SNAKE_CASE — l'équivalent du `static final` de Java.
const KM_PER_MILE: f64 = 1.609344;

/// Convertit des degrés Celsius en degrés Fahrenheit.
///
/// Fonction PURE : pas d'entrée clavier, pas d'affichage, juste un calcul.
/// C'est ce qui la rend triviale à tester et à réutiliser.
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    // Le corps est une unique expression, sans `;` : sa valeur est la valeur
    // de retour de la fonction. Pas besoin de `return`.
    celsius * 9.0 / 5.0 + 32.0
}

/// Convertit des degrés Fahrenheit en degrés Celsius.
fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    // `5.0 / 9.0` et non `5 / 9` : entre entiers, `/` est une division
    // entière (5 / 9 == 0) et Rust ne mélange jamais entiers et flottants
    // implicitement.
    (fahrenheit - 32.0) * 5.0 / 9.0
}

/// Convertit des kilomètres en miles.
fn kilometers_to_miles(kilometers: f64) -> f64 {
    kilometers / KM_PER_MILE
}

/// Convertit des miles en kilomètres.
fn miles_to_kilometers(miles: f64) -> f64 {
    miles * KM_PER_MILE
}

/// Lit une ligne sur l'entrée standard.
/// Retourne `None` si l'entrée est terminée (Ctrl-D), `Some(ligne)` sinon.
/// La `String` retournée est DONNÉE à l'appelant : la propriété sort de la
/// fonction avec la valeur de retour.
fn read_line() -> Option<String> {
    let mut input = String::new();
    let bytes_read = io::stdin()
        .read_line(&mut input)
        .expect("échec de la lecture de l'entrée standard");
    if bytes_read == 0 {
        None
    } else {
        // `trim()` retourne un `&str` emprunté à `input` ; `to_string()` en
        // fait une nouvelle `String` indépendante qu'on peut retourner.
        Some(input.trim().to_string())
    }
}

fn main() {
    println!("=== Convertisseur d'unités ===");

    loop {
        println!();
        println!("1. Celsius     → Fahrenheit");
        println!("2. Fahrenheit  → Celsius");
        println!("3. Kilomètres  → Miles");
        println!("4. Miles       → Kilomètres");
        println!("q. Quitter");
        println!("Ton choix :");

        // `let ... else` : si `read_line()` retourne `None` (fin de l'entrée),
        // la branche `else` doit quitter (ici : sortir de la boucle).
        let Some(choice) = read_line() else {
            break;
        };

        if choice == "q" {
            break;
        }

        // `as_str()` : emprunte la `String` en `&str` pour pouvoir la
        // comparer à des littéraux dans le `match`.
        let is_valid_choice = matches!(choice.as_str(), "1" | "2" | "3" | "4");
        if !is_valid_choice {
            println!("Choix inconnu : « {choice} ».");
            continue;
        }

        println!("Valeur à convertir :");
        let Some(raw_value) = read_line() else {
            break;
        };

        // Même motif que dans le jeu : `parse()` retourne un `Result`,
        // le `match` le déballe ou relance la boucle.
        let value: f64 = match raw_value.parse() {
            Ok(number) => number,
            Err(_) => {
                println!("« {raw_value} » n'est pas un nombre valide.");
                continue;
            }
        };

        // `match` est une expression : chaque bras produit la ligne à
        // afficher. `:.2` arrondit l'affichage à deux décimales.
        let output = match choice.as_str() {
            "1" => format!("{value} °C = {:.2} °F", celsius_to_fahrenheit(value)),
            "2" => format!("{value} °F = {:.2} °C", fahrenheit_to_celsius(value)),
            "3" => format!("{value} km = {:.2} mi", kilometers_to_miles(value)),
            "4" => format!("{value} mi = {:.2} km", miles_to_kilometers(value)),
            // Inatteignable (choix déjà validé), mais `match` doit être
            // exhaustif : `unreachable!` documente cette impossibilité.
            _ => unreachable!("choix déjà validé plus haut"),
        };
        println!("{output}");
    }

    println!("À bientôt !");
}

// ---------------------------------------------------------------------------
// Tests unitaires : les mêmes que ceux fournis dans le squelette.
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
