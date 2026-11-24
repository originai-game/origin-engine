//! Edits are deterministic and a snapshot restores them.

use crate::support;
use og_tilemap::MapState;
use support::*;

fn edits(s: &mut MapState) {
    for i in 0..200u32 {
        let (x, y) = ((i * 37) % 20, (i * 11) % 15);
        if i % 3 == 0 {
            s.set(1, x, y, (i % 2) as i32).unwrap();
        } else {
            s.set(0, x, y, (i % 40) as i32 - 1).unwrap();
        }
    }
}

#[test]
fn equal_edits_give_equal_states_and_bytes_and_decode_restores_them() {
    let a = map(20, 15);
    let (mut s1, mut s2) = (state(&a, "filled"), state(&a, "filled"));
    edits(&mut s1);
    edits(&mut s2);
    let (mut b1, mut b2) = (Vec::new(), Vec::new());
    s1.encode(&mut b1);
    s2.encode(&mut b2);
    assert_eq!(b1, b2);
    let back = MapState::decode(&a, &[tileset("filled")], &b1).unwrap();
    assert_eq!(back, s1);
    for l in 0..2 {
        for y in 0..15 {
            for x in 0..20 {
                assert_eq!(back.tile(l, x, y), s1.tile(l, x, y), "layer {l} ({x}, {y})");
                assert_eq!(back.get(l, x, y), s1.get(l, x, y));
            }
        }
    }
    assert_ne!(back, state(&a, "filled"), "the edits are in the snapshot");
    let e = MapState::decode(&map(19, 15), &[tileset("filled")], &b1).unwrap_err();
    assert!(e.to_string().contains("another size"), "{e}");
}

#[test]
fn collision_unions_keep_slopes_and_remove_chunk_seams() {
    let mut a = map(36, 2);
    a.layers[0].collision = true;
    let mut s = state(&a, "filled");
    // Two adjacent full tiles spanning a chunk boundary.
    s.set(0, 31, 0, 60).unwrap();
    s.set(0, 32, 0, 60).unwrap();
    let lines: Vec<_> = (0..3)
        .flat_map(|cx| {
            let o = og_tilemap::chunk_outline(&s, 0, cx, 0, 1);
            o.segments
                .iter()
                .map(|&[a, b]| (o.positions[a as usize], o.positions[b as usize]))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(
        !lines.iter().any(|(a, b)| a.x == 32.0 && b.x == 32.0),
        "no internal chunk wall: {lines:?}"
    );
    let perimeter: f32 = lines.iter().map(|(a, b)| a.distance(*b)).sum();
    assert!((perimeter - 6.0).abs() < 1e-5);
    // An authored triangle remains a slope, not the tile's AABB.
    let mut ts = (*tileset("filled")).clone();
    ts.tiles.get_mut(&62).unwrap().collision = None;
    ts.tiles.get_mut(&62).unwrap().polygons = vec![vec![[0.0, 16.0], [16.0, 16.0], [16.0, 0.0]]];
    let text = ts.to_text();
    let ts = std::sync::Arc::new(og_tilemap::Tileset::parse(&text).unwrap());
    let mut slope = MapState::new(&a, &[ts]).unwrap();
    slope.set(0, 2, 0, 62).unwrap();
    let o = og_tilemap::chunk_outline(&slope, 0, 0, 0, 1);
    assert!(o.segments.iter().any(|&[a, b]| {
        let d = o.positions[a as usize] - o.positions[b as usize];
        d.x != 0.0 && d.y != 0.0
    }));
}
