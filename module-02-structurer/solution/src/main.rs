//! Point d'entrée du gestionnaire de tâches.
//!
//! `main` ne contient AUCUNE logique métier : il orchestre les trois modules
//! (cli -> storage -> task) et soigne l'affichage des erreurs.

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
    // args() rend un itérateur ; collect() le matérialise en Vec<String>.
    let args: Vec<String> = env::args().collect();

    // args[0] est le chemin du programme : on passe le reste au parseur.
    let command = match cli::parse_args(&args[1..]) {
        Ok(command) => command,
        Err(message) => {
            // Les erreurs vont sur stderr, jamais sur stdout.
            eprintln!("Erreur : {message}");
            eprintln!();
            eprintln!("{}", cli::USAGE);
            // Code 2 : erreur d'utilisation (convention des outils Unix).
            process::exit(2);
        }
    };

    // Toute la logique faillible vit dans `run`, qui retourne un Result :
    // l'affichage de l'erreur est centralisé ici, en un seul endroit.
    if let Err(error) = run(command) {
        eprintln!("Erreur : {}", error.message());
        process::exit(1);
    }
}

/// Exécute la commande : charge, agit, sauvegarde.
/// Retourner un Result autorise l'opérateur `?` sur load et save.
fn run(command: Command) -> Result<(), StorageError> {
    // `?` : si le chargement échoue, on remonte l'erreur immédiatement.
    let mut list = TaskList::from_tasks(storage::load(STORAGE_FILE)?);

    // Un match EXHAUSTIF : ajouter une variante à Command demain forcera
    // à décider de son comportement ici.
    match command {
        Command::Help => {
            println!("{}", cli::USAGE);
            return Ok(()); // rien à sauvegarder
        }
        Command::List => {
            print_list(&list);
            return Ok(()); // lecture seule : rien à sauvegarder non plus
        }
        Command::Add(description) => {
            let task = list.add(description);
            println!("Ajoutée : [{}] {}", task.id, task.description);
        }
        Command::Done(id) => match list.mark_done(id) {
            Some(task) => println!("Terminée : [{}] {}", task.id, task.description),
            None => {
                eprintln!("Aucune tâche avec l'identifiant {id}");
                process::exit(1); // rien n'a changé : on sort sans sauvegarder
            }
        },
        Command::Remove(id) => match list.remove(id) {
            Some(task) => println!("Supprimée : [{}] {}", task.id, task.description),
            None => {
                eprintln!("Aucune tâche avec l'identifiant {id}");
                process::exit(1);
            }
        },
    }

    // On n'arrive ici que pour Add/Done/Remove : la liste a changé.
    storage::save(STORAGE_FILE, list.tasks())
}

/// Affiche la liste : une case à cocher, l'id aligné à droite, la description.
fn print_list(list: &TaskList) {
    if list.tasks().is_empty() {
        println!("Aucune tâche. Ajoutez-en une avec : tasks add \"...\"");
        return;
    }
    for task in list.tasks() {
        // encore une itération par emprunt
        let marker = if task.done { "x" } else { " " };
        println!("[{marker}] {:>3}  {}", task.id, task.description);
    }
}
