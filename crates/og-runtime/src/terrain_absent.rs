//! The IDL's `terrain` module in the 2D profile (og-runtime's `profile2d`:
//! the 2D web modules): not built. og-render draws no 3D world there
//! (`RenderCapabilities.world3d` is false), so the terrain and its tile
//! streaming (`terrain.rs`, `terrain_stream.rs`) are left out with it, and
//! this module stands in for them: every `og.terrain` call answers
//! [`ABSENT`], a SHAPE_HEIGHTFIELD collider is refused with it (no height
//! field is ever registered with physics), og.Terrain and og.TerrainScatter
//! draw nothing and leave sprites' drop shadows on their own ground, and
//! `terrain_stream` refuses. `og build --publish` refuses a 2D project
//! whose scenes hold a terrain (og-cli's `publish::refuse_3d_world`).

use super::extract::{Hierarchy, RenderState};
use og_ecs::World;
use serde_json::Value;

/// Why the terrain answers nothing here.
pub(crate) const ABSENT: &str = "terrain is not built into this module: the 2D profile's publish and lanes modules leave the 3D world out (RenderCapabilities.world3d is false); a terrain needs a 3D project (project.ogt: profile \"3d\")";

/// A loaded terrain: none is ever loaded here. What other modules read of
/// one (the texture arrays its layers sample) is empty.
pub(crate) struct LoadedTerrain {
    pub layer_arrays: [Option<usize>; 0],
}

/// The render state's terrains: always none.
#[derive(Default)]
pub(crate) struct TerrainState {
    pub loaded: Vec<LoadedTerrain>,
    /// Read by captures as the full module's (nothing to settle).
    pub settle: bool,
}

pub(crate) fn retire(_id: u32) {}

/// The edit log's state block, so a world here is the bytes the full build
/// writes (a save, the world hash; `og_ecs::snapshot`'s blocks section):
/// never an edit. A save that carries one is a 3D game's and is refused
/// with the reason, never silently replayed into no terrain.
struct TerrainBlock;

impl og_ecs::StateBlock for TerrainBlock {
    fn save(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&0u32.to_le_bytes());
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn std::any::Any + Send>> {
        let edits = bytes
            .get(..4)
            .and_then(|b| b.try_into().ok())
            .map(u32::from_le_bytes);
        match edits {
            Some(0) if bytes.len() == 4 => Ok(Box::new(())),
            Some(n) if n > 0 => Err(og_core::Error::new(
                og_core::ErrorKind::Invalid,
                format!("a save with {n} terrain edit(s): {ABSENT}"),
            )),
            _ => Err(og_core::Error::new(
                og_core::ErrorKind::Parse,
                "a damaged terrain edit log",
            )),
        }
    }

    fn apply(&mut self, _decoded: Option<Box<dyn std::any::Any + Send>>) {}
}

/// The edit log's state block (the world's state is the same here as in the
/// full build); the og.terrain system has nothing to replay, so none.
pub(crate) fn mount(world: &mut World, _engine_id: u32) -> Result<(), String> {
    world
        .add_state_block("og.terrain", Box::new(TerrainBlock))
        .map_err(|e| e.reason().to_string())
}

pub(crate) fn set_streaming(_engine_id: u32, _args: &[String]) -> Result<(), String> {
    Err(format!("terrain_stream: {ABSENT}"))
}

/// `dump_stream`'s `terrain`: null, there is no streamer.
pub(crate) fn stream_json(_engine_id: u32) -> Value {
    Value::Null
}

pub(crate) fn field_of_geometry(
    _t: &TerrainState,
    _geometry: u64,
) -> Option<og_render::TerrainData> {
    None
}

pub(crate) fn ground_drop_shadows(_rs: &mut RenderState) {}

pub(crate) fn extract(
    _world: &World,
    _rs: &mut RenderState,
    _h: &mut Hierarchy,
) -> Vec<(u64, u32)> {
    Vec::new()
}

fn absent() -> String {
    super::refuse(format!("og.terrain: {ABSENT}"))
}

pub fn og_terrain_load(_world: u32, _path: String) -> String {
    absent()
}

pub fn og_terrain_sample(_world: u32, _desc: Option<String>) -> String {
    absent()
}

pub fn og_terrain_bounds(
    _world: u32,
    _data: f64,
    _origin: Option<String>,
    _quaternion: Option<String>,
    _scale: Option<String>,
) -> String {
    absent()
}

pub fn og_terrain_edit(_world: u32, _data: f64, _brush: Option<String>) -> String {
    absent()
}

pub fn og_terrain_layer(_world: u32, _data: f64, _index: u32) -> String {
    absent()
}

pub fn og_terrain_set_layer(
    _world: u32,
    _data: f64,
    _index: u32,
    _layer: Option<String>,
) -> String {
    absent()
}
