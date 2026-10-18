//! Terrain layers resolve every filled cell through its tileset's rules:
//! blob47 (eight neighbours, corners only between filled edges) and
//! bitmask16 (four edges); edits re-resolve the 3×3 neighbourhood.

mod support;
use og_tilemap::mask::*;
use og_tilemap::{reduce_blob, MapState, Tileset};
use support::*;

const WALLS: usize = 1;

#[test]
fn there_are_47_reduced_blob_masks_and_reduction_drops_lone_corners() {
    assert_eq!(blob_masks().len(), 47);
    assert_eq!(reduce_blob(NE), 0, "a corner without its edges is dropped");
    assert_eq!(reduce_blob(N | NE), N);
    assert_eq!(reduce_blob(N | NE | E), N | NE | E);
    assert_eq!(reduce_blob(255), 255);
}

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
    assert_eq!(s.get(WALLS, 3, 3), 1);
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

#[test]
fn the_map_edge_counts_as_filled_unless_the_terrain_says_empty() {
    let mut filled = state(&map(3, 3), "filled");
    let mut empty = state(&map(3, 3), "empty");
    for s in [&mut filled, &mut empty] {
        s.set(WALLS, 0, 0, 1).unwrap();
    }
    assert_eq!(filled.tile(WALLS, 0, 0).unwrap().0, blob_tile(N | NW | W));
    assert_eq!(empty.tile(WALLS, 0, 0).unwrap().0, blob_tile(0));
}

#[test]
fn bitmask16_reads_the_four_edges() {
    let mut a = map(4, 4);
    a.layers[1].terrain = Some("path".into());
    let mut s = MapState::new(&a, &[tileset("empty")]).unwrap();
    s.set(WALLS, 1, 1, 1).unwrap();
    s.set(WALLS, 2, 1, 1).unwrap();
    s.set(WALLS, 1, 2, 1).unwrap();
    // (1,1): E=2 and S=4 -> mask 6 -> tile 48 + 6.
    assert_eq!(s.tile(WALLS, 1, 1).unwrap().0, 54);
    assert_eq!(s.tile(WALLS, 2, 1).unwrap().0, 48 + 8, "only W");
    assert_eq!(s.tile(WALLS, 1, 2).unwrap().0, 48 + 1, "only N");
}

#[test]
fn rules_must_name_reduced_masks_and_terrain_cells_are_0_or_1() {
    let text = blob_tileset("filled").replace("  default 47", "  rule 2 3\n  default 47");
    let e = Tileset::parse(&text).unwrap_err();
    assert!(
        e.to_string()
            .contains("mask 2 is not a reduced blob47 mask"),
        "{e}"
    );
    assert!(e.to_string().contains("line"), "{e}");
    let mut s = state(&map(3, 3), "filled");
    let e = s.set(WALLS, 0, 0, 5).unwrap_err();
    assert!(e.to_string().contains("terrain layer"), "{e}");
    let mut a = map(3, 3);
    a.layers[1].terrain = Some("lava".into());
    let e = MapState::new(&a, &[tileset("filled")]).unwrap_err();
    assert!(e.to_string().contains("terrain 'lava'"), "{e}");
}
