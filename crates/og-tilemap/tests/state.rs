//! Edits are deterministic and a snapshot holds only what changed.

mod support;
use og_tilemap::{Layer, MapState, TilemapAsset, TilesetRef};
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
fn an_untouched_256x256_four_layer_map_snapshots_in_a_few_bytes() {
    let mut a = TilemapAsset {
        width: 256,
        height: 256,
        tile_width_px: 16,
        tile_height_px: 16,
        unit: 1.0,
        tilesets: vec![TilesetRef::Asset("tiles/t.ogt".into())],
        layers: (0..4)
            .map(|i| Layer::new(format!("l{i}"), 256, 256, i))
            .collect(),
        objects: Vec::new(),
    };
    for l in &mut a.layers {
        for (i, c) in l.cells.iter_mut().enumerate() {
            *c = (i % 40) as u32 + 1;
        }
    }
    let mut s = MapState::new(&a, &[tileset("filled")]).unwrap();
    let mut b = Vec::new();
    s.encode(&mut b);
    assert_eq!(b.len(), 16);
    s.set(2, 100, 200, -1).unwrap();
    b.clear();
    s.encode(&mut b);
    assert_eq!(b.len(), 28, "one edit is one triple");
    s.set(2, 100, 200, (200 * 256 + 100) % 40).unwrap();
    b.clear();
    s.encode(&mut b);
    assert_eq!(
        b.len(),
        16,
        "setting a cell back to the asset's removes the edit"
    );
}

#[test]
fn tile_layer_edits_are_checked_and_bump_only_their_chunk() {
    let mut s = state(&map(40, 20), "filled");
    assert_eq!(s.get(0, 3, 3), -1);
    s.set(0, 3, 3, 5).unwrap();
    assert_eq!(
        (s.get(0, 3, 3), s.tile(0, 3, 3)),
        (5, Some((5, false, false)))
    );
    assert_eq!((s.chunk_version(0, 0, 0), s.chunk_version(0, 1, 0)), (1, 0));
    s.set(0, 20, 3, 5).unwrap();
    assert_eq!(s.chunk_version(0, 1, 0), 1);
    assert!(
        s.set(0, 40, 0, 1)
            .unwrap_err()
            .to_string()
            .contains("outside")
    );
    assert!(
        s.set(0, 0, 0, 64)
            .unwrap_err()
            .to_string()
            .contains("not in layer 'floor'")
    );
    assert!(s.set(7, 0, 0, 1).is_err());
}
