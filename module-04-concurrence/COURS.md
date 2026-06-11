# Module 4 — Concurrence & lifetimes

> **Prérequis** : modules 1 à 3 (ownership/borrowing, structs, enums, `Option`/`Result`, traits, génériques, closures, itérateurs, tests).
> **Projet associé** : un scanner de fichiers parallèle (voir `PROJET.md`).

Ce module attaque les deux sujets qui ont la réputation de faire transpirer : les **lifetimes** (le deuxième mur après le borrow checker) et la **concurrence**. Bonne nouvelle : les deux sont intimement liés, et une fois les lifetimes comprises, la concurrence en Rust devient presque… reposante. Vous allez découvrir pourquoi on dit que Rust offre la *fearless concurrency* : le compilateur refuse de compiler un programme contenant une data race. En Java ou en Node, ce genre de bug se découvre en production, un vendredi soir.

## Sommaire

1. [Les lifetimes en profondeur](#1-les-lifetimes-en-profondeur)
2. [Les smart pointers : `Box`, `Rc`, `RefCell`](#2-les-smart-pointers--box-rc-refcell)
3. [Les threads](#3-les-threads)
4. [Les channels (mpsc) : communiquer par messages](#4-les-channels-mpsc--communiquer-par-messages)
5. [État partagé : `Arc` + `Mutex`](#5-état-partagé--arc--mutex)
6. [`Send` et `Sync` : le superpouvoir de Rust](#6-send-et-sync--le-superpouvoir-de-rust)
7. [Mesurer le temps avec `Instant`](#7-mesurer-le-temps-avec-instant)
8. [Récapitulatif : ce que le projet utilise](#8-récapitulatif--ce-que-le-projet-utilise)

---

## 1. Les lifetimes en profondeur

### 1.1 Pourquoi les lifetimes existent

**Définition.** Une *lifetime* (durée de vie) est la portion de code pendant laquelle une référence est garantie valide. Chaque référence en Rust a une lifetime, et le borrow checker vérifie qu'**aucune référence ne survit à la donnée qu'elle pointe**. La plupart du temps, ce travail est invisible ; les annotations explicites (`'a`) ne servent que lorsque le compilateur ne peut pas deviner seul les relations entre références.

**Comparaison Java/TS.** En Java et en TypeScript, le garbage collector garantit qu'un objet vit *au moins aussi longtemps* que la dernière référence qui le pointe : une référence ne peut jamais pointer vers de la mémoire libérée. Le prix : un GC qui tourne à l'exécution. Rust obtient la même garantie **à la compilation**, sans GC : c'est le compilateur qui prouve, fonction par fonction, que chaque référence est toujours valide. Quand il n'y arrive pas, il vous demande de l'aider avec des annotations — c'est tout ce que sont les lifetimes : des indications pour aider la preuve, pas des instructions exécutées.

**Le problème que les lifetimes empêchent : la référence pendante.** En C, ce code compile et explose à l'exécution. En Rust :

```rust
fn main() {
    let r;
    {
        let x = 5;
        r = &x;          // r emprunte x…
    }                    // …mais x meurt ici !
    println!("{r}");     // r pointerait vers de la mémoire libérée
}
```

Le compilateur refuse, avec l'erreur la plus célèbre après celles du borrow checker :

```text
error[E0597]: `x` does not live long enough
 --> src/main.rs:5:13
  |
4 |         let x = 5;
  |             - binding `x` declared here
5 |         r = &x;
  |             ^^ borrowed value does not live long enough
6 |     }
  |     - `x` dropped here while still borrowed
7 |     println!("{r}");
  |               --- borrow later used here
```

### 1.2 Décortiquons « borrowed value does not live long enough »

Cette erreur dit toujours la même chose, en trois temps. Apprenez à la lire, vous la croiserez souvent :

1. **`borrowed value does not live long enough`** : une valeur empruntée (ici `x`) meurt trop tôt.
2. **`` `x` dropped here while still borrowed ``** : le compilateur pointe la ligne exacte où la valeur est détruite (la fermeture du bloc) *alors qu'un emprunt existe encore*.
3. **`borrow later used here`** : il pointe l'usage ultérieur de la référence, qui prouve que l'emprunt devait encore être valide.

Autrement dit : le compilateur vous montre **la naissance de l'emprunt, la mort de la donnée, et l'usage fautif**. Trois lignes, toute l'histoire. Pour corriger, deux familles de solutions :

- **Faire vivre la donnée plus longtemps** : déclarer `x` dans une portée englobante.
- **Ne pas garder de référence** : prendre possession (déplacer ou cloner la valeur) plutôt qu'emprunter. `let r = x;` au lieu de `let r = &x;`.

Vous rencontrerez cette erreur dans le projet si vous essayez, par exemple, de renvoyer une référence vers une `String` construite localement dans une fonction. Réflexe : *qui possède la donnée, et jusqu'à quand ?*

### 1.3 Les annotations `<'a>` : quand le compilateur a besoin d'aide

Tant que les références restent à l'intérieur d'une fonction, le compilateur voit tout et se débrouille. Le problème apparaît **aux frontières** : signatures de fonctions et structs. Exemple classique :

```rust
fn longest(a: &str, b: &str) -> &str {
    if a.len() > b.len() { a } else { b }
}
```

```text
error[E0106]: missing lifetime specifier
 --> src/main.rs:1:33
  |
1 | fn longest(a: &str, b: &str) -> &str {
  |               ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the
          signature does not say whether it is borrowed from `a` or `b`
help: consider introducing a named lifetime parameter
  |
1 | fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
```

Pourquoi cette erreur ? Le compilateur vérifie chaque fonction **isolément**, à partir de sa seule signature (comme les types : il ne « regarde pas dans le corps » des autres fonctions). Or la signature ne dit pas si la référence renvoyée emprunte `a` ou `b` — donc impossible de savoir, côté appelant, combien de temps le résultat reste valide. La solution, suggérée par le compilateur lui-même :

```rust
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}

fn main() {
    let s1 = String::from("bonjour");
    let s2 = String::from("salut");
    println!("le plus long : {}", longest(&s1, &s2));
}
```

Lisez `<'a>` comme un **paramètre générique de durée de vie** (même syntaxe que `<T>`, mais avec une apostrophe). La signature dit : « il existe une durée `'a` telle que `a` et `b` vivent au moins `'a`, et le résultat est valide pendant `'a` ». Concrètement, le compilateur prend la **plus courte** des deux durées de vie : le résultat n'est utilisable que tant que *les deux* arguments sont vivants.

Trois points essentiels à retenir :

- Annoter ne **change rien à l'exécution** : aucune durée de vie n'est « allongée ». On décrit des contraintes, on n'agit pas. C'est un contrat, comme un type.
- `'a` est choisi par l'appelant, comme un `T` générique : à chaque appel sa durée concrète.
- Si le compilateur vous suggère une annotation, prenez le temps de comprendre *quelle relation* elle exprime — recopier la suggestion sans comprendre est le meilleur moyen de bloquer à la prochaine erreur.

Démonstration que la contrainte fonctionne :

```rust
fn main() {
    let s1 = String::from("bonjour");
    let result;
    {
        let s2 = String::from("salut");
        result = longest(&s1, &s2);   // 'a = la durée de vie de s2 (la plus courte)
    }                                  // s2 meurt ici
    println!("{result}");              // erreur E0597 : `s2` does not live long enough
}
```

Même si `longest` renvoie en réalité `&s1` (le plus long), le compilateur raisonne **sur la signature** : le résultat *pourrait* emprunter `s2`, donc il refuse. C'est conservateur, et c'est voulu.

### 1.4 L'élision : pourquoi vous n'avez (presque) jamais écrit `'a` jusqu'ici

Vous avez écrit trois modules entiers sans annoter une seule lifetime. Merci à l'**élision** : trois règles que le compilateur applique automatiquement aux signatures, parce qu'elles couvrent les cas écrasants en pratique.

1. Chaque paramètre référence reçoit sa propre lifetime : `fn f(x: &str, y: &str)` devient `fn f<'a, 'b>(x: &'a str, y: &'b str)`.
2. S'il n'y a **qu'un seul** paramètre référence, sa lifetime est attribuée à toutes les références du retour : `fn first_word(s: &str) -> &str` devient `fn first_word<'a>(s: &'a str) -> &'a str`.
3. Dans une méthode avec `&self` ou `&mut self`, la lifetime de `self` est attribuée au retour.

Si aucune règle ne s'applique (comme pour `longest` : deux paramètres, règle 2 inapplicable, pas de `self`), le compilateur exige une annotation. Voilà tout le mystère : **vous n'annotez que ce que l'élision ne sait pas deviner**.

```rust
// Élision (règle 2) : aucune annotation nécessaire.
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}
```

### 1.5 Lifetimes dans les structs

Une struct peut stocker des références — à condition de déclarer leur lifetime, car le compilateur doit garantir que **la struct ne survit pas à ce qu'elle emprunte** :

```rust
/// Un extrait de texte : emprunte une tranche d'une String existante.
struct Excerpt<'a> {
    text: &'a str,
}

impl<'a> Excerpt<'a> {
    fn shout(&self) -> String {
        self.text.to_uppercase()
    }
}

fn main() {
    let novel = String::from("Appelez-moi Ismaël. Il y a quelques années…");
    let first_sentence = novel.split('.').next().unwrap();
    let excerpt = Excerpt { text: first_sentence };
    println!("{}", excerpt.shout());
}   // ordre de destruction : excerpt, puis novel — tout va bien
```

`Excerpt<'a>` se lit : « un `Excerpt` qui emprunte quelque chose vivant au moins `'a` ». Si `novel` mourait avant `excerpt`, le compilateur refuserait avec un E0597. Comparaison Java/TS : là-bas, un objet qui garde une référence vers un autre le maintient en vie (le GC suit le graphe). En Rust, c'est l'inverse : la struct *dépend* de la donnée empruntée, et le compilateur vérifie l'ordre des morts.

**Conseil pratique pour débuter** : une struct qui emprunte est contagieuse (`'a` se propage partout) et contraignante. Quand vous hésitez, faites posséder la donnée par la struct (`String` plutôt que `&str`, `PathBuf` plutôt que `&Path`) — quitte à cloner. C'est exactement le choix que fera le projet : la struct `FileInfo` possède son `PathBuf`. On optimise *ensuite*, si besoin.

### 1.6 `'static`, la lifetime spéciale

`'static` signifie « valide pendant toute la durée du programme ». Deux usages courants :

- Les littéraux de chaînes : `let s: &'static str = "embarqué dans le binaire";`
- La contrainte `T: 'static` sur les threads (section 3) : « `T` ne contient aucune référence temporaire ». Un type *possédant* ses données (comme `String`, `Vec<u8>`, `PathBuf`) satisfait `'static` — il ne s'agit pas de vivre éternellement, mais de **ne dépendre de rien d'éphémère**.

**Dans le projet** : vous ne devriez pas avoir à écrire une seule annotation `'a` — vos structs possèdent leurs données. Mais vous *verrez* les lifetimes à l'œuvre : la contrainte `'static` de `thread::spawn` est la raison d'être du mot-clé `move` (section 3.3).

---

## 2. Les smart pointers : `Box`, `Rc`, `RefCell`

**Définition.** Un *smart pointer* est une struct qui se comporte comme une référence (on peut la déréférencer) mais qui ajoute des garanties ou des capacités : possession de données sur le tas, comptage de références, vérifications à l'exécution… Vous en connaissez déjà sans le savoir : `String` et `Vec<T>` sont des smart pointers (ils possèdent un bloc sur le tas et le libèrent à leur mort).

**Comparaison Java/TS.** En Java comme en JS, *tout objet* vit sur le tas et *toute variable d'objet* est déjà un pointeur géré par le GC — vous n'avez jamais eu à choisir. Rust vous donne le choix, donc les outils : `Box` (un propriétaire unique sur le tas), `Rc` (plusieurs propriétaires), `RefCell` (mutation contrôlée à l'exécution).

### 2.1 `Box<T>` : allouer sur le tas

`Box<T>` place une valeur sur le tas et la possède. À la mort du `Box`, la valeur est libérée. C'est le smart pointer le plus simple : pas de surcoût autre que l'allocation.

```rust
fn main() {
    let boxed: Box<i32> = Box::new(42);   // 42 vit sur le tas
    println!("{}", *boxed + 1);           // * pour déréférencer (souvent implicite)
}
```

Pourquoi en aurait-on besoin ? Le cas d'école : les **types récursifs**. Rust doit connaître la taille de chaque type à la compilation, or un type qui se contient lui-même aurait une taille infinie :

```rust
// ERREUR : taille infinie
enum List {
    Node(i32, List),   // error[E0072]: recursive type `List` has infinite size
    End,
}
```

`Box` casse la récursion : un pointeur a une taille connue (8 octets sur une machine 64 bits), quel que soit ce qu'il pointe.

```rust
enum List {
    Node(i32, Box<List>),   // OK : Node = un i32 + un pointeur
    End,
}

fn main() {
    use List::{End, Node};
    let list = Node(1, Box::new(Node(2, Box::new(End))));
    let mut current = &list;
    while let Node(value, next) = current {
        println!("{value}");
        current = next;
    }
}
```

En Java, `class Node { int value; Node next; }` fonctionne directement… parce que `Node next` est *déjà* un pointeur (potentiellement `null`). Rust rend ce pointeur explicite, et `Box<List>` ne peut pas être nul — l'absence se dit avec la variante `End` (ou un `Option<Box<Node>>`).

Autres usages de `Box` : les objets traits (`Box<dyn Error>`, croisé au module 3) et les très grosses valeurs qu'on ne veut pas copier de pile en pile.

### 2.2 `Rc<T>` : plusieurs propriétaires (comptage de références)

Tout le cours jusqu'ici a martelé : *une valeur, un propriétaire*. Parfois, c'est trop strict : pensez à un graphe où deux nœuds partagent un voisin, ou à une config partagée par plusieurs sous-systèmes. `Rc<T>` (*Reference Counted*) autorise **plusieurs propriétaires** : la valeur est libérée quand le *dernier* `Rc` meurt.

```rust
use std::rc::Rc;

fn main() {
    let config = Rc::new(String::from("mode=rapide"));
    println!("compteur : {}", Rc::strong_count(&config));   // 1

    let user_a = Rc::clone(&config);   // n'incrémente qu'un compteur :
    let user_b = Rc::clone(&config);   // la String n'est PAS copiée
    println!("compteur : {}", Rc::strong_count(&config));   // 3

    drop(user_a);
    println!("compteur : {}", Rc::strong_count(&config));   // 2
}   // user_b puis config meurent : compteur 0, la String est libérée
```

`Rc::clone` ne clone que le pointeur et incrémente un compteur — c'est bon marché, contrairement à un `config.clone()` qui copierait la donnée. La convention `Rc::clone(&x)` plutôt que `x.clone()` rend cette différence visible à la lecture.

**Comparaison Java/JS : compté, pas tracé.** Le GC de la JVM et de V8 est *traçant* : périodiquement, il part des racines, marque tout ce qui est joignable, et libère le reste — d'où des pauses, mais les cycles (A pointe B qui pointe A) sont collectés. `Rc` est un *comptage de références* : libération **immédiate et déterministe** dès que le compteur tombe à zéro, aucune pause, aucun runtime… mais les **cycles fuient** (deux `Rc` qui se pointent mutuellement gardent leurs compteurs à 1 pour toujours). Rust fournit `Weak<T>` (une référence qui ne compte pas) pour casser les cycles — hors programme de ce module, mais retenez le nom. C'est le même mécanisme que les `shared_ptr` de C++ ou le comptage de références de Python et Swift.

Deux limites importantes de `Rc` :

1. `Rc<T>` ne donne qu'un **accès partagé en lecture** : pas de `&mut` vers le contenu (plusieurs propriétaires + mutation = la recette du chaos, règle du module 1). Solution : `RefCell`, juste en dessous.
2. `Rc` n'est **pas utilisable entre threads** : son compteur n'est pas atomique. Le compilateur le refusera (vous verrez l'erreur, section 5.1) ; la version thread-safe s'appelle `Arc`.

### 2.3 `RefCell<T>` : l'interior mutability

**Définition.** `RefCell<T>` enveloppe une valeur et déplace la vérification des règles d'emprunt **de la compilation vers l'exécution**. On peut demander un emprunt mutable à travers une référence partagée (`&RefCell<T>` → `&mut T`) — c'est l'*interior mutability* (mutabilité intérieure). Les règles ne changent pas (plusieurs lecteurs *ou* un seul écrivain), seul le moment de la vérification change : violer la règle ne donne plus une erreur de compilation, mais un **panic**.

```rust
use std::cell::RefCell;

fn main() {
    let cell = RefCell::new(vec![1, 2, 3]);

    cell.borrow_mut().push(4);            // emprunt mutable, rendu aussitôt
    println!("{:?}", cell.borrow());      // emprunt partagé : [1, 2, 3, 4]

    let _reader = cell.borrow();          // un emprunt partagé encore vivant…
    let _writer = cell.borrow_mut();      // PANIC !
}
```

```text
thread 'main' panicked at src/main.rs:9:24:
already mutably borrowed: BorrowError
```

(et dans l'autre sens, deux `borrow_mut()` simultanés donnent `already borrowed: BorrowMutError`). Le compilateur n'a rien dit ; le crash est arrivé à l'exécution. C'est le compromis de `RefCell` : de la souplesse contre une vérification repoussée.

**Comparaison Java/TS.** C'est le monde par défaut de Java et de JS : n'importe quelle référence permet de muter l'objet, sans aucune vérification — ni à la compilation, ni à l'exécution. `RefCell` est un intermédiaire honnête : la mutation partagée est permise, mais les violations *se voient* (panic immédiat, net, localisé) au lieu de corrompre silencieusement l'état.

**Le duo `Rc<RefCell<T>>`** : plusieurs propriétaires (`Rc`) d'une donnée mutable (`RefCell`). C'est l'idiome standard pour un état partagé mutable *dans un seul thread* :

```rust
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let shared = Rc::new(RefCell::new(Vec::new()));
    let writer_a = Rc::clone(&shared);
    let writer_b = Rc::clone(&shared);

    writer_a.borrow_mut().push("depuis a");
    writer_b.borrow_mut().push("depuis b");
    println!("{:?}", shared.borrow());   // ["depuis a", "depuis b"]
}
```

Retenez la symétrie, elle structure toute la suite du module :

| | Un seul thread | Plusieurs threads |
|---|---|---|
| Plusieurs propriétaires | `Rc<T>` | `Arc<T>` |
| Mutation partagée | `RefCell<T>` (vérif. à l'exécution, panic) | `Mutex<T>` (verrou à l'exécution, blocage) |
| Le duo complet | `Rc<RefCell<T>>` | `Arc<Mutex<T>>` |

**Dans le projet** : `Box`, `Rc` et `RefCell` ne sont pas directement utilisés (le projet est multi-thread, donc colonne de droite !), mais comprendre `Rc<RefCell<T>>` rend `Arc<Mutex<T>>` évident : c'est le même duo, durci pour les threads.

---

## 3. Les threads

### 3.1 `thread::spawn` et `join`

**Définition.** Un thread est un fil d'exécution indépendant, ordonnancé par l'OS. `std::thread::spawn` lance un thread qui exécute une closure ; il renvoie un `JoinHandle`, dont la méthode `join()` attend la fin du thread et récupère sa valeur de retour.

**Comparaison Java/TS.**
- **Java** : même modèle — des threads OS (`new Thread(runnable).start()`, `thread.join()`, ou un `ExecutorService`). Un thread Rust correspond à un thread « platform » de Java (pas un virtual thread). La grande différence n'est pas le modèle, c'est ce que le compilateur **interdit** de partager entre threads (section 6).
- **Node/TS** : modèle radicalement différent — JavaScript est mono-thread avec une boucle d'événements ; la concurrence est de l'entrelacement de tâches (`async`), pas du parallélisme. Pour du vrai parallélisme, Node a les `worker_threads`, qui ne partagent (presque) pas de mémoire et communiquent par messages. Rust offre les deux : vrai parallélisme *et* mémoire partagée — sécurisée par le compilateur.

```rust
use std::thread;
use std::time::Duration;

fn main() {
    let handle = thread::spawn(|| {
        for i in 1..=3 {
            println!("thread enfant : {i}");
            thread::sleep(Duration::from_millis(10));
        }
        42   // un thread peut renvoyer une valeur
    });

    println!("thread principal : je continue pendant ce temps");

    let answer = handle.join().expect("le thread a paniqué");
    println!("le thread a renvoyé {answer}");
}
```

Trois choses à noter :

- L'entrelacement des `println!` varie d'une exécution à l'autre : l'ordonnancement appartient à l'OS.
- `join()` renvoie un `Result` : `Err` si le thread a **paniqué**. Ne pas appeler `join` est permis, mais si `main` se termine, tous les threads sont tués net (comme les threads *daemon* en Java).
- La valeur de retour du thread transite par le `join` — pratique pour récupérer un résultat sans état partagé.

### 3.2 Combien de threads ? `available_parallelism`

```rust
use std::thread;

fn main() {
    let cores = thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    println!("parallélisme disponible : {cores}");
}
```

C'est l'équivalent de `Runtime.getRuntime().availableProcessors()` en Java. Pour du travail purement CPU, dépasser ce nombre n'apporte rien ; pour du travail d'attente (disque, réseau), davantage de threads peut aider. Le projet vous fera mesurer ça concrètement.

### 3.3 `move` : pourquoi le compilateur l'exige

Essayons de faire lire une variable locale par un thread :

```rust
use std::thread;

fn main() {
    let data = vec![1, 2, 3];
    let handle = thread::spawn(|| {
        println!("{data:?}");
    });
    handle.join().unwrap();
}
```

```text
error[E0373]: closure may outlive the current function, but it borrows `data`,
              which is owned by the current function
 --> src/main.rs:5:32
  |
5 |     let handle = thread::spawn(|| {
  |                                ^^ may outlive borrowed value `data`
6 |         println!("{data:?}");
  |                    ---- `data` is borrowed here
  |
note: function requires argument type to outlive `'static`
help: to force the closure to take ownership of `data` (and any other
      referenced variables), use the `move` keyword
  |
5 |     let handle = thread::spawn(move || {
  |                                ++++
```

Lisez bien : c'est une **erreur de lifetime** (revoilà `'static` !). Le compilateur ne sait pas combien de temps le thread va vivre — peut-être plus longtemps que `main`, donc plus longtemps que `data`. Une closure qui *emprunte* `data` pourrait alors lire de la mémoire morte. D'où la contrainte de `thread::spawn` : la closure doit être `'static`, c'est-à-dire ne rien emprunter d'éphémère.

La solution : `move`, qui force la closure à **prendre possession** des variables capturées :

```rust
let data = vec![1, 2, 3];
let handle = thread::spawn(move || {
    println!("{data:?}");      // la closure possède data, plus d'emprunt
});
// println!("{data:?}");       // ERREUR E0382 ici : data a été déplacé
handle.join().unwrap();
```

En Java, vous auriez passé la référence au thread sans y penser — et créé, sans le savoir, un partage non synchronisé. Rust transforme ce piège en décision explicite : soit le thread **possède** la donnée (`move`), soit on met en place un vrai partage (`Arc`, section 5). Pas d'entre-deux accidentel.

**Dans le projet** : chaque worker du pool est lancé avec `thread::spawn(move || …)` et prend possession de ses clones d'`Arc`.

---

## 4. Les channels (mpsc) : communiquer par messages

**Définition.** Un *channel* est un tuyau à sens unique entre threads : des émetteurs (`Sender<T>`) envoient des valeurs, un récepteur (`Receiver<T>`) les reçoit, dans l'ordre. Le module standard s'appelle `std::sync::mpsc` : *multiple producer, single consumer* — on peut cloner l'émetteur (plusieurs producteurs), pas le récepteur.

La philosophie vient de Go : *« Do not communicate by sharing memory; instead, share memory by communicating »* — ne communiquez pas en partageant de la mémoire, partagez la mémoire en communiquant. Plutôt que dix threads qui tripotent la même structure sous verrou, chacun envoie ses résultats dans un tuyau ; la propriété des valeurs **est transférée avec le message** (le `send` déplace la valeur — ownership, toujours), donc aucun accès concurrent possible.

**Comparaison Java/TS.** En Java, l'équivalent est une `BlockingQueue` (par exemple `LinkedBlockingQueue`) entre producteurs et consommateurs — sauf qu'en Java, rien n'empêche le producteur de *garder une référence* vers l'objet envoyé et de le modifier pendant que le consommateur le lit. En Rust, c'est impossible : la valeur est déplacée. Côté Node, c'est exactement le modèle `postMessage` des `worker_threads` — mais sans la sérialisation/copie : Rust transfère la propriété, pas une copie.

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (sender, receiver) = mpsc::channel::<String>();

    for id in 0..3 {
        let sender = sender.clone();          // plusieurs producteurs
        thread::spawn(move || {
            let message = format!("rapport du worker {id}");
            sender.send(message).unwrap();    // send DÉPLACE la valeur
        });
    }
    drop(sender);   // on lâche l'émetteur original (important, voir ci-dessous)

    // Le Receiver est un itérateur : la boucle se termine quand
    // TOUS les Sender sont morts et que le tuyau est vide.
    for message in receiver {
        println!("reçu : {message}");
    }
    println!("tous les workers ont terminé");
}
```

Points clés :

- `sender.clone()` : autant de producteurs qu'on veut.
- **`drop(sender)`** : le piège classique. La boucle `for message in receiver` ne s'arrête que lorsque *tous* les émetteurs sont détruits. Les clones meurent à la fin de chaque thread, mais l'original vit dans `main` : sans le `drop`, la boucle attend indéfiniment. Si votre programme « se fige » sur un channel, cherchez un `Sender` oublié.
- `recv()` (utilisé implicitement par la boucle) **bloque** en attendant un message, et renvoie `Err(RecvError)` quand le canal est fermé — c'est le signal de fin, pas une erreur grave. Il existe aussi `try_recv()` (non bloquant).
- À l'inverse, `send` échoue si le récepteur est mort : chaque bout du tuyau sait quand l'autre a raccroché.

**Dans le projet** : le channel sert de **file de travail**. Le thread principal parcourt l'arborescence et envoie chaque chemin de fichier dans le tuyau ; les workers se servent dedans. La fin du parcours (drop du `Sender`) signale naturellement aux workers qu'il n'y a plus rien à faire. Petite subtilité : nos workers *partagent* l'unique `Receiver` (single consumer !) via un `Mutex` — vous verrez ça en pratique.

---

## 5. État partagé : `Arc` + `Mutex`

Le message passing ne couvre pas tout : parfois, plusieurs threads doivent lire et écrire **la même structure** (dans le projet : les statistiques globales). Il faut alors résoudre deux problèmes distincts : *qui possède la donnée ?* (`Arc`) et *qui a le droit d'y toucher, et quand ?* (`Mutex`).

### 5.1 Pourquoi `Rc` ne suffit pas

Essayons naïvement de partager un `Rc` entre threads :

```rust
use std::rc::Rc;
use std::thread;

fn main() {
    let shared = Rc::new(vec![1, 2, 3]);
    let clone = Rc::clone(&shared);
    thread::spawn(move || {
        println!("{clone:?}");
    });
}
```

```text
error[E0277]: `Rc<Vec<i32>>` cannot be sent between threads safely
   --> src/main.rs:7:19
    |
7   |       thread::spawn(move || {
    |       ------------- ^------
    |       |             |
    |  _____|_____________within this `{closure@src/main.rs:7:19}`
    | |
8   | |         println!("{clone:?}");
9   | |     });
    | |_____^ `Rc<Vec<i32>>` cannot be sent between threads safely
    |
    = help: within `{closure@src/main.rs:7:19}`, the trait `Send` is not
            implemented for `Rc<Vec<i32>>`
note: required by a bound in `spawn`
```

Le compilateur dit : `Rc` n'implémente pas `Send` (section 6), donc interdiction de l'envoyer dans un thread. La raison profonde : le compteur de références de `Rc` est un entier ordinaire, incrémenté sans synchronisation. Deux threads qui clonent en même temps pourraient produire un compteur faux → libération prématurée ou fuite. **`Arc`** (*Atomically Reference Counted*) est le même objet avec un compteur **atomique** (opérations matérielles indivisibles) : légèrement plus coûteux, mais correct entre threads. C'est pour ça que les deux existent : ne payer le surcoût atomique que quand c'est nécessaire.

### 5.2 `Mutex<T>` : un verrou qui POSSÈDE sa donnée

`Arc` règle la possession partagée, mais comme `Rc`, il ne donne pas de `&mut` :

```text
error[E0596]: cannot borrow data in an `Arc` as mutable
```

Pour muter, il faut un `Mutex<T>` (*mutual exclusion*) : un verrou qu'un seul thread peut tenir à la fois. La conception de Rust est remarquable : **le mutex contient la donnée**. On ne peut accéder à `T` qu'en passant par `lock()` — oublier de verrouiller n'est pas un bug possible, c'est une erreur de compilation.

**Comparaison Java.** En Java, le verrou et la donnée sont découplés : `synchronized(lock) { list.add(x); }` *espère* que tout le monde utilise le même `lock` pour toucher `list`. Rien n'empêche un collègue (ou vous, six mois plus tard) d'écrire `list.add(y)` sans `synchronized` : ça compile, ça tourne, ça corrompt — silencieusement. En Rust, `Mutex<Vec<i32>>` rend la donnée **inaccessible autrement** : la discipline n'est plus une convention d'équipe, c'est le système de types.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();

    for _ in 0..8 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..1_000 {
                let mut guard = counter.lock().unwrap();
                *guard += 1;
            }   // le guard meurt ici → verrou relâché automatiquement
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
    println!("total : {}", *counter.lock().unwrap());   // toujours 8000
}
```

Mécanique à bien comprendre :

- `lock()` **bloque** jusqu'à obtenir le verrou, puis renvoie un `MutexGuard<T>` — un smart pointer vers la donnée.
- Le déverrouillage est **automatique** à la mort du guard (principe RAII, comme la fermeture d'un fichier). Pas d'`unlock` à oublier — l'équivalent du `finally { lock.unlock(); }` de Java, gratuit et infaillible.
- Corollaire : **tenez le verrou le moins longtemps possible**. Un guard gardé vivant pendant un long calcul met tous les autres threads en file d'attente. Idiome utile : `{ let g = m.lock().unwrap(); …courte section critique… }` — le bloc borne la vie du guard. Attention aussi au deadlock classique : verrouiller deux fois le même mutex dans le même thread, ou deux mutex dans des ordres différents. Ça, le compilateur ne le voit pas.

### 5.3 Le poison : quand un thread panique verrou en main

Pourquoi `lock()` renvoie-t-il un `Result` ? Si un thread **panique pendant qu'il tient le verrou**, la donnée protégée est peut-être dans un état incohérent (mise à jour à moitié faite). Le mutex est alors marqué **empoisonné** (*poisoned*), et tous les `lock()` suivants renvoient `Err(PoisonError)`. C'est un garde-fou de propagation : plutôt que de laisser les autres threads consommer un état corrompu sans le savoir, on les prévient.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));

    let clone = Arc::clone(&data);
    let _ = thread::spawn(move || {
        let _guard = clone.lock().unwrap();
        panic!("aïe, je panique avec le verrou en main !");
    })
    .join();   // Err : le thread a paniqué

    match data.lock() {
        Ok(_) => println!("tout va bien"),
        Err(poisoned) => {
            // On peut récupérer la donnée malgré tout, en connaissance de cause :
            println!("mutex empoisonné, données : {:?}", poisoned.into_inner());
        }
    }
}
```

En pratique, `lock().unwrap()` (ou `.expect("...")`) est l'idiome courant : si un thread a paniqué, paniquer à son tour est souvent le comportement le plus sain. Java n'a pas d'équivalent : après une exception dans un bloc `synchronized`, le verrou est relâché et les autres threads consomment l'état possiblement incohérent sans aucun avertissement.

**Dans le projet** : les statistiques globales vivent dans un `Arc<Mutex<Stats>>` cloné vers chaque worker ; et le `Receiver` du channel, qui n'admet qu'un consommateur, est partagé entre workers via un `Arc<Mutex<Receiver<PathBuf>>>`. À la fin, `Arc::try_unwrap(...)` + `into_inner()` permettent de récupérer le `Stats` nu une fois tous les threads joints.

---

## 6. `Send` et `Sync` : le superpouvoir de Rust

### 6.1 Définitions

Deux traits *marqueurs* (sans méthode) du module `std::marker` :

- **`Send`** : le type peut être **transféré** à un autre thread (sa propriété peut changer de thread).
- **`Sync`** : le type peut être **partagé par référence** entre threads (`&T` est `Send`).

Vous ne les implémentez jamais à la main : le compilateur les dérive **automatiquement** (ce sont des *auto traits*) — un type est `Send`/`Sync` si tous ses champs le sont. Presque tout est `Send + Sync` : les types primitifs, `String`, `Vec<T>`, `HashMap<K, V>`, `Mutex<T>`, `Arc<T>`… Les exceptions notables, et leurs raisons :

| Type | `Send` | `Sync` | Pourquoi |
|---|---|---|---|
| `Rc<T>` | non | non | compteur non atomique |
| `RefCell<T>` | oui | non | vérification d'emprunts non synchronisée |
| `MutexGuard<T>` | non | — | un verrou doit être relâché par le thread qui l'a pris |
| `Arc<T>`, `Mutex<T>` | oui | oui | conçus pour ça |

Et voici la clé de voûte : `thread::spawn` exige que sa closure soit `Send + 'static`. **Toute la sûreté concurrente de Rust tient dans cette signature.** Capturez un `Rc`, un `RefCell`, une référence éphémère ? La closure perd `Send` ou `'static`, et `spawn` refuse de compiler. Aucune analyse magique du code : juste le système de traits, appliqué mécaniquement.

### 6.2 Une data race arrêtée à la compilation

**Définition.** Une *data race* : deux threads accèdent à la même mémoire au même moment, dont au moins un en écriture, sans synchronisation. Résultat : comportement indéfini — valeurs corrompues, crashs aléatoires, bugs impossibles à reproduire.

Essayons d'en écrire une en Rust — deux threads qui poussent dans le même `Vec` :

```rust
use std::thread;

fn main() {
    let mut data = vec![1, 2, 3];

    let handle = thread::spawn(|| {
        data.push(4);          // écriture depuis le thread enfant
    });
    data.push(5);              // écriture depuis le thread principal : DATA RACE

    handle.join().unwrap();
}
```

```text
error[E0373]: closure may outlive the current function, but it borrows `data`,
              which is owned by the current function
 --> src/main.rs:6:32
  |
6 |     let handle = thread::spawn(|| {
  |                                ^^ may outlive borrowed value `data`
7 |         data.push(4);
  |         ---- `data` is borrowed here
  |
note: function requires argument type to outlive `'static`
help: to force the closure to take ownership of `data` (and any other
      referenced variables), use the `move` keyword
```

Première tentative bloquée. Obéissons au compilateur, ajoutons `move` :

```text
error[E0382]: borrow of moved value: `data`
 --> src/main.rs:9:5
  |
4 |     let mut data = vec![1, 2, 3];
  |         -------- move occurs because `data` has type `Vec<i32>`,
  |                  which does not implement the `Copy` trait
6 |     let handle = thread::spawn(move || {
  |                                ------- value moved into closure here
7 |         data.push(4);
  |         ---- variable moved due to use in closure
8 |     });
9 |     data.push(5);
  |     ^^^^ value borrowed here after move
```

Deuxième tentative bloquée : si le thread enfant possède `data`, le principal n'y touche plus. Troisième tentative, partager via `Arc` sans `Mutex` ? `cannot borrow data in an Arc as mutable` (E0596). Via `Rc<RefCell<…>>` ? `Rc … cannot be sent between threads safely` (E0277). **Toutes les issues sont murées.** Les seules portes que le compilateur laisse ouvertes sont, par construction, sûres : transférer la propriété, passer des messages (channel), ou partager sous verrou (`Arc<Mutex<T>>`). Une data race en Rust sûr n'est pas « déconseillée » : elle est **inexprimable**.

### 6.3 Le même bug en Java : silence radio

Le programme équivalent en Java compile sans un murmure :

```java
var data = new ArrayList<Integer>(List.of(1, 2, 3));
var thread = new Thread(() -> data.add(4));   // aucun avertissement
thread.start();
data.add(5);                                   // data race : compile très bien
thread.join();
```

À l'exécution : souvent ça « marche ». Parfois `data` contient 4 éléments au lieu de 5 (deux écritures au même index interne). Sous forte charge, une `ArrayIndexOutOfBoundsException` surgit au fond d'`ArrayList.add` — ou pire, rien d'observable avant des semaines. Le compilateur Java ne peut pas aider : *rien dans le type `ArrayList` ne dit s'il est partageable entre threads*. C'est documenté en anglais dans la Javadoc (« Note that this implementation is not synchronized »), à charge pour l'humain de le lire et d'y penser. En Rust, cette phrase de Javadoc **est un trait**, et le compilateur la lit pour vous, à chaque compilation. Voilà pourquoi on parle de *fearless concurrency* : si ça compile, il n'y a pas de data race¹.

¹ Les data races, pas tous les bugs de concurrence : deadlocks et famines restent possibles (le compilateur ne voit pas l'ordre de prise des verrous). Mais la catégorie de bugs la plus sournoise — la corruption mémoire silencieuse — disparaît.

**Dans le projet** : vous n'écrirez jamais `Send` ni `Sync`, mais ils travailleront pour vous. Si vous vous trompez (un `Rc` au lieu d'un `Arc`, un emprunt au lieu d'un `move`), le compilateur vous renverra précisément les erreurs de cette section — vous saurez les lire.

---

## 7. Mesurer le temps avec `Instant`

Le projet compare les performances séquentiel/parallèle. L'outil : `std::time::Instant`, une horloge **monotone** (jamais réglée en arrière, contrairement à l'horloge système) faite pour mesurer des durées :

```rust
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let total: u64 = (0..50_000_000).sum();
    let elapsed = start.elapsed();   // une Duration
    println!("somme = {total}, calculée en {elapsed:.2?}");
    // exemple de sortie : somme = 1249999975000000, calculée en 12.34ms
}
```

C'est l'équivalent de `System.nanoTime()` en Java ou `performance.now()` en JS — pas `System.currentTimeMillis()`/`Date.now()`, qui suivent l'horloge murale. `Duration` s'affiche joliment avec `{:?}` (unité choisie automatiquement) et se convertit avec `as_secs_f64()`, `as_millis()`, etc.

Mise en garde de bencheur : mesurer une seule exécution sur un système de fichiers, c'est approximatif. Le cache disque de l'OS fausse tout — la *deuxième* passe sur la même arborescence est toujours plus rapide que la première, quel que soit le code. Le projet en tiendra compte, et le défi 1 vous fera mesurer proprement.

---

## 8. Récapitulatif : ce que le projet utilise

Le scanner de fichiers parallèle assemble tout ce module :

| Concept | Où dans le projet |
|---|---|
| Ownership / `move` | les workers prennent possession de leurs clones d'`Arc` |
| `'static` (lifetimes) | contrainte de `thread::spawn`, satisfaite car nos types possèdent leurs données (`PathBuf`, pas `&Path`) |
| `thread::spawn` / `join` | le pool de workers, créé puis joint |
| Channel mpsc | la file de travail : le thread principal envoie les chemins, les workers consomment |
| `Arc<Mutex<Receiver>>` | partager l'unique récepteur entre les workers |
| `Arc<Mutex<Stats>>` | l'agrégation concurrente des statistiques |
| Poison | les `expect` sur `lock()` : si un worker panique, on veut le savoir |
| `Send`/`Sync` | invisibles et omniprésents : ils valident chaque ligne ci-dessus |
| `Instant` | le duel chronométré séquentiel vs parallèle |

Une dernière chose avant d'ouvrir `PROJET.md` : dans la vraie vie, on n'écrit pas toujours ce pool de threads à la main — la crate [`rayon`](https://crates.io/crates/rayon) transforme un `.iter()` en `.par_iter()` et s'occupe de tout. Mais on ne comprend `rayon` qu'après avoir construit la machinerie soi-même une fois. C'est parti.
