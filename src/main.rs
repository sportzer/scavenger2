fn main() {
    let siv = &mut cursive::Cursive::default();
    let seed = rand::random();
    scavenger::build_ui(siv, seed);
    siv.run();
}
