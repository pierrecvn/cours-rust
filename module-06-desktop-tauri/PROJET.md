# Projet 6 — Application desktop de notes (Tauri 2)

Vous allez construire une **vraie application de bureau** : une fenêtre native qui liste, crée, modifie et supprime des notes. L'interface est en TypeScript (votre zone de confort) ; le backend est en Rust ; les deux dialoguent par le **pont Tauri** (`invoke`). Les commandes Rust appellent l'API REST du module 5 avec `reqwest` — le frontend, lui, ne fait **aucun** appel réseau direct.

À la fin, vous aurez une application qui ressemble à ça :

```text
┌─────────────────────────────────────────────┐
│  Mes notes                      [Rafraîchir] │
│                                              │
│  Nouvelle note                               │
│  [ Titre........................ ]           │
│  [ Contenu de la note…          ]            │
│  [ Créer ]                                   │
│                                              │
│  ┌────────────────────────────────────────┐ │
│  │ Courses          11/06/2026 14:32       │ │
│  │ Lait, pain, café                        │ │
│  │              [ Modifier ] [ Supprimer ] │ │
│  └────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

Le squelette est dans `projet/`, le corrigé dans `solution/` (à ne regarder qu'après avoir vraiment essayé). Tout le texte affiché est en français, tous les identifiants de code en anglais.

---

> ## ⚠️ Prérequis — à lire et à exécuter AVANT toute chose
>
> ### 1. Dépendances système (Linux)
>
> Tauri utilise la **webview système** (voir `COURS.md` §1). Sous Linux, il faut le moteur WebKitGTK et la bibliothèque de rendu SVG. Sur cette machine **elles ne sont pas installées** : tant que vous ne lancez pas la commande ci-dessous, **toute** compilation Tauri (`npm run tauri dev` comme `npm run tauri build`) échouera à l'édition de liens, avec une erreur du genre `package 'webkit2gtk-4.1' not found`.
>
> ```bash
> sudo apt install libwebkit2gtk-4.1-dev librsvg2-dev
> ```
>
> (Sous Windows et macOS, la webview est préinstallée : ce prérequis ne concerne que Linux. Les autres dépendances Tauri — `build-essential`, `curl`, `wget`, `libssl-dev`, `libayatana-appindicator3-dev` — sont déjà présentes ici.)
>
> ### 2. L'API du module 5 doit tourner
>
> L'application **consomme** l'API REST du module 5. Sans elle, l'app se lance mais n'affiche que le bandeau d'erreur « Impossible de joindre l'API ». Dans un terminal **séparé**, laissez tourner :
>
> ```bash
> cd ../module-05-backend-web/solution   # ou votre propre projet du module 5
> cargo run                              # écoute sur http://localhost:3000
> ```
>
> Contrat consommé (rappel du `COURS.md` du module 5) : `GET /notes`, `POST /notes`, `GET/PUT/DELETE /notes/{id}`, base `http://localhost:3000`.
>
> **Astuce sans webkit.** Vous pouvez déjà tout écrire et vérifier la moitié TypeScript sans installer webkit : `cd projet && npm install && npm run build` lance `tsc --noEmit` (votre TS doit compiler). Seules les étapes qui lancent **la fenêtre** (`npm run tauri dev`) exigent les paquets système.

---

## Étape 0 — Prise en main du squelette

**Objectif.** Comprendre la structure du projet, installer les dépendances, et savoir ce qui est fourni et ce qui reste à écrire.

**Pourquoi.** Un projet Tauri, c'est **deux projets dans un seul dossier** (`COURS.md` §2) : un frontend web (npm) et un backend Rust (`src-tauri/`, un crate Cargo). Vous allez naviguer entre les deux ; autant savoir où est quoi.

```text
projet/
├── index.html              ← UI (fourni) : formulaire + liste + bandeau d'erreur
├── package.json            ← scripts npm (dev, build, tauri) (fourni)
├── vite.config.ts          ← bundler, port 1420 (fourni)
├── tsconfig.json           ← config TypeScript stricte (fourni)
├── src/
│   ├── main.ts             ← FRONTEND : 3 fonctions à écrire (TODO)
│   └── styles.css          ← (fourni)
└── src-tauri/              ← LE PROJET RUST
    ├── Cargo.toml          ← dépendances (tauri, reqwest, serde, thiserror) (fourni)
    ├── build.rs            ← script de build Tauri (fourni)
    ├── tauri.conf.json     ← config de l'app (fourni)
    ├── capabilities/
    │   └── default.json    ← permissions : core:default (fourni)
    ├── icons/              ← icônes (fournies)
    └── src/
        ├── main.rs         ← appelle lib::run() (fourni)
        ├── lib.rs          ← COMMANDES + Builder : à compléter (TODO)
        ├── api.rs          ← client reqwest : 4 méthodes à écrire (TODO)
        └── error.rs        ← type ApiError (fourni, complet)
```

**Ce qui est fourni, complet.** Toute la plomberie : `index.html`, le CSS, tout le DOM de `main.ts` (récupération des éléments, rendu de la liste, bascule création/édition, branchement des boutons), le type `ApiError` (`error.rs`), les structs `Note`/`NoteInput`/`ApiClient::new` (`api.rs`), et toute la config Tauri.

**Ce que vous écrivez.** Dans `src-tauri/src/api.rs` : les 4 méthodes HTTP (`list_notes`, `create_note`, `update_note`, `delete_note`). Dans `src-tauri/src/lib.rs` : les 5 commandes (`greet` + les 4 CRUD), le `.manage(...)` et le `generate_handler![...]`. Dans `src/main.ts` : les 3 fonctions `refreshNotes`, `submitForm`, `removeNote`. Chaque endroit à compléter est marqué `TODO (étape N)`.

**À faire.**

```bash
cd projet
npm install            # dépendances frontend + CLI Tauri (≈ 100 Mo, normal)
npm run build          # lance tsc --noEmit : doit compiler (le squelette est valide)
```

**Point de contrôle.** `npm run build` réussit (le squelette TS compile : les `TODO` côté front sont des `showError("TODO …")`, pas du code invalide). Si webkit est installé, `npm run tauri dev` ouvre une fenêtre affichant le bandeau d'erreur « Impossible de joindre l'API » (normal : aucune commande n'est encore branchée, et l'API n'est peut-être pas lancée).

---

## Étape 1 — La commande `greet` : prouver que le pont fonctionne

**Objectif.** Écrire la commande de démonstration `greet` dans `src-tauri/src/lib.rs`, l'enregistrer, et l'appeler une fois depuis le frontend pour valider l'aller-retour TS → Rust → TS.

**Pourquoi.** Avant de brancher quoi que ce soit sur le réseau, on isole **le pont** : une fonction Rust triviale, appelée depuis TypeScript. Si `greet` répond, l'infrastructure Tauri est saine et tout le reste n'est plus que de la logique métier.

**Indices.**

- Côté Rust (`lib.rs`), une fonction synchrone, sans erreur possible, donc retour direct `String` :

  ```rust
  #[tauri::command]
  fn greet(name: String) -> String {
      format!("Bonjour, {name} ! Le pont TypeScript ↔ Rust fonctionne.")
  }
  ```

- **Le branchement obligatoire** (`COURS.md` §3.1) : ajoutez `greet` dans `generate_handler!` du `run()`, sinon `invoke("greet", …)` échoue à l'exécution avec « command not found ».

  ```rust
  .invoke_handler(tauri::generate_handler![greet])
  ```

- Pour tester depuis le front, ajoutez **temporairement** en haut de `main.ts` (vous le retirerez après) :

  ```typescript
  invoke<string>("greet", { name: "Ada" }).then(console.log);
  ```

  La clé `name` de l'objet correspond, **par nom**, au paramètre `name: String` de la fonction Rust.

**Point de contrôle.** Avec webkit installé : `npm run tauri dev`, ouvrez la console de la webview (clic droit → Inspecter), vous lisez `Bonjour, Ada ! Le pont TypeScript ↔ Rust fonctionne.`. Sans webkit : au moins, `npm run build` compile toujours. Retirez ensuite l'appel de test de `main.ts`.

---

## Étape 2 — Le state managé : un `ApiClient` partagé

**Objectif.** Enregistrer une instance unique d'`ApiClient` sur le `Builder` Tauri avec `.manage()`, prête à être injectée dans les commandes.

**Pourquoi.** Toutes les commandes CRUD vont parler à la même API. On veut **un seul** `reqwest::Client` (il porte un pool de connexions), créé une fois, partagé partout — exactement le state partagé du module 5 (`COURS.md` §5). `ApiClient::new(...)` est déjà écrit dans `api.rs` ; il vous reste à l'enregistrer.

**Indices.**

- Dans `run()` (`lib.rs`), avant `.invoke_handler(...)` :

  ```rust
  .manage(ApiClient::new("http://localhost:3000"))
  ```

  C'est l'adresse fixe du contrat (module 5). Le `State<'_, ApiClient>` que demanderont vos commandes sera servi par ce `.manage(...)`.

- Rien n'est visible à l'écran à cette étape : c'est de la tuyauterie. Mais sans elle, les commandes des étapes suivantes ne compileront pas (Tauri ne saura pas d'où sortir le `State`).

**Point de contrôle.** `npm run build` compile encore (vous n'avez touché qu'au Rust ; rien à vérifier côté TS). Le vrai contrôle viendra quand `list_notes` injectera ce state à l'étape 4.

---

## Étape 3 — Le client HTTP : `list_notes` dans `api.rs`

**Objectif.** Implémenter `ApiClient::list_notes` dans `src-tauri/src/api.rs` (GET /notes → `Vec<Note>`).

**Pourquoi.** C'est ici que Rust parle réellement à l'API. `api.rs` est du Rust **pur** (reqwest + serde), sans une ligne de Tauri : on peut le raisonner, le tester, le réutiliser indépendamment. On commence par la lecture (GET), la plus simple.

**Indices.**

- La structure d'un appel reqwest (`COURS.md` §6.3) :

  ```rust
  let response = self
      .http
      .get(format!("{}/notes", self.base_url))
      .send()
      .await
      .map_err(ApiError::from_reqwest)?;   // panne réseau → ApiError::Unreachable
  ```

- Ensuite, vérifiez le statut (attendu : `StatusCode::OK`) puis désérialisez le corps. Le squelette suggère deux voies : soit à la main, soit en factorisant dans un petit helper `decode`. La solution utilise un helper générique réutilisé par les 4 méthodes :

  ```rust
  async fn decode<T: serde::de::DeserializeOwned>(
      response: reqwest::Response,
      expected: StatusCode,
  ) -> Result<T, ApiError> { /* vérifie le statut, puis response.json::<T>().await */ }
  ```

  Si vous préférez rester simple pour commencer, désérialisez directement : `response.json::<Vec<Note>>().await.map_err(...)?`. Vous factoriserez plus tard si vous voulez.

- `.json::<T>()` **valide** le JSON reçu contre votre type Rust : un champ manquant produit une `Err` claire, pas un `undefined` (contraste avec `fetch`, `COURS.md` §6.3).

**Point de contrôle.** Pas encore visible (la commande qui appelle cette méthode arrive à l'étape 4). Mais le Rust doit compiler : si webkit était installé, `cargo check` dans `src-tauri/` passerait. Sans webkit, vérifiez au moins que la résolution des dépendances tient : `cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 >/dev/null` (ne compile pas, mais valide `Cargo.toml`).

---

## Étape 4 — Première vraie commande : afficher les notes

**Objectif.** Écrire la commande Rust `list_notes` (dans `lib.rs`), l'enregistrer, puis implémenter `refreshNotes()` côté TS (`main.ts`). À la fin, l'application affiche les notes de l'API.

**Pourquoi.** C'est le premier bout de bout complet : un clic (ou le chargement) → `invoke` → commande Rust → `ApiClient` → API → retour jusqu'à l'écran. La commande qui fait un appel HTTP doit être **`async`**, et elle reçoit le state.

**Indices côté Rust.**

- La commande délègue tout à la méthode de l'étape 3 :

  ```rust
  #[tauri::command]
  async fn list_notes(client: State<'_, ApiClient>) -> Result<Vec<Note>, ApiError> {
      client.list_notes().await
  }
  ```

- **Le `'_` est obligatoire** (`COURS.md` §4, encadré) : une commande `async` qui emprunte un `State` exige le lifetime anonyme `State<'_, ApiClient>`. C'est l'erreur de compilation la plus fréquente du module — si le compilateur parle de *lifetime* sur le state, c'est ça.
- N'oubliez pas d'ajouter `list_notes` dans `generate_handler![greet, list_notes]`.

**Indices côté TypeScript.** Implémentez `refreshNotes()` (le `TODO` étape 4 dans `main.ts`) :

```typescript
async function refreshNotes(): Promise<void> {
  try {
    const notes = await invoke<Note[]>("list_notes");  // Result Ok → résout
    renderNotes(notes);                                 // fourni
    hideError();                                         // fourni
  } catch (err) {
    showError(err);   // Result Err → Promise rejetée → string lisible (COURS §4.2)
  }
}
```

`invoke<Note[]>` type le retour côté TS ; le `try`/`catch` attrape un `Err` Rust devenu rejet de Promise. `renderNotes`, `hideError`, `showError` sont déjà fournis.

**Point de contrôle.** Lancez l'API du module 5 (encadré prérequis), puis `npm run tauri dev`. Au démarrage, l'app appelle `refreshNotes()` : si l'API a des notes, elles s'affichent ; sinon, « Aucune note pour l'instant ». **Coupez** l'API (Ctrl-C dans son terminal) et cliquez « Réessayer » : le bandeau affiche « Impossible de joindre l'API… Le serveur du module 5 est-il démarré ? » — la gestion d'erreur de bout en bout fonctionne déjà.

---

## Étape 5 — Créer une note (POST)

**Objectif.** Implémenter `ApiClient::create_note` (`api.rs`), la commande `create_note` (`lib.rs`), et la branche « création » de `submitForm()` (`main.ts`).

**Pourquoi.** On passe en écriture. C'est le premier appel avec un **corps JSON** : `reqwest` sérialise un `NoteInput` et pose l'en-tête `Content-Type` tout seul.

**Indices côté Rust.**

- `api.rs` — l'appel POST attend un `201 CREATED` :

  ```rust
  pub async fn create_note(&self, input: &NoteInput) -> Result<Note, ApiError> {
      let response = self
          .http
          .post(format!("{}/notes", self.base_url))
          .json(input)        // sérialise NoteInput + pose Content-Type: application/json
          .send().await.map_err(ApiError::from_reqwest)?;
      // vérifier StatusCode::CREATED puis désérialiser en Note
  }
  ```

- `lib.rs` — la commande reçoit `title` et `content` du frontend, plus le `client` injecté :

  ```rust
  #[tauri::command]
  async fn create_note(
      client: State<'_, ApiClient>,   // injecté par Tauri, PAS envoyé par le front
      title: String,
      content: String,
  ) -> Result<Note, ApiError> {
      client.create_note(&NoteInput { title, content }).await
  }
  ```

  Tauri distingue les paramètres injectés (`client`) de ceux venus du JSON d'`invoke` (`title`, `content`) — `COURS.md` §5.2. Ajoutez `create_note` au `generate_handler!`.

**Indices côté TypeScript.** Dans `submitForm()`, la branche `editingId === null` (création) :

```typescript
await invoke<Note>("create_note", { title, content });
resetForm();           // fourni
await refreshNotes();  // recharge la liste depuis l'API
```

Les clés `{ title, content }` correspondent par nom aux paramètres de la commande Rust.

**Point de contrôle.** API lancée, `npm run tauri dev`. Remplissez le formulaire, cliquez « Créer » : la note apparaît dans la liste (parce que `refreshNotes()` recharge). Vérifiez côté serveur ou avec `curl http://localhost:3000/notes` que la note est bien persistée par l'API.

---

## Étape 6 — Modifier et supprimer (PUT, DELETE) : le CRUD complet

**Objectif.** Implémenter `update_note` et `delete_note` côté `api.rs` **et** `lib.rs`, puis compléter la branche « édition » de `submitForm()` et la fonction `removeNote()` côté TS.

**Pourquoi.** On termine le CRUD, et on rencontre le **404** : une note inexistante. C'est l'occasion de traduire un statut HTTP en variante d'erreur métier (`ApiError::NotFound`), qui remontera jusqu'au bandeau du frontend.

**Indices côté Rust (`api.rs`).**

- `update_note` (PUT, attend `200`, ou `404` → `NotFound`) :

  ```rust
  pub async fn update_note(&self, id: i64, input: &NoteInput) -> Result<Note, ApiError> {
      let response = self.http
          .put(format!("{}/notes/{id}", self.base_url))
          .json(input)
          .send().await.map_err(ApiError::from_reqwest)?;
      if response.status() == StatusCode::NOT_FOUND {
          return Err(ApiError::NotFound(id));
      }
      // sinon : vérifier 200 et désérialiser en Note
  }
  ```

- `delete_note` (DELETE, `204` → `Ok(())`, `404` → `NotFound`, autre → `Unexpected`). Le `match` sur le statut est le cœur (`COURS.md` §6.4) :

  ```rust
  match response.status() {
      StatusCode::NO_CONTENT => Ok(()),
      StatusCode::NOT_FOUND  => Err(ApiError::NotFound(id)),
      other => Err(ApiError::Unexpected(format!("statut {other}"))),
  }
  ```

**Indices côté Rust (`lib.rs`).** Deux commandes async de plus, sur le même modèle (toutes deux avec `State<'_, ApiClient>`) :

```rust
#[tauri::command]
async fn update_note(client: State<'_, ApiClient>, id: i64, title: String, content: String)
    -> Result<Note, ApiError> { client.update_note(id, &NoteInput { title, content }).await }

#[tauri::command]
async fn delete_note(client: State<'_, ApiClient>, id: i64) -> Result<(), ApiError> {
    client.delete_note(id).await   // Ok(()) → undefined côté TS
}
```

Le `generate_handler!` final liste **les cinq** : `greet, list_notes, create_note, update_note, delete_note`. Un oubli ici = `invoke` qui échoue à l'exécution.

**Indices côté TypeScript.**

- Branche « édition » de `submitForm()` (quand `editingId !== null`) :

  ```typescript
  await invoke<Note>("update_note", { id: editingId, title, content });
  ```

- `removeNote(id)` :

  ```typescript
  await invoke<void>("delete_note", { id });   // Ok(()) → Promise résolue sans valeur
  if (editingId === id) resetForm();           // la note éditée vient d'être supprimée
  await refreshNotes();
  ```

  Le tout dans un `try`/`catch` → `showError(err)`.

**Point de contrôle.** CRUD complet, à tester dans la fenêtre, API lancée :

1. Créez une note, cliquez « Modifier » (le formulaire bascule en mode édition), changez le texte, « Enregistrer » → la note est mise à jour.
2. « Supprimer » → la note disparaît.
3. Le bandeau d'erreur n'apparaît jamais tant que l'API tourne.

---

## Étape 7 — Robustesse et finitions

**Objectif.** Vérifier le comportement quand l'API est absente, nettoyer le code, et (si webkit est installé) produire un bundle.

**Pourquoi.** Une application desktop doit survivre à un backend éteint sans crasher : c'est la différence entre un prototype et un outil utilisable. Et tout l'intérêt de Tauri se voit au `build` (`COURS.md` §9).

**À faire.**

- **API absente.** Lancez l'app **sans** démarrer l'API. Vous devez voir, immédiatement, le bandeau « Impossible de joindre l'API… ». Démarrez l'API, cliquez « Réessayer » : la liste se charge. Ce parcours est déjà géré par `from_reqwest` (`error.rs`) + le `try`/`catch` du front — vérifiez qu'il marche réellement.
- **Nettoyage.** Retirez l'appel de test à `greet` (étape 1) s'il traîne encore. Dans `lib.rs`, supprimez la ligne `#![allow(unused_variables, unused_imports, dead_code)]` du squelette : votre code utilise désormais tout, et le compilateur ne doit plus rien signaler.
- **Build (si webkit installé).**

  ```bash
  npm run tauri build
  ```

  Cela compile le frontend, compile le Rust en mode release, et produit les **bundles** déclarés dans `tauri.conf.json` (`["deb", "appimage"]`). Vous les trouverez sous `src-tauri/target/release/bundle/`. Notez la **taille** de l'AppImage : quelques mégaoctets, là où un équivalent Electron en pèserait cent ou deux cents (`COURS.md` §9.3).

**Point de contrôle final.**

```bash
cd projet
npm run build          # le TypeScript compile (tsc --noEmit)
# avec webkit :
npm run tauri dev      # l'app tourne, CRUD complet contre l'API du module 5
npm run tauri build    # produit .deb et .AppImage sous target/release/bundle/
```

Quand tout fonctionne : comparez votre code avec `solution/` (notez le helper générique `decode` de `api.rs`, une factorisation propre des 4 appels), puis ouvrez `DEFIS.md`.

> **Sur cette machine précise.** Sans `libwebkit2gtk-4.1-dev` et `librsvg2-dev`, `npm run tauri dev`/`build` échouent à l'édition de liens — ce n'est **pas** un bug de votre code. La partie TypeScript (`npm run build`) reste, elle, entièrement vérifiable. Installez les paquets (encadré prérequis) pour voir la fenêtre.
