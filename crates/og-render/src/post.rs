//! After the main pass: TAA, auto-exposure metering, bloom and the tonemap,
//! and the size-dependent targets they and the main pass draw into.

use crate::graph::{GraphError, GraphPlan, RenderGraph, TargetDesc};
use crate::scene::{Exposure, PostEffects, Tonemap};
use crate::{target_texture, Antialias, DEPTH_FORMAT, HDR_FORMAT};
use std::collections::HashMap;

const FULLSCREEN_WGSL: &str = include_str!("../shaders/fullscreen.wgsl");
const POST_WGSL: &str = include_str!("../shaders/post.wgsl");
const TAA_WGSL: &str = include_str!("../shaders/taa.wgsl");
const LUMINANCE_WGSL: &str = include_str!("../shaders/luminance.wgsl");

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
    post_buffer: wgpu::Buffer,
    pub taa_buffer: wgpu::Buffer,
}

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
            ],
        });
        let taa_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("taa"),
            entries: &[
                layout_entry(0, float_texture(false)),
                layout_entry(1, float_texture(true)),
                layout_entry(
                    2,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
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
        PostPasses {
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

    pub fn write_uniform(&self, queue: &wgpu::Queue, exposure: Exposure, post: &PostEffects) {
        let (exposure, compensation) = match exposure {
            Exposure::Manual { ev } => ([ev, 0.0, 0.0, 0.0], 0.0),
            Exposure::Auto {
                compensation,
                min_ev,
                max_ev,
            } => ([0.0, 1.0, min_ev, max_ev.max(min_ev)], compensation),
        };
        let bloom = match post.bloom {
            Some(b) => [b.threshold, b.knee, b.intensity, compensation],
            None => [1.0, 0.5, 0.0, compensation],
        };
        let tonemap = match post.tonemap {
            Tonemap::Aces => 0.0,
            Tonemap::None => 1.0,
        };
        let uniform = PostUniform {
            exposure,
            bloom,
            mode: [tonemap, 1.0, 0.0, 0.0],
        };
        queue.write_buffer(&self.post_buffer, 0, bytemuck::bytes_of(&uniform));
    }
}

/// The size-dependent targets that outlive a frame (imported into the
/// graph): the offscreen output and the TAA history.
pub(crate) struct Persistent {
    pub offscreen: wgpu::Texture,
    pub offscreen_view: wgpu::TextureView,
    /// TAA history ping-pong: frame `n` writes `history[n % 2]`.
    pub history: Option<[wgpu::TextureView; 2]>,
}

impl Persistent {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        output_format: wgpu::TextureFormat,
        antialias: Antialias,
    ) -> Persistent {
        let attach_bind =
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let offscreen = target_texture(
            device,
            "offscreen",
            width,
            height,
            output_format,
            attach_bind | wgpu::TextureUsages::COPY_SRC,
        );
        let offscreen_view = offscreen.create_view(&Default::default());
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
    /// Opaque meshes, sky, transparent meshes, sprites and text into the HDR target.
    Main,
    Taa,
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
            FramePass::Main => "main".into(),
            FramePass::Taa => "taa".into(),
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
}

fn luminance_name(i: u32) -> String {
    format!("luminance.{i}")
}

fn bloom_name(i: u32) -> String {
    format!("bloom.{i}")
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
    let mut g = RenderGraph::new();
    g.import("cascades")
        .import("local-shadows")
        .import("output")
        .transient("hdr", desc(w, h, HDR_FORMAT, 1, attach_bind));
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
    let mut size = METER_SIZE;
    for i in 0..meter_levels() {
        g.transient(
            luminance_name(i),
            desc(size, size, LUMINANCE_FORMAT, 1, attach_bind),
        );
        size /= 8;
    }
    let (bw, bh) = ((w / 2).max(1), (h / 2).max(1));
    for i in 0..bloom_levels(w, h) {
        g.transient(
            bloom_name(i),
            desc(
                (bw >> i).max(1),
                (bh >> i).max(1),
                HDR_FORMAT,
                1,
                attach_bind,
            ),
        );
    }
    let taa = passes.contains(&FramePass::Taa);
    let auto = passes.contains(&FramePass::Meter);
    let bloom = passes.contains(&FramePass::BloomPrefilter);
    let lit = if taa { "history-write" } else { "hdr" };
    let lum = luminance_name(meter_levels() - 1);
    for &p in passes {
        let name = p.name();
        match p {
            FramePass::Cascade(_) => g.pass(name, &[], &["cascades"]),
            FramePass::LocalFace(_) => g.pass(name, &[], &["local-shadows"]),
            FramePass::Main if msaa => g.pass(
                name,
                &["cascades", "local-shadows"],
                &["hdr-msaa", "hdr", "depth"],
            ),
            FramePass::Main => g.pass(name, &["cascades", "local-shadows"], &["hdr", "depth"]),
            FramePass::Taa => g.pass(name, &["hdr", "depth", "history-read"], &["history-write"]),
            FramePass::Meter => g.pass(name, &[lit], &["luminance.0"]),
            FramePass::Reduce(i) => g.pass(name, &[&luminance_name(i)], &[&luminance_name(i + 1)]),
            FramePass::BloomPrefilter if auto => g.pass(name, &[lit, &lum], &["bloom.0"]),
            FramePass::BloomPrefilter => g.pass(name, &[lit], &["bloom.0"]),
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
                g.pass(name, &reads, &["output"])
            }
        };
    }
    g
}

/// The post passes' bind groups for one realization of the graph.
struct Bindings {
    taa: Option<[wgpu::BindGroup; 2]>,
    meter: Option<[wgpu::BindGroup; 2]>,
    reduce: Vec<wgpu::BindGroup>,
    prefilter: Option<[wgpu::BindGroup; 2]>,
    down: Vec<wgpu::BindGroup>,
    up: Vec<wgpu::BindGroup>,
    post: [wgpu::BindGroup; 2],
}

/// The frame's render graph: the compiled plan for the current pass list,
/// the transient textures realized from it (kept in a pool across frames
/// and reused by description) and the bind groups that read them.
#[derive(Default)]
pub(crate) struct FrameGraph {
    passes: Vec<FramePass>,
    order: Vec<FramePass>,
    plan: Option<GraphPlan>,
    pool: Vec<(TargetDesc, wgpu::TextureView)>,
    views: HashMap<String, wgpu::TextureView>,
    bindings: Option<Bindings>,
}

impl FrameGraph {
    /// Forgets everything size-dependent (on resize).
    pub fn invalidate(&mut self) {
        *self = FrameGraph::default();
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

    /// Compiles the graph for `passes` and realizes its transient targets,
    /// unless the previous frame had the same passes.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        passes: Vec<FramePass>,
        cfg: &GraphConfig,
        post: &PostPasses,
        persistent: &Persistent,
    ) -> Result<(), GraphError> {
        if self.plan.is_some() && passes == self.passes {
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
        self.bindings = Some(self.bind(device, post, persistent, &passes));
        self.passes = passes;
        self.plan = Some(plan);
        Ok(())
    }

    fn bind(
        &self,
        device: &wgpu::Device,
        passes: &PostPasses,
        persistent: &Persistent,
        frame: &[FramePass],
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
        let hdr = self.view("hdr");
        let taa = has(FramePass::Taa);
        let history = persistent.history.as_ref().filter(|_| taa);
        // The lit image post reads in frame slot `i`.
        let source = |slot: usize| match history {
            Some(h) => &h[slot],
            None => hdr,
        };
        let taa_groups = history.map(|h| {
            [0usize, 1].map(|slot| {
                group(
                    "taa",
                    &passes.taa_layout,
                    &[
                        tex(hdr),
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
                ],
            )
        };
        let bloom = has(FramePass::BloomPrefilter);
        let bloom_count = frame
            .iter()
            .filter(|p| matches!(p, FramePass::BloomDown(_)))
            .count() as u32
            + 1;
        let prefilter = bloom
            .then(|| [0, 1].map(|slot| post_group("bloom prefilter", source(slot), &passes.black)));
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
        Bindings {
            taa: taa_groups,
            meter,
            reduce,
            prefilter,
            down,
            up,
            post,
        }
    }

    /// Records one post pass (TAA, metering, bloom, tonemap); returns its draws.
    pub fn encode_post(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        passes: &PostPasses,
        persistent: &Persistent,
        pass: FramePass,
        slot: usize,
        output: &wgpu::TextureView,
    ) -> u32 {
        let b = self.bindings.as_ref().expect("prepared");
        let clear = wgpu::LoadOp::Clear(wgpu::Color::BLACK);
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
            FramePass::Cascade(_) | FramePass::LocalFace(_) | FramePass::Main => 0,
        }
    }
}

/// Every transient target name a graph for `cfg` can declare.
fn transient_names(cfg: &GraphConfig) -> Vec<String> {
    let mut names = vec!["hdr".to_string(), "hdr-msaa".into(), "depth".into()];
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
