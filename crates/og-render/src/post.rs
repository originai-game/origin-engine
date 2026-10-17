//! After the main pass: TAA, auto-exposure metering, bloom and the tonemap,
//! and the size-dependent targets they and the main pass draw into.

use crate::scene::{Exposure, PostEffects, Tonemap};
use crate::{target_texture, Antialias, DEPTH_FORMAT, HDR_FORMAT};

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

/// Every size-dependent target and the bind groups that read them.
pub(crate) struct Targets {
    /// The single-sample HDR image of the main pass (the MSAA resolve target).
    pub hdr: wgpu::TextureView,
    /// The multisampled HDR target when MSAA is on.
    pub hdr_msaa: Option<wgpu::TextureView>,
    /// Main-pass depth (multisampled with MSAA; sampled by TAA).
    pub depth: wgpu::TextureView,
    pub offscreen: wgpu::Texture,
    pub offscreen_view: wgpu::TextureView,
    /// TAA history ping-pong: frame `n` writes `history[n % 2]`.
    history: Option<[wgpu::TextureView; 2]>,
    taa_groups: Option<[wgpu::BindGroup; 2]>,
    lum_levels: Vec<wgpu::TextureView>,
    meter_groups: [wgpu::BindGroup; 2],
    reduce_groups: Vec<wgpu::BindGroup>,
    bloom_levels: Vec<wgpu::TextureView>,
    prefilter_groups: [wgpu::BindGroup; 2],
    down_groups: Vec<wgpu::BindGroup>,
    up_groups: Vec<wgpu::BindGroup>,
    post_groups: [wgpu::BindGroup; 2],
}

/// What the post chain does this frame.
pub(crate) struct PostFrame {
    /// The history slot TAA writes (and post reads) this frame.
    pub slot: usize,
    pub taa: bool,
    pub auto_exposure: bool,
    pub bloom: bool,
}

impl Targets {
    pub fn new(
        device: &wgpu::Device,
        passes: &PostPasses,
        width: u32,
        height: u32,
        output_format: wgpu::TextureFormat,
        samples: u32,
        antialias: Antialias,
    ) -> Targets {
        let attach_bind =
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let view = |t: wgpu::Texture| t.create_view(&Default::default());
        let hdr = view(target_texture(
            device,
            "hdr",
            width,
            height,
            HDR_FORMAT,
            attach_bind,
        ));
        let multisampled = |label: &str, format: wgpu::TextureFormat| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: samples,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        let hdr_msaa = (samples > 1).then(|| multisampled("hdr msaa", HDR_FORMAT));
        let taa = antialias == Antialias::Taa;
        let depth = if samples > 1 {
            multisampled("depth msaa", DEPTH_FORMAT)
        } else {
            let usage = if taa {
                attach_bind
            } else {
                wgpu::TextureUsages::RENDER_ATTACHMENT
            };
            view(target_texture(
                device,
                "depth",
                width,
                height,
                DEPTH_FORMAT,
                usage,
            ))
        };
        let offscreen = target_texture(
            device,
            "offscreen",
            width,
            height,
            output_format,
            attach_bind | wgpu::TextureUsages::COPY_SRC,
        );
        let offscreen_view = offscreen.create_view(&Default::default());
        let history = taa.then(|| {
            [0, 1].map(|_| {
                view(target_texture(
                    device,
                    "taa history",
                    width,
                    height,
                    HDR_FORMAT,
                    attach_bind,
                ))
            })
        });
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
        let taa_groups = history.as_ref().map(|h| {
            [0usize, 1].map(|slot| {
                group(
                    "taa",
                    &passes.taa_layout,
                    &[
                        tex(&hdr),
                        tex(&h[1 - slot]),
                        tex(&depth),
                        sampler.clone(),
                        passes.taa_buffer.as_entire_binding(),
                    ],
                )
            })
        });
        // The lit image post reads in frame slot `i`.
        let source = |slot: usize| match &history {
            Some(h) => &h[slot],
            None => &hdr,
        };

        let mut lum_levels = Vec::new();
        let mut size = METER_SIZE;
        loop {
            lum_levels.push(view(target_texture(
                device,
                "luminance",
                size,
                size,
                LUMINANCE_FORMAT,
                attach_bind,
            )));
            if size == 1 {
                break;
            }
            size /= 8;
        }
        let meter_groups = [0, 1].map(|slot| {
            group(
                "meter",
                &passes.source_layout,
                &[tex(source(slot)), sampler.clone()],
            )
        });
        let reduce_groups = lum_levels[..lum_levels.len() - 1]
            .iter()
            .map(|level| {
                group(
                    "reduce",
                    &passes.source_layout,
                    &[tex(level), sampler.clone()],
                )
            })
            .collect();
        let lum = lum_levels.last().expect("at least one level");

        let (bw, bh) = ((width / 2).max(1), (height / 2).max(1));
        let levels = (32 - bw.min(bh).max(1).leading_zeros())
            .saturating_sub(3)
            .clamp(1, MAX_BLOOM_LEVELS);
        let bloom_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("bloom"),
            size: wgpu::Extent3d {
                width: bw,
                height: bh,
                depth_or_array_layers: 1,
            },
            mip_level_count: levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: HDR_FORMAT,
            usage: attach_bind,
            view_formats: &[],
        });
        let bloom_levels: Vec<_> = (0..levels)
            .map(|mip| {
                bloom_texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("bloom level"),
                    base_mip_level: mip,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();
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
        let prefilter_groups =
            [0, 1].map(|slot| post_group("bloom prefilter", source(slot), &passes.black));
        let down_groups = bloom_levels[..bloom_levels.len() - 1]
            .iter()
            .map(|level| post_group("bloom down", level, &passes.black))
            .collect();
        let up_groups = bloom_levels[1..]
            .iter()
            .map(|level| post_group("bloom up", level, &passes.black))
            .collect();
        let post_groups = [0, 1].map(|slot| post_group("post", source(slot), &bloom_levels[0]));
        Targets {
            hdr,
            hdr_msaa,
            depth,
            offscreen,
            offscreen_view,
            history,
            taa_groups,
            lum_levels,
            meter_groups,
            reduce_groups,
            bloom_levels,
            prefilter_groups,
            down_groups,
            up_groups,
            post_groups,
        }
    }

    /// Records TAA, metering, bloom and the tonemap into `output`; returns the
    /// draw calls recorded.
    pub fn encode(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        passes: &PostPasses,
        frame: &PostFrame,
        output: &wgpu::TextureView,
    ) -> u32 {
        let clear = wgpu::LoadOp::Clear(wgpu::Color::BLACK);
        let mut draws = 0;
        let slot = frame.slot;
        if frame.taa {
            if let (Some(pipeline), Some(history), Some(groups)) =
                (&passes.taa, &self.history, &self.taa_groups)
            {
                fullscreen_pass(
                    encoder,
                    "taa",
                    &history[slot],
                    clear,
                    pipeline,
                    &groups[slot],
                );
                draws += 1;
            }
        }
        if frame.auto_exposure {
            fullscreen_pass(
                encoder,
                "meter",
                &self.lum_levels[0],
                clear,
                &passes.meter,
                &self.meter_groups[slot],
            );
            for (i, group) in self.reduce_groups.iter().enumerate() {
                fullscreen_pass(
                    encoder,
                    "meter reduce",
                    &self.lum_levels[i + 1],
                    clear,
                    &passes.reduce,
                    group,
                );
            }
            draws += self.lum_levels.len() as u32;
        }
        if frame.bloom {
            fullscreen_pass(
                encoder,
                "bloom prefilter",
                &self.bloom_levels[0],
                clear,
                &passes.prefilter,
                &self.prefilter_groups[slot],
            );
            for (i, group) in self.down_groups.iter().enumerate() {
                fullscreen_pass(
                    encoder,
                    "bloom down",
                    &self.bloom_levels[i + 1],
                    clear,
                    &passes.down,
                    group,
                );
            }
            for (i, group) in self.up_groups.iter().enumerate().rev() {
                fullscreen_pass(
                    encoder,
                    "bloom up",
                    &self.bloom_levels[i],
                    wgpu::LoadOp::Load,
                    &passes.up,
                    group,
                );
            }
            draws += 1 + 2 * self.down_groups.len() as u32;
        }
        fullscreen_pass(
            encoder,
            "tonemap",
            output,
            clear,
            &passes.tonemap,
            &self.post_groups[slot],
        );
        draws + 1
    }
}

/// The current frame's weight in the TAA blend: 1 when there is no history.
pub(crate) fn taa_weight(history_valid: bool) -> f32 {
    if history_valid {
        TAA_CURRENT_WEIGHT
    } else {
        1.0
    }
}
