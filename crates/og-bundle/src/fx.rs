//! Native particle resource bindings and colour lookup tables.

pub mod cooked;

use og_render_data::{ColorLut, EmitterHandle, MeshHandle, TextureArrayHandle, TextureHandle};

/// The built-in particle meshes.
pub const PARTICLE_MESHES: &[&str] = &["box", "sphere", "tetrahedron"];

/// A built-in particle shape or a model's first mesh.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleMesh<'a> {
    Builtin(&'a str),
    Asset(&'a str),
}

/// Resolve native emitter resources to host handles. Capturing checked input
/// uses these same callbacks to construct the cooked resource table.
pub struct EmitterLinks<'a> {
    pub texture: &'a mut dyn FnMut(&str) -> Result<Option<TextureHandle>, String>,
    pub array: &'a mut dyn FnMut(&str) -> Result<Option<TextureArrayHandle>, String>,
    pub mesh: &'a mut dyn FnMut(ParticleMesh<'_>) -> Result<Option<MeshHandle>, String>,
    pub emitter: &'a mut dyn FnMut(&str) -> Result<Option<EmitterHandle>, String>,
    pub effect:
        &'a mut dyn FnMut(&str) -> Result<Option<og_render_data::EffectMaterialHandle>, String>,
}

/// A `.cube` colour table as a 32³ LUT (other sizes are resampled).
pub fn parse_cube(text: &str) -> Result<ColorLut, String> {
    ColorLut::from_cube(text)
}
