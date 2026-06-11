//! Jeu du nombre mystère — corrigé commenté du module 1.
//!
//! Lance-le avec `cargo run`.

use rand::Rng; // trait : il doit être importé pour que `random_range` existe
use std::cmp::Ordering;
use std::io;

fn main() {
    println!("=== Le nombre mystère ===");
    println!("J'ai choisi un nombre entre 1 et 100. À toi de le deviner !");

    // `rand::rng()` : générateur aléatoire local au thread.
    // `1..=100` : range INCLUSIF (1 et 100 sont possibles).
    // L'annotation `u32` guide l'inférence : sans elle, le compilateur ne
    // saurait pas quel type d'entier produire.
    let secret: u32 = rand::rng().random_range(1..=100);

    // `mut` car le compteur change à chaque tour — sans `mut`, la ligne
    // `attempts += 1` ne compilerait pas (en Rust, l'immuabilité est le défaut).
    let mut attempts: u32 = 0;

    // `loop` : boucle infinie assumée ; on en sort explicitement avec `break`.
    loop {
        println!("Ta proposition :");

        // Une `String` vide, possédée par nous, que `read_line` va remplir.
        let mut input = String::new();

        // `&mut input` : on PRÊTE la chaîne en écriture à `read_line`, qui y
        // ajoute la ligne lue. On en reste propriétaire après l'appel.
        // `read_line` retourne `Result<usize, _>` : le nombre d'octets lus.
        let bytes_read = io::stdin()
            .read_line(&mut input)
            .expect("échec de la lecture de l'entrée standard");

        // 0 octet lu = fin de l'entrée (Ctrl-D, ou fin d'un fichier redirigé).
        // Sans ce test, un Ctrl-D ferait boucler le programme à l'infini.
        if bytes_read == 0 {
            println!("\nFin de l'entrée, à bientôt !");
            return;
        }

        // `trim()` retire le '\n' final (et les espaces) ; `parse()` tente la
        // conversion en `u32` (type déduit de l'annotation sur `guess`).
        // `match` est une EXPRESSION : sa valeur est affectée à `guess`.
        let guess: u32 = match input.trim().parse() {
            Ok(number) => number,
            Err(_) => {
                println!("« {} » n'est pas un nombre entre 1 et 100, réessaie.", input.trim());
                continue; // entrée invalide : on repart en début de boucle
            }
        };

        attempts += 1;

        // `cmp` compare deux valeurs et retourne un `Ordering` ; il prend une
        // référence (`&secret`) : pas besoin de céder la propriété pour lire.
        // Le `match` doit couvrir les TROIS variantes — le compilateur y veille.
        match guess.cmp(&secret) {
            Ordering::Less => println!("Trop petit !"),
            Ordering::Greater => println!("Trop grand !"),
            Ordering::Equal => {
                println!("Gagné en {attempts} essai(s) ! Le nombre était bien {secret}.");
                break; // sortie de la boucle : la partie est finie
            }
        }
    }
}
