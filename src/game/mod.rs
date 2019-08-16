use std::collections::HashMap;

use rand::prelude::*;

pub mod fov;
pub mod geometry;

use geometry::{Direction, Position};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum EntityType {
    Player,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Tile {
    Wall,
    Ground,
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
    Reachable,
    Unknown,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Action {
    Wait,
    Move(Direction),
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
        for x in -20..=20 {
            for y in -20..=20 {
                let sq = (x*x + y*y) as u32;
                if sq <= 25 || sq < 20*20 && !g.rng.gen_ratio(sq, 20*20) {
                    g.tiles.insert(Position { x, y }, Tile::Ground);
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

    // TODO: return Result?
    pub fn step(&mut self, action: Action) {
        match action {
            Action::Wait => {}
            Action::Move(dir) => {
                let pos = self.player_pos;
                let new_pos = pos.step(dir);
                if self.tile(new_pos) == Tile::Wall {
                    return;
                }
                if let Some((a, b)) = match dir {
                    Direction::NorthEast => Some((Direction::North, Direction::East)),
                    Direction::SouthEast => Some((Direction::South, Direction::East)),
                    Direction::SouthWest => Some((Direction::South, Direction::West)),
                    Direction::NorthWest => Some((Direction::North, Direction::West)),
                    _ => None,
                } {
                    if self.tile(pos.step(a)) == Tile::Wall && self.tile(pos.step(b)) == Tile::Wall {
                        return;
                    }
                }
                self.player_pos = new_pos;
            }
        };
        fov::update_view(self);
    }

    fn tile(&self, pos: Position) -> Tile {
        self.tiles.get(&pos).cloned().unwrap_or(Tile::Wall)
    }
}
