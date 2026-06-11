# Projet 5 — API REST de gestion de notes

Vous allez construire un **serveur HTTP** qui expose une API REST pour gérer des notes, persistées dans une base SQLite. C'est le genre de backend que vous écririez avec Express ou Spring Boot — mais en Rust, asynchrone, et vérifié par le compilateur de bout en bout.

L'API respecte le **contrat du cours**, que le module 6 (app desktop Tauri) viendra consommer telle quelle :

| Méthode & chemin | Corps attendu | Réponse |
|---|---|---|
| `GET /notes` | — | `200` + liste de notes |
| `POST /notes` | `{"title": string, "content": string}` | `201` + note créée |
| `GET /notes/{id}` | — | `200` + note, ou `404` |
| `PUT /notes/{id}` | `{"title": string, "content": string}` | `200` + note mise à jour, ou `404` |
| `DELETE /notes/{id}` | — | `204` sans corps, ou `404` |

Forme d'une note : `{"id": i64, "title": string, "content": string, "created_at": string ISO 8601}`.

**Règles du jeu.** Le squelette est dans `projet/`, le corrigé dans `solution/` (à ne regarder qu'après avoir vraiment essayé). Les tests d'intégration de `tests/api.rs` sont **fournis** : votre travail est de les faire passer **sans les modifier**. Deux exemples complets (`examples/hello.rs` et `examples/memory_api.rs`) accompagnent les premières étapes. Tout le texte affiché est en français, tous les identifiants de code en anglais.

> **Avant de commencer**, préparez votre environnement :
> ```bash
> source $HOME/.cargo/env
> export CARGO_BUILD_JOBS=4   # limite la charge de compilation
> ```

---

## Étape 0 — Prise en main du squelette

**Objectif.** Comprendre la structure du projet et vérifier que tout compile.

**Pourquoi.** Comme aux modules 3 et 4, le projet est découpé en bibliothèque (`src/lib.rs`, testable) et binaire (`src/main.rs`, le branchement réseau). Cette séparation permet aux tests de construire l'application sans lancer de vrai serveur (revoir COURS.md §9.1).

```text
projet/
├── Cargo.toml
├── src/
│   ├── lib.rs        ← la fonction app() : le routeur (étape 4)
│   ├── main.rs       ← FOURNI : le branchement réseau final (lisez-le)
│   ├── models.rs     ← FOURNI : les types Note et NoteInput
│   ├── db.rs         ← connexion SQLite et schéma (étape 3)
│   ├── error.rs      ← le type d'erreur ApiError (étape 5)
│   └── handlers.rs   ← un handler par route (étapes 4 et 5)
├── examples/
│   ├── hello.rs      ← FOURNI : le plus petit serveur Axum (étape 1)
│   └── memory_api.rs ← FOURNI : l'API en mémoire (étape 2)
└── tests/
    └── api.rs        ← tests fournis, NE PAS modifier (le contrat d'API)
```

**À faire.** Lisez `src/models.rs`, `src/main.rs` et `tests/api.rs`, puis :

```bash
cd projet
cargo test --no-run    # doit compiler (le squelette utilise des todo!())
```

**Point de contrôle.** `cargo test --no-run` compile sans erreur. Les `todo!()` sont des trous à combler : tant qu'ils sont là, le code compile mais les tests paniqueraient à l'exécution. C'est votre liste de courses, étapes 3 à 5.

---

## Étape 1 — Le plus petit serveur possible

**Objectif.** Lancer un serveur Axum qui répond sur une route, et le tester à la main.

**Pourquoi.** Avant de toucher à la base ou au JSON, sentez le squelette d'un serveur : un `Router`, un handler, un `TcpListener`, `axum::serve`. C'est `examples/hello.rs`, **fourni complet** : votre seul travail ici est de le lancer et d'observer.

**À faire.**

```bash
cargo run --example hello
```

Dans un autre terminal :

```bash
curl http://localhost:3000/
# → Hello from Axum!
```

**Indices à comprendre dans le code.**

- `async fn hello() -> &'static str` : un handler sans extracteur (il ne lit rien dans la requête). Son retour implémente `IntoResponse` (COURS.md §3.2, §5.1).
- `Router::new().route("/", get(hello))` : une route GET.
- `#[tokio::main]` démarre le runtime (COURS.md §2.2). Sans lui, rien ne s'exécuterait.

**Point de contrôle.** `curl` répond `Hello from Axum!`. Coupez le serveur avec Ctrl+C.

---

## Étape 2 — L'API de notes EN MÉMOIRE

**Objectif.** Manipuler les extracteurs (`State`, `Json`) et un state partagé, avant d'introduire SQLx.

**Pourquoi.** On isole une difficulté à la fois. Cette étape utilise les outils du **module 4** (`Arc<Mutex<Vec<Note>>>`) comme « base de données » jetable, pour pratiquer Axum sans le bruit de SQL. C'est `examples/memory_api.rs`, **fourni complet** : lisez-le attentivement, c'est le pont entre le module 4 et ce module.

**À faire.**

```bash
cargo run --example memory_api
```

Testez les deux routes à la main :

```bash
# Lister (vide au départ)
curl http://localhost:3000/notes
# → []

# Créer une note
curl -X POST http://localhost:3000/notes \
     -H 'Content-Type: application/json' \
     -d '{"title":"Essai","content":"En mémoire"}'
# → {"id":1,"title":"Essai","content":"En mémoire"}

# Re-lister
curl http://localhost:3000/notes
# → [{"id":1,"title":"Essai","content":"En mémoire"}]
```

**Indices à comprendre dans le code.**

- `type Db = Arc<Mutex<Vec<Note>>>` : le motif exact du module 4. Le `Mutex` est ici **nécessaire** car un `Vec` nu n'est pas partageable entre tâches (à l'étape 3, le `SqlitePool` rendra ce `Mutex` inutile — COURS.md §6.3).
- `State(db): State<Db>` : l'extracteur `State` récupère la « base » partagée.
- `Json(input): Json<NoteInput>` : l'extracteur `Json` désérialise le corps.
- Le retour `(StatusCode::CREATED, Json(note))` produit un `201` (COURS.md §5.2).
- Détail crucial commenté dans le code : on clone les données pour **sortir du verrou avant** tout `.await` — ne jamais garder un `MutexGuard` de `std` à travers un `.await`.

**Point de contrôle.** Les trois `curl` se comportent comme ci-dessus. Notez que les notes **disparaissent** à l'arrêt du serveur : c'est tout l'intérêt de passer à SQLite ensuite. Ctrl+C.

---

## Étape 3 — Brancher SQLite avec SQLx

**Objectif.** Implémenter `init_db` et `create_schema` dans `src/db.rs`.

**Pourquoi.** On remplace le `Vec` en mémoire par une vraie base persistante. Deux responsabilités : ouvrir (ou créer) la base avec un **pool** de connexions, et créer la table `notes` au démarrage de façon idempotente (COURS.md §7.1, §7.2).

**À faire.** Remplacez les deux `todo!()` de `db.rs`.

**Indices.**

- `init_db`, dans l'ordre :
  1. `let options = SqliteConnectOptions::from_str(url)?.create_if_missing(true);` — le `create_if_missing` est indispensable, sinon SQLx refuse une base inexistante ;
  2. `let pool = SqlitePoolOptions::new().max_connections(5).connect_with(options).await?;`
  3. appelez `create_schema(&pool).await?;` **avant** de renvoyer `Ok(pool)`.
- `create_schema` : une seule requête `sqlx::query("CREATE TABLE IF NOT EXISTS notes (...)").execute(pool).await?;`. Les colonnes (recopiez-les soigneusement, les tests en dépendent) :

  ```sql
  CREATE TABLE IF NOT EXISTS notes (
      id          INTEGER PRIMARY KEY AUTOINCREMENT,
      title       TEXT NOT NULL,
      content     TEXT NOT NULL,
      created_at  TEXT NOT NULL
                  DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
  )
  ```

  Le `DEFAULT (strftime(...))` fait calculer le `created_at` au format ISO 8601 par SQLite elle-même — vous n'aurez jamais à gérer de date en Rust.

**Point de contrôle.** `cargo build` compile. Vous ne pouvez pas encore tester ces fonctions seules ; elles seront exercées dès que les handlers existeront (étape 4).

---

## Étape 4 — Le routeur et les premiers handlers (GET et POST)

**Objectif.** Écrire la fonction `app()` (dans `lib.rs`) et les handlers `list_notes` et `create_note` (dans `handlers.rs`).

**Pourquoi.** C'est le cœur du projet : assembler le routeur et brancher les deux routes les plus simples. Vous y pratiquez les extracteurs, `query_as` à liaison à l'exécution, et le choix du code de statut.

**À faire — d'abord le routeur** (`lib.rs`, remplacez le `todo!()`) :

```rust
Router::new()
    .route("/notes", get(handlers::list_notes).post(handlers::create_note))
    .route(
        "/notes/{id}",
        get(handlers::get_note)
            .put(handlers::update_note)
            .delete(handlers::delete_note),
    )
    .with_state(pool)
```

Vous référencez `get_note`, `update_note`, `delete_note` qui contiennent encore des `todo!()` : c'est normal, ils compilent (un `todo!()` a le type voulu). Vous les remplirez à l'étape 5.

**À faire — `list_notes`.** Un `SELECT` qui rend toutes les notes triées :

```rust
let notes = sqlx::query_as::<_, Note>(
    "SELECT id, title, content, created_at FROM notes ORDER BY id",
)
.fetch_all(&pool)
.await?;
Ok(Json(notes))
```

Le `?` final convertit une éventuelle `sqlx::Error` en `ApiError` (via le `#[from]`, déjà fourni dans `error.rs`).

**À faire — `create_note`.** Un `INSERT ... RETURNING` qui rend la ligne créée, avec un statut `201` :

```rust
let note = sqlx::query_as::<_, Note>(
    "INSERT INTO notes (title, content) VALUES (?, ?)
     RETURNING id, title, content, created_at",
)
.bind(&input.title)
.bind(&input.content)
.fetch_one(&pool)
.await?;
Ok((StatusCode::CREATED, Json(note)))
```

**Indices.**

- `query_as::<_, Note>` mappe les colonnes vers `Note` grâce au derive `FromRow` (déjà présent dans `models.rs`). Les noms de colonnes du `SELECT`/`RETURNING` doivent correspondre aux champs.
- `.bind(...)` lie les valeurs aux `?` : paramètres préparés, anti-injection (COURS.md §7.3). N'insérez **jamais** une valeur en concaténant la chaîne SQL.
- `RETURNING` récupère l'`id` auto-incrémenté et le `created_at` calculé par la base, sans second `SELECT` (COURS.md §7.4).

**Point de contrôle.**

```bash
cargo test create_note      # les tests POST passent
cargo test list_notes       # les tests GET /notes passent
```

Si `list_notes_empty_returns_200_and_empty_array` et `create_note_returns_201_and_full_note` passent, vos deux premières routes sont bonnes.

---

## Étape 5 — Les routes par id et la gestion d'erreurs (GET/{id}, PUT, DELETE)

**Objectif.** Implémenter l'`impl IntoResponse for ApiError` (dans `error.rs`) puis les trois handlers restants (`get_note`, `update_note`, `delete_note`).

**Pourquoi.** C'est ici qu'apparaît la traduction « erreur du domaine → code HTTP », la pièce maîtresse du module (COURS.md §8). Une note absente doit devenir un `404`, une erreur SQL un `500`, le tout sans plomberie dans les handlers.

**À faire — `IntoResponse`** (`error.rs`, remplacez le `todo!()`) :

```rust
fn into_response(self) -> Response {
    let status = match &self {
        ApiError::NotFound(_) => StatusCode::NOT_FOUND,
        ApiError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = Json(json!({ "error": self.to_string() }));
    (status, body).into_response()
}
```

Le `match` est **exhaustif** : si vous ajoutiez une variante à `ApiError` sans la traiter, le code ne compilerait pas (COURS.md §8.2).

**À faire — `get_note`.** `fetch_optional` + `ok_or` pour le 404 :

```rust
let note = sqlx::query_as::<_, Note>(
    "SELECT id, title, content, created_at FROM notes WHERE id = ?",
)
.bind(id)
.fetch_optional(&pool)
.await?
.ok_or(ApiError::NotFound(id))?;
Ok(Json(note))
```

`fetch_optional` rend `Option<Note>` ; `.ok_or(ApiError::NotFound(id))?` transforme le `None` en 404.

**À faire — `update_note`.** Même idiome, avec `UPDATE ... RETURNING` :

```rust
let note = sqlx::query_as::<_, Note>(
    "UPDATE notes SET title = ?, content = ? WHERE id = ?
     RETURNING id, title, content, created_at",
)
.bind(&input.title)
.bind(&input.content)
.bind(id)
.fetch_optional(&pool)
.await?
.ok_or(ApiError::NotFound(id))?;
Ok(Json(note))
```

Si l'`id` n'existe pas, l'`UPDATE` n'affecte aucune ligne, `RETURNING` ne renvoie rien, `fetch_optional` donne `None` → 404. Élégant : le 404 est gratuit.

**À faire — `delete_note`.** Ici, l'absence n'est PAS une erreur SQL, il faut vérifier `rows_affected` :

```rust
let result = sqlx::query("DELETE FROM notes WHERE id = ?")
    .bind(id)
    .execute(&pool)
    .await?;
if result.rows_affected() == 0 {
    return Err(ApiError::NotFound(id));
}
Ok(StatusCode::NO_CONTENT)
```

Un `DELETE` qui ne supprime rien réussit côté SQL : c'est à vous de distinguer 204 de 404 (COURS.md §8.3).

**À faire — le nettoyage final.** Retirez la ligne `#![allow(unused_variables, unused_imports)]` en tête de `lib.rs` et corrigez les éventuels avertissements (il ne devrait plus y en avoir une fois tous les `todo!()` remplacés).

**Point de contrôle.**

```bash
cargo test    # LES 10 TESTS PASSENT
```

Attendez-vous à voir les 10 tests verts, dont `get_unknown_note_returns_404`, `update_note_returns_200_with_updated_note`, `delete_note_returns_204_then_note_is_gone`. Si l'un échoue, lisez son nom : il vous dit exactement quelle route revoir.

---

## Étape 6 — Le vrai serveur, testé à la main

**Objectif.** Lancer le binaire complet et exercer toutes les routes du contrat avec `curl`.

**Pourquoi.** Les tests passent en mémoire ; vérifions le serveur réel, avec un vrai fichier `notes.db` persistant. C'est aussi exactement ce que fera le module 6.

**À faire.**

```bash
cargo run
# → Notes API listening on http://localhost:3000
```

Dans un autre terminal, parcourez tout le contrat :

```bash
# 1. Lister (vide au premier lancement)
curl -s http://localhost:3000/notes
# → []

# 2. Créer
curl -s -X POST http://localhost:3000/notes \
     -H 'Content-Type: application/json' \
     -d '{"title":"Courses","content":"Acheter du pain"}'
# → {"id":1,"title":"Courses","content":"Acheter du pain","created_at":"2026-...T...Z"}

# 3. Récupérer par id
curl -s http://localhost:3000/notes/1
# → la même note

# 4. Mettre à jour
curl -s -X PUT http://localhost:3000/notes/1 \
     -H 'Content-Type: application/json' \
     -d '{"title":"Courses","content":"Acheter du pain et du lait"}'
# → la note mise à jour (même id, même created_at)

# 5. Supprimer (204, pas de corps)
curl -s -o /dev/null -w "%{http_code}\n" -X DELETE http://localhost:3000/notes/1
# → 204

# 6. Récupérer une note inconnue → 404
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:3000/notes/999
# → 404
```

> **Astuce.** `-w "%{http_code}\n"` affiche le code de statut HTTP, utile pour les réponses sans corps (204) ou les erreurs (404). `-o /dev/null` jette le corps.

**Point de contrôle.** Les six commandes se comportent comme indiqué. Coupez le serveur (Ctrl+C). Un fichier `notes.db` a été créé : relancez `cargo run` et faites `curl http://localhost:3000/notes` — vos données d'avant la suppression ont disparu, mais celles que vous n'auriez pas supprimées **persisteraient** (c'est la différence avec l'étape 2 en mémoire). Vous pouvez supprimer `notes.db` pour repartir d'une base vierge.

---

## C'est fini

Vous avez une API REST complète, asynchrone, persistée, testée, qui respecte le contrat du cours. Comparez votre code avec `solution/` : notez comment les commentaires y expliquent chaque choix.

Quand tout fonctionne, ouvrez `DEFIS.md` pour étendre l'API (pagination et recherche, journalisation, authentification).
