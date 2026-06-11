//! La persistance : lire et écrire le fichier de tâches.
//!
//! Format : une tâche par ligne, `id<TAB>statut<TAB>description`.
//! Deux familles d'échec — le disque (io::Error) et le contenu (ParseError) —
//! réunies dans un seul type somme : StorageError.

use crate::task::{ParseError, Task};
use std::fs;
use std::io::ErrorKind;

/// Tout ce qui peut mal se passer en chargeant ou sauvegardant.
#[derive(Debug)]
pub enum StorageError {
    /// Problème d'entrée/sortie (permissions, disque plein…).
    /// On emballe l'erreur standard pour ne perdre aucun détail.
    Io(std::io::Error),
    /// Une ligne du fichier est invalide ; on retient laquelle et pourquoi.
    Parse { line_number: usize, error: ParseError },
}

impl StorageError {
    /// Message destiné à l'utilisateur.
    pub fn message(&self) -> String {
        match self {
            StorageError::Io(error) => format!("problème d'accès au fichier : {error}"),
            StorageError::Parse { line_number, error } => {
                format!("ligne {line_number} invalide : {}", error.message())
            }
        }
    }
}

/// Sérialise les tâches : une ligne par tâche, terminée par `\n`.
pub fn serialize(tasks: &[Task]) -> String {
    let mut content = String::new();
    for task in tasks {
        // itération par emprunt : `tasks` n'est que lu
        content.push_str(&task.to_line());
        content.push('\n');
    }
    content
}

/// Parse le contenu complet d'un fichier de tâches.
///
/// Séparé de `load` pour être testable sans toucher au disque.
/// On s'arrête à la PREMIÈRE ligne invalide : continuer avec un fichier à
/// moitié compris, puis sauvegarder par-dessus, détruirait des données.
pub fn parse_content(content: &str) -> Result<Vec<Task>, StorageError> {
    let mut tasks = Vec::new();
    let mut line_number = 0;
    for line in content.lines() {
        line_number += 1; // les humains comptent les lignes à partir de 1
        if line.is_empty() {
            continue; // une ligne vide n'est pas une tâche corrompue
        }
        match Task::from_line(line) {
            Ok(task) => tasks.push(task),
            Err(error) => return Err(StorageError::Parse { line_number, error }),
        }
    }
    Ok(tasks)
}

/// Charge les tâches depuis le fichier.
///
/// Un fichier ABSENT n'est pas une erreur : c'est la première utilisation
/// du programme. Toute autre erreur d'E/S, en revanche, est remontée.
pub fn load(path: &str) -> Result<Vec<Task>, StorageError> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        // Garde de motif (`if`) : ce bras ne matche que les NotFound.
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(StorageError::Io(error)),
    };
    parse_content(&content)
}

/// Sauvegarde les tâches dans le fichier (écrasement complet).
pub fn save(path: &str, tasks: &[Task]) -> Result<(), StorageError> {
    match fs::write(path, serialize(tasks)) {
        Ok(()) => Ok(()),
        Err(error) => Err(StorageError::Io(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tasks() -> Vec<Task> {
        vec![
            Task {
                id: 1,
                description: String::from("première"),
                done: false,
            },
            Task {
                id: 2,
                description: String::from("deuxième"),
                done: true,
            },
        ]
    }

    #[test]
    fn serialize_writes_one_task_per_line() {
        let content = serialize(&sample_tasks());
        assert_eq!(content, "1\ttodo\tpremière\n2\tdone\tdeuxième\n");
    }

    #[test]
    fn parse_content_reads_tasks_back() {
        let tasks =
            parse_content("1\ttodo\tpremière\n2\tdone\tdeuxième\n").expect("contenu valide");
        assert_eq!(tasks, sample_tasks());
    }

    #[test]
    fn parse_content_skips_empty_lines() {
        let tasks = parse_content("1\ttodo\tune\n\n2\ttodo\tdeux\n").expect("contenu valide");
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn parse_content_reports_line_number_on_error() {
        let result = parse_content("1\ttodo\tok\nn'importe quoi\n");
        match result {
            Err(StorageError::Parse { line_number, .. }) => assert_eq!(line_number, 2),
            other => panic!("attendu une erreur de parsing, obtenu {other:?}"),
        }
    }

    #[test]
    fn serialize_parse_round_trip() {
        let tasks = sample_tasks();
        let reparsed = parse_content(&serialize(&tasks)).expect("contenu valide");
        assert_eq!(reparsed, tasks);
    }

    #[test]
    fn load_returns_empty_list_when_file_is_missing() {
        let tasks =
            load("/chemin/qui/n/existe/surement/pas/tasks.tsv").expect("absent = liste vide");
        assert!(tasks.is_empty());
    }

    #[test]
    fn save_then_load_round_trip_on_disk() {
        // Un nom unique dans /tmp pour ne pas marcher sur un autre test.
        let path = std::env::temp_dir().join(format!("tasks-test-{}.tsv", std::process::id()));
        let path = path.to_str().expect("chemin UTF-8 valide").to_string();

        let tasks = sample_tasks();
        save(&path, &tasks).expect("écriture possible dans le dossier temporaire");
        let reloaded = load(&path).expect("relecture du fichier écrit");
        let _ = std::fs::remove_file(&path); // nettoyage avant les assertions

        assert_eq!(reloaded, tasks);
    }
}
