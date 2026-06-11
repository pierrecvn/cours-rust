# Module 2 — Structurer son code

Au module 1, vous avez écrit des programmes dans une seule fonction `main`, avec des types primitifs et des `String`. Ce module vous donne les outils pour construire de vrais programmes : modéliser vos données (structs, enums), dire adieu au `null` et aux exceptions (`Option`, `Result`), stocker des collections (`Vec`, `HashMap`) et organiser le code en modules.

C'est le module où Rust cesse de ressembler à « du C en plus strict » et commence à ressembler à ce qu'il est vraiment : un langage avec un système de types expressif, plus proche d'un langage fonctionnel typé que de Java.

**Projet du module** : un gestionnaire de tâches en ligne de commande, avec persistance dans un fichier texte parsé à la main. Voir `PROJET.md`.

---

## 1. Les structs : modéliser vos données

### Définition

Une `struct` regroupe des données nommées sous un même type. C'est l'équivalent d'une classe Java ou d'une interface/classe TypeScript… mais **sans héritage** et avec une séparation nette entre les données et le comportement.

```rust
struct Task {
    id: u64,
    description: String,
    done: bool,
}
```

### Comparaison Java/TS

```java
// Java — données et méthodes mélangées dans la classe
public class Task {
    private long id;
    private String description;
    private boolean done;
    // constructeur, getters, setters…
}
```

```typescript
// TypeScript
interface Task {
    id: number;
    description: string;
    done: boolean;
}
```

Différences clés :

- **Pas de constructeur spécial.** On crée une struct avec la syntaxe littérale : `Task { id: 1, description: ..., done: false }`. Par convention, on fournit une fonction associée `new` (voir plus bas), mais c'est une fonction ordinaire, pas un mot-clé.
- **Pas de `null` possible.** Un champ `description: String` contient *toujours* une `String` valide. Si l'absence est possible, le type doit le dire : `Option<String>` (section 4).
- **Visibilité privée par défaut**, comme en Java — mais au niveau du *module*, pas de la classe (section 8).
- **La mutabilité dépend de la variable, pas du type.** `let task = ...` rend *tous* les champs immuables ; `let mut task = ...` les rend tous modifiables. Pas de `readonly` champ par champ comme en TS.

### Exemple minimal compilable

```rust
struct Task {
    id: u64,
    description: String,
    done: bool,
}

fn main() {
    let mut task = Task {
        id: 1,
        description: String::from("Apprendre les structs"),
        done: false,
    };
    task.done = true; // possible car `let mut`
    println!("Tâche n°{} : {} (terminée : {})", task.id, task.description, task.done);
}
```

### `impl` : méthodes et fonctions associées

Le comportement se définit dans un bloc `impl`, séparé de la struct. Deux sortes de fonctions y vivent :

- les **méthodes**, qui prennent `self` (ou `&self`, ou `&mut self`) en premier paramètre et s'appellent avec `task.method()` ;
- les **fonctions associées**, sans `self`, qui s'appellent avec `Task::function()` — l'équivalent des méthodes `static` de Java/TS. La convention `Task::new(...)` joue le rôle du constructeur.

```rust
struct Task {
    id: u64,
    description: String,
    done: bool,
}

impl Task {
    /// Fonction associée (pas de `self`) : le « constructeur » par convention.
    fn new(id: u64, description: String) -> Self {
        Self { id, description, done: false }
    }

    /// Méthode en lecture seule : emprunte `self` sans le modifier.
    fn is_done(&self) -> bool {
        self.done
    }

    /// Méthode qui modifie : emprunte `self` de façon mutable.
    fn mark_done(&mut self) {
        self.done = true;
    }
}

fn main() {
    let mut task = Task::new(1, String::from("Lire le module 2"));
    task.mark_done();
    println!("Terminée ? {}", task.is_done());
}
```

Trois choses à retenir :

1. **`Self`** (majuscule) est un alias pour « le type du bloc `impl` courant », ici `Task`. `Self { id, description, done: false }` construit une `Task`. Notez aussi le raccourci `id` au lieu de `id: id` quand la variable porte le même nom que le champ — exactement comme le *shorthand* des objets en TS.
2. **Le receveur est explicite.** Là où Java et TS ont un `this` implicite, Rust vous force à choisir :
   - `&self` : lecture seule (un emprunt partagé, cf. module 1) ;
   - `&mut self` : modification (un emprunt exclusif) ;
   - `self` : la méthode **consomme** la valeur — après l'appel, la variable n'est plus utilisable. Rare, mais utile (transformations, fermetures de ressources).
3. **Le borrow checker s'applique aux méthodes.** Appeler `task.mark_done()` exige que `task` soit déclarée `mut`. Le compilateur vous le dira sinon.

### Pourquoi pas d'héritage ?

Rust n'a **pas d'héritage de classes**, et c'est un choix délibéré, pas un manque. L'héritage profond crée des hiérarchies fragiles (le fameux problème de la classe de base fragile, que vous avez peut-être déjà subi en Java). Rust propose à la place :

- la **composition** : une struct contient d'autres structs ;
- les **enums** (section suivante) : quand « un `Shape` est soit un `Circle`, soit un `Square` », Rust modélise cela directement, sans hiérarchie de classes ;
- les **traits** (module 3) : du polymorphisme à la `interface` Java/TS, mais sans héritage d'état.

Si votre réflexe Java est « je fais une classe abstraite `AbstractTask` », le réflexe Rust est presque toujours « je fais un enum ou un trait ».

---

## 2. Les enums à données : les types sommes

### Définition

Attention, gros morceau. L'enum Rust n'a **presque rien à voir** avec l'enum de Java ou de TypeScript. En Java/TS, un enum est une liste de constantes nommées. En Rust, un enum est un **type somme** : un type dont chaque valeur est *une parmi plusieurs variantes*, et **chaque variante peut transporter des données différentes**.

```rust
enum Command {
    List,                    // variante sans donnée (comme un enum Java)
    Add(String),             // variante avec une donnée
    Done(u64),               // variante avec une autre donnée
    Move { id: u64, position: usize }, // variante avec champs nommés (comme une struct)
}
```

Une valeur de type `Command` est *exactement une* de ces quatre formes. Le compilateur le sait, et `match` (section 5) vous forcera à toutes les traiter.

### Comparaison Java/TS

Le cousin le plus proche n'est pas l'enum TS, mais l'**union discriminée** :

```typescript
// TypeScript — union discriminée, le vrai équivalent
type Command =
    | { kind: "list" }
    | { kind: "add"; description: string }
    | { kind: "done"; id: number }
    | { kind: "move"; id: number; position: number };
```

En Java, il faut une interface scellée et des records (Java 17+) pour approcher cela :

```java
// Java 17+ — sealed interface + records
sealed interface Command permits List, Add, Done, Move {}
record List() implements Command {}
record Add(String description) implements Command {}
record Done(long id) implements Command {}
record Move(long id, int position) implements Command {}
```

L'enum Rust fait tout cela nativement, en quatre lignes, avec l'exhaustivité vérifiée par le compilateur. Les types sommes sont l'outil de modélisation central de Rust : « rendre les états invalides irreprésentables ». Au lieu d'une struct avec des booléens et des champs nullables qui peuvent se contredire, on écrit un enum dont chaque variante est un état cohérent.

### Exemple minimal compilable

```rust
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
}

impl Shape {
    // Oui, les enums peuvent avoir des méthodes, comme les structs.
    fn area(&self) -> f64 {
        match self {
            Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
            Shape::Rectangle { width, height } => width * height,
        }
    }
}

fn main() {
    let shapes = [
        Shape::Circle { radius: 1.0 },
        Shape::Rectangle { width: 3.0, height: 4.0 },
    ];
    for shape in &shapes {
        println!("aire = {:.2}", shape.area());
    }
}
```

Remarquez : pas de classe abstraite `Shape`, pas de méthode abstraite `area()` redéfinie dans chaque sous-classe. Une seule méthode, un `match`, et le compilateur garantit qu'aucune variante n'est oubliée — si vous ajoutez `Triangle` demain, **tous les `match` du projet refuseront de compiler** tant que vous ne les aurez pas mis à jour. C'est l'inverse du `default:` silencieux d'un `switch` Java.

---

## 3. `Option<T>` : la mort du `null`

### Définition

Rust n'a **pas de `null`**. Pas de `NullPointerException`, pas de `undefined is not a function`. Quand une valeur peut être absente, on le dit dans le type avec `Option<T>`, un enum de la bibliothèque standard :

```rust
enum Option<T> {
    Some(T), // une valeur est là
    None,    // pas de valeur
}
```

C'est tout. `Option` n'est pas un mécanisme magique du langage, c'est un enum ordinaire (avec un paramètre de type `T`, comme les génériques Java/TS — approfondis au module 3). `Some` et `None` sont importés automatiquement, on écrit `Some(5)` et non `Option::Some(5)`.

### Comparaison Java/TS

```typescript
// TypeScript — proche dans l'esprit, avec strictNullChecks
function findTask(id: number): Task | undefined { ... }

const task = findTask(42);
task.description;  // erreur TS : task est peut-être undefined
task?.description; // OK
```

`Option<T>` joue le rôle de `T | undefined`, mais en mieux : en TS, `strictNullChecks` peut être désactivé, des `any` peuvent passer, et `null`/`undefined` se glissent partout dans les bibliothèques. En Rust, il n'existe **aucun moyen** d'obtenir un « `Task` qui n'en est pas un ». Si la signature dit `Task`, il y a une tâche. Si elle dit `Option<Task>`, vous êtes *obligé* de traiter le cas `None` pour accéder à la valeur.

```java
// Java — Optional existe, mais null reste possible partout
Optional<Task> task = findTask(42);
// ... et rien n'empêche une autre méthode de retourner null directement.
```

L'`Optional` de Java est un pansement : `null` reste un habitant légal de tous les types objets. En Rust, l'absence de valeur n'existe que là où le type l'annonce. Tony Hoare, l'inventeur du `null`, l'a appelé « son erreur à un milliard de dollars » ; Rust a simplement décidé de ne pas la commettre.

### Exemple minimal compilable

```rust
fn find_task_description(id: u64) -> Option<String> {
    if id == 1 {
        Some(String::from("Apprendre Option"))
    } else {
        None
    }
}

fn main() {
    // Impossible d'utiliser le résultat sans traiter les deux cas :
    match find_task_description(1) {
        Some(description) => println!("Trouvée : {description}"),
        None => println!("Aucune tâche avec cet id"),
    }
}
```

### Les méthodes utilitaires à connaître

Déballer un `Option` au `match` à chaque fois serait verbeux. La bibliothèque standard fournit des raccourcis :

```rust
let maybe: Option<u64> = Some(5);

maybe.is_some();          // -> bool
maybe.unwrap_or(0);       // la valeur, ou 0 si None (≈ `x ?? 0` en TS)
maybe.map(|n| n * 2);     // transforme la valeur si présente (≈ `Optional.map` Java)
maybe.unwrap();           // ⚠️ la valeur, ou PANIQUE si None
maybe.expect("id manquant"); // pareil, avec un message d'erreur
```

`unwrap()` fait planter le programme si la valeur est absente — c'est l'équivalent de déréférencer sans vérifier. Acceptable dans un prototype ou un test, à éviter dans du vrai code : préférez `match`, `if let` (section 5) ou `unwrap_or`. La différence avec une NPE Java : le `unwrap()` est **visible et volontaire** dans le code, on le repère en relecture.

---

## 4. `Result<T, E>` : les erreurs comme valeurs

### Définition

Rust n'a **pas d'exceptions**. Une fonction qui peut échouer retourne un `Result<T, E>` :

```rust
enum Result<T, E> {
    Ok(T),   // succès, avec la valeur
    Err(E),  // échec, avec l'erreur
}
```

L'erreur est une **valeur ordinaire**, retournée par la fonction, visible dans sa signature. Comme `Option`, c'est un simple enum ; `Ok` et `Err` sont importés automatiquement.

### Comparaison Java/TS

```java
// Java — l'échec passe par une exception
long parseLong(String s) throws NumberFormatException
```

```typescript
// TypeScript — l'échec est invisible dans le type
function parseId(s: string): number {
    const n = Number(s);
    if (Number.isNaN(n)) throw new Error("id invalide"); // rien ne l'annonce
    return n;
}
```

Problèmes des exceptions que `Result` élimine :

- **En TS**, rien dans la signature n'indique qu'une fonction peut lancer. On le découvre en production.
- **En Java**, les *checked exceptions* annoncent l'échec (`throws`), mais le mécanisme est si pénible que l'écosystème a fui vers les `RuntimeException`… invisibles à nouveau.
- Dans les deux cas, une exception **traverse silencieusement** toutes les fonctions intermédiaires. Le flot de contrôle devient invisible.

Avec `Result`, l'échec est dans le type de retour, le compilateur vous avertit si vous ignorez un `Result` (`#[must_use]`), et la propagation est explicite — mais légère, grâce à `?`.

### Exemple minimal compilable

```rust
fn parse_id(input: &str) -> Result<u64, String> {
    // str::parse retourne déjà un Result ; on le traduit en message lisible.
    match input.parse::<u64>() {
        Ok(id) => Ok(id),
        Err(_) => Err(format!("« {input} » n'est pas un identifiant valide")),
    }
}

fn main() {
    match parse_id("42") {
        Ok(id) => println!("id = {id}"),
        Err(message) => eprintln!("Erreur : {message}"),
    }
}
```

En pratique, `E` est rarement une `String` : on définit un enum d'erreurs (`enum ParseError { MissingField, InvalidId, ... }`) pour que l'appelant puisse réagir cas par cas. C'est exactement ce que fait le projet de ce module.

### L'opérateur `?` : propager sans bruit

Le vrai confort de `Result` vient de l'opérateur `?`. Posé après une expression `Result`, il signifie : « si c'est `Ok(v)`, déballe `v` et continue ; si c'est `Err(e)`, **retourne immédiatement** `Err(e)` depuis la fonction courante ».

```rust
use std::fs;
use std::io;

fn read_first_line(path: &str) -> Result<String, io::Error> {
    let content = fs::read_to_string(path)?; // Err ? -> retour immédiat
    let first = content.lines().next().unwrap_or("").to_string();
    Ok(first)
}

fn main() {
    match read_first_line("notes.txt") {
        Ok(line) => println!("Première ligne : {line}"),
        Err(error) => eprintln!("Lecture impossible : {error}"),
    }
}
```

Sans `?`, il faudrait un `match` à chaque appel faillible. Avec `?`, le code reste aussi fluide que du code à exceptions, mais chaque point de propagation est **marqué d'un caractère visible**. En lisant `fs::read_to_string(path)?`, vous savez que la fonction peut s'arrêter ici. Deux contraintes :

- `?` ne s'utilise que dans une fonction qui retourne elle-même un `Result` (ou un `Option` : `?` propage aussi les `None`) ;
- le type d'erreur doit correspondre (ou être convertible — on verra `From` et `anyhow` au module 3).

Bonus : `main` peut retourner un `Result`, ce qui autorise `?` directement dedans :

```rust
fn main() -> Result<(), std::io::Error> {
    let content = std::fs::read_to_string("notes.txt")?;
    println!("{content}");
    Ok(())
}
```

---

## 5. `match` en profondeur, `if let`, `let else`

### `match` exhaustif

Vous avez vu `match` au module 1 comme un `switch` amélioré. Sa vraie puissance : il **destructure** (extrait les données des variantes) et il est **exhaustif** (le compilateur vérifie que tous les cas sont couverts).

```rust
enum Command {
    List,
    Add(String),
    Done(u64),
}

fn describe(command: &Command) -> String {
    match command {
        Command::List => String::from("afficher les tâches"),
        Command::Add(description) => format!("ajouter « {description} »"),
        Command::Done(id) => format!("terminer la tâche {id}"),
    }
}
```

Si vous oubliez une variante :

```text
error[E0004]: non-exhaustive patterns: `&Command::Done(_)` not covered
```

Le compilateur **refuse de compiler**. Comparez avec le `switch` Java/TS : un `case` oublié passe inaperçu (au mieux un avertissement de linter), et le `default:` avale les futurs cas en silence. En Rust, ajouter une variante à un enum, c'est recevoir la liste exacte des endroits à mettre à jour. C'est ce qui rend les gros refactorings sereins.

Autres formes de motifs utiles :

```rust
match id {
    0 => println!("zéro"),
    1..=9 => println!("un chiffre"),          // intervalle inclusif
    n if n % 2 == 0 => println!("pair : {n}"), // garde
    _ => println!("autre"),                    // joker (à utiliser avec parcimonie !)
}
```

⚠️ Le joker `_` désactive l'exhaustivité pour les cas qu'il couvre : sur un enum métier, préférez lister les variantes explicitement, pour que le compilateur vous rattrape lors d'un ajout futur.

`match` est une **expression** : il produit une valeur (`let label = match ... { ... };`), comme le ternaire `? :` mais en plus général.

### `if let` : quand un seul cas vous intéresse

Un `match` à deux bras dont l'un ne fait rien, c'est lourd. `if let` teste *un* motif :

```rust
let maybe_id: Option<u64> = Some(42);

if let Some(id) = maybe_id {
    println!("id présent : {id}");
} else {
    println!("pas d'id"); // le else est facultatif
}
```

Lisez-le « si `maybe_id` correspond au motif `Some(id)`, alors… ». Le `id` extrait n'existe que dans le bloc. C'est l'analogue du *narrowing* TS (`if (task !== undefined) { ... }`), mais piloté par les motifs.

### `let else` : déballer ou sortir

Très idiomatique pour les validations en début de fonction : « déballe, sinon quitte ».

```rust
fn process(input: Option<&str>) {
    let Some(value) = input else {
        eprintln!("aucune valeur fournie");
        return; // le bloc else DOIT diverger : return, break, continue, panic!…
    };
    // Ici, `value` est disponible, déballé, pour tout le reste de la fonction.
    println!("valeur : {value}");
}
```

C'est l'équivalent du *guard clause* (`if (x == null) return;` en Java/TS), avec le déballage en prime. Là où `if let` indente le « cas heureux » dans un bloc, `let else` le laisse à plat — parfait pour enchaîner plusieurs validations.

---

## 6. Les collections : `Vec<T>` et `HashMap<K, V>`

### `Vec<T>` : le tableau dynamique

`Vec<T>` est la liste de Rust : taille variable, éléments contigus en mémoire. L'équivalent d'`ArrayList<T>` en Java et de `T[]` en TS.

```rust
fn main() {
    let mut tasks: Vec<String> = Vec::new();      // vide
    tasks.push(String::from("apprendre Vec"));     // ajouter
    tasks.push(String::from("faire une pause"));

    let numbers = vec![1, 2, 3];                   // macro de création

    println!("{} tâches", tasks.len());
    let first: Option<&String> = tasks.first();    // accès sûr -> Option
    let second: &String = &tasks[1];               // accès direct -> panique si hors bornes
    if let Some(task) = first {
        println!("première : {task}");
    }
    println!("deuxième : {second}");
    let _ = numbers;
}
```

Remarquez le retour d'`Option` : `tasks.first()` et `tasks.get(i)` retournent `Option<&T>` au lieu de lancer une `IndexOutOfBoundsException`. L'indexation `tasks[i]` existe aussi, mais elle panique hors bornes — réservez-la aux cas où l'indice est garanti valide.

### Itérer : les trois formes

C'est ici que l'ownership du module 1 rencontre les collections. Il y a trois façons d'écrire une boucle `for`, et elles ne font **pas** la même chose :

```rust
for task in &tasks      { /* task: &String      — emprunte, lit             */ }
for task in &mut tasks  { /* task: &mut String  — emprunte, peut modifier   */ }
for task in tasks       { /* task: String       — CONSOMME le Vec entier !  */ }
```

En Java/TS, `for (const t of tasks)` ne « détruit » jamais la liste. En Rust, `for task in tasks` **déplace** le `Vec` dans la boucle : après elle, `tasks` n'existe plus. Le réflexe par défaut : **itérer avec `&`**.

### Erreur réelle n°1 : utiliser un `Vec` après l'avoir consommé

```rust
fn main() {
    let tasks: Vec<String> = vec![String::from("apprendre Rust")];
    for task in tasks {
        println!("{task}");
    }
    println!("{} tâches au total", tasks.len()); // 💥
}
```

Le compilateur répond :

```text
error[E0382]: borrow of moved value: `tasks`
 --> src/main.rs:6:36
  |
2 |     let tasks: Vec<String> = vec![String::from("apprendre Rust")];
  |         ----- move occurs because `tasks` has type `Vec<String>`, which does not implement the `Copy` trait
3 |     for task in tasks {
  |                 ----- `tasks` moved due to this implicit call to `.into_iter()`
...
6 |     println!("{} tâches au total", tasks.len());
  |                                    ^^^^^ value borrowed here after move
  |
help: consider iterating over a slice of the `Vec<String>`'s content to avoid moving into the `for` loop
  |
3 |     for task in &tasks {
  |                 +
```

Décortiquons :

- `move occurs because ... does not implement the Copy trait` : un `Vec<String>` ne se copie pas implicitement (ce serait coûteux) ; le passer « par valeur » le **déplace** (module 1).
- `moved due to this implicit call to .into_iter()` : `for x in tasks` appelle `into_iter()`, qui prend possession du `Vec` pour en redistribuer les éléments.
- `value borrowed here after move` : ligne 6, on tente de lire un cadavre.
- Et la solution est fournie : `for task in &tasks` — ajouter un seul caractère. Le compilateur Rust est réputé pour ça : il ne dit pas seulement *non*, il dit *comment*.

### Erreur réelle n°2 : `push` pendant un emprunt

Le piège classique. On garde une référence vers un élément, puis on `push` :

```rust
fn main() {
    let mut tasks: Vec<String> = vec![String::from("apprendre Rust")];
    let first = &tasks[0];                          // emprunt partagé
    tasks.push(String::from("faire les courses")); // 💥 emprunt exclusif
    println!("{first}");
}
```

```text
error[E0502]: cannot borrow `tasks` as mutable because it is also borrowed as immutable
 --> src/main.rs:4:5
  |
3 |     let first = &tasks[0];
  |                  ----- immutable borrow occurs here
4 |     tasks.push(String::from("faire les courses"));
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
5 |     println!("{first}");
  |                ----- immutable borrow later used here
```

Pourquoi est-ce interdit ? Ce n'est pas du zèle : un `push` peut forcer le `Vec` à **réallouer** sa mémoire ailleurs si sa capacité est pleine. La référence `first` pointerait alors vers la zone libérée — un *dangling pointer*, la famille de bugs (use-after-free) qui mine le C/C++. En Java, le ramasse-miettes maintient l'objet en vie ; en Rust, pas de ramasse-miettes, donc le borrow checker applique la règle du module 1 : **soit plusieurs lecteurs, soit un seul écrivain, jamais les deux**. `&tasks[0]` est un lecteur, `push` exige un écrivain (`&mut self`).

Le remède est presque toujours de **réordonner** ou de **raccourcir l'emprunt** :

```rust
let mut tasks = vec![String::from("apprendre Rust")];
let first = tasks[0].clone();                   // copie indépendante : plus d'emprunt
tasks.push(String::from("faire les courses")); // OK
println!("{first}");
```

ou simplement lire `first` *avant* le `push`, ou le relire *après*. Même cause, même sanction pour la variante « modifier la collection dans la boucle qui l'itère » (`for t in &tasks { tasks.push(...) }`) — légale en apparence en Java jusqu'à la `ConcurrentModificationException` *à l'exécution* ; en Rust, c'est refusé *à la compilation*.

### `HashMap<K, V>` : le dictionnaire

L'équivalent de `HashMap` Java et de `Map`/objet TS. Contrairement à `Vec`, il faut l'importer :

```rust
use std::collections::HashMap;

fn main() {
    let mut ages: HashMap<String, u32> = HashMap::new();
    ages.insert(String::from("Alice"), 31);
    ages.insert(String::from("Bilal"), 27);

    // get retourne Option<&V> : pas de null, pas de surprise.
    match ages.get("Alice") {
        Some(age) => println!("Alice a {age} ans"),
        None => println!("Alice est inconnue"),
    }

    // L'API entry : « modifie si présent, insère sinon » en une passe.
    *ages.entry(String::from("Alice")).or_insert(0) += 1;

    // Itération (ordre NON garanti, comme HashMap Java).
    for (name, age) in &ages {
        println!("{name} : {age} ans");
    }
}
```

Points d'attention :

- `get` retourne `Option<&V>` — là où `map.get(key)` Java retourne `null` et `map.get(key)` TS retourne `T | undefined`. Même idée que TS, mais le traitement du cas absent est obligatoire.
- `insert` sur une clé existante **remplace** la valeur (et retourne l'ancienne dans un `Some`).
- L'ownership s'applique : `ages.insert(name, 31)` avec `name: String` **déplace** `name` dans la map. Pour les clés `String`, on interroge ensuite avec un `&str` (`ages.get("Alice")`) — la coercion vue au module 1.
- Le piège n°2 vaut aussi ici : conserver un `&V` issu de `get` puis faire `insert` déclenche le même `E0502`.

---

## 7. Le système de modules : `mod`, `pub`, `use`

### Définition

Dès qu'un fichier dépasse quelques centaines de lignes, on découpe. L'unité d'organisation de Rust est le **module** : un espace de noms avec des règles de visibilité, déclaré avec `mod`, et qui peut vivre dans son propre fichier.

### Comparaison Java/TS

| | Java | TypeScript | Rust |
|---|---|---|---|
| Unité | package (= dossier) | module (= fichier) | `mod` (déclaré explicitement) |
| Découverte | implicite via l'arborescence | implicite via `import "./x"` | **explicite** : `mod x;` dans le parent |
| Visibilité par défaut | package-private | privé sauf `export` | privé au module sauf `pub` |
| Import | `import a.b.C;` | `import { C } from "./b"` | `use crate::b::C;` |

La grande différence avec TS : en TS, créer `storage.ts` suffit, n'importe qui peut l'importer. En Rust, un fichier **n'existe pas** pour le compilateur tant qu'un module parent ne l'a pas déclaré avec `mod storage;`. L'arbre des modules part de la racine du crate (`main.rs` ou `lib.rs`) et tout y est rattaché explicitement.

### Exemple : découpage en fichiers

```text
src/
├── main.rs      ← racine du crate
├── task.rs      ← module task
└── storage.rs   ← module storage
```

```rust
// src/task.rs
pub struct Task {            // pub : visible hors du module
    pub id: u64,             // les champs aussi ont leur visibilité
    pub description: String,
    secret_note: String,     // privé : inaccessible depuis main.rs
}

pub fn format_task(task: &Task) -> String {
    format!("[{}] {}", task.id, task.description)
}
```

```rust
// src/main.rs
mod task;     // « le module task existe, son code est dans task.rs »
mod storage;  // idem pour storage.rs

use task::Task; // raccourci : Task au lieu de task::Task

fn main() {
    let t = Task { /* ... */ };
}
```

Règles à retenir :

- **`mod task;`** dans `main.rs` rattache le fichier `task.rs` à l'arbre. Sans cette ligne, le fichier est invisible (erreur fréquente de débutant : « j'ai créé le fichier mais Rust ne le voit pas »).
- **Tout est privé par défaut.** `pub` rend visible aux modules parents : fonctions, structs, enums, et — piège — **chaque champ de struct individuellement**. Une struct `pub` aux champs privés ne peut être construite hors de son module qu'avec une fonction associée `pub fn new(...)` : c'est l'encapsulation à la Rust (les variantes d'un enum `pub`, elles, sont automatiquement publiques).
- **`use` crée un raccourci**, comme `import` en Java/TS. Sans `use`, on peut toujours écrire le chemin complet : `task::format_task(...)`. Chemins utiles : `crate::...` (depuis la racine du crate), `self::...` (module courant), `super::...` (module parent — utilisé par les tests, voir ci-dessous).
- Les modules peuvent aussi être **déclarés en ligne** : `mod helpers { ... }` dans le même fichier. C'est exactement ainsi que sont écrits les tests unitaires :

```rust
// À la fin de task.rs
#[cfg(test)]              // compilé uniquement pour `cargo test`
mod tests {
    use super::*;          // importe tout le module parent (Task, etc.)

    #[test]
    fn format_includes_id() {
        let task = Task::new(1, String::from("test"));
        assert_eq!(format_task(&task), "[1] test");
    }
}
```

Le projet de ce module vous fait pratiquer ce découpage (`task.rs`, `storage.rs`, `cli.rs`).

---

## 8. Les traits dérivés : `#[derive(...)]`

Dernier outil, en mode boîte noire (les traits sont le sujet du module 3). Vous croiserez vite ce problème :

```rust
struct Task { id: u64, description: String, done: bool }

fn main() {
    let task = Task { id: 1, description: String::from("test"), done: false };
    println!("{:?}", task); // 💥 error[E0277]: `Task` doesn't implement `Debug`
}
```

`{:?}` (l'affichage de débogage, vu au module 1) ne sait pas afficher votre type : personne ne lui a appris. L'attribut `#[derive(...)]` demande au compilateur de **générer automatiquement** ces capacités standard :

```rust
#[derive(Debug, Clone, PartialEq)]
struct Task {
    id: u64,
    description: String,
    done: bool,
}
```

- **`Debug`** : active `{:?}` et `{:#?}` (version indentée). Le `toString()` de débogage, gratuit. Réflexe : le dériver sur quasiment tout.
- **`Clone`** : active `.clone()`, la copie profonde explicite. Sans lui, votre struct se déplace mais ne se duplique pas.
- **`PartialEq`** : active `==` et `!=`, en comparant champ à champ. Indispensable pour `assert_eq!` dans les tests. Sans lui, `task1 == task2` ne compile pas — pas de comparaison par référence par défaut comme le `==` Java ou le `===` JS sur les objets : Rust préfère ne rien comparer plutôt que comparer la mauvaise chose.

Ces dérives ne marchent que si **tous les champs** ont eux-mêmes la capacité (on ne peut dériver `Clone` que si chaque champ est `Clone` — c'est le cas de tous les types standard). Il en existe d'autres (`Default`, `Hash`, `Eq`, `Copy`…) ; on les découvrira au fil des besoins. Pour l'instant, retenez le combo de travail : `#[derive(Debug, Clone, PartialEq)]`.

---

## Récapitulatif

| Concept | Équivalent Java/TS le plus proche | La différence Rust |
|---|---|---|
| `struct` + `impl` | classe / interface TS | données et comportement séparés, pas d'héritage, receveur explicite |
| `enum` à données | union discriminée TS, sealed interface Java | type somme natif, exhaustivité vérifiée |
| `Option<T>` | `T \| undefined`, `Optional<T>` | `null` n'existe pas, traitement du cas absent obligatoire |
| `Result<T, E>` + `?` | exceptions, `throws` | l'erreur est une valeur, propagation visible |
| `match` / `if let` / `let else` | `switch`, narrowing TS | destructuration + exhaustivité |
| `Vec`, `HashMap` | `ArrayList`/`HashMap`, `T[]`/`Map` | l'ownership s'applique : itérer avec `&`, pas de mutation sous emprunt |
| `mod` / `pub` / `use` | packages Java, imports TS | rattachement explicite (`mod x;`), tout privé par défaut |
| `#[derive(...)]` | `toString`/`equals` générés par l'IDE, lombok | généré par le compilateur, opt-in par type |

Vous avez tout ce qu'il faut pour le projet : ouvrez `PROJET.md`.
