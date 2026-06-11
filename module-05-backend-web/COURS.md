# Module 5 — Backend web avec Axum

> **Prérequis** : modules 1 à 4 (ownership/borrowing, structs, enums, `Option`/`Result`, traits, génériques, closures, gestion d'erreurs avec `thiserror`, `Arc`/`Mutex`, threads).
> **Projet associé** : une API REST de gestion de notes (voir `PROJET.md`).

Vous savez désormais écrire du code Rust correct et concurrent. Ce module l'ouvre sur le monde : vous allez construire un **serveur HTTP**, le genre de service que vous écririez avec Express en Node ou Spring Boot en Java. Trois nouveautés majeures vous attendent :

1. **l'asynchrone** (`async`/`await`) et le **runtime** qui l'exécute (Tokio) — la façon dont Rust gère des milliers de connexions sans bloquer ;
2. **Axum**, le framework web : routeur, handlers, extracteurs, conversion en réponses HTTP ;
3. **SQLx**, pour parler à une base de données SQLite de façon asynchrone, avec une gestion d'erreurs propre traduite en codes HTTP.

À la fin, vous aurez une API qui respecte le **contrat du cours** (`GET/POST/PUT/DELETE /notes` sur `http://localhost:3000`), testée automatiquement, que le module 6 (app desktop Tauri) viendra consommer telle quelle.

## Sommaire

1. [`async`/`await` : la concurrence sans threads](#1-asyncawait--la-concurrence-sans-threads)
2. [Tokio : le runtime asynchrone](#2-tokio--le-runtime-asynchrone)
3. [Axum : le routeur et les handlers](#3-axum--le-routeur-et-les-handlers)
4. [Les extracteurs : la magie expliquée](#4-les-extracteurs--la-magie-expliquée)
5. [`IntoResponse` : transformer une valeur en réponse HTTP](#5-intoresponse--transformer-une-valeur-en-réponse-http)
6. [Le state partagé : injecter le pool de connexions](#6-le-state-partagé--injecter-le-pool-de-connexions)
7. [SQLx + SQLite : parler à la base](#7-sqlx--sqlite--parler-à-la-base)
8. [Gestion d'erreurs HTTP : du domaine au code de statut](#8-gestion-derreurs-http--du-domaine-au-code-de-statut)
9. [Tester une API sans lancer de serveur](#9-tester-une-api-sans-lancer-de-serveur)
10. [Récapitulatif : ce que le projet utilise](#10-récapitulatif--ce-que-le-projet-utilise)

---

## 1. `async`/`await` : la concurrence sans threads

### 1.1 Le problème : un serveur passe son temps à attendre

Un serveur web ne calcule presque rien. Il **attend** : il attend qu'une requête arrive sur le réseau, attend que la base de données réponde, attend que le disque rende un fichier. Pendant ces attentes, le processeur ne fait rien d'utile.

La question fondamentale d'un serveur est donc : *comment servir des centaines de clients simultanés sans gaspiller un thread (et sa pile mémoire) par client à attendre bêtement ?*

**Comparaison Java/TS.** Vous connaissez déjà deux réponses :

- **Node.js** : un unique thread et une *event loop*. Quand vous écrivez `await db.query(...)`, Node ne bloque pas : il met votre fonction de côté, va servir d'autres clients, et la reprend quand la base répond. Une `Promise` est l'objet qui représente « ce résultat, plus tard ».
- **Java** : historiquement, un thread du système d'exploitation par requête (lourd : ~1 Mio de pile chacun). Depuis Java 21, les *virtual threads* (threads virtuels) offrent une troisième voie : des millions de threads légers que la JVM multiplexe sur quelques threads réels.

Rust adopte le **modèle de Node** (des tâches coopératives multiplexées), mais sans imposer un seul thread : son runtime répartit les tâches sur tous vos cœurs.

### 1.2 `async fn` et `.await`

**Définition.** Une fonction marquée `async` ne renvoie pas directement son résultat : elle renvoie une **future** — une valeur qui *produira* le résultat plus tard. L'opérateur `.await` sur une future signifie « suspends-moi ici jusqu'à ce que ce résultat soit prêt, et laisse d'autres tâches s'exécuter pendant ce temps ».

```rust
async fn read_user(id: i64) -> String {
    let row = fetch_from_db(id).await; // suspension possible ici
    format!("user {id}: {row}")
}
```

**Comparaison Java/TS.** C'est presque mot pour mot le `async`/`await` de TypeScript :

```typescript
async function readUser(id: number): Promise<string> {
  const row = await fetchFromDb(id);
  return `user ${id}: ${row}`;
}
```

Mais une différence est **cruciale**, et c'est le piège n°1 du débutant.

### 1.3 Les futures sont PARESSEUSES (différence majeure avec les Promises)

En TypeScript, dès que vous **appelez** une fonction `async`, elle **commence à s'exécuter**. La `Promise` est déjà « en vol » ; le `await` ne fait qu'attendre sa fin. Vous pouvez même appeler la fonction sans `await` du tout, le travail se fera quand même en arrière-plan.

En Rust, c'est l'inverse : **appeler une `async fn` ne lance rien**. Elle vous rend une future inerte, comme une recette de cuisine qu'on a écrite mais pas encore préparée. Rien ne se passe tant que :

- soit vous faites `.await` dessus (quelqu'un d'autre attend déjà votre résultat) ;
- soit vous la confiez à un **runtime** qui la fait avancer (`tokio::spawn`).

```rust
async fn hello() {
    println!("hello !");
}

#[tokio::main]
async fn main() {
    let future = hello(); // RIEN ne s'affiche : la future est inerte
    future.await;         // MAINTENANT « hello ! » s'affiche
}
```

C'est pour ça que tout ce module tourne autour d'un runtime : sans lui, vos `async fn` ne s'exécutent **jamais**. Le commentaire en tête de `main.rs` le résume :

```rust
// Sans runtime, un `async fn` ne s'exécute jamais : les futures sont paresseuses.
```

> **Erreur classique.** Vous appelez une `async fn` et « rien ne se passe », sans erreur de compilation. Cause : vous avez oublié le `.await`. Heureusement, le compilateur vous prévient presque toujours avec un avertissement `unused implementer of Future that must be used`. Lisez-le, c'est un cadeau.

---

## 2. Tokio : le runtime asynchrone

### 2.1 Pourquoi un runtime externe ?

Java et Node embarquent leur machinerie asynchrone (la JVM, l'event loop de V8). Rust, lui, est minimaliste : le langage définit *ce qu'est* une future (le trait `Future` de la bibliothèque standard) mais **ne fournit pas de moteur pour les exécuter**. C'est un choix : un serveur réseau et un micro-contrôleur embarqué n'ont pas les mêmes besoins, alors le langage laisse le choix du runtime.

**Tokio** est le runtime de référence pour le réseau. C'est lui qui :

- maintient un *executor* (la boucle qui fait avancer les futures prêtes) ;
- multiplexe les tâches sur un pool de threads (par défaut, un par cœur) ;
- gère les entrées/sorties non bloquantes (sockets, timers).

**Comparaison Java/TS.** Tokio est à Rust ce que l'event loop de libuv est à Node, ou ce que le scheduler de virtual threads est à la JVM moderne : l'infrastructure invisible qui décide quelle tâche s'exécute quand.

### 2.2 `#[tokio::main]`

**Définition.** L'attribut `#[tokio::main]` transforme votre `async fn main` en un `fn main` classique qui démarre le runtime Tokio, puis exécute votre future dessus.

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ... du code asynchrone ...
    Ok(())
}
```

En coulisses, c'est l'équivalent d'écrire :

```rust
fn main() -> anyhow::Result<()> {
    tokio::runtime::Runtime::new()?.block_on(async {
        // ... votre code ...
        Ok(())
    })
}
```

L'attribut fait juste ce passe-plat pour vous. C'est le pont entre le monde synchrone (`fn main`, le point d'entrée que le système d'exploitation appelle) et le monde asynchrone (votre code `async`).

Dans le projet, on demande explicitement un runtime multi-threadé via les features de Tokio dans `Cargo.toml` :

```toml
tokio = { version = "1.52.3", features = ["macros", "net", "rt-multi-thread"] }
```

- `macros` : active `#[tokio::main]` et `#[tokio::test]` ;
- `net` : le `TcpListener` asynchrone ;
- `rt-multi-thread` : le runtime qui répartit les tâches sur tous les cœurs.

### 2.3 Tâches vs threads : `tokio::spawn`

**Définition.** `tokio::spawn(future)` confie une future au runtime pour qu'elle s'exécute **en concurrence** des autres. On parle de **tâche** (*task*), pas de thread.

```rust
tokio::spawn(async {
    do_some_background_work().await;
});
```

**La différence clé avec `thread::spawn` du module 4 :**

| | `thread::spawn` (module 4) | `tokio::spawn` (ce module) |
|---|---|---|
| Coût de création | élevé (un thread OS, ~1 Mio de pile) | minuscule (quelques octets) |
| Combien ? | des dizaines | des centaines de milliers |
| Bloque-t-il à l'attente ? | oui, le thread reste figé | non, le runtime exécute autre chose |
| Idéal pour | du calcul lourd (CPU-bound) | de l'attente réseau/disque (I/O-bound) |

Vous n'écrirez pas vous-même de `tokio::spawn` dans le projet : **Axum le fait pour vous**. Chaque requête entrante devient une tâche. C'est précisément ce que dit le commentaire de `main.rs` :

```rust
// `axum::serve` accepte les connexions en boucle et confie chaque
// requête à une tâche Tokio. C'est notre `app.listen(3000)` d'Express.
axum::serve(listener, app).await?;
```

Comprendre le mécanisme reste utile : vous saurez *pourquoi* votre serveur tient la charge, et pourquoi il ne faut **jamais** faire de calcul long et bloquant directement dans un handler (cela gèlerait le thread du runtime et bloquerait d'autres tâches — pour ça, on utiliserait `tokio::task::spawn_blocking`).

---

## 3. Axum : le routeur et les handlers

**Axum** est un framework web bâti sur Tokio. Mentalement, c'est l'Express de Rust : on déclare des routes, chacune pointe vers une fonction (un *handler*), et le framework s'occupe du HTTP. Sa particularité — qu'on détaillera au §4 — est de tout faire passer par le **système de types**.

### 3.1 Le `Router`

**Définition.** Le `Router` associe une méthode HTTP + un chemin à un handler. C'est la table de routage de l'application.

**Comparaison Express.** En Express :

```javascript
const app = express();
app.route("/notes").get(listNotes).post(createNote);
app.route("/notes/:id").get(getNote).put(updateNote).delete(deleteNote);
```

En Axum, c'est l'`app(...)` de notre `lib.rs`, presque ligne pour ligne :

```rust
pub fn app(pool: SqlitePool) -> Router {
    Router::new()
        .route(
            "/notes",
            get(handlers::list_notes).post(handlers::create_note),
        )
        .route(
            "/notes/{id}",
            get(handlers::get_note)
                .put(handlers::update_note)
                .delete(handlers::delete_note),
        )
        .with_state(pool)
}
```

Trois choses à noter :

- `get(handler)` enregistre un handler pour la méthode GET. On enchaîne `.post(...)`, `.put(...)`, `.delete(...)` pour brancher plusieurs méthodes sur **le même chemin**.
- **Axum 0.8** écrit les paramètres de chemin entre **accolades** : `"/notes/{id}"` (les versions plus anciennes utilisaient `:id` comme Express — attention si vous lisez d'anciens tutoriels).
- `.with_state(pool)` rend le pool de base de données accessible à tous les handlers (voir §6).

### 3.2 Le handler

**Définition.** Un handler est une **`async fn`** dont les paramètres sont des *extracteurs* (ce qu'on lit dans la requête) et dont la valeur de retour se transforme en réponse HTTP.

Le plus petit handler possible (tiré de `examples/hello.rs`) :

```rust
async fn hello() -> &'static str {
    "Hello from Axum!"
}
```

Aucun paramètre (il ne lit rien dans la requête), et un retour `&'static str` qu'Axum sait transformer en réponse `200 OK` avec le bon `Content-Type`.

**Comparaison.** C'est le `(req, res) => { ... }` d'Express ou la méthode annotée `@GetMapping` d'un `@RestController` Spring. La différence est que vous ne touchez ni un objet `req` ni un objet `res` : vous **déclarez** ce qui vous intéresse en paramètres, et vous **renvoyez** une valeur. Tout le reste est automatique. C'est l'objet du §4.

---

## 4. Les extracteurs : la magie expliquée

C'est le concept le plus déroutant d'Axum quand on vient d'Express, et le plus élégant une fois compris.

### 4.1 L'idée

En Express, vous fouillez vous-même l'objet `req` : `req.params.id`, `req.body`, `req.query.q`. C'est à vous de savoir où regarder et de gérer le cas où ce n'est pas là.

En Axum, vous **déclarez en paramètre le type de la chose que vous voulez**, et le framework va la chercher, la valider, la convertir — ou répond une erreur tout seul si elle est absente ou mal formée. Chacun de ces paramètres est un **extracteur**.

Regardez `update_note` dans `handlers.rs` :

```rust
pub async fn update_note(
    State(pool): State<SqlitePool>,   // → le pool, depuis le state de l'app
    Path(id): Path<i64>,              // → l'{id} de l'URL, parsé en i64
    Json(input): Json<NoteInput>,     // → le corps JSON, désérialisé en NoteInput
) -> Result<Json<Note>, ApiError> {
    // ...
}
```

Trois extracteurs, trois sources différentes, zéro ligne de plomberie. Comparez avec l'équivalent Express, où tout est manuel et non typé :

```javascript
async function updateNote(req, res) {
  const id = parseInt(req.params.id, 10);     // et si ce n'est pas un nombre ?
  const input = req.body;                      // et si le JSON est invalide ?
  // ... à vous de valider tout ça ...
}
```

### 4.2 Le catalogue des extracteurs du projet

| Extracteur | Source | Équivalent Express | Échec géré par Axum |
|---|---|---|---|
| `Path<i64>` | le `{id}` de l'URL | `req.params.id` | `/notes/abc` → `400` automatique |
| `Query<T>` | la *query string* `?q=...` | `req.query` | type invalide → `400` (utilisé dans les défis) |
| `Json<T>` | le corps de la requête | `req.body` (avec `express.json()`) | JSON invalide → `4xx` automatique |
| `State<T>` | le state de l'application | un singleton injecté | — (toujours présent) |

**Le bénéfice énorme :** si le client envoie `/notes/abc` ou un corps JSON cassé, **votre handler n'est même pas appelé**. Axum a déjà renvoyé une erreur 4xx. Vous ne codez que le chemin heureux. Le commentaire de `create_note` le souligne :

```rust
// `Json<NoteInput>` désérialise le corps de la requête. Si le JSON est
// invalide ou qu'un champ manque, Axum répond 4xx tout seul : le handler
// n'est même pas appelé. Comparable à `@Valid @RequestBody` en Spring.
```

C'est exactement ce que vérifie un de nos tests, `create_note_with_invalid_body_returns_client_error` : il envoie « ceci n'est pas du JSON » et s'attend à un statut 4xx.

### 4.3 La règle d'or : l'ordre des extracteurs

Un seul piège : **l'extracteur qui consomme le corps de la requête (`Json`) doit toujours être le DERNIER paramètre**. Pourquoi ? Le corps d'une requête HTTP est un flux qu'on ne peut lire qu'une fois ; Axum applique les extracteurs dans l'ordre, et celui qui « mange » le corps doit donc passer en dernier. Le commentaire de `update_note` le rappelle :

```rust
// Attention à l'ordre des extracteurs : celui qui consomme le corps de
// la requête (`Json`) doit toujours être le DERNIER paramètre.
```

`State` et `Path` ne consomment pas le corps : ils peuvent venir avant, dans l'ordre que vous voulez.

### 4.4 Comment ça marche (en une phrase)

La « magie » est un trait : `FromRequestParts` (ou `FromRequest` pour ceux qui lisent le corps). Chaque type extracteur implémente ce trait, qui dit « voici comment me fabriquer à partir de la requête, et voici l'erreur si je n'y arrive pas ». Axum, via un peu de génie des génériques, accepte n'importe quelle fonction dont **tous** les paramètres sont des extracteurs. Vous n'aurez pas à écrire d'extracteur vous-même dans ce module ; il suffit de savoir que c'est *du code, pas de la magie* — et que c'est vérifié à la compilation.

---

## 5. `IntoResponse` : transformer une valeur en réponse HTTP

### 5.1 Le trait

**Définition.** `IntoResponse` est le trait qui dit « voici comment me transformer en réponse HTTP ». Le type de retour d'un handler doit l'implémenter. La bibliothèque le fournit déjà pour les types courants, ce qui explique pourquoi des retours très différents fonctionnent tous :

| Type de retour | Réponse produite |
|---|---|
| `&'static str`, `String` | `200` + corps texte |
| `Json<T>` | `200` + corps JSON (sérialisé par serde) |
| `(StatusCode, Json<T>)` | le statut choisi + corps JSON |
| `StatusCode` | un statut seul, sans corps |
| `Result<T, E>` où `T` et `E` implémentent `IntoResponse` | la réponse de `T` ou celle de `E` |

**Comparaison.** En Express, vous *mutez* l'objet `res` : `res.status(201).json(note)`. En Axum, vous **renvoyez une valeur** et Axum la convertit. C'est plus déclaratif et plus testable : un handler est une fonction pure de la requête vers une valeur.

### 5.2 Choisir un statut autre que 200

Pour renvoyer un `201 Created`, on retourne un **tuple** `(StatusCode, Json<...>)`. Vu dans `create_note` :

```rust
// Renvoyer un tuple `(StatusCode, Json<...>)` est la façon idiomatique
// de choisir le statut : ici 201 Created au lieu du 200 par défaut.
Ok((StatusCode::CREATED, Json(note)))
```

Et pour un `204 No Content` (réponse sans corps), on renvoie juste un `StatusCode`, comme dans `delete_note` :

```rust
Ok(StatusCode::NO_CONTENT)
```

### 5.3 Le cas `Result`

La plupart de nos handlers renvoient `Result<T, ApiError>`. Pourquoi est-ce que ça marche ? Parce que **`Result<T, E>` implémente `IntoResponse`** dès lors que `T` et `E` l'implémentent : en cas de `Ok(t)`, Axum utilise la réponse de `t` ; en cas de `Err(e)`, celle de `e`. C'est ce qui nous permet d'utiliser l'opérateur `?` librement dans les handlers et de laisser notre type d'erreur décider du code de statut. C'est l'objet du §8 — mais notez déjà l'élégance : la gestion d'erreur n'est pas un `try/catch` éparpillé, c'est le système de types qui la porte.

---

## 6. Le state partagé : injecter le pool de connexions

### 6.1 Le problème

Tous les handlers ont besoin d'accéder à la base de données. Il faut donc leur partager **un seul** pool de connexions, créé une fois au démarrage. C'est le même besoin qu'au module 4 (un état partagé entre plusieurs unités d'exécution), mais ici les unités sont des tâches Tokio (une par requête).

**Comparaison.** C'est l'injection de dépendances : le `@Autowired DataSource` de Spring, ou le `db` que vous passez en clôture à vos routes Express. La différence est qu'en Axum c'est **explicite et typé**.

### 6.2 `with_state` et l'extracteur `State`

On attache le state au routeur avec `.with_state(pool)` (vu au §3.1), et les handlers le réclament avec l'extracteur `State<T>` :

```rust
pub async fn list_notes(State(pool): State<SqlitePool>) -> Result<Json<Vec<Note>>, ApiError> {
    // `pool` est ici une copie (bon marché) du pool partagé
}
```

Le state doit être `Clone`, car Axum en donne une copie à chaque requête.

### 6.3 Pourquoi PAS de `Mutex` autour du pool ?

Au module 4, partager un `Vec` mutable imposait un `Arc<Mutex<Vec<...>>>` : le `Mutex` sérialisait les accès pour éviter les data races. On pourrait croire qu'il faut faire pareil ici.

**Non — et c'est une leçon importante.** Le `SqlitePool` est *déjà* conçu pour le partage concurrent :

- en interne, il contient déjà un `Arc` : le cloner ne duplique pas les connexions, il incrémente juste un compteur, exactement comme au module 4 ;
- il gère lui-même un **pool** de connexions et distribue à chaque requête une connexion libre. La synchronisation est encapsulée *à l'intérieur* du pool.

Mettre un `Mutex` par-dessus serait non seulement inutile, mais **catastrophique pour les performances** : il sérialiserait toutes les requêtes (une seule à la fois), réduisant à néant l'intérêt d'un pool. Le commentaire de `lib.rs` le dit :

```rust
// Le pool est déjà un `Arc` en interne :
// le cloner est bon marché, pas besoin de l'envelopper nous-mêmes.
```

> **Règle générale.** Un type qui se documente comme *cheap to clone* / *Clone is cheap* contient déjà un `Arc` et gère sa propre synchronisation. Ne le mettez pas dans un `Mutex` : partagez-le directement. On ne met un `Mutex` que pour protéger une donnée *qui n'est pas déjà thread-safe* — comme le `Vec` du `memory_api.rs`, l'étape jetable du projet.

> **Pont avec le module 4.** L'exemple `examples/memory_api.rs` utilise justement un `Arc<Mutex<Vec<Note>>>` comme « base de données » : c'est volontaire, pour réviser le module 4 avant d'introduire SQLx. Là, le `Mutex` est nécessaire car un `Vec` nu n'est pas partageable. Une subtilité y est rappelée : ne **jamais** garder un `MutexGuard` de `std` à travers un `.await` (on clone les données pour sortir du verrou tout de suite).

---

## 7. SQLx + SQLite : parler à la base

### 7.1 Le pool de connexions

**Définition.** Un *pool* maintient un petit nombre de connexions ouvertes et les recycle. Ouvrir une connexion à une base est coûteux ; on ne veut pas le faire à chaque requête HTTP.

Notre `db.rs` le construit dans `init_db` :

```rust
let options = SqliteConnectOptions::from_str(url)?.create_if_missing(true);

let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect_with(options)
    .await?;
```

- `create_if_missing(true)` crée le fichier `notes.db` au premier lancement (sans ça, SQLx refuse d'ouvrir une base inexistante) ;
- `max_connections(5)` : cinq connexions suffisent largement pour SQLite (base embarquée, un seul écrivain à la fois). L'essentiel est le **principe** : on réutilise les connexions au lieu d'en ouvrir une par requête.

**Comparaison.** C'est le `DataSource` HikariCP de Spring, ou un pool `pg`/`mysql2` en Node.

### 7.2 Créer le schéma au démarrage

Plutôt qu'un outil de migration, le projet crée la table au lancement, de façon **idempotente** :

```rust
sqlx::query(
    r#"
    CREATE TABLE IF NOT EXISTS notes (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        title       TEXT NOT NULL,
        content     TEXT NOT NULL,
        created_at  TEXT NOT NULL
                    DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
    )
    "#,
)
.execute(pool)
.await?;
```

Le `IF NOT EXISTS` rend l'opération sûre à relancer : c'est la version artisanale de Flyway/Liquibase (Java) ou de `sqlx migrate`. Astuce élégante : `created_at` reçoit une valeur par défaut **calculée par SQLite** via `strftime`, qui produit directement le format ISO 8601 du contrat — pas besoin de gérer les dates côté Rust.

### 7.3 `query` vs `query_as`, à liaison à l'exécution

SQLx propose deux familles de fonctions, et nous utilisons les versions **à liaison à l'exécution** (*runtime-bound*) :

- `sqlx::query("...")` : exécute une requête qui ne renvoie pas de lignes typées (un `INSERT`, un `DELETE`). On l'utilise pour `delete_note` et pour `CREATE TABLE`. Elle renvoie des infos d'exécution, dont `.rows_affected()`.
- `sqlx::query_as::<_, Note>("...")` : exécute une requête et **mappe chaque ligne** vers une struct (ici `Note`) grâce au derive `sqlx::FromRow`. C'est notre `SELECT`, notre `INSERT ... RETURNING`, notre `UPDATE ... RETURNING`.

Exemple complet, `get_note` :

```rust
let note = sqlx::query_as::<_, Note>(
    "SELECT id, title, content, created_at FROM notes WHERE id = ?",
)
.bind(id)
.fetch_optional(&pool)
.await?
.ok_or(ApiError::NotFound(id))?;
```

Deux points :

- **`.bind(id)`** lie une valeur à chaque `?`. Ce sont des **paramètres préparés** : aucune injection SQL possible, exactement comme un `PreparedStatement` JDBC. **Ne concaténez jamais** une valeur dans la chaîne SQL.
- Les **finisseurs** choisissent la forme du résultat :

| Finisseur | Renvoie | Cas d'usage |
|---|---|---|
| `.fetch_all(&pool)` | `Vec<Note>` | lister (`list_notes`) |
| `.fetch_one(&pool)` | `Note` (erreur si 0 ligne) | insérer avec `RETURNING` (`create_note`) |
| `.fetch_optional(&pool)` | `Option<Note>` (`None` si 0 ligne) | chercher par id (`get_note`, `update_note`) |
| `.execute(&pool)` | infos d'exécution (`rows_affected`) | supprimer (`delete_note`) |

### 7.4 L'astuce `RETURNING`

SQLite (≥ 3.35) sait renvoyer la ligne affectée par un `INSERT`/`UPDATE`. On en profite pour récupérer l'`id` auto-incrémenté et le `created_at` calculé par la base **en une seule requête**, sans second `SELECT` :

```rust
let note = sqlx::query_as::<_, Note>(
    "INSERT INTO notes (title, content) VALUES (?, ?)
     RETURNING id, title, content, created_at",
)
.bind(&input.title)
.bind(&input.content)
.fetch_one(&pool)
.await?;
```

Et pour `update_note`, le même `RETURNING` combiné à `fetch_optional` gère le 404 gratuitement : si l'`id` n'existe pas, l'`UPDATE` n'affecte aucune ligne, `RETURNING` ne renvoie rien, `fetch_optional` donne `None` → on en fait un 404.

### 7.5 Et les fameuses macros `query!` ?

Vous croiserez en ligne les macros `sqlx::query!` et `sqlx::query_as!` (avec un **`!`**). Elles ont un superpouvoir : elles se connectent à une *vraie base* **pendant la compilation** pour vérifier que votre SQL est correct et que les types des colonnes correspondent à votre struct. Une faute de frappe dans un nom de colonne devient une erreur de compilation. Magnifique — mais cela impose une base accessible au moment du `cargo build`, configurée via une variable d'environnement `DATABASE_URL` (ou un cache `.sqlx` à committer).

**Nous avons délibérément choisi les fonctions sans `!`** (`query`, `query_as`), à liaison à l'exécution. Elles vérifient le SQL au moment où il s'exécute, pas à la compilation. On y perd la vérification statique, mais on y gagne une simplicité énorme : aucun `DATABASE_URL` à configurer, le projet compile et se teste avec une base en mémoire, sans aucune étape préalable. C'est le bon compromis pour apprendre. Une fois à l'aise, les macros vaudront le détour pour un vrai projet.

---

## 8. Gestion d'erreurs HTTP : du domaine au code de statut

### 8.1 Le type d'erreur du domaine

On rassemble toutes les façons d'échouer dans un seul enum, avec `thiserror` (revu au module 3) :

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("note {0} not found")]
    NotFound(i64),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

Le `#[from]` sur `Database` génère `impl From<sqlx::Error> for ApiError`. C'est lui qui rend l'opérateur **`?`** utilisable sur les appels SQLx : une `sqlx::Error` est automatiquement convertie en `ApiError::Database`. Comme une exception qui remonte en Java — mais **visible dans la signature** de la fonction, et impossible à oublier.

### 8.2 Le pont vers HTTP : `impl IntoResponse for ApiError`

C'est la pièce maîtresse du module. Cet `impl` associe un **code de statut HTTP** à chaque variante d'erreur :

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,            // 404
            ApiError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR, // 500
        };

        let body = Json(json!({ "error": self.to_string() }));

        (status, body).into_response()
    }
}
```

**Comparaison.** C'est l'équivalent exact d'un `@ControllerAdvice` + `@ExceptionHandler` de Spring, ou du middleware d'erreur d'Express (`app.use((err, req, res, next) => ...)`). La supériorité de Rust : le `match` est **exhaustif**. Ajoutez une variante à `ApiError` sans l'y traiter, et le code **ne compile pas**. Impossible d'oublier de gérer un cas d'erreur — le compilateur monte la garde.

### 8.3 Le résultat : des handlers limpides

Grâce à tout ça, un handler ne décrit que le chemin heureux et laisse le `?` propager les erreurs :

```rust
let note = sqlx::query_as::<_, Note>("SELECT ... WHERE id = ?")
    .bind(id)
    .fetch_optional(&pool)
    .await?               // erreur SQL → ApiError::Database → 500
    .ok_or(ApiError::NotFound(id))?; // None → ApiError::NotFound → 404
Ok(Json(note))
```

Une erreur SQL devient un 500, une note absente un 404, le tout en deux lignes, sans un seul `if err != null` ni `try/catch`. Le `delete_note` montre le cas particulier où l'absence n'est *pas* une erreur SQL : un `DELETE` qui ne supprime rien réussit côté SQL, il faut donc vérifier `rows_affected() == 0` soi-même pour distinguer le 204 du 404.

---

## 9. Tester une API sans lancer de serveur

### 9.1 Pourquoi `lib.rs` ET `main.rs` ?

Comme au module 3, le projet sépare la **bibliothèque** (`lib.rs` : tout le code, dont la fonction `app()`) du **binaire** (`main.rs` : juste le branchement réseau final). Cette séparation permet aux tests de construire l'application — `notes_api::app(pool)` — **sans ouvrir de port ni démarrer de serveur**.

### 9.2 `tower::ServiceExt::oneshot`

**Définition.** Un `Router` Axum est en réalité un *service* `tower` : une fonction qui prend une requête et renvoie une réponse. La méthode `oneshot` lui envoie **une** requête directement, en mémoire, et renvoie la réponse — sans réseau.

```rust
let response = app.clone().oneshot(request).await.expect("request failed");
let status = response.status();
```

**Comparaison.** C'est exactement `supertest` en Node ou `MockMvc` en Spring : on teste l'application au niveau HTTP, mais en mémoire, donc **rapide et isolé**. Pas de port à réserver, pas de course entre tests parallèles.

### 9.3 Une base en mémoire pour les tests

Chaque test construit une application sur une base SQLite **en mémoire** (`sqlite::memory:`), vierge à chaque fois :

```rust
async fn test_app() -> Router {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)               // crucial, voir ci-dessous
        .connect("sqlite::memory:")
        .await
        .expect("failed to open in-memory database");

    notes_api::db::create_schema(&pool).await.expect("failed to create schema");
    notes_api::app(pool)
}
```

> **Le piège `max_connections(1)`.** Avec `sqlite::memory:`, *chaque connexion* ouvre sa **propre** base vide. Si le pool en ouvrait plusieurs, un `INSERT` et le `SELECT` suivant pourraient tomber sur deux bases différentes : le test verrait des données fantômes. Limiter à une seule connexion garantit que tout le test partage la même base.

Les tests (dans `tests/api.rs`) vérifient le **contrat du cours** route par route : `200` + `[]` sur une base vide, `201` + note complète à la création, `404` sur un id inconnu, `204` à la suppression, etc. S'ils passent, le module 6 fonctionnera.

---

## 10. Récapitulatif : ce que le projet utilise

| Concept | Où, dans le projet | À retenir |
|---|---|---|
| `async`/`await`, future paresseuse | partout | une future ne s'exécute que sous un runtime ou sur `.await` |
| `#[tokio::main]`, runtime | `main.rs`, `Cargo.toml` | démarre Tokio, exécute votre `async main` |
| tâche Tokio | `axum::serve` (implicite) | une par requête ; légère, faite pour l'attente I/O |
| `Router`, routes | `lib.rs` → `app()` | `get(h).post(h)`, chemin `{id}` en Axum 0.8 |
| handler | `handlers.rs` | `async fn`, extracteurs en entrée, valeur convertible en réponse |
| extracteurs `Path`/`Json`/`State` | `handlers.rs` | déclaratifs ; `Json` toujours en dernier ; échec → 4xx auto |
| `IntoResponse` | `error.rs`, retours de handlers | `Json`, `(StatusCode, Json)`, `Result<T, ApiError>` |
| state partagé | `lib.rs` (`with_state`), `handlers.rs` (`State`) | le pool est déjà un `Arc`, pas de `Mutex` |
| pool SQLx, `query_as` à liaison à l'exécution | `db.rs`, `handlers.rs` | `.bind()` (anti-injection), finisseurs `fetch_*`/`execute` |
| schéma au démarrage | `db.rs` → `create_schema` | `CREATE TABLE IF NOT EXISTS`, idempotent |
| gestion d'erreur HTTP | `error.rs` | enum domaine + `IntoResponse` → 404/500, `match` exhaustif |
| tests d'API | `tests/api.rs` | `oneshot` + base en mémoire `max_connections(1)` |

Vous avez tous les concepts. Direction `PROJET.md` pour les assembler étape par étape, puis `DEFIS.md` pour aller plus loin.
