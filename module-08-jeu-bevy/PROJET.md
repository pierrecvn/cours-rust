# Projet 8 — Snake avec Bevy

Vous allez construire un **Snake jouable** : un serpent qui grandit en mangeant, meurt contre les murs ou contre lui-même, avec un score, un menu, un écran de game over et un redémarrage. Le tout en Rust, avec le moteur **Bevy** et l'architecture **ECS**.

À la fin, `cargo run` ouvre une fenêtre comme celle-ci :

```text
┌──────────────────────────────┐
│          Score : 7           │
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │
│ ░░░░░░░░░░░██░░░░░░░░░░░░░░░░ │   ██ = nourriture (rouge)
│ ░░░░░░░▓▓▓▓▒▒░░░░░░░░░░░░░░░░ │   ▓▓ = tête (vert vif)
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │   ▒▒ = corps (vert)
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │
└──────────────────────────────┘
   Flèches ou WASD pour diriger
```

**L'idée d'architecture, à garder en tête du début à la fin.** Le jeu est coupé en deux :

- `src/logic.rs` — **les règles du jeu**, en Rust pur, **sans une ligne de Bevy**. Testable unitairement. C'est votre travail des **étapes 1 à 5**, validé par les tests fournis (`tests/logic_test.rs`).
- `src/main.rs` — **l'adaptateur Bevy**, qui branche cette logique sur une fenêtre et un clavier. C'est votre travail des **étapes 6 à 11**. On le vérifie en **jouant** (`cargo run`), pas avec des tests.

> ⚠️ **Bevy compile lentement.** Le premier `cargo build`/`cargo test`/`cargo run` télécharge et compile tout le moteur : comptez **plusieurs minutes**, et plusieurs Go de disque. C'est normal, ne l'interrompez pas. Les compilations suivantes sont rapides. Pensez à `cargo clean` quand vous quittez le module. Lisez d'abord `COURS.md` en entier : ce document s'appuie dessus à chaque étape.

---

## Étape 0 — Prise en main du squelette

**Objectif.** Comprendre la structure et vérifier que tout compile.

**Pourquoi.** Comme aux modules 3 et 4, le projet est en bibliothèque + binaire :

```text
projet/
├── Cargo.toml          ← dépendances (bevy, rand) et profil de compilation
├── src/
│   ├── lib.rs          ← expose le module `logic` (déjà fait)
│   ├── logic.rs        ← LES RÈGLES, en Rust pur (étapes 1 à 5)
│   └── main.rs         ← l'adaptateur Bevy (étapes 6 à 11)
└── tests/
    └── logic_test.rs   ← tests fournis de la logique, NE PAS modifier
```

Les corps des méthodes et systèmes à écrire sont remplacés par `todo!("étape N: …")` : c'est votre liste de courses. Le `Cargo.toml` contient un détail important, déjà réglé pour vous :

```toml
[profile.dev.package."*"]
opt-level = 3
```

Il compile **les dépendances** (donc Bevy) avec les optimisations maximales, tout en gardant **votre** code en compilation rapide. Sans cela, Bevy tourne à un framerate catastrophique en mode debug. Ne le retirez pas.

**À faire.**

```bash
cd projet
cargo test --no-run    # compile tout (LONG la 1re fois) — doit réussir
cargo test logic       # lance les tests de la logique — ils échouent sur des « not yet implemented »
```

**Point de contrôle.** `cargo test --no-run` compile sans erreur (après une longue première compilation). `cargo test` montre des panics `not yet implemented: étape N …` : exactement les fonctions que vous allez écrire.

---

## PARTIE A — La logique pure (`src/logic.rs`)

> Dans toute cette partie, **aucun `use bevy::…`**. On écrit des règles de jeu comme une bibliothèque ordinaire. Lancez `cargo test <mot-clé>` pour tester un sous-ensemble (les noms de tests sont préfixés `direction_`, `new_`, `turn_`, `step_`, `free_`/`food_`).

## Étape 1 — La géométrie : `Coord` et `Direction`

**Objectif.** Implémenter `Coord::in_bounds`, `Coord::moved`, `Direction::opposite`, `Direction::delta`.

**Pourquoi.** Ce sont les briques de base : une case (`Coord`), les quatre directions, et les deux opérations dont tout le reste dépend — « cette case est-elle dans la grille ? » et « quelle case en allant par là ? ». La convention du repère est **`y` vers le haut** (comme Bevy) : `Up` augmente `y`.

**Indices.**

- `in_bounds` : `(0..GRID_WIDTH).contains(&self.x)` et de même pour `y`. La grille fait `GRID_WIDTH × GRID_HEIGHT` cases, indices `0` à `GRID-1`.
- `opposite` et `delta` : un simple `match` sur les quatre variantes. `delta` renvoie un `Coord` utilisé comme vecteur : `Up → (0, 1)`, `Down → (0, -1)`, `Left → (-1, 0)`, `Right → (1, 0)`.
- `moved` : déstructurez le delta puis additionnez — `let Coord { x: dx, y: dy } = dir.delta();` et renvoyez `Coord::new(self.x + dx, self.y + dy)`.

**Point de contrôle.** `cargo test direction` : les quatre tests `direction_*` passent.

---

## Étape 2 — L'état d'une partie : `Game::new` et les accesseurs

**Objectif.** Implémenter `Game::new`, `head`, `len`, `occupies` (et vérifier que `body`, `score`, `food` sont déjà fournis).

**Pourquoi.** `Game` est le cœur de la logique : tout l'état d'une partie. Le serpent est un `VecDeque<Coord>`, **tête en premier** (`front`), queue en dernier (`back`) — `VecDeque` permet de pousser devant et retirer derrière en O(1), exactement ce qu'un serpent fait à chaque pas.

**Indices.**

- `new` : tête au centre `(GRID_WIDTH / 2, GRID_HEIGHT / 2)`, puis deux segments vers la gauche (`cx - 1`, `cx - 2`). Construisez avec `VecDeque::from([tête, milieu, queue])`. Direction et `pending` initiales : `Right`. `score : 0`. `food` : le paramètre reçu (la logique ne tire **jamais** l'aléatoire elle-même).
- `head` : `self.body.front()` renvoie une `Option` — un `.expect("…")` est légitime car l'invariant « le corps n'est jamais vide » est garanti par le reste du code.
- `len` : la taille du `VecDeque`. `occupies` : `self.body.contains(&cell)`.

**Point de contrôle.** `cargo test new_game` : les tests `new_game_*` passent (trois segments, tête centrée, corps vers la gauche, score nul, nourriture mémorisée).

---

## Étape 3 — Tourner sans demi-tour : `try_turn`

**Objectif.** Implémenter `Game::try_turn`.

**Pourquoi.** Un serpent ne peut pas faire **demi-tour** (il se mordrait instantanément). `try_turn` enregistre la direction *demandée* (`pending`) pour le prochain pas, mais **refuse** un demi-tour.

**Le piège subtil (lisez bien).** On compare la demande à `self.direction` — la direction **réellement suivie au dernier pas** — et **non** à `pending`. Sinon, en allant à droite, presser Haut *puis* Gauche très vite entre deux ticks ferait passer `pending` à `Left`, soit un demi-tour, soit une mort injuste. C'est un bug présent dans une foule de Snake amateurs. Le test `turn_double_input_between_ticks_cannot_reverse` le vérifie expressément.

**Indices.**

- Une ligne : `if dir != self.direction.opposite() { self.pending = dir; }`. Sinon, on ignore silencieusement.

**Point de contrôle.** `cargo test turn` : les trois tests `turn_*` passent (virage perpendiculaire accepté, demi-tour ignoré, double-appui ne peut pas inverser).

---

## Étape 4 — Le pas de simulation : `step`

**Objectif.** Implémenter `Game::step` — LA fonction centrale du jeu.

**Pourquoi.** Un appel à `step` = un pas du serpent. Elle renvoie un `StepOutcome` (`Moved` / `Ate` / `Died`) : la logique **constate**, l'adaptateur **réagira**. Deux pièges classiques du Snake se nichent ici ; les tests les traquent.

**L'algorithme, dans l'ordre exact :**

1. La direction demandée devient effective : `self.direction = self.pending`.
2. Calculez la case visée : `let new_head = self.head().moved(self.direction);`.
3. **Mur ?** Si `!new_head.in_bounds()` → `return StepOutcome::Died`.
4. **Nourriture ?** Si `new_head == self.food` : `push_front(new_head)`, `self.score += 1`, `return StepOutcome::Ate`. **On ne retire PAS la queue** : c'est ainsi que le serpent grandit.
5. **Déplacement normal :** `self.body.pop_back()` (la queue libère sa case) **AVANT** de tester la collision. Puis : si `self.occupies(new_head)` → `Died` ; sinon `push_front(new_head)` et `StepOutcome::Moved`.

**Les deux pièges.**

- *Manger fait grandir* (piège 1) : à l'étape 4, on ajoute la tête sans retirer la queue → +1 segment.
- *La case que la queue libère* (piège 2) : à l'étape 5, on retire la queue **avant** le test de collision. Quand le serpent tourne en carré serré, sa tête a le droit d'entrer dans la case que la queue quitte au même instant. Inverser l'ordre = mort injuste. Les tests `step_self_collision_dies` et `step_can_enter_cell_freed_by_tail` distinguent précisément ces deux cas.

**Point de contrôle.** `cargo test step` : les sept tests `step_*` passent (avance par défaut, manger grandit et marque un point, murs droite et bas, auto-collision, case libérée par la queue).

---

## Étape 5 — La nourriture : `place_food` et `free_cells`

**Objectif.** Implémenter `Game::place_food` et `Game::free_cells`.

**Pourquoi.** La logique ne tire pas l'aléatoire, mais elle doit fournir à l'adaptateur **la liste des cases libres** (`free_cells`) pour qu'il y tire la prochaine nourriture, et un moyen de la **poser** (`place_food`). L'aléatoire reste dehors ; le déterminisme reste dedans.

**Indices.**

- `place_food` : une ligne, `self.food = cell;`.
- `free_cells` : toutes les cases `(x, y)` de la grille telles que `!self.occupies((x, y))`. Idiome itérateur : `(0..GRID_WIDTH).flat_map(|x| (0..GRID_HEIGHT).map(move |y| Coord::new(x, y)))` puis `.filter(...).collect()`. Attention au `move` dans la closure intérieure (elle capture `x`).

**Point de contrôle.**

```bash
cargo test        # TOUS les tests de logique passent (le dossier tests/ est vert)
```

La logique est complète et prouvée correcte, **sans jamais avoir ouvert de fenêtre**. C'est tout l'intérêt de l'avoir isolée. Place au moteur.

---

## PARTIE B — L'adaptateur Bevy (`src/main.rs`)

> Ici, on branche la logique sur Bevy. Le **câblage** (l'`App`, les systèmes enregistrés, les composants, ressources, messages, et les fonctions de conversion grille→pixels) est **déjà écrit**. Vous remplissez le **corps** des systèmes marqués `todo!()`. On vérifie en **jouant** : `cargo run`. Gardez `COURS.md` §4 à §13 ouvert.

## Étape 6 — La caméra, le menu et le démarrage

**Objectif.** Implémenter `setup_camera`, `spawn_menu_text`, `start_on_space`, `despawn_menu`.

**Pourquoi.** Avant tout jeu, il faut une **caméra** (sinon, écran noir) et un **écran d'accueil**. C'est aussi la première rencontre avec `Commands`, `spawn`, `Text2d` et les `States`.

**Indices.**

- `setup_camera` : `commands.spawn(Camera2d);`. Une caméra 2D suffit (COURS §10).
- `spawn_menu_text` : `commands.spawn((Text2d::new("SNAKE\n\nEspace pour jouer"), TextFont { font_size: 36.0, ..default() }, TextColor(COLOR_TEXT), OnMenuScreen));`. L'étiquette `OnMenuScreen` permettra de tout nettoyer d'un coup.
- `start_on_space` : `if keyboard.just_pressed(KeyCode::Space) { next_state.set(AppState::Playing); }`. **`just_pressed`** et non `pressed` (COURS §9) : sinon on validerait en rafale.
- `despawn_menu` : `for entity in &query { commands.entity(entity).despawn(); }`.

**Point de contrôle.** `cargo run`. Une fenêtre s'ouvre, fond sombre, le texte « SNAKE — Espace pour jouer » au centre. Appuyer sur Espace fait disparaître le menu (l'écran de jeu n'est pas encore peuplé — c'est l'étape suivante). Le programme **panique** dès qu'on appuie sur Espace, sur `todo!("étape 7: spawn_game")` : c'est attendu, on n'a pas encore écrit la partie.

---

## Étape 7 — Démarrer une partie : `spawn_game`

**Objectif.** Implémenter `spawn_game`, appelé à l'entrée dans l'état `Playing`.

**Pourquoi.** C'est le **pont** entre les deux mondes : on crée le `Game` de la logique pure, on l'enveloppe dans la ressource `GameState`, on installe le minuteur et le score. Ici, et seulement ici, on **tire l'aléatoire** pour la première nourriture (COURS §14 : l'aléatoire est dans l'adaptateur).

**Indices.**

- RNG : `let mut rng = rand::rng();`. La case libre : `game.free_cells().choose(&mut rng)` (le trait `IndexedRandom` est déjà importé). Posez-la : `game.place_food(cell)` (rendez `game` mutable).
- Ressources : `commands.insert_resource(GameState { game });` et `commands.insert_resource(StepTimer(Timer::from_seconds(STEP_SECONDS, TimerMode::Repeating)));` (COURS §8).
- Le score : spawnez un `Text2d::new("Score : 0")` avec `TextFont`, `TextColor`, les étiquettes **`ScoreText` ET `OnPlayScreen`**, et un `Transform::from_xyz(0.0, GRID_HEIGHT as f32 * CELL_SIZE / 2.0 + 24.0, 1.0)` pour le placer au-dessus de la grille.

**Point de contrôle.** `cargo run`, puis Espace. Le menu disparaît, « Score : 0 » s'affiche en haut. Le serpent n'est pas encore dessiné (`render` est vide) : on n'y est pas tout à fait. Pas de panique au lancement de la partie : bon signe.

---

## Étape 8 — Diriger le serpent : `handle_input`

**Objectif.** Implémenter `handle_input` : transformer les touches en appel à `try_turn`.

**Pourquoi.** C'est la frontière entrée→logique. Ce système **lit le clavier** et **délègue la décision** à la logique pure. Il ne connaît, lui, aucune règle (le refus du demi-tour est dans `try_turn`, étape 3).

**Indices.**

- Construisez une `Option<Direction>` avec une chaîne de `if / else if`. Pour accepter flèches **et** WASD : `keyboard.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW])` pour le haut, etc. (`KeyCode::KeyW/KeyA/KeyS/KeyD`).
- Si une direction a été pressée : `state.game.try_turn(direction);`.

**Point de contrôle.** Pas encore d'effet visible (le serpent ne bouge qu'une fois `step_game` écrit), mais ça doit compiler. On enchaîne directement.

---

## Étape 9 — Faire vivre le jeu : `step_game` et `place_new_food`

**Objectif.** Implémenter `step_game` (cadencé par le minuteur) et `place_new_food` (réaction au message).

**Pourquoi.** Le serpent doit avancer **à intervalle régulier**, pas à 60 images/s. Et quand il mange, la nourriture doit réapparaître ailleurs — mais c'est un **autre** système qui s'en charge, prévenu par un **message** (COURS §8 et §12).

**Indices.**

- `step_game` : commencez par le minuteur — `if !timer.0.tick(time.delta()).just_finished() { return; }`. Puis `match state.game.step()` :
  - `StepOutcome::Moved` → rien ;
  - `StepOutcome::Ate` → `ate_writer.write(AteFood);` (on **n'replace pas** la nourriture ici) ;
  - `StepOutcome::Died` → `next_state.set(AppState::GameOver);`.
- `place_new_food` : `for _ in ate_reader.read() { … }`. Dans la boucle : `let mut rng = rand::rng();` puis, si `state.game.free_cells().choose(&mut rng)` renvoie une case, `state.game.place_food(cell)`. (Plus de case libre = partie gagnée : on laisse, la prochaine collision finira le jeu.)

**Point de contrôle.** Toujours rien à l'écran tant que `render` est vide (étape 10), mais la machinerie tourne : à la prochaine étape, tout s'animera d'un coup.

---

## Étape 10 — Dessiner : `render` et `update_score_text`

**Objectif.** Implémenter `render` (les sprites) et `update_score_text` (le compteur).

**Pourquoi.** C'est la frontière état→sortie : on traduit l'état de `Game` en carrés colorés. Stratégie volontairement simple (COURS §5) : on **efface** les sprites de l'image précédente et on **recrée** tout depuis `game.body()` et `game.food()`.

**Indices.**

- `render`, dans l'ordre :
  1. effacez l'ancien : `for entity in &segments { commands.entity(entity).despawn(); }` puis pareil pour `&food`.
  2. dessinez le corps : `for (index, cell) in state.game.body().enumerate() { … }`. Couleur : `COLOR_SNAKE_HEAD` si `index == 0`, sinon `COLOR_SNAKE_BODY`. Spawnez `(Sprite::from_color(color, cell_sprite_size()), Transform::from_translation(cell_to_world(cell)), SnakeSegment, OnPlayScreen)`.
  3. dessinez la nourriture : `(Sprite::from_color(COLOR_FOOD, cell_sprite_size()), Transform::from_translation(cell_to_world(state.game.food())), FoodSprite, OnPlayScreen)`.
- `update_score_text` : `**text = Text2d::new(format!("Score : {}", state.game.score()));`. Le `Single` (COURS §7) garantit qu'il y a exactement un texte de score ; le double `**` traverse l'enveloppe d'accès mutable.

**Point de contrôle.** **`cargo run` : le jeu est jouable !** Le serpent avance, tourne aux flèches/WASD, mange (la nourriture saute ailleurs, le score monte). Foncez dans un mur : le serpent meurt… et le jeu **panique** sur `todo!("étape 11: spawn_game_over_text")`. Dernière ligne droite.

---

## Étape 11 — Mourir et recommencer : game over et nettoyage

**Objectif.** Implémenter `despawn_play_screen`, `spawn_game_over_text`, `restart_on_space`.

**Pourquoi.** Boucler le cycle des états (COURS §11) : quand on quitte `Playing`, on **nettoie** tous les sprites ; on **affiche** le score final ; Espace **relance** une partie neuve.

**Indices.**

- `despawn_play_screen` : comme `despawn_menu`, mais sur `Query<Entity, With<OnPlayScreen>>` (tous les sprites + le texte de score portent cette étiquette).
- `spawn_game_over_text` : `commands.spawn((Text2d::new(format!("PERDU\n\nScore : {}\n\nEspace pour rejouer", state.game.score())), TextFont { font_size: 30.0, ..default() }, TextColor(COLOR_TEXT), OnMenuScreen));`. On réutilise l'étiquette `OnMenuScreen` pour profiter de `despawn_menu` à la sortie.
- `restart_on_space` : `if keyboard.just_pressed(KeyCode::Space) { next_state.set(AppState::Playing); }`. Repasser en `Playing` rappelle `spawn_game` (`OnEnter`), qui recrée une partie neuve.

**Point de contrôle final.**

```bash
cargo run
```

Le cycle complet tourne : **Menu → (Espace) → partie → mort → écran de game over avec le score → (Espace) → nouvelle partie**. Le serpent grandit, le score s'incrémente, les murs et l'auto-collision tuent, le demi-tour est impossible. Vérifiez aussi que la logique tient toujours :

```bash
cargo test    # tous les tests de logic.rs restent verts
```

Vous avez un vrai jeu, et une architecture propre : des règles pures et testées d'un côté, un adaptateur moteur de l'autre. Comparez avec `solution/` si vous voulez, puis attaquez `DEFIS.md`.

> Quand vous quittez le module, `cargo clean` dans `projet/` (et `solution/`) : Bevy laisse plusieurs Go dans `target/`.
