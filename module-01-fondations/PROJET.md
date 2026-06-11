# Projet du module 1 — Le nombre mystère & le convertisseur d'unités

Deux programmes dans un seul projet Cargo :

1. **Partie 1 (étapes 1 à 7)** — le **jeu du nombre mystère** (`src/main.rs`) : le programme tire un nombre entre 1 et 100, tu le devines, il répond « trop petit » ou « trop grand ».
2. **Partie 2 (étapes 8 à 10)** — un **convertisseur d'unités** (`src/bin/converter.rs`) : températures et distances, construit autour de fonctions pures testées.

Tu codes dans le dossier **`projet/`**, déjà initialisé. Lis le `COURS.md` d'abord ; chaque étape ci-dessous te renvoie aux sections utiles. Bloqué malgré les indices ? La `solution/` est là — mais bats-toi un peu avant, c'est là qu'on apprend.

> Convention des étapes : **Objectif** (quoi faire) → **Pourquoi** (ce que ça t'apprend) → **Indices** → **Point de contrôle** (comment vérifier).

---

## Partie 1 — Le jeu du nombre mystère

### Étape 1 — Prendre ses marques

**Objectif.** Explorer le squelette et vérifier qu'il compile.

**Pourquoi.** Savoir lancer `cargo run` et `cargo test`, et comprendre la structure d'un projet Cargo, c'est le b.a.-ba de tout ce qui suit. Note la présence de **deux binaires** : `src/main.rs` (le jeu, lancé par `cargo run`) et `src/bin/converter.rs` (le convertisseur, lancé par `cargo run --bin converter`).

**Indices.**
- Ouvre `Cargo.toml` : repère la dépendance `rand` et la clé `default-run`.
- Ouvre `src/main.rs` : les TODO numérotés correspondent aux étapes de ce document.

**Point de contrôle.**
```bash
cd projet
cargo run            # compile (Cargo télécharge rand), puis s'arrête sur :
                     # « not yet implemented: suis les étapes 2 à 7 de PROJET.md »
cargo test --no-run  # tout compile, y compris les tests du convertisseur
```
Le panic sur `todo!()` est normal : c'est ton point de départ.

---

### Étape 2 — Générer le nombre secret

**Objectif.** Dans `main`, remplace le début du `todo!()` : génère un entier aléatoire entre 1 et 100 inclus, stocké dans `secret`.

**Pourquoi.** Première utilisation d'une crate externe (`rand`) et des ranges. C'est exactement comme importer un paquet npm — sauf que Cargo a déjà tout géré.

**Indices.**
- L'import `use rand::Rng;` est déjà là : sans lui, la méthode `random_range` n'existe pas (c'est une méthode de *trait* — notion approfondie au module 3).
- `rand::rng().random_range(1..=100)` — note le range **inclusif** `..=` (cours § 6).
- Annote le type : `let secret: u32 = ...;` (cours § 4 : l'inférence a besoin d'aide ici).
- Pour vérifier visuellement, affiche temporairement le secret : `println!("(triche : {secret})");` — tu retireras cette ligne à l'étape 6.

**Point de contrôle.** `cargo run` affiche un nombre différent à chaque lancement (puis panique sur le `todo!()` restant — normal).

---

### Étape 3 — Lire la proposition du joueur

**Objectif.** Lis une ligne au clavier dans une `String`, affiche-la telle quelle.

**Pourquoi.** C'est ta première rencontre *concrète* avec l'ownership : `read_line` a besoin d'écrire dans **ta** chaîne, donc tu la lui prêtes avec `&mut` (cours § 8.4 et § 9).

**Indices.**
- `let mut input = String::new();` — pourquoi `mut` ? Parce que `read_line` va la modifier.
- `io::stdin().read_line(&mut input).expect("échec de la lecture");`
- `read_line` retourne un `Result` : `.expect(...)` suffit pour l'instant (cours § 9).
- Essaie d'écrire `read_line(input)` ou `read_line(&input)` pour voir : lis l'erreur du compilateur, elle te dira exactement ce qu'elle attend. Casser pour comprendre, c'est gratuit en Rust.

**Point de contrôle.** `cargo run`, tape `coucou` : le programme affiche ce que tu as tapé (avec un saut de ligne en trop — le `\n` que `trim()` éliminera à l'étape suivante).

---

### Étape 4 — Convertir le texte en nombre

**Objectif.** Transforme `input` en `u32` nommé `guess`, avec un `match` sur le résultat de `parse`. En cas d'échec, affiche un message et quitte (provisoirement) avec `return;`.

**Pourquoi.** Le duo `trim` + `parse` + `match` sur `Result` est LE motif de validation d'entrée en Rust — pas d'exception, pas de `NaN` : l'échec est une valeur qu'on traite (cours § 10, usage 2).

**Indices.**
- `input.trim().parse()` — sans `trim()`, le `\n` final fait échouer **tous** les parses. Piège classique !
- L'annotation guide l'inférence : `let guess: u32 = match input.trim().parse() { ... };`
- Bras `Ok(number) => number` : le `match` est une expression, ce bras *produit* la valeur de `guess`.
- Bras `Err(_) => { ... }` : message + `return;` pour l'instant (ça deviendra `continue;` à l'étape 6).

**Point de contrôle.** `cargo run` : `42` → continue ; `abc` → message d'erreur propre, fin du programme. Bonus : sais-tu prédire ce que donne ` 42 ` (avec espaces) ?

---

### Étape 5 — Comparer avec `match` sur `Ordering`

**Objectif.** Compare `guess` au `secret` et affiche « Trop petit ! », « Trop grand ! » ou « Gagné ! ».

**Pourquoi.** Le `match` exhaustif sur un enum est le réflexe Rust par excellence : trois variantes, trois bras, et le compilateur refuse que tu en oublies une (cours § 10, usage 1).

**Indices.**
- `use std::cmp::Ordering;` est déjà importé.
- `match guess.cmp(&secret) { ... }` — pourquoi `&secret` ? Un emprunt partagé suffit pour *lire* (cours § 8.4).
- Les trois motifs : `Ordering::Less`, `Ordering::Greater`, `Ordering::Equal`.
- Expérience : commente le bras `Equal` et lance `cargo build`. Lis l'erreur `non-exhaustive patterns`. Décommente.

**Point de contrôle.** `cargo run` avec la ligne de triche de l'étape 2 : propose un nombre plus petit, plus grand, puis le bon — les trois messages s'affichent correctement (une seule proposition par lancement pour l'instant).

---

### Étape 6 — Boucler jusqu'à la victoire

**Objectif.** Enveloppe les étapes 3-4-5 dans un `loop`. Sur `Equal` : affiche le nombre d'essais et `break`. Dans le bras `Err` : remplace `return;` par `continue;`. Compte les essais dans une variable `mut`.

**Pourquoi.** `loop`/`break`/`continue` + une variable mutable : tu combines tout ce que tu as vu, et tu obtiens un programme complet (cours § 3 et § 6).

**Indices.**
- Déclare `let mut attempts: u32 = 0;` **avant** le `loop` (à l'intérieur, elle serait remise à zéro à chaque tour…).
- Incrémente `attempts += 1;` **après** le parse réussi : une entrée invalide ne devrait pas compter comme un essai.
- La `String` d'entrée, elle, doit être créée **dans** la boucle : une `String` neuve à chaque tour (souviens-toi : `read_line` *ajoute* à la chaîne, il n'écrase pas).
- Retire la ligne de triche de l'étape 2 : le jeu devient jouable !

**Point de contrôle.** `cargo run` : joue une vraie partie en dichotomie (50, puis 25 ou 75…). Tu dois gagner en 7 essais maximum, et le programme annonce ton score.

---

### Étape 7 — Blinder l'entrée

**Objectif.** Deux finitions : (1) vérifie que toute entrée invalide (`abc`, ligne vide, `3.14`, `-5`) affiche un message clair et redonne la main sans compter d'essai ; (2) gère la fin de l'entrée (Ctrl-D) proprement.

**Pourquoi.** Un programme robuste ne panique jamais sur une entrée utilisateur. Et le cas Ctrl-D est sournois : sans traitement, ton `continue` boucle à l'infini sur une entrée terminée.

**Indices.**
- `read_line` retourne `Ok(n)` où `n` est le **nombre d'octets lus** : capture-le (`let bytes_read = ...`). S'il vaut `0`, l'entrée est finie → affiche un au revoir et `return;` (ou `break;`).
- Remarque élégante : `-5` et `3.14` sont rejetés *gratuitement* par `parse::<u32>()` — un `u32` ne peut être ni négatif ni décimal. Le bon choix de type a éliminé des cas d'erreur.
- Retire le `#![allow(unused)]` en haut du fichier : plus aucun avertissement ne doit rester.

**Point de contrôle.**
```bash
cargo run    # teste : abc, entrée vide, 3.14, -5, 0, 101, puis gagne
             # teste aussi Ctrl-D : le programme dit au revoir au lieu de boucler
```
La partie 1 est terminée. 🎉

---

## Partie 2 — Le convertisseur d'unités

On change de fichier : tout se passe dans **`src/bin/converter.rs`**. La philosophie change aussi : cette fois, on écrit d'abord des **fonctions pures** (un calcul, rien d'autre), validées par des **tests fournis**, et seulement ensuite l'interface interactive. Sépare le calcul de l'entrée/sortie : c'est ce qui rend le code testable et réutilisable.

### Étape 8 — Les fonctions de conversion (pilotées par les tests)

**Objectif.** Implémente les quatre fonctions : `celsius_to_fahrenheit`, `fahrenheit_to_celsius`, `kilometers_to_miles`, `miles_to_kilometers`. Les tests sont déjà écrits en bas du fichier — ne les modifie pas.

**Pourquoi.** Style « expression finale sans `;` » (cours § 5), pièges des flottants (cours § 4), et ta première boucle rouge → vert : tests qui échouent, code, tests qui passent. C'est le quotidien du développeur Rust.

**Indices.**
- Formules : `F = C × 9/5 + 32` ; `C = (F − 32) × 5/9` ; `miles = km / 1,609344`.
- **Écris `9.0 / 5.0`**, jamais `9 / 5` : entre entiers, `/` tronque (`9 / 5 == 1`) et Rust refuse de mélanger `i32` et `f64` sans conversion explicite.
- La constante `KM_PER_MILE` est fournie : utilise-la dans les deux fonctions de distance (une vraie valeur n'est écrite qu'une fois).
- Lance les tests **avant** de coder pour voir les 9 échecs : c'est ton tableau de bord.

**Point de contrôle.**
```bash
cargo test --bin converter
# avant : 9 failed (panics sur todo!())
# après : test result: ok. 9 passed; 0 failed
```

---

### Étape 9 — Le menu interactif

**Objectif.** Remplace le `todo!()` de `main` par une boucle de menu : afficher les 4 conversions + « q. Quitter », lire le choix, lire la valeur, afficher le résultat, recommencer.

**Pourquoi.** Tu réutilises *tout* le savoir-faire de la partie 1 (boucle, `read_line`, `parse`, `match`) sans guidage ligne à ligne — et tes fonctions pures de l'étape 8 s'emboîtent naturellement.

**Indices.**
- Même squelette de lecture que le jeu ; cette fois `parse()` vise un `f64` (annotation `let value: f64 = ...`).
- Pour brancher sur le choix : un `match` sur `choice.trim()` avec les motifs `"1"`, `"2"`, `"3"`, `"4"`, `"q"` et un bras `_ => ...` pour tout le reste (avec des `&str`, le motif « attrape-tout » `_` est obligatoire — l'exhaustivité, toujours).
- Affichage propre : `println!("{value} °C = {result:.2} °F");` — le `:.2` limite à deux décimales.
- Astuce factorisation : écris une petite fonction `read_line() -> Option<String>` utilisée pour le choix ET la valeur (regarde comment la solution s'y prend si tu sèches — il y est question de rendre la propriété d'une `String` à l'appelant, cours § 8).
- Pense aux mêmes blindages qu'à l'étape 7 (entrée invalide, Ctrl-D), et retire le `#![allow(unused)]` final.

**Point de contrôle.** `cargo run --bin converter` :
```text
1 puis 100   → 100 °C = 212.00 °F
2 puis 98.6  → 98.6 °F = 37.00 °C
3 puis 42.195 → 42.195 km = 26.22 mi
z            → message d'erreur, le menu revient
q            → le programme quitte poliment
```

---

### Étape 10 — Vérification finale du module

**Objectif.** Tout doit compiler sans avertissement, tous les tests doivent passer.

**Point de contrôle.**
```bash
cargo test           # 9 passed
cargo run            # une partie du jeu, victoire annoncée
cargo run --bin converter   # quelques conversions, sortie propre par q
cargo fmt            # formate ton code
cargo clippy         # zéro avertissement ? bravo, tu codes déjà idiomatique
cargo clean          # libère le disque (target/ pèse vite des centaines de Mo)
```

Module terminé ! Va consolider avec les **DEFIS.md** — puis rendez-vous au module 2 pour structurer tout ça avec des structs et des enums.
