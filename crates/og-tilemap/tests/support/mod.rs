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

pub fn tileset(edge: &str) -> Arc<Tileset> {
    use og_tilemap::{Collision, Filter, Terrain, TerrainMode, TileAnim, TileDef};
    Arc::new(Tileset {
        texture: "tiles/t.png".into(),
        tile_width: 16,
        tile_height: 16,
        columns: 8,
        rows: 8,
        margin: 0,
        spacing: 0,
        filter: Filter::Nearest,
        tiles: [
            (
                60,
                TileDef {
                    collision: Some(Collision::Full),
                    ..Default::default()
                },
            ),
            (
                61,
                TileDef {
                    collision: Some(Collision::Rect {
                        x: 0.0,
                        y: 8.0,
                        w: 16.0,
                        h: 8.0,
                    }),
                    ..Default::default()
                },
            ),
            (
                62,
                TileDef {
                    collision: Some(Collision::Rect {
                        x: 2.0,
                        y: 0.0,
                        w: 4.0,
                        h: 4.0,
                    }),
                    ..Default::default()
                },
            ),
            (
                50,
                TileDef {
                    anim: Some(TileAnim {
                        frames: vec![50, 51, 52],
                        durations: vec![0.25; 3],
                    }),
                    ..Default::default()
                },
            ),
        ]
        .into_iter()
        .collect(),
        terrains: vec![
            Terrain {
                name: "wall".into(),
                mode: TerrainMode::Blob47,
                rules: blob_masks()
                    .into_iter()
                    .enumerate()
                    .map(|(i, m)| (m, i as u32))
                    .collect(),
                default: 47,
                edge_filled: edge == "filled",
            },
            Terrain {
                name: "path".into(),
                mode: TerrainMode::Bitmask16,
                rules: (0..16).map(|m| (m, 48 + u32::from(m))).collect(),
                default: 63,
                edge_filled: true,
            },
        ],
    })
}

/// A `w × h` map with one empty tile layer and one empty terrain layer `wall`.
pub fn map(w: u32, h: u32) -> TilemapAsset {
    let mut terrain = Layer::new("walls", w, h, 1);
    terrain.terrain = Some("wall".into());
    TilemapAsset {
        orientation: Default::default(),
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
