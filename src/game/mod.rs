use std::collections::HashMap;

use rand::prelude::*;

pub mod fov;
pub mod geometry;

use geometry::{Direction, Position};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum EntityType {
    Player,
    // TODO: creatures
    // Rat,
    // Deer,
    // Wolf,
    // Dragon,
    // TODO: items
    // Scroll,
    // Herb,
    // Sword,
    // Bow,
    // Arrow,
    // Rock,
    // Diamond,
    // TODO: cosmetic
    // Skeleton,
    // Corpse(Creature),
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Tile {
    Wall,
    Tree,
    Ground,
    // TODO:
    // ShallowWater,
    // DeepWater,
    // ShortGrass,
    // LongGrass,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Obstruction {
    Full,
    Partial,
    None,
}

impl Tile {
    pub fn obstruction(self) -> Obstruction {
        match self {
            Tile::Wall => Obstruction::Full,
            Tile::Tree => Obstruction::Partial,
            Tile::Ground => Obstruction::None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TileView {
    Visible {
        actor: Option<EntityType>,
        item: Option<EntityType>,
        tile: Tile,
    },
    Remembered {
        item: Option<EntityType>,
        tile: Tile,
    },
    Explorable,
    Unknown,
}

impl TileView {
    pub fn actor(&self) -> Option<EntityType> {
        match self {
            &TileView::Visible { actor, .. } => actor,
            _ => None,
        }
    }

    pub fn item(&self) -> Option<EntityType> {
        match self {
            &TileView::Visible { item, .. } => item,
            &TileView::Remembered { item, .. } => item,
            _ => None,
        }
    }

    pub fn tile(&self) -> Option<Tile> {
        match self {
            &TileView::Visible { tile, .. } => Some(tile),
            &TileView::Remembered { tile, .. } => Some(tile),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Action {
    Wait,
    Move(Direction),
    // TODO:
    // EatHerb
    // ReadScroll
    // ThrowRock(Position),
    // FireBow(Direction),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ActionError {
    Impassible(Tile),
    IllegalDiagonal,
}

pub struct Game {
    tiles: HashMap<Position, Tile>,
    player_pos: Position,
    rng: StdRng,
    view: HashMap<Position, TileView>,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        let mut g = Game {
            tiles: HashMap::new(),
            player_pos: Position { x: 0, y: 0 },
            rng: StdRng::seed_from_u64(seed),
            view: HashMap::new(),
        };
        // TODO: real map gen
        for x in -20..=20 {
            for y in -20..=20 {
                let sq = (x*x + y*y) as u32;
                if sq <= 25 || sq < 20*20 && !g.rng.gen_ratio(sq, 20*20) {
                    if sq <= 2 || g.rng.gen_ratio(14, 15) {
                        g.tiles.insert(Position { x, y }, Tile::Ground);
                    } else {
                        g.tiles.insert(Position { x, y }, Tile::Tree);
                    }
                }
            }
        }
        fov::update_view(&mut g);
        g
    }

    pub fn view(&self, pos: Position) -> TileView {
        self.view.get(&pos).cloned().unwrap_or(TileView::Unknown)
    }

    pub fn player_position(&self) -> Position {
        self.player_pos
    }

    pub fn take_action(&mut self, action: Action) -> Result<(), ActionError> {
        match action {
            Action::Wait => {}
            Action::Move(dir) => {
                let pos = self.player_pos;
                let new_pos = pos.step(dir);
                let new_tile = self.tile(new_pos);
                if new_tile.obstruction() != Obstruction::None {
                    return Err(ActionError::Impassible(new_tile));
                }
                if let Some((a, b)) = match dir {
                    Direction::NorthEast => Some((Direction::North, Direction::East)),
                    Direction::SouthEast => Some((Direction::South, Direction::East)),
                    Direction::SouthWest => Some((Direction::South, Direction::West)),
                    Direction::NorthWest => Some((Direction::North, Direction::West)),
                    _ => None,
                } {
                    if self.tile(pos.step(a)).obstruction() == Obstruction::Full
                        && self.tile(pos.step(b)).obstruction() == Obstruction::Full
                    {
                        return Err(ActionError::IllegalDiagonal);
                    }
                }
                self.player_pos = new_pos;
            }
        };
        fov::update_view(self);
        Ok(())
    }

    fn tile(&self, pos: Position) -> Tile {
        self.tiles.get(&pos).cloned().unwrap_or(Tile::Wall)
    }
}
