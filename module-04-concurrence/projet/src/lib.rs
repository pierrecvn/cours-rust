//! parallel-scanner — bibliothèque du scanner de fichiers parallèle.
//!
//! Deux modules :
//! - [`stats`] : l'agrégation des statistiques (pure logique, testée dans
//!   `tests/stats_tests.rs`) ;
//! - [`scanner`] : le parcours de l'arborescence, en séquentiel et en
//!   parallèle (threads + channel + `Arc<Mutex<…>>`).

pub mod scanner;
pub mod stats;
