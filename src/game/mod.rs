use std::collections::HashMap;
use std::num::NonZeroU64;

use rand::prelude::*;

mod fov;
mod map;

pub mod geometry;

use geometry::{Direction, Position};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct Entity(NonZeroU64);

// TODO: switch to new(1).unwrap() once that's const
const PLAYER: Entity = Entity(unsafe { NonZeroU64::new_unchecked(1) });

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum EntityType {
    Player,
    // TODO: creatures
    Rat,
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
    Attack(Direction),
    MoveAttack(Direction),
    // TODO:
    // EatHerb
    // ReadScroll
    // ThrowRock(Position),
    // FireBow(Direction),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ActionError {
    Unspecified,
    // Include info on what you bumped into?
    IllegalDiagonal,
    Impassible,
    Occupied,
}

type ActionResult<Ok = ()> = Result<Ok, ActionError>;

pub struct Game {
    tiles: HashMap<Position, Tile>,
    types: HashMap<Entity, EntityType>,

    // TODO: replace with some sort of indexed map thing
    positions: HashMap<Entity, Position>,
    actors: HashMap<Position, Entity>,

    rng: StdRng,
    prev_entity: Entity,
    view: HashMap<Position, TileView>,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        let mut g = Game {
            tiles: HashMap::new(),
            types: HashMap::new(),
            positions: HashMap::new(),
            actors: HashMap::new(),
            rng: StdRng::seed_from_u64(seed),
            prev_entity: PLAYER,
            view: HashMap::new(),
        };
        g.types.insert(PLAYER, EntityType::Player);
        map::generate_basin(&mut g);
        // TODO: handle errors
        let _ = g.set_position(PLAYER, Position { x: 0, y: 0 });
        fov::update_view(&mut g);
        g
    }

    pub fn view(&self, pos: Position) -> TileView {
        self.view.get(&pos).cloned().unwrap_or(TileView::Unknown)
    }

    pub fn player_position(&self) -> Option<Position> {
        self.positions.get(&PLAYER).cloned()
    }

    pub fn take_player_action(&mut self, action: Action) -> ActionResult {
        let pos = self.player_position().ok_or(ActionError::Unspecified)?;
        // TODO: at some point the various checks used could leak info, so should consume a turn
        // (and update known map information) if you don't already know they're invalid
        match action {
            Action::Wait => {}
            Action::Move(dir) => {
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
                self.set_position(PLAYER, pos.step(dir))?;
            }
            Action::Attack(dir) => {
                // TODO: real damage and death handling
                let target_pos = pos.step(dir);
                if let Some(&e) = self.actors.get(&target_pos) {
                    self.actors.remove(&target_pos);
                    self.positions.remove(&e);
                    self.types.remove(&e);
                }
            }
            Action::MoveAttack(dir) => {
                let r = self.take_player_action(Action::Move(dir));
                if r == Err(ActionError::Occupied) {
                    self.take_player_action(Action::Attack(dir))?;
                } else {
                    r?;
                }
            }
        };
        fov::update_view(self);
        Ok(())
    }

    fn tile(&self, pos: Position) -> Tile {
        self.tiles.get(&pos).cloned().unwrap_or(Tile::Wall)
    }

    fn new_entity(&mut self, entity_type: EntityType) -> Entity {
        if let Some(id) = self.prev_entity.0.get().checked_add(1).and_then(NonZeroU64::new) {
            let new_entity = Entity(id);
            self.prev_entity = new_entity;
            self.types.insert(new_entity, entity_type);
            new_entity
        } else {
            // TODO: not actually unreachable...
            unreachable!();
        }
    }

    fn set_position(&mut self, e: Entity, pos: Position) -> ActionResult<Option<Position>> {
        let new_tile = self.tile(pos);
        if new_tile.obstruction() != Obstruction::None {
            return Err(ActionError::Impassible);
        }

        if let Some(&other) = self.actors.get(&pos) {
            if other == e {
                return Ok(Some(pos));
            } else {
                return Err(ActionError::Occupied);
            }
        }

        let old_pos = self.positions.insert(e, pos);
        if let Some(old_pos) = old_pos {
            self.actors.remove(&old_pos);
        }
        self.actors.insert(pos, e);
        Ok(old_pos)
    }
}
