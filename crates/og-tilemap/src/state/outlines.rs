//! Chunk-local planar union boundaries. Tile polygons are split at their
//! intersections; edges inside the union are discarded, including shared
//! edges across chunk borders. No triangles or synthetic chunk walls.

use super::*;
use crate::map::FLIP_DIAGONAL;

#[derive(Clone, Debug, PartialEq)]
pub struct MapOutline {
    pub positions: Vec<Vec2>,
    pub segments: Vec<[u32; 2]>,
    pub layers: u32,
    pub mask: u32,
}

fn inside(p: Vec2, polygon: &[Vec2]) -> bool {
    let mut odd = false;
    for (&a, &b) in polygon.iter().zip(polygon.iter().cycle().skip(1)) {
        if (a.y > p.y) != (b.y > p.y) && p.x < a.x + (p.y - a.y) * (b.x - a.x) / (b.y - a.y) {
            odd = !odd;
        }
    }
    odd
}

/// Collision polygons in map-local coordinates, in cell order. The bool
/// says this chunk owns the polygon; neighbours in the authored overhang
/// halo participate in boundary unions but do not emit duplicate surfaces.
pub fn chunk_polygons(state: &MapState, layer: usize, cx: u32, cy: u32) -> Vec<(bool, Vec<Vec2>)> {
    let l = &state.asset.layers[layer];
    if !l.collision {
        return Vec::new();
    }
    let Some(ts) = state.tileset(layer) else {
        return Vec::new();
    };
    let size = state.asset.tile_size();
    let px = state.asset.unit / state.asset.tile_width_px.max(1) as f32;
    let (tw, th) = (ts.tile_width as f32, ts.tile_height as f32);
    let mut polys = Vec::new();
    // Tiled allows overhangs. Derive the halo from authored bounds instead
    // of scanning the map or assuming every polygon fits within its tile.
    let ortho = state.asset.orientation == Orientation::Orthogonal;
    let halo = ts.collision_halo() * if ortho { 1 } else { 2 };
    let (hx, hy) = (halo, halo);
    for y in (cy * CHUNK).saturating_sub(hy)..((cy + 1) * CHUNK + hy).min(state.asset.height) {
        for x in (cx * CHUNK).saturating_sub(hx)..((cx + 1) * CHUNK + hx).min(state.asset.width) {
            let Some((tile, fx, fy)) = state.tile(layer, x, y) else {
                continue;
            };
            let owned = x / CHUNK == cx && y / CHUNK == cy;
            let centre = cell_center(&state.asset, x, y) + l.offset;
            let def = ts.tiles.get(&tile);
            let mut shapes = def.map_or_else(Vec::new, |d| d.polygons.clone());
            match ts.collision(tile) {
                Some(Collision::Full) if !ortho => {
                    let corners: &[[f32; 2]] = match state.asset.orientation {
                        Orientation::Hexagonal { flat: false } => &[
                            [0.0, 0.5],
                            [-0.5, 0.25],
                            [-0.5, -0.25],
                            [0.0, -0.5],
                            [0.5, -0.25],
                            [0.5, 0.25],
                        ],
                        Orientation::Hexagonal { flat: true } => &[
                            [0.5, 0.0],
                            [0.25, 0.5],
                            [-0.25, 0.5],
                            [-0.5, 0.0],
                            [-0.25, -0.5],
                            [0.25, -0.5],
                        ],
                        _ => &[[0.5, 0.0], [0.0, 0.5], [-0.5, 0.0], [0.0, -0.5]],
                    };
                    polys.push((
                        owned,
                        corners
                            .iter()
                            .map(|&p| centre + Vec2::from(p) * size)
                            .collect(),
                    ));
                }
                Some(Collision::Full) => {
                    shapes.push(vec![[0.0, 0.0], [tw, 0.0], [tw, th], [0.0, th]])
                }
                Some(Collision::Rect { x, y, w, h }) => {
                    shapes.push(vec![[x, y], [x + w, y], [x + w, y + h], [x, y + h]])
                }
                None => {}
            }
            let diagonal = state.layers[layer].resolved[(y * state.asset.width + x) as usize]
                & FLIP_DIAGONAL
                != 0;
            for shape in shapes {
                let polygon: Vec<Vec2> = shape
                    .into_iter()
                    .map(|[mut rx, mut ry]| {
                        if diagonal {
                            (rx, ry) = (ry * tw / th, rx * th / tw);
                        }
                        if fx {
                            rx = tw - rx;
                        }
                        if fy {
                            ry = th - ry;
                        }
                        let top_left = if ortho {
                            Vec2::new(x as f32 * size.x, -(y as f32 * size.y)) + l.offset
                        } else {
                            centre + Vec2::new(-tw * px * 0.5, -size.y * 0.5 + th * px)
                        };
                        top_left + Vec2::new(rx, -ry) * px
                    })
                    .collect();
                polys.push((owned, polygon));
            }
        }
    }
    polys
}

/// Map collision owned by this chunk. Other chunks' polygons participate
/// in the union but never add edges here. No triangulation round trip.
pub fn chunk_outline(
    state: &MapState,
    layer: usize,
    cx: u32,
    cy: u32,
    default_layers: u32,
) -> MapOutline {
    let l = &state.asset.layers[layer];
    let mut out = MapOutline {
        positions: Vec::new(),
        segments: Vec::new(),
        layers: if l.collider_layers == 0 {
            default_layers
        } else {
            l.collider_layers
        },
        mask: if l.collider_mask == 0 {
            u32::MAX
        } else {
            l.collider_mask
        },
    };
    let polys = chunk_polygons(state, layer, cx, cy);
    let size = state.asset.tile_size();
    let eps = size.min_element() * 1e-5;
    let mut emitted = std::collections::BTreeSet::new();
    for (owned, poly) in &polys {
        for (&a, &b) in poly.iter().zip(poly.iter().cycle().skip(1)) {
            let d = b - a;
            let len = d.length();
            if len <= eps {
                continue;
            }
            let mut cuts = vec![0.0, 1.0];
            for (_, q) in &polys {
                for (&c, &e) in q.iter().zip(q.iter().cycle().skip(1)) {
                    let v = e - c;
                    let den = d.perp_dot(v);
                    if den.abs() > eps * eps {
                        let t = (c - a).perp_dot(v) / den;
                        let u = (c - a).perp_dot(d) / den;
                        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
                            cuts.push(t);
                        }
                    } else if (c - a).perp_dot(d).abs() <= eps * len {
                        for p in [c, e] {
                            let t = (p - a).dot(d) / d.length_squared();
                            if (0.0..=1.0).contains(&t) {
                                cuts.push(t);
                            }
                        }
                    }
                }
            }
            cuts.sort_by(f32::total_cmp);
            cuts.dedup();
            let normal = Vec2::new(-d.y, d.x) * (eps / len);
            for t in cuts.windows(2) {
                let mid = a + d * ((t[0] + t[1]) * 0.5);
                let left = polys.iter().any(|(_, p)| inside(mid + normal, p));
                let right = polys.iter().any(|(_, p)| inside(mid - normal, p));
                if left == right {
                    continue;
                }
                let (p, q) = if right {
                    (a + d * t[0], a + d * t[1])
                } else {
                    (a + d * t[1], a + d * t[0])
                };
                if p.distance_squared(q) <= eps * eps {
                    continue;
                }
                let key = (
                    p.to_array().map(f32::to_bits),
                    q.to_array().map(f32::to_bits),
                );
                if !emitted.insert(key) {
                    continue;
                }
                if !owned {
                    continue;
                }
                let n = out.positions.len() as u32;
                out.positions.extend([p, q]);
                out.segments.push([n, n + 1]);
            }
        }
    }
    out
}
