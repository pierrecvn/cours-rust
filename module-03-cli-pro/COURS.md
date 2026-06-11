# Module 3 — CLI professionnel : traits, génériques, itérateurs et l'écosystème

Dans les modules 1 et 2, vous avez écrit du Rust « à la main » : parsing maison, erreurs bricolées, tout dans `main.rs`. Ce module vous fait passer du Rust scolaire au Rust professionnel : vous allez comprendre les **traits** (le mécanisme d'abstraction central du langage), écrire des **génériques**, manipuler des **closures** et des **itérateurs**, puis vous appuyer sur l'écosystème — **clap**, **serde**, **anyhow**, **thiserror** — comme le fait n'importe quelle équipe Rust en production. Enfin, vous apprendrez à **tester** sérieusement, en organisant votre projet en bibliothèque + binaire.

Projet du module : un **mini-grep** digne de ce nom (options, fichier de configuration, erreurs propres, tests).

Sommaire :

1. [Les traits — des contrats de comportement](#1-les-traits--des-contrats-de-comportement)
2. [Les génériques — du code pour tous les types](#2-les-génériques--du-code-pour-tous-les-types)
3. [Les closures — des fonctions qui capturent leur environnement](#3-les-closures--des-fonctions-qui-capturent-leur-environnement)
4. [Les itérateurs — des pipelines paresseux et gratuits](#4-les-itérateurs--des-pipelines-paresseux-et-gratuits)
5. [L'écosystème des crates](#5-lécosystème-des-crates)
6. [serde — la (dé)sérialisation universelle](#6-serde--la-désérialisation-universelle)
7. [clap — la ligne de commande en mode derive](#7-clap--la-ligne-de-commande-en-mode-derive)
8. [Gestion d'erreurs professionnelle — thiserror et anyhow](#8-gestion-derreurs-professionnelle--thiserror-et-anyhow)
9. [Les tests — unitaires, d'intégration, et l'architecture lib + bin](#9-les-tests--unitaires-dintégration-et-larchitecture-lib--bin)
10. [Récapitulatif — où chaque concept sert dans le projet](#10-récapitulatif--où-chaque-concept-sert-dans-le-projet)

---

## 1. Les traits — des contrats de comportement

### 1.1 Définition

Un **trait** déclare un ensemble de méthodes qu'un type peut promettre de fournir. C'est le mécanisme d'abstraction central de Rust : là où Java raisonne en hiérarchies de classes, Rust raisonne en **capacités**. « Ce type sait s'afficher » (`Display`), « ce type sait se cloner » (`Clone`), « ce type sait se comparer » (`PartialEq`).

```rust
/// Tout type qui sait se décrire en une phrase.
trait Describe {
    fn describe(&self) -> String;
}

struct Task {
    title: String,
    done: bool,
}

// L'implémentation est un bloc SÉPARÉ de la définition de la struct.
impl Describe for Task {
    fn describe(&self) -> String {
        let status = if self.done { "done" } else { "pending" };
        format!("[{status}] {}", self.title)
    }
}

fn main() {
    let task = Task { title: String::from("learn traits"), done: false };
    println!("{}", task.describe()); // [pending] learn traits
}
```

### 1.2 Comparaison Java / TypeScript

À première vue, un trait ressemble à une `interface` :

```java
// Java
interface Describable {
    String describe();
}
class Task implements Describable {
    public String describe() { return "..."; } // DANS la classe
}
```

```typescript
// TypeScript
interface Describable {
    describe(): string;
}
class Task implements Describable {
    describe(): string { return "..."; } // DANS la classe
}
```

Mais trois différences changent tout :

| | Java / TS | Rust |
|---|---|---|
| Où vit l'implémentation ? | Dans la classe, au moment où on l'écrit | Dans un bloc `impl Trait for Type` **séparé**, écrit où et quand on veut |
| Implémenter pour un type existant ? | Impossible (on ne rouvre pas `String` pour lui ajouter une interface) | **Oui** : on peut implémenter *son* trait pour `i32`, `String`, `Vec<T>`… |
| Héritage ? | Hiérarchies de classes (`extends`), interfaces par-dessus | **Pas d'héritage de classes.** Composition de structs + traits, point final |

La deuxième différence est la plus déroutante — et la plus puissante. En Rust, ceci est parfaitement légal :

```rust
trait Describe {
    fn describe(&self) -> String;
}

// On enrichit un type de la bibliothèque standard avec NOTRE trait !
impl Describe for i32 {
    fn describe(&self) -> String {
        format!("the integer {self}")
    }
}

fn main() {
    println!("{}", 42.describe()); // the integer 42
}
```

Une seule limite, la **règle de l'orphelin** : pour écrire `impl Trait for Type`, le trait *ou* le type doit appartenir à votre crate. Vous ne pouvez pas implémenter `Display` (std) pour `Vec<u8>` (std) — sinon deux crates pourraient fournir des implémentations contradictoires.

Enfin, l'absence d'héritage n'est pas un manque : `Task` ne peut pas « hériter » de `Item`, mais elle peut *contenir* un `Item` (composition) et implémenter les mêmes traits que lui. En pratique, on ne s'en plaint jamais — et on évite toute la fragilité des hiérarchies profondes que vous avez peut-être déjà subie en Java.

### 1.3 Méthodes par défaut

Comme les méthodes `default` des interfaces Java 8+, un trait peut fournir une implémentation par défaut, que les types peuvent garder ou remplacer :

```rust
trait Describe {
    fn describe(&self) -> String;

    // Méthode par défaut : elle peut appeler les méthodes abstraites du trait.
    fn describe_loudly(&self) -> String {
        format!("{}!!!", self.describe().to_uppercase())
    }
}

struct Cat;

impl Describe for Cat {
    fn describe(&self) -> String {
        String::from("a cat")
    }
    // describe_loudly est héritée du trait : "A CAT!!!"
}
```

### 1.4 Les traits incontournables de la bibliothèque standard

#### `Display` — l'affichage pour les humains

`Display` est ce que `{}` utilise dans `println!`. C'est l'équivalent du `toString()` de Java — sauf qu'il n'existe **pas** d'implémentation par défaut : Rust refuse d'afficher un type qui n'a pas explicitement dit comment.

```rust
use std::fmt;

struct Temperature {
    celsius: f64,
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1} °C", self.celsius)
    }
}

fn main() {
    let t = Temperature { celsius: 21.537 };
    println!("{t}");              // 21.5 °C
    let s: String = t.to_string(); // offert : Display fournit to_string()
    assert_eq!(s, "21.5 °C");
}
```

> **Erreur classique du compilateur — E0277.** Si vous écrivez `println!("{}", task)` sans avoir implémenté `Display` :
>
> ```text
> error[E0277]: `Task` doesn't implement `std::fmt::Display`
>   = help: the trait `std::fmt::Display` is not implemented for `Task`
>   = note: in format strings you may be able to use `{:?}` (or {:#?} for pretty-print) instead
> ```
>
> Le compilateur vous dit tout : soit vous implémentez `Display` (affichage destiné aux utilisateurs), soit vous utilisez `{:?}` avec `#[derive(Debug)]` (affichage destiné aux développeurs). `Debug` se dérive, `Display` s'écrit à la main — c'est voulu : un affichage « public » mérite réflexion.

#### `From` / `Into` — les conversions

`From<A> for B` dit « on sait construire un `B` à partir d'un `A` ». Implémentez `From`, et Rust vous offre `Into` dans l'autre sens :

```rust
struct Meters(f64);
struct Kilometers(f64);

impl From<Kilometers> for Meters {
    fn from(km: Kilometers) -> Self {
        Meters(km.0 * 1000.0)
    }
}

fn main() {
    let m = Meters::from(Kilometers(3.2));
    let m2: Meters = Kilometers(3.2).into(); // offert par le même impl
    assert_eq!(m.0, 3200.0);
    assert_eq!(m2.0, 3200.0);
}
```

Pourquoi c'est important : l'opérateur `?` (module 2) utilise `From` pour convertir automatiquement les erreurs. Quand `thiserror` génère `impl From<std::io::Error> for GrepError` (§8), c'est exactement ce qui permet d'écrire `let raw = fs::read_to_string(path)?;` dans une fonction qui retourne `Result<_, GrepError>`.

#### `Default` — la valeur par défaut

```rust
#[derive(Debug, Default)] // Default se dérive si tous les champs ont un Default
struct SearchOptions {
    ignore_case: bool, // false par défaut
    invert: bool,      // false par défaut
}

fn main() {
    let opts = SearchOptions::default();
    // Mise à jour partielle : tout par défaut SAUF ignore_case.
    let opts2 = SearchOptions { ignore_case: true, ..SearchOptions::default() };
    println!("{opts:?} / {opts2:?}");
}
```

C'est l'équivalent discipliné des paramètres par défaut de TS ou des *builders* Java. Vous le verrez partout — y compris dans `#[serde(default)]` (§6).

### 1.5 `derive`, la boîte noire ouverte

Depuis le module 2, vous écrivez `#[derive(Debug, Clone)]` sans trop savoir ce qui se passe. Maintenant vous savez : **`derive` écrit un bloc `impl` à votre place**. `#[derive(Debug)]` génère `impl fmt::Debug for Task { ... }` ; `#[derive(Clone)]` génère `impl Clone for Task { ... }` qui clone champ par champ. Rien de magique : du code généré à la compilation, que vous auriez pu écrire vous-même. `serde`, `clap` et `thiserror` (§6, §7, §8) poussent simplement cette idée plus loin avec leurs propres derives.

**Dans le projet** : `SearchOptions` dérive `Default`, `GrepError` implémente `Display` et `std::error::Error` via `thiserror`, et le `#[from]` de `thiserror` génère des `impl From`.

---

## 2. Les génériques — du code pour tous les types

### 2.1 Fonctions génériques et bounds

Vous connaissez déjà des types génériques sans le savoir : `Vec<T>`, `Option<T>`, `Result<T, E>`, `HashMap<K, V>`. Écrivons maintenant notre propre fonction générique :

```rust
// Ne compile PAS encore — voir l'erreur plus bas.
fn largest<T>(items: &[T]) -> &T {
    let mut largest = &items[0];
    for item in items {
        if item > largest { // ⚠️ comparer des T quelconques ?
            largest = item;
        }
    }
    largest
}
```

> **Erreur classique — E0369.**
>
> ```text
> error[E0369]: binary operation `>` cannot be applied to type `&T`
>   help: consider restricting type parameter `T` with trait `PartialOrd`
>      |
>    1 | fn largest<T: std::cmp::PartialOrd>(items: &[T]) -> &T {
> ```
>
> Rust refuse de supposer quoi que ce soit sur `T`. Java vous laisserait appeler `.equals()` sur n'importe quoi (tout descend d'`Object`) ; TypeScript vérifierait peu de choses. Rust exige que vous déclariez les capacités requises : c'est un **bound** (contrainte).

```rust
fn largest<T: PartialOrd>(items: &[T]) -> &T {
    let mut largest = &items[0];
    for item in items {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn main() {
    assert_eq!(largest(&[3, 7, 2]), &7);          // T = i32
    assert_eq!(largest(&["a", "c", "b"]), &"c");  // T = &str
}
```

Plusieurs contraintes se cumulent avec `+`, et la clause `where` aère les signatures chargées :

```rust
use std::fmt::Display;

// Version inline...
fn print_all<T: Display + Clone>(items: &[T]) { /* ... */ }

// ... et version `where`, identique mais plus lisible quand ça s'allonge :
fn print_all_where<T>(items: &[T])
where
    T: Display + Clone,
{
    for item in items {
        println!("- {item}");
    }
}
```

Les structs aussi sont génériques, avec des `impl` éventuellement conditionnels :

```rust
struct Pair<T> {
    first: T,
    second: T,
}

// Ces méthodes n'existent QUE si T sait se comparer et s'afficher.
impl<T: PartialOrd + std::fmt::Display> Pair<T> {
    fn print_largest(&self) {
        if self.first > self.second {
            println!("{}", self.first);
        } else {
            println!("{}", self.second);
        }
    }
}
```

### 2.2 Monomorphisation : la grande différence avec Java

En Java, les génériques fonctionnent par **effacement de type** (*type erasure*) : à l'exécution, `List<String>` et `List<Integer>` sont la même classe `List`, qui manipule des `Object` avec des *casts* cachés (et du *boxing* pour les primitives — d'où `Integer` au lieu d'`int`). En TypeScript, c'est encore plus radical : les types disparaissent entièrement à la compilation, le JavaScript produit n'en sait rien.

Rust fait l'inverse : la **monomorphisation**. Pour chaque type concret utilisé, le compilateur génère une version spécialisée de la fonction :

```rust
fn double<T: std::ops::Add<Output = T> + Copy>(x: T) -> T { x + x }

fn main() {
    double(21);   // le compilateur génère un double_i32
    double(1.5);  // ... et un double_f64
}
```

Conséquences :

- **Aucun coût à l'exécution** : pas de cast, pas de boxing, pas de dispatch dynamique. Un appel générique est aussi rapide qu'un appel écrit pour le type concret, et peut être *inliné*.
- En contrepartie : un binaire un peu plus gros et une compilation un peu plus longue (chaque version est compilée).

### 2.3 Aperçu : `impl Trait` et `dyn Trait`

Deux écritures que vous croiserez dans la documentation des crates — retenez l'intuition, on les approfondira au module 4 :

```rust
use std::fmt::Display;

// `impl Trait` : « un type concret, choisi à la COMPILATION, qui implémente
// Display ». Dispatch statique, monomorphisé. C'est un raccourci de générique.
fn make_greeting() -> impl Display {
    format!("hello")
}

// `dyn Trait` : « n'importe quel type implémentant Display, choisi à
// l'EXÉCUTION ». Dispatch dynamique via une vtable (comme TOUS les appels de
// méthode Java !), nécessite une indirection (Box, &).
fn pick(flag: bool) -> Box<dyn Display> {
    if flag { Box::new(42) } else { Box::new("quarante-deux") } // 2 types différents !
}
```

Règle pratique : `impl Trait`/génériques par défaut (rapide, vérifié statiquement) ; `dyn Trait` quand on a *besoin* de mélanger des types différents dans une même valeur (une `Vec<Box<dyn Shape>>`, par exemple). Java fait du `dyn` en permanence sans vous demander votre avis ; Rust vous laisse choisir.

**Dans le projet** : vous consommerez surtout des génériques (chaque adaptateur d'itérateur est générique, `collect` aussi), et les bounds expliqueront les messages d'erreur que vous rencontrerez.

---

## 3. Les closures — des fonctions qui capturent leur environnement

### 3.1 Syntaxe

Une **closure** est une fonction anonyme qui peut **capturer des variables** de la portée où elle est définie :

```rust
fn main() {
    let factor = 3;
    let multiply = |x| x * factor; // capture `factor`
    //             └─ types inférés : |x: i32| -> i32

    assert_eq!(multiply(7), 21);
}
```

Côté syntaxe, vous êtes en terrain connu :

```java
// Java                                 // bloc si plusieurs instructions
Function<Integer, Integer> multiply = x -> x * factor;
```

```typescript
// TypeScript
const multiply = (x: number) => x * factor;
```

### 3.2 La capture : là où Rust diverge

En TypeScript, une arrow function capture tout par référence, sans poser de questions — quitte à créer des bugs de partage d'état. En Java, un lambda ne peut capturer que des variables *effectively final* (le compilateur copie la valeur). En Rust, l'ownership s'applique **aussi aux closures** : chaque variable capturée l'est de la façon la moins contraignante possible —

1. par référence partagée `&T` si la closure ne fait que lire ;
2. par référence mutable `&mut T` si elle modifie ;
3. par valeur (move) si elle a besoin de posséder.

Et le borrow checker surveille tout ça comme d'habitude :

```rust
fn main() {
    let mut count = 0;
    let mut increment = || count += 1; // capture &mut count

    increment();
    increment();
    // println!("{count}"); ← ICI, ok : l'emprunt mutable est terminé.
    assert_eq!(count, 2);
}
```

> **Erreur classique — E0502.** Si on essaie de lire `count` *entre* deux appels, pendant que la closure détient encore l'emprunt mutable :
>
> ```rust
> let mut count = 0;
> let mut increment = || count += 1;
> println!("{count}"); // ⚠️ lecture pendant l'emprunt mutable
> increment();
> ```
>
> ```text
> error[E0502]: cannot borrow `count` as immutable because it is also borrowed as mutable
>    | let mut increment = || count += 1;
>    |                     -- ----- first borrow occurs due to use of `count` in closure
>    | println!("{count}");
>    |            ^^^^^ immutable borrow occurs here
>    | increment();
>    | --------- mutable borrow later used here
> ```
>
> La règle du module 1 (un seul emprunt mutable à la fois) s'applique : la closure *est* un emprunteur comme un autre. Solution : réordonner, ou réduire la durée de vie de la closure.

Le mot-clé `move` force la capture **par valeur** (la closure devient propriétaire). Indispensable quand la closure doit survivre à la portée courante (threads au module 4, handlers async au module 5) :

```rust
fn make_counter() -> impl FnMut() -> u32 {
    let mut count = 0;
    move || { // sans `move`, count mourrait à la fin de make_counter → refus
        count += 1;
        count
    }
}

fn main() {
    let mut counter = make_counter();
    assert_eq!(counter(), 1);
    assert_eq!(counter(), 2);
}
```

### 3.3 `Fn`, `FnMut`, `FnOnce`

Chaque closure implémente automatiquement un ou plusieurs de ces trois traits, selon ce qu'elle fait de ses captures :

| Trait | La closure… | Appelable | Exemple |
|---|---|---|---|
| `FnOnce` | consomme ses captures (move + utilisation destructrice) | **une seule fois** | `move \|\| drop(data)` |
| `FnMut` | modifie ses captures | plusieurs fois, `mut` requis | `\|\| count += 1` |
| `Fn` | lit seulement (ou ne capture rien) | plusieurs fois, librement | `\|x\| x * factor` |

Hiérarchie : toute `Fn` est aussi `FnMut`, toute `FnMut` est aussi `FnOnce`. Quand vous écrivez une fonction qui *reçoit* une closure, demandez le trait le moins exigeant qui vous suffit :

```rust
// `Fn` : on promet de ne faire qu'appeler f en lecture, autant de fois qu'on veut.
fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))
}

fn main() {
    assert_eq!(apply_twice(|x| x + 10, 1), 21);
}
```

C'est l'équivalent des interfaces fonctionnelles Java (`Function<T,R>`, `Supplier<T>`, `Consumer<T>`…), mais en trois traits universels — et le compilateur choisit le bon tout seul quand vous écrivez la closure.

**Dans le projet** : vous n'écrirez pas de fonction prenant une closure, mais vous en *passerez* à `filter` et `map` à chaque ligne du pipeline de recherche. Comprendre la capture vous évitera les deux erreurs classiques ci-dessus.

---

## 4. Les itérateurs — des pipelines paresseux et gratuits

### 4.1 Le trait `Iterator`

Tout repose sur un seul trait, élégamment minimal :

```rust
trait Iterator {
    type Item;                              // type associé : ce qu'on produit
    fn next(&mut self) -> Option<Self::Item>; // Some(valeur) ou None à la fin
}
```

Les ~75 autres méthodes (`map`, `filter`, `sum`…) sont des **méthodes par défaut** (§1.3) construites sur `next`. Implémentez `next`, et tout l'arsenal est offert.

### 4.2 `iter()`, `iter_mut()`, `into_iter()` : la question de l'ownership

Trois façons d'itérer une collection, et c'est encore une histoire d'ownership :

```rust
fn main() {
    let names = vec![String::from("Ada"), String::from("Grace")];

    // iter() → emprunts &String : la collection reste utilisable après.
    for name in names.iter() {
        println!("{name}");
    }

    // iter_mut() → emprunts &mut String : modification en place.
    let mut scores = vec![1, 2, 3];
    for score in scores.iter_mut() {
        *score *= 10;
    }
    assert_eq!(scores, vec![10, 20, 30]);

    // into_iter() → valeurs String : CONSOMME la collection.
    for name in names.into_iter() {
        let owned: String = name; // on possède chaque élément
        drop(owned);
    }
    // println!("{names:?}"); ← E0382 : `names` a été déplacée (moved)
}
```

La boucle `for` est du sucre : `for x in &v` appelle `v.iter()`, `for x in &mut v` appelle `v.iter_mut()`, `for x in v` appelle `v.into_iter()`.

### 4.3 Adaptateurs et consommateurs

Deux familles de méthodes :

- les **adaptateurs** transforment un itérateur en un autre itérateur, sans rien exécuter : `map`, `filter`, `enumerate`, `take`, `skip`, `rev`, `zip`, `chain`… ;
- les **consommateurs** tirent réellement les valeurs et produisent un résultat : `collect`, `sum`, `count`, `fold`, `any`, `all`, `find`, `for_each`, `max`…

```rust
fn main() {
    let words = ["rust", "go", "typescript", "java"];

    let long_words: Vec<String> = words
        .iter()
        .filter(|w| w.len() > 3)     // adaptateur : garde rust, typescript, java
        .map(|w| w.to_uppercase())   // adaptateur : RUST, TYPESCRIPT, JAVA
        .collect();                  // consommateur : matérialise en Vec

    assert_eq!(long_words, vec!["RUST", "TYPESCRIPT", "JAVA"]);

    // fold = le `reduce` que vous connaissez :
    let total: usize = words.iter().fold(0, |acc, w| acc + w.len());
    assert_eq!(total, 24);

    // enumerate : ajoute l'index — fini les compteurs manuels.
    for (i, w) in words.iter().enumerate() {
        println!("{i}: {w}");
    }
}
```

Comparaison directe avec ce que vous connaissez :

```java
// Java Streams — même esprit, même paresse :
List<String> longWords = Arrays.stream(words)
    .filter(w -> w.length() > 3)
    .map(String::toUpperCase)
    .collect(Collectors.toList());
```

```typescript
// TypeScript — même esprit, mais EAGER : chaque étape alloue un tableau
// intermédiaire complet avant de passer à la suivante.
const longWords = words.filter(w => w.length > 3).map(w => w.toUpperCase());
```

### 4.4 La paresse

Un adaptateur ne fait **rien** tant qu'un consommateur ne tire pas les valeurs — comme les streams Java, contrairement aux méthodes de `Array` en TS :

```rust
fn main() {
    let v = vec![1, 2, 3];
    v.iter().map(|x| x + 1); // ne calcule RIEN
}
```

```text
warning: unused `Map` that must be used
  = note: iterators are lazy and do nothing unless consumed
```

Le compilateur vous prévient (merci `#[must_use]` — encore un attribut !). La paresse n'est pas un piège, c'est une force : `lines().filter(...).take(5)` ne lira que ce qu'il faut pour produire 5 résultats, sans jamais matérialiser le reste.

> **Erreur classique — E0282 et le turbofish.** `collect` est générique sur sa cible : il peut produire un `Vec`, un `String`, un `HashMap`, même un `Result` ! Il faut donc lui dire quoi produire :
>
> ```rust
> let doubled = vec![1, 2].iter().map(|x| x * 2).collect();
> ```
>
> ```text
> error[E0282]: type annotations needed
>   = help: consider giving `doubled` an explicit type
> ```
>
> Deux remèdes équivalents : annoter la variable, ou le « turbofish » `::<>` :
>
> ```rust
> let a: Vec<i32> = vec![1, 2].iter().map(|x| x * 2).collect();
> let b = vec![1, 2].iter().map(|x| x * 2).collect::<Vec<i32>>();
> ```

### 4.5 Aussi rapide qu'une boucle — vraiment

Le réflexe Java/TS « les streams, c'est élégant mais plus lent qu'une boucle » est **faux en Rust**. Les adaptateurs sont des structs génériques monomorphisées (§2.2), agressivement inlinées : le pipeline complet se compile en l'équivalent de la boucle `for` écrite à la main — parfois mieux, car certains itérateurs éliminent les vérifications de bornes. C'est l'exemple canonique des **abstractions à coût nul** (*zero-cost abstractions*) : le Rust Book lui-même montre, benchmark à l'appui, la version itérateurs d'un décodeur audio battre la boucle manuelle. Écrivez des pipelines sans culpabiliser.

**Dans le projet** : tout le cœur du mini-grep est UN pipeline :

```rust
content
    .lines()        // itérateur paresseux sur les lignes
    .enumerate()    // (index, ligne)
    .filter(|(_, line)| /* la ligne matche-t-elle ? */)
    .map(|(i, line)| /* construit un MatchedLine */)
    .collect()      // Vec<MatchedLine>
```

---

## 5. L'écosystème des crates

### 5.1 crates.io, Cargo.toml et `cargo add`

Une **crate** est l'unité de compilation et de distribution de Rust : l'équivalent d'un artefact Maven ou d'un paquet npm. Le registre central est [crates.io](https://crates.io), et Cargo joue à la fois le rôle de Maven/Gradle et de npm — en mieux intégré.

Ajouter une dépendance :

```bash
cargo add clap --features derive
cargo add serde --features derive
cargo add toml anyhow thiserror
```

… ce qui édite la section `[dependencies]` de `Cargo.toml` :

```toml
[dependencies]
anyhow = "1.0.102"
clap = { version = "4.6.1", features = ["derive"] }
serde = { version = "1.0.228", features = ["derive"] }
thiserror = "2.0.18"
toml = "1.1.2"
```

Points de repère :

- `"4.6.1"` signifie en réalité `^4.6.1` (toute version `4.x` compatible, comme le `^` de npm) — Cargo suit **semver** strictement.
- `Cargo.lock` fige les versions exactes résolues, comme `package-lock.json`. On le commite pour une application, pour des builds reproductibles.
- Contrairement à npm, pas de dossier `node_modules` par projet : les sources des crates sont mises en cache globalement (`~/.cargo`), compilées dans `target/`.

### 5.2 Les features

Une crate peut découper ses fonctionnalités en **features** optionnelles, activées à la demande — du code conditionnel (`#[cfg(feature = "...")]`) qui n'est compilé que si demandé. C'est pour ça qu'on écrit `clap = { features = ["derive"] }` : le mode derive de clap est optionnel, pour que ceux qui ne l'utilisent pas ne paient ni sa compilation ni ses dépendances. Java n'a pas vraiment d'équivalent (au mieux des artefacts séparés) ; c'est proche des *optional peer dependencies* — en vérifié par le compilateur.

### 5.3 docs.rs et comment choisir une crate

Chaque crate publiée a sa documentation générée automatiquement sur [docs.rs](https://docs.rs) — même outil (`rustdoc`) que `cargo doc`, donc présentation uniforme partout, types cliquables, exemples testés. Réflexe à prendre : `docs.rs/<nom-de-la-crate>`.

Pour choisir : téléchargements et dépendants sur crates.io, date de dernière publication, qualité de la doc. Les crates de ce module (`clap`, `serde`, `anyhow`, `thiserror`, `toml`) sont des standards de facto, présents dans une énorme partie des projets Rust.

---

## 6. serde — la (dé)sérialisation universelle

### 6.1 Le principe

**serde** (*ser*ialization / *de*serialization) est LE framework de conversion de données de l'écosystème. Son architecture en deux moitiés est sa grande idée :

- la crate `serde` définit deux traits, `Serialize` et `Deserialize`, et leurs derives ;
- des crates de **format** indépendantes font le pont vers un format concret : `serde_json`, `toml`, `serde_yaml`, etc.

Votre struct dérive les traits une fois ; elle devient lisible/écrivable dans *tous* les formats.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Server {
    host: String,
    port: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TOML → struct (crate `toml`)
    let config: Server = toml::from_str("host = \"localhost\"\nport = 8080")?;
    assert_eq!(config.port, 8080);

    // struct → JSON (crate `serde_json`)
    let json = serde_json::to_string(&config)?;
    assert_eq!(json, r#"{"host":"localhost","port":8080}"#);
    Ok(())
}
```

(Pour compiler cet exemple : `cargo add serde --features derive`, `cargo add toml serde_json`.)

### 6.2 Comparaison Java / TypeScript

- **Java (Jackson/Gson)** : la conversion repose sur la **réflexion à l'exécution** — coûteuse, et les erreurs de structure explosent au runtime. serde génère du code spécialisé **à la compilation** : aucune réflexion (Rust n'en a pas), des performances de référence, et une struct non conforme au format est une erreur *typée* immédiate.
- **TypeScript** : `JSON.parse` retourne `any` et **ne valide rien** — le JSON peut contredire votre interface, vous le découvrirez en production. C'est pour combler ce trou qu'existent zod & co. serde, c'est zod intégré au langage : `toml::from_str::<Server>` vérifie chaque champ, chaque type, et retourne un `Result` détaillé (« missing field `port` at line 1 »).

### 6.3 Les attributs qui font le travail

serde se règle finement par attributs. Ceux du projet :

```rust
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileConfig {
    ignore_case: bool,
    line_numbers: bool,
}
```

- `#[serde(default)]` : tout champ absent du fichier prend sa valeur `Default` (§1.4). Un fichier partiel, voire vide, devient valide — exactement ce qu'on attend d'un fichier de configuration.
- `#[serde(deny_unknown_fields)]` : une clé inconnue est une **erreur** au lieu d'être ignorée. Sans lui, l'utilisateur qui écrit `ignore_caze = true` croira son option active. Les fautes silencieuses sont les pires.

Autres classiques à connaître : `#[serde(rename_all = "camelCase")]` (JSON camelCase ↔ champs snake_case), `#[serde(skip)]`, `#[serde(rename = "...")]`.

**Dans le projet** : `FileConfig` désérialise le fichier `.minigrep.toml` ; les deux attributs ci-dessus sont à poser vous-même à l'étape 4.

---

## 7. clap — la ligne de commande en mode derive

**clap** (*Command Line Argument Parser*) est le standard pour les CLI. En mode **derive**, vous décrivez vos arguments par une struct, et clap génère le parsing, `--help`, `--version`, les erreurs d'usage, les suggestions de correction :

```rust
use clap::Parser;

/// Count the words of a file. // ← devient la description du --help !
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// The file to analyze
    file: String, // argument positionnel obligatoire

    /// Minimum word length to count
    #[arg(short, long, default_value_t = 1)]
    min_length: usize, // -m 3 ou --min-length 3

    /// Show details for each line
    #[arg(short, long)]
    verbose: bool, // drapeau : présent = true
}

fn main() {
    let cli = Cli::parse();
    println!("file = {}, min = {}", cli.file, cli.min_length);
}
```

La correspondance type ↔ comportement est systématique :

| Type du champ | Signification CLI |
|---|---|
| `String`, `PathBuf` (sans `#[arg]`) | positionnel obligatoire |
| `Option<T>` | optionnel |
| `bool` + `#[arg(short, long)]` | drapeau on/off |
| `Vec<T>` | répétable / plusieurs valeurs |

Et les commentaires de doc `///` deviennent les textes d'aide : la documentation EST l'interface.

Comparaison : c'est picocli côté Java (annotations) ou commander/yargs côté Node — mais vérifié à la compilation : une option `usize` qui reçoit `"abc"` produit un message d'erreur propre généré pour vous, et il est impossible de lire une option qui n'existe pas (le champ n'existerait pas).

**Dans le projet** : la struct `Cli` du binaire déclare le motif, la liste de fichiers, les drapeaux `-i`, `-n`, `-v`, `-c` et l'option `--config <FILE>`.

---

## 8. Gestion d'erreurs professionnelle — thiserror et anyhow

### 8.1 Le problème

Au module 2, vos fonctions retournaient `Result<T, String>` ou un enum d'erreur maison, avec des `impl Display` fastidieux et des `map_err` partout. Deux crates jumelles, du même auteur, industrialisent tout ça — chacune pour un usage précis.

Rappel du paysage : pas d'exceptions en Rust. Une erreur est une **valeur** (`Result<T, E>`), l'opérateur `?` la propage, et `From` (§1.4) la convertit au passage. Java a les exceptions *checked* (signature honnête mais ergonomie pénible) ; TS lance n'importe quoi sans que rien n'apparaisse dans les types. Rust tient les deux bouts : honnêteté du type **et** ergonomie — grâce à ces deux crates.

### 8.2 `thiserror` — pour les bibliothèques

`thiserror` est un derive qui élimine le code rituel des **types d'erreur concrets**. Voici l'erreur du projet, en entier :

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GrepError {
    #[error("the search pattern is empty")]
    EmptyPattern,

    #[error("invalid configuration file: {0}")]
    InvalidConfig(#[from] toml::de::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

Trois attributs, trois générations de code :

- `#[derive(Error)]` : implémente `std::error::Error` ;
- `#[error("...")]` : implémente `Display` avec ce message (`{0}` interpole le champ) ;
- `#[from]` : génère `impl From<toml::de::Error> for GrepError` → l'opérateur `?` convertit tout seul. `fs::read_to_string(path)?` dans une fonction retournant `Result<_, GrepError>` : ça compile, l'erreur io est enveloppée dans `GrepError::Io`.

Pourquoi pour les bibliothèques ? Parce que l'appelant peut **réagir différemment selon la variante** :

```rust
match minigrep::search(pattern, content, &options) {
    Err(GrepError::EmptyPattern) => eprintln!("give me something to search!"),
    Err(other) => eprintln!("{other}"),
    Ok(results) => { /* ... */ }
}
```

Un type d'erreur précis fait partie de l'API publique, au même titre que les fonctions.

### 8.3 `anyhow` — pour les binaires

À l'autre bout, le `main` d'une application ne *réagit* pas aux erreurs : il les affiche et s'arrête. Distinguer les variantes ne sert à rien ; ce qui compte, c'est un **message riche en contexte**. `anyhow` fournit :

- `anyhow::Error` : un type valise qui absorbe n'importe quelle erreur implémentant `std::error::Error` (les vôtres via thiserror, celles de la std, celles des crates) ;
- `anyhow::Result<T>` : alias de `Result<T, anyhow::Error>` ;
- le trait `Context` : ajouter du contexte au passage.

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let path = "notes.txt";
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read file `{path}`"))?;
    println!("{} bytes", content.len());
    Ok(())
}
```

Si le fichier manque, l'utilisateur voit la chaîne complète des causes :

```text
Error: cannot read file `notes.txt`

Caused by:
    No such file or directory (os error 2)
```

Bonus : `with_context` fonctionne aussi sur `Option<T>` (un `None` devient une erreur avec message) — utilisé dans le corrigé du projet.

### 8.4 Quand utiliser quoi

| Vous écrivez… | Utilisez | Pourquoi |
|---|---|---|
| une bibliothèque (du code que d'autres appellent) | `thiserror` | l'appelant doit pouvoir matcher des variantes précises |
| un binaire / une application | `anyhow` | on veut juste propager avec `?` et afficher un message riche |
| les deux (comme ce projet !) | les deux | la lib expose `GrepError`, le `main` l'absorbe dans `anyhow` |

La combinaison est sans friction : `GrepError` implémente `std::error::Error` (merci thiserror), donc `?` le convertit automatiquement en `anyhow::Error` dans le binaire. C'est l'architecture de référence de l'écosystème.

---

## 9. Les tests — unitaires, d'intégration, et l'architecture lib + bin

### 9.1 Tests unitaires : `#[cfg(test)]`

En Rust, le framework de test est **intégré** : pas de JUnit ni de Jest à installer. Et la convention surprend quand on vient de Java : les tests unitaires vivent **dans le même fichier** que le code testé :

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]          // ce module n'est compilé QUE pour `cargo test` :
mod tests {           // zéro octet dans le binaire de production
    use super::*;     // importe tout le module parent (add, etc.)

    #[test]
    fn adds_two_numbers() {
        assert_eq!(add(2, 2), 4);
    }

    #[test]
    fn supports_negatives() {
        assert!(add(-2, 2) == 0);
    }
}
```

L'outillage de base :

- `assert!`, `assert_eq!`, `assert_ne!` — en cas d'échec, `assert_eq!` affiche les deux valeurs (d'où l'exigence `Debug` sur vos types) ;
- `assert!(matches!(error, GrepError::EmptyPattern))` — vérifier une variante d'enum sans exiger `PartialEq` ;
- `.unwrap()` / `.unwrap_err()` — dans un test, paniquer est exactement le comportement voulu en cas d'imprévu ;
- `#[should_panic]`, tests retournant `Result<(), E>` (pour utiliser `?`) — à connaître.

Commandes : `cargo test` (tout), `cargo test ignore_case` (les tests dont le nom contient la chaîne), `cargo test --no-run` (compiler sans exécuter), `cargo test -- --nocapture` (voir les `println!`).

Avantage du module dans le fichier : les tests accèdent aux fonctions **privées**. Débat tranché en Rust : oui, on teste les détails internes quand c'est utile, et le test vit à côté du code qu'il protège.

### 9.2 Tests d'intégration : le dossier `tests/`

À la racine du projet (à côté de `src/`), chaque fichier de `tests/` est compilé comme une **crate indépendante** qui dépend de votre bibliothèque :

```text
minigrep/
├── src/
│   ├── lib.rs        ← la bibliothèque (la logique)
│   ├── main.rs       ← le binaire (l'enrobage CLI)
│   ├── search.rs
│   └── config.rs
└── tests/
    ├── search_in_files.rs   ← tests d'intégration
    └── fixtures/
        ├── poem.txt         ← fichiers d'exemple
        └── config.toml
```

```rust
// tests/search_in_files.rs — une crate à part : on importe minigrep
// comme le ferait n'importe quel utilisateur externe.
use minigrep::{SearchOptions, search};

#[test]
fn finds_a_word_in_a_real_file() {
    let content = std::fs::read_to_string("tests/fixtures/poem.txt").unwrap();
    let results = search("nobody", &content, &SearchOptions::default()).unwrap();
    assert_eq!(results.len(), 2);
}
```

Différence clé avec les tests unitaires : ici, **seule l'API publique** est visible — `use minigrep::...`, comme un client. Si un test d'intégration ne peut pas s'écrire, c'est que votre API publique a un trou : le test vérifie aussi votre *design*. Les fichiers d'exemple (*fixtures*) vivent dans `tests/fixtures/` ; Cargo exécute toujours les tests depuis la racine de la crate, les chemins relatifs sont donc fiables.

En résumé : unitaire = la mécanique interne, au plus près du code ; intégration = le contrat public, depuis l'extérieur. Les deux familles tournent avec un seul `cargo test`.

### 9.3 L'architecture lib + bin

Le point qui change tout : **les tests d'intégration ne voient que la bibliothèque, pas le binaire**. Si toute votre logique est dans `main.rs`, elle est intestable. La parade est une convention universelle en Rust :

- `src/lib.rs` — la bibliothèque : TOUTE la logique (recherche, configuration, erreurs) ;
- `src/main.rs` — le binaire : une couche fine qui parse la CLI (clap), appelle la lib et affiche.

Les deux coexistent dans le même `Cargo.toml`, sous le même nom ; le binaire utilise la lib comme une dépendance externe (`use minigrep::search;`). Règle d'or : **si une fonction mérite un test, elle appartient à la lib.** Le `main` devient si bête qu'un test manuel (`cargo run -- ...`) suffit ; pour le tester automatiquement quand même, la crate `assert_cmd` existe (voir DEFIS.md).

Bonus de la séparation : votre outil CLI devient *aussi* une bibliothèque réutilisable — la doc `cargo doc`, les doc-tests (les exemples de doc sont compilés et exécutés par `cargo test` !) et la publication éventuelle sur crates.io viennent gratuitement.

---

## 10. Récapitulatif — où chaque concept sert dans le projet

| Concept | Où vous le pratiquerez |
|---|---|
| Traits (`Default`, `Display`, `From`) | `SearchOptions::default()`, messages de `GrepError`, conversions du `?` |
| Génériques & bounds | signatures de `filter`/`map`/`collect`, lecture des erreurs du compilateur |
| Closures | les `\|(_, line)\| ...` du pipeline de recherche |
| Itérateurs | `lines().enumerate().filter().map().collect()` — le cœur du programme |
| Crates & features | `Cargo.toml` : clap (feature `derive`), serde (feature `derive`), toml… |
| serde | `FileConfig` ← `.minigrep.toml`, attributs `default` + `deny_unknown_fields` |
| clap derive | la struct `Cli` : positionnels, drapeaux courts/longs, `--config` |
| thiserror | `GrepError` dans la lib (`src/error.rs`) |
| anyhow | `main() -> anyhow::Result<()>`, `.with_context(...)` |
| Tests & lib + bin | tests unitaires fournis dans `src/`, tests d'intégration dans `tests/` |

Ouvrez maintenant `PROJET.md` : le squelette vous attend dans `projet/`, avec ses tests qui ne demandent qu'à passer au vert.
