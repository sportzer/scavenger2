use rand::prelude::*;

pub struct Game {
    rng: StdRng,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        Game {
            rng: StdRng::seed_from_u64(seed),
        }
    }
}
