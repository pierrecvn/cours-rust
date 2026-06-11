//! Crate bibliothèque du projet Snake.
//!
//! On expose ici la logique pure du jeu (`logic`) pour qu'elle soit
//! importable à la fois par le binaire (`src/main.rs`) et par les tests
//! d'intégration (`tests/logic_test.rs`). C'est exactement l'organisation
//! lib/bin vue au module 3.

pub mod logic;
