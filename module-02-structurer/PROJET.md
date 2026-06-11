# Projet — Gestionnaire de tâches en ligne de commande

Vous allez construire `tasks`, un gestionnaire de tâches utilisable depuis le terminal :

```text
$ tasks add "Acheter du pain"
Ajoutée : [1] Acheter du pain

$ tasks add "Relire le module 2"
Ajoutée : [2] Relire le module 2

$ tasks list
[ ]   1  Acheter du pain
[ ]   2  Relire le module 2

$ tasks done 1
Terminée : [1] Acheter du pain

$ tasks list
[x]   1  Acheter du pain
[ ]   2  Relire le module 2

$ tasks remove 1
Supprimée : [1] Acheter du pain
```

Les tâches survivent entre deux exécutions : elles sont enregistrées dans un fichier `tasks.tsv` (créé dans le répertoire courant), **une tâche par ligne, champs séparés par des tabulations** :

```text
1	todo	Acheter du pain
2	done	Relire le module 2
```

C'est volontairement rustique : pas de `clap`, pas de `serde`, pas de JSON — tout cela arrive au module 3. Lire les arguments avec `std::env::args` et parser un fichier à la main, c'est l'exercice parfait pour `Option`, `Result`, `match` et les modules. Au module 3, vous mesurerez ce que les crates vous font gagner.

## Ce que vous allez pratiquer

- structs et enums pour modéliser (la tâche, les commandes, les erreurs) ;
- `Option`/`Result` et l'opérateur `?` (parsing, fichier absent, id introuvable) ;
- `match`, `if let`, `let else` partout ;
- `Vec` et l'ownership (la liste des tâches) ;
- le découpage en modules : `task.rs`, `storage.rs`, `cli.rs`, `main.rs`.

## Architecture

```text
projet/
├── Cargo.toml
└── src/
    ├── main.rs      ← point d'entrée : orchestre les trois modules
    ├── task.rs      ← le modèle : Task, TaskList, ParseError
    ├── storage.rs   ← la persistance : load, save, StorageError
    └── cli.rs       ← les arguments : Command, parse_args
```

Le squelette fournit **les types et les signatures** (pour que les tests compilent) ; à vous d'écrire **les corps**, marqués `todo!()`. La macro `todo!()` fait compiler n'importe quel corps de fonction en paniquant à l'exécution — c'est pour cela que les tests compilent mais échouent tant que vous n'avez rien codé.

Travaillez dans `projet/`. En cas de blocage, les indices sont là pour ça ; la `solution/` commentée est le dernier recours (ou le moyen de comparer une fois votre version terminée).

---

## Étape 0 — Prendre ses marques

**Objectif** : comprendre le squelette avant d'écrire la moindre ligne.

**Pourquoi** : repérer comment `main.rs` déclare les modules (`mod task;`…) et comment les tests sont organisés (`#[cfg(test)] mod tests` en bas de chaque fichier) vous fera gagner du temps sur tout le reste.

**À faire** :

1. Lisez les quatre fichiers de `src/`. Notez les `pub`, les types de retour, les `TODO`.
2. Vérifiez que le squelette compile :

```bash
cd projet
cargo test --no-run   # compile les tests sans les exécuter
```

3. Lancez les tests pour constater l'état initial :

```bash
cargo test
```

**Point de contrôle** : la compilation réussit ; les tests échouent tous avec `not yet implemented` (les panics des `todo!()`). C'est votre ligne de départ : le projet est fini quand tout est vert.

---

## Étape 1 — Le modèle : `Task::new` et `Task::to_line`

**Objectif** : dans `src/task.rs`, implémenter la fonction associée `new` et la méthode `to_line`, qui sérialise une tâche en une ligne du fichier.

**Pourquoi** : on commence par le cœur du domaine, sans entrée/sortie — facile à tester, aucun piège d'ownership. Et écrire la sérialisation *avant* le parsing fixe le format dans votre tête : `id<TAB>statut<TAB>description`, statut `todo` ou `done`.

**Indices** :

- `new` reçoit un `id` et une `description`, et crée une tâche **non terminée**. Souvenez-vous de `Self` et du raccourci de champ (`description` au lieu de `description: description`).
- Pour `to_line` : `format!` avec `\t` pour les tabulations. Un `if` (qui est une expression !) ou un `match` sur `self.done` donne le statut : `let status = if self.done { "done" } else { "todo" };`.
- Ne mettez **pas** de `\n` final : c'est le rôle de la couche storage.

**Point de contrôle** :

```bash
cargo test to_line
```

Les deux tests `to_line_*` passent.

---

## Étape 2 — Le parsing : `Task::from_line` et `ParseError`

**Objectif** : implémenter `Task::from_line(line: &str) -> Result<Task, ParseError>`, l'opération inverse, ainsi que `ParseError::message`.

**Pourquoi** : c'est le morceau le plus « module 2 » du projet : un `Result` avec un enum d'erreurs maison, du déballage d'`Option`, des conversions. Un fichier modifié à la main (ou corrompu) ne doit jamais faire paniquer le programme : chaque problème devient une variante de `ParseError`.

**Indices** :

- Découpez la ligne avec `line.splitn(3, '\t')` : « au plus 3 morceaux », si bien qu'une description contenant elle-même des tabulations reste entière (un test le vérifie). `splitn` retourne un itérateur ; chaque appel à `.next()` rend une `Option<&str>` — terrain idéal pour `let else` :

```rust
let mut fields = line.splitn(3, '\t');
let Some(id_field) = fields.next() else {
    return Err(ParseError::MissingField);
};
// ... même chose pour le statut et la description
```

- Pour l'id : `id_field.parse::<u64>()` retourne un `Result` — convertissez son erreur en `ParseError::InvalidId` avec un `match` (le `?` ne convient pas ici : les types d'erreur ne correspondent pas, on verra la conversion automatique au module 3).
- Pour le statut : un `match` sur le `&str`, avec les bras `"todo"`, `"done"` et un bras `other => ...` qui capture la valeur fautive dans `ParseError::InvalidStatus`.
- Les champs sont des `&str` empruntés à la ligne ; la `Task` doit posséder sa description : `.to_string()`.
- `message` : un `match` exhaustif sur `self`, un `format!` par variante. Les messages doivent citer la valeur fautive (un test le vérifie).

**Point de contrôle** :

```bash
cargo test from_line
cargo test parse_error
cargo test line_round_trip
```

Tout passe, y compris l'aller-retour `to_line` → `from_line`.

---

## Étape 3 — La collection : `TaskList`

**Objectif** : implémenter `TaskList` : `new`, `from_tasks`, `tasks`, `next_id`, `add`, `mark_done`, `remove`.

**Pourquoi** : c'est l'étape « `Vec` + ownership ». Vous allez itérer en empruntant, modifier un élément à travers `&mut`, et retourner des références sans vous faire gronder par le borrow checker. Si vous croisez `E0502` ou `E0382`, retournez voir la section 6 du cours : c'est exactement ces erreurs-là.

**Indices** :

- `next_id` : le plus grand id existant + 1 (une liste vide donne 1). Une boucle `for task in &self.tasks` et une variable `max` suffisent. Ne supposez pas que la dernière tâche porte le plus grand id.
- `add` : créez la tâche avec `Task::new(self.next_id(), description)`, poussez-la, puis retournez une référence vers elle — `self.tasks.last()` retourne une `Option<&Task>` qu'il faut déballer ; `.expect("...")` est honnête ici, car le `push` vient de garantir qu'un dernier élément existe.
- Attention à l'ordre dans `add` : calculer `next_id()` **après** avoir commencé à construire l'appel à `push` vous mettrait face à un conflit d'emprunts. Calculez l'id d'abord, dans une variable.
- `mark_done` : parcourez avec `for task in &mut self.tasks`, et dès que l'id correspond, modifiez et faites `return Some(task);`. Après la boucle : `None`.
- `remove` : `Vec` n'a pas de « supprimer par valeur de champ », seulement `remove(index)`. Trouvez d'abord l'**indice** (une boucle `for index in 0..self.tasks.len()`), puis `self.tasks.remove(index)` vous rend la tâche par valeur — parfaite pour la retourner dans un `Some`.

**Point de contrôle** :

```bash
cargo test add_
cargo test mark_done
cargo test remove
```

Tous les tests de `task.rs` sont verts.

---

## Étape 4 — La persistance : `storage.rs`

**Objectif** : implémenter `serialize`, `parse_content`, `load`, `save` et `StorageError::message`.

**Pourquoi** : première rencontre avec les entrées/sorties faillibles. Deux familles d'échec — le disque (`io::Error`) et le contenu (`ParseError`) — réunies dans un seul enum `StorageError` : exactement le genre de modélisation que les types sommes rendent naturelle.

**Indices** :

- `serialize` : une `String` accumulée avec `push_str(&task.to_line())` puis `push('\n')`, pour chaque tâche (itérée par emprunt !).
- `parse_content` : parcourez `content.lines()`, ignorez les lignes vides (`continue`), comptez les numéros de ligne **en partant de 1**, et au premier `Err` de `Task::from_line`, retournez `StorageError::Parse { line_number, error }`. Un fichier à moitié lu ne vaut rien : on s'arrête à la première ligne invalide.
- `load` : `std::fs::read_to_string(path)` retourne `Result<String, io::Error>`. Cas particulier crucial : **un fichier absent n'est pas une erreur**, c'est la première utilisation du programme — retournez `Ok(Vec::new())`. Pour distinguer ce cas, un `match` avec une garde :

```rust
Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
```

  Les autres erreurs d'E/S (permissions…) deviennent `StorageError::Io`. Déléguez ensuite à `parse_content` — c'est précisément pour tester le parsing sans toucher au disque que les deux fonctions sont séparées.
- `save` : `std::fs::write(path, serialize(tasks))`, dont l'erreur éventuelle s'emballe dans `StorageError::Io`.

**Point de contrôle** :

```bash
cargo test --lib 2>/dev/null; cargo test serialize parse_content load save round_trip
```

ou plus simplement `cargo test` : il ne doit rester en échec que les tests de `cli.rs`.

---

## Étape 5 — Les arguments : `cli.rs`

**Objectif** : implémenter `parse_args(args: &[String]) -> Result<Command, String>` (et son auxiliaire `parse_id_argument`).

**Pourquoi** : transformer du texte brut tapé par un humain en un type précis (`Command`), c'est le travail de toute frontière d'un programme. L'enum `Command` garantit qu'au-delà de cette fonction, plus aucune commande invalide ne peut circuler.

**Indices** :

- `args` ne contient **pas** le nom du programme : `main` s'en chargera (étape 6).
- Aucun argument → `Ok(Command::Help)` (afficher l'aide est plus aimable qu'une erreur). `args.first()` retourne une `Option<&String>` : encore un `let else`.
- `match command.as_str() { ... }` pour aiguiller (on matche un `&str`, pas une `String`).
- Pour `add` : la description peut arriver en plusieurs arguments si l'utilisateur a oublié les guillemets (`tasks add acheter du pain`). Recollez-les : `args[1..].join(" ")`. Si le résultat est vide → `Err` avec un message clair.
- Pour `done` et `remove`, le travail est identique : c'est le rôle de `parse_id_argument`. `args.get(1)` (et non `args[1]`, qui paniquerait), puis `parse::<u64>()` traduit en message d'erreur. Dans `parse_args`, appelez-le avec `?` : les deux fonctions retournent une erreur `String`, le `?` propage directement.
- Commande inconnue → `Err` citant la commande reçue.

**Point de contrôle** :

```bash
cargo test
```

**Tous** les tests du projet passent. Le moteur est complet — il ne reste plus qu'à le brancher.

---

## Étape 6 — Le branchement : `main.rs`

**Objectif** : écrire `main` et `run` pour relier les trois modules : lire les arguments, charger le fichier, exécuter la commande, sauvegarder, et soigner les erreurs.

**Pourquoi** : c'est ici qu'on voit la récompense de l'architecture : `main` ne contient aucune logique métier, juste de l'orchestration et de l'affichage. Et c'est votre premier vrai usage de l'opérateur `?` en conditions réelles.

**Indices** :

- Les arguments : `let args: Vec<String> = std::env::args().collect();` — `args[0]` est le chemin du programme, passez `&args[1..]` à `cli::parse_args`.
- Si `parse_args` échoue : message sur **stderr** (`eprintln!`), affichez `cli::USAGE`, puis `std::process::exit(2)`.
- `run(command)` retourne `Result<(), StorageError>`, ce qui autorise `storage::load(...)?` et permet à `main` de centraliser l'affichage d'erreur : `if let Err(error) = run(command) { ... exit(1) }`.
- Dans `run` : chargez, construisez la liste (`TaskList::from_tasks`), puis un grand `match command`. `Help` et `List` n'ont rien à sauvegarder (`return Ok(())` après affichage) ; `Add`, `Done`, `Remove` modifient puis laissent la sauvegarde finale s'exécuter.
- `Done`/`Remove` sur un id inconnu : les méthodes retournent `None` — affichez un message et `exit(1)`, sans sauvegarder.
- Pour l'affichage de `list` : une case `[x]`/`[ ]` selon `task.done`, l'id, la description. `{:>3}` aligne l'id à droite sur 3 caractères. Liste vide : un petit message d'accueil plutôt que rien.

**Point de contrôle** : la démonstration du haut de ce document, en vrai :

```bash
cargo run -- add "Acheter du pain"
cargo run -- add "Relire le module 2"
cargo run -- list
cargo run -- done 1
cargo run -- list
cargo run -- remove 1
cargo run -- list
cat tasks.tsv
```

(le `--` sépare les arguments de Cargo de ceux de votre programme). Vérifiez aussi les chemins d'erreur :

```bash
cargo run -- done 999        # id inconnu → message + code de sortie 1
cargo run -- frobnicate      # commande inconnue → usage + code 2
cargo run -- done abc        # id invalide → message + code 2
echo $?                      # affiche le code de sortie de la dernière commande
```

Et le test ultime du parsing robuste : ouvrez `tasks.tsv`, sabotez une ligne, relancez `tasks list` — vous devez obtenir un message propre citant le numéro de ligne, pas un panic.

---

## Terminé ?

- `cargo test` : tout vert.
- Comparez votre code à `solution/` : les différences sont souvent instructives (et la vôtre a le droit d'être meilleure).
- Supprimez le fichier d'essai : `rm -f tasks.tsv`.
- Place aux `DEFIS.md`.
