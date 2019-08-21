use super::{Entity, EntityType, Game, Obstruction, PLAYER, Tile, TileView};
use super::geometry::{Direction, Position};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum ActorType {
    Player,
    Rat,
    // TODO: more creatures
    // Deer,
    // Wolf,
    // Dragon,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(super) enum ActorState {
    Wait,
    Wander(Position),
    Pursue(Entity, Position),
    Flee(Entity, Position),
}

pub(super) fn take_actions(g: &mut Game) {
    // TODO
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
                    ActorType::Rat => {
                        *s = ActorState::Pursue(PLAYER, player_pos)
                    }
                }
            }
        }
    }
}
