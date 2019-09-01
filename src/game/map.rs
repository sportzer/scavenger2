use rand::prelude::*;

use super::{EntityType, Game, Tile};
use super::geometry::Position;
use super::actor::{ActorState, ActorType};

pub fn generate_basin(g: &mut Game) {
    // TODO: real map gen
    for x in -20..=20 {
        for y in -20..=20 {
            let sq = (x*x + y*y) as u32;
            if sq <= 25 || sq < 20*20 && !g.rng.gen_ratio(sq, 20*20) {
                let pos = Position { x, y };
                if sq <= 2 {
                    g.tiles.insert(pos, Tile::Ground);
                } else if g.rng.gen_ratio(14, 15) {
                    g.tiles.insert(pos, Tile::Ground);
                    if g.rng.gen_ratio(1, 15) {
                        let rat = g.new_entity(EntityType::Actor(ActorType::Rat));
                        let _ = g.set_actor_position(rat, pos);
                        g.states.insert(rat, ActorState::Wait);
                    } else if g.rng.gen_ratio(1, 50) {
                        let wolf = g.new_entity(EntityType::Actor(ActorType::Wolf));
                        let _ = g.set_actor_position(wolf, pos);
                        g.states.insert(wolf, ActorState::Wait);
                    }
                } else {
                    g.tiles.insert(pos, Tile::Tree);
                }
            }
        }
    }
}
