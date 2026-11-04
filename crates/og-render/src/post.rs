//! After the main pass: outlines, TAA (and its sharpening), depth of field,
//! motion blur, god rays, auto-exposure metering, bloom (and the 2D glow),
//! the tonemap with grading, vignette, chromatic aberration, grain and
//! flash, and FXAA after it; the lens flare after bloom (`flare.rs`); before it, the ambient occlusion over the
//! depth-normal prepass. Also the size-dependent targets they and the main
//! pass draw into, as render graph targets.

mod flare;
mod graph;
mod passes;
mod plan;
mod smaa;
mod targets;
pub(crate) use flare::*;
pub(crate) use graph::*;
pub(crate) use plan::*;
pub(crate) use targets::*;

use crate::deferred::{AsModule, Deferred};
use crate::graph::{GraphError, GraphPlan, RenderGraph, TargetDesc};
use crate::scene::{Exposure, PostEffects, Tonemap};
use crate::{Antialias, DEPTH_FORMAT, HDR_FORMAT, TaaHistory, target_texture};
use std::collections::HashMap;

pub(crate) const FULLSCREEN_WGSL: &str = wgsl!("fullscreen.wgsl");
/// Exposure, bloom and the tonemap, with the bloom chain's filters (shared
/// with a view's glow, `views_glow.wgsl`).
const POST_WGSL: &str = concat!(
    wgsl!("hdr_lpm.wgsl"),
    wgsl!("post.wgsl"),
    wgsl!("bloom_chain.wgsl")
);
const TAA_WGSL: &str = concat!(wgsl!("taa_uniform.wgsl"), wgsl!("taa.wgsl"));
const LUMINANCE_WGSL: &str = wgsl!("luminance.wgsl");
const EXPOSURE_WGSL: &str = wgsl!("exposure.wgsl");
// Screen-space reflections (ssr.rs) and global illumination (ssgi.rs) are
// more effect passes of the module.
const FXPOST_WGSL: &str = concat!(wgsl!("fxpost.wgsl"), wgsl!("ssr.wgsl"), wgsl!("ssgi.wgsl"));
/// The full-resolution AO blur by gathers (`fs_ao_blur_gather`), appended
/// to the effect module ([`fx_source`]).
const FX_GATHER_WGSL: &str = wgsl!("fxpost_gather.wgsl");

/// The effect passes' module body.
fn fx_source() -> String {
    format!("{FXPOST_WGSL}{FX_GATHER_WGSL}")
}
const FXAA_WGSL: &str = wgsl!("fxaa.wgsl");
const STYLISE_WGSL: &str = wgsl!("stylise.wgsl");

/// The tonemapped image FXAA reads (sRGB-encoded 8 bits, as the output).
pub(crate) const LDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// TAA's refinements ([`crate::RenderDesc::taa_history`],
/// [`crate::RenderDesc::taa_sharpen`]).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AaOptions {
    pub taa_history: TaaHistory,
    /// 0 (no pass) to 1.
    pub taa_sharpen: f32,
}

/// The prepass targets: world normal and velocity, view depth, outline ids.
pub(crate) const GBUFFER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
/// r: view depth; g: the surface's reflectivity for screen-space
/// reflections (`ssr.rs`; the prepass's own, 0 elsewhere).
pub(crate) const LINDEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg32Float;
pub(crate) const OUTLINE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub(crate) const AO_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
/// The AO texture with contact shadows (fxpost.wgsl `fs_contact`): the AO
/// in r, the contact lights' masks in gba.
pub(crate) const CONTACT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// The view depth the prepass clears to (sky).
pub(crate) const FAR_DEPTH: f32 = 1.0e9;

/// The metering grid of auto exposure (reduced to 1x1 by 8x8 averages).
const METER_SIZE: u32 = 64;
const LUMINANCE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;
/// The adapted exposure (EV) of temporal adaptation: full precision, so a
/// step of a fraction of a stop per tick is not lost to rounding.
const EXPOSURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;
/// Adapted exposures kept for [`PostPasses::save_exposure`] (a ring: the
/// probe keeps 16 snapshots, a sweep one more).
const EXPOSURE_SAVES: u32 = 64;
/// The most bloom levels (the first at half resolution).
const MAX_BLOOM_LEVELS: u32 = 6;
/// The weight of the current frame in the TAA blend.
const TAA_CURRENT_WEIGHT: f32 = 0.1;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PostUniform {
    exposure: [f32; 4],
    bloom: [f32; 4],
    mode: [f32; 4],
    grade: [f32; 4],
    vignette: [f32; 4],
    grain: [f32; 4],
    flash: [f32; 4],
    extra: [f32; 4],
    unreal: [f32; 4],
    hdr: [f32; 4],
    /// The lens flare (`flare.rs`): x its intensity (0: off), y 1 with the
    /// starburst, z the starburst's turn with the camera.
    flare: [f32; 4],
}

/// `struct Adapt` in exposure.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct AdaptUniform {
    speed: [f32; 4],
    meter: [f32; 4],
}

/// What [`PostPasses::save_exposure`] kept: the adapted exposure's texel in
/// the save ring and the tick it was adapted to.
#[derive(Clone, Copy, Debug)]
struct ExposureSave {
    id: u64,
    last_tick: Option<u64>,
}

/// The adapted exposure between frames: two 1x1 textures (frame `n` reads
/// `n % 2`'s other one and writes `n % 2`), the tick the latest was adapted
/// to (`None`: the next adapted frame starts at the metered exposure), and
/// the saves of world snapshots.
pub(crate) struct ExposureState {
    textures: [wgpu::Texture; 2],
    pub views: [wgpu::TextureView; 2],
    saves: wgpu::Texture,
    ring: Vec<Option<ExposureSave>>,
    next_id: u64,
    /// The tick the latest adapted exposure belongs to, the texture it is
    /// in, and whether the other one holds it too (after a restore).
    last: Option<(u64, usize, bool)>,
}

/// `struct FxPost` in fxpost.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct FxPostUniform {
    pub proj: [[f32; 4]; 4],
    pub inv_proj: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub screen: [f32; 4],
    pub cam: [f32; 4],
    pub ao: [f32; 4],
    /// x: the AO radius per metre of depth (0: fixed), y: 1 for the screen method.
    pub ao2: [f32; 4],
    pub outline_color: [f32; 4],
    pub outline: [f32; 4],
    pub dof: [f32; 4],
    pub dof2: [f32; 4],
    pub mb: [f32; 4],
    pub god: [f32; 4],
    pub god2: [f32; 4],
    pub god_tint: [f32; 4],
    /// Depth bands (`stylise.rs`): x: count (0: none), y: near, z: far,
    /// w: flatten.
    pub bands: [f32; 4],
    pub bands_near: [f32; 4],
    pub bands_far: [f32; 4],
    /// Screen-space reflections (`ssr.rs`): `ScreenSpaceReflections::rows`.
    pub ssr: [[f32; 4]; 2],
    /// The screen blur (`ScreenBlur::rows`).
    pub blur: [[f32; 4]; 2],
    /// Screen-space global illumination
    /// (`ScreenSpaceGlobalIllumination::rows`).
    pub gi: [[f32; 4]; 3],
    /// Screen-space contact shadows (fxpost.wgsl `fs_contact`): rows 0 to 2
    /// the contact lights' view-space positions (w 1: drawn), row 3 Bend's
    /// surface thickness, bilinear threshold, shadow contrast and sample
    /// count.
    pub contact: [[f32; 4]; 4],
}

/// `struct Stylise` in stylise.wgsl: the post-tonemap stylisation passes'
/// parameters (`PostPasses::write_stylise`).
#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct StyliseUniform {
    pub screen: [f32; 4],
    pub output: [f32; 4],
    pub offset: [f32; 4],
    pub background: [f32; 4],
    pub quantize: [f32; 4],
    pub halftone: [f32; 4],
    pub halftone_color: [f32; 4],
    pub hatching: [f32; 4],
    pub hatching_color: [f32; 4],
    pub paint: [f32; 4],
    pub canvas: [f32; 4],
    pub ink: [f32; 4],
    pub ink_color: [f32; 4],
    /// x: the camera's far plane (the prepass's sky depth), y: the depth
    /// step that is an edge (relative), z: the normal turn that is a crease,
    /// w: how far those edges darken the hatching's tone (`Hatching::depth`).
    pub ink_depth: [f32; 4],
    /// Surface space (`StyliseSpace::Surface`): view space from clip space,
    /// to place a prepass depth.
    pub inv_proj: [[f32; 4]; 4],
    /// The camera's rotation from view to world space (no translation).
    pub view_rotation: [[f32; 4]; 4],
    /// xyz: the camera's position from the surface origin (the 1024 m grid
    /// point under it, [`surface_origin`]), so a pattern far from the
    /// world's origin keeps its precision.
    pub eye: [f32; 4],
    /// Where the canvas (x), the hatching (z) and the halftone (w) lie: 0
    /// over the frame, 1 on the surfaces in world space, 2 in each
    /// instance's space (`StyliseSpace::code`); y: canvas texels per metre
    /// on surfaces.
    pub surface: [f32; 4],
    /// The canvas uv at the surface origin (fractional), so the grain is the
    /// same wherever the origin lies: x along world x (u), y along y (v),
    /// z along z as u, w along z as v.
    pub canvas_origin: [f32; 4],
    /// Lens distortion ([`crate::LensDistortion`]): x the amount, y the
    /// scale that keeps a barrel's farthest corner on the image, zw the
    /// centre.
    pub lens: [f32; 4],
}

/// The grid the surface patterns are measured from (metres): a whole
/// number of every hatching and halftone octave (powers of two to 1024 m).
pub(crate) const SURFACE_GRID: f64 = 1024.0;

/// The surface origin for a camera at `eye`: the [`SURFACE_GRID`] point at
/// or below it on each axis.
pub(crate) fn surface_origin(eye: glam::DVec3) -> glam::DVec3 {
    (eye / SURFACE_GRID).floor() * SURFACE_GRID
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct TaaUniform {
    pub inv_view_proj: [[f32; 4]; 4],
    pub cur_view_proj: [[f32; 4]; 4],
    pub prev_view_proj: [[f32; 4]; 4],
    /// x: the current frame's weight, yz: the scene's (render) size, w:
    /// the sharpening.
    pub params: [f32; 4],
    /// x: 1 when the camera did not move since the history's frame (its
    /// view-projection bit for bit the same): every pixel reads its own
    /// history texel, no reprojection computed on the GPU. y: 1 when the
    /// prepass's velocity is bound (object and skinned motion). zw: this
    /// frame's jitter in scene pixels (x right, y down).
    pub still: [f32; 4],
    /// xy: the history's size (the output's under TAAU, else the scene's),
    /// z: 1 under TAAU (the scene drawn smaller or larger than the output
    /// and resolved up to it here). w: SMAA area layer (0 for 1x, 1/2 for T2x).
    pub upscale: [f32; 4],
}

pub(crate) fn layout_entry(binding: u32, ty: wgpu::BindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty,
        count: None,
    }
}

pub(crate) fn float_texture(filterable: bool) -> wgpu::BindingType {
    wgpu::BindingType::Texture {
        sample_type: wgpu::TextureSampleType::Float { filterable },
        view_dimension: wgpu::TextureViewDimension::D2,
        multisampled: false,
    }
}

pub(crate) const UNIFORM: wgpu::BindingType = wgpu::BindingType::Buffer {
    ty: wgpu::BufferBindingType::Uniform,
    has_dynamic_offset: false,
    min_binding_size: None,
};

/// A pipeline drawing one fullscreen triangle into `format`.
pub(crate) fn fullscreen_pipeline(
    device: &wgpu::Device,
    label: &str,
    module: &wgpu::ShaderModule,
    entry: &str,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some(entry),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: crate::pipeline_cache::of(device).as_ref(),
    })
}

/// [`fullscreen_pipeline`] made when a pass first sets it (deferred.rs).
pub(crate) fn deferred_fullscreen(
    device: &wgpu::Device,
    label: &'static str,
    module: &(impl AsModule + Clone + 'static),
    entry: &'static str,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
) -> Deferred<wgpu::RenderPipeline> {
    let (device, module, layout) = (device.clone(), module.clone(), layout.clone());
    Deferred::new(move || {
        fullscreen_pipeline(
            &device,
            label,
            module.module(),
            entry,
            &layout,
            format,
            blend,
        )
    })
}

/// Records one fullscreen pass into `target`.
pub(crate) fn fullscreen_pass(
    encoder: &mut crate::Encoder,
    label: &str,
    target: &wgpu::TextureView,
    load: wgpu::LoadOp<wgpu::Color>,
    pipeline: &wgpu::RenderPipeline,
    bind_group: &wgpu::BindGroup,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.draw(0..3, 0..1);
}

/// The post pipelines and their fixed resources.
pub(crate) struct PostPasses {
    source_layout: wgpu::BindGroupLayout,
    post_layout: wgpu::BindGroupLayout,
    taa_layout: wgpu::BindGroupLayout,
    meter: Deferred<wgpu::RenderPipeline>,
    reduce: Deferred<wgpu::RenderPipeline>,
    adapt_layout: wgpu::BindGroupLayout,
    adapt: Deferred<wgpu::RenderPipeline>,
    adapt_buffer: wgpu::Buffer,
    pub exposure: ExposureState,
    prefilter: Deferred<wgpu::RenderPipeline>,
    down: Deferred<wgpu::RenderPipeline>,
    up: Deferred<wgpu::RenderPipeline>,
    tonemap: Deferred<wgpu::RenderPipeline>,
    taa: Option<Deferred<wgpu::RenderPipeline>>,
    /// TAA's sharpening pass, when [`AaOptions::taa_sharpen`] is above 0.
    taa_sharpen_pipeline: Option<Deferred<wgpu::RenderPipeline>>,
    /// Its strength (`TaaUniform::params.w`).
    pub taa_sharpen: f32,
    /// The tonemap into [`LDR_FORMAT`], for a frame with passes after it
    /// (FXAA, the stylisation chain); made on first use.
    tonemap_ldr: Option<wgpu::RenderPipeline>,
    /// The passes after the tonemap ([`chain_passes`]), each as (into the
    /// next LDR target, into the output); made on first use.
    chain: HashMap<FramePass, (wgpu::RenderPipeline, wgpu::RenderPipeline)>,
    stylise_layout: wgpu::BindGroupLayout,
    stylise_pl: wgpu::PipelineLayout,
    source_pl: wgpu::PipelineLayout,
    post_pl: wgpu::PipelineLayout,
    post_shader: crate::deferred::Module,
    stylise_shader: Option<wgpu::ShaderModule>,
    fxaa_shader: Option<wgpu::ShaderModule>,
    smaa: Option<smaa::Smaa>,
    output_format: wgpu::TextureFormat,
    hdr: [f32; 4],
    pub chain_format: wgpu::TextureFormat,
    /// `struct Stylise` (stylise.wgsl), written per frame.
    pub stylise_buffer: wgpu::Buffer,
    /// Repeats: the paint pass's canvas texture.
    repeat_sampler: wgpu::Sampler,
    /// The blue-noise dither's thresholds ([`crate::blue_noise`]).
    blue_noise: wgpu::TextureView,
    sampler: wgpu::Sampler,
    /// TAA's depth read: nearest at the pixel's centre, which reads exactly
    /// its texel (taa.wgsl `depth_sampler`).
    nearest_sampler: wgpu::Sampler,
    /// Black and clear: also the 2D light directions' placeholder (none).
    pub(crate) black: wgpu::TextureView,
    /// The identity 32³ grading LUT (bound when no grading is set).
    pub identity_lut: wgpu::TextureView,
    /// The table the tonemap samples while [`crate::ColorAdjust`] is on:
    /// the grading table and the adjustments baked together, with what it
    /// was baked from ([`PostPasses::bake_adjusted_lut`]); made on first use.
    adjusted_lut: Option<(AdjustedKey, wgpu::Texture, wgpu::TextureView)>,
    /// 1x1 placeholders: view depth at [`FAR_DEPTH`], white AO.
    pub far_depth: wgpu::TextureView,
    /// The object pass's "no instance" texel, bound where a frame has none.
    pub no_object: wgpu::TextureView,
    pub white: wgpu::TextureView,
    post_buffer: wgpu::Buffer,
    pub taa_buffer: wgpu::Buffer,
    fx_layout: wgpu::BindGroupLayout,
    pub fx_buffer: wgpu::Buffer,
    /// The effect passes' uniform after TAA ([`FramePass::after_taa`]): the
    /// same but sized to the post chain's frame, the output's under TAAU.
    pub fx_after_buffer: wgpu::Buffer,
    fx: HashMap<&'static str, Deferred<wgpu::RenderPipeline>>,
    /// The lens flare's passes (`flare.rs`).
    pub flare: FlarePasses,
}

/// A 3D RGBA8 LUT texture from a 32³ table.
pub(crate) fn lut_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    lut: &crate::ColorLut,
) -> wgpu::TextureView {
    let texture = new_lut_texture(device, "grading lut", lut.size);
    write_lut(queue, &texture, lut);
    texture.create_view(&Default::default())
}

/// An empty `size`³ RGBA8 LUT texture.
fn new_lut_texture(device: &wgpu::Device, label: &str, size: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: size,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// Writes a table into a LUT texture of its size ([`lut_texture`]).
fn write_lut(queue: &wgpu::Queue, texture: &wgpu::Texture, lut: &crate::ColorLut) {
    queue.write_texture(
        texture.as_image_copy(),
        &lut.rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(lut.size * 4),
            rows_per_image: Some(lut.size),
        },
        texture.size(),
    );
}

/// What the adjusted table ([`PostPasses::bake_adjusted_lut`]) was baked
/// from: the adjustments and, with grading, the table's scene, slot and
/// revision and the intensity.
type AdjustedKey = (crate::ColorAdjust, Option<(u64, usize, u64, f32)>);

fn one_texel(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat,
    bytes: &[u8],
) -> wgpu::TextureView {
    texture_with_data(device, queue, 1, format, bytes)
}

/// A square `size` × `size` texture of `bytes`, sampled only.
fn texture_with_data(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    size: u32,
    format: wgpu::TextureFormat,
    bytes: &[u8],
) -> wgpu::TextureView {
    use wgpu::util::DeviceExt;
    device
        .create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("post data"),
                size: wgpu::Extent3d {
                    width: size,
                    height: size,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            bytes,
        )
        .create_view(&Default::default())
}

/// The full-resolution AO blur (fxpost_gather.wgsl): `fs_ao_blur` to the bit.
const AO_BLUR_GATHER: &str = "fs_ao_blur_gather";

/// The fullscreen effect passes of fxpost.wgsl: (entry point, target format).
const FX_PASSES: &[(&str, wgpu::TextureFormat)] = &[
    ("fs_gtao", AO_FORMAT),
    ("fs_ssao", AO_FORMAT),
    ("fs_ao_blur", AO_FORMAT),
    ("fs_ao_upsample", AO_FORMAT),
    ("fs_contact", CONTACT_FORMAT),
    ("fs_outline", HDR_FORMAT),
    ("fs_dof_coc", HDR_FORMAT),
    ("fs_dof_blur", HDR_FORMAT),
    ("fs_dof_composite", HDR_FORMAT),
    ("fs_motion_blur", HDR_FORMAT),
    ("fs_screen_blur", HDR_FORMAT),
    ("fs_godrays_mask", HDR_FORMAT),
    ("fs_godrays_blur", HDR_FORMAT),
    ("fs_depth_bands", HDR_FORMAT),
    ("fs_ssr", HDR_FORMAT),
    ("fs_ssgi", crate::ssgi::GI_FORMAT),
    ("fs_gi_blur", crate::ssgi::GI_FORMAT),
    ("fs_gi_upsample", crate::ssgi::GI_FORMAT),
    ("fs_gtao_bent", crate::ssgi::GI_FORMAT),
];

/// The current frame's weight in the TAA blend: 1 when there is no history.
pub(crate) fn taa_weight(history_valid: bool) -> f32 {
    if history_valid {
        TAA_CURRENT_WEIGHT
    } else {
        1.0
    }
}

/// A post pass's module: the fullscreen vertex stage and `body`.
pub(crate) fn fullscreen_source(body: &str) -> String {
    format!("{FULLSCREEN_WGSL}{body}")
}

/// The post passes' modules by label, as [`PostPasses::new`] compiles them.
pub(crate) fn post_sources() -> Vec<(&'static str, String)> {
    let fx = fx_source();
    [
        ("post", POST_WGSL),
        ("luminance", LUMINANCE_WGSL),
        ("exposure", EXPOSURE_WGSL),
        ("taa", TAA_WGSL),
        ("fxaa", FXAA_WGSL),
        ("smaa edges", smaa::EDGES),
        ("smaa weights", smaa::WEIGHTS),
        ("smaa blend", smaa::BLEND),
        ("hdr transfer", crate::hdr::TRANSFER),
        ("stylise", STYLISE_WGSL),
        ("fx post", fx.as_str()),
        ("lens flare", FLARE_WGSL),
    ]
    .into_iter()
    .map(|(label, body)| (label, fullscreen_source(body)))
    .collect()
}
