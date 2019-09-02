use rand::prelude::*;

use super::{Action, ActionError, Entity, EntityType, Game, TileView};
use super::geometry::{Direction, Position};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum ActorType {
    // TODO: more creatures
    Player,
    Rat,
    Wolf,
    Crab,
    Beetle,
    BigJelly,
    LittleJelly,
    Ghost,
    Dragonfly,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(super) enum ActorState {
    Wait,
    Pursue(Position),
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
        [dir1, dir2, dir1.rotate_clockwise(), dir2.rotate_counterclockwise()]
    } else {
        [dir2, dir1, dir2.rotate_counterclockwise(), dir1.rotate_clockwise()]
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

fn knights_move(g: &mut Game, e: Entity, pos: Position) {
    if let Some(&epos) = g.positions.get(&e) {
        let mut dests = {
            use Direction::*;
            [
                epos.step(North).step(NorthWest),
                epos.step(North).step(NorthEast),
                epos.step(East).step(NorthEast),
                epos.step(East).step(SouthEast),
                epos.step(South).step(SouthEast),
                epos.step(South).step(SouthWest),
                epos.step(West).step(SouthWest),
                epos.step(West).step(NorthWest),
            ]
        };
        let dist = epos.chebyshev_distance(pos);
        dests.shuffle(&mut g.rng);
        // TODO: sort by distance rather than partitioning?
        dests.sort_by_key(|p| p.chebyshev_distance(pos) > dist);
        for &dest in &dests {
            if !super::fov::has_los(g, epos, dest) {
                continue;
            }
            match g.set_actor_position(e, dest) {
                Ok(_) => { return; }
                Err(ActionError::Occupied) => {
                    if Some(dest) == g.player_position() {
                        if let Ok(_) = g.kill_actor(super::PLAYER) {
                            let _ = g.set_actor_position(e, dest);
                            return;
                        }
                    }
                }
                Err(_) => {}
            }
        }
    }
    g.states.insert(e, ActorState::Wait);
}

pub(super) fn take_actions(g: &mut Game) {
    // TODO: randomize order
    for (e, state) in g.states.clone() {
        let actor_type = match g.types.get(&e).cloned() {
            Some(EntityType::Actor(a)) => a,
            _ => { continue; }
        };
        match state {
            ActorState::Wait => {}
            ActorState::Pursue(pos) => match actor_type {
                ActorType::Player => {}
                ActorType::Wolf => {
                    move_towards(g, e, pos);
                    move_towards(g, e, pos);
                }
                ActorType::Dragonfly => {
                    knights_move(g, e, pos);
                }
                _ => {
                    move_towards(g, e, pos);
                }
            }
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
        if actor_type == ActorType::Player {
            continue;
        }
        if let Some(pos) = g.positions.get(&e).cloned() {
            if let Some(&TileView::Visible { .. }) = g.view.get(&pos) {
                *s = ActorState::Pursue(player_pos)
            }
        }
    }
}
