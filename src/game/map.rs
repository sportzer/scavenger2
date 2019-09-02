use rand::prelude::*;

use super::{Game, Tile};
use super::geometry::Position;
use super::actor::ActorType;

pub fn generate_basin(g: &mut Game) {
    // TODO: real map gen
    for x in -20..=20 {
        for y in -20..=20 {
            let sq = (x*x + y*y) as u32;
            if sq <= 25 || sq < 20*20 && !g.rng.gen_ratio(sq, 20*20) {
                let pos = Position { x, y };
                if sq <= 2 || g.rng.gen_ratio(14, 15) {
                    g.tiles.insert(pos, Tile::Ground);
                    // TODO: actual weighted selection
                    if sq > 10 || sq > 5 && g.rng.gen_ratio(1, 3) {
                        if g.rng.gen_ratio(1, 50) {
                            let _ = g.spawn_actor(ActorType::Rat, pos);
                        } else if g.rng.gen_ratio(1, 100) {
                            let _ = g.spawn_actor(ActorType::Wolf, pos);
                        } else if g.rng.gen_ratio(1, 50) {
                            let _ = g.spawn_actor(ActorType::Crab, pos);
                        } else if g.rng.gen_ratio(1, 50) {
                            let _ = g.spawn_actor(ActorType::Beetle, pos);
                        } else if g.rng.gen_ratio(1, 50) {
                            let _ = g.spawn_actor(ActorType::BigJelly, pos);
                        } else if g.rng.gen_ratio(1, 100) {
                            let _ = g.spawn_actor(ActorType::LittleJelly, pos);
                        } else if g.rng.gen_ratio(1, 50) {
                            let _ = g.spawn_actor(ActorType::Ghost, pos);
                        } else if g.rng.gen_ratio(1, 50) {
                            let _ = g.spawn_actor(ActorType::Dragonfly, pos);
                        }
                    }
                } else {
                    g.tiles.insert(pos, Tile::Tree);
                }
            }
        }
    }
}
