//! The renderer's input: everything a frame needs, with no knowledge of the ECS.
//!
//! A [`RenderScene`] owns its resources (meshes, textures, materials) and hands
//! out handles for them; instances, the camera, the light and the environment
//! are plain public fields the caller rewrites every frame. Resources are
//! append-only: a handle stays valid for the scene's lifetime; materials,
//! unskinned meshes and textures can be replaced in place
//! ([`RenderScene::set_material`], [`RenderScene::set_mesh`],
//! [`RenderScene::set_texture`]).

mod environment;
mod groups;
mod resources;
mod sprites;
mod store;
pub use environment::*;
pub use groups::*;
pub use resources::*;
pub use sprites::*;

use crate::effects::{
    AmbientOcclusion, CameraShake, ChromaticAberration, ColorGrading, ColorLut, Decal,
    DepthOfField, FilmGrain, Glow, GodRays, Light2d, Lighting2d, MotionBlur, Occluder2d, Outline,
    ScreenFlash, SpriteFx, Vignette,
};
use crate::material::StandardMaterial;
use crate::particles::{EmitterHandle, ParticleEmitter, ParticleSystem};
use crate::ribbons::{Beam, Ribbon, Trail, TrailStyle, TrailStyleHandle};
use crate::skin::{JointPalette, MorphTargets, SkinData, SkinnedInstance};
use crate::texture::TextureUpload;
use glam::{Mat4, Vec3};
pub use og_render_data::scene::*;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SCENE_ID: AtomicU64 = AtomicU64::new(1);

/// The in-place terrain edits a scene remembers per terrain
/// ([`RenderScene::edit_terrain`]); a renderer that missed more builds every
/// tile again.
pub const TERRAIN_EDIT_LOG: usize = 64;

/// Everything one frame draws. See the module documentation.
#[derive(Debug)]
pub struct RenderScene {
    id: u64,
    meshes: Vec<MeshData>,
    /// Per mesh: how many times [`RenderScene::set_mesh`] replaced it.
    mesh_revisions: Vec<u64>,
    /// Per mesh: [`RenderScene::set_vertex_encoding`].
    mesh_encodings: Vec<VertexEncoding>,
    /// Per mesh: its levels cut into clusters ([`RenderScene::set_mesh_clusters`]).
    mesh_clusters: Vec<Option<std::sync::Arc<crate::meshlet::MeshClusters>>>,
    /// Per mesh: its foliage deformation ([`RenderScene::set_mesh_foliage`]).
    mesh_foliage: Vec<Option<std::sync::Arc<crate::meshlet::MeshFoliage>>>,
    /// Coarse simulation surfaces; never clustered, skinned or compact.
    pub(crate) mesh_surfaces: Vec<Option<crate::SimulationSurface>>,
    /// Per simulation surface: the GPU visual body it is ([`RenderScene::set_visual_body`]).
    pub(crate) mesh_visual: Vec<Option<std::sync::Arc<crate::VisualBody>>>,
    /// The 3D objects: mesh instances (layers, levels, previous transforms),
    /// skinned instances with their palettes, morphs and baked clips, crowds,
    /// deformed vertices, impostors, debris, and how meshes are drawn (meshlets,
    /// LOD bias and debug, visibility buffer, cloth subdivisions).
    pub objects: SceneObjects,
    textures: Vec<TextureSource>,
    /// Per texture: how many times [`RenderScene::set_texture`] replaced it.
    texture_revisions: Vec<u64>,
    /// What every material draws as.
    materials: Vec<StandardMaterial>,
    /// Per material: its shader and parameters.
    material_shaders: Vec<Option<(ShaderHandle, [[f32; 4]; crate::MAX_SHADER_PARAMS])>>,
    shaders: Vec<crate::MaterialShader>,
    /// The glTF-shaped form of the materials added as [`Material`].
    gltf_materials: Vec<Option<Material>>,
    material_revisions: Vec<u64>,
    /// Counts every change to a material, its revision or its shader's
    /// parameters: while it holds, the renderer skips its per-frame pass
    /// over the materials (engine s96).
    material_serial: u64,
    /// Per material: the one it draws in the batches of
    /// ([`RenderScene::alias_material`]), with both revisions when the two
    /// last drew the same; it holds only while both are unchanged since.
    material_aliases: Vec<Option<(u32, u64, u64)>>,
    /// Per material: the copies aliased to it, so an edit of it checks
    /// them again.
    material_copies: Vec<Vec<u32>>,
    fonts: Vec<MsdfFont>,
    /// Bumped by [`RenderScene::set_font`].
    font_revisions: Vec<u64>,
    /// Per mesh: its skin, for meshes added with `add_skinned_mesh`.
    skins: Vec<Option<SkinData>>,
    /// Per mesh: its blendshape targets.
    morphs: Vec<Option<MorphTargets>>,
    emitters: Vec<ParticleEmitter>,
    /// Effect materials (`effect.rs`), by handle.
    effect_materials: Vec<crate::EffectMaterial>,
    luts: Vec<ColorLut>,
    /// Per LUT slot, bumped when it is filled again or emptied.
    lut_revisions: Vec<u64>,
    palettes: Vec<crate::Palette>,
    /// The colours the cook put the drawn conformed art on (every
    /// texture's own palette, `<id>#palette`), sorted and unique, and how
    /// many times the set grew: the frame quantiser keeps a pixel of one
    /// as drawn (`Scene::add_conformed_colors`).
    conformed: Vec<[u8; 3]>,
    conformed_revision: u64,
    /// Per palette, environment map and texture array: how many times its
    /// slot was refilled or emptied (the renderer uploads or frees it again).
    palette_revisions: Vec<u64>,
    environment_maps: Vec<crate::EnvironmentMapData>,
    environment_map_revisions: Vec<u64>,
    /// Per environment map: the equirectangular image the renderer bakes
    /// on the GPU (`add_environment_equirect`), or `None` for a prefiltered
    /// map.
    environment_equirects: Vec<Option<std::sync::Arc<crate::EnvironmentEquirect>>>,
    texture_arrays: Vec<crate::TextureArrayData>,
    texture_array_revisions: Vec<u64>,
    /// How each texture array is sampled by sprites (by array index).
    texture_array_filters: Vec<TextureFilter>,
    trail_styles: Vec<TrailStyle>,
    terrain_data: Vec<crate::terrain::TerrainSource>,
    /// Per terrain: how many times [`RenderScene::set_terrain`] replaced it.
    terrain_revisions: Vec<u64>,
    /// Per terrain: the regions [`RenderScene::edit_terrain`] changed in
    /// place since it was last replaced whole, each with the revision it
    /// made (the latest [`TERRAIN_EDIT_LOG`]).
    terrain_edits: Vec<Vec<(u64, glam::Vec2, glam::Vec2)>>,
    /// The frame's clock and view: its tick and rate, interpolation, the camera
    /// (its cut and layers), the viewport views, the world origin, the debug
    /// view, the quality, the camera shake and the CSS height.
    pub frame: SceneFrame,
    /// The 3D lighting: the sun and the local lights, their cookies and the
    /// baked light and light volumes, the reflection probes, the hemisphere and
    /// the environment.
    pub lighting: SceneLighting,
    /// The frame's look: its post effects and stylise, the custom post passes
    /// and their parameters, and the colour filter.
    pub look: SceneLook,
    /// The 2D scene: sprites and texts with their order, outlines, shadows,
    /// shading, layers, light masks and facing, tile layers, sprite arrays,
    /// sprite effects and meshes, the 2D lighting, lights and occluders, 2D
    /// water and metaballs.
    pub sprite: SceneSprites,
    /// The effects drawn among the scene: particles and their history, trails,
    /// ribbons and beams, effect meshes, lines and points (each with its
    /// layers), and the decals with their atlas.
    pub effects: SceneEffects,
    /// The 3D world's ground and waters: water, terrains and their stamps,
    /// scattered copies, grass, 3D fluids and the colliders they meet.
    pub world: SceneWorld,
    /// The overlay (its UI tree and canvases) and the UI's vector paths and
    /// materials.
    pub ui: SceneUi,
    /// The air and the weather: fog, volumetric fog, clouds, aurora, the eye's
    /// adaptation, the surface weather, the wind and what pushes the grass.
    pub atmosphere: SceneAtmosphere,
    /// How each texture is sampled by sprites, tiles and text (by texture index).
    texture_filters: Vec<TextureFilter>,
    /// The size a streamed texture reports while it holds only its mip
    /// tail ([`RenderScene::set_texture_size`]): its whole chain's.
    texture_sizes: std::collections::HashMap<TextureHandle, (u32, u32)>,
    /// Each texture's normal map for 2D lighting (by texture index).
    texture_normals: Vec<Option<TextureHandle>>,
    /// The textures [`RenderScene::paletted_texture`] made, by (index
    /// texture, palette texture, row): the revisions of both it was made
    /// at, and the texture.
    paletted:
        std::collections::HashMap<(TextureHandle, TextureHandle, u32), (u64, u64, TextureHandle)>,
    /// Live slots and generations of the removable kinds.
    mesh_slots: Slots,
    texture_slots: Slots,
    /// Streamed video textures' newest frames, by texture index: a serial
    /// (bumped by every write) and the RGBA8 rows ([`crate::video`]).
    pub(crate) stream_frames: std::collections::BTreeMap<usize, (u64, std::sync::Arc<Vec<u8>>)>,
    pub(crate) stream_serial: u64,
    /// On the web: each stream texture's newest decoded VideoFrame, copied
    /// on the GPU (copyExternalImageToTexture) and closed before the next
    /// frame draws.
    #[cfg(target_arch = "wasm32")]
    pub(crate) stream_external:
        std::cell::RefCell<std::collections::BTreeMap<usize, wgpu::web_sys::VideoFrame>>,
    material_slots: Slots,
    font_slots: Slots,
    environment_map_slots: Slots,
    texture_array_slots: Slots,
    palette_slots: Slots,
    lut_slots: Slots,
}
