//! Tests de la logique pure du Snake.
//!
//! Remarque : pas une ligne de Bevy ici. On teste des règles de jeu comme
//! on testerait n'importe quelle bibliothèque — c'est tout l'intérêt d'avoir
//! isolé la logique dans `logic.rs`.
//!
//! Les noms sont préfixés par étape (`direction_`, `new_`, `turn_`, `step_`,
//! `food_`, `free_`) pour pouvoir lancer un sous-ensemble :
//!     cargo test step        # ne lance que les tests dont le nom contient « step »

use snake::logic::{Coord, Direction, Game, StepOutcome, GRID_HEIGHT, GRID_WIDTH};

/// Une partie avec la nourriture dans un coin, loin du serpent :
/// pratique pour tester le déplacement sans manger par accident.
fn game_with_food_in_corner() -> Game {
    Game::new(Coord::new(0, 0))
}

// ---------------------------------------------------------------- étape 1

#[test]
fn direction_opposites_are_correct() {
    assert_eq!(Direction::Up.opposite(), Direction::Down);
    assert_eq!(Direction::Down.opposite(), Direction::Up);
    assert_eq!(Direction::Left.opposite(), Direction::Right);
    assert_eq!(Direction::Right.opposite(), Direction::Left);
}

#[test]
fn direction_deltas_use_y_up() {
    // Convention du repère Bevy : y croît vers le haut.
    assert_eq!(Direction::Up.delta(), Coord::new(0, 1));
    assert_eq!(Direction::Down.delta(), Coord::new(0, -1));
    assert_eq!(Direction::Left.delta(), Coord::new(-1, 0));
    assert_eq!(Direction::Right.delta(), Coord::new(1, 0));
}

#[test]
fn direction_moved_applies_delta() {
    let cell = Coord::new(3, 5);
    assert_eq!(cell.moved(Direction::Up), Coord::new(3, 6));
    assert_eq!(cell.moved(Direction::Left), Coord::new(2, 5));
}

#[test]
fn direction_in_bounds_detects_walls() {
    assert!(Coord::new(0, 0).in_bounds());
    assert!(Coord::new(GRID_WIDTH - 1, GRID_HEIGHT - 1).in_bounds());
    assert!(!Coord::new(-1, 0).in_bounds());
    assert!(!Coord::new(0, -1).in_bounds());
    assert!(!Coord::new(GRID_WIDTH, 0).in_bounds());
    assert!(!Coord::new(0, GRID_HEIGHT).in_bounds());
}

// ---------------------------------------------------------------- étape 2

#[test]
fn new_game_snake_has_three_segments() {
    let game = game_with_food_in_corner();
    assert_eq!(game.len(), 3);
}

#[test]
fn new_game_head_is_centered() {
    let game = game_with_food_in_corner();
    assert_eq!(game.head(), Coord::new(GRID_WIDTH / 2, GRID_HEIGHT / 2));
}

#[test]
fn new_game_body_extends_left_of_head() {
    let game = game_with_food_in_corner();
    let body: Vec<Coord> = game.body().collect();
    let (cx, cy) = (GRID_WIDTH / 2, GRID_HEIGHT / 2);
    assert_eq!(
        body,
        vec![
            Coord::new(cx, cy),
            Coord::new(cx - 1, cy),
            Coord::new(cx - 2, cy)
        ],
        "le corps doit être renvoyé tête en premier, puis vers la gauche"
    );
}

#[test]
fn new_game_score_is_zero() {
    assert_eq!(game_with_food_in_corner().score(), 0);
}

#[test]
fn new_game_occupies_only_snake_cells() {
    let game = game_with_food_in_corner();
    let (cx, cy) = (GRID_WIDTH / 2, GRID_HEIGHT / 2);
    assert!(game.occupies(Coord::new(cx, cy)));
    assert!(game.occupies(Coord::new(cx - 2, cy)));
    assert!(!game.occupies(Coord::new(cx + 1, cy)));
    assert!(!game.occupies(Coord::new(0, 0)));
}

#[test]
fn new_game_stores_given_food() {
    let game = Game::new(Coord::new(2, 3));
    assert_eq!(game.food(), Coord::new(2, 3));
}

// ---------------------------------------------------------------- étape 3

#[test]
fn turn_perpendicular_is_accepted() {
    let mut game = game_with_food_in_corner(); // vers la droite
    game.try_turn(Direction::Up);
    game.step();
    assert_eq!(
        game.head(),
        Coord::new(GRID_WIDTH / 2, GRID_HEIGHT / 2 + 1),
        "après un virage vers le haut, la tête doit monter d'une case"
    );
}

#[test]
fn turn_reverse_is_ignored() {
    let mut game = game_with_food_in_corner(); // vers la droite
    game.try_turn(Direction::Left); // demi-tour interdit
    game.step();
    assert_eq!(
        game.head(),
        Coord::new(GRID_WIDTH / 2 + 1, GRID_HEIGHT / 2),
        "le demi-tour doit être ignoré : le serpent continue vers la droite"
    );
}

#[test]
fn turn_double_input_between_ticks_cannot_reverse() {
    // En allant à droite, presser Haut PUIS Gauche avant le tick suivant
    // ne doit pas aboutir à un demi-tour : Gauche est refusée car comparée
    // à la direction réellement suivie (droite), pas à la demande en attente.
    let mut game = game_with_food_in_corner();
    game.try_turn(Direction::Up);
    game.try_turn(Direction::Left);
    game.step();
    assert_eq!(
        game.head(),
        Coord::new(GRID_WIDTH / 2, GRID_HEIGHT / 2 + 1),
        "c'est la demande Haut qui doit s'appliquer, pas Gauche"
    );
}

// ---------------------------------------------------------------- étape 4

#[test]
fn step_moves_right_by_default() {
    let mut game = game_with_food_in_corner();
    let outcome = game.step();
    assert_eq!(outcome, StepOutcome::Moved);
    assert_eq!(game.head(), Coord::new(GRID_WIDTH / 2 + 1, GRID_HEIGHT / 2));
    assert_eq!(game.len(), 3, "sans manger, le serpent ne grandit pas");
}

#[test]
fn step_eating_grows_and_scores() {
    let mut game = game_with_food_in_corner();
    let in_front = Coord::new(GRID_WIDTH / 2 + 1, GRID_HEIGHT / 2);
    game.place_food(in_front);
    let outcome = game.step();
    assert_eq!(outcome, StepOutcome::Ate);
    assert_eq!(game.head(), in_front, "la tête arrive sur la nourriture");
    assert_eq!(game.len(), 4, "manger fait grandir d'une case");
    assert_eq!(game.score(), 1);
}

#[test]
fn step_hitting_right_wall_dies() {
    let mut game = game_with_food_in_corner();
    // La tête part de x = GRID_WIDTH / 2 et va vers la droite :
    // elle peut encore avancer jusqu'à x = GRID_WIDTH - 1, puis c'est le mur.
    let moves_before_wall = (GRID_WIDTH - 1) - GRID_WIDTH / 2;
    for i in 0..moves_before_wall {
        assert_eq!(game.step(), StepOutcome::Moved, "pas n°{i} encore libre");
    }
    assert_eq!(game.step(), StepOutcome::Died, "le pas suivant sort de la grille");
}

#[test]
fn step_hitting_bottom_wall_dies() {
    let mut game = game_with_food_in_corner();
    game.try_turn(Direction::Down);
    let moves_before_wall = GRID_HEIGHT / 2;
    for _ in 0..moves_before_wall {
        assert_eq!(game.step(), StepOutcome::Moved);
    }
    assert_eq!(game.step(), StepOutcome::Died);
}

#[test]
fn step_self_collision_dies() {
    // On fait grandir le serpent à 5 cases, puis on l'enroule sur lui-même :
    // droite, droite (en mangeant), haut, gauche, bas → la tête frappe le corps.
    let mut game = game_with_food_in_corner();
    let (cx, cy) = (GRID_WIDTH / 2, GRID_HEIGHT / 2);
    game.place_food(Coord::new(cx + 1, cy));
    assert_eq!(game.step(), StepOutcome::Ate);
    game.place_food(Coord::new(cx + 2, cy));
    assert_eq!(game.step(), StepOutcome::Ate); // longueur 5
    game.try_turn(Direction::Up);
    assert_eq!(game.step(), StepOutcome::Moved);
    game.try_turn(Direction::Left);
    assert_eq!(game.step(), StepOutcome::Moved);
    game.try_turn(Direction::Down);
    assert_eq!(game.step(), StepOutcome::Died, "la tête redescend sur le corps");
}

#[test]
fn step_can_enter_cell_freed_by_tail() {
    // Cas subtil : le serpent (longueur 4) tourne en carré serré. Sa tête
    // entre dans la case que la queue libère AU MÊME pas. Ce n'est PAS une
    // collision : il faut retirer la queue avant de tester la collision.
    let mut game = game_with_food_in_corner();
    let (cx, cy) = (GRID_WIDTH / 2, GRID_HEIGHT / 2);
    game.place_food(Coord::new(cx + 1, cy));
    assert_eq!(game.step(), StepOutcome::Ate); // longueur 4
    game.try_turn(Direction::Up);
    assert_eq!(game.step(), StepOutcome::Moved);
    game.try_turn(Direction::Left);
    assert_eq!(game.step(), StepOutcome::Moved);
    game.try_turn(Direction::Down);
    assert_eq!(
        game.step(),
        StepOutcome::Moved,
        "entrer dans la case que la queue vient de libérer est autorisé"
    );
}

// ---------------------------------------------------------------- étape 5

#[test]
fn free_cells_excludes_snake() {
    let game = game_with_food_in_corner();
    let free = game.free_cells();
    let total = (GRID_WIDTH * GRID_HEIGHT) as usize;
    assert_eq!(free.len(), total - 3, "toutes les cases sauf les 3 du serpent");
    for cell in game.body() {
        assert!(!free.contains(&cell), "{cell:?} est sous le serpent");
    }
}

#[test]
fn free_cells_are_all_in_bounds() {
    let game = game_with_food_in_corner();
    assert!(game.free_cells().iter().all(|cell| cell.in_bounds()));
}

#[test]
fn food_can_be_replaced() {
    let mut game = game_with_food_in_corner();
    game.place_food(Coord::new(5, 7));
    assert_eq!(game.food(), Coord::new(5, 7));
}
