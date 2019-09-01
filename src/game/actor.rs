use rand::prelude::*;

use super::{Action, Entity, EntityType, Game, Obstruction, PLAYER, Tile, TileView};
use super::geometry::{Direction, Position};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum ActorType {
    Player,
    Rat,
    Wolf,
    // TODO: more creatures
    // Deer,
    // Dragon,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(super) enum ActorState {
    Wait,
    Wander(Position),
    Pursue(Entity, Position),
    Flee(Entity, Position),
}

fn path(rng: &mut impl Rng, from: Position, to: Position) -> Option<[Direction; 4]> {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    use std::cmp::Ordering::*;
    use Direction::*;
    let (dir2, w2, dir1, w1, diag) = match (dx.cmp(&0), dy.cmp(&0)) {
        (Greater, Less) => (North, -dy, East, dx, NorthEast),
        (Greater, Greater) => (East, dx, South, dy, SouthEast),
        (Less, Greater) => (South, dy, West, -dx, SouthWest),
        (Less, Less) => (West, -dx, North, -dy, NorthWest),
        (Equal, Less) => (North, 1, North, 1, North),
        (Greater, Equal) => (East, 1, East, 1, East),
        (Equal, Greater) => (South, 1, South, 1, South),
        (Less, Equal) => (West, 1, West, 1, West),
        (Equal, Equal) => { return None; }
    };
    let (dir1, dir2, w1, w2) = if w1 == w2 {
        (diag, diag, 1, 1)
    } else if w1 > w2 {
        (dir1, diag, w1-w2, w2)
    } else {
        (diag, dir2, w1, w2-w1)
    };
    Some(if rng.gen_ratio(w1 as u32, (w1+w2) as u32) {
        [dir1, dir2, dir1.rotate_counterclockwise(), dir2.rotate_clockwise()]
    } else {
        [dir2, dir1, dir2.rotate_clockwise(), dir1.rotate_counterclockwise()]
    })
}

fn move_towards(g: &mut Game, e: Entity, pos: Position) {
    if let Some(&epos) = g.positions.get(&e) {
        if let Some(dirs) = path(&mut g.rng, epos, pos) {
            for &dir in &dirs {
                if let Ok(_) = g.take_action(e, Action::MoveAttack(dir)) {
                    return;
                }
            }
        }
    }
    g.states.insert(e, ActorState::Wait);
}

pub(super) fn take_actions(g: &mut Game) {
    // TODO
    for (e, state) in g.states.clone() {
        let actor_type = match g.types.get(&e).cloned() {
            Some(EntityType::Actor(a)) => a,
            _ => { continue; }
        };
        match state {
            ActorState::Wait => {}
            ActorState::Wander(pos) => {}
            ActorState::Pursue(o, pos) => match actor_type {
                ActorType::Player => {}
                ActorType::Rat => {
                    move_towards(g, e, pos);
                }
                ActorType::Wolf => {
                    move_towards(g, e, pos);
                    move_towards(g, e, pos);
                }
            }
            ActorState::Flee(o, pos) => {}
        }
    }
}

pub(super) fn notice_player(g: &mut Game) {
    let player_pos = match g.player_position() {
        Some(pos) => pos,
        None => { return; }
    };

    for (e, s) in g.states.iter_mut() {
        let actor_type = match g.types.get(&e).cloned() {
            Some(EntityType::Actor(a)) => a,
            _ => { continue; }
        };
        if let Some(pos) = g.positions.get(&e).cloned() {
            if let Some(&TileView::Visible { .. }) = g.view.get(&pos) {
                match actor_type {
                    ActorType::Player => {}
                    ActorType::Rat | ActorType::Wolf => {
                        *s = ActorState::Pursue(PLAYER, player_pos)
                    }
                }
            }
        }
    }
}
