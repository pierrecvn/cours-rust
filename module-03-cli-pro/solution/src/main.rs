//! # minigrep — binaire
//!
//! Couche « présentation » du projet : parser la ligne de commande (clap),
//! charger l'éventuel fichier de configuration, appeler la bibliothèque,
//! afficher les résultats. Aucune logique de recherche ici.
//!
//! Côté erreurs : le binaire utilise `anyhow`. Toutes les erreurs (les nôtres,
//! celles de la std) convergent vers `anyhow::Error`, enrichies de contexte
//! avec `.with_context(...)`. C'est l'outil du *consommateur* d'erreurs ;
//! la bibliothèque, elle, expose des erreurs typées via `thiserror`.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

use minigrep::{FileConfig, SearchOptions, search};

/// Chemin du fichier de configuration lu par défaut (s'il existe).
const DEFAULT_CONFIG_PATH: &str = ".minigrep.toml";

/// Search for a pattern in one or more files.
///
/// clap (mode derive) génère tout à partir de cette struct : le parsing,
/// `--help`, `--version`, les messages d'erreur. Les commentaires de doc
/// `///` deviennent les textes d'aide affichés à l'utilisateur.
#[derive(Debug, Parser)]
#[command(name = "minigrep", version, about)]
struct Cli {
    /// The pattern to search for
    pattern: String,

    /// The files to search in (at least one)
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Ignore case when matching
    #[arg(short = 'i', long)]
    ignore_case: bool,

    /// Prefix each line with its line number
    #[arg(short = 'n', long)]
    line_numbers: bool,

    /// Select the lines that do NOT match the pattern
    #[arg(short = 'v', long)]
    invert: bool,

    /// Only print the number of matching lines per file
    #[arg(short = 'c', long)]
    count: bool,

    /// Path to a TOML configuration file (default: .minigrep.toml if present)
    #[arg(long)]
    config: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let file_config = load_config(&cli)?;

    // Fusion configuration + ligne de commande : le fichier fournit des
    // valeurs par défaut, un drapeau CLI peut toujours ACTIVER une option.
    // (`||` suffit : un booléen CLI absent vaut false.)
    let options = SearchOptions {
        ignore_case: cli.ignore_case || file_config.ignore_case,
        invert: cli.invert,
    };
    let show_numbers = cli.line_numbers || file_config.line_numbers;

    // Comme grep : avec plusieurs fichiers, chaque ligne est préfixée
    // par le nom du fichier d'où elle vient.
    let show_file_names = cli.files.len() > 1;

    for path in &cli.files {
        // `read_to_string` seul dirait juste « No such file or directory » ;
        // le contexte ajoute QUEL fichier pose problème.
        let content = fs::read_to_string(path)
            .with_context(|| format!("cannot read file `{}`", path.display()))?;

        let results = search(&cli.pattern, &content, &options)?;

        if cli.count {
            print_count(path, results.len(), show_file_names);
        } else {
            for matched in &results {
                print_match(path, matched.number, &matched.text, show_file_names, show_numbers);
            }
        }
    }

    Ok(())
}

/// Détermine la configuration effective.
///
/// - `--config <chemin>` : le fichier DOIT exister et être valide.
/// - Sinon : on lit `.minigrep.toml` s'il est présent, et on retombe sur
///   `FileConfig::default()` dans le cas contraire (`unwrap_or_default`).
fn load_config(cli: &Cli) -> Result<FileConfig> {
    match &cli.config {
        Some(path) => {
            FileConfig::load_if_exists(path)
                .with_context(|| format!("invalid configuration file `{}`", path.display()))?
                // `with_context` fonctionne aussi sur Option : None → erreur.
                .with_context(|| format!("configuration file not found: `{}`", path.display()))
        }
        None => Ok(FileConfig::load_if_exists(Path::new(DEFAULT_CONFIG_PATH))
            .context("invalid `.minigrep.toml` in the current directory")?
            .unwrap_or_default()),
    }
}

/// Affiche le nombre de lignes retenues pour un fichier (option `--count`).
fn print_count(path: &Path, count: usize, show_file_names: bool) {
    if show_file_names {
        println!("{}:{count}", path.display());
    } else {
        println!("{count}");
    }
}

/// Affiche une ligne retenue, avec ses préfixes optionnels,
/// au format de grep : `fichier:numéro:texte`.
fn print_match(path: &Path, number: usize, text: &str, show_file_names: bool, show_numbers: bool) {
    let mut prefix = String::new();
    if show_file_names {
        prefix.push_str(&format!("{}:", path.display()));
    }
    if show_numbers {
        prefix.push_str(&format!("{number}:"));
    }
    println!("{prefix}{text}");
}
