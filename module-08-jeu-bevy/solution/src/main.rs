//! Adaptateur Bevy du Snake — le « branchement » de la logique pure sur le moteur.
//!
//! Tout ce qui touche au moteur (fenêtre, sprites, clavier, horloge, états de
//! jeu) vit ICI. La logique du jeu, elle, est dans `logic.rs` et ne connaît pas
//! Bevy. Ce fichier ne fait que deux choses :
//!   - traduire les ENTRÉES (touches, tick d'horloge) en appels de méthodes de
//!     `Game` (`try_turn`, `step`) ;
//!   - traduire l'ÉTAT de `Game` en SORTIES (sprites colorés à l'écran).
//!
//! C'est l'architecture « ports et adaptateurs » : le cœur du jeu est testé
//! sans ouvrir de fenêtre (voir `tests/logic_test.rs`), l'adaptateur ne contient
//! pas de règle de jeu qu'on aurait envie de tester.
//!
//! Lance le jeu avec :  cargo run
//! (compile Bevy au premier lancement — c'est long, c'est normal.)

use bevy::prelude::*;
use rand::seq::IndexedRandom;

use snake::logic::{Coord, Direction, Game, StepOutcome, GRID_HEIGHT, GRID_WIDTH};

// --------------------------------------------------------------------------
// Constantes d'affichage
// --------------------------------------------------------------------------

/// Côté d'une case, en pixels logiques.
const CELL_SIZE: f32 = 28.0;
/// Petit espace entre deux cases, pour qu'on distingue les segments.
const CELL_GAP: f32 = 2.0;
/// Intervalle entre deux pas de simulation, en secondes (la « vitesse » du jeu).
const STEP_SECONDS: f32 = 0.18;

const COLOR_BACKGROUND: Color = Color::srgb(0.10, 0.10, 0.14);
const COLOR_SNAKE_HEAD: Color = Color::srgb(0.30, 0.85, 0.40);
const COLOR_SNAKE_BODY: Color = Color::srgb(0.20, 0.65, 0.30);
const COLOR_FOOD: Color = Color::srgb(0.90, 0.30, 0.35);
const COLOR_TEXT: Color = Color::srgb(0.92, 0.92, 0.95);

// --------------------------------------------------------------------------
// États de l'application
// --------------------------------------------------------------------------

/// Les trois écrans du jeu. Un `State` n'est qu'une donnée (ici une énumération)
/// que Bevy surveille ; on accroche des systèmes à l'entrée (`OnEnter`), à la
/// sortie (`OnExit`) ou à la présence (`in_state`) d'un état.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum AppState {
    /// Écran d'accueil : « Appuie sur Espace ».
    #[default]
    Menu,
    /// Une partie est en cours.
    Playing,
    /// La partie est perdue : on affiche le score et « Espace pour rejouer ».
    GameOver,
}

// --------------------------------------------------------------------------
// Ressources (données globales, une seule instance)
// --------------------------------------------------------------------------

/// La partie en cours. On enveloppe le `Game` de la logique pure dans une
/// ressource Bevy : c'est le pont entre les deux mondes. L'adaptateur lit et
/// fait avancer cet état, mais toutes les RÈGLES restent dans `logic.rs`.
#[derive(Resource)]
struct GameState {
    game: Game,
}

/// Le minuteur qui cadence la simulation. Sans lui, le serpent avancerait à la
/// fréquence d'affichage (souvent 60 images/s) : injouable. On veut un pas
/// toutes les `STEP_SECONDS` secondes, quel que soit le matériel.
#[derive(Resource)]
struct StepTimer(Timer);

// --------------------------------------------------------------------------
// Composants (étiquettes attachées aux entités)
// --------------------------------------------------------------------------

/// Marque une entité « sprite d'un segment du serpent ». Sert à retrouver et à
/// supprimer tous les segments à chaque image, avant de les redessiner.
#[derive(Component)]
struct SnakeSegment;

/// Marque le sprite de la nourriture.
#[derive(Component)]
struct FoodSprite;

/// Marque le texte du score affiché pendant la partie.
#[derive(Component)]
struct ScoreText;

/// Marque tout ce qui n'existe que sur l'écran de jeu (sprites, texte de score).
/// Permet de tout nettoyer d'un coup en quittant l'état `Playing`.
#[derive(Component)]
struct OnPlayScreen;

/// Marque le texte des écrans Menu / Game Over, pour le nettoyer pareil.
#[derive(Component)]
struct OnMenuScreen;

// --------------------------------------------------------------------------
// Messages (événements bufferisés : écrits dans un système, lus dans un autre)
// --------------------------------------------------------------------------

/// Émis quand le serpent mange. Un système le PRODUIT (la simulation), un autre
/// le CONSOMME (replacer la nourriture). Découpler ainsi « ce qui s'est passé »
/// de « ce qu'on en fait » est l'esprit même de l'ECS.
#[derive(Message)]
struct AteFood;

// --------------------------------------------------------------------------
// Point d'entrée
// --------------------------------------------------------------------------

fn main() {
    App::new()
        // DefaultPlugins apporte tout le nécessaire : fenêtre, rendu, audio,
        // gestion des entrées, horloge… C'est l'équivalent d'un « starter ».
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snake — Rust & Bevy".to_string(),
                resolution: (
                    (GRID_WIDTH as f32 * CELL_SIZE) as u32 + 40,
                    (GRID_HEIGHT as f32 * CELL_SIZE) as u32 + 80,
                )
                    .into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(COLOR_BACKGROUND))
        // On déclare l'état initial (Menu) et le type de message.
        .init_state::<AppState>()
        .add_message::<AteFood>()
        // La caméra existe pour toute la durée de vie de l'app.
        .add_systems(Startup, setup_camera)
        // --- Écran Menu ---
        .add_systems(OnEnter(AppState::Menu), spawn_menu_text)
        .add_systems(OnExit(AppState::Menu), despawn_menu)
        .add_systems(Update, start_on_space.run_if(in_state(AppState::Menu)))
        // --- Écran de jeu ---
        .add_systems(OnEnter(AppState::Playing), spawn_game)
        .add_systems(OnExit(AppState::Playing), despawn_play_screen)
        // L'ordre compte : on lit le clavier, on avance la simulation, puis on
        // réagit aux conséquences (nourriture), puis on redessine. `chain()`
        // garantit cet enchaînement à l'intérieur d'une même image.
        .add_systems(
            Update,
            (handle_input, step_game, place_new_food, render, update_score_text)
                .chain()
                .run_if(in_state(AppState::Playing)),
        )
        // --- Écran Game Over ---
        .add_systems(OnEnter(AppState::GameOver), spawn_game_over_text)
        .add_systems(OnExit(AppState::GameOver), despawn_menu)
        .add_systems(
            Update,
            restart_on_space.run_if(in_state(AppState::GameOver)),
        )
        .run();
}

// --------------------------------------------------------------------------
// Systèmes communs
// --------------------------------------------------------------------------

/// Place une caméra 2D. Sans elle, rien ne s'affiche. `Camera2d` insère
/// automatiquement les composants dont une caméra a besoin (c'est le système
/// de « composants requis » de Bevy 0.18).
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// --------------------------------------------------------------------------
// Écran Menu
// --------------------------------------------------------------------------

fn spawn_menu_text(mut commands: Commands) {
    commands.spawn((
        Text2d::new("SNAKE\n\nEspace pour jouer"),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(COLOR_TEXT),
        OnMenuScreen,
    ));
}

/// `ButtonInput<KeyCode>` est la ressource d'état du clavier. `just_pressed`
/// est vrai uniquement à l'image où la touche passe d'inactive à active —
/// exactement ce qu'on veut pour « valider », pour ne pas déclencher 60 fois.
fn start_on_space(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(AppState::Playing);
    }
}

fn despawn_menu(mut commands: Commands, query: Query<Entity, With<OnMenuScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// --------------------------------------------------------------------------
// Écran de jeu
// --------------------------------------------------------------------------

/// Démarre une partie neuve : on crée le `Game` (logique pure), on installe les
/// ressources, on affiche le score. Les sprites du serpent et de la nourriture
/// seront créés par `render` dès la première image.
fn spawn_game(mut commands: Commands) {
    // La nourriture initiale : une case libre tirée au hasard. L'aléatoire est
    // dans l'adaptateur, JAMAIS dans la logique pure (qui reste déterministe).
    let mut rng = rand::rng();
    let game = Game::new(Coord::new(0, 0));
    let first_food = *game
        .free_cells()
        .choose(&mut rng)
        .expect("la grille n'est jamais pleine au départ");
    let mut game = game;
    game.place_food(first_food);

    commands.insert_resource(GameState { game });
    // `Timer::from_seconds(.., Repeating)` se réarme tout seul après chaque
    // déclenchement : parfait pour un tick régulier.
    commands.insert_resource(StepTimer(Timer::from_seconds(
        STEP_SECONDS,
        TimerMode::Repeating,
    )));

    commands.spawn((
        Text2d::new("Score : 0"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(COLOR_TEXT),
        // On remonte le texte au-dessus de la grille.
        Transform::from_xyz(0.0, GRID_HEIGHT as f32 * CELL_SIZE / 2.0 + 24.0, 1.0),
        ScoreText,
        OnPlayScreen,
    ));
}

/// Lit les flèches (et ZQSD/WASD) et transmet l'intention à la logique via
/// `try_turn`. C'est la logique, pas l'adaptateur, qui décidera si le virage
/// est légal (pas de demi-tour).
fn handle_input(keyboard: Res<ButtonInput<KeyCode>>, mut state: ResMut<GameState>) {
    let direction = if keyboard.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        Some(Direction::Up)
    } else if keyboard.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        Some(Direction::Down)
    } else if keyboard.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        Some(Direction::Left)
    } else if keyboard.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        Some(Direction::Right)
    } else {
        None
    };

    if let Some(direction) = direction {
        state.game.try_turn(direction);
    }
}

/// Fait avancer la simulation, mais seulement quand le minuteur a sonné.
/// `time.delta()` est la durée écoulée depuis l'image précédente ; on l'injecte
/// dans le minuteur, qui nous dit s'il a (peut-être plusieurs fois) fini.
fn step_game(
    time: Res<Time>,
    mut timer: ResMut<StepTimer>,
    mut state: ResMut<GameState>,
    mut ate_writer: MessageWriter<AteFood>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    match state.game.step() {
        StepOutcome::Moved => {}
        StepOutcome::Ate => {
            // On NE replace PAS la nourriture ici : on signale juste l'événement.
            // Le système `place_new_food` s'en chargera (séparation des rôles).
            ate_writer.write(AteFood);
        }
        StepOutcome::Died => {
            next_state.set(AppState::GameOver);
        }
    }
}

/// Réagit au message `AteFood` : tire une nouvelle case libre au hasard et y
/// place la nourriture. Aucun calcul de règle ici, juste de l'aléatoire (qui
/// n'a pas sa place dans la logique pure).
fn place_new_food(
    mut ate_reader: MessageReader<AteFood>,
    mut state: ResMut<GameState>,
) {
    // `read()` vide le tampon des messages de cette image.
    for _ in ate_reader.read() {
        let mut rng = rand::rng();
        if let Some(&cell) = state.game.free_cells().choose(&mut rng) {
            state.game.place_food(cell);
        }
        // S'il n'y a plus de case libre, le joueur a gagné : on laisse la
        // nourriture où elle est, la prochaine collision finira la partie.
    }
}

/// Redessine tout l'écran de jeu à partir de l'état de `Game`. Stratégie la plus
/// simple et parfaitement fluide ici : on supprime les anciens sprites et on en
/// crée de neufs. (Un vrai gros jeu réutiliserait les entités ; pour un Snake,
/// inutile de se compliquer la vie.)
fn render(
    mut commands: Commands,
    state: Res<GameState>,
    segments: Query<Entity, With<SnakeSegment>>,
    food: Query<Entity, With<FoodSprite>>,
) {
    for entity in &segments {
        commands.entity(entity).despawn();
    }
    for entity in &food {
        commands.entity(entity).despawn();
    }

    // Le serpent, tête en premier : la tête a une couleur plus vive.
    for (index, cell) in state.game.body().enumerate() {
        let color = if index == 0 {
            COLOR_SNAKE_HEAD
        } else {
            COLOR_SNAKE_BODY
        };
        commands.spawn((
            Sprite::from_color(color, cell_sprite_size()),
            Transform::from_translation(cell_to_world(cell)),
            SnakeSegment,
            OnPlayScreen,
        ));
    }

    // La nourriture.
    commands.spawn((
        Sprite::from_color(COLOR_FOOD, cell_sprite_size()),
        Transform::from_translation(cell_to_world(state.game.food())),
        FoodSprite,
        OnPlayScreen,
    ));
}

/// Met le texte du score à jour. `Single` exige qu'il existe exactement UNE
/// entité correspondante — pratique quand on sait qu'il n'y en a qu'une.
fn update_score_text(
    state: Res<GameState>,
    mut text: Single<&mut Text2d, With<ScoreText>>,
) {
    **text = Text2d::new(format!("Score : {}", state.game.score()));
}

/// Nettoie l'écran de jeu en sortant de l'état `Playing` : tous les sprites et
/// le texte de score sont marqués `OnPlayScreen`.
fn despawn_play_screen(mut commands: Commands, query: Query<Entity, With<OnPlayScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// --------------------------------------------------------------------------
// Écran Game Over
// --------------------------------------------------------------------------

fn spawn_game_over_text(mut commands: Commands, state: Res<GameState>) {
    commands.spawn((
        Text2d::new(format!(
            "PERDU\n\nScore : {}\n\nEspace pour rejouer",
            state.game.score()
        )),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(COLOR_TEXT),
        OnMenuScreen,
    ));
}

fn restart_on_space(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(AppState::Playing);
    }
}

// --------------------------------------------------------------------------
// Conversions grille logique → monde Bevy
// --------------------------------------------------------------------------

/// Taille d'un sprite de case (légèrement plus petite que `CELL_SIZE` pour
/// laisser apparaître le quadrillage).
fn cell_sprite_size() -> Vec2 {
    Vec2::splat(CELL_SIZE - CELL_GAP)
}

/// Traduit une case logique (0..GRID) en position monde (centrée sur l'origine).
/// La grille a `(0,0)` en bas-gauche et `y` vers le haut — même convention que
/// Bevy, ce qui rend la conversion directe.
fn cell_to_world(cell: Coord) -> Vec3 {
    let x = (cell.x as f32 - (GRID_WIDTH as f32 - 1.0) / 2.0) * CELL_SIZE;
    let y = (cell.y as f32 - (GRID_HEIGHT as f32 - 1.0) / 2.0) * CELL_SIZE;
    Vec3::new(x, y, 0.0)
}
