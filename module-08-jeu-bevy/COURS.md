# Module 8 — Jeu 2D avec Bevy et l'architecture ECS

> **Prérequis** : modules 1 à 4 (ownership/borrowing, structs, enums, `Option`/`Result`, traits, dérive de traits, génériques, closures, itérateurs, `VecDeque`, tests). Le reste du cours (async, web, WASM) n'est pas nécessaire.
> **Projet associé** : un Snake complet et jouable (voir `PROJET.md`), avec menu, score, game over et restart.

Voici le module final, et le plus ludique : vous allez écrire un vrai jeu, avec une fenêtre, des sprites colorés, du clavier et une boucle de jeu à 60 images par seconde. Le moteur s'appelle **Bevy**, et il repose sur une architecture qui va vous dérouter au début parce qu'elle prend l'inverse de tout ce que la POO Java vous a appris : l'**ECS** (Entity-Component-System). Pas d'héritage, pas de méthodes sur les objets, pas de hiérarchie de classes — juste des données pures d'un côté, des fonctions de l'autre. La bonne nouvelle : cette inversion colle parfaitement à la philosophie de Rust (composition plutôt qu'héritage, données et comportement séparés), et elle exploite à fond le système de types et le borrow checker que vous connaissez maintenant bien.

Le projet est construit autour d'une idée d'architecture que vous garderez bien après ce cours : **la logique de jeu est écrite en Rust pur, sans une ligne de Bevy** (fichier `logic.rs`, testé unitairement comme n'importe quelle bibliothèque), et le moteur n'est qu'un **adaptateur** qui branche cette logique sur un écran et un clavier (fichier `main.rs`). On dessine, mais on ne mélange jamais les règles du jeu avec le code d'affichage.

## Sommaire

1. [Un moteur de jeu, c'est quoi ? La boucle de jeu](#1-un-moteur-de-jeu-cest-quoi--la-boucle-de-jeu)
2. [L'architecture ECS : l'inverse de la POO](#2-larchitecture-ecs--linverse-de-la-poo)
3. [Pourquoi l'ECS épouse Rust : types et borrow checker](#3-pourquoi-lecs-épouse-rust--types-et-borrow-checker)
4. [Bevy : l'`App` et les plugins](#4-bevy--lapp-et-les-plugins)
5. [Entités et composants : `Commands`, spawn et despawn](#5-entités-et-composants--commands-spawn-et-despawn)
6. [Les systèmes et leurs paramètres : `Query`, `Res`, `ResMut`](#6-les-systèmes-et-leurs-paramètres--query-res-resmut)
7. [Filtrer les entités : `With` et `Without`, `Single`](#7-filtrer-les-entités--with-et-without-single)
8. [Le temps et les minuteurs : `Time` et `Timer`](#8-le-temps-et-les-minuteurs--time-et-timer)
9. [Le clavier : `ButtonInput<KeyCode>`](#9-le-clavier--buttoninputkeycode)
10. [Afficher : `Camera2d`, `Sprite`, `Transform`, `Text2d`](#10-afficher--camera2d-sprite-transform-text2d)
11. [Les états de jeu : `States`, menu et game over](#11-les-états-de-jeu--states-menu-et-game-over)
12. [Les messages : communiquer entre systèmes](#12-les-messages--communiquer-entre-systèmes)
13. [L'ordonnancement des systèmes : `Update`, `chain`, run conditions](#13-lordonnancement-des-systèmes--update-chain-run-conditions)
14. [L'architecture du projet : logique pure vs adaptateur](#14-larchitecture-du-projet--logique-pure-vs-adaptateur)
15. [Récapitulatif : ce que le projet utilise](#15-récapitulatif--ce-que-le-projet-utilise)

---

## 1. Un moteur de jeu, c'est quoi ? La boucle de jeu

**Définition.** Un *moteur de jeu* (game engine) est une bibliothèque qui prend en charge tout le plomberie d'un jeu : ouvrir une fenêtre, dessiner à l'écran, lire le clavier et la souris, jouer du son, gérer le temps. Vous, vous écrivez la logique du jeu ; lui s'occupe du reste. Bevy est un moteur écrit **en Rust, pour Rust** — pas un binding vers un moteur C++.

**La boucle de jeu (game loop).** C'est le cœur battant de tout jeu. Une application classique (un serveur web, une CLI) attend un événement, y répond, puis se rendort. Un jeu, lui, ne dort jamais : il tourne en boucle, des dizaines de fois par seconde, et à chaque tour (chaque *image*, ou *frame*) il fait toujours les trois mêmes choses :

```text
boucle infinie {
    1. lire les entrées   (quelles touches sont pressées ?)
    2. mettre à jour       (avancer la simulation : déplacer, tester les collisions…)
    3. dessiner            (afficher l'état courant à l'écran)
}
```

À 60 images par seconde, ce tour complet se répète toutes les ~16 millisecondes. C'est ce qui donne l'illusion du mouvement.

**Comparaison Java/TS.** Si vous avez touché au jeu vidéo amateur, vous avez peut-être écrit cette boucle à la main avec un `while (true)` et un `Thread.sleep(16)` en Java, ou un `requestAnimationFrame` en JavaScript. Bevy écrit la boucle **pour** vous : vous ne voyez jamais le `while`. Vous lui donnez des **systèmes** (des fonctions), il les appelle à chaque tour de boucle, dans l'ordre que vous avez fixé. `App::run()` démarre la boucle et ne rend la main qu'à la fermeture de la fenêtre.

> Dans notre Snake, on ne veut PAS que le serpent avance 60 fois par seconde (injouable). La boucle, elle, tourne bien à 60 images/s — mais on n'avance la simulation qu'une fois toutes les 0,18 s, grâce à un **minuteur** (§8). La boucle dessine vite ; la simulation, elle, avance lentement.

---

## 2. L'architecture ECS : l'inverse de la POO

Accrochez-vous, c'est le concept central du module.

**Le réflexe POO (Java).** Pour modéliser un serpent en Java, vous écririez sans doute une classe :

```java
class Snake extends GameObject {
    List<Cell> body;
    Direction direction;
    void move() { /* … */ }
    void render(Graphics g) { /* … */ }
}
```

Les **données** (`body`, `direction`) et le **comportement** (`move`, `render`) vivent ensemble dans l'objet, et on réutilise par **héritage** (`extends GameObject`). C'est la POO classique.

**L'approche ECS.** L'ECS éclate tout ça en trois notions distinctes :

- **Entity (entité)** — un simple **identifiant**, un numéro. Rien d'autre. Pas de données, pas de méthodes. C'est juste « l'objet n°42 ».
- **Component (composant)** — un bloc de **données pures**, attaché à une entité. `Transform` (position), `Sprite` (apparence), `SnakeSegment` (une étiquette « ceci est un segment de serpent »). Un composant ne contient **aucune méthode de comportement** : juste des champs.
- **System (système)** — une **fonction** qui agit sur toutes les entités possédant certains composants. Tout le comportement est là. `move_snake`, `render` sont des systèmes.

Une entité, dans ce modèle, n'est qu'un **sac de composants**. « Un segment de serpent » n'est pas une classe : c'est l'entité n°42 qui se trouve avoir, attachés à elle, un `Transform`, un `Sprite` et l'étiquette `SnakeSegment`.

**Le retournement de cerveau.** En POO, vous demandez à un objet « fais ceci » (`snake.move()`). En ECS, un système dit « pour TOUTES les entités qui ont un `Transform` et un `Velocity`, additionne l'un à l'autre ». Le comportement ne vit plus *dans* les objets ; il vit *au-dessus* d'eux, dans des fonctions qui balayent les entités.

**Composition plutôt qu'héritage.** En POO, pour donner une nouvelle capacité à un objet, on hérite (`extends`). En ECS, on **ajoute un composant**. Vous voulez qu'une entité devienne « ramassable » ? Attachez-lui un composant `Pickable`. Plus de hiérarchie de classes, plus de problème du « diamant » de l'héritage multiple, plus de classes-Dieu de 2000 lignes. C'est exactement la philosophie que Rust pousse depuis le module 3 : Rust n'a **pas** d'héritage de structs, justement parce qu'il parie sur la composition (des structs qui contiennent d'autres structs, des traits qu'on implémente). L'ECS est cette philosophie poussée à son terme.

**Dans notre code.** Voici comment on déclare un composant dans `main.rs` — remarquez qu'il ne contient *rien* du tout :

```rust
/// Marque une entité « sprite d'un segment du serpent ».
#[derive(Component)]
struct SnakeSegment;
```

`SnakeSegment` est une étiquette pure (un *marker component*). Il ne porte aucune donnée : sa seule présence sur une entité dit « ceci est un segment, dessine-le comme tel et n'oublie pas de l'effacer à la prochaine image ». La couleur, la position ? Elles vivent dans d'**autres** composants (`Sprite`, `Transform`) attachés à la même entité. Une entité-segment, chez nous, c'est donc trois composants empilés : `Sprite + Transform + SnakeSegment`.

---

## 3. Pourquoi l'ECS épouse Rust : types et borrow checker

L'ECS n'est pas qu'une mode. En Rust, il résout élégamment un problème que vous connaissez bien depuis le module 4 : l'accès concurrent aux données.

**Les systèmes déclarent ce qu'ils touchent.** Un système Rust annonce, **dans sa signature**, exactement quelles données il lit et lesquelles il modifie :

```rust
fn handle_input(keyboard: Res<ButtonInput<KeyCode>>, mut state: ResMut<GameState>) {
    // lit le clavier (Res = lecture seule), modifie l'état (ResMut = lecture/écriture)
}
```

`Res<T>` = « je lis `T` » (emprunt partagé, `&`). `ResMut<T>` = « je lis et j'écris `T` » (emprunt exclusif, `&mut`). C'est **exactement** la distinction `&` / `&mut` du borrow checker, mais hissée au niveau des systèmes.

**D'où le parallélisme automatique.** Comme chaque système déclare ses accès, Bevy peut, avant de lancer la boucle, calculer quels systèmes touchent des données **disjointes**. Deux systèmes qui ne partagent rien — ou qui ne font que *lire* les mêmes données — peuvent tourner **en parallèle, sur des threads différents**, sans une ligne de code de votre part. Deux systèmes qui veulent tous deux *écrire* le même composant, eux, seront forcément séquentiels (un seul `&mut` à la fois, la règle d'or du module 1). Le borrow checker, qui empêchait les data races *à l'intérieur* d'une fonction au module 4, fait ici le même travail *entre* les systèmes. C'est la *fearless concurrency* appliquée à un jeu : impossible d'introduire une data race par accident, le moteur ne vous laisserait pas faire.

> Notre Snake est petit : le parallélisme automatique ne lui apporte pas grand-chose de visible. Mais le principe est là, gratuit, et c'est *la* raison pour laquelle l'ECS et Rust forment un si bon couple. Vous écrivez un code à un seul fil de pensée ; le moteur le parallélise pour vous, en toute sécurité.

---

## 4. Bevy : l'`App` et les plugins

**Définition.** L'`App` est l'objet racine d'un programme Bevy : le conteneur dans lequel on enregistre tout (plugins, états, ressources, systèmes) avant de lancer la boucle. On la construit par enchaînement de méthodes (*builder pattern*, déjà vu avec les itérateurs), et on termine par `.run()`.

**Les plugins.** Un *plugin* est un paquet de fonctionnalités prêt à brancher : un ensemble de systèmes et de ressources regroupés sous un nom. `DefaultPlugins` est le méga-plugin qui apporte tout ce qu'il faut pour un vrai jeu : fenêtre, rendu, son, gestion des entrées, horloge. C'est l'équivalent d'un *starter* Spring Boot : une ligne, et toute l'infrastructure est là.

**Comparaison Java/TS.** Pensez à `DefaultPlugins` comme à `@SpringBootApplication` ou à un `create-react-app` : un point d'entrée qui câble des dizaines de briques par défaut, que vous pouvez ensuite configurer finement.

**Dans notre code.** Voici le squelette de notre `main`, où l'on configure au passage le titre et la taille de la fenêtre :

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snake — Rust & Bevy".to_string(),
                resolution: ( /* largeur */, /* hauteur */ ).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(COLOR_BACKGROUND))  // couleur de fond
        .init_state::<AppState>()                       // l'état initial (§11)
        .add_message::<AteFood>()                       // un type de message (§12)
        .add_systems(Startup, setup_camera)             // tourne une fois au démarrage
        // … d'autres systèmes …
        .run();                                         // démarre la boucle
}
```

`.set(WindowPlugin { … })` reconfigure un sous-plugin de `DefaultPlugins`. Le `..default()` (la *struct update syntax* du module 2) garde toutes les autres options par défaut et n'écrase que ce qu'on précise. `default()` est le raccourci de Bevy pour `Default::default()`.

`Startup` est un *planning* (schedule) spécial : ses systèmes tournent **une seule fois**, au lancement, avant la boucle. C'est là qu'on place la caméra.

---

## 5. Entités et composants : `Commands`, spawn et despawn

**Définition.** On ne crée pas une entité avec un `new` : on la **demande** via `Commands`, un « carnet de commandes » qu'un système reçoit en paramètre. On y note « crée cette entité », « supprime celle-là », et Bevy applique tout en bloc à la fin du système. C'est différé exprès : pendant qu'un système balaye les entités, le modifier en pleine course serait dangereux ; les commandes sont donc mises en file et exécutées au moment sûr.

**Spawn (faire naître).** `commands.spawn(bundle)` crée une entité avec un *bundle* — un ou plusieurs composants groupés dans un tuple :

```rust
commands.spawn((
    Sprite::from_color(color, cell_sprite_size()),       // apparence
    Transform::from_translation(cell_to_world(cell)),    // position
    SnakeSegment,                                        // étiquette
    OnPlayScreen,                                        // étiquette de nettoyage
));
```

Une seule entité naît ici, avec **quatre** composants attachés. Le tuple `(A, B, C, D)` est le bundle. Comparé à Java : au lieu d'un constructeur `new Snake(...)`, vous assemblez l'objet par accumulation de composants.

**Despawn (faire disparaître).** `commands.entity(e).despawn()` supprime l'entité `e` (et, depuis Bevy 0.16, ses entités-enfants avec elle). Notre stratégie d'affichage du Snake est volontairement simple : à chaque image, on **détruit** tous les sprites de la frame précédente et on en **recrée** de neufs à partir de l'état du jeu.

```rust
for entity in &segments {           // `segments` est une Query (§6)
    commands.entity(entity).despawn();
}
```

> « Détruire et tout recréer à chaque image » serait absurde dans un AAA avec des milliers d'objets ; pour un Snake d'une trentaine de cases, c'est trivial et parfaitement fluide. Choisir la simplicité quand elle suffit est une vraie compétence.

---

## 6. Les systèmes et leurs paramètres : `Query`, `Res`, `ResMut`

**Définition.** Un *système* est une fonction ordinaire que vous enregistrez auprès de l'`App`. Sa magie tient dans ses **paramètres** : Bevy regarde leur type et **injecte** automatiquement la bonne donnée. Vous ne fournissez jamais les arguments vous-même ; le moteur le fait. C'est de l'injection de dépendances, comme `@Autowired` en Spring — mais vérifiée à la compilation par le système de types.

Les trois familles de paramètres que notre projet utilise :

### `Res<T>` et `ResMut<T>` — les ressources

**Une ressource est une donnée globale unique** : il n'en existe qu'un seul exemplaire dans tout le jeu. Le score, le minuteur, l'état du clavier sont des ressources. On en déclare une avec `#[derive(Resource)]` :

```rust
#[derive(Resource)]
struct GameState {
    game: Game,   // le `Game` de notre logique pure, enveloppé pour Bevy
}
```

- `Res<GameState>` : un système qui veut **lire** la partie.
- `ResMut<GameState>` : un système qui veut la **modifier**.

C'est le pendant « donnée globale » des composants (qui, eux, sont *par entité*). Comparaison TS : une ressource, c'est un singleton injecté ; mais ici l'accès lecture/écriture est tracé par le type, donc parallélisable sans risque.

### `Query<…>` — interroger les entités

**Définition.** Une *query* est une requête sur le monde des entités : « donne-moi tous les composants de tel type, pour toutes les entités qui les possèdent ». C'est l'outil n°1 d'un système.

```rust
fn render(
    mut commands: Commands,
    state: Res<GameState>,
    segments: Query<Entity, With<SnakeSegment>>,   // toutes les entités-segments
    food: Query<Entity, With<FoodSprite>>,         // l'entité-nourriture
) {
    for entity in &segments {                       // on itère dessus
        commands.entity(entity).despawn();
    }
    // …
}
```

`Query<Entity, …>` demande juste l'identifiant des entités (pour les détruire). On peut aussi demander leurs composants : `Query<&Transform>` (lecture), `Query<&mut Transform>` (écriture), ou un tuple `Query<(&Sprite, &mut Transform)>`. La query est l'équivalent typé et parallélisable d'un `SELECT … WHERE` sur la base d'entités. Le premier paramètre générique dit **ce qu'on récupère**, le second (optionnel) dit **comment filtrer** (§7).

---

## 7. Filtrer les entités : `With` et `Without`, `Single`

**`With<C>` et `Without<C>`.** Ce sont des **filtres** : ils restreignent la query aux entités qui possèdent (ou non) un composant, **sans récupérer ce composant**. La nuance est importante.

```rust
Query<Entity, With<SnakeSegment>>      // entités AYANT l'étiquette SnakeSegment
Query<&Transform, Without<Camera2d>>   // les Transform, sauf celui de la caméra
```

`With<SnakeSegment>` ne renvoie pas le `SnakeSegment` (il est vide, sans intérêt) ; il sert juste à dire « uniquement les segments ». C'est exactement pourquoi les *marker components* (composants-étiquettes vides) existent : ils ne portent pas de donnée, mais ils permettent de **cibler** un sous-ensemble d'entités. `Without<C>` fait l'inverse et sert souvent à éviter qu'une query n'attrape des entités qu'elle ne devrait pas — par exemple distinguer « le texte du score » du « texte du menu » alors que tous deux ont un `Text2d`.

**`Single<…>`.** Quand on *sait* qu'exactement une entité correspond, `Single` évite la boucle. On l'utilise pour le texte du score, dont il n'existe qu'un :

```rust
fn update_score_text(
    state: Res<GameState>,
    mut text: Single<&mut Text2d, With<ScoreText>>,
) {
    **text = Text2d::new(format!("Score : {}", state.game.score()));
}
```

`Single<&mut Text2d, With<ScoreText>>` cible l'unique entité ayant à la fois un `Text2d` et l'étiquette `ScoreText`, et donne directement un accès mutable au texte. Le `**text` déréférence deux fois (Bevy enveloppe l'accès mutable, on traverse deux couches) pour remplacer le texte. Si zéro ou plusieurs entités correspondaient, le système serait simplement sauté — pratique, mais à n'utiliser que quand l'unicité est garantie.

---

## 8. Le temps et les minuteurs : `Time` et `Timer`

**`Time`.** C'est la ressource d'horloge du moteur. Son information essentielle : `time.delta()`, la durée écoulée **depuis l'image précédente**. Pourquoi raisonner en delta plutôt qu'en « nombre d'images » ? Parce que la fréquence d'affichage varie d'une machine à l'autre (60 Hz, 144 Hz…). En se basant sur le temps réel écoulé, le jeu tourne à la même vitesse partout. C'est une règle d'or du jeu vidéo.

**`Timer`.** Un minuteur qu'on « nourrit » avec le delta de chaque image et qui nous dit quand il a sonné. On le crée en mode répétitif pour notre tick régulier :

```rust
#[derive(Resource)]
struct StepTimer(Timer);

// à la création de la partie :
StepTimer(Timer::from_seconds(STEP_SECONDS, TimerMode::Repeating))
```

`TimerMode::Repeating` le réarme tout seul après chaque déclenchement. Et voici comment on l'utilise pour ne faire avancer le serpent qu'au bon rythme :

```rust
fn step_game(time: Res<Time>, mut timer: ResMut<StepTimer>, /* … */) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;   // le minuteur n'a pas (encore) sonné : on ne fait rien cette image
    }
    // ici, et seulement ici, on avance la simulation d'un pas
}
```

`timer.0.tick(delta)` avance le minuteur ; `just_finished()` est vrai uniquement à l'image où il vient de sonner. Résultat : la fonction est appelée 60 fois par seconde, mais ne fait avancer le serpent qu'une fois toutes les `STEP_SECONDS`. **C'est la pièce maîtresse du rythme du jeu.**

**Comparaison Java/TS.** En JS vous auriez tenté un `setInterval(step, 180)`. Le problème : `setInterval` vit en dehors de la boucle de rendu, dérive, et se désynchronise. Ici le minuteur est *dans* la boucle, piloté par le temps réel — robuste et précis.

---

## 9. Le clavier : `ButtonInput<KeyCode>`

**Définition.** L'état du clavier est une ressource : `ButtonInput<KeyCode>`. Un système qui veut lire les touches la prend en `Res` (lecture seule). Trois questions principales :

- `pressed(key)` — la touche est-elle **maintenue** en ce moment ? (vrai à chaque image tant qu'on appuie)
- `just_pressed(key)` — la touche **vient-elle** d'être enfoncée ? (vrai une seule image)
- `just_released(key)` — vient-elle d'être relâchée ?

La distinction est cruciale. Pour valider un menu, on veut `just_pressed` : sinon, maintenir Espace une demi-seconde enchaînerait 30 validations. Pour un virage du serpent aussi : on enregistre l'**intention** une fois.

**Dans notre code.** On accepte les flèches *et* WASD/ZQSD, grâce à `any_just_pressed` qui teste plusieurs touches d'un coup :

```rust
fn handle_input(keyboard: Res<ButtonInput<KeyCode>>, mut state: ResMut<GameState>) {
    let direction = if keyboard.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        Some(Direction::Up)
    } else if keyboard.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        Some(Direction::Down)
    }
    // … gauche, droite …
    else { None };

    if let Some(direction) = direction {
        state.game.try_turn(direction);   // on transmet à la LOGIQUE pure
    }
}
```

Remarquez la frontière nette : ce système **lit le clavier** (rôle de l'adaptateur) puis **délègue la décision** à `try_turn` (rôle de la logique). C'est la logique, pas ce système, qui refusera un demi-tour. L'adaptateur ne connaît aucune règle du jeu.

---

## 10. Afficher : `Camera2d`, `Sprite`, `Transform`, `Text2d`

**`Camera2d`.** Sans caméra, rien ne s'affiche : la caméra définit *par où l'on regarde* la scène. Pour de la 2D, une `Camera2d` suffit, placée une fois au démarrage :

```rust
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
```

En Bevy 0.18, `Camera2d` est un *composant requis* : l'attacher insère automatiquement tous les autres composants dont une caméra a besoin. Vous ne spawnez qu'un composant, le moteur complète.

**`Transform`.** Le composant de **position, rotation et échelle** d'une entité dans le monde. Pour nous, surtout la position (en pixels, l'origine `(0,0)` étant le centre de l'écran) :

```rust
Transform::from_translation(cell_to_world(cell))   // place l'entité à une position 3D
Transform::from_xyz(0.0, y, 1.0)                   // raccourci x, y, z
```

Le `z` sert à l'empilement (qui est devant qui) : le texte du score a un `z` supérieur aux sprites pour passer au-dessus. Nous traduisons les coordonnées de notre grille logique (cases 0..17) en pixels avec une petite fonction `cell_to_world` — c'est tout le travail de l'adaptateur : convertir le langage du jeu (cases) en langage du moteur (pixels).

**`Sprite`.** Le composant d'**apparence 2D**. On peut afficher une image, mais pour un Snake un simple **carré coloré** suffit, sans aucun fichier d'asset :

```rust
Sprite::from_color(COLOR_SNAKE_HEAD, cell_sprite_size())
```

`from_color(couleur, taille)` crée un rectangle plein. Les couleurs viennent de `Color::srgb(r, g, b)` (composantes de 0,0 à 1,0). Le `Sprite` dit *à quoi ça ressemble*, le `Transform` dit *où c'est* : deux composants complémentaires sur la même entité.

**`Text2d`.** Pour afficher du texte dans le monde 2D (score, menu, game over). Bevy embarque une police par défaut, donc nul besoin de charger un fichier :

```rust
commands.spawn((
    Text2d::new("SNAKE\n\nEspace pour jouer"),
    TextFont { font_size: 36.0, ..default() },
    TextColor(COLOR_TEXT),
    OnMenuScreen,
));
```

Le texte est, lui aussi, une entité faite de composants : le contenu (`Text2d`), la fonte (`TextFont`), la couleur (`TextColor`). Toujours la même mécanique : empiler des composants de données.

---

## 11. Les états de jeu : `States`, menu et game over

**Définition.** Un *state* (état) est une donnée — typiquement une énumération — qui décrit dans quel **écran** se trouve le jeu. Le moteur la surveille et déclenche des systèmes selon l'écran courant et les transitions. C'est ce qui structure un jeu en « menu → partie → écran de fin → menu ».

On déclare l'énumération avec `#[derive(States)]` et on désigne la valeur de départ avec `#[default]` :

```rust
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum AppState {
    #[default]
    Menu,       // écran d'accueil
    Playing,    // une partie est en cours
    GameOver,   // partie perdue
}
```

On l'enregistre avec `.init_state::<AppState>()`. Ensuite, trois manières d'y accrocher des systèmes :

- **`OnEnter(state)`** — tourne une fois, à l'**entrée** dans l'état. On y crée la scène : `OnEnter(AppState::Playing)` lance une partie neuve (spawn du jeu, du minuteur, du score).
- **`OnExit(state)`** — tourne une fois, à la **sortie** de l'état. On y nettoie : `OnExit(AppState::Playing)` détruit tous les sprites de la partie.
- **`.run_if(in_state(state))`** — une *run condition* : le système ne tourne **que si** on est dans cet état. Le système d'entrée clavier du jeu ne doit pas tourner dans le menu : on le conditionne à `in_state(AppState::Playing)`.

**Changer d'état.** Un système ne modifie pas l'état directement ; il **demande** la transition via la ressource `NextState`, et Bevy l'applique entre deux images :

```rust
fn start_on_space(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(AppState::Playing);   // au prochain tour, on bascule en jeu
    }
}
```

**Le cycle complet de notre Snake :**

```text
        Espace                 collision (Died)            Espace
Menu ───────────────► Playing ───────────────────► GameOver ────────► Playing …
       OnEnter:                 OnExit(Playing):            OnEnter:
       spawn_menu_text          despawn_play_screen         spawn_game (partie neuve)
```

Le grand bénéfice : chaque écran a ses systèmes de création (`OnEnter`) et de nettoyage (`OnExit`), si bien qu'on ne mélange jamais le menu et la partie. C'est une machine à états propre, le tout en données et fonctions, sans une seule classe.

---

## 12. Les messages : communiquer entre systèmes

**Le problème.** Deux systèmes doivent coopérer sans s'appeler l'un l'autre : quand le serpent mange, *un* système le constate (la simulation), mais c'est *un autre* qui doit replacer la nourriture (celui qui a le droit de tirer au hasard). Comment passer l'information ?

**La solution : les messages.** Un *message* (en Bevy 0.18, c'est le nom des événements *bufferisés*) est une donnée qu'un système **écrit** dans une boîte aux lettres et qu'un autre **lit** ensuite. Émetteur et récepteur ne se connaissent pas : couplage minimal, exactement comme les channels du module 4, mais à l'échelle du jeu.

> Note de version : en Bevy 0.18, ces événements bufferisés s'appellent des **messages** (`#[derive(Message)]`, `MessageWriter`, `MessageReader`, `add_message`). Les versions antérieures les appelaient « events » (`EventWriter`, etc.) ; le concept est identique, seul le nom a changé. Le mot « événement » est désormais réservé à un autre mécanisme (les *observers*), que notre projet n'utilise pas.

On déclare le type, on l'enregistre, puis on l'écrit et le lit :

```rust
#[derive(Message)]
struct AteFood;                         // un message vide : sa simple existence suffit

// dans main : .add_message::<AteFood>()

// l'ÉMETTEUR (le système de simulation) :
fn step_game(/* … */, mut ate_writer: MessageWriter<AteFood>, /* … */) {
    match state.game.step() {
        StepOutcome::Ate => { ate_writer.write(AteFood); }   // « quelqu'un a mangé ! »
        // …
    }
}

// le RÉCEPTEUR (le système qui replace la nourriture) :
fn place_new_food(mut ate_reader: MessageReader<AteFood>, mut state: ResMut<GameState>) {
    for _ in ate_reader.read() {                              // pour chaque message reçu
        let mut rng = rand::rng();
        if let Some(&cell) = state.game.free_cells().choose(&mut rng) {
            state.game.place_food(cell);
        }
    }
}
```

`MessageWriter::write` poste le message ; `MessageReader::read` itère sur les messages de l'image. La séparation est nette : `step_game` ne sait pas *comment* la nourriture est replacée, et `place_new_food` ne sait pas *pourquoi* elle l'est. Chacun fait une chose.

> On *pourrait* replacer la nourriture directement dans `step_game`. Le message ne sert pas la fonctionnalité ici — il sert l'**architecture** : il découple « ce qui s'est passé » de « ce qu'on en fait », ce qui paie dès que plusieurs systèmes veulent réagir au même fait (un son, un effet visuel, un compteur…).

---

## 13. L'ordonnancement des systèmes : `Update`, `chain`, run conditions

**Les plannings (schedules).** Un système est enregistré dans un *schedule* qui dit *quand* il tourne :

- **`Startup`** — une seule fois, au lancement (la caméra).
- **`Update`** — à **chaque image** de la boucle. La grande majorité des systèmes vivent ici.
- **`FixedUpdate`** — à intervalle de temps **fixe** (par défaut 64 fois/s), indépendamment de la fréquence d'affichage. Utile pour une physique stable. *Notre* projet cadence sa simulation autrement (un `Timer` lent dans `Update`, §8), parce qu'on veut un pas toutes les 0,18 s, bien plus lent que `FixedUpdate` — mais sachez que `FixedUpdate` existe et serait le choix idiomatique pour de la physique.

**L'ordre des systèmes.** Par défaut, dans un même schedule, Bevy choisit l'ordre librement (et parallélise quand il peut, §3). Mais parfois l'ordre **compte** : chez nous, il faut lire le clavier *avant* d'avancer la simulation, et dessiner *après*. On force l'ordre avec `.chain()` :

```rust
.add_systems(
    Update,
    (handle_input, step_game, place_new_food, render, update_score_text)
        .chain()                              // exécutés DANS CET ORDRE
        .run_if(in_state(AppState::Playing)), // et seulement pendant une partie
)
```

`.chain()` enchaîne le tuple de systèmes strictement dans l'ordre écrit : on lit l'intention (`handle_input`), on avance (`step_game`), on réagit (`place_new_food`), on dessine (`render`), on met le score à jour (`update_score_text`). Sans `chain()`, Bevy pourrait dessiner avant d'avoir avancé — un bug d'une image de retard, classique et fuyant.

**Les run conditions.** `.run_if(condition)` saute le système si la condition est fausse. `in_state(AppState::Playing)` est la plus utile : elle réserve ces systèmes à la partie en cours. Le menu et le game over ont leurs propres systèmes, conditionnés à *leurs* états. Une run condition est juste une fonction qui renvoie un `bool` ; le moteur l'évalue à chaque image.

**Le tableau complet de notre `main` :**

```text
Startup         : setup_camera
OnEnter(Menu)   : spawn_menu_text          OnExit(Menu)    : despawn_menu
OnEnter(Playing): spawn_game               OnExit(Playing) : despawn_play_screen
OnEnter(GameOver): spawn_game_over_text    OnExit(GameOver): despawn_menu
Update si Menu     : start_on_space
Update si Playing  : (handle_input → step_game → place_new_food → render → update_score_text).chain()
Update si GameOver : restart_on_space
```

Lisez ce tableau comme la table des matières du jeu : il dit *qui* tourne, *quand*, et *dans quel ordre*. C'est tout le pilotage du Snake.

---

## 14. L'architecture du projet : logique pure vs adaptateur

C'est la leçon d'architecture du module, celle qui dépasse Bevy.

**Deux fichiers, deux mondes :**

| | `src/logic.rs` (la logique pure) | `src/main.rs` (l'adaptateur Bevy) |
|---|---|---|
| Contient | les **règles** du jeu | le **branchement** sur le moteur |
| Dépend de | rien (juste la `std`) | Bevy, `rand`, et `logic.rs` |
| Aléatoire | **jamais** (déterministe) | oui (tirage de la nourriture) |
| Testé par | `tests/logic_test.rs` (unitaire) | la main, en jouant (`cargo run`) |
| Exemple | `Game::step` décide « Moved / Ate / Died » | `step_game` appelle `step` et réagit |

**Pourquoi séparer ?** Trois raisons concrètes :

1. **Testabilité.** On ne peut pas ouvrir de fenêtre dans un test automatique. Mais on peut tester `Game::step` mille fois en une milliseconde, parce qu'il ne dépend de rien. Tout le fichier `tests/logic_test.rs` teste les règles (déplacement, demi-tour interdit, collisions, croissance, score) **sans une ligne de Bevy**.
2. **Déterminisme.** La logique ne tire jamais l'aléatoire elle-même : `Game::new(food)` *reçoit* la position de la nourriture. C'est l'adaptateur qui tire au hasard et la lui passe. Conséquence : les tests sont reproductibles à 100 %, et la logique est analysable au raisonnement.
3. **Portabilité.** Cette logique pourrait être branchée demain sur un terminal, un serveur, un autre moteur — sans la réécrire. Le moteur est un *détail* remplaçable ; les règles du jeu sont le *cœur* stable.

**Le contrat entre les deux.** L'adaptateur ne fait que traduire dans les deux sens :

```text
   clavier  ──►  Direction  ──►  game.try_turn(dir)        (entrée → logique)
   tick     ──►  game.step() ──► StepOutcome               (temps → logique → constat)
   game.body() / game.food() ──► sprites colorés           (état → sortie)
```

`StepOutcome` (`Moved` / `Ate` / `Died`) est la clé du contrat : la logique **constate** un fait, l'adaptateur **réagit** (replacer la nourriture, basculer en game over). La logique ne sait rien des sprites ; l'adaptateur ne sait rien des règles. Cette discipline — souvent appelée *ports et adaptateurs* ou *architecture hexagonale* — est ce que vous emporterez de plus précieux de ce module, jeu ou pas.

---

## 15. Récapitulatif : ce que le projet utilise

À la fin du projet, vous aurez manipulé tout l'arsenal d'un jeu Bevy 2D :

- **Boucle de jeu** pilotée par `App::run()`, sans `while` à écrire (§1).
- **ECS** : entités (identifiants), **composants** de données pures (`#[derive(Component)]`, dont des étiquettes vides), **systèmes** (fonctions à paramètres injectés) — l'inverse de la POO, en phase avec Rust (§2, §3).
- **`App` et plugins** : `DefaultPlugins`, configuration de la fenêtre, `Startup` (§4).
- **`Commands`** : `spawn` (bundles de composants) et `despawn` (§5).
- **Paramètres de systèmes** : `Res`/`ResMut` pour les **ressources** (`#[derive(Resource)]`), `Query` pour interroger les entités (§6).
- **Filtres** : `With`, `Without`, et `Single` pour l'entité unique (§7).
- **Temps** : `Time` (delta) et `Timer` répétitif pour cadencer la simulation (§8).
- **Clavier** : `ButtonInput<KeyCode>`, `just_pressed` vs `pressed`, `any_just_pressed` (§9).
- **Affichage** : `Camera2d`, `Sprite::from_color`, `Transform`, `Text2d`/`TextFont`/`TextColor`, `Color::srgb` (§10).
- **États** : `#[derive(States)]`, `init_state`, `OnEnter`/`OnExit`, `in_state`, `NextState` (§11).
- **Messages** : `#[derive(Message)]`, `MessageWriter`/`MessageReader`, `add_message` (§12).
- **Ordonnancement** : `Update`, `FixedUpdate`, `.chain()`, `.run_if(...)` (§13).
- **Architecture** : logique pure testable vs adaptateur moteur — la vraie leçon (§14).

Ouvrez maintenant `PROJET.md` : vous allez tout assembler, étape par étape, jusqu'à un Snake jouable.
