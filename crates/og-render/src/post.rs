//! After the main pass: outlines, TAA (and its sharpening), depth of field,
//! motion blur, god rays, auto-exposure metering, bloom (and the 2D glow),
//! the tonemap with grading, vignette, chromatic aberration, grain and
//! flash, and FXAA after it; before it, the ambient occlusion over the
//! depth-normal prepass. Also the size-dependent targets they and the main
//! pass draw into, as render graph targets.

use crate::deferred::{AsModule, Deferred};
use crate::graph::{GraphError, GraphPlan, RenderGraph, TargetDesc};
use crate::scene::{Exposure, PostEffects, Tonemap};
use crate::{Antialias, DEPTH_FORMAT, HDR_FORMAT, TaaHistory, target_texture};
use std::collections::HashMap;

pub(crate) const FULLSCREEN_WGSL: &str = wgsl!("fullscreen.wgsl");
/// Exposure, bloom and the tonemap, with the bloom chain's filters (shared
/// with a view's glow, `views_glow.wgsl`).
const POST_WGSL: &str = concat!(wgsl!("post.wgsl"), wgsl!("bloom_chain.wgsl"));
const TAA_WGSL: &str = wgsl!("taa.wgsl");
const LUMINANCE_WGSL: &str = wgsl!("luminance.wgsl");
const EXPOSURE_WGSL: &str = wgsl!("exposure.wgsl");
// Screen-space reflections (ssr.rs) are one more effect pass of the module.
const FXPOST_WGSL: &str = concat!(wgsl!("fxpost.wgsl"), wgsl!("ssr.wgsl"));
/// The full tier's AO blur by gathers (`fs_ao_blur_gather`), appended to
/// the effect module there: GLSL ES 3.0 has no gather ([`fx_source`]).
const FX_GATHER_WGSL: &str = wgsl!("fxpost_gather.wgsl");

/// The effect passes' module body on the full tier (`full`) or the GL tier.
fn fx_source(full: bool) -> String {
    if full {
        format!("{FXPOST_WGSL}{FX_GATHER_WGSL}")
    } else {
        FXPOST_WGSL.to_string()
    }
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
}

/// The grid the surface patterns are measured from (metres): a whole
/// number of every hatching and halftone octave (powers of two to 1024 m).
pub(crate) const SURFACE_GRID: f64 = 1024.0;

/// The surface origin for a camera at `eye`: the [`SURFACE_GRID`] point at
/// or below it on each axis.
pub(crate) fn surface_origin(eye: glam::Vec3) -> glam::DVec3 {
    (eye.as_dvec3() / SURFACE_GRID).floor() * SURFACE_GRID
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct TaaUniform {
    pub inv_view_proj: [[f32; 4]; 4],
    pub cur_view_proj: [[f32; 4]; 4],
    pub prev_view_proj: [[f32; 4]; 4],
    pub params: [f32; 4],
    /// x: 1 when the camera did not move since the history's frame (its
    /// view-projection bit for bit the same): every pixel reads its own
    /// history texel, no reprojection computed on the GPU.
    pub still: [f32; 4],
}

fn layout_entry(binding: u32, ty: wgpu::BindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty,
        count: None,
    }
}

fn float_texture(filterable: bool) -> wgpu::BindingType {
    wgpu::BindingType::Texture {
        sample_type: wgpu::TextureSampleType::Float { filterable },
        view_dimension: wgpu::TextureViewDimension::D2,
        multisampled: false,
    }
}

const UNIFORM: wgpu::BindingType = wgpu::BindingType::Buffer {
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
    output_format: wgpu::TextureFormat,
    /// `struct Stylise` (stylise.wgsl), written per frame.
    pub stylise_buffer: wgpu::Buffer,
    /// Repeats: the paint pass's canvas texture.
    repeat_sampler: wgpu::Sampler,
    /// The blue-noise dither's thresholds ([`crate::blue_noise`]).
    blue_noise: wgpu::TextureView,
    sampler: wgpu::Sampler,
    /// TAA's depth read: nearest, so the GL backend binds a sampler to the
    /// depth texture's unit (taa.wgsl `depth_sampler`).
    nearest_sampler: wgpu::Sampler,
    black: wgpu::TextureView,
    /// The identity 32³ grading LUT (bound when no grading is set).
    pub identity_lut: wgpu::TextureView,
    /// 1x1 placeholders: view depth at [`FAR_DEPTH`], white AO.
    pub far_depth: wgpu::TextureView,
    /// The object pass's "no instance" texel, bound where a frame has none.
    pub no_object: wgpu::TextureView,
    pub white: wgpu::TextureView,
    post_buffer: wgpu::Buffer,
    pub taa_buffer: wgpu::Buffer,
    fx_layout: wgpu::BindGroupLayout,
    pub fx_buffer: wgpu::Buffer,
    fx: HashMap<&'static str, Deferred<wgpu::RenderPipeline>>,
}

/// A 3D RGBA8 LUT texture from a 32³ table.
pub(crate) fn lut_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    lut: &crate::ColorLut,
) -> wgpu::TextureView {
    use wgpu::util::DeviceExt;
    device
        .create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("grading lut"),
                size: wgpu::Extent3d {
                    width: lut.size,
                    height: lut.size,
                    depth_or_array_layers: lut.size,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D3,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &lut.rgba,
        )
        .create_view(&Default::default())
}

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

/// The full tier's AO blur (fxpost_gather.wgsl): `fs_ao_blur` to the bit.
const AO_BLUR_GATHER: &str = "fs_ao_blur_gather";

/// The fullscreen effect passes of fxpost.wgsl: (entry point, target format).
const FX_PASSES: &[(&str, wgpu::TextureFormat)] = &[
    ("fs_gtao", AO_FORMAT),
    ("fs_ssao", AO_FORMAT),
    ("fs_ao_blur", AO_FORMAT),
    ("fs_ao_upsample", AO_FORMAT),
    ("fs_outline", HDR_FORMAT),
    ("fs_dof_coc", HDR_FORMAT),
    ("fs_dof_blur", HDR_FORMAT),
    ("fs_dof_composite", HDR_FORMAT),
    ("fs_motion_blur", HDR_FORMAT),
    ("fs_godrays_mask", HDR_FORMAT),
    ("fs_godrays_blur", HDR_FORMAT),
    ("fs_depth_bands", HDR_FORMAT),
    ("fs_ssr", HDR_FORMAT),
];

impl PostPasses {
    /// Makes one of its pipelines no frame used yet: the tonemap, the
    /// exposure meter and bloom first, then TAA and the effect passes
    /// (Renderer::warm).
    pub(crate) fn warm(&self) -> bool {
        let mut items: Vec<&dyn crate::deferred::Warm> = vec![
            &self.tonemap,
            &self.meter,
            &self.reduce,
            &self.adapt,
            &self.prefilter,
            &self.down,
            &self.up,
        ];
        items.extend(self.taa.iter().map(|p| p as &dyn crate::deferred::Warm));
        items.extend(
            self.taa_sharpen_pipeline
                .iter()
                .map(|p| p as &dyn crate::deferred::Warm),
        );
        for entry in FX_PASSES.iter().map(|(e, _)| e).chain([&AO_BLUR_GATHER]) {
            if let Some(p) = self.fx.get(entry) {
                items.push(p);
            }
        }
        crate::deferred::warm_one(&items)
    }

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_format: wgpu::TextureFormat,
        antialias: Antialias,
        aa: AaOptions,
        full: bool,
    ) -> PostPasses {
        let sampler_ty = wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering);
        let source_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post source"),
            entries: &[
                layout_entry(0, float_texture(true)),
                layout_entry(1, sampler_ty),
            ],
        });
        let post_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post"),
            entries: &[
                layout_entry(0, float_texture(true)),
                layout_entry(1, sampler_ty),
                layout_entry(2, UNIFORM),
                layout_entry(3, float_texture(false)),
                layout_entry(4, float_texture(true)),
                layout_entry(
                    5,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                ),
                layout_entry(6, float_texture(true)),
            ],
        });
        let fx_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fx post"),
            entries: &[
                layout_entry(0, UNIFORM),
                layout_entry(1, sampler_ty),
                layout_entry(2, float_texture(true)),
                layout_entry(3, float_texture(false)),
                layout_entry(4, float_texture(false)),
                layout_entry(5, float_texture(true)),
                // Nearest, for the full tier's gathers of the unfilterable
                // view depth (fxpost_gather.wgsl).
                layout_entry(
                    6,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                ),
            ],
        });
        let taa_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("taa"),
            entries: &[
                layout_entry(0, float_texture(false)),
                layout_entry(1, float_texture(true)),
                // The depth buffer as an unfilterable float texture: GLSL
                // has no `textureLoad` from a depth texture (WebGL2), and
                // every backend reads the same depth this way.
                layout_entry(2, float_texture(false)),
                layout_entry(3, sampler_ty),
                layout_entry(4, UNIFORM),
                // The depth's own nearest sampler: on the GL backend a
                // texture read with `textureLoad` gets no sampler, and a
                // depth texture under its default linear filtering is
                // incomplete and reads 0 (taa.wgsl, `depth_sampler`).
                layout_entry(
                    5,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                ),
            ],
        });
        let module = |label: &'static str, body: &str| {
            crate::deferred::module(device, label, fullscreen_source(body))
        };
        let post_shader = module("post", POST_WGSL);
        let luminance_shader = module("luminance", LUMINANCE_WGSL);
        let pipeline_layout = |label: &str, layout: &wgpu::BindGroupLayout| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &[Some(layout)],
                immediate_size: 0,
            })
        };
        let source_pl = pipeline_layout("post source", &source_layout);
        let post_pl = pipeline_layout("post", &post_layout);
        // The stylisation passes (stylise.wgsl): the source, its sampler,
        // the parameters, the palette, the canvas and its repeating sampler,
        // the blue-noise tile, the prepass's view depth and normals (ink on
        // depth edges; placeholders without a prepass) and the object
        // pass's positions (object space; a zero texel without one).
        let stylise_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("stylise"),
            entries: &[
                layout_entry(0, float_texture(true)),
                layout_entry(1, sampler_ty),
                layout_entry(2, UNIFORM),
                layout_entry(3, float_texture(true)),
                layout_entry(4, float_texture(true)),
                layout_entry(5, sampler_ty),
                layout_entry(6, float_texture(true)),
                layout_entry(7, float_texture(false)),
                layout_entry(8, float_texture(false)),
                layout_entry(
                    9,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
            ],
        });
        let stylise_pl = pipeline_layout("stylise", &stylise_layout);
        let meter = deferred_fullscreen(
            device,
            "meter",
            &luminance_shader,
            "fs_meter",
            &source_pl,
            LUMINANCE_FORMAT,
            None,
        );
        let reduce = deferred_fullscreen(
            device,
            "meter reduce",
            &luminance_shader,
            "fs_reduce",
            &source_pl,
            LUMINANCE_FORMAT,
            None,
        );
        let adapt_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("exposure adapt"),
            entries: &[
                layout_entry(0, float_texture(false)),
                layout_entry(1, float_texture(false)),
                layout_entry(2, UNIFORM),
            ],
        });
        let adapt = deferred_fullscreen(
            device,
            "exposure adapt",
            &module("exposure", EXPOSURE_WGSL),
            "fs_adapt",
            &pipeline_layout("exposure adapt", &adapt_layout),
            EXPOSURE_FORMAT,
            None,
        );
        let exposure_texture = |label: &str, width: u32| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: EXPOSURE_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        };
        let textures = [
            exposure_texture("adapted exposure", 1),
            exposure_texture("adapted exposure", 1),
        ];
        let exposure = ExposureState {
            views: [
                textures[0].create_view(&Default::default()),
                textures[1].create_view(&Default::default()),
            ],
            textures,
            saves: exposure_texture("adapted exposure saves", EXPOSURE_SAVES),
            ring: vec![None; EXPOSURE_SAVES as usize],
            next_id: 1,
            last: None,
        };
        let prefilter = deferred_fullscreen(
            device,
            "bloom prefilter",
            &post_shader,
            "fs_bloom_prefilter",
            &post_pl,
            HDR_FORMAT,
            None,
        );
        let down = deferred_fullscreen(
            device,
            "bloom down",
            &post_shader,
            "fs_bloom_down",
            &post_pl,
            HDR_FORMAT,
            None,
        );
        let additive = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::REPLACE,
        };
        let up = deferred_fullscreen(
            device,
            "bloom up",
            &post_shader,
            "fs_bloom_up",
            &post_pl,
            HDR_FORMAT,
            Some(additive),
        );
        // A non-sRGB output (web canvases) gets the sRGB curve in the shader.
        let tonemap_entry = if output_format.is_srgb() {
            "fs"
        } else {
            "fs_srgb_encode"
        };
        let tonemap = deferred_fullscreen(
            device,
            "tonemap",
            &post_shader,
            tonemap_entry,
            &post_pl,
            output_format,
            None,
        );
        let taa_on = antialias == Antialias::Taa;
        let taa_shader = taa_on.then(|| module("taa", TAA_WGSL));
        let taa_pl = pipeline_layout("taa", &taa_layout);
        let taa = taa_shader.as_ref().map(|taa_shader| {
            deferred_fullscreen(
                device,
                "taa",
                taa_shader,
                match aa.taa_history {
                    TaaHistory::Bilinear => "fs",
                    TaaHistory::CatmullRom => "fs_catmull_rom",
                },
                &taa_pl,
                HDR_FORMAT,
                None,
            )
        });
        let taa_sharpen = if taa_on {
            aa.taa_sharpen.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let taa_sharpen_pipeline =
            taa_shader
                .as_ref()
                .filter(|_| taa_sharpen > 0.0)
                .map(|taa_shader| {
                    deferred_fullscreen(
                        device,
                        "taa sharpen",
                        taa_shader,
                        "fs_sharpen",
                        &taa_pl,
                        HDR_FORMAT,
                        None,
                    )
                });
        let repeat_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("stylise canvas"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("post"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("post nearest"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let black_texture = target_texture(
            device,
            "black",
            1,
            1,
            HDR_FORMAT,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        queue.write_texture(
            black_texture.as_image_copy(),
            &[0u8; 8],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(8),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let uniform = |label: &str, size: usize| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: size as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let fx_shader = module("fx post", &fx_source(full));
        let fx_pl = pipeline_layout("fx post", &fx_layout);
        let mut fx: HashMap<&'static str, Deferred<wgpu::RenderPipeline>> = FX_PASSES
            .iter()
            .map(|&(entry, format)| {
                (
                    entry,
                    deferred_fullscreen(device, entry, &fx_shader, entry, &fx_pl, format, None),
                )
            })
            .collect();
        if full {
            fx.insert(
                AO_BLUR_GATHER,
                deferred_fullscreen(
                    device,
                    AO_BLUR_GATHER,
                    &fx_shader,
                    AO_BLUR_GATHER,
                    &fx_pl,
                    AO_FORMAT,
                    None,
                ),
            );
        }
        let identity_lut = lut_texture(device, queue, &crate::ColorLut::identity());
        let far_depth = one_texel(
            device,
            queue,
            LINDEPTH_FORMAT,
            bytemuck::bytes_of(&[FAR_DEPTH, 0.0]),
        );
        let white = one_texel(device, queue, AO_FORMAT, &[255]);
        let no_object = one_texel(
            device,
            queue,
            crate::stylise_object::OBJECT_FORMAT,
            &[0; 16],
        );
        let blue_noise = texture_with_data(
            device,
            queue,
            crate::blue_noise::BLUE_NOISE_SIZE,
            wgpu::TextureFormat::R8Unorm,
            crate::blue_noise::BLUE_NOISE,
        );
        PostPasses {
            blue_noise,
            identity_lut,
            far_depth,
            white,
            no_object,
            fx_layout,
            fx_buffer: uniform("fx post", std::mem::size_of::<FxPostUniform>()),
            fx,
            source_layout,
            post_layout,
            taa_layout,
            meter,
            reduce,
            adapt_layout,
            adapt,
            adapt_buffer: uniform("exposure adapt", std::mem::size_of::<AdaptUniform>()),
            exposure,
            prefilter,
            down,
            up,
            tonemap,
            taa,
            taa_sharpen_pipeline,
            taa_sharpen,
            tonemap_ldr: None,
            chain: HashMap::new(),
            stylise_layout,
            stylise_pl,
            source_pl,
            post_pl,
            post_shader,
            stylise_shader: None,
            fxaa_shader: None,
            output_format,
            stylise_buffer: uniform("stylise", std::mem::size_of::<StyliseUniform>()),
            repeat_sampler,
            sampler,
            nearest_sampler,
            black: black_texture.create_view(&Default::default()),
            post_buffer: uniform("post", std::mem::size_of::<PostUniform>()),
            taa_buffer: uniform("taa", std::mem::size_of::<TaaUniform>()),
        }
    }

    /// Makes the pipelines of the passes after the tonemap that `passes`
    /// holds and this renderer lacks ([`chain_passes`]): the tonemap into
    /// the LDR target, and each pass into the next target and into the
    /// output (sRGB-encoded in the shader where the output is not sRGB).
    pub fn ensure_chain(&mut self, device: &wgpu::Device, passes: &[FramePass]) {
        let chain = chain_passes(passes);
        if chain.is_empty() {
            return;
        }
        if self.tonemap_ldr.is_none() {
            self.tonemap_ldr = Some(fullscreen_pipeline(
                device,
                "tonemap (ldr)",
                &self.post_shader,
                "fs",
                &self.post_pl,
                LDR_FORMAT,
                None,
            ));
        }
        let srgb = self.output_format.is_srgb();
        for pass in chain {
            if self.chain.contains_key(&pass) {
                continue;
            }
            let entry: &str = match pass {
                FramePass::Fxaa => "fs",
                FramePass::Paint => "fs_paint",
                FramePass::Lines => "fs_lines",
                FramePass::Quantize => "fs_quantize",
                FramePass::Upscale => "fs_upscale",
                _ => unreachable!("not a pass after the tonemap"),
            };
            let encoded = if pass == FramePass::Fxaa {
                "fs_srgb_encode".to_string()
            } else {
                format!("{entry}_srgb")
            };
            let output_entry = if srgb { entry } else { encoded.as_str() };
            let (shader, layout) = if pass == FramePass::Fxaa {
                let shader = self.fxaa_shader.get_or_insert_with(|| {
                    device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("fxaa"),
                        source: wgpu::ShaderSource::Wgsl(fullscreen_source(FXAA_WGSL).into()),
                    })
                });
                (&*shader, &self.source_pl)
            } else {
                let shader = self.stylise_shader.get_or_insert_with(|| {
                    device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("stylise"),
                        source: wgpu::ShaderSource::Wgsl(fullscreen_source(STYLISE_WGSL).into()),
                    })
                });
                (&*shader, &self.stylise_pl)
            };
            let label = pass.name();
            let to_ldr = fullscreen_pipeline(
                device,
                &format!("{label} (ldr)"),
                shader,
                entry,
                layout,
                LDR_FORMAT,
                None,
            );
            let to_output = fullscreen_pipeline(
                device,
                &label,
                shader,
                output_entry,
                layout,
                self.output_format,
                None,
            );
            self.chain.insert(pass, (to_ldr, to_output));
        }
    }

    /// Writes the stylisation passes' parameters.
    pub fn write_stylise(&self, queue: &wgpu::Queue, uniform: &StyliseUniform) {
        queue.write_buffer(&self.stylise_buffer, 0, bytemuck::bytes_of(uniform));
    }

    /// Writes the post uniform. `glow`: the glow buffer feeds the bloom
    /// chain; `god_rays`: their texture is added; `untonemapped`: the lit
    /// image's alpha is the fraction of each pixel to tone map (sprites
    /// drawn untonemapped lowered it); `tick` seeds the grain.
    #[allow(clippy::too_many_arguments)]
    pub fn write_uniform(
        &self,
        queue: &wgpu::Queue,
        exposure: Exposure,
        post: &PostEffects,
        glow: bool,
        god_rays: bool,
        untonemapped: bool,
        tick: u64,
        width: u32,
    ) {
        let adapted = exposure.adaptation().is_some();
        let (exposure, compensation) = match exposure {
            Exposure::Manual { ev } => ([ev, 0.0, 0.0, 0.0], 0.0),
            Exposure::Auto {
                compensation,
                min_ev,
                max_ev,
                ..
            } => (
                [
                    0.0,
                    if adapted { 2.0 } else { 1.0 },
                    min_ev,
                    max_ev.max(min_ev),
                ],
                compensation,
            ),
        };
        let lit_bloom = post.bloom.filter(|b| b.intensity > 0.0);
        let bloom = match (lit_bloom, post.glow.filter(|_| glow)) {
            // UnrealBloomPass: the mips are stored weighted, so the top
            // level is their mix; the additive copy multiplies it by the
            // strength again (its alpha, strength times the weights' sum).
            (Some(b), _) => match post.bloom_unreal {
                Some(r) => {
                    let sum: f32 = (0..5)
                        .map(|i| {
                            let f = 1.0 - 0.2 * i as f32;
                            f + (1.2 - 2.0 * f) * r
                        })
                        .sum();
                    [
                        b.threshold,
                        b.knee,
                        b.intensity * b.intensity * sum,
                        compensation,
                    ]
                }
                None => [b.threshold, b.knee, b.intensity, compensation],
            },
            (None, Some(g)) => [1.0, 0.5, g.intensity.max(0.0), compensation],
            (None, None) => [1.0, 0.5, 0.0, compensation],
        };
        let tonemap = match post.tonemap {
            Tonemap::Aces => 0.0,
            Tonemap::None => 1.0,
            Tonemap::Agx => 2.0,
        };
        let v = post.vignette.unwrap_or(crate::Vignette {
            intensity: 0.0,
            ..Default::default()
        });
        let ca = post
            .chromatic_aberration
            .map_or(0.0, |c| (c.intensity.max(0.0) * 2.0 / 720.0).min(0.1));
        let grain = post.film_grain.map_or([0.0; 4], |g| {
            let seed = crate::particles::hash_unit(tick as u32 ^ (tick >> 32) as u32);
            [g.intensity.max(0.0), g.size.max(1.0), seed, 0.0]
        });
        let flash = post.flash.map_or([0.0; 4], |f| {
            f.color.extend(f.amount.clamp(0.0, 1.0)).to_array()
        });
        let uniform = PostUniform {
            exposure,
            bloom,
            mode: [
                tonemap,
                1.0,
                lit_bloom.is_some() as u32 as f32,
                glow as u32 as f32,
            ],
            grade: [
                post.color_grading
                    .map_or(0.0, |g| g.intensity.clamp(0.0, 1.0)),
                v.intensity.clamp(0.0, 1.0),
                v.radius,
                v.smoothness,
            ],
            vignette: v.color.extend(ca).to_array(),
            grain,
            flash,
            extra: [
                god_rays as u32 as f32,
                untonemapped as u32 as f32,
                post.sharpen.is_some() as u32 as f32,
                post.sharpen.unwrap_or(0.0).clamp(0.0, 1.0),
            ],
            unreal: match (lit_bloom, post.bloom_unreal) {
                (Some(_), Some(r)) => [
                    1.0,
                    r,
                    (width / 2).max(1) as f32,
                    if post.bloom_contrast > 0.0 {
                        post.bloom_contrast
                    } else {
                        1.0
                    },
                ],
                _ => [0.0; 4],
            },
        };
        queue.write_buffer(&self.post_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    /// Advances the adapted exposure's clock to this frame's tick and writes
    /// the adaptation uniform. `exposure` adapts ([`Exposure::adaptation`]);
    /// `slot` is the texture this frame writes. Returns nothing: the frame's
    /// adapt pass does the rest. A frame that does not adapt calls
    /// [`PostPasses::forget_exposure`] instead.
    pub fn advance_exposure(
        &mut self,
        queue: &wgpu::Queue,
        exposure: Exposure,
        tick: u64,
        tick_rate: f32,
        slot: usize,
    ) {
        let Exposure::Auto {
            compensation,
            min_ev,
            max_ev,
            ..
        } = exposure
        else {
            return;
        };
        let (up, down) = exposure.adaptation().unwrap_or((0.0, 0.0));
        // The previous adapted exposure must be in the texture this frame
        // reads; a frame whose tick went back starts again. A frame of tick
        // 0, before the first tick, meters at once and is never adapted
        // from: one host draws it (the web shell's first frame, behind the
        // play gate) and another does not (`og run` draws from tick 1), so
        // the adaptation starts at the first ticked frame's metered
        // exposure on every host (web findings #5: Lantern Vale's first two
        // seconds were brighter on the web, which adapted from its tick 0
        // frame).
        let (dt, snap) = match self.exposure.last {
            _ if tick == 0 => (0.0, true),
            Some((t, written, both)) if tick >= t && (written != slot || both) => {
                ((tick - t) as f32 / tick_rate.max(1e-3), false)
            }
            _ => (0.0, true),
        };
        self.exposure.last = (tick > 0).then_some((tick, slot, false));
        let u = AdaptUniform {
            speed: [dt, up, down, snap as u32 as f32],
            meter: [compensation, min_ev, max_ev.max(min_ev), 0.0],
        };
        queue.write_buffer(&self.adapt_buffer, 0, bytemuck::bytes_of(&u));
    }

    /// A frame without adaptation: the next adapted frame starts afresh.
    pub fn forget_exposure(&mut self) {
        self.exposure.last = None;
    }

    fn texel_copy(
        encoder: &mut crate::Encoder,
        from: &wgpu::Texture,
        from_x: u32,
        to: &wgpu::Texture,
        to_x: u32,
    ) {
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: from,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: from_x,
                    y: 0,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: to,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: to_x,
                    y: 0,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Keeps the adapted exposure as it is now; the answer names it for
    /// [`PostPasses::restore_exposure`]. Recorded into `encoder`, so it
    /// copies what the frames submitted before it wrote.
    pub fn save_exposure(&mut self, encoder: &mut crate::Encoder) -> u64 {
        let id = self.exposure.next_id;
        self.exposure.next_id += 1;
        let at = (id % EXPOSURE_SAVES as u64) as u32;
        if let Some((_, written, _)) = self.exposure.last {
            Self::texel_copy(
                encoder,
                &self.exposure.textures[written],
                0,
                &self.exposure.saves,
                at,
            );
        }
        self.exposure.ring[at as usize] = Some(ExposureSave {
            id,
            last_tick: self.exposure.last.map(|(t, ..)| t),
        });
        id
    }

    /// Brings back the adapted exposure [`PostPasses::save_exposure`] kept
    /// as `id`; one no longer kept (or 0) starts the next adapted frame
    /// afresh. Recorded into `encoder`.
    pub fn restore_exposure(&mut self, encoder: &mut crate::Encoder, id: u64) {
        let at = (id % EXPOSURE_SAVES as u64) as usize;
        let kept = self.exposure.ring[at].filter(|s| s.id == id && id != 0);
        self.exposure.last = match kept.and_then(|s| s.last_tick) {
            Some(tick) => {
                for t in &self.exposure.textures {
                    Self::texel_copy(encoder, &self.exposure.saves, at as u32, t, 0);
                }
                // Both textures hold it: whichever the next frame reads.
                Some((tick, 0, true))
            }
            None => None,
        };
    }
}

/// The size-dependent targets that outlive a frame (imported into the
/// graph): the offscreen output and the TAA history.
pub(crate) struct Persistent {
    pub offscreen: wgpu::Texture,
    pub offscreen_view: wgpu::TextureView,
    /// The offscreen target seen as sRGB when its format is not
    /// ([`crate::linear_view_format`]): where the overlay blends.
    pub offscreen_linear: Option<wgpu::TextureView>,
    /// TAA history ping-pong: frame `n` writes `history[n % 2]`.
    pub history: Option<[wgpu::TextureView; 2]>,
}

impl Persistent {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        output_format: wgpu::TextureFormat,
        linear_format: Option<wgpu::TextureFormat>,
        antialias: Antialias,
    ) -> Persistent {
        let attach_bind =
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let offscreen = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: output_format,
            usage: attach_bind | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
            view_formats: linear_format.as_slice(),
        });
        let offscreen_view = offscreen.create_view(&Default::default());
        let offscreen_linear = linear_format.map(|format| {
            offscreen.create_view(&wgpu::TextureViewDescriptor {
                format: Some(format),
                ..Default::default()
            })
        });
        let history = (antialias == Antialias::Taa).then(|| {
            [0, 1].map(|_| {
                target_texture(
                    device,
                    "taa history",
                    width,
                    height,
                    HDR_FORMAT,
                    attach_bind,
                )
                .create_view(&Default::default())
            })
        });
        Persistent {
            offscreen,
            offscreen_view,
            offscreen_linear,
            history,
        }
    }
}

impl Persistent {
    /// The TAA history at a new scene size (the offscreen target keeps the
    /// output's).
    pub fn resize_history(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        antialias: Antialias,
    ) {
        let attach_bind =
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        self.history = (antialias == Antialias::Taa).then(|| {
            [0, 1].map(|_| {
                target_texture(
                    device,
                    "taa history",
                    width,
                    height,
                    HDR_FORMAT,
                    attach_bind,
                )
                .create_view(&Default::default())
            })
        });
    }
}

/// A `width` × `height` texture of `format` from its RGBA8 texels.
pub(crate) fn texture_from_rgba8(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    rgba: &[u8],
) -> wgpu::TextureView {
    use wgpu::util::DeviceExt;
    device
        .create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
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
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            rgba,
        )
        .create_view(&Default::default())
}

/// The bloom chain's length for a target size (the first level at half
/// resolution, each next one half the previous).
pub(crate) fn bloom_levels(width: u32, height: u32) -> u32 {
    let (bw, bh) = ((width / 2).max(1), (height / 2).max(1));
    (32 - bw.min(bh).max(1).leading_zeros())
        .saturating_sub(3)
        .clamp(1, MAX_BLOOM_LEVELS)
}

/// Levels of the auto-exposure metering chain (64, 8, 1).
pub(crate) fn meter_levels() -> u32 {
    let mut n = 1;
    let mut size = METER_SIZE;
    while size > 1 {
        size /= 8;
        n += 1;
    }
    n
}

/// One pass of a frame, as the render graph knows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum FramePass {
    /// Directional light cascade `i`.
    Cascade(u32),
    /// Local light shadow layer (6 per point light, 1 per spot).
    LocalFace(u32),
    /// The depth-normal prepass (normal, velocity, view depth, outline ids).
    Prepass,
    /// Ambient occlusion (GTAO or SSAO), then its blur.
    Ao,
    AoBlur,
    /// Half-resolution ambient occlusion's depth-aware upsample
    /// ([`crate::AoResolution::Half`]): with it, `ao` and its blur run at
    /// half resolution and this pass writes the full-resolution `ao`.
    AoUpsample,
    /// The particle simulation (compute).
    Particles,
    /// The 2D light map.
    Light2d,
    /// The 2D fluids' metaball field.
    Metaballs,
    /// The 3D fluids' depth, thickness and smoothing (fluid3d.rs).
    Fluid3d,
    /// The reflection probes' captures due this frame (`probes.rs`).
    Probes,
    /// The scene mirrored in a water surface (its planar reflection): slot 0 the one material shaders read, slot 1 a
    /// second surface's (targets `reflection*`, `reflection.1*`).
    Reflection(u8),
    /// Secondary camera `i` of the frame's views (`views.rs`): its lit
    /// pass and its resolve into a render target (a viewport view resolves
    /// after the post chain).
    View(u32),
    /// Volumetric fog's froxel grid lit and summed (compute; full tier).
    Volumetric,
    /// Opaque meshes, sky, transparent meshes, particles, strips, sprites
    /// and text into the HDR target.
    Main,
    /// Screen-space reflections over the lit image (`ssr.rs`; the full tier).
    Ssr,
    /// Glowing sprites into the glow buffer.
    Glow,
    Outline,
    Taa,
    /// The sharpening after TAA (into `hdr.sharp`; the history stays soft).
    TaaSharpen,
    DofCoc,
    DofBlur,
    DofComposite,
    MotionBlur,
    /// Custom post pass `i` of the frame (`post_custom.rs`): a game's WGSL
    /// over the lit image, after motion blur, before exposure and bloom.
    CustomPost(u32),
    GodRaysMask,
    GodRaysBlur,
    Meter,
    /// Metering level `i` into `i + 1`.
    Reduce(u32),
    /// Temporal exposure adaptation: the metered exposure approached from
    /// the previous frame's (`Exposure::Auto` with adapt speeds).
    Adapt,
    BloomPrefilter,
    /// Bloom level `i` into `i + 1`.
    BloomDown(u32),
    /// Bloom level `i + 1` added onto `i`.
    BloomUp(u32),
    /// Flat colour bands by depth over the lit image (`stylise.rs`).
    DepthBands,
    Tonemap,
    /// The passes after the tonemap ([`chain_passes`]), each from one LDR
    /// target into the next, the last into the output: painterly
    /// smoothing, hatching/halftone/ink lines, posterise and palette
    /// quantisation, FXAA, and the pixel grid scaled up to the output.
    Paint,
    Lines,
    Quantize,
    /// FXAA over the tonemapped image (the GL tier's MSAA alternative).
    Fxaa,
    Upscale,
}

/// The passes after the tonemap that `passes` holds, in the order they run.
pub(crate) fn chain_passes(passes: &[FramePass]) -> Vec<FramePass> {
    [
        FramePass::Paint,
        FramePass::Lines,
        FramePass::Quantize,
        FramePass::Fxaa,
        FramePass::Upscale,
    ]
    .into_iter()
    .filter(|p| passes.contains(p))
    .collect()
}

/// The LDR target chain pass `i` reads (the tonemap writes `ldr.0`).
fn ldr_name(i: usize) -> String {
    format!("ldr.{i}")
}

impl FramePass {
    pub fn name(self) -> String {
        match self {
            FramePass::Cascade(i) => format!("cascade.{i}"),
            FramePass::LocalFace(i) => format!("local-shadow.{i}"),
            FramePass::Prepass => "prepass".into(),
            FramePass::Ao => "ao".into(),
            FramePass::AoBlur => "ao-blur".into(),
            FramePass::AoUpsample => "ao-upsample".into(),
            FramePass::Particles => "particles".into(),
            FramePass::Light2d => "lights-2d".into(),
            FramePass::Metaballs => "metaballs".into(),
            FramePass::Fluid3d => "fluid-3d".into(),
            FramePass::Probes => "reflection-probes".into(),
            FramePass::Reflection(k) => reflection_target(k, "").into(),
            FramePass::View(i) => format!("view-{i}"),
            FramePass::Volumetric => "volumetric".into(),
            FramePass::Main => "main".into(),
            FramePass::Ssr => "ssr".into(),
            FramePass::Glow => "glow".into(),
            FramePass::Outline => "outline".into(),
            FramePass::Taa => "taa".into(),
            FramePass::TaaSharpen => "taa-sharpen".into(),
            FramePass::DofCoc => "dof-coc".into(),
            FramePass::DofBlur => "dof-blur".into(),
            FramePass::DofComposite => "dof-composite".into(),
            FramePass::MotionBlur => "motion-blur".into(),
            FramePass::CustomPost(i) => format!("custom-post.{i}"),
            FramePass::GodRaysMask => "god-rays-mask".into(),
            FramePass::GodRaysBlur => "god-rays-blur".into(),
            FramePass::Meter => "meter".into(),
            FramePass::Reduce(i) => format!("meter-reduce.{i}"),
            FramePass::Adapt => "exposure-adapt".into(),
            FramePass::BloomPrefilter => "bloom-prefilter".into(),
            FramePass::BloomDown(i) => format!("bloom-down.{i}"),
            FramePass::BloomUp(i) => format!("bloom-up.{i}"),
            FramePass::Tonemap => "tonemap".into(),
            FramePass::DepthBands => "depth-bands".into(),
            FramePass::Paint => "paint".into(),
            FramePass::Lines => "lines".into(),
            FramePass::Quantize => "quantize".into(),
            FramePass::Fxaa => "fxaa".into(),
            FramePass::Upscale => "upscale".into(),
        }
    }
}

/// What the transient targets depend on besides the passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GraphConfig {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub taa: bool,
    /// The 2D light map's size.
    pub lightmap: (u32, u32),
    /// GTAO (or SSAO).
    pub gtao: bool,
    /// The main pass draws water or a 3D fluid: it copies the scene drawn
    /// so far.
    pub water: bool,
    /// The main pass copies the opaque scene for transmissive materials
    /// (into the renderer's own mip-chained target, not a graph target).
    pub transmission: bool,
    /// Metaball field targets the frame's fluids need (1 to
    /// `metaballs::FIELDS` when it draws any).
    pub fluid_fields: u32,
    /// The planar reflections' sizes (`Some` with [`FramePass::Reflection`] of that slot).
    pub reflection: [Option<(u32, u32)>; 2],
    /// Stylise lays a pattern in object space: the prepass also writes
    /// `object-space` (stylise_object.rs), which the paint and lines
    /// passes read.
    pub object_space: bool,
}

/// The metaball field targets, by field.
pub(crate) const FIELD_NAMES: [&str; crate::FIELDS] =
    ["metaballs", "metaballs.1", "metaballs.2", "metaballs.3"];

fn luminance_name(i: u32) -> String {
    format!("luminance.{i}")
}

fn bloom_name(i: u32) -> String {
    format!("bloom.{i}")
}

/// The names of the lit image as the effects hand it on: after the main
/// pass (and outlines), after TAA (`None`: the history; its sharpening:
/// `hdr.sharp`), after depth of field and motion blur.
struct LitNames {
    /// The lit image after the main pass and screen-space reflections.
    base: &'static str,
    /// The lit image after the main pass, reflections and the depth bands.
    lit: &'static str,
    main: &'static str,
    /// `None` when TAA writes the history and nothing sharpens it.
    aa: Option<&'static str>,
    after_dof: Option<&'static str>,
    /// The lit image the first custom post pass reads (after motion blur).
    before_custom: Option<&'static str>,
    last: Option<&'static str>,
}

/// The targets the custom post passes write, in order (`post_custom.rs`).
pub(crate) const CUSTOM_POST_NAMES: [&str; crate::post_custom::MAX_CUSTOM_POSTS] = [
    "hdr.post.0",
    "hdr.post.1",
    "hdr.post.2",
    "hdr.post.3",
    "hdr.post.4",
    "hdr.post.5",
    "hdr.post.6",
    "hdr.post.7",
];

fn lit_names(passes: &[FramePass]) -> LitNames {
    let has = |p| passes.contains(&p);
    let base = if has(FramePass::Ssr) {
        "hdr.ssr"
    } else {
        "hdr"
    };
    let lit = if has(FramePass::DepthBands) {
        "hdr.bands"
    } else {
        base
    };
    let main = if has(FramePass::Outline) {
        "hdr.outline"
    } else {
        lit
    };
    let aa = if has(FramePass::TaaSharpen) {
        Some("hdr.sharp")
    } else {
        (!has(FramePass::Taa)).then_some(main)
    };
    let after_dof = if has(FramePass::DofComposite) {
        Some("hdr.dof")
    } else {
        aa
    };
    let before_custom = if has(FramePass::MotionBlur) {
        Some("hdr.mb")
    } else {
        after_dof
    };
    let customs = passes
        .iter()
        .filter(|p| matches!(p, FramePass::CustomPost(_)))
        .count();
    let last = match customs {
        0 => before_custom,
        n => Some(CUSTOM_POST_NAMES[n - 1]),
    };
    LitNames {
        base,
        lit,
        main,
        aa,
        after_dof,
        before_custom,
        last,
    }
}

/// The render graph of a frame made of `passes` (in recording order).
pub(crate) fn build_graph(passes: &[FramePass], cfg: &GraphConfig) -> RenderGraph {
    let attach_bind = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
    let desc = |width: u32, height: u32, format, samples, usage| TargetDesc {
        width,
        height,
        format,
        samples,
        usage,
    };
    let (w, h) = (cfg.width, cfg.height);
    let (hw, hh) = ((w / 2).max(1), (h / 2).max(1));
    let mut g = RenderGraph::new();
    g.import("cascades")
        .import("local-shadows")
        .import("views")
        .import("output")
        .import("particles")
        .import("froxels")
        .transient(
            "hdr",
            desc(
                w,
                h,
                HDR_FORMAT,
                1,
                if cfg.water || cfg.transmission {
                    attach_bind | wgpu::TextureUsages::COPY_SRC
                } else {
                    attach_bind
                },
            ),
        );
    if cfg.water {
        g.transient(
            "scene-copy",
            desc(
                w,
                h,
                HDR_FORMAT,
                1,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            ),
        );
    }
    let msaa = cfg.samples > 1;
    if msaa {
        g.transient(
            "hdr-msaa",
            desc(
                w,
                h,
                HDR_FORMAT,
                cfg.samples,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            ),
        );
    }
    let depth_usage = if cfg.taa && !msaa {
        attach_bind
    } else {
        wgpu::TextureUsages::RENDER_ATTACHMENT
    };
    g.transient("depth", desc(w, h, DEPTH_FORMAT, cfg.samples, depth_usage));
    if cfg.taa {
        g.import("history-read").import("history-write");
    }
    let has = |p| passes.contains(&p);
    if has(FramePass::Prepass) {
        g.transient(
            "pre-depth",
            desc(
                w,
                h,
                DEPTH_FORMAT,
                1,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            ),
        )
        .transient("gbuffer", desc(w, h, GBUFFER_FORMAT, 1, attach_bind))
        .transient(
            "lindepth",
            desc(
                w,
                h,
                LINDEPTH_FORMAT,
                1,
                attach_bind | wgpu::TextureUsages::COPY_SRC,
            ),
        )
        .transient("outline-id", desc(w, h, OUTLINE_FORMAT, 1, attach_bind));
        if cfg.object_space {
            g.transient(
                "object-space",
                desc(w, h, crate::stylise_object::OBJECT_FORMAT, 1, attach_bind),
            );
        }
    }
    if has(FramePass::AoUpsample) {
        // Searched and blurred at half resolution (each texel the depth's
        // at twice its coordinate), then upsampled to `ao`.
        let (hw, hh) = (w.div_ceil(2), h.div_ceil(2));
        g.transient("ao.raw", desc(hw, hh, AO_FORMAT, 1, attach_bind))
            .transient("ao.blur", desc(hw, hh, AO_FORMAT, 1, attach_bind))
            .transient("ao", desc(w, h, AO_FORMAT, 1, attach_bind));
    } else if has(FramePass::Ao) {
        g.transient("ao.raw", desc(w, h, AO_FORMAT, 1, attach_bind))
            .transient("ao", desc(w, h, AO_FORMAT, 1, attach_bind));
    }
    if has(FramePass::Light2d) {
        let (lw, lh) = cfg.lightmap;
        g.transient("lightmap", desc(lw, lh, HDR_FORMAT, 1, attach_bind));
    }
    let fields = &FIELD_NAMES[..(cfg.fluid_fields as usize).clamp(1, FIELD_NAMES.len())];
    if has(FramePass::Metaballs) {
        for name in fields {
            g.transient(*name, desc(w, h, HDR_FORMAT, 1, attach_bind));
        }
    }
    if has(FramePass::Fluid3d) {
        g.transient("fluid.depth", desc(w, h, LINDEPTH_FORMAT, 1, attach_bind))
            .transient(
                "fluid.zbuf",
                desc(
                    w,
                    h,
                    DEPTH_FORMAT,
                    1,
                    wgpu::TextureUsages::RENDER_ATTACHMENT,
                ),
            )
            .transient("fluid.thick", desc(w, h, HDR_FORMAT, 1, attach_bind))
            .transient("fluid.blur", desc(w, h, LINDEPTH_FORMAT, 1, attach_bind))
            .transient("fluid.smooth", desc(w, h, LINDEPTH_FORMAT, 1, attach_bind));
    }
    for k in 0..2u8 {
        let Some((rw, rh)) = cfg.reflection[k as usize].filter(|_| has(FramePass::Reflection(k)))
        else {
            continue;
        };
        g.transient(
            reflection_target(k, ""),
            desc(rw, rh, HDR_FORMAT, 1, attach_bind),
        )
        .transient(
            reflection_target(k, "-depth"),
            desc(
                rw,
                rh,
                DEPTH_FORMAT,
                cfg.samples,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            ),
        );
        if msaa {
            g.transient(
                reflection_target(k, "-msaa"),
                desc(
                    rw,
                    rh,
                    HDR_FORMAT,
                    cfg.samples,
                    wgpu::TextureUsages::RENDER_ATTACHMENT,
                ),
            );
        }
    }
    if has(FramePass::Glow) {
        g.transient("glow", desc(hw, hh, HDR_FORMAT, 1, attach_bind));
    }
    let chain = chain_passes(passes);
    for i in 0..chain.len() {
        g.transient(ldr_name(i), desc(w, h, LDR_FORMAT, 1, attach_bind));
    }
    if has(FramePass::DepthBands) {
        g.transient("hdr.bands", desc(w, h, HDR_FORMAT, 1, attach_bind));
    }
    if has(FramePass::Ssr) {
        g.transient("hdr.ssr", desc(w, h, HDR_FORMAT, 1, attach_bind));
    }
    for (pass, name) in [
        (FramePass::Outline, "hdr.outline"),
        (FramePass::TaaSharpen, "hdr.sharp"),
        (FramePass::DofComposite, "hdr.dof"),
        (FramePass::MotionBlur, "hdr.mb"),
    ] {
        if has(pass) {
            g.transient(name, desc(w, h, HDR_FORMAT, 1, attach_bind));
        }
    }
    if has(FramePass::DofCoc) {
        g.transient("dof.coc", desc(hw, hh, HDR_FORMAT, 1, attach_bind))
            .transient("dof.blur", desc(hw, hh, HDR_FORMAT, 1, attach_bind));
    }
    let customs = passes
        .iter()
        .filter(|p| matches!(p, FramePass::CustomPost(_)))
        .count();
    for &name in &CUSTOM_POST_NAMES[..customs] {
        g.transient(name, desc(w, h, HDR_FORMAT, 1, attach_bind));
    }
    if has(FramePass::GodRaysMask) {
        g.transient("god.mask", desc(hw, hh, HDR_FORMAT, 1, attach_bind))
            .transient("god.blur", desc(hw, hh, HDR_FORMAT, 1, attach_bind));
    }
    let mut size = METER_SIZE;
    for i in 0..meter_levels() {
        g.transient(
            luminance_name(i),
            desc(size, size, LUMINANCE_FORMAT, 1, attach_bind),
        );
        size /= 8;
    }
    for i in 0..bloom_levels(w, h) {
        g.transient(
            bloom_name(i),
            desc(
                (hw >> i).max(1),
                (hh >> i).max(1),
                HDR_FORMAT,
                1,
                attach_bind,
            ),
        );
    }
    let names = lit_names(passes);
    let aa = names.aa.unwrap_or("history-write");
    let after_dof = names.after_dof.unwrap_or("history-write");
    let lit = names.last.unwrap_or("history-write");
    let auto = has(FramePass::Meter);
    let bloom = has(FramePass::BloomPrefilter);
    let metered = luminance_name(meter_levels() - 1);
    // What the exposure is read from: the adapted exposure when it adapts.
    let lum = if has(FramePass::Adapt) {
        g.import("exposure");
        "exposure".to_string()
    } else {
        metered.clone()
    };
    let prepass = has(FramePass::Prepass);
    // With water and AO: the prepass depth with the water's surfaces in it, which the AO reads (lib.rs, water.rs
    // `draw_ao_depth`); the prepass pass writes it after its own targets.
    let ao_depth = prepass && cfg.water && has(FramePass::Ao);
    if ao_depth {
        g.transient(
            "ao-depth",
            desc(
                w,
                h,
                LINDEPTH_FORMAT,
                1,
                attach_bind | wgpu::TextureUsages::COPY_DST,
            ),
        );
    }
    let ao_reads: &[&str] = if ao_depth {
        &["ao-depth", "gbuffer"]
    } else {
        &["lindepth", "gbuffer"]
    };
    for &p in passes {
        let name = p.name();
        match p {
            FramePass::Cascade(_) => g.pass(name, &[], &["cascades"]),
            FramePass::LocalFace(_) => g.pass(name, &[], &["local-shadows"]),
            FramePass::Prepass => {
                let mut writes = vec!["pre-depth", "gbuffer", "lindepth", "outline-id"];
                if cfg.object_space {
                    writes.push("object-space");
                }
                if ao_depth {
                    writes.push("ao-depth");
                }
                g.pass(name, &[], &writes)
            }
            FramePass::Ao => g.pass(name, ao_reads, &["ao.raw"]),
            FramePass::AoBlur if has(FramePass::AoUpsample) => {
                g.pass(name, &["ao.raw", "lindepth", "gbuffer"], &["ao.blur"])
            }
            FramePass::AoBlur => g.pass(name, &["ao.raw", "lindepth", "gbuffer"], &["ao"]),
            FramePass::AoUpsample => g.pass(name, &["ao.blur", "lindepth"], &["ao"]),
            FramePass::Particles if prepass => {
                g.pass(name, &["lindepth", "gbuffer"], &["particles"])
            }
            FramePass::Particles => g.pass(name, &[], &["particles"]),
            FramePass::Light2d => g.pass(name, &[], &["lightmap"]),
            FramePass::Metaballs => g.pass(name, &[], fields),
            FramePass::Fluid3d => g.pass(
                name,
                &["lindepth"],
                &[
                    "fluid.depth",
                    "fluid.zbuf",
                    "fluid.thick",
                    "fluid.blur",
                    "fluid.smooth",
                ],
            ),
            // Into the probes' own cubes, not graph targets.
            FramePass::Probes => g.pass(name, &["cascades", "local-shadows"], &[]),
            FramePass::Reflection(k) => {
                let mut writes = vec![reflection_target(k, ""), reflection_target(k, "-depth")];
                if msaa {
                    writes.push(reflection_target(k, "-msaa"));
                }
                g.pass(name, &["cascades", "local-shadows"], &writes)
            }
            // Views write their own targets (outside the graph); the
            // imported "views" orders them before the main pass, whose
            // materials may sample them.
            FramePass::View(_) => g.pass(name, &["cascades", "local-shadows"], &["views"]),
            FramePass::Volumetric => g.pass(name, &["cascades", "local-shadows"], &["froxels"]),
            FramePass::Main => {
                let mut reads = vec!["cascades", "local-shadows"];
                if passes.iter().any(|p| matches!(p, FramePass::View(_))) {
                    reads.push("views");
                }
                if has(FramePass::Volumetric) {
                    reads.push("froxels");
                }
                if has(FramePass::AoBlur) {
                    reads.push("ao");
                }
                if prepass {
                    reads.push("lindepth");
                }
                if has(FramePass::Particles) {
                    reads.push("particles");
                }
                if has(FramePass::Light2d) {
                    reads.push("lightmap");
                }
                if has(FramePass::Metaballs) {
                    reads.extend_from_slice(fields);
                }
                for k in 0..2u8 {
                    if has(FramePass::Reflection(k)) {
                        reads.push(reflection_target(k, ""));
                    }
                }
                if has(FramePass::Fluid3d) {
                    reads.extend(["fluid.smooth", "fluid.thick"]);
                }
                let mut writes = vec!["hdr", "depth"];
                if msaa {
                    writes.push("hdr-msaa");
                }
                if cfg.water {
                    writes.push("scene-copy");
                }
                g.pass(name, &reads, &writes)
            }
            FramePass::Glow => g.pass(name, &[], &["glow"]),
            FramePass::Ssr => g.pass(name, &["hdr", "lindepth", "gbuffer"], &["hdr.ssr"]),
            FramePass::DepthBands => g.pass(name, &[names.base, "lindepth"], &["hdr.bands"]),
            FramePass::Outline => g.pass(
                name,
                &[names.lit, "gbuffer", "lindepth", "outline-id"],
                &["hdr.outline"],
            ),
            FramePass::Taa => g.pass(
                name,
                &[names.main, "depth", "history-read"],
                &["history-write"],
            ),
            FramePass::TaaSharpen => g.pass(name, &["history-write", "depth"], &["hdr.sharp"]),
            // Every effect pass binds the prepass's lindepth and gbuffer
            // (`fx_group`), so each declares them: a target written here may
            // not alias a texture the pass still has bound.
            FramePass::DofCoc => g.pass(name, &[aa, "lindepth", "gbuffer"], &["dof.coc"]),
            FramePass::DofBlur => g.pass(name, &["dof.coc", "lindepth", "gbuffer"], &["dof.blur"]),
            FramePass::DofComposite => {
                g.pass(name, &[aa, "dof.blur", "lindepth", "gbuffer"], &["hdr.dof"])
            }
            FramePass::MotionBlur => g.pass(name, &[after_dof, "gbuffer", "lindepth"], &["hdr.mb"]),
            FramePass::CustomPost(i) => {
                let read = match i {
                    0 => names.before_custom.unwrap_or("history-write"),
                    i => CUSTOM_POST_NAMES[i as usize - 1],
                };
                g.pass(
                    name,
                    &[read, "lindepth", "gbuffer"],
                    &[CUSTOM_POST_NAMES[i as usize]],
                )
            }
            FramePass::GodRaysMask => g.pass(name, &[lit, "lindepth", "gbuffer"], &["god.mask"]),
            FramePass::GodRaysBlur => {
                g.pass(name, &["god.mask", "lindepth", "gbuffer"], &["god.blur"])
            }
            FramePass::Meter => g.pass(name, &[lit], &["luminance.0"]),
            FramePass::Reduce(i) => g.pass(name, &[&luminance_name(i)], &[&luminance_name(i + 1)]),
            FramePass::Adapt => g.pass(name, &[&metered], &["exposure"]),
            FramePass::BloomPrefilter => {
                let mut reads = vec![lit];
                if auto {
                    reads.push(&lum);
                }
                if has(FramePass::Glow) {
                    reads.push("glow");
                }
                g.pass(name, &reads, &["bloom.0"])
            }
            FramePass::BloomDown(i) => g.pass(name, &[&bloom_name(i)], &[&bloom_name(i + 1)]),
            FramePass::BloomUp(i) => g.pass(name, &[&bloom_name(i + 1)], &[&bloom_name(i)]),
            FramePass::Tonemap => {
                let mut reads = vec![lit];
                if auto {
                    reads.push(&lum);
                }
                if bloom {
                    reads.push("bloom.0");
                }
                if has(FramePass::GodRaysBlur) {
                    reads.push("god.blur");
                }
                let target = if chain.is_empty() {
                    "output".to_string()
                } else {
                    ldr_name(0)
                };
                g.pass(name, &reads, &[target.as_str()])
            }
            FramePass::Paint
            | FramePass::Lines
            | FramePass::Quantize
            | FramePass::Fxaa
            | FramePass::Upscale => {
                let i = chain.iter().position(|&c| c == p).expect("in the chain");
                let read = ldr_name(i);
                let write = if i + 1 == chain.len() {
                    "output".to_string()
                } else {
                    ldr_name(i + 1)
                };
                // The paint and lines passes bind the prepass
                // (`Bindings::chain`).
                let mut reads = vec![read.as_str()];
                if matches!(p, FramePass::Paint | FramePass::Lines) && prepass {
                    reads.extend(["lindepth", "gbuffer"]);
                    if cfg.object_space {
                        reads.push("object-space");
                    }
                }
                g.pass(name, &reads, &[write.as_str()])
            }
        };
    }
    g
}

/// The post passes' bind groups for one realization of the graph. Groups
/// in pairs are per TAA history slot.
struct Bindings {
    taa: Option<[wgpu::BindGroup; 2]>,
    taa_sharpen: Option<[wgpu::BindGroup; 2]>,
    /// The passes after the tonemap, each reading its LDR target.
    chain: HashMap<FramePass, wgpu::BindGroup>,
    meter: Option<[wgpu::BindGroup; 2]>,
    reduce: Vec<wgpu::BindGroup>,
    adapt: Option<[wgpu::BindGroup; 2]>,
    prefilter: Option<[wgpu::BindGroup; 2]>,
    down: Vec<wgpu::BindGroup>,
    up: Vec<wgpu::BindGroup>,
    post: [wgpu::BindGroup; 2],
    fx: HashMap<FramePass, [wgpu::BindGroup; 2]>,
}

/// The frame's render graph: the compiled plan for the current pass list,
/// the transient textures realized from it (kept in a pool across frames
/// and reused by description) and the bind groups that read them.
#[derive(Default)]
pub(crate) struct FrameGraph {
    passes: Vec<FramePass>,
    cfg: Option<GraphConfig>,
    /// Identifies the grading LUT the bindings were made with.
    lut_key: Option<(u64, usize)>,
    /// Identifies the palette and canvas the stylise bindings were made with.
    stylise_key: u64,
    order: Vec<FramePass>,
    plan: Option<GraphPlan>,
    pool: Vec<(TargetDesc, wgpu::TextureView)>,
    views: HashMap<String, wgpu::TextureView>,
    bindings: Option<Bindings>,
    /// Bumped whenever the targets are realized anew (views change).
    pub generation: u64,
}

impl FrameGraph {
    /// Forgets everything size-dependent (on resize).
    pub fn invalidate(&mut self) {
        let generation = self.generation + 1;
        *self = FrameGraph::default();
        self.generation = generation;
    }

    /// The plan the last [`FrameGraph::prepare`] compiled.
    pub fn plan(&self) -> Option<&GraphPlan> {
        self.plan.as_ref()
    }

    /// The passes in execution order.
    pub fn order(&self) -> &[FramePass] {
        &self.order
    }

    /// Physical transient textures of the current plan.
    pub fn transient_textures(&self) -> usize {
        self.plan.as_ref().map_or(0, |p| p.physical().len())
    }

    pub fn view(&self, name: &str) -> &wgpu::TextureView {
        self.views
            .get(name)
            .unwrap_or_else(|| panic!("render graph target '{name}' is not realized"))
    }

    /// What custom post pass `i` reads and writes (`post_custom.rs`): the
    /// lit image before it (the TAA history of `slot` when TAA wrote it
    /// last) and its own target.
    pub fn custom_post_views<'a>(
        &'a self,
        persistent: &'a Persistent,
        i: u32,
        slot: usize,
    ) -> (&'a wgpu::TextureView, &'a wgpu::TextureView) {
        let read = match i {
            0 => match lit_names(&self.passes).before_custom {
                Some(n) => self.view(n),
                None => &persistent.history.as_ref().expect("TAA history")[slot],
            },
            i => self.view(CUSTOM_POST_NAMES[i as usize - 1]),
        };
        (read, self.view(CUSTOM_POST_NAMES[i as usize]))
    }

    /// The view of `name` when the current graph has it.
    pub fn try_view(&self, name: &str) -> Option<&wgpu::TextureView> {
        self.views.get(name)
    }

    /// Compiles the graph for `passes` and realizes its transient targets,
    /// unless the previous frame had the same passes, configuration and LUT.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        passes: Vec<FramePass>,
        cfg: &GraphConfig,
        post: &PostPasses,
        persistent: &Persistent,
        lut: (Option<(u64, usize)>, &wgpu::TextureView),
        stylise: (&wgpu::TextureView, &wgpu::TextureView, u64),
    ) -> Result<(), GraphError> {
        if self.plan.is_some()
            && passes == self.passes
            && self.cfg.as_ref() == Some(cfg)
            && self.lut_key == lut.0
            && self.stylise_key == stylise.2
        {
            return Ok(());
        }
        let plan = build_graph(&passes, cfg).compile()?;
        self.order = plan.order().iter().map(|&i| passes[i]).collect();
        // Physical slots to pool textures: the first unused entry with the
        // same description, else a new texture.
        let mut taken = vec![false; self.pool.len()];
        let mut slot_views = Vec::with_capacity(plan.physical().len());
        for desc in plan.physical() {
            let found = (0..self.pool.len()).find(|&i| !taken[i] && self.pool[i].0 == *desc);
            let index = match found {
                Some(i) => i,
                None => {
                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("graph transient"),
                        size: wgpu::Extent3d {
                            width: desc.width,
                            height: desc.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: desc.samples,
                        dimension: wgpu::TextureDimension::D2,
                        format: desc.format,
                        usage: desc.usage,
                        view_formats: &[],
                    });
                    self.pool
                        .push((*desc, texture.create_view(&Default::default())));
                    taken.push(false);
                    self.pool.len() - 1
                }
            };
            taken[index] = true;
            slot_views.push(self.pool[index].1.clone());
        }
        self.views.clear();
        for name in transient_names(cfg) {
            if let Some(slot) = plan.slot(&name) {
                self.views.insert(name, slot_views[slot].clone());
            }
        }
        self.bindings = Some(self.bind(
            device,
            post,
            persistent,
            &passes,
            cfg,
            lut.1,
            (stylise.0, stylise.1),
        ));
        self.generation += 1;
        self.passes = passes;
        self.cfg = Some(*cfg);
        self.lut_key = lut.0;
        self.stylise_key = stylise.2;
        self.plan = Some(plan);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn bind(
        &self,
        device: &wgpu::Device,
        passes: &PostPasses,
        persistent: &Persistent,
        frame: &[FramePass],
        cfg: &GraphConfig,
        lut: &wgpu::TextureView,
        stylise: (&wgpu::TextureView, &wgpu::TextureView),
    ) -> Bindings {
        fn tex(v: &wgpu::TextureView) -> wgpu::BindingResource<'_> {
            wgpu::BindingResource::TextureView(v)
        }
        let sampler = wgpu::BindingResource::Sampler(&passes.sampler);
        let group = |label: &str, layout: &wgpu::BindGroupLayout, res: &[wgpu::BindingResource]| {
            let entries: Vec<_> = res
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
        };
        let has = |p: FramePass| frame.contains(&p);
        let names = lit_names(frame);
        let taa = has(FramePass::Taa);
        let history = persistent.history.as_ref().filter(|_| taa);
        // The lit image in frame slot `i` at each stage.
        let stage = |name: Option<&'static str>, slot: usize| -> &wgpu::TextureView {
            match name {
                Some(n) => self.view(n),
                None => &history.expect("TAA history")[slot],
            }
        };
        let source = |slot: usize| stage(names.last, slot);
        let taa_groups = history.map(|h| {
            [0usize, 1].map(|slot| {
                group(
                    "taa",
                    &passes.taa_layout,
                    &[
                        tex(self.view(names.main)),
                        tex(&h[1 - slot]),
                        tex(self.view("depth")),
                        sampler.clone(),
                        passes.taa_buffer.as_entire_binding(),
                        wgpu::BindingResource::Sampler(&passes.nearest_sampler),
                    ],
                )
            })
        });
        // The sharpening reads what TAA wrote this frame (as both images).
        let taa_sharpen = history.filter(|_| has(FramePass::TaaSharpen)).map(|h| {
            [0usize, 1].map(|slot| {
                group(
                    "taa sharpen",
                    &passes.taa_layout,
                    &[
                        tex(&h[slot]),
                        tex(&h[slot]),
                        tex(self.view("depth")),
                        sampler.clone(),
                        passes.taa_buffer.as_entire_binding(),
                        wgpu::BindingResource::Sampler(&passes.nearest_sampler),
                    ],
                )
            })
        });
        let mut chain = HashMap::new();
        // The paint and lines passes read the prepass (a canvas, hatching
        // or halftone on surfaces, ink on depth edges) when the frame has
        // one; the other passes bind placeholders.
        let prepass_of = |p: FramePass| {
            let lines = matches!(p, FramePass::Paint | FramePass::Lines);
            (
                self.try_view("lindepth")
                    .filter(|_| lines)
                    .unwrap_or(&passes.far_depth),
                self.try_view("gbuffer")
                    .filter(|_| lines)
                    .unwrap_or(&passes.black),
                self.try_view("object-space")
                    .filter(|_| lines)
                    .unwrap_or(&passes.no_object),
            )
        };
        for (i, p) in chain_passes(frame).into_iter().enumerate() {
            let source = self.view(&ldr_name(i));
            let (depth, normals, object) = prepass_of(p);
            let g = if p == FramePass::Fxaa {
                group(
                    "fxaa",
                    &passes.source_layout,
                    &[tex(source), sampler.clone()],
                )
            } else {
                group(
                    &p.name(),
                    &passes.stylise_layout,
                    &[
                        tex(source),
                        sampler.clone(),
                        passes.stylise_buffer.as_entire_binding(),
                        tex(stylise.0),
                        tex(stylise.1),
                        wgpu::BindingResource::Sampler(&passes.repeat_sampler),
                        tex(&passes.blue_noise),
                        tex(depth),
                        tex(normals),
                        tex(object),
                    ],
                )
            };
            chain.insert(p, g);
        }
        let levels = meter_levels();
        let auto = has(FramePass::Meter);
        let metered = if auto {
            self.view(&luminance_name(levels - 1))
        } else {
            &passes.black
        };
        // The exposure the passes of frame slot `i` read: the adapted one
        // this slot writes, or the metered texel.
        let adapting = has(FramePass::Adapt);
        let lum = |slot: usize| -> &wgpu::TextureView {
            if adapting {
                &passes.exposure.views[slot]
            } else {
                metered
            }
        };
        let adapt = adapting.then(|| {
            [0usize, 1].map(|slot| {
                group(
                    "exposure adapt",
                    &passes.adapt_layout,
                    &[
                        tex(metered),
                        tex(&passes.exposure.views[1 - slot]),
                        passes.adapt_buffer.as_entire_binding(),
                    ],
                )
            })
        });
        let meter = auto.then(|| {
            [0, 1].map(|slot| {
                group(
                    "meter",
                    &passes.source_layout,
                    &[tex(source(slot)), sampler.clone()],
                )
            })
        });
        let reduce = if auto {
            (0..levels - 1)
                .map(|i| {
                    group(
                        "reduce",
                        &passes.source_layout,
                        &[tex(self.view(&luminance_name(i))), sampler.clone()],
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        let god = if has(FramePass::GodRaysBlur) {
            self.view("god.blur")
        } else {
            &passes.black
        };
        let post_group =
            |label: &str, src: &wgpu::TextureView, bloom: &wgpu::TextureView, slot: usize| {
                group(
                    label,
                    &passes.post_layout,
                    &[
                        tex(src),
                        sampler.clone(),
                        passes.post_buffer.as_entire_binding(),
                        tex(lum(slot)),
                        tex(bloom),
                        tex(lut),
                        tex(god),
                    ],
                )
            };
        let bloom = has(FramePass::BloomPrefilter);
        let bloom_count = frame
            .iter()
            .filter(|p| matches!(p, FramePass::BloomDown(_)))
            .count() as u32
            + 1;
        let glow = if has(FramePass::Glow) {
            self.view("glow")
        } else {
            &passes.black
        };
        let prefilter = bloom
            .then(|| [0, 1].map(|slot| post_group("bloom prefilter", source(slot), glow, slot)));
        let (down, up) = if bloom {
            (
                (0..bloom_count - 1)
                    .map(|i| post_group("bloom down", self.view(&bloom_name(i)), &passes.black, 0))
                    .collect(),
                (1..bloom_count)
                    .map(|i| post_group("bloom up", self.view(&bloom_name(i)), &passes.black, 0))
                    .collect(),
            )
        } else {
            (Vec::new(), Vec::new())
        };
        let bloom_top = if bloom {
            self.view("bloom.0")
        } else {
            &passes.black
        };
        let post = [0, 1].map(|slot| post_group("post", source(slot), bloom_top, slot));

        // The effect passes: (a, depth, gbuffer, b) per slot.
        let depth = self.try_view("lindepth").unwrap_or(&passes.far_depth);
        let gbuffer = self.try_view("gbuffer").unwrap_or(&passes.black);
        let fx_group = |a: &wgpu::TextureView, b: &wgpu::TextureView| {
            group(
                "fx post",
                &passes.fx_layout,
                &[
                    passes.fx_buffer.as_entire_binding(),
                    sampler.clone(),
                    tex(a),
                    tex(depth),
                    tex(gbuffer),
                    tex(b),
                    wgpu::BindingResource::Sampler(&passes.nearest_sampler),
                ],
            )
        };
        let black = &passes.black;
        let mut fx = HashMap::new();
        for &p in frame {
            let pair = |f: &dyn Fn(usize) -> wgpu::BindGroup| [f(0), f(1)];
            let groups = match p {
                // The AO reads the depth with the water's surfaces in it when there is one.
                FramePass::Ao => match self.try_view("ao-depth") {
                    Some(d) => pair(&|_| {
                        group(
                            "fx post",
                            &passes.fx_layout,
                            &[
                                passes.fx_buffer.as_entire_binding(),
                                sampler.clone(),
                                tex(black),
                                tex(d),
                                tex(gbuffer),
                                tex(black),
                            ],
                        )
                    }),
                    None => pair(&|_| fx_group(black, black)),
                },
                FramePass::AoBlur => pair(&|_| fx_group(black, self.view("ao.raw"))),
                FramePass::AoUpsample => pair(&|_| fx_group(black, self.view("ao.blur"))),
                FramePass::Ssr => pair(&|_| fx_group(self.view("hdr"), black)),
                FramePass::DepthBands => pair(&|_| fx_group(self.view(names.base), black)),
                FramePass::Outline => {
                    pair(&|_| fx_group(self.view(names.lit), self.view("outline-id")))
                }
                FramePass::DofCoc => pair(&|slot| fx_group(stage(names.aa, slot), black)),
                FramePass::DofBlur => pair(&|_| fx_group(self.view("dof.coc"), black)),
                FramePass::DofComposite => {
                    pair(&|slot| fx_group(stage(names.aa, slot), self.view("dof.blur")))
                }
                FramePass::MotionBlur => {
                    pair(&|slot| fx_group(stage(names.after_dof, slot), black))
                }
                FramePass::GodRaysMask => pair(&|slot| fx_group(source(slot), black)),
                FramePass::GodRaysBlur => pair(&|_| fx_group(black, self.view("god.mask"))),
                _ => continue,
            };
            fx.insert(p, groups);
        }
        let _ = cfg;
        Bindings {
            taa: taa_groups,
            taa_sharpen,
            chain,
            meter,
            reduce,
            adapt,
            prefilter,
            down,
            up,
            post,
            fx,
        }
    }

    /// Records one post or effect pass; returns its draws.
    #[allow(clippy::too_many_arguments)]
    pub fn encode_post(
        &self,
        encoder: &mut crate::Encoder,
        passes: &PostPasses,
        persistent: &Persistent,
        pass: FramePass,
        slot: usize,
        output: &wgpu::TextureView,
        gtao: bool,
    ) -> u32 {
        let b = self.bindings.as_ref().expect("prepared");
        let clear = wgpu::LoadOp::Clear(wgpu::Color::BLACK);
        let effect = match pass {
            FramePass::Ao => Some((if gtao { "fs_gtao" } else { "fs_ssao" }, "ao.raw")),
            // Half resolution: the blur's depth is at twice its texel, which
            // gathers do not read.
            FramePass::AoBlur if self.views.contains_key("ao.blur") => {
                Some(("fs_ao_blur", "ao.blur"))
            }
            FramePass::AoBlur if passes.fx.contains_key(AO_BLUR_GATHER) => {
                Some((AO_BLUR_GATHER, "ao"))
            }
            FramePass::AoBlur => Some(("fs_ao_blur", "ao")),
            FramePass::AoUpsample => Some(("fs_ao_upsample", "ao")),
            FramePass::Outline => Some(("fs_outline", "hdr.outline")),
            FramePass::DofCoc => Some(("fs_dof_coc", "dof.coc")),
            FramePass::DofBlur => Some(("fs_dof_blur", "dof.blur")),
            FramePass::DofComposite => Some(("fs_dof_composite", "hdr.dof")),
            FramePass::MotionBlur => Some(("fs_motion_blur", "hdr.mb")),
            FramePass::GodRaysMask => Some(("fs_godrays_mask", "god.mask")),
            FramePass::GodRaysBlur => Some(("fs_godrays_blur", "god.blur")),
            FramePass::DepthBands => Some(("fs_depth_bands", "hdr.bands")),
            FramePass::Ssr => Some(("fs_ssr", "hdr.ssr")),
            _ => None,
        };
        if let Some((entry, target)) = effect {
            fullscreen_pass(
                encoder,
                entry,
                self.view(target),
                clear,
                &passes.fx[entry],
                &b.fx[&pass][slot],
            );
            return 1;
        }
        match pass {
            FramePass::Taa => {
                if let (Some(pipeline), Some(history), Some(groups)) =
                    (&passes.taa, &persistent.history, &b.taa)
                {
                    fullscreen_pass(
                        encoder,
                        "taa",
                        &history[slot],
                        clear,
                        pipeline,
                        &groups[slot],
                    );
                    return 1;
                }
                0
            }
            FramePass::Meter => {
                let groups = b.meter.as_ref().expect("metering bound");
                fullscreen_pass(
                    encoder,
                    "meter",
                    self.view("luminance.0"),
                    clear,
                    &passes.meter,
                    &groups[slot],
                );
                1
            }
            FramePass::Reduce(i) => {
                fullscreen_pass(
                    encoder,
                    "meter reduce",
                    self.view(&luminance_name(i + 1)),
                    clear,
                    &passes.reduce,
                    &b.reduce[i as usize],
                );
                1
            }
            FramePass::Adapt => {
                let groups = b.adapt.as_ref().expect("adaptation bound");
                fullscreen_pass(
                    encoder,
                    "exposure adapt",
                    &passes.exposure.views[slot],
                    clear,
                    &passes.adapt,
                    &groups[slot],
                );
                1
            }
            FramePass::BloomPrefilter => {
                let groups = b.prefilter.as_ref().expect("bloom bound");
                fullscreen_pass(
                    encoder,
                    "bloom prefilter",
                    self.view("bloom.0"),
                    clear,
                    &passes.prefilter,
                    &groups[slot],
                );
                1
            }
            FramePass::BloomDown(i) => {
                fullscreen_pass(
                    encoder,
                    "bloom down",
                    self.view(&bloom_name(i + 1)),
                    clear,
                    &passes.down,
                    &b.down[i as usize],
                );
                1
            }
            FramePass::BloomUp(i) => {
                fullscreen_pass(
                    encoder,
                    "bloom up",
                    self.view(&bloom_name(i)),
                    wgpu::LoadOp::Load,
                    &passes.up,
                    &b.up[i as usize],
                );
                1
            }
            FramePass::Tonemap => {
                let chain = chain_passes(&self.passes);
                match (&passes.tonemap_ldr, chain.is_empty()) {
                    (Some(tonemap_ldr), false) => fullscreen_pass(
                        encoder,
                        "tonemap",
                        self.view(&ldr_name(0)),
                        clear,
                        tonemap_ldr,
                        &b.post[slot],
                    ),
                    _ => fullscreen_pass(
                        encoder,
                        "tonemap",
                        output,
                        clear,
                        &passes.tonemap,
                        &b.post[slot],
                    ),
                }
                1
            }
            FramePass::Paint
            | FramePass::Lines
            | FramePass::Quantize
            | FramePass::Fxaa
            | FramePass::Upscale => {
                let chain = chain_passes(&self.passes);
                let i = chain.iter().position(|&p| p == pass).expect("in the chain");
                let last = i + 1 == chain.len();
                match (passes.chain.get(&pass), b.chain.get(&pass)) {
                    (Some((to_ldr, to_output)), Some(group)) => {
                        let (target, pipeline) = if last {
                            (output, to_output)
                        } else {
                            (self.view(&ldr_name(i + 1)), to_ldr)
                        };
                        fullscreen_pass(encoder, &pass.name(), target, clear, pipeline, group);
                        1
                    }
                    _ => 0,
                }
            }
            FramePass::TaaSharpen => match (&passes.taa_sharpen_pipeline, &b.taa_sharpen) {
                (Some(pipeline), Some(groups)) => {
                    fullscreen_pass(
                        encoder,
                        "taa sharpen",
                        self.view("hdr.sharp"),
                        clear,
                        pipeline,
                        &groups[slot],
                    );
                    1
                }
                _ => 0,
            },
            FramePass::Ao
            | FramePass::AoBlur
            | FramePass::AoUpsample
            | FramePass::Outline
            | FramePass::DofCoc
            | FramePass::DofBlur
            | FramePass::DofComposite
            | FramePass::MotionBlur
            | FramePass::CustomPost(_)
            | FramePass::GodRaysMask
            | FramePass::GodRaysBlur
            | FramePass::Cascade(_)
            | FramePass::LocalFace(_)
            | FramePass::Prepass
            | FramePass::Particles
            | FramePass::Light2d
            | FramePass::Metaballs
            | FramePass::Fluid3d
            | FramePass::Probes
            | FramePass::Reflection(_)
            | FramePass::View(_)
            | FramePass::Volumetric
            | FramePass::Main
            | FramePass::DepthBands
            | FramePass::Ssr
            | FramePass::Glow => 0,
        }
    }
}

/// The graph target of planar reflection slot `k` (0: `reflection`, 1: `reflection.1`) with `suffix`.
pub(crate) fn reflection_target(k: u8, suffix: &str) -> &'static str {
    match (k, suffix) {
        (0, "") => "reflection",
        (0, "-depth") => "reflection-depth",
        (0, "-msaa") => "reflection-msaa",
        (_, "") => "reflection.1",
        (_, "-depth") => "reflection.1-depth",
        _ => "reflection.1-msaa",
    }
}

/// Every transient target name a graph for `cfg` can declare.
fn transient_names(cfg: &GraphConfig) -> Vec<String> {
    let mut names: Vec<String> = [
        "hdr",
        "hdr-msaa",
        "scene-copy",
        "depth",
        "pre-depth",
        "gbuffer",
        "lindepth",
        "ao-depth",
        "outline-id",
        "object-space",
        "ao.raw",
        "ao.blur",
        "ao",
        "lightmap",
        "metaballs",
        "metaballs.1",
        "metaballs.2",
        "metaballs.3",
        "fluid.depth",
        "fluid.zbuf",
        "fluid.thick",
        "fluid.blur",
        "fluid.smooth",
        "reflection",
        "reflection-depth",
        "reflection-msaa",
        "reflection.1",
        "reflection.1-depth",
        "reflection.1-msaa",
        "glow",
        "ldr.0",
        "ldr.1",
        "ldr.2",
        "ldr.3",
        "ldr.4",
        "hdr.bands",
        "hdr.ssr",
        "hdr.sharp",
        "hdr.outline",
        "hdr.dof",
        "hdr.mb",
        "dof.coc",
        "dof.blur",
        "god.mask",
        "god.blur",
    ]
    .iter()
    .chain(CUSTOM_POST_NAMES.iter())
    .map(|s| s.to_string())
    .collect();
    names.extend((0..meter_levels()).map(luminance_name));
    names.extend((0..bloom_levels(cfg.width, cfg.height)).map(bloom_name));
    names
}

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

/// The post passes' modules by label, as [`PostPasses::new`] compiles them
/// on the full tier (`full`) or the GL tier.
pub(crate) fn post_sources(full: bool) -> Vec<(&'static str, String)> {
    let fx = fx_source(full);
    [
        ("post", POST_WGSL),
        ("luminance", LUMINANCE_WGSL),
        ("exposure", EXPOSURE_WGSL),
        ("taa", TAA_WGSL),
        ("fxaa", FXAA_WGSL),
        ("stylise", STYLISE_WGSL),
        ("fx post", fx.as_str()),
    ]
    .into_iter()
    .map(|(label, body)| (label, fullscreen_source(body)))
    .collect()
}
