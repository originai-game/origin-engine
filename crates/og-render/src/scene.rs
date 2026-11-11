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
    /// Draw segments per coarse cloth/soft-body edge; zero selects two, 1–8.
    pub cloth_subdivisions: u32,
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
    /// Objects to draw this frame, in any order.
    pub instances: Vec<Instance>,
    pub camera: Camera,
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
    pub debug: DebugView,
    /// The game's tick (drives beams' scroll and film grain; particle
    /// systems and trails carry their own).
    pub tick: u64,
    /// Ticks per second (the fixed step of particles and trails).
    pub tick_rate: f32,
    /// A display frame drawn between the last two ticks (render
    /// interpolation): its alpha, 0 at the tick before `tick` and 1 at
    /// `tick`, as the blended transforms are drawn; `None` draws `tick`.
    /// The sea's clock runs at it, so waves move at the display's rate and
    /// stay in step with the floaters drawn on them.
    pub interpolation: Option<f32>,
    /// The effects drawn among the scene: particles and their history, trails,
    /// ribbons and beams, effect meshes, lines and points (each with its
    /// layers), and the decals with their atlas.
    pub effects: SceneEffects,
    /// The 3D world's ground and waters: water, terrains and their stamps,
    /// scattered copies, grass, 3D fluids and the colliders they meet.
    pub world: SceneWorld,
    /// Fine pieces of broken props, simulated and drawn on the GPU only
    /// ([`crate::DebrisPiece`], `debris.rs`): each drawn by instances of
    /// [`RenderScene::instances`] whose transforms the GPU writes. They
    /// collide with [`RenderScene::colliders`]. Empty by default.
    pub debris: Vec<crate::DebrisPiece>,
    /// The output's height in CSS pixels, which line widths and point sizes
    /// are measured in (0: one CSS pixel per device pixel).
    pub css_height: f32,
    /// Screen shake applied to the camera.
    pub shake: CameraShake,
    /// Each instance's transform in the previous frame (same order as
    /// `instances`), for per-object motion blur. Ignored unless its length
    /// equals `instances`'; then only the camera's motion blurs.
    pub previous_transforms: Vec<Mat4>,
    /// Each joint palette's joints as the previous frame drew them, in
    /// the world (the transform its instances were drawn at times each
    /// joint), same order as `joint_palettes`: TAA's motion vectors of
    /// skinned meshes. Ignored unless its length equals
    /// `joint_palettes`'; a palette whose joint count differs, or a mesh
    /// with morph targets alone, moves with the camera only.
    pub previous_joint_palettes: Vec<JointPalette>,
    /// Each skinned instance's blendshape weights as the previous frame drew
    /// them, same order as `skinned_instances`: TAA's motion vectors of a
    /// morphing mesh (the palette buffer's previous section, `skin.rs`).
    /// Ignored unless its length equals `skinned_instances`'; then the
    /// morph moves with the camera only.
    pub previous_morph_weights: Vec<[f32; crate::MAX_MORPH_TARGETS]>,
    /// Animation clips baked into joint palettes, played on the GPU by
    /// skinned instances with [`crate::SkinnedInstance::baked`] (crowds):
    /// kept from frame to frame; a frame uploads only the clips its
    /// instances use (`skin.rs`, the palette buffer's layout).
    pub baked_clips: Vec<crate::BakedClip>,
    /// Crowds: thousands of copies of one skinned model, each posed by a
    /// baked clip, culled, given a level and drawn on the GPU-driven path
    /// (`crowd.rs`).
    pub crowds: Vec<crate::Crowd>,
    /// The vertices of skinned instances that bring their own
    /// ([`crate::SkinnedInstance::deformed`] indexes it): a simulated
    /// garment's points, in each instance's space.
    pub deformed_vertices: Vec<crate::DeformedVertices>,
    /// Each `deformed_vertices` entry as the previous frame drew it, same
    /// order, in this frame's instance space: TAA's motion vectors of a
    /// swinging garment (the palette buffer's previous section, `skin.rs`).
    /// An entry whose count differs, or a list whose length differs, moves
    /// with the camera and the instance only.
    pub previous_deformed_vertices: Vec<crate::DeformedVertices>,
    /// The camera cut into this frame (a teleport or a cut between shots):
    /// motion blur takes no camera motion from the frame before, and TAA
    /// starts from this frame's samples ([`crate::Renderer::reset_history`]).
    pub camera_cut: bool,
    /// This frame's joint palettes ([`SkinnedInstance::palette`] indexes them).
    pub joint_palettes: Vec<JointPalette>,
    pub skinned_instances: Vec<SkinnedInstance>,
    /// The overlay (its UI tree and canvases) and the UI's vector paths and
    /// materials.
    pub ui: SceneUi,
    /// The air and the weather: fog, volumetric fog, clouds, aurora, the eye's
    /// adaptation, the surface weather, the wind and what pushes the grass.
    pub atmosphere: SceneAtmosphere,
    /// What the frame may cost: switches and sizes over what the rest of
    /// the scene asks for (`quality.rs`). The default draws all of it.
    pub quality: crate::RenderQuality,
    /// Models drawn as their impostors this frame (`impostor.rs`): each a
    /// camera-facing card, drawn with the scattered copies' passes.
    pub impostors: Vec<crate::ImpostorDraw>,
    /// Three.js layer masks of the instances, by index in `instances`
    /// (bit `n` set: on layer `n`): an index past the end is on layer 0
    /// alone (mask 1), as every object is in three.js. A camera draws the
    /// instances whose mask shares a bit with its own
    /// ([`RenderScene::camera_layers`], [`crate::View::layers`]); the shadow
    /// maps take every caster whatever its layers, so a body hidden from a
    /// first-person camera still casts its shadow.
    pub instance_layers: Vec<u32>,
    /// The same for `skinned_instances`.
    pub skinned_layers: Vec<u32>,
    /// The main camera's layer mask (three.js `Camera.layers`): it draws
    /// the instances, skinned instances, world sprites, scene lines and
    /// points and particle systems whose masks share a bit with it, as each
    /// [`crate::View`] does with its own. 1 by default; 0 draws none of
    /// them. What carries no mask (the sky, terrain, scattered copies,
    /// water, trails, text, tile layers and sprite meshes) draws on every
    /// camera.
    pub camera_layers: u32,
    /// Whether instances of clustered meshes draw through the meshlet path
    /// (`meshlet.rs`): their level picked and their clusters culled on the
    /// GPU. True by default; false draws them as any mesh, level 0.
    pub meshlets: bool,
    /// The scene's level-of-detail bias for clustered meshes (`lod.rs`'s
    /// bias: 2 keeps detail twice as far, 0 the full mesh), 1 by default.
    pub lod_bias: f32,
    /// Colours each clustered instance by the level it draws (green the
    /// full mesh, yellow, orange, red: [`crate::lod::debug_color`]), as
    /// og-runtime's `debugView lod` colours the others.
    pub lod_debug: bool,
    /// Whether the meshlet path rasterises the camera's small clusters of
    /// opaque materials in compute into a visibility buffer and shades
    /// them in one resolve per batch (`meshlet.rs`); true by default.
    /// False draws every cluster by the hardware.
    pub visibility_buffer: bool,
    /// The level each clustered instance draws on the meshlet path, by
    /// instance index (same order as `instances`), when its caller picked
    /// it: og-runtime picks one level per model by its largest error and
    /// keeps it near a boundary (`lod.rs` [`crate::lod::select`] with
    /// hysteresis), so the meshlet path draws what the regular path draws
    /// for its parts. Every view and shadow pass draws that level (a
    /// level past the mesh's coarsest draws the coarsest); `u8::MAX`, or
    /// an instance past the end, leaves it to the renderer, which picks
    /// each instance's level by its own screen-space error per view.
    pub instance_levels: Vec<u8>,
    /// Secondary cameras drawn into render targets or viewports of the
    /// frame, before it in [`crate::View::order`] (`views.rs`).
    pub views: Vec<crate::View>,
    /// Where the zero of the scene's coordinates is in the caller's world
    /// ([`RenderScene::rebase`], `origin.rs`): the frame is drawn relative
    /// to it, and the shaders that read world position for its look add
    /// it back. Zero by default.
    pub origin: Vec3,
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
