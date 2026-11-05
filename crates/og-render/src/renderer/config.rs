//! Renderer configuration, capabilities and frame statistics.

use super::*;

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
/// Everything that can go wrong, as values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderError {
    /// No GPU adapter for the requested backend.
    NoAdapter(String),
    /// The adapter refused to create a device.
    NoDevice(String),
    /// The surface could not be configured or acquired.
    Surface(String),
    /// No drawable while the native window is hidden or occluded. No frame
    /// was presented; wait for visibility before retrying, without reconfiguring.
    SurfaceOccluded,
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
    /// The GPU device was lost (`Renderer::lost`): no frame draws on this
    /// renderer again; make a new one with `Renderer::recreate`.
    DeviceLost(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::NoAdapter(m) => write!(f, "no GPU adapter: {m}"),
            RenderError::NoDevice(m) => write!(f, "no GPU device: {m}"),
            RenderError::Surface(m) => write!(f, "surface: {m}"),
            RenderError::SurfaceOccluded => write!(f, "surface is occluded; no frame presented"),
            RenderError::InvalidHandle(m) => write!(f, "invalid handle: {m}"),
            RenderError::Readback(m) => write!(f, "readback: {m}"),
            RenderError::Unsupported(m) => write!(f, "unsupported: {m}"),
            RenderError::InvalidData(m) => write!(f, "invalid data: {m}"),
            RenderError::Graph(m) => write!(f, "{m}"),
            RenderError::DeviceLost(m) => write!(f, "the GPU device was lost ({m})"),
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
    ///
    /// Except on DX12, where that pass is also what makes an indirect draw's
    /// `@builtin(vertex_index)` and `@builtin(instance_index)` count from its
    /// `base_vertex` and `first_instance`. D3D12's SV_VertexID and
    /// SV_InstanceID never include them, so wgpu-hal passes them as root
    /// constants, and only through the command signatures wgpu-core asks for
    /// when the pass is on (wgpu-core 30 `PipelineLayoutFlags::
    /// INDIRECT_BUILTIN_UPDATE`; without them `draw_indexed_indirect`
    /// writes 0). Without the pass, grass's LOD 1 and 2 blades, whose
    /// draws start at `l * GRASS_LOD_STRIDE`, drew as LOD 0, and the
    /// grass past LOD 0's ring was missing on DX12 (studio-win, 2026-11-05;
    /// tests/evidence/compat/windows/native/).
    pub fn instance(self) -> wgpu::Instance {
        let mut flags = wgpu::InstanceFlags::from_build_config();
        if !self.backends().contains(wgpu::Backends::DX12) {
            flags.remove(wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL);
        }
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
/// it to (never `Tier`). Every backend (WebGPU, Metal, Vulkan, DX12) has
/// all antialiasing methods.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Antialias {
    /// None: every pixel is one sample.
    #[default]
    Off,
    /// The engine's default method: TAA.
    Tier,
    /// 4x MSAA (FXAA where the HDR format cannot be multisampled). Sharp
    /// sprites and text; no temporal blur.
    Msaa4,
    /// Temporal antialiasing: a sub-pixel jitter per frame and a
    /// reprojected, neighbourhood-clamped history. Converges over ~8 frames.
    /// [`RenderDesc::taa_history`] and [`RenderDesc::taa_sharpen`] refine it.
    Taa,
    /// FXAA 3.11 (quality preset 12) over the tonemapped image, before the
    /// overlay: one full-screen pass, no extra samples and no history.
    /// Softer than MSAA on texture detail; nothing to converge.
    Fxaa,
    /// SMAA 1x high preset: perceptual luma edges, area/search weights,
    /// neighbourhood blending before the overlay. No jitter or history.
    Smaa,
    /// SMAA temporal 2x: two morphologically filtered, jittered frames,
    /// reprojected with TAA's motion vectors. Exclusive of TAA/TAAU.
    SmaaT2x,
}

impl Antialias {
    pub(crate) fn temporal(self) -> bool {
        matches!(self, Self::Taa | Self::SmaaT2x)
    }
}

/// The temporal resolve under [`Antialias::Taa`] ([`crate::RenderQuality::upscaler`]):
/// how the jittered frame, drawn at the render scale or at the output's
/// size, becomes the output. Both read the same jitter, depth, motion
/// vectors and OIT revealage, and write the same history, so switching
/// between them costs one frame of history and nothing else. A frame
/// supersampled (render scale above 1) resolves with TAAU whatever this
/// says. Ordered by cost, the cheaper first ([`crate::RenderQuality::capped`]
/// takes the lesser).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Upscaler {
    /// The engine's TAA and TAAU (taa.wgsl): one fullscreen pass, a 3x3
    /// YCoCg clamp, a jitter-weighted sample per output pixel.
    #[default]
    Taau,
    /// AMD FidelityFX Super Resolution 3.1's upscaler (post/fsr.rs, MIT):
    /// eight compute passes; Lanczos reconstruction, depth-based
    /// disocclusion, thin-feature locks, shading-change and luma-instability
    /// detection and the OIT revealage as its reactive mask; RCAS for
    /// [`RenderDesc::taa_sharpen`]. Core WebGPU only (no fp16, no subgroups),
    /// so every host runs the same shaders.
    Fsr3,
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
/// platform's primary API with an sRGB RGBA8 output, no antialiasing, up to
/// four 2048² shadow cascades (three drawn at the default quality) and BC/ASTC/ETC2 texture compression where the GPU
/// has it. [`RenderDesc::mobile`] is the preset for phones.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderDesc {
    pub width: u32,
    pub height: u32,
    pub backend: Backend,
    /// The output (offscreen, surface or caller view) format. `None` picks
    /// [`DEFAULT_OUTPUT_FORMAT`] offscreen and the surface's preference on a surface.
    pub output_format: Option<wgpu::TextureFormat>,
    /// Opt into an extended-range float surface where supported; otherwise SDR.
    pub hdr: bool,
    /// HDR paper white in nits; zero means 203. Relative on EDR/web.
    pub paper_white_nits: f32,
    /// HDR display peak in nits; zero means 1000, at least paper white.
    pub peak_nits: f32,
    pub antialias: Antialias,
    /// The side of each directional light cascade in texels (clamped to
    /// 64..=the device's limit). 2048 by default; 1024 is the mobile
    /// recommendation (each cascade costs `size² × 4` bytes, plus one depth
    /// buffer of the same size).
    pub shadow_map_size: u32,
    /// The most directional light cascades a frame may draw, 1 to
    /// [`MAX_CASCADES`] (4 by default); each frame draws its quality's
    /// [`crate::RenderQuality::shadow_cascades`] within it (fewer cover the
    /// same distance with coarser near shadows).
    pub shadow_cascades: u32,
    /// Which compressed texture family [`RenderInfo::texture_formats`] picks.
    pub texture_compression: TextureCompression,
    /// Where particles simulate: in compute, or on the CPU
    /// ([`ParticleSimulation::Cpu`]).
    pub particles: ParticleSimulation,
    /// Where instances are culled: in compute with indirect draws for large
    /// scenes (frustum and occlusion), or on the CPU ([`Culling`]).
    pub culling: Culling,
    /// Adapter features the renderer treats as absent, so a device that
    /// lacks them is reproducible on one that has them (the fallbacks'
    /// tests: `INDIRECT_FIRST_INSTANCE` takes crowds off the GPU-driven
    /// path, [`RenderInfo::gpu_crowds`]). Empty by default.
    pub without_features: wgpu::Features,
    /// Every GPU-culled indirect draw reads its args from a buffer of its
    /// own, as on WebKit, where it always does
    /// ([`RenderInfo::indirect_per_draw`]): reproduces that path on any
    /// device. False by default.
    pub indirect_per_draw: bool,
    /// How point lights' shadows are stored: cube faces, or dual-paraboloid
    /// hemispheres (two passes instead of six; [`PointShadows`]). `Auto`:
    /// cube.
    pub point_shadows: PointShadows,
    /// With TAA: how the history is resampled ([`TaaHistory`]).
    pub taa_history: TaaHistory,
    /// With TAA: a sharpening pass after it, 0 (off, the default) to 1: a
    /// clamped unsharp mask over the 4 neighbours that restores what the
    /// history's blend softened, never past the neighbourhood's range.
    pub taa_sharpen: f32,
    /// How volumetric fog is drawn: `None` is the froxel grid (a compute
    /// pass); `Some(Raymarch)` marches each pixel's ray in the fragment
    /// shader instead ([`VolumetricMethod::Raymarch`]).
    pub volumetric: Option<VolumetricMethod>,
    /// How many water surfaces draw a planar reflection in one frame, 1
    /// (the default) to [`MAX_PLANAR_REFLECTIONS`]. 1: the first surface
    /// that asks (a grid surface first) mirrors the scene; the others reflect
    /// as without one, and the lit pipelines' effect group is the four
    /// bindings it has always been. 2: a second surface in view mirrors the
    /// scene in a pass of its own, and material shaders read the first
    /// mirror (`planar_reflection`): one more texture in every lit
    /// pipeline, which pushes the
    /// lightmapped pipelines' lightmap pages past sixteen textures, so they
    /// take an effect group without it ([`RenderInfo::planar_reflections`]
    /// says what the renderer was made with).
    pub planar_reflections: u32,
    /// How the main camera's blended surfaces are composited: weighted
    /// blended order-independent transparency (the default), or the
    /// per-pixel linked lists where the device can write storage from
    /// fragment shaders ([`RenderInfo::transparency`] says which). See
    /// `oit.rs`.
    pub transparency: Transparency,
    /// With the linked lists: the fragments a pixel keeps and sorts
    /// exactly (0: [`DEFAULT_TRANSPARENCY_LAYERS`], at most
    /// [`MAX_TRANSPARENCY_LAYERS`]), as far as the device's largest storage
    /// buffer holds them for every pixel; a pixel with more is resolved
    /// weighted ([`FrameStats::transparency_fallback_pixels`]).
    pub transparency_layers: u32,
    /// Ask the adapter for the low-power GPU (a laptop's integrated one)
    /// instead of the high-performance one: the player's power preference
    /// (og.render.setDisplay's `powerPreference`), which only a new
    /// renderer takes (an adapter is chosen once).
    pub low_power: bool,
    /// Ask for timestamp queries where the adapter has them, in a `publish`
    /// build too: og.motion measures its compute slices with them (the
    /// engine sets it for a game whose bundle declares og.motion). Outside
    /// `publish` they are asked for anyway, for the GPU pass timing.
    pub timestamps: bool,
}

/// The most planar reflections one frame draws
/// ([`RenderDesc::planar_reflections`]).
pub const MAX_PLANAR_REFLECTIONS: u32 = 2;

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
    /// Always on the CPU (frustum only).
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
            hdr: false,
            paper_white_nits: 0.0,
            peak_nits: 0.0,
            antialias: Antialias::Off,
            shadow_map_size: SHADOW_SIZE,
            shadow_cascades: MAX_CASCADES,
            texture_compression: TextureCompression::Auto,
            particles: ParticleSimulation::Auto,
            culling: Culling::Auto,
            without_features: wgpu::Features::empty(),
            indirect_per_draw: false,
            point_shadows: PointShadows::Auto,
            taa_history: TaaHistory::Bilinear,
            taa_sharpen: 0.0,
            volumetric: None,
            planar_reflections: 1,
            transparency: Transparency::Weighted,
            transparency_layers: 0,
            low_power: false,
            timestamps: false,
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

/// What the renderer runs on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderInfo {
    pub adapter: String,
    pub backend: wgpu::Backend,
    pub device_type: wgpu::DeviceType,
    /// The surface supports extended-range float output (not a monitor measurement).
    pub hdr_supported: bool,
    /// The surface was configured for HDR. False without a surface.
    pub hdr: bool,
    /// The resolved antialiasing method (never `Tier`).
    pub antialias: Antialias,
    /// The samples per pixel the device multisamples the scene at:
    /// [`MSAA_SAMPLES`] where the antialiasing resolved to
    /// [`Antialias::Msaa4`] when the renderer was made, else 1 (TAA, FXAA,
    /// none, or FXAA for want of a multisampled HDR format). It caps
    /// [`RenderQuality::msaa_samples`] as [`Self::shadowed_lights`] caps
    /// [`RenderQuality::shadowed_lights`]; [`Self::antialias`] is what the
    /// frame draws now (FXAA while the quality asks 1).
    pub msaa_samples: u32,
    /// The cascade size and count in use (after clamping).
    pub shadow_map_size: u32,
    pub shadow_cascades: u32,
    /// How many point and spot lights cast shadows in one frame
    /// ([`MAX_SHADOWED_LIGHTS`]).
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
    /// How volumetric fog is drawn ([`RenderDesc::volumetric`]).
    pub volumetric: VolumetricMethod,
    /// The most point, spot and rect lights that light one sample of the
    /// volumetric fog: the cluster's ([`MAX_CLUSTER_LIGHTS`]) in the froxel
    /// grid, [`RAYMARCH_LIGHTS`] in the raymarch.
    pub volumetric_lights: u32,
    /// The anisotropic filtering of the sampler sprites and the UI's images
    /// draw through (text keeps a trilinear one: its distance field): [`SPRITE_ANISOTROPY`] where the device filters
    /// anisotropically (wgpu's `ANISOTROPIC_FILTERING`: Vulkan with
    /// `samplerAnisotropy`, Metal, DX12, WebGPU), 1 where it has none and on a
    /// software rasterizer ([`is_software_adapter`]): lavapipe's anisotropic
    /// filter is an elliptical weighted average that softens even an image
    /// drawn 1:1 (a 4-texel outline reads 216, 124, 40, 40, 124, 216 where
    /// trilinear reads it exactly), so every sprite would blur. Above 1, an
    /// image squeezed along one axis reads its mip level by the other axis
    /// (a nine-slice's middle row stays as crisp as its corners); at 1 it
    /// reads it by the squeezed axis and blurs, which og-ui's 2× bound on a
    /// middle's squeeze keeps slight.
    pub sprite_anisotropy: u16,
    /// How many water surfaces draw a planar reflection in one frame:
    /// [`RenderDesc::planar_reflections`] clamped to 1 to
    /// [`MAX_PLANAR_REFLECTIONS`], 0 without the 3D world ([`WORLD_3D`]).
    pub planar_reflections: u32,
    /// What [`RenderDesc::transparency`] resolved to: the linked lists only
    /// where the device writes storage from fragment shaders and has room
    /// for the arena's binding beside the lit pipelines' own.
    pub transparency: Transparency,
    /// Whether crowds ([`crate::Crowd`]) are culled, levelled and drawn on
    /// the GPU (`crowd_gpu.rs`): where the device executes indirect draws
    /// with a first instance (`INDIRECT_FIRST_INSTANCE`). Without it they
    /// draw through the CPU's skinned lists, the same members at the same
    /// levels.
    pub gpu_crowds: bool,
    /// Whether visual bodies ([`crate::VisualBody`]) simulate: the device
    /// runs compute and reads storage in the vertex stage. Without, they
    /// draw their rest shape.
    pub visual_simulation: bool,
    /// Whether GPU culling's indirect draws each read their args from a
    /// buffer of their own, copied after the cull, rather than from one
    /// buffer at their offsets (`cull_gpu.rs` `per_draw`): on WebKit,
    /// whose Metal backend races indirect draws that share a buffer, and
    /// where [`RenderDesc::indirect_per_draw`] asks for it.
    pub indirect_per_draw: bool,
}

/// The anisotropy clamp of the sprite sampler where the device filters
/// anisotropically ([`RenderInfo::sprite_anisotropy`]): wgpu's largest, so a
/// sprite squeezed up to 16× along one axis keeps the other's detail.
pub const SPRITE_ANISOTROPY: u16 = 16;

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
pub(crate) fn disabled_error() -> RenderError {
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
/// rasterizer), `webgpu`, `dx12`. It names the reference-frame
/// directories and is the probe's `RenderTier`, so a frame time measured on
/// a CPU rasterizer is never read as a GPU's.
pub fn tier_name(backend: wgpu::Backend, device_type: wgpu::DeviceType, adapter: &str) -> String {
    match backend {
        wgpu::Backend::Metal => "metal".into(),
        wgpu::Backend::Vulkan if is_software_adapter(device_type, adapter) => "vulkan-cpu".into(),
        wgpu::Backend::Vulkan => "vulkan".into(),
        wgpu::Backend::BrowserWebGpu => "webgpu".into(),
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
    /// A caller's display texture view of the renderer's size and output format.
    /// With HDR this receives the same surface transfer as `Surface`, not radiance.
    /// Scene render-to-texture cameras use `ViewTarget::Texture` instead.
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
    /// Of those, the layers whose static casters came from the static
    /// shadow cache instead of being drawn again (only the dynamic
    /// casters, if any, were drawn over them).
    pub shadow_cached: u32,
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
    /// past [`MAX_CLUSTER_LIGHTS`].
    pub cluster_lights: u32,
    pub cluster_lights_dropped: u32,
    /// Sprites and glyphs drawn, and the draws they took.
    pub sprites: u32,
    pub glyphs: u32,
    pub sprite_draws: u32,
    /// Faced world sprites (billboards, [`RenderScene::sprite_facing`]) the
    /// main camera left out because their bounds miss its frustum.
    pub sprites_culled: u32,
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
    /// collision could not run here (no colliders on the CPU path, or sprite mode).
    pub particle_systems: u32,
    pub particle_capacity: u32,
    pub particles_cpu: u32,
    pub particle_ticks: u32,
    pub particle_collision_skipped: u32,
    /// Trail, ribbon and beam strips submitted, and their segments. With
    /// compute, a trail's history is on the GPU: it counts as a strip from
    /// its first frame, and its segments are its ring's (the points its
    /// lifetime can hold), drawn or not.
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
    /// Bytes of instance transforms (and, with object motion, their previous
    /// transforms) written to the GPU this frame: only the 4 KiB blocks a
    /// draw reads that changed since the last frame (0 for a scene that
    /// stood still).
    pub instance_upload_bytes: u64,
    /// Draws of the planar reflection pass (0: it did not run).
    pub reflection_draws: u32,
    /// Command buffers the water submitted ahead of the frame (the sea's
    /// fields, foam and caustics in one; one per rippling surface), and the
    /// foam steps of the sea's history recorded among them: a restored or
    /// first frame warms the foam up in some 170 steps, still in one submit.
    pub water_submits: u32,
    pub foam_steps: u32,
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
    /// false on the first frame, after [`Renderer::reset_history`] or on a
    /// frame with [`RenderScene::camera_cut`] (a camera cut), a resize, a new scene, a change of the stylisation's
    /// primitives ([`Stylise::chain_bits`]), and without TAA.
    pub taa_history: bool,
    /// Whether the main camera's blended surfaces were drawn order
    /// independently this frame (`oit.rs`): false when it drew none.
    pub transparency_oit: bool,
    /// The fragments each pixel kept this frame in the linked lists (the
    /// arena at this frame's size), 0 when they did not run (weighted).
    pub transparency_layers: u32,
    /// Pixels whose blended fragments outnumbered their nodes and were
    /// resolved weighted, as counted on the GPU for the newest frame read
    /// back, `transparency_fallback_lag` frames old (`u32::MAX`: none read
    /// yet). 0 without the linked lists.
    pub transparency_fallback_pixels: u32,
    pub transparency_fallback_lag: u32,
}

/// A frame's order-independent transparency, as the hosts keep it for the
/// render dump ([`FrameStats::transparency`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransparencyStats {
    /// [`FrameStats::transparency_oit`].
    pub oit: bool,
    /// [`FrameStats::transparency_layers`].
    pub layers: u32,
    /// [`FrameStats::transparency_fallback_pixels`].
    pub fallback_pixels: u32,
    /// [`FrameStats::transparency_fallback_lag`].
    pub fallback_lag: u32,
}

impl FrameStats {
    /// The frame's order-independent transparency counts.
    pub fn transparency(&self) -> TransparencyStats {
        TransparencyStats {
            oit: self.transparency_oit,
            layers: self.transparency_layers,
            fallback_pixels: self.transparency_fallback_pixels,
            fallback_lag: self.transparency_fallback_lag,
        }
    }
}

/// An RGBA8 frame, rows top to bottom.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// What [`Renderer::save_state`] kept, by number ([`RendererState::id`] and
/// [`RendererState::from_id`] carry it through a host's snapshot as a u64).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RendererState {
    pub(crate) exposure: u64,
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
