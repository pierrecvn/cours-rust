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
    // Le premier argument est le dossier à scanner ; par défaut, le dossier
    // courant.
    let root_arg = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let root = Path::new(&root_arg);

    println!("Scan de {} …\n", root.display());

    // Nombre de workers : autant que de cœurs logiques disponibles.
    let num_workers = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    // --- Version séquentielle, chronométrée (référence de correction). ---
    let start = Instant::now();
    let sequential_stats = match scan_sequential(root) {
        Ok(stats) => stats,
        Err(err) => {
            eprintln!("erreur : impossible de scanner {} ({err})", root.display());
            return ExitCode::FAILURE;
        }
    };
    let sequential_time = start.elapsed();

    // --- Version parallèle, chronométrée. ---
    let start = Instant::now();
    let parallel_stats = match scan_parallel(root, num_workers) {
        Ok(stats) => stats,
        Err(err) => {
            eprintln!("erreur : impossible de scanner {} ({err})", root.display());
            return ExitCode::FAILURE;
        }
    };
    let parallel_time = start.elapsed();

    // Garde-fou : les deux versions doivent trouver les mêmes nombres.
    if sequential_stats.file_count != parallel_stats.file_count {
        eprintln!(
            "avertissement : les deux scans diffèrent ({} vs {} fichiers)",
            sequential_stats.file_count, parallel_stats.file_count
        );
    }

    // Le rapport est construit sur les stats du scan parallèle.
    print_report(&parallel_stats);

    println!("\n=== Temps d'exécution ===");
    println!("Séquentiel : {sequential_time:>12.2?}");
    println!("Parallèle  : {parallel_time:>12.2?} ({num_workers} workers)");
    let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
    println!("Accélération : ×{speedup:.2}");

    ExitCode::SUCCESS
}

/// Affiche le rapport des statistiques : totaux, répartition par extension
/// (triée par taille décroissante) et top des plus gros fichiers.
///
/// Prend `&Stats` : la fonction lit, elle ne possède pas.
fn print_report(stats: &Stats) {
    println!("=== Statistiques ===");
    println!("Fichiers : {}", stats.file_count);
    println!("Taille totale : {}", format_size(stats.total_size));

    println!("\n--- Par extension (triées par taille) ---");
    // Un HashMap n'a pas d'ordre : on collecte puis on trie nous-mêmes.
    let mut entries: Vec<_> = stats.by_extension.iter().collect();
    entries.sort_by(|a, b| b.1.total_size.cmp(&a.1.total_size));
    for (extension, ext_stats) in entries {
        println!(
            "{:<20} {:>8} fichier(s) {:>12}",
            extension,
            ext_stats.count,
            format_size(ext_stats.total_size)
        );
    }

    println!("\n--- Top {} des plus gros fichiers ---", stats.largest.len());
    for (rank, file) in stats.largest.iter().enumerate() {
        println!(
            "{:>2}. {:>12}  {}",
            rank + 1,
            format_size(file.size),
            file.path.display()
        );
    }
}
