# Projet du module 3 — minigrep, un grep professionnel

Tu vas construire un **mini-grep** : un outil en ligne de commande qui cherche un motif dans un ou plusieurs fichiers, ligne par ligne, avec les options classiques de `grep` (insensible à la casse, numéros de ligne, sélection inversée, comptage) et un fichier de configuration TOML optionnel.

Tu codes dans le dossier **`projet/`**, déjà initialisé. Contrairement aux modules précédents, le projet est organisé en **bibliothèque + binaire** (COURS.md §9.3) :

```
projet/
├── src/
│   ├── lib.rs      ← assemble les modules + ré-exports (FOURNI)
│   ├── error.rs    ← le type d'erreur GrepError, via thiserror (FOURNI, à LIRE)
│   ├── search.rs   ← le cœur de la recherche          → étapes 1 à 3
│   ├── config.rs   ← le fichier de config TOML (serde) → étape 4
│   └── main.rs     ← l'enrobage CLI (clap + anyhow)    → étapes 5 à 7
└── tests/
    ├── search_in_files.rs   ← tests d'intégration (FOURNIS)
    └── fixtures/            ← un poème + un fichier de config d'exemple
```

Les **tests sont déjà écrits** (unitaires dans `src/`, d'intégration dans `tests/`). Ton travail consiste à remplir les `todo!()` jusqu'à ce que tout passe au vert. Lis `COURS.md` d'abord ; chaque étape te renvoie aux sections utiles. Bloqué malgré les indices ? La `solution/` est là — mais bats-toi un peu avant, c'est là qu'on apprend.

> Convention des étapes : **Objectif** (quoi faire) → **Pourquoi** (ce que ça t'apprend) → **Indices** → **Point de contrôle** (comment vérifier).

---

## Étape 0 — Prendre ses marques

**Objectif.** Explorer le squelette et vérifier qu'il compile.

**Pourquoi.** Comprendre l'architecture lib + bin et savoir lancer les tests avant d'écrire une ligne, c'est le socle de tout le reste.

**Indices.**
- Ouvre `Cargo.toml` : repère les cinq dépendances (`clap`, `serde`, `toml`, `anyhow`, `thiserror`) et la feature `derive` activée sur `clap` et `serde` (COURS.md §5.2).
- Lis `src/lib.rs` : il déclare les modules (`pub mod ...`) et **ré-exporte** les types utiles, pour qu'on écrive `minigrep::search` au lieu de `minigrep::search::search`.
- Lis `src/error.rs` en entier : il est **entièrement fourni**, tu n'as rien à y changer. C'est ton premier contact concret avec `thiserror` (COURS.md §8.2). Repère les trois variantes, l'attribut `#[error("...")]` (qui génère `Display`) et l'attribut `#[from]` (qui génère `impl From` pour que `?` convertisse les erreurs tout seul).
- Parcours les `tests/` et les fixtures : tu connais déjà le résultat attendu, c'est ta feuille de route.

**Point de contrôle.**
```bash
cd projet
cargo test --no-run   # tout compile (Cargo télécharge les crates au 1er appel)
cargo test            # de nombreux tests échouent (panics sur todo!()) : normal,
                      # c'est ton tableau de bord. On va les faire passer un par un.
```

---

## Étape 1 — La recherche de base

**Objectif.** Dans `src/search.rs`, commence à remplir la fonction `search`. Pour l'instant : si `pattern` est vide, retourne `Err(GrepError::EmptyPattern)` ; sinon, retourne la liste des lignes (sous forme de `MatchedLine`) qui **contiennent** le motif, en recherche **sensible à la casse**, avec leur numéro de ligne à partir de 1.

**Pourquoi.** C'est le cœur du programme et l'occasion d'écrire ton premier vrai **pipeline d'itérateurs** (COURS.md §4) : `lines()` → `enumerate()` → `filter()` → `map()` → `collect()`. Objectif de style : **pas de boucle `for` avec `push`**.

**Indices.**
- Supprime d'abord la ligne `let _ = (pattern, content, options);` qui n'est là que pour éviter les avertissements.
- Le motif vide se traite en premier, avec un *early return* : `if pattern.is_empty() { return Err(GrepError::EmptyPattern); }`.
- Le pipeline :
  ```rust
  let results = content
      .lines()                 // itérateur paresseux sur les &str de chaque ligne
      .enumerate()             // ajoute l'index : (0, "ligne 1"), (1, "ligne 2"), ...
      .filter(|(_, line)| line.contains(pattern))
      .map(|(index, line)| MatchedLine {
          number: index + 1,   // les humains comptent à partir de 1
          text: line.to_string(),
      })
      .collect();              // matérialise en Vec<MatchedLine>
  Ok(results)
  ```
- `line.contains(pattern)` accepte un `&str` directement (COURS.md §1.4 parle de `From`/`Into` ; ici c'est `Pattern`, mais l'intuition « ça accepte plusieurs types » est la même).
- `enumerate` renvoie un tuple `(index, line)` : dans la closure, on le déstructure avec `|(index, line)|`. Le `_` dans `|(_, line)|` veut dire « cet index, on l'ignore ici ».

**Point de contrôle.**
```bash
cargo test --lib search   # cible les tests de search.rs
```
Doivent déjà passer : `finds_a_simple_match`, `is_case_sensitive_by_default`, `line_numbers_start_at_one`, `empty_pattern_is_an_error`, `no_match_returns_an_empty_vec`. Restent rouges `ignore_case_finds_more_lines` et `invert_selects_non_matching_lines` : les étapes 2 et 3.

---

## Étape 2 — L'option « insensible à la casse »

**Objectif.** Quand `options.ignore_case` vaut `true`, la recherche ne doit plus tenir compte de la casse (`rUsT` trouve `Rust` **et** `Trust`).

**Pourquoi.** Première option qui modifie le comportement du pipeline, et un petit réflexe de performance : on normalise le motif **une seule fois**, avant la boucle, pas à chaque ligne.

**Indices.**
- Avant le pipeline, calcule un « motif normalisé » une fois pour toutes :
  ```rust
  let needle = if options.ignore_case {
      pattern.to_lowercase()
  } else {
      pattern.to_string()
  };
  ```
  Note que `needle` est une `String` dans les deux cas (les deux bras d'un `if`/`else` doivent produire le **même type**).
- Dans le `filter`, compare en minuscules quand l'option est active :
  ```rust
  let is_match = if options.ignore_case {
      line.to_lowercase().contains(&needle)
  } else {
      line.contains(needle.as_str())
  };
  ```
- Attention au piège de l'emprunt : `contains` veut un `&str`. Sur une `String`, écris `contains(&needle)` ou `contains(needle.as_str())`.

**Point de contrôle.**
```bash
cargo test --lib search
```
`ignore_case_finds_more_lines` passe au vert. Reste `invert_selects_non_matching_lines`.

---

## Étape 3 — L'option « sélection inversée »

**Objectif.** Quand `options.invert` vaut `true`, garder les lignes qui **ne contiennent PAS** le motif (le `-v` de `grep`).

**Pourquoi.** Tu vas voir qu'une seule comparaison booléenne suffit pour basculer entre « garde ce qui matche » et « garde ce qui ne matche pas » — élégance des booléens.

**Indices.**
- Tu as déjà calculé `is_match` à l'étape 2. La condition finale du `filter` devient :
  ```rust
  is_match != options.invert
  ```
- Pourquoi `!=` ? Sur des booléens, `!=` joue le rôle d'un **XOR** :
  - `invert == false` → on garde les lignes où `is_match == true` ;
  - `invert == true` → on garde les lignes où `is_match == false`.
- Évite la tentation du `if options.invert { ... } else { ... }` autour de tout le filtre : une seule expression est plus claire et plus juste.

**Point de contrôle.**
```bash
cargo test --lib            # les 7 tests unitaires de search.rs passent
cargo test --test search_in_files   # les 5 tests de recherche du fichier passent aussi
```
Le cœur de la recherche est terminé. Restent rouges les tests de `config.rs` (étape 4) et un test d'intégration sur la config.

---

## Étape 4 — Le fichier de configuration (serde + TOML)

**Objectif.** Dans `src/config.rs` : (a) rendre la struct `FileConfig` désérialisable depuis du TOML ; (b) écrire `parse` (une ligne) ; (c) écrire `load_if_exists`.

**Pourquoi.** C'est l'usage type de **serde** (COURS.md §6) : une struct qui dérive `Deserialize` devient lisible depuis n'importe quel format, ici le TOML d'un fichier `.minigrep.toml`. Tu rencontres aussi le motif « parsing pur séparé de la lecture disque », qui rend `parse` testable sans toucher au système de fichiers.

**Indices.**
- **(a) Les derives et attributs.** Il manque `Deserialize` dans le `#[derive(...)]`, et deux réglages serde. Vise exactement :
  ```rust
  #[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
  #[serde(default, deny_unknown_fields)]
  pub struct FileConfig { ... }
  ```
  - `#[serde(default)]` : un champ absent du fichier prend sa valeur `Default` (ici `false`) → un fichier partiel, voire vide, reste valide.
  - `#[serde(deny_unknown_fields)]` : une clé inconnue (faute de frappe) devient une **erreur** au lieu d'être ignorée silencieusement.
  - N'oublie pas l'import `use serde::Deserialize;` en haut du fichier.
- **(b) `parse`.** Une seule ligne grâce au `#[from]` de `error.rs` qui convertit `toml::de::Error` → `GrepError` automatiquement :
  ```rust
  pub fn parse(input: &str) -> Result<Self, GrepError> {
      Ok(toml::from_str(input)?)
  }
  ```
- **(c) `load_if_exists`.** Si le fichier n'existe pas → `Ok(None)` (la config est optionnelle, ce n'est pas une erreur). Sinon, lis-le et parse-le :
  ```rust
  pub fn load_if_exists(path: &Path) -> Result<Option<Self>, GrepError> {
      if !path.exists() {
          return Ok(None);
      }
      let raw = fs::read_to_string(path)?; // io::Error → GrepError::Io via #[from]
      Ok(Some(Self::parse(&raw)?))
  }
  ```
  Ajoute l'import `use std::fs;` (le `use std::path::Path;` est déjà là).

**Point de contrôle.**
```bash
cargo test           # TOUS les tests (lib + intégration) doivent passer désormais
```
Tu dois voir les 12 tests unitaires (7 dans `search`, 5 dans `config`) et les 7 tests d'intégration au vert. La bibliothèque est complète et testée. Reste le binaire.

---

## Étape 5 — La ligne de commande avec clap

**Objectif.** Dans `src/main.rs`, déclare la struct `Cli` avec `#[derive(Parser)]` (clap, mode derive) : un motif positionnel, une liste de fichiers, et les drapeaux `-i`, `-n`, `-v`, `-c` plus l'option `--config`.

**Pourquoi.** Tu découvres **clap derive** (COURS.md §7) : tu décris l'interface par une struct, clap génère le parsing, le `--help`, le `--version` et les messages d'erreur. La documentation `///` *est* l'interface.

**Indices.**
- Les imports utiles en haut du fichier :
  ```rust
  use std::fs;
  use std::path::{Path, PathBuf};
  use anyhow::{Context, Result};
  use clap::Parser;
  use minigrep::{FileConfig, SearchOptions, search};
  ```
- La struct (les commentaires `///` deviennent les textes d'aide) :
  ```rust
  /// Search for a pattern in one or more files.
  #[derive(Debug, Parser)]
  #[command(name = "minigrep", version, about)]
  struct Cli {
      /// The pattern to search for
      pattern: String,                 // positionnel obligatoire

      /// The files to search in (at least one)
      #[arg(required = true)]
      files: Vec<PathBuf>,             // un Vec = plusieurs valeurs

      /// Ignore case when matching
      #[arg(short = 'i', long)]
      ignore_case: bool,               // drapeau : présent = true

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
      config: Option<PathBuf>,         // Option<T> = optionnel
  }
  ```
- Rappel des correspondances type ↔ comportement (COURS.md §7) : un `String`/`PathBuf` nu est un positionnel obligatoire, un `bool` un drapeau, un `Vec<T>` accepte plusieurs valeurs, un `Option<T>` est optionnel.
- Déclare aussi une constante en haut : `const DEFAULT_CONFIG_PATH: &str = ".minigrep.toml";` (servira à l'étape 7).
- Dans `main`, remplace le `todo!()` par `let cli = Cli::parse();` pour commencer.

**Point de contrôle.**
```bash
cargo run -- --help      # clap affiche une aide complète générée pour toi
cargo run -- --version   # affiche le numéro de version
```
Le `--help` doit lister le motif, les fichiers et toutes les options : c'est gratuit, clap l'a généré à partir de ta struct.

---

## Étape 6 — Lire les fichiers et afficher les résultats

**Objectif.** Boucler sur `cli.files`, lire chaque fichier, appeler `minigrep::search`, et afficher les lignes retenues. Gère aussi `--count` (n'afficher que le nombre) et `-n` (préfixer par le numéro de ligne).

**Pourquoi.** C'est l'enrobage qui transforme la bibliothèque en outil utilisable, et ton premier usage d'**anyhow** (COURS.md §8.3) : `?` propage les erreurs, `.with_context(...)` ajoute un message utile (« quel fichier pose problème ? »).

**Indices.**
- Construis les options de recherche à partir des drapeaux CLI (la fusion avec la config vient à l'étape 7 — pour l'instant, branche-toi directement sur `cli`) :
  ```rust
  let options = SearchOptions {
      ignore_case: cli.ignore_case,
      invert: cli.invert,
  };
  let show_numbers = cli.line_numbers;
  let show_file_names = cli.files.len() > 1; // comme grep : préfixe si plusieurs fichiers
  ```
- La boucle de lecture, avec contexte d'erreur :
  ```rust
  for path in &cli.files {
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
  ```
- Écris les deux petites fonctions d'affichage, au format de `grep` (`fichier:numéro:texte`) :
  ```rust
  fn print_count(path: &Path, count: usize, show_file_names: bool) {
      if show_file_names {
          println!("{}:{count}", path.display());
      } else {
          println!("{count}");
      }
  }

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
  ```
- Pourquoi `.with_context(...)` et pas `?` seul ? Sans contexte, l'utilisateur lirait juste « No such file or directory ». Avec, il sait **quel** fichier (COURS.md §8.3).

**Point de contrôle.**
```bash
cargo run -- nobody tests/fixtures/poem.txt          # 2 lignes affichées
cargo run -- nobody tests/fixtures/poem.txt -n       # préfixées par 1: et 2:
cargo run -- o tests/fixtures/poem.txt -v            # les lignes SANS « o »
cargo run -- us tests/fixtures/poem.txt -c           # 2
cargo run -- nobody tests/fixtures/inexistant.txt    # erreur claire : « cannot read file ... »
```

---

## Étape 7 — Charger la configuration et la fusionner

**Objectif.** Avant la boucle, charger l'éventuel fichier de configuration (via `--config <chemin>` ou `.minigrep.toml` par défaut) et fusionner ses valeurs avec les drapeaux de la ligne de commande.

**Pourquoi.** Tu boucles la boucle : la lib (serde/`FileConfig`) rencontre le binaire (anyhow). Tu vois aussi `.with_context` fonctionner sur un `Option` (un `None` devient une erreur avec message).

**Indices.**
- Écris une fonction `load_config` qui distingue deux cas :
  ```rust
  fn load_config(cli: &Cli) -> Result<FileConfig> {
      match &cli.config {
          // --config explicite : le fichier DOIT exister et être valide.
          Some(path) => FileConfig::load_if_exists(path)
              .with_context(|| format!("invalid configuration file `{}`", path.display()))?
              // with_context marche aussi sur Option : None → erreur.
              .with_context(|| format!("configuration file not found: `{}`", path.display())),
          // Sinon : on lit .minigrep.toml s'il existe, sinon valeurs par défaut.
          None => Ok(FileConfig::load_if_exists(Path::new(DEFAULT_CONFIG_PATH))
              .context("invalid `.minigrep.toml` in the current directory")?
              .unwrap_or_default()),
      }
  }
  ```
- Dans `main`, appelle-la et fusionne. Règle : le fichier fournit des valeurs par défaut, un drapeau CLI peut toujours **activer** une option (`||` suffit, un booléen CLI absent vaut `false`) :
  ```rust
  let file_config = load_config(&cli)?;
  let options = SearchOptions {
      ignore_case: cli.ignore_case || file_config.ignore_case,
      invert: cli.invert,
  };
  let show_numbers = cli.line_numbers || file_config.line_numbers;
  ```
- `unwrap_or_default()` : si `load_if_exists` rend `Ok(None)` (pas de fichier), on retombe sur `FileConfig::default()` (COURS.md §1.4) — élégant.

**Point de contrôle.**
```bash
# La fixture config.toml active ignore_case et line_numbers :
cargo run -- WHO tests/fixtures/poem.txt --config tests/fixtures/config.toml
# → trouve « Who » (insensible à la casse) ET préfixe par le numéro de ligne,
#   alors qu'aucun drapeau -i / -n n'a été passé.

cargo run -- WHO tests/fixtures/poem.txt              # sans config : ne trouve rien (casse)
cargo run -- x tests/fixtures/poem.txt --config tests/fixtures/inexistant.toml
# → erreur : « configuration file not found: ... »
```

---

## Étape 8 — Vérification finale du module

**Objectif.** Tout doit compiler sans avertissement, et tous les tests doivent passer.

**Point de contrôle.**
```bash
cargo test           # 12 tests unitaires + 7 d'intégration + 1 doc-test, tous verts
cargo run -- nobody tests/fixtures/poem.txt -n   # comportement attendu
cargo fmt            # formate ton code
cargo clippy         # zéro avertissement ? tu codes déjà idiomatique
cargo doc --open     # bonus : ta lib a une vraie doc, générée comme sur docs.rs
cargo clean          # libère le disque (target/ pèse vite des centaines de Mo)
```

Module terminé. Tu as écrit un outil CLI complet, organisé en lib + bin, testé à deux niveaux, qui s'appuie sur cinq crates de l'écosystème. Va consolider avec les **DEFIS.md** — puis rendez-vous au module 4 pour la concurrence.
