//! Origin Engine tilemaps (`README.md`): tilesets with collision, animated
//! tiles and auto-tiling terrains ([`tileset`]); layered maps, from
//! hand-written `.ogt` or the cook's Tiled and LDtk importers ([`map`]);
//! and a placed map's state ([`state`]): deterministic edits that
//! re-resolve terrain neighbourhoods, snapshots of what changed, collision
//! rectangles merged into runs and chunked quads for the sprite batch.
//!
//! Pure Rust over og-core and glam; builds for `wasm32-unknown-unknown`.
//! Cells are (x right, y down) with row 0 at the top; world coordinates are
//! map-local with y up from the map's top-left corner.

pub mod map;
pub mod state;
pub mod tileset;

pub use map::{
    FLIP_DIAGONAL, FLIP_X, FLIP_Y, Layer, MapObject, TILE_MASK, TilemapAsset, TilesetRef,
    cell_center, world_to_cell,
};
pub use state::{
    AnimatedQuad, CHUNK, ChunkQuads, MapState, Rect, TileQuad, animated_frame, chunk_quads,
    chunks_in, collision_rects, collision_rects_all,
};
pub use tileset::{
    Collision, Filter, Terrain, TerrainMode, TileAnim, TileDef, Tileset, mask, reduce_blob,
};
