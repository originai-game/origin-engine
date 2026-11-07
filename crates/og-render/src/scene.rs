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
mod resources;
mod sprites;
mod store;
pub use environment::*;
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
    /// Per material: the one it draws in the batches of
    /// ([`RenderScene::alias_material`]), with both revisions when it was
    /// set; it holds only while both are unchanged.
    material_aliases: Vec<Option<(u32, u64, u64)>>,
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
    pub light: DirectionalLight,
    /// Point and spot lights; the first [`MAX_LIGHTS`] are drawn.
    pub lights: Vec<Light>,
    pub environment: Environment,
    /// A cooked environment map lighting the scene instead of
    /// [`Environment::sky`] (three.js `scene.environment`), drawn as the
    /// background by [`Background::Sky`]; `None`: the sky lights it.
    pub environment_map: Option<crate::EnvironmentMap>,
    /// A hemisphere light, as three.js `HemisphereLight`: diffuse-only
    /// ambient irradiance `mix(ground, sky, 0.5 n.y + 0.5) * intensity`
    /// (sky, ground linear RGB; intensity 0: off). Unlike light baked into
    /// the environment map it adds no reflection.
    pub hemisphere: ([f32; 3], [f32; 3], f32),
    /// The lit shaders' specular: the engine's ([`Specular::Engine`], the
    /// default) or three.js's.
    pub specular: Specular,
    pub post: PostEffects,
    /// Stylisation after (and, for depth bands, with) the post stack: off
    /// by default ([`crate::Stylise`]).
    pub stylise: crate::Stylise,
    /// 2D quads drawn after the 3D scene, blended, sorted by layer.
    pub sprites: Vec<Sprite>,
    /// MSDF text, sorted with the sprites by layer.
    pub texts: Vec<Text>,
    pub debug: DebugView,
    /// The game's tick (drives beams' scroll and film grain; particle
    /// systems and trails carry their own).
    pub tick: u64,
    /// Ticks per second (the fixed step of particles and trails).
    pub tick_rate: f32,
    /// Real seconds since the host's previous frame (its display's frame
    /// interval, ticks or none: paused worlds still draw), for what eases
    /// over seconds of the player's time whatever the frame rate and
    /// never feeds the tick: a streamed terrain tile's fade-in. Default
    /// one sixtieth.
    pub frame_seconds: f32,
    /// A display frame drawn between the last two ticks (render
    /// interpolation): its alpha, 0 at the tick before `tick` and 1 at
    /// `tick`, as the blended transforms are drawn; `None` draws `tick`.
    /// The sea's clock runs at it, so waves move at the display's rate and
    /// stay in step with the floaters drawn on them.
    pub interpolation: Option<f32>,
    /// Running particle systems (state is kept by their id).
    pub particles: Vec<ParticleSystem>,
    /// The scene's solid shapes ([`crate::SceneCollider`]): what
    /// CPU-simulated particles with collision bounce off (the CPU path's
    /// stand-in for the depth buffer the compute path reads) and what
    /// visual bodies ([`crate::VisualBody`]) meet on the GPU. Empty by
    /// default.
    pub colliders: Vec<crate::SceneCollider>,
    /// Fine pieces of broken props, simulated and drawn on the GPU only
    /// ([`crate::DebrisPiece`], `debris.rs`): each drawn by instances of
    /// [`RenderScene::instances`] whose transforms the GPU writes. They
    /// collide with [`RenderScene::colliders`]. Empty by default.
    pub debris: Vec<crate::DebrisPiece>,
    /// Systems the host simulated itself (its tick ran
    /// [`crate::CpuParticles::advance`] to read their events), by
    /// [`ParticleSystem::id`]: the renderer draws these particles and their
    /// sub-emitters' as they are instead of simulating them, so what is
    /// drawn is what the game read. A system absent here simulates as usual.
    pub hosted_particles: Vec<(u64, std::sync::Arc<crate::CpuParticles>)>,
    pub trails: Vec<Trail>,
    pub ribbons: Vec<Ribbon>,
    pub beams: Vec<Beam>,
    /// Meshes drawn with an effect material this frame (`effect.rs`).
    pub effect_meshes: Vec<crate::EffectMesh>,
    /// Line sets drawn this frame (`lines.rs`): in the scene or over the
    /// finished frame (debug draw, screen lines).
    pub lines: Vec<crate::Lines>,
    /// Point clouds drawn this frame (`points.rs`).
    pub points: Vec<crate::Points>,
    /// The output's height in CSS pixels, which line widths and point sizes
    /// are measured in (0: one CSS pixel per device pixel).
    pub css_height: f32,
    /// Projected decals (the first [`crate::MAX_DECALS`] are drawn).
    pub decals: Vec<Decal>,
    /// The texture decals take their `uv_rect` from (`None`: solid colour).
    pub decal_atlas: Option<TextureHandle>,
    /// 2D lighting of the sprite layers (`None`: sprites are unlit).
    pub lighting_2d: Option<Lighting2d>,
    pub lights_2d: Vec<Light2d>,
    pub occluders_2d: Vec<Occluder2d>,
    /// Effects on single sprites (flash, glow, blend), by sprite index.
    pub sprite_fx: Vec<SpriteFx>,
    /// Textured triangles drawn in the sprite order (a 2D skeleton's
    /// attachments): [`crate::SpriteMesh`].
    pub sprite_meshes: Vec<crate::SpriteMesh>,
    /// Screen shake applied to the camera.
    pub shake: CameraShake,
    /// Reflection probes: boxes reflecting a cube captured inside them
    /// instead of the environment (the first [`crate::MAX_REFLECTION_PROBES`]).
    pub reflection_probes: Vec<crate::ReflectionProbe>,
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
    /// Screen-space sprites and text over the finished frame (`None`: none).
    pub overlay: Option<Overlay>,
    /// Fog over the main pass (`None`: none).
    pub fog: Option<crate::Fog>,
    /// Clouds in the procedural sky (`clouds.rs`; `None`: none). Drawn
    /// only with [`crate::Background::Procedural`].
    pub clouds: Option<crate::Clouds>,
    /// An aurora in the procedural sky (`aurora.rs`; `None`: none). Drawn
    /// only with [`crate::Background::Procedural`].
    pub aurora: Option<crate::Aurora>,
    /// How much of the image-based light's strength
    /// ([`crate::Environment::intensity`]) is the eye's adaptation to shade
    /// rather than the sky's own light: the day-night cycle's twilight lift
    /// ([`crate::DayNightLook::adaptation`]). It lights surfaces, but a
    /// mirror of the sky (the sea's reflection) divides it out, so it shows
    /// the sky as bright as the sky is drawn. 1: none.
    pub adaptation: f32,
    /// Rain, puddles and snow on the lit surfaces (`None`: dry).
    pub surface_weather: Option<crate::SurfaceWeather>,
    /// Fog lit as a participating medium (`volumetric.rs`; `None`: none).
    /// It replaces [`RenderScene::fog`] over the frame while it is set
    /// (and [`crate::RenderQuality::volumetric_fog`] lets it).
    pub volumetric_fog: Option<crate::VolumetricFog>,
    /// What the frame may cost: switches and sizes over what the rest of
    /// the scene asks for (`quality.rs`). The default draws all of it.
    pub quality: crate::RenderQuality,
    /// Baked lighting: lightmap pages, the lightmap uvs of the meshes drawn
    /// with them, light volumes (`lightmap.rs`). Empty by default.
    pub baked: crate::BakedLighting,
    /// Light volumes relit while the game runs (`probe_grid.rs`), sampled
    /// over the baked ones; the first [`crate::PROBE_VOLUMES`] with the
    /// baked ones fit. Empty by default.
    pub light_volumes: Vec<crate::RealtimeLightVolume>,
    /// Light cookies of point and spot lights, by index into
    /// [`RenderScene::lights`] (`area.rs`).
    pub light_cookies: Vec<crate::LightCookie>,
    /// Planar 3D water surfaces, drawn after the opaque scene and the sky.
    pub water: Vec<crate::Water>,
    /// Terrains drawn this frame (clipmaps of [`RenderScene::add_terrain`] data).
    pub terrains: Vec<crate::Terrain>,
    /// Meshes drawn as part of this frame's terrains (cliffs, arches,
    /// overhangs placed as terrain stamps): shaded with their terrain's
    /// layers and blended into its ground (`terrain.rs`).
    pub terrain_stamps: Vec<crate::TerrainStampDraw>,
    /// Copies scattered over this frame's terrains, placed and drawn by
    /// the renderer from cached chunks.
    pub scatters: Vec<crate::ScatterDraw>,
    /// Fields of grass over this frame's terrains, generated and drawn by
    /// the renderer on the GPU (`grass.rs`).
    pub grass: Vec<crate::GrassDraw>,
    /// Models drawn as their impostors this frame (`impostor.rs`): each a
    /// camera-facing card, drawn with the scattered copies' passes.
    pub impostors: Vec<crate::ImpostorDraw>,
    /// The wind that sways the scattered copies (`None` or strength 0:
    /// still).
    pub wind: Option<crate::Wind>,
    /// Local pushes on the simulated wind field (og.WindMotor,
    /// `wind_field.rs`): the nearest [`crate::wind_field::MAX_MOTORS`] drive it.
    pub wind_motors: Vec<crate::WindMotor>,
    /// What presses the grass and scattered copies (og.GrassPusher,
    /// `trample.rs`): the nearest [`crate::trample::MAX_PUSHERS`] press.
    pub grass_pushers: Vec<crate::GrassPusher>,
    /// Side-on 2D water bodies, drawn in the sprite order.
    pub water_2d: Vec<crate::Water2d>,
    /// 2D fluids drawn as metaballs in the sprite order (the first
    /// [`crate::MAX_FLUIDS`]; fluids of one group are one surface).
    pub metaballs: Vec<crate::Metaballs>,
    /// 3D particle fluids, drawn as one screen-space surface after the
    /// opaque scene, the sky and water.
    pub fluids_3d: Vec<crate::Fluid3d>,
    /// Sort keys of single sprites within their layer (top-down y sorting):
    /// a layer draws its sprites by ascending key, then texture, then
    /// submission order; a sprite without one has key 0.
    pub sprite_order: Vec<SpriteOrder>,
    /// Outlines of single world texts, by text index.
    pub text_outlines: Vec<TextOutline>,
    /// Tile layers: chunked quads drawn through the sprite batch, each
    /// under the sprites of its layer.
    pub tile_layers: Vec<TileLayer>,
    /// Trails drawn in the sprite order at a layer (2D) instead of with the
    /// 3D scene: `(Trail::id, layer)`.
    pub trail_layers: Vec<(u64, i32)>,
    /// Sprites drawn from a layer of a texture array instead of their
    /// texture, by sprite index (world sprites only; the last entry for an
    /// index wins). Sprites of one array draw as one run whatever their
    /// layers, so a sheet of frames in an array batches across frames.
    pub sprite_arrays: Vec<SpriteArray>,
    /// Drop shadows of single world sprites, by sprite index (the last
    /// entry for an index wins): each a flattened dark copy of the sprite
    /// on the ground plane y = 0 cast along [`RenderScene::light`], drawn
    /// before every sprite of its layer.
    pub sprite_shadows: Vec<SpriteShadow>,
    /// Pixel outlines of single world sprites, by sprite index (the last
    /// entry for an index wins; [`SpriteOutline`]).
    pub sprite_outlines: Vec<SpriteOutline>,
    /// Material shaders of single world sprites, by sprite index (the last
    /// entry for an index wins; [`SpriteShading`]).
    pub sprite_shading: Vec<SpriteShading>,
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
    /// The same for the world `sprites` (with their drop shadows, glow and
    /// afterimages drawn from them).
    pub sprite_layers: Vec<u32>,
    /// Each world sprite's 2D light layers (`og.LightMask2D`), by index in
    /// `sprites`: lit by the [`crate::Light2d`]s whose `layers` share a
    /// bit with it. An index past the end is 1 (layer 0, every light's
    /// default). A frame lights at most [`crate::MAX_LIGHT_MASKS_2D`]
    /// different masks apart; a further one is lit as mask 1.
    pub sprite_light_masks: Vec<u32>,
    /// How each world sprite faces the camera, by index in `sprites`
    /// (billboards): an index past the end is [`SpriteFacing::None`], the
    /// xy plane. A faced sprite without a [`RenderScene::sprite_order`] key
    /// (and tone mapped) draws among the transparent meshes, back to front
    /// with them by the view depth of its anchor from the eye that draws
    /// it, its drop shadow before them; with a key it keeps the sprites'
    /// (layer, key) order. Culled by its bounds (a sphere round the anchor
    /// through the farthest corner) against that camera's frustum. Its
    /// drop shadow is cast by the quad stood upright facing the sun, so it
    /// shows the sprite's whole shape from every camera.
    pub sprite_facing: Vec<SpriteFacing>,
    /// World texts turned to the camera ([`TextFacing`], by text index; the
    /// last entry for an index wins; a text without one stays in the xy
    /// plane), as [`RenderScene::sprite_facing`] turns sprites.
    pub text_facing: Vec<TextFacing>,
    /// The same for `lines` and `points` (scene sets; overlay and screen
    /// sets draw on the main frame whatever their masks).
    pub line_layers: Vec<u32>,
    pub point_layers: Vec<u32>,
    /// The same for `particles` (billboards and 2D sprite particles alike).
    pub particle_layers: Vec<u32>,
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
    /// Custom post passes ([`RenderScene::add_post`], `post_custom.rs`) and
    /// their parameters, by handle.
    pub posts: Vec<crate::PostShader>,
    pub post_params: Vec<[[f32; 4]; crate::MAX_SHADER_PARAMS]>,
    /// Cooked vectors the overlay's UI draws ([`RenderScene::add_ui_vector`]).
    pub ui_vectors: Vec<crate::ui::UiVectorData>,
    /// Custom materials UI layers composite through
    /// ([`RenderScene::add_ui_material`]).
    pub ui_materials: Vec<crate::ui::UiMaterialShader>,
    /// The custom post passes the frame runs, in order (at most
    /// [`crate::MAX_CUSTOM_POSTS`]), after the built-in effects and before
    /// exposure, bloom and the tone curve (`PostEffects.custom` in the API).
    pub custom_post: Vec<crate::PostHandle>,
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
