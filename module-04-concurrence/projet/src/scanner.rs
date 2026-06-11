//! Parcours de l'arborescence : versions séquentielle et parallèle
//! (étapes 4 à 6 du projet).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::stats::{FileInfo, Stats};

/// Parcourt récursivement `dir` et appelle `on_file` pour chaque fichier.
///
/// - Les liens symboliques sont ignorés (un lien vers un dossier parent
///   créerait une récursion infinie).
/// - Un sous-dossier illisible est signalé sur stderr puis ignoré ; seule
///   une erreur sur `dir` lui-même fait échouer l'appel.
///
/// La fonction est générique sur une closure `FnMut` : le parcours est
/// séparé de l'action. Le scan séquentiel y branche « remplir un Stats »,
/// le scan parallèle y branche « envoyer dans le channel ». On passe
/// `&mut F` (et non `F`) pour pouvoir réutiliser la même closure dans les
/// appels récursifs sans la déplacer.
pub fn visit_dirs<F: FnMut(PathBuf)>(dir: &Path, on_file: &mut F) -> io::Result<()> {
    // TODO (étape 4) : le parcours récursif, notre `walkdir` maison.
    //  - `fs::read_dir(dir)?` rend un itérateur de `Result<DirEntry, _>` :
    //    `for entry in fs::read_dir(dir)? { let entry = entry?; … }` (deux `?`).
    //  - pour chaque entrée : `entry.file_type()?` et `entry.path()`.
    //    Testez `is_symlink()` EN PREMIER et `continue` (évite les cycles) ;
    //    sinon, si `is_dir()`, appel récursif `visit_dirs(&path, on_file)`
    //    entouré d'un `if let Err(err) = …` qui `eprintln!` un avertissement et
    //    continue ; sinon, si `is_file()`, `on_file(path)`.
    //  - les autres types (sockets, FIFO…) sont ignorés.
    todo!("étape 4 : parcours récursif appelant on_file pour chaque fichier")
}

/// Scanne `root` séquentiellement : la version de référence.
pub fn scan_sequential(root: &Path) -> io::Result<Stats> {
    // TODO (étape 5) :
    //  - créez un `Stats` mutable.
    //  - appelez `visit_dirs(root, &mut |path| …)` avec une closure qui
    //    capture `&mut stats` (d'où le `FnMut`). Pour chaque chemin :
    //    `fs::metadata(&path)` donne la taille (`metadata.len()`) ; en cas de
    //    succès, `stats.add_file(FileInfo::new(path, size))` ; en cas d'erreur,
    //    `eprintln!` un avertissement et on continue.
    //  - propagez l'erreur du parcours (`?`) puis renvoyez `Ok(stats)`.
    todo!("étape 5 : scan séquentiel, version de référence")
}

/// Scanne `root` avec un pool de `num_workers` threads.
///
/// Architecture (voir PROJET.md, étape 6) :
/// - le thread principal parcourt l'arborescence et envoie chaque chemin
///   dans un channel mpsc (la file de travail) ;
/// - les workers consomment le channel, font le `fs::metadata` (le travail
///   parallélisé) et agrègent dans un `Stats` partagé ;
/// - la fermeture du channel (drop du `Sender`) signale la fin du travail.
pub fn scan_parallel(root: &Path, num_workers: usize) -> io::Result<Stats> {
    assert!(num_workers > 0, "il faut au moins un worker");

    // TODO (étape 6) : LE morceau du module.
    //  1. créez le channel : `mpsc::channel::<PathBuf>()`.
    //  2. mpsc = single consumer : enveloppez le récepteur dans un
    //     `Arc::new(Mutex::new(receiver))` pour le partager entre workers.
    //  3. créez le `Stats` partagé : `Arc::new(Mutex::new(Stats::new()))`.
    //  4. pour `_ in 0..num_workers` : clonez les DEUX `Arc` (`Arc::clone`)
    //     AVANT le `thread::spawn(move || …)`, et stockez le `JoinHandle`.
    //     Boucle d'un worker :
    //       loop {
    //           let message = receiver.lock().expect("…").recv(); // UNE expr !
    //           let path = match message { Ok(p) => p, Err(_) => break };
    //           // fs::metadata + add_file (verrou des stats le plus bref possible)
    //       }
    //     ⚠ `receiver.lock().…recv()` en une seule expression : le guard meurt
    //     en fin d'instruction, le verrou est relâché AVANT le `metadata`.
    //  5. produisez le travail :
    //     `visit_dirs(root, &mut |path| { let _ = sender.send(path); })`,
    //     puis **`drop(sender)`** (sinon les `recv()` attendent pour toujours).
    //  6. `join` chaque handle (`.expect("un worker a paniqué")`), PUIS
    //     propagez l'erreur du parcours (`?`).
    //  7. récupérez le `Stats` nu : `Arc::try_unwrap(stats)` puis `.into_inner()`
    //     sur le `Mutex` (les deux rendent un `Result` : `expect` clair).
    todo!("étape 6 : pool de workers + channel + Arc<Mutex<Stats>>")
}
