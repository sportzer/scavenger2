use super::{EntityType, Game, Obstruction, Tile, TileView};
use super::geometry::{ORTHOGONAL_DIRECTIONS, Position};
use super::actor::{ActorState, ActorType};

enum RulePt {
    F(i32, i32),
    P(i32, i32),
}

static FOV_RULES: &'static [(&'static [RulePt], (i32, i32))] = {
    use RulePt::*;
    &[
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
    ]
};

// TODO: optimize this shit
pub fn has_los(g: &mut Game, pos1: Position, pos2: Position) -> bool {
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

    use RulePt::*;
    for mapping in mappings {
        let to_pos = |pt| {
            let (dx, dy) = mapping(pt);
            Position { x: pos1.x + dx, y: pos1.y + dy }
        };
        for &(ray, pt) in FOV_RULES {
            if pos2 == to_pos(pt) && ray.iter().all(|pt| match pt {
                &F(dx, dy) => g.tile(to_pos((dx, dy))).obstruction() == Obstruction::None,
                &P(dx, dy) => g.tile(to_pos((dx, dy))).obstruction() != Obstruction::Full,
            }) {
                return true;
            }
        }
    }

    false
}

pub fn update_view(g: &mut Game) {
    for v in g.view.values_mut() {
        if let &mut TileView::Visible { object, tile, .. } = v {
            *v = TileView::Remembered { object, tile };
        }
    }

    let player_pos = match g.player_position() {
        Some(pos) => pos,
        None => { return; }
    };

    let mark_visible = |g: &mut Game, pos| {
        // TODO: what if entity type not actor?
        // TODO: simplify this a bunch please
        // TODO: ghost should maybe go invisible again once no longer in FOV
        let actor_type = if let Some(&actor) = g.actors.get(&pos) {
            if let Some(&EntityType::Actor(actor_type)) = g.types.get(&actor) {
                if actor_type == ActorType::Ghost {
                    if g.visible_ghosts.contains(&actor) {
                        Some(ActorType::Ghost)
                    } else {
                        if pos.adjacent_to(player_pos) {
                            g.states.insert(actor, ActorState::Wait);
                            g.visible_ghosts.insert(actor);
                            Some(ActorType::Ghost)
                        } else {
                            None
                        }
                    }
                } else {
                    Some(actor_type)
                }
            } else { None }
        } else { None };
        g.view.insert(pos, TileView::Visible {
            actor: actor_type,
            object: g.objects.get(&pos).and_then(|v| v.last())
                .and_then(|&e| g.types.get(&e).cloned()),
            tile: g.tile(pos),
        });
    };

    mark_visible(g, player_pos);

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

    use RulePt::*;
    for mapping in mappings {
        let to_pos = |pt| {
            let (dx, dy) = mapping(pt);
            Position { x: player_pos.x + dx, y: player_pos.y + dy }
        };
        for &(ray, pt) in FOV_RULES {
            if ray.iter().all(|pt| match pt {
                &F(dx, dy) => g.tile(to_pos((dx, dy))).obstruction() == Obstruction::None,
                &P(dx, dy) => g.tile(to_pos((dx, dy))).obstruction() != Obstruction::Full,
            }) {
                mark_visible(g, to_pos(pt));
            }
        }
    }

    for dx in -4..=4 {
        for dy in -4..=4 {
            let pos = Position { x: player_pos.x + dx, y: player_pos.y + dy };
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
