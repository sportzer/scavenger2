use std::cell::RefCell;
use std::rc::Rc;

use cursive::{
    Cursive,
    event::Event,
};

mod game;

pub fn build_ui(siv: &mut Cursive, seed: u64) {
    let game = Rc::new(RefCell::new(game::Game::new(seed)));

    siv.add_global_callback(Event::CtrlChar('q'), |s| s.quit());
}
