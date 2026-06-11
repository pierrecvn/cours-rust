# Module 1 — Fondations

Bienvenue dans Rust ! Ce module pose les bases du langage : l'outillage, la syntaxe, et surtout **l'ownership**, le concept qui fait de Rust un langage à part. Tout ce qui est présenté ici sert directement dans le projet du module (jeu du nombre mystère + convertisseur d'unités).

Chaque concept suit le même schéma : définition → comparaison avec Java/TypeScript → exemple minimal compilable → où tu t'en serviras dans le projet.

---

## 1. Qu'est-ce que Rust, et pourquoi ?

**Définition.** Rust est un langage **compilé en code machine natif**, **sans garbage collector**, qui garantit la **sûreté mémoire à la compilation**. Il vise les performances du C/C++ avec les garanties de sécurité d'un langage de haut niveau.

**Comparaison Java/TS.** Voici où s'exécute ton code dans chaque monde :

| | Java | TypeScript/Node | Rust |
|---|---|---|---|
| Compilation | vers du bytecode JVM | transpilé en JS, interprété/JIT par V8 | vers du **code machine natif** |
| Exécution | machine virtuelle (JVM) | runtime Node (V8) | **directement par le CPU**, pas de runtime lourd |
| Mémoire | garbage collector | garbage collector | **ownership**, vérifié à la compilation |
| Types | statiques, vérifiés à la compilation | statiques… mais effacés à l'exécution | statiques, vérifiés à la compilation |
| Erreurs mémoire (null, use-after-free, data races) | `NullPointerException` à l'exécution | `undefined is not a function` à l'exécution | **refusées à la compilation** |

Les conséquences pratiques :

- **Pas de GC** : pas de pauses imprévisibles, consommation mémoire minimale et déterministe. La mémoire est libérée à des moments connus, décidés à la compilation (on verra comment dans la section ownership).
- **Un binaire autonome** : `cargo build --release` produit un exécutable unique. Pas de JVM ni de `node_modules` à installer sur la machine cible.
- **Le compilateur est strict… et pédagogue** : beaucoup de bugs qui en Java/TS explosent à l'exécution (ou en production) sont en Rust des erreurs de compilation, avec des messages remarquablement clairs. La règle d'or du débutant : **lis le message d'erreur en entier**, il contient presque toujours la solution.

Le prix à payer : le compilateur refuse du code que tu juges « évidemment correct » tant que tu n'as pas intégré les règles d'ownership. C'est l'objet de la section 8 — le cœur de ce module.

---

## 2. L'outillage : rustup et Cargo

### rustup

**Définition.** `rustup` installe et met à jour les toolchains Rust (compilateur `rustc`, Cargo, documentation). L'équivalent de `sdkman`/`jenv` pour Java ou `nvm` pour Node.

```bash
rustup update          # met à jour Rust
rustc --version        # rustc 1.96.0 — déjà installé sur cette machine
```

### Cargo

**Définition.** Cargo est l'outil unique qui gère **tout** le cycle de vie d'un projet Rust : création, compilation, dépendances, tests, exécution, publication.

**Comparaison Java/TS.** Cargo regroupe à lui seul ce que tu connais sous plusieurs outils : **npm/yarn** (gestion de dépendances et scripts) + **tsc** (compilation) + **Maven/Gradle** (build, tests, packaging, publication). Un seul outil, officiel, identique dans tous les projets Rust du monde.

| Action | Node/TS | Java (Maven) | Rust |
|---|---|---|---|
| Créer un projet | `npm init` | `mvn archetype:generate` | `cargo new mon-projet` |
| Déclarer une dépendance | `package.json` | `pom.xml` | `Cargo.toml` |
| Verrouiller les versions | `package-lock.json` | — | `Cargo.lock` |
| Compiler | `tsc` | `mvn compile` | `cargo build` |
| Exécuter | `node dist/index.js` | `mvn exec:java` | `cargo run` |
| Tester | `npm test` (avec jest…) | `mvn test` (avec JUnit) | `cargo test` (intégré !) |
| Registre public | npmjs.com | Maven Central | **crates.io** |

Les paquets Rust s'appellent des **crates** (caisses). Le fichier `Cargo.toml` est ton `package.json` :

```toml
[package]
name = "mystery-game"
version = "0.1.0"
edition = "2024"        # « édition » du langage : un millésime de syntaxe

[dependencies]
rand = "0.9"            # crate de génération aléatoire, depuis crates.io
```

Commandes à connaître dès maintenant :

```bash
cargo new projet       # crée un projet binaire (avec src/main.rs)
cargo check            # vérifie que ça compile, sans produire d'exécutable (rapide)
cargo build            # compile en mode debug (target/debug/)
cargo run              # compile puis exécute
cargo test             # compile et lance les tests
cargo fmt              # formate le code (l'équivalent de prettier, officiel)
cargo clippy           # lints supplémentaires (l'équivalent d'eslint, officiel)
cargo clean            # supprime target/ (à faire en fin de module, ça pèse lourd !)
```

**Dans le projet** : tu utiliseras `cargo run`, `cargo test --bin converter` et la crate `rand`.

---

## 3. Variables et mutabilité

**Définition.** En Rust, une variable se déclare avec `let` et est **immuable par défaut**. Pour pouvoir la modifier, il faut demander explicitement `let mut`.

```rust
fn main() {
    let x = 5;
    // x = 6;          // ERREUR de compilation : `cannot assign twice to immutable variable`
    let mut y = 5;
    y = 6;             // OK : y est déclaré mutable
    println!("{x} {y}");
}
```

**Comparaison Java/TS.** C'est l'inverse de ce que tu connais :

- En TypeScript, `let` = mutable, `const` = immuable, et il faut *choisir* l'immuabilité.
- En Java, tout est mutable sauf si tu ajoutes `final`.
- En Rust, `let` = **immuable**, et il faut *choisir* la mutabilité avec `mut`.

Autrement dit : le `let` de Rust ressemble au `const` de TS, et `let mut` au `let` de TS. Ce choix de défaut est délibéré — la majorité des variables n'ont pas besoin de changer, et le compilateur te signalera tout `mut` inutile.

### Le shadowing (masquage)

**Définition.** On peut redéclarer une variable avec le même nom : la nouvelle déclaration **masque** l'ancienne. Le type peut même changer.

```rust
fn main() {
    let input = "  42  ";          // &str (du texte)
    let input = input.trim();      // &str, sans les espaces — masque la première
    let input: u32 = input.parse().expect("pas un nombre"); // u32 — masque la deuxième
    println!("{input}");           // 42
}
```

**Comparaison Java/TS.** Interdit dans les deux : en TS, `let input = ...; let input = ...` dans le même bloc est une erreur ; en Java aussi. En Rust c'est **idiomatique**, surtout pour les transformations successives d'une même donnée (« le texte brut » → « le texte nettoyé » → « le nombre ») : pas besoin d'inventer `inputRaw`, `inputTrimmed`, `inputParsed`. À ne pas confondre avec la mutation : chaque `let` crée une **nouvelle** variable.

### Les constantes

```rust
const MAX_ATTEMPTS: u32 = 10;     // type obligatoire, nom en SCREAMING_SNAKE_CASE
```

`const` est une valeur figée à la compilation — l'équivalent du `static final` Java. **Dans le projet** : la constante `KM_PER_MILE` du convertisseur.

---

## 4. Types de base et inférence

**Définition.** Rust est statiquement typé avec **inférence locale** : le compilateur déduit le type quand c'est possible, mais chaque variable a un type unique et définitif.

Les types primitifs essentiels :

| Type Rust | Description | Java | TypeScript |
|---|---|---|---|
| `i8 i16 i32 i64 i128` | entiers signés (le nombre = la taille en bits) | `byte short int long` | `number` (ou `bigint`) |
| `u8 u16 u32 u64 u128` | entiers **non signés** | — (pas d'équivalent !) | — |
| `f32`, `f64` | flottants | `float`, `double` | `number` |
| `bool` | `true` / `false` | `boolean` | `boolean` |
| `char` | un caractère Unicode (4 octets) | `char` (UTF-16…) | — |
| `(i32, bool)` | tuple | — | `[number, boolean]` |
| `[i32; 3]` | tableau de taille **fixe** | `int[]` | — |

À retenir, venant de Java/TS :

- **Pas un `number` fourre-tout** : tu choisis la taille et le signe. Par défaut, un littéral entier est `i32` et un littéral flottant est `f64`. Les non-signés (`u32`…) sont parfaits pour « un nombre d'essais » ou « un nombre entre 1 et 100 » : les valeurs négatives sont impossibles *par construction*.
- **Aucune conversion implicite.** `let x: f64 = 5;` ne compile pas (5 est un entier). `5 / 9` vaut `0` (division entière) ; pour un calcul flottant, écris `5.0 / 9.0`. Java convertit silencieusement `int` → `double` ; Rust, jamais — on convertit explicitement avec `as` ou `From`.
- **L'inférence est puissante mais a parfois besoin d'aide** :

```rust
fn main() {
    let a = 42;                // i32, inféré par défaut
    let b: u32 = 42;           // u32, annoté
    let c = "7".parse::<u32>().unwrap();  // « turbofish » ::<> pour préciser le type
    let d: u32 = "7".parse().unwrap();    // ou annotation sur la variable (plus courant)
    println!("{a} {b} {c} {d}");
}
```

`parse()` peut produire n'importe quel type numérique : il faut dire lequel, par annotation ou turbofish. **Dans le projet** : `let guess: u32 = input.trim().parse()…` — l'annotation `u32` pilote à la fois `parse` et la comparaison avec le secret.

---

## 5. Fonctions, expressions et le `;` qui change tout

**Définition.** Une fonction se déclare avec `fn`, les types des paramètres et du retour sont **obligatoires**. Rust distingue les **instructions** (statements — ne produisent pas de valeur) des **expressions** (produisent une valeur) ; la **dernière expression d'un bloc, sans `;`, est la valeur du bloc**.

```rust
/// F = C × 9/5 + 32
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0    // pas de `;` : c'est la valeur de retour
}

fn main() {
    println!("{}", celsius_to_fahrenheit(100.0)); // 212
}
```

**Comparaison Java/TS.** En Java/TS, `return` est obligatoire pour renvoyer une valeur. En Rust, `return` existe (pour les sorties anticipées) mais le style idiomatique est l'expression finale sans `;`. Et surtout, **les blocs eux-mêmes sont des expressions** :

```rust
fn main() {
    let message = {
        let hour = 14;
        if hour < 12 { "bonjour" } else { "bon après-midi" }  // valeur du bloc
    };
    println!("{message}");
}
```

**Le piège du `;`.** Ajouter un `;` transforme une expression en instruction, dont la valeur devient `()` (le « unit », l'équivalent de `void`). Le bug classique du débutant :

```rust
fn double(x: i32) -> i32 {
    x * 2;    // ← ce `;` de trop fait retourner `()` au lieu de `x * 2`
}
```

```text
error[E0308]: mismatched types
 --> src/main.rs:1:22
  |
1 | fn double(x: i32) -> i32 {
  |    ------            ^^^ expected `i32`, found `()`
  |    |
  |    implicitly returns `()` as its body has no tail or `return` expression
2 |     x * 2;
  |          - help: remove this semicolon to return this value
```

Le compilateur te dit littéralement quoi faire : *remove this semicolon*. **Dans le projet** : toutes les fonctions de conversion sont écrites dans ce style « expression finale ».

---

## 6. Contrôle de flux

### `if` est une expression

```rust
fn main() {
    let score = 75;
    let grade = if score >= 50 { "reçu" } else { "recalé" };  // pas de ternaire ?: — pas besoin !
    println!("{grade}");
}
```

**Comparaison Java/TS.** Là où tu écrirais `score >= 50 ? "reçu" : "recalé"`, Rust utilise `if`/`else` directement, car ils produisent une valeur. Deux différences de syntaxe : pas de parenthèses autour de la condition, mais accolades **obligatoires**. Et la condition doit être un vrai `bool` : `if x { }` avec `x: i32` ne compile pas (pas de notion de truthy/falsy comme en JS).

### Les boucles : `loop`, `while`, `for`

```rust
fn main() {
    // `loop` : boucle infinie assumée, on sort avec `break`.
    let mut count = 0;
    let result = loop {
        count += 1;
        if count == 10 {
            break count * 2;   // break peut renvoyer une valeur !
        }
    };
    println!("{result}");      // 20

    // `while` : comme partout.
    let mut fuel = 3;
    while fuel > 0 {
        fuel -= 1;
    }

    // `for` : itère sur un range (1..=5 : inclusif ; 1..5 : 5 exclu).
    for i in 1..=5 {
        println!("tour {i}");
    }
}
```

**Comparaison Java/TS.** Le `for (let i = 0; i < n; i++)` à trois clauses **n'existe pas** en Rust : on itère toujours sur quelque chose (un range, une collection), comme le `for...of` de TS ou le for-each de Java. `loop` remplace le `while (true)` — et le compilateur sait qu'on n'en sort que par `break`, ce qui permet à `let result = loop { ... }` de compiler. **Dans le projet** : la partie de jeu est un `loop` dont on sort par `break` à la victoire, et `continue` relance le tour en cas d'entrée invalide.

---

## 7. `String` vs `&str`

**Définition.** Rust a deux types principaux pour le texte :

- **`String`** : chaîne **possédée**, modifiable, allouée sur le tas. Tu en es propriétaire, tu peux la faire grandir (`push_str`), elle est libérée quand son propriétaire sort de portée.
- **`&str`** (« string slice ») : une **vue empruntée** sur du texte qui appartient à quelqu'un d'autre — un morceau de `String`, ou un littéral `"bonjour"` gravé dans le binaire.

```rust
fn main() {
    let mut owned = String::from("bonjour");  // String : possédée, mutable
    owned.push_str(" le monde");              // on peut la faire grandir

    let view: &str = &owned[..7];             // &str : une vue sur "bonjour"
    let literal: &str = "salut";              // les littéraux sont des &str

    println!("{owned} / {view} / {literal}");
}
```

**Comparaison Java/TS.** En Java comme en TS, il n'y a qu'un type `string`/`String`, immuable, géré par le GC — tu ne te demandes jamais « qui possède ce texte ». En Rust, la distinction possédé/emprunté est partout (c'est l'ownership appliqué au texte). Première intuition, suffisante pour ce module :

- **Je stocke ou je construis du texte** → `String`.
- **Je ne fais que lire du texte** (paramètre de fonction, par exemple) → `&str`.
- `&une_string` se convertit automatiquement en `&str` ; dans l'autre sens, `"abc".to_string()` ou `String::from("abc")` créent une `String`.

Méthodes que tu utiliseras tout de suite : `input.trim()` (retire espaces et `\n`, retourne un `&str` emprunté à `input`) et `parse()` (convertit un `&str` en nombre). **Dans le projet** : `read_line` remplit une `String` ; `trim` + `parse` la transforment en `u32`.

---

## 8. Ownership & borrowing — LE concept de Rust

Prends ton temps sur cette section : c'est **le** concept sans équivalent en Java ou TypeScript, et tout le reste du langage en découle. C'est aussi ce qui permet à Rust de se passer de garbage collector.

### 8.1 Le problème que ça résout

En Java/TS, le GC scrute la mémoire à l'exécution et libère ce qui n'est plus référencé. Coût : un runtime, des pauses, de la mémoire en plus. En C/C++, c'est au programmeur de libérer — et de ne pas se tromper (use-after-free, double free…).

Rust choisit une troisième voie : des **règles de possession vérifiées à la compilation**. Si ça compile, la mémoire est gérée correctement — sans GC et sans `free` manuel.

### 8.2 Les trois règles de l'ownership

1. Chaque valeur a un unique **propriétaire** (une variable).
2. Il ne peut y avoir qu'**un seul propriétaire à la fois**.
3. Quand le propriétaire sort de portée, la valeur est **libérée** (drop).

```rust
fn main() {
    {
        let s = String::from("éphémère");  // s possède la String
        println!("{s}");
    }   // ← fin de portée : la String est libérée ICI, automatiquement,
        //   à un endroit connu à la compilation. Pas de GC nécessaire.
}
```

### 8.3 Le move : l'affectation déménage

Affecter une `String` à une autre variable ne copie pas : ça **transfère la propriété** (move). L'ancienne variable devient inutilisable.

```rust
fn main() {
    let name = String::from("Ada");
    let other = name;            // la propriété DÉMÉNAGE vers `other`
    println!("{name}");          // ERREUR : `name` ne possède plus rien
}
```

Le message exact du compilateur (apprends à le lire, il dit tout) :

```text
error[E0382]: borrow of moved value: `name`
 --> src/main.rs:4:16
  |
2 |     let name = String::from("Ada");
  |         ---- move occurs because `name` has type `String`, which does not implement the `Copy` trait
3 |     let other = name;
  |                 ---- value moved here
4 |     println!("{name}");
  |                ^^^^ value borrowed here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
3 |     let other = name.clone();
  |                     ++++++++
```

Lecture guidée : le compilateur montre (1) **pourquoi** il y a move (`String` n'est pas `Copy`), (2) **où** la valeur a déménagé (ligne 3), (3) **où** tu essaies de l'utiliser après coup (ligne 4), et (4) propose une solution (`clone()` — copie profonde explicite, si tu acceptes son coût).

**Comparaison Java/TS.** En Java/TS, `let other = name;` copie une *référence* : les deux variables pointent vers le même objet, et le GC s'occupe du reste. En Rust, il ne peut y avoir qu'un propriétaire (règle 2) — sinon, qui libérerait la valeur, et combien de fois ?

Nuance importante : les types simples (`i32`, `u32`, `f64`, `bool`, `char`) sont **`Copy`** — l'affectation les copie bit à bit, pas de move. C'est pour ça que `let a = 5; let b = a; println!("{a}");` compile sans problème. Le move concerne les types qui possèdent des ressources (comme `String`).

Le move s'applique aussi aux **appels de fonction** : passer une `String` en paramètre la déménage dans la fonction. Pour la « prêter » sans la perdre, il faut…

### 8.4 Les références : emprunter au lieu de déménager

**Définition.** Une **référence** (`&valeur`) permet d'accéder à une valeur **sans en prendre la propriété**. On dit qu'on l'**emprunte** (borrow). Deux formes :

- `&T` : référence **partagée** (lecture seule) ;
- `&mut T` : référence **exclusive** (lecture/écriture), qui exige que la variable soit `mut`.

```rust
fn shout(text: &str) -> String {      // emprunte en lecture
    text.to_uppercase()
}

fn add_exclamation(text: &mut String) {  // emprunte en écriture
    text.push('!');
}

fn main() {
    let mut message = String::from("bonjour");
    let loud = shout(&message);          // on PRÊTE message…
    add_exclamation(&mut message);       // …puis on le prête en écriture…
    println!("{message} / {loud}");      // …et on l'utilise encore : il est toujours à nous
}
```

**Comparaison Java/TS.** Toutes les variables d'objet Java/TS sont *déjà* des références — mais sans aucune règle : n'importe qui peut garder une référence pour toujours et muter l'objet n'importe quand (d'où les bugs d'aliasing, et les `ConcurrentModificationException` en Java). En Rust, l'emprunt est **explicite** (`&`, `&mut` visibles dans le code) et **discipliné** par la règle suivante.

### 8.5 La règle d'or : un seul `&mut` OU plusieurs `&`

À tout instant, pour une valeur donnée, tu peux avoir :

> **soit** autant de références partagées `&` que tu veux,
> **soit** UNE SEULE référence exclusive `&mut` —
> **jamais les deux en même temps.**

Intuition : autant de **lecteurs** que tu veux, ou un unique **écrivain** — mais pas d'écrivain pendant qu'on lit. Exactement la sémantique d'un verrou lecteurs/écrivain… vérifiée à la compilation. C'est cette règle qui élimine les data races et les invalidations de références.

Voyons les deux erreurs que tu rencontreras forcément.

**Erreur n° 1 : muter pendant qu'un lecteur existe.**

```rust
fn main() {
    let mut message = String::from("salut");
    let r = &message;        // emprunt partagé (lecteur)
    message.push('!');       // mutation : exige un &mut… conflit !
    println!("{r}");         // le lecteur est encore utilisé ici
}
```

```text
error[E0502]: cannot borrow `message` as mutable because it is also borrowed as immutable
 --> src/main.rs:4:5
  |
3 |     let r = &message;
  |             -------- immutable borrow occurs here
4 |     message.push('!');
  |     ^^^^^^^^^^^^^^^^^ mutable borrow occurs here
5 |     println!("{r}");
  |                - immutable borrow later used here
```

Lecture guidée : trois annotations — où naît l'emprunt partagé (ligne 3), où survient la mutation interdite (ligne 4 : `push` prend `&mut self` en coulisses), et **pourquoi l'emprunt est encore vivant** (ligne 5 : `r` est utilisé après la mutation). Pourquoi est-ce dangereux ? `push` peut réallouer la chaîne ailleurs en mémoire : `r` pointerait alors sur de la mémoire libérée. En Java, le GC masque ce problème ; en C++, c'est un crash ; en Rust, ça ne compile pas. La correction est souvent triviale : déplacer le `println!("{r}")` **avant** le `push`, ou ne créer `r` qu'après.

**Erreur n° 2 : deux écrivains en même temps.**

```rust
fn main() {
    let mut score = String::from("42");
    let a = &mut score;
    let b = &mut score;      // deuxième &mut alors que `a` vit encore
    println!("{a} {b}");
}
```

```text
error[E0499]: cannot borrow `score` as mutable more than once at a time
 --> src/main.rs:4:13
  |
3 |     let a = &mut score;
  |             ---------- first mutable borrow occurs here
4 |     let b = &mut score;
  |             ^^^^^^^^^^ second mutable borrow occurs here
5 |     println!("{a} {b}");
  |                - first borrow later used here
```

Deux `&mut` simultanés = deux écrivains qui peuvent se marcher dessus (et dans un contexte multi-thread, une data race). Refusé.

**La subtilité qui sauve : les emprunts ont une durée de vie.** Un emprunt se termine à sa **dernière utilisation**, pas à la fin du bloc. Ceci compile parfaitement :

```rust
fn main() {
    let mut score = String::from("42");
    let a = &mut score;
    a.push('!');             // dernière utilisation de `a` → l'emprunt se termine
    let b = &mut score;      // OK : plus aucun autre emprunt vivant
    b.push('?');
    println!("{score}");     // 42!?
}
```

Les emprunts **séquentiels** sont libres ; seuls les emprunts **simultanés** sont réglementés. Quand le borrow checker te bloque, demande-toi : « quels emprunts sont encore vivants à cette ligne, et lequel puis-je terminer plus tôt ? » — réordonner deux lignes suffit très souvent.

**Dans le projet** : `io::stdin().read_line(&mut input)` — tu prêtes ta `String` en écriture à `read_line`, qui la remplit et te la rend ; et `guess.cmp(&secret)` — un emprunt partagé suffit pour comparer. Tu utilises déjà l'ownership !

> L'ownership reviendra en profondeur dans les modules suivants (structs au module 2, lifetimes au module 4). Pour l'instant, retiens : **move par défaut, `&` pour lire, `&mut` pour écrire, un écrivain OU des lecteurs.**

---

## 9. Lire au clavier avec `std::io`

**Définition.** Le module `std::io` fournit l'accès à l'entrée/sortie standard. `io::stdin().read_line(&mut buffer)` lit une ligne au clavier et **l'ajoute** dans la `String` prêtée.

```rust
use std::io;

fn main() {
    println!("Comment t'appelles-tu ?");

    let mut input = String::new();             // String vide, possédée, mutable
    io::stdin()
        .read_line(&mut input)                 // on la prête en écriture
        .expect("échec de la lecture");        // read_line retourne un Result

    let name = input.trim();                   // retire le '\n' final
    println!("Bonjour, {name} !");
}
```

**Comparaison Java/TS.** L'équivalent du `Scanner(System.in)` de Java ou de `readline` en Node — en plus direct. Trois points spécifiques à Rust :

1. **`&mut input`** : c'est l'ownership en action — `read_line` a besoin d'écrire dans ta chaîne, donc tu la prêtes en `&mut` (relis la section 8 si ce `&mut` ne te semble pas évident).
2. **`read_line` retourne un `Result<usize, io::Error>`** : la lecture peut échouer, et Rust t'oblige à le prendre en compte — pas d'exception surprise comme l'`IOException` de Java. `.expect("message")` est le traitement minimal : « donne-moi la valeur, et si c'est une erreur, arrête le programme avec ce message ». Le `usize` retourné est le nombre d'octets lus — `0` signifie que l'entrée est terminée (Ctrl-D).
3. **La ligne lue contient le `\n` final** : d'où le `trim()` quasi systématique avant tout `parse()`.

**Dans le projet** : c'est exactement le code de lecture du jeu et du convertisseur.

---

## 10. `match`, version minimale

**Définition.** `match` compare une valeur à une série de motifs (patterns) et exécute le bras correspondant. Il est **exhaustif** (tous les cas doivent être couverts — le compilateur vérifie) et c'est une **expression** (il produit une valeur).

**Comparaison Java/TS.** Bien plus puissant que le `switch` classique : pas de fall-through, pas de `break`, exhaustivité vérifiée, et il déstructure les valeurs. C'est le cousin du `switch` à expressions et du pattern matching de Java 21+ — mais au cœur du langage depuis toujours. Le `match` complet (sur tes propres enums, avec gardes…) arrive au module 2 ; voici les deux usages dont le projet a besoin.

**Usage 1 — comparer deux nombres avec `Ordering`.** La méthode `a.cmp(&b)` retourne un `std::cmp::Ordering`, un enum à trois variantes : `Less`, `Greater`, `Equal`.

```rust
use std::cmp::Ordering;

fn main() {
    let guess = 50;
    let secret = 37;

    match guess.cmp(&secret) {
        Ordering::Less => println!("Trop petit !"),
        Ordering::Greater => println!("Trop grand !"),
        Ordering::Equal => println!("Gagné !"),
    }
}
```

Oublie un bras et le compilateur refuse : `non-exhaustive patterns: Ordering::Equal not covered`. Impossible d'oublier un cas — compare avec un `if/else if` où le cas manquant passe inaperçu.

**Usage 2 — déballer le `Result` de `parse`.** `"42".parse::<u32>()` retourne un `Result` : `Ok(nombre)` si ça a marché, `Err(erreur)` sinon. Pas d'exception à attraper : l'échec est une **valeur**, et `match` permet de la traiter — y compris au milieu d'une boucle :

```rust
fn main() {
    let input = "42x";

    let number: u32 = match input.trim().parse() {
        Ok(n) => n,                       // le match PRODUIT n
        Err(_) => {                       // `_` : peu importe le détail de l'erreur
            println!("« {input} » n'est pas un nombre.");
            return;                       // (dans le jeu : `continue;`)
        }
    };
    println!("{number}");
}
```

Note le motif `Ok(n)` : il **déstructure** le `Result` et capture la valeur qu'il contient. En Java tu aurais un `try/catch (NumberFormatException)` ; en TS un `Number(...)` qui rend `NaN` silencieusement. En Rust, le type `Result` rend l'échec impossible à ignorer. (`Option` et `Result` en profondeur : module 2.)

**Dans le projet** : ces deux `match` sont le cœur du jeu — un pour valider l'entrée, un pour annoncer « trop petit / trop grand / gagné ».

---

## Récapitulatif

| Concept | À retenir | Réflexe venant de Java/TS |
|---|---|---|
| Compilation | binaire natif, pas de GC, pas de runtime | oublier JVM et node_modules |
| Cargo | npm + tsc + Maven en un seul outil | `cargo run`, `cargo test` |
| `let` / `let mut` | immuable par défaut | le `let` Rust ≈ le `const` TS |
| Shadowing | redéclarer pour transformer | idiomatique, pas un bug |
| Types | tailles explicites, zéro conversion implicite | `u32` ≠ `f64`, `5 / 9 == 0` |
| `;` | sans `;` = valeur du bloc ; avec = instruction | plus besoin de `return` final |
| `if` / `loop` | ce sont des expressions | remplace `?:` et `while(true)` |
| Ownership | move par défaut ; `&` lire, `&mut` écrire ; un écrivain OU des lecteurs | lire les erreurs E0382/E0502/E0499 calmement |
| `String` / `&str` | possédé vs emprunté | `String` pour stocker, `&str` pour lire |
| `Result` + `match` | l'échec est une valeur, traitée exhaustivement | plus de try/catch, plus de NaN |

Tu as tout ce qu'il faut : ouvre **PROJET.md** et construis ton premier programme Rust.
