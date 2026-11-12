//! Origin Engine render data (`docs/design.md` §2.1, "Formats below
//! features"): what the renderer draws, as data with no GPU dependency, so
//! the cook that writes it (og-cook) and the bundle that loads it
//! (og-bundle) do not link the renderer. og-render draws it and re-exports
//! every item at the path it had there.
//!
//! - [`scene`]: meshes and vertices, textures and their sources, the
//!   glTF-shaped [`Material`] and the resource handles (og-draw's).
//! - [`texture`], [`texture_array`], [`targets`]: texture encodings,
//!   pre-encoded uploads, texture arrays and render target textures.
//! - [`material`]: [`StandardMaterial`], its uv transforms and physical
//!   extensions; [`environment`]: prefiltered environment maps (and their
//!   CPU bake, feature `bake`); [`terrain`] and [`terrain_tiles`]: height
//!   fields, layers and the tile pyramid; [`particles`], [`effect`],
//!   [`effects`]: emitters, effect materials and colour tables.
//! - The CPU builders whose output is cooked data: [`shapes`],
//!   [`geometry`] (primitives and the triangle BVH), [`meshlet`] clusters,
//!   [`foliage`] vertices, the [`impostor`] frame layout, [`gltf`]'s
//!   physical extensions (feature `gltf`).
//! - Material shaders ([`hooks`], [`custom`]), post passes and UI
//!   materials, and the WGSL modules they compile into
//!   ([`lit_module_source`], ...), which the cook validates with naga.

/// A file of og-render's `shaders/` as the renderer compiles it: the copy
/// build.rs wrote with its `//` comments removed (every token and line
/// kept), so the comments stay in the sources and out of the wasm.
macro_rules! wgsl {
    ($file:literal) => {
        include_str!(concat!(env!("OUT_DIR"), "/shaders/", $file))
    };
}

pub mod clusters;
pub mod custom;
pub mod effect;
pub mod effects;
pub mod environment;
pub mod foliage;
pub mod geometry;
#[cfg(feature = "gltf")]
pub mod gltf;
pub mod hooks;
pub mod ibl;
pub mod impostor;
pub mod lightmap;
pub mod lod;
pub mod material;
pub mod meshlet;
pub mod oit;
pub mod particles;
pub mod particles_gpu;
pub mod post_custom;
pub mod scene;
pub mod shader_source;
pub mod shapes;
pub mod skin;
pub mod sprite_hooks;
pub mod targets;
pub mod terrain;
pub mod terrain_error;
pub mod terrain_tiles;
pub mod texture;
pub mod texture_array;
pub mod ui;

pub use clusters::{CLUSTER_GRID, MAX_CLUSTER_LIGHTS, MAX_LIGHTS};
pub use custom::FULL_SHADER_ENTRIES;
pub use effect::{
    EFFECT_DRIVERS, EFFECT_PARAMS, EffectChannel, EffectMaterial, EffectMaterialHandle, EffectMesh,
    EffectRing, EffectTexture, EffectUv, EffectValue, Erosion, Flow, Fresnel, effect_cylinder,
    effect_ring,
};
pub use effects::{ColorLut, LUT_SIZE};
#[cfg(feature = "bake")]
pub use environment::bake;
pub use environment::{
    ENVIRONMENT_MAP_LEVELS, ENVIRONMENT_MAP_MAX_SIZE, ENVIRONMENT_MAP_MIN_SIZE,
    EnvironmentEquirect, EnvironmentMapData, EnvironmentSun, EquirectIrradiance, face_direction,
    rgb9e5, rgb9e5_decode, sh9,
};
pub use hooks::{
    MAX_SHADER_PARAMS, MaterialShader, ParamKind, SHADER_HOOKS, ShaderMode, ShaderParam,
    lit_module_source, physical_module_source, shadow_module_source, sprite_module_source,
};
pub use impostor::{Impostor, ImpostorDraw, MAX_IMPOSTOR_GRID};
pub use lightmap::PROBE_VOLUMES;
pub use material::{PhysicalExtensions, Side, StandardMaterial, UvTransform};
pub use particles::*;
pub use post_custom::{MAX_CUSTOM_POSTS, PostShader, example_crt, post_module_source};
pub use scene::*;
pub use shader_source::*;
pub use targets::{TargetFormat, TargetKind, TargetTexture};
pub use terrain::{
    DEFAULT_TERRAIN_BUDGET, MAX_TERRAIN_LAYERS, MAX_TERRAIN_LEVELS, TERRAIN_TEXTURED_LAYERS,
    TerrainControl, TerrainData, TerrainLayer, TerrainLod, TerrainSampler,
};
pub use terrain_tiles::{
    ControlPyramid, HeightPyramid, HoleCounts, MAX_TERRAIN_PYRAMID, MAX_TERRAIN_TILES,
    TERRAIN_TILE, TerrainLayout, TerrainTile, hole_bit, mark_holes, with_hole_bit,
};
pub use texture::{TextureCompression, TextureDesc, TextureFormat, TextureFormats, TextureUpload};
pub use texture_array::{TextureArrayData, TextureArrayHandle, TextureArrayInfo};
pub use ui::{UiMaterialShader, example_ui_material, ui_material_module_source};
