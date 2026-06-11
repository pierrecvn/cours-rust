//! Logique pure du jeu Snake — AUCUNE dépendance à Bevy.
//!
//! C'est le cœur du module : toutes les règles du jeu (déplacement,
//! croissance, collisions, score) vivent ici, dans des types et fonctions
//! ordinaires. Aucun `use bevy::...` n'est autorisé dans ce fichier.
//!
//! C'est ICI que tu travailles aux étapes 1 à 5 du PROJET.md. Les corps des
//! méthodes clés sont remplacés par `todo!("étape N: …")` : à toi de les écrire.
//! Les tests de `tests/logic_test.rs` (fournis, à ne pas modifier) te disent
//! exactement ce que chaque fonction doit faire.
//!
//! Avantages de cette séparation (architecture « ports et adaptateurs ») :
//! - la logique est testable unitairement, sans ouvrir de fenêtre ;
//! - elle est indépendante du moteur : on pourrait la brancher demain sur
//!   un autre moteur, un terminal ou un serveur, sans la réécrire ;
//! - les systèmes Bevy restent minuscules : ils ne font que traduire
//!   « entrées clavier → appels de méthodes » et « état du jeu → sprites ».

use std::collections::VecDeque;

/// Dimensions de la grille logique, en nombre de cases.
/// Des nombres impairs donnent une case centrale exacte pour le départ.
pub const GRID_WIDTH: i32 = 17;
pub const GRID_HEIGHT: i32 = 17;

/// Une case de la grille. (0, 0) est le coin bas-gauche ;
/// `y` croît vers le haut, comme dans le repère 2D de Bevy.
///
/// `Copy` est légitime : deux entiers, c'est moins lourd qu'une référence.
/// `Hash` + `Eq` permettraient de ranger des `Coord` dans un `HashSet`
/// (utile pour le défi « obstacles »).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// La case est-elle dans la grille ?
    pub fn in_bounds(self) -> bool {
        // TODO étape 1: vrai si x ∈ [0, GRID_WIDTH[ ET y ∈ [0, GRID_HEIGHT[.
        // Indice : `(0..GRID_WIDTH).contains(&self.x)`.
        todo!("étape 1: in_bounds")
    }

    /// La case voisine dans la direction donnée (peut sortir de la grille :
    /// c'est `step` qui décide quoi en faire).
    pub fn moved(self, dir: Direction) -> Self {
        // TODO étape 1: additionne le delta de la direction aux coordonnées.
        // Indice : `let Coord { x: dx, y: dy } = dir.delta();`
        todo!("étape 1: moved")
    }
}

/// Les quatre directions possibles du serpent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// La direction opposée — sert à interdire le demi-tour.
    pub fn opposite(self) -> Self {
        // TODO étape 1: un `match` sur les quatre variantes.
        todo!("étape 1: opposite")
    }

    /// Le déplacement élémentaire correspondant, sous forme de `Coord`
    /// utilisé comme vecteur. `Up` augmente `y` (repère « y vers le haut »).
    pub fn delta(self) -> Coord {
        // TODO étape 1: Up = (0, 1), Down = (0, -1), Left = (-1, 0), Right = (1, 0).
        todo!("étape 1: delta")
    }
}

/// Résultat d'un pas de simulation. L'appelant (le système Bevy) décide
/// de la suite : replacer la nourriture, déclencher le game over…
/// La logique CONSTATE, l'adaptateur RÉAGIT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOutcome {
    /// Le serpent a simplement avancé d'une case.
    Moved,
    /// Le serpent a mangé la nourriture : il grandit, le score augmente.
    Ate,
    /// Collision (mur ou propre corps) : la partie est terminée.
    Died,
}

/// L'état complet d'une partie.
///
/// Les champs sont privés : l'extérieur passe par les méthodes, ce qui
/// garantit les invariants (le corps n'est jamais vide, la direction ne
/// fait jamais demi-tour…). C'est l'encapsulation du module 2.
#[derive(Debug, Clone)]
pub struct Game {
    /// Le corps du serpent, tête en premier (`front`), queue en dernier
    /// (`back`). `VecDeque` est parfait : on pousse devant, on retire
    /// derrière, les deux en O(1).
    body: VecDeque<Coord>,
    /// La direction réellement suivie au dernier pas.
    direction: Direction,
    /// La direction demandée pour le PROCHAIN pas. On sépare les deux pour
    /// interdire le demi-tour même quand deux touches sont pressées entre
    /// deux ticks (voir `try_turn`).
    pending: Direction,
    /// La case de la nourriture.
    food: Coord,
    /// Le nombre de nourritures mangées.
    score: u32,
}

impl Game {
    /// Crée une partie : serpent de trois cases au centre, orienté vers la
    /// droite. La position de la nourriture est fournie par l'appelant —
    /// la logique pure ne tire JAMAIS de nombre aléatoire elle-même,
    /// c'est ce qui la rend déterministe et testable.
    pub fn new(food: Coord) -> Self {
        // TODO étape 2: construis l'état initial.
        // - tête au centre : (GRID_WIDTH / 2, GRID_HEIGHT / 2) ;
        // - deux segments derrière, vers la gauche (cx - 1, cx - 2) ;
        // - `VecDeque::from([...])`, tête en premier ;
        // - direction et pending = Right, score = 0, food = paramètre reçu.
        todo!("étape 2: new")
    }

    /// La case occupée par la tête.
    pub fn head(&self) -> Coord {
        // TODO étape 2: la case en tête du VecDeque (`front`). L'invariant
        // « corps jamais vide » rend un `expect` légitime ici.
        todo!("étape 2: head")
    }

    /// Itère sur les cases du corps, tête en premier.
    /// On retourne un itérateur plutôt qu'une référence au `VecDeque` :
    /// l'extérieur n'a pas à connaître la structure interne.
    pub fn body(&self) -> impl Iterator<Item = Coord> + '_ {
        // TODO étape 2: `self.body.iter().copied()`.
        // (On garde un corps explicite pour que le type de retour soit connu.)
        self.body.iter().copied()
    }

    /// Longueur actuelle du serpent.
    pub fn len(&self) -> usize {
        // TODO étape 2: la taille du VecDeque.
        todo!("étape 2: len")
    }

    pub fn is_empty(&self) -> bool {
        // Toujours faux en pratique, mais Clippy le réclame dès qu'on a `len`.
        self.body.is_empty()
    }

    pub fn food(&self) -> Coord {
        self.food
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    /// La case est-elle occupée par le serpent ?
    pub fn occupies(&self, cell: Coord) -> bool {
        // TODO étape 2: le corps contient-il cette case ? (`VecDeque::contains`)
        todo!("étape 2: occupies")
    }

    /// Demande un changement de direction pour le prochain pas.
    ///
    /// La demande est refusée si elle ferait faire demi-tour au serpent.
    /// Point subtil : on compare à `direction` (la direction réellement
    /// suivie au dernier pas) et non à `pending`. Sinon, en allant à droite,
    /// presser Haut puis Gauche très vite entre deux ticks ferait passer
    /// `pending` à `Left`… c'est-à-dire un demi-tour, et une mort instantanée
    /// injuste. Ce bug existe dans beaucoup de Snake amateurs !
    pub fn try_turn(&mut self, dir: Direction) {
        // TODO étape 3: si `dir` n'est PAS l'opposé de `self.direction`,
        // alors `self.pending = dir`. Sinon, on ignore la demande.
        todo!("étape 3: try_turn")
    }

    /// Avance la simulation d'un pas. C'est LA fonction centrale du jeu.
    pub fn step(&mut self) -> StepOutcome {
        // TODO étape 4: la fonction centrale. Dans l'ordre :
        //  1. la direction demandée devient effective : direction = pending ;
        //  2. calcule la case visée par la tête (`head().moved(direction)`) ;
        //  3. hors grille ? → StepOutcome::Died ;
        //  4. cette case == food ? → push_front(new_head), score += 1,
        //     StepOutcome::Ate (on NE retire PAS la queue : le serpent grandit) ;
        //  5. sinon déplacement normal : pop_back() AVANT de tester la collision
        //     (la tête a le droit d'entrer dans la case que la queue libère) ;
        //     si occupies(new_head) → Died ; sinon push_front(new_head), Moved.
        todo!("étape 4: step")
    }

    /// Place la nourriture sur la case donnée. L'appelant est responsable
    /// de choisir une case libre (voir `free_cells`).
    pub fn place_food(&mut self, cell: Coord) {
        // TODO étape 5: range simplement `cell` dans `self.food`.
        todo!("étape 5: place_food")
    }

    /// Toutes les cases non occupées par le serpent. L'adaptateur Bevy en
    /// tirera une au hasard pour replacer la nourriture : l'aléatoire reste
    /// HORS de la logique pure.
    pub fn free_cells(&self) -> Vec<Coord> {
        // TODO étape 5: toutes les cases (x, y) de la grille telles que
        // !self.occupies((x, y)). Indice : `(0..GRID_WIDTH).flat_map(...)`
        // puis `.filter(...).collect()`.
        todo!("étape 5: free_cells")
    }
}
