//! Tileset and map author data, before references and grid sizes are resolved.
use super::clips::Filter;
use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Tileset {
    #[schemars(extend("x-og-ref-kinds" = ["texture"]))]
    pub texture: Option<ContentRef>,
    #[schemars(range(min = 1), extend("x-og-unit" = "px"))]
    pub tile_width: u32,
    #[schemars(range(min = 1), extend("x-og-unit" = "px"))]
    pub tile_height: u32,
    #[schemars(range(min = 1))]
    pub columns: u32,
    #[schemars(range(min = 1))]
    pub rows: u32,
    #[serde(default)]
    pub margin: u32,
    #[serde(default)]
    pub spacing: u32,
    pub filter: Option<Filter>,
    /// Per-tileset art conformance; omission uses the project's art contract.
    pub conform: Option<TilesetConform>,
    #[serde(default)]
    pub tiles: BTreeMap<u32, Tile>,
    #[serde(default)]
    pub terrains: Vec<Terrain>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum TilesetConform {
    Off,
    Size {
        #[schemars(range(min = 1, max = 4096))]
        size: u32,
    },
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Tile {
    pub collision: Option<Collision>,
    /// Closed polygons, top-left pixel coordinates, y down.
    #[serde(default)]
    #[schemars(inner(length(min = 3)))]
    pub polygons: Vec<Vec<[f32; 2]>>,
    pub animation: Option<Animation>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Collision {
    Full,
    Rect {
        x: f32,
        y: f32,
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "px"))]
        w: f32,
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "px"))]
        h: f32,
    },
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Animation {
    #[schemars(length(min = 1))]
    pub frames: Vec<u32>,
    /// One strictly positive duration per frame; domain checks matching counts.
    #[schemars(length(min = 1), extend("items" = {"type": "number", "format": "float", "exclusiveMinimum": 0}, "x-og-unit" = "s"))]
    pub durations: Vec<f32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Terrain {
    pub name: String,
    pub mode: TerrainMode,
    /// Neighbor mask to tile id. Domain checks reachable masks and tile bounds.
    pub rules: BTreeMap<u8, u32>,
    pub default: u32,
    pub edge_filled: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase")]
pub enum TerrainMode {
    Blob47,
    Bitmask16,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Tilemap {
    pub orientation: Option<Orientation>,
    #[schemars(range(min = 1))]
    pub width: u32,
    #[schemars(range(min = 1))]
    pub height: u32,
    #[schemars(range(min = 1), extend("x-og-unit" = "px"))]
    pub tile_width: u32,
    #[schemars(range(min = 1), extend("x-og-unit" = "px"))]
    pub tile_height: u32,
    #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "m/tile"))]
    pub unit: Option<f32>,
    #[schemars(extend("x-og-ref-kinds" = ["tileset"]))]
    pub tilesets: Vec<ContentRef>,
    pub layers: Vec<Layer>,
    #[serde(default)]
    pub objects: Vec<Object>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase")]
pub enum Orientation {
    Orthogonal,
    Isometric,
    Staggered,
    Hex,
    HexFlat,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Layer {
    pub name: String,
    /// Index into the map's declared tilesets, checked by the domain.
    pub tileset: u32,
    /// Row-major, top row first: 0 empty; otherwise tile+1 with flip bits.
    pub cells: Vec<u32>,
    pub terrain: Option<String>,
    pub parallax: Option<[f32; 2]>,
    #[schemars(extend("x-og-unit" = "m"))]
    pub offset: Option<[f32; 2]>,
    #[schemars(range(min = 0, max = 1))]
    pub opacity: Option<f32>,
    pub visible: Option<bool>,
    pub collision: Option<bool>,
    pub collider_layers: Option<u32>,
    pub collider_mask: Option<u32>,
    pub cell_colliders: Option<bool>,
    pub repeat_x: Option<bool>,
    pub repeat_y: Option<bool>,
    pub z: Option<i32>,
    pub lit: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Object {
    pub name: String,
    pub kind: String,
    /// Map-local world units, y up from the map's top-left.
    pub x: f32,
    pub y: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

schema!(Tileset, Tilemap);
