//! The ground on the GPU, as what grows on it reads it.

use crate::wind::WindFieldGpu;

pub(crate) const TERRAIN_WGSL: &str = wgsl3d!("terrain.wgsl");
/// What terrain.wgsl's far grass needs declared (after wind_common.wgsl).
pub(crate) const GRASS_COMMON_WGSL: &str = wgsl3d!("grass_common.wgsl");

/// The eight layer textures' declarations (terrain.wgsl), and the layer
/// arrays' that replace them in the layer-array variant.
const LAYER_TEXTURES: &str = "@group(1) @binding(5) var albedo0: texture_2d<f32>;
@group(1) @binding(6) var albedo1: texture_2d<f32>;
@group(1) @binding(7) var albedo2: texture_2d<f32>;
@group(1) @binding(8) var albedo3: texture_2d<f32>;
@group(1) @binding(9) var normal0: texture_2d<f32>;
@group(1) @binding(10) var normal1: texture_2d<f32>;
@group(1) @binding(11) var normal2: texture_2d<f32>;
@group(1) @binding(12) var normal3: texture_2d<f32>;
";
const LAYER_ARRAYS: &str = "@group(1) @binding(5) var albedo_array: texture_2d_array<f32>;
@group(1) @binding(9) var normal_array: texture_2d_array<f32>;
";

/// `source` (a terrain module) with the layer arrays declared in place of
/// the eight layer textures.
pub(crate) fn terrain_array_declarations(source: &str) -> String {
    assert!(
        source.contains(LAYER_TEXTURES),
        "the layer-array variant cannot find terrain.wgsl's layer textures"
    );
    source.replacen(LAYER_TEXTURES, LAYER_ARRAYS, 1)
}

/// `TerrainGrass` in terrain.wgsl: the far colour of a field.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct FarGpu {
    pub(crate) color: [f32; 4],
    pub(crate) fade: [f32; 4],
    pub(crate) clump: [f32; 4],
    pub(crate) dry: [f32; 4],
    pub(crate) root: [f32; 4],
    pub(crate) canopy: [f32; 4],
    pub(crate) wind: WindFieldGpu,
}
