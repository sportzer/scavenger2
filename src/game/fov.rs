use super::{EntityType, Game, Obstruction, Tile, TileView};
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
    enum RulePt {
        F(i32, i32),
        P(i32, i32),
    }
    use RulePt::*;
    let fov_rules: &[(&[_], _)] = &[
        (&[], (1, 0)),
        (&[P(1, 0)], (1, 1)),
        (&[F(1, 0)], (2, 0)),
        (&[P(0, 1), F(1, 1)], (2, 1)),
        (&[P(1, 0), F(1, 1)], (2, 1)),
        (&[F(1, 0), P(1, 1)], (2, 1)),
        (&[F(1, 0), P(2, 0)], (2, 1)),
        (&[P(1, 0), F(1, 1), P(2, 1)], (2, 2)),
        (&[P(0, 1), F(1, 1), P(2, 1)], (2, 2)),
        (&[F(1, 0), F(2, 0)], (3, 0)),
        (&[F(1, 0), P(2, 0), F(2, 1)], (3, 1)),
        (&[F(1, 0), P(1, 1), F(2, 1)], (3, 1)),
    ];

    for mapping in mappings {
        let to_pos = |pt| {
            let (dx, dy) = mapping(pt);
            Position { x: pos.x + dx, y: pos.y + dy }
        };
        for &(ray, pt) in fov_rules {
            if ray.iter().all(|pt| match pt {
                &F(dx, dy) => g.tile(to_pos((dx, dy))).obstruction() == Obstruction::None,
                &P(dx, dy) => g.tile(to_pos((dx, dy))).obstruction() != Obstruction::Full,
            }) {
                let pos = to_pos(pt);
                let tile = g.tile(pos);
                g.view.insert(pos, TileView::Visible {
                    actor: None,
                    item: None,
                    tile,
                });
            }
        }
    }

    for dx in -4..=4 {
        for dy in -4..=4 {
            let pos = Position { x: pos.x + dx, y: pos.y + dy };
            if g.view(pos) == TileView::Unknown {
                for &dir in &ORTHOGONAL_DIRECTIONS {
                    let adj_pos = pos.step(dir);
                    if let Some(tile) = g.view(adj_pos).tile() {
                        if match tile.obstruction() {
                            Obstruction::None => true,
                            Obstruction::Partial => ORTHOGONAL_DIRECTIONS.iter().cloned().any(|d| {
                                d != dir && d != dir.reverse()
                                    && g.view(adj_pos.step(d)).tile().map(Tile::obstruction) == Some(Obstruction::None)
                            }),
                            Obstruction::Full => false,
                        } {
                            g.view.insert(pos, TileView::Explorable);
                            break;
                        }
                    }
                }
            }
        }
    }
}
