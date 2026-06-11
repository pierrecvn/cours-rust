//! Tests de la logique d'agrégation des statistiques.
//!
//! Ces tests sont FOURNIS : ne les modifiez pas. Votre travail (étapes 1 à 3)
//! est de les faire passer en implémentant `src/stats.rs`.

use std::path::PathBuf;

use parallel_scanner::stats::{
    FileInfo, NO_EXTENSION, Stats, TOP_N, extension_of, format_size,
};

// --- Étape 1 : extension_of -------------------------------------------------

#[test]
fn extension_is_lowercased() {
    assert_eq!(extension_of(&PathBuf::from("photo.JPG")), "jpg");
    assert_eq!(extension_of(&PathBuf::from("photo.jpg")), "jpg");
}

#[test]
fn extension_keeps_only_the_last_component() {
    assert_eq!(extension_of(&PathBuf::from("archive.tar.gz")), "gz");
}

#[test]
fn files_without_extension_are_grouped() {
    // `Makefile` n'a pas d'extension ; `.gitignore` non plus (le point initial
    // d'un fichier caché n'introduit pas une extension).
    assert_eq!(extension_of(&PathBuf::from("Makefile")), NO_EXTENSION);
    assert_eq!(extension_of(&PathBuf::from(".gitignore")), NO_EXTENSION);
}

// --- Étape 2 : add_file -----------------------------------------------------

#[test]
fn add_file_updates_global_counters() {
    let mut stats = Stats::new();
    stats.add_file(FileInfo::new(PathBuf::from("a.rs"), 100));
    stats.add_file(FileInfo::new(PathBuf::from("b.rs"), 50));

    assert_eq!(stats.file_count, 2);
    assert_eq!(stats.total_size, 150);
}

#[test]
fn add_file_aggregates_by_extension() {
    let mut stats = Stats::new();
    stats.add_file(FileInfo::new(PathBuf::from("a.rs"), 100));
    stats.add_file(FileInfo::new(PathBuf::from("b.RS"), 50));
    stats.add_file(FileInfo::new(PathBuf::from("Makefile"), 10));

    let rs = stats.by_extension.get("rs").expect("clé rs manquante");
    assert_eq!(rs.count, 2);
    assert_eq!(rs.total_size, 150);

    let none = stats
        .by_extension
        .get(NO_EXTENSION)
        .expect("clé sans extension manquante");
    assert_eq!(none.count, 1);
    assert_eq!(none.total_size, 10);
}

#[test]
fn largest_keeps_only_the_top_n_sorted_descending() {
    let mut stats = Stats::new();
    // On insère TOP_N + 5 fichiers de tailles croissantes, dans le désordre.
    for size in 0..(TOP_N as u64 + 5) {
        stats.add_file(FileInfo::new(PathBuf::from(format!("f{size}.bin")), size));
    }

    // On ne garde que TOP_N fichiers.
    assert_eq!(stats.largest.len(), TOP_N);

    // Triés par taille décroissante.
    for pair in stats.largest.windows(2) {
        assert!(pair[0].size >= pair[1].size, "largest doit être trié décroissant");
    }

    // Le plus gros est bien le dernier inséré (taille TOP_N + 4).
    assert_eq!(stats.largest[0].size, TOP_N as u64 + 4);
}

// --- Étape 3 : merge & format_size -----------------------------------------

#[test]
fn merge_combines_two_stats() {
    let mut a = Stats::new();
    a.add_file(FileInfo::new(PathBuf::from("a.rs"), 100));
    a.add_file(FileInfo::new(PathBuf::from("img.png"), 2000));

    let mut b = Stats::new();
    b.add_file(FileInfo::new(PathBuf::from("b.rs"), 50));
    b.add_file(FileInfo::new(PathBuf::from("c.txt"), 10));

    a.merge(b);

    assert_eq!(a.file_count, 4);
    assert_eq!(a.total_size, 2160);

    let rs = a.by_extension.get("rs").expect("clé rs manquante");
    assert_eq!(rs.count, 2);
    assert_eq!(rs.total_size, 150);

    // Le plus gros fichier (img.png, 2000) reste en tête après fusion.
    assert_eq!(a.largest[0].size, 2000);
}

#[test]
fn merge_respects_top_n() {
    let mut a = Stats::new();
    for size in 0..TOP_N as u64 {
        a.add_file(FileInfo::new(PathBuf::from(format!("a{size}.bin")), size));
    }

    let mut b = Stats::new();
    for size in 0..TOP_N as u64 {
        b.add_file(FileInfo::new(PathBuf::from(format!("b{size}.bin")), size + 1000));
    }

    a.merge(b);

    // Après fusion, on ne garde toujours que TOP_N fichiers.
    assert_eq!(a.largest.len(), TOP_N);
    // Tous viennent de `b` (les plus gros).
    assert!(a.largest.iter().all(|f| f.size >= 1000));
}

#[test]
fn format_size_handles_units() {
    assert_eq!(format_size(0), "0 o");
    assert_eq!(format_size(512), "512 o");
    assert_eq!(format_size(1024), "1.0 Kio");
    assert_eq!(format_size(1536), "1.5 Kio");
    assert_eq!(format_size(1024 * 1024), "1.0 Mio");
    assert_eq!(format_size(3 * 1024 * 1024 + 200 * 1024), "3.2 Mio");
    assert_eq!(format_size(1024 * 1024 * 1024), "1.0 Gio");
}
