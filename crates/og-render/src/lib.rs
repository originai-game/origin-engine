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

mod capture;
mod frame;
mod ibl;
mod mips;
mod post;
mod scene;
pub mod shapes;
mod sprites;

#[cfg(feature = "gltf")]
pub mod gltf;
#[cfg(not(target_arch = "wasm32"))]
mod headless;

#[cfg(not(target_arch = "wasm32"))]
pub use headless::render_to_rgba;
pub use scene::*;
/// The wgpu this crate is built on, for hosts that create surfaces or pass views.
pub use wgpu;

use frame::{Cascade, Frustum, LocalShadow, CASCADES, LOCAL_SHADOW_LAYERS};
use glam::{Mat4, Vec2, Vec3};
use sprites::{FontLayout, RunSource, SPRITE_LAYOUT, SPRITE_STRIDE};
use wgpu::util::DeviceExt;

/// The linear HDR colour target every lit pass draws into.
pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
/// Side of each of the directional light's three cascades, in texels.
pub const SHADOW_SIZE: u32 = 2048;
/// Side of each face of a local light's shadow map, in texels.
pub const LOCAL_SHADOW_SIZE: u32 = 512;
/// The format of the offscreen target when [`RenderDesc::output_format`] is unset.
pub const DEFAULT_OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
/// MSAA sample count when MSAA is on.
pub const MSAA_SAMPLES: u32 = 4;

const FRAME_WGSL: &str = include_str!("../shaders/frame.wgsl");
const PBR_WGSL: &str = include_str!("../shaders/pbr.wgsl");
const SHADOW_WGSL: &str = include_str!("../shaders/shadow.wgsl");
const SKY_WGSL: &str = include_str!("../shaders/sky.wgsl");
const SPRITE_WGSL: &str = include_str!("../shaders/sprite.wgsl");

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
/// platform's primary API with an sRGB RGBA8 output and no antialiasing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderDesc {
    pub width: u32,
    pub height: u32,
    pub backend: Backend,
    /// The output (offscreen, surface or caller view) format. `None` picks
    /// [`DEFAULT_OUTPUT_FORMAT`] offscreen and the surface's preference on a surface.
    pub output_format: Option<wgpu::TextureFormat>,
    pub antialias: Antialias,
}

impl Default for RenderDesc {
    fn default() -> Self {
        RenderDesc {
            width: 1280,
            height: 720,
            backend: Backend::Auto,
            output_format: None,
            antialias: Antialias::Off,
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
}

impl RenderInfo {
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
    /// Characters of [`RenderScene::texts`] missing from their font.
    pub missing_glyphs: u32,
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
struct MaterialUniform {
    base_color: [f32; 4],
    mr: [f32; 4],
    emissive: [f32; 4],
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
    /// Local-space bounds as (center, half extent).
    bounds: Option<(Vec3, Vec3)>,
}

struct GpuMaterial {
    revision: u64,
    bind_group: wgpu::BindGroup,
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
    textures: Vec<wgpu::TextureView>,
    materials: Vec<GpuMaterial>,
    /// Sprite bind groups per texture, made on first use.
    sprite_groups: Vec<Option<wgpu::BindGroup>>,
    fonts: Vec<GpuFont>,
}

/// One (mesh, material) run of the sorted instances.
struct Batch {
    mesh: usize,
    material: usize,
    first: u32,
    count: u32,
}

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
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: layers,
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
    surface: Option<(wgpu::Surface<'static>, wgpu::SurfaceConfiguration)>,
    samples: u32,
    targets: post::Targets,
    post: post::PostPasses,
    cascade_map: ShadowMap,
    local_map: ShadowMap,
    material_sampler: wgpu::Sampler,
    sprite_sampler: wgpu::Sampler,
    frame_layout: wgpu::BindGroupLayout,
    material_layout: wgpu::BindGroupLayout,
    sprite_layout: wgpu::BindGroupLayout,
    frame_buffer: wgpu::Buffer,
    shadow_buffer: wgpu::Buffer,
    shadow_bind_group: wgpu::BindGroup,
    pbr_pipeline: wgpu::RenderPipeline,
    shadow_pipeline: wgpu::RenderPipeline,
    sky_pipeline: wgpu::RenderPipeline,
    sprite_pipeline: wgpu::RenderPipeline,
    msdf_pipeline: wgpu::RenderPipeline,
    mips: mips::MipGenerator,
    /// The frame bind group (camera, shadow maps, IBL) for the sky it was generated from.
    environment: Option<(ProceduralSky, wgpu::BindGroup)>,
    default_white: wgpu::TextureView,
    default_normal: wgpu::TextureView,
    white_sprite_group: wgpu::BindGroup,
    instance_buffer: wgpu::Buffer,
    sprite_buffer: wgpu::Buffer,
    cache: SceneCache,
    /// Frames rendered so far.
    frame_index: u64,
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
struct HdrPipeline<'a> {
    label: &'a str,
    layout: &'a wgpu::PipelineLayout,
    module: &'a wgpu::ShaderModule,
    vs: &'a str,
    fs: &'a str,
    buffers: &'a [Option<wgpu::VertexBufferLayout<'a>>],
    cull: Option<wgpu::Face>,
    depth_write: bool,
    depth_compare: wgpu::CompareFunction,
    blend: Option<wgpu::BlendState>,
    samples: u32,
}

fn hdr_pipeline(device: &wgpu::Device, p: HdrPipeline) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(p.label),
        layout: Some(p.layout),
        vertex: wgpu::VertexState {
            module: p.module,
            entry_point: Some(p.vs),
            compilation_options: Default::default(),
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
            compilation_options: Default::default(),
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
        let format = match desc.output_format {
            Some(f) => f,
            None => caps
                .formats
                .iter()
                .copied()
                .find(|f| {
                    matches!(
                        f,
                        wgpu::TextureFormat::Bgra8Unorm
                            | wgpu::TextureFormat::Rgba8Unorm
                            | wgpu::TextureFormat::Bgra8UnormSrgb
                            | wgpu::TextureFormat::Rgba8UnormSrgb
                    )
                })
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
            view_formats: vec![],
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
        let info = RenderInfo {
            adapter: adapter_info.name.clone(),
            backend: adapter_info.backend,
            device_type: adapter_info.device_type,
            tier,
            antialias,
        };
        log::info!(
            "og-render: {} ({:?}, {:?}, {:?}, antialias {:?})",
            info.adapter,
            info.backend,
            info.device_type,
            info.tier,
            info.antialias
        );
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("og-render"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
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
                texture_entry(7, true, wgpu::TextureViewDimension::Cube),
            ],
        });
        let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::FRAGMENT),
                tex2d(1),
                tex2d(2),
                tex2d(3),
                tex2d(4),
                sampler_entry(5, wgpu::SamplerBindingType::Filtering),
            ],
        });
        let sprite_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite"),
            entries: &[
                tex2d(0),
                sampler_entry(1, wgpu::SamplerBindingType::Filtering),
                uniform_entry(2, wgpu::ShaderStages::FRAGMENT),
            ],
        });
        let shadow_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        let cascade_map = ShadowMap::new(&device, "cascades", SHADOW_SIZE, CASCADES as u32);
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
            layout: &shadow_layout,
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
        let pbr_shader = shader(&device, "pbr", format!("{FRAME_WGSL}{PBR_WGSL}"));
        let sky_shader = shader(&device, "sky", format!("{FRAME_WGSL}{SKY_WGSL}"));
        let sprite_shader = shader(&device, "sprite", format!("{FRAME_WGSL}{SPRITE_WGSL}"));
        let shadow_shader = shader(&device, "shadow", SHADOW_WGSL.to_string());
        let pipeline_layout = |label: &str, layouts: &[&wgpu::BindGroupLayout]| {
            let layouts: Vec<_> = layouts.iter().map(|l| Some(*l)).collect();
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &layouts,
                immediate_size: 0,
            })
        };
        let pbr_pl = pipeline_layout("pbr", &[&frame_layout, &material_layout]);
        let sky_pl = pipeline_layout("sky", &[&frame_layout]);
        let sprite_pl = pipeline_layout("sprite", &[&frame_layout, &sprite_layout]);
        let mesh_buffers = [Some(VERTEX_LAYOUT), Some(INSTANCE_LAYOUT)];
        let pbr_pipeline = hdr_pipeline(
            &device,
            HdrPipeline {
                label: "pbr",
                layout: &pbr_pl,
                module: &pbr_shader,
                vs: "vs",
                fs: "fs",
                buffers: &mesh_buffers,
                cull: Some(wgpu::Face::Back),
                depth_write: true,
                depth_compare: wgpu::CompareFunction::Less,
                blend: None,
                samples,
            },
        );
        // At the far plane, after the opaque batches: only uncovered pixels pass.
        let sky_pipeline = hdr_pipeline(
            &device,
            HdrPipeline {
                label: "sky",
                layout: &sky_pl,
                module: &sky_shader,
                vs: "vs_sky",
                fs: "fs_sky",
                buffers: &[],
                cull: None,
                depth_write: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                blend: None,
                samples,
            },
        );
        let sprite_buffers = [Some(SPRITE_LAYOUT)];
        let sprite_state = |label, fs| HdrPipeline {
            label,
            layout: &sprite_pl,
            module: &sprite_shader,
            vs: "vs",
            fs,
            buffers: &sprite_buffers,
            cull: None,
            depth_write: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            samples,
        };
        let sprite_pipeline = hdr_pipeline(&device, sprite_state("sprite", "fs"));
        let msdf_pipeline = hdr_pipeline(&device, sprite_state("msdf text", "fs_msdf"));
        let shadow_pl = pipeline_layout("shadow", &[&shadow_layout]);
        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow"),
            layout: Some(&shadow_pl),
            vertex: wgpu::VertexState {
                module: &shadow_shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &mesh_buffers,
            },
            // Back faces only: the stored depth is the far side of a closed
            // caster, so its lit side never compares against itself.
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Front),
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
                module: &shadow_shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R32Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::RED,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

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
        let sprite_buffer = vertex_buffer("sprites", SPRITE_STRIDE * 1024);
        let white_sprite_group = sprite_bind_group(
            &device,
            &sprite_layout,
            &default_white,
            &sprite_sampler,
            [0.0; 4],
        );
        let targets = post::Targets::new(
            &device,
            &post,
            width,
            height,
            output_format,
            samples,
            antialias,
        );

        Ok(Renderer {
            device,
            queue,
            info,
            width,
            height,
            output_format,
            surface,
            samples,
            targets,
            post,
            cascade_map,
            local_map,
            material_sampler,
            sprite_sampler,
            frame_layout,
            material_layout,
            sprite_layout,
            frame_buffer,
            shadow_buffer,
            shadow_bind_group,
            pbr_pipeline,
            shadow_pipeline,
            sky_pipeline,
            sprite_pipeline,
            msdf_pipeline,
            mips,
            environment: None,
            default_white,
            default_normal,
            white_sprite_group,
            instance_buffer,
            sprite_buffer,
            cache: SceneCache::default(),
            frame_index: 0,
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

    /// Resizes every size-dependent target (and the surface, if any). TAA
    /// history restarts.
    pub fn resize(&mut self, width: u32, height: u32) {
        let (width, height) = (width.max(1), height.max(1));
        if (width, height) == (self.width, self.height) {
            return;
        }
        self.width = width;
        self.height = height;
        self.targets = post::Targets::new(
            &self.device,
            &self.post,
            width,
            height,
            self.output_format,
            self.samples,
            self.info.antialias,
        );
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
        for mesh in &scene.meshes()[self.cache.meshes.len()..] {
            let gpu = self.upload_mesh(mesh);
            self.cache.meshes.push(gpu);
        }
        if self.cache.textures.len() < scene.textures().len() {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("texture upload"),
                });
            for texture in &scene.textures()[self.cache.textures.len()..] {
                let view = self.upload_texture(texture, &mut encoder)?;
                self.cache.textures.push(view);
                self.cache.sprite_groups.push(None);
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
            let gpu = GpuMaterial {
                revision,
                bind_group: self.material_bind_group(i, material)?,
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
            let (Some(view), Some(data)) =
                (self.cache.textures.get(atlas), scene.textures().get(atlas))
            else {
                return Err(RenderError::InvalidHandle(format!(
                    "font {index}: atlas texture {atlas}"
                )));
            };
            let atlas_size = Vec2::new(data.width as f32, data.height as f32);
            let bind_group = sprite_bind_group(
                &self.device,
                &self.sprite_layout,
                view,
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
                usage: wgpu::BufferUsages::VERTEX,
            });
        let indices = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("indices"),
                contents: &bytes(bytemuck::cast_slice(&mesh.indices)),
                usage: wgpu::BufferUsages::INDEX,
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

    fn material_bind_group(
        &self,
        index: usize,
        m: &Material,
    ) -> Result<wgpu::BindGroup, RenderError> {
        let view = |t: Option<TextureHandle>,
                    fallback: &wgpu::TextureView|
         -> Result<wgpu::TextureView, RenderError> {
            match t {
                None => Ok(fallback.clone()),
                Some(h) => self.cache.textures.get(h.index()).cloned().ok_or_else(|| {
                    RenderError::InvalidHandle(format!(
                        "material {index} uses texture {}",
                        h.index()
                    ))
                }),
            }
        };
        let base = view(m.base_color_texture, &self.default_white)?;
        let mr = view(m.metallic_roughness_texture, &self.default_white)?;
        let normal = view(m.normal_texture, &self.default_normal)?;
        let emissive = view(m.emissive_texture, &self.default_white)?;
        let cutoff = match m.alpha_mode {
            AlphaMode::Opaque => -1.0,
            AlphaMode::Mask { cutoff } => cutoff,
        };
        let has_normal = m.normal_texture.is_some();
        let uniform = MaterialUniform {
            base_color: m.base_color,
            mr: [
                m.metallic,
                m.roughness,
                if has_normal { m.normal_scale } else { 1.0 },
                if has_normal { 1.0 } else { 0.0 },
            ],
            emissive: [m.emissive[0], m.emissive[1], m.emissive[2], cutoff],
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
            wgpu::BindingResource::TextureView(&mr),
            wgpu::BindingResource::TextureView(&normal),
            wgpu::BindingResource::TextureView(&emissive),
            wgpu::BindingResource::Sampler(&self.material_sampler),
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
        if self.environment.as_ref().is_some_and(|(s, _)| s == sky) {
            return;
        }
        let ibl = ibl::Ibl::generate(&self.device, &self.queue, sky.sun_direction, sky.sun_color);
        let entries = [
            self.frame_buffer.as_entire_binding(),
            wgpu::BindingResource::TextureView(&self.cascade_map.array_view),
            wgpu::BindingResource::TextureView(&self.local_map.array_view),
            wgpu::BindingResource::TextureView(&ibl.irradiance),
            wgpu::BindingResource::TextureView(&ibl.prefiltered),
            wgpu::BindingResource::TextureView(&ibl.brdf_lut),
            wgpu::BindingResource::Sampler(&ibl.sampler),
            wgpu::BindingResource::TextureView(&ibl.sky),
        ];
        let group = bind_group(&self.device, "frame", &self.frame_layout, &entries);
        self.environment = Some((*sky, group));
    }

    /// Sorts the instances into (mesh, material) batches; returns the batches,
    /// the sorted transforms and each sorted instance's world bounding sphere.
    #[allow(clippy::type_complexity)]
    fn build_batches(
        &self,
        scene: &RenderScene,
    ) -> Result<(Vec<Batch>, Vec<[[f32; 4]; 4]>, Vec<(Vec3, f32)>), RenderError> {
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
                _ => batches.push(Batch {
                    mesh,
                    material,
                    first: transforms.len() as u32,
                    count: 1,
                }),
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
        }
        Ok((batches, transforms, spheres))
    }

    /// The draw ranges of one pass: per batch, every instance whose sphere
    /// touches `frustum`. A batch that loses less than a tenth of its
    /// instances is drawn whole (cheaper than copying its transforms);
    /// otherwise its visible instances are appended to `transforms`.
    fn cull(
        &self,
        batches: &[Batch],
        spheres: &[(Vec3, f32)],
        transforms: &mut Vec<[[f32; 4]; 4]>,
        frustum: &Frustum,
    ) -> (DrawList, u32) {
        let mut list = DrawList::default();
        let mut culled = 0;
        let mut visible = Vec::new();
        for (b_index, b) in batches.iter().enumerate() {
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
                }
                (first, visible.len() as u32)
            };
            list.ranges.push((b_index as u32, first, count));
            list.instances += count;
            list.triangles += (index_count / 3) as u64 * count as u64;
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
        let slot = &mut self.cache.sprite_groups[t.index()];
        if slot.is_none() {
            *slot = Some(sprite_bind_group(
                &self.device,
                &self.sprite_layout,
                &self.cache.textures[t.index()],
                &self.sprite_sampler,
                [0.0; 4],
            ));
        }
        slot.as_ref().expect("made above")
    }

    /// Draws one frame of `scene` into `target`.
    pub fn render(
        &mut self,
        scene: &RenderScene,
        target: &RenderTarget,
    ) -> Result<FrameStats, RenderError> {
        self.sync(scene)?;
        self.ensure_environment(&scene.environment.sky);
        let (batches, mut transforms, spheres) = self.build_batches(scene)?;
        let fonts = &self.cache.fonts;
        let sprite_frame = sprites::build(
            scene,
            |i| fonts.get(i).map(|f| &f.layout),
            self.cache.textures.len(),
        )?;
        for run in &sprite_frame.runs {
            if let RunSource::Sprite(t) = run.source {
                self.sprite_group(t);
            }
        }

        // Camera, jitter, culling.
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
        let cam = frame::camera_frame(&scene.camera, aspect, jitter);
        let (main_list, culled) = self.cull(
            &batches,
            &spheres,
            &mut transforms,
            &Frustum::from_view_proj(cam.view_proj),
        );

        // The directional light's cascades.
        let light = &scene.light;
        let to_light = light.direction_to_light.normalize_or(Vec3::Y);
        let cascades: Vec<Cascade> = if light.cast_shadows && !batches.is_empty() {
            let bounds = match light.shadow_bounds {
                ShadowBounds::Sphere { center, radius } => (center, radius.max(1e-3)),
                ShadowBounds::Auto => Self::instance_bounds(&spheres),
            };
            frame::fit_cascades(
                &scene.camera,
                &cam,
                aspect,
                to_light,
                bounds,
                light.max_shadow_distance,
                SHADOW_SIZE,
            )
        } else {
            Vec::new()
        };
        let cascade_lists: Vec<DrawList> = cascades
            .iter()
            .map(|c| self.cull(&batches, &spheres, &mut transforms, &c.frustum).0)
            .collect();

        // Local lights and their shadow faces.
        let (lights_gpu, local_shadows) = frame::local_lights(&scene.lights, LOCAL_SHADOW_SIZE);
        let local_lists: Vec<Vec<DrawList>> = local_shadows
            .iter()
            .map(|ls: &LocalShadow| {
                ls.faces
                    .iter()
                    .map(|(_, f)| self.cull(&batches, &spheres, &mut transforms, f).0)
                    .collect()
            })
            .collect();

        // Upload instances and sprites.
        grow_and_write(
            &self.device,
            &self.queue,
            &mut self.instance_buffer,
            "instances",
            bytemuck::cast_slice(&transforms),
        );
        grow_and_write(
            &self.device,
            &self.queue,
            &mut self.sprite_buffer,
            "sprites",
            bytemuck::cast_slice(&sprite_frame.instances),
        );

        // Uniforms.
        let mut uniform = frame::FrameUniform {
            view_proj: cam.jittered_view_proj.to_cols_array_2d(),
            inv_view_proj: cam.jittered_view_proj.inverse().to_cols_array_2d(),
            view_pos: scene
                .camera
                .position
                .extend(if cam.orthographic { 1.0 } else { 0.0 })
                .to_array(),
            light_dir: to_light.extend(cascades.len() as f32).to_array(),
            light_color: (light.color * light.intensity)
                .extend(match scene.debug {
                    DebugView::None => 0.0,
                    DebugView::Cascades => 1.0,
                })
                .to_array(),
            params: [
                scene.environment.intensity,
                (ibl::PREFILTER_MIPS - 1) as f32,
                scene.lights.len().min(MAX_LIGHTS) as f32,
                1.0 / SHADOW_SIZE as f32,
            ],
            view_forward: cam
                .forward
                .extend(1.0 / LOCAL_SHADOW_SIZE as f32)
                .to_array(),
            cascade_view_proj: [[[0.0; 4]; 4]; CASCADES],
            cascade_bias: [[0.0; 4]; CASCADES],
            lights: lights_gpu,
            local_view_proj: [[[0.0; 4]; 4]; LOCAL_SHADOW_LAYERS],
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
        self.post
            .write_uniform(&self.queue, scene.camera.exposure, &scene.post);
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
                        Some((t, view))
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
            RenderTarget::Offscreen => &self.targets.offscreen_view,
            RenderTarget::View(v) => v,
            RenderTarget::Surface => &surface_frame.as_ref().expect("acquired above").1,
        };

        let lights = scene.lights.len().min(MAX_LIGHTS) as u32;
        let mut stats = FrameStats {
            batches: batches.len() as u32,
            instances: scene.instances.len() as u32,
            culled,
            triangles: main_list.triangles,
            lights,
            lights_dropped: scene.lights.len() as u32 - lights,
            sprites: sprite_frame.sprites,
            glyphs: sprite_frame.glyphs,
            sprite_draws: sprite_frame.runs.len() as u32,
            missing_glyphs: sprite_frame.missing_glyphs,
            frame: self.frame_index,
            ..Default::default()
        };
        let frame_bind_group = &self
            .environment
            .as_ref()
            .expect("environment generated above")
            .1;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        let draw_list = |pass: &mut wgpu::RenderPass, list: &DrawList, with_materials: bool| {
            for &(b, first, count) in &list.ranges {
                let batch = &batches[b as usize];
                let mesh = &self.cache.meshes[batch.mesh];
                if with_materials {
                    pass.set_bind_group(1, &self.cache.materials[batch.material].bind_group, &[]);
                }
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
        // Shadow passes: the cascades, then each shadowed local light's faces.
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
            pass.set_pipeline(&self.shadow_pipeline);
            pass.set_bind_group(
                0,
                &self.shadow_bind_group,
                &[(slot as u64 * SHADOW_SLOT_STRIDE) as u32],
            );
            draw_list(&mut pass, list, false);
            stats.draw_calls += list.ranges.len() as u32;
            stats.shadow_casters += list.instances;
            stats.shadow_passes += 1;
        };
        for (i, list) in cascade_lists.iter().enumerate() {
            shadow_pass(
                &mut encoder,
                &self.cascade_map.layers[i],
                &self.cascade_map.depth,
                i,
                list,
                &mut stats,
            );
        }
        for (ls, lists) in local_shadows.iter().zip(&local_lists) {
            for (face, list) in lists.iter().enumerate() {
                let layer = ls.slot * 6 + face;
                shadow_pass(
                    &mut encoder,
                    &self.local_map.layers[layer],
                    &self.local_map.depth,
                    CASCADES + layer,
                    list,
                    &mut stats,
                );
            }
        }
        {
            let clear = match scene.environment.background {
                Background::Color(c) => wgpu::Color {
                    r: c.x as f64,
                    g: c.y as f64,
                    b: c.z as f64,
                    a: 1.0,
                },
                Background::Sky => wgpu::Color::BLACK,
            };
            let (view, resolve_target, store) = match &self.targets.hdr_msaa {
                Some(msaa) => (msaa, Some(&self.targets.hdr), wgpu::StoreOp::Discard),
                None => (&self.targets.hdr, None, wgpu::StoreOp::Store),
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.targets.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        // TAA reads the depth to reproject the history.
                        store: if taa {
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
            pass.set_pipeline(&self.pbr_pipeline);
            pass.set_bind_group(0, frame_bind_group, &[]);
            draw_list(&mut pass, &main_list, true);
            stats.draw_calls += main_list.ranges.len() as u32;
            if scene.environment.background == Background::Sky {
                pass.set_pipeline(&self.sky_pipeline);
                pass.draw(0..3, 0..1);
                stats.draw_calls += 1;
            }
            // Sprites and text, blended over the scene in sorted order.
            let mut current: Option<bool> = None;
            for run in &sprite_frame.runs {
                let (msdf, group) = match run.source {
                    RunSource::Sprite(None) => (false, &self.white_sprite_group),
                    RunSource::Sprite(Some(t)) => (
                        false,
                        self.cache.sprite_groups[t.index()]
                            .as_ref()
                            .expect("made before the pass"),
                    ),
                    RunSource::Font(f) => (true, &self.cache.fonts[f.index()].bind_group),
                };
                if current != Some(msdf) {
                    pass.set_pipeline(if msdf {
                        &self.msdf_pipeline
                    } else {
                        &self.sprite_pipeline
                    });
                    current = Some(msdf);
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
        let auto_exposure = matches!(scene.camera.exposure, Exposure::Auto { .. });
        stats.draw_calls += self.targets.encode(
            &mut encoder,
            &self.post,
            &post::PostFrame {
                slot,
                taa,
                auto_exposure,
                bloom: scene.post.bloom.is_some_and(|b| b.intensity > 0.0),
            },
            output,
        );
        self.queue.submit(Some(encoder.finish()));
        if let Some((frame, _)) = surface_frame {
            self.queue.present(frame);
        }
        if taa {
            self.taa_history = Some(cam.view_proj);
        }
        self.frame_index += 1;
        Ok(stats)
    }

    /// Forgets the TAA history: the next frame starts from its own samples
    /// (call it on a camera cut).
    pub fn reset_history(&mut self) {
        self.taa_history = None;
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
