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

#![recursion_limit = "256"]
#![cfg_attr(test, feature(custom_test_frameworks, test))]
#![cfg_attr(test, test_runner(gpu_tests::run))]

#[cfg(test)]
use wgpu as gpu_wgpu;
#[cfg(test)]
#[path = "../../../tests/gpu.rs"]
mod gpu_tests;

/// A file of `shaders/` as the renderer compiles it: the copy build.rs
/// wrote with its `//` comments removed (every token and line kept), so
/// the comments stay in the sources and out of the wasm.
macro_rules! wgsl {
    ($file:literal) => {
        include_str!(concat!(env!("OG_RENDER_SHADERS"), "/", $file))
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

mod fields;
mod gpu;
mod media;
mod mesh;
mod renderer;
mod shader;
mod shading;
mod systems;
mod visibility;
use gpu::indirect_own;
pub use renderer::*;

pub use systems::environment::wind_field;
pub use systems::vegetation::trample;
pub use trample::GrassPusher;
pub use wind_field::{WindMotor, WindMotorKind};

use debug::capture;
use frame::auto_quality;
use media::clouds;
use mesh::compact;
pub use og_draw::{ColorFilter, ColorVision, ColorVisionFilter};
use og_render_data::custom;
use post::color_vision;
use shader::blue_noise;
use shading::area;
use shading::clusters;
use systems::crowd;
use systems::crowd::crowd_gpu;
use systems::crowd::crowd_impostor;
use systems::destruction::debris;
use systems::environment::atmosphere;
use systems::environment::aurora;
use visibility::cull_gpu;
pub mod debug;
pub use deferred::settle_pipelines;
use gpu::deferred;
use particles::effect;
use scene::effects;
use shading::environment;
use systems::fluids::fluid3d;
pub use systems::vegetation::foliage;
mod frame;
use debug::id_pass;
use frame::fx;
pub use frame::graph;
use frame::origin;
use frame::quality;
pub use gi_probes::ProbeGlobalIllumination;
use gpu::mips;
use mesh::lines;
pub use og_render_data::geometry;
use og_render_data::hooks;
pub use og_render_data::meshlet;
pub use oit::{
    DEFAULT_TRANSPARENCY_LAYERS, MAX_TRANSPARENCY_LAYERS, Transparency, TransparencyCounts,
    weighted_order_bound, weighted_order_codes,
};
use post::glass;
use post::hdr;
pub use probe_content::{ProbeContent, ProbeRoom};
pub use probe_grid::{MAX_GRID_CAPTURES, RealtimeLightVolume};
pub use probes::{
    MAX_REFLECTION_PROBES, PROBE_MAX_RESOLUTION, PROBE_MIN_RESOLUTION, ProbeCapture, ProbeMode,
    ReflectionProbe,
};
use shader::pipeline_features;
use shading::gi_probes;
use shading::gi_scene;
use shading::ibl;
use shading::lightmap;
use shading::material;
use shading::oit;
use shading::probe_content;
use shading::probe_grid;
use shading::probe_select;
use shading::probes;
use shading::ssgi;
use shading::ssr;
pub use ssgi::ScreenSpaceGlobalIllumination;
pub use ssr::ScreenSpaceReflections;
use systems::environment::sky_capture;
use systems::fluids::metaballs;
use systems::sprites::light2d;
use systems::sprites::overlay;
use visibility::instances;
pub use visibility::lod;
use visibility::meshlet_gpu;
pub mod particles;
use gpu::pipeline_cache;
use particles::particles_gpu;
pub use pipeline_cache::set_pipeline_cache_dir;
pub use pipeline_features::LitFeatures;
pub use systems::ui;
pub use ui::{
    UI_GRADIENT_STOPS, UI_MASS_KEYS, UI_MASS_QUADS, UI_MATERIAL_PARAMS, UI_QUAD_LAYER_MAX, UiAlign,
    UiBackground, UiBackgroundSize, UiBlend, UiBorderImage, UiBuiltin, UiCap, UiClip,
    UiColorMatrix, UiCorner, UiCssCorners, UiDrape, UiEdge, UiElevation, UiFillRule, UiFilter,
    UiFilterComposite, UiFilterInput, UiFilterOp, UiFilterPrimitive, UiFit, UiGeometry, UiGlass,
    UiGlassFallback, UiGlassShadow, UiGlassStyle, UiGlassWeight, UiGradient, UiGradientImage,
    UiGradientKind, UiGround, UiImage, UiJoin, UiLayer, UiLight, UiList, UiMask, UiMassBatch,
    UiMassInstance, UiMassKey, UiMassKind, UiMassSpace, UiMassTemplate, UiMaterial,
    UiMaterialHandle, UiMaterialShader, UiMode, UiOccluder, UiOcclusion, UiPaint, UiParticleShape,
    UiPath, UiRepeat, UiShape, UiSpread, UiStop, UiStroke, UiTextEffects, UiTextOutline,
    UiTextShadow, UiTransfer, UiVectorData, UiVectorDraw, UiVectorHandle, UiVectorPaint,
    UiVectorStroke, UiVectorVertex, UiVerb, example_ui_material, flatten_cubic, flatten_quad,
    format_number, oklab, oklab_to_linear, square_to_quad, ui_material_module_source, ui_scrim,
};

thread_local! {
    /// Fork s60: the host's hook around the frame pipelines
    /// ([`set_frame_pipelines_hook`]).
    static FRAME_PIPELINES_HOOK: std::cell::Cell<Option<fn(bool)>> = const { std::cell::Cell::new(None) };
    /// How many [`frame_pipelines`] spans are open (only the outermost calls the hook).
    static FRAME_PIPELINES_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Fork s60: `hook(true)` before the renderer makes its frame pipelines
/// (the materials' lit, shadow and prepass pipelines, the blended
/// surfaces' order-independent transparency ones and its resolve, the
/// particles', [`Renderer::warm`]'s) or encodes the passes it draws again
/// every frame (which make their pipelines when they first set them,
/// deferred.rs), and
/// `hook(false)` after, on this thread; `None` (the default) calls nothing.
/// The web host makes the render pipelines asked for in between with
/// `createRenderPipelineAsync` and, in those passes, skips the draws of one
/// not made yet (packages/engine/src/web/pipelines.ts), so a frame that
/// meets dozens of new pipelines does not hold the browser's GPU process
/// while they compile. The passes whose output later frames keep (shadow
/// maps, captures, IBL, mips) are encoded outside it and make theirs at
/// once: their draws must not be skipped.
pub fn set_frame_pipelines_hook(hook: Option<fn(bool)>) {
    FRAME_PIPELINES_HOOK.with(|c| c.set(hook));
}

/// Runs `f`, which makes frame pipelines, inside the host's hook.
fn frame_pipelines<R>(f: impl FnOnce() -> R) -> R {
    let _span = FrameSpan::open();
    f()
}

/// Fork s60: the host's frame span, open while this lives (only the
/// outermost calls the hook). What is made and encoded inside is the
/// frame's: the web host makes its render pipelines asynchronously and
/// skips a pass's draws while their pipeline compiles, so only passes whose
/// output is drawn again next frame are encoded in one
/// (`FramePass::redrawn`); a pass outside makes a pipeline still
/// compiling at once instead.
struct FrameSpan(Option<fn(bool)>);

impl FrameSpan {
    fn open() -> FrameSpan {
        let hook = FRAME_PIPELINES_HOOK.with(|c| c.get());
        if let Some(hook) = hook
            && FRAME_PIPELINES_DEPTH.with(|d| d.replace(d.get() + 1)) == 0
        {
            hook(true);
        }
        FrameSpan(hook)
    }
}

impl Drop for FrameSpan {
    fn drop(&mut self) {
        if let Some(hook) = self.0
            && FRAME_PIPELINES_DEPTH.with(|d| {
                d.set(d.get() - 1);
                d.get()
            }) == 0
        {
            hook(false);
        }
    }
}

/// Fork s63: the texture bytes a frame uploads at most (0: no limit, the
/// default). A texture past it is drawn from a 1x1 stand-in (mid grey and
/// transparent for colour, a flat normal for a two-channel map, white
/// occlusion and roughness for another linear one) and uploaded by a later
/// frame, as a replaced texture is; a frame always uploads at least one.
static UPLOAD_BUDGET: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// Fork s63: the textures still drawn from their stand-ins after the last frame.
static TEXTURES_DEFERRED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
/// The revision a deferred texture's slot holds: never a scene's.
const DEFERRED_REVISION: u64 = u64::MAX;

/// Set fork s63's per-frame texture upload budget in bytes (0: none).
pub fn set_upload_budget(bytes: u64) {
    UPLOAD_BUDGET.store(bytes, std::sync::atomic::Ordering::Relaxed);
}

/// Fork s63: the textures waiting for a later frame's upload.
pub fn textures_deferred() -> u32 {
    TEXTURES_DEFERRED.load(std::sync::atomic::Ordering::Relaxed)
}

/// Fork s70: the page covers the surface (a loading screen over the canvas),
/// so a frame to it prepares what it would draw (the scene's buffers and
/// texture uploads, the pipelines its batches need, the frame graph) and
/// encodes, submits and presents nothing.
static COVERED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Set fork s70's covered surface (the web host: while the page's loading
/// screen hides the canvas). Frames to an offscreen target or a view draw as
/// ever.
pub fn set_covered(on: bool) {
    COVERED.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// The bytes a texture source uploads (a render target: none).
fn source_upload_bytes(source: &TextureSource) -> u64 {
    match source {
        TextureSource::Rgba(t) => t.rgba.len() as u64,
        TextureSource::Encoded(u) => u.levels.iter().map(|l| l.len() as u64).sum(),
        TextureSource::Target(_) => 0,
    }
}
use mesh::points;
use shading::physical;
use systems::vegetation::grass;
use systems::vegetation::impostor;
mod post;
use particles::ribbons;
use particles::ribbons_gpu;
use post::post_custom;
use systems::vegetation::scatter;
use systems::vegetation::scatter_cull;
use systems::vegetation::scatter_place;
mod scene;
pub use caps::{Caps, Tier};
use frame::timing;
use frame::views;
pub use garment::{GarmentDeformation, GarmentTopology};
pub use gpu::caps;
use gpu::targets;
use gpu::texture;
use gpu::texture_array;
use gpu::video;
use media::volumetric;
use mesh::skin;
pub use og_render_data::shapes;
use og_render_data::terrain_error;
pub use og_render_data::{Capability, Holds};
use particles::scene_colliders;
use shader::shaders;
pub use shaders::{ShaderSource, UNBOUNDED_MODULES, shader_sources};
use shading::shadow_cache;
use shading::shadow_fit;
use shading::shadow_keep;
use shading::shadow_target;
use shading::shadows;
pub use surface::SimulationSurface;
use systems::environment::sky_physical;
use systems::environment::weather;
use systems::environment::wind;
use systems::fluids::visual_fluid;
use systems::softbody::garment;
use systems::softbody::garment_gpu;
use systems::softbody::surface;
use systems::softbody::visual_sim;
use systems::sprites;
use systems::sprites::sprite_array;
use systems::sprites::sprite_hooks;
use systems::sprites::sprite_mesh;
use systems::sprites::sprite_normals;
use systems::sprites::sprite_skin;
use systems::sprites::tiles_gpu;
use systems::stylise;
use systems::stylise::stylise_object;
use systems::terrain;
use systems::terrain::terrain_tiles;
use systems::water;
pub use systems::water::ocean_fft;
use systems::water::sea_wakes;
use systems::water::water_ripples;
use systems::water::water_river;
use visibility::upload;
pub use visual_fluid::{VisualFluid, at_rest as visual_fluid_at_rest};
pub use visual_sim::{VisualBody, VisualGarment};

#[cfg(not(target_arch = "wasm32"))]
use gpu::headless;
#[cfg(any(target_os = "macos", target_os = "linux", windows))]
pub use gpu::shared_frames;
#[cfg(feature = "gltf")]
pub use mesh::gltf;

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
pub use debris::{
    CHIP_VARIANTS, DEBRIS_POINTS, DEBRIS_SETTLE_TICKS, DebrisPiece, MAX_DEBRIS, chip,
};
pub use effect::{
    EFFECT_DRIVERS, EFFECT_PARAMS, EffectChannel, EffectMaterial, EffectMaterialHandle, EffectMesh,
    EffectRing, EffectTexture, EffectUv, EffectValue, Erosion, Flow, Fresnel, effect_cylinder,
    effect_ring,
};
pub use effects::*;
#[cfg(feature = "bake")]
pub use environment::bake;
pub use environment::{
    ENVIRONMENT_BLEND_STEPS, ENVIRONMENT_MAP_LEVELS, ENVIRONMENT_MAP_MAX_SIZE,
    ENVIRONMENT_MAP_MIN_SIZE, EnvironmentBlend, EnvironmentEquirect, EnvironmentMap,
    EnvironmentMapData, EnvironmentSun, EquirectIrradiance, face_direction, rgb9e5, rgb9e5_decode,
    sh9,
};
pub use fluid3d::Fluid3d;
pub use graph::{GraphError, GraphPlan, RenderGraph, TargetDesc};
pub use grass::{DistanceLod, Grass, GrassDraw, GrassType};
#[cfg(not(target_arch = "wasm32"))]
pub use headless::render_to_rgba;
pub use hooks::{
    MAX_SHADER_PARAMS, MaterialShader, ParamKind, SHADER_HOOKS, ShaderMode, ShaderParam,
    lit_module_source, physical_module_source, shadow_module_source, sprite_module_source,
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
    Attractor, MAX_ATTRACTORS, MAX_FIELD_CELLS, MAX_SPLINE_POINTS, VectorField, shape_points,
    spline_points,
};
pub use particles::{
    Burst, ChildParticles, CollisionTarget, ColorCurve, CpuParticles, Curve, EmitterHandle,
    EmitterShape, MAX_CURVE_KEYS, MAX_SUB_EMITTERS, ParticleAlign, ParticleBlend, ParticleClock,
    ParticleCollision, ParticleEmitter, ParticleEvent, ParticleEventKind, ParticleHistory,
    ParticleNoise, ParticleRender, ParticleSimulation, ParticleState, ParticleSystem, ParticleTick,
    SceneCollider, SheetMode, SimulationSpace, SpriteSheet, SubEmitter, SubTrigger, mesh_points,
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
pub use crowd::{
    Crowd, CrowdImpostor, CrowdMember, CrowdPart, MAX_CROWD_LEVELS, drawn_level, member_level,
};
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
    DEFAULT_TOON_SOFTNESS, DEFAULT_WIREFRAME_ANGLE, DepthBands, DitherPattern, Halftone,
    HalftoneMode, HalftoneRegion, Hatching, Ink, MAX_PALETTE_COLORS, MAX_WASH_DILUTION,
    MaterialStyle, MaterialSurface, OutlinePlace, Paint, Palette, PaletteHandle, PixelFit,
    PixelScale, PosterizeMode, Quantize, Retro3d, STYLISE_DRAWN, Stylise, StyliseSpace, Watercolor,
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
pub use water::{WAKE_TICKS, Water, Water2d, WaterCut, WaterHeights, WaterShore, WaterWakes};
pub use weather::{
    BOLT_CORE, SurfaceWeather, WEATHER_DUST, WEATHER_MIST, WEATHER_NONE, WEATHER_RAIN,
    WEATHER_SNOW, WeatherLook, add_lightning, lightning_bolt, lightning_branches, lightning_flash,
    weather,
};
/// The wgpu this crate is built on, for hosts that create surfaces or pass views.
pub use wgpu;

use deferred::{AsModule, Deferred, Sampled};
use frame::{CASCADES, Frustum};
use glam::{Mat4, Vec2, Vec3};
use material::MaterialUniform;
use post::FramePass;
use shadow_fit::Cascade;
use shadow_target::{LOCAL_SHADOW_FORMAT, ShadowSet, ShadowTarget};
use shadows::LOCAL_SHADOW_LAYERS;
use sprites::{FontLayout, RunSource, SPRITE_LAYOUT, SPRITE_STRIDE};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// A buffer holding `desc.contents`, as `create_buffer_init` makes it.
///
/// On the web (fork s71) it is created unmapped and written with
/// `queue.writeBuffer` straight from wasm memory: `mappedAtCreation` copies
/// the bytes into a JS ArrayBuffer first, and Chrome's mapped-at-creation
/// path was most of the long GPU-process tasks of a large boot. The write
/// lands before the next submission, so every use of the buffer sees it.
/// A buffer that maps again (`MAP_READ` or `MAP_WRITE`) and natively, it is
/// `create_buffer_init`. Every buffer with initial contents in og-render is
/// made here (clippy.toml refuses `create_buffer_init`).
pub(crate) fn buffer_init(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    desc: &wgpu::util::BufferInitDescriptor,
) -> wgpu::Buffer {
    #[cfg(target_arch = "wasm32")]
    if !desc.contents.is_empty()
        && !desc
            .usage
            .intersects(wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE)
    {
        // The size create_buffer_init gives it: the contents rounded up to
        // COPY_BUFFER_ALIGNMENT, the padding zero.
        let align = wgpu::COPY_BUFFER_ALIGNMENT as usize;
        let whole = desc.contents.len() / align * align;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: desc.label,
            size: desc.contents.len().div_ceil(align) as u64 * wgpu::COPY_BUFFER_ALIGNMENT,
            usage: desc.usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        if whole > 0 {
            queue.write_buffer(&buffer, 0, &desc.contents[..whole]);
        }
        if whole < desc.contents.len() {
            let mut tail = [0u8; wgpu::COPY_BUFFER_ALIGNMENT as usize];
            tail[..desc.contents.len() - whole].copy_from_slice(&desc.contents[whole..]);
            queue.write_buffer(&buffer, whole as u64, &tail);
        }
        return buffer;
    }
    let _ = queue;
    #[allow(clippy::disallowed_methods)]
    device.create_buffer_init(desc)
}

/// The renderer. Create it once per output, call [`Renderer::render`] per frame.
pub struct Renderer {
    adapter: wgpu::Adapter,
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
    /// The instance records every pass that draws meshes reads: this
    /// frame's and the previous frame's, the skinned instances', the joint
    /// palette, the sway and the debris moved on the GPU.
    instances: renderer::InstanceRecords,
    /// The secondary cameras' GPU state (`views.rs`).
    views: views::Views,
    /// The custom post passes on the device (`post_custom.rs`).
    custom_posts: post_custom::CustomPosts,
    /// Visibility: where instances are culled, and the GPU culling, indirect
    /// draws and meshlets every view registers into.
    visibility: renderer::Visibility,
    /// The shadow passes' state: the cascade and local shadow arrays, their
    /// uniforms, pipelines and caches, the shelter map's key and the light
    /// cookies.
    shadows: renderer::Shadows,
    post: post::PostPasses,
    material_sampler: wgpu::Sampler,
    /// The sprites' resources: their samplers and layouts, pipelines (plain, additive, untonemapped, cover, glow, text), effects uniform and instance buffer, the white group, and the optional array, mesh, normal, hook and GPU tile renderers.
    sprite: renderer::SpriteState,
    /// The font atlases', the decal atlas's and the 2D lightmap's:
    /// trilinear, clamped, never anisotropic.
    trilinear_sampler: wgpu::Sampler,
    frame_layout: wgpu::BindGroupLayout,
    /// The sky's resources: its layout, cloud noise, parameters, aurora map, its two pipelines (the sky cube's and the procedural look's) and the live sky's capture.
    sky: renderer::SkyState,
    material_layout: wgpu::BindGroupLayout,
    /// The frame's lighting every lit pass binds: the frame uniform, the light
    /// tables and clusters, the light volumes, the IBL and environment, the
    /// decals, and the effect buffer and groups.
    lights: renderer::Lights,
    /// Volumetric fog (made on first use).
    volumetric: volumetric::Volumetric,
    /// The planar reflections' per-slot buffers and clusters: their frame and effects uniforms, their light tables and the clusters the mirrored cameras build, and whether a missing planar slot was reported.
    reflections: renderer::ReflectionTargets,
    /// The lit passes' pipelines: the standard, lean and physical ones by key,
    /// their layouts and shader, the array maps' variants, and the features a
    /// scene used that the renderer was made without.
    lit: renderer::LitPipelines,
    /// The shadow pass's slot (group 0 of every caster's pipeline).
    shadow_bind_layout: wgpu::BindGroupLayout,
    /// Lit pipelines, each made when a batch first needs it (or by
    /// [`Renderer::warm`]).
    /// The features this frame's lit passes draw with
    /// (`pipeline_features::Features::of`; every one in a capture).
    frame_features: pipeline_features::Features,
    /// Drawing a capture ([`Renderer::render_capture`]): its lit pipelines
    /// are the full variants, waited for, so its bytes never depend on
    /// which variants a compile worker has finished.
    capturing: bool,
    /// The depth prepass's pipelines: the standard and the lean ones by key,
    /// and stylise's object pass.
    prepass: renderer::PrepassState,
    /// The 3D world's renderers: water, 3D fluids, terrain, scattered copies, grass, metaballs, the wind field and trample map, the visual simulations, surfaces and garments, and the crowds' GPU path and impostors.
    world: renderer::WorldRenderers,
    /// The probe GI (`gi_probes.rs`), made the first frame that asks for it.
    gi_probes: Option<gi_probes::GiProbes>,
    /// Reflection probes' cubes and frame uniforms (`probes.rs`).
    probes: probes::Probes,
    /// The strips' resources: their two pipelines, shader, vertex buffer and sampler, and the CPU and GPU trails.
    strips: renderer::StripState,
    fx_layout: wgpu::BindGroupLayout,
    /// `fx_layout` without the planar reflection, where it has it (two
    /// planar reflections, [`planar_slot`]): the lightmapped pipelines'
    /// group 2, whose lightmap pages would otherwise be a seventeenth
    /// texture (`Renderer::fx_lean_group`). `None` with one: `fx_layout`
    /// serves them.
    fx_lean_layout: Option<wgpu::BindGroupLayout>,
    palette_layout: wgpu::BindGroupLayout,
    /// Lines, points and wireframes (`lines.rs`).
    lines: lines::LineRenderer,
    particles: renderer::Particles,
    lights_2d: light2d::Light2dRenderer,
    /// What the frame carries to the next: the previous camera, clock and
    /// wind (motion vectors), TAA's history, and the origin it is in.
    history: renderer::history::FrameHistory,
    mips: mips::MipGenerator,
    /// The copy of the opaque scene transmission refracts, with its mip
    /// chain, at the renderer's size (made when a frame first needs it),
    /// its view and the chain's views and bind groups (made with it, not
    /// every frame).
    transmission: Option<(wgpu::Texture, wgpu::TextureView, mips::MipChain)>,
    default_white: wgpu::TextureView,
    /// The conformed colours' hash table the stylise quantiser reads
    /// (`stylise::conformed_table`) and the scene revision it was made at.
    conformed: (u64, wgpu::TextureView),
    /// The paint canvas's mean height and spread (linear red; the
    /// watercolour wash reads the paper's relief in those units), and the
    /// texture slot and revision they were measured from.
    canvas_stats: (Option<(usize, u64)>, [f32; 2]),
    default_normal: wgpu::TextureView,
    overlay: overlay::OverlayRenderer,
    /// The UI pass's pipelines, tables and layers (src/ui/pass.rs).
    ui: ui::pass::UiRenderer,
    /// The overlay's glass: its passes, and its textures for the output's size.
    glass: glass::GlassPasses,
    glass_targets: Option<glass::GlassTargets>,
    cache: SceneCache,
    /// Frames rendered so far.
    frame_index: u64,
    /// `frame_index` at the last [`Renderer::render_capture`]: the
    /// temporal phase ([`Renderer::phase`]) counts from it.
    phase_base: u64,
    /// The frame histories kept for world snapshots (`renderer/history.rs`).
    saves: renderer::history::Saves,
    /// Compiled pipelines kept across runs (`pipeline_cache.rs`), when kept.
    pipeline_disk: Option<pipeline_cache::DiskCache>,
    /// The frame whose lit image screen-space GI's copy holds
    /// (`Persistent::gi`, without TAA); forgotten with the TAA history.
    gi_history: Option<u64>,
    /// Screen-space reflections' passes, targets and history (`ssr.rs`).
    ssr: ssr::SsrRenderer,
    /// The last frame's stylisation primitives, on or off
    /// ([`Stylise::chain_bits`]): a change resets the TAA history.
    stylise_chain: Option<u16>,
    /// Order-independent transparency's mode, arena, resolve and
    /// pipelines (`oit.rs`).
    oit: oit::OitState,
}
