# Défis du module 2

Trois extensions **non guidées**, par difficulté croissante. Pas d'étapes détaillées : un objectif, des critères de réussite et quelques pistes. À toi de chercher, de te tromper, de relire le cours et la documentation. La `solution/` ne les couvre pas — c'est voulu.

> Conseil : commence chaque défi par « comment je le testerais ? ». Le projet t'a appris le réflexe : une fonction métier dans `task.rs`/`storage.rs`, un test à côté, et seulement ensuite le branchement dans `cli.rs`/`main.rs`. Ajoute le test **avant** de coder.

> Compatibilité du fichier : tu vas changer le format de `tasks.tsv` (un champ de plus par ligne). Le plus simple est de partir d'un fichier neuf (`rm -f tasks.tsv`). Le plus instructif est de garder la rétrocompatibilité : une ligne sans le nouveau champ doit rester lisible (valeur par défaut). Les pistes t'indiquent comment.

---

## Défi 1 — Des priorités

**Objectif.** Donner à chaque tâche une **priorité** parmi trois niveaux, et l'afficher.

- À l'ajout : `tasks add "Acheter du pain" --high` (ou `--low`) ; sans option, la priorité est **normale**.
- `list` montre la priorité, par exemple avec un préfixe : `[ ] !!  1  Acheter du pain` pour une priorité haute, `!` pour normale, `·` pour basse (libre à toi).
- La priorité est persistée dans `tasks.tsv` et survit aux relances.

**Critères de réussite.**
- Modélise la priorité avec un **enum** (`Priority { High, Normal, Low }`), pas avec un entier ni une `String` qui traîne. Dérive `Debug, Clone, PartialEq`.
- `to_line`/`from_line` gèrent le nouveau champ, et l'aller-retour reste couvert par un test (`line_round_trip`).
- Une priorité inconnue dans le fichier devient une variante de `ParseError` (`InvalidPriority(String)`), avec un message qui cite la valeur fautive — comme `InvalidStatus`.

**Pistes.**
- Donne une méthode à ton enum : `impl Priority { fn as_str(&self) -> &str { match self { ... } } }` pour la sérialisation, et l'inverse côté parsing avec un `match` sur le `&str`.
- Choisis la position du champ dans la ligne. Si tu le mets **en fin** (`id<TAB>statut<TAB>priorité<TAB>description`), méfie-toi : `splitn(3, …)` collerait la description et la priorité. Mets plutôt la priorité **avant** la description et passe à `splitn(4, '\t')`. Réfléchis à ce que ça change pour les descriptions contenant des tabulations.
- Rétrocompatibilité (optionnelle) : si la ligne n'a que 3 champs, considère la priorité comme `Normal` au lieu d'échouer. Un `match` sur le nombre de champs récupérés, ou un `unwrap_or` bien placé.
- Côté CLI : `--high`/`--low` peuvent apparaître n'importe où dans les arguments. Repère-les (`args.iter().any(|a| a == "--high")`), **retire-les** avant de recoller la description, sinon ils finiront dans le texte de la tâche.

---

## Défi 2 — Dates d'échéance et tri

**Objectif.** Ajouter une **date d'échéance** optionnelle, et trier l'affichage par échéance.

- `tasks add "Rendre le rapport" --due 2026-07-15` ; sans `--due`, la tâche n'a pas d'échéance.
- `list` affiche l'échéance quand elle existe, et **trie** les tâches : d'abord celles qui ont une échéance (les plus proches en premier), puis celles qui n'en ont pas.
- Bonus : `tasks list --overdue` ne montre que les tâches dont l'échéance est passée.

**Critères de réussite.**
- L'absence d'échéance est modélisée par `Option<...>` (un champ `due: Option<Date>`), **jamais** par une date « magique » comme `0000-00-00`. C'est le cœur du défi : `Option` au bon endroit.
- Le tri est testé sur une `TaskList` montée à la main, sans toucher au disque (compare l'ordre des ids après tri).
- Une date mal formée dans `tasks.tsv` ou en argument produit une erreur propre (variante de `ParseError`, message citant la valeur).

**Pistes.**
- **Pas de crate de dates** (`chrono` viendra plus tard) : une date est un simple `struct Date { year: u16, month: u8, day: u8 }`. Pour la persistance, sérialise-la en `2026-07-15` et parse-la en découpant sur `'-'` (`split('-')`, trois `parse`). Une validation minimale (mois 1..=12, jour 1..=31) suffit ; la validation calendaire exacte est un sur-défi.
- Pour comparer deux dates, le plus propre est de dériver `PartialOrd, Ord` sur `Date` — **l'ordre des champs compte** : `year` doit être déclaré avant `month`, lui-même avant `day`, car `Ord` dérivé compare champ par champ, dans l'ordre de déclaration. Tu obtiens alors `date_a < date_b` gratuitement.
- Pour trier : copie les tâches dans un `Vec`, puis `sort_by_key` ou `sort_by`. Ranger les `None` après les `Some` : `Option` s'ordonne déjà avec `None < Some(_)`, donc trier par `std::cmp::Reverse(task.due.is_some())` puis par `task.due` fait l'affaire — ou écris ton propre comparateur, c'est formateur.
- `--overdue` a besoin de « la date d'aujourd'hui ». Inutile de la calculer vraiment : prends-la en argument, ou code une `Date` du jour en dur pour le défi. Le sujet ici, c'est `Option` et le tri, pas l'horloge système.

---

## Défi 3 — Recherche et statistiques

**Objectif.** Chercher dans les descriptions, et résumer l'état des tâches.

- `tasks search pain` affiche toutes les tâches dont la description contient « pain » (insensible à la casse).
- `tasks stats` affiche un récapitulatif : nombre total, terminées, restantes, et le **nombre d'occurrences de chaque mot** dans les descriptions (un mini nuage de mots, les plus fréquents en premier).

**Critères de réussite.**
- `search` est une méthode de `TaskList` qui retourne un `Vec<&Task>` (des références, pas des copies — l'occasion de manier l'ownership), testée sans disque.
- Le comptage de mots utilise une **`HashMap<String, u32>`** et son API `entry`/`or_insert`, exactement comme dans le cours.
- `stats` n'affiche rien d'incohérent sur une liste vide (0 partout, pas de panic).

**Pistes.**
- Recherche insensible à la casse : compare `description.to_lowercase()` avec `needle.to_lowercase()`, via `contains`. Attention, `to_lowercase` alloue une nouvelle `String` ; ça reste parfaitement acceptable ici.
- Comptage : pour chaque tâche, `for word in task.description.split_whitespace()`, puis `*counts.entry(word.to_string()).or_insert(0) += 1;`. Pense à normaliser (minuscules, retirer la ponctuation collée) si tu veux un résultat propre — ou laisse-le brut et note la limite.
- Trier la `HashMap` par fréquence : une `HashMap` n'a pas d'ordre. Verse ses paires dans un `Vec<(String, u32)>` (`counts.into_iter().collect()`), puis `sort_by` sur le compte décroissant. Pour n'afficher que le top 5, `.iter().take(5)`.
- Nouvelles variantes de `Command` (`Search(String)`, `Stats`) : ajoute-les à l'enum et le `match` de `main`/`run` **refusera de compiler** tant que tu ne les auras pas traitées. C'est l'exhaustivité du module 2 qui travaille pour toi : laisse le compilateur te guider.

---

Quand tes défis passent `cargo test` (et idéalement `cargo clippy` sans avertissement), tu es prêt pour le **module 3 — CLI professionnel**, où `clap`, `serde` et les itérateurs vont rendre tout ce code… beaucoup plus court.
