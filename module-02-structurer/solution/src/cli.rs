//! La frontière avec l'utilisateur : transformer les arguments bruts de la
//! ligne de commande en un type précis.
//!
//! Au-delà de `parse_args`, plus aucune commande invalide ne peut circuler
//! dans le programme : le type `Command` les rend irreprésentables.

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
    // Aucun argument -> l'aide, pas une erreur : c'est plus accueillant.
    let Some(command) = args.first() else {
        return Ok(Command::Help);
    };

    // On matche un &str (d'où le .as_str()) : les littéraux "add", "list"…
    // sont des &'static str, pas des String.
    match command.as_str() {
        "add" => {
            // Sans guillemets dans le shell, la description arrive en
            // plusieurs arguments : on les recolle.
            let description = args[1..].join(" ");
            if description.is_empty() {
                Err(String::from(
                    "la commande add attend une description (ex. : tasks add \"Acheter du pain\")",
                ))
            } else {
                Ok(Command::Add(description))
            }
        }
        "list" => Ok(Command::List),
        "done" => {
            // Les deux fonctions retournent une erreur String : `?` propage.
            let id = parse_id_argument(args)?;
            Ok(Command::Done(id))
        }
        "remove" => {
            let id = parse_id_argument(args)?;
            Ok(Command::Remove(id))
        }
        "help" | "--help" | "-h" => Ok(Command::Help),
        other => Err(format!("commande inconnue : « {other} »")),
    }
}

/// Extrait et valide l'identifiant attendu en deuxième position
/// (factorisé : `done` et `remove` font exactement le même travail).
fn parse_id_argument(args: &[String]) -> Result<u64, String> {
    // `get` plutôt que `args[1]` : pas de panique si l'argument manque.
    let Some(value) = args.get(1) else {
        return Err(format!("la commande {} attend un identifiant", args[0]));
    };
    match value.parse::<u64>() {
        Ok(id) => Ok(id),
        Err(_) => Err(format!("identifiant invalide : « {value} »")),
    }
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
