# Cours complet de Rust 🦀

Bienvenue ! Ce cours t'emmène de zéro jusqu'à la création d'applications réelles en Rust : outils en ligne de commande, API web, application desktop, WebAssembly et même un jeu 2D.

Il est conçu pour un développeur qui connaît déjà un langage comme **Java** ou **TypeScript** : chaque concept nouveau de Rust est comparé à ce que tu connais déjà.

## Philosophie

**On apprend Rust en codant.** Chaque module est centré sur un projet concret que tu construis toi-même, étape par étape. La théorie est introduite au moment où le projet en a besoin — pas avant. Le compilateur de Rust est exigeant : c'est normal de se battre avec lui au début, et ce cours t'apprend à lire ses erreurs comme un allié plutôt qu'un ennemi.

## Le parcours

| Module | Sujet | Projet |
|--------|-------|--------|
| [01 — Fondations](module-01-fondations/) | Cargo, syntaxe, ownership & borrowing | Jeu du nombre mystère + convertisseur |
| [02 — Structurer son code](module-02-structurer/) | structs, enums, `Option`/`Result`, modules | Gestionnaire de tâches CLI |
| [03 — CLI professionnel](module-03-cli-pro/) | traits, génériques, itérateurs, crates, tests | mini-grep |
| [04 — Concurrence & lifetimes](module-04-concurrence/) | threads, `Arc`/`Mutex`, lifetimes, smart pointers | Scanner de fichiers parallèle |
| [05 — Backend web](module-05-backend-web/) | async, tokio, Axum, SQLx | API REST de notes |
| [06 — App desktop](module-06-desktop-tauri/) | Tauri 2, frontend TS ↔ backend Rust | App desktop de notes |
| [07 — WebAssembly](module-07-wasm/) | wasm-bindgen, interop JS | Filtres d'image dans le navigateur |
| [08 — Jeu 2D](module-08-jeu-bevy/) | Bevy, architecture ECS | Snake |

Les modules 1 à 4 forment le socle du langage : fais-les dans l'ordre. Les modules 5 à 8 sont plus indépendants, mais l'ordre proposé reste le plus progressif (et le module 6 réutilise l'API du module 5).

## Comment travailler un module

1. Lis le **`COURS.md`** du module — c'est la théorie dont le projet a besoin.
2. Ouvre **`PROJET.md`** et suis les étapes dans l'ordre. Tu codes dans le dossier **`projet/`**, qui est déjà initialisé.
3. À chaque point de contrôle, vérifie avec `cargo test` ou `cargo run`.
4. Bloqué ? Relis l'indice de l'étape, puis seulement en dernier recours regarde **`solution/`**.
5. Module terminé ? Attaque les **`DEFIS.md`** pour consolider, puis passe au module suivant.

## Outils

- **Rust** : installé via [rustup](https://rustup.rs) (déjà fait sur cette machine — `rustc 1.96`).
- **Éditeur** : VS Code + extension *rust-analyzer* fortement recommandés.
- **Commandes de base** : `cargo run` (exécuter), `cargo test` (tester), `cargo check` (vérifier sans exécuter), `cargo clippy` (lints), `cargo fmt` (formatage).

> ⚠️ **Disque** : les dossiers `target/` générés par la compilation peuvent devenir très gros (plusieurs Go pour Bevy ou Tauri). Pense à `cargo clean` dans les projets que tu ne travailles plus.

## Référence

- [GLOSSAIRE.md](GLOSSAIRE.md) — index des concepts : retrouve en un coup d'œil où chaque notion est expliquée.
- [Le Rust Book](https://doc.rust-lang.org/book/) — la référence officielle, pour approfondir.
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) — exemples annotés.
- [docs.rs](https://docs.rs) — documentation de toutes les crates.

Bon courage, et bienvenue chez les Rustacés ! 🦀
