# Défis du module 8

Trois extensions non guidées, par difficulté croissante. Pas d'étapes détaillées : un objectif, quelques pistes, à vous de jouer. Le corrigé ne les couvre pas — c'est voulu.

Avant de commencer, gardez en tête la règle d'or du module : **ce qui est une règle de jeu va dans `logic.rs` (et se teste) ; ce qui touche au moteur va dans `main.rs`.** Chaque défi vous fera trancher de ce côté-là.

---

## Défi 1 — Vitesse croissante

**Objectif.** Faire **accélérer** le jeu à mesure que le serpent grandit : plus le score monte, plus le pas de simulation se raccourcit. Le défi devient tendu en fin de partie.

**Pistes.**

- La vitesse, c'est l'intervalle du `StepTimer`. Aujourd'hui il est fixé une fois pour toutes à `STEP_SECONDS` dans `spawn_game`. Il faut maintenant le **raccourcir** quand le score augmente.
- La durée idéale d'un pas est une **fonction du score** : c'est une règle de jeu, donc une bonne candidate pour `logic.rs`. Ajoutez à `Game` une méthode `step_duration() -> f32` (ou `Duration`) qui décroît avec le score — par exemple `BASE * 0.95.powi(score)` borné par un minimum (sinon le jeu devient injouable). Testez-la unitairement : elle décroît, elle ne passe jamais sous le plancher.
- Côté `main.rs`, après chaque pas, ajustez le minuteur. Deux approches : recréer le timer (`*timer = StepTimer(Timer::from_seconds(state.game.step_duration(), TimerMode::Repeating))`) quand le score change, ou modifier sa durée via `timer.0.set_duration(...)`. La première est plus simple à raisonner.
- Soignez le ressenti : une décroissance trop brutale frustre. Faites quelques essais de constantes — c'est du *game design*, ça se règle à la main, manette en main.

---

## Défi 2 — Des obstacles

**Objectif.** Parsemer la grille de **murs internes** (cases infranchissables) que le serpent doit éviter, en plus des bords. Heurter un obstacle tue, comme un mur.

**Pistes.**

- Un obstacle est une **règle de jeu** : il vit dans `logic.rs`. Stockez l'ensemble des cases-obstacles dans `Game` — un `HashSet<Coord>` est parfait (et `Coord` dérive déjà `Hash` + `Eq`, justement prévu pour ça : relisez le commentaire dans `logic.rs`).
- Intégrez-les au pas : dans `step`, une case occupée par un obstacle compte comme une collision (`Died`), au même titre qu'un mur ou que le corps. Et un obstacle ne doit jamais être une case « libre » : `free_cells` doit aussi les exclure (sinon la nourriture pourrait apparaître dans un mur). Ajustez les deux, puis ajoutez des tests : on meurt sur un obstacle, la nourriture n'y apparaît jamais.
- Attention au placement initial : ne posez pas d'obstacle sur le serpent de départ (les trois cases du centre) ni juste devant lui, sinon mort immédiate. Un petit motif fixe (quelques croix, un cadre intérieur) est plus lisible qu'un placement aléatoire.
- Côté `main.rs`, dessinez les obstacles **une fois** à l'entrée dans `Playing` (ils ne bougent pas) avec une couleur distincte et une étiquette `Obstacle` + `OnPlayScreen`. Inutile de les redessiner à chaque image comme le serpent.

---

## Défi 3 — Le meilleur score, persisté entre deux lancements

**Objectif.** Mémoriser le **meilleur score** obtenu et l'afficher sur l'écran de game over (« Meilleur : 42 »). Il doit **survivre à la fermeture** du jeu : relancer `cargo run` se souvient du record.

**Pistes.**

- Le record en cours de session est une donnée globale : une **ressource** `#[derive(Resource)] struct HighScore(u32)`, insérée au démarrage. À chaque game over, comparez le score de la partie au record et mettez-le à jour.
- La persistance est de la plomberie d'I/O, donc côté `main.rs` (pas dans la logique pure). Au plus simple : lire/écrire un petit fichier texte avec `std::fs::read_to_string` et `std::fs::write` — vous savez faire depuis le module 2. Lisez le fichier au démarrage (un score de 0 s'il n'existe pas encore), réécrivez-le quand le record tombe.
- Pour un format plus propre (et un clin d'œil au module 3), `serde` + un petit struct sérialisé en JSON ou en TOML. Mais un simple `u32` écrit en toutes lettres dans un fichier suffit amplement ici.
- Soignez les cas limites : fichier absent au premier lancement, contenu corrompu (un `parse::<u32>()` qui échoue → repartez de 0 plutôt que de planter). Affichez le record sur l'écran de game over, à côté du score de la partie.
