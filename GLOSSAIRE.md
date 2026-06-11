# Glossaire des concepts 🦀

Index de tous les concepts Rust abordés dans le cours, regroupés par module dans l'ordre où on les rencontre. Chaque entrée pointe vers le `COURS.md` où la notion est expliquée en détail. Utilise `Ctrl+F` pour retrouver un terme.

> Astuce : un concept peut réapparaître plus loin avec plus de profondeur (ex. `Result` est introduit au module 1, approfondi au module 2). Le module indiqué est celui où il est **expliqué pour la première fois**.

---

## Module 1 — Fondations

- **ownership (possession)** — chaque valeur a un unique propriétaire ; quand il sort de portée, la valeur est libérée, sans GC — `module-01-fondations/COURS.md`
- **move** — l'affectation d'une valeur non-`Copy` transfère la propriété et invalide l'ancienne variable (erreur E0382 si on l'utilise après) — `module-01-fondations/COURS.md`
- **borrowing (emprunt)** — accéder à une valeur via une référence `&` (lecture partagée) ou `&mut` (écriture exclusive) sans en prendre la propriété — `module-01-fondations/COURS.md`
- **règle des emprunts** — à tout instant : plusieurs `&` OU un seul `&mut`, jamais les deux (erreurs E0502/E0499) — `module-01-fondations/COURS.md`
- **`Copy`** — trait des types simples (`i32`, `f64`, `bool`…) copiés bit à bit à l'affectation au lieu d'être déplacés — `module-01-fondations/COURS.md`
- **`let` / `let mut`** — déclaration de variable, immuable par défaut ; `mut` rend la mutation possible (l'inverse de Java/TS) — `module-01-fondations/COURS.md`
- **shadowing** — redéclarer une variable du même nom pour masquer la précédente, type modifiable, idiomatique pour les transformations successives — `module-01-fondations/COURS.md`
- **`const`** — constante figée à la compilation, type obligatoire, équivalent du `static final` Java — `module-01-fondations/COURS.md`
- **inférence de type** — le compilateur déduit les types localement ; on l'aide par annotation (`let x: u32`) ou turbofish (`parse::<u32>()`) — `module-01-fondations/COURS.md`
- **expression vs instruction** — la dernière expression d'un bloc, sans `;`, est la valeur du bloc ; un `;` la transforme en instruction valant `()` — `module-01-fondations/COURS.md`
- **`if` expression** — `if`/`else` produit une valeur et remplace l'opérateur ternaire `?:` — `module-01-fondations/COURS.md`
- **`loop` / `break` / `continue`** — boucle infinie assumée dont on sort par `break` (qui peut renvoyer une valeur) — `module-01-fondations/COURS.md`
- **range** — intervalle d'itération ou de tirage : `1..5` (fin exclue) ou `1..=5` (inclusif) — `module-01-fondations/COURS.md`
- **`String` vs `&str`** — chaîne possédée, mutable, sur le tas vs vue empruntée en lecture sur du texte appartenant à autrui — `module-01-fondations/COURS.md`
- **`Result`** — type de retour des opérations faillibles (`Ok(valeur)` / `Err(erreur)`) ; l'échec est une valeur, pas une exception — `module-01-fondations/COURS.md`
- **`match`** — expression de pattern matching exhaustive (le compilateur vérifie que tous les cas sont couverts) qui déstructure les valeurs — `module-01-fondations/COURS.md`
- **`Ordering`** — enum à trois variantes (`Less`, `Greater`, `Equal`) retourné par `a.cmp(&b)` — `module-01-fondations/COURS.md`
- **`expect`** — déballe un `Result` ou arrête le programme avec le message donné ; traitement d'erreur minimal des débuts — `module-01-fondations/COURS.md`
- **Cargo** — outil unique du cycle de vie Rust : dépendances, compilation, tests, exécution (npm + tsc + Maven en un) — `module-01-fondations/COURS.md`
- **crate** — paquet Rust, publié sur crates.io et déclaré dans `Cargo.toml` — `module-01-fondations/COURS.md`
- **rustup** — installeur/gestionnaire de toolchains Rust (équivalent de nvm/sdkman) — `module-01-fondations/COURS.md`
- **`todo!()`** — macro qui fait compiler du code inachevé et panique à l'exécution ; idéale pour les squelettes — `module-01-fondations/PROJET.md`
- **test unitaire** — fonction `#[test]` dans un module `#[cfg(test)]`, lancée par `cargo test` — `module-01-fondations/COURS.md`

## Module 2 — Structurer son code

- **structs / impl** — type produit nommant ses champs (`struct Task { id, description, done }`), méthodes et fonctions associées dans un bloc `impl` — `module-02-structurer/COURS.md`
- **enums à données** — type somme dont chaque variante peut transporter ses propres valeurs (`ParseError::InvalidId(String)`) — `module-02-structurer/COURS.md`
- **`Option`** — `Option<T>` : `Some(valeur)` ou `None`, l'absence rendue explicite par le type au lieu du `null` de Java/TS — `module-02-structurer/COURS.md`
- **`Result` (approfondi)** — `Result<T, E>` : `Ok(valeur)` ou `Err(erreur)`, la faillibilité encodée dans le type de retour — `module-02-structurer/COURS.md`
- **opérateur `?`** — propage l'erreur (`Err`)/l'absence (`None`) vers l'appelant, ou déballe la valeur en cas de succès — `module-02-structurer/COURS.md`
- **match exhaustif** — aiguillage par filtrage de motifs que le compilateur force à couvrir tous les cas — `module-02-structurer/COURS.md`
- **`if let`** — déballe un seul motif intéressant sans le `match` complet (`if let Err(e) = run(cmd) { ... }`) — `module-02-structurer/COURS.md`
- **`Vec`** — tableau dynamique possédant ses éléments sur le tas, équivalent de l'`ArrayList`/array — `module-02-structurer/COURS.md`
- **`HashMap`** — table de hachage clé→valeur, avec l'API `entry`/`or_insert` pour insérer-ou-modifier — `module-02-structurer/COURS.md`
- **modules `mod` / `pub` / `use`** — `mod` rattache un fichier à l'arbre, `pub` expose un élément, `use` importe un chemin — `module-02-structurer/COURS.md`
- **`derive` Debug / Clone / PartialEq** — `#[derive(...)]` génère automatiquement l'affichage `{:?}`, la copie explicite `.clone()` et la comparaison `==` — `module-02-structurer/COURS.md`

## Module 3 — CLI professionnel

- **trait** — contrat de comportement (ensemble de méthodes) qu'un type promet de fournir, équivalent souple d'une interface Java/TS implémentable même sur des types existants — `module-03-cli-pro/COURS.md`
- **méthodes par défaut** — implémentation fournie par le trait lui-même, que les types peuvent garder ou redéfinir, comme les méthodes `default` des interfaces Java 8+ — `module-03-cli-pro/COURS.md`
- **`Display`** — trait de l'affichage destiné aux humains (`{}`), écrit à la main, qui offre gratuitement `to_string()` — `module-03-cli-pro/COURS.md`
- **`From` / `Into`** — traits de conversion : implémenter `From<A> for B` donne `Into` dans l'autre sens et alimente l'opérateur `?` pour convertir les erreurs — `module-03-cli-pro/COURS.md`
- **`Default`** — trait fournissant la valeur par défaut d'un type, dérivable, exploité par `#[serde(default)]` et la syntaxe `..Default::default()` — `module-03-cli-pro/COURS.md`
- **génériques + bounds** — code paramétré par un type `T` contraint par les capacités requises (`T: PartialOrd`), déclarées explicitement car Rust ne suppose rien — `module-03-cli-pro/COURS.md`
- **monomorphisation** — génération à la compilation d'une version spécialisée par type concret utilisé, à coût d'exécution nul (contraire de l'effacement de type Java) — `module-03-cli-pro/COURS.md`
- **`dyn` vs `impl Trait`** — `impl Trait`/génériques = dispatch statique choisi à la compilation ; `dyn Trait` = dispatch dynamique via vtable, choisi à l'exécution, pour mélanger des types — `module-03-cli-pro/COURS.md`
- **closures `Fn` / `FnMut` / `FnOnce`** — fonctions anonymes capturant leur environnement ; le trait implémenté dépend de l'usage des captures (lecture / mutation / consommation) — `module-03-cli-pro/COURS.md`
- **itérateurs paresseux** — les adaptateurs ne calculent rien tant qu'un consommateur ne tire pas les valeurs, permettant des pipelines efficaces sans allocations intermédiaires — `module-03-cli-pro/COURS.md`
- **`map` / `filter` / `collect` / `fold`** — adaptateurs (transformer/sélectionner) et consommateurs (matérialiser/réduire) du trait `Iterator` — `module-03-cli-pro/COURS.md`
- **crates.io / `cargo add`** — registre central des paquets Rust et commande qui ajoute une dépendance versionnée (semver) au `Cargo.toml` — `module-03-cli-pro/COURS.md`
- **serde** — framework de (dé)sérialisation par derive `Serialize`/`Deserialize`, indépendant du format, sans réflexion, vérifié à la compilation — `module-03-cli-pro/COURS.md`
- **clap derive** — parseur d'arguments CLI qui génère parsing, `--help`, `--version` et erreurs à partir d'une struct annotée `#[derive(Parser)]` — `module-03-cli-pro/COURS.md`
- **thiserror** — derive qui élimine le code rituel des types d'erreur concrets d'une bibliothèque (`#[error]` pour `Display`, `#[from]` pour `From`) — `module-03-cli-pro/COURS.md`
- **anyhow** — type d'erreur valise pour les binaires, absorbant n'importe quelle erreur et ajoutant du contexte via `.with_context(...)` — `module-03-cli-pro/COURS.md`
- **tests d'intégration** — fichiers de `tests/` compilés comme des crates indépendantes ne voyant que l'API publique de la bibliothèque — `module-03-cli-pro/COURS.md`
- **organisation lib + bin** — `src/lib.rs` porte toute la logique testable, `src/main.rs` est une fine couche CLI qui la consomme comme une dépendance externe — `module-03-cli-pro/COURS.md`

## Module 4 — Concurrence & lifetimes

- **lifetimes** — durées de vie qui garantissent qu'une référence ne survit jamais à la donnée qu'elle pointe — `module-04-concurrence/COURS.md`
- **annotation `'a`** — nom de lifetime explicite reliant les références d'une signature (entrée/sortie) pour le compilateur — `module-04-concurrence/COURS.md`
- **élision** — règles qui laissent le compilateur déduire les lifetimes courantes sans qu'on les écrive — `module-04-concurrence/COURS.md`
- **`Box`** — pointeur intelligent qui possède une valeur allouée sur le tas, taille connue côté pile — `module-04-concurrence/COURS.md`
- **`Rc`** — compteur de références mono-thread permettant plusieurs propriétaires d'une même donnée — `module-04-concurrence/COURS.md`
- **`RefCell`** — conteneur à emprunt vérifié à l'exécution (et non à la compilation), peut paniquer si violé — `module-04-concurrence/COURS.md`
- **interior mutability** — mutation d'une donnée derrière une référence partagée, autorisée via Cell/RefCell/Mutex — `module-04-concurrence/COURS.md`
- **`thread::spawn`** — lance un nouveau thread OS exécutant une closure, contrainte `'static` + `Send` — `module-04-concurrence/COURS.md`
- **`join`** — attend la fin d'un thread via son `JoinHandle` et récupère son résultat — `module-04-concurrence/COURS.md`
- **move closure** — closure qui prend possession des variables capturées, indispensable pour les envoyer à un thread — `module-04-concurrence/COURS.md`
- **channel mpsc** — file multi-producteurs / consommateur unique pour passer des messages entre threads — `module-04-concurrence/COURS.md`
- **`Arc`** — compteur de références atomique (thread-safe), version multi-thread de `Rc` — `module-04-concurrence/COURS.md`
- **`Mutex`** — verrou d'exclusion mutuelle protégeant une donnée partagée, accès via `lock()` — `module-04-concurrence/COURS.md`
- **`Send`** — trait marquant un type transférable en toute sécurité vers un autre thread — `module-04-concurrence/COURS.md`
- **`Sync`** — trait marquant un type partageable par référence entre threads (`&T` est `Send`) — `module-04-concurrence/COURS.md`

## Module 5 — Backend web

- **async / await** — syntaxe qui suspend une fonction en attendant un résultat, libérant le runtime pour d'autres tâches — `module-05-backend-web/COURS.md`
- **future paresseuse** — valeur représentant un résultat futur qui ne s'exécute que sous un runtime ou via `.await` (contrairement aux Promises TS) — `module-05-backend-web/COURS.md`
- **runtime** — moteur externe (Tokio) qui exécute les futures, Rust ne fournissant pas d'exécuteur intégré — `module-05-backend-web/COURS.md`
- **tokio** — runtime asynchrone de référence pour le réseau : executor, multiplexage sur les cœurs, I/O non bloquantes — `module-05-backend-web/COURS.md`
- **`#[tokio::main]`** — attribut transformant un `async fn main` en `fn main` qui démarre le runtime puis exécute la future — `module-05-backend-web/COURS.md`
- **Axum Router** — table de routage associant méthode HTTP + chemin à un handler (`get(h).post(h)`, paramètre `{id}`) — `module-05-backend-web/COURS.md`
- **handler** — `async fn` aux paramètres extracteurs et au retour convertible en réponse HTTP — `module-05-backend-web/COURS.md`
- **extracteur Path / Query / Json / State** — type déclaré en paramètre qu'Axum extrait et valide depuis la requête, échec → 4xx automatique — `module-05-backend-web/COURS.md`
- **`IntoResponse`** — trait transformant une valeur de retour en réponse HTTP (`Json`, `(StatusCode, Json)`, `Result`) — `module-05-backend-web/COURS.md`
- **SQLx pool** — ensemble de connexions recyclées à la base, déjà thread-safe (contient un `Arc`), partagé sans `Mutex` — `module-05-backend-web/COURS.md`
- **`query_as` (liaison à l'exécution)** — requête SQLx mappant les lignes vers une struct via `FromRow`, SQL vérifié à l'exécution (sans macro `!` ni `DATABASE_URL`) — `module-05-backend-web/COURS.md`
- **gestion d'erreur HTTP** — enum d'erreur du domaine traduit en code de statut via `IntoResponse` (404/500), `match` exhaustif — `module-05-backend-web/COURS.md`
- **tower `oneshot`** — méthode envoyant une requête au Router en mémoire, sans serveur ni port, pour les tests d'API — `module-05-backend-web/COURS.md`

## Module 6 — App desktop (Tauri)

- **Tauri 2** — framework d'app desktop/mobile à UI web (HTML/CSS/TS) et logique Rust, via un pont IPC — `module-06-desktop-tauri/COURS.md`
- **webview système vs Electron** — Tauri utilise le moteur web de l'OS (0 Mo embarqué) là où Electron embarque Chromium (~150 Mo) : bundle et mémoire bien moindres — `module-06-desktop-tauri/COURS.md`
- **commande `#[tauri::command]`** — fonction Rust exposée au frontend, seul moyen pour le code web de déclencher du Rust ; listée dans `generate_handler!` — `module-06-desktop-tauri/COURS.md`
- **`invoke`** — appel côté TS d'une commande Rust par son nom (`invoke<T>("nom", { args })`), RPC local typé rendant une `Promise` — `module-06-desktop-tauri/COURS.md`
- **capabilities / permissions** — liste blanche déclarative (JSON) de ce que le frontend a le droit de faire ; `core:default` est le socle minimal — `module-06-desktop-tauri/COURS.md`
- **state managé** — objet enregistré une fois par `.manage()` et injecté dans les commandes via `State<'_, T>` (injection de dépendances vérifiée à la compilation) — `module-06-desktop-tauri/COURS.md`
- **`reqwest`** — client HTTP async de Rust (un `fetch` typé) qui sérialise les requêtes et valide les réponses JSON contre les structs serde — `module-06-desktop-tauri/COURS.md`
- **pont sérialisé** — l'IPC Tauri : tout passe en JSON entre TS et Rust ; un `Result::Err` Rust devient une `Promise` rejetée côté TS — `module-06-desktop-tauri/COURS.md`
- **`tauri build` / bundles** — compile front + Rust en release et empaquette en installeurs natifs (`.deb`, `.AppImage` sous Linux) — `module-06-desktop-tauri/COURS.md`

## Module 7 — WebAssembly

- **WebAssembly (WASM)** — format binaire portable et sandboxé, cible de compilation exécutée par le navigateur à vitesse proche du natif — `module-07-wasm/COURS.md`
- **wasm-bindgen** — crate qui génère la « glue » Rust↔JS pour passer chaînes, buffers et objets au-delà des simples nombres — `module-07-wasm/COURS.md`
- **wasm-pack** — outil CLI qui compile la crate vers WASM, lance wasm-bindgen et produit un dossier `pkg/` prêt à l'emploi (`.wasm` + glue + `.d.ts`) — `module-07-wasm/COURS.md`
- **cible wasm32-unknown-unknown** — triplet de compilation Rust (architecture WASM 32 bits, sans fournisseur ni OS) produisant le binaire `.wasm` — `module-07-wasm/COURS.md`
- **crate-type cdylib** — type d'artefact à interface C qui, pour la cible WASM, génère le `.wasm` attendu par wasm-bindgen (couplé à `rlib` pour `cargo test`) — `module-07-wasm/COURS.md`
- **interop JS** — frontière où JS pilote et WASM calcule, n'échangeant nativement que des nombres, le reste transitant via la glue générée — `module-07-wasm/COURS.md`
- **ImageData / RGBA** — buffer plat de 4 octets par pixel (R, V, B, A de 0 à 255), manipulé côté Rust comme une tranche `&mut [u8]` via `chunks_exact_mut(4)` — `module-07-wasm/COURS.md`
- **coût des copies JS↔WASM** — mémoires étanches : chaque chaîne ou buffer traversant la frontière est copié, d'où la règle « peu d'appels gros plutôt que beaucoup de petits » — `module-07-wasm/COURS.md`

## Module 8 — Jeu 2D (Bevy)

- **game loop (boucle de jeu)** — cycle qui à chaque image lit les entrées, met à jour l'état du monde, puis dessine — `module-08-jeu-bevy/COURS.md`
- **ECS (Entity-Component-System)** — architecture qui sépare les données (composants) du comportement (systèmes), à l'inverse de la POO où l'objet réunit les deux — `module-08-jeu-bevy/COURS.md`
- **entity (entité)** — simple identifiant qui ne contient rien, mais auquel on attache des composants — `module-08-jeu-bevy/COURS.md`
- **component (composant)** — donnée pure attachée à une entité (`struct` marquée `#[derive(Component)]`), sans logique — `module-08-jeu-bevy/COURS.md`
- **system (système)** — fonction libre qui interroge des composants/ressources et applique le comportement ; le moteur peut en exécuter plusieurs en parallèle — `module-08-jeu-bevy/COURS.md`
- **Bevy `App` / plugins** — le conteneur qui enregistre ressources, systèmes et états ; les plugins regroupent et réutilisent ces enregistrements — `module-08-jeu-bevy/COURS.md`
- **`Commands`** — file d'opérations différées pour créer (`spawn`) ou supprimer (`despawn`) des entités pendant l'exécution d'un système — `module-08-jeu-bevy/COURS.md`
- **`Query`** — paramètre de système qui itère sur les entités possédant un ensemble de composants donné — `module-08-jeu-bevy/COURS.md`
- **`With` / `Without`** — filtres de `Query` qui restreignent aux entités ayant (ou non) un composant, sans le lire — `module-08-jeu-bevy/COURS.md`
- **`Res` / `ResMut`** — accès en lecture / écriture à une ressource, donnée globale unique du monde (score, minuteur, état) — `module-08-jeu-bevy/COURS.md`
- **ressource** — donnée singleton partagée par tous les systèmes, hors du modèle entité/composant — `module-08-jeu-bevy/COURS.md`
- **event / message** — valeur émise par un système et lue par un autre, pour communiquer sans couplage direct — `module-08-jeu-bevy/COURS.md`
- **`States`** — machine à états du jeu (Menu / EnJeu / GameOver) conditionnant l'exécution des systèmes — `module-08-jeu-bevy/COURS.md`
- **`Time` / `Timer`** — ressource du temps écoulé et minuteur qui déclenche une action à intervalle fixe (déplacement du serpent) — `module-08-jeu-bevy/COURS.md`
- **`Transform`** — composant de position/rotation/échelle d'une entité dans le monde 2D/3D — `module-08-jeu-bevy/COURS.md`
- **`Sprite`** — composant d'affichage 2D (ici de simples rectangles colorés, sans asset externe) — `module-08-jeu-bevy/COURS.md`
- **ordonnancement des systèmes** — organisation de l'exécution via `Update`, le chaînage `.chain()` et les run conditions (conditions d'exécution) — `module-08-jeu-bevy/COURS.md`
- **logique pure vs adaptateur** — toute la logique de jeu testable vit dans `logic.rs` sans Bevy ; les systèmes ne font que la brancher au moteur — `module-08-jeu-bevy/COURS.md`
