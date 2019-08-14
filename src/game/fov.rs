use super::{EntityType, Game, Tile, TileView};
use super::geometry::{ORTHOGONAL_DIRECTIONS, Position};

pub fn update_view(g: &mut Game) {
    for v in g.view.values_mut() {
        if let &mut TileView::Visible { item, tile, .. } = v {
            *v = TileView::Remembered { item, tile };
        }
    }

    let pos = g.player_pos;
    g.view.insert(pos, TileView::Visible {
        actor: Some(EntityType::Player),
        item: None,
        tile: g.tile(pos),
    });

    let mappings: &[fn((i32, i32)) -> (i32, i32)] = &[
        |(x, y)| (x, y),
        |(x, y)| (x, -y),
        |(x, y)| (-x, y),
        |(x, y)| (-x, -y),
        |(x, y)| (y, x),
        |(x, y)| (y, -x),
        |(x, y)| (-y, x),
        |(x, y)| (-y, -x),
    ];
    let fov_rules: &[(&[_], _)] = &[
        (&[], (1, 0)),
        (&[(1, 0)], (1, 1)),
        (&[(1, 0)], (2, 0)),
        (&[(0, 1), (1, 1)], (2, 1)),
        (&[(1, 0), (1, 1)], (2, 1)),
        (&[(1, 0), (2, 0)], (2, 1)),
        (&[(1, 0), (1, 1), (2, 1)], (2, 2)),
        (&[(0, 1), (1, 1), (2, 1)], (2, 2)),
        (&[(1, 0), (2, 0)], (3, 0)),
        (&[(1, 0), (2, 0), (2, 1)], (3, 1)),
        (&[(1, 0), (1, 1), (2, 1)], (3, 1)),
    ];
    for mapping in mappings {
        let to_pos = |(dx, dy)| {
            let (dx, dy) = mapping((dx, dy));
            Position { x: pos.x + dx, y: pos.y + dy }
        };
        for &(ray, pt) in fov_rules {
            if ray.iter().cloned().map(to_pos).all(|p| g.tile(p) == Tile::Floor) {
                let pos = to_pos(pt);
                let tile = g.tile(pos);
                g.view.insert(pos, TileView::Visible {
                    actor: None,
                    item: None,
                    tile,
                });
                if tile == Tile::Floor {
                    for &dir in &ORTHOGONAL_DIRECTIONS {
                        let pos = pos.step(dir);
                        if g.view(pos) == TileView::Unknown {
                            g.view.insert(pos, TileView::Reachable);
                        }
                    }
                }
            }
        }
    }
}
