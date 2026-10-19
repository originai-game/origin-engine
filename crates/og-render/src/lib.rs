//! Origin Engine renderer (`docs/design.md` §2.6, `crates/og-render/README.md`).
//!
//! A wgpu forward renderer: PBR metallic-roughness, one directional light with
//! three cascaded shadow maps, up to eight point and spot lights (two of them
//! shadowed), split-sum image-based lighting generated from a procedural sky,
//! antialiasing by tier (TAA on WebGPU-class backends, 4x MSAA on WebGL2),
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

mod atmosphere;
mod capture;
mod effects;
mod frame;
mod fx;
pub mod graph;
mod hooks;
mod ibl;
mod light2d;
mod material;
mod metaballs;
mod mips;
mod overlay;
pub mod particles;
mod particles_gpu;
mod post;
mod ribbons;
mod scene;
pub mod shapes;
mod skin;
mod sprites;
mod texture;
mod water;
mod weather;

#[cfg(feature = "gltf")]
pub mod gltf;
#[cfg(not(target_arch = "wasm32"))]
mod headless;

pub use atmosphere::{DayNightLook, Fog, SkyLook, day_night};
pub use capture::{CAPTURE_TIMEOUT, CaptureRequest};
pub use effects::*;
pub use graph::{GraphError, GraphPlan, RenderGraph, TargetDesc};
#[cfg(not(target_arch = "wasm32"))]
pub use headless::render_to_rgba;
pub use hooks::{
    MAX_SHADER_PARAMS, MaterialShader, ParamKind, SHADER_HOOKS, ShaderParam, lit_module_source,
};
pub use material::{Side, StandardMaterial};
pub use metaballs::{FIELDS, MAX_FLUIDS, Metaballs};
pub use particles::{
    Burst, ColorCurve, Curve, EmitterHandle, EmitterShape, MAX_CURVE_KEYS, ParticleAlign,
    ParticleBlend, ParticleCollision, ParticleEmitter, ParticleNoise, ParticleRender,
    ParticleSimulation, ParticleState, ParticleSystem, SheetMode, SimulationSpace, SpriteSheet,
};
pub use ribbons::{Beam, Ribbon, RibbonPoint, RibbonUv, Trail, TrailStyle, TrailStyleHandle};
pub use scene::*;
pub use skin::{
    JointPalette, MAX_JOINTS, MAX_MORPH_TARGETS, MorphTargets, SkinData, SkinnedInstance,
};
pub use texture::{TextureCompression, TextureDesc, TextureFormat, TextureFormats, TextureUpload};
pub use water::{Water, Water2d, WaterHeights};
pub use weather::{
    WEATHER_DUST, WEATHER_MIST, WEATHER_NONE, WEATHER_RAIN, WEATHER_SNOW, WeatherLook, weather,
};
/// The wgpu this crate is built on, for hosts that create surfaces or pass views.
pub use wgpu;

use frame::{CASCADES, Cascade, Frustum, LOCAL_SHADOW_LAYERS};
use glam::{Mat4, Vec2, Vec3};
use material::MaterialUniform;
use post::FramePass;
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

const FRAME_WGSL: &str = include_str!("../shaders/frame.wgsl");
const PBR_WGSL: &str = include_str!("../shaders/pbr.wgsl");

/// The lit shader's source with `hooks` (`hooks.rs`) and the joint palette
/// read as `palette`.
fn lit_source_with(hooks: &str, palette: skin::PaletteMode) -> String {
    format!(
        "{FRAME_WGSL}{}{MATERIAL_WGSL}{}{hooks}{PBR_WGSL}",
        particles_gpu::NOISE_WGSL,
        palette.wgsl(3)
    )
}

/// [`lit_source_with`] for a tier (`full`: the storage-buffer palette).
pub(crate) fn lit_source(hooks: &str, full: bool) -> String {
    lit_source_with(
        hooks,
        if full {
            skin::PaletteMode::Storage
        } else {
            skin::PaletteMode::Texture
        },
    )
}
const SHADOW_WGSL: &str = include_str!("../shaders/shadow.wgsl");
const SKY_WGSL: &str = include_str!("../shaders/sky.wgsl");
const SPRITE_WGSL: &str = include_str!("../shaders/sprite.wgsl");
const MATERIAL_WGSL: &str = include_str!("../shaders/material.wgsl");
const RIBBON_WGSL: &str = include_str!("../shaders/ribbon.wgsl");

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
    /// The platform's primary API: Metal or Vulkan natively, WebGPU in a browser.
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
            Backend::Auto => wgpu::Backends::PRIMARY,
            Backend::WebGpu => wgpu::Backends::BROWSER_WEBGPU,
            Backend::WebGl2 => wgpu::Backends::GL,
            Backend::Metal => wgpu::Backends::METAL,
            Backend::Vulkan => wgpu::Backends::VULKAN,
        }
    }

    /// A wgpu instance for this backend, for hosts that create their own surface
    /// before calling [`Renderer::with_surface`].
    pub fn instance(self) -> wgpu::Instance {
        wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: self.backends(),
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        })
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
    /// MSAA on the no-compute tier (WebGL2, GL).
    Tier,
    /// 4x MSAA on any tier (falls back to `Off` where the HDR format cannot be
    /// multisampled). Sharp sprites and text; no temporal blur.
    Msaa4,
    /// Temporal antialiasing on any tier: a sub-pixel jitter per frame and a
    /// reprojected, neighbourhood-clamped history. Converges over ~8 frames.
    Taa,
}

/// How to create a [`Renderer`]. The default is a 1280x720 renderer on the
/// platform's primary API with an sRGB RGBA8 output, no antialiasing, three
/// 2048² shadow cascades and BC/ASTC/ETC2 texture compression where the GPU
/// has it. [`RenderDesc::mobile`] is the preset for phones.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

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
    /// The antialiasing in use: `Off`, `Msaa4` or `Taa`.
    pub antialias: Antialias,
    /// The cascade size and count in use (after clamping).
    pub shadow_map_size: u32,
    pub shadow_cascades: u32,
    /// The encoding each kind of texture should be uploaded in, and every
    /// encoding this device accepts.
    pub texture_formats: TextureFormats,
}

impl RenderInfo {
    /// Whether the adapter is a software rasterizer (lavapipe, llvmpipe,
    /// SwiftShader): the `vulkan-cpu` tier.
    pub fn software(&self) -> bool {
        self.device_type == wgpu::DeviceType::Cpu
    }

    /// The name gfx reference frames are filed under: `metal`, `vulkan`,
    /// `vulkan-cpu` (lavapipe/llvmpipe), `webgpu`, `webgl2`, `gl`, `dx12`.
    pub fn backend_tier(&self) -> String {
        let cpu = self.device_type == wgpu::DeviceType::Cpu;
        match self.backend {
            wgpu::Backend::Metal => "metal".into(),
            wgpu::Backend::Vulkan if cpu => "vulkan-cpu".into(),
            wgpu::Backend::Vulkan => "vulkan".into(),
            wgpu::Backend::BrowserWebGpu => "webgpu".into(),
            wgpu::Backend::Gl if cfg!(target_arch = "wasm32") => "webgl2".into(),
            wgpu::Backend::Gl => "gl".into(),
            wgpu::Backend::Dx12 => "dx12".into(),
            other => format!("{other:?}").to_lowercase(),
        }
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
    /// Instances outside the camera's view, not drawn in the main pass.
    pub culled: u32,
    /// Triangles drawn in the main pass.
    pub triangles: u64,
    /// Instance draws across every shadow pass (after culling per pass).
    pub shadow_casters: u32,
    /// Shadow passes (cascades plus local light faces).
    pub shadow_passes: u32,
    /// Local lights drawn, and those beyond [`MAX_LIGHTS`] that were not.
    pub lights: u32,
    pub lights_dropped: u32,
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
    /// Decals drawn, and those beyond [`MAX_DECALS`].
    pub decals: u32,
    pub decals_dropped: u32,
    /// 2D lights and occluders drawn.
    pub lights_2d: u32,
    pub occluders_2d: u32,
    /// Skinned instances drawn and the joints of the frame's palettes.
    pub skinned_instances: u32,
    pub joints: u32,
    /// Frames rendered by this renderer before this one (drives the TAA jitter).
    pub frame: u64,
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
    pad: [f32; 44],
}

const INSTANCE_STRIDE: u64 = std::mem::size_of::<[[f32; 4]; 4]>() as u64;

const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Float32x4],
};

const INSTANCE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: INSTANCE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4],
};

struct GpuMesh {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_count: u32,
    /// The scene's revision of the mesh this holds (`RenderScene::set_mesh`).
    revision: u64,
    /// Local-space bounds as (center, half extent).
    bounds: Option<(Vec3, Vec3)>,
}

struct GpuMaterial {
    revision: u64,
    bind_group: wgpu::BindGroup,
    side: Side,
    transparent: bool,
    /// Discards pixels (shadows draw it with the material).
    dissolve: bool,
    /// Has its own outline (turns the outline pass on).
    outline: bool,
    /// Uses toon, rim, flash or dissolve.
    effects: bool,
    /// Its material shader (the scene's index).
    shader: Option<usize>,
}

struct GpuTexture {
    view: wgpu::TextureView,
    format: TextureFormat,
}

struct GpuFont {
    layout: FontLayout,
    bind_group: wgpu::BindGroup,
}

/// GPU copies of one scene's resources, indexed like the scene's handles.
#[derive(Default)]
struct SceneCache {
    scene_id: u64,
    meshes: Vec<GpuMesh>,
    /// The lit shader with each of the scene's material shaders.
    shaders: Vec<wgpu::ShaderModule>,
    /// Their pipelines by (shader, side, blended, effects).
    hook_pipelines: HashMap<(usize, Side, bool, bool), wgpu::RenderPipeline>,
    textures: Vec<GpuTexture>,
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
    luts: Vec<wgpu::TextureView>,
}

/// One (mesh, material) run of the sorted instances.
struct Batch {
    mesh: usize,
    material: usize,
    first: u32,
    count: u32,
    side: Side,
    transparent: bool,
    dissolve: bool,
    /// Needs the effect shader (its material's or the frame's effects).
    effects: bool,
    /// Its material's shader (never for skinned batches).
    shader: Option<usize>,
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
/// decal and AO code is compiled out (the features-off cost).
type PbrKey = (Side, bool, bool, bool);
/// The shadow pipelines: (side, skinned, dissolving).
type ShadowKey = (Side, bool, bool);

/// What one pass draws: (batch, first instance, count) ranges in the instance buffer.
#[derive(Default)]
struct DrawList {
    ranges: Vec<(u32, u32, u32)>,
    instances: u32,
    triangles: u64,
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
        // wgpu's GL backend must pick a texture's target when creating it and
        // takes a square texture of a multiple of six layers for a cube map
        // (array), which WebGL2 then neither renders to by layer nor reads as
        // a 2D array: the twelve local shadow layers read as zero (every
        // shadowed light dark). One spare layer keeps it a 2D array.
        let allocated = if layers.is_multiple_of(6) {
            layers + 1
        } else {
            layers
        };
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
    width: u32,
    height: u32,
    output_format: wgpu::TextureFormat,
    /// The offscreen target's sRGB view format ([`linear_view_format`]).
    offscreen_linear: Option<wgpu::TextureFormat>,
    surface: Option<(wgpu::Surface<'static>, wgpu::SurfaceConfiguration)>,
    samples: u32,
    persistent: post::Persistent,
    graph: post::FrameGraph,
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
    sky_pipeline: wgpu::RenderPipeline,
    /// The procedural sky of a chosen look drawn per pixel ([`Background::Procedural`]).
    sky_look_pipeline: wgpu::RenderPipeline,
    water: water::WaterRenderer,
    metaballs: metaballs::MetaballRenderer,
    /// The image-based light, regenerated into the same textures for a new sky.
    ibl: ibl::IblGenerator,
    sprite_pipeline: wgpu::RenderPipeline,
    sprite_additive_pipeline: wgpu::RenderPipeline,
    sprite_glow_pipeline: wgpu::RenderPipeline,
    msdf_pipeline: wgpu::RenderPipeline,
    ribbon_pipelines: [wgpu::RenderPipeline; 2],
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
    particles: particles_gpu::ParticleRenderer,
    lights_2d: light2d::Light2dRenderer,
    trails: ribbons::Trails,
    /// The previous frame's unjittered view-projection (motion vectors).
    prev_view_proj: Option<Mat4>,
    fx_groups: Option<FxGroups>,
    mips: mips::MipGenerator,
    /// The frame bind group (camera, shadow maps, IBL) and the sky pass's
    /// (the sky cube) for the sky they were generated from.
    environment: Option<(ProceduralSky, wgpu::BindGroup, wgpu::BindGroup)>,
    default_white: wgpu::TextureView,
    default_normal: wgpu::TextureView,
    white_sprite_group: wgpu::BindGroup,
    instance_buffer: wgpu::Buffer,
    sprite_buffer: wgpu::Buffer,
    overlay: overlay::OverlayRenderer,
    cache: SceneCache,
    /// Frames rendered so far.
    frame_index: u64,
    /// Where particles simulate ([`RenderDesc::particles`]).
    desc_particles: ParticleSimulation,
    /// The previous frame's unjittered view-projection, when TAA history is valid.
    taa_history: Option<Mat4>,
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

pub(crate) fn hdr_pipeline(device: &wgpu::Device, p: HdrPipeline) -> wgpu::RenderPipeline {
    hdr_pipeline_with(device, &[], p)
}

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
        cache: None,
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
    (side, transparent, skinned, effects): PbrKey,
    samples: u32,
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
            label: if transparent { "pbr blended" } else { "pbr" },
            layout: if skinned { layouts.1 } else { layouts.0 },
            module,
            vs: if skinned { "vs_skinned" } else { "vs" },
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

/// `PrevIn` in pbr.wgsl: the previous transform (motion vectors).
const PREV_INSTANCE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: INSTANCE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![11 => Float32x4, 12 => Float32x4, 13 => Float32x4, 14 => Float32x4],
};

/// The depth-normal prepass pipeline for (side, skinned).
fn prepass_pipeline(
    device: &wgpu::Device,
    layouts: (&wgpu::PipelineLayout, &wgpu::PipelineLayout),
    module: &wgpu::ShaderModule,
    side: Side,
    skinned: bool,
) -> wgpu::RenderPipeline {
    let plain = [
        Some(VERTEX_LAYOUT),
        Some(INSTANCE_LAYOUT),
        Some(PREV_INSTANCE_LAYOUT),
    ];
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
        label: Some("prepass"),
        layout: Some(if skinned { layouts.1 } else { layouts.0 }),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some(if skinned {
                "vs_prepass_skinned"
            } else {
                "vs_prepass"
            }),
            compilation_options: Default::default(),
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
            entry_point: Some("fs_prepass"),
            compilation_options: Default::default(),
            targets: &[
                target(post::GBUFFER_FORMAT),
                target(post::LINDEPTH_FORMAT),
                target(post::OUTLINE_FORMAT),
            ],
        }),
        multiview_mask: None,
        cache: None,
    })
}

/// The shadow pipeline for casters drawn with (side, skinned, dissolve).
fn shadow_pipeline(
    device: &wgpu::Device,
    layouts: &[wgpu::PipelineLayout; 3],
    module: &wgpu::ShaderModule,
    (side, skinned, dissolve): ShadowKey,
) -> wgpu::RenderPipeline {
    let plain = [Some(VERTEX_LAYOUT), Some(INSTANCE_LAYOUT)];
    let skin = [
        Some(VERTEX_LAYOUT),
        Some(skin::SKINNED_INSTANCE_LAYOUT),
        Some(skin::SKIN_VERTEX_LAYOUT),
    ];
    let layout = match (skinned, dissolve) {
        (true, _) => &layouts[2],
        (false, true) => &layouts[1],
        (false, false) => &layouts[0],
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some(if skinned { "vs_skinned" } else { "vs" }),
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
            entry_point: Some(if dissolve { "fs_dissolve" } else { "fs" }),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::R32Float,
                blend: None,
                write_mask: wgpu::ColorWrites::RED,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

impl Renderer {
    /// A headless renderer (draws to [`RenderTarget::Offscreen`] or caller views).
    /// On the web, WebGL2 needs a canvas: use [`Renderer::with_surface`].
    pub async fn new(desc: &RenderDesc) -> Result<Renderer, RenderError> {
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
        let msaa_ok = supports_msaa(&adapter);
        let antialias = match desc.antialias {
            Antialias::Off => Antialias::Off,
            Antialias::Taa => Antialias::Taa,
            Antialias::Tier if tier == Tier::Full => Antialias::Taa,
            Antialias::Tier | Antialias::Msaa4 if msaa_ok => Antialias::Msaa4,
            Antialias::Tier | Antialias::Msaa4 => Antialias::Off,
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
        let shadow_size = desc
            .shadow_map_size
            .clamp(64, limits.max_texture_dimension_2d.max(64));
        let cascades = desc.shadow_cascades.clamp(1, MAX_CASCADES);
        let (features, texture_formats) =
            TextureFormats::choose(adapter.features(), desc.texture_compression);
        let info = RenderInfo {
            adapter: adapter_info.name.clone(),
            backend: adapter_info.backend,
            device_type: adapter_info.device_type,
            tier,
            antialias,
            shadow_map_size: shadow_size,
            shadow_cascades: cascades,
            texture_formats,
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
        if let Some((surface, config)) = &surface {
            surface.configure(&device, config);
        }
        let (width, height) = (desc.width.max(1), desc.height.max(1));

        // Layouts.
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
        // At least two layers: GL cannot view a single-layer texture as an array.
        let cascade_map = ShadowMap::new(&device, "cascades", shadow_size, cascades.max(2));
        let local_map = ShadowMap::new(
            &device,
            "local shadows",
            LOCAL_SHADOW_SIZE,
            LOCAL_SHADOW_LAYERS as u32,
        );

        // Uniforms.
        let frame_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("frame"),
            size: std::mem::size_of::<frame::FrameUniform>() as u64,
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
            lit_source_with(&hooks::default_hooks(), palette_mode),
        );
        let sky_shader = shader(&device, "sky", format!("{FRAME_WGSL}{SKY_WGSL}"));
        let sprite_shader = shader(&device, "sprite", format!("{FRAME_WGSL}{SPRITE_WGSL}"));
        let ribbon_shader = shader(&device, "ribbon", format!("{FRAME_WGSL}{RIBBON_WGSL}"));
        let shadow_shader = shader(
            &device,
            "shadow",
            format!(
                "{}{MATERIAL_WGSL}{}{SHADOW_WGSL}",
                particles_gpu::NOISE_WGSL,
                palette_mode.wgsl(2)
            ),
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
            shadow_pipeline(&device, &shadow_layouts, &shadow_shader, key),
        );
        // At the far plane, after the opaque batches: only uncovered pixels pass.
        let sky_state = |fs| HdrPipeline {
            label: "sky",
            layout: &sky_pl,
            module: &sky_shader,
            vs: "vs_sky",
            fs,
            buffers: &[],
            cull: None,
            depth_write: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            blend: None,
            samples,
        };
        let sky_pipeline = hdr_pipeline(&device, sky_state("fs_sky"));
        let sky_look_pipeline = hdr_pipeline(&device, sky_state("fs_sky_look"));
        let ibl = ibl::IblGenerator::new(&device);
        let water = water::WaterRenderer::new(&device, &frame_layout, FRAME_WGSL, samples);
        let metaballs =
            metaballs::MetaballRenderer::new(&device, &frame_layout, FRAME_WGSL, samples);
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
        let sprite_buffers = [Some(SPRITE_LAYOUT)];
        let sprite_state = |label, fs, blend| HdrPipeline {
            label,
            layout: &sprite_pl,
            module: &sprite_shader,
            vs: "vs",
            fs,
            buffers: &sprite_buffers,
            cull: None,
            depth_write: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            blend: Some(blend),
            samples,
        };
        let sprite_pipeline = hdr_pipeline(
            &device,
            sprite_state("sprite", "fs", wgpu::BlendState::ALPHA_BLENDING),
        );
        let sprite_additive_pipeline =
            hdr_pipeline(&device, sprite_state("sprite additive", "fs", additive));
        let msdf_pipeline = hdr_pipeline(
            &device,
            sprite_state("msdf text", "fs_msdf", wgpu::BlendState::ALPHA_BLENDING),
        );
        // The glow buffer: no depth, one sample, added.
        let sprite_glow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite glow"),
            layout: Some(&sprite_pl),
            vertex: wgpu::VertexState {
                module: &sprite_shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &sprite_buffers,
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
            cache: None,
        });
        let ribbon_buffers = [Some(ribbons::RIBBON_LAYOUT)];
        let ribbon_state = |blend| HdrPipeline {
            label: "ribbon",
            layout: &ribbon_pl,
            module: &ribbon_shader,
            vs: "vs",
            fs: "fs",
            buffers: &ribbon_buffers,
            cull: None,
            depth_write: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            blend: Some(blend),
            samples,
        };
        let ribbon_pipelines = [
            hdr_pipeline(&device, ribbon_state(additive)),
            hdr_pipeline(&device, ribbon_state(wgpu::BlendState::ALPHA_BLENDING)),
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

        let post = post::PostPasses::new(&device, &queue, output_format, antialias);
        let mips = mips::MipGenerator::new(&device);
        let default_white = solid_texture(&device, &queue, [255; 4]);
        let default_normal = solid_texture(&device, &queue, [128, 128, 255, 255]);
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

        Ok(Renderer {
            device,
            queue,
            info,
            width,
            height,
            output_format,
            offscreen_linear,
            surface,
            samples,
            persistent,
            graph: post::FrameGraph::default(),
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
            sky_pipeline,
            sky_look_pipeline,
            water,
            metaballs,
            ibl,
            sprite_pipeline,
            sprite_additive_pipeline,
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
            particles,
            lights_2d,
            trails: ribbons::Trails::default(),
            prev_view_proj: None,
            fx_groups: None,
            mips,
            environment: None,
            default_white,
            default_normal,
            white_sprite_group,
            instance_buffer,
            sprite_buffer,
            overlay,
            cache: SceneCache::default(),
            frame_index: 0,
            desc_particles: desc.particles,
            taa_history: None,
        })
    }

    pub fn info(&self) -> &RenderInfo {
        &self.info
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
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
    /// history restarts.
    pub fn resize(&mut self, width: u32, height: u32) {
        let (width, height) = (width.max(1), height.max(1));
        if (width, height) == (self.width, self.height) {
            return;
        }
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
            let hooks = scene.shaders()[index].hooks_wgsl();
            let module = shader(
                &self.device,
                "pbr (material shader)",
                lit_source_with(&hooks, self.palette.mode()),
            );
            self.cache.shaders.push(module);
        }
        self.refresh_meshes(scene);
        for index in self.cache.meshes.len()..scene.meshes().len() {
            let mesh = &scene.meshes()[index];
            let mut gpu = self.upload_mesh(mesh);
            gpu.revision = scene.mesh_revisions()[index];
            self.cache.meshes.push(gpu);
            let mut range = (0u32, 0u32);
            if let Some(m) = &scene.morphs()[index] {
                let n = mesh.vertices.len();
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
            // A morphed mesh without a skin: all-zero weights (bind pose).
            let zero_skin;
            let source = match &scene.skins()[index] {
                None if range.1 > 0 => {
                    zero_skin = SkinData {
                        joints: vec![[0; 4]; mesh.vertices.len()],
                        weights: vec![[0.0; 4]; mesh.vertices.len()],
                    };
                    Some(&zero_skin)
                }
                other => other.as_ref(),
            };
            let skin = match source {
                None => None,
                Some(skin) => {
                    let n = mesh.vertices.len();
                    if skin.joints.len() != n || skin.weights.len() != n {
                        return Err(RenderError::InvalidData(format!(
                            "mesh {index}: a skin of {} joints and {} weights for {n} vertices",
                            skin.joints.len(),
                            skin.weights.len()
                        )));
                    }
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
        for (index, lut) in scene.luts()[self.cache.luts.len()..].iter().enumerate() {
            lut.validate().map_err(|e| {
                RenderError::InvalidData(format!("lut {}: {e}", self.cache.luts.len() + index))
            })?;
            let view = post::lut_texture(&self.device, &self.queue, lut);
            self.cache.luts.push(view);
        }
        if self.cache.textures.len() < scene.textures().len() {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("texture upload"),
                });
            for index in self.cache.textures.len()..scene.textures().len() {
                let gpu = match &scene.textures()[index] {
                    TextureSource::Rgba(texture) => GpuTexture {
                        view: self.upload_texture(texture, &mut encoder)?,
                        format: TextureFormat::Rgba8,
                    },
                    TextureSource::Encoded(upload) => self.upload_encoded(index, upload)?,
                };
                self.cache.textures.push(gpu);
                self.cache.sprite_groups.push(None);
                self.cache.ribbon_groups.push(None);
            }
            self.queue.submit(Some(encoder.finish()));
        }
        for (i, (material, revision)) in scene.materials().enumerate() {
            if self
                .cache
                .materials
                .get(i)
                .is_some_and(|m| m.revision == revision)
            {
                continue;
            }
            let hooked = scene.material_shader(MaterialHandle::from_index(i));
            let gpu = GpuMaterial {
                revision,
                bind_group: self.material_bind_group(i, material, hooked.map(|h| h.1))?,
                side: material.side(),
                transparent: material.is_transparent(),
                dissolve: material.dissolves(),
                effects: material.toon != 0
                    || material.rimIntensity > 0.0
                    || material.flash > 0.0
                    || material.dissolves(),
                outline: material.outlineWidth > 0.0,
                shader: hooked.map(|h| h.0.index()),
            };
            if i < self.cache.materials.len() {
                self.cache.materials[i] = gpu;
            } else {
                self.cache.materials.push(gpu);
            }
        }
        for (i, font) in scene.fonts()[self.cache.fonts.len()..].iter().enumerate() {
            let index = self.cache.fonts.len() + i;
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
                layout: FontLayout::new(&font.glyphs, font.line_height, atlas_size),
                bind_group,
            });
        }
        Ok(())
    }

    fn upload_mesh(&self, mesh: &MeshData) -> GpuMesh {
        // Never create an empty buffer: an empty mesh keeps a 16-byte placeholder and draws nothing.
        let bytes = |b: &[u8]| {
            if b.is_empty() {
                vec![0u8; 16]
            } else {
                b.to_vec()
            }
        };
        let vertices = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertices"),
                contents: &bytes(bytemuck::cast_slice(&mesh.vertices)),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
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
            let mesh = &scene.meshes()[i];
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
                let mut fresh = self.upload_mesh(mesh);
                fresh.revision = revision;
                self.cache.meshes[i] = fresh;
            }
        }
    }

    fn upload_texture(
        &mut self,
        texture: &TextureData,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<wgpu::TextureView, RenderError> {
        let expected = texture.width as usize * texture.height as usize * 4;
        if texture.width == 0 || texture.height == 0 || texture.rgba.len() != expected {
            return Err(RenderError::InvalidData(format!(
                "texture {}x{} with {} bytes (expected {expected})",
                texture.width,
                texture.height,
                texture.rgba.len()
            )));
        }
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
        Ok(tex.create_view(&Default::default()))
    }

    /// A texture whose levels arrive encoded (no mip generation).
    fn upload_encoded(
        &self,
        index: usize,
        upload: &TextureUpload,
    ) -> Result<GpuTexture, RenderError> {
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
        let format = desc.format.wgpu(desc.color_space).expect("validated");
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
        Ok(GpuTexture {
            view: texture.create_view(&Default::default()),
            format: desc.format,
        })
    }

    fn material_bind_group(
        &self,
        index: usize,
        m: &StandardMaterial,
        custom: Option<[[f32; 4]; MAX_SHADER_PARAMS]>,
    ) -> Result<wgpu::BindGroup, RenderError> {
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
        Ok(bind_group(
            &self.device,
            "material",
            &self.material_layout,
            &entries,
        ))
    }

    /// (Re)generates the image-based lighting when the sky changed.
    fn ensure_environment(&mut self, sky: &ProceduralSky) {
        if self.environment.as_ref().is_some_and(|(s, ..)| s == sky) {
            return;
        }
        self.ibl.generate(
            &self.device,
            &self.queue,
            sky.sun_direction,
            sky.sun_color,
            sky.look.as_ref(),
        );
        let ibl = self.ibl.views();
        let entries = [
            self.frame_buffer.as_entire_binding(),
            wgpu::BindingResource::TextureView(&self.cascade_map.array_view),
            wgpu::BindingResource::TextureView(&self.local_map.array_view),
            wgpu::BindingResource::TextureView(&ibl.irradiance),
            wgpu::BindingResource::TextureView(&ibl.prefiltered),
            wgpu::BindingResource::TextureView(&ibl.brdf_lut),
            wgpu::BindingResource::Sampler(&ibl.sampler),
        ];
        let group = bind_group(&self.device, "frame", &self.frame_layout, &entries);
        let sky_group = bind_group(
            &self.device,
            "sky",
            &self.sky_layout,
            &[wgpu::BindingResource::TextureView(&ibl.sky)],
        );
        self.environment = Some((*sky, group, sky_group));
    }

    /// Sorts the instances into (mesh, material) batches; returns the batches,
    /// the sorted transforms, each sorted instance's world bounding sphere and
    /// its submission index.
    #[allow(clippy::type_complexity)]
    fn build_batches(
        &self,
        scene: &RenderScene,
    ) -> Result<(Vec<Batch>, Vec<[[f32; 4]; 4]>, Vec<(Vec3, f32)>, Vec<u32>), RenderError> {
        for (i, inst) in scene.instances.iter().enumerate() {
            if inst.mesh.index() >= self.cache.meshes.len() {
                return Err(RenderError::InvalidHandle(format!(
                    "instance {i}: mesh {}",
                    inst.mesh.index()
                )));
            }
            if inst.material.index() >= self.cache.materials.len() {
                return Err(RenderError::InvalidHandle(format!(
                    "instance {i}: material {}",
                    inst.material.index()
                )));
            }
        }
        let mut order: Vec<usize> = (0..scene.instances.len()).collect();
        // Stable: instances of one batch keep their submission order.
        order.sort_by_key(|&i| (scene.instances[i].mesh, scene.instances[i].material));
        let mut transforms = Vec::with_capacity(order.len());
        let mut spheres = Vec::with_capacity(order.len());
        let mut submission = Vec::with_capacity(order.len());
        let mut batches: Vec<Batch> = Vec::new();
        for &i in &order {
            let inst = &scene.instances[i];
            let (mesh, material) = (inst.mesh.index(), inst.material.index());
            let gpu = &self.cache.meshes[mesh];
            if gpu.index_count == 0 {
                continue;
            }
            match batches.last_mut() {
                Some(b) if b.mesh == mesh && b.material == material => b.count += 1,
                _ => {
                    let m = &self.cache.materials[material];
                    batches.push(Batch {
                        mesh,
                        material,
                        first: transforms.len() as u32,
                        count: 1,
                        side: m.side,
                        transparent: m.transparent,
                        dissolve: m.dissolve,
                        effects: m.effects,
                        shader: m.shader,
                    })
                }
            }
            let m = inst.transform;
            spheres.push(match gpu.bounds {
                Some((c, e)) => {
                    let scale = m
                        .x_axis
                        .truncate()
                        .length()
                        .max(m.y_axis.truncate().length())
                        .max(m.z_axis.truncate().length());
                    (m.transform_point3(c), e.length() * scale)
                }
                None => (m.w_axis.truncate(), 0.0),
            });
            transforms.push(m.to_cols_array_2d());
            submission.push(i as u32);
        }
        Ok((batches, transforms, spheres, submission))
    }

    /// The draw ranges of one pass: per batch (`include`d), every instance
    /// whose sphere touches `frustum`. A batch that loses less than a tenth
    /// of its instances is drawn whole (cheaper than copying its
    /// transforms); otherwise its visible instances are appended to
    /// `transforms`.
    fn cull(
        &self,
        batches: &[Batch],
        spheres: &[(Vec3, f32)],
        transforms: &mut Vec<[[f32; 4]; 4]>,
        origins: &mut Vec<u32>,
        frustum: &Frustum,
        include: impl Fn(&Batch) -> bool,
    ) -> (DrawList, u32) {
        let mut list = DrawList::default();
        let mut culled = 0;
        let mut visible = Vec::new();
        for (b_index, b) in batches.iter().enumerate() {
            if !include(b) {
                continue;
            }
            let range = b.first as usize..(b.first + b.count) as usize;
            visible.clear();
            visible.extend(
                range
                    .clone()
                    .filter(|&k| frustum.intersects_sphere(spheres[k].0, spheres[k].1)),
            );
            culled += b.count - visible.len() as u32;
            let index_count = self.cache.meshes[b.mesh].index_count;
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
        transforms: &mut Vec<[[f32; 4]; 4]>,
        origins: &mut Vec<u32>,
        frustum: &Frustum,
        eye: Vec3,
        forward: Vec3,
    ) -> (DrawList, u32) {
        let mut keyed: Vec<(f32, u32, usize, u32)> = Vec::new();
        let mut culled = 0;
        for (b_index, b) in batches.iter().enumerate().filter(|(_, b)| b.transparent) {
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

    /// World bounds of every instance as (center, radius).
    fn instance_bounds(spheres: &[(Vec3, f32)]) -> (Vec3, f32) {
        let mut lo = Vec3::splat(f32::MAX);
        let mut hi = Vec3::splat(f32::MIN);
        for &(c, r) in spheres {
            lo = lo.min(c - Vec3::splat(r));
            hi = hi.max(c + Vec3::splat(r));
        }
        if lo.x > hi.x {
            return (Vec3::ZERO, 1.0);
        }
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

    /// Makes the lit, shadow and prepass pipelines this frame's batches need.
    fn ensure_pipelines(&mut self, batches: &[Batch], skinned: &[Batch], prepass: bool) {
        let all = batches
            .iter()
            .map(|b| (b, false))
            .chain(skinned.iter().map(|b| (b, true)));
        for (b, is_skinned) in all {
            if let Some(sh) = b.shader {
                let key = (sh, b.side, b.transparent, b.effects);
                if !self.cache.hook_pipelines.contains_key(&key) {
                    let p = pbr_pipeline(
                        &self.device,
                        (&self.pbr_layout, &self.pbr_skinned_layout),
                        &self.cache.shaders[sh],
                        (b.side, b.transparent, false, b.effects),
                        self.samples,
                    );
                    self.cache.hook_pipelines.insert(key, p);
                }
            }
            let key = (b.side, b.transparent, is_skinned, b.effects);
            if !self.pbr_pipelines.contains_key(&key) {
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
                let p =
                    shadow_pipeline(&self.device, &self.shadow_layouts, &self.shadow_shader, key);
                self.shadow_pipelines.insert(key, p);
            }
            if prepass
                && !b.transparent
                && !self.prepass_pipelines.contains_key(&(b.side, is_skinned))
            {
                let p = prepass_pipeline(
                    &self.device,
                    (&self.pbr_layout, &self.pbr_skinned_layout),
                    &self.pbr_shader,
                    b.side,
                    is_skinned,
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
        let joint_base = self.palette.joint_base(self.cache.morph_data.len());
        let mut order: Vec<usize> = (0..scene.skinned_instances.len()).collect();
        order.sort_by_key(|&i| {
            let s = &scene.skinned_instances[i];
            (s.mesh, s.material)
        });
        let mut batches: Vec<Batch> = Vec::new();
        let mut data = Vec::with_capacity(order.len());
        let mut spheres = Vec::with_capacity(order.len());
        for &i in &order {
            let s = &scene.skinned_instances[i];
            let (mesh, material) = (s.mesh.index(), s.material.index());
            if self.cache.meshes[mesh].index_count == 0 {
                continue;
            }
            match batches.last_mut() {
                Some(b) if b.mesh == mesh && b.material == material => b.count += 1,
                _ => {
                    let m = &self.cache.materials[material];
                    batches.push(Batch {
                        mesh,
                        material,
                        first: data.len() as u32,
                        count: 1,
                        side: m.side,
                        transparent: m.transparent,
                        dissolve: m.dissolve,
                        effects: m.effects,
                        shader: None,
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
                transform: s.transform.to_cols_array_2d(),
                palette: [
                    first as f32,
                    count as f32,
                    morph_first as f32,
                    targets as f32,
                ],
                morph0: [w[0], w[1], w[2], w[3]],
                morph1: [w[4], w[5], w[6], w[7]],
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
    fn cull_skinned(frame: &mut SkinnedFrame, frustum: &Frustum) -> (Vec<Batch>, u32) {
        let mut visible = Vec::new();
        let mut culled = 0;
        for b in &frame.batches {
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
        self.sync(scene)?;
        self.ensure_environment(&scene.environment.sky);
        let (mut batches, mut transforms, spheres, submission) = self.build_batches(scene)?;
        let mut origins = submission.clone();
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
        let (mut skinned_visible, skinned_culled) = Self::cull_skinned(&mut skinned, &view_frustum);
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
        let prepass = ao.is_some()
            || outline
            || dof.is_some()
            || motion_blur.is_some()
            || god_rays.is_some()
            || particle_frame.needs_depth
            // Water reads the depth below it.
            || !scene.water.is_empty();
        // Decals and screen-space AO need the effect shader on every batch.
        if ao.is_some() || !scene.decals.is_empty() {
            for b in batches.iter_mut().chain(skinned_batches.iter_mut()) {
                b.effects = true;
            }
            for b in skinned_visible.iter_mut() {
                b.effects = true;
            }
        }
        self.ensure_pipelines(&batches, &skinned_batches, prepass);
        self.particles
            .ensure_pipelines(&self.device, &particle_frame, self.samples);
        let fonts = &self.cache.fonts;
        let overlay_frame = match &scene.overlay {
            Some(o) => Some(overlay::build(
                o,
                |i| fonts.get(i).map(|f| &f.layout),
                self.cache.textures.len(),
            )?),
            None => None,
        };
        if let Some(of) = &overlay_frame {
            let textures: Vec<_> = overlay::run_textures(of).collect();
            for t in textures {
                self.sprite_group(t);
            }
        }
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
        let sprite_frame = sprites::build(
            scene,
            |i| fonts.get(i).map(|f| &f.layout),
            self.cache.textures.len(),
            &particle_frame.sprites,
            |layer| lit_range.is_some_and(|(lo, hi)| (lo..=hi).contains(&layer)),
            view,
            &layered,
        )?;
        for run in sprite_frame.runs.iter().chain(&sprite_frame.glow_runs) {
            if let RunSource::Sprite(_, t) = run.source {
                self.sprite_group(t);
            }
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
        if let Some(i) = lut_index
            && i >= self.cache.luts.len()
        {
            return Err(RenderError::InvalidHandle(format!(
                "colour grading lut {i}"
            )));
        }
        let glow = post_fx.glow.is_some() && !sprite_frame.glow_runs.is_empty();

        let (main_list, mut culled) = self.cull(
            &batches,
            &spheres,
            &mut transforms,
            &mut origins,
            &view_frustum,
            |b| !b.transparent,
        );
        let (blended_list, blended_culled) = self.sort_transparent(
            &batches,
            &spheres,
            &submission,
            &mut transforms,
            &mut origins,
            &view_frustum,
            camera.position,
            cam.forward,
        );
        culled += blended_culled + skinned_culled;

        // The directional light's cascades.
        let light = &scene.light;
        let to_light = light.direction_to_light.normalize_or(Vec3::Y);
        let shadow_size = self.info.shadow_map_size;
        let has_casters = !batches.is_empty() || !skinned_batches.is_empty();
        let cascades: Vec<Cascade> = if light.cast_shadows && has_casters {
            let bounds = match light.shadow_bounds {
                ShadowBounds::Sphere { center, radius } => (center, radius.max(1e-3)),
                ShadowBounds::Auto => {
                    let mut all = spheres.clone();
                    all.extend(scene.skinned_instances.iter().map(|s| {
                        let r = self.cache.meshes[s.mesh.index()]
                            .bounds
                            .map_or(1.0, |(_, e)| e.length() * 2.0);
                        (s.transform.w_axis.truncate(), r)
                    }));
                    Self::instance_bounds(&all)
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
            )
        } else {
            Vec::new()
        };
        let cascade_lists: Vec<DrawList> = cascades
            .iter()
            .map(|c| {
                self.cull(
                    &batches,
                    &spheres,
                    &mut transforms,
                    &mut origins,
                    &c.frustum,
                    |_| true,
                )
                .0
            })
            .collect();

        // Local lights and their shadow faces.
        let (lights_gpu, local_shadows) = frame::local_lights(&scene.lights, LOCAL_SHADOW_SIZE);
        let mut local_lists: Vec<Option<DrawList>> =
            (0..LOCAL_SHADOW_LAYERS).map(|_| None).collect();
        for ls in &local_shadows {
            for (face, (_, f)) in ls.faces.iter().enumerate() {
                let list = self
                    .cull(&batches, &spheres, &mut transforms, &mut origins, f, |_| {
                        true
                    })
                    .0;
                local_lists[ls.slot * 6 + face] = Some(list);
            }
        }

        // Upload instances, previous transforms, skinned data, strips and sprites.
        grow_and_write(
            &self.device,
            &self.queue,
            &mut self.instance_buffer,
            "instances",
            bytemuck::cast_slice(&transforms),
        );
        let object_motion = motion_blur.is_some()
            && !scene.previous_transforms.is_empty()
            && scene.previous_transforms.len() == scene.instances.len();
        if object_motion {
            let prev: Vec<[[f32; 4]; 4]> = origins
                .iter()
                .map(|&i| scene.previous_transforms[i as usize].to_cols_array_2d())
                .collect();
            grow_and_write(
                &self.device,
                &self.queue,
                &mut self.prev_instance_buffer,
                "previous instances",
                bytemuck::cast_slice(&prev),
            );
        }
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
                self.width,
                self.height,
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
                scene.lights.len().min(MAX_LIGHTS) as f32,
                1.0 / shadow_size as f32,
            ],
            view_forward: cam
                .forward
                .extend(1.0 / LOCAL_SHADOW_SIZE as f32)
                .to_array(),
            cascade_view_proj: [[[0.0; 4]; 4]; CASCADES],
            cascade_bias: [[0.0; 4]; CASCADES],
            lights: lights_gpu,
            local_view_proj: [[[0.0; 4]; 4]; LOCAL_SHADOW_LAYERS],
            atmosphere: atmosphere::AtmosphereGpu::new(
                scene.fog.as_ref(),
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
        };
        let mut shadow_slots = vec![bytemuck::Zeroable::zeroed(); SHADOW_SLOTS];
        for (i, c) in cascades.iter().enumerate() {
            uniform.cascade_view_proj[i] = c.view_proj.to_cols_array_2d();
            uniform.cascade_bias[i] = [c.normal_offset, c.depth_bias, c.split_far, 0.0];
            shadow_slots[i] = ShadowPassUniform {
                view_proj: c.view_proj.to_cols_array_2d(),
                light: [0.0; 4],
                pad: [0.0; 44],
            };
        }
        for ls in &local_shadows {
            for (face, (vp, _)) in ls.faces.iter().enumerate() {
                let layer = ls.slot * 6 + face;
                uniform.local_view_proj[layer] = vp.to_cols_array_2d();
                shadow_slots[CASCADES + layer] = ShadowPassUniform {
                    view_proj: vp.to_cols_array_2d(),
                    light: ls.position.extend(ls.range).to_array(),
                    pad: [0.0; 44],
                };
            }
        }
        self.queue
            .write_buffer(&self.frame_buffer, 0, bytemuck::bytes_of(&uniform));
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
            scene.tick,
        );
        let prev_view_proj = self.prev_view_proj.unwrap_or(cam.view_proj);
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
                passes.push(FramePass::LocalFace((ls.slot * 6 + face) as u32));
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
        passes.push(FramePass::Main);
        if glow {
            passes.push(FramePass::Glow);
        }
        if outline {
            passes.push(FramePass::Outline);
        }
        if taa {
            passes.push(FramePass::Taa);
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
        if god_rays.is_some() {
            passes.extend([FramePass::GodRaysMask, FramePass::GodRaysBlur]);
        }
        if auto_exposure {
            passes.push(FramePass::Meter);
            passes.extend((0..post::meter_levels() - 1).map(FramePass::Reduce));
        }
        if bloom {
            let levels = post::bloom_levels(self.width, self.height);
            passes.push(FramePass::BloomPrefilter);
            passes.extend((0..levels - 1).map(FramePass::BloomDown));
            passes.extend((0..levels - 1).rev().map(FramePass::BloomUp));
        }
        passes.push(FramePass::Tonemap);
        let lightmap_scale = scene
            .lighting_2d
            .map_or(0.5, |l| l.resolution.clamp(0.25, 1.0));
        let lut_view = match lut_index {
            Some(i) => &self.cache.luts[i],
            None => &self.post.identity_lut,
        };
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
                    water: !scene.water.is_empty(),
                    fluid_fields: fluids as u32,
                },
                &self.post,
                &self.persistent,
                (lut_index.map(|i| (scene.id(), i)), lut_view),
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

        let lights = scene.lights.len().min(MAX_LIGHTS) as u32;
        let mut stats = FrameStats {
            batches: batches.len() as u32,
            instances: scene.instances.len() as u32,
            culled,
            triangles: main_list.triangles + blended_list.triangles,
            lights,
            lights_dropped: scene.lights.len() as u32 - lights,
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
            decals,
            decals_dropped: scene.decals.len() as u32 - decals,
            lights_2d,
            occluders_2d,
            skinned_instances: scene.skinned_instances.len() as u32,
            joints: joint_matrices.len() as u32,
            ..Default::default()
        };
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
        let (_, frame_bind_group, sky_group) = self
            .environment
            .as_ref()
            .expect("environment generated above");
        let groups = self.fx_groups.as_ref().expect("made above");
        let (fx_group, sprite_fx_group, glow_fx_group) = (&groups.fx, &groups.sprite, &groups.glow);
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
        let water_draws = match self.graph.try_view("scene-copy") {
            Some(copy) if !scene.water.is_empty() => self.water.prepare(
                &self.device,
                &self.queue,
                &scene.water,
                (self.width, self.height),
                self.info.tier == Tier::Full,
                copy,
                scene_depth,
            ),
            _ => Vec::new(),
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        let palette_group = self.palette.bind_group.as_ref();
        let lit_skinned: Vec<&wgpu::RenderPipeline> = skinned_visible
            .iter()
            .map(|b| &self.pbr_pipelines[&(b.side, b.transparent, true, b.effects)])
            .collect();
        let shadow_skinned: Vec<&wgpu::RenderPipeline> = skinned_batches
            .iter()
            .map(|b| &self.shadow_pipelines[&(b.side, true, b.dissolve)])
            .collect();
        let prepass_skinned: Vec<&wgpu::RenderPipeline> = if prepass {
            skinned_visible
                .iter()
                .map(|b| {
                    self.prepass_pipelines
                        .get(&(b.side, true))
                        .unwrap_or(lit_skinned[0])
                })
                .collect()
        } else {
            Vec::new()
        };
        // Draws a list's ranges. Lit passes switch the pipeline when the
        // batch's (side, blending) changes and bind each batch's material;
        // shadow passes switch on the side and dissolve (with the material).
        let draw_list =
            |pass: &mut wgpu::RenderPass,
             list: &DrawList,
             lit: bool,
             current: &mut Option<(Side, bool, bool, Option<usize>)>| {
                let mut shadow_key: Option<ShadowKey> = None;
                for &(b, first, count) in &list.ranges {
                    let batch = &batches[b as usize];
                    if lit {
                        let key = (batch.side, batch.transparent, batch.effects, batch.shader);
                        if *current != Some(key) {
                            pass.set_pipeline(match batch.shader {
                                Some(sh) => {
                                    &self.cache.hook_pipelines
                                        [&(sh, batch.side, batch.transparent, batch.effects)]
                                }
                                None => {
                                    &self.pbr_pipelines
                                        [&(batch.side, batch.transparent, false, batch.effects)]
                                }
                            });
                            *current = Some(key);
                        }
                        pass.set_bind_group(
                            1,
                            &self.cache.materials[batch.material].bind_group,
                            &[],
                        );
                    } else {
                        let key = (batch.side, false, batch.dissolve);
                        if shadow_key != Some(key) {
                            pass.set_pipeline(&self.shadow_pipelines[&key]);
                            shadow_key = Some(key);
                        }
                        if batch.dissolve {
                            pass.set_bind_group(
                                1,
                                &self.cache.materials[batch.material].bind_group,
                                &[],
                            );
                        }
                    }
                    let mesh = &self.cache.meshes[batch.mesh];
                    pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                    // A buffer slice, not first_instance: WebGL2 has no base instance.
                    let start = first as u64 * INSTANCE_STRIDE;
                    pass.set_vertex_buffer(
                        1,
                        self.instance_buffer
                            .slice(start..start + count as u64 * INSTANCE_STRIDE),
                    );
                    pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..mesh.index_count, 0, 0..count);
                }
            };
        // The skinned batches (`transparent`: which of them) with the
        // pipelines `pipeline` picks; materials at group `material_group`.
        let draw_skinned = |pass: &mut wgpu::RenderPass,
                            list: &[Batch],
                            transparent: bool,
                            material_group: u32,
                            palette_at: u32,
                            pipelines: &[&wgpu::RenderPipeline]|
         -> u32 {
            let Some(palette) = palette_group else {
                return 0;
            };
            let mut draws = 0;
            for (batch, pipeline) in list.iter().zip(pipelines) {
                if batch.transparent != transparent {
                    continue;
                }
                pass.set_pipeline(pipeline);
                pass.set_bind_group(
                    material_group,
                    &self.cache.materials[batch.material].bind_group,
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
            draws
        };
        // One shadow pass: a cascade or a local light's face.
        let shadow_pass = |encoder: &mut wgpu::CommandEncoder,
                           color: &wgpu::TextureView,
                           depth: &wgpu::TextureView,
                           slot: usize,
                           list: &DrawList,
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
            draw_list(
                &mut pass,
                list,
                false,
                &mut Some((Side::Front, false, false, None)),
            );
            stats.draw_calls += list.ranges.len() as u32;
            stats.shadow_casters += list.instances;
            stats.shadow_passes += 1;
            // Skinned casters, unculled.
            let n = draw_skinned(&mut pass, &skinned_batches, false, 1, 2, &shadow_skinned);
            stats.draw_calls += n;
            stats.shadow_casters += skinned_batches
                .iter()
                .filter(|b| !b.transparent)
                .map(|b| b.count)
                .sum::<u32>();
        };
        for &pass in self.graph.order() {
            match pass {
                FramePass::Cascade(i) => shadow_pass(
                    &mut encoder,
                    &self.cascade_map.layers[i as usize],
                    &self.cascade_map.depth,
                    i as usize,
                    &cascade_lists[i as usize],
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
                                store: wgpu::StoreOp::Discard,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    p.set_bind_group(0, frame_bind_group, &[]);
                    p.set_bind_group(2, fx_group, &[]);
                    let prev = if object_motion {
                        &self.prev_instance_buffer
                    } else {
                        &self.instance_buffer
                    };
                    let mut current = None;
                    for &(b, first, count) in &main_list.ranges {
                        let batch = &batches[b as usize];
                        if current != Some(batch.side) {
                            p.set_pipeline(&self.prepass_pipelines[&(batch.side, false)]);
                            current = Some(batch.side);
                        }
                        p.set_bind_group(1, &self.cache.materials[batch.material].bind_group, &[]);
                        let mesh = &self.cache.meshes[batch.mesh];
                        let start = first as u64 * INSTANCE_STRIDE;
                        let end = start + count as u64 * INSTANCE_STRIDE;
                        p.set_vertex_buffer(0, mesh.vertices.slice(..));
                        p.set_vertex_buffer(1, self.instance_buffer.slice(start..end));
                        p.set_vertex_buffer(2, prev.slice(start..end));
                        p.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                        p.draw_indexed(0..mesh.index_count, 0, 0..count);
                        stats.prepass_draws += 1;
                    }
                    stats.prepass_draws +=
                        draw_skinned(&mut p, &skinned_visible, false, 1, 3, &prepass_skinned);
                    stats.draw_calls += stats.prepass_draws;
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
                FramePass::Light2d => {
                    let ambient = scene.lighting_2d.map_or(Vec3::ONE, |l| l.ambient);
                    self.lights_2d
                        .encode(&mut encoder, self.graph.view("lightmap"), ambient);
                    stats.draw_calls += (lights_2d > 0) as u32;
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
                    // With water the pass is split after the sky: the
                    // scene so far is copied for the water to refract and
                    // reflect, then the pass goes on over it.
                    let split = !water_draws.is_empty();
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
                        &mut Some((Side::Front, false, false, None)),
                    );
                    stats.draw_calls += main_list.ranges.len() as u32;
                    stats.draw_calls +=
                        draw_skinned(&mut pass, &skinned_visible, false, 1, 3, &lit_skinned);
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
                        encoder.copy_texture_to_texture(
                            hdr.texture().as_image_copy(),
                            self.graph.view("scene-copy").texture().as_image_copy(),
                            size,
                        );
                        pass = begin(&mut encoder, false);
                        pass.set_bind_group(0, frame_bind_group, &[]);
                        stats.draw_calls += self.water.draw(&mut pass, &water_draws);
                    }
                    // Transparent meshes, back to front over the opaque scene.
                    pass.set_bind_group(2, fx_group, &[]);
                    draw_list(&mut pass, &blended_list, true, &mut None);
                    stats.draw_calls += blended_list.ranges.len() as u32;
                    stats.draw_calls +=
                        draw_skinned(&mut pass, &skinned_visible, true, 1, 3, &lit_skinned);
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
                    // Sprites and text, blended over the scene in sorted order.
                    pass.set_bind_group(2, sprite_fx_group, &[]);
                    let mut current: Option<(bool, SpriteBlend)> = None;
                    for run in &sprite_frame.runs {
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
                        let (msdf, blend, group) = match run.source {
                            RunSource::Strip(..) | RunSource::Metaballs(_) => {
                                unreachable!("drawn above")
                            }
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
                        if current != Some((msdf, blend)) {
                            pass.set_pipeline(match (msdf, blend) {
                                (true, _) => &self.msdf_pipeline,
                                (false, SpriteBlend::Alpha) => &self.sprite_pipeline,
                                (false, SpriteBlend::Additive) => &self.sprite_additive_pipeline,
                            });
                            current = Some((msdf, blend));
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
                    pass.set_pipeline(&self.sprite_glow_pipeline);
                    pass.set_bind_group(0, frame_bind_group, &[]);
                    pass.set_bind_group(2, glow_fx_group, &[]);
                    for run in &sprite_frame.glow_runs {
                        let group = match run.source {
                            RunSource::Sprite(_, Some(t)) => self.cache.sprite_groups[t.index()]
                                .as_ref()
                                .expect("made before the pass"),
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
                post_pass => {
                    stats.draw_calls += self.graph.encode_post(
                        &mut encoder,
                        &self.post,
                        &self.persistent,
                        post_pass,
                        slot,
                        output,
                        gtao,
                    );
                }
            }
        }
        // The overlay: over the finished frame, in the output (or its sRGB view).
        if let Some(of) = overlay_frame.as_ref().filter(|f| !f.runs.is_empty()) {
            let pipelines = match overlay_view {
                Some(_) => self
                    .overlay
                    .linear
                    .as_ref()
                    .expect("an sRGB view's pipelines"),
                None => &self.overlay.output,
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("overlay"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: overlay_view.unwrap_or(output),
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
            let mut msdf_bound: Option<bool> = None;
            for run in &of.runs {
                let (msdf, group) = match run.source {
                    RunSource::Sprite(_, None) => (false, &self.white_sprite_group),
                    RunSource::Sprite(_, Some(t)) => (
                        false,
                        self.cache.sprite_groups[t.index()]
                            .as_ref()
                            .expect("made before the pass"),
                    ),
                    RunSource::Font(f) => (true, &self.cache.fonts[f.index()].bind_group),
                    RunSource::Strip(..) | RunSource::Metaballs(_) => {
                        unreachable!("the overlay has no strips or fluids")
                    }
                };
                if msdf_bound != Some(msdf) {
                    pass.set_pipeline(if msdf {
                        &pipelines.msdf
                    } else {
                        &pipelines.sprite
                    });
                    msdf_bound = Some(msdf);
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
        }
        self.queue.submit(Some(encoder.finish()));
        if let Some((frame, ..)) = surface_frame {
            self.queue.present(frame);
        }
        if taa {
            self.taa_history = Some(cam.view_proj);
        }
        self.prev_view_proj = Some(cam.view_proj);
        self.frame_index += 1;
        Ok(stats)
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

/// Writes `bytes` at the start of `buffer`, replacing it with a larger one first if needed.
fn grow_and_write(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffer: &mut wgpu::Buffer,
    label: &str,
    bytes: &[u8],
) {
    if bytes.len() as u64 > buffer.size() {
        *buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (bytes.len() as u64).next_power_of_two(),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    }
    if !bytes.is_empty() {
        queue.write_buffer(buffer, 0, bytes);
    }
}
