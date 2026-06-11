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
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();

        if file_type.is_symlink() {
            // Ignorés : suivre les liens exposerait aux cycles et aux
            // doubles comptages.
            continue;
        } else if file_type.is_dir() {
            // Un sous-dossier illisible (permissions) ne doit pas faire
            // échouer tout le scan : avertissement, puis on continue.
            if let Err(err) = visit_dirs(&path, on_file) {
                eprintln!("avertissement : {} ignoré ({err})", path.display());
            }
        } else if file_type.is_file() {
            on_file(path);
        }
        // Les autres types (sockets, FIFO…) sont ignorés silencieusement.
    }
    Ok(())
}

/// Scanne `root` séquentiellement : la version de référence.
pub fn scan_sequential(root: &Path) -> io::Result<Stats> {
    let mut stats = Stats::new();

    // La closure capture `stats` par emprunt mutable : c'est exactement
    // pour cela que `visit_dirs` exige `FnMut` et non `Fn`.
    visit_dirs(root, &mut |path| match fs::metadata(&path) {
        Ok(metadata) => stats.add_file(FileInfo::new(path, metadata.len())),
        Err(err) => eprintln!("avertissement : {} ignoré ({err})", path.display()),
    })?;

    Ok(stats)
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

    let (sender, receiver) = mpsc::channel::<PathBuf>();

    // mpsc = "multiple producer, SINGLE consumer" : il n'existe qu'un seul
    // Receiver, or N workers veulent piocher dedans. On le partage donc
    // derrière Arc (possession partagée) + Mutex (un seul lecteur à la fois).
    let receiver = Arc::new(Mutex::new(receiver));

    // Les statistiques partagées. `Rc<RefCell<Stats>>` serait refusé par le
    // compilateur (Rc n'est pas Send) : entre threads, c'est Arc + Mutex.
    let stats = Arc::new(Mutex::new(Stats::new()));

    let mut handles = Vec::with_capacity(num_workers);
    for _ in 0..num_workers {
        // Un clone des deux Arc PAR worker, créé avant le spawn : la
        // closure `move` prend possession de ses propres clones, les
        // originaux restent disponibles pour le tour de boucle suivant.
        let receiver = Arc::clone(&receiver);
        let stats = Arc::clone(&stats);

        handles.push(thread::spawn(move || {
            loop {
                // Tout en UNE expression : le MutexGuard est temporaire et
                // meurt en fin d'instruction, donc le verrou du channel est
                // relâché AVANT l'appel système. Garder le guard vivant ici
                // rendrait le scan… séquentiel, avec des frais en plus.
                let message = receiver.lock().expect("mutex du channel empoisonné").recv();

                let path = match message {
                    Ok(path) => path,
                    // Err = le Sender est mort et la file est vide :
                    // c'est le signal de fin, pas une erreur.
                    Err(_) => break,
                };

                match fs::metadata(&path) {
                    Ok(metadata) => {
                        let info = FileInfo::new(path, metadata.len());
                        // Section critique la plus courte possible : on
                        // verrouille seulement pour l'agrégation, jamais
                        // pendant les entrées/sorties.
                        stats.lock().expect("mutex des stats empoisonné").add_file(info);
                    }
                    Err(err) => {
                        eprintln!("avertissement : {} ignoré ({err})", path.display());
                    }
                }
            }
        }));
    }

    // Le thread principal produit le travail : un chemin envoyé = un chemin
    // DÉPLACÉ dans le channel (ownership transféré avec le message).
    let walk_result = visit_dirs(root, &mut |path| {
        // `send` n'échoue que si tous les Receiver sont morts, c'est-à-dire
        // si tous les workers ont paniqué ; le join ci-dessous le signalera.
        let _ = sender.send(path);
    });

    // Indispensable : détruire le Sender ferme le channel, ce qui fait
    // sortir les workers de leur boucle. Sans ce drop, les recv() attendent
    // pour toujours et le programme se fige.
    drop(sender);

    for handle in handles {
        handle.join().expect("un worker a paniqué");
    }

    // L'erreur éventuelle du parcours est remontée APRÈS le join : on ne
    // laisse jamais des threads orphelins derrière soi.
    walk_result?;

    // Tous les workers sont terminés, leurs clones d'Arc sont morts : il ne
    // reste qu'un propriétaire. On peut donc déballer l'Arc puis le Mutex
    // pour rendre un Stats nu, agréable pour l'appelant.
    let stats = Arc::try_unwrap(stats).expect("des clones de l'Arc existent encore");
    Ok(stats.into_inner().expect("mutex des stats empoisonné"))
}
