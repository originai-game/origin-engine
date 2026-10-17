//! Origin Engine renderer (`docs/design.md` §2.6, `crates/og-render/README.md`).
//!
//! A wgpu forward renderer: PBR metallic-roughness, one directional light with
//! a PCF shadow map, split-sum image-based lighting generated from a
//! procedural sky, ACES tonemapping. It runs natively (Metal, Vulkan; headless
//! for executors) and in the browser (WebGPU, or WebGL2 through wgpu's GL
//! backend) from the same code.
//!
//! The crate knows nothing about the ECS. Its whole input is a [`RenderScene`]:
//! resources with handles, instances submitted one by one, a camera, a light
//! and an environment. [`Renderer::render`] batches the instances by
//! (mesh, material) into one instanced draw per pass and reports
//! [`FrameStats`].
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

mod ibl;
mod mips;
mod scene;
pub mod shapes;

#[cfg(feature = "gltf")]
pub mod gltf;
#[cfg(not(target_arch = "wasm32"))]
mod headless;

#[cfg(not(target_arch = "wasm32"))]
pub use headless::render_to_rgba;
pub use scene::*;

use glam::Vec3;
use wgpu::util::DeviceExt;

/// The linear HDR colour target every lit pass draws into.
pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
/// Side of the directional light's shadow map, in texels.
pub const SHADOW_SIZE: u32 = 2048;
/// The format of the offscreen target when [`RenderDesc::output_format`] is unset.
pub const DEFAULT_OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

const FRAME_WGSL: &str = include_str!("../shaders/frame.wgsl");
const PBR_WGSL: &str = include_str!("../shaders/pbr.wgsl");
const SHADOW_WGSL: &str = include_str!("../shaders/shadow.wgsl");
const POST_WGSL: &str = include_str!("../shaders/post.wgsl");
const SKY_WGSL: &str = include_str!("../shaders/sky.wgsl");

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

/// How to create a [`Renderer`]. The default is a 1280x720 renderer on the
/// platform's primary API with an sRGB RGBA8 output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderDesc {
    pub width: u32,
    pub height: u32,
    pub backend: Backend,
    /// The output (offscreen, surface or caller view) format. `None` picks
    /// [`DEFAULT_OUTPUT_FORMAT`] offscreen and the surface's preference on a surface.
    pub output_format: Option<wgpu::TextureFormat>,
}

impl Default for RenderDesc {
    fn default() -> Self {
        RenderDesc {
            width: 1280,
            height: 720,
            backend: Backend::Auto,
            output_format: None,
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
    /// The renderer's own offscreen texture (read back with `Renderer::read_offscreen`).
    Offscreen,
    /// The surface the renderer was created with: acquired, drawn and presented.
    Surface,
    /// A caller's texture view of the renderer's size and output format.
    View(&'a wgpu::TextureView),
}

/// What one [`Renderer::render`] call submitted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameStats {
    /// Draw calls across every pass (shadow, main, sky, post).
    pub draw_calls: u32,
    /// Distinct (mesh, material) pairs drawn: one instanced draw per pass each.
    pub batches: u32,
    pub instances: u32,
    /// Triangles in the main pass.
    pub triangles: u64,
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
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    shadow_view_proj: [[f32; 4]; 4],
    view_pos: [f32; 4],
    light_dir: [f32; 4],
    light_color: [f32; 4],
    params: [f32; 4],
    inv_view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MaterialUniform {
    base_color: [f32; 4],
    mr: [f32; 4],
    emissive: [f32; 4],
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

/// GPU copies of one scene's resources, indexed like the scene's handles.
#[derive(Default)]
struct SceneCache {
    scene_id: u64,
    meshes: Vec<GpuMesh>,
    textures: Vec<wgpu::TextureView>,
    materials: Vec<GpuMaterial>,
}

/// Size-dependent targets.
struct Targets {
    hdr_view: wgpu::TextureView,
    depth_view: wgpu::TextureView,
    /// Read back natively; on the web, capture readback is still owed (M3).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    offscreen: wgpu::Texture,
    offscreen_view: wgpu::TextureView,
    post_bind_group: wgpu::BindGroup,
}

struct Batch {
    mesh: usize,
    material: usize,
    first: u32,
    count: u32,
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
    targets: Targets,
    shadow_view: wgpu::TextureView,
    shadow_color_view: wgpu::TextureView,
    shadow_sampler: wgpu::Sampler,
    material_sampler: wgpu::Sampler,
    post_sampler: wgpu::Sampler,
    frame_layout: wgpu::BindGroupLayout,
    material_layout: wgpu::BindGroupLayout,
    post_layout: wgpu::BindGroupLayout,
    camera_buffer: wgpu::Buffer,
    shadow_camera_buffer: wgpu::Buffer,
    post_buffer: wgpu::Buffer,
    shadow_bind_group: wgpu::BindGroup,
    pbr_pipeline: wgpu::RenderPipeline,
    shadow_pipeline: wgpu::RenderPipeline,
    post_pipeline: wgpu::RenderPipeline,
    sky_pipeline: wgpu::RenderPipeline,
    mips: mips::MipGenerator,
    /// The frame bind group (camera, shadow map, IBL) for the sky it was generated from.
    environment: Option<(ProceduralSky, wgpu::BindGroup)>,
    default_white: wgpu::TextureView,
    default_normal: wgpu::TextureView,
    instance_buffer: wgpu::Buffer,
    cache: SceneCache,
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

fn target_texture(
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
        let info = RenderInfo {
            adapter: adapter_info.name.clone(),
            backend: adapter_info.backend,
            device_type: adapter_info.device_type,
            tier,
        };
        log::info!(
            "og-render: {} ({:?}, {:?}, {:?})",
            info.adapter,
            info.backend,
            info.device_type,
            info.tier
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
                texture_entry(1, false, wgpu::TextureViewDimension::D2),
                sampler_entry(2, wgpu::SamplerBindingType::NonFiltering),
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
        let shadow_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow"),
            entries: &[uniform_entry(0, wgpu::ShaderStages::VERTEX)],
        });
        let post_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post"),
            entries: &[
                tex2d(0),
                sampler_entry(1, wgpu::SamplerBindingType::Filtering),
                uniform_entry(2, wgpu::ShaderStages::FRAGMENT),
            ],
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
        // Shadow depth is read with textureLoad and explicit compares: a plain
        // non-filtering sampler (WebGPU forbids filtering a depth texture).
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let post_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("post"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let attach_bind =
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let shadow_view = target_texture(
            &device,
            "shadow depth",
            SHADOW_SIZE,
            SHADOW_SIZE,
            DEPTH_FORMAT,
            attach_bind,
        )
        .create_view(&Default::default());
        let shadow_color_view = target_texture(
            &device,
            "shadow depth as colour",
            SHADOW_SIZE,
            SHADOW_SIZE,
            wgpu::TextureFormat::R32Float,
            attach_bind,
        )
        .create_view(&Default::default());

        // Uniforms.
        let uniform = |label: &str, size: u64| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let camera_buffer = uniform("camera", std::mem::size_of::<CameraUniform>() as u64);
        let shadow_camera_buffer = uniform("shadow camera", 64);
        let post_buffer = uniform("post", 16);
        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow"),
            layout: &shadow_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: shadow_camera_buffer.as_entire_binding(),
            }],
        });

        // Pipelines.
        let pbr_shader = shader(&device, "pbr", format!("{FRAME_WGSL}{PBR_WGSL}"));
        let shadow_shader = shader(&device, "shadow", SHADOW_WGSL.to_string());
        let post_shader = shader(&device, "post", POST_WGSL.to_string());
        let pbr_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pbr"),
            bind_group_layouts: &[Some(&frame_layout), Some(&material_layout)],
            immediate_size: 0,
        });
        let pbr_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pbr"),
            layout: Some(&pbr_pl),
            vertex: wgpu::VertexState {
                module: &pbr_shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Some(VERTEX_LAYOUT), Some(INSTANCE_LAYOUT)],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
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
                module: &pbr_shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: HDR_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let shadow_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow"),
            bind_group_layouts: &[Some(&shadow_layout)],
            immediate_size: 0,
        });
        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow"),
            layout: Some(&shadow_pl),
            vertex: wgpu::VertexState {
                module: &shadow_shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Some(VERTEX_LAYOUT), Some(INSTANCE_LAYOUT)],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Front),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
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
        let sky_shader = shader(&device, "sky", format!("{FRAME_WGSL}{SKY_WGSL}"));
        let sky_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky"),
            bind_group_layouts: &[Some(&frame_layout)],
            immediate_size: 0,
        });
        let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky"),
            layout: Some(&sky_pl),
            vertex: wgpu::VertexState {
                module: &sky_shader,
                entry_point: Some("vs_sky"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            // At the far plane, after the opaque batches: only uncovered pixels pass.
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &sky_shader,
                entry_point: Some("fs_sky"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: HDR_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let post_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("post"),
            bind_group_layouts: &[Some(&post_layout)],
            immediate_size: 0,
        });
        // A non-sRGB output (web canvases) gets the sRGB curve in the shader.
        let post_entry = if output_format.is_srgb() {
            "fs"
        } else {
            "fs_srgb_encode"
        };
        let post_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("post"),
            layout: Some(&post_pl),
            vertex: wgpu::VertexState {
                module: &post_shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &post_shader,
                entry_point: Some(post_entry),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let mips = mips::MipGenerator::new(&device);
        let default_white = solid_texture(&device, &queue, [255; 4]);
        let default_normal = solid_texture(&device, &queue, [128, 128, 255, 255]);
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instances"),
            size: INSTANCE_STRIDE * 1024,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let targets = Self::make_targets(
            &device,
            &post_layout,
            &post_sampler,
            &post_buffer,
            width,
            height,
            output_format,
        );

        Ok(Renderer {
            device,
            queue,
            info,
            width,
            height,
            output_format,
            surface,
            targets,
            shadow_view,
            shadow_color_view,
            shadow_sampler,
            material_sampler,
            post_sampler,
            frame_layout,
            material_layout,
            post_layout,
            camera_buffer,
            shadow_camera_buffer,
            post_buffer,
            shadow_bind_group,
            pbr_pipeline,
            shadow_pipeline,
            post_pipeline,
            sky_pipeline,
            mips,
            environment: None,
            default_white,
            default_normal,
            instance_buffer,
            cache: SceneCache::default(),
        })
    }

    fn make_targets(
        device: &wgpu::Device,
        post_layout: &wgpu::BindGroupLayout,
        post_sampler: &wgpu::Sampler,
        post_buffer: &wgpu::Buffer,
        width: u32,
        height: u32,
        output_format: wgpu::TextureFormat,
    ) -> Targets {
        let attach_bind =
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let hdr_view = target_texture(device, "hdr", width, height, HDR_FORMAT, attach_bind)
            .create_view(&Default::default());
        let depth_view = target_texture(
            device,
            "depth",
            width,
            height,
            DEPTH_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        )
        .create_view(&Default::default());
        let offscreen = target_texture(
            device,
            "offscreen",
            width,
            height,
            output_format,
            attach_bind | wgpu::TextureUsages::COPY_SRC,
        );
        let offscreen_view = offscreen.create_view(&Default::default());
        let post_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post"),
            layout: post_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(post_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: post_buffer.as_entire_binding(),
                },
            ],
        });
        Targets {
            hdr_view,
            depth_view,
            offscreen,
            offscreen_view,
            post_bind_group,
        }
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

    /// Resizes every size-dependent target (and the surface, if any).
    pub fn resize(&mut self, width: u32, height: u32) {
        let (width, height) = (width.max(1), height.max(1));
        if (width, height) == (self.width, self.height) {
            return;
        }
        self.width = width;
        self.height = height;
        self.targets = Self::make_targets(
            &self.device,
            &self.post_layout,
            &self.post_sampler,
            &self.post_buffer,
            width,
            height,
            self.output_format,
        );
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
        Ok(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("material"),
            layout: &self.material_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: factors.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&base),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&mr),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&normal),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&emissive),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.material_sampler),
                },
            ],
        }))
    }

    /// (Re)generates the image-based lighting when the sky changed.
    fn ensure_environment(&mut self, sky: &ProceduralSky) {
        if self.environment.as_ref().is_some_and(|(s, _)| s == sky) {
            return;
        }
        let ibl = ibl::Ibl::generate(&self.device, &self.queue, sky.sun_direction, sky.sun_color);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("frame"),
            layout: &self.frame_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.shadow_color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.shadow_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&ibl.irradiance),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&ibl.prefiltered),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&ibl.brdf_lut),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(&ibl.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(&ibl.sky),
                },
            ],
        });
        self.environment = Some((*sky, bind_group));
    }

    /// Sorts the instances into (mesh, material) batches and uploads their
    /// transforms into one buffer, batch after batch.
    fn build_batches(&mut self, scene: &RenderScene) -> Result<Vec<Batch>, RenderError> {
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
        let mut transforms: Vec<[[f32; 4]; 4]> = Vec::with_capacity(order.len());
        let mut batches: Vec<Batch> = Vec::new();
        for &i in &order {
            let inst = &scene.instances[i];
            let (mesh, material) = (inst.mesh.index(), inst.material.index());
            match batches.last_mut() {
                Some(b) if b.mesh == mesh && b.material == material => b.count += 1,
                _ => batches.push(Batch {
                    mesh,
                    material,
                    first: transforms.len() as u32,
                    count: 1,
                }),
            }
            transforms.push(inst.transform.to_cols_array_2d());
        }
        let bytes = transforms.len() as u64 * INSTANCE_STRIDE;
        if bytes > self.instance_buffer.size() {
            self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("instances"),
                size: bytes.next_power_of_two(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if !transforms.is_empty() {
            self.queue
                .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&transforms));
        }
        batches.retain(|b| self.cache.meshes[b.mesh].index_count > 0);
        Ok(batches)
    }

    /// World bounds of every instance as (center, radius).
    fn instance_bounds(&self, scene: &RenderScene) -> (Vec3, f32) {
        let mut lo = Vec3::splat(f32::MAX);
        let mut hi = Vec3::splat(f32::MIN);
        for inst in &scene.instances {
            let Some((c, e)) = self
                .cache
                .meshes
                .get(inst.mesh.index())
                .and_then(|m| m.bounds)
            else {
                continue;
            };
            let m = inst.transform;
            let center = m.transform_point3(c);
            let extent = m.x_axis.truncate().abs() * e.x
                + m.y_axis.truncate().abs() * e.y
                + m.z_axis.truncate().abs() * e.z;
            lo = lo.min(center - extent);
            hi = hi.max(center + extent);
        }
        if lo.x > hi.x {
            return (Vec3::ZERO, 1.0);
        }
        ((lo + hi) * 0.5, ((hi - lo).length() * 0.5).max(1e-3))
    }

    fn write_uniforms(&self, scene: &RenderScene) {
        let cam = &scene.camera;
        let view = glam::camera::rh::view::look_at_mat4(cam.position, cam.target, cam.up);
        let proj = glam::camera::rh::proj::directx::perspective(
            cam.fov_y_degrees.to_radians(),
            self.width as f32 / self.height as f32,
            cam.near,
            cam.far,
        );
        let view_proj = proj * view;
        let light = &scene.light;
        let to_light = light.direction_to_light.normalize_or(Vec3::Y);
        let (center, r) = match light.shadow_bounds {
            ShadowBounds::Sphere { center, radius } => (center, radius.max(1e-3)),
            ShadowBounds::Auto => self.instance_bounds(scene),
        };
        // An orthographic box around the bounds, seen from the light.
        let up = if to_light.y.abs() > 0.99 {
            Vec3::Z
        } else {
            Vec3::Y
        };
        let light_view =
            glam::camera::rh::view::look_at_mat4(center + to_light * r * 2.0, center, up);
        let light_proj = glam::camera::rh::proj::directx::orthographic(-r, r, -r, r, 0.1, r * 4.0);
        let shadow_view_proj = light_proj * light_view;
        let radiance = light.color * light.intensity;
        let uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            shadow_view_proj: shadow_view_proj.to_cols_array_2d(),
            view_pos: cam.position.extend(1.0).to_array(),
            light_dir: to_light.extend(0.0).to_array(),
            light_color: [radiance.x, radiance.y, radiance.z, 1.0 / SHADOW_SIZE as f32],
            params: [
                scene.environment.intensity,
                (ibl::PREFILTER_MIPS - 1) as f32,
                if light.cast_shadows { 1.0 } else { 0.0 },
                0.0,
            ],
            inv_view_proj: view_proj.inverse().to_cols_array_2d(),
        };
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&uniform));
        self.queue.write_buffer(
            &self.shadow_camera_buffer,
            0,
            bytemuck::bytes_of(&shadow_view_proj.to_cols_array_2d()),
        );
        self.queue.write_buffer(
            &self.post_buffer,
            0,
            bytemuck::bytes_of(&[cam.exposure, 0.0, 0.0, 0.0]),
        );
    }

    /// Draws one frame of `scene` into `target`.
    pub fn render(
        &mut self,
        scene: &RenderScene,
        target: &RenderTarget,
    ) -> Result<FrameStats, RenderError> {
        self.sync(scene)?;
        self.ensure_environment(&scene.environment.sky);
        let batches = self.build_batches(scene)?;
        self.write_uniforms(scene);

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

        let mut stats = FrameStats {
            batches: batches.len() as u32,
            instances: scene.instances.len() as u32,
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
        let draw_batches =
            |pass: &mut wgpu::RenderPass, with_materials: bool, stats: &mut FrameStats| {
                for b in &batches {
                    let mesh = &self.cache.meshes[b.mesh];
                    if with_materials {
                        pass.set_bind_group(1, &self.cache.materials[b.material].bind_group, &[]);
                    }
                    pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                    // A buffer slice, not first_instance: WebGL2 has no base instance.
                    let start = b.first as u64 * INSTANCE_STRIDE;
                    pass.set_vertex_buffer(
                        1,
                        self.instance_buffer
                            .slice(start..start + b.count as u64 * INSTANCE_STRIDE),
                    );
                    pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..mesh.index_count, 0, 0..b.count);
                    stats.draw_calls += 1;
                    if with_materials {
                        stats.triangles += (mesh.index_count / 3) as u64 * b.count as u64;
                    }
                }
            };
        if scene.light.cast_shadows {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.shadow_color_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.shadow_pipeline);
            pass.set_bind_group(0, &self.shadow_bind_group, &[]);
            draw_batches(&mut pass, false, &mut stats);
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
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.targets.hdr_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.targets.depth_view,
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
            pass.set_pipeline(&self.pbr_pipeline);
            pass.set_bind_group(0, frame_bind_group, &[]);
            draw_batches(&mut pass, true, &mut stats);
            if scene.environment.background == Background::Sky {
                pass.set_pipeline(&self.sky_pipeline);
                pass.draw(0..3, 0..1);
                stats.draw_calls += 1;
            }
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("post"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output,
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
            pass.set_pipeline(&self.post_pipeline);
            pass.set_bind_group(0, &self.targets.post_bind_group, &[]);
            pass.draw(0..3, 0..1);
            stats.draw_calls += 1;
        }
        self.queue.submit(Some(encoder.finish()));
        if let Some((frame, _)) = surface_frame {
            self.queue.present(frame);
        }
        Ok(stats)
    }
}
