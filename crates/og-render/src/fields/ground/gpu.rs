//! The ground on the GPU, as what grows on it reads it.

use crate::wind::WindFieldGpu;
use glam::Vec2;

/// The grounds on the GPU as what grows on them reads them; the terrain
/// renderer, which holds their tiles, answers.
pub(crate) trait GroundGpu {
    /// A ground group's layout: the standard one, or (`arrayed`, once
    /// made) the layer-array variant's.
    fn group_layout(&self, arrayed: bool) -> Option<&wgpu::BindGroupLayout>;

    /// Level-0 tiles `tiles` of terrain data `data` as the GPU holds them
    /// this frame (each one's data, as its address: another when it is
    /// replaced), or `None` while one is not resident.
    fn level0_slots(&self, data: usize, tiles: &[u32]) -> Option<Vec<usize>>;

    /// A terrain data's size along x and z and its height range, once
    /// uploaded.
    fn extent_of(&self, data: usize) -> Option<(Vec2, (f32, f32))>;
}

/// One ground drawn this frame, as what grows on it binds it.
pub(crate) trait GroundDraw {
    /// Its group ([`TERRAIN_WGSL`]'s declarations), at group 1.
    fn group(&self) -> &wgpu::BindGroup;
    /// Its index in [`crate::RenderScene::terrains`].
    fn terrain(&self) -> usize;
    /// Whether its group is the layer-array variant's.
    fn arrayed(&self) -> bool;
}

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
