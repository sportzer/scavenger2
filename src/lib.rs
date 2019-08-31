use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub use cursive;
use cursive::{
    Cursive,
    Printer,
    direction::Orientation,
    event::{Event, EventResult, Key},
    theme::{BaseColor, Color, ColorStyle},
    vec::Vec2,
    view::View,
    views::{BoxView, LinearLayout},
};

mod game;

use game::{
    Action,
    ActorType,
    EntityType,
    Game,
    Obstruction,
    Tile,
    TileView,
    geometry::{Direction, Position},
};

#[derive(Copy, Clone)]
struct Camera {
    screen_size: Vec2,
    screen_focus: Vec2,
    map_focus: Position,
}

impl Camera {
    fn centered(size: Vec2, pos: Position) -> Camera {
        Camera {
            screen_size: size,
            screen_focus: Vec2::new(size.x/2, size.y/2),
            map_focus: pos,
        }
    }

    fn map_position(&self, offset: Vec2) -> Position {
        Position {
            x: self.map_focus.x - self.screen_focus.x as i32 + offset.x as i32,
            y: self.map_focus.y - self.screen_focus.y as i32 + offset.y as i32,
        }
    }
}

struct GameMap {
    game: Rc<RefCell<Game>>,
    camera: Cell<Option<Camera>>,
}

impl GameMap {
    fn render_tile(view: TileView) -> (&'static str, ColorStyle) {
        // TODO: what if actor/object is embedded in a solid wall?
        let black_bg = |color| ColorStyle::new(color, Color::Dark(BaseColor::Black));
        if let TileView::Visible { actor: Some(actor), .. } = view {
            return match actor {
                ActorType::Player => ("@", black_bg(Color::Light(BaseColor::White))),
                ActorType::Rat => ("r", black_bg(Color::Light(BaseColor::White))),
            };
        }
        let (object, tile, vis) = match view {
            TileView::Visible { object, tile, .. } => (object, tile, true),
            TileView::Remembered { object, tile, .. } => (object, tile, false),
            TileView::Explorable => {
                return ("?", black_bg(Color::Dark(BaseColor::Magenta)));
            }
            TileView::Unknown => {
                return (" ", black_bg(Color::Dark(BaseColor::Black)));
            }
        };
        if let Some(object) = object {
            let color = if vis {
                Color::Light(BaseColor::Red)
            } else {
                Color::Dark(BaseColor::Red)
            };
            return match object {
                // TODO: handle Actor some other way?
                EntityType::Actor(_) => ("!", black_bg(color)),
                EntityType::Corpse(_) => ("%", black_bg(color)),
            };
        }
        let (ch, color) = match tile {
            Tile::Wall => ("#", Color::Dark(BaseColor::Yellow)),
            Tile::Tree => ("#", Color::Dark(BaseColor::Green)),
            Tile::Ground => (".", Color::Light(BaseColor::Yellow)),
        };
        let color = if vis { color } else { Color::Light(BaseColor::Black) };
        let color_style = if tile.obstruction() == Obstruction::Full {
            ColorStyle::new(Color::Dark(BaseColor::Black), color)
        } else {
            black_bg(color)
        };
        (ch, color_style)
    }

    fn event_direction(ev: Event) -> Option<Direction> {
        Some(match ev {
            // arrow keys
            Event::Key(Key::Up) => Direction::North,
            Event::Key(Key::Down) => Direction::South,
            Event::Key(Key::Left) => Direction::West,
            Event::Key(Key::Right) => Direction::East,
            // number keys
            Event::Char('1') => Direction::SouthWest,
            Event::Char('2') => Direction::South,
            Event::Char('3') => Direction::SouthEast,
            Event::Char('4') => Direction::West,
            Event::Char('6') => Direction::East,
            Event::Char('7') => Direction::NorthWest,
            Event::Char('8') => Direction::North,
            Event::Char('9') => Direction::NorthEast,
            // vi keys
            Event::Char('h') => Direction::West,
            Event::Char('j') => Direction::South,
            Event::Char('k') => Direction::North,
            Event::Char('l') => Direction::East,
            Event::Char('y') => Direction::NorthWest,
            Event::Char('u') => Direction::NorthEast,
            Event::Char('b') => Direction::SouthWest,
            Event::Char('n') => Direction::SouthEast,
            _ => { return None; }
        })
    }
}

impl View for GameMap {
    fn draw(&self, pr: &Printer) {
        let game = self.game.borrow();
        let player_pos = match game.player_position() {
            Some(pos) => pos,
            None => { return; }
        };

        // TODO: recenter camera if off screen
        // TODO: manage screen resize
        let camera = self.camera.get().unwrap_or_else(|| {
            Camera::centered(pr.size, player_pos)
        });
        self.camera.set(Some(camera));
        for x in 0..pr.size.x {
            for y in 0..pr.size.y {
                let pos = camera.map_position(Vec2 { x, y });
                let (ch, color_style) = GameMap::render_tile(game.view(pos));
                pr.with_color(color_style, |pr| {
                    pr.print(Vec2::new(x, y), ch);
                });
            }
        }
    }

    fn on_event(&mut self, ev: Event) -> EventResult {
        let action_cb = move |action| {
            let game = self.game.clone();
            EventResult::with_cb(move |_| {
                let mut game = game.borrow_mut();
                // TODO: log error
                let _ = game.take_player_action(action);
            })
        };
        match ev {
            Event::Char('5') => action_cb(Action::Wait),
            Event::Char('.') => action_cb(Action::Wait),
            _ => GameMap::event_direction(ev).map(|dir| action_cb(Action::MoveAttack(dir)))
                .unwrap_or(EventResult::Ignored),
        }
    }

    fn required_size(&mut self, _constraint: Vec2) -> Vec2 {
        Vec2::new(11, 11)
    }
}

pub fn build_ui(siv: &mut Cursive, seed: u64) {
    let game = Rc::new(RefCell::new(Game::new(seed)));

    siv.add_global_callback(Event::CtrlChar('q'), |s| s.quit());

    siv.add_fullscreen_layer(BoxView::with_full_screen(
        LinearLayout::new(Orientation::Vertical)
            .child(BoxView::with_full_screen(GameMap {
                game: game.clone(),
                camera: Cell::new(None),
            }))
    ));
}
