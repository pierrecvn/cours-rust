# Défis du module 6

Trois extensions non guidées, par difficulté croissante. Pas d'étapes détaillées cette fois : un objectif, quelques pistes, à vous de jouer. Le corrigé ne les couvre pas — c'est voulu.

> **Rappel prérequis.** Ces défis se testent dans la **fenêtre** de l'application : ils exigent `libwebkit2gtk-4.1-dev` et `librsvg2-dev` installés (`sudo apt install …`, voir `PROJET.md`) et l'API du module 5 lancée sur `http://localhost:3000`.

---

## Défi 1 — Mode hors-ligne avec cache local

**Objectif.** Quand l'API est joignable, mettez en cache la dernière liste de notes obtenue. Quand elle ne l'est pas (serveur éteint), au lieu d'un bandeau d'erreur sec, affichez la **dernière liste connue** avec un avertissement « Mode hors-ligne — données du \<date\> ». Le cache vit côté Rust, dans le state managé.

**Pistes.**

- C'est le défi annoncé en `COURS.md` §5 (encadré « State mutable ? ») : votre `ApiClient` est en lecture seule, mais un cache se **mute**. Il faut donc un state **mutable** partagé. Le motif est celui du module 4 : `Mutex`. Enregistrez un second state, par exemple `.manage(Mutex::new(NotesCache::default()))`, et recevez-le dans la commande par `cache: State<'_, Mutex<NotesCache>>`.
- Logique de `list_notes` revisitée : tenter l'appel API. En cas d'`Ok`, **écrire** la liste (et l'instant `SystemTime::now()`) dans le cache, puis la renvoyer. En cas d'`Err(Unreachable)`, **lire** le cache : s'il contient quelque chose, renvoyer ces notes (éventuellement enveloppées dans une structure `{ notes, offline: true, cachedAt }` pour informer le front) ; sinon, propager l'erreur comme aujourd'hui.
- Attention au verrou et à l'`async` : ne gardez **jamais** un `MutexGuard` (std) vivant à travers un `.await` (le futur deviendrait `!Send` et Tauri refusera). Verrouillez, copiez/clonez ce dont vous avez besoin, relâchez le verrou **avant** l'appel réseau. Variante propre : utilisez `tokio::sync::Mutex` (son `.lock().await` traverse les points d'attente). Comparez les deux approches, c'est instructif.
- Côté TypeScript : si la réponse signale `offline: true`, affichez une bannière distincte (jaune, pas rouge) avec la date du cache au lieu du bandeau d'erreur. Adaptez l'interface `Note`/le type de retour pour refléter la nouvelle forme.
- Pour aller plus loin : persistez le cache sur disque (un fichier JSON) pour qu'il survive à la fermeture de l'app. Le plugin `tauri-plugin-fs`, ou simplement `std::fs` + `serde_json` côté Rust, font l'affaire — souvenez-vous que le frontend, lui, n'a pas le droit de toucher au disque (modèle de permissions, `COURS.md` §7).

---

## Défi 2 — Raccourcis clavier

**Objectif.** Rendre l'application utilisable sans souris : `Ctrl+N` met le focus sur le champ titre (nouvelle note), `Ctrl+R` rafraîchit la liste, `Échap` annule l'édition en cours (équivalent du bouton « Annuler »).

**Pistes.**

- Le plus simple et le plus portable : rester **côté frontend**. Un `window.addEventListener("keydown", (e) => …)` dans `main.ts`, qui teste `e.ctrlKey && e.key === "n"`, appelle `e.preventDefault()` puis votre fonction (`titleInput.focus()`, `refreshNotes()`, `resetForm()`). Aucune permission Tauri requise : c'est du DOM standard, ça marche dans la webview comme dans un navigateur.
- Subtilité d'ergonomie : `Échap` ne doit annuler l'édition que si on est **en mode édition** (`editingId !== null`) — sinon laissez le comportement par défaut. Et ne déclenchez pas vos raccourcis si l'utilisateur est en train de composer dans un champ avec un raccourci natif attendu (testez `e.target`).
- La variante **Tauri** (raccourcis **globaux**, actifs même quand l'app n'a pas le focus) passe par le plugin `tauri-plugin-global-shortcut`. C'est l'occasion de pratiquer le cycle complet d'ajout d'un plugin : `cargo add tauri-plugin-global-shortcut` dans `src-tauri/`, `.plugin(tauri_plugin_global_shortcut::Builder::new().build())` sur le `Builder`, l'ajout de la permission correspondante dans `capabilities/default.json` (`COURS.md` §7.2), et l'enregistrement des raccourcis. Réservez-le aux raccourcis qui ont du sens hors focus ; pour des raccourcis internes à la fenêtre, le `keydown` DOM est plus simple et suffit.
- Note de portabilité : sur macOS, l'usage est `Cmd` (`e.metaKey`) plutôt que `Ctrl`. Gérez les deux si vous visez plusieurs OS.

---

## Défi 3 — Notification système via un plugin Tauri

**Objectif.** Afficher une **notification native du système d'exploitation** (pas une `<div>` dans la page) après chaque création réussie : « Note créée : \<titre\> ». C'est l'occasion de pratiquer un plugin Tauri de bout en bout — la chaîne crate Rust + permission + appel.

**Pistes.**

- Le plugin officiel est `tauri-plugin-notification`. La chaîne complète à mettre en place (c'est tout l'intérêt du défi) :
  1. **Dépendance Rust** : `cargo add tauri-plugin-notification` dans `src-tauri/`.
  2. **Branchement** : `.plugin(tauri_plugin_notification::init())` sur le `tauri::Builder`.
  3. **Dépendance JS** : `npm install @tauri-apps/plugin-notification`.
  4. **Permission** : ajoutez `"notification:default"` à la liste `permissions` de `capabilities/default.json` (`COURS.md` §7.2). **Sans cette ligne, l'appel sera refusé** par le modèle de sécurité — c'est le piège classique, et c'est précisément le point pédagogique.
- Côté TypeScript, après un `create_note` réussi dans `submitForm()` :

  ```typescript
  import { isPermissionGranted, requestPermission, sendNotification }
    from "@tauri-apps/plugin-notification";

  let granted = await isPermissionGranted();
  if (!granted) granted = (await requestPermission()) === "granted";
  if (granted) sendNotification({ title: "Note créée", body: title });
  ```

  Notez le **double** consentement : la permission Tauri (statique, dans `capabilities/`) autorise le plugin à **exister** ; la permission OS (dynamique, `requestPermission()`) est celle que l'utilisateur accorde à l'exécution, comme dans un navigateur. Les deux sont nécessaires.
- Variante « tout en Rust » : émettez la notification **depuis la commande** `create_note` (le plugin expose une API Rust via `app.notification()`), en ajoutant un paramètre `app: tauri::AppHandle`. Avantage : le frontend n'a même pas besoin de connaître le plugin. Comparez avec l'approche « depuis le front » ci-dessus — c'est le même arbitrage que pour les events (`COURS.md` §8) : qui, du Rust ou du TS, est le mieux placé pour déclencher l'effet ?
- Pour aller plus loin : notifiez aussi les suppressions et, en combinant avec le défi 1, signalez par notification le passage en mode hors-ligne.
