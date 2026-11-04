//! Origin Engine renderer (`docs/design.md` §2.6, `crates/og-render/README.md`).
//!
//! A wgpu Forward+ renderer: PBR metallic-roughness, one directional light
//! with three cascaded shadow maps, up to 256 point and spot lights in view,
//! clustered (`clusters.rs`; up to eight of them shadowed, `shadows.rs`), split-sum image-based lighting generated from a procedural sky,
//! antialiasing (TAA by default, 4x MSAA or FXAA on request),
//! bloom, manual or metered exposure, ACES tonemapping, and a 2D batch of
//! sprites and MSDF text. It runs natively (Metal, Vulkan; headless for
//! executors) and in the browser (WebGPU) from the same code.
//!
//! The crate knows nothing about the ECS. Its whole input is a [`RenderScene`]:
//! resources with handles, instances and sprites submitted one by one, a
//! camera, lights and an environment. [`Renderer::render`] batches the
//! instances by (mesh, material) and the sprites by (layer, texture), and
//! reports [`FrameStats`].
//!
//! ```no_run
//! # fn main() -> Result<(), og_render::RenderError> {
//! use og_render::{shapes, Material, RenderDesc, RenderScene};
//! let mut scene = RenderScene::new();
//! let sphere = scene.add_mesh(shapes::sphere(16, 12));
//! let red = scene.add_material(Material { base_color: [0.8, 0.2, 0.1, 1.0], ..Default::default() });
//! scene.add_instance(sphere, red, glam::Mat4::IDENTITY);
//! # #[cfg(not(target_arch = "wasm32"))]
//! let frame = og_render::render_to_rgba(&RenderDesc::default(), &scene)?;
//! # Ok(()) }
//! ```

/// A file of `shaders/` as the renderer compiles it: the copy build.rs
/// wrote with its `//` comments removed (every token and line kept), so
/// the comments stay in the sources and out of the wasm.
macro_rules! wgsl {
    ($file:literal) => {
        include_str!(concat!(env!("OUT_DIR"), "/shaders/", $file))
    };
}

/// [`wgsl!`] for a 3D world subsystem's own file (terrain, water, the 3D
/// fluids, the scattered copies, volumetric fog): empty in a `profile2d`
/// build, which never draws them ([`WORLD_3D`]), so their text is not in
/// the module.
#[cfg(not(feature = "profile2d"))]
macro_rules! wgsl3d {
    ($file:literal) => {
        wgsl!($file)
    };
}
#[cfg(feature = "profile2d")]
macro_rules! wgsl3d {
    ($file:literal) => {
        ""
    };
}

/// Whether this renderer draws the 3D world subsystems: terrain, planar
/// water, the 3D fluids, the scattered copies and volumetric fog. False in
/// a `profile2d` build (the 2D profile's web modules), which leaves their
/// shaders and their per-frame work out and draws a scene as if it had
/// none of them (`RenderCapabilities.world3d`).
pub const WORLD_3D: bool = cfg!(not(feature = "profile2d"));

mod renderer;
pub use renderer::*;

pub mod wind_field;
pub use wind_field::{WindMotor, WindMotorKind};
pub mod trample;
pub use trample::GrassPusher;

mod area;
mod atmosphere;
mod aurora;
mod auto_quality;
mod blue_noise;
mod capture;
mod clouds;
mod clusters;
mod compact;
mod cull_gpu;
mod custom;
pub mod debug;
mod deferred;
mod effects;
mod environment;
mod fluid3d;
pub mod foliage;
mod frame;
mod fx;
pub mod geometry;
mod gi_probes;
mod gi_scene;
mod glass;
pub mod graph;
mod hdr;
mod hooks;
mod ibl;
mod id_pass;
mod instances;
mod light2d;
mod lightmap;
mod lines;
pub mod lod;
pub mod meshlet;
mod meshlet_gpu;
mod probe_content;
mod probe_grid;
mod probe_select;
mod probes;
mod quality;
mod sky_capture;
mod ssgi;
mod ssr;
pub use gi_probes::ProbeGlobalIllumination;
pub use probe_content::{ProbeContent, ProbeRoom};
pub use probe_grid::{MAX_GRID_CAPTURES, RealtimeLightVolume};
pub use probes::{
    MAX_REFLECTION_PROBES, PROBE_MAX_RESOLUTION, PROBE_MIN_RESOLUTION, ProbeCapture, ProbeMode,
    ReflectionProbe,
};
pub use ssgi::ScreenSpaceGlobalIllumination;
pub use ssr::ScreenSpaceReflections;
mod material;
mod metaballs;
mod mips;
mod oit;
pub use oit::{
    DEFAULT_TRANSPARENCY_LAYERS, MAX_TRANSPARENCY_LAYERS, Transparency, TransparencyCounts,
    weighted_order_bound, weighted_order_codes,
};
mod origin;
mod overlay;
pub mod particles;
pub mod ui;
pub use ui::{
    UI_GRADIENT_STOPS, UI_MASS_KEYS, UI_MASS_QUADS, UI_MATERIAL_PARAMS, UiAlign, UiBlend,
    UiBuiltin, UiCap, UiClip, UiCorner, UiEdge, UiElevation, UiFit, UiGeometry, UiGlass,
    UiGlassFallback, UiGlassShadow, UiGlassStyle, UiGlassWeight, UiGradient, UiGradientKind,
    UiImage, UiJoin, UiLayer, UiList, UiMask, UiMassBatch, UiMassEase, UiMassInstance, UiMassKey,
    UiMassKind, UiMassSpace, UiMassTemplate, UiMaterial, UiMaterialHandle, UiMaterialShader,
    UiMode, UiOccluder, UiOcclusion, UiPaint, UiParticleShape, UiShape, UiSpread, UiStop, UiStroke,
    UiTextEffects, UiTextOutline, UiTextShadow, UiVectorData, UiVectorDraw, UiVectorHandle,
    UiVectorPaint, UiVectorStroke, UiVectorVertex, example_ui_material, flatten_cubic,
    flatten_quad, format_number, oklab, oklab_to_linear, ui_material_module_source, ui_scrim,
};
mod particles_gpu;
mod pipeline_cache;
pub use pipeline_cache::set_pipeline_cache_dir;
mod grass;
mod impostor;
mod physical;
mod points;
mod post;
mod post_custom;
mod ribbons;
mod ribbons_gpu;
mod scatter;
mod scatter_cull;
mod scene;
mod shaders;
pub use shaders::{ShaderSource, shader_sources};
mod garment;
mod garment_gpu;
pub mod ocean_fft;
mod shadow_cache;
mod shadows;
pub mod shapes;
mod skin;
pub use garment::{GarmentDeformation, GarmentTopology};
mod sky_physical;
mod sprite_array;
mod sprite_mesh;
mod sprite_normals;
mod sprite_skin;
mod sprites;
mod stylise;
mod stylise_object;
mod surface;
pub use surface::SimulationSurface;
mod targets;
mod terrain;
mod terrain_tiles;
mod texture;
mod texture_array;
mod tiles_gpu;
mod timing;
mod upload;
mod video;
mod views;
mod volumetric;
mod water;
mod water_ripples;
mod weather;
mod wind;

#[cfg(feature = "gltf")]
pub mod gltf;
#[cfg(not(target_arch = "wasm32"))]
mod headless;
#[cfg(any(target_os = "macos", target_os = "linux", windows))]
pub mod shared_frames;

pub use area::{CookieLayers, LightCookie, MAX_COOKIE_LAYERS, RECT_SHADOW_FOV, cookie_layers};
pub use atmosphere::{DayNightLook, Fog, SkyLook, day_night};
pub use aurora::Aurora;
pub use auto_quality::{AutoQuality, AutoQualityBounds, FrameSample};
pub use capture::{
    BACKLOG_TIMEOUT, CAPTURE_TIMEOUT, CaptureRequest, capture_timeouts, png, png_best,
};
pub use clouds::Clouds;
pub use clusters::{CLUSTER_GRID, MAX_CLUSTER_LIGHTS};
pub use custom::{FULL_SHADER_ENTRIES, example as example_full_shader};
pub use effects::*;
#[cfg(feature = "bake")]
pub use environment::bake;
pub use environment::{
    ENVIRONMENT_BLEND_STEPS, ENVIRONMENT_MAP_LEVELS, ENVIRONMENT_MAP_MAX_SIZE,
    ENVIRONMENT_MAP_MIN_SIZE, EnvironmentBlend, EnvironmentEquirect, EnvironmentMap,
    EnvironmentMapData, EquirectIrradiance, face_direction, rgb9e5, rgb9e5_decode, sh9,
};
pub use fluid3d::Fluid3d;
pub use graph::{GraphError, GraphPlan, RenderGraph, TargetDesc};
pub use grass::{DistanceLod, Grass, GrassDraw, GrassType};
#[cfg(not(target_arch = "wasm32"))]
pub use headless::render_to_rgba;
pub use hooks::{
    MAX_SHADER_PARAMS, MaterialShader, ParamKind, SHADER_HOOKS, ShaderMode, ShaderParam,
    lit_module_source, physical_module_source, shadow_module_source,
};
pub use id_pass::{
    IdCamera, IdFrame, IdObject, IdSources, id_pass_size, projection_name, render_ids,
    render_ids_from, view_id_cameras,
};
pub use impostor::{Impostor, ImpostorDraw, MAX_IMPOSTOR_GRID};
pub use lightmap::{
    BakedLighting, BakedMesh, LightmapPages, LightmapUvs, PROBE_VOLUMES, Probe, ProbeVolume,
};
pub use lines::{LineLayer, LineMode, Lines, wire_edges};
pub use material::{PhysicalExtensions, Side, StandardMaterial, UvTransform};
pub use metaballs::{FIELDS, MAX_FLUIDS, Metaballs};
pub use particles::{
    Burst, ChildParticles, ColorCurve, CpuParticles, Curve, EmitterHandle, EmitterShape,
    MAX_CURVE_KEYS, MAX_SUB_EMITTERS, ParticleAlign, ParticleBlend, ParticleClock,
    ParticleCollider, ParticleCollision, ParticleEmitter, ParticleEvent, ParticleEventKind,
    ParticleNoise, ParticleRender, ParticleSimulation, ParticleState, ParticleSystem, SheetMode,
    SimulationSpace, SpriteSheet, SubEmitter, mesh_points,
};
pub use points::Points;
pub use post_custom::{MAX_CUSTOM_POSTS, PostHandle, PostShader, example_crt, post_module_source};
pub use quality::{QualityPreset, RenderQuality};
pub use ribbons::{Beam, Ribbon, RibbonPoint, RibbonUv, Trail, TrailStyle, TrailStyleHandle};
pub use scatter::{ScatterContext, ScatterDraw, WIND_WAVELENGTH, Wind, wind_phases, wind_shader};
/// The octahedral impostors' frame layout, shared with the cook's bake.
pub mod impostors {
    pub use crate::impostor::{
        blend_frames, frame_basis, frame_direction, hemi_oct_decode, hemi_oct_encode,
    };
}
pub use scene::*;
pub use shadows::PointShadows;
pub use skin::{
    BakedClip, BakedPose, DeformedVertices, JointPalette, MAX_JOINTS, MAX_MORPH_TARGETS,
    MorphTargets, SkinData, SkinnedInstance,
};
pub use sky_physical::PhysicalAtmosphere;
pub use sprite_mesh::{SpriteInfluence, SpriteMesh, SpriteMeshVertex, SpriteSkin};
pub use stylise::{
    DEFAULT_CANVAS_WORLD_SCALE, DEFAULT_CHARACTER_DETAIL, DEFAULT_PAINT_DETAIL,
    DEFAULT_TOON_SOFTNESS, DEFAULT_WIREFRAME_ANGLE, DepthBands, DitherPattern, Halftone, Hatching,
    Ink, MAX_PALETTE_COLORS, MaterialStyle, MaterialSurface, OutlinePlace, Paint, Palette,
    PaletteHandle, PixelFit, PixelScale, Quantize, Retro3d, STYLISE_DRAWN, Stylise, StyliseSpace,
};
pub use targets::{RenderTargetDesc, RenderTargetHandle, TargetFormat, TargetKind, TargetTexture};
pub use terrain::{
    DEFAULT_TERRAIN_BUDGET, MAX_SCATTER_INSTANCES, MAX_TERRAIN_LAYERS, MAX_TERRAIN_LEVELS,
    TERRAIN_TEXTURED_LAYERS, Terrain, TerrainClipmap, TerrainControl, TerrainData, TerrainLayer,
    TerrainLayerArrays, TerrainLevel, TerrainLod, TerrainSampler, TerrainScatter, TerrainStampDraw,
};
pub use terrain_tiles::{
    ControlPyramid, HeightPyramid, HoleCounts, MAX_TERRAIN_PYRAMID, MAX_TERRAIN_TILES,
    TERRAIN_TILE, TerrainLayout, TerrainTile, TerrainView, hole_bit, mark_holes,
    terrain_tiles_wanted, with_hole_bit,
};
pub use texture::{TextureCompression, TextureDesc, TextureFormat, TextureFormats, TextureUpload};
pub use texture_array::{TextureArrayData, TextureArrayHandle, TextureArrayInfo};
pub(crate) use timing::Encoder;
pub use timing::{GpuFrame, GpuPass, PassTiming, gpu_timing_absent};
pub use video::FrameTexture;
pub use views::{MirrorQuad, View, ViewTarget};
pub use volumetric::{FROXELS, RAYMARCH_LIGHTS, RAYMARCH_STEPS, VolumetricFog, VolumetricMethod};
pub use water::{WAKE_TICKS, Water, Water2d, WaterCut, WaterHeights, WaterWakes};
pub use weather::{
    BOLT_CORE, SurfaceWeather, WEATHER_DUST, WEATHER_MIST, WEATHER_NONE, WEATHER_RAIN,
    WEATHER_SNOW, WeatherLook, add_lightning, lightning_bolt, lightning_branches, lightning_flash,
    weather,
};
/// The wgpu this crate is built on, for hosts that create surfaces or pass views.
pub use wgpu;

use deferred::{AsModule, Deferred, Sampled};
use frame::{CASCADES, Cascade, Frustum};
use glam::{Mat4, Vec2, Vec3};
use material::MaterialUniform;
use post::FramePass;
use shadows::LOCAL_SHADOW_LAYERS;
use sprites::{FontLayout, RunSource, SPRITE_LAYOUT, SPRITE_STRIDE};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// The renderer. Create it once per output, call [`Renderer::render`] per frame.
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    info: RenderInfo,
    /// What it was made with, for [`Renderer::recreate`].
    desc: RenderDesc,
    /// The scene's size as a fraction of the output's
    /// ([`Renderer::set_render_scale`]).
    render_scale: f32,
    /// TAA resolves the scene drawn at the render scale up (or down) to
    /// the output (TAAU): its history and every pass after it are at the
    /// output's size, and no upscale pass runs.
    taau: bool,
    /// Set once the device is lost ([`Renderer::lost`]).
    lost: renderer::LostFlag,
    /// The size the scene is drawn at: the output's, or the stylisation's
    /// logical pixel grid ([`Stylise::pixel`](crate::Stylise::pixel)),
    /// scaled up to the output by the last post pass.
    width: u32,
    height: u32,
    /// The output's size (the offscreen target, the surface, the overlay).
    out_width: u32,
    out_height: u32,
    output_format: wgpu::TextureFormat,
    hdr: Option<hdr::Output>,
    /// The offscreen target's sRGB view format ([`linear_view_format`]).
    offscreen_linear: Option<wgpu::TextureFormat>,
    surface: Option<(wgpu::Surface<'static>, wgpu::SurfaceConfiguration)>,
    /// Copy each presented surface frame into the offscreen target, so
    /// [`Renderer::begin_capture`] reads it ([`Renderer::set_keep_frame`]).
    keep_frame: bool,
    /// The last capture's buffer once read, for the next capture of its size.
    capture_spare: capture::Spare,
    /// Native offscreen draws have no surface to bound their frames in flight.
    #[cfg(not(target_arch = "wasm32"))]
    offscreen_submissions: std::collections::VecDeque<wgpu::SubmissionIndex>,
    samples: u32,
    persistent: post::Persistent,
    graph: post::FrameGraph,
    /// Measures each graph pass's encoding (`set_clock`); measurement only.
    clock: Option<fn() -> f64>,
    /// The last frame's passes in execution order, each with the start and
    /// length of its encoding (`pass_timings`).
    pass_ms: Vec<PassTiming>,
    /// Each pass's GPU time where the device has timestamp queries and a
    /// clock is set (`timing.rs`, `gpu_timings`).
    gpu_timer: Option<timing::GpuTimer>,
    /// The CPU cull's per-instance scratch ([`Renderer::cull`]), kept
    /// between frames so no pass allocates it.
    /// The sorted instances, kept across frames (`instances.rs`).
    sorted: instances::SortedInstances,
    /// The secondary cameras' GPU state (`views.rs`).
    views: views::Views,
    /// The custom post passes on the device (`post_custom.rs`).
    custom_posts: post_custom::CustomPosts,
    cull_inside: CullScratch,
    /// What the weather occlusion map in the cascade array's last layer
    /// was last drawn from (`weather::shelter_key`; `None`: not drawn
    /// since the array was made).
    shelter_key: Option<u64>,
    post: post::PostPasses,
    cascade_map: ShadowMap,
    local_map: ShadowMap,
    /// The cascades' side at [`RenderQuality::shadow_map_scale`] 1 (the
    /// [`RenderDesc`]'s, clamped); [`RenderInfo::shadow_map_size`] is the
    /// side drawn now.
    full_shadow_size: u32,
    /// The local lights' layers' side now ([`LOCAL_SHADOW_SIZE`] at scale 1).
    local_shadow_size: u32,
    material_sampler: wgpu::Sampler,
    sprite_sampler: wgpu::Sampler,
    sprite_sampler_nearest: wgpu::Sampler,
    /// The font atlases', the decal atlas's and the 2D lightmap's:
    /// trilinear, clamped, never anisotropic.
    trilinear_sampler: wgpu::Sampler,
    frame_layout: wgpu::BindGroupLayout,
    sky_layout: wgpu::BindGroupLayout,
    /// The clouds' noise volume, bound in the sky group (`clouds.rs`).
    cloud_noise: clouds::CloudNoise,
    /// The sky pass's own uniform (sky.wgsl `SkyParams`): the background's
    /// blurriness and the map it draws, written every frame.
    sky_params: wgpu::Buffer,
    /// The aurora's footprint and table, bound in the sky group (`aurora.rs`).
    aurora_map: aurora::AuroraMap,
    material_layout: wgpu::BindGroupLayout,
    sprite_layout: wgpu::BindGroupLayout,
    frame_buffer: wgpu::Buffer,
    /// The local lights, the cluster table and its light references (`clusters.rs`).
    light_buffers: [wgpu::Buffer; 3],
    /// Baked lighting: lightmap pages and uvs, the light volumes' buffer
    /// (frame binding 10).
    lightmaps: lightmap::Lightmaps,
    /// The rect lights' LTC table (frame binding 11).
    ltc: wgpu::Buffer,
    /// Light cookies in the local shadow array's last layers.
    cookies: area::Cookies,
    /// Volumetric fog (made on first use).
    volumetric: volumetric::Volumetric,
    clusters: clusters::ClusterBuilder,
    /// Per planar reflection slot, as many as [`RenderInfo::planar_reflections`] (0: the one material shaders read
    /// with two, 1: a second surface's): the frame uniform of its mirrored camera.
    reflection_buffer: Vec<wgpu::Buffer>,
    /// Its effect uniform (decals; no screen-space AO).
    reflection_fx_buffer: Vec<wgpu::Buffer>,
    /// Its cluster table and references, over the main view's light records.
    reflection_light_buffers: Vec<[wgpu::Buffer; 3]>,
    reflection_clusters: Vec<clusters::ClusterBuilder>,
    shadow_buffer: wgpu::Buffer,
    shadow_bind_group: wgpu::BindGroup,
    pbr_layout: wgpu::PipelineLayout,
    pbr_skinned_layout: wgpu::PipelineLayout,
    /// Parsed when the first lit pipeline is made (deferred.rs).
    pbr_shader: deferred::Module,
    /// Shadow layouts: plain, with the material (dissolve), skinned.
    shadow_layouts: [wgpu::PipelineLayout; 3],
    /// The shadow pass's slot (group 0 of every caster's pipeline).
    shadow_bind_layout: wgpu::BindGroupLayout,
    shadow_shader: deferred::Module,
    /// Lit pipelines, each made when a batch first needs it (or by
    /// [`Renderer::warm`]).
    pbr_pipelines: HashMap<PbrKey, wgpu::RenderPipeline>,
    shadow_pipelines: HashMap<ShadowKey, wgpu::RenderPipeline>,
    /// Prepass pipelines by (side, skinned).
    prepass_pipelines: HashMap<(Side, bool), wgpu::RenderPipeline>,
    /// The lean lit pipelines of plain materials, made at first use.
    lean_pipelines: HashMap<LeanKey, wgpu::RenderPipeline>,
    /// Their prepass pipelines by (side, derivatives, compact).
    lean_prepass_pipelines: HashMap<(Side, bool, bool), wgpu::RenderPipeline>,
    /// The shadow pipelines of compact casters (lean, from the compact
    /// vertices) by side.
    compact_shadow_pipelines: HashMap<Side, wgpu::RenderPipeline>,
    /// Stylise's object pass, made the first frame a pattern is laid in
    /// object space (`stylise_object.rs`).
    stylise_object: Option<stylise_object::ObjectPass>,
    sky_pipeline: Sampled<wgpu::RenderPipeline>,
    /// The procedural sky of a chosen look drawn per pixel ([`Background::Procedural`]).
    sky_look_pipeline: Sampled<wgpu::RenderPipeline>,
    water: water::WaterRenderer,
    fluid3d: fluid3d::Fluid3dRenderer,
    terrain: terrain::TerrainRenderer,
    scatter: scatter::ScatterRenderer,
    grass: grass::GrassRenderer,
    metaballs: metaballs::MetaballRenderer,
    /// The image-based light, regenerated into the same textures for a new sky.
    ibl: ibl::IblGenerator,
    /// The live sky (`sky_capture.rs`): made while the background is the
    /// per-pixel sky.
    sky_capture: Option<sky_capture::SkyCapture>,
    /// The probe GI (`gi_probes.rs`), made the first frame that asks for it.
    gi_probes: Option<gi_probes::GiProbes>,
    /// Reflection probes' cubes and frame uniforms (`probes.rs`).
    probes: probes::Probes,
    sprite_pipeline: Sampled<wgpu::RenderPipeline>,
    sprite_additive_pipeline: Sampled<wgpu::RenderPipeline>,
    /// Alpha-blended sprites kept out of the tone curve, and the cover
    /// that sets the HDR alpha to 1 before the first of them
    /// ([`UNTONEMAPPED_BLEND`]).
    sprite_untonemapped_pipeline: Sampled<wgpu::RenderPipeline>,
    sprite_cover_pipeline: Sampled<wgpu::RenderPipeline>,
    sprite_glow_pipeline: Deferred<wgpu::RenderPipeline>,
    msdf_pipeline: Sampled<wgpu::RenderPipeline>,
    ribbon_pipelines: [Sampled<wgpu::RenderPipeline>; 2],
    fx_layout: wgpu::BindGroupLayout,
    /// `fx_layout` without the planar reflection, where it has it (two
    /// planar reflections, [`planar_slot`]): the lightmapped pipelines'
    /// group 2, whose lightmap pages would otherwise be a seventeenth
    /// texture (`Renderer::fx_lean_group`). `None` with one: `fx_layout`
    /// serves them.
    fx_lean_layout: Option<wgpu::BindGroupLayout>,
    fx_buffer: wgpu::Buffer,
    /// The frame's decal records and their clusters (`fx.rs`), and the CPU
    /// vectors they are packed in, kept between frames.
    decal_buffers: fx::DecalBuffers,
    decal_records: Vec<fx::DecalGpu>,
    decal_spheres: Vec<(glam::Vec3, f32)>,
    decal_bins: clusters::DecalBins,
    sprite_fx_layout: wgpu::BindGroupLayout,
    sprite_fx_buffer: wgpu::Buffer,
    palette_layout: wgpu::BindGroupLayout,
    palette: skin::GpuPalette,
    skinned_buffer: wgpu::Buffer,
    prev_instance_buffer: wgpu::Buffer,
    /// What `prev_instance_buffer` holds ([`upload::DirtyUpload`]).
    prev_instance_upload: upload::DirtyUpload,
    ribbon_buffer: wgpu::Buffer,
    ribbon_sampler: wgpu::Sampler,
    /// Lines, points and wireframes (`lines.rs`).
    lines: lines::LineRenderer,
    particles: particles_gpu::ParticleRenderer,
    lights_2d: light2d::Light2dRenderer,
    trails: ribbons::Trails,
    /// Trails, ribbons and beams expanded on the GPU, with the trails'
    /// histories stepped in compute (`None`: the device lacks compute or
    /// storage in the vertex stage, and `trails` builds them on the CPU).
    gpu_trails: Option<ribbons_gpu::GpuTrails>,
    surfaces: Option<surface::SurfacePipelines>,
    garments: Option<garment_gpu::Pipelines>,
    /// The previous frame's unjittered view-projection (motion vectors).
    prev_view_proj: Option<Mat4>,
    /// The last drawn clock, including frames with no simulation tick.
    previous_time: Option<f32>,
    /// The last drawn wind, tick and tick rate, including paused frames.
    previous_wind: Option<(Wind, u64, f32)>,
    /// Each instance's sway at the last drawn frame, by instance index
    /// ([`Instance::sway`]): a swaying vertex's motion vector starts there.
    previous_sway: Vec<f32>,
    /// What the shadow layers keep from frame to frame (`shadow_cache.rs`),
    /// and per shadow slot the texture a layer's static casters are kept
    /// in while dynamic ones are drawn over it.
    shadow_cache: shadow_cache::ShadowCache,
    shadow_cache_maps: Vec<Option<CachedShadow>>,
    /// A second water surface asked for a planar reflection the renderer
    /// does not draw ([`RenderDesc::planar_reflections`] 1), and the log
    /// said so once.
    warned_planar: bool,
    fx_groups: Option<FxGroups>,
    mips: mips::MipGenerator,
    /// The physical lit pipelines, made when a scene first draws a physical
    /// material (`physical.rs`).
    physical: Option<physical::PhysicalPipelines>,
    /// The array-map variant of the lit, prepass and shadow pipelines, made
    /// when a scene first draws a material with array maps (`material.rs`).
    array_maps: Option<material::ArrayMaps>,
    /// The copy of the opaque scene transmission refracts, with its mip
    /// chain, at the renderer's size (made when a frame first needs it),
    /// its view and the chain's views and bind groups (made with it, not
    /// every frame).
    transmission: Option<(wgpu::Texture, wgpu::TextureView, mips::MipChain)>,
    /// The frame bind group (camera, shadow maps, IBL) and the sky pass's
    /// (the sky cube) for the sky they were generated from.
    environment: Option<(
        EnvironmentKey,
        wgpu::BindGroup,
        wgpu::BindGroup,
        Vec<wgpu::BindGroup>,
    )>,
    default_white: wgpu::TextureView,
    default_normal: wgpu::TextureView,
    white_sprite_group: wgpu::BindGroup,
    instance_buffer: wgpu::Buffer,
    /// What `instance_buffer` holds, so a frame writes only what changed.
    instance_upload: upload::DirtyUpload,
    sprite_buffer: wgpu::Buffer,
    /// The instances of the sprite-mode particles simulated in compute
    /// (`particle_sprites.wgsl` writes them; `RunSource::Particles` runs
    /// draw them), and what each range of this frame draws with.
    particle_sprite_buffer: wgpu::Buffer,
    particle_sprite_sources: Vec<RunSource>,
    overlay: overlay::OverlayRenderer,
    /// The UI pass's pipelines, tables and layers (src/ui/pass.rs).
    ui: ui::pass::UiRenderer,
    /// The overlay's glass: its passes, and its textures for the output's size.
    glass: glass::GlassPasses,
    glass_targets: Option<glass::GlassTargets>,
    cache: SceneCache,
    /// Frames rendered so far.
    frame_index: u64,
    /// Compiled pipelines kept across runs (`pipeline_cache.rs`), when kept.
    pipeline_disk: Option<pipeline_cache::DiskCache>,
    /// Where particles simulate ([`RenderDesc::particles`]).
    desc_particles: ParticleSimulation,
    /// Where instances are culled ([`RenderDesc::culling`]), whether the
    /// adapter can on the GPU (compute and indirect draws), and the GPU
    /// path's state once a frame used it.
    desc_culling: Culling,
    indirect: bool,
    gpu_cull: Option<cull_gpu::GpuCulling>,
    /// The meshlet path (meshlet_gpu.rs), once a scene has a clustered mesh.
    meshlet: Option<meshlet_gpu::MeshletGpu>,
    /// Whether the next frame's GPU culling counts are read back
    /// ([`Renderer::request_culling_counts`]).
    culling_counts: bool,
    /// The previous frame's unjittered view-projection, when TAA history is valid.
    taa_history: Option<Mat4>,
    /// Frames TAA has accumulated into its history since it was last
    /// reset, this one included (0 without TAA): [`Renderer::history`].
    taa_history_frames: u64,
    /// The vegetation's simulated wind field (`wind_field.rs`).
    wind_field: wind_field::WindFieldSim,
    /// The grass trample map (`trample.rs`).
    trample: trample::Trample,
    /// The frame whose lit image screen-space GI's copy holds
    /// (`Persistent::gi`, without TAA); forgotten with the TAA history.
    gi_history: Option<u64>,
    /// The scene origin the last frame was drawn at ([`RenderScene::origin`]):
    /// the history above is in its coordinates.
    drawn_origin: Vec3,
    /// The last frame's stylisation primitives, on or off
    /// ([`Stylise::chain_bits`]): a change resets the TAA history.
    stylise_chain: Option<u16>,
    /// The array sprite pipelines, made on the first frame that draws one.
    sprite_arrays: Option<sprite_array::SpriteArrayPipelines>,
    /// The sprite mesh pipelines and buffer, made on the first frame that draws one.
    sprite_meshes: Option<sprite_mesh::SpriteMeshRenderer>,
    sprite_skin_supported: bool,
    /// The normal-mapped sprite pipelines and texture groups, made on the
    /// first frame that draws a normal-mapped texture lit.
    sprite_normals: Option<sprite_normals::SpriteNormals>,
    tiles_gpu: Option<tiles_gpu::Tiles>,
    tiles_gpu_supported: bool,
    /// Order-independent transparency's mode, arena, resolve and
    /// pipelines (`oit.rs`).
    oit: oit::OitState,
    /// The sprite and strip modules, for their OIT pipelines.
    sprite_shader: deferred::Module,
    ribbon_shader: deferred::Module,
}
