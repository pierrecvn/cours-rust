# Défis du module 3

Trois extensions **non guidées** de ton minigrep : pas d'étapes ligne à ligne, pas de squelette — juste un objectif, des critères de réussite et quelques pistes. C'est volontairement plus difficile que le projet : tu vas devoir lire de la documentation de crates sur [docs.rs](https://docs.rs), te tromper, relire le cours. C'est exactement le but.

Travaille sur une **copie** de ta solution (ou une branche git), pour ne pas casser le projet de base. Et garde le réflexe du module : **si une fonction mérite un test, elle va dans la lib** (`src/`), pas dans le `main`. Chaque défi peut s'accompagner d'un test unitaire ou d'intégration.

> Conseil : commence chaque défi par « comment je le testerais ? ». Le pipeline d'itérateurs et l'architecture lib + bin t'ont préparé exactement à ça.

---

## Défi 1 — Recherche récursive dans un dossier

**Objectif.** Ajouter un drapeau `-r` / `--recursive` : quand un des « fichiers » passés est en réalité un **dossier**, minigrep le parcourt récursivement et cherche dans tous les fichiers qu'il contient.

```bash
minigrep -r nobody ./poemes        # cherche « nobody » dans tous les fichiers du dossier
```

**Critères de réussite.**
- Sans `-r`, passer un dossier en argument produit une **erreur claire** (et non un panic).
- Avec `-r`, chaque fichier trouvé est cherché, et ses résultats sont préfixés par son chemin (comme le multi-fichiers de l'étape 6).
- Les sous-dossiers sont explorés en profondeur ; un fichier illisible n'interrompt pas tout le parcours (signale-le sur `stderr`, continue).

**Pistes.**
- Deux approches pour lister les fichiers : la bibliothèque standard `std::fs::read_dir` (récursion à écrire toi-même, retourne des `Result` à chaque entrée), ou la crate **`walkdir`** (`cargo add walkdir`) qui aplatit toute l'arborescence en un seul itérateur — `WalkDir::new(path).into_iter()`. La seconde est l'outil idiomatique ; la première te fait pratiquer la récursion et `read_dir`.
- Écris une fonction de la lib `fn collect_files(root: &Path) -> Result<Vec<PathBuf>, GrepError>` que tu peux tester sur une petite arborescence de fixtures, indépendamment de l'affichage.
- `path.is_dir()` / `path.is_file()` aiguillent le traitement. Pense à filtrer pour ne garder que les fichiers (`entry.file_type().is_file()`).
- Côté erreurs : un nouveau cas mérite peut-être une nouvelle variante dans `GrepError`, ou bien `anyhow` côté binaire suffit — à toi de juger selon qui doit pouvoir réagir à l'erreur.

---

## Défi 2 — Support des expressions régulières

**Objectif.** Ajouter un drapeau `-E` / `--regex` : le motif est alors interprété comme une **expression régulière** au lieu d'une sous-chaîne littérale.

```bash
minigrep -E '^How' tests/fixtures/poem.txt    # les lignes qui COMMENCENT par « How »
minigrep -E 'no(body|one)' fichier.txt
```

**Critères de réussite.**
- Sans `-E`, le comportement actuel (recherche littérale) est strictement inchangé.
- Avec `-E`, le motif est une vraie regex ; une regex invalide produit une **erreur typée et lisible**, pas un panic.
- `-E` se combine avec `-i` (insensible à la casse) et `-v` (inversion).

**Pistes.**
- La crate de référence est **`regex`** (`cargo add regex`) — rapide, sûre, sans backtracking catastrophique. Lis sa page docs.rs : `Regex::new(pattern)?` retourne un `Result`, `re.is_match(line)` teste une ligne.
- Le défi de conception est dans `search` : aujourd'hui elle prend un `&str` littéral. Plusieurs options propres —
  - faire de la « stratégie de match » une closure ou un petit enum (`Matcher::Literal(String)` / `Matcher::Regex(Regex)`) construit **avant** le pipeline, puis appelé dans le `filter` ;
  - ou ajouter un champ `regex: bool` à `SearchOptions` et brancher dedans.
  Choisis-en une et assume-la ; c'est l'occasion de repenser une API (les traits et enums du module 2 et 3 sont là pour ça).
- Pour l'insensibilité à la casse en regex : plutôt que `to_lowercase`, utilise le drapeau intégré `(?i)` ou `RegexBuilder::new(pattern).case_insensitive(true)`.
- Erreurs : ajoute une variante `GrepError::InvalidRegex(#[from] regex::Error)` — tu réutilises exactement le motif `thiserror` + `#[from]` de `error.rs` (COURS.md §8.2).
- Teste une regex valide, une invalide (doit donner `Err`), et la combinaison avec `-i`.

---

## Défi 3 — Sortie colorée

**Objectif.** Mettre en **surbrillance** la portion de chaque ligne qui correspond au motif (comme `grep --color`), et colorer le préfixe (nom de fichier, numéro de ligne) comme le vrai `grep`.

**Critères de réussite.**
- Le motif trouvé apparaît en couleur (rouge gras, par exemple) au sein de chaque ligne affichée ; le reste de la ligne reste normal.
- La couleur est **désactivée automatiquement** quand la sortie n'est pas un terminal (redirection vers un fichier ou un `| autre-commande`) : aucun code d'échappement ne doit polluer un fichier de sortie.
- Un drapeau `--color <auto|always|never>` permet de forcer le comportement (défaut : `auto`).

**Pistes.**
- Crates utiles : **`anstream`** + **`anstyle`** (l'écosystème officiel de clap, gèrent la détection terminal et Windows proprement), ou la plus simple **`owo-colors`** / **`colored`**. `cargo add` puis docs.rs.
- La détection « est-ce un terminal ? » se fait avec `std::io::IsTerminal` (`std::io::stdout().is_terminal()`) — pas besoin de crate pour ça.
- Pour surligner, il te faut la **position** du match dans la ligne, pas juste un booléen. En recherche littérale : `line.find(pattern)` donne l'indice de départ. Pense à découper la ligne en trois morceaux : avant / le match / après. Avec `-i` ou la regex du défi 2, c'est plus subtil (positions sur la version normalisée vs l'originale) : commence par le cas littéral sensible à la casse.
- Le drapeau `--color` est un bel usage d'un **enum + clap** : `#[derive(clap::ValueEnum)]` sur un enum `ColorChoice { Auto, Always, Never }`, et clap valide tout seul que l'utilisateur n'écrit pas `--color bleu`.
- Garde la logique de surlignage dans une fonction pure et testable (`fn highlight(line: &str, pattern: &str) -> String`) : la décision « couleur ou pas » reste dans le binaire, le calcul reste dans la lib.

---

Quand les trois défis tournent sans broncher (`cargo test`, `cargo clippy`), tu maîtrises un outil CLI digne d'être publié sur crates.io. Tu es prêt pour le **module 4 — Concurrence & lifetimes**.
