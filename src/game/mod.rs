use rand::prelude::*;

pub mod geometry;

use geometry::Position;

pub struct Game {
    player_pos: Position,
    rng: StdRng,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum EntityType {
    Player,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Tile {
    Wall,
    Floor,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
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
    Unknown,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        Game {
            player_pos: Position { x: 0, y: 0 },
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn view(&self, pos: Position) -> TileView {
        if pos == self.player_position() {
            TileView::Visible {
                actor: Some(EntityType::Player),
                item: None,
                tile: Tile::Floor,
            }
        } else {
            TileView::Visible {
                actor: None,
                item: None,
                tile: Tile::Floor,
            }
        }
    }

    pub fn player_position(&self) -> Position {
        self.player_pos
    }
}