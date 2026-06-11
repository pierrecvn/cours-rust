//! Le modèle de données : la tâche, ses erreurs de parsing, et la liste.
//!
//! Ce module ne fait AUCUNE entrée/sortie : il ne connaît ni le disque ni le
//! terminal. C'est ce qui le rend trivial à tester.

/// Une tâche du gestionnaire.
///
/// Les trois dérives du module : `Debug` pour `{:?}`, `Clone` pour copier
/// explicitement, `PartialEq` pour `==` (et donc `assert_eq!` dans les tests).
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: u64,
    pub description: String,
    pub done: bool,
}

/// Tout ce qui peut rendre une ligne du fichier invalide.
///
/// Un enum plutôt qu'une `String` : l'appelant peut réagir cas par cas, et
/// les tests peuvent comparer des variantes précises au lieu de textes.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Moins de trois champs séparés par des tabulations.
    MissingField,
    /// Le premier champ n'est pas un entier ; on garde la valeur fautive.
    InvalidId(String),
    /// Le deuxième champ n'est ni « todo » ni « done ».
    InvalidStatus(String),
}

impl ParseError {
    /// Message destiné à l'utilisateur. Un `match` exhaustif : si on ajoute
    /// une variante demain, le compilateur exigera un message ici aussi.
    pub fn message(&self) -> String {
        match self {
            ParseError::MissingField => {
                String::from("champs manquants (attendu : id<TAB>statut<TAB>description)")
            }
            ParseError::InvalidId(value) => format!("identifiant invalide : « {value} »"),
            ParseError::InvalidStatus(value) => {
                format!("statut invalide : « {value} » (attendu : todo ou done)")
            }
        }
    }
}

impl Task {
    /// Fonction associée (pas de `self`) : le « constructeur » par convention.
    /// Une tâche naît non terminée.
    pub fn new(id: u64, description: String) -> Self {
        Self {
            id,
            description, // raccourci de champ : `description: description`
            done: false,
        }
    }

    /// Sérialise la tâche en une ligne du fichier : `id<TAB>statut<TAB>description`.
    /// Sans `\n` final : c'est la couche storage qui assemble les lignes.
    pub fn to_line(&self) -> String {
        // `if` est une expression : il produit directement la valeur.
        let status = if self.done { "done" } else { "todo" };
        format!("{}\t{}\t{}", self.id, status, self.description)
    }

    /// Parse une ligne du fichier. Chaque problème possible est une variante
    /// de `ParseError` : un fichier corrompu ne fait jamais paniquer.
    pub fn from_line(line: &str) -> Result<Task, ParseError> {
        // splitn(3, ...) : coupe en 3 morceaux AU PLUS. Le troisième morceau
        // garde donc ses éventuelles tabulations : une description en
        // contenant survit à l'aller-retour.
        let mut fields = line.splitn(3, '\t');

        // `let else` : déballe l'Option ou sort. Trois gardes d'affilée,
        // le « cas heureux » reste à plat, sans indentation.
        let Some(id_field) = fields.next() else {
            return Err(ParseError::MissingField);
        };
        let Some(status_field) = fields.next() else {
            return Err(ParseError::MissingField);
        };
        let Some(description) = fields.next() else {
            return Err(ParseError::MissingField);
        };

        // `parse` retourne un Result, mais son erreur (ParseIntError) n'est
        // pas la nôtre : on traduit avec un `match` plutôt qu'un `?`.
        // (La conversion automatique d'erreurs, c'est au module 3.)
        let id = match id_field.parse::<u64>() {
            Ok(id) => id,
            Err(_) => return Err(ParseError::InvalidId(id_field.to_string())),
        };

        // Un match sur le &str : les deux statuts connus, et un bras `other`
        // qui capture la valeur fautive pour le message d'erreur.
        let done = match status_field {
            "todo" => false,
            "done" => true,
            other => return Err(ParseError::InvalidStatus(other.to_string())),
        };

        Ok(Task {
            id,
            description: description.to_string(), // &str emprunté -> String possédée
            done,
        })
    }
}

/// La collection de tâches, avec ses règles métier (attribution des ids).
///
/// Le champ `tasks` est PRIVÉ : l'extérieur ne peut pas pousser une tâche
/// avec un id arbitraire. Seules les méthodes publiques font foi — c'est
/// l'encapsulation à la Rust, au niveau du module.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskList {
    tasks: Vec<Task>,
}

impl TaskList {
    /// Une liste vide.
    ///
    /// Le binaire n'en a pas (encore) besoin — le chargement passe par
    /// `from_tasks` — mais les tests s'en servent. Sans cet `allow`,
    /// `cargo build` signalerait du code mort (lint `dead_code`).
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Construit la liste à partir de tâches existantes (celles du fichier).
    pub fn from_tasks(tasks: Vec<Task>) -> Self {
        Self { tasks }
    }

    /// Lecture seule sur les tâches. `&[Task]` (une slice) plutôt que
    /// `&Vec<Task>` : l'appelant peut lire, jamais modifier.
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Le prochain id : le plus grand id existant + 1 (1 pour une liste vide).
    /// On ne suppose pas la liste triée : on balaie tout.
    fn next_id(&self) -> u64 {
        let mut max = 0;
        for task in &self.tasks {
            // itération par EMPRUNT : la liste reste utilisable après
            if task.id > max {
                max = task.id;
            }
        }
        max + 1
    }

    /// Ajoute une tâche et retourne une référence vers elle (pour l'affichage).
    pub fn add(&mut self, description: String) -> &Task {
        // L'id est calculé AVANT le push : `Task::new(self.next_id(), ...)`
        // dans l'argument de `push` créerait un conflit d'emprunts (E0502).
        let id = self.next_id();
        self.tasks.push(Task::new(id, description));
        // `last()` retourne Option<&Task> ; le push garantit Some.
        // `expect` documente pourquoi ce déballage ne peut pas échouer.
        self.tasks
            .last()
            .expect("le Vec ne peut pas être vide juste après un push")
    }

    /// Marque une tâche comme terminée. `None` si l'id est inconnu :
    /// l'appelant est OBLIGÉ de traiter ce cas — pas de NPE possible.
    pub fn mark_done(&mut self, id: u64) -> Option<&Task> {
        // `&mut self.tasks` : emprunt mutable, on modifie en place.
        for task in &mut self.tasks {
            if task.id == id {
                task.done = true;
                return Some(task);
            }
        }
        None
    }

    /// Supprime une tâche et la retourne par VALEUR (l'ownership passe à
    /// l'appelant). `None` si l'id est inconnu.
    pub fn remove(&mut self, id: u64) -> Option<Task> {
        // Vec ne supprime que par indice : on cherche d'abord la position.
        for index in 0..self.tasks.len() {
            if self.tasks[index].id == id {
                // Vec::remove sort l'élément du Vec et nous le donne.
                return Some(self.tasks.remove(index));
            }
        }
        None
    }
}

// Module de tests « en ligne », compilé seulement par `cargo test`.
#[cfg(test)]
mod tests {
    // `super::*` : tout le module parent (Task, TaskList, ParseError…).
    use super::*;

    // --- Sérialisation -----------------------------------------------------

    #[test]
    fn to_line_formats_pending_task() {
        let task = Task::new(1, String::from("acheter du pain"));
        assert_eq!(task.to_line(), "1\ttodo\tacheter du pain");
    }

    #[test]
    fn to_line_formats_done_task() {
        let task = Task {
            id: 7,
            description: String::from("relire le cours"),
            done: true,
        };
        assert_eq!(task.to_line(), "7\tdone\trelire le cours");
    }

    // --- Parsing -----------------------------------------------------------

    #[test]
    fn from_line_parses_pending_task() {
        let task = Task::from_line("1\ttodo\tacheter du pain").expect("la ligne est valide");
        assert_eq!(
            task,
            Task {
                id: 1,
                description: String::from("acheter du pain"),
                done: false,
            }
        );
    }

    #[test]
    fn from_line_parses_done_task() {
        let task = Task::from_line("12\tdone\tappeler le garage").expect("la ligne est valide");
        assert_eq!(task.id, 12);
        assert!(task.done);
    }

    #[test]
    fn from_line_keeps_tabs_inside_description() {
        let task = Task::from_line("3\ttodo\tcolonnes\tavec\ttabulations")
            .expect("la ligne est valide");
        assert_eq!(task.description, "colonnes\tavec\ttabulations");
    }

    #[test]
    fn from_line_rejects_missing_fields() {
        assert_eq!(Task::from_line("1\ttodo"), Err(ParseError::MissingField));
    }

    #[test]
    fn from_line_rejects_invalid_id() {
        assert_eq!(
            Task::from_line("abc\ttodo\tfaire un truc"),
            Err(ParseError::InvalidId(String::from("abc")))
        );
    }

    #[test]
    fn from_line_rejects_invalid_status() {
        assert_eq!(
            Task::from_line("1\tfinished\tfaire un truc"),
            Err(ParseError::InvalidStatus(String::from("finished")))
        );
    }

    #[test]
    fn line_round_trip() {
        let original = Task {
            id: 42,
            description: String::from("boucler la boucle"),
            done: true,
        };
        let reparsed = Task::from_line(&original.to_line()).expect("la ligne générée est valide");
        assert_eq!(reparsed, original);
    }

    #[test]
    fn parse_error_messages_mention_the_culprit() {
        assert!(ParseError::InvalidId(String::from("abc")).message().contains("abc"));
        assert!(
            ParseError::InvalidStatus(String::from("finished"))
                .message()
                .contains("finished")
        );
    }

    // --- TaskList ----------------------------------------------------------

    #[test]
    fn add_assigns_sequential_ids() {
        let mut list = TaskList::new();
        let first_id = list.add(String::from("première")).id;
        let second_id = list.add(String::from("deuxième")).id;
        assert_eq!(first_id, 1);
        assert_eq!(second_id, 2);
    }

    #[test]
    fn add_continues_after_existing_ids() {
        // Les tâches chargées depuis le fichier peuvent avoir n'importe quels ids.
        let mut list = TaskList::from_tasks(vec![Task::new(7, String::from("ancienne"))]);
        assert_eq!(list.add(String::from("nouvelle")).id, 8);
    }

    #[test]
    fn mark_done_updates_matching_task() {
        let mut list = TaskList::new();
        list.add(String::from("première"));
        list.add(String::from("deuxième"));

        let task = list.mark_done(2).expect("la tâche 2 existe");
        assert_eq!(task.id, 2);
        assert!(task.done);
        // La première tâche n'a pas bougé.
        assert!(!list.tasks()[0].done);
    }

    #[test]
    fn mark_done_returns_none_for_unknown_id() {
        let mut list = TaskList::new();
        list.add(String::from("première"));
        assert_eq!(list.mark_done(99), None);
    }

    #[test]
    fn remove_returns_the_removed_task() {
        let mut list = TaskList::new();
        list.add(String::from("première"));
        list.add(String::from("deuxième"));

        let removed = list.remove(1).expect("la tâche 1 existe");
        assert_eq!(removed.description, "première");
        assert_eq!(list.tasks().len(), 1);
        assert_eq!(list.tasks()[0].id, 2);
    }

    #[test]
    fn remove_returns_none_for_unknown_id() {
        let mut list = TaskList::new();
        assert_eq!(list.remove(1), None);
    }
}
