//! Terrain layers resolve every filled cell through its tileset's rules:
//! blob47 (eight neighbours, corners only between filled edges) and
//! bitmask16 (four edges); edits re-resolve the 3×3 neighbourhood.

use crate::support;
use og_tilemap::mask::*;
use support::*;

const WALLS: usize = 1;

#[test]
fn an_isolated_cell_and_a_3x3_block_resolve_to_their_neighbourhoods() {
    let mut s = state(&map(7, 7), "empty");
    s.set(WALLS, 1, 1, 1).unwrap();
    assert_eq!(
        s.tile(WALLS, 1, 1),
        Some((blob_tile(0), false, false)),
        "isolated: mask 0"
    );
    s.set(WALLS, 1, 1, 0).unwrap();
    for y in 2..5 {
        for x in 2..5 {
            s.set(WALLS, x, y, 1).unwrap();
        }
    }
    let t = |x, y| s.tile(WALLS, x, y).map(|t| t.0);
    assert_eq!(t(3, 3), Some(blob_tile(255)), "centre: every neighbour");
    assert_eq!(t(2, 2), Some(blob_tile(E | SE | S)), "top-left corner");
    assert_eq!(t(4, 2), Some(blob_tile(W | SW | S)), "top-right corner");
    assert_eq!(t(2, 4), Some(blob_tile(N | NE | E)), "bottom-left corner");
    assert_eq!(t(4, 4), Some(blob_tile(N | NW | W)), "bottom-right corner");
    assert_eq!(t(3, 2), Some(blob_tile(E | SE | S | SW | W)), "top edge");
    assert_eq!(t(2, 3), Some(blob_tile(N | NE | E | SE | S)), "left edge");
    assert_eq!(t(3, 4), Some(blob_tile(W | NW | N | NE | E)), "bottom edge");
    assert_eq!(t(4, 3), Some(blob_tile(N | S | SW | W | NW)), "right edge");
    assert_eq!(t(1, 1), None, "empty cells draw nothing");
    assert_eq!(
        s.get(WALLS, 3, 3),
        blob_tile(255) as i32,
        "getTile answers the resolved tile"
    );
    assert_eq!(s.get(WALLS, 1, 1), -1, "an empty terrain cell has no tile");
}

#[test]
fn a_fill_then_clear_restores_the_neighbourhood_exactly() {
    let mut s = state(&map(7, 7), "filled");
    for y in 1..6 {
        for x in 1..6 {
            s.set(WALLS, x, y, 1).unwrap();
        }
    }
    s.set(WALLS, 3, 3, 0).unwrap();
    let before: Vec<_> = (0..49).map(|i| s.tile(WALLS, i % 7, i / 7)).collect();
    let fresh_version = s.layer_version(WALLS);
    s.set(WALLS, 3, 3, 1).unwrap();
    assert_eq!(
        s.tile(WALLS, 3, 2).unwrap().0,
        blob_tile(255),
        "the hole's neighbour closes up"
    );
    s.set(WALLS, 3, 3, 0).unwrap();
    let after: Vec<_> = (0..49).map(|i| s.tile(WALLS, i % 7, i / 7)).collect();
    assert_eq!(before, after);
    assert!(
        s.layer_version(WALLS) > fresh_version,
        "edits bump the layer version"
    );
}
