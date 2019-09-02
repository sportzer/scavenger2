use std::collections::{BTreeMap, HashMap};
use std::num::NonZeroU64;

use rand::prelude::*;

mod actor;
mod fov;
mod map;

use actor::ActorState;
use geometry::{Direction, Position};

pub mod geometry;

pub use actor::ActorType;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct Entity(NonZeroU64);

// TODO: switch to new(1).unwrap() once that's const
const PLAYER: Entity = Entity(unsafe { NonZeroU64::new_unchecked(1) });

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum EntityType {
    Actor(ActorType),
    Corpse(ActorType),
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Tile {
    Wall,
    Tree,
    Ground,
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
        actor: Option<ActorType>,
        object: Option<EntityType>,
        tile: Tile,
    },
    Remembered {
        object: Option<EntityType>,
        tile: Tile,
    },
    Explorable,
    Unknown,
}

impl TileView {
    pub fn actor(&self) -> Option<ActorType> {
        match self {
            &TileView::Visible { actor, .. } => actor,
            _ => None,
        }
    }

    pub fn object(&self) -> Option<EntityType> {
        match self {
            &TileView::Visible { object, .. } => object,
            &TileView::Remembered { object, .. } => object,
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
}

// Include info on what exactly went wrong in error?
// Per action type errors?
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ActionError {
    IllegalDiagonal,
    Impassible,
    Occupied,
    InvalidActor,
    NoValidTarget,
}

type ActionResult<Ok = ()> = Result<Ok, ActionError>;

pub struct Game {
    tiles: HashMap<Position, Tile>,

    types: HashMap<Entity, EntityType>,
    states: BTreeMap<Entity, ActorState>,

    // TODO: replace with some sort of indexed map thing
    positions: HashMap<Entity, Position>,
    actors: HashMap<Position, Entity>,
    objects: HashMap<Position, Vec<Entity>>,

    rng: StdRng,
    prev_entity: Entity,
    view: HashMap<Position, TileView>,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        Game::new_from_rng(StdRng::seed_from_u64(seed))
    }

    fn new_from_rng(rng: StdRng) -> Game {
        let mut g = Game {
            tiles: HashMap::new(),
            types: HashMap::new(),
            states: BTreeMap::new(),
            positions: HashMap::new(),
            actors: HashMap::new(),
            objects: HashMap::new(),
            rng: rng,
            prev_entity: PLAYER,
            view: HashMap::new(),
        };
        g.types.insert(PLAYER, EntityType::Actor(ActorType::Player));
        map::generate_basin(&mut g);
        // TODO: handle errors
        let _ = g.set_actor_position(PLAYER, Position { x: 0, y: 0 });
        fov::update_view(&mut g);
        actor::notice_player(&mut g);
        g
    }

    pub fn restart(&mut self) {
        *self = Game::new_from_rng(self.rng.clone());
    }

    pub fn view(&self, pos: Position) -> TileView {
        self.view.get(&pos).cloned().unwrap_or(TileView::Unknown)
    }

    pub fn player_position(&self) -> Option<Position> {
        self.positions.get(&PLAYER).cloned()
    }

    pub fn take_player_action(&mut self, action: Action) -> ActionResult {
        self.take_action(PLAYER, action)?;
        fov::update_view(self);
        actor::take_actions(self);
        fov::update_view(self);
        actor::notice_player(self);
        Ok(())
    }

    fn tile(&self, pos: Position) -> Tile {
        self.tiles.get(&pos).cloned().unwrap_or(Tile::Wall)
    }

    fn take_action(&mut self, e: Entity, action: Action) -> ActionResult {
        let actor_type = match self.types.get(&e) {
            Some(&EntityType::Actor(a)) => a,
            _ => { return Err(ActionError::InvalidActor); }
        };
        let pos = self.positions.get(&e).cloned().ok_or(ActionError::InvalidActor)?;
        // TODO: at some point the various checks used could leak info, so should consume a turn
        // (and update known map information) if you don't already know they're invalid
        match action {
            Action::Wait => {}
            Action::Move(dir) => {
                if actor_type == ActorType::Crab && !dir.is_orthogonal() {
                    return Err(ActionError::IllegalDiagonal);
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
                self.set_actor_position(e, pos.step(dir))?;
            }
            Action::Attack(dir) => {
                let target_pos = pos.step(dir);
                if let Some(&target) = self.actors.get(&target_pos) {
                    if let Some(&EntityType::Actor(target_type)) = self.types.get(&target) {
                        // TODO: not being able to attack crabs diagonally could cause them get
                        // stuck on terrain and be unkillable, but I do like the idea of making them
                        // a bit harder to kill to compensate for being less mobile
                        if (actor_type == ActorType::Crab || target_type == ActorType::Crab) && !dir.is_orthogonal() {
                            return Err(ActionError::IllegalDiagonal);
                        }
                        if actor_type == ActorType::Player || target_type == ActorType::Player {
                            // TODO: chain push multiple beetles?
                            if target_type == ActorType::Beetle {
                                match self.take_action(target, Action::Move(dir)) {
                                    Ok(_) => {
                                        self.states.insert(target, ActorState::Wait);
                                        return Ok(());
                                    }
                                    Err(ActionError::Occupied) => {
                                        if let Some(&victim) = self.actors.get(&target_pos.step(dir)) {
                                            // TODO: use force attack action so know what tried to kill victim?
                                            let _ = self.kill_actor(victim);
                                            let _ = self.take_action(target, Action::Move(dir));
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                            self.kill_actor(target)?;
                            return Ok(());
                        }
                    }
                }
                return Err(ActionError::NoValidTarget);
            }
            Action::MoveAttack(dir) => {
                let result = self.take_action(e, Action::Move(dir));
                if result == Err(ActionError::Occupied) {
                    self.take_action(e, Action::Attack(dir))?;
                } else {
                    result?;
                }
            }
        };
        Ok(())
    }

    fn kill_actor(&mut self, e: Entity) -> ActionResult {
        // TODO: some type system level stuff to avoid having to revalidate this stuff
        let actor_type = match self.types.get(&e) {
            Some(&EntityType::Actor(a)) => a,
            _ => { return Err(ActionError::InvalidActor); }
        };
        self.states.remove(&e);
        self.types.insert(e, EntityType::Corpse(actor_type));
        if let Some(&pos) = self.positions.get(&e) {
            self.actors.remove(&pos);
            self.objects.entry(pos).or_insert_with(Vec::new).push(e);
            if actor_type == ActorType::BigJelly {
                for &dir in &geometry::ORTHOGONAL_DIRECTIONS {
                    let _ = self.spawn_actor(ActorType::LittleJelly, pos.step(dir));
                }
            }
        }
        return Ok(());
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

    fn set_actor_position(&mut self, e: Entity, pos: Position) -> ActionResult<Option<Position>> {
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

    fn spawn_actor(&mut self, t: ActorType, pos: Position) -> ActionResult<Entity> {
        let e = self.new_entity(EntityType::Actor(t));
        // TODO: don't leak entity on invalid placement
        self.set_actor_position(e, pos)?;
        if t != ActorType::Player {
            self.states.insert(e, ActorState::Wait);
        }
        Ok(e)
    }
}
