//! Point d'entrée du gestionnaire de tâches.
//!
//! `main` ne contient AUCUNE logique métier : il orchestre les trois modules
//! (cli -> storage -> task) et soigne l'affichage des erreurs.
//!
//! Suis l'étape 6 de PROJET.md.

// `#![allow(unused)]` évite les avertissements (imports, paramètres, code mort)
// tant que le squelette n'est pas terminé. Retire cette ligne à la fin du
// projet : les avertissements du compilateur redeviendront tes alliés.
#![allow(unused)]

// Rattachement explicite des fichiers à l'arbre des modules : sans ces trois
// lignes, task.rs, storage.rs et cli.rs n'existent pas pour le compilateur.
mod cli;
mod storage;
mod task;

use cli::Command;
use std::env;
use std::process;
use storage::StorageError;
use task::TaskList;

/// Le fichier de persistance, créé dans le répertoire courant.
const STORAGE_FILE: &str = "tasks.tsv";

fn main() {
    // TODO (étape 6) :
    //   1. `let args: Vec<String> = env::args().collect();` — `args[0]` est le
    //      chemin du programme ; passe `&args[1..]` à `cli::parse_args`.
    //   2. En cas d'erreur de `parse_args` : message sur STDERR (`eprintln!`),
    //      affiche `cli::USAGE`, puis `process::exit(2)`.
    //   3. Sinon, appelle `run(command)`. Centralise l'affichage d'erreur ici :
    //      `if let Err(error) = run(command) { eprintln!(...); process::exit(1); }`
    todo!("étape 6 : lire les arguments, parser, exécuter, gérer les erreurs")
}

/// Exécute la commande : charge, agit, sauvegarde.
/// Retourner un Result autorise l'opérateur `?` sur load et save.
fn run(command: Command) -> Result<(), StorageError> {
    // TODO (étape 6) :
    //   1. Charge et construis la liste :
    //      `let mut list = TaskList::from_tasks(storage::load(STORAGE_FILE)?);`
    //      (le `?` remonte une éventuelle erreur de chargement).
    //   2. Un grand `match command` EXHAUSTIF :
    //      - `Help` : affiche `cli::USAGE`, `return Ok(())` (rien à sauvegarder) ;
    //      - `List` : `print_list(&list)`, `return Ok(())` (lecture seule) ;
    //      - `Add(description)` : `list.add(...)`, affiche « Ajoutée : [id] desc » ;
    //      - `Done(id)` : `match list.mark_done(id)` — `Some(task)` affiche
    //        « Terminée : ... » ; `None` affiche un message et `process::exit(1)`
    //        (rien n'a changé, on sort sans sauvegarder) ;
    //      - `Remove(id)` : idem avec `list.remove(id)` et « Supprimée : ... ».
    //   3. Après le `match` (seulement atteint pour Add/Done/Remove) :
    //      `storage::save(STORAGE_FILE, list.tasks())`.
    todo!("étape 6 : orchestrer load -> action -> save")
}

/// Affiche la liste : une case à cocher, l'id aligné à droite, la description.
fn print_list(list: &TaskList) {
    // TODO (étape 6) :
    //   - liste vide : un petit message d'accueil, puis `return` ;
    //   - sinon, `for task in list.tasks()` (par emprunt) : une case `[x]`/`[ ]`
    //     selon `task.done`, puis `println!("[{marker}] {:>3}  {}", task.id, ...)`
    //     (`{:>3}` aligne l'id à droite sur 3 caractères).
    todo!("étape 6 : afficher la liste des tâches")
}
