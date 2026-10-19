//! After the main pass: outlines, TAA, depth of field, motion blur, god
//! rays, auto-exposure metering, bloom (and the 2D glow), the tonemap with
//! grading, vignette, chromatic aberration, grain and flash; before it, the
//! ambient occlusion over the depth-normal prepass. Also the size-dependent
//! targets they and the main pass draw into, as render graph targets.

use crate::graph::{GraphError, GraphPlan, RenderGraph, TargetDesc};
use crate::scene::{Exposure, PostEffects, Tonemap};
use crate::{Antialias, DEPTH_FORMAT, HDR_FORMAT, target_texture};
use std::collections::HashMap;

const FULLSCREEN_WGSL: &str = include_str!("../shaders/fullscreen.wgsl");
const POST_WGSL: &str = include_str!("../shaders/post.wgsl");
const TAA_WGSL: &str = include_str!("../shaders/taa.wgsl");
const LUMINANCE_WGSL: &str = include_str!("../shaders/luminance.wgsl");
const FXPOST_WGSL: &str = include_str!("../shaders/fxpost.wgsl");

/// The prepass targets: world normal and velocity, view depth, outline ids.
pub(crate) const GBUFFER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub(crate) const LINDEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;
pub(crate) const OUTLINE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub(crate) const AO_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
/// The view depth the prepass clears to (sky).
pub(crate) const FAR_DEPTH: f32 = 1.0e9;

/// The metering grid of auto exposure (reduced to 1x1 by 8x8 averages).
const METER_SIZE: u32 = 64;
const LUMINANCE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;
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
    pub outline_color: [f32; 4],
    pub outline: [f32; 4],
    pub dof: [f32; 4],
    pub dof2: [f32; 4],
    pub mb: [f32; 4],
    pub god: [f32; 4],
    pub god2: [f32; 4],
    pub god_tint: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct TaaUniform {
    pub inv_view_proj: [[f32; 4]; 4],
    pub cur_view_proj: [[f32; 4]; 4],
    pub prev_view_proj: [[f32; 4]; 4],
    pub params: [f32; 4],
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
        cache: None,
    })
}

/// Records one fullscreen pass into `target`.
pub(crate) fn fullscreen_pass(
    encoder: &mut wgpu::CommandEncoder,
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
    meter: wgpu::RenderPipeline,
    reduce: wgpu::RenderPipeline,
    prefilter: wgpu::RenderPipeline,
    down: wgpu::RenderPipeline,
    up: wgpu::RenderPipeline,
    tonemap: wgpu::RenderPipeline,
    taa: Option<wgpu::RenderPipeline>,
    sampler: wgpu::Sampler,
    black: wgpu::TextureView,
    /// The identity 32³ grading LUT (bound when no grading is set).
    pub identity_lut: wgpu::TextureView,
    /// 1x1 placeholders: view depth at [`FAR_DEPTH`], white AO.
    pub far_depth: wgpu::TextureView,
    pub white: wgpu::TextureView,
    post_buffer: wgpu::Buffer,
    pub taa_buffer: wgpu::Buffer,
    fx_layout: wgpu::BindGroupLayout,
    pub fx_buffer: wgpu::Buffer,
    fx: HashMap<&'static str, wgpu::RenderPipeline>,
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
    use wgpu::util::DeviceExt;
    device
        .create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("placeholder"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
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

/// The fullscreen effect passes of fxpost.wgsl: (entry point, target format).
const FX_PASSES: &[(&str, wgpu::TextureFormat)] = &[
    ("fs_gtao", AO_FORMAT),
    ("fs_ssao", AO_FORMAT),
    ("fs_ao_blur", AO_FORMAT),
    ("fs_outline", HDR_FORMAT),
    ("fs_dof_coc", HDR_FORMAT),
    ("fs_dof_blur", HDR_FORMAT),
    ("fs_dof_composite", HDR_FORMAT),
    ("fs_motion_blur", HDR_FORMAT),
    ("fs_godrays_mask", HDR_FORMAT),
    ("fs_godrays_blur", HDR_FORMAT),
];

impl PostPasses {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_format: wgpu::TextureFormat,
        antialias: Antialias,
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
            ],
        });
        let module = |label: &str, body: &str| {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(format!("{FULLSCREEN_WGSL}{body}").into()),
            })
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
        let meter = fullscreen_pipeline(
            device,
            "meter",
            &luminance_shader,
            "fs_meter",
            &source_pl,
            LUMINANCE_FORMAT,
            None,
        );
        let reduce = fullscreen_pipeline(
            device,
            "meter reduce",
            &luminance_shader,
            "fs_reduce",
            &source_pl,
            LUMINANCE_FORMAT,
            None,
        );
        let prefilter = fullscreen_pipeline(
            device,
            "bloom prefilter",
            &post_shader,
            "fs_bloom_prefilter",
            &post_pl,
            HDR_FORMAT,
            None,
        );
        let down = fullscreen_pipeline(
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
        let up = fullscreen_pipeline(
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
        let tonemap = fullscreen_pipeline(
            device,
            "tonemap",
            &post_shader,
            tonemap_entry,
            &post_pl,
            output_format,
            None,
        );
        let taa = (antialias == Antialias::Taa).then(|| {
            let taa_shader = module("taa", TAA_WGSL);
            fullscreen_pipeline(
                device,
                "taa",
                &taa_shader,
                "fs",
                &pipeline_layout("taa", &taa_layout),
                HDR_FORMAT,
                None,
            )
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("post"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
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
        let fx_shader = module("fx post", FXPOST_WGSL);
        let fx_pl = pipeline_layout("fx post", &fx_layout);
        let fx = FX_PASSES
            .iter()
            .map(|&(entry, format)| {
                (
                    entry,
                    fullscreen_pipeline(device, entry, &fx_shader, entry, &fx_pl, format, None),
                )
            })
            .collect();
        let identity_lut = lut_texture(device, queue, &crate::ColorLut::identity());
        let far_depth = one_texel(
            device,
            queue,
            LINDEPTH_FORMAT,
            bytemuck::bytes_of(&FAR_DEPTH),
        );
        let white = one_texel(device, queue, AO_FORMAT, &[255]);
        PostPasses {
            identity_lut,
            far_depth,
            white,
            fx_layout,
            fx_buffer: uniform("fx post", std::mem::size_of::<FxPostUniform>()),
            fx,
            source_layout,
            post_layout,
            taa_layout,
            meter,
            reduce,
            prefilter,
            down,
            up,
            tonemap,
            taa,
            sampler,
            black: black_texture.create_view(&Default::default()),
            post_buffer: uniform("post", std::mem::size_of::<PostUniform>()),
            taa_buffer: uniform("taa", std::mem::size_of::<TaaUniform>()),
        }
    }

    /// Writes the post uniform. `glow`: the glow buffer feeds the bloom
    /// chain; `god_rays`: their texture is added; `tick` seeds the grain.
    pub fn write_uniform(
        &self,
        queue: &wgpu::Queue,
        exposure: Exposure,
        post: &PostEffects,
        glow: bool,
        god_rays: bool,
        tick: u64,
    ) {
        let (exposure, compensation) = match exposure {
            Exposure::Manual { ev } => ([ev, 0.0, 0.0, 0.0], 0.0),
            Exposure::Auto {
                compensation,
                min_ev,
                max_ev,
            } => ([0.0, 1.0, min_ev, max_ev.max(min_ev)], compensation),
        };
        let lit_bloom = post.bloom.filter(|b| b.intensity > 0.0);
        let bloom = match (lit_bloom, post.glow.filter(|_| glow)) {
            (Some(b), _) => [b.threshold, b.knee, b.intensity, compensation],
            (None, Some(g)) => [1.0, 0.5, g.intensity.max(0.0), compensation],
            (None, None) => [1.0, 0.5, 0.0, compensation],
        };
        let tonemap = match post.tonemap {
            Tonemap::Aces => 0.0,
            Tonemap::None => 1.0,
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
            extra: [god_rays as u32 as f32, 0.0, 0.0, 0.0],
        };
        queue.write_buffer(&self.post_buffer, 0, bytemuck::bytes_of(&uniform));
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
            usage: attach_bind | wgpu::TextureUsages::COPY_SRC,
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
    /// The particle simulation (compute).
    Particles,
    /// The 2D light map.
    Light2d,
    /// The 2D fluids' metaball field.
    Metaballs,
    /// The scene mirrored in a water surface (its planar reflection).
    Reflection,
    /// Opaque meshes, sky, transparent meshes, particles, strips, sprites
    /// and text into the HDR target.
    Main,
    /// Glowing sprites into the glow buffer.
    Glow,
    Outline,
    Taa,
    DofCoc,
    DofBlur,
    DofComposite,
    MotionBlur,
    GodRaysMask,
    GodRaysBlur,
    Meter,
    /// Metering level `i` into `i + 1`.
    Reduce(u32),
    BloomPrefilter,
    /// Bloom level `i` into `i + 1`.
    BloomDown(u32),
    /// Bloom level `i + 1` added onto `i`.
    BloomUp(u32),
    Tonemap,
}

impl FramePass {
    pub fn name(self) -> String {
        match self {
            FramePass::Cascade(i) => format!("cascade.{i}"),
            FramePass::LocalFace(i) => format!("local-shadow.{i}"),
            FramePass::Prepass => "prepass".into(),
            FramePass::Ao => "ao".into(),
            FramePass::AoBlur => "ao-blur".into(),
            FramePass::Particles => "particles".into(),
            FramePass::Light2d => "lights-2d".into(),
            FramePass::Metaballs => "metaballs".into(),
            FramePass::Reflection => "reflection".into(),
            FramePass::Main => "main".into(),
            FramePass::Glow => "glow".into(),
            FramePass::Outline => "outline".into(),
            FramePass::Taa => "taa".into(),
            FramePass::DofCoc => "dof-coc".into(),
            FramePass::DofBlur => "dof-blur".into(),
            FramePass::DofComposite => "dof-composite".into(),
            FramePass::MotionBlur => "motion-blur".into(),
            FramePass::GodRaysMask => "god-rays-mask".into(),
            FramePass::GodRaysBlur => "god-rays-blur".into(),
            FramePass::Meter => "meter".into(),
            FramePass::Reduce(i) => format!("meter-reduce.{i}"),
            FramePass::BloomPrefilter => "bloom-prefilter".into(),
            FramePass::BloomDown(i) => format!("bloom-down.{i}"),
            FramePass::BloomUp(i) => format!("bloom-up.{i}"),
            FramePass::Tonemap => "tonemap".into(),
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
    /// The main pass draws water: it copies the scene drawn so far.
    pub water: bool,
    /// The planar reflection's size (`Some` with [`FramePass::Reflection`]).
    pub reflection: Option<(u32, u32)>,
}

fn luminance_name(i: u32) -> String {
    format!("luminance.{i}")
}

fn bloom_name(i: u32) -> String {
    format!("bloom.{i}")
}

/// The names of the lit image as the effects hand it on: after the main
/// pass (and outlines), after TAA (`None`: the history), after depth of
/// field and motion blur.
struct LitNames {
    main: &'static str,
    /// `None` when TAA writes the history.
    aa: Option<&'static str>,
    after_dof: Option<&'static str>,
    last: Option<&'static str>,
}

fn lit_names(passes: &[FramePass]) -> LitNames {
    let has = |p| passes.contains(&p);
    let main = if has(FramePass::Outline) {
        "hdr.outline"
    } else {
        "hdr"
    };
    let aa = (!has(FramePass::Taa)).then_some(main);
    let after_dof = if has(FramePass::DofComposite) {
        Some("hdr.dof")
    } else {
        aa
    };
    let last = if has(FramePass::MotionBlur) {
        Some("hdr.mb")
    } else {
        after_dof
    };
    LitNames {
        main,
        aa,
        after_dof,
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
        .import("output")
        .import("particles")
        .transient(
            "hdr",
            desc(
                w,
                h,
                HDR_FORMAT,
                1,
                if cfg.water {
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
        .transient("lindepth", desc(w, h, LINDEPTH_FORMAT, 1, attach_bind))
        .transient("outline-id", desc(w, h, OUTLINE_FORMAT, 1, attach_bind));
    }
    if has(FramePass::Ao) {
        g.transient("ao.raw", desc(w, h, AO_FORMAT, 1, attach_bind))
            .transient("ao", desc(w, h, AO_FORMAT, 1, attach_bind));
    }
    if has(FramePass::Light2d) {
        let (lw, lh) = cfg.lightmap;
        g.transient("lightmap", desc(lw, lh, HDR_FORMAT, 1, attach_bind));
    }
    if has(FramePass::Metaballs) {
        g.transient("metaballs", desc(w, h, HDR_FORMAT, 1, attach_bind));
    }
    if let (true, Some((rw, rh))) = (has(FramePass::Reflection), cfg.reflection) {
        g.transient("reflection", desc(rw, rh, HDR_FORMAT, 1, attach_bind))
            .transient(
                "reflection-depth",
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
                "reflection-msaa",
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
    for (pass, name) in [
        (FramePass::Outline, "hdr.outline"),
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
    let lum = luminance_name(meter_levels() - 1);
    let prepass = has(FramePass::Prepass);
    for &p in passes {
        let name = p.name();
        match p {
            FramePass::Cascade(_) => g.pass(name, &[], &["cascades"]),
            FramePass::LocalFace(_) => g.pass(name, &[], &["local-shadows"]),
            FramePass::Prepass => g.pass(
                name,
                &[],
                &["pre-depth", "gbuffer", "lindepth", "outline-id"],
            ),
            FramePass::Ao => g.pass(name, &["lindepth", "gbuffer"], &["ao.raw"]),
            FramePass::AoBlur => g.pass(name, &["ao.raw", "lindepth", "gbuffer"], &["ao"]),
            FramePass::Particles if prepass => {
                g.pass(name, &["lindepth", "gbuffer"], &["particles"])
            }
            FramePass::Particles => g.pass(name, &[], &["particles"]),
            FramePass::Light2d => g.pass(name, &[], &["lightmap"]),
            FramePass::Metaballs => g.pass(name, &[], &["metaballs"]),
            FramePass::Reflection => {
                let mut writes = vec!["reflection", "reflection-depth"];
                if msaa {
                    writes.push("reflection-msaa");
                }
                g.pass(name, &["cascades", "local-shadows"], &writes)
            }
            FramePass::Main => {
                let mut reads = vec!["cascades", "local-shadows"];
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
                    reads.push("metaballs");
                }
                if has(FramePass::Reflection) {
                    reads.push("reflection");
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
            FramePass::Outline => g.pass(
                name,
                &["hdr", "gbuffer", "lindepth", "outline-id"],
                &["hdr.outline"],
            ),
            FramePass::Taa => g.pass(
                name,
                &[names.main, "depth", "history-read"],
                &["history-write"],
            ),
            // Every effect pass binds the prepass's lindepth and gbuffer
            // (`fx_group`), so each declares them: a target written here may
            // not alias a texture the pass still has bound.
            FramePass::DofCoc => g.pass(name, &[aa, "lindepth", "gbuffer"], &["dof.coc"]),
            FramePass::DofBlur => g.pass(name, &["dof.coc", "lindepth", "gbuffer"], &["dof.blur"]),
            FramePass::DofComposite => {
                g.pass(name, &[aa, "dof.blur", "lindepth", "gbuffer"], &["hdr.dof"])
            }
            FramePass::MotionBlur => g.pass(name, &[after_dof, "gbuffer", "lindepth"], &["hdr.mb"]),
            FramePass::GodRaysMask => g.pass(name, &[lit, "lindepth", "gbuffer"], &["god.mask"]),
            FramePass::GodRaysBlur => {
                g.pass(name, &["god.mask", "lindepth", "gbuffer"], &["god.blur"])
            }
            FramePass::Meter => g.pass(name, &[lit], &["luminance.0"]),
            FramePass::Reduce(i) => g.pass(name, &[&luminance_name(i)], &[&luminance_name(i + 1)]),
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
                g.pass(name, &reads, &["output"])
            }
        };
    }
    g
}

/// The post passes' bind groups for one realization of the graph. Groups
/// in pairs are per TAA history slot.
struct Bindings {
    taa: Option<[wgpu::BindGroup; 2]>,
    meter: Option<[wgpu::BindGroup; 2]>,
    reduce: Vec<wgpu::BindGroup>,
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
    ) -> Result<(), GraphError> {
        if self.plan.is_some()
            && passes == self.passes
            && self.cfg.as_ref() == Some(cfg)
            && self.lut_key == lut.0
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
        self.bindings = Some(self.bind(device, post, persistent, &passes, cfg, lut.1));
        self.generation += 1;
        self.passes = passes;
        self.cfg = Some(*cfg);
        self.lut_key = lut.0;
        self.plan = Some(plan);
        Ok(())
    }

    fn bind(
        &self,
        device: &wgpu::Device,
        passes: &PostPasses,
        persistent: &Persistent,
        frame: &[FramePass],
        cfg: &GraphConfig,
        lut: &wgpu::TextureView,
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
                    ],
                )
            })
        });
        let levels = meter_levels();
        let auto = has(FramePass::Meter);
        let lum = if auto {
            self.view(&luminance_name(levels - 1))
        } else {
            &passes.black
        };
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
        let post_group = |label: &str, src: &wgpu::TextureView, bloom: &wgpu::TextureView| {
            group(
                label,
                &passes.post_layout,
                &[
                    tex(src),
                    sampler.clone(),
                    passes.post_buffer.as_entire_binding(),
                    tex(lum),
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
        let prefilter =
            bloom.then(|| [0, 1].map(|slot| post_group("bloom prefilter", source(slot), glow)));
        let (down, up) = if bloom {
            (
                (0..bloom_count - 1)
                    .map(|i| post_group("bloom down", self.view(&bloom_name(i)), &passes.black))
                    .collect(),
                (1..bloom_count)
                    .map(|i| post_group("bloom up", self.view(&bloom_name(i)), &passes.black))
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
        let post = [0, 1].map(|slot| post_group("post", source(slot), bloom_top));

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
                ],
            )
        };
        let black = &passes.black;
        let mut fx = HashMap::new();
        for &p in frame {
            let pair = |f: &dyn Fn(usize) -> wgpu::BindGroup| [f(0), f(1)];
            let groups = match p {
                FramePass::Ao => pair(&|_| fx_group(black, black)),
                FramePass::AoBlur => pair(&|_| fx_group(black, self.view("ao.raw"))),
                FramePass::Outline => {
                    pair(&|_| fx_group(self.view("hdr"), self.view("outline-id")))
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
            meter,
            reduce,
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
        encoder: &mut wgpu::CommandEncoder,
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
            FramePass::AoBlur => Some(("fs_ao_blur", "ao")),
            FramePass::Outline => Some(("fs_outline", "hdr.outline")),
            FramePass::DofCoc => Some(("fs_dof_coc", "dof.coc")),
            FramePass::DofBlur => Some(("fs_dof_blur", "dof.blur")),
            FramePass::DofComposite => Some(("fs_dof_composite", "hdr.dof")),
            FramePass::MotionBlur => Some(("fs_motion_blur", "hdr.mb")),
            FramePass::GodRaysMask => Some(("fs_godrays_mask", "god.mask")),
            FramePass::GodRaysBlur => Some(("fs_godrays_blur", "god.blur")),
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
                fullscreen_pass(
                    encoder,
                    "tonemap",
                    output,
                    clear,
                    &passes.tonemap,
                    &b.post[slot],
                );
                1
            }
            FramePass::Ao
            | FramePass::AoBlur
            | FramePass::Outline
            | FramePass::DofCoc
            | FramePass::DofBlur
            | FramePass::DofComposite
            | FramePass::MotionBlur
            | FramePass::GodRaysMask
            | FramePass::GodRaysBlur
            | FramePass::Cascade(_)
            | FramePass::LocalFace(_)
            | FramePass::Prepass
            | FramePass::Particles
            | FramePass::Light2d
            | FramePass::Metaballs
            | FramePass::Reflection
            | FramePass::Main
            | FramePass::Glow => 0,
        }
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
        "outline-id",
        "ao.raw",
        "ao",
        "lightmap",
        "metaballs",
        "reflection",
        "reflection-depth",
        "reflection-msaa",
        "glow",
        "hdr.outline",
        "hdr.dof",
        "hdr.mb",
        "dof.coc",
        "dof.blur",
        "god.mask",
        "god.blur",
    ]
    .iter()
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
