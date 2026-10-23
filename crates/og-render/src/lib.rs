//! Origin Engine renderer (`docs/design.md` §2.6, `crates/og-render/README.md`).
//!
//! A wgpu Forward+ renderer: PBR metallic-roughness, one directional light
//! with three cascaded shadow maps, up to 256 point and spot lights in view,
//! clustered (`clusters.rs`; up to eight of them shadowed, four on WebGL2, `shadows.rs`), split-sum image-based lighting generated from a procedural sky,
//! antialiasing by tier (TAA on WebGPU-class backends, 4x MSAA or FXAA on WebGL2),
//! bloom, manual or metered exposure, ACES tonemapping, and a 2D batch of
//! sprites and MSDF text. It runs natively (Metal, Vulkan; headless for
//! executors) and in the browser (WebGPU, or WebGL2 through wgpu's GL
//! backend) from the same code.
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

mod atmosphere;
mod blue_noise;
mod capture;
mod clusters;
mod compact;
mod cull_gpu;
mod custom;
pub mod debug;
mod deferred;
mod effects;
mod environment;
mod fluid3d;
mod frame;
mod fx;
pub mod geometry;
mod glass;
pub mod graph;
mod hooks;
mod ibl;
mod id_pass;
mod instances;
mod light2d;
mod lines;
mod material;
mod metaballs;
mod mips;
mod overlay;
pub mod particles;
mod particles_gpu;
mod pipeline_cache;
pub use pipeline_cache::set_pipeline_cache_dir;
mod physical;
mod points;
mod post;
mod post_custom;
mod ribbons;
mod scatter;
mod scene;
mod shaders;
pub use shaders::{ShaderSource, shader_sources};
mod shadows;
pub mod shapes;
mod skin;
mod sprite_array;
mod sprite_mesh;
mod sprites;
mod stylise;
mod stylise_object;
mod targets;
mod terrain;
mod terrain_tiles;
mod texture;
mod texture_array;
mod upload;
mod video;
mod views;
mod water;
mod weather;

#[cfg(feature = "gltf")]
pub mod gltf;
#[cfg(not(target_arch = "wasm32"))]
mod headless;

pub use atmosphere::{DayNightLook, Fog, SkyLook, day_night};
pub use capture::{CAPTURE_TIMEOUT, CaptureRequest};
pub use clusters::{CLUSTER_GRID, MAX_CLUSTER_LIGHTS, UNIFORM_LIGHT_REFS};
pub use custom::{FULL_SHADER_ENTRIES, example as example_full_shader};
pub use effects::*;
#[cfg(feature = "bake")]
pub use environment::bake;
pub use environment::{
    ENVIRONMENT_MAP_LEVELS, ENVIRONMENT_MAP_MAX_SIZE, ENVIRONMENT_MAP_MIN_SIZE, EnvironmentMap,
    EnvironmentMapData, face_direction, rgb9e5, rgb9e5_decode, sh9,
};
pub use fluid3d::Fluid3d;
pub use graph::{GraphError, GraphPlan, RenderGraph, TargetDesc};
#[cfg(not(target_arch = "wasm32"))]
pub use headless::render_to_rgba;
pub use hooks::{
    MAX_SHADER_PARAMS, MaterialShader, ParamKind, SHADER_HOOKS, ShaderMode, ShaderParam,
    lit_module_source, physical_module_source, shadow_module_source,
};
pub use id_pass::{IdFrame, IdObject, IdSources, id_pass_size, projection_name, render_ids};
pub use lines::{LineLayer, LineMode, Lines, wire_edges};
pub use material::{PhysicalExtensions, Side, StandardMaterial};
pub use metaballs::{FIELDS, MAX_FLUIDS, Metaballs};
pub use particles::{
    Burst, ColorCurve, Curve, EmitterHandle, EmitterShape, MAX_CURVE_KEYS, ParticleAlign,
    ParticleBlend, ParticleCollider, ParticleCollision, ParticleEmitter, ParticleNoise,
    ParticleRender, ParticleSimulation, ParticleState, ParticleSystem, SheetMode, SimulationSpace,
    SpriteSheet,
};
pub use points::Points;
pub use post_custom::{MAX_CUSTOM_POSTS, PostHandle, PostShader, example_crt, post_module_source};
pub use ribbons::{Beam, Ribbon, RibbonPoint, RibbonUv, Trail, TrailStyle, TrailStyleHandle};
pub use scatter::ScatterDraw;
pub use scene::*;
pub use shadows::PointShadows;
pub use skin::{
    JointPalette, MAX_JOINTS, MAX_MORPH_TARGETS, MorphTargets, SkinData, SkinnedInstance,
};
pub use sprite_mesh::{SpriteMesh, SpriteMeshVertex};
pub use stylise::{
    DEFAULT_CANVAS_WORLD_SCALE, DepthBands, DitherPattern, Halftone, Hatching, Ink,
    MAX_PALETTE_COLORS, MaterialStyle, Paint, Palette, PaletteHandle, PixelFit, PixelScale,
    Quantize, Retro3d, STYLISE_DRAWN, Stylise, StyliseSpace,
};
pub use targets::{RenderTargetDesc, RenderTargetHandle, TargetFormat, TargetKind, TargetTexture};
pub use terrain::{
    DEFAULT_TERRAIN_BUDGET, MAX_SCATTER_INSTANCES, MAX_TERRAIN_LAYERS, MAX_TERRAIN_LEVELS, Terrain,
    TerrainControl, TerrainData, TerrainLayer, TerrainLayerArrays, TerrainLod, TerrainSampler,
    TerrainScatter,
};
pub use terrain_tiles::{
    ControlPyramid, MAX_TERRAIN_PYRAMID, MAX_TERRAIN_TILES, TERRAIN_TILE, TerrainLayout,
    TerrainTile, TerrainView, terrain_tiles_wanted, tile_heights,
};
pub use texture::{TextureCompression, TextureDesc, TextureFormat, TextureFormats, TextureUpload};
pub use texture_array::{TextureArrayData, TextureArrayHandle, TextureArrayInfo};
pub use video::FrameTexture;
pub use views::{MirrorQuad, View, ViewTarget};
pub use water::{Water, Water2d, WaterHeights};
pub use weather::{
    WEATHER_DUST, WEATHER_MIST, WEATHER_NONE, WEATHER_RAIN, WEATHER_SNOW, WeatherLook, weather,
};
/// The wgpu this crate is built on, for hosts that create surfaces or pass views.
pub use wgpu;

use deferred::{AsModule, Deferred};
use frame::{CASCADES, Cascade, Frustum};
use glam::{Mat4, Vec2, Vec3};
use material::MaterialUniform;
use post::FramePass;
use shadows::LOCAL_SHADOW_LAYERS;
use sprites::{FontLayout, RunSource, SPRITE_LAYOUT, SPRITE_STRIDE};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// The linear HDR colour target every lit pass draws into.
pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
/// The default side of each directional light cascade, in texels
/// ([`RenderDesc::shadow_map_size`]).
pub const SHADOW_SIZE: u32 = 2048;
/// The cascade size the [`RenderDesc::mobile`] preset uses (a quarter of the memory).
pub const MOBILE_SHADOW_SIZE: u32 = 1024;
/// The most cascades the directional light can have.
pub const MAX_CASCADES: u32 = CASCADES as u32;
/// Side of each face of a local light's shadow map, in texels.
pub const LOCAL_SHADOW_SIZE: u32 = 512;
/// The format of the offscreen target when [`RenderDesc::output_format`] is unset.
pub const DEFAULT_OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
/// MSAA sample count when MSAA is on.
pub const MSAA_SAMPLES: u32 = 4;

const FRAME_WGSL: &str = wgsl!("frame.wgsl");
/// The lit shader's vertex stages, lights and decals; the standard
/// fragment shader and the prepass's follow it (`pbr_fs.wgsl`,
/// `pbr_prepass.wgsl`), the physical one replaces both (`physical.wgsl`).
const PBR_WGSL: &str = wgsl!("pbr.wgsl");
const PBR_FS_WGSL: &str = wgsl!("pbr_fs.wgsl");
const PBR_PREPASS_WGSL: &str = wgsl!("pbr_prepass.wgsl");
/// The lean entry points plain materials draw through ([`GpuMaterial::lean`]).
const PBR_LEAN_WGSL: &str = wgsl!("pbr_lean.wgsl");
const PHYSICAL_WGSL: &str = wgsl!("physical.wgsl");
/// `instance_matrix`: an instance's transform from its columns in the
/// instance buffers ([`instances::GpuTransform`]); `mul_point`: how the
/// points it moves are projected.
pub(crate) const INSTANCE_WGSL: &str = wgsl!("instance.wgsl");
/// The BRDF and the shadow lookups the lit passes share (pbr, terrain, water).
pub(crate) const LIGHTING_WGSL: &str = wgsl!("lighting.wgsl");

/// The lit shader's source with `hooks` (`hooks.rs`), the joint palette
/// read as `palette` and the light tables as `tables`.
fn lit_source_with(
    hooks: &str,
    palette: skin::PaletteMode,
    tables: clusters::LightTables,
) -> String {
    format!(
        "{FRAME_WGSL}{}{}{MATERIAL_WGSL}{MATERIAL_MAPS_WGSL}{MATERIAL_DISSOLVE_WGSL}{}{hooks}{LIGHTING_WGSL}{INSTANCE_WGSL}{PBR_WGSL}{PBR_FS_WGSL}{PBR_PREPASS_WGSL}{PBR_LEAN_WGSL}",
        tables.wgsl(),
        particles_gpu::NOISE_WGSL,
        palette.wgsl(3)
    )
}

/// The physical lit shader's source (`physical.rs`): [`lit_source_with`]
/// with the physical material group and fragment shader.
fn physical_source_with(
    hooks: &str,
    palette: skin::PaletteMode,
    tables: clusters::LightTables,
) -> String {
    format!(
        "{FRAME_WGSL}{}{}{MATERIAL_WGSL}{MATERIAL_PHYSICAL_WGSL}{MATERIAL_DISSOLVE_WGSL}{}{hooks}{LIGHTING_WGSL}{INSTANCE_WGSL}{PBR_WGSL}{PHYSICAL_WGSL}",
        tables.wgsl(),
        particles_gpu::NOISE_WGSL,
        palette.wgsl(3)
    )
}

/// [`physical_source_with`] for a tier, as [`lit_source`].
pub(crate) fn physical_source(hooks: &str, full: bool) -> String {
    if full {
        physical_source_with(
            hooks,
            skin::PaletteMode::Storage,
            clusters::LightTables::Storage,
        )
    } else {
        physical_source_with(
            hooks,
            skin::PaletteMode::Texture,
            clusters::LightTables::Uniform,
        )
    }
}

/// [`lit_source_with`] for a tier (`full`: storage buffers for the palette
/// and the light tables).
pub(crate) fn lit_source(hooks: &str, full: bool) -> String {
    if full {
        lit_source_with(
            hooks,
            skin::PaletteMode::Storage,
            clusters::LightTables::Storage,
        )
    } else {
        lit_source_with(
            hooks,
            skin::PaletteMode::Texture,
            clusters::LightTables::Uniform,
        )
    }
}
const SHADOW_WGSL: &str = wgsl!("shadow.wgsl");
/// The shadow pass's slot and the paraboloid projection every caster shares.
pub(crate) const SHADOW_PASS_WGSL: &str = wgsl!("shadow_pass.wgsl");

/// The shadow shader's source with `hooks` (a material shader's, or the
/// built-in identities) and the joint palette read as `palette`.
pub(crate) fn shadow_source(hooks: &str, palette: skin::PaletteMode) -> String {
    format!(
        "{}{MATERIAL_WGSL}{MATERIAL_MAPS_WGSL}{MATERIAL_DISSOLVE_WGSL}{}{hooks}{SHADOW_PASS_WGSL}{INSTANCE_WGSL}{SHADOW_WGSL}",
        particles_gpu::NOISE_WGSL,
        palette.wgsl(2)
    )
}
/// The scattered copies' lit module with the light tables `tables`.
fn scatter_lit_source(tables: clusters::LightTables) -> String {
    format!(
        "{FRAME_WGSL}{}{}{MATERIAL_WGSL}{MATERIAL_MAPS_WGSL}{MATERIAL_DISSOLVE_WGSL}{LIGHTING_WGSL}{INSTANCE_WGSL}{}{}",
        tables.wgsl(),
        particles_gpu::NOISE_WGSL,
        wgsl!("scatter_common.wgsl"),
        wgsl!("scatter.wgsl")
    )
}

/// The scattered copies' shadow module.
fn scatter_shadow_source() -> String {
    format!(
        "{}{MATERIAL_WGSL}{MATERIAL_MAPS_WGSL}{MATERIAL_DISSOLVE_WGSL}{}{SHADOW_PASS_WGSL}{INSTANCE_WGSL}{}",
        particles_gpu::NOISE_WGSL,
        wgsl!("scatter_common.wgsl"),
        wgsl!("scatter_shadow.wgsl")
    )
}

/// What the 3D fluid's module starts with: the frame group, the light
/// tables `tables` and the lighting.
fn fluid_shared_source(tables: clusters::LightTables) -> String {
    format!("{FRAME_WGSL}{}{LIGHTING_WGSL}", tables.wgsl())
}
const SKY_WGSL: &str = wgsl!("sky.wgsl");
const SPRITE_WGSL: &str = wgsl!("sprite.wgsl");
/// The material uniform; the standard group's textures and the dissolve
/// follow it (`material_maps.wgsl`, `material_dissolve.wgsl`), the
/// physical group's replace the first (`material_physical.wgsl`).
const MATERIAL_WGSL: &str = wgsl!("material.wgsl");
const MATERIAL_MAPS_WGSL: &str = wgsl!("material_maps.wgsl");
const MATERIAL_DISSOLVE_WGSL: &str = wgsl!("material_dissolve.wgsl");
const MATERIAL_PHYSICAL_WGSL: &str = wgsl!("material_physical.wgsl");
const RIBBON_WGSL: &str = wgsl!("ribbon.wgsl");

/// Uniform slots of the shadow passes (dynamic offsets): 3 cascades, then 12 local faces.
const SHADOW_SLOTS: usize = CASCADES + LOCAL_SHADOW_LAYERS;
const SHADOW_SLOT_STRIDE: u64 = 256;

/// Everything that can go wrong, as values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderError {
    /// No GPU adapter for the requested backend.
    NoAdapter(String),
    /// The adapter refused to create a device.
    NoDevice(String),
    /// The surface could not be configured or acquired.
    Surface(String),
    /// A handle in the scene points at nothing (it came from another scene).
    InvalidHandle(String),
    /// Reading a frame back failed.
    Readback(String),
    /// The call needs something the renderer was not created with.
    Unsupported(String),
    /// Input data is malformed (a texture of the wrong byte length, an unreadable glTF).
    InvalidData(String),
    /// The frame's render graph did not compile (a cycle, a missing target).
    Graph(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::NoAdapter(m) => write!(f, "no GPU adapter: {m}"),
            RenderError::NoDevice(m) => write!(f, "no GPU device: {m}"),
            RenderError::Surface(m) => write!(f, "surface: {m}"),
            RenderError::InvalidHandle(m) => write!(f, "invalid handle: {m}"),
            RenderError::Readback(m) => write!(f, "readback: {m}"),
            RenderError::Unsupported(m) => write!(f, "unsupported: {m}"),
            RenderError::InvalidData(m) => write!(f, "invalid data: {m}"),
            RenderError::Graph(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for RenderError {}

/// Which graphics API to run on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Backend {
    /// The platform's primary API: DX12 on Windows, Metal or Vulkan elsewhere,
    /// WebGPU in a browser.
    #[default]
    Auto,
    /// Browser WebGPU.
    WebGpu,
    /// WebGL2 through wgpu's GL backend (the reduced tier).
    WebGl2,
    Metal,
    Vulkan,
}

impl Backend {
    fn backends(self) -> wgpu::Backends {
        match self {
            Backend::Auto if cfg!(target_arch = "wasm32") => wgpu::Backends::BROWSER_WEBGPU,
            Backend::Auto if cfg!(target_os = "windows") => wgpu::Backends::DX12,
            Backend::Auto => wgpu::Backends::PRIMARY,
            Backend::WebGpu => wgpu::Backends::BROWSER_WEBGPU,
            Backend::WebGl2 => wgpu::Backends::GL,
            Backend::Metal => wgpu::Backends::METAL,
            Backend::Vulkan => wgpu::Backends::VULKAN,
        }
    }

    /// A wgpu instance for this backend, for hosts that create their own surface
    /// before calling [`Renderer::with_surface`].
    ///
    /// Without wgpu's validation of indirect draws (`VALIDATION_INDIRECT_CALL`,
    /// on by default natively): it re-checks every indirect draw's
    /// arguments in a compute pass of its own each frame, and the only
    /// indirect draws are GPU culling's (`cull_gpu.rs`), whose arguments
    /// cull.wgsl writes in range by construction (a batch's index count,
    /// at most its instances, zero offsets) over buffers sized for them.
    /// About 0.4 ms of CPU a frame on spheres-20k natively (release); the
    /// browsers validate WebGPU's indirect draws themselves.
    pub fn instance(self) -> wgpu::Instance {
        let mut flags = wgpu::InstanceFlags::from_build_config();
        flags.remove(wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL);
        let descriptor = wgpu::InstanceDescriptor {
            backends: self.backends(),
            flags,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        };
        #[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc"))]
        let descriptor = {
            let mut descriptor = descriptor;
            descriptor.backend_options.dx12.shader_compiler = wgpu::Dx12Compiler::StaticDxc;
            descriptor
        };
        wgpu::Instance::new(descriptor)
    }
}

/// Antialiasing. [`RenderInfo::antialias`] reports what a renderer resolved
/// it to (never `Tier`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Antialias {
    /// None: every pixel is one sample.
    #[default]
    Off,
    /// The tier's method: TAA on the full tier (WebGPU, Metal, Vulkan), 4x
    /// MSAA on the no-compute tier (WebGL2, GL), or FXAA there where the
    /// HDR format cannot be multisampled.
    Tier,
    /// 4x MSAA on any tier (FXAA where the HDR format cannot be
    /// multisampled). Sharp sprites and text; no temporal blur.
    Msaa4,
    /// Temporal antialiasing on any tier: a sub-pixel jitter per frame and a
    /// reprojected, neighbourhood-clamped history. Converges over ~8 frames.
    /// [`RenderDesc::taa_history`] and [`RenderDesc::taa_sharpen`] refine it.
    Taa,
    /// FXAA 3.11 (quality preset 12) over the tonemapped image, before the
    /// overlay: one full-screen pass, no extra samples and no history, on
    /// any tier. Softer than MSAA on texture detail; nothing to converge.
    Fxaa,
}

/// How TAA reads its history ([`RenderDesc::taa_history`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TaaHistory {
    /// One bilinear tap: cheap, slightly blurred under motion.
    #[default]
    Bilinear,
    /// Catmull-Rom over 4×4 texels in 9 bilinear taps: keeps the history
    /// sharp when the camera moves (each frame resamples it).
    CatmullRom,
}

/// How to create a [`Renderer`]. The default is a 1280x720 renderer on the
/// platform's primary API with an sRGB RGBA8 output, no antialiasing, three
/// 2048² shadow cascades and BC/ASTC/ETC2 texture compression where the GPU
/// has it. [`RenderDesc::mobile`] is the preset for phones.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderDesc {
    pub width: u32,
    pub height: u32,
    pub backend: Backend,
    /// The output (offscreen, surface or caller view) format. `None` picks
    /// [`DEFAULT_OUTPUT_FORMAT`] offscreen and the surface's preference on a surface.
    pub output_format: Option<wgpu::TextureFormat>,
    pub antialias: Antialias,
    /// The side of each directional light cascade in texels (clamped to
    /// 64..=the device's limit). 2048 by default; 1024 is the mobile
    /// recommendation (each cascade costs `size² × 4` bytes, plus one depth
    /// buffer of the same size).
    pub shadow_map_size: u32,
    /// Directional light cascades, 1 to 3 (3 by default; fewer cover the
    /// same distance with coarser near shadows).
    pub shadow_cascades: u32,
    /// Which compressed texture family [`RenderInfo::texture_formats`] picks.
    pub texture_compression: TextureCompression,
    /// Where particles simulate: compute on the full tier, or the CPU path
    /// everywhere ([`ParticleSimulation::Cpu`]).
    pub particles: ParticleSimulation,
    /// Where instances are culled: in compute with indirect draws on the
    /// full tier for large scenes (frustum and occlusion), or the CPU path
    /// ([`Culling`]).
    pub culling: Culling,
    /// How point lights' shadows are stored: cube faces, or dual-paraboloid
    /// hemispheres (two passes instead of six; [`PointShadows`]). `Auto`:
    /// cube on the full tier, paraboloid on WebGL2.
    pub point_shadows: PointShadows,
    /// With TAA: how the history is resampled ([`TaaHistory`]).
    pub taa_history: TaaHistory,
    /// With TAA: a sharpening pass after it, 0 (off, the default) to 1: a
    /// clamped unsharp mask over the 4 neighbours that restores what the
    /// history's blend softened, never past the neighbourhood's range.
    pub taa_sharpen: f32,
}

/// Where instances are culled ([`RenderDesc::culling`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Culling {
    /// As [`Culling::Gpu`] for a frame of at least
    /// [`GPU_CULLING_MIN_INSTANCES`] instances (not skinned), else on the
    /// CPU: a small frame's CPU culling costs less than the GPU path's
    /// fixed per-frame work.
    #[default]
    Auto,
    /// On the GPU where the adapter has compute and indirect draws (each
    /// pass's frustum, and for the camera the depth of last frame's
    /// visible occluders), whatever the frame's size, else on the CPU. A
    /// frame with object motion blur, or more instances than one storage
    /// binding holds, is culled on the CPU.
    Gpu,
    /// Always on the CPU (frustum only; the no-compute tier's path).
    Cpu,
}

/// The instances (not skinned) from which [`Culling::Auto`] culls a frame
/// on the GPU. Measured on an M4 Pro (tests/evidence/P3/culling/NOTES.md):
/// at 40,000 instances the GPU path saved about 1.1 ms of `render` per
/// frame natively (city-street, median 6.40 to 5.27 ms) and 0.6 ms on
/// WebGPU, while on scenes of tens of instances it cost 0.4 to 1 ms more
/// per frame (point-spot-lights 1.0 to 1.6 ms against 0.6 ms, showcase
/// 3.5 to 6.1 against 3.2). The CPU test costs about 40 ns per instance
/// at that size, so the fixed cost breaks even between about 10,000 and
/// 25,000 instances; 16,384 sits between, below the city's 40,000.
pub const GPU_CULLING_MIN_INSTANCES: u32 = 16_384;

impl Default for RenderDesc {
    fn default() -> Self {
        RenderDesc {
            width: 1280,
            height: 720,
            backend: Backend::Auto,
            output_format: None,
            antialias: Antialias::Off,
            shadow_map_size: SHADOW_SIZE,
            shadow_cascades: MAX_CASCADES,
            texture_compression: TextureCompression::Auto,
            particles: ParticleSimulation::Auto,
            culling: Culling::Auto,
            point_shadows: PointShadows::Auto,
            taa_history: TaaHistory::Bilinear,
            taa_sharpen: 0.0,
        }
    }
}

impl RenderDesc {
    /// The mobile preset: two 1024² cascades (12 MB of shadow maps instead of
    /// 64 MB) and ASTC (then ETC2) textures. Everything else as the default.
    pub fn mobile() -> RenderDesc {
        RenderDesc {
            shadow_map_size: MOBILE_SHADOW_SIZE,
            shadow_cascades: 2,
            texture_compression: TextureCompression::PreferAstc,
            ..RenderDesc::default()
        }
    }
}

/// The renderer's feature tier (`docs/design.md` §8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tier {
    /// WebGPU, Metal, Vulkan: compute available.
    Full,
    /// WebGL2 (and other GL): no compute; CPU fallbacks.
    NoCompute,
}

/// What the renderer runs on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderInfo {
    pub adapter: String,
    pub backend: wgpu::Backend,
    pub device_type: wgpu::DeviceType,
    pub tier: Tier,
    /// The antialiasing in use: `Off`, `Msaa4`, `Taa` or `Fxaa`.
    pub antialias: Antialias,
    /// The cascade size and count in use (after clamping).
    pub shadow_map_size: u32,
    pub shadow_cascades: u32,
    /// How many point and spot lights cast shadows in one frame on this
    /// tier ([`MAX_SHADOWED_LIGHTS`] on the full tier,
    /// [`NO_COMPUTE_SHADOWED_LIGHTS`] on WebGL2).
    pub shadowed_lights: u32,
    /// How point lights' shadows are stored: `Cube` or `Paraboloid` (never `Auto`).
    pub point_shadows: PointShadows,
    /// The encoding each kind of texture should be uploaded in, and every
    /// encoding this device accepts.
    pub texture_formats: TextureFormats,
    /// Compiled pipelines are kept across runs ([`set_pipeline_cache_dir`]
    /// was given a directory and the backend has a cache: Vulkan).
    pub pipeline_cache: bool,
    /// The largest side of a 2D texture, so of a render target.
    pub max_texture_size: u32,
}

impl RenderInfo {
    /// Whether the adapter is a software rasterizer (lavapipe, llvmpipe,
    /// SwiftShader): the `vulkan-cpu` tier ([`is_software_adapter`]).
    pub fn software(&self) -> bool {
        is_software_adapter(self.device_type, &self.adapter)
    }

    /// The name gfx reference frames are filed under, and the tier the probe
    /// and `og polish` report ([`tier_name`]).
    pub fn backend_tier(&self) -> String {
        tier_name(self.backend, self.device_type, &self.adapter)
    }
}

/// The environment switch that takes the renderer away: `OG_RENDERER=none`
/// makes every [`Renderer::new`] and [`Renderer::with_surface`] answer
/// [`RenderError::NoAdapter`], as on a machine with no usable adapter (the
/// platform's `engine` and `probe` sandboxes, docs/design.md §5), so the
/// no-renderer path is testable where a GPU is present (docs/ci.md
/// §Switches). Any other value, or none, leaves adapters as they are. Read
/// natively only; the web always has its canvas's adapter.
pub const RENDERER_SWITCH: &str = "OG_RENDERER";

/// Whether [`RENDERER_SWITCH`] is `none` in this process's environment.
pub fn renderer_disabled() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        false
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var(RENDERER_SWITCH).is_ok_and(|v| v.trim().eq_ignore_ascii_case("none"))
    }
}

/// The error a disabled renderer answers, naming the switch.
fn disabled_error() -> RenderError {
    RenderError::NoAdapter(format!(
        "{RENDERER_SWITCH}=none: this process runs without a renderer"
    ))
}

/// Whether an adapter rasterizes on the CPU: wgpu reports a CPU device type,
/// or its name is one of the software rasterizers (Mesa's lavapipe and
/// llvmpipe, SwiftShader), which some drivers report as another type.
pub fn is_software_adapter(device_type: wgpu::DeviceType, name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    device_type == wgpu::DeviceType::Cpu
        || ["llvmpipe", "lavapipe", "swiftshader"]
            .iter()
            .any(|s| name.contains(s))
}

/// The tier name of an adapter: `metal`, `vulkan`, `vulkan-cpu` (a software
/// rasterizer), `webgpu`, `webgl2`, `gl`, `dx12`. It names the reference-frame
/// directories and is the probe's `RenderTier`, so a frame time measured on
/// a CPU rasterizer is never read as a GPU's.
pub fn tier_name(backend: wgpu::Backend, device_type: wgpu::DeviceType, adapter: &str) -> String {
    match backend {
        wgpu::Backend::Metal => "metal".into(),
        wgpu::Backend::Vulkan if is_software_adapter(device_type, adapter) => "vulkan-cpu".into(),
        wgpu::Backend::Vulkan => "vulkan".into(),
        wgpu::Backend::BrowserWebGpu => "webgpu".into(),
        wgpu::Backend::Gl if cfg!(target_arch = "wasm32") => "webgl2".into(),
        wgpu::Backend::Gl => "gl".into(),
        wgpu::Backend::Dx12 => "dx12".into(),
        other => format!("{other:?}").to_lowercase(),
    }
}

/// Where a frame goes.
pub enum RenderTarget<'a> {
    /// The renderer's own offscreen texture (read back with [`Renderer::capture`]).
    Offscreen,
    /// The surface the renderer was created with: acquired, drawn and presented.
    Surface,
    /// A caller's texture view of the renderer's size and output format.
    View(&'a wgpu::TextureView),
}

/// What one [`Renderer::render`] call submitted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameStats {
    /// Draw calls across every pass (shadows, main, sky, sprites, post).
    pub draw_calls: u32,
    /// Tile quads drawn this frame (after culling) and the chunks they came from.
    pub tiles: u32,
    pub tile_chunks: u32,
    /// Distinct (mesh, material) pairs submitted.
    pub batches: u32,
    /// Instances submitted.
    pub instances: u32,
    /// Instances outside the camera's view, not drawn in the main pass
    /// (with GPU culling, an earlier frame's count: see `counts_lag`).
    pub culled: u32,
    /// Triangles drawn in the main pass.
    pub triangles: u64,
    /// Instance draws across every shadow pass (after culling per pass).
    pub shadow_casters: u32,
    /// Shadow passes (cascades plus local light faces).
    pub shadow_passes: u32,
    /// Point and spot lights drawn with shadows, and those in view whose
    /// shadows were left out past [`RenderInfo::shadowed_lights`] (the
    /// least radiant at the camera).
    pub shadowed_lights: u32,
    pub shadowed_lights_dropped: u32,
    /// Local lights drawn; those in view beyond [`MAX_LIGHTS`] that were
    /// not (the least important); those outside the view.
    pub lights: u32,
    pub lights_dropped: u32,
    pub lights_culled: u32,
    /// Light references the clusters held, and those dropped from clusters
    /// past [`MAX_CLUSTER_LIGHTS`] (or the tier's total, [`UNIFORM_LIGHT_REFS`]
    /// on WebGL2).
    pub cluster_lights: u32,
    pub cluster_lights_dropped: u32,
    /// Sprites and glyphs drawn, and the draws they took.
    pub sprites: u32,
    pub glyphs: u32,
    pub sprite_draws: u32,
    /// Characters of [`RenderScene::texts`] (and the overlay's) missing from their font.
    pub missing_glyphs: u32,
    /// The overlay's share of `sprites`, `glyphs` and `sprite_draws`
    /// ([`RenderScene::overlay`]).
    pub overlay_sprites: u32,
    pub overlay_glyphs: u32,
    pub overlay_draws: u32,
    /// Transparent instances drawn (sorted back to front), and their draws.
    pub transparent: u32,
    pub transparent_draws: u32,
    /// Passes of the frame's render graph, and the transient textures it allocated.
    pub graph_passes: u32,
    pub transient_textures: u32,
    /// Instance draws of the depth-normal prepass (0: it did not run).
    pub prepass_draws: u32,
    /// Particle systems, their summed capacity, the live particles of the
    /// CPU-simulated ones, the simulation ticks run, and systems whose
    /// collision could not run here (no compute, or sprite mode).
    pub particle_systems: u32,
    pub particle_capacity: u32,
    pub particles_cpu: u32,
    pub particle_ticks: u32,
    pub particle_collision_skipped: u32,
    /// Trail, ribbon and beam strips drawn, and their segments.
    pub strips: u32,
    pub strip_segments: u32,
    /// Line segments and points drawn (`lines.rs`; wireframe edges included).
    pub line_instances: u32,
    /// Decals drawn, and those beyond [`MAX_DECALS`].
    pub decals: u32,
    pub decals_dropped: u32,
    /// 2D lights and occluders drawn.
    pub lights_2d: u32,
    pub occluders_2d: u32,
    /// Skinned instances drawn and the joints of the frame's palettes.
    pub skinned_instances: u32,
    pub joints: u32,
    /// GPU memory of the scene's textures (every mip level, as allocated:
    /// RGBA8 with its generated chain, or the encoded levels as uploaded),
    /// texture arrays (padding layers included), environment maps (every
    /// level of the six RGB9E5 faces) and palettes (256 x 1 RGBA8 each).
    /// A removed resource is freed before the frame and counts nothing.
    pub texture_bytes: u64,
    /// Terrain clipmap levels drawn in the main pass, their triangles
    /// (also in `triangles`), and the levels drawn into shadow maps.
    pub terrain_draws: u32,
    pub terrain_triangles: u64,
    pub terrain_shadow_draws: u32,
    /// Terrain tiles resident on the GPU after this frame, and those
    /// uploaded for it (phase 3 streaming).
    pub terrain_tiles: u32,
    pub terrain_uploads: u32,
    /// Scattered copies' chunk draws in the main pass, the copies in them
    /// (before the distance drops the farthest) and the chunks built for
    /// this frame.
    pub scatter_draws: u32,
    pub scatter_copies: u32,
    pub scatter_built: u32,
    /// Bytes of instance transforms written to the GPU this frame: only the
    /// 4 KiB blocks that changed since the last frame (0 for a scene that
    /// stood still).
    pub instance_upload_bytes: u64,
    /// Draws of the planar reflection pass (0: it did not run).
    pub reflection_draws: u32,
    /// Frames rendered by this renderer before this one (drives the TAA jitter).
    pub frame: u64,
    /// Whether the GPU culled this frame ([`RenderDesc::culling`]).
    pub gpu_culling: bool,
    /// Opaque instances in view that nearer occluders hid (GPU culling).
    pub occluded: u32,
    /// Their triangles, counted on the GPU per hidden instance (GPU
    /// culling; 0 on the CPU path).
    pub occluded_triangles: u64,
    /// Triangles of the opaque instances inside the camera's view, before
    /// occlusion: every one is drawn in the main pass or counted in
    /// `occluded_triangles`. The CPU path counts the instances in view
    /// (it may draw a whole batch, so `triangles` can exceed it); GPU
    /// culling adds its main-pass instance triangles and
    /// `occluded_triangles`.
    pub in_view_triangles: u64,
    /// Draws of the GPU culling's occluder depth pass (also in `draw_calls`).
    pub occluder_draws: u32,
    /// With GPU culling, `culled`, `occluded`, `occluded_triangles`,
    /// `in_view_triangles`, `triangles` and `shadow_casters` hold the
    /// GPU's counts of the newest frame read back
    /// by then, this many frames old (`u32::MAX`: none yet). Only frames
    /// after [`Renderer::request_culling_counts`] are read back; natively
    /// [`Renderer::finish_stats`] puts the frame's own in their place (0).
    /// 0 on the CPU path. While they lag, `culled`, `occluded` and the
    /// instances drawn come from different frames and need not add up to
    /// `instances`.
    pub counts_lag: u32,
    /// Whether TAA blended this frame with the earlier frames' history:
    /// false on the first frame, after [`Renderer::reset_history`] (a camera
    /// cut), a resize, a new scene, a change of the stylisation's
    /// primitives ([`Stylise::chain_bits`]), and without TAA.
    pub taa_history: bool,
}

/// An RGBA8 frame, rows top to bottom.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ShadowPassUniform {
    view_proj: [[f32; 4]; 4],
    light: [f32; 4],
    /// x: the scene clock in seconds (a material shader's `h.time`).
    time: [f32; 4],
    /// x: 1 for a paraboloid hemisphere, y: +1 the one below the light, -1
    /// above, z: the map's side, w: the projection's scale (shadow_pass.wgsl).
    paraboloid: [f32; 4],
    pad: [f32; 36],
}

/// An instance in the instance buffers: its columns' xyz, its
/// texture-array layer and its look ([`instances::GpuTransform`]).
const INSTANCE_STRIDE: u64 = std::mem::size_of::<instances::GpuTransform>() as u64;

const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Float32x4, 15 => Float32x4],
};

const INSTANCE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: INSTANCE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![4 => Float32x3, 5 => Float32x3, 6 => Float32x3, 7 => Float32x3, 8 => Float32x4],
};

/// One attribute of [`Vertex`] at its offset in the shared vertex buffer.
const fn vertex_attribute(
    format: wgpu::VertexFormat,
    offset: usize,
    location: u32,
) -> wgpu::VertexAttribute {
    wgpu::VertexAttribute {
        format,
        offset: offset as u64,
        shader_location: location,
    }
}

const POSITION_ATTRIBUTE: wgpu::VertexAttribute = vertex_attribute(
    wgpu::VertexFormat::Float32x3,
    std::mem::offset_of!(Vertex, position),
    0,
);
const NORMAL_ATTRIBUTE: wgpu::VertexAttribute = vertex_attribute(
    wgpu::VertexFormat::Float32x3,
    std::mem::offset_of!(Vertex, normal),
    1,
);
const COLOR_ATTRIBUTE: wgpu::VertexAttribute = vertex_attribute(
    wgpu::VertexFormat::Float32x4,
    std::mem::offset_of!(Vertex, color),
    3,
);

/// The shared vertex buffer as the lean lit stage reads it
/// (`pbr_lean.wgsl` `vs_lean`): position, normal and colour.
const LEAN_VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[POSITION_ATTRIBUTE, NORMAL_ATTRIBUTE, COLOR_ATTRIBUTE],
};

/// As the lean prepass reads it (`vs_prepass_lean`): position and normal.
const POSITION_NORMAL_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[POSITION_ATTRIBUTE, NORMAL_ATTRIBUTE],
};

/// As the lean shadow caster reads it (`shadow.wgsl` `vs_lean`): position.
const POSITION_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[POSITION_ATTRIBUTE],
};

struct GpuMesh {
    /// The f32 vertices ([`Vertex`]); a 16-byte placeholder while `full`
    /// is false.
    vertices: wgpu::Buffer,
    /// Whether `vertices` holds the mesh: false for a compact mesh no
    /// pipeline that reads [`Vertex`] has drawn yet ([`Renderer::ensure_full`]).
    full: bool,
    /// The compact vertices ([`VertexEncoding::Compact`], `compact.rs`)
    /// and how to undo their positions' quantisation.
    compact: Option<(wgpu::Buffer, compact::Dequant)>,
    indices: wgpu::Buffer,
    index_count: u32,
    /// The scene's revision of the mesh this holds (`RenderScene::set_mesh`).
    revision: u64,
    /// Local-space bounds as (center, half extent).
    bounds: Option<(Vec3, Vec3)>,
}

impl GpuMesh {
    /// What a draw binds: the compact vertices for a [`Batch::compact`]
    /// draw, else the f32 ones.
    fn vertex_buffer(&self, compact: bool) -> &wgpu::Buffer {
        match &self.compact {
            Some((buffer, _)) if compact => buffer,
            _ => {
                debug_assert!(self.full, "a compact mesh's f32 vertices were not uploaded");
                &self.vertices
            }
        }
    }
}

struct GpuMaterial {
    revision: u64,
    /// The standard group: every pass but a physical material's lit one.
    bind_group: wgpu::BindGroup,
    /// A physical material's group for its lit pass (`physical.rs`).
    physical: Option<wgpu::BindGroup>,
    /// An array-map material's group for its lit and prepass passes
    /// ([`material::ArrayMaps`]); its shadows bind the standard group.
    array: Option<wgpu::BindGroup>,
    side: Side,
    /// Drawn in the sorted pass, back to front: blended or transmissive.
    transparent: bool,
    /// Blended (no depth write).
    blend: bool,
    /// Refracts the copy of the opaque scene.
    transmissive: bool,
    /// Discards pixels (shadows draw it with the material).
    dissolve: bool,
    /// Has its own outline (turns the outline pass on).
    outline: bool,
    /// Uses toon, rim, flash or dissolve.
    effects: bool,
    /// Its material shader (the scene's index).
    shader: Option<usize>,
    /// Writes exactly the depth of its triangles (opaque, no alpha cut,
    /// dissolve or shader): GPU culling may draw it as an occluder.
    occluder: bool,
    /// Plain: an occluder with no texture, no effect (toon, rim, flash,
    /// outline) and not physical. Its unskinned instances draw through the
    /// lean lit and prepass pipelines (`pbr_lean.wgsl`): the same pixels
    /// with fewer attributes read, fewer varyings and (without screen
    /// derivatives) no discard.
    lean: bool,
    /// Flat-shaded: its normals come from screen derivatives.
    flat: bool,
    /// Drawn as its triangles' edges in this colour (`lines.rs`), not as triangles.
    wireframe: Option<[f32; 4]>,
}

struct GpuTexture {
    view: wgpu::TextureView,
    format: TextureFormat,
    /// Bytes allocated, every level.
    bytes: u64,
}

/// What an RGBA8 texture allocates on the GPU, every level.
fn rgba_texture_bytes(texture: &TextureData) -> u64 {
    match texture.mips {
        Mips::Generate => (0..mips::mip_count(texture.width, texture.height))
            .map(|l| TextureFormat::Rgba8.level_bytes(texture.width, texture.height, l) as u64)
            .sum(),
        Mips::None => texture.rgba.len() as u64,
    }
}

struct GpuFont {
    layout: FontLayout,
    bind_group: wgpu::BindGroup,
    /// The scene's revision of the font this was built from.
    revision: u64,
}

/// GPU copies of one scene's resources, indexed like the scene's handles.
#[derive(Default)]
struct SceneCache {
    /// The scene's palettes (`add_palette`) as 256 x 1 textures (`None`:
    /// removed), and the scene's revision of each when it was uploaded.
    palettes: Vec<Option<wgpu::TextureView>>,
    palette_revisions: Vec<u64>,
    /// The material style the materials were uploaded with.
    material_style: MaterialStyle,
    scene_id: u64,
    meshes: Vec<GpuMesh>,
    /// The lit shader with each of the scene's material shaders.
    shaders: Vec<wgpu::ShaderModule>,
    /// The shadow shader with each of them.
    shadow_shaders: Vec<wgpu::ShaderModule>,
    /// Their lit pipelines by (shader, side, blended, effects, skinned).
    hook_pipelines: HashMap<(usize, Side, bool, bool, bool), wgpu::RenderPipeline>,
    /// Their shadow and prepass pipelines by (shader, side, skinned).
    hook_shadow_pipelines: HashMap<(usize, Side, bool), wgpu::RenderPipeline>,
    hook_prepass_pipelines: HashMap<(usize, Side, bool), wgpu::RenderPipeline>,
    /// The physical lit shader with a material shader, made when a physical
    /// material first uses it, and its pipelines by (shader, key).
    physical_shaders: HashMap<usize, wgpu::ShaderModule>,
    physical_hook_pipelines: HashMap<(usize, PbrKey), wgpu::RenderPipeline>,
    /// Physical materials' packed maps, shared by materials with the same maps.
    packs: HashMap<physical::PackKey, wgpu::TextureView>,
    textures: Vec<GpuTexture>,
    /// The scene's revision of each texture when it was uploaded.
    texture_revisions: Vec<u64>,
    materials: Vec<GpuMaterial>,
    /// Sprite bind groups per texture, made on first use.
    sprite_groups: Vec<Option<wgpu::BindGroup>>,
    /// The filter each texture's sprite group was made with.
    filters: Vec<TextureFilter>,
    /// The same with a repeating sampler (trails, ribbons, beams).
    ribbon_groups: Vec<Option<wgpu::BindGroup>>,
    fonts: Vec<GpuFont>,
    /// Joints and weights of skinned (or morphed) meshes, by mesh.
    skins: Vec<Option<wgpu::Buffer>>,
    /// Per mesh: (first delta, targets) in `morph_data`.
    morph_ranges: Vec<(u32, u32)>,
    /// Every mesh's morph deltas: per vertex, per target, position then normal.
    morph_data: Vec<[f32; 4]>,
    /// Whether `morph_data` changed since it was uploaded.
    morphs_dirty: bool,
    /// Colour LUTs as 3D textures with the bytes each allocates (`None`:
    /// removed), and the scene's revision of each when it was uploaded.
    luts: Vec<Option<(wgpu::TextureView, u64)>>,
    lut_revisions: Vec<u64>,
    /// Environment maps as cube textures (kept for their views; `None`:
    /// removed), and the scene's revision of each when it was uploaded.
    environment_maps: Vec<Option<GpuEnvironmentMap>>,
    environment_map_revisions: Vec<u64>,
    /// Texture arrays, each one 2D-array texture (`None`: removed), and
    /// the scene's revision of each when it was uploaded.
    texture_arrays: Vec<Option<texture_array::GpuTextureArray>>,
    texture_array_revisions: Vec<u64>,
    /// Array sprite bind groups per texture array, made on first use, with
    /// the filter each was made with.
    sprite_array_groups: Vec<Option<(wgpu::BindGroup, TextureFilter)>>,
}

/// An environment map's cube on the GPU.
struct GpuEnvironmentMap {
    /// Held for the view.
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    /// Bytes allocated, every level of every face.
    bytes: u64,
}

/// What a palette's 256 x 1 RGBA8 texture allocates.
const PALETTE_BYTES: u64 = MAX_PALETTE_COLORS as u64 * 4;

/// What the image-based light was last generated from: the sky, or an
/// environment map of a scene (its id and index), turned and with its
/// background scaled (as bits, so a key compares exactly).
#[derive(Clone, Copy, Debug, PartialEq)]
enum EnvironmentKey {
    Sky(ProceduralSky),
    Map {
        scene: u64,
        map: usize,
        /// The scene's revision of the map's slot (a slot refilled after a
        /// removal is another map).
        revision: u64,
        rotation: u32,
        background: u32,
    },
}

/// One (mesh, material) run of the sorted instances.
struct Batch {
    mesh: usize,
    material: usize,
    first: u32,
    count: u32,
    side: Side,
    /// In the sorted pass (blended or transmissive).
    transparent: bool,
    /// Blended: the pipeline blends and writes no depth.
    blend: bool,
    /// Drawn with the physical lit pipelines.
    physical: bool,
    /// Refracts the scene copy.
    transmissive: bool,
    dissolve: bool,
    /// Needs the effect shader (its material's or the frame's effects).
    effects: bool,
    /// Its material's shader.
    shader: Option<usize>,
    /// Drawn with the lean pipelines ([`GpuMaterial::lean`]; never skinned).
    lean: bool,
    /// Its fragments take screen derivatives (flat shading, or the frame's
    /// decals): its lean pipelines keep the discard (`DISCARD` in
    /// pbr_fs.wgsl), so the derivatives are the full pipeline's.
    derivatives: bool,
    /// Drawn from its mesh's compact vertices (lean, the mesh
    /// [`VertexEncoding::Compact`]): the instance transforms include the
    /// dequantisation (`compact.rs`).
    compact: bool,
    /// Drawn with the array-map pipelines ([`GpuMaterial::array`]): its
    /// instances carry their layer in the transform.
    array: bool,
    /// Drawn into the shadow maps ([`Instance::cast_shadows`],
    /// [`SkinnedInstance::cast_shadows`]).
    cast: bool,
    /// Its instances' three.js layer mask (`views.rs`).
    layers: u32,
}

/// The effect bind groups and what they were made from.
struct FxGroups {
    key: (u64, u64, Option<TextureHandle>, bool, bool),
    fx: wgpu::BindGroup,
    sprite: wgpu::BindGroup,
    glow: wgpu::BindGroup,
}

/// The skinned instances of a frame: batches over `data` (every instance,
/// for shadows), each entry's widened world bounds, and the joint matrices.
struct SkinnedFrame {
    batches: Vec<Batch>,
    data: Vec<skin::SkinnedInstanceGpu>,
    spheres: Vec<(Vec3, f32)>,
    matrices: Vec<[[f32; 4]; 4]>,
}

/// The lit pipelines: (side, blended, skinned, effects). Without effects the
/// shader's `EFFECTS` override is false: the toon, rim, flash, dissolve,
/// decal and AO code is compiled out (the features-off cost). The physical
/// pipelines are keyed the same (`physical.rs`).
type PbrKey = (Side, bool, bool, bool);
/// The shadow pipelines: (side, skinned, dissolving).
type ShadowKey = (Side, bool, bool);
/// What a lit pass's draws switch pipelines on: (side, blended, effects,
/// material shader, physical, lean, derivatives, compact, array).
type LitDrawKey = (
    Side,
    bool,
    bool,
    Option<usize>,
    bool,
    bool,
    bool,
    bool,
    bool,
);
/// The lean lit pipelines: (side, effects, derivatives, compact); `effects`
/// as in [`PbrKey`] (a plain material has none of its own: the frame's
/// decals or AO), `derivatives` as [`Batch::derivatives`], `compact`: the
/// vertices are [`VertexEncoding::Compact`]'s.
type LeanKey = (Side, bool, bool, bool);

/// What one pass draws: (batch, first instance, count) ranges in the instance buffer.
#[derive(Default)]
struct DrawList {
    ranges: Vec<(u32, u32, u32)>,
    instances: u32,
    triangles: u64,
    /// Triangles of the instances inside the planes ([`Renderer::cull`];
    /// `triangles` counts whole batches drawn past 90 % in view).
    in_view_triangles: u64,
    /// GPU culling's pass: the ranges are in its `instances` and each draw's
    /// count is in its `args` (`instances` and `triangles` are 0 here).
    indirect: Option<u32>,
}

/// GPU culling's list for pass `g`: every `include`d batch, drawn indirect
/// from the pass's region.
fn indirect_list(batches: &[Batch], n: u32, g: u32, include: impl Fn(&Batch) -> bool) -> DrawList {
    DrawList {
        ranges: batches
            .iter()
            .enumerate()
            .filter(|(_, b)| include(b))
            .map(|(i, b)| (i as u32, (g + 1) * n + b.first, b.count))
            .collect(),
        indirect: Some(g),
        ..DrawList::default()
    }
}

/// The layers to allocate for a square 2D array texture that is to hold
/// `layers` layers. wgpu's GL backend picks the GL texture target from the
/// descriptor when it creates the texture: one layer makes a plain 2D
/// texture and a multiple of six layers makes a cube map (array). A 2D-array
/// sampler bound to either reads zero on WebGL2, with no validation error, and
/// neither can be rendered to by layer as an array. At least two layers, and
/// one spare when that is a multiple of six, keep it a 2D array; callers
/// still address only the layers they asked for.
pub(crate) fn gl_safe_array_layers(layers: u32) -> u32 {
    let layers = layers.max(2);
    if layers.is_multiple_of(6) {
        layers + 1
    } else {
        layers
    }
}

/// A shadow map: one R32Float array the lit shaders read, a view per layer,
/// and the depth buffer its passes test against.
struct ShadowMap {
    array_view: wgpu::TextureView,
    layers: Vec<wgpu::TextureView>,
    depth: wgpu::TextureView,
}

impl ShadowMap {
    fn new(device: &wgpu::Device, label: &str, size: u32, layers: u32) -> ShadowMap {
        // A single cascade or the twelve local shadow layers would otherwise
        // read as zero on WebGL2 (see `gl_safe_array_layers`).
        let allocated = gl_safe_array_layers(layers);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: allocated,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        ShadowMap {
            array_view: texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(label),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            }),
            layers: (0..layers)
                .map(|layer| {
                    texture.create_view(&wgpu::TextureViewDescriptor {
                        label: Some(label),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        base_array_layer: layer,
                        array_layer_count: Some(1),
                        ..Default::default()
                    })
                })
                .collect(),
            depth: target_texture(
                device,
                label,
                size,
                size,
                DEPTH_FORMAT,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            )
            .create_view(&Default::default()),
        }
    }
}

/// The renderer. Create it once per output, call [`Renderer::render`] per frame.
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    info: RenderInfo,
    /// The size the scene is drawn at: the output's, or the stylisation's
    /// logical pixel grid ([`Stylise::pixel`](crate::Stylise::pixel)),
    /// scaled up to the output by the last post pass.
    width: u32,
    height: u32,
    /// The output's size (the offscreen target, the surface, the overlay).
    out_width: u32,
    out_height: u32,
    output_format: wgpu::TextureFormat,
    /// The offscreen target's sRGB view format ([`linear_view_format`]).
    offscreen_linear: Option<wgpu::TextureFormat>,
    surface: Option<(wgpu::Surface<'static>, wgpu::SurfaceConfiguration)>,
    samples: u32,
    persistent: post::Persistent,
    graph: post::FrameGraph,
    /// Measures each graph pass's encoding (`set_clock`); measurement only.
    clock: Option<fn() -> f64>,
    /// The last frame's `(pass, ms)` in execution order (`pass_timings`).
    pass_ms: Vec<(String, f64)>,
    /// The CPU cull's per-instance scratch ([`Renderer::cull`]), kept
    /// between frames so no pass allocates it.
    /// The sorted instances, kept across frames (`instances.rs`).
    sorted: instances::SortedInstances,
    /// The secondary cameras' GPU state (`views.rs`).
    views: views::Views,
    /// The custom post passes on the device (`post_custom.rs`).
    custom_posts: post_custom::CustomPosts,
    cull_inside: Vec<bool>,
    post: post::PostPasses,
    cascade_map: ShadowMap,
    local_map: ShadowMap,
    material_sampler: wgpu::Sampler,
    sprite_sampler: wgpu::Sampler,
    sprite_sampler_nearest: wgpu::Sampler,
    frame_layout: wgpu::BindGroupLayout,
    sky_layout: wgpu::BindGroupLayout,
    material_layout: wgpu::BindGroupLayout,
    sprite_layout: wgpu::BindGroupLayout,
    frame_buffer: wgpu::Buffer,
    /// The local lights, the cluster table and its light references (`clusters.rs`).
    light_tables: clusters::LightTables,
    light_buffers: [wgpu::Buffer; 3],
    clusters: clusters::ClusterBuilder,
    /// The frame uniform of the planar reflection's mirrored camera.
    reflection_buffer: wgpu::Buffer,
    /// Its effect uniform (decals; no screen-space AO).
    reflection_fx_buffer: wgpu::Buffer,
    /// Its cluster table and references, over the main view's light records.
    reflection_light_buffers: [wgpu::Buffer; 3],
    reflection_clusters: clusters::ClusterBuilder,
    shadow_buffer: wgpu::Buffer,
    shadow_bind_group: wgpu::BindGroup,
    pbr_layout: wgpu::PipelineLayout,
    pbr_skinned_layout: wgpu::PipelineLayout,
    pbr_shader: wgpu::ShaderModule,
    /// Shadow layouts: plain, with the material (dissolve), skinned.
    shadow_layouts: [wgpu::PipelineLayout; 3],
    shadow_shader: wgpu::ShaderModule,
    /// Lit pipelines; front opaque made up front.
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
    sky_pipeline: Deferred<wgpu::RenderPipeline>,
    /// The procedural sky of a chosen look drawn per pixel ([`Background::Procedural`]).
    sky_look_pipeline: Deferred<wgpu::RenderPipeline>,
    water: water::WaterRenderer,
    fluid3d: fluid3d::Fluid3dRenderer,
    terrain: terrain::TerrainRenderer,
    scatter: scatter::ScatterRenderer,
    metaballs: metaballs::MetaballRenderer,
    /// The image-based light, regenerated into the same textures for a new sky.
    ibl: ibl::IblGenerator,
    sprite_pipeline: Deferred<wgpu::RenderPipeline>,
    sprite_additive_pipeline: Deferred<wgpu::RenderPipeline>,
    /// Alpha-blended sprites kept out of the tone curve, and the cover
    /// that sets the HDR alpha to 1 before the first of them
    /// ([`UNTONEMAPPED_BLEND`]).
    sprite_untonemapped_pipeline: Deferred<wgpu::RenderPipeline>,
    sprite_cover_pipeline: Deferred<wgpu::RenderPipeline>,
    sprite_glow_pipeline: Deferred<wgpu::RenderPipeline>,
    msdf_pipeline: Deferred<wgpu::RenderPipeline>,
    ribbon_pipelines: [Deferred<wgpu::RenderPipeline>; 2],
    fx_layout: wgpu::BindGroupLayout,
    fx_buffer: wgpu::Buffer,
    sprite_fx_layout: wgpu::BindGroupLayout,
    sprite_fx_buffer: wgpu::Buffer,
    palette_layout: wgpu::BindGroupLayout,
    palette: skin::GpuPalette,
    skinned_buffer: wgpu::Buffer,
    prev_instance_buffer: wgpu::Buffer,
    ribbon_buffer: wgpu::Buffer,
    ribbon_sampler: wgpu::Sampler,
    /// Lines, points and wireframes (`lines.rs`).
    lines: lines::LineRenderer,
    particles: particles_gpu::ParticleRenderer,
    lights_2d: light2d::Light2dRenderer,
    trails: ribbons::Trails,
    /// The previous frame's unjittered view-projection (motion vectors).
    prev_view_proj: Option<Mat4>,
    fx_groups: Option<FxGroups>,
    mips: mips::MipGenerator,
    /// The physical lit pipelines, made when a scene first draws a physical
    /// material (`physical.rs`).
    physical: Option<physical::PhysicalPipelines>,
    /// The array-map variant of the lit, prepass and shadow pipelines, made
    /// when a scene first draws a material with array maps (`material.rs`).
    array_maps: Option<material::ArrayMaps>,
    /// The copy of the opaque scene transmission refracts, with its mip
    /// chain, at the renderer's size (made when a frame first needs it).
    transmission: Option<wgpu::Texture>,
    /// The frame bind group (camera, shadow maps, IBL) and the sky pass's
    /// (the sky cube) for the sky they were generated from.
    environment: Option<(
        EnvironmentKey,
        wgpu::BindGroup,
        wgpu::BindGroup,
        wgpu::BindGroup,
    )>,
    default_white: wgpu::TextureView,
    default_normal: wgpu::TextureView,
    white_sprite_group: wgpu::BindGroup,
    instance_buffer: wgpu::Buffer,
    /// What `instance_buffer` holds, so a frame writes only what changed.
    instance_upload: upload::DirtyUpload,
    sprite_buffer: wgpu::Buffer,
    overlay: overlay::OverlayRenderer,
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
    /// Whether the next frame's GPU culling counts are read back
    /// ([`Renderer::request_culling_counts`]).
    culling_counts: bool,
    /// The previous frame's unjittered view-projection, when TAA history is valid.
    taa_history: Option<Mat4>,
    /// The last frame's stylisation primitives, on or off
    /// ([`Stylise::chain_bits`]): a change resets the TAA history.
    stylise_chain: Option<u16>,
    /// The array sprite pipelines, made on the first frame that draws one.
    sprite_arrays: Option<sprite_array::SpriteArrayPipelines>,
    /// The sprite mesh pipelines and buffer, made on the first frame that draws one.
    sprite_meshes: Option<sprite_mesh::SpriteMeshRenderer>,
}

fn uniform_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn texture_entry(
    binding: u32,
    filterable: bool,
    dimension: wgpu::TextureViewDimension,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable },
            view_dimension: dimension,
            multisampled: false,
        },
        count: None,
    }
}

fn sampler_entry(binding: u32, kind: wgpu::SamplerBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(kind),
        count: None,
    }
}

fn tex2d(binding: u32) -> wgpu::BindGroupLayoutEntry {
    texture_entry(binding, true, wgpu::TextureViewDimension::D2)
}

/// The sRGB view format of a non-sRGB 8-bit output, when the backend can
/// view the texture (`surface`: a surface's) in it: the overlay blends
/// there in linear light, as on an sRGB output. `None` on an sRGB output,
/// or where reinterpretation is missing (GL: textures; GL and Android
/// Vulkan: surfaces), where the overlay shader encodes instead.
pub(crate) fn linear_view_format(
    adapter: &wgpu::Adapter,
    output: wgpu::TextureFormat,
    surface: bool,
) -> Option<wgpu::TextureFormat> {
    let flag = if surface {
        wgpu::DownlevelFlags::SURFACE_VIEW_FORMATS
    } else {
        wgpu::DownlevelFlags::VIEW_FORMATS
    };
    let srgb = output.add_srgb_suffix();
    (srgb != output && adapter.get_downlevel_capabilities().flags.contains(flag)).then_some(srgb)
}

pub(crate) fn target_texture(
    device: &wgpu::Device,
    label: &str,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

fn solid_texture(device: &wgpu::Device, queue: &wgpu::Queue, rgba: [u8; 4]) -> wgpu::TextureView {
    device
        .create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("default texture"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &rgba,
        )
        .create_view(&Default::default())
}

fn shader(device: &wgpu::Device, label: &str, source: String) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}

/// The pipeline state shared by the passes that draw into the HDR target.
pub(crate) struct HdrPipeline<'a> {
    pub label: &'a str,
    pub layout: &'a wgpu::PipelineLayout,
    pub module: &'a wgpu::ShaderModule,
    pub vs: &'a str,
    pub fs: &'a str,
    pub buffers: &'a [Option<wgpu::VertexBufferLayout<'a>>],
    pub cull: Option<wgpu::Face>,
    pub depth_write: bool,
    pub depth_compare: wgpu::CompareFunction,
    pub blend: Option<wgpu::BlendState>,
    pub samples: u32,
}

/// An untonemapped sprite's blend (`SpriteFx::tone_mapped` false): its
/// colour over the scene as alpha blending does, and the HDR target's
/// alpha (1 under the sprites, `fs_cover`) taken towards 0 by its
/// coverage, so the tonemap knows how much of each pixel to leave
/// uncurved (post.wgsl `tonemapped`). A tone-mapped sprite or glyph drawn
/// over it takes the alpha back towards 1 (`ALPHA_BLENDING`).
pub(crate) const UNTONEMAPPED_BLEND: wgpu::BlendState = wgpu::BlendState {
    color: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::SrcAlpha,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    },
    alpha: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Zero,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    },
};

/// Whether TAA's history was drawn from where the camera is now: the two
/// view-projections equal to a millionth of each entry, below what any
/// pixel could show. TAA then reads every pixel's own history texel
/// (taa.wgsl `still`), decided here rather than from the reprojection's
/// arithmetic on the GPU.
fn still_camera(prev: &Mat4, cur: &Mat4) -> bool {
    prev.to_cols_array()
        .iter()
        .zip(cur.to_cols_array())
        .all(|(p, c)| (p - c).abs() <= 1e-6 * c.abs().max(1.0))
}

pub(crate) fn hdr_pipeline(device: &wgpu::Device, p: HdrPipeline) -> wgpu::RenderPipeline {
    hdr_pipeline_with(device, &[], p)
}

/// [`hdr_pipeline`] made when a pass first sets it (deferred.rs): `state`
/// describes it over the layout and module it is handed.
pub(crate) fn deferred_hdr(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    module: impl AsModule + 'static,
    state: impl for<'a> FnOnce(&'a wgpu::PipelineLayout, &'a wgpu::ShaderModule) -> HdrPipeline<'a>
    + 'static,
) -> Deferred<wgpu::RenderPipeline> {
    let (device, layout) = (device.clone(), layout.clone());
    Deferred::new(move || hdr_pipeline(&device, state(&layout, module.module())))
}

/// The sprite pass's vertex buffers.
const SPRITE_BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] = &[Some(SPRITE_LAYOUT)];
/// The ribbon pass's vertex buffers.
const RIBBON_BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] =
    &[Some(ribbons::RIBBON_LAYOUT)];

/// [`hdr_pipeline`] with pipeline-overridable constants for both stages.
fn hdr_pipeline_with(
    device: &wgpu::Device,
    constants: &[(&str, f64)],
    p: HdrPipeline,
) -> wgpu::RenderPipeline {
    let options = wgpu::PipelineCompilationOptions {
        constants,
        ..Default::default()
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(p.label),
        layout: Some(p.layout),
        vertex: wgpu::VertexState {
            module: p.module,
            entry_point: Some(p.vs),
            compilation_options: options.clone(),
            buffers: p.buffers,
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: p.cull,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(p.depth_write),
            depth_compare: Some(p.depth_compare),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: p.samples,
            ..Default::default()
        },
        fragment: Some(wgpu::FragmentState {
            module: p.module,
            entry_point: Some(p.fs),
            compilation_options: options.clone(),
            targets: &[Some(wgpu::ColorTargetState {
                format: HDR_FORMAT,
                blend: p.blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: crate::pipeline_cache::of(device).as_ref(),
    })
}

/// Whether `adapter` can render the HDR target and depth with 4x MSAA and resolve it.
fn supports_msaa(adapter: &wgpu::Adapter) -> bool {
    let hdr = adapter.get_texture_format_features(HDR_FORMAT).flags;
    let depth = adapter.get_texture_format_features(DEPTH_FORMAT).flags;
    hdr.sample_count_supported(MSAA_SAMPLES)
        && hdr.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE)
        && depth.sample_count_supported(MSAA_SAMPLES)
}

/// Instances per lane task in the frustum tests: a test is a few
/// nanoseconds, so only thousands of instances are worth a second thread.
const CULL_GRAIN: usize = 1024;

/// The clear colour of a pass for the scene's background (black under a sky).
fn background_clear(background: &Background) -> wgpu::Color {
    match background {
        Background::Color(c) => wgpu::Color {
            r: c.x as f64,
            g: c.y as f64,
            b: c.z as f64,
            a: 1.0,
        },
        Background::Sky | Background::Procedural { .. } => wgpu::Color::BLACK,
    }
}

fn cull_mode(side: Side) -> Option<wgpu::Face> {
    match side {
        Side::Front => Some(wgpu::Face::Back),
        Side::Back => Some(wgpu::Face::Front),
        Side::Double => None,
    }
}

/// The lit pipeline for one (side, blending, skinned) combination.
fn pbr_pipeline(
    device: &wgpu::Device,
    layouts: (&wgpu::PipelineLayout, &wgpu::PipelineLayout),
    module: &wgpu::ShaderModule,
    key: PbrKey,
    samples: u32,
) -> wgpu::RenderPipeline {
    lit_pipeline(device, layouts, module, key, samples, false)
}

/// [`pbr_pipeline`] for the standard or the physical lit shader (whose
/// vertex stages also hand on the model's scale).
fn lit_pipeline(
    device: &wgpu::Device,
    layouts: (&wgpu::PipelineLayout, &wgpu::PipelineLayout),
    module: &wgpu::ShaderModule,
    (side, transparent, skinned, effects): PbrKey,
    samples: u32,
    physical: bool,
) -> wgpu::RenderPipeline {
    let plain = [Some(VERTEX_LAYOUT), Some(INSTANCE_LAYOUT)];
    let skin = [
        Some(VERTEX_LAYOUT),
        Some(skin::SKINNED_INSTANCE_LAYOUT),
        Some(skin::SKIN_VERTEX_LAYOUT),
    ];
    let constants = [("EFFECTS", effects as u32 as f64)];
    hdr_pipeline_with(
        device,
        &constants,
        HdrPipeline {
            label: match (physical, transparent) {
                (false, false) => "pbr",
                (false, true) => "pbr blended",
                (true, false) => "physical",
                (true, true) => "physical blended",
            },
            layout: if skinned { layouts.1 } else { layouts.0 },
            module,
            vs: match (physical, skinned) {
                (false, false) => "vs",
                (false, true) => "vs_skinned",
                (true, false) => "vs_physical",
                (true, true) => "vs_physical_skinned",
            },
            fs: "fs",
            buffers: if skinned { &skin } else { &plain },
            cull: cull_mode(side),
            // Blended surfaces test against the opaque depth but do not write it.
            depth_write: !transparent,
            depth_compare: wgpu::CompareFunction::Less,
            blend: transparent.then_some(wgpu::BlendState::ALPHA_BLENDING),
            samples,
        },
    )
}

/// The lean lit pipeline of a plain material ([`GpuMaterial::lean`]) for
/// (side, effects, derivatives, compact): opaque and unskinned by
/// construction, `pbr_lean.wgsl`'s entry points with `EFFECTS` as
/// [`lit_pipeline`]'s and without the discard unless the fragments take
/// derivatives, reading [`Vertex`] or the compact vertices.
fn lean_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
    (side, effects, derivatives, compact): LeanKey,
    samples: u32,
) -> wgpu::RenderPipeline {
    let constants = [("EFFECTS", effects as u32 as f64), ("DISCARD", 0.0)];
    hdr_pipeline_with(
        device,
        &constants[..if derivatives { 1 } else { 2 }],
        HdrPipeline {
            label: if compact {
                "pbr lean compact"
            } else {
                "pbr lean"
            },
            layout,
            module,
            vs: if compact {
                "vs_lean_compact"
            } else {
                "vs_lean"
            },
            fs: "fs_lean",
            buffers: &[
                Some(if compact {
                    compact::LEAN_LAYOUT
                } else {
                    LEAN_VERTEX_LAYOUT
                }),
                Some(INSTANCE_LAYOUT),
            ],
            cull: cull_mode(side),
            depth_write: true,
            depth_compare: wgpu::CompareFunction::Less,
            blend: None,
            samples,
        },
    )
}

/// `PrevIn` in pbr.wgsl: the previous transform (motion vectors).
const PREV_INSTANCE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: INSTANCE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![11 => Float32x3, 12 => Float32x3, 13 => Float32x3, 14 => Float32x3],
};

/// The depth-normal prepass pipeline for (side, skinned); `lean`: a plain
/// material's (unskinned, `pbr_lean.wgsl`'s entry points), as
/// `Some((derivatives, compact))` ([`Batch::derivatives`]: with them the
/// discard stays; `compact`: from the compact vertices).
fn prepass_pipeline(
    device: &wgpu::Device,
    layouts: (&wgpu::PipelineLayout, &wgpu::PipelineLayout),
    module: &wgpu::ShaderModule,
    side: Side,
    skinned: bool,
    lean: Option<(bool, bool)>,
) -> wgpu::RenderPipeline {
    let (derivatives, compact) = lean.unwrap_or_default();
    let lean = lean.is_some();
    let plain = [
        Some(match (lean, compact) {
            (true, true) => compact::POSITION_NORMAL_LAYOUT,
            (true, false) => POSITION_NORMAL_LAYOUT,
            (false, _) => VERTEX_LAYOUT,
        }),
        Some(INSTANCE_LAYOUT),
        Some(PREV_INSTANCE_LAYOUT),
    ];
    let constants = [("DISCARD", 0.0)];
    let options = wgpu::PipelineCompilationOptions {
        constants: if lean && !derivatives {
            &constants
        } else {
            &[]
        },
        ..Default::default()
    };
    let skin = [
        Some(VERTEX_LAYOUT),
        Some(skin::SKINNED_INSTANCE_LAYOUT),
        Some(skin::SKIN_VERTEX_LAYOUT),
    ];
    let target = |format| {
        Some(wgpu::ColorTargetState {
            format,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        })
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(if lean { "prepass lean" } else { "prepass" }),
        layout: Some(if skinned { layouts.1 } else { layouts.0 }),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some(match (skinned, lean) {
                (true, _) => "vs_prepass_skinned",
                (false, true) if compact => "vs_prepass_lean_compact",
                (false, true) => "vs_prepass_lean",
                (false, false) => "vs_prepass",
            }),
            compilation_options: options.clone(),
            buffers: if skinned { &skin } else { &plain },
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: cull_mode(side),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some(if lean && !skinned {
                "fs_prepass_lean"
            } else {
                "fs_prepass"
            }),
            compilation_options: options,
            targets: &[
                target(post::GBUFFER_FORMAT),
                target(post::LINDEPTH_FORMAT),
                target(post::OUTLINE_FORMAT),
            ],
        }),
        multiview_mask: None,
        cache: crate::pipeline_cache::of(device).as_ref(),
    })
}

/// The shadow pipeline for casters drawn with (side, skinned, dissolve);
/// `hooked`: with the module's material shader (its vertex offset and
/// surface colour, the material bound). Casters that are neither skinned,
/// dissolving nor hooked need only their positions: `shadow.wgsl`'s lean
/// entry points (the same depths), from the compact vertices when
/// `compact`.
fn shadow_pipeline(
    device: &wgpu::Device,
    layouts: &[wgpu::PipelineLayout; 3],
    module: &wgpu::ShaderModule,
    (side, skinned, dissolve): ShadowKey,
    hooked: bool,
    compact: bool,
) -> wgpu::RenderPipeline {
    let lean = !skinned && !dissolve && !hooked;
    let compact = lean && compact;
    let plain = [
        Some(match (lean, compact) {
            (_, true) => compact::POSITION_LAYOUT,
            (true, false) => POSITION_LAYOUT,
            (false, false) => VERTEX_LAYOUT,
        }),
        Some(INSTANCE_LAYOUT),
    ];
    let skin = [
        Some(VERTEX_LAYOUT),
        Some(skin::SKINNED_INSTANCE_LAYOUT),
        Some(skin::SKIN_VERTEX_LAYOUT),
    ];
    let layout = match (skinned, dissolve || hooked) {
        (true, _) => &layouts[2],
        (false, true) => &layouts[1],
        (false, false) => &layouts[0],
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some(match (skinned, hooked) {
                (true, true) => "vs_skinned_hooked",
                (true, false) => "vs_skinned",
                (false, true) => "vs_hooked",
                (false, false) if compact => "vs_lean_compact",
                (false, false) if lean => "vs_lean",
                (false, false) => "vs",
            }),
            compilation_options: Default::default(),
            buffers: if skinned { &skin } else { &plain },
        },
        // Front-sided casters draw back faces only: the stored depth is the
        // far side of a closed caster, so its lit side never compares
        // against itself. Back-sided ones draw front faces; double-sided
        // ones (open surfaces) both.
        primitive: wgpu::PrimitiveState {
            cull_mode: match side {
                Side::Front => Some(wgpu::Face::Front),
                Side::Back => Some(wgpu::Face::Back),
                Side::Double => None,
            },
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some(match (hooked, dissolve) {
                (true, _) => "fs_hooked",
                (false, true) => "fs_dissolve",
                (false, false) if lean => "fs_lean",
                (false, false) => "fs",
            }),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::R32Float,
                blend: None,
                write_mask: wgpu::ColorWrites::RED,
            })],
        }),
        multiview_mask: None,
        cache: crate::pipeline_cache::of(device).as_ref(),
    })
}

impl Renderer {
    /// Times each render-graph pass's CPU encoding with `clock` (ms);
    /// `None` stops timing. Measurement only: nothing drawn depends on it.
    pub fn set_clock(&mut self, clock: Option<fn() -> f64>) {
        self.clock = clock;
        if clock.is_none() {
            self.pass_ms.clear();
        }
    }

    /// The last frame's render-graph passes in execution order with the
    /// milliseconds their encoding took; empty without a clock.
    pub fn pass_timings(&self) -> &[(String, f64)] {
        &self.pass_ms
    }

    /// The custom post passes of `scene` that failed to compile on this
    /// device, with the device's message: they are skipped
    /// (`post_custom.rs`).
    pub fn post_errors(&self, scene: &RenderScene) -> Vec<(PostHandle, String)> {
        self.custom_posts.errors(scene.id())
    }

    /// A headless renderer (draws to [`RenderTarget::Offscreen`] or caller views).
    /// On the web, WebGL2 needs a canvas: use [`Renderer::with_surface`].
    pub async fn new(desc: &RenderDesc) -> Result<Renderer, RenderError> {
        if renderer_disabled() {
            return Err(disabled_error());
        }
        let instance = desc.backend.instance();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
                ..Default::default()
            })
            .await
            .map_err(|e| RenderError::NoAdapter(format!("{:?}: {e}", desc.backend)))?;
        let format = desc.output_format.unwrap_or(DEFAULT_OUTPUT_FORMAT);
        Self::from_adapter(adapter, None, format, desc).await
    }

    /// A renderer that presents to `surface` ([`RenderTarget::Surface`]); `instance`
    /// must be the one the surface was created from (see [`Backend::instance`]).
    pub async fn with_surface(
        desc: &RenderDesc,
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
    ) -> Result<Renderer, RenderError> {
        if renderer_disabled() {
            return Err(disabled_error());
        }
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(|e| RenderError::NoAdapter(format!("{:?}: {e}", desc.backend)))?;
        let caps = surface.get_capabilities(&adapter);
        // An sRGB surface where one is offered (WebGL2, native windows): the
        // hardware encodes and blends in linear light, as offscreen. A
        // WebGPU canvas offers none; its sRGB view (below) does the same.
        let eight_bit = |f: &wgpu::TextureFormat, srgb: bool| {
            f.is_srgb() == srgb
                && matches!(
                    f.remove_srgb_suffix(),
                    wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
                )
        };
        let format = match desc.output_format {
            Some(f) => f,
            None => caps
                .formats
                .iter()
                .copied()
                .find(|f| eight_bit(f, true))
                .or_else(|| caps.formats.iter().copied().find(|f| eight_bit(f, false)))
                .ok_or_else(|| {
                    RenderError::Surface(format!(
                        "no 8-bit RGBA surface format in {:?}",
                        caps.formats
                    ))
                })?,
        };
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: desc.width.max(1),
            height: desc.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: linear_view_format(&adapter, format, true)
                .into_iter()
                .collect(),
            desired_maximum_frame_latency: 2,
        };
        Self::from_adapter(adapter, Some((surface, config)), format, desc).await
    }

    async fn from_adapter(
        adapter: wgpu::Adapter,
        surface: Option<(wgpu::Surface<'static>, wgpu::SurfaceConfiguration)>,
        output_format: wgpu::TextureFormat,
        desc: &RenderDesc,
    ) -> Result<Renderer, RenderError> {
        let adapter_info = adapter.get_info();
        let tier = if adapter
            .get_downlevel_capabilities()
            .flags
            .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
        {
            Tier::Full
        } else {
            Tier::NoCompute
        };
        let indirect = tier == Tier::Full
            && adapter
                .get_downlevel_capabilities()
                .flags
                .contains(wgpu::DownlevelFlags::INDIRECT_EXECUTION);
        let msaa_ok = supports_msaa(&adapter);
        let antialias = match desc.antialias {
            Antialias::Off => Antialias::Off,
            Antialias::Taa => Antialias::Taa,
            Antialias::Fxaa => Antialias::Fxaa,
            Antialias::Tier if tier == Tier::Full => Antialias::Taa,
            Antialias::Tier | Antialias::Msaa4 if msaa_ok => Antialias::Msaa4,
            // No multisampled HDR target: the post-process instead.
            Antialias::Tier | Antialias::Msaa4 => Antialias::Fxaa,
        };
        let samples = if antialias == Antialias::Msaa4 {
            MSAA_SAMPLES
        } else {
            1
        };
        // The full tier asks for compute and storage buffers (particles, the
        // skinning palette); the no-compute tier for WebGL2's limits.
        let base_limits = if tier == Tier::Full {
            wgpu::Limits::downlevel_defaults()
        } else {
            wgpu::Limits::downlevel_webgl2_defaults()
        };
        let limits = base_limits.using_resolution(adapter.limits());
        let palette_mode = if tier == Tier::Full
            && adapter
                .get_downlevel_capabilities()
                .flags
                .contains(wgpu::DownlevelFlags::VERTEX_STORAGE)
        {
            skin::PaletteMode::Storage
        } else {
            skin::PaletteMode::Texture
        };
        let light_tables = if tier == Tier::Full {
            clusters::LightTables::Storage
        } else {
            clusters::LightTables::Uniform
        };
        let shadow_size = desc
            .shadow_map_size
            .clamp(64, limits.max_texture_dimension_2d.max(64));
        let cascades = desc.shadow_cascades.clamp(1, MAX_CASCADES);
        let (mut features, texture_formats) =
            TextureFormats::choose(adapter.features(), desc.texture_compression);
        let cache_dir = pipeline_cache::dir();
        if cache_dir.is_some() && adapter.features().contains(wgpu::Features::PIPELINE_CACHE) {
            features |= wgpu::Features::PIPELINE_CACHE;
        }
        let mut info = RenderInfo {
            adapter: adapter_info.name.clone(),
            backend: adapter_info.backend,
            device_type: adapter_info.device_type,
            tier,
            antialias,
            shadow_map_size: shadow_size,
            shadow_cascades: cascades,
            shadowed_lights: shadows::shadowed_lights(tier) as u32,
            point_shadows: desc.point_shadows.resolve(tier),
            texture_formats,
            pipeline_cache: false,
            max_texture_size: limits.max_texture_dimension_2d,
        };
        log::info!(
            "og-render: {} ({:?}, {:?}, {:?}, antialias {:?}, shadows {}x{}², textures {:?}/{:?})",
            info.adapter,
            info.backend,
            info.device_type,
            info.tier,
            info.antialias,
            cascades,
            shadow_size,
            info.texture_formats.color,
            info.texture_formats.normal
        );
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("og-render"),
                required_features: features,
                required_limits: limits,
                ..Default::default()
            })
            .await
            .map_err(|e| RenderError::NoDevice(e.to_string()))?;
        // Before any pipeline is made: every one is made with the cache.
        let pipeline_disk =
            cache_dir.and_then(|dir| pipeline_cache::DiskCache::open(&device, &adapter_info, dir));
        info.pipeline_cache = pipeline_disk.is_some();
        // No uncaptured-error handler on purpose: wgpu-core's default (native
        // and WebGL2) panics on a validation error, which is how the web host
        // ends the Worker and shows the shell's crash card and how native
        // hosts and tests fail loudly; logging instead would leave a silently
        // broken frame. The browser's WebGPU reports to its own console.
        let max = device.limits().max_texture_dimension_2d;
        let mut surface = surface;
        if let Some((surface, config)) = &mut surface {
            config.width = config.width.clamp(1, max);
            config.height = config.height.clamp(1, max);
            surface.configure(&device, config);
        }
        let (width, height) = (desc.width.clamp(1, max), desc.height.clamp(1, max));

        // Layouts.
        let [lights_entry, table_entry, refs_entry] = light_tables.layout_entries();
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("frame"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT),
                texture_entry(1, false, wgpu::TextureViewDimension::D2Array),
                texture_entry(2, false, wgpu::TextureViewDimension::D2Array),
                texture_entry(3, true, wgpu::TextureViewDimension::Cube),
                texture_entry(4, true, wgpu::TextureViewDimension::Cube),
                tex2d(5),
                sampler_entry(6, wgpu::SamplerBindingType::Filtering),
                // The local lights and their clusters: buffers, so WebGL2's
                // skinned lit pipeline keeps its sixteen texture slots.
                lights_entry,
                table_entry,
                refs_entry,
            ],
        });
        // The sky cube, bound by the sky pass alone: in the frame group it
        // would take a texture slot from every lit pipeline, and WebGL2's
        // skinned one needs them all (wgpu's GL backend has 16 per pipeline).
        let sky_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sky"),
            entries: &[texture_entry(0, true, wgpu::TextureViewDimension::Cube)],
        });
        let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material"),
            entries: &[
                // The vertex stage reads a material shader's parameters.
                uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT),
                tex2d(1),
                tex2d(2),
                tex2d(3),
                tex2d(4),
                tex2d(5),
                tex2d(6),
                sampler_entry(7, wgpu::SamplerBindingType::Filtering),
                tex2d(8),
                tex2d(9),
            ],
        });
        let fx_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fx"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT),
                tex2d(1),
                tex2d(2),
                sampler_entry(3, wgpu::SamplerBindingType::Filtering),
            ],
        });
        let sprite_fx_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite fx"),
            entries: &[
                tex2d(0),
                sampler_entry(1, wgpu::SamplerBindingType::Filtering),
                uniform_entry(2, wgpu::ShaderStages::FRAGMENT),
            ],
        });
        let palette_layout = palette_mode.layout(&device);
        let sprite_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite"),
            entries: &[
                tex2d(0),
                sampler_entry(1, wgpu::SamplerBindingType::Filtering),
                uniform_entry(2, wgpu::ShaderStages::FRAGMENT),
            ],
        });
        let shadow_bind_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shadow"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<ShadowPassUniform>() as u64,
                        ),
                    },
                    count: None,
                }],
            });

        // Samplers and fixed-size targets.
        let material_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("material"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let sprite_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let sprite_sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite (nearest)"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let cascade_map = ShadowMap::new(&device, "cascades", shadow_size, cascades);
        // Twelve layers to begin with; a frame that needs more grows it to
        // the tier's capacity (`grow_local_shadows`).
        let local_map = ShadowMap::new(
            &device,
            "local shadows",
            LOCAL_SHADOW_SIZE,
            shadows::INITIAL_LOCAL_LAYERS
                .min(shadows::capacity(
                    info.shadowed_lights as usize,
                    info.point_shadows,
                ))
                .max(1) as u32,
        );

        // Uniforms.
        let frame_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("frame"),
            size: std::mem::size_of::<frame::FrameUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_buffers = light_tables.buffers(&device);
        let reflection_light_buffers = light_tables.buffers(&device);
        let reflection_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("frame (planar reflection)"),
            size: std::mem::size_of::<frame::FrameUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let reflection_fx_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fx (planar reflection)"),
            size: std::mem::size_of::<fx::FxUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let shadow_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shadow passes"),
            size: SHADOW_SLOT_STRIDE * SHADOW_SLOTS as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow"),
            layout: &shadow_bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &shadow_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<ShadowPassUniform>() as u64),
                }),
            }],
        });

        // Pipelines.
        let pbr_shader = shader(
            &device,
            "pbr",
            lit_source_with(&hooks::default_hooks(), palette_mode, light_tables),
        );
        // Parsed when the first pipeline that uses it is made (deferred.rs).
        let sky_shader = deferred::module(&device, "sky", format!("{FRAME_WGSL}{SKY_WGSL}"));
        let sprite_shader = shader(
            &device,
            "sprite",
            format!("{FRAME_WGSL}{LIGHTING_WGSL}{SPRITE_WGSL}"),
        );
        let ribbon_shader =
            deferred::module(&device, "ribbon", format!("{FRAME_WGSL}{RIBBON_WGSL}"));
        let shadow_shader = shader(
            &device,
            "shadow",
            shadow_source(&hooks::default_hooks(), palette_mode),
        );
        let pipeline_layout = |label: &str, layouts: &[&wgpu::BindGroupLayout]| {
            let layouts: Vec<_> = layouts.iter().map(|l| Some(*l)).collect();
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &layouts,
                immediate_size: 0,
            })
        };
        let pbr_layout = pipeline_layout("pbr", &[&frame_layout, &material_layout, &fx_layout]);
        let pbr_skinned_layout = pipeline_layout(
            "pbr skinned",
            &[&frame_layout, &material_layout, &fx_layout, &palette_layout],
        );
        let sky_pl = pipeline_layout("sky", &[&frame_layout, &sky_layout]);
        let sprite_pl = pipeline_layout(
            "sprite",
            &[&frame_layout, &sprite_layout, &sprite_fx_layout],
        );
        let ribbon_pl = pipeline_layout("ribbon", &[&frame_layout, &sprite_layout]);
        let shadow_layouts = [
            pipeline_layout("shadow", &[&shadow_bind_layout]),
            pipeline_layout("shadow dissolve", &[&shadow_bind_layout, &material_layout]),
            pipeline_layout(
                "shadow skinned",
                &[&shadow_bind_layout, &material_layout, &palette_layout],
            ),
        ];
        let mut pbr_pipelines = HashMap::new();
        let key = (Side::Front, false, false, false);
        pbr_pipelines.insert(
            key,
            pbr_pipeline(
                &device,
                (&pbr_layout, &pbr_skinned_layout),
                &pbr_shader,
                key,
                samples,
            ),
        );
        let mut shadow_pipelines = HashMap::new();
        let key = (Side::Front, false, false);
        shadow_pipelines.insert(
            key,
            shadow_pipeline(&device, &shadow_layouts, &shadow_shader, key, false, false),
        );
        // At the far plane, after the opaque batches: only uncovered pixels pass.
        let sky_state = |fs: &'static str| {
            deferred_hdr(
                &device,
                &sky_pl,
                sky_shader.clone(),
                move |layout, module| HdrPipeline {
                    label: "sky",
                    layout,
                    module,
                    vs: "vs_sky",
                    fs,
                    buffers: &[],
                    cull: None,
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: None,
                    samples,
                },
            )
        };
        let sky_pipeline = sky_state("fs_sky");
        let sky_look_pipeline = sky_state("fs_sky_look");
        let ibl = ibl::IblGenerator::new(&device);
        let water =
            water::WaterRenderer::new(&device, &frame_layout, FRAME_WGSL, LIGHTING_WGSL, samples);
        let metaballs =
            metaballs::MetaballRenderer::new(&device, &frame_layout, FRAME_WGSL, samples);
        let fluid3d = fluid3d::Fluid3dRenderer::new(
            &device,
            &frame_layout,
            &fluid_shared_source(light_tables),
            samples,
        );
        let additive = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let sprite_state = |label: &'static str, fs: &'static str, blend| {
            deferred_hdr(
                &device,
                &sprite_pl,
                sprite_shader.clone(),
                move |layout, module| HdrPipeline {
                    label,
                    layout,
                    module,
                    vs: "vs",
                    fs,
                    buffers: SPRITE_BUFFERS,
                    cull: None,
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: Some(blend),
                    samples,
                },
            )
        };
        let sprite_pipeline = sprite_state("sprite", "fs", wgpu::BlendState::ALPHA_BLENDING);
        let sprite_additive_pipeline = sprite_state("sprite additive", "fs", additive);
        let sprite_untonemapped_pipeline =
            sprite_state("sprite untonemapped", "fs_untonemapped", UNTONEMAPPED_BLEND);
        // The colour kept, the alpha replaced by 1, over every pixel. No
        // groups: whichever sprite or array group is bound stays valid.
        let cover_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite cover"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let sprite_cover_pipeline = deferred_hdr(
            &device,
            &cover_pl,
            sprite_shader.clone(),
            move |layout, module| HdrPipeline {
                label: "sprite cover",
                layout,
                module,
                vs: "vs_cover",
                fs: "fs_cover",
                buffers: &[],
                cull: None,
                depth_write: false,
                depth_compare: wgpu::CompareFunction::Always,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Zero,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                samples,
            },
        );
        let msdf_pipeline = sprite_state("msdf text", "fs_msdf", wgpu::BlendState::ALPHA_BLENDING);
        // The glow buffer: no depth, one sample, added.
        let sprite_glow_pipeline = {
            let (device, sprite_pl, sprite_shader) =
                (device.clone(), sprite_pl.clone(), sprite_shader.clone());
            Deferred::new(move || {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("sprite glow"),
                    layout: Some(&sprite_pl),
                    vertex: wgpu::VertexState {
                        module: &sprite_shader,
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: SPRITE_BUFFERS,
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &sprite_shader,
                        entry_point: Some("fs_glow"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: HDR_FORMAT,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::One,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        let ribbon_state = |blend| {
            deferred_hdr(
                &device,
                &ribbon_pl,
                ribbon_shader.clone(),
                move |layout, module| HdrPipeline {
                    label: "ribbon",
                    layout,
                    module,
                    vs: "vs",
                    fs: "fs",
                    buffers: RIBBON_BUFFERS,
                    cull: None,
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: Some(blend),
                    samples,
                },
            )
        };
        let ribbon_pipelines = [
            ribbon_state(additive),
            ribbon_state(wgpu::BlendState::ALPHA_BLENDING),
        ];
        let particles = particles_gpu::ParticleRenderer::new(
            &device,
            &frame_layout,
            &sprite_layout,
            FRAME_WGSL,
            tier == Tier::Full,
        );
        let lights_2d = light2d::Light2dRenderer::new(&device);
        let ribbon_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ribbon"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let uniform_buffer = |label: &str, size: usize| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: size as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let fx_buffer = uniform_buffer("fx", std::mem::size_of::<fx::FxUniform>());
        let sprite_fx_buffer = uniform_buffer("sprite fx", 16);

        let post = post::PostPasses::new(
            &device,
            &queue,
            output_format,
            antialias,
            post::AaOptions {
                taa_history: desc.taa_history,
                taa_sharpen: desc.taa_sharpen.clamp(0.0, 1.0),
            },
        );
        let mips = mips::MipGenerator::new(&device);
        let default_white = solid_texture(&device, &queue, [255; 4]);
        let default_normal = solid_texture(&device, &queue, [128, 128, 255, 255]);
        let terrain = terrain::TerrainRenderer::new(
            &device,
            &queue,
            terrain::TerrainPipelines {
                frame_layout: &frame_layout,
                shadow_layout: &shadow_bind_layout,
                frame_wgsl: FRAME_WGSL,
                lighting_wgsl: LIGHTING_WGSL,
                light_tables_wgsl: &light_tables.wgsl(),
                samples,
                white: default_white.clone(),
                flat_normal: default_normal.clone(),
            },
        );
        let scatter = scatter::ScatterRenderer::new(
            &device,
            scatter::ScatterPipelines {
                frame_layout: &frame_layout,
                material_layout: &material_layout,
                shadow_layout: &shadow_bind_layout,
                lit_wgsl: scatter_lit_source(light_tables),
                shadow_wgsl: scatter_shadow_source(),
                samples,
            },
        );
        let vertex_buffer = |label: &str, size: u64| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let instance_buffer = vertex_buffer("instances", INSTANCE_STRIDE * 1024);
        let prev_instance_buffer = vertex_buffer("previous instances", INSTANCE_STRIDE * 1024);
        let skinned_buffer = vertex_buffer("skinned instances", skin::SKINNED_INSTANCE_STRIDE * 64);
        let ribbon_buffer = vertex_buffer("strips", 48 * 1024);
        let sprite_buffer = vertex_buffer("sprites", SPRITE_STRIDE * 1024);
        let white_sprite_group = sprite_bind_group(
            &device,
            &sprite_layout,
            &default_white,
            &sprite_sampler,
            [0.0; 4],
        );
        let offscreen_linear = linear_view_format(&adapter, output_format, false);
        let persistent = post::Persistent::new(
            &device,
            width,
            height,
            output_format,
            offscreen_linear,
            antialias,
        );
        let surface_linear = surface
            .as_ref()
            .and_then(|(_, config)| config.view_formats.first().copied());
        let overlay = overlay::OverlayRenderer::new(
            &device,
            output_format,
            offscreen_linear.or(surface_linear),
            &sprite_layout,
        );
        let glass = glass::GlassPasses::new(&device, output_format);
        let lines = lines::LineRenderer::new(
            &device,
            samples,
            output_format,
            offscreen_linear.or(surface_linear),
        );

        Ok(Renderer {
            device,
            queue,
            info,
            width,
            height,
            out_width: width,
            out_height: height,
            output_format,
            offscreen_linear,
            surface,
            samples,
            persistent,
            graph: post::FrameGraph::default(),
            clock: None,
            pass_ms: Vec::new(),
            cull_inside: Vec::new(),
            sorted: instances::SortedInstances::default(),
            views: views::Views::default(),
            custom_posts: post_custom::CustomPosts::default(),
            post,
            cascade_map,
            local_map,
            material_sampler,
            sprite_sampler,
            sprite_sampler_nearest,
            frame_layout,
            sky_layout,
            material_layout,
            sprite_layout,
            frame_buffer,
            light_tables,
            light_buffers,
            clusters: clusters::ClusterBuilder::default(),
            reflection_buffer,
            reflection_fx_buffer,
            reflection_light_buffers,
            reflection_clusters: clusters::ClusterBuilder::default(),
            shadow_buffer,
            shadow_bind_group,
            pbr_layout,
            pbr_skinned_layout,
            pbr_shader,
            shadow_layouts,
            shadow_shader,
            pbr_pipelines,
            shadow_pipelines,
            prepass_pipelines: HashMap::new(),
            lean_pipelines: HashMap::new(),
            lean_prepass_pipelines: HashMap::new(),
            compact_shadow_pipelines: HashMap::new(),
            stylise_object: None,
            sky_pipeline,
            sky_look_pipeline,
            water,
            fluid3d,
            terrain,
            scatter,
            metaballs,
            ibl,
            sprite_pipeline,
            sprite_additive_pipeline,
            sprite_untonemapped_pipeline,
            sprite_cover_pipeline,
            sprite_glow_pipeline,
            msdf_pipeline,
            ribbon_pipelines,
            fx_layout,
            fx_buffer,
            sprite_fx_layout,
            sprite_fx_buffer,
            palette_layout,
            palette: skin::GpuPalette::new(palette_mode),
            skinned_buffer,
            prev_instance_buffer,
            ribbon_buffer,
            ribbon_sampler,
            lines,
            particles,
            lights_2d,
            trails: ribbons::Trails::default(),
            prev_view_proj: None,
            fx_groups: None,
            mips,
            physical: None,
            array_maps: None,
            transmission: None,
            environment: None,
            default_white,
            default_normal,
            white_sprite_group,
            instance_buffer,
            instance_upload: upload::DirtyUpload::default(),
            sprite_buffer,
            overlay,
            glass,
            glass_targets: None,
            cache: SceneCache::default(),
            frame_index: 0,
            pipeline_disk,
            desc_particles: desc.particles,
            desc_culling: desc.culling,
            indirect,
            gpu_cull: None,
            culling_counts: false,
            taa_history: None,
            stylise_chain: None,
            sprite_arrays: None,
            sprite_meshes: None,
        })
    }

    pub fn info(&self) -> &RenderInfo {
        &self.info
    }

    pub fn size(&self) -> (u32, u32) {
        (self.out_width, self.out_height)
    }

    pub fn output_format(&self) -> wgpu::TextureFormat {
        self.output_format
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// The render graph the last frame was recorded from: its passes in
    /// execution order and its transient textures.
    pub fn frame_graph(&self) -> Option<&GraphPlan> {
        self.graph.plan()
    }

    /// Resizes every size-dependent target (and the surface, if any). TAA
    /// history restarts. Each side is clamped to 1 ..= the device's largest
    /// 2D texture.
    pub fn resize(&mut self, width: u32, height: u32) {
        let max = self.device.limits().max_texture_dimension_2d;
        let (width, height) = (width.clamp(1, max), height.clamp(1, max));
        if (width, height) == (self.out_width, self.out_height) {
            return;
        }
        self.out_width = width;
        self.out_height = height;
        // The scene follows the output until the next frame's stylisation
        // sets its own grid (`scene_size`).
        self.width = width;
        self.height = height;
        self.persistent = post::Persistent::new(
            &self.device,
            width,
            height,
            self.output_format,
            self.offscreen_linear,
            self.info.antialias,
        );
        self.graph.invalidate();
        self.taa_history = None;
        if let Some((surface, config)) = &mut self.surface {
            config.width = width;
            config.height = height;
            surface.configure(&self.device, config);
        }
    }

    /// The size the scene is drawn at this frame: the output's, or the
    /// stylisation's logical grid ([`PixelScale::fit`]), with the
    /// whole-number upscale and the scaled frame's offset in the output.
    fn pixel_grid(&self, pixel: Option<PixelScale>) -> ((u32, u32), u32, (u32, u32)) {
        let out = (self.out_width, self.out_height);
        let Some(p) = pixel.filter(|p| p.height > 0) else {
            return (out, 1, (0, 0));
        };
        let f = p.fit(out);
        ((f.width, f.height), f.upscale, f.offset)
    }

    /// Draws the scene at `width` × `height` from now on (the output keeps
    /// its size): the graph, the TAA history and the glass targets follow.
    fn scene_size(&mut self, width: u32, height: u32) {
        if (width, height) == (self.width, self.height) {
            return;
        }
        self.width = width;
        self.height = height;
        self.persistent
            .resize_history(&self.device, width, height, self.info.antialias);
        self.graph.invalidate();
        self.taa_history = None;
    }

    /// The environment maps, texture arrays and palettes the scene added,
    /// refilled or removed since the last frame: each slot whose revision
    /// moved is uploaded again, or freed when the scene removed it.
    fn sync_assets(&mut self, scene: &RenderScene) -> Result<(), RenderError> {
        let revisions = scene.environment_map_revisions();
        for (i, &revision) in revisions.iter().enumerate() {
            let fresh = i >= self.cache.environment_maps.len();
            if !fresh && self.cache.environment_map_revisions[i] == revision {
                continue;
            }
            let gpu = if scene.environment_map_slot(i).0 {
                let map = &scene.environment_maps()[i];
                map.validate()
                    .map_err(|e| RenderError::InvalidData(format!("environment map {i}: {e}")))?;
                let (texture, view) = environment::upload(&self.device, &self.queue, map);
                Some(GpuEnvironmentMap {
                    _texture: texture,
                    view,
                    bytes: 4 * environment::EnvironmentMapData::texel_count(map.size) as u64,
                })
            } else {
                None
            };
            if fresh {
                self.cache.environment_maps.push(gpu);
                self.cache.environment_map_revisions.push(revision);
            } else {
                self.cache.environment_maps[i] = gpu;
                self.cache.environment_map_revisions[i] = revision;
            }
        }
        let revisions = scene.texture_array_revisions();
        for (i, &revision) in revisions.iter().enumerate() {
            let fresh = i >= self.cache.texture_arrays.len();
            if !fresh && self.cache.texture_array_revisions[i] == revision {
                continue;
            }
            let (live, generation) = scene.texture_array_slot(i);
            let gpu = if live {
                let data = &scene.texture_arrays()[i];
                let bad = |e: String| RenderError::InvalidData(format!("texture array {i}: {e}"));
                let info = data.info().map_err(bad)?;
                if info.format != TextureFormat::Rgba8
                    && !self.info.texture_formats.supported.contains(&info.format)
                {
                    return Err(bad(format!("{} is not supported here", info.format.name())));
                }
                let mut gpu =
                    texture_array::upload(&self.device, &self.queue, data, info).map_err(bad)?;
                gpu.generation = generation;
                Some(gpu)
            } else {
                None
            };
            if fresh {
                self.cache.texture_arrays.push(gpu);
                self.cache.texture_array_revisions.push(revision);
                self.cache.sprite_array_groups.push(None);
            } else {
                self.cache.texture_arrays[i] = gpu;
                self.cache.texture_array_revisions[i] = revision;
                self.cache.sprite_array_groups[i] = None;
            }
        }
        let revisions = scene.palette_revisions();
        for (i, &revision) in revisions.iter().enumerate() {
            let fresh = i >= self.cache.palettes.len();
            if !fresh && self.cache.palette_revisions[i] == revision {
                continue;
            }
            let view = scene.palette_live(i).then(|| {
                let palette = &scene.palettes()[i];
                let mut rgba = vec![0u8; MAX_PALETTE_COLORS * 4];
                for (k, c) in palette.colors.iter().enumerate() {
                    rgba[k * 4..k * 4 + 3].copy_from_slice(c);
                    rgba[k * 4 + 3] = 255;
                }
                post::texture_from_rgba8(
                    &self.device,
                    &self.queue,
                    "palette",
                    MAX_PALETTE_COLORS as u32,
                    1,
                    wgpu::TextureFormat::Rgba8UnormSrgb,
                    &rgba,
                )
            });
            if fresh {
                self.cache.palettes.push(view);
                self.cache.palette_revisions.push(revision);
            } else {
                self.cache.palettes[i] = view;
                self.cache.palette_revisions[i] = revision;
            }
        }
        Ok(())
    }

    /// GPU bytes of the scene's environment maps, texture arrays, palettes
    /// and colour LUTs (removed ones are freed and count nothing).
    fn asset_bytes(&self) -> u64 {
        let maps: u64 = self
            .cache
            .environment_maps
            .iter()
            .flatten()
            .map(|m| m.bytes)
            .sum();
        let arrays: u64 = self
            .cache
            .texture_arrays
            .iter()
            .flatten()
            .map(|a| a.info.bytes)
            .sum();
        let palettes = self.cache.palettes.iter().flatten().count() as u64 * PALETTE_BYTES;
        let luts: u64 = self.cache.luts.iter().flatten().map(|l| l.1).sum();
        maps + arrays + palettes + luts
    }

    /// The array sprite pipelines and the bind group of `array` (made on
    /// first use; again when the array's slot or filter changed).
    fn sprite_array_group(&mut self, scene: &RenderScene, array: TextureArrayHandle) {
        if self.sprite_arrays.is_none() {
            self.sprite_arrays = Some(sprite_array::SpriteArrayPipelines::new(
                &self.device,
                &self.frame_layout,
                &self.sprite_fx_layout,
                self.samples,
            ));
        }
        let i = array.index();
        let filter = scene.texture_array_filter(array);
        if self.cache.sprite_array_groups[i]
            .as_ref()
            .is_some_and(|(_, f)| *f == filter)
        {
            return;
        }
        let Some(gpu) = self.cache.texture_arrays[i].as_ref() else {
            return;
        };
        let sampler = match filter {
            TextureFilter::Nearest => &self.sprite_sampler_nearest,
            TextureFilter::Linear => &self.sprite_sampler,
        };
        let group = self.sprite_arrays.as_ref().expect("made above").bind_group(
            &self.device,
            &gpu.texture,
            sampler,
        );
        self.cache.sprite_array_groups[i] = Some((group, filter));
    }

    /// Uploads whatever the scene added (or changed) since the last frame.
    fn sync(&mut self, scene: &RenderScene) -> Result<(), RenderError> {
        if self.cache.scene_id != scene.id() {
            self.cache = SceneCache {
                scene_id: scene.id(),
                ..Default::default()
            };
            self.taa_history = None;
        }
        for index in self.cache.shaders.len()..scene.shaders().len() {
            let (lit, shadow) = custom::module_sources(
                &scene.shaders()[index],
                self.palette.mode(),
                self.light_tables,
            );
            let module = shader(&self.device, "pbr (material shader)", lit);
            self.cache.shaders.push(module);
            let module = shader(&self.device, "shadow (material shader)", shadow);
            self.cache.shadow_shaders.push(module);
        }
        self.refresh_meshes(scene);
        for index in self.cache.meshes.len()..scene.meshes().len() {
            let mesh = &scene.meshes()[index];
            let n = mesh.vertices.len();
            // Checked before anything joins the cache: `meshes`, `morph_ranges`
            // and `skins` stay the same length, and a bad mesh fails again
            // next frame instead of shifting its neighbours' entries.
            let morph = scene.morphs()[index].as_ref();
            if let Some(m) = morph {
                let targets = m.positions.len();
                let bad = targets == 0
                    || targets > MAX_MORPH_TARGETS
                    || m.positions.iter().any(|t| t.len() != n)
                    || m.normals
                        .as_ref()
                        .is_some_and(|ns| ns.len() != targets || ns.iter().any(|t| t.len() != n));
                if bad {
                    return Err(RenderError::InvalidData(format!(
                        "mesh {index}: {targets} morph targets (1 to {MAX_MORPH_TARGETS}, one delta per vertex each, {n} vertices)"
                    )));
                }
            }
            // A morphed mesh without a skin: all-zero weights (bind pose).
            let zero_skin;
            let source = match &scene.skins()[index] {
                None if morph.is_some() => {
                    zero_skin = SkinData {
                        joints: vec![[0; 4]; n],
                        weights: vec![[0.0; 4]; n],
                    };
                    Some(&zero_skin)
                }
                other => other.as_ref(),
            };
            if let Some(skin) = source
                && (skin.joints.len() != n || skin.weights.len() != n)
            {
                return Err(RenderError::InvalidData(format!(
                    "mesh {index}: a skin of {} joints and {} weights for {n} vertices",
                    skin.joints.len(),
                    skin.weights.len()
                )));
            }
            let compact = scene.mesh_encodings()[index] == VertexEncoding::Compact
                && source.is_none()
                && morph.is_none();
            let mut gpu = self.upload_mesh(mesh, compact);
            gpu.revision = scene.mesh_revisions()[index];
            self.cache.meshes.push(gpu);
            let mut range = (0u32, 0u32);
            if let Some(m) = morph {
                let targets = m.positions.len();
                range = (self.cache.morph_data.len() as u32, targets as u32);
                for v in 0..n {
                    for t in 0..targets {
                        self.cache
                            .morph_data
                            .push(m.positions[t][v].extend(0.0).to_array());
                        let nd = m.normals.as_ref().map_or(Vec3::ZERO, |ns| ns[t][v]);
                        self.cache.morph_data.push(nd.extend(0.0).to_array());
                    }
                }
                self.cache.morphs_dirty = true;
            }
            self.cache.morph_ranges.push(range);
            let skin = match source {
                None => None,
                Some(skin) => {
                    let data: Vec<skin::SkinVertex> = skin
                        .joints
                        .iter()
                        .zip(&skin.weights)
                        .map(|(&joints, &weights)| skin::SkinVertex { joints, weights })
                        .collect();
                    let mut bytes = bytemuck::cast_slice(&data).to_vec();
                    if bytes.is_empty() {
                        bytes = vec![0u8; 24];
                    }
                    Some(
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("skin"),
                                contents: &bytes,
                                usage: wgpu::BufferUsages::VERTEX,
                            }),
                    )
                }
            };
            self.cache.skins.push(skin);
        }
        self.sync_assets(scene)?;
        let revisions = scene.lut_revisions();
        for (i, &revision) in revisions.iter().enumerate() {
            let fresh = i >= self.cache.luts.len();
            if !fresh && self.cache.lut_revisions[i] == revision {
                continue;
            }
            let gpu = if scene.lut_live(i) {
                let lut = &scene.luts()[i];
                lut.validate()
                    .map_err(|e| RenderError::InvalidData(format!("lut {i}: {e}")))?;
                let bytes = u64::from(lut.size).pow(3) * 4;
                Some((post::lut_texture(&self.device, &self.queue, lut), bytes))
            } else {
                None
            };
            if fresh {
                self.cache.luts.push(gpu);
                self.cache.lut_revisions.push(revision);
            } else {
                self.cache.luts[i] = gpu;
                self.cache.lut_revisions[i] = revision;
            }
        }
        if self.cache.textures.len() < scene.textures().len() {
            // Every texture is checked before any is recorded: an error
            // never drops an encoder holding earlier textures' mip passes.
            for index in self.cache.textures.len()..scene.textures().len() {
                self.check_texture(index, &scene.textures()[index])?;
            }
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("texture upload"),
                });
            for index in self.cache.textures.len()..scene.textures().len() {
                let gpu = self.upload_source(&scene.textures()[index], &mut encoder);
                self.cache.textures.push(gpu);
                self.cache
                    .texture_revisions
                    .push(scene.texture_revisions()[index]);
                self.cache.sprite_groups.push(None);
                self.cache.ribbon_groups.push(None);
            }
            self.queue.submit(Some(encoder.finish()));
        }
        // Replaced textures: uploaded again, and everything bound to the old
        // view (materials, sprite and trail groups, fonts, the effect
        // groups' decal atlas, terrain layers) made again.
        let replaced: Vec<usize> = (0..self.cache.textures.len())
            .filter(|&i| self.cache.texture_revisions[i] != scene.texture_revisions()[i])
            .collect();
        if !replaced.is_empty() {
            for &index in &replaced {
                self.check_texture(index, &scene.textures()[index])?;
            }
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("texture replace"),
                });
            for &index in &replaced {
                let gpu = self.upload_source(&scene.textures()[index], &mut encoder);
                self.cache.textures[index] = gpu;
                self.cache.texture_revisions[index] = scene.texture_revisions()[index];
                self.cache.sprite_groups[index] = None;
                self.cache.ribbon_groups[index] = None;
            }
            self.queue.submit(Some(encoder.finish()));
            self.fx_groups = None;
            self.terrain.forget_groups();
        }
        // A pack holding a replaced texture is made again with its materials.
        let replaced_slot = |slot: &u32| {
            TextureHandle::from_slot(*slot).is_some_and(|h| replaced.contains(&h.index()))
        };
        self.cache
            .packs
            .retain(|key, _| !key.iter().any(replaced_slot));
        let samples_replaced = |m: &StandardMaterial| {
            m.textures()
                .iter()
                .chain(m.physical_textures().iter())
                .flatten()
                .any(|h| replaced.contains(&h.index()))
        };
        // The scene's material style: every lit material takes it where it
        // leaves a field at its default, so a change restyles them all.
        let restyle = self.cache.material_style != scene.stylise.material;
        self.cache.material_style = scene.stylise.material;
        for (i, (material, revision)) in scene.materials().enumerate() {
            if self
                .cache
                .materials
                .get(i)
                .is_some_and(|m| m.revision == revision)
                && !samples_replaced(material)
                && !restyle
            {
                continue;
            }
            let styled = material::styled(material, &scene.stylise.material);
            let material = &styled;
            let hooked = scene.material_shader(MaterialHandle::from_index(i));
            if material.draws_array_maps() && hooked.is_none() && self.array_maps.is_none() {
                self.array_maps = Some(material::ArrayMaps::new(
                    &self.device,
                    &self.queue,
                    [&self.frame_layout, &self.fx_layout, &self.palette_layout],
                    lit_source_with(
                        &hooks::default_hooks(),
                        self.palette.mode(),
                        self.light_tables,
                    ),
                    shadow_source(&hooks::default_hooks(), self.palette.mode()),
                ));
            }
            let (bind_group, array, factors) =
                self.material_bind_group(i, material, hooked.map(|h| h.1))?;
            // A full shader (custom.rs) draws a physical material with its
            // own entries: no physical pipelines.
            let full = hooked.is_some_and(|(sh, _)| {
                scene.shaders()[sh.index()].mode != hooks::ShaderMode::Hooks
            });
            let physical = if material.is_physical() && !full {
                if let Some((sh, _)) = hooked
                    && !self.cache.physical_shaders.contains_key(&sh.index())
                {
                    let hooks = scene.shaders()[sh.index()].hooks_wgsl();
                    let module = shader(
                        &self.device,
                        "physical (material shader)",
                        physical_source_with(&hooks, self.palette.mode(), self.light_tables),
                    );
                    self.cache.physical_shaders.insert(sh.index(), module);
                }
                Some(self.physical_bind_group(i, material, &factors)?)
            } else {
                None
            };
            let transmissive = physical.is_some() && material.is_transmissive();
            let gpu = GpuMaterial {
                revision,
                bind_group,
                physical,
                array,
                side: material.side(),
                transparent: material.is_transparent() || transmissive,
                blend: material.is_transparent(),
                transmissive,
                dissolve: material.dissolves(),
                effects: material.toon != 0
                    || material.rimIntensity > 0.0
                    || material.flash > 0.0
                    || material.dissolves(),
                outline: material.outlineWidth > 0.0,
                shader: hooked.map(|h| h.0.index()),
                // An array-map instance's transform carries its layer: the
                // occluder depth pass reads transforms as they are.
                occluder: !material.is_transparent()
                    && !transmissive
                    && !material.dissolves()
                    && material.alphaTest <= 0.0
                    && hooked.is_none()
                    && !material.draws_array_maps(),
                lean: false,
                flat: material.flatShading != 0,
                wireframe: (material.wireframe != 0).then(|| {
                    let c = material.color + material.emissive * material.emissiveIntensity;
                    [c.x, c.y, c.z, material.opacity]
                }),
            };
            let gpu = GpuMaterial {
                lean: gpu.occluder
                    && gpu.physical.is_none()
                    && !gpu.effects
                    && !gpu.outline
                    && material.textures().iter().all(Option::is_none),
                ..gpu
            };
            if i < self.cache.materials.len() {
                self.cache.materials[i] = gpu;
            } else {
                self.cache.materials.push(gpu);
            }
        }
        let stale_fonts: Vec<usize> = (0..self.cache.fonts.len())
            .filter(|&i| {
                replaced.contains(&scene.fonts()[i].atlas.index())
                    || self.cache.fonts[i].revision != scene.font_revisions()[i]
            })
            .collect();
        self.cache
            .fonts
            .truncate(stale_fonts.first().copied().unwrap_or(usize::MAX));
        let start = self.cache.fonts.len();
        for (i, font) in scene.fonts()[start..].iter().enumerate() {
            let index = start + i;
            let atlas = font.atlas.index();
            let (Some(gpu), Some((w, h))) = (
                self.cache.textures.get(atlas),
                scene.textures().get(atlas).map(TextureSource::size),
            ) else {
                return Err(RenderError::InvalidHandle(format!(
                    "font {index}: atlas texture {atlas}"
                )));
            };
            let atlas_size = Vec2::new(w as f32, h as f32);
            let bind_group = sprite_bind_group(
                &self.device,
                &self.sprite_layout,
                &gpu.view,
                &self.sprite_sampler,
                [font.distance_range, atlas_size.x, atlas_size.y, 0.0],
            );
            self.cache.fonts.push(GpuFont {
                layout: FontLayout::new(font, atlas_size),
                bind_group,
                revision: scene.font_revisions()[index],
            });
        }
        Ok(())
    }

    /// The f32 vertices of `mesh` on the GPU (its tangents generated where
    /// it lacks them); a 16-byte placeholder for a mesh without vertices.
    fn upload_vertices(&self, mesh: &MeshData) -> wgpu::Buffer {
        let tangents;
        let mesh = if mesh.lacks_tangents() {
            let mut m = mesh.clone();
            m.generate_tangents();
            tangents = m;
            &tangents
        } else {
            mesh
        };
        let bytes: &[u8] = bytemuck::cast_slice(&mesh.vertices);
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertices"),
                contents: if bytes.is_empty() { &[0u8; 16] } else { bytes },
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
    }

    /// `mesh` on the GPU; `compact`: as [`VertexEncoding::Compact`], whose
    /// f32 vertices wait for a pipeline that reads them.
    fn upload_mesh(&self, mesh: &MeshData, compact: bool) -> GpuMesh {
        // Never create an empty buffer: an empty mesh keeps a 16-byte placeholder and draws nothing.
        let bytes = |b: &[u8]| {
            if b.is_empty() {
                vec![0u8; 16]
            } else {
                b.to_vec()
            }
        };
        let compact = compact
            .then(|| compact::encode(mesh))
            .flatten()
            .map(|(v, dq)| {
                let buffer = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("compact vertices"),
                        contents: bytemuck::cast_slice(&v),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                (buffer, dq)
            });
        let full = compact.is_none();
        let vertices = if full {
            self.upload_vertices(mesh)
        } else {
            self.upload_vertices(&MeshData::default())
        };
        let indices = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("indices"),
                contents: &bytes(bytemuck::cast_slice(&mesh.indices)),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });
        let bounds = mesh
            .bounds()
            .map(|(lo, hi)| ((lo + hi) * 0.5, (hi - lo) * 0.5));
        GpuMesh {
            vertices,
            full,
            compact,
            indices,
            index_count: if mesh.vertices.is_empty() {
                0
            } else {
                mesh.indices.len() as u32
            },
            bounds,
            revision: 0,
        }
    }

    /// Uploads meshes the scene replaced since they were last uploaded: in
    /// place when the sizes match, else as new buffers.
    fn refresh_meshes(&mut self, scene: &RenderScene) {
        for (i, &revision) in scene.mesh_revisions().iter().enumerate() {
            let Some(gpu) = self.cache.meshes.get(i) else {
                break;
            };
            if gpu.revision == revision {
                continue;
            }
            let source = &scene.meshes()[i];
            let compact = scene.mesh_encodings()[i] == VertexEncoding::Compact
                && scene.skins()[i].is_none()
                && scene.morphs()[i].is_none();
            if compact || gpu.compact.is_some() {
                let mut fresh = self.upload_mesh(source, compact);
                fresh.revision = revision;
                self.cache.meshes[i] = fresh;
                continue;
            }
            let tangents;
            let mesh = if source.lacks_tangents() {
                let mut m = source.clone();
                m.generate_tangents();
                tangents = m;
                &tangents
            } else {
                source
            };
            let vb: &[u8] = bytemuck::cast_slice(&mesh.vertices);
            let ib: &[u8] = bytemuck::cast_slice(&mesh.indices);
            if !vb.is_empty()
                && vb.len() as u64 == gpu.vertices.size()
                && ib.len() as u64 == gpu.indices.size()
            {
                self.queue.write_buffer(&gpu.vertices, 0, vb);
                self.queue.write_buffer(&gpu.indices, 0, ib);
                let bounds = mesh
                    .bounds()
                    .map(|(lo, hi)| ((lo + hi) * 0.5, (hi - lo) * 0.5));
                let gpu = &mut self.cache.meshes[i];
                gpu.bounds = bounds;
                gpu.revision = revision;
            } else {
                let mut fresh = self.upload_mesh(mesh, false);
                fresh.revision = revision;
                self.cache.meshes[i] = fresh;
            }
        }
    }

    /// Uploads a compact mesh's f32 vertices, for a pipeline that reads
    /// [`Vertex`] (a material that is not plain, a scatter, a skinned
    /// instance); nothing for a mesh that has them.
    fn ensure_full(&mut self, scene: &RenderScene, mesh: usize) {
        if self.cache.meshes.get(mesh).is_none_or(|m| m.full) {
            return;
        }
        let vertices = self.upload_vertices(&scene.meshes()[mesh]);
        let gpu = &mut self.cache.meshes[mesh];
        gpu.vertices = vertices;
        gpu.full = true;
    }

    /// Checks a texture before it is uploaded: its data, its format on this
    /// adapter and its size against the device's limit.
    fn check_texture(&self, index: usize, source: &TextureSource) -> Result<(), RenderError> {
        match source {
            TextureSource::Rgba(texture) => {
                let expected = texture.width as usize * texture.height as usize * 4;
                if texture.width == 0 || texture.height == 0 || texture.rgba.len() != expected {
                    return Err(RenderError::InvalidData(format!(
                        "texture {index}: {}x{} with {} bytes (expected {expected})",
                        texture.width,
                        texture.height,
                        texture.rgba.len()
                    )));
                }
            }
            TextureSource::Encoded(upload) => {
                let desc = &upload.desc;
                desc.validate(&upload.levels)
                    .map_err(|e| RenderError::InvalidData(format!("texture {index}: {e}")))?;
                if !self.info.texture_formats.supported.contains(&desc.format) {
                    return Err(RenderError::Unsupported(format!(
                        "texture {index}: {:?} on {} {:?} (supported: {:?}; transcode to RenderInfo::texture_formats)",
                        desc.format,
                        self.info.adapter,
                        self.info.backend,
                        self.info.texture_formats.supported
                    )));
                }
            }
            TextureSource::Target(_) => {}
        }
        let (width, height) = source.size();
        let max = self.device.limits().max_texture_dimension_2d;
        if width > max || height > max {
            return Err(RenderError::Unsupported(format!(
                "texture {index}: {width}x{height} exceeds this device's {max}x{max} (resize it, or cook a smaller level 0)"
            )));
        }
        Ok(())
    }

    /// Uploads a texture [`Renderer::check_texture`] accepted.
    fn upload_source(
        &mut self,
        source: &TextureSource,
        encoder: &mut wgpu::CommandEncoder,
    ) -> GpuTexture {
        match source {
            TextureSource::Rgba(texture) => GpuTexture {
                view: self.upload_texture(texture, encoder),
                format: TextureFormat::Rgba8,
                bytes: rgba_texture_bytes(texture),
            },
            TextureSource::Encoded(upload) => self.upload_encoded(upload),
            TextureSource::Target(t) => GpuTexture {
                view: targets::create(&self.device, *t).create_view(&Default::default()),
                format: TextureFormat::Rgba8,
                bytes: t.bytes(),
            },
        }
    }

    fn upload_texture(
        &mut self,
        texture: &TextureData,
        encoder: &mut wgpu::CommandEncoder,
    ) -> wgpu::TextureView {
        let format = match texture.color_space {
            ColorSpace::Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            ColorSpace::Linear => wgpu::TextureFormat::Rgba8Unorm,
        };
        let mip_level_count = match texture.mips {
            Mips::Generate => mips::mip_count(texture.width, texture.height),
            Mips::None => 1,
        };
        let mut usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        if mip_level_count > 1 {
            usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        }
        let size = wgpu::Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: 1,
        };
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texture.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(texture.width * 4),
                rows_per_image: Some(texture.height),
            },
            size,
        );
        if mip_level_count > 1 {
            self.mips.generate(&self.device, encoder, &tex);
        }
        tex.create_view(&Default::default())
    }

    /// A texture whose levels arrive encoded (no mip generation).
    fn upload_encoded(&self, upload: &TextureUpload) -> GpuTexture {
        let desc = &upload.desc;
        let format = desc.format.wgpu(desc.color_space).expect("checked");
        let texture = self.device.create_texture_with_data(
            &self.queue,
            &wgpu::TextureDescriptor {
                label: Some("texture (encoded)"),
                size: wgpu::Extent3d {
                    width: desc.width,
                    height: desc.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: desc.mip_levels,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &upload.levels.concat(),
        );
        GpuTexture {
            view: texture.create_view(&Default::default()),
            format: desc.format,
            bytes: upload.levels.iter().map(|l| l.len() as u64).sum(),
        }
    }

    /// A material's standard group, its array-map group when it draws with
    /// array maps (made by the caller's [`material::ArrayMaps`]), and its
    /// uniform buffer.
    #[allow(clippy::type_complexity)]
    fn material_bind_group(
        &self,
        index: usize,
        m: &StandardMaterial,
        custom: Option<[[f32; 4]; MAX_SHADER_PARAMS]>,
    ) -> Result<(wgpu::BindGroup, Option<wgpu::BindGroup>, wgpu::Buffer), RenderError> {
        let [
            map,
            metalness,
            roughness,
            normal,
            emissive,
            ao,
            gradient,
            dissolve,
        ] = m.textures();
        let texture = |t: Option<TextureHandle>| -> Result<Option<&GpuTexture>, RenderError> {
            match t {
                None => Ok(None),
                Some(h) => self.cache.textures.get(h.index()).map(Some).ok_or_else(|| {
                    RenderError::InvalidHandle(format!(
                        "material {index} uses texture {} (slot {})",
                        h.index(),
                        h.slot()
                    ))
                }),
            }
        };
        let view = |t: Option<TextureHandle>, fallback: &wgpu::TextureView| {
            texture(t).map(|g| g.map_or_else(|| fallback.clone(), |g| g.view.clone()))
        };
        let base = view(map, &self.default_white)?;
        let metal = view(metalness, &self.default_white)?;
        let rough = view(roughness, &self.default_white)?;
        let normal_gpu = texture(normal)?;
        let normal_view =
            normal_gpu.map_or_else(|| self.default_normal.clone(), |g| g.view.clone());
        let emissive = view(emissive, &self.default_white)?;
        let ao = view(ao, &self.default_white)?;
        let gradient = view(gradient, &self.default_white)?;
        let dissolve = view(dissolve, &self.default_white)?;
        let two_channel = normal_gpu.is_some_and(|g| g.format.two_channel());
        let mut uniform: MaterialUniform = material::uniform(m, two_channel);
        if let Some(c) = custom {
            uniform.custom = c;
        }
        // The array maps' views (a material shader keeps the standard path).
        let arrays = match self.array_maps.as_ref() {
            Some(a) if custom.is_none() && m.draws_array_maps() => {
                let [map_array, normal_array] = m.array_maps().map(|h| {
                    h.map(|h| {
                        self.cache
                            .texture_arrays
                            .get(h.index())
                            .and_then(Option::as_ref)
                            .filter(|g| g.generation == h.generation())
                            .ok_or_else(|| {
                                RenderError::InvalidHandle(format!(
                                    "material {index} uses texture array {} (slot {})",
                                    h.index(),
                                    h.slot()
                                ))
                            })
                    })
                    .transpose()
                });
                let (map_array, normal_array) = (map_array?, normal_array?);
                let view = |g: Option<&texture_array::GpuTextureArray>,
                            fallback: &wgpu::TextureView| {
                    g.map_or_else(
                        || fallback.clone(),
                        |g| {
                            g.texture.create_view(&wgpu::TextureViewDescriptor {
                                dimension: Some(wgpu::TextureViewDimension::D2Array),
                                ..Default::default()
                            })
                        },
                    )
                };
                material::array_uniform(
                    &mut uniform,
                    m,
                    normal_array.is_some_and(|g| g.info.format.two_channel()),
                );
                Some((
                    a,
                    view(map_array, &a.white),
                    view(normal_array, &a.flat_normal),
                ))
            }
            _ => None,
        };
        let factors = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("material"),
                contents: bytemuck::bytes_of(&uniform),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let entries = [
            factors.as_entire_binding(),
            wgpu::BindingResource::TextureView(&base),
            wgpu::BindingResource::TextureView(&metal),
            wgpu::BindingResource::TextureView(&rough),
            wgpu::BindingResource::TextureView(&normal_view),
            wgpu::BindingResource::TextureView(&emissive),
            wgpu::BindingResource::TextureView(&ao),
            wgpu::BindingResource::Sampler(&self.material_sampler),
            wgpu::BindingResource::TextureView(&gradient),
            wgpu::BindingResource::TextureView(&dissolve),
        ];
        let group = bind_group(&self.device, "material", &self.material_layout, &entries);
        let array = arrays.map(|(a, base, normal)| {
            let mut entries = entries.clone();
            entries[1] = wgpu::BindingResource::TextureView(&base);
            entries[4] = wgpu::BindingResource::TextureView(&normal);
            bind_group(&self.device, "material (array maps)", &a.layout, &entries)
        });
        Ok((group, array, factors))
    }

    /// A physical material's group for its lit pass (`physical.rs`): its
    /// standard uniform `factors`, the base colour, normal, emissive and
    /// gradient textures, its pack (made here unless a material with the
    /// same maps made it) and its physical uniform.
    fn physical_bind_group(
        &mut self,
        index: usize,
        m: &StandardMaterial,
        factors: &wgpu::Buffer,
    ) -> Result<wgpu::BindGroup, RenderError> {
        if self.physical.is_none() {
            self.physical = Some(physical::PhysicalPipelines::new(
                &self.device,
                &self.frame_layout,
                &self.palette_layout,
                physical_source_with(
                    &hooks::default_hooks(),
                    self.palette.mode(),
                    self.light_tables,
                ),
            ));
        }
        let textures = &self.cache.textures;
        let gpu = |h: TextureHandle| {
            textures.get(h.index()).ok_or_else(|| {
                RenderError::InvalidHandle(format!(
                    "material {index} uses texture {} (slot {})",
                    h.index(),
                    h.slot()
                ))
            })
        };
        let (key, layers_of, layers) = physical::pack_plan(m, |h| {
            textures
                .get(h.index())
                .is_some_and(|g| g.format.two_channel())
        });
        let physical = self.physical.as_ref().expect("made above");
        let pack = match self.cache.packs.get(&key) {
            Some(view) => view.clone(),
            None if layers.is_empty() => physical.empty_pack.clone(),
            None => {
                let mut size = (1, 1);
                let mut sources = Vec::with_capacity(layers.len());
                for layer in &layers {
                    let mut views: [wgpu::TextureView; 4] =
                        std::array::from_fn(|_| self.default_white.clone());
                    for (k, &h) in layer.sources.iter().enumerate() {
                        let view = &gpu(h)?.view;
                        let t = view.texture();
                        size = (size.0.max(t.width()), size.1.max(t.height()));
                        views[k] = view.clone();
                    }
                    sources.push((layer.uniform, views));
                }
                let mut encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("material pack"),
                        });
                let view = physical.pack(
                    &self.device,
                    &mut encoder,
                    &sources,
                    size,
                    &self.material_sampler,
                );
                self.queue.submit(Some(encoder.finish()));
                self.cache.packs.insert(key, view.clone());
                view
            }
        };
        let view =
            |t: u32, fallback: &wgpu::TextureView| -> Result<wgpu::TextureView, RenderError> {
                match TextureHandle::from_slot(t) {
                    None => Ok(fallback.clone()),
                    Some(h) => Ok(gpu(h)?.view.clone()),
                }
            };
        let base = view(m.map, &self.default_white)?;
        let normal = view(m.normalMap, &self.default_normal)?;
        let emissive = view(m.emissiveMap, &self.default_white)?;
        let gradient = view(m.gradientMap, &self.default_white)?;
        let uniform = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("physical material"),
                contents: bytemuck::bytes_of(&physical::uniform(m, layers_of)),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        Ok(bind_group(
            &self.device,
            "physical material",
            &physical.material_layout,
            &[
                factors.as_entire_binding(),
                wgpu::BindingResource::TextureView(&base),
                wgpu::BindingResource::TextureView(&normal),
                wgpu::BindingResource::TextureView(&emissive),
                wgpu::BindingResource::Sampler(&self.material_sampler),
                wgpu::BindingResource::TextureView(&gradient),
                wgpu::BindingResource::TextureView(&pack),
                uniform.as_entire_binding(),
            ],
        ))
    }

    /// Grows the local shadow array to the tier's capacity when the scene's
    /// shadowed lights could need more layers than it has (once per
    /// renderer: the frame bind groups are made again with it).
    fn grow_local_shadows(&mut self, scene: &RenderScene) {
        let capacity =
            shadows::capacity(self.info.shadowed_lights as usize, self.info.point_shadows);
        if self.local_map.layers.len() >= capacity {
            return;
        }
        // At most every shadowed light of the scene's, and the tier's capacity.
        let needed = scene
            .lights
            .iter()
            .filter(|l| l.cast_shadows)
            .map(|l| match l.kind {
                LightKind::Point => self.info.point_shadows.layers(),
                LightKind::Spot { .. } => 1,
            })
            .sum::<usize>()
            .min(capacity);
        if needed <= self.local_map.layers.len() {
            return;
        }
        self.local_map = ShadowMap::new(
            &self.device,
            "local shadows",
            LOCAL_SHADOW_SIZE,
            capacity as u32,
        );
        self.environment = None;
    }

    /// (Re)generates the image-based lighting when the sky, or the
    /// environment map that replaces it, changed.
    fn ensure_environment(&mut self, scene: &RenderScene) {
        let sky = &scene.environment.sky;
        let map = scene.environment_map.filter(|m| {
            self.cache
                .environment_maps
                .get(m.map.index())
                .is_some_and(Option::is_some)
        });
        let key = match map {
            Some(m) => EnvironmentKey::Map {
                scene: scene.id(),
                map: m.map.index(),
                revision: self.cache.environment_map_revisions[m.map.index()],
                rotation: m.rotation.to_bits(),
                background: m.background_intensity.to_bits(),
            },
            None => EnvironmentKey::Sky(*sky),
        };
        if self.environment.as_ref().is_some_and(|(k, ..)| *k == key) {
            return;
        }
        match map {
            Some(m) => self.ibl.generate_map(
                &self.device,
                &self.queue,
                &self.cache.environment_maps[m.map.index()]
                    .as_ref()
                    .expect("filtered above")
                    .view,
                &scene.environment_maps()[m.map.index()].irradiance,
                m.rotation,
                m.background_intensity.max(0.0),
            ),
            None => self.ibl.generate(
                &self.device,
                &self.queue,
                sky.sun_direction,
                sky.sun_color,
                sky.look.as_ref(),
            ),
        }
        let ibl = self.ibl.views();
        let entries = [
            self.frame_buffer.as_entire_binding(),
            wgpu::BindingResource::TextureView(&self.cascade_map.array_view),
            wgpu::BindingResource::TextureView(&self.local_map.array_view),
            wgpu::BindingResource::TextureView(&ibl.irradiance),
            wgpu::BindingResource::TextureView(&ibl.prefiltered),
            wgpu::BindingResource::TextureView(&ibl.brdf_lut),
            wgpu::BindingResource::Sampler(&ibl.sampler),
            self.light_buffers[0].as_entire_binding(),
            self.light_buffers[1].as_entire_binding(),
            self.light_buffers[2].as_entire_binding(),
        ];
        let group = bind_group(&self.device, "frame", &self.frame_layout, &entries);
        let mut reflected = entries.clone();
        reflected[0] = self.reflection_buffer.as_entire_binding();
        reflected[8] = self.reflection_light_buffers[1].as_entire_binding();
        reflected[9] = self.reflection_light_buffers[2].as_entire_binding();
        let reflection_group = bind_group(
            &self.device,
            "frame (planar reflection)",
            &self.frame_layout,
            &reflected,
        );
        let sky_group = bind_group(
            &self.device,
            "sky",
            &self.sky_layout,
            &[wgpu::BindingResource::TextureView(&ibl.sky)],
        );
        self.environment = Some((key, group, sky_group, reflection_group));
    }

    /// Sorts the instances into (mesh, material) batches
    /// ([`instances::SortedInstances`]: from the last frame's sort while the
    /// submission's keys hold); returns the batches and the sorted
    /// instances (transforms, world bounding spheres, submission indices),
    /// lent to the frame until [`Renderer::restore_sorted`].
    fn build_batches(
        &mut self,
        scene: &RenderScene,
    ) -> Result<(Vec<Batch>, instances::Sorted), RenderError> {
        let (meshes, materials) = (&self.cache.meshes, &self.cache.materials);
        let sorted = self.sorted.update(
            scene,
            meshes.len(),
            |m| (meshes[m].bounds, meshes[m].index_count == 0),
            materials.len(),
            |mesh, material| Self::dequant(meshes, materials, mesh, material),
            |material| materials[material].array.is_some(),
            |material| materials[material].wireframe.is_some(),
        )?;
        let batches = sorted
            .runs
            .iter()
            .map(|r| {
                let m = &self.cache.materials[r.material];
                Batch {
                    mesh: r.mesh,
                    material: r.material,
                    first: r.first,
                    count: r.count,
                    side: m.side,
                    transparent: m.transparent,
                    blend: m.blend,
                    physical: m.physical.is_some(),
                    transmissive: m.transmissive,
                    dissolve: m.dissolve,
                    effects: m.effects,
                    shader: m.shader,
                    lean: m.lean,
                    derivatives: m.flat,
                    compact: r.dequant.is_some(),
                    array: r.array,
                    cast: r.cast,
                    layers: r.layers,
                }
            })
            .collect();
        Ok((batches, sorted))
    }

    /// How a (mesh, material) run undoes its vertices' quantisation
    /// ([`instances::Run::dequant`]): a plain material drawing a compact
    /// mesh draws its compact vertices; anything else the f32 ones.
    fn dequant(
        meshes: &[GpuMesh],
        materials: &[GpuMaterial],
        mesh: usize,
        material: usize,
    ) -> Option<Mat4> {
        materials[material]
            .lean
            .then(|| meshes[mesh].compact.as_ref().map(|(_, dq)| dq.matrix()))
            .flatten()
    }

    /// The draw ranges of one pass: per batch (`include`d), every instance
    /// whose sphere touches `frustum`. A batch that loses less than a tenth
    /// of its instances is drawn whole (cheaper than copying its
    /// transforms); otherwise its visible instances are appended to
    /// `transforms`. `inside` is the caller's scratch, reused across the
    /// frame's passes.
    #[allow(clippy::too_many_arguments)]
    fn cull(
        &self,
        batches: &[Batch],
        spheres: &[(Vec3, f32)],
        transforms: &mut Vec<instances::GpuTransform>,
        origins: &mut Vec<u32>,
        inside: &mut Vec<bool>,
        frustum: &Frustum,
        include: impl Fn(&Batch) -> bool,
    ) -> (DrawList, u32) {
        let mut list = DrawList::default();
        let mut culled = 0;
        let mut visible = Vec::new();
        // Only the included batches' spheres are tested: they are marked
        // first, and a lane leaves an unmarked one alone (no read of it
        // follows). The tests are lanes (og-lanes) once there are thousands.
        inside.clear();
        inside.resize(spheres.len(), false);
        for b in batches.iter().filter(|b| include(b)) {
            inside[b.first as usize..(b.first + b.count) as usize].fill(true);
        }
        og_lanes::for_each_indexed(og_lanes::Site::Cull, inside, CULL_GRAIN, |k, v| {
            if *v {
                let (c, r) = spheres[k];
                *v = frustum.intersects_sphere(c, r);
            }
        });
        for (b_index, b) in batches.iter().enumerate() {
            if !include(b) {
                continue;
            }
            let range = b.first as usize..(b.first + b.count) as usize;
            visible.clear();
            visible.extend(range.clone().filter(|&k| inside[k]));
            culled += b.count - visible.len() as u32;
            let index_count = self.cache.meshes[b.mesh].index_count;
            list.in_view_triangles += (index_count / 3) as u64 * visible.len() as u64;
            let (first, count) = if visible.is_empty() {
                continue;
            } else if visible.len() as u32 * 10 >= b.count * 9 {
                (b.first, b.count)
            } else {
                let first = transforms.len() as u32;
                for &k in &visible {
                    transforms.push(transforms[k]);
                    origins.push(origins[k]);
                }
                (first, visible.len() as u32)
            };
            list.ranges.push((b_index as u32, first, count));
            list.instances += count;
            list.triangles += (index_count / 3) as u64 * count as u64;
        }
        (list, culled)
    }

    /// The transparent instances in view, sorted back to front by the view
    /// depth of their bounds' centre (ties keep submission order), as runs of
    /// one (mesh, material) appended to `transforms`. Returns the list and
    /// how many were outside the view.
    #[allow(clippy::too_many_arguments)]
    fn sort_transparent(
        &self,
        batches: &[Batch],
        spheres: &[(Vec3, f32)],
        submission: &[u32],
        transforms: &mut Vec<instances::GpuTransform>,
        origins: &mut Vec<u32>,
        frustum: &Frustum,
        eye: Vec3,
        forward: Vec3,
        layers: u32,
    ) -> (DrawList, u32) {
        let mut keyed: Vec<(f32, u32, usize, u32)> = Vec::new();
        let mut culled = 0;
        let drawn = |b: &Batch| b.transparent && b.layers & layers != 0;
        for (b_index, b) in batches.iter().enumerate().filter(|(_, b)| drawn(b)) {
            for k in b.first as usize..(b.first + b.count) as usize {
                let (c, r) = spheres[k];
                if frustum.intersects_sphere(c, r) {
                    keyed.push(((c - eye).dot(forward), submission[k], k, b_index as u32));
                } else {
                    culled += 1;
                }
            }
        }
        keyed.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut list = DrawList::default();
        for (_, _, k, b) in keyed {
            let at = transforms.len() as u32;
            transforms.push(transforms[k]);
            origins.push(origins[k]);
            match list.ranges.last_mut() {
                Some(r) if r.0 == b && r.1 + r.2 == at => r.2 += 1,
                _ => list.ranges.push((b, at, 1)),
            }
            list.instances += 1;
            list.triangles += (self.cache.meshes[batches[b as usize].mesh].index_count / 3) as u64;
        }
        (list, culled)
    }

    /// World bounds of every instance as (center, radius), from their box
    /// ([`instances::fold_box`]; `None`: no instance).
    fn instance_bounds(bounds: Option<(Vec3, Vec3)>) -> (Vec3, f32) {
        let Some((lo, hi)) = bounds else {
            return (Vec3::ZERO, 1.0);
        };
        ((lo + hi) * 0.5, ((hi - lo).length() * 0.5).max(1e-3))
    }

    fn sprite_group(&mut self, texture: Option<TextureHandle>) -> &wgpu::BindGroup {
        let Some(t) = texture else {
            return &self.white_sprite_group;
        };
        let nearest = self.cache.filters.get(t.index()) == Some(&TextureFilter::Nearest);
        let slot = &mut self.cache.sprite_groups[t.index()];
        if slot.is_none() {
            *slot = Some(sprite_bind_group(
                &self.device,
                &self.sprite_layout,
                &self.cache.textures[t.index()].view,
                if nearest {
                    &self.sprite_sampler_nearest
                } else {
                    &self.sprite_sampler
                },
                [0.0; 4],
            ));
        }
        slot.as_ref().expect("made above")
    }

    /// The lit pipeline of a batch (made by [`Renderer::ensure_pipelines`]):
    /// its material shader's or the built-in one, physical or standard.
    fn lit_pipeline_of(&self, b: &Batch, skinned: bool, physical: bool) -> &wgpu::RenderPipeline {
        if b.lean && !skinned {
            return &self.lean_pipelines[&(b.side, b.effects, b.derivatives, b.compact)];
        }
        let key = (b.side, b.blend, skinned, b.effects);
        if b.array && !physical {
            return &self.array_maps.as_ref().expect("an array batch").lit[&key];
        }
        match (physical, b.shader) {
            (true, None) => &self.physical.as_ref().expect("a physical batch").pipelines[&key],
            (true, Some(sh)) => &self.cache.physical_hook_pipelines[&(sh, key)],
            (false, Some(sh)) => {
                &self.cache.hook_pipelines[&(sh, b.side, b.blend, b.effects, skinned)]
            }
            (false, None) => &self.pbr_pipelines[&key],
        }
    }

    /// Makes the lit, shadow and prepass pipelines this frame's batches need.
    fn ensure_pipelines(&mut self, batches: &[Batch], skinned: &[Batch], prepass: bool) {
        let all = batches
            .iter()
            .map(|b| (b, false))
            .chain(skinned.iter().map(|b| (b, true)));
        for (b, is_skinned) in all {
            if b.array {
                // The array-map variant: the standard builders over its
                // module and layouts (`material::ArrayMaps`).
                let a = self.array_maps.as_mut().expect("made with the material");
                let layouts = (&a.layouts.0, &a.layouts.1);
                let key = (b.side, b.blend, is_skinned, b.effects);
                if !a.lit.contains_key(&key) {
                    let p = pbr_pipeline(&self.device, layouts, &a.module, key, self.samples);
                    a.lit.insert(key, p);
                }
                let key = (b.side, is_skinned);
                if prepass && !b.transparent && !a.prepass.contains_key(&key) {
                    let p = prepass_pipeline(
                        &self.device,
                        layouts,
                        &a.module,
                        b.side,
                        is_skinned,
                        None,
                    );
                    a.prepass.insert(key, p);
                }
                let key = (b.side, b.dissolve);
                if !is_skinned && !a.shadow.contains_key(&key) {
                    let p = shadow_pipeline(
                        &self.device,
                        &self.shadow_layouts,
                        &a.shadow_module,
                        (b.side, false, b.dissolve),
                        false,
                        false,
                    );
                    a.shadow.insert(key, p);
                }
            }
            if b.physical {
                // The physical lit pipeline (the standard ones below draw the
                // material's reflection).
                let key = (b.side, b.blend, is_skinned, b.effects);
                let physical = self.physical.as_mut().expect("made with the material");
                let layouts = (&physical.layouts.0, &physical.layouts.1);
                match b.shader {
                    None if !physical.pipelines.contains_key(&key) => {
                        let p = lit_pipeline(
                            &self.device,
                            layouts,
                            &physical.module,
                            key,
                            self.samples,
                            true,
                        );
                        physical.pipelines.insert(key, p);
                    }
                    Some(sh) if !self.cache.physical_hook_pipelines.contains_key(&(sh, key)) => {
                        let p = lit_pipeline(
                            &self.device,
                            layouts,
                            &self.cache.physical_shaders[&sh],
                            key,
                            self.samples,
                            true,
                        );
                        self.cache.physical_hook_pipelines.insert((sh, key), p);
                    }
                    _ => {}
                }
            }
            if let Some(sh) = b.shader {
                let key = (sh, b.side, b.blend, b.effects, is_skinned);
                if !self.cache.hook_pipelines.contains_key(&key) {
                    let p = pbr_pipeline(
                        &self.device,
                        (&self.pbr_layout, &self.pbr_skinned_layout),
                        &self.cache.shaders[sh],
                        (b.side, b.blend, is_skinned, b.effects),
                        self.samples,
                    );
                    self.cache.hook_pipelines.insert(key, p);
                }
                // Shadows and the prepass see the material shader's offset.
                let key = (sh, b.side, is_skinned);
                if !self.cache.hook_shadow_pipelines.contains_key(&key) {
                    let p = shadow_pipeline(
                        &self.device,
                        &self.shadow_layouts,
                        &self.cache.shadow_shaders[sh],
                        (b.side, is_skinned, b.dissolve),
                        true,
                        false,
                    );
                    self.cache.hook_shadow_pipelines.insert(key, p);
                }
                if prepass
                    && !b.transparent
                    && !self.cache.hook_prepass_pipelines.contains_key(&key)
                {
                    let p = prepass_pipeline(
                        &self.device,
                        (&self.pbr_layout, &self.pbr_skinned_layout),
                        &self.cache.shaders[sh],
                        b.side,
                        is_skinned,
                        None,
                    );
                    self.cache.hook_prepass_pipelines.insert(key, p);
                }
            }
            if b.lean && !is_skinned {
                // A plain material: the lean lit and prepass pipelines (its
                // shadows take the shadow pipeline below, lean as well).
                let key = (b.side, b.effects, b.derivatives, b.compact);
                if !self.lean_pipelines.contains_key(&key) {
                    let p = lean_pipeline(
                        &self.device,
                        &self.pbr_layout,
                        &self.pbr_shader,
                        key,
                        self.samples,
                    );
                    self.lean_pipelines.insert(key, p);
                }
                let key = (b.side, b.derivatives, b.compact);
                if prepass && !self.lean_prepass_pipelines.contains_key(&key) {
                    let p = prepass_pipeline(
                        &self.device,
                        (&self.pbr_layout, &self.pbr_skinned_layout),
                        &self.pbr_shader,
                        b.side,
                        false,
                        Some((b.derivatives, b.compact)),
                    );
                    self.lean_prepass_pipelines.insert(key, p);
                }
                if b.compact && !self.compact_shadow_pipelines.contains_key(&b.side) {
                    let p = shadow_pipeline(
                        &self.device,
                        &self.shadow_layouts,
                        &self.shadow_shader,
                        (b.side, false, false),
                        false,
                        true,
                    );
                    self.compact_shadow_pipelines.insert(b.side, p);
                }
            }
            let key = (b.side, b.blend, is_skinned, b.effects);
            if !(b.lean && !is_skinned) && !self.pbr_pipelines.contains_key(&key) {
                let p = pbr_pipeline(
                    &self.device,
                    (&self.pbr_layout, &self.pbr_skinned_layout),
                    &self.pbr_shader,
                    key,
                    self.samples,
                );
                self.pbr_pipelines.insert(key, p);
            }
            let key = (b.side, is_skinned, b.dissolve);
            if !self.shadow_pipelines.contains_key(&key) {
                let p = shadow_pipeline(
                    &self.device,
                    &self.shadow_layouts,
                    &self.shadow_shader,
                    key,
                    false,
                    false,
                );
                self.shadow_pipelines.insert(key, p);
            }
            if prepass
                && !b.transparent
                && !(b.lean && !is_skinned)
                && !self.prepass_pipelines.contains_key(&(b.side, is_skinned))
            {
                let p = prepass_pipeline(
                    &self.device,
                    (&self.pbr_layout, &self.pbr_skinned_layout),
                    &self.pbr_shader,
                    b.side,
                    is_skinned,
                    None,
                );
                self.prepass_pipelines.insert((b.side, is_skinned), p);
            }
        }
    }

    /// The skinned instances as (mesh, material) batches over the skinned
    /// instance buffer, their instance data, and the frame's joint matrices.
    #[allow(clippy::type_complexity)]
    fn build_skinned(&self, scene: &RenderScene) -> Result<SkinnedFrame, RenderError> {
        let mut offsets = Vec::with_capacity(scene.joint_palettes.len());
        let mut matrices = Vec::new();
        for p in &scene.joint_palettes {
            offsets.push((matrices.len() as u32, p.matrices.len() as u32));
            matrices.extend(p.matrices.iter().map(|m| m.to_cols_array_2d()));
        }
        if matrices.len() > MAX_JOINTS {
            return Err(RenderError::InvalidData(format!(
                "{} joints in the frame's palettes (at most {MAX_JOINTS})",
                matrices.len()
            )));
        }
        for (i, s) in scene.skinned_instances.iter().enumerate() {
            let mesh = s.mesh.index();
            if mesh >= self.cache.meshes.len() || self.cache.skins[mesh].is_none() {
                return Err(RenderError::InvalidHandle(format!(
                    "skinned instance {i}: mesh {mesh} has neither a skin nor morph targets"
                )));
            }
            if s.material.index() >= self.cache.materials.len() {
                return Err(RenderError::InvalidHandle(format!(
                    "skinned instance {i}: material {}",
                    s.material.index()
                )));
            }
            if s.palette as usize >= offsets.len() {
                return Err(RenderError::InvalidHandle(format!(
                    "skinned instance {i}: palette {} of {}",
                    s.palette,
                    offsets.len()
                )));
            }
        }
        if !scene.skinned_instances.is_empty()
            && let Some(rows) = self
                .palette
                .texture_rows(self.cache.morph_data.len(), matrices.len())
        {
            let max = self.device.limits().max_texture_dimension_2d;
            if rows > max as u64 {
                return Err(RenderError::Unsupported(format!(
                    "{} joints and {} morph deltas take {rows} palette texture rows, this device allows {max}",
                    matrices.len(),
                    self.cache.morph_data.len()
                )));
            }
        }
        let joint_base = self.palette.joint_base(self.cache.morph_data.len());
        let mut order: Vec<usize> = (0..scene.skinned_instances.len()).collect();
        // A (mesh, material)'s casters first, its non-casters a batch of
        // their own (left out of the shadow passes).
        order.sort_by_key(|&i| {
            let s = &scene.skinned_instances[i];
            (
                s.mesh,
                s.material,
                !s.cast_shadows,
                views::layer_of(&scene.skinned_layers, i),
            )
        });
        let mut batches: Vec<Batch> = Vec::new();
        let mut data = Vec::with_capacity(order.len());
        let mut spheres = Vec::with_capacity(order.len());
        for &i in &order {
            let s = &scene.skinned_instances[i];
            let (mesh, material, cast) = (s.mesh.index(), s.material.index(), s.cast_shadows);
            let layers = views::layer_of(&scene.skinned_layers, i);
            if self.cache.meshes[mesh].index_count == 0 {
                continue;
            }
            match batches.last_mut() {
                Some(b)
                    if b.mesh == mesh
                        && b.material == material
                        && b.cast == cast
                        && b.layers == layers =>
                {
                    b.count += 1
                }
                _ => {
                    let m = &self.cache.materials[material];
                    batches.push(Batch {
                        mesh,
                        material,
                        first: data.len() as u32,
                        count: 1,
                        side: m.side,
                        transparent: m.transparent,
                        blend: m.blend,
                        physical: m.physical.is_some(),
                        transmissive: m.transmissive,
                        dissolve: m.dissolve,
                        effects: m.effects,
                        shader: m.shader,
                        lean: false,
                        derivatives: m.flat,
                        compact: false,
                        array: m.array.is_some(),
                        cast,
                        layers,
                    });
                }
            }
            let m = s.transform;
            let scale = m
                .x_axis
                .truncate()
                .length()
                .max(m.y_axis.truncate().length())
                .max(m.z_axis.truncate().length());
            spheres.push(match self.cache.meshes[mesh].bounds {
                Some((c, e)) => (
                    m.transform_point3(c),
                    e.length() * scale * s.bounds_scale.max(1.0),
                ),
                None => (m.w_axis.truncate(), 0.0),
            });
            let (first, count) = offsets[s.palette as usize];
            let first = first + joint_base;
            let (morph_first, targets) = self.cache.morph_ranges[mesh];
            let w = s.morph_weights;
            data.push(skin::SkinnedInstanceGpu {
                transform: instances::gpu_columns(&s.transform),
                palette: [
                    first as f32,
                    count as f32,
                    morph_first as f32,
                    targets as f32,
                ],
                morph0: [w[0], w[1], w[2], w[3]],
                morph1: [w[4], w[5], w[6], w[7]],
                look: {
                    let [flash, color, tint] = instances::fx_bits(&s.fx);
                    [
                        instances::layer_bits(0.0, s.receive_shadows),
                        flash,
                        color,
                        tint,
                    ]
                },
            });
        }
        Ok(SkinnedFrame {
            batches,
            data,
            spheres,
            matrices,
        })
    }

    /// The skinned batches the camera sees: instances whose widened bounds
    /// touch `frustum`, appended to `frame.data` as new runs.
    /// Only the batches on a layer of `layers` (`views.rs`).
    fn cull_skinned(frame: &mut SkinnedFrame, frustum: &Frustum, layers: u32) -> (Vec<Batch>, u32) {
        let mut visible = Vec::new();
        let mut culled = 0;
        for b in frame.batches.iter().filter(|b| b.layers & layers != 0) {
            let first = frame.data.len() as u32;
            for k in b.first as usize..(b.first + b.count) as usize {
                let (c, r) = frame.spheres[k];
                if frustum.intersects_sphere(c, r) {
                    let d = frame.data[k];
                    frame.data.push(d);
                } else {
                    culled += 1;
                }
            }
            let count = frame.data.len() as u32 - first;
            if count > 0 {
                visible.push(Batch { first, count, ..*b });
            }
        }
        (visible, culled)
    }

    fn ribbon_group(&mut self, texture: Option<TextureHandle>) {
        let Some(t) = texture else {
            return;
        };
        if self.cache.ribbon_groups[t.index()].is_none() {
            self.cache.ribbon_groups[t.index()] = Some(sprite_bind_group(
                &self.device,
                &self.sprite_layout,
                &self.cache.textures[t.index()].view,
                &self.ribbon_sampler,
                [0.0; 4],
            ));
        }
    }

    /// Draws one frame of `scene` into `target`.
    pub fn render(
        &mut self,
        scene: &RenderScene,
        target: &RenderTarget,
    ) -> Result<FrameStats, RenderError> {
        if let Some(stale) = scene.stale_handle() {
            return Err(RenderError::InvalidHandle(stale));
        }
        self.sync(scene)?;
        // A look that switches the stylisation on or off (passes added or
        // removed, the pixel grid, the material style) is a cut: no ghost of
        // the old look in the first frame of the new one.
        let chain = scene.stylise.chain_bits();
        if self.stylise_chain.is_some_and(|c| c != chain) {
            self.taa_history = None;
        }
        self.stylise_chain = Some(chain);
        let (grid, upscale, grid_offset) = self.pixel_grid(scene.stylise.pixel);
        self.scene_size(grid.0, grid.1);
        self.grow_local_shadows(scene);
        self.ensure_environment(scene);
        let (mut batches, sorted_instances) = self.build_batches(scene)?;
        let instances::Sorted {
            mut transforms,
            spheres,
            submission,
            sphere_box,
            ..
        } = sorted_instances;
        let mut origins = submission.clone();
        // The f32 vertices of compact meshes drawn without their compact
        // ones: by a material that is not plain, skinned or scattered.
        let full: Vec<usize> = batches
            .iter()
            .filter(|b| !b.compact)
            .map(|b| b.mesh)
            .chain(scene.skinned_instances.iter().map(|s| s.mesh.index()))
            .chain(scene.scatters.iter().map(|s| s.mesh.index()))
            .collect();
        for mesh in full {
            self.ensure_full(scene, mesh);
        }
        let mut skinned = self.build_skinned(scene)?;

        // Camera (with the shake), jitter, culling.
        let aspect = self.width as f32 / self.height as f32;
        let taa = self.info.antialias == Antialias::Taa;
        let jitter = if taa {
            let px = frame::halton_jitter(self.frame_index);
            Vec2::new(
                px.x * 2.0 / self.width as f32,
                -px.y * 2.0 / self.height as f32,
            )
        } else {
            Vec2::ZERO
        };
        let camera = shaken_camera(&scene.camera, scene.shake);
        let cam = frame::camera_frame(&camera, aspect, jitter);
        let view_frustum = Frustum::from_view_proj(cam.view_proj);
        // Water: the surface the camera is under (its fog replaces the
        // frame's), and the one drawn with a planar reflection (the camera
        // mirrored in it, clipped at the surface by an oblique near plane).
        let under = water::underwater(&scene.water, camera.position);
        let frame_fog = match under {
            Some(i) => Some(water::underwater_fog(&scene.water[i])),
            None => scene.fog,
        };
        let planar = match (under, cam.orthographic) {
            (None, false) => scene.water.iter().position(|w| {
                let (n, at) = water::plane(w);
                w.planar_reflection > 0.0 && (camera.position - at).dot(n) > 0.0
            }),
            _ => None,
        };
        let reflection = planar.map(|i| {
            let w = &scene.water[i];
            let (n, at) = water::plane(w);
            let mirrored = water::mirrored_camera(&camera, n, at);
            let f = frame::camera_frame(&mirrored, aspect, Vec2::ZERO);
            let proj = f.view_proj * f.view.inverse();
            let view_proj = water::oblique(proj, f.view, n, at, 0.02) * f.view;
            let scale = w.planar_reflection.clamp(0.1, 1.0);
            let size = (
                ((self.width as f32 * scale) as u32).max(1),
                ((self.height as f32 * scale) as u32).max(1),
            );
            (i, mirrored, f, view_proj, size)
        });
        // Secondary cameras (views.rs): the views due this frame, each
        // with its skinned instances in view on its layers.
        let view_plans = views::plan(
            scene,
            &camera,
            (self.out_width, self.out_height),
            self.info.tier == Tier::Full,
            |i| self.views.stale || self.views.slots.get(i).is_none_or(|s| s.is_none()),
        );
        self.views.stale = false;
        self.views.retain(scene.views.len());
        let mut view_skinned: Vec<Vec<Batch>> = view_plans
            .iter()
            .map(|p| {
                let f = Frustum::from_view_proj(p.frame.view_proj);
                Self::cull_skinned(&mut skinned, &f, p.layers).0
            })
            .collect();
        let (mut skinned_visible, skinned_culled) =
            Self::cull_skinned(&mut skinned, &view_frustum, scene.camera_layers);
        let mut skinned_batches = std::mem::take(&mut skinned.batches);
        let skinned_data = std::mem::take(&mut skinned.data);
        let joint_matrices = std::mem::take(&mut skinned.matrices);

        // Effects this frame.
        let post_fx = &scene.post;
        let particle_frame = self.particles.prepare(
            &self.device,
            &self.queue,
            scene,
            self.info.tier == Tier::Full && self.desc_particles == ParticleSimulation::Auto,
            self.cache.textures.len(),
            camera.position,
            cam.forward,
        )?;
        let ao = post_fx.ao.filter(|a| a.intensity > 0.0 && a.radius > 0.0);
        let outline = post_fx.outline.is_some()
            || batches
                .iter()
                .chain(&skinned_batches)
                .any(|b| !b.transparent && self.cache.materials[b.material].outline);
        let dof = post_fx.depth_of_field;
        let motion_blur = post_fx.motion_blur;
        let god_rays = post_fx.god_rays;
        let fluids_3d = scene.fluids_3d.iter().any(|f| !f.particles.is_empty());
        let depth_bands = scene.stylise.depth_bands.filter(|b| b.count >= 2);
        // Ink or hatching on depth edges reads the prepass in the lines
        // pass, a canvas, hatching or halftone on surfaces in the paint or
        // lines pass.
        let ink_depth = scene.stylise.depth_edges();
        // Custom post passes (post_custom.rs) that compiled; they read the
        // prepass's depth and normals.
        let custom_posts = self.custom_posts.prepare(&self.device, scene);
        let prepass = !custom_posts.is_empty()
            || ao.is_some()
            || depth_bands.is_some()
            || ink_depth
            || scene.stylise.on_surfaces()
            || outline
            || dof.is_some()
            || motion_blur.is_some()
            || god_rays.is_some()
            || particle_frame.needs_depth
            // Water and 3D fluids read the depth below them.
            || !scene.water.is_empty()
            || fluids_3d;
        // Decals and screen-space AO need the effect shader on every batch;
        // decals take screen derivatives (the lean pipelines keep the discard).
        if ao.is_some() || !scene.decals.is_empty() {
            let decals = !scene.decals.is_empty();
            for b in batches.iter_mut().chain(skinned_batches.iter_mut()) {
                b.effects = true;
                b.derivatives |= decals;
            }
            for b in skinned_visible
                .iter_mut()
                .chain(view_skinned.iter_mut().flatten())
            {
                b.effects = true;
                b.derivatives |= decals;
            }
        }
        self.ensure_pipelines(&batches, &skinned_batches, prepass);
        // A view may draw a skinned batch the main camera culled or hides.
        for v in &view_skinned {
            self.ensure_pipelines(&[], v, false);
        }
        // A pattern in object space: the object pass after the prepass.
        let object_space = prepass && scene.stylise.in_object_space();
        if object_space {
            let object = self.stylise_object.get_or_insert_with(|| {
                stylise_object::ObjectPass::new(
                    &self.device,
                    self.palette.mode(),
                    self.light_tables,
                )
            });
            let opaque = batches.iter().filter(|b| !b.transparent);
            for (side, skinned) in opaque
                .map(|b| (b.side, false))
                .chain(skinned_visible.iter().map(|b| (b.side, true)))
            {
                object.ensure(
                    &self.device,
                    (&self.pbr_layout, &self.pbr_skinned_layout),
                    side,
                    skinned,
                );
            }
        }
        self.particles
            .ensure_pipelines(&self.device, &particle_frame, self.samples);
        let fonts = &self.cache.fonts;
        let arrays = &self.cache.texture_arrays;
        let array_layers = |a: TextureArrayHandle| {
            scene
                .contains_texture_array(a)
                .then(|| arrays.get(a.index()).and_then(Option::as_ref))
                .flatten()
                .map(|g| g.info.layers)
        };
        let overlay_frame = match &scene.overlay {
            Some(o) => Some(overlay::build(
                o,
                |i| fonts.get(i).map(|f| &f.layout),
                self.cache.textures.len(),
                array_layers,
            )?),
            None => None,
        };
        if let Some(of) = &overlay_frame {
            let textures: Vec<_> = overlay::run_textures(of).collect();
            for t in textures {
                self.sprite_group(t);
            }
            let arrays: Vec<_> = overlay::run_arrays(of).collect();
            for a in &arrays {
                self.sprite_array_group(scene, *a);
            }
            if !arrays.is_empty() {
                self.sprite_arrays
                    .as_mut()
                    .expect("made with the groups")
                    .make_overlay(
                        &self.device,
                        &self.overlay.layout,
                        self.overlay.output_format,
                        self.overlay.linear_format,
                    );
            }
        }
        // Glass: the tonemap renders into the glass source, which becomes
        // the output and the blur pyramid after the post passes.
        let glass_on = scene.overlay.as_ref().is_some_and(|o| !o.glass.is_empty());
        if glass_on
            && self
                .glass_targets
                .as_ref()
                .is_none_or(|g| g.size != (self.out_width, self.out_height))
        {
            self.glass_targets = Some(self.glass.targets(
                &self.device,
                self.output_format,
                self.offscreen_linear,
                self.out_width,
                self.out_height,
                &self.sprite_layout,
            ));
        }
        // Lines, points and wireframes (lines.rs): made and uploaded now,
        // drawn in the main pass and over the finished frame.
        let any_wire = self.cache.materials.iter().any(|m| m.wireframe.is_some());
        let materials = &self.cache.materials;
        let line_frame = self
            .lines
            .prepare(&self.device, &self.queue, scene, any_wire, |m| {
                materials.get(m).and_then(|g| g.wireframe)
            });
        self.lines.set_scene_uniform(
            &self.queue,
            &lines::LineUniform::new(
                cam.jittered_view_proj,
                cam.proj,
                self.width,
                self.height,
                scene.css_height,
                false,
            ),
        );
        let mut ribbon_frame = self
            .trails
            .build(
                scene.id(),
                scene.trail_styles(),
                &scene.trails,
                &scene.trail_layers,
                &scene.ribbons,
                &scene.beams,
                scene.tick,
                scene.tick_rate,
                camera.position,
                cam.forward,
                cam.orthographic,
            )
            .map_err(RenderError::InvalidHandle)?;
        for run in &ribbon_frame.runs {
            if let Some(t) = run.texture
                && t.index() >= self.cache.textures.len()
            {
                return Err(RenderError::InvalidHandle(format!(
                    "strip texture {}",
                    t.index()
                )));
            }
            self.ribbon_group(run.texture);
        }
        // Texture filters: a sprite group made with the other sampler is remade.
        for i in 0..self.cache.textures.len() {
            let f = scene.texture_filter(TextureHandle::from_index(i));
            if self.cache.filters.len() <= i {
                self.cache.filters.resize(i + 1, TextureFilter::Linear);
            }
            if self.cache.filters[i] != f {
                self.cache.filters[i] = f;
                self.cache.sprite_groups[i] = None;
            }
        }
        let lit_range = scene.lighting_2d.map(|l| (l.min_layer, l.max_layer));
        let view = match camera.projection {
            Projection::Orthographic { height } => {
                let half = Vec2::new(height * 0.5 * aspect, height * 0.5);
                let c = camera.position.truncate();
                Some(sprites::ViewRect {
                    min: c - half,
                    max: c + half,
                })
            }
            Projection::Perspective { .. } => None,
        };
        water::water_2d_strips(&scene.water_2d, &mut ribbon_frame);
        let mut layered: Vec<sprites::LayeredStrip> = ribbon_frame
            .runs
            .iter()
            .filter_map(|r| {
                Some(sprites::LayeredStrip {
                    layer: r.layer?,
                    source: RunSource::Strip(r.blend as u8, r.texture),
                    first: r.first,
                    count: r.count,
                })
            })
            .collect();
        let fluids = self.metaballs.prepare(
            &self.device,
            &self.queue,
            &scene.metaballs,
            (self.width, self.height),
        );
        layered.extend(
            metaballs::layers(&scene.metaballs)
                .into_iter()
                .map(|(i, layer)| sprites::LayeredStrip {
                    layer,
                    source: RunSource::Metaballs(i as u8),
                    first: 0,
                    count: 1,
                }),
        );
        let fonts = &self.cache.fonts;
        let arrays = &self.cache.texture_arrays;
        let sprite_frame = sprites::build(
            scene,
            |i| fonts.get(i).map(|f| &f.layout),
            self.cache.textures.len(),
            |a| {
                scene
                    .contains_texture_array(a)
                    .then(|| arrays.get(a.index()).and_then(Option::as_ref))
                    .flatten()
                    .map(|g| g.info.layers)
            },
            &particle_frame.sprites,
            |layer| lit_range.is_some_and(|(lo, hi)| (lo..=hi).contains(&layer)),
            view,
            &layered,
        )?;
        for run in sprite_frame.runs.iter().chain(&sprite_frame.glow_runs) {
            match run.source {
                RunSource::Sprite(_, t) => {
                    self.sprite_group(t);
                }
                RunSource::Array(_, a) => self.sprite_array_group(scene, a),
                RunSource::Mesh(i) => {
                    self.sprite_group(scene.sprite_meshes[i as usize].texture);
                }
                _ => {}
            }
        }
        // The sprite meshes' corners (sprite_mesh.rs), lit as their layers' sprites.
        if !scene.sprite_meshes.is_empty() {
            let corners = sprite_mesh::vertices(scene, |layer| {
                lit_range.is_some_and(|(lo, hi)| (lo..=hi).contains(&layer))
            });
            let meshes = self.sprite_meshes.get_or_insert_with(|| {
                sprite_mesh::SpriteMeshRenderer::new(
                    &self.device,
                    [
                        &self.frame_layout,
                        &self.sprite_layout,
                        &self.sprite_fx_layout,
                    ],
                    self.samples,
                )
            });
            grow_and_write(
                &self.device,
                &self.queue,
                &mut meshes.buffer,
                "sprite meshes",
                bytemuck::cast_slice(&corners),
            );
        }
        for s in &particle_frame.systems {
            self.sprite_group(s.texture);
        }
        if let Some(t) = scene.decal_atlas
            && t.index() >= self.cache.textures.len()
        {
            return Err(RenderError::InvalidHandle(format!(
                "decal atlas {}",
                t.index()
            )));
        }
        let lut_index = post_fx.color_grading.map(|g| g.lut.index());
        if let Some(g) = post_fx.color_grading
            && !(scene.contains_lut(g.lut)
                && self
                    .cache
                    .luts
                    .get(g.lut.index())
                    .is_some_and(Option::is_some))
        {
            return Err(RenderError::InvalidHandle(format!(
                "colour grading lut {}",
                g.lut.index()
            )));
        }
        let glow = post_fx.glow.is_some() && !sprite_frame.glow_runs.is_empty();

        // GPU-driven culling (cull_gpu.rs): each pass's list is indirect
        // draws the compute fills; the CPU sorts the transparent instances
        // and culls the skinned ones as before.
        let object_motion = motion_blur.is_some()
            && !scene.previous_transforms.is_empty()
            && scene.previous_transforms.len() == scene.instances.len();
        let sorted = transforms.len() as u32;
        // The passes this frame can cull at most: the camera, the
        // occluders, the planar reflection, the cascades and the shadowed
        // local lights' faces. The buffers must fit them; `encode` sizes
        // them for the frame's actual passes and regrows when that grows.
        let max_passes = 2
            + reflection.is_some() as usize
            + if scene.light.cast_shadows {
                (self.info.shadow_cascades as usize).clamp(1, CASCADES)
            } else {
                0
            }
            + scene
                .lights
                .iter()
                .filter(|l| l.cast_shadows)
                .map(|l| match l.kind {
                    LightKind::Point => self.info.point_shadows.layers(),
                    LightKind::Spot { .. } => 1,
                })
                .sum::<usize>()
                .min(shadows::capacity(
                    self.info.shadowed_lights as usize,
                    self.info.point_shadows,
                ));
        let gpu_cull = self.indirect
            && match self.desc_culling {
                Culling::Auto => sorted >= GPU_CULLING_MIN_INSTANCES,
                Culling::Gpu => true,
                Culling::Cpu => false,
            }
            && !object_motion
            // A view's CPU-culled lists index the sorted instances.
            && view_plans.is_empty()
            && !batches.is_empty()
            && {
                let limits = self.device.limits();
                let gc = self.gpu_cull.get_or_insert_with(|| {
                    cull_gpu::GpuCulling::new(
                        &self.device,
                        limits
                            .max_storage_buffer_binding_size
                            .min(limits.max_buffer_size),
                    )
                });
                gc.fits(sorted, max_passes)
            };
        let mut cull_passes: Vec<cull_gpu::CullPass> = Vec::new();
        let mut gpu_pass = |kind, frustum: &Frustum| {
            cull_passes.push(cull_gpu::CullPass {
                kind,
                planes: frustum.planes().to_vec(),
            });
            cull_passes.len() as u32 - 1
        };
        // Lent to every CPU cull of this frame, given back after the last.
        let mut cull_inside = std::mem::take(&mut self.cull_inside);
        // Occlusion needs a batch that can occlude and an earlier frame's
        // camera visibility (its visible occluders); without them the
        // occluders' depth, the pyramid and the occlusion test are skipped.
        let occlusion = gpu_cull
            && self.gpu_cull.as_ref().is_some_and(|gc| gc.primed())
            && batches
                .iter()
                .any(|b| self.cache.materials[b.material].occluder);
        // The main camera's layers (views.rs).
        let mask = scene.camera_layers;
        let (main_list, mut culled) = if gpu_cull {
            let g = gpu_pass(cull_gpu::PassKind::Main, &view_frustum);
            if occlusion {
                gpu_pass(cull_gpu::PassKind::Occluders, &view_frustum);
            }
            (
                indirect_list(&batches, sorted, g, |b| {
                    !b.transparent && b.layers & mask != 0
                }),
                0,
            )
        } else {
            self.cull(
                &batches,
                &spheres,
                &mut transforms,
                &mut origins,
                &mut cull_inside,
                &view_frustum,
                |b| !b.transparent && b.layers & mask != 0,
            )
        };
        let (mut blended_list, blended_culled) = self.sort_transparent(
            &batches,
            &spheres,
            &submission,
            &mut transforms,
            &mut origins,
            &view_frustum,
            camera.position,
            cam.forward,
            mask,
        );
        culled += blended_culled + skinned_culled;
        let reflection_list = reflection.as_ref().map(|r| {
            let frustum = Frustum::from_view_proj(r.3);
            if gpu_cull {
                let g = gpu_pass(cull_gpu::PassKind::Reflection, &frustum);
                indirect_list(&batches, sorted, g, |b| {
                    !b.transparent && b.layers & mask != 0
                })
            } else {
                self.cull(
                    &batches,
                    &spheres,
                    &mut transforms,
                    &mut origins,
                    &mut cull_inside,
                    &frustum,
                    |b| !b.transparent && b.layers & mask != 0,
                )
                .0
            }
        });

        // Each view's opaque meshes on its layers (CPU culled: a frame with
        // views culls on the CPU).
        let view_lists: Vec<DrawList> = view_plans
            .iter()
            .map(|p| {
                self.cull(
                    &batches,
                    &spheres,
                    &mut transforms,
                    &mut origins,
                    &mut cull_inside,
                    &Frustum::from_view_proj(p.frame.view_proj),
                    |b| !b.transparent && b.layers & p.layers != 0,
                )
                .0
            })
            .collect();

        // Terrains: new or edited data uploaded, this frame's clipmap levels.
        let prev_view_proj = self.prev_view_proj.unwrap_or(cam.view_proj);
        let textures = &self.cache.textures;
        let arrays = &self.cache.texture_arrays;
        let terrain_draws = self.terrain.prepare(
            &self.device,
            &self.queue,
            scene,
            camera.position,
            camera.far,
            self.height,
            cam.view_proj,
            prev_view_proj,
            |h| textures.get(h.index()).map(|t| &t.view),
            |h| {
                arrays
                    .get(h.index())
                    .and_then(Option::as_ref)
                    .filter(|a| a.generation == h.generation())
                    .map(|a| &a.texture)
            },
        )?;
        let terrain_casters = terrain_draws.iter().any(|d| d.cast_shadows);
        // Scattered copies: the chunks this frame needs built, what to draw.
        let scatter_batches = {
            let materials = &self.cache.materials;
            let meshes = &self.cache.meshes;
            self.scatter.prepare(
                &self.device,
                &self.queue,
                scene,
                camera.position,
                cam.view_proj,
                prev_view_proj,
                |m| materials.get(m).map(|g| g.side),
                |m| {
                    meshes
                        .get(m)
                        .filter(|g| g.index_count > 0)
                        .map(|g| g.bounds.map_or(1.0, |(c, e)| c.length() + e.length()))
                },
            )
        };
        let scatter_material = |m: usize| self.cache.materials.get(m).map(|g| &g.bind_group);
        let scatter_mesh = |m: usize| {
            self.cache
                .meshes
                .get(m)
                .map(|g| (&g.vertices, &g.indices, g.index_count))
        };

        // The directional light's cascades.
        let light = &scene.light;
        let to_light = light.direction_to_light.normalize_or(Vec3::Y);
        let shadow_size = self.info.shadow_map_size;
        let has_casters = batches.iter().chain(&skinned_batches).any(|b| b.cast) || terrain_casters;
        let cascades: Vec<Cascade> = if light.cast_shadows && has_casters {
            let bounds = match light.shadow_bounds {
                ShadowBounds::Sphere { center, radius } => (center, radius.max(1e-3)),
                ShadowBounds::Auto => {
                    // The instances' box (kept while they stand still), then
                    // the skinned instances and the terrains.
                    let skinned = scene.skinned_instances.iter().map(|s| {
                        let r = self.cache.meshes[s.mesh.index()]
                            .bounds
                            .map_or(1.0, |(_, e)| e.length() * 2.0);
                        (s.transform.w_axis.truncate(), r)
                    });
                    let all = instances::fold_box(sphere_box, skinned);
                    Self::instance_bounds(instances::fold_box(all, self.terrain.bounds(scene)))
                }
            };
            frame::fit_cascades(
                &camera,
                &cam,
                aspect,
                to_light,
                bounds,
                light.max_shadow_distance,
                shadow_size,
                self.info.shadow_cascades as usize,
                light.cascade_blend,
            )
        } else {
            Vec::new()
        };
        let cascade_lists: Vec<DrawList> = cascades
            .iter()
            .map(|c| {
                if gpu_cull {
                    let g = gpu_pass(cull_gpu::PassKind::Shadow, &c.frustum);
                    indirect_list(&batches, sorted, g, |b| b.cast)
                } else {
                    self.cull(
                        &batches,
                        &spheres,
                        &mut transforms,
                        &mut origins,
                        &mut cull_inside,
                        &c.frustum,
                        |b| b.cast,
                    )
                    .0
                }
            })
            .collect();

        // Local lights: the clusters they reach, their records, their shadow faces.
        self.clusters.build(
            &scene.lights,
            &clusters::ClusterView {
                camera: &cam,
                frustum: &view_frustum,
                position: camera.position,
                near: camera.near,
                far: camera.far,
                width: self.width,
                height: self.height,
            },
            self.light_tables.ref_capacity(),
        );
        let (lights_gpu, local_shadows, shadowed_dropped) = frame::local_lights(
            &scene.lights,
            &self.clusters.kept,
            camera.position,
            frame::LocalShadowBudget {
                max_shadowed: self.info.shadowed_lights as usize,
                point: self.info.point_shadows,
                shadow_size: LOCAL_SHADOW_SIZE,
            },
        );
        debug_assert!(
            local_shadows
                .last()
                .is_none_or(|ls| ls.layer + ls.faces.len() <= self.local_map.layers.len())
        );
        if !lights_gpu.is_empty() {
            self.queue
                .write_buffer(&self.light_buffers[0], 0, bytemuck::cast_slice(&lights_gpu));
            self.queue.write_buffer(
                &self.light_buffers[1],
                0,
                bytemuck::cast_slice(&self.clusters.table),
            );
            self.queue.write_buffer(
                &self.light_buffers[2],
                0,
                bytemuck::cast_slice(&self.clusters.ref_words()),
            );
        }
        // The planar reflection's clusters: the same light records seen by
        // the mirrored camera (its oblique projection moves only depth).
        if let Some((_, mirrored, f, _, (rw, rh))) = &reflection {
            self.reflection_clusters.build_kept(
                &scene.lights,
                &self.clusters.kept,
                &clusters::ClusterView {
                    camera: f,
                    frustum: &Frustum::from_view_proj(f.view_proj),
                    position: mirrored.position,
                    near: mirrored.near,
                    far: mirrored.far,
                    width: *rw,
                    height: *rh,
                },
                self.light_tables.ref_capacity(),
            );
            if !lights_gpu.is_empty() {
                self.queue.write_buffer(
                    &self.reflection_light_buffers[1],
                    0,
                    bytemuck::cast_slice(&self.reflection_clusters.table),
                );
                self.queue.write_buffer(
                    &self.reflection_light_buffers[2],
                    0,
                    bytemuck::cast_slice(&self.reflection_clusters.ref_words()),
                );
            }
        }
        let mut local_lists: Vec<Option<DrawList>> =
            (0..LOCAL_SHADOW_LAYERS).map(|_| None).collect();
        let mut local_frustums: Vec<Option<Frustum>> =
            (0..LOCAL_SHADOW_LAYERS).map(|_| None).collect();
        for ls in &local_shadows {
            for (face, (_, f)) in ls.faces.iter().enumerate() {
                local_frustums[ls.layer + face] = Some(f.clone());
                let list = if gpu_cull {
                    let g = gpu_pass(cull_gpu::PassKind::Shadow, f);
                    indirect_list(&batches, sorted, g, |b| b.cast)
                } else {
                    self.cull(
                        &batches,
                        &spheres,
                        &mut transforms,
                        &mut origins,
                        &mut cull_inside,
                        f,
                        |b| b.cast,
                    )
                    .0
                };
                local_lists[ls.layer + face] = Some(list);
            }
        }
        self.cull_inside = cull_inside;

        // Upload instances, previous transforms, skinned data, strips and
        // sprites. With GPU culling the sorted instances go to its buffer and
        // this one holds only the transparent draws.
        let mut gpu_counts = None;
        let copy_counts = std::mem::take(&mut self.culling_counts);
        let cull_commands = if gpu_cull {
            let gc = self
                .gpu_cull
                .as_mut()
                .expect("made when gpu_cull was decided");
            if gc.in_flight() {
                let _ = self.device.poll(wgpu::PollType::Poll);
            }
            let lagged = gc.poll_counts();
            debug_assert!(cull_passes.len() <= max_passes);
            let (commands, occluder_draws) = gc.encode(
                &self.device,
                &self.queue,
                &transforms[..sorted as usize],
                &batches,
                &self.cache.meshes,
                &self.cache.materials,
                &cull_passes,
                cam.jittered_view_proj,
                (self.width, self.height, self.samples),
                self.frame_index,
                lagged.unwrap_or_default(),
                copy_counts,
            );
            gpu_counts = Some((lagged, occluder_draws));
            for r in &mut blended_list.ranges {
                r.1 -= sorted;
            }
            Some(commands)
        } else {
            None
        };
        let cpu_instances = if gpu_cull {
            &transforms[sorted as usize..]
        } else {
            &transforms[..]
        };
        // Only the blocks that changed since the last frame are written
        // (a buffer just replaced is written whole).
        let instance_bytes: &[u8] = bytemuck::cast_slice(cpu_instances);
        if grow(
            &self.device,
            &mut self.instance_buffer,
            "instances",
            instance_bytes.len() as u64,
        ) {
            self.instance_upload.invalidate();
        }
        let mut instance_upload_bytes =
            self.instance_upload
                .write(&self.queue, &self.instance_buffer, instance_bytes);
        if gpu_cull && let Some(gc) = &self.gpu_cull {
            instance_upload_bytes += gc.uploaded;
        }
        if object_motion {
            // As the instances' own: with a compact run's dequantisation.
            let (meshes, materials) = (&self.cache.meshes, &self.cache.materials);
            let prev: Vec<instances::GpuTransform> = origins
                .iter()
                .map(|&i| {
                    let (inst, p) = (
                        &scene.instances[i as usize],
                        scene.previous_transforms[i as usize],
                    );
                    let d =
                        Self::dequant(meshes, materials, inst.mesh.index(), inst.material.index());
                    instances::gpu_transform(&d.map_or(p, |d| p * d))
                })
                .collect();
            grow_and_write(
                &self.device,
                &self.queue,
                &mut self.prev_instance_buffer,
                "previous instances",
                bytemuck::cast_slice(&prev),
            );
        }
        self.sorted.restore(transforms, spheres);
        if !skinned_data.is_empty() {
            grow_and_write(
                &self.device,
                &self.queue,
                &mut self.skinned_buffer,
                "skinned instances",
                bytemuck::cast_slice(&skinned_data),
            );
            let morphs_changed = std::mem::take(&mut self.cache.morphs_dirty);
            self.palette.write(
                &self.device,
                &self.queue,
                &self.palette_layout,
                &joint_matrices,
                &self.cache.morph_data,
                morphs_changed,
            );
        }
        grow_and_write(
            &self.device,
            &self.queue,
            &mut self.ribbon_buffer,
            "strips",
            bytemuck::cast_slice(&ribbon_frame.vertices),
        );
        grow_and_write(
            &self.device,
            &self.queue,
            &mut self.sprite_buffer,
            "sprites",
            bytemuck::cast_slice(&sprite_frame.instances),
        );
        if let (Some(of), Some(o)) = (&overlay_frame, &scene.overlay) {
            grow_and_write(
                &self.device,
                &self.queue,
                &mut self.overlay.buffer,
                "overlay instances",
                bytemuck::cast_slice(&of.instances),
            );
            let u = overlay::OverlayRenderer::uniform(
                self.out_width,
                self.out_height,
                o,
                !self.output_format.is_srgb() && !self.has_linear_view(target),
            );
            self.queue
                .write_buffer(&self.overlay.uniform, 0, bytemuck::cast_slice(&u));
        }
        let (lights_2d, occluders_2d) = match scene.lighting_2d {
            Some(_) => self.lights_2d.write(
                &self.queue,
                cam.view_proj,
                &scene.lights_2d,
                &scene.occluders_2d,
            ),
            None => (0, 0),
        };

        // Uniforms.
        let mut uniform = frame::FrameUniform {
            view_proj: cam.jittered_view_proj.to_cols_array_2d(),
            inv_view_proj: cam.jittered_view_proj.inverse().to_cols_array_2d(),
            view_pos: camera
                .position
                .extend(if cam.orthographic { 1.0 } else { 0.0 })
                .to_array(),
            light_dir: to_light.extend(cascades.len() as f32).to_array(),
            light_color: (light.color * light.intensity)
                .extend(match scene.debug {
                    DebugView::None => 0.0,
                    DebugView::Cascades => 1.0,
                    DebugView::AmbientOcclusion => 2.0,
                })
                .to_array(),
            params: [
                scene.environment.intensity,
                (ibl::PREFILTER_MIPS - 1) as f32,
                lights_gpu.len() as f32,
                1.0 / shadow_size as f32,
            ],
            view_forward: cam
                .forward
                .extend(1.0 / LOCAL_SHADOW_SIZE as f32)
                .to_array(),
            cascade_view_proj: [[[0.0; 4]; 4]; CASCADES],
            cascade_bias: [[0.0; 4]; CASCADES],
            cluster: self.clusters.params,
            local_view_proj: [[[0.0; 4]; 4]; LOCAL_SHADOW_LAYERS],
            atmosphere: atmosphere::AtmosphereGpu::new(
                frame_fog.as_ref(),
                match scene.environment.background {
                    Background::Procedural {
                        sun_direction,
                        sun_color,
                    } => Some((
                        scene
                            .environment
                            .sky
                            .look
                            .as_ref()
                            .unwrap_or(&SkyLook::DEFAULT),
                        sun_direction,
                        sun_color,
                    )),
                    _ => None,
                },
                scene.tick,
                scene.tick_rate,
            ),
            caustics: water::caustics(&scene.water),
            shadow: [
                if cascades.is_empty() {
                    0.0
                } else {
                    light.shadow_softness.max(0.0)
                },
                (self.info.point_shadows == PointShadows::Paraboloid) as u32 as f32,
                0.0,
                0.0,
            ],
            stylise: match scene.stylise.retro {
                Some(r) if r.vertex_snap > 0.0 || r.affine > 0.0 || r.depth_bits > 0 => [
                    if r.vertex_snap > 0.0 {
                        self.width as f32 / r.vertex_snap
                    } else {
                        0.0
                    },
                    if r.vertex_snap > 0.0 {
                        self.height as f32 / r.vertex_snap
                    } else {
                        0.0
                    },
                    r.affine.clamp(0.0, 1.0),
                    r.depth_bits.min(24) as f32,
                ],
                _ => [0.0; 4],
            },
            shadow_tint: {
                let m = &scene.stylise.material;
                let a = m.shadow_tint_amount.clamp(0.0, 1.0);
                [m.shadow_tint[0], m.shadow_tint[1], m.shadow_tint[2], a]
            },
            outline_style: [
                scene.stylise.material.outline_distance.max(0.0),
                0.0,
                0.0,
                0.0,
            ],
        };
        let mut shadow_slots = vec![bytemuck::Zeroable::zeroed(); SHADOW_SLOTS];
        for (i, c) in cascades.iter().enumerate() {
            uniform.cascade_view_proj[i] = c.view_proj.to_cols_array_2d();
            uniform.cascade_bias[i] = [c.normal_offset, c.depth_bias, c.split_far, c.blend_start];
            shadow_slots[i] = ShadowPassUniform {
                view_proj: c.view_proj.to_cols_array_2d(),
                light: [0.0; 4],
                time: uniform.atmosphere.time,
                paraboloid: [0.0; 4],
                pad: [0.0; 36],
            };
        }
        for ls in &local_shadows {
            for (face, (vp, _)) in ls.faces.iter().enumerate() {
                let layer = ls.layer + face;
                uniform.local_view_proj[layer] = vp.to_cols_array_2d();
                shadow_slots[CASCADES + layer] = ShadowPassUniform {
                    view_proj: vp.to_cols_array_2d(),
                    light: ls.position.extend(ls.range).to_array(),
                    time: uniform.atmosphere.time,
                    // Hemisphere 0 below the light, 1 above (frame.rs).
                    paraboloid: if ls.paraboloid {
                        [
                            1.0,
                            if face == 0 { 1.0 } else { -1.0 },
                            LOCAL_SHADOW_SIZE as f32,
                            shadows::PARABOLOID_SCALE,
                        ]
                    } else {
                        [0.0; 4]
                    },
                    pad: [0.0; 36],
                };
            }
        }
        self.queue
            .write_buffer(&self.frame_buffer, 0, bytemuck::bytes_of(&uniform));
        if let Some((_, mirrored, f, view_proj, _)) = &reflection {
            let mut u = uniform;
            u.view_proj = view_proj.to_cols_array_2d();
            // The sky's view rays: the mirrored camera's own projection
            // (the oblique one tilts its far plane through the sky).
            u.inv_view_proj = f.view_proj.inverse().to_cols_array_2d();
            u.view_pos = mirrored.position.extend(0.0).to_array();
            u.view_forward = f.forward.extend(u.view_forward[3]).to_array();
            u.caustics = [[0.0; 4]; 2];
            u.cluster = self.reflection_clusters.params;
            self.queue
                .write_buffer(&self.reflection_buffer, 0, bytemuck::bytes_of(&u));
        }
        self.queue
            .write_buffer(&self.shadow_buffer, 0, bytemuck::cast_slice(&shadow_slots));

        // The sun on screen, for god rays.
        let sun_clip = cam.view_proj * (camera.position + to_light * camera.far * 0.5).extend(1.0);
        let sun_ndc = sun_clip.truncate().truncate() / sun_clip.w.max(1e-6);
        let sun_front = sun_clip.w > 0.0 && to_light.dot(cam.forward) > 0.0;
        let sun_fade = (2.0 - sun_ndc.abs().max_element()).clamp(0.0, 1.0);
        let god_visible = god_rays.is_some() && sun_front && sun_fade > 0.0;
        self.post.write_uniform(
            &self.queue,
            camera.exposure,
            post_fx,
            glow,
            god_rays.is_some(),
            sprite_frame.untonemapped,
            scene.tick,
        );
        if camera.exposure.adaptation().is_some() {
            self.post.advance_exposure(
                &self.queue,
                camera.exposure,
                scene.tick,
                scene.tick_rate,
                (self.frame_index % 2) as usize,
            );
        } else {
            self.post.forget_exposure();
        }
        let mut fx_uniform =
            fx::fx_uniform(cam.view_proj, prev_view_proj, (self.width, self.height));
        let decals = fx::pack_decals(&scene.decals, &mut fx_uniform.decals);
        fx_uniform.decal_info[0] = decals as f32;
        fx_uniform.decal_info[1] = scene.decal_atlas.is_some() as u32 as f32;
        if ao.is_some() {
            fx_uniform.ao = [1.0, 1.0, 0.0, 0.0];
        }
        self.queue
            .write_buffer(&self.fx_buffer, 0, bytemuck::bytes_of(&fx_uniform));
        if let Some((_, _, _, view_proj, (rw, rh))) = &reflection {
            let mut u = fx_uniform;
            u.ao = [0.0; 4];
            u.view_proj_unjittered = view_proj.to_cols_array_2d();
            u.prev_view_proj = view_proj.to_cols_array_2d();
            u.screen = [*rw as f32, *rh as f32, 1.0 / *rw as f32, 1.0 / *rh as f32];
            self.queue
                .write_buffer(&self.reflection_fx_buffer, 0, bytemuck::bytes_of(&u));
        }
        let viewport_format = self.viewport_format(target);
        self.views.write(
            &self.device,
            &self.queue,
            (self.light_tables, self.samples),
            &self.clusters.kept,
            scene,
            &view_plans,
            &uniform,
            &fx_uniform,
            !lights_gpu.is_empty(),
            viewport_format,
        );
        let proj = cam.view_proj * cam.view.inverse();
        let global_outline = post_fx.outline;
        let fxp = post::FxPostUniform {
            proj: proj.to_cols_array_2d(),
            inv_proj: proj.inverse().to_cols_array_2d(),
            view: cam.view.to_cols_array_2d(),
            screen: [
                self.width as f32,
                self.height as f32,
                1.0 / self.width as f32,
                1.0 / self.height as f32,
            ],
            cam: [camera.far, cam.orthographic as u32 as f32, camera.near, 0.0],
            ao: ao.map_or([0.0; 4], |a| [a.radius, a.intensity, 0.0, 0.0]),
            outline_color: global_outline.map_or([0.0; 4], |o| o.color),
            outline: match global_outline {
                Some(o) => [o.width.max(0.0), o.depth_threshold, o.normal_threshold, 1.0],
                None => [0.0, 0.04, 0.4, 0.0],
            },
            dof: dof.map_or([0.0; 4], |d| {
                [
                    d.focus_distance,
                    d.focus_range.max(1e-3),
                    (d.max_blur * self.height as f32).max(0.0),
                    d.falloff.max(1e-3),
                ]
            }),
            dof2: [
                match dof.map(|d| d.quality) {
                    Some(DofQuality::High) => 48.0,
                    _ => 16.0,
                },
                0.0,
                0.0,
                0.0,
            ],
            mb: motion_blur.map_or([0.0; 4], |m| {
                [
                    m.shutter.max(0.0),
                    m.samples.clamp(4, 32) as f32,
                    m.max_pixels.max(1.0),
                    0.0,
                ]
            }),
            god: [
                sun_ndc.x * 0.5 + 0.5,
                0.5 - sun_ndc.y * 0.5,
                god_rays.map_or(0.96, |g| g.decay),
                god_rays.map_or(0.8, |g| g.density),
            ],
            god2: [
                god_rays.map_or(48.0, |g| g.samples.clamp(8, 128) as f32),
                god_visible as u32 as f32,
                god_rays.map_or(0.0, |g| g.intensity * sun_fade),
                god_rays.map_or(2.0, |g| g.threshold.max(0.0)),
            ],
            god_tint: god_rays
                .map_or(Vec3::ONE, |g| g.tint)
                .extend(1.0)
                .to_array(),
            bands: depth_bands.map_or([0.0; 4], |b| {
                [
                    b.count.clamp(2, 8) as f32,
                    b.near.max(0.0),
                    b.far.max(b.near + 0.01),
                    b.flatten.clamp(0.0, 1.0),
                ]
            }),
            bands_near: depth_bands.map_or([0.0; 4], |b| {
                [b.near_color[0], b.near_color[1], b.near_color[2], 1.0]
            }),
            bands_far: depth_bands.map_or([0.0; 4], |b| {
                [b.far_color[0], b.far_color[1], b.far_color[2], 1.0]
            }),
        };
        self.queue
            .write_buffer(&self.post.fx_buffer, 0, bytemuck::bytes_of(&fxp));
        let view_t = cam.view.transpose();
        let pview = particles_gpu::ParticleViewGpu {
            view_proj: cam.view_proj.to_cols_array_2d(),
            cam_pos: camera
                .position
                .extend(cam.orthographic as u32 as f32)
                .to_array(),
            cam_forward: cam.forward.extend(0.0).to_array(),
            cam_right: view_t.x_axis.truncate().extend(0.0).to_array(),
            cam_up: view_t.y_axis.truncate().extend(0.0).to_array(),
            screen: fxp.screen,
        };
        self.queue
            .write_buffer(&self.particles.view_buffer, 0, bytemuck::bytes_of(&pview));
        self.queue
            .write_buffer(&self.sprite_fx_buffer, 0, bytemuck::bytes_of(&fxp.screen));
        let slot = (self.frame_index % 2) as usize;
        let taa_blend = taa && self.taa_history.is_some();
        if taa {
            let prev = self.taa_history.unwrap_or(cam.view_proj);
            let taa_uniform = post::TaaUniform {
                inv_view_proj: cam.jittered_view_proj.inverse().to_cols_array_2d(),
                cur_view_proj: cam.view_proj.to_cols_array_2d(),
                prev_view_proj: prev.to_cols_array_2d(),
                params: [
                    post::taa_weight(self.taa_history.is_some()),
                    self.width as f32,
                    self.height as f32,
                    self.post.taa_sharpen,
                ],
                still: [
                    still_camera(&prev, &cam.view_proj) as u32 as f32,
                    0.0,
                    0.0,
                    0.0,
                ],
            };
            self.queue
                .write_buffer(&self.post.taa_buffer, 0, bytemuck::bytes_of(&taa_uniform));
        }

        // The frame's passes, compiled into the render graph (cached while
        // the pass list stays the same).
        let auto_exposure = matches!(camera.exposure, Exposure::Auto { .. });
        let bloom = post_fx.bloom.is_some_and(|b| b.intensity > 0.0) || glow;
        let gtao = match ao.map(|a| a.method) {
            Some(AoMethod::Gtao) => true,
            Some(AoMethod::Ssao) => false,
            _ => self.info.tier == Tier::Full,
        };
        let mut passes: Vec<FramePass> =
            (0..cascades.len() as u32).map(FramePass::Cascade).collect();
        for ls in &local_shadows {
            for face in 0..ls.faces.len() {
                passes.push(FramePass::LocalFace((ls.layer + face) as u32));
            }
        }
        if prepass {
            passes.push(FramePass::Prepass);
        }
        if ao.is_some() {
            passes.extend([FramePass::Ao, FramePass::AoBlur]);
        }
        if particle_frame.any_gpu {
            passes.push(FramePass::Particles);
        }
        if scene.lighting_2d.is_some() {
            passes.push(FramePass::Light2d);
        }
        if fluids > 0 {
            passes.push(FramePass::Metaballs);
        }
        if fluids_3d {
            passes.push(FramePass::Fluid3d);
        }
        if reflection.is_some() {
            passes.push(FramePass::Reflection);
        }
        passes.extend((0..view_plans.len() as u32).map(FramePass::View));
        passes.push(FramePass::Main);
        if depth_bands.is_some() {
            passes.push(FramePass::DepthBands);
        }
        if glow {
            passes.push(FramePass::Glow);
        }
        if outline {
            passes.push(FramePass::Outline);
        }
        if taa {
            passes.push(FramePass::Taa);
            if self.post.taa_sharpen > 0.0 {
                passes.push(FramePass::TaaSharpen);
            }
        }
        if dof.is_some() {
            passes.extend([
                FramePass::DofCoc,
                FramePass::DofBlur,
                FramePass::DofComposite,
            ]);
        }
        if motion_blur.is_some() {
            passes.push(FramePass::MotionBlur);
        }
        passes.extend((0..custom_posts.len() as u32).map(FramePass::CustomPost));
        if god_rays.is_some() {
            passes.extend([FramePass::GodRaysMask, FramePass::GodRaysBlur]);
        }
        if auto_exposure {
            passes.push(FramePass::Meter);
            passes.extend((0..post::meter_levels() - 1).map(FramePass::Reduce));
            if camera.exposure.adaptation().is_some() {
                passes.push(FramePass::Adapt);
            }
        }
        if bloom {
            let levels = post::bloom_levels(self.width, self.height);
            passes.push(FramePass::BloomPrefilter);
            passes.extend((0..levels - 1).map(FramePass::BloomDown));
            passes.extend((0..levels - 1).rev().map(FramePass::BloomUp));
        }
        passes.push(FramePass::Tonemap);
        // The stylisation chain (post.rs `chain_passes`), then FXAA, then
        // the pixel grid scaled up to the output.
        let st = &scene.stylise;
        if st.paint.is_some() {
            passes.push(FramePass::Paint);
        }
        if st.hatching.is_some() || st.halftone.is_some() || st.ink.is_some() {
            passes.push(FramePass::Lines);
        }
        if st.quantize.is_some() {
            passes.push(FramePass::Quantize);
        }
        if self.info.antialias == Antialias::Fxaa {
            passes.push(FramePass::Fxaa);
        }
        if (self.width, self.height) != (self.out_width, self.out_height) {
            passes.push(FramePass::Upscale);
        }
        self.post.ensure_chain(&self.device, &passes);
        // The stylise passes' parameters, palette and canvas.
        let palette_view = st
            .quantize
            .and_then(|q| q.palette)
            .and_then(|p| self.cache.palettes.get(p.index()))
            .and_then(Option::as_ref)
            .unwrap_or(&self.default_white);
        let canvas_view = st
            .paint
            .and_then(|p| p.canvas)
            .and_then(|t| self.cache.textures.get(t.index()))
            .map_or(&self.default_white, |g| &g.view);
        let stylise_key = {
            // The slot's revision too: a palette added in a freed slot is
            // another texture under the same index.
            let p = st.quantize.and_then(|q| q.palette).map_or(u64::MAX, |p| {
                let revision = self.cache.palette_revisions.get(p.index()).copied();
                p.index() as u64 ^ revision.unwrap_or(0).rotate_left(24)
            });
            let c = st
                .paint
                .and_then(|p| p.canvas)
                .map_or(u64::MAX, |t| t.index() as u64);
            scene.id() ^ p.rotate_left(16) ^ c.rotate_left(40)
        };
        {
            let (w, h) = (self.width as f32, self.height as f32);
            let bg = match scene.environment.background {
                Background::Color(c) => c,
                _ => Vec3::ZERO,
            };
            let q = st.quantize;
            let pattern = match q.map(|q| q.dither) {
                Some(DitherPattern::Bayer2) => 2.0,
                Some(DitherPattern::Bayer4) => 4.0,
                Some(DitherPattern::Bayer8) => 8.0,
                Some(DitherPattern::InterleavedGradient) => -1.0,
                Some(DitherPattern::BlueNoise) => -2.0,
                _ => 0.0,
            };
            let palette_count = q
                .and_then(|q| q.palette)
                .and_then(|p| scene.palettes().get(p.index()))
                .map_or(0.0, |p| p.colors.len() as f32);
            let ht = st.halftone;
            let hc = st.hatching;
            let pt = st.paint;
            let ink = st.ink;
            // Surface space: positions from the surface origin (f64 on the
            // CPU, so a camera far from 0 keeps the patterns' precision),
            // and the canvas uv there.
            let origin = post::surface_origin(camera.position);
            let surface_eye = (camera.position.as_dvec3() - origin).as_vec3();
            let canvas_world_scale = pt.map_or(0.0, |p| {
                if p.canvas_world_scale > 0.0 {
                    p.canvas_world_scale.min(4096.0)
                } else {
                    stylise::DEFAULT_CANVAS_WORLD_SCALE
                }
            });
            let canvas_size = pt
                .and_then(|p| p.canvas)
                .and_then(|t| scene.texture_size(t))
                .map_or(glam::DVec2::ONE, |(w, h)| {
                    glam::DVec2::new(w.max(1) as f64, h.max(1) as f64)
                });
            let canvas_origin = {
                // The uv of the origin along x (u), y (v) and z (u on the
                // x plane, v on the y plane), as stylise.wgsl's
                // `canvas_surface` lays the three planes.
                let k = canvas_world_scale as f64;
                let c = |o: f64, size: f64| ((o * k / size).rem_euclid(1.0)) as f32;
                [
                    c(origin.x, canvas_size.x),
                    c(origin.y, canvas_size.y),
                    c(origin.z, canvas_size.x),
                    c(origin.z, canvas_size.y),
                ]
            };
            let u = post::StyliseUniform {
                screen: [w, h, 1.0 / w, 1.0 / h],
                output: [
                    self.out_width as f32,
                    self.out_height as f32,
                    upscale as f32,
                    0.0,
                ],
                offset: [
                    grid_offset.0 as f32,
                    grid_offset.1 as f32,
                    pt.and_then(|p| p.canvas).is_some() as u32 as f32,
                    palette_count,
                ],
                background: [bg.x, bg.y, bg.z, 1.0],
                quantize: [
                    q.map_or(0.0, |q| q.posterize.min(64) as f32),
                    pattern,
                    q.map_or(0.0, |q| q.dither_strength.clamp(0.0, 2.0)),
                    q.map_or(1.0, |q| q.dither_scale.max(1) as f32),
                ],
                halftone: ht.map_or([0.0; 4], |h| {
                    [
                        h.strength.clamp(0.0, 1.0),
                        h.size.max(2.0),
                        h.angle_degrees.to_radians(),
                        0.0,
                    ]
                }),
                halftone_color: ht.map_or([0.0; 4], |h| [h.color[0], h.color[1], h.color[2], 1.0]),
                hatching: hc.map_or([0.0; 4], |h| {
                    [
                        h.strength.clamp(0.0, 1.0),
                        h.scale.max(2.0),
                        h.levels.clamp(1, 6) as f32,
                        h.angle_degrees.to_radians(),
                    ]
                }),
                hatching_color: hc.map_or([0.0; 4], |h| [h.color[0], h.color[1], h.color[2], 1.0]),
                paint: pt.map_or([0.0; 4], |p| {
                    [
                        p.radius.clamp(0.0, 8.0),
                        p.anisotropy.clamp(0.0, 1.0),
                        p.edge_darkening.clamp(0.0, 1.0),
                        p.bleed.clamp(0.0, 1.0),
                    ]
                }),
                canvas: pt.map_or([0.0; 4], |p| {
                    [
                        p.canvas_strength.clamp(0.0, 1.0),
                        p.canvas_scale.max(0.125),
                        0.0,
                        0.0,
                    ]
                }),
                ink: ink.map_or([0.0; 4], |i| {
                    [
                        1.0,
                        i.threshold.clamp(0.0, 1.0),
                        i.width.max(0.5),
                        i.depth.clamp(0.0, 1.0),
                    ]
                }),
                ink_color: ink.map_or([0.0; 4], |i| [i.color[0], i.color[1], i.color[2], 1.0]),
                ink_depth: {
                    let hatching_depth = hc.map_or(0.0, |h| h.depth.clamp(0.0, 1.0));
                    ink.map_or([camera.far, 0.04, 0.4, hatching_depth], |i| {
                        [
                            camera.far,
                            i.depth_threshold.max(0.0),
                            i.normal_threshold.max(0.0),
                            hatching_depth,
                        ]
                    })
                },
                inv_proj: cam.proj.inverse().to_cols_array_2d(),
                view_rotation: glam::Mat4::from_mat3(glam::Mat3::from_mat4(cam.view.inverse()))
                    .to_cols_array_2d(),
                eye: surface_eye.extend(0.0).to_array(),
                // 0 screen, 1 world, 2 object space (StyliseSpace::code).
                surface: [
                    pt.filter(|p| p.canvas_on_surfaces())
                        .map_or(0.0, |p| p.canvas_space.code()),
                    canvas_world_scale,
                    hc.map_or(0.0, |h| h.space.code()),
                    ht.map_or(0.0, |h| h.space.code()),
                ],
                canvas_origin,
            };
            self.post.write_stylise(&self.queue, &u);
        }
        let lightmap_scale = scene
            .lighting_2d
            .map_or(0.5, |l| l.resolution.clamp(0.25, 1.0));
        let lut_view = match lut_index {
            Some(i) => &self.cache.luts[i].as_ref().expect("checked above").0,
            None => &self.post.identity_lut,
        };
        // Transmissive draws refract a copy of the opaque scene with its
        // mip chain: the main pass splits for it, as for water.
        let transmission = blended_list
            .ranges
            .iter()
            .any(|r| batches[r.0 as usize].transmissive)
            || skinned_visible.iter().any(|b| b.transmissive);
        if transmission {
            let stale = self
                .transmission
                .as_ref()
                .is_none_or(|t| (t.width(), t.height()) != (self.width, self.height));
            if stale {
                self.transmission = Some(self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("transmission"),
                    size: wgpu::Extent3d {
                        width: self.width,
                        height: self.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: mips::mip_count(self.width, self.height),
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: HDR_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                }));
            }
            self.mips.prepare(&self.device, HDR_FORMAT);
        }
        let transmission_view = self
            .transmission
            .as_ref()
            .filter(|_| transmission)
            .map(|t| t.create_view(&Default::default()));
        self.graph
            .prepare(
                &self.device,
                passes,
                &post::GraphConfig {
                    width: self.width,
                    height: self.height,
                    samples: self.samples,
                    taa,
                    lightmap: (
                        ((self.width as f32 * lightmap_scale) as u32).max(1),
                        ((self.height as f32 * lightmap_scale) as u32).max(1),
                    ),
                    gtao,
                    water: !scene.water.is_empty() || fluids_3d,
                    transmission,
                    fluid_fields: fluids as u32,
                    reflection: reflection.as_ref().map(|r| r.4),
                    object_space,
                },
                &self.post,
                &self.persistent,
                (lut_index.map(|i| (scene.id(), i)), lut_view),
                (palette_view, canvas_view, stylise_key),
            )
            .map_err(|e| RenderError::Graph(e.to_string()))?;

        let surface_frame = match target {
            RenderTarget::Surface => {
                let (surface, config) = self.surface.as_ref().ok_or_else(|| {
                    RenderError::Unsupported(
                        "RenderTarget::Surface on a renderer created without one".into(),
                    )
                })?;
                match surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(t)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                        let view = t.texture.create_view(&wgpu::TextureViewDescriptor {
                            format: Some(config.format),
                            ..Default::default()
                        });
                        let linear = config.view_formats.first().map(|&format| {
                            t.texture.create_view(&wgpu::TextureViewDescriptor {
                                format: Some(format),
                                ..Default::default()
                            })
                        });
                        Some((t, view, linear))
                    }
                    other => {
                        surface.configure(&self.device, config);
                        return Err(RenderError::Surface(format!("{other:?}")));
                    }
                }
            }
            _ => None,
        };
        let output: &wgpu::TextureView = match target {
            RenderTarget::Offscreen => &self.persistent.offscreen_view,
            RenderTarget::View(v) => v,
            RenderTarget::Surface => &surface_frame.as_ref().expect("acquired above").1,
        };
        // Where the overlay blends: the output when it is sRGB, else its sRGB
        // view where the backend has one (linear light either way), else the
        // output itself with the shader encoding (a caller's non-sRGB view).
        let overlay_view = match target {
            _ if self.output_format.is_srgb() => None,
            RenderTarget::Offscreen => self.persistent.offscreen_linear.as_ref(),
            RenderTarget::Surface => surface_frame.as_ref().and_then(|f| f.2.as_ref()),
            RenderTarget::View(_) => None,
        };

        let cluster_stats = &self.clusters.stats;
        let mut stats = FrameStats {
            instance_upload_bytes,
            terrain_tiles: self.terrain.resident,
            terrain_uploads: self.terrain.uploads,
            scatter_built: self.scatter.built,
            batches: batches.len() as u32,
            instances: scene.instances.len() as u32,
            culled,
            triangles: main_list.triangles + blended_list.triangles,
            in_view_triangles: main_list.in_view_triangles,
            lights: self.clusters.kept.len() as u32,
            shadowed_lights: local_shadows.len() as u32,
            shadowed_lights_dropped: shadowed_dropped,
            lights_dropped: cluster_stats.dropped,
            lights_culled: cluster_stats.culled,
            cluster_lights: cluster_stats.refs,
            cluster_lights_dropped: cluster_stats.refs_dropped,
            texture_bytes: self.cache.textures.iter().map(|t| t.bytes).sum::<u64>()
                + self.asset_bytes(),
            sprites: sprite_frame.sprites,
            tiles: sprite_frame.tiles,
            tile_chunks: sprite_frame.tile_chunks,
            glyphs: sprite_frame.glyphs,
            sprite_draws: sprite_frame.runs.len() as u32,
            missing_glyphs: sprite_frame.missing_glyphs,
            transparent: blended_list.instances,
            transparent_draws: blended_list.ranges.len() as u32,
            graph_passes: self.graph.order().len() as u32,
            transient_textures: self.graph.transient_textures() as u32,
            frame: self.frame_index,
            particle_systems: scene.particles.len() as u32,
            particle_capacity: particle_frame.capacity,
            particles_cpu: particle_frame.cpu_alive,
            particle_ticks: particle_frame.ticks,
            particle_collision_skipped: particle_frame.collision_skipped,
            strips: ribbon_frame.strips,
            strip_segments: ribbon_frame.segments,
            line_instances: line_frame.instances,
            decals,
            decals_dropped: scene.decals.len() as u32 - decals,
            lights_2d,
            occluders_2d,
            skinned_instances: scene.skinned_instances.len() as u32,
            joints: joint_matrices.len() as u32,
            taa_history: taa_blend,
            ..Default::default()
        };
        if let Some((lagged, occluder_draws)) = gpu_counts {
            let c = lagged.unwrap_or_default();
            stats.gpu_culling = true;
            stats.culled += c.culled;
            stats.occluded = c.occluded;
            stats.occluded_triangles = c.occluded_triangles;
            stats.in_view_triangles = c.triangles + c.occluded_triangles;
            stats.triangles += c.triangles;
            stats.shadow_casters += c.shadow_casters;
            stats.counts_lag = lagged.map_or(u32::MAX, |l| (self.frame_index - l.frame) as u32);
            stats.occluder_draws = occluder_draws;
            stats.draw_calls += occluder_draws;
        }
        if let Some(of) = &overlay_frame {
            stats.overlay_sprites = of.sprites;
            stats.overlay_glyphs = of.glyphs;
            stats.overlay_draws = of.runs.len() as u32;
            stats.sprites += of.sprites;
            stats.glyphs += of.glyphs;
            stats.sprite_draws += of.runs.len() as u32;
            stats.missing_glyphs += of.missing_glyphs;
        }
        for b in &skinned_visible {
            stats.triangles += (self.cache.meshes[b.mesh].index_count / 3) as u64 * b.count as u64;
        }

        // Per-frame bind groups of the effects.
        // The effect bind groups, rebuilt only when what they read changed.
        let fx_key = (
            self.graph.generation,
            scene.id(),
            scene.decal_atlas,
            ao.is_some(),
            scene.lighting_2d.is_some(),
        );
        if self.fx_groups.as_ref().map(|g| g.key) != Some(fx_key) {
            let atlas_view = scene.decal_atlas.map_or(&self.default_white, |t| {
                &self.cache.textures[t.index()].view
            });
            let ao_view = if ao.is_some() {
                self.graph.view("ao")
            } else {
                &self.post.white
            };
            let lightmap_view = if scene.lighting_2d.is_some() {
                self.graph.view("lightmap")
            } else {
                &self.default_white
            };
            let sprite_group = |light: &wgpu::TextureView, label: &str| {
                bind_group(
                    &self.device,
                    label,
                    &self.sprite_fx_layout,
                    &[
                        wgpu::BindingResource::TextureView(light),
                        wgpu::BindingResource::Sampler(&self.sprite_sampler),
                        self.sprite_fx_buffer.as_entire_binding(),
                    ],
                )
            };
            let groups = FxGroups {
                key: fx_key,
                fx: bind_group(
                    &self.device,
                    "fx",
                    &self.fx_layout,
                    &[
                        self.fx_buffer.as_entire_binding(),
                        wgpu::BindingResource::TextureView(ao_view),
                        wgpu::BindingResource::TextureView(atlas_view),
                        wgpu::BindingResource::Sampler(&self.sprite_sampler),
                    ],
                ),
                sprite: sprite_group(lightmap_view, "sprite fx"),
                // The glow pass draws into a target that may alias the light
                // map: its sprites read a placeholder instead.
                glow: sprite_group(&self.default_white, "sprite fx (glow)"),
            };
            self.fx_groups = Some(groups);
        }
        let (_, frame_bind_group, sky_group, reflection_frame_group) = self
            .environment
            .as_ref()
            .expect("environment generated above");
        let groups = self.fx_groups.as_ref().expect("made above");
        let (fx_group, sprite_fx_group, glow_fx_group) = (&groups.fx, &groups.sprite, &groups.glow);
        // The physical pipelines' effect group: the same plus the scene copy
        // (a placeholder in a frame without transmission).
        let physical_fx_group = self.physical.as_ref().and_then(|p| {
            batches
                .iter()
                .chain(&skinned_visible)
                .any(|b| b.physical)
                .then(|| {
                    let atlas_view = scene.decal_atlas.map_or(&self.default_white, |t| {
                        &self.cache.textures[t.index()].view
                    });
                    let ao_view = if ao.is_some() {
                        self.graph.view("ao")
                    } else {
                        &self.post.white
                    };
                    bind_group(
                        &self.device,
                        "physical fx",
                        &p.fx_layout,
                        &[
                            self.fx_buffer.as_entire_binding(),
                            wgpu::BindingResource::TextureView(ao_view),
                            wgpu::BindingResource::TextureView(atlas_view),
                            wgpu::BindingResource::Sampler(&self.sprite_sampler),
                            wgpu::BindingResource::TextureView(
                                transmission_view.as_ref().unwrap_or(&self.post.white),
                            ),
                        ],
                    )
                })
        });
        let scene_depth = self
            .graph
            .try_view("lindepth")
            .unwrap_or(&self.post.far_depth);
        let particle_group = (!particle_frame.systems.is_empty())
            .then(|| self.particles.draw_group(&self.device, scene_depth));
        let metaball_groups: Vec<wgpu::BindGroup> = post::FIELD_NAMES
            .iter()
            .map_while(|name| self.graph.try_view(name))
            .map(|v| self.metaballs.group(&self.device, v))
            .collect();
        let reflection_view = self.graph.try_view("reflection");
        let water_draws = match self.graph.try_view("scene-copy") {
            Some(copy) if !scene.water.is_empty() => self.water.prepare(
                &self.device,
                &self.queue,
                &scene.water,
                (self.width, self.height),
                self.info.tier == Tier::Full,
                copy,
                scene_depth,
                planar.zip(reflection_view),
                &self.post.white,
            ),
            _ => Vec::new(),
        };
        let fluid_frame = match fluid3d::views(&self.graph, scene_depth) {
            Some(views) if fluids_3d => self.fluid3d.prepare(
                &self.device,
                &self.queue,
                &scene.fluids_3d,
                &fluid3d::Fluid3dCamera {
                    view: cam.view,
                    proj: cam.proj,
                    orthographic: cam.orthographic,
                },
                (self.width, self.height),
                &views,
            ),
            _ => None,
        };
        // The planar reflection's effect group: its decals, no AO.
        let reflection_fx_group = reflection.as_ref().map(|_| {
            let atlas_view = scene.decal_atlas.map_or(&self.default_white, |t| {
                &self.cache.textures[t.index()].view
            });
            bind_group(
                &self.device,
                "fx (planar reflection)",
                &self.fx_layout,
                &[
                    self.reflection_fx_buffer.as_entire_binding(),
                    wgpu::BindingResource::TextureView(&self.post.white),
                    wgpu::BindingResource::TextureView(atlas_view),
                    wgpu::BindingResource::Sampler(&self.sprite_sampler),
                ],
            )
        });
        let view_groups = self.view_groups(scene, &view_plans);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        let palette_group = self.palette.bind_group.as_ref();
        // The main pass draws physical materials with their pipelines, the
        // planar reflection with the standard ones.
        let lit_skinned: Vec<&wgpu::RenderPipeline> = skinned_visible
            .iter()
            .map(|b| self.lit_pipeline_of(b, true, b.physical))
            .collect();
        let lit_skinned_standard: Vec<&wgpu::RenderPipeline> = skinned_visible
            .iter()
            .map(|b| self.lit_pipeline_of(b, true, false))
            .collect();
        let shadow_skinned: Vec<&wgpu::RenderPipeline> = skinned_batches
            .iter()
            .map(|b| match b.shader {
                Some(sh) => &self.cache.hook_shadow_pipelines[&(sh, b.side, true)],
                None => &self.shadow_pipelines[&(b.side, true, b.dissolve)],
            })
            .collect();
        let prepass_skinned: Vec<&wgpu::RenderPipeline> = if prepass {
            skinned_visible
                .iter()
                .map(|b| {
                    let hooked = b
                        .shader
                        .and_then(|sh| self.cache.hook_prepass_pipelines.get(&(sh, b.side, true)));
                    let array = self
                        .array_maps
                        .as_ref()
                        .filter(|_| b.array)
                        .and_then(|a| a.prepass.get(&(b.side, true)));
                    hooked
                        .or(array)
                        .or_else(|| self.prepass_pipelines.get(&(b.side, true)))
                        .unwrap_or(lit_skinned[0])
                })
                .collect()
        } else {
            Vec::new()
        };
        let culled_gpu = self.gpu_cull.as_ref().filter(|_| gpu_cull);
        // The effect groups of the standard and the physical pipelines, in
        // the passes that draw physical materials with theirs.
        let physical_fx = physical_fx_group.as_ref().map(|p| (fx_group, p));
        // Draws a list's ranges. Lit passes switch the pipeline when the
        // batch's (side, blending, effects, shader, physical, lean) changes and
        // bind each batch's material (and, between standard and physical,
        // the effect group: `physical_fx`); shadow passes switch on the
        // side and dissolve (with the material).
        let draw_list =
            |pass: &mut wgpu::RenderPass,
             list: &DrawList,
             lit: bool,
             current: &mut Option<LitDrawKey>,
             physical_fx: Option<(&wgpu::BindGroup, &wgpu::BindGroup)>| {
                let mut shadow_key: Option<(ShadowKey, Option<usize>, bool, bool)> = None;
                let mut physical_bound = false;
                for &(b, first, count) in &list.ranges {
                    let batch = &batches[b as usize];
                    if lit {
                        let physical = batch.physical && physical_fx.is_some();
                        let key = (
                            batch.side,
                            batch.blend,
                            batch.effects,
                            batch.shader,
                            physical,
                            batch.lean,
                            batch.derivatives,
                            batch.compact,
                            batch.array,
                        );
                        if *current != Some(key) {
                            pass.set_pipeline(self.lit_pipeline_of(batch, false, physical));
                            *current = Some(key);
                        }
                        if let Some((fx, pfx)) = physical_fx
                            && physical != physical_bound
                        {
                            pass.set_bind_group(2, if physical { pfx } else { fx }, &[]);
                            physical_bound = physical;
                        }
                        let m = &self.cache.materials[batch.material];
                        pass.set_bind_group(
                            1,
                            match (&m.physical, &m.array) {
                                (Some(g), _) if physical => g,
                                (_, Some(g)) => g,
                                _ => &m.bind_group,
                            },
                            &[],
                        );
                    } else {
                        let key = (
                            (batch.side, false, batch.dissolve),
                            batch.shader,
                            batch.compact,
                            batch.array,
                        );
                        if shadow_key != Some(key) {
                            pass.set_pipeline(match batch.shader {
                                Some(sh) => {
                                    &self.cache.hook_shadow_pipelines[&(sh, batch.side, false)]
                                }
                                None if batch.array => {
                                    &self.array_maps.as_ref().expect("an array batch").shadow
                                        [&(batch.side, batch.dissolve)]
                                }
                                None if batch.compact => {
                                    &self.compact_shadow_pipelines[&batch.side]
                                }
                                None => &self.shadow_pipelines[&key.0],
                            });
                            shadow_key = Some(key);
                        }
                        if batch.dissolve || batch.shader.is_some() {
                            pass.set_bind_group(
                                1,
                                &self.cache.materials[batch.material].bind_group,
                                &[],
                            );
                        }
                    }
                    let mesh = &self.cache.meshes[batch.mesh];
                    pass.set_vertex_buffer(0, mesh.vertex_buffer(batch.compact).slice(..));
                    // A buffer slice, not first_instance: WebGL2 has no base instance.
                    let start = first as u64 * INSTANCE_STRIDE;
                    let instances = match (list.indirect, culled_gpu) {
                        (Some(_), Some(gc)) => &gc.instances,
                        _ => &self.instance_buffer,
                    };
                    pass.set_vertex_buffer(
                        1,
                        instances.slice(start..start + count as u64 * INSTANCE_STRIDE),
                    );
                    pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                    match (list.indirect, culled_gpu) {
                        (Some(g), Some(gc)) => {
                            pass.draw_indexed_indirect(&gc.args, gc.draw_offset(g, b))
                        }
                        _ => pass.draw_indexed(0..mesh.index_count, 0, 0..count),
                    }
                }
                if let Some((fx, _)) = physical_fx
                    && physical_bound
                {
                    pass.set_bind_group(2, fx, &[]);
                }
            };
        // The skinned batches (`transparent`: which of them) with the
        // pipelines `pipeline` picks; materials at group `material_group`.
        // `physical_fx` as in draw_list: the main pass's physical batches
        // bind their physical groups (their pipelines are in `pipelines`).
        let draw_skinned = |pass: &mut wgpu::RenderPass,
                            list: &[Batch],
                            transparent: bool,
                            material_group: u32,
                            palette_at: u32,
                            pipelines: &[&wgpu::RenderPipeline],
                            physical_fx: Option<(&wgpu::BindGroup, &wgpu::BindGroup)>|
         -> u32 {
            let Some(palette) = palette_group else {
                return 0;
            };
            let mut draws = 0;
            let mut physical_bound = false;
            // The shadow layouts put the palette at 2: shadows draw an
            // array-map material with its standard group, and only casters.
            let shadow = palette_at == 2;
            for (batch, pipeline) in list.iter().zip(pipelines) {
                if batch.transparent != transparent || (shadow && !batch.cast) {
                    continue;
                }
                let physical = batch.physical && physical_fx.is_some();
                pass.set_pipeline(pipeline);
                if let Some((fx, pfx)) = physical_fx
                    && physical != physical_bound
                {
                    pass.set_bind_group(2, if physical { pfx } else { fx }, &[]);
                    physical_bound = physical;
                }
                let m = &self.cache.materials[batch.material];
                pass.set_bind_group(
                    material_group,
                    match (&m.physical, &m.array) {
                        (Some(g), _) if physical => g,
                        (_, Some(g)) if !shadow => g,
                        _ => &m.bind_group,
                    },
                    &[],
                );
                pass.set_bind_group(palette_at, palette, &[]);
                let mesh = &self.cache.meshes[batch.mesh];
                let skin = self.cache.skins[batch.mesh]
                    .as_ref()
                    .expect("a skinned mesh");
                pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                let start = batch.first as u64 * skin::SKINNED_INSTANCE_STRIDE;
                pass.set_vertex_buffer(
                    1,
                    self.skinned_buffer
                        .slice(start..start + batch.count as u64 * skin::SKINNED_INSTANCE_STRIDE),
                );
                pass.set_vertex_buffer(2, skin.slice(..));
                pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..mesh.index_count, 0, 0..batch.count);
                draws += 1;
            }
            if let Some((fx, _)) = physical_fx
                && physical_bound
            {
                pass.set_bind_group(2, fx, &[]);
            }
            draws
        };
        // One shadow pass: a cascade or a local light's face.
        let terrain_indices = self.terrain.index_buffer();
        let shadow_pass = |encoder: &mut wgpu::CommandEncoder,
                           color: &wgpu::TextureView,
                           depth: &wgpu::TextureView,
                           slot: usize,
                           list: &DrawList,
                           frustum: &Frustum,
                           stats: &mut FrameStats| {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Depth 1 and distance / range 1: nothing nearer.
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.shadow_pipelines[&(Side::Front, false, false)]);
            pass.set_bind_group(
                0,
                &self.shadow_bind_group,
                &[(slot as u64 * SHADOW_SLOT_STRIDE) as u32],
            );
            draw_list(&mut pass, list, false, &mut None, None);
            stats.draw_calls += list.ranges.len() as u32;
            stats.shadow_casters += list.instances;
            stats.shadow_passes += 1;
            // Skinned casters, unculled.
            let n = draw_skinned(
                &mut pass,
                &skinned_batches,
                false,
                1,
                2,
                &shadow_skinned,
                None,
            );
            stats.draw_calls += n;
            stats.shadow_casters += skinned_batches
                .iter()
                .filter(|b| !b.transparent && b.cast)
                .map(|b| b.count)
                .sum::<u32>();
            // Terrain levels that reach into the pass's volume.
            if let Some(indices) = terrain_indices
                && terrain_casters
            {
                let shadow = terrain::TerrainPass::Shadow;
                let (n, _) =
                    terrain::draw_levels(&mut pass, &terrain_draws, indices, shadow, |c, r| {
                        frustum.intersects_sphere(c, r)
                    });
                stats.draw_calls += n;
                stats.terrain_shadow_draws += n;
            }
            let (n, _) = self.scatter.draw(
                &mut pass,
                scatter::ScatterPass::Shadow,
                &scatter_batches,
                scatter_material,
                scatter_mesh,
                |c, r| frustum.intersects_sphere(c, r),
            );
            stats.draw_calls += n;
        };
        // Per-pass CPU encode time, with a clock (`Renderer::set_clock`).
        let clock = self.clock;
        let mut pass_ms: Vec<(String, f64)> = Vec::new();
        for &pass in self.graph.order() {
            let t0 = clock.map(|c| c());
            match pass {
                FramePass::Cascade(i) => shadow_pass(
                    &mut encoder,
                    &self.cascade_map.layers[i as usize],
                    &self.cascade_map.depth,
                    i as usize,
                    &cascade_lists[i as usize],
                    &cascades[i as usize].frustum,
                    &mut stats,
                ),
                FramePass::LocalFace(layer) => shadow_pass(
                    &mut encoder,
                    &self.local_map.layers[layer as usize],
                    &self.local_map.depth,
                    CASCADES + layer as usize,
                    local_lists[layer as usize]
                        .as_ref()
                        .expect("a list per shadowed face"),
                    local_frustums[layer as usize]
                        .as_ref()
                        .expect("a frustum per shadowed face"),
                    &mut stats,
                ),
                FramePass::Prepass => {
                    let attachment = |view, clear: f64| {
                        Some(wgpu::RenderPassColorAttachment {
                            view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: clear,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 0.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })
                    };
                    let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("prepass"),
                        color_attachments: &[
                            attachment(self.graph.view("gbuffer"), 0.0),
                            attachment(self.graph.view("lindepth"), post::FAR_DEPTH as f64),
                            attachment(self.graph.view("outline-id"), 0.0),
                        ],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: self.graph.view("pre-depth"),
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                // Stylise's object pass tests against it.
                                store: if object_space {
                                    wgpu::StoreOp::Store
                                } else {
                                    wgpu::StoreOp::Discard
                                },
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    p.set_bind_group(0, frame_bind_group, &[]);
                    p.set_bind_group(2, fx_group, &[]);
                    let instances = match (main_list.indirect, culled_gpu) {
                        (Some(_), Some(gc)) => &gc.instances,
                        _ => &self.instance_buffer,
                    };
                    let prev = if object_motion {
                        &self.prev_instance_buffer
                    } else {
                        instances
                    };
                    let mut current = None;
                    for &(b, first, count) in &main_list.ranges {
                        let batch = &batches[b as usize];
                        let key = (
                            batch.side,
                            batch.shader,
                            batch.lean,
                            batch.derivatives,
                            batch.compact,
                            batch.array,
                        );
                        if current != Some(key) {
                            p.set_pipeline(match batch.shader {
                                Some(sh) => {
                                    &self.cache.hook_prepass_pipelines[&(sh, batch.side, false)]
                                }
                                None if batch.lean => {
                                    &self.lean_prepass_pipelines
                                        [&(batch.side, batch.derivatives, batch.compact)]
                                }
                                None if batch.array => {
                                    &self.array_maps.as_ref().expect("an array batch").prepass
                                        [&(batch.side, false)]
                                }
                                None => &self.prepass_pipelines[&(batch.side, false)],
                            });
                            current = Some(key);
                        }
                        let m = &self.cache.materials[batch.material];
                        p.set_bind_group(1, m.array.as_ref().unwrap_or(&m.bind_group), &[]);
                        let mesh = &self.cache.meshes[batch.mesh];
                        let start = first as u64 * INSTANCE_STRIDE;
                        let end = start + count as u64 * INSTANCE_STRIDE;
                        p.set_vertex_buffer(0, mesh.vertex_buffer(batch.compact).slice(..));
                        p.set_vertex_buffer(1, instances.slice(start..end));
                        p.set_vertex_buffer(2, prev.slice(start..end));
                        p.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                        match (main_list.indirect, culled_gpu) {
                            (Some(g), Some(gc)) => {
                                p.draw_indexed_indirect(&gc.args, gc.draw_offset(g, b))
                            }
                            _ => p.draw_indexed(0..mesh.index_count, 0, 0..count),
                        }
                        stats.prepass_draws += 1;
                    }
                    stats.prepass_draws += draw_skinned(
                        &mut p,
                        &skinned_visible,
                        false,
                        1,
                        3,
                        &prepass_skinned,
                        None,
                    );
                    if let Some(indices) = terrain_indices {
                        let prepass = terrain::TerrainPass::Prepass;
                        let (n, _) = terrain::draw_levels(
                            &mut p,
                            &terrain_draws,
                            indices,
                            prepass,
                            |c, r| view_frustum.intersects_sphere(c, r),
                        );
                        stats.prepass_draws += n;
                    }
                    let (n, _) = self.scatter.draw(
                        &mut p,
                        scatter::ScatterPass::Prepass,
                        &scatter_batches,
                        scatter_material,
                        scatter_mesh,
                        |c, r| view_frustum.intersects_sphere(c, r),
                    );
                    stats.prepass_draws += n;
                    stats.draw_calls += stats.prepass_draws;
                    drop(p);
                    if let Some(object) = self.stylise_object.as_ref().filter(|_| object_space) {
                        // Stylise's object pass (stylise_object.rs): the
                        // prepass's instances again, against its depth.
                        let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("stylise object"),
                            color_attachments: &[attachment(self.graph.view("object-space"), 0.0)],
                            depth_stencil_attachment: Some(
                                wgpu::RenderPassDepthStencilAttachment {
                                    view: self.graph.view("pre-depth"),
                                    depth_ops: Some(wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: wgpu::StoreOp::Discard,
                                    }),
                                    stencil_ops: None,
                                },
                            ),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });
                        p.set_bind_group(0, frame_bind_group, &[]);
                        p.set_bind_group(2, fx_group, &[]);
                        let mut side = None;
                        for &(b, first, count) in &main_list.ranges {
                            let batch = &batches[b as usize];
                            if side != Some(batch.side) {
                                p.set_pipeline(object.pipeline(batch.side, false));
                                side = Some(batch.side);
                            }
                            p.set_bind_group(
                                1,
                                &self.cache.materials[batch.material].bind_group,
                                &[],
                            );
                            let mesh = &self.cache.meshes[batch.mesh];
                            let start = first as u64 * INSTANCE_STRIDE;
                            let end = start + count as u64 * INSTANCE_STRIDE;
                            p.set_vertex_buffer(0, mesh.vertices.slice(..));
                            p.set_vertex_buffer(1, instances.slice(start..end));
                            p.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                            match (main_list.indirect, culled_gpu) {
                                (Some(g), Some(gc)) => {
                                    p.draw_indexed_indirect(&gc.args, gc.draw_offset(g, b))
                                }
                                _ => p.draw_indexed(0..mesh.index_count, 0, 0..count),
                            }
                            stats.draw_calls += 1;
                        }
                        let skinned: Vec<&wgpu::RenderPipeline> = skinned_visible
                            .iter()
                            .map(|b| object.pipeline(b.side, true))
                            .collect();
                        stats.draw_calls +=
                            draw_skinned(&mut p, &skinned_visible, false, 1, 3, &skinned, None);
                    }
                }
                FramePass::Particles => {
                    self.particles.simulate(
                        &self.device,
                        &mut encoder,
                        &particle_frame,
                        scene_depth,
                        self.graph.try_view("gbuffer").unwrap_or(&self.post.white),
                    );
                }
                FramePass::Metaballs => {
                    let fields: Vec<&wgpu::TextureView> = post::FIELD_NAMES
                        .iter()
                        .map_while(|name| self.graph.try_view(name))
                        .collect();
                    self.metaballs
                        .encode_field(&mut encoder, &fields, frame_bind_group);
                    stats.draw_calls += fields.len() as u32;
                }
                FramePass::Fluid3d => {
                    if let (Some(f), Some(views)) =
                        (&fluid_frame, fluid3d::views(&self.graph, scene_depth))
                    {
                        stats.draw_calls +=
                            self.fluid3d
                                .encode(&mut encoder, f, frame_bind_group, &views);
                    }
                }
                FramePass::Light2d => {
                    let ambient = scene.lighting_2d.map_or(Vec3::ONE, |l| l.ambient);
                    self.lights_2d
                        .encode(&mut encoder, self.graph.view("lightmap"), ambient);
                    stats.draw_calls += (lights_2d > 0) as u32;
                }
                FramePass::Reflection => {
                    let clear = match scene.environment.background {
                        Background::Color(c) => wgpu::Color {
                            r: c.x as f64,
                            g: c.y as f64,
                            b: c.z as f64,
                            a: 1.0,
                        },
                        Background::Sky | Background::Procedural { .. } => wgpu::Color::BLACK,
                    };
                    let target = self.graph.view("reflection");
                    let (view, resolve_target) = if self.samples > 1 {
                        (self.graph.view("reflection-msaa"), Some(target))
                    } else {
                        (target, None)
                    };
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("planar reflection"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view,
                            depth_slice: None,
                            resolve_target,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(clear),
                                store: if resolve_target.is_some() {
                                    wgpu::StoreOp::Discard
                                } else {
                                    wgpu::StoreOp::Store
                                },
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: self.graph.view("reflection-depth"),
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Discard,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    pass.set_pipeline(&self.pbr_pipelines[&(Side::Front, false, false, false)]);
                    pass.set_bind_group(0, reflection_frame_group, &[]);
                    pass.set_bind_group(
                        2,
                        reflection_fx_group
                            .as_ref()
                            .expect("made with the reflection"),
                        &[],
                    );
                    let list = reflection_list
                        .as_ref()
                        .expect("culled with the reflection");
                    draw_list(
                        &mut pass,
                        list,
                        true,
                        &mut Some((
                            Side::Front,
                            false,
                            false,
                            None,
                            false,
                            false,
                            false,
                            false,
                            false,
                        )),
                        None,
                    );
                    stats.draw_calls += list.ranges.len() as u32;
                    stats.reflection_draws += list.ranges.len() as u32;
                    let n = draw_skinned(
                        &mut pass,
                        &skinned_visible,
                        false,
                        1,
                        3,
                        &lit_skinned_standard,
                        None,
                    );
                    stats.draw_calls += n;
                    stats.reflection_draws += n;
                    if let (Some(indices), Some(r)) = (terrain_indices, &reflection) {
                        let frustum = Frustum::from_view_proj(r.3);
                        let (n, _) = terrain::draw_levels(
                            &mut pass,
                            &terrain_draws,
                            indices,
                            terrain::TerrainPass::Lit,
                            |c, rad| frustum.intersects_sphere(c, rad),
                        );
                        stats.draw_calls += n;
                        stats.reflection_draws += n;
                    }
                    let sky = match scene.environment.background {
                        Background::Sky => Some(&self.sky_pipeline),
                        Background::Procedural { .. } => Some(&self.sky_look_pipeline),
                        Background::Color(_) => None,
                    };
                    if let Some(sky) = sky {
                        pass.set_pipeline(sky);
                        pass.set_bind_group(1, sky_group, &[]);
                        pass.draw(0..3, 0..1);
                        stats.draw_calls += 1;
                        stats.reflection_draws += 1;
                    }
                }
                FramePass::View(i) => {
                    // A secondary camera (views.rs): opaque meshes,
                    // terrain and sky, then its resolve into its target.
                    let (i, clear) = (i as usize, background_clear(&scene.environment.background));
                    let p = &view_plans[i];
                    let frustum = Frustum::from_view_proj(p.frame.view_proj);
                    {
                        let mut pass = self.views.begin(&mut encoder, p, clear);
                        pass.set_bind_group(0, &view_groups[i].0, &[]);
                        pass.set_bind_group(2, &view_groups[i].1, &[]);
                        draw_list(&mut pass, &view_lists[i], true, &mut None, None);
                        stats.draw_calls += view_lists[i].ranges.len() as u32;
                        let pipelines: Vec<&wgpu::RenderPipeline> = view_skinned[i]
                            .iter()
                            .map(|b| self.lit_pipeline_of(b, true, false))
                            .collect();
                        stats.draw_calls += draw_skinned(
                            &mut pass,
                            &view_skinned[i],
                            false,
                            1,
                            3,
                            &pipelines,
                            None,
                        );
                        if let Some(indices) = terrain_indices {
                            stats.draw_calls += terrain::draw_levels(
                                &mut pass,
                                &terrain_draws,
                                indices,
                                terrain::TerrainPass::Lit,
                                |c, rad| frustum.intersects_sphere(c, rad),
                            )
                            .0;
                        }
                        let sky = match scene.environment.background {
                            Background::Sky => Some(&self.sky_pipeline),
                            Background::Procedural { .. } => Some(&self.sky_look_pipeline),
                            Background::Color(_) => None,
                        };
                        if let (Some(sky), None) = (sky, p.clear) {
                            pass.set_pipeline(sky);
                            pass.set_bind_group(1, sky_group, &[]);
                            pass.draw(0..3, 0..1);
                            stats.draw_calls += 1;
                        }
                    }
                    if let views::Dest::Texture {
                        color,
                        format,
                        depth,
                    } = p.dest
                    {
                        let tonemap = post_fx.tonemap;
                        let (device, queue) = (&self.device, &self.queue);
                        let view = &self.cache.textures[color].view;
                        self.views.resolve(
                            device,
                            queue,
                            &mut encoder,
                            p,
                            tonemap,
                            view,
                            format.wgpu(),
                            false,
                            self.samples,
                        );
                        if let Some(d) = depth {
                            let view = &self.cache.textures[d].view;
                            let f = wgpu::TextureFormat::Rgba16Float;
                            self.views.resolve(
                                device,
                                queue,
                                &mut encoder,
                                p,
                                tonemap,
                                view,
                                f,
                                true,
                                self.samples,
                            );
                        }
                        stats.draw_calls += 1 + depth.is_some() as u32;
                    }
                }
                FramePass::Main => {
                    let clear = match scene.environment.background {
                        Background::Color(c) => wgpu::Color {
                            r: c.x as f64,
                            g: c.y as f64,
                            b: c.z as f64,
                            a: 1.0,
                        },
                        Background::Sky | Background::Procedural { .. } => wgpu::Color::BLACK,
                    };
                    let hdr = self.graph.view("hdr");
                    let msaa = self.samples > 1;
                    // With water or a 3D fluid the pass is split after the
                    // sky: the scene so far is copied for them to refract
                    // and reflect, then the pass goes on over it.
                    // So is it for transmissive materials, which refract the
                    // copy's mip chain (`physical.rs`).
                    let split = !water_draws.is_empty()
                        || fluid_frame.is_some()
                        || transmission_view.is_some();
                    let begin = |encoder: &mut wgpu::CommandEncoder, first: bool| {
                        let (view, resolve_target, store) = if msaa {
                            (
                                self.graph.view("hdr-msaa"),
                                Some(hdr),
                                if first && split {
                                    wgpu::StoreOp::Store
                                } else {
                                    wgpu::StoreOp::Discard
                                },
                            )
                        } else {
                            (hdr, None, wgpu::StoreOp::Store)
                        };
                        encoder
                            .begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some(if first { "main" } else { "main (after water)" }),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view,
                                    depth_slice: None,
                                    resolve_target,
                                    ops: wgpu::Operations {
                                        load: if first {
                                            wgpu::LoadOp::Clear(clear)
                                        } else {
                                            wgpu::LoadOp::Load
                                        },
                                        store,
                                    },
                                })],
                                depth_stencil_attachment: Some(
                                    wgpu::RenderPassDepthStencilAttachment {
                                        view: self.graph.view("depth"),
                                        depth_ops: Some(wgpu::Operations {
                                            load: if first {
                                                wgpu::LoadOp::Clear(1.0)
                                            } else {
                                                wgpu::LoadOp::Load
                                            },
                                            // TAA reads the depth to reproject the history.
                                            store: if taa || (first && split) {
                                                wgpu::StoreOp::Store
                                            } else {
                                                wgpu::StoreOp::Discard
                                            },
                                        }),
                                        stencil_ops: None,
                                    },
                                ),
                                timestamp_writes: None,
                                occlusion_query_set: None,
                                multiview_mask: None,
                            })
                            .forget_lifetime()
                    };
                    let mut pass = begin(&mut encoder, true);
                    pass.set_pipeline(&self.pbr_pipelines[&(Side::Front, false, false, false)]);
                    pass.set_bind_group(0, frame_bind_group, &[]);
                    pass.set_bind_group(2, fx_group, &[]);
                    draw_list(
                        &mut pass,
                        &main_list,
                        true,
                        &mut Some((
                            Side::Front,
                            false,
                            false,
                            None,
                            false,
                            false,
                            false,
                            false,
                            false,
                        )),
                        physical_fx,
                    );
                    stats.draw_calls += main_list.ranges.len() as u32;
                    stats.draw_calls += draw_skinned(
                        &mut pass,
                        &skinned_visible,
                        false,
                        1,
                        3,
                        &lit_skinned,
                        physical_fx,
                    );
                    if let Some(indices) = terrain_indices {
                        let (n, tris) = terrain::draw_levels(
                            &mut pass,
                            &terrain_draws,
                            indices,
                            terrain::TerrainPass::Lit,
                            |c, r| view_frustum.intersects_sphere(c, r),
                        );
                        stats.draw_calls += n;
                        stats.terrain_draws += n;
                        stats.terrain_triangles += tris;
                        stats.triangles += tris;
                    }
                    let (n, copies) = self.scatter.draw(
                        &mut pass,
                        scatter::ScatterPass::Lit,
                        &scatter_batches,
                        scatter_material,
                        scatter_mesh,
                        |c, r| view_frustum.intersects_sphere(c, r),
                    );
                    if n > 0 {
                        // The copies' group 2 is theirs: the effects' back for what follows.
                        pass.set_bind_group(2, fx_group, &[]);
                    }
                    stats.draw_calls += n;
                    stats.scatter_draws += n;
                    stats.scatter_copies += copies;
                    let sky = match scene.environment.background {
                        Background::Sky => Some(&self.sky_pipeline),
                        Background::Procedural { .. } => Some(&self.sky_look_pipeline),
                        Background::Color(_) => None,
                    };
                    if let Some(sky) = sky {
                        pass.set_pipeline(sky);
                        pass.set_bind_group(1, sky_group, &[]);
                        pass.draw(0..3, 0..1);
                        stats.draw_calls += 1;
                    }
                    if split {
                        drop(pass);
                        let size = wgpu::Extent3d {
                            width: self.width,
                            height: self.height,
                            depth_or_array_layers: 1,
                        };
                        if let Some(copy) = self.graph.try_view("scene-copy") {
                            encoder.copy_texture_to_texture(
                                hdr.texture().as_image_copy(),
                                copy.texture().as_image_copy(),
                                size,
                            );
                        }
                        if let Some(view) = &transmission_view {
                            encoder.copy_texture_to_texture(
                                hdr.texture().as_image_copy(),
                                view.texture().as_image_copy(),
                                size,
                            );
                            self.mips.record(&self.device, &mut encoder, view.texture());
                        }
                        pass = begin(&mut encoder, false);
                        pass.set_bind_group(0, frame_bind_group, &[]);
                        stats.draw_calls += self.water.draw(&mut pass, &water_draws);
                        if let Some(f) = &fluid_frame {
                            stats.draw_calls += self.fluid3d.draw(&mut pass, f);
                        }
                    }
                    // Transparent meshes, back to front over the opaque scene.
                    pass.set_bind_group(2, fx_group, &[]);
                    draw_list(&mut pass, &blended_list, true, &mut None, physical_fx);
                    stats.draw_calls += blended_list.ranges.len() as u32;
                    stats.draw_calls += draw_skinned(
                        &mut pass,
                        &skinned_visible,
                        true,
                        1,
                        3,
                        &lit_skinned,
                        physical_fx,
                    );
                    // Particles, far systems first.
                    if let Some(group) = &particle_group {
                        let groups = &self.cache.sprite_groups;
                        let white = &self.white_sprite_group;
                        stats.draw_calls += self.particles.draw(
                            &mut pass,
                            &particle_frame,
                            self.samples,
                            group,
                            |t| match t {
                                None => white,
                                Some(t) => {
                                    groups[t.index()].as_ref().expect("made before the pass")
                                }
                            },
                        );
                    }
                    // Trails, ribbons and beams (layered 2D trails go with the sprites).
                    for run in ribbon_frame.runs.iter().filter(|r| r.layer.is_none()) {
                        pass.set_pipeline(
                            &self.ribbon_pipelines[match run.blend {
                                ParticleBlend::Additive => 0,
                                ParticleBlend::Alpha => 1,
                            }],
                        );
                        pass.set_bind_group(
                            1,
                            match run.texture {
                                None => &self.white_sprite_group,
                                Some(t) => self.cache.ribbon_groups[t.index()]
                                    .as_ref()
                                    .expect("made before the pass"),
                            },
                            &[],
                        );
                        let stride = std::mem::size_of::<ribbons::RibbonVertex>() as u64;
                        let start = run.first as u64 * stride;
                        pass.set_vertex_buffer(
                            0,
                            self.ribbon_buffer
                                .slice(start..start + run.count as u64 * stride),
                        );
                        pass.draw(0..run.count, 0..1);
                        stats.draw_calls += 1;
                    }
                    // Lines, points and wireframes (lines.rs), with their own group 0.
                    let n = self.lines.draw_scene(&mut pass, &line_frame);
                    if n > 0 {
                        stats.draw_calls += n;
                        pass.set_bind_group(0, frame_bind_group, &[]);
                    }
                    // Sprites and text, blended over the scene in sorted order.
                    pass.set_bind_group(2, sprite_fx_group, &[]);
                    let mut current: Option<(bool, SpriteBlend, bool)> = None;
                    let mut covered = false;
                    for run in &sprite_frame.runs {
                        if !run.tone_mapped && !covered {
                            // The HDR alpha to 1 before the first untonemapped run.
                            pass.set_pipeline(&self.sprite_cover_pipeline);
                            pass.draw(0..3, 0..1);
                            stats.draw_calls += 1;
                            covered = true;
                            current = None;
                        }
                        if let RunSource::Metaballs(i) = run.source
                            && let Some(group) = self
                                .metaballs
                                .field_of(i)
                                .and_then(|f| metaball_groups.get(f))
                        {
                            self.metaballs.composite(&mut pass, group, i);
                            stats.draw_calls += 1;
                            pass.set_bind_group(2, sprite_fx_group, &[]);
                            current = None;
                            continue;
                        }
                        if let RunSource::Strip(blend, texture) = run.source {
                            pass.set_pipeline(
                                &self.ribbon_pipelines
                                    [usize::from(blend != ParticleBlend::Additive as u8)],
                            );
                            pass.set_bind_group(
                                1,
                                match texture {
                                    None => &self.white_sprite_group,
                                    Some(t) => self.cache.ribbon_groups[t.index()]
                                        .as_ref()
                                        .expect("made before the pass"),
                                },
                                &[],
                            );
                            let stride = std::mem::size_of::<ribbons::RibbonVertex>() as u64;
                            let start = run.first as u64 * stride;
                            pass.set_vertex_buffer(
                                0,
                                self.ribbon_buffer
                                    .slice(start..start + run.count as u64 * stride),
                            );
                            pass.draw(0..run.count, 0..1);
                            stats.draw_calls += 1;
                            // The sprite pipeline and its groups again.
                            pass.set_bind_group(2, sprite_fx_group, &[]);
                            current = None;
                            continue;
                        }
                        if let RunSource::Mesh(i) = run.source {
                            let mesh = &scene.sprite_meshes[i as usize];
                            pass.set_bind_group(
                                1,
                                match mesh.texture {
                                    None => &self.white_sprite_group,
                                    Some(t) => self.cache.sprite_groups[t.index()]
                                        .as_ref()
                                        .expect("made before the pass"),
                                },
                                &[],
                            );
                            self.sprite_meshes
                                .as_ref()
                                .expect("made before the pass")
                                .draw(&mut pass, mesh.blend, run.first, run.count);
                            stats.draw_calls += 1;
                            current = None;
                            continue;
                        }
                        if let RunSource::Array(blend, a) = run.source {
                            let arrays = self.sprite_arrays.as_ref().expect("made before the pass");
                            pass.set_pipeline(arrays.pipeline(blend, run.tone_mapped));
                            let (group, _) = self.cache.sprite_array_groups[a.index()]
                                .as_ref()
                                .expect("made before the pass");
                            pass.set_bind_group(1, group, &[]);
                            let start = run.first as u64 * SPRITE_STRIDE;
                            pass.set_vertex_buffer(
                                0,
                                self.sprite_buffer
                                    .slice(start..start + run.count as u64 * SPRITE_STRIDE),
                            );
                            pass.draw(0..6, 0..run.count);
                            stats.draw_calls += 1;
                            current = None;
                            continue;
                        }
                        let (msdf, blend, group) = match run.source {
                            RunSource::Strip(..)
                            | RunSource::Metaballs(_)
                            | RunSource::Mesh(_)
                            | RunSource::Array(..) => {
                                unreachable!("drawn above")
                            }
                            RunSource::Glass => unreachable!("glass is the overlay's only"),
                            RunSource::Sprite(blend, None) => {
                                (false, blend, &self.white_sprite_group)
                            }
                            RunSource::Sprite(blend, Some(t)) => (
                                false,
                                blend,
                                self.cache.sprite_groups[t.index()]
                                    .as_ref()
                                    .expect("made before the pass"),
                            ),
                            RunSource::Font(f) => (
                                true,
                                SpriteBlend::Alpha,
                                &self.cache.fonts[f.index()].bind_group,
                            ),
                        };
                        if current != Some((msdf, blend, run.tone_mapped)) {
                            pass.set_pipeline(match (msdf, blend, run.tone_mapped) {
                                (true, _, _) => &self.msdf_pipeline,
                                (false, SpriteBlend::Alpha, false) => {
                                    &self.sprite_untonemapped_pipeline
                                }
                                (false, SpriteBlend::Alpha, true) => &self.sprite_pipeline,
                                (false, SpriteBlend::Additive, _) => &self.sprite_additive_pipeline,
                            });
                            current = Some((msdf, blend, run.tone_mapped));
                        }
                        pass.set_bind_group(1, group, &[]);
                        let start = run.first as u64 * SPRITE_STRIDE;
                        pass.set_vertex_buffer(
                            0,
                            self.sprite_buffer
                                .slice(start..start + run.count as u64 * SPRITE_STRIDE),
                        );
                        pass.draw(0..6, 0..run.count);
                        stats.draw_calls += 1;
                    }
                }
                FramePass::Glow => {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("glow"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: self.graph.view("glow"),
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    pass.set_bind_group(0, frame_bind_group, &[]);
                    pass.set_bind_group(2, glow_fx_group, &[]);
                    // Plain sprites first (sorted before arrays), then each array's.
                    let mut on_array = None;
                    for run in &sprite_frame.glow_runs {
                        let array = matches!(run.source, RunSource::Array(..));
                        if on_array != Some(array) {
                            pass.set_pipeline(match array {
                                false => &self.sprite_glow_pipeline,
                                true => {
                                    &self
                                        .sprite_arrays
                                        .as_ref()
                                        .expect("made before the pass")
                                        .glow
                                }
                            });
                            on_array = Some(array);
                        }
                        let group = match run.source {
                            RunSource::Sprite(_, Some(t)) => self.cache.sprite_groups[t.index()]
                                .as_ref()
                                .expect("made before the pass"),
                            RunSource::Array(_, a) => {
                                &self.cache.sprite_array_groups[a.index()]
                                    .as_ref()
                                    .expect("made before the pass")
                                    .0
                            }
                            _ => &self.white_sprite_group,
                        };
                        pass.set_bind_group(1, group, &[]);
                        let start = run.first as u64 * SPRITE_STRIDE;
                        pass.set_vertex_buffer(
                            0,
                            self.sprite_buffer
                                .slice(start..start + run.count as u64 * SPRITE_STRIDE),
                        );
                        pass.draw(0..6, 0..run.count);
                        stats.draw_calls += 1;
                    }
                }
                FramePass::CustomPost(i) => {
                    let (read, write) = self.graph.custom_post_views(&self.persistent, i, slot);
                    let lit = self.graph.custom_post_views(&self.persistent, 0, slot).0;
                    stats.draw_calls += self.custom_posts.encode(
                        &self.device,
                        &self.queue,
                        &mut encoder,
                        scene,
                        custom_posts[i as usize],
                        [
                            read,
                            lit,
                            self.graph.view("lindepth"),
                            self.graph.view("gbuffer"),
                            write,
                        ],
                        (self.width, self.height),
                    );
                }
                post_pass => {
                    let post_output = match (&self.glass_targets, glass_on) {
                        (Some(g), true) => &g.source_view,
                        _ => output,
                    };
                    stats.draw_calls += self.graph.encode_post(
                        &mut encoder,
                        &self.post,
                        &self.persistent,
                        post_pass,
                        slot,
                        post_output,
                        gtao,
                    );
                }
            }
            if let (Some(c), Some(t0)) = (clock, t0) {
                pass_ms.push((pass.name(), c() - t0));
            }
        }
        self.pass_ms = pass_ms;
        // Viewport views (split-screen, views.rs) over the finished frame,
        // under the lines and the overlay.
        for p in view_plans
            .iter()
            .filter(|p| matches!(p.dest, views::Dest::Viewport(_)))
        {
            let dest = overlay_view.unwrap_or(output);
            self.views.resolve(
                &self.device,
                &self.queue,
                &mut encoder,
                p,
                post_fx.tonemap,
                dest,
                viewport_format,
                false,
                self.samples,
            );
            stats.draw_calls += 1;
        }
        // Glass over overlay content (src/glass.rs): when overlay runs come
        // before a glass run, the overlay draws into the glass source, the
        // pyramid is rebuilt before each such glass run, and the source is
        // copied to the output at the end. Otherwise the source is copied
        // and blurred once, before the overlay, which draws into the output.
        let runs: &[sprites::SpriteRun] = overlay_frame.as_ref().map_or(&[], |f| &f.runs);
        let decode = !self.output_format.is_srgb();
        let mut rebuilds = Vec::new();
        let mut layered = None;
        if let (Some(g), true) = (&self.glass_targets, glass_on) {
            rebuilds = glass::rebuilds(
                &runs
                    .iter()
                    .map(|r| r.source == RunSource::Glass)
                    .collect::<Vec<_>>(),
            );
            let over_content = rebuilds.first().is_some_and(|&i| i > 0) || rebuilds.len() > 1;
            // The overlay draws into the source through the view it blends in.
            layered = match (over_content, overlay_view) {
                (false, _) => None,
                (true, None) => Some(&g.source_view),
                (true, Some(_)) => g.source_linear_view.as_ref(),
            };
            if layered.is_none() {
                stats.draw_calls += self
                    .glass
                    .encode(&self.queue, &mut encoder, g, output, decode);
                rebuilds.clear();
            }
        }
        // Lines over the finished frame (debug draw, screen lines), under
        // the overlay, where the overlay draws (lines.rs).
        if line_frame.overlay.1 > 0 {
            let srgb_view = overlay_view.is_some();
            self.lines.set_overlay_uniform(
                &self.queue,
                &lines::LineUniform::new(
                    cam.view_proj,
                    cam.proj,
                    self.out_width,
                    self.out_height,
                    scene.css_height,
                    !srgb_view && !self.output_format.is_srgb(),
                ),
            );
            let target = layered.unwrap_or(overlay_view.unwrap_or(output));
            stats.draw_calls +=
                self.lines
                    .draw_overlay(&mut encoder, target, srgb_view, &line_frame);
        }
        // The overlay: over the finished frame, in the output (or its sRGB
        // view), or in the glass source when glass lies over overlay content.
        if !runs.is_empty() {
            let pipelines = match overlay_view {
                Some(_) => self
                    .overlay
                    .linear
                    .as_ref()
                    .expect("an sRGB view's pipelines"),
                None => &self.overlay.output,
            };
            let target = layered.unwrap_or(overlay_view.unwrap_or(output));
            // The runs between pyramid builds, each in a pass of its own:
            // cut `k` ends the runs drawn before the k-th build.
            let mut cuts: Vec<usize> = rebuilds.clone();
            cuts.push(runs.len());
            let mut from = 0;
            for (k, &to) in cuts.iter().enumerate() {
                // The pyramid as the overlay stands before run `from`.
                if k > 0
                    && let Some(g) = self.glass_targets.as_ref()
                {
                    stats.draw_calls += self.glass.pyramid(&self.queue, &mut encoder, g, decode);
                }
                if to == from {
                    continue;
                }
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("overlay"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: target,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_bind_group(0, &self.overlay.group, &[]);
                let mut bound: Option<u8> = None;
                for run in &runs[from..to] {
                    // 0 sprite, 1 MSDF text, 2 glass, 3 a texture array's sprites.
                    let (kind, group) = match run.source {
                        RunSource::Sprite(_, None) => (0u8, &self.white_sprite_group),
                        RunSource::Sprite(_, Some(t)) => (
                            0,
                            self.cache.sprite_groups[t.index()]
                                .as_ref()
                                .expect("made before the pass"),
                        ),
                        RunSource::Font(f) => (1, &self.cache.fonts[f.index()].bind_group),
                        RunSource::Glass => (
                            2,
                            &self
                                .glass_targets
                                .as_ref()
                                .expect("made with the overlay's glass")
                                .overlay_group,
                        ),
                        RunSource::Array(_, a) => (
                            3,
                            &self.cache.sprite_array_groups[a.index()]
                                .as_ref()
                                .expect("made before the pass")
                                .0,
                        ),
                        RunSource::Strip(..) | RunSource::Metaballs(_) | RunSource::Mesh(_) => {
                            unreachable!("the overlay has no strips, fluids or meshes")
                        }
                    };
                    if bound != Some(kind) {
                        pass.set_pipeline(match kind {
                            0 => &pipelines.sprite,
                            1 => &pipelines.msdf,
                            2 => &pipelines.glass,
                            _ => self
                                .sprite_arrays
                                .as_ref()
                                .expect("made before the pass")
                                .overlay(overlay_view.is_some()),
                        });
                        bound = Some(kind);
                    }
                    pass.set_bind_group(1, group, &[]);
                    let start = run.first as u64 * overlay::OVERLAY_STRIDE;
                    pass.set_vertex_buffer(
                        0,
                        self.overlay
                            .buffer
                            .slice(start..start + run.count as u64 * overlay::OVERLAY_STRIDE),
                    );
                    pass.draw(0..6, 0..run.count);
                    stats.draw_calls += 1;
                }
                from = to;
            }
        }
        if let (Some(g), Some(_)) = (&self.glass_targets, layered) {
            stats.draw_calls += self.glass.blit(&mut encoder, g, output);
        }
        self.queue
            .submit(cull_commands.into_iter().chain(Some(encoder.finish())));
        if let Some(gc) = self.gpu_cull.as_mut().filter(|_| gpu_cull) {
            gc.after_submit();
        }
        if let Some((frame, ..)) = surface_frame {
            self.queue.present(frame);
        }
        if taa {
            self.taa_history = Some(cam.view_proj);
        }
        self.prev_view_proj = Some(cam.view_proj);
        self.frame_index += 1;
        // The first frame made the scene's pipelines: keep them now, not
        // only when the renderer is dropped (a killed process never is).
        if self.frame_index == 1
            && let Some(cache) = &mut self.pipeline_disk
        {
            cache.save();
        }
        Ok(stats)
    }

    /// Reads the next frame's GPU culling counts back (`culled`,
    /// `occluded`, `triangles`, `shadow_casters` and the triangle counts
    /// in and out of view; see
    /// [`FrameStats::counts_lag`]), for one frame: a frame rendered
    /// without a request copies none, so the stats keep the newest
    /// requested frame's. Natively [`Renderer::finish_stats`] needs no
    /// request for the last frame rendered.
    pub fn request_culling_counts(&mut self) {
        self.culling_counts = true;
    }

    /// Makes one pipeline no frame has drawn with yet, the likeliest to be
    /// drawn soon first (the sprites and the overlay, the post passes, then
    /// sky, ribbons, 2D lights, glass, water, terrain, metaballs, fluids);
    /// false when every pipeline is made. The renderer makes a pipeline when
    /// a pass first sets it (deferred.rs), so a frame compiles only what it
    /// draws; a host with idle time (the web page behind its play gate,
    /// where WebGL2 compiles each program synchronously) calls this until
    /// it answers false, so a pass met later does not stop a frame. The
    /// frames are the same either way.
    pub fn warm(&mut self) -> bool {
        deferred::warm_one(&[
            &self.sprite_pipeline,
            &self.msdf_pipeline,
            &self.sprite_cover_pipeline,
            &self.sprite_untonemapped_pipeline,
            &self.sprite_additive_pipeline,
            &self.sprite_glow_pipeline,
        ]) || self.overlay.warm()
            || self.post.warm()
            || deferred::warm_one(&[
                &self.sky_pipeline,
                &self.sky_look_pipeline,
                &self.ribbon_pipelines[0],
                &self.ribbon_pipelines[1],
            ])
            || self.lights_2d.warm()
            || self.glass.warm()
            || self.water.warm()
            || self.terrain.warm()
            || self.metaballs.warm()
            || self.fluid3d.warm()
            || self.lines.warm()
    }

    /// Natively: waits for the GPU culling counts of the frame `stats` came
    /// from and puts them in place of the older ones it was given
    /// ([`FrameStats::counts_lag`] becomes 0); for the last frame rendered
    /// they are copied now when it copied none. Whether it could: false on
    /// the CPU path, or for a frame that found no readback free and was
    /// not the last one rendered.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn finish_stats(&mut self, stats: &mut FrameStats) -> bool {
        if !stats.gpu_culling {
            return false;
        }
        let Some((exact, given)) = self
            .gpu_cull
            .as_mut()
            .and_then(|gc| gc.finish(&self.device, &self.queue, stats.frame))
        else {
            return false;
        };
        stats.culled = stats.culled - given.culled + exact.culled;
        stats.occluded = exact.occluded;
        stats.occluded_triangles = exact.occluded_triangles;
        stats.in_view_triangles = exact.triangles + exact.occluded_triangles;
        stats.triangles = stats.triangles - given.triangles + exact.triangles;
        stats.shadow_casters = stats.shadow_casters - given.shadow_casters + exact.shadow_casters;
        stats.counts_lag = 0;
        true
    }

    /// A copy of a particle system's slots after the last frame (the GPU
    /// path reads its buffer back): for tests and tools (native only).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read_particles(&self, id: u64) -> Option<Vec<ParticleState>> {
        self.particles.read(&self.device, &self.queue, id)
    }

    /// Whether `target`, not sRGB itself, has an sRGB view the overlay
    /// blends in ([`linear_view_format`]).
    fn has_linear_view(&self, target: &RenderTarget) -> bool {
        match target {
            RenderTarget::Offscreen => self.persistent.offscreen_linear.is_some(),
            RenderTarget::Surface => self
                .surface
                .as_ref()
                .is_some_and(|(_, config)| !config.view_formats.is_empty()),
            RenderTarget::View(_) => false,
        }
    }

    /// Forgets the TAA history: the next frame starts from its own samples
    /// (call it on a camera cut).
    pub fn reset_history(&mut self) {
        self.taa_history = None;
    }

    /// Keeps the renderer's state that belongs to the world's time (the
    /// adapted exposure of [`Exposure::Auto`] with adapt speeds, and the
    /// tick it was adapted to) for a world snapshot; [`Renderer::restore_state`]
    /// with the answer brings it back, so frames after a restore are the
    /// frames the snapshot's world would have drawn. Kept on the GPU (no
    /// readback): the last 64 saves are kept.
    pub fn save_state(&mut self) -> RendererState {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("save state"),
            });
        let exposure = self.post.save_exposure(&mut encoder);
        self.queue.submit(Some(encoder.finish()));
        RendererState { exposure }
    }

    /// Brings back what [`Renderer::save_state`] kept; state no longer kept
    /// starts afresh (the next adapted frame takes the metered exposure).
    pub fn restore_state(&mut self, state: RendererState) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("restore state"),
            });
        self.post.restore_exposure(&mut encoder, state.exposure);
        self.queue.submit(Some(encoder.finish()));
        self.views.stale = true;
    }
}

/// What [`Renderer::save_state`] kept, by number ([`RendererState::id`] and
/// [`RendererState::from_id`] carry it through a host's snapshot as a u64).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RendererState {
    exposure: u64,
}

impl RendererState {
    /// The number a host keeps with its snapshot (0: nothing kept).
    pub fn id(self) -> u64 {
        self.exposure
    }

    pub fn from_id(id: u64) -> RendererState {
        RendererState { exposure: id }
    }
}

/// The camera moved by a [`CameraShake`]: offset in view space (a fraction
/// of the view height at the target) and rolled about the view axis.
fn shaken_camera(camera: &Camera, shake: CameraShake) -> Camera {
    if shake == CameraShake::default() {
        return *camera;
    }
    let forward = (camera.target - camera.position).normalize_or(Vec3::NEG_Z);
    let right = forward.cross(camera.up).normalize_or(Vec3::X);
    let up = right.cross(forward);
    let height = match camera.projection {
        Projection::Perspective { fov_y_degrees } => {
            2.0 * (fov_y_degrees.to_radians() * 0.5).tan()
                * (camera.target - camera.position).length()
        }
        Projection::Orthographic { height } => height,
    };
    let offset = (right * shake.offset.x + up * shake.offset.y) * height;
    let roll = glam::Quat::from_axis_angle(forward, shake.roll_degrees.to_radians());
    Camera {
        position: camera.position + offset,
        target: camera.target + offset,
        up: roll * camera.up,
        ..*camera
    }
}

fn bind_group(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    resources: &[wgpu::BindingResource],
) -> wgpu::BindGroup {
    let entries: Vec<_> = resources
        .iter()
        .enumerate()
        .map(|(i, r)| wgpu::BindGroupEntry {
            binding: i as u32,
            resource: r.clone(),
        })
        .collect();
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &entries,
    })
}

fn sprite_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    params: [f32; 4],
) -> wgpu::BindGroup {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("sprite params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    bind_group(
        device,
        "sprite",
        layout,
        &[
            wgpu::BindingResource::TextureView(view),
            wgpu::BindingResource::Sampler(sampler),
            buffer.as_entire_binding(),
        ],
    )
}

/// Replaces `buffer` (a vertex buffer) with a larger one when `len` bytes
/// do not fit; true when it did.
fn grow(device: &wgpu::Device, buffer: &mut wgpu::Buffer, label: &str, len: u64) -> bool {
    if len <= buffer.size() {
        return false;
    }
    *buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: len.next_power_of_two(),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    true
}

/// Writes `bytes` at the start of `buffer`, replacing it with a larger one first if needed.
fn grow_and_write(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffer: &mut wgpu::Buffer,
    label: &str,
    bytes: &[u8],
) {
    grow(device, buffer, label, bytes.len() as u64);
    if !bytes.is_empty() {
        queue.write_buffer(buffer, 0, bytes);
    }
}
