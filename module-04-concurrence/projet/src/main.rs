//! parallel-scanner — le binaire (étapes 5 et 7 du projet).
//!
//! Parse le dossier à scanner, lance les deux versions (séquentielle puis
//! parallèle) en les chronométrant, puis affiche un rapport et l'accélération.

use std::env;
use std::path::Path;
use std::process::ExitCode;
use std::thread;
use std::time::Instant;

use parallel_scanner::scanner::{scan_parallel, scan_sequential};
use parallel_scanner::stats::{Stats, format_size};

fn main() -> ExitCode {
    // Le squelette parse déjà l'argument : le premier argument est le dossier
    // à scanner ; par défaut, le dossier courant.
    let root_arg = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let root = Path::new(&root_arg);

    println!("Scan de {} …\n", root.display());

    // Nombre de workers : autant que de cœurs logiques disponibles (étape 7).
    let num_workers = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    // TODO (étapes 5 puis 7) :
    //  - étape 5 : scan SÉQUENTIEL chronométré. Entourez `scan_sequential(root)`
    //    de `let start = Instant::now();` … `let elapsed = start.elapsed();`.
    //    `scan_sequential` rend un `io::Result<Stats>` : un `match` qui, en cas
    //    d'erreur, `eprintln!` et renvoie `ExitCode::FAILURE`.
    //  - étape 7 : scan PARALLÈLE chronométré (`scan_parallel(root, num_workers)`),
    //    même gestion d'erreur. Petit garde-fou : si les deux `file_count`
    //    diffèrent, un avertissement.
    //  - affichez le rapport via `print_report(&parallel_stats)`.
    //  - affichez les temps (`{elapsed:>12.2?}`) et l'accélération :
    //    `sequential.as_secs_f64() / parallel.as_secs_f64()`, format `×{:.2}`.
    //  - renvoyez `ExitCode::SUCCESS`.
    let _ = (root, num_workers); // (à retirer une fois les scans branchés)
    todo!("étapes 5 et 7 : chronométrer les deux scans et afficher le rapport")
}

/// Affiche le rapport des statistiques : totaux, répartition par extension
/// (triée par taille décroissante) et top des plus gros fichiers.
///
/// Prend `&Stats` : la fonction lit, elle ne possède pas.
#[allow(dead_code)]
fn print_report(stats: &Stats) {
    // TODO (étape 7) :
    //  - totaux : `file_count` et `total_size` (via `format_size`).
    //  - par extension : un `HashMap` n'a pas d'ordre, collectez puis triez :
    //    `let mut entries: Vec<_> = stats.by_extension.iter().collect();`
    //    `entries.sort_by(|a, b| b.1.total_size.cmp(&a.1.total_size));`
    //    Alignement : `{:<20}` (extension), `{:>8}` et `{:>12}` (nombres).
    //  - top des plus gros : itérez `stats.largest` (avec `.enumerate()` pour
    //    le rang) ; affichez rang, `format_size(file.size)` et `file.path.display()`.
    todo!("étape 7 : afficher le rapport (totaux, extensions, top N)")
}
