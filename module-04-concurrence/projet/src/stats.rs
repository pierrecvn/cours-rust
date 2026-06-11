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
        // TODO (étape 2) : `Stats` dérive `Default`. Une seule ligne suffit
        // (`Self::default()`) : compteurs à zéro, collections vides.
        todo!("étape 2 : construire un Stats vide")
    }

    /// Intègre un fichier aux statistiques.
    pub fn add_file(&mut self, file: FileInfo) {
        // TODO (étape 2) : mettre à jour QUATRE choses.
        //  1. `file_count` (+ 1) et `total_size` (+ `file.size`).
        //  2. l'entrée `by_extension` correspondante : calculez
        //     `extension_of(&file.path)` AVANT de pousser `file` dans `largest`
        //     (le `push` déplace `file`, on ne pourrait plus l'emprunter après).
        //     Idiome HashMap : `self.by_extension.entry(ext).or_default()` rend
        //     un `&mut ExtensionStats` à incrémenter.
        //  3. le classement `largest` : `push`, puis tri décroissant
        //     (`sort_by(|a, b| b.size.cmp(&a.size))`, notez le `b` en premier),
        //     puis `truncate(TOP_N)`.
        todo!("étape 2 : intégrer un fichier aux statistiques")
    }

    /// Fusionne `other` dans `self`.
    ///
    /// `other` est prise PAR VALEUR : on la consomme, ce qui permet de
    /// déplacer ses `String` et ses `FileInfo` sans aucun clonage.
    pub fn merge(&mut self, other: Stats) {
        // TODO (étape 3) : mêmes règles que `add_file`, mais en gros.
        //  - `file_count` et `total_size` : addition.
        //  - `by_extension` : itérez PAR VALEUR
        //    (`for (ext, ext_stats) in other.by_extension`) pour obtenir des
        //    `String` possédées, prêtes pour `entry(ext)`.
        //  - `largest` : `extend(other.largest)`, re-tri décroissant,
        //    `truncate(TOP_N)`.
        todo!("étape 3 : fusionner deux Stats")
    }
}

/// Extension d'un chemin, en minuscules, ou [`NO_EXTENSION`].
///
/// `Path::extension()` gère déjà les pièges : `None` pour `Makefile`
/// comme pour `.gitignore`, et `Some("gz")` pour `archive.tar.gz`.
pub fn extension_of(path: &Path) -> String {
    // TODO (étape 1) : la fonction tient en ~4 lignes.
    //  - `path.extension()` rend un `Option<&OsStr>` (déjà `None` pour
    //    `Makefile` et `.gitignore`, `Some("gz")` pour `archive.tar.gz`).
    //  - Un `OsStr` n'est pas une `String` : `to_string_lossy()` convertit.
    //  - `to_lowercase()` pour regrouper `JPG` et `jpg`.
    //  - Cas `None` : renvoyer `NO_EXTENSION.to_string()`.
    todo!("étape 1 : extension en minuscules, ou NO_EXTENSION")
}

/// Formate une taille en octets de façon lisible : `512 o`, `1.5 Kio`, `3.2 Mio`…
pub fn format_size(bytes: u64) -> String {
    // TODO (étape 3) :
    //  - unités : `["o", "Kio", "Mio", "Gio"]`.
    //  - divisez la valeur (en `f64`) par 1024 tant qu'elle est >= 1024 (sans
    //    dépasser la dernière unité), en avançant dans le tableau.
    //  - les octets s'affichent SANS décimale (`512 o`), le reste avec UNE
    //    décimale (`{value:.1} Kio` → `1.5 Kio`).
    todo!("étape 3 : formater une taille lisible")
}
