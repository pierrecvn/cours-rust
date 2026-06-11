//! Jeu du nombre mystère — partie 1 du projet du module 1.
//!
//! Le programme tire un nombre secret entre 1 et 100, puis demande au joueur
//! de le deviner. À chaque proposition, il répond « trop petit » ou
//! « trop grand », jusqu'à la victoire.
//!
//! Suis les étapes 1 à 7 de PROJET.md : tu remplaceras le `todo!()` ci-dessous
//! par ton propre code, petit à petit.

// `#![allow(unused)]` évite les avertissements « import inutilisé » tant que le
// squelette n'est pas terminé. Retire cette ligne à la fin du projet : les
// avertissements du compilateur redeviendront tes alliés.
#![allow(unused)]

// Les trois imports dont tu auras besoin (étapes 2 à 5) :
use rand::Rng; // le trait qui fournit `random_range`
use std::cmp::Ordering; // l'enum retourné par la comparaison `cmp`
use std::io; // l'entrée/sortie standard, pour lire le clavier

fn main() {
    println!("=== Le nombre mystère ===");
    println!("J'ai choisi un nombre entre 1 et 100. À toi de le deviner !");

    // TODO (étape 2) : génère le nombre secret.
    //   let secret: u32 = rand::rng().random_range(...);
    //   Indice : un range inclusif s'écrit `1..=100`.

    // TODO (étape 6) : déclare un compteur d'essais mutable, puis ouvre une
    //   boucle `loop { ... }` qui englobe les étapes 3 à 5.

    // TODO (étape 3) : lis la proposition du joueur.
    //   - crée une `String` vide et mutable ;
    //   - `io::stdin().read_line(&mut ...)` la remplit (pourquoi `&mut` ?
    //     relis la section ownership du cours !) ;
    //   - `read_line` retourne un `Result` : déballe-le avec `.expect(...)`.

    // TODO (étape 4) : convertis le texte en nombre.
    //   `input.trim().parse()` retourne un `Result<u32, _>` : utilise un
    //   `match` avec les bras `Ok(n)` et `Err(_)`.
    //   En cas d'erreur (étape 7) : affiche un message et `continue;`.

    // TODO (étape 5) : compare la proposition au secret.
    //   `guess.cmp(&secret)` retourne un `Ordering` : fais un `match` sur ses
    //   trois variantes (`Less`, `Greater`, `Equal`). Sur `Equal`, affiche le
    //   nombre d'essais et `break;`.

    todo!("suis les étapes 2 à 7 de PROJET.md");
}
