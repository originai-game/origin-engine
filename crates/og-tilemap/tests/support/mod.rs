//! Shared fixtures: a blob47 tileset whose rule for reduced mask m is the
//! mask's rank among the 47 (so a tile id names its neighbourhood), and maps.
#![allow(dead_code)]

use og_tilemap::{Layer, MapState, TilemapAsset, Tileset, TilesetRef, reduce_blob};
use std::sync::Arc;

/// The 47 reduced blob masks, ascending.
pub fn blob_masks() -> Vec<u8> {
    let mut m: Vec<u8> = (0..=255u8).filter(|&m| reduce_blob(m) == m).collect();
    m.dedup();
    m
}

/// Tile id of reduced mask `m` in [`blob_tileset`].
pub fn blob_tile(m: u8) -> u32 {
    blob_masks()
        .iter()
        .position(|&x| x == m)
        .expect("a reduced mask") as u32
}

pub fn blob_tileset(edge: &str) -> String {
    let mut t = String::from(
        "type tileset\nversion 1\n---\ntexture \"tiles/t.png\"\ngrid 16 16 8 8\nsolid 60\ntile 61 {\n  collide 0 8 16 8\n}\ntile 62 {\n  collide 2 0 4 4\n}\ntile 50 {\n  frames 50 51 52\n  fps 4\n}\nterrain wall {\n  mode blob47\n",
    );
    for (i, m) in blob_masks().into_iter().enumerate() {
        t.push_str(&format!("  rule {m} {i}\n"));
    }
    t.push_str(&format!("  default 47\n  edge {edge}\n}}\n"));
    t.push_str("terrain path {\n  mode bitmask16\n");
    for m in 0..16 {
        t.push_str(&format!("  rule {m} {}\n", 48 + m));
    }
    t.push_str("  default 63\n}\n");
    t
}

pub fn tileset(edge: &str) -> Arc<Tileset> {
    Arc::new(Tileset::parse(&blob_tileset(edge)).expect("the fixture tileset parses"))
}

/// A `w × h` map with one empty tile layer and one empty terrain layer `wall`.
pub fn map(w: u32, h: u32) -> TilemapAsset {
    let mut terrain = Layer::new("walls", w, h, 1);
    terrain.terrain = Some("wall".into());
    TilemapAsset {
        width: w,
        height: h,
        tile_width_px: 16,
        tile_height_px: 16,
        unit: 1.0,
        tilesets: vec![TilesetRef::Asset("tiles/t.ogt".into())],
        layers: vec![Layer::new("floor", w, h, 0), terrain],
        objects: Vec::new(),
    }
}

pub fn state(asset: &TilemapAsset, edge: &str) -> MapState {
    MapState::new(asset, &[tileset(edge)]).expect("the map is valid")
}
