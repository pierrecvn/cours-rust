//! Agrégation des statistiques du scan (étapes 1 à 3 du projet).
//!
//! Ce module est de la pure logique : aucune entrée/sortie, aucun thread.
//! C'est ce qui le rend trivial à tester — et c'est un découpage à imiter.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Nombre de fichiers conservés dans le classement des plus gros.
pub const TOP_N: usize = 10;

/// Clé sous laquelle sont regroupés les fichiers sans extension.
pub const NO_EXTENSION: &str = "<sans extension>";

/// Un fichier rencontré pendant le parcours : son chemin et sa taille en octets.
///
/// La struct POSSÈDE son chemin (`PathBuf`, pas `&Path`) : aucune lifetime à
/// annoter, et surtout elle satisfait la contrainte `'static` de
/// `thread::spawn` — on peut l'envoyer librement d'un thread à l'autre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
}

impl FileInfo {
    pub fn new(path: PathBuf, size: u64) -> Self {
        Self { path, size }
    }
}

/// Statistiques cumulées pour une extension donnée.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExtensionStats {
    pub count: u64,
    pub total_size: u64,
}

/// Les statistiques globales d'un scan.
#[derive(Debug, Default)]
pub struct Stats {
    /// Nombre total de fichiers vus.
    pub file_count: u64,
    /// Taille cumulée de tous les fichiers, en octets.
    pub total_size: u64,
    /// Statistiques par extension (clé : extension en minuscules,
    /// ou [`NO_EXTENSION`]).
    pub by_extension: HashMap<String, ExtensionStats>,
    /// Les [`TOP_N`] plus gros fichiers, triés par taille décroissante.
    pub largest: Vec<FileInfo>,
}

impl Stats {
    pub fn new() -> Self {
        // `#[derive(Default)]` fait le travail : compteurs à zéro,
        // collections vides.
        Self::default()
    }

    /// Intègre un fichier aux statistiques.
    pub fn add_file(&mut self, file: FileInfo) {
        self.file_count += 1;
        self.total_size += file.size;

        // L'extension est calculée AVANT de pousser `file` dans `largest` :
        // le `push` déplace `file`, on ne pourrait plus l'emprunter après.
        let extension = extension_of(&file.path);
        let entry = self.by_extension.entry(extension).or_default();
        entry.count += 1;
        entry.total_size += file.size;

        // Classement des plus gros : push, tri décroissant, troncature.
        // O(n log n) à chaque insertion sur un vecteur d'au plus TOP_N + 1
        // éléments : largement assez rapide ici, et limpide à lire.
        self.largest.push(file);
        self.largest.sort_by(|a, b| b.size.cmp(&a.size));
        self.largest.truncate(TOP_N);
    }

    /// Fusionne `other` dans `self`.
    ///
    /// `other` est prise PAR VALEUR : on la consomme, ce qui permet de
    /// déplacer ses `String` et ses `FileInfo` sans aucun clonage.
    pub fn merge(&mut self, other: Stats) {
        self.file_count += other.file_count;
        self.total_size += other.total_size;

        // Itération par valeur sur le HashMap : chaque tour donne une
        // `(String, ExtensionStats)` possédée.
        for (extension, ext_stats) in other.by_extension {
            let entry = self.by_extension.entry(extension).or_default();
            entry.count += ext_stats.count;
            entry.total_size += ext_stats.total_size;
        }

        self.largest.extend(other.largest);
        self.largest.sort_by(|a, b| b.size.cmp(&a.size));
        self.largest.truncate(TOP_N);
    }
}

/// Extension d'un chemin, en minuscules, ou [`NO_EXTENSION`].
///
/// `Path::extension()` gère déjà les pièges : `None` pour `Makefile`
/// comme pour `.gitignore`, et `Some("gz")` pour `archive.tar.gz`.
pub fn extension_of(path: &Path) -> String {
    match path.extension() {
        // Un nom de fichier n'est pas forcément de l'UTF-8 valide :
        // `to_string_lossy` convertit en remplaçant les octets fautifs.
        Some(ext) => ext.to_string_lossy().to_lowercase(),
        None => NO_EXTENSION.to_string(),
    }
}

/// Formate une taille en octets de façon lisible : `512 o`, `1.5 Kio`, `3.2 Mio`…
pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["o", "Kio", "Mio", "Gio"];

    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        // Les octets s'affichent exacts, sans décimale.
        format!("{bytes} {}", UNITS[0])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
