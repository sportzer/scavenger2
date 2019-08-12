use std::cell::{Cell, RefCell};
use std::rc::Rc;

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
    EntityType,
    Game,
    Tile,
    TileView,
    geometry::{Direction, Position},
};

struct GameMap {
    game: Rc<RefCell<Game>>,
    camera: Cell<Option<Camera>>,
}

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

impl View for GameMap {
    fn draw(&self, pr: &Printer) {
        let game = self.game.borrow();

        let camera = self.camera.get().unwrap_or_else(|| {
            Camera::centered(pr.size, game.player_position())
        });
        self.camera.set(Some(camera));
        for x in 0..pr.size.x {
            for y in 0..pr.size.y {
                let pos = camera.map_position(Vec2 { x, y });
                let v = game.view(pos);
                let (ch, color) = match v {
                    TileView::Visible { actor: Some(_), .. } => ("@", Color::Light(BaseColor::White)),
                    _ => (".", Color::Dark(BaseColor::Green)),
                };
                let color_style = ColorStyle::new(color, Color::Dark(BaseColor::Black));
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
                game.step(action);
            })
        };
        match ev {
            Event::Key(Key::Up) => action_cb(Action::Move(Direction::North)),
            Event::Key(Key::Down) => action_cb(Action::Move(Direction::South)),
            Event::Key(Key::Left) => action_cb(Action::Move(Direction::West)),
            Event::Key(Key::Right) => action_cb(Action::Move(Direction::East)),
            _ => EventResult::Ignored,
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
