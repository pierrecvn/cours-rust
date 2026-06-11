# Projet 4 — Scanner de fichiers parallèle

Vous allez construire un outil en ligne de commande qui parcourt récursivement une arborescence de fichiers et en tire des statistiques :

- nombre de fichiers et taille totale ;
- répartition par extension (nombre de fichiers et taille cumulée) ;
- top 10 des plus gros fichiers.

D'abord en **séquentiel**, puis en **parallèle** avec un pool de threads alimenté par un channel — et un chronomètre pour comparer les deux. Exemple de sortie attendue à la fin :

```text
Scan de /home/vous/projets …

=== Statistiques ===
Fichiers : 12842
Taille totale : 1.7 Gio

--- Par extension (triées par taille) ---
rs                        4210 fichier(s)    812.4 Mio
png                        310 fichier(s)    421.9 Mio
toml                       198 fichier(s)      2.1 Mio
<sans extension>           654 fichier(s)      1.3 Mio
…

--- Top 10 des plus gros fichiers ---
 1.    98.2 Mio  /home/vous/projets/demo/assets/intro.png
 2.    64.0 Mio  /home/vous/projets/big/data.bin
…

=== Temps d'exécution ===
Séquentiel :    412.18ms
Parallèle  :    103.67ms (8 workers)
Accélération : ×3.98
```

**Règles du jeu** : uniquement la bibliothèque standard (pas de `walkdir`, pas de `rayon` — on construit la machinerie soi-même, c'est le but). Le squelette est dans `projet/`, le corrigé dans `solution/` (à ne regarder qu'après avoir vraiment essayé). Tout le texte affiché est en français, tous les identifiants de code en anglais.

---

## Étape 0 — Prise en main du squelette

**Objectif.** Comprendre la structure du projet et vérifier que tout compile.

**Pourquoi.** Comme au module 3, le projet est découpé en bibliothèque (`src/lib.rs`, testable) et binaire (`src/main.rs`, l'interface). Les tests de la logique d'agrégation sont **fournis** dans `tests/stats_tests.rs` : votre travail des étapes 1 à 3 est de les faire passer, sans les modifier.

```text
projet/
├── Cargo.toml
├── src/
│   ├── lib.rs        ← déclare les modules stats et scanner
│   ├── stats.rs      ← agrégation des statistiques (étapes 1 à 3)
│   ├── scanner.rs    ← parcours séquentiel et parallèle (étapes 4 à 6)
│   └── main.rs       ← le binaire (étapes 5 et 7)
└── tests/
    └── stats_tests.rs ← tests fournis, ne pas modifier
```

**À faire.** Lisez `src/stats.rs` et `tests/stats_tests.rs`, puis :

```bash
cd projet
cargo test --no-run    # doit compiler sans erreur
cargo test             # doit échouer… sur des panics « not yet implemented »
```

**Point de contrôle.** `cargo test --no-run` compile. `cargo test` lance les tests et chaque test échoue avec `not yet implemented: étape N …` — c'est votre liste de courses.

---

## Étape 1 — Classer les fichiers par extension

**Objectif.** Implémenter `extension_of` dans `src/stats.rs`.

**Pourquoi.** C'est la clé de regroupement de nos statistiques. Les règles, dictées par les tests : extension en **minuscules** (`photo.JPG` et `photo.jpg` vont ensemble), seule la dernière compte (`archive.tar.gz` → `gz`), et les fichiers sans extension (`Makefile`, `.gitignore`) sont regroupés sous la constante `NO_EXTENSION`.

**Indices.**

- `Path::extension()` renvoie un `Option<&OsStr>` — et il gère déjà les cas piégeux : il renvoie `None` pour `Makefile` *et* pour `.gitignore` (le point initial d'un fichier caché n'introduit pas une extension).
- Un `OsStr` n'est pas une `String` (un nom de fichier Unix peut ne pas être de l'UTF-8 valide !). `to_string_lossy()` fait la conversion en remplaçant les octets invalides — parfait ici.
- `to_lowercase()`, puis un `match` ou un `map_or` sur l'`Option`, et c'est plié : la fonction tient en 4 lignes.

**Point de contrôle.** `cargo test extension` : les 3 tests `extension_*` et `files_without_extension_*` passent.

---

## Étape 2 — Accumuler les statistiques

**Objectif.** Implémenter `Stats::new` et `Stats::add_file`.

**Pourquoi.** `Stats` est le cœur du projet : la structure que la version séquentielle remplira tranquillement, et que les threads se disputeront ensuite. `add_file` doit maintenir quatre choses à jour : le compteur global, la taille totale, l'entrée `by_extension` correspondante, et le classement `largest` (au plus `TOP_N` fichiers, triés par taille décroissante).

**Indices.**

- `Stats` dérive `Default` : `Self::default()` suffit pour `new`.
- Pour `by_extension`, l'idiome `HashMap` vu au module 2 : `self.by_extension.entry(ext).or_default()` renvoie un `&mut ExtensionStats` (créé à zéro si absent) que vous incrémentez.
- Pour `largest`, la stratégie simple est la bonne : `push`, tri décroissant, `truncate(TOP_N)`. Tri décroissant sans `Ordering::reverse` : `sort_by(|a, b| b.size.cmp(&a.size))` (notez le `b` en premier).
- Attention à l'ordre des opérations : `extension_of(&file.path)` emprunte `file`, faites-le **avant** de `push` `file` dans `largest` (le push déplace `file` — le borrow checker vous le rappellera).

**Point de contrôle.** `cargo test add_file` puis `cargo test largest` : les tests `add_file_*` et `largest_keeps_only_the_top_n_sorted_descending` passent.

---

## Étape 3 — Fusionner et formater

**Objectif.** Implémenter `Stats::merge` et `format_size`.

**Pourquoi.** `merge` combine deux `Stats` en un : mêmes règles que `add_file`, mais en gros. Elle ne sert pas dans la version « un seul `Mutex` » du scanner, mais c'est la brique d'une variante plus performante (chaque worker accumule ses propres stats, on fusionne à la fin — vous y reviendrez au défi 1) et elle force à bien comprendre la structure. `format_size` rend les tailles lisibles : `1536` → `1.5 Kio`.

**Indices.**

- `merge` prend `other: Stats` **par valeur** : vous pouvez consommer ses champs. Itérer sur `other.by_extension` (par valeur : `for (ext, ext_stats) in other.by_extension`) donne des `String` possédées, prêtes pour `entry(ext)`.
- Pour `largest` : `extend`, re-tri, `truncate` — comme à l'étape 2.
- `format_size` : divisez par 1024 tant que c'est ≥ 1024 (en `f64`), en avançant dans `["o", "Kio", "Mio", "Gio"]`. Les octets s'affichent sans décimale (`512 o`), le reste avec une décimale (`{value:.1}` → `1.5 Kio`).

**Point de contrôle.** `cargo test` : **tous** les tests passent. La logique métier est finie — place à l'exploration du disque.

---

## Étape 4 — Le parcours récursif

**Objectif.** Implémenter `visit_dirs` dans `src/scanner.rs` : parcourir récursivement un dossier et appeler une closure pour chaque fichier rencontré.

**Pourquoi.** C'est notre `walkdir` maison. La signature mérite un regard :

```rust
pub fn visit_dirs<F: FnMut(PathBuf)>(dir: &Path, on_file: &mut F) -> io::Result<()>
```

Une fonction générique sur une closure `FnMut` (module 3) : le *parcours* est séparé de l'*action*. La même fonction servira au scan séquentiel (la closure remplit un `Stats`) et au scan parallèle (la closure envoie dans un channel). On passe `&mut F` plutôt que `F` pour pouvoir réutiliser la même closure dans les appels récursifs.

**Indices.**

- `std::fs::read_dir(dir)?` renvoie un itérateur de `Result<DirEntry, _>` — chaque entrée peut échouer individuellement, d'où le `?` à deux niveaux : `for entry in fs::read_dir(dir)? { let entry = entry?; … }`.
- `entry.file_type()?` dit si l'entrée est un fichier, un dossier… ou un lien symbolique. Testez `is_symlink()` **en premier** et ignorez les liens : un lien vers un dossier parent créerait une récursion infinie.
- Dossier → appel récursif `visit_dirs(&path, on_file)` ; fichier → `on_file(path)`.
- Gestion d'erreurs nuancée : si `read_dir` échoue sur le dossier *racine*, l'erreur remonte (`?`). Mais un sous-dossier illisible (permissions) ne doit pas faire tout échouer : entourez l'appel récursif d'un `if let Err(err) = …` qui affiche un avertissement sur `eprintln!` et continue.

**Point de contrôle.** Pas encore de test automatique (ça vient à l'étape 5), mais vous pouvez vérifier à la main depuis `main` ou avec un petit test à vous : `visit_dirs(Path::new("src"), &mut |p| println!("{}", p.display()))` doit lister `lib.rs`, `main.rs`, `scanner.rs`, `stats.rs`.

---

## Étape 5 — La version séquentielle, chronométrée

**Objectif.** Implémenter `scan_sequential`, puis brancher le tout dans `main` avec un chronomètre.

**Pourquoi.** Toujours faire marcher la version simple avant de paralléliser : elle sert de référence de **correction** (les deux versions doivent trouver les mêmes nombres) et de référence de **performance** (que vaut une accélération sans point de comparaison ?).

**Indices.**

- `scan_sequential` : créez un `Stats` mutable, appelez `visit_dirs` avec une closure qui capture `&mut stats` — pour chaque chemin, `fs::metadata(&path)` donne la taille (`meta.len()`), et `stats.add_file(FileInfo::new(path, size))`. Un fichier dont les métadonnées sont illisibles : avertissement sur `eprintln!` et on continue.
- Vous venez d'écrire une closure `FnMut` qui mute son environnement — exactement pourquoi la signature de `visit_dirs` demande `FnMut` et pas `Fn`.
- Dans `main` (le squelette parse déjà l'argument) : entourez l'appel de `let start = Instant::now();` … `let elapsed = start.elapsed();`, puis affichez fichiers, taille totale (avec `format_size` !) et durée (`{elapsed:.2?}`).
- `scan_sequential` renvoie un `io::Result<Stats>` : dans `main`, un `match` qui affiche l'erreur sur `eprintln!` et renvoie `ExitCode::FAILURE` en cas d'échec.

**Point de contrôle.** `cargo run -- .` depuis `projet/` affiche un compte de fichiers plausible et un temps. Comparez avec `find . -type f | wc -l` (les liens symboliques en moins). Essayez sur un gros dossier : `cargo run --release -- ~/` — notez le temps, il servira de référence.

---

## Étape 6 — Le pool de workers

**Objectif.** Implémenter `scan_parallel` : N threads workers, un channel comme file de travail, un `Arc<Mutex<Stats>>` pour agréger.

**Pourquoi.** C'est LE morceau du module. L'architecture :

```text
thread principal                      workers (× N)
────────────────                      ─────────────────────────────
visit_dirs(root)                      boucle :
  └─ sender.send(path) ──channel──►     1. recevoir un chemin
drop(sender)                            2. fs::metadata (le gros du travail,
join de chaque worker                      SANS tenir aucun verrou)
récupération du Stats final             3. stats.lock() → add_file (bref !)
                                      sortie de boucle quand le channel ferme
```

Le parcours du disque (rapide) reste sur le thread principal ; les appels `metadata` (nombreux, c'est eux qu'on parallélise) partent aux workers. Deux subtilités font tout l'intérêt de l'exercice :

1. **mpsc = single consumer** : il n'existe qu'un `Receiver`, et N workers veulent piocher dedans. Solution : l'envelopper dans un `Arc<Mutex<Receiver<PathBuf>>>` — chaque worker verrouille, prend *un* chemin, déverrouille.
2. **Le `Stats` partagé** : `Arc<Mutex<Stats>>`, cloné (`Arc::clone`) pour chaque worker.

**Indices.**

- Construction, dans l'ordre : créez le channel, enveloppez le récepteur (`Arc::new(Mutex::new(receiver))`), créez le `Arc<Mutex<Stats>>`, puis une boucle `for _ in 0..num_workers` qui clone les deux `Arc` **avant** le `thread::spawn(move || …)` et stocke chaque `JoinHandle` dans un `Vec`.
- La boucle d'un worker :

  ```rust
  loop {
      let message = receiver.lock().expect("…").recv();
      let path = match message {
          Ok(path) => path,
          Err(_) => break,   // canal fermé : plus de travail
      };
      // metadata + add_file, comme en séquentiel
  }
  ```

  Le détail qui tue : `receiver.lock().expect("…").recv()` en **une seule expression**. Le `MutexGuard` est temporaire, il meurt à la fin de l'instruction — le verrou est donc relâché *avant* `fs::metadata`. Si vous écrivez `let guard = receiver.lock()…; guard.recv()` et gardez `guard` vivant, un seul worker travaille à la fois : votre version « parallèle » devient séquentielle avec des frais en plus. (`recv()` bloque en attendant un message, c'est normal et voulu.)
- Après les spawns : `visit_dirs(root, &mut |path| { let _ = sender.send(path); })`. Puis **`drop(sender)`** — sans lui, les `recv()` attendent un message qui ne viendra jamais et le programme se fige (revoir COURS.md §4). C'est le signal de fin, élégant et sans compteur.
- Joignez chaque handle (`handle.join().expect("un worker a paniqué")`), puis récupérez le `Stats` nu : `Arc::try_unwrap(stats)` (tous les clones sont morts avec les workers, il ne reste qu'un propriétaire) puis `.into_inner()` sur le `Mutex`. Les deux renvoient des `Result` : `expect` avec un message clair.
- Si le compilateur vous parle de `Send`, de `'static` ou de *closure may outlive* : retournez voir COURS.md §3.3 et §6, l'erreur y est décortiquée. C'est presque toujours un `Arc::clone` manquant ou un `move` oublié.

**Point de contrôle.** Ajoutez temporairement dans `main` un appel à `scan_parallel(root, 4)` et vérifiez que `file_count` et `total_size` sont **identiques** au séquentiel sur le même dossier (lancez plusieurs fois : un résultat qui varie d'une exécution à l'autre trahit un bug de synchronisation). `cargo test` passe toujours.

---

## Étape 7 — Le duel et le rapport final

**Objectif.** Dans `main` : chronométrer les deux versions, afficher un rapport complet et l'accélération obtenue.

**Pourquoi.** Un outil se juge à sa sortie. Et la comparaison des temps est la récompense du module : voir ses 8 cœurs enfin utilisés.

**Indices.**

- Nombre de workers : `thread::available_parallelism().map(|n| n.get()).unwrap_or(4)`.
- Enchaînez : scan séquentiel chronométré, scan parallèle chronométré, puis le rapport (construit sur les stats du scan parallèle). Petit garde-fou apprécié : si les deux `file_count` diffèrent, un avertissement.
- Écrivez une fonction `print_report(stats: &Stats)` (elle prend `&Stats` : elle lit, elle ne possède pas). Pour le tableau par extension : `let mut entries: Vec<_> = stats.by_extension.iter().collect();` puis `sort_by` sur la taille décroissante — un `HashMap` n'a pas d'ordre, c'est à vous de trier.
- Alignement : `{:<20}` (cadré à gauche sur 20 colonnes) pour les extensions, `{:>8}` et `{:>12}` (cadrés à droite) pour les nombres. Relisez l'exemple de sortie en tête de ce document.
- L'accélération : `sequential.as_secs_f64() / parallel.as_secs_f64()`, affichée `×{speedup:.2}`.

**Point de contrôle final.**

```bash
cargo test                      # tout vert
cargo run -- .                  # rapport propre sur le dossier du projet
cargo run --release -- ~/       # le vrai test : un gros dossier
```

Sur un gros dossier, attendez-vous à une accélération de ×2 à ×6 selon la machine. **Ne soyez pas surpris** si sur un *petit* dossier le parallèle est plus lent (créer 8 threads coûte plus cher que scanner 40 fichiers), ni si la première exécution est très lente et les suivantes rapides : c'est le cache disque de l'OS — le scan séquentiel, exécuté en premier, « paie » la lecture du disque pour tout le monde. Pour une comparaison équitable, relancez le programme une deuxième fois (tout le monde est alors servi par le cache). Le défi 1 creuse la question.

Quand tout fonctionne : comparez avec `solution/`, puis ouvrez `DEFIS.md`.
