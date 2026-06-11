//! Logique pure du jeu Snake — AUCUNE dépendance à Bevy.
//!
//! C'est le cœur du module : toutes les règles du jeu (déplacement,
//! croissance, collisions, score) vivent ici, dans des types et fonctions
//! ordinaires. Aucun `use bevy::...` n'est autorisé dans ce fichier.
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
        (0..GRID_WIDTH).contains(&self.x) && (0..GRID_HEIGHT).contains(&self.y)
    }

    /// La case voisine dans la direction donnée (peut sortir de la grille :
    /// c'est `step` qui décide quoi en faire).
    pub fn moved(self, dir: Direction) -> Self {
        let Coord { x: dx, y: dy } = dir.delta();
        Self::new(self.x + dx, self.y + dy)
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
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    /// Le déplacement élémentaire correspondant, sous forme de `Coord`
    /// utilisé comme vecteur. `Up` augmente `y` (repère « y vers le haut »).
    pub fn delta(self) -> Coord {
        match self {
            Direction::Up => Coord::new(0, 1),
            Direction::Down => Coord::new(0, -1),
            Direction::Left => Coord::new(-1, 0),
            Direction::Right => Coord::new(1, 0),
        }
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
        let cx = GRID_WIDTH / 2;
        let cy = GRID_HEIGHT / 2;
        // Tête au centre, deux segments derrière elle (vers la gauche).
        let body = VecDeque::from([
            Coord::new(cx, cy),
            Coord::new(cx - 1, cy),
            Coord::new(cx - 2, cy),
        ]);
        Self {
            body,
            direction: Direction::Right,
            pending: Direction::Right,
            food,
            score: 0,
        }
    }

    /// La case occupée par la tête.
    pub fn head(&self) -> Coord {
        // L'invariant « le corps n'est jamais vide » rend l'`expect` sûr.
        *self.body.front().expect("le serpent a toujours une tête")
    }

    /// Itère sur les cases du corps, tête en premier.
    /// On retourne un itérateur plutôt qu'une référence au `VecDeque` :
    /// l'extérieur n'a pas à connaître la structure interne.
    pub fn body(&self) -> impl Iterator<Item = Coord> + '_ {
        self.body.iter().copied()
    }

    /// Longueur actuelle du serpent.
    pub fn len(&self) -> usize {
        self.body.len()
    }

    pub fn is_empty(&self) -> bool {
        // Toujours faux, mais Clippy recommande `is_empty` dès qu'on a `len`.
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
        self.body.contains(&cell)
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
        if dir != self.direction.opposite() {
            self.pending = dir;
        }
    }

    /// Avance la simulation d'un pas. C'est LA fonction centrale du jeu.
    pub fn step(&mut self) -> StepOutcome {
        // 1. La direction demandée devient effective.
        self.direction = self.pending;

        // 2. Où la tête veut-elle aller ?
        let new_head = self.head().moved(self.direction);

        // 3. Mur ?
        if !new_head.in_bounds() {
            return StepOutcome::Died;
        }

        // 4. Nourriture ? Le serpent grandit : on ajoute la tête SANS
        //    retirer la queue. La nourriture ne peut pas se trouver sous le
        //    corps (elle n'est placée que sur des cases libres), donc pas de
        //    collision possible dans ce cas.
        if new_head == self.food {
            self.body.push_front(new_head);
            self.score += 1;
            return StepOutcome::Ate;
        }

        // 5. Déplacement normal : la queue libère sa case AVANT le test de
        //    collision. C'est le deuxième piège classique du Snake : la tête
        //    a le droit d'entrer dans la case que la queue quitte au même
        //    instant (cas du serpent qui « se suit » en boucle serrée).
        self.body.pop_back();
        if self.occupies(new_head) {
            return StepOutcome::Died;
        }
        self.body.push_front(new_head);
        StepOutcome::Moved
    }

    /// Place la nourriture sur la case donnée. L'appelant est responsable
    /// de choisir une case libre (voir `free_cells`).
    pub fn place_food(&mut self, cell: Coord) {
        self.food = cell;
    }

    /// Toutes les cases non occupées par le serpent. L'adaptateur Bevy en
    /// tirera une au hasard pour replacer la nourriture : l'aléatoire reste
    /// HORS de la logique pure.
    pub fn free_cells(&self) -> Vec<Coord> {
        (0..GRID_WIDTH)
            .flat_map(|x| (0..GRID_HEIGHT).map(move |y| Coord::new(x, y)))
            .filter(|cell| !self.occupies(*cell))
            .collect()
    }
}
