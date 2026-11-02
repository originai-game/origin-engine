//! Isometric, staggered and hex maps (G2-6): where cells are, which cell a
//! point is in, which cells are next to each other, how they draw and
//! collide, and resizing at run time.
//!
//! Ways this could fail: a picking formula off by one row on the odd rows
//! or past a slanted edge; neighbours that are not mutual or do not touch;
//! a resize that loses edits, re-staggers a hex map or leaves a chunk
//! cached; a collider shape the size of the box instead of the cell.

use crate::support;

use glam::Vec2;
use og_tilemap::{
    Layer, MapObject, MapShape, MapState, Orientation, TilemapAsset, TilesetRef, cell_center,
    chunk_quads, map_colliders, neighbours, world_to_cell,
};
use support::*;

const ALL: [Orientation; 5] = [
    Orientation::Orthogonal,
    Orientation::Isometric,
    Orientation::Staggered,
    Orientation::Hexagonal { flat: false },
    Orientation::Hexagonal { flat: true },
];

/// A `w × h` map of one tile layer, tiles 32 × 16 px (a 2 × 1 world-unit
/// cell, the usual isometric proportion).
fn flat_map(o: Orientation, w: u32, h: u32) -> TilemapAsset {
    let mut floor = Layer::new("floor", w, h, 0);
    floor.collision = true;
    TilemapAsset {
        orientation: o,
        width: w,
        height: h,
        tile_width_px: 32,
        tile_height_px: 16,
        unit: 2.0,
        tilesets: vec![TilesetRef::Asset("tiles/t.ogt".into())],
        layers: vec![floor],
        objects: Vec::new(),
    }
}

/// Whether `p` is inside cell (x, y)'s shape, from the shape's corners
/// (independent of the engine's gauge): the cell's convex polygon.
fn polygon(o: Orientation, c: Vec2, t: Vec2) -> Vec<Vec2> {
    let (hw, hh) = (t.x * 0.5, t.y * 0.5);
    let pts: Vec<(f32, f32)> = match o {
        Orientation::Orthogonal => vec![(hw, hh), (-hw, hh), (-hw, -hh), (hw, -hh)],
        Orientation::Isometric | Orientation::Staggered => {
            vec![(hw, 0.0), (0.0, hh), (-hw, 0.0), (0.0, -hh)]
        }
        Orientation::Hexagonal { flat: false } => vec![
            (0.0, hh),
            (-hw, hh * 0.5),
            (-hw, -hh * 0.5),
            (0.0, -hh),
            (hw, -hh * 0.5),
            (hw, hh * 0.5),
        ],
        Orientation::Hexagonal { flat: true } => vec![
            (hw, 0.0),
            (hw * 0.5, hh),
            (-hw * 0.5, hh),
            (-hw, 0.0),
            (-hw * 0.5, -hh),
            (hw * 0.5, -hh),
        ],
    };
    pts.into_iter().map(|(x, y)| c + Vec2::new(x, y)).collect()
}

/// The least signed distance from `p` to the polygon's edges (positive
/// inside, counter-clockwise corners).
fn inside_by(poly: &[Vec2], p: Vec2) -> f32 {
    (0..poly.len())
        .map(|i| {
            let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
            let e = (b - a).normalize();
            e.perp_dot(p - a)
        })
        .fold(f32::MAX, f32::min)
}

#[test]
fn every_point_picks_the_cell_whose_shape_holds_it_and_centres_pick_their_cell() {
    for o in ALL {
        let m = flat_map(o, 7, 6);
        let t = m.tile_size();
        for y in 0..6 {
            for x in 0..7 {
                assert_eq!(
                    world_to_cell(&m, cell_center(&m, x, y)),
                    Some((x, y)),
                    "{o:?}"
                );
            }
        }
        // A fine grid over the box around the map and a margin.
        let size = m.world_size();
        let mut inside = 0;
        for j in -10..=130 {
            for i in -10..=130 {
                let p = Vec2::new(size.x * i as f32 / 120.0, -size.y * j as f32 / 120.0);
                let holder = (0..6u32)
                    .flat_map(|y| (0..7u32).map(move |x| (x, y)))
                    .map(|(x, y)| ((x, y), inside_by(&polygon(o, cell_center(&m, x, y), t), p)))
                    .fold(((0, 0), f32::MIN), |a, b| if b.1 > a.1 { b } else { a });
                let picked = world_to_cell(&m, p);
                if holder.1 > 1e-4 {
                    inside += 1;
                    assert_eq!(picked, Some(holder.0), "{o:?} at {p}");
                } else if holder.1 < -1e-4 {
                    assert_eq!(picked, None, "{o:?} at {p} is outside every cell");
                }
            }
        }
        assert!(inside > 5000, "{o:?}: {inside}");
    }
}

#[test]
fn cell_centres_follow_tiled_layouts() {
    let t = |o| flat_map(o, 4, 3);
    // Isometric: cell (x, y)'s top corner at ((x - y + height) w/2, -(x + y) h/2).
    assert_eq!(
        cell_center(&t(Orientation::Isometric), 2, 1),
        Vec2::new(4.0, -2.0)
    );
    // Staggered: odd rows half a tile right, rows h/2 apart.
    assert_eq!(
        cell_center(&t(Orientation::Staggered), 1, 1),
        Vec2::new(4.0, -1.0)
    );
    assert_eq!(
        cell_center(&t(Orientation::Staggered), 1, 2),
        Vec2::new(3.0, -1.5)
    );
    // Pointy hex: rows 3h/4 apart, odd rows half a tile right.
    assert_eq!(
        cell_center(&t(Orientation::Hexagonal { flat: false }), 0, 1),
        Vec2::new(2.0, -1.25)
    );
    // Flat hex: columns 3w/4 apart, odd columns half a tile down.
    assert_eq!(
        cell_center(&t(Orientation::Hexagonal { flat: true }), 1, 0),
        Vec2::new(2.5, -1.0)
    );
    assert_eq!(t(Orientation::Isometric).world_size(), Vec2::new(7.0, 3.5));
    assert_eq!(
        t(Orientation::Hexagonal { flat: false }).world_size(),
        Vec2::new(9.0, 2.5)
    );
}

#[test]
fn neighbours_are_mutual_and_share_an_edge() {
    for o in ALL {
        let m = flat_map(o, 6, 5);
        let t = m.tile_size();
        let want = match o {
            Orientation::Hexagonal { .. } => 6,
            _ => 4,
        };
        for y in 0..5 {
            for x in 0..6 {
                let n = neighbours(&m, x, y);
                if (1..5).contains(&x) && (1..4).contains(&y) {
                    assert_eq!(n.len(), want, "{o:?} ({x}, {y})");
                }
                for &(nx, ny) in &n {
                    assert!(
                        neighbours(&m, nx, ny).contains(&(x, y)),
                        "{o:?} ({x}, {y}) - ({nx}, {ny})"
                    );
                    if o == Orientation::Orthogonal || o == Orientation::Isometric {
                        continue;
                    }
                    // The midpoint of the centres is on both outlines.
                    let mid = (cell_center(&m, x, y) + cell_center(&m, nx, ny)) * 0.5;
                    for c in [(x, y), (nx, ny)] {
                        let d = inside_by(&polygon(o, cell_center(&m, c.0, c.1), t), mid);
                        assert!(d.abs() < 1e-4, "{o:?} ({x}, {y}) - ({nx}, {ny}): {d}");
                    }
                }
            }
        }
    }
}

#[test]
fn hex_and_iso_solids_collide_as_their_cell_shapes() {
    for (o, shape) in [
        (Orientation::Isometric, MapShape::Diamond),
        (
            Orientation::Hexagonal { flat: false },
            MapShape::Hexagon { flat: false },
        ),
        (
            Orientation::Hexagonal { flat: true },
            MapShape::Hexagon { flat: true },
        ),
    ] {
        let m = flat_map(o, 5, 4);
        let mut s = MapState::new(&m, &[tileset("filled")]).unwrap();
        s.set(0, 1, 1, 60).unwrap();
        s.set(0, 2, 1, 60).unwrap();
        s.set(0, 3, 2, 61).unwrap();
        let c = map_colliders(&s, 1);
        assert_eq!(
            c.len(),
            3,
            "{o:?}: one collider per solid cell, never merged"
        );
        assert_eq!(
            (c[0].shape, c[1].shape, c[2].shape),
            (shape, shape, MapShape::Box)
        );
        let mid = (c[0].rect.min + c[0].rect.max) * 0.5;
        assert_eq!(mid, cell_center(&m, 1, 1));
        assert_eq!(c[0].rect.max - c[0].rect.min, m.tile_size());
        // Tile 61's rect (0, 8, 16, 8 px of a 16 px tile) is the lower half
        // of its image, which rests on the cell's bottom.
        let b = cell_center(&m, 3, 2) - m.tile_size() * 0.5;
        assert_eq!(c[2].rect.min.y, b.y, "{o:?}");
    }
}

#[test]
fn an_isometric_tile_taller_than_its_cell_stands_on_the_cell_bottom() {
    let mut m = flat_map(Orientation::Isometric, 3, 3);
    m.tile_height_px = 8; // a 32 × 8 px diamond; the tileset's tiles are 16 × 16
    let mut s = MapState::new(&m, &[tileset("filled")]).unwrap();
    s.set(0, 1, 1, 3).unwrap();
    let q = chunk_quads(&s, 0, 0, 0, 128, 128).quads;
    assert_eq!(q.len(), 1);
    let c = cell_center(&m, 1, 1);
    // The image is 16 × 16 px at 2/32 units per px: 1 × 1, centred on the
    // cell, its bottom on the diamond's bottom corner.
    assert_eq!(q[0].size, Vec2::new(1.0, 1.0));
    assert_eq!(q[0].min, Vec2::new(c.x - 0.5, c.y - 0.25));
}

#[test]
fn a_resize_moves_cells_and_edits_and_redraws_every_chunk() {
    let mut a = map(20, 18);
    a.layers[0].cells[0] = 6; // tile 5 at (0, 0) in the asset
    a.objects.push(MapObject {
        name: "door".into(),
        kind: String::new(),
        x: 1.0,
        y: -1.0,
        width: 1.0,
        height: 1.0,
        properties: Vec::new(),
    });
    let mut s = state(&a, "filled");
    s.set(0, 3, 4, 9).unwrap();
    s.set(1, 5, 5, 1).unwrap();
    s.set(1, 6, 5, 1).unwrap();
    let before: Vec<u64> = (0..2).map(|l| s.chunk_version(l, 0, 0)).collect();
    let b = a.resized(40, 10, 2, -1).unwrap();
    assert_eq!((b.width, b.height), (40, 10));
    assert_eq!(b.objects[0].x, 3.0, "objects move with the cells");
    assert_eq!(b.objects[0].y, 0.0);
    let r = s.resize(&b, 2, -1).unwrap();
    assert_eq!(r.get(0, 5, 3), 9, "the edit moved by (2, -1)");
    assert_eq!(r.get(0, 2, 0), -1, "row 0 of the old map is cropped");
    assert_eq!(
        r.get(1, 7, 4),
        s.get(1, 5, 5),
        "a terrain cell re-tiled the same"
    );
    assert_eq!(r.get(1, 8, 4), s.get(1, 6, 5));
    for (l, &old) in before.iter().enumerate() {
        for cx in 0..3 {
            assert!(r.chunk_version(l, cx, 0) > old, "layer {l} chunk ({cx}, 0)");
        }
    }
    // The resized state saves against the resized asset and restores.
    let mut bytes = Vec::new();
    r.encode(&mut bytes);
    assert_eq!(
        MapState::decode(&b, &[tileset("filled")], &bytes).unwrap(),
        r
    );
    // A hex map keeps its stagger: an odd shift along it is refused.
    let mut h = flat_map(Orientation::Hexagonal { flat: false }, 4, 4);
    h.layers[0].collision = false;
    let e = h.resized(5, 5, 1, 1).unwrap_err();
    assert!(e.to_string().contains("even y"), "{e}");
    assert!(h.resized(5, 6, 1, 2).is_ok());
    let e = a.resized(0, 3, 0, 0).unwrap_err();
    assert!(e.to_string().contains("0 x 3"), "{e}");
}

#[test]
fn a_map_saved_whole_reads_back_equal_and_hex_terrain_is_refused() {
    let mut a = map(6, 5);
    a.orientation = Orientation::Isometric;
    a.layers[0].cells[7] = 3 | og_tilemap::FLIP_X;
    a.layers[0].offset = Vec2::new(0.5, -0.25);
    a.tilesets
        .push(TilesetRef::Embedded((*tileset("empty")).clone()));
    a.objects.push(MapObject {
        name: "spawn".into(),
        kind: "player".into(),
        x: 2.0,
        y: -3.0,
        width: 0.0,
        height: 0.0,
        properties: vec![("facing".into(), "down".into())],
    });
    let mut bytes = Vec::new();
    a.encode(&mut bytes);
    assert_eq!(TilemapAsset::decode(&bytes).unwrap(), a);
    assert!(TilemapAsset::decode(&bytes[..bytes.len() - 1]).is_err());
    a.orientation = Orientation::Hexagonal { flat: false };
    let e = a.validate().unwrap_err();
    assert!(e.to_string().contains("terrain layer"), "{e}");
}
