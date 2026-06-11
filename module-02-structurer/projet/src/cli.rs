//! La frontière avec l'utilisateur : transformer les arguments bruts de la
//! ligne de commande en un type précis.
//!
//! Au-delà de `parse_args`, plus aucune commande invalide ne peut circuler
//! dans le programme : le type `Command` les rend irreprésentables.
//!
//! Suis l'étape 5 de PROJET.md.

/// Les commandes que le programme sait exécuter. Un type somme : chaque
/// variante transporte exactement les données dont elle a besoin.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// `tasks add <description>`
    Add(String),
    /// `tasks list`
    List,
    /// `tasks done <id>`
    Done(u64),
    /// `tasks remove <id>`
    Remove(u64),
    /// `tasks help` (ou aucun argument)
    Help,
}

/// Le texte d'aide, affiché par `help` et en cas d'erreur d'arguments.
pub const USAGE: &str = "\
Usage : tasks <commande> [arguments]

Commandes :
  add <description>   Ajouter une tâche
  list                Afficher toutes les tâches
  done <id>           Marquer une tâche comme terminée
  remove <id>         Supprimer une tâche
  help                Afficher cette aide";

/// Parse les arguments (SANS le nom du programme, retiré par l'appelant).
///
/// L'erreur est une simple `String` : à cette frontière, la seule chose à
/// faire d'une erreur est de l'afficher.
pub fn parse_args(args: &[String]) -> Result<Command, String> {
    // TODO (étape 5) :
    //   1. Aucun argument -> `Ok(Command::Help)` (plus accueillant qu'une erreur).
    //      `args.first()` rend une `Option<&String>` : encore un `let else`.
    //   2. `match command.as_str() { ... }` pour aiguiller (on matche un `&str`).
    //      - `"add"` : recolle la description avec `args[1..].join(" ")` ; si
    //        elle est vide -> `Err(...)` ; sinon -> `Ok(Command::Add(description))`.
    //      - `"list"` : `Ok(Command::List)`.
    //      - `"done"` : `let id = parse_id_argument(args)?;` puis `Ok(Command::Done(id))`.
    //      - `"remove"` : idem avec `Command::Remove`.
    //      - `"help" | "--help" | "-h"` : `Ok(Command::Help)`.
    //      - `other` : `Err(...)` citant la commande inconnue.
    todo!("étape 5 : parser les arguments en Command")
}

/// Extrait et valide l'identifiant attendu en deuxième position
/// (factorisé : `done` et `remove` font exactement le même travail).
fn parse_id_argument(args: &[String]) -> Result<u64, String> {
    // TODO (étape 5) :
    //   - `args.get(1)` (et non `args[1]`, qui paniquerait) rend une
    //     `Option<&String>` : un `let else` qui retourne une erreur claire si
    //     l'argument manque (cite `args[0]`, le nom de la commande) ;
    //   - `value.parse::<u64>()` traduit en `Ok(id)` ou en message d'erreur.
    todo!("étape 5 : valider l'identifiant en deuxième position")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Petit utilitaire : des &str de test vers le Vec<String> de la vraie vie.
    fn args(values: &[&str]) -> Vec<String> {
        let mut result = Vec::new();
        for value in values {
            result.push(value.to_string());
        }
        result
    }

    #[test]
    fn no_argument_means_help() {
        assert_eq!(parse_args(&[]), Ok(Command::Help));
    }

    #[test]
    fn parses_list() {
        assert_eq!(parse_args(&args(&["list"])), Ok(Command::List));
    }

    #[test]
    fn parses_add_with_description() {
        assert_eq!(
            parse_args(&args(&["add", "acheter du pain"])),
            Ok(Command::Add(String::from("acheter du pain")))
        );
    }

    #[test]
    fn parses_add_with_multiple_words() {
        // Sans guillemets dans le shell, la description arrive éclatée.
        assert_eq!(
            parse_args(&args(&["add", "acheter", "du", "pain"])),
            Ok(Command::Add(String::from("acheter du pain")))
        );
    }

    #[test]
    fn rejects_add_without_description() {
        assert!(parse_args(&args(&["add"])).is_err());
    }

    #[test]
    fn parses_done_with_id() {
        assert_eq!(parse_args(&args(&["done", "3"])), Ok(Command::Done(3)));
    }

    #[test]
    fn rejects_done_without_id() {
        assert!(parse_args(&args(&["done"])).is_err());
    }

    #[test]
    fn rejects_done_with_invalid_id() {
        assert!(parse_args(&args(&["done", "abc"])).is_err());
    }

    #[test]
    fn parses_remove_with_id() {
        assert_eq!(parse_args(&args(&["remove", "3"])), Ok(Command::Remove(3)));
    }

    #[test]
    fn parses_help() {
        assert_eq!(parse_args(&args(&["help"])), Ok(Command::Help));
    }

    #[test]
    fn rejects_unknown_command_with_its_name() {
        match parse_args(&args(&["frobnicate"])) {
            Err(message) => assert!(message.contains("frobnicate")),
            Ok(command) => panic!("attendu une erreur, obtenu {command:?}"),
        }
    }
}
