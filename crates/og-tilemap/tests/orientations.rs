//! Isometric, staggered and hex maps (G2-6): where cells are, which cell a
//! point is in, which cells are next to each other, how they draw and
//! collide, and resizing at run time.
//!
//! Ways this could fail: a picking formula off by one row on the odd rows
//! or past a slanted edge; neighbours that are not mutual or do not touch;
//! a resize that loses edits, re-staggers a hex map or leaves a chunk
//! cached; a collider shape the size of the box instead of the cell.

use crate::support;

use og_tilemap::{
    Layer, MapObject, MapState, Orientation, TilemapAsset, TilesetRef, cell_center, map_colliders,
    map_regions, neighbours,
};
use support::*;

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

/// A side by its two corners' coordinates (bits), lower first.
type Side = ((u32, u32), (u32, u32));

#[test]
fn solid_regions_merge_into_one_collider_each_their_shared_sides_cancelling() {
    // A ring of solid cells around an empty one, and two lone cells: two
    // ring regions would mean the sides did not connect, a ring with as
    // many outline edges as its cells have sides would mean the shared
    // corners did not land on the same coordinates.
    for (o, ring, sides, area) in [
        // The 8 cells around (3, 3): 12 outer sides and the hole's 4.
        (
            Orientation::Isometric,
            vec![
                (2, 2),
                (3, 2),
                (4, 2),
                (2, 3),
                (4, 3),
                (2, 4),
                (3, 4),
                (4, 4),
            ],
            16,
            0.5,
        ),
        // The 6 neighbours of (3, 3): 18 outer sides and the hole's 6.
        (Orientation::Hexagonal { flat: false }, vec![], 24, 0.75),
        (Orientation::Hexagonal { flat: true }, vec![], 24, 0.75),
    ] {
        let m = flat_map(o, 9, 9);
        let mut s = MapState::new(&m, &[tileset("filled")]).unwrap();
        let ring = if ring.is_empty() {
            neighbours(&m, 3, 3)
        } else {
            ring
        };
        for &(x, y) in &ring {
            s.set(0, x, y, 60).unwrap();
        }
        s.set(0, 7, 7, 60).unwrap();
        s.set(0, 6, 0, 60).unwrap();
        let r = map_regions(&s, 1);
        assert_eq!(
            r.iter().map(|r| r.cells).collect::<Vec<_>>(),
            vec![1, ring.len() as u32, 1],
            "{o:?}: regions in row-major order of their first cell"
        );
        assert!(map_colliders(&s, 1).is_empty(), "{o:?}: no per-cell shapes");
        let ring_region = &r[1];
        // Edges no other triangle shares, by exact coordinates.
        let mut count: std::collections::BTreeMap<Side, i32> = Default::default();
        let mut total = 0.0;
        for t in &ring_region.triangles {
            let p = t.map(|i| ring_region.positions[i as usize]);
            total += (p[1] - p[0]).perp_dot(p[2] - p[0]) * 0.5;
            assert!(
                (p[1] - p[0]).perp_dot(p[2] - p[0]) > 0.0,
                "{o:?}: counter-clockwise"
            );
            for k in 0..3 {
                let (a, b) = (p[k], p[(k + 1) % 3]);
                let (ka, kb) = (
                    (a.x.to_bits(), a.y.to_bits()),
                    (b.x.to_bits(), b.y.to_bits()),
                );
                *count
                    .entry(if ka < kb { (ka, kb) } else { (kb, ka) })
                    .or_default() += 1;
            }
        }
        // A cell's fan has (corners - 3) inner diagonals, each counted twice.
        let outline = count.values().filter(|&&n| n == 1).count();
        assert_eq!(outline, sides, "{o:?}");
        let t = m.tile_size();
        assert!(
            (total - ring.len() as f32 * area * t.x * t.y).abs() < 1e-3,
            "{o:?}: {total}"
        );
        // Its origin and positions put the cells where they are drawn.
        let first = ring_region.triangles[0]
            .map(|i| ring_region.positions[i as usize] + ring_region.origin);
        let c = (first[0] + first[1] + first[2]) / 3.0;
        assert!(
            ring.iter()
                .any(|&(x, y)| (cell_center(&m, x, y) - c).length() < t.x * 0.5),
            "{o:?}: {c}"
        );
        // Two lone cells are the same mesh at two origins.
        assert_eq!(
            (&r[0].positions, &r[0].triangles),
            (&r[2].positions, &r[2].triangles),
            "{o:?}"
        );
        assert_ne!(r[0].origin, r[2].origin);
    }
}
