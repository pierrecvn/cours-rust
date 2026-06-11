# Module 6 — Application desktop avec Tauri 2 : le pont TypeScript ↔ Rust

Au module 5, vous avez écrit une API REST en Rust (Axum + SQLx). Elle tourne, mais elle n'a pas de visage : on la teste avec `curl`. Ce module lui donne une interface graphique — une vraie application de bureau, installable, qui s'ouvre dans une fenêtre. Et pour ça vous n'allez pas réécrire votre frontend en Rust : vous allez l'écrire en **TypeScript** (votre zone de confort), et faire dialoguer ce frontend avec un **backend Rust** grâce à **Tauri**.

C'est le module charnière entre vos deux mondes : le HTML/CSS/TS que vous connaissez d'un côté, le Rust que vous apprenez de l'autre, et un **pont** entre les deux. Vous découvrirez que ce pont n'est pas du `fetch`, mais quelque chose de plus simple et de plus sûr : des **commandes** Rust appelées par `invoke`, typées et sérialisées des deux côtés.

Projet du module : une **application desktop de notes** qui consomme l'API du module 5. Le frontend TS appelle des commandes Rust ; les commandes Rust appellent l'API avec `reqwest`. Le frontend ne fait aucun appel réseau lui-même.

Sommaire :

1. [Tauri vs Electron — la webview système](#1-tauri-vs-electron--la-webview-système)
2. [Anatomie d'un projet Tauri 2](#2-anatomie-dun-projet-tauri-2)
3. [Les commandes — `#[tauri::command]` et `invoke`](#3-les-commandes--tauricommand-et-invoke)
4. [Commandes asynchrones et erreurs à travers le pont](#4-commandes-asynchrones-et-erreurs-à-travers-le-pont)
5. [Le state managé — `tauri::State` et `.manage()`](#5-le-state-managé--tauristate-et-manage)
6. [Appeler l'API du module 5 avec `reqwest`](#6-appeler-lapi-du-module-5-avec-reqwest)
7. [Les capabilities — le modèle de permissions](#7-les-capabilities--le-modèle-de-permissions)
8. [Aperçu : les events Rust → frontend](#8-aperçu--les-events-rust--frontend)
9. [Packaging — `tauri build`, `.deb` et `.AppImage`](#9-packaging--tauri-build-deb-et-appimage)
10. [Récapitulatif — où chaque concept sert dans le projet](#10-récapitulatif--où-chaque-concept-sert-dans-le-projet)

---

## 1. Tauri vs Electron — la webview système

### 1.1 Définition

**Tauri** est un framework pour construire des applications de bureau (et mobiles) dont **l'interface est une page web** (HTML/CSS/JS) et dont **la logique métier est en Rust**. Vous écrivez votre UI exactement comme un site : du DOM, du CSS, votre framework TS préféré ou rien du tout. Tauri l'affiche dans une fenêtre native et lui adjoint un backend Rust auquel elle peut parler.

L'idée n'est pas neuve — c'est celle d'**Electron** (VS Code, Slack, Discord…). La différence est dans le « comment ».

### 1.2 La webview système, le cœur de la différence

Electron **embarque** un navigateur Chromium complet et un runtime Node.js **dans chaque application**. Résultat : la moindre app « Hello world » pèse 100 à 200 Mo et lance plusieurs processus Chromium gourmands en mémoire — vous payez un navigateur entier par application installée.

Tauri, lui, **n'embarque pas de navigateur**. Il utilise la **webview système** — le moteur de rendu web déjà présent sur le système d'exploitation :

| OS | Webview utilisée |
|---|---|
| Linux | WebKitGTK (`libwebkit2gtk`) |
| Windows | WebView2 (Edge/Chromium, fourni par Windows) |
| macOS | WKWebView (Safari) |

Votre application ne transporte que son propre code : le HTML/CSS/JS et un petit binaire Rust. Le moteur de rendu, lui, est celui de l'OS.

> **Pourquoi `libwebkit2gtk-4.1-dev` est un prérequis sous Linux.** C'est précisément cette webview système. Sous Windows et macOS elle est préinstallée ; sous Linux, il faut installer le paquet de développement pour compiler contre elle. C'est ce que documente l'encadré en tête de `PROJET.md` — sans lui, `cargo` ne peut pas lier l'application.

### 1.3 Comparaison chiffrée

| | Electron | Tauri |
|---|---|---|
| Moteur de rendu | Chromium **embarqué** (~150 Mo) | webview **système** (0 Mo embarqué) |
| Backend | Node.js (JavaScript) | Rust (binaire natif) |
| Taille d'un « Hello world » | ~120–250 Mo | ~3–10 Mo |
| Mémoire au repos | plusieurs centaines de Mo | quelques dizaines de Mo |
| Surface d'attaque | Node.js complet exposé | rien par défaut (voir §7) |

Pour un développeur Java/TS, le repère mental le plus juste : **Electron = un fat-jar qui embarque sa propre JVM ; Tauri = un binaire qui utilise la JVM déjà installée sur la machine**. La même UI web dans les deux cas, mais un poids et une consommation sans commune mesure.

### 1.4 La sécurité par défaut

Dans Electron, le code de l'UI a accès (selon la configuration) à Node.js : `require("fs")`, `require("child_process")`… Une faille XSS dans votre page peut alors lire des fichiers ou lancer des processus. C'est une responsabilité lourde.

Dans Tauri, le frontend est **du web et rien d'autre** : pas d'accès au système de fichiers, pas de réseau privilégié, pas de processus. Tout ce qui touche à la machine passe par des **commandes Rust que vous exposez explicitement** (§3) et que vous **autorisez explicitement** (§7). C'est le principe du moindre privilège, appliqué par construction. Dans notre projet, le frontend ne peut même pas appeler `http://localhost:3000` en `fetch` direct : c'est une commande Rust qui le fait pour lui.

---

## 2. Anatomie d'un projet Tauri 2

Un projet Tauri, c'est **deux projets côte à côte** dans le même dossier : un projet frontend (web) et un projet backend (Rust). Voici la structure exacte de notre `projet/` :

```text
notes-desktop/
├── index.html              ← page de l'UI (point d'entrée web)
├── package.json            ← dépendances npm (frontend + CLI Tauri)
├── vite.config.ts          ← bundler frontend (Vite)
├── tsconfig.json           ← config TypeScript
├── src/
│   ├── main.ts             ← le code du frontend (TS) : invoke, DOM
│   └── styles.css
└── src-tauri/              ← LE PROJET RUST (un crate Cargo à part entière)
    ├── Cargo.toml          ← dépendances Rust (tauri, reqwest, serde…)
    ├── build.rs            ← script de build (appelle tauri-build)
    ├── tauri.conf.json     ← LA configuration de l'app Tauri
    ├── capabilities/
    │   └── default.json    ← permissions accordées au frontend (§7)
    ├── icons/              ← icônes de l'app (toutes tailles)
    └── src/
        ├── main.rs         ← binaire : appelle simplement lib::run()
        ├── lib.rs          ← les commandes + le Builder Tauri
        ├── api.rs          ← le client HTTP vers l'API du module 5
        └── error.rs        ← le type d'erreur des commandes
```

Deux gestionnaires de paquets cohabitent, et c'est normal : **npm** pour le frontend (`package.json`), **Cargo** pour le backend (`src-tauri/Cargo.toml`). C'est exactement comme si vous aviez un dossier `frontend/` (React/Vite) et un dossier `backend/` (votre API Rust) — sauf qu'ici Tauri les soude en une seule application.

### 2.1 `tauri.conf.json` — la pièce maîtresse

C'est le fichier qui décrit l'application. Le nôtre, commenté :

```json
{
  "productName": "notes-desktop",
  "identifier": "com.coursrust.notes",      // identifiant unique (style reverse-DNS)
  "build": {
    "beforeDevCommand": "npm run dev",       // Tauri lance Vite avant d'ouvrir la fenêtre
    "devUrl": "http://localhost:1420",        // l'URL servie par Vite en développement
    "beforeBuildCommand": "npm run build",    // build du frontend avant le packaging
    "frontendDist": "../dist"                 // où trouver le frontend compilé
  },
  "app": {
    "windows": [
      { "title": "Notes — module 6", "width": 900, "height": 650 }
    ]
  },
  "bundle": {
    "active": true,
    "targets": ["deb", "appimage"],           // formats d'installeur Linux (§9)
    "icon": ["icons/32x32.png", "..."]
  }
}
```

Le point clé : **en développement, Tauri ne sert pas vos fichiers lui-même**. Il démarre Vite (`beforeDevCommand`), puis pointe la webview vers `devUrl` (le serveur Vite). Vous gardez donc le *hot reload* de Vite. **En production**, il n'y a plus de serveur : Tauri embarque le frontend compilé (`frontendDist`) directement dans le binaire.

### 2.2 La structure `main.rs` + `lib.rs`

Vous remarquez que `main.rs` ne fait presque rien :

```rust
fn main() {
    notes_desktop_lib::run() // toute la logique est dans la bibliothèque
}
```

C'est la convention standard de Tauri 2 : toute la logique vit dans `lib.rs` (la bibliothèque), `main.rs` n'est qu'un point d'entrée minimal. Vous avez déjà vu l'idée « lib + bin » au module 3 ; ici, elle est imposée par le framework, car le même `lib.rs` peut servir de point d'entrée sur mobile (iOS/Android). D'où aussi le nom de bibliothèque distinct dans `Cargo.toml` :

```toml
[lib]
name = "notes_desktop_lib"
crate-type = ["staticlib", "cdylib", "rlib"]
```

---

## 3. Les commandes — `#[tauri::command]` et `invoke`

### 3.1 Définition

Une **commande** est une fonction Rust marquée `#[tauri::command]`, exposée au frontend. Le frontend l'appelle par son nom avec `invoke("nom", { args })`. C'est le seul moyen, pour le code web, de déclencher du code Rust.

Côté Rust (`src-tauri/src/lib.rs`), la commande de démonstration du projet :

```rust
#[tauri::command]
fn greet(name: String) -> String {
    format!("Bonjour, {name} ! Le pont TypeScript ↔ Rust fonctionne.")
}
```

Côté TypeScript (`src/main.ts`) :

```typescript
import { invoke } from "@tauri-apps/api/core";

// invoke<T> : T type la valeur de retour côté TS.
const message = await invoke<string>("greet", { name: "Ada" });
console.log(message); // "Bonjour, Ada ! Le pont TypeScript ↔ Rust fonctionne."
```

Trois choses se passent à chaque appel :

1. `invoke` sérialise l'objet `{ name: "Ada" }` en JSON et l'envoie à Rust ;
2. Tauri désérialise le JSON vers les paramètres de la fonction (`name: String`), **par nom de clé** ;
3. la valeur de retour est re-sérialisée en JSON et renvoyée à TS, sous forme de `Promise`.

> **Le branchement obligatoire.** Une commande n'existe pour le frontend que si elle est listée dans `generate_handler!` :
>
> ```rust
> tauri::Builder::default()
>     .invoke_handler(tauri::generate_handler![greet, list_notes, create_note])
>     .run(tauri::generate_context!())
> ```
>
> Si vous oubliez d'y ajouter une commande, l'`invoke` correspondant échoue à l'exécution avec « command not found ». C'est l'oubli classique du module.

### 3.2 Comparaison : ce n'est pas du `fetch`

Le réflexe Java/TS serait d'imaginer un serveur HTTP local et des appels `fetch`. **Ce n'est pas ça du tout** :

| | `fetch` (vers un serveur HTTP) | `invoke` (vers une commande Tauri) |
|---|---|---|
| Transport | réseau TCP, HTTP, sérialisation JSON sur le fil | canal IPC interne au processus (pas de socket réseau) |
| Cible | une URL, une route, un verbe | un nom de fonction Rust |
| Typage | aucun (`fetch` rend `Response`, vous castez) | `invoke<T>` côté TS, signature typée côté Rust |
| Disponibilité | il faut un serveur qui tourne | la fonction est compilée dans l'app |
| Erreurs | code HTTP à interpréter | `Result::Err` → `Promise` rejetée (§4) |

Le terme exact pour ce mécanisme est **IPC** (*Inter-Process Communication*), ou plus simplement le **pont** Tauri. Mentalement : `invoke` est un **appel de procédure distante (RPC) local**, sérialisé en JSON, où « distant » veut dire « l'autre côté du pont », pas « sur le réseau ».

### 3.3 Le typage des deux côtés

Le pont sérialise en JSON, donc vos types doivent être **sérialisables**. Côté Rust, c'est le rôle de serde (que vous connaissez du module 3) :

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub created_at: String,
}
```

Côté TypeScript, vous écrivez l'**interface miroir** à la main :

```typescript
interface Note {
  id: number;       // i64 Rust → number TS
  title: string;
  content: string;
  created_at: string;
}
```

> **Point d'attention.** Le pont ne vérifie **pas** que la struct Rust et l'interface TS coïncident — c'est à vous de les garder synchronisées. Si vous renommez `title` en Rust sans le faire en TS, ça compile des deux côtés, mais le champ arrivera `undefined`. (Des outils comme `ts-rs` génèrent les interfaces TS depuis les structs Rust pour supprimer ce risque ; hors périmètre ici, mais bon à connaître.)

---

## 4. Commandes asynchrones et erreurs à travers le pont

### 4.1 Pourquoi `async`

`greet` est instantanée. Mais nos vraies commandes font un **appel HTTP** vers l'API du module 5 : c'est lent et bloquant. Si on l'exécutait de façon synchrone, la webview se figerait pendant l'appel. La solution est la même qu'au module 5 : `async`.

```rust
#[tauri::command]
async fn list_notes(client: State<'_, ApiClient>) -> Result<Vec<Note>, ApiError> {
    client.list_notes().await
}
```

Quand une commande est `async`, Tauri l'exécute sur son **runtime asynchrone** (tokio, comme au module 5), sans jamais bloquer le fil de la webview. Côté TS, rien ne change : `invoke` rend **toujours** une `Promise`, que la commande soit sync ou async. Vous écrivez `await invoke(...)` dans les deux cas.

### 4.2 `Result` Rust → `Promise` rejetée

C'est le mécanisme le plus élégant du pont. Une commande qui retourne `Result<T, E>` se traduit ainsi côté TS :

| Côté Rust | Côté TypeScript |
|---|---|
| `Ok(valeur)` | la `Promise` se **résout** avec `valeur` |
| `Err(erreur)` | la `Promise` est **rejetée** avec `erreur` sérialisée |

Vous récupérez donc les erreurs Rust dans un `try`/`catch` TypeScript ordinaire :

```typescript
try {
  const notes = await invoke<Note[]>("list_notes");
  renderNotes(notes);
} catch (err) {
  // err est l'erreur Rust, sérialisée. Ici, une chaîne (voir ci-dessous).
  showError(err);
}
```

C'est plus propre que `fetch` : avec `fetch`, un `await` ne rejette que sur erreur réseau, pas sur un `404` ou un `500` (il faut tester `response.ok` à la main). Avec `invoke`, **toute** erreur Rust devient un rejet. Le modèle `Result`/`?` du module 5 traverse le pont sans accroc.

### 4.3 Rendre une erreur sérialisable

Pour qu'un `Err` traverse le pont, le type d'erreur doit implémenter `Serialize`. Notre `ApiError` (`src-tauri/src/error.rs`) le fait en se transformant en **simple chaîne** — le message que verra l'utilisateur :

```rust
use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(
        "Impossible de joindre l'API ({0}). Le serveur du module 5 est-il démarré ?"
    )]
    Unreachable(String),

    #[error("note introuvable (id {0})")]
    NotFound(i64),

    #[error("réponse inattendue de l'API : {0}")]
    Unexpected(String),
}

// On sérialise l'erreur vers sa représentation Display (le message #[error]).
impl Serialize for ApiError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
```

Côté TS, `catch (err)` reçoit donc directement une `string` lisible. C'est la combinaison parfaite des outils du module 3 (thiserror pour les messages) et du module 5 (`Result` partout) : un `Err` Rust idiomatique devient un message d'erreur affiché à l'utilisateur, sans code de plomberie.

> **Le `'_` obligatoire.** Une commande `async` qui prend un `State` doit écrire `State<'_, ApiClient>` (le lifetime anonyme). Sans lui, le compilateur refuse : une commande async retourne un *future*, et Rust exige que la durée de vie de l'emprunt du state soit explicite. C'est l'erreur de compilation la plus fréquente du module — retenez la forme `State<'_, T>` pour les commandes async.

---

## 5. Le state managé — `tauri::State` et `.manage()`

### 5.1 Le problème

Notre `ApiClient` (un `reqwest::Client` avec son pool de connexions) doit être **créé une seule fois** et **partagé** par toutes les commandes. On ne veut pas en reconstruire un à chaque appel. C'est exactement le problème du state partagé du module 5 (le pool SQLx passé à Axum).

### 5.2 La solution : `.manage()` + `State<'_, T>`

Tauri offre le même mécanisme qu'Axum. On enregistre l'objet une fois sur le `Builder` avec `.manage()` :

```rust
pub fn run() {
    tauri::Builder::default()
        // Une seule instance d'ApiClient pour toute la durée de l'app.
        .manage(ApiClient::new("http://localhost:3000"))
        .invoke_handler(tauri::generate_handler![
            greet, list_notes, create_note, update_note, delete_note
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de l'application Tauri");
}
```

… et n'importe quelle commande peut alors le **demander** en le déclarant comme paramètre `State<'_, ApiClient>`. Tauri l'injecte automatiquement :

```rust
#[tauri::command]
async fn create_note(
    client: State<'_, ApiClient>,   // injecté par Tauri (pas envoyé par le frontend !)
    title: String,                  // celui-là vient bien de invoke({ title, content })
    content: String,
) -> Result<Note, ApiError> {
    client.create_note(&NoteInput { title, content }).await
}
```

Point important : **le `State` n'est pas un argument que le frontend fournit**. Tauri distingue les paramètres injectés (le state) des paramètres venus du JSON d'`invoke` (`title`, `content`). Côté TS, vous appelez juste `invoke("create_note", { title, content })` : le client est invisible pour le frontend.

### 5.3 Comparaison

| | Spring / Java | Axum (module 5) | Tauri |
|---|---|---|---|
| Enregistrer | `@Bean` / conteneur DI | `.with_state(pool)` | `.manage(client)` |
| Recevoir | `@Autowired` | `State(pool): State<Pool>` | `client: State<'_, Client>` |

C'est de l'**injection de dépendances**, en version explicite et vérifiée à la compilation : si vous demandez un `State<Foo>` qu'aucun `.manage()` n'a enregistré, l'`invoke` échoue à l'exécution. Le pattern est identique à celui d'Axum — un acquis du module 5 directement réutilisé.

> **State mutable ?** Notre `ApiClient` est partagé en lecture seule (un `reqwest::Client` est `Clone` et thread-safe). Si vous aviez besoin de muter le state (un compteur, un cache…), il faudrait l'envelopper dans un `Mutex` — `State<'_, Mutex<Cache>>` — exactement comme `Arc<Mutex<T>>` au module 4. Le défi « mode hors-ligne » de `DEFIS.md` vous y amènera.

---

## 6. Appeler l'API du module 5 avec `reqwest`

### 6.1 `reqwest`, le client HTTP

**`reqwest`** est le client HTTP de référence de l'écosystème Rust : un `fetch` côté Rust, asynchrone et typé. C'est lui qui, dans nos commandes, parle réellement à l'API du module 5. Le frontend ne connaît jamais `http://localhost:3000` : il appelle une commande, la commande appelle l'API.

```text
Frontend (TS)            Backend Tauri (Rust)              API module 5
  invoke("list_notes") ───►  list_notes()                   
                              client.list_notes()  ──GET /notes──►
                                                   ◄──200 JSON────
                          ◄── Vec<Note> sérialisé
  Promise<Note[]>
```

### 6.2 Le client réutilisable

On construit le `reqwest::Client` une fois (c'est lui qu'on met dans le state, §5) :

```rust
pub struct ApiClient {
    http: reqwest::Client,   // contient un pool de connexions, Clone, thread-safe
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(5)) // sans timeout, une API morte bloque ~30 s
            .build()
            .expect("configuration du client HTTP invalide");
        Self { http, base_url: base_url.into() }
    }
}
```

### 6.3 Un appel typique : GET et POST

```rust
// GET /notes → Vec<Note>
pub async fn list_notes(&self) -> Result<Vec<Note>, ApiError> {
    let response = self
        .http
        .get(format!("{}/notes", self.base_url))
        .send()
        .await
        .map_err(ApiError::from_reqwest)?;     // erreur réseau → ApiError
    Self::decode(response, StatusCode::OK).await
}

// POST /notes → la note créée (201)
pub async fn create_note(&self, input: &NoteInput) -> Result<Note, ApiError> {
    let response = self
        .http
        .post(format!("{}/notes", self.base_url))
        .json(input)        // sérialise NoteInput en JSON + pose Content-Type
        .send()
        .await
        .map_err(ApiError::from_reqwest)?;
    Self::decode(response, StatusCode::CREATED).await
}
```

Comparaison avec le `fetch` que vous connaissez :

```typescript
// fetch équivalent — mais NON validé : .json() rend `any`
const res = await fetch(`${base}/notes`, {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify(input),
});
const note = await res.json(); // type `any`, aucune garantie de structure
```

```rust
// reqwest : .json(input) sérialise ET pose l'en-tête ; .json::<Note>() VALIDE
// le JSON reçu contre la struct Note. Un champ manquant = Err, pas un undefined.
```

C'est `fetch` + serde + zod réunis : la requête est sérialisée proprement, et la réponse est **validée** contre votre type Rust. Un JSON qui ne correspond pas à `Note` produit une `Err` explicite, pas un `undefined` qui plante trois lignes plus loin.

### 6.4 Traduire les statuts HTTP en erreurs métier

L'API du module 5 répond `404` sur un id inconnu. On traduit ce statut en variante d'erreur :

```rust
pub async fn delete_note(&self, id: i64) -> Result<(), ApiError> {
    let response = self.http.delete(format!("{}/notes/{id}", self.base_url))
        .send().await.map_err(ApiError::from_reqwest)?;
    match response.status() {
        StatusCode::NO_CONTENT => Ok(()),                  // 204 → succès, rien à rendre
        StatusCode::NOT_FOUND  => Err(ApiError::NotFound(id)),
        other => Err(ApiError::Unexpected(format!("statut {other}"))),
    }
}
```

`from_reqwest` classe les pannes de transport (connexion refusée, timeout → `Unreachable`) ; le `match` sur le statut classe les réponses HTTP. Le frontend, lui, ne voit qu'un message d'erreur — toute cette logique reste côté Rust.

> **Feature `default-features = false`.** Dans `Cargo.toml`, on écrit `reqwest = { version = "0.13", default-features = false, features = ["json", "rustls"] }`. On désactive les features par défaut (qui tirent OpenSSL, une dépendance système) et on n'active que `json` (les méthodes `.json()`) et `rustls` (TLS en Rust pur, sans dépendance système). C'est l'application directe des features du module 3 : un build plus léger et plus portable.

---

## 7. Les capabilities — le modèle de permissions

### 7.1 Le principe

On l'a dit au §1.4 : par défaut, le frontend Tauri ne peut **rien** toucher du système. Tout pouvoir doit être **accordé explicitement** par une **capability** — un fichier JSON qui dit « telle fenêtre a le droit d'utiliser telles permissions ».

Notre `src-tauri/capabilities/default.json` :

```json
{
  "identifier": "default",
  "description": "Socle Tauri uniquement (invoke, events). Tout le réseau passe par les commandes Rust.",
  "windows": ["main"],
  "permissions": ["core:default"]
}
```

`core:default` est le socle minimal : il autorise `invoke`, les events, et le contrôle de base de la fenêtre. **Rien de plus.** Comme tout notre réseau passe par des commandes Rust (qui n'ont besoin d'aucune permission frontend pour exister), c'est tout ce qu'il nous faut.

### 7.2 Quand on a besoin de plus

Tauri fournit des **plugins** (système de fichiers, dialogues, notifications, presse-papier…). Chacun apporte ses permissions, qu'il faut ajouter à la liste pour que le frontend puisse l'appeler. Par exemple, pour autoriser les notifications système (voir `DEFIS.md`) :

```json
"permissions": ["core:default", "notification:default"]
```

Sans cette ligne, l'appel JS au plugin notification échouerait — refusé par le modèle de sécurité.

### 7.3 Comparaison

C'est le même esprit que le `AndroidManifest.xml` d'Android ou les permissions d'une extension de navigateur : **une liste blanche déclarative de ce que le code a le droit de faire**. À l'opposé d'Electron, où le code de l'UI a par défaut accès à tout Node. Ici, le défaut est « rien », et vous ouvrez les portes une par une, en les justifiant.

---

## 8. Aperçu : les events Rust → frontend

Les commandes vont du frontend vers Rust (`invoke`). Pour le sens inverse — **Rust qui prévient le frontend** sans qu'on le lui demande — Tauri offre les **events**. Indispensable quand Rust a quelque chose à signaler spontanément : progression d'un téléchargement, donnée poussée, tâche de fond terminée.

Côté Rust, on émet :

```rust
use tauri::Emitter;

#[tauri::command]
async fn sync(app: tauri::AppHandle) {
    // ... travail ...
    app.emit("sync-done", 42).unwrap(); // "événement nommé" + charge utile
}
```

Côté TS, on écoute :

```typescript
import { listen } from "@tauri-apps/api/event";

await listen<number>("sync-done", (event) => {
  console.log("synchronisation terminée :", event.payload); // 42
});
```

C'est l'équivalent d'un `EventEmitter` Node, ou d'un `WebSocket`/SSE — mais à travers le pont local. Le repère mental : **`invoke` = requête/réponse (comme un appel de fonction) ; les events = publication/abonnement (comme un flux)**. Notre projet n'en a pas besoin (chaque action utilisateur attend une réponse), mais c'est la brique à connaître pour tout ce qui est temps réel.

---

## 9. Packaging — `tauri build`, `.deb` et `.AppImage`

### 9.1 Développement vs production

Deux commandes, deux mondes :

- `npm run tauri dev` : lance Vite (hot reload), ouvre la fenêtre, compile le Rust en mode debug. C'est votre boucle de développement.
- `npm run tauri build` : compile le frontend (`npm run build`), compile le Rust en mode **release** (optimisé), puis **empaquette** le tout en installeurs natifs.

### 9.2 Les formats produits

Notre `tauri.conf.json` demande deux cibles Linux :

```json
"bundle": { "active": true, "targets": ["deb", "appimage"] }
```

- **`.deb`** : le paquet d'installation Debian/Ubuntu, installable par `sudo dpkg -i` ou `apt`. Il s'intègre au système (menu, désinstallation propre).
- **`.AppImage`** : un fichier unique, exécutable, **portable** — pas d'installation, on le rend exécutable et on le lance. Pratique pour distribuer sans droits admin.

Selon l'OS, `tauri build` produit les formats adaptés : `.msi`/`.exe` sous Windows, `.dmg`/`.app` sous macOS, `.deb`/`.rpm`/`.AppImage` sous Linux. Le terme générique pour ces artefacts est le **bundle**.

### 9.3 La différence de poids, concrètement

C'est ici que le choix de Tauri paie : votre installeur ne contient **que votre application** (le binaire Rust + le frontend compilé), pas de navigateur. Quelques mégaoctets, là où l'équivalent Electron en pèserait cent ou deux cents. Le moteur de rendu, lui, est déjà sur la machine de l'utilisateur (§1.2).

> **Sur cette machine.** La compilation Tauri (`dev` comme `build`) exige `libwebkit2gtk-4.1-dev` et `librsvg2-dev`, absentes ici. `PROJET.md` ouvre sur la commande d'installation. Tant qu'elles ne sont pas installées, seule la partie TypeScript se vérifie (`npm run build` ne fait que `tsc --noEmit` côté front) ; le `cargo` de `src-tauri` échoue à l'édition de liens.

---

## 10. Récapitulatif — où chaque concept sert dans le projet

| Concept | Où vous le pratiquerez |
|---|---|
| Webview système / Tauri vs Electron | le prérequis `libwebkit2gtk`, le poids du bundle |
| Anatomie (`src-tauri`, `tauri.conf.json`) | tout le squelette `projet/` |
| `#[tauri::command]` + `invoke` | `greet`, puis les 4 commandes CRUD ↔ les `invoke` de `main.ts` |
| `async` + `Result` → `Promise` | `list_notes` & co. ; le `try`/`catch` du frontend |
| Erreur sérialisable | `ApiError` (`error.rs`), affichée dans le bandeau d'erreur |
| `State<'_, T>` + `.manage()` | l'`ApiClient` partagé, injecté dans chaque commande |
| `reqwest` | `api.rs` : les 4 appels HTTP vers l'API du module 5 |
| Capabilities / permissions | `capabilities/default.json` (`core:default`) |
| Events (aperçu) | non utilisé ici, brique pour le temps réel |
| Packaging / bundle | `tauri build` → `.deb` / `.AppImage` (quand webkit sera installé) |

Ouvrez maintenant `PROJET.md`. **Lisez l'encadré des prérequis en tout premier** : sans les paquets système, rien ne se lance.
