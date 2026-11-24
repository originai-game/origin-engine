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
