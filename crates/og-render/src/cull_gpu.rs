//! GPU-driven culling on the full tier: the frustum and occlusion tests,
//! the compaction of the visible instances and the draws' instance counts
//! run in compute, and every mesh pass submits indexed indirect draws. The
//! CPU path ([`crate::Renderer::cull`]) stays for WebGL2 and for frames
//! this cannot take (object motion blur, buffers over the device limits).
//!
//! Per frame: the sorted instances are uploaded once (region 0 of
//! `instances`); stage A tests them against every shadow face, the planar
//! reflection and, for last frame's visible occluders, the camera; the
//! occluders are drawn into a depth buffer of their own; a depth pyramid
//! (the farthest depth per texel, every level) is built from it; stage B
//! tests the camera pass's instances against the frustum and the pyramid.
//! The occluders are drawn at this frame's camera, so the pyramid holds
//! real depth: an instance whose box lies behind it everywhere would draw
//! no pixel, and the frontmost occluder at a pixel is never behind its own
//! depth, so occlusion never removes a visible pixel. Each pass's visible instances land, batch by
//! batch in sorted order, in its own region (a deterministic prefix sum,
//! not atomics, decides where).
//!
//! The counts the CPU path knows at once (culled, occluded, triangles,
//! shadow casters) come back through a small readback ring: a frame's
//! [`crate::FrameStats`] carries those of the newest frame that has
//! landed (`counts_lag` frames old), and [`crate::Renderer::finish_stats`]
//! waits for the frame's own natively.

use crate::{Batch, DEPTH_FORMAT, GpuMaterial, GpuMesh, INSTANCE_STRIDE, Side, cull_mode};
use glam::{Mat4, Vec4};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const CULL_WGSL: &str = include_str!("../shaders/cull.wgsl");
const HIZ_WGSL: &str = include_str!("../shaders/hiz.wgsl");
const DEPTH_WGSL: &str = include_str!("../shaders/cull_depth.wgsl");

/// Passes one frame may cull: three cascades, twelve local light faces,
/// the planar reflection, the occluders and the camera.
pub(crate) const MAX_PASSES: usize = 18;
/// Planes per pass (a cascade's box and the slice planes facing the light).
const MAX_PLANES: usize = 12;
const WORKGROUP: u32 = 256;
const READBACKS: usize = 3;

const TRANSPARENT: u32 = 1;
const OCCLUDER: u32 = 2;
const OCCLUDABLE: u32 = 4;

/// What a pass draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PassKind {
    /// Every batch inside the planes (shadow cascades and light faces).
    Shadow,
    /// The opaque batches inside the planes (the planar reflection).
    Reflection,
    /// Last frame's visible occluders inside the camera's planes.
    Occluders,
    /// The opaque batches inside the camera's planes and not behind the
    /// occluders' depth.
    Main,
}

impl PassKind {
    fn code(self) -> u32 {
        match self {
            PassKind::Shadow => 0,
            PassKind::Reflection => 1,
            PassKind::Occluders => 2,
            PassKind::Main => 3,
        }
    }
}

pub(crate) struct CullPass {
    pub kind: PassKind,
    pub planes: Vec<Vec4>,
}

/// `struct PassGpu` in cull.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PassGpu {
    planes: [[f32; 4]; MAX_PLANES],
    info: [u32; 4],
}

/// `struct Params` in cull.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    counts: [u32; 4],
    sizes: [u32; 4],
    view_proj: [[f32; 4]; 4],
    passes: [PassGpu; MAX_PASSES],
}

/// `struct BatchGpu` in cull.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BatchGpu {
    first: u32,
    count: u32,
    index_count: u32,
    flags: u32,
    bounds: [f32; 4],
    extent: [f32; 4],
}

/// The GPU's counts for one frame (see the module docs).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuCounts {
    pub frame: u64,
    /// Opaque instances outside the camera's planes.
    pub culled: u32,
    /// Opaque instances inside them that the occluders hid.
    pub occluded: u32,
    /// Triangles of the camera pass's instance draws.
    pub triangles: u64,
    /// Instance draws across the shadow passes.
    pub shadow_casters: u32,
}

/// What a readback's words mean.
#[derive(Clone)]
struct CountLayout {
    frame: u64,
    batches: u32,
    kinds: Vec<PassKind>,
    /// Triangles per instance, per batch.
    triangles: Vec<u64>,
    opaque: u32,
}

impl CountLayout {
    fn counts(&self, words: &[u32]) -> GpuCounts {
        let b = self.batches as usize;
        let draws = |g: usize| (0..b).map(move |i| (i, words[(g * b + i) * 5 + 1]));
        let inside = |g: usize| words[self.kinds.len() * b * 5 + g];
        let mut c = GpuCounts {
            frame: self.frame,
            ..GpuCounts::default()
        };
        for (g, kind) in self.kinds.iter().enumerate() {
            match kind {
                PassKind::Shadow => {
                    c.shadow_casters += draws(g).map(|(_, n)| n).sum::<u32>();
                }
                PassKind::Main => {
                    let drawn: u32 = draws(g).map(|(_, n)| n).sum();
                    c.triangles = draws(g).map(|(i, n)| self.triangles[i] * n as u64).sum();
                    c.culled = self.opaque.saturating_sub(inside(g));
                    c.occluded = inside(g).saturating_sub(drawn);
                }
                PassKind::Reflection | PassKind::Occluders => {}
            }
        }
        c
    }
}

struct Readback {
    buffer: wgpu::Buffer,
    /// Set by `map_async`'s callback: whether the mapping succeeded.
    mapped: Arc<Mutex<Option<bool>>>,
    /// The frame copied into it, until it is read.
    layout: Option<CountLayout>,
    /// Copied but not yet asked to map (that happens after the submit).
    unmapped: bool,
    /// The counts this frame's stats were given in its place.
    applied: GpuCounts,
}

/// The depth pyramid and the views and groups that build it.
struct Pyramid {
    size: (u32, u32, u32),
    depth: wgpu::TextureView,
    full: wgpu::TextureView,
    levels: u32,
    base_group: wgpu::BindGroup,
    reduce_groups: Vec<wgpu::BindGroup>,
    level_sizes: Vec<(u32, u32)>,
}

pub(crate) struct GpuCulling {
    layout: wgpu::BindGroupLayout,
    visibility: wgpu::ComputePipeline,
    scan_blocks: wgpu::ComputePipeline,
    prefix: wgpu::ComputePipeline,
    scatter: wgpu::ComputePipeline,
    hiz_base_layout: [wgpu::BindGroupLayout; 2],
    hiz_reduce_layout: wgpu::BindGroupLayout,
    hiz_base: [Option<wgpu::ComputePipeline>; 2],
    hiz_reduce: wgpu::ComputePipeline,
    depth_module: wgpu::ShaderModule,
    depth_pipeline_layout: wgpu::PipelineLayout,
    depth_pipelines: HashMap<(Side, u32), wgpu::RenderPipeline>,
    view: wgpu::Buffer,
    view_group: wgpu::BindGroup,
    params: [wgpu::Buffer; 2],
    pub instances: wgpu::Buffer,
    batches: wgpu::Buffer,
    scan: wgpu::Buffer,
    pub args: wgpu::Buffer,
    groups: Option<[wgpu::BindGroup; 2]>,
    pyramid: Option<Pyramid>,
    readbacks: Vec<Readback>,
    latest: Option<GpuCounts>,
    /// The last encoded frame's layout and given counts (`finish` copies
    /// its counts late when it found no readback free).
    last: Option<(CountLayout, GpuCounts)>,
    max_binding: u64,
    /// This frame's layout: instances, batches, passes.
    pub n: u32,
    pub b: u32,
}

fn storage(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn texture(
    binding: u32,
    sample_type: wgpu::TextureSampleType,
    multisampled: bool,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Texture {
            sample_type,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled,
        },
        count: None,
    }
}

fn level_out(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::R32Float,
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    }
}

fn buffer(
    device: &wgpu::Device,
    label: &str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(16),
        usage,
        mapped_at_creation: false,
    })
}

/// The pyramid's base: the farthest of a pixel's samples.
fn hiz_source(multisampled: bool) -> String {
    let decl = if multisampled {
        "@group(0) @binding(2) var depth_in: texture_depth_multisampled_2d;\n\
         fn farthest_sample(xy: vec2<u32>) -> f32 {\n\
           var d = 0.0;\n\
           for (var s = 0u; s < textureNumSamples(depth_in); s++) { d = max(d, textureLoad(depth_in, xy, i32(s))); }\n\
           return d;\n\
         }\n"
    } else {
        "@group(0) @binding(2) var depth_in: texture_depth_2d;\n\
         fn farthest_sample(xy: vec2<u32>) -> f32 { return textureLoad(depth_in, xy, 0); }\n"
    };
    format!("{decl}{HIZ_WGSL}")
}

impl GpuCulling {
    pub fn new(device: &wgpu::Device, max_binding: u64) -> GpuCulling {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cull"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                storage(1, false),
                storage(2, true),
                storage(3, false),
                storage(4, false),
                texture(
                    5,
                    wgpu::TextureSampleType::Float { filterable: false },
                    false,
                ),
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cull"),
            source: wgpu::ShaderSource::Wgsl(CULL_WGSL.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cull"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let compute = |layout: &wgpu::PipelineLayout, module: &wgpu::ShaderModule, entry: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(layout),
                module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        let hiz_base_layout = [false, true].map(|multisampled| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("depth pyramid base"),
                entries: &[
                    level_out(1),
                    texture(2, wgpu::TextureSampleType::Depth, multisampled),
                ],
            })
        });
        let hiz_reduce_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("depth pyramid"),
            entries: &[
                texture(
                    0,
                    wgpu::TextureSampleType::Float { filterable: false },
                    false,
                ),
                level_out(1),
            ],
        });
        let hiz_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("depth pyramid"),
            source: wgpu::ShaderSource::Wgsl(hiz_source(false).into()),
        });
        let reduce_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("depth pyramid"),
            bind_group_layouts: &[Some(&hiz_reduce_layout)],
            immediate_size: 0,
        });
        let view = buffer(
            device,
            "occluder view",
            64,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let view_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("occluder view"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let view_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("occluder view"),
            layout: &view_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view.as_entire_binding(),
            }],
        });
        let params = std::array::from_fn(|_| {
            buffer(
                device,
                "cull params",
                std::mem::size_of::<Params>() as u64,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            )
        });
        let readbacks = Vec::new();
        GpuCulling {
            visibility: compute(&pipeline_layout, &module, "visibility"),
            scan_blocks: compute(&pipeline_layout, &module, "scan_blocks"),
            prefix: compute(&pipeline_layout, &module, "prefix"),
            scatter: compute(&pipeline_layout, &module, "scatter"),
            hiz_reduce: compute(&reduce_layout, &hiz_module, "reduce"),
            hiz_base: [None, None],
            hiz_base_layout,
            hiz_reduce_layout,
            depth_module: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("occluder depth"),
                source: wgpu::ShaderSource::Wgsl(DEPTH_WGSL.into()),
            }),
            depth_pipeline_layout: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("occluder depth"),
                bind_group_layouts: &[Some(&view_layout)],
                immediate_size: 0,
            }),
            depth_pipelines: HashMap::new(),
            view,
            view_group,
            params,
            instances: buffer(device, "culled instances", 0, Self::instance_usage()),
            batches: buffer(device, "cull batches", 0, Self::storage_usage()),
            scan: buffer(device, "cull scan", 0, Self::storage_usage()),
            args: buffer(device, "cull draws", 0, Self::args_usage()),
            layout,
            groups: None,
            pyramid: None,
            readbacks,
            latest: None,
            last: None,
            max_binding,
            n: 0,
            b: 0,
        }
    }

    fn instance_usage() -> wgpu::BufferUsages {
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
    }
    fn storage_usage() -> wgpu::BufferUsages {
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
    }
    fn args_usage() -> wgpu::BufferUsages {
        wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::INDIRECT
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST
    }

    /// Whether `n` instances in `passes` passes fit the device's storage
    /// binding limit.
    pub fn fits(&self, n: u32, passes: usize) -> bool {
        (passes as u64 + 1) * n as u64 * INSTANCE_STRIDE <= self.max_binding
            && Self::scan_words(n, passes) * 4 <= self.max_binding
    }

    fn scan_words(n: u32, passes: usize) -> u64 {
        let blocks = n.div_ceil(WORKGROUP) as u64;
        n as u64 + passes as u64 * (n as u64 + 1) + passes as u64 * blocks
    }

    /// The byte offset of pass `g`'s draw of batch `b` in `args`.
    pub fn draw_offset(&self, g: u32, b: u32) -> u64 {
        (g as u64 * self.b as u64 + b as u64) * 20
    }

    /// Where pass `g`'s instances of the batch starting at `first` begin.
    pub fn region(&self, g: u32, first: u32) -> u32 {
        (g + 1) * self.n + first
    }

    /// Takes every landed readback; the newest frame's counts are kept.
    pub fn poll_counts(&mut self) -> Option<GpuCounts> {
        for r in &mut self.readbacks {
            let ok = r.mapped.lock().unwrap_or_else(|e| e.into_inner()).take();
            let Some(ok) = ok else { continue };
            if let Some(layout) = r.layout.take()
                && ok
            {
                let counts = r
                    .buffer
                    .slice(..)
                    .get_mapped_range()
                    .ok()
                    .map(|data| layout.counts(bytemuck::cast_slice(&data)));
                if let Some(counts) = counts
                    && self.latest.is_none_or(|l| l.frame < counts.frame)
                {
                    self.latest = Some(counts);
                }
            }
            if ok {
                r.buffer.unmap();
            }
        }
        self.latest
    }

    /// Waits for `frame`'s readback (native): its counts and the ones its
    /// stats were given in their place. When the frame found no readback
    /// free and is the last one encoded (`args` still holds its counts),
    /// they are copied now.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn finish(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: u64,
    ) -> Option<(GpuCounts, GpuCounts)> {
        let find = |readbacks: &[Readback]| {
            readbacks
                .iter()
                .position(|r| r.layout.as_ref().is_some_and(|l| l.frame == frame))
        };
        let i = match find(&self.readbacks) {
            Some(i) => i,
            None => {
                let (layout, given) = self.last.clone().filter(|(l, _)| l.frame == frame)?;
                // Every readback in flight lands and is taken.
                device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
                self.poll_counts();
                let r = self.readbacks.iter_mut().find(|r| r.layout.is_none())?;
                let words = layout.kinds.len() as u64 * (layout.batches as u64 * 5 + 1);
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("cull counts"),
                });
                encoder.copy_buffer_to_buffer(&self.args, 0, &r.buffer, 0, words * 4);
                r.layout = Some(layout);
                r.applied = given;
                r.unmapped = true;
                queue.submit(Some(encoder.finish()));
                self.after_submit();
                find(&self.readbacks)?
            }
        };
        let applied = self.readbacks[i].applied;
        loop {
            if self.readbacks[i]
                .mapped
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_some()
            {
                break;
            }
            device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        }
        self.poll_counts();
        self.latest
            .filter(|l| l.frame == frame)
            .map(|exact| (exact, applied))
    }

    /// After the frame's submit: maps the readback it copied into.
    pub fn after_submit(&mut self) {
        for r in &mut self.readbacks {
            if std::mem::take(&mut r.unmapped) {
                let mapped = r.mapped.clone();
                r.buffer
                    .slice(..)
                    .map_async(wgpu::MapMode::Read, move |result| {
                        *mapped.lock().unwrap_or_else(|e| e.into_inner()) = Some(result.is_ok());
                    });
            }
        }
    }

    fn ensure_pyramid(&mut self, device: &wgpu::Device, width: u32, height: u32, samples: u32) {
        if self
            .pyramid
            .as_ref()
            .is_some_and(|p| p.size == (width, height, samples))
        {
            return;
        }
        let multisampled = samples > 1;
        let depth = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("occluder depth"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default());
        let levels = 32 - width.max(height).leading_zeros();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth pyramid"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let level_views: Vec<wgpu::TextureView> = (0..levels)
            .map(|l| {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    base_mip_level: l,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();
        let base_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("depth pyramid base"),
            layout: &self.hiz_base_layout[multisampled as usize],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&level_views[0]),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&depth),
                },
            ],
        });
        let reduce_groups = (1..levels as usize)
            .map(|l| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("depth pyramid"),
                    layout: &self.hiz_reduce_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&level_views[l - 1]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&level_views[l]),
                        },
                    ],
                })
            })
            .collect();
        if self.hiz_base[multisampled as usize].is_none() {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("depth pyramid base"),
                source: wgpu::ShaderSource::Wgsl(hiz_source(multisampled).into()),
            });
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("depth pyramid base"),
                bind_group_layouts: &[Some(&self.hiz_base_layout[multisampled as usize])],
                immediate_size: 0,
            });
            self.hiz_base[multisampled as usize] = Some(device.create_compute_pipeline(
                &wgpu::ComputePipelineDescriptor {
                    label: Some("depth pyramid base"),
                    layout: Some(&layout),
                    module: &module,
                    entry_point: Some("base"),
                    compilation_options: Default::default(),
                    cache: None,
                },
            ));
        }
        self.pyramid = Some(Pyramid {
            size: (width, height, samples),
            depth,
            full: texture.create_view(&Default::default()),
            levels,
            base_group,
            reduce_groups,
            level_sizes: (0..levels)
                .map(|l| ((width >> l).max(1), (height >> l).max(1)))
                .collect(),
        });
        self.groups = None;
    }

    fn depth_pipeline(&mut self, device: &wgpu::Device, side: Side, samples: u32) {
        self.depth_pipelines
            .entry((side, samples))
            .or_insert_with(|| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("occluder depth"),
                    layout: Some(&self.depth_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &self.depth_module,
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: &[Some(crate::VERTEX_LAYOUT), Some(crate::INSTANCE_LAYOUT)],
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
                    multisample: wgpu::MultisampleState {
                        count: samples,
                        ..Default::default()
                    },
                    fragment: None,
                    multiview_mask: None,
                    cache: None,
                })
            });
    }

    /// Grows a buffer to at least `size` bytes (dropping its bind groups).
    #[allow(clippy::too_many_arguments)]
    fn grow(
        device: &wgpu::Device,
        buf: &mut wgpu::Buffer,
        groups: &mut Option<[wgpu::BindGroup; 2]>,
        label: &str,
        size: u64,
        cap: u64,
        usage: wgpu::BufferUsages,
    ) {
        if size > buf.size() {
            // Bound whole: never past the binding limit.
            *buf = buffer(
                device,
                label,
                size.next_power_of_two().min(cap).max(size),
                usage,
            );
            *groups = None;
        }
    }

    /// The frame's culling: uploads the sorted `transforms` and the batch
    /// table, and encodes both stages, the occluders' depth and the
    /// pyramid. Returns the commands to submit before the frame's own and
    /// the occluder depth pass's draws.
    #[allow(clippy::too_many_arguments)]
    pub fn encode(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        transforms: &[[[f32; 4]; 4]],
        batches: &[Batch],
        meshes: &[GpuMesh],
        materials: &[GpuMaterial],
        passes: &[CullPass],
        view_proj: Mat4,
        (width, height, samples): (u32, u32, u32),
        frame: u64,
        stats_given: GpuCounts,
    ) -> (wgpu::CommandBuffer, u32) {
        let n = transforms.len() as u32;
        let b = batches.len() as u32;
        let p = passes.len();
        self.n = n;
        self.b = b;
        self.ensure_pyramid(device, width, height, samples);
        let occluder_sides: Vec<Side> = {
            let mut s: Vec<Side> = batches
                .iter()
                .filter(|x| materials[x.material].occluder)
                .map(|x| x.side)
                .collect();
            s.sort();
            s.dedup();
            s
        };
        for side in occluder_sides {
            self.depth_pipeline(device, side, samples);
        }

        // Buffers (grown as needed; new ones start zeroed: nothing was
        // visible last frame).
        let blocks = n.div_ceil(WORKGROUP);
        let args_words = (p as u64 * b as u64 * 5) + p as u64;
        Self::grow(
            device,
            &mut self.instances,
            &mut self.groups,
            "culled instances",
            (p as u64 + 1) * n as u64 * INSTANCE_STRIDE,
            self.max_binding,
            Self::instance_usage(),
        );
        Self::grow(
            device,
            &mut self.batches,
            &mut self.groups,
            "cull batches",
            b as u64 * std::mem::size_of::<BatchGpu>() as u64,
            self.max_binding,
            Self::storage_usage(),
        );
        Self::grow(
            device,
            &mut self.scan,
            &mut self.groups,
            "cull scan",
            Self::scan_words(n, p) * 4,
            self.max_binding,
            Self::storage_usage(),
        );
        Self::grow(
            device,
            &mut self.args,
            &mut self.groups,
            "cull draws",
            args_words * 4,
            self.max_binding,
            Self::args_usage(),
        );
        let readback_size = (args_words * 4).next_power_of_two().max(256);
        if self
            .readbacks
            .first()
            .is_some_and(|r| r.buffer.size() < args_words * 4)
        {
            // Mapped or in-flight ones are dropped (their counts lost).
            self.readbacks.clear();
        }
        while self.readbacks.len() < READBACKS {
            self.readbacks.push(Readback {
                buffer: buffer(
                    device,
                    "cull counts",
                    readback_size,
                    wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                ),
                mapped: Arc::new(Mutex::new(None)),
                layout: None,
                unmapped: false,
                applied: GpuCounts::default(),
            });
        }

        // Uploads.
        queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(transforms));
        let table: Vec<BatchGpu> = batches
            .iter()
            .map(|x| {
                let m = &materials[x.material];
                let (c, e) = meshes[x.mesh].bounds.unwrap_or_default();
                BatchGpu {
                    first: x.first,
                    count: x.count,
                    index_count: meshes[x.mesh].index_count,
                    flags: (x.transparent as u32 * TRANSPARENT)
                        | (m.occluder as u32 * OCCLUDER)
                        | (x.shader.is_none() as u32 * OCCLUDABLE),
                    bounds: [c.x, c.y, c.z, e.length()],
                    extent: [e.x, e.y, e.z, 0.0],
                }
            })
            .collect();
        queue.write_buffer(&self.batches, 0, bytemuck::cast_slice(&table));
        queue.write_buffer(
            &self.view,
            0,
            bytemuck::cast_slice(&view_proj.to_cols_array()),
        );
        let pyramid = self.pyramid.as_ref().expect("made above");
        let stage = |which: &[usize]| {
            let mut params = Params {
                counts: [n, b, which.len() as u32, p as u32],
                sizes: [blocks, pyramid.levels, width, height],
                view_proj: view_proj.to_cols_array_2d(),
                passes: [bytemuck::Zeroable::zeroed(); MAX_PASSES],
            };
            for (slot, &g) in which.iter().enumerate() {
                let pass = &passes[g];
                let planes = &pass.planes[..pass.planes.len().min(MAX_PLANES)];
                for (i, plane) in planes.iter().enumerate() {
                    params.passes[slot].planes[i] = plane.to_array();
                }
                params.passes[slot].info = [pass.kind.code(), planes.len() as u32, g as u32, 0];
            }
            params
        };
        let main: Vec<usize> = (0..p)
            .filter(|&g| passes[g].kind == PassKind::Main)
            .collect();
        let others: Vec<usize> = (0..p)
            .filter(|&g| passes[g].kind != PassKind::Main)
            .collect();
        queue.write_buffer(&self.params[0], 0, bytemuck::bytes_of(&stage(&others)));
        queue.write_buffer(&self.params[1], 0, bytemuck::bytes_of(&stage(&main)));
        if self.groups.is_none() {
            let group = |params: &wgpu::Buffer| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("cull"),
                    layout: &self.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: params.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: self.instances.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: self.batches.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: self.scan.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: self.args.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::TextureView(&pyramid.full),
                        },
                    ],
                })
            };
            self.groups = Some([group(&self.params[0]), group(&self.params[1])]);
        }
        let groups = self.groups.as_ref().expect("made above");

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("cull"),
        });
        // The inside counters start at zero.
        encoder.clear_buffer(&self.args, p as u64 * b as u64 * 20, Some(p as u64 * 4));
        let run = |pass: &mut wgpu::ComputePass, group: &wgpu::BindGroup, stage_passes: u32| {
            pass.set_bind_group(0, group, &[]);
            pass.set_pipeline(&self.visibility);
            pass.dispatch_workgroups(blocks, stage_passes, 1);
            pass.set_pipeline(&self.scan_blocks);
            pass.dispatch_workgroups(1, stage_passes, 1);
            pass.set_pipeline(&self.prefix);
            pass.dispatch_workgroups(blocks, stage_passes, 1);
            pass.set_pipeline(&self.scatter);
            pass.dispatch_workgroups(blocks, stage_passes, 1);
        };
        if !others.is_empty() {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("cull: shadows, reflection, occluders"),
                timestamp_writes: None,
            });
            run(&mut pass, &groups[0], others.len() as u32);
        }
        let mut occluder_draws = 0;
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("occluder depth"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &pyramid.depth,
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
            if let Some(g) = passes.iter().position(|x| x.kind == PassKind::Occluders) {
                pass.set_bind_group(0, &self.view_group, &[]);
                let mut side = None;
                for (i, x) in batches.iter().enumerate() {
                    if !materials[x.material].occluder {
                        continue;
                    }
                    if side != Some(x.side) {
                        pass.set_pipeline(&self.depth_pipelines[&(x.side, samples)]);
                        side = Some(x.side);
                    }
                    let mesh = &meshes[x.mesh];
                    let start = self.region(g as u32, x.first) as u64 * INSTANCE_STRIDE;
                    pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                    pass.set_vertex_buffer(
                        1,
                        self.instances
                            .slice(start..start + x.count as u64 * INSTANCE_STRIDE),
                    );
                    pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed_indirect(&self.args, self.draw_offset(g as u32, i as u32));
                    occluder_draws += 1;
                }
            }
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("cull: depth pyramid, camera"),
                timestamp_writes: None,
            });
            pass.set_pipeline(
                self.hiz_base[(samples > 1) as usize]
                    .as_ref()
                    .expect("made with the pyramid"),
            );
            pass.set_bind_group(0, &pyramid.base_group, &[]);
            pass.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
            pass.set_pipeline(&self.hiz_reduce);
            for (l, group) in pyramid.reduce_groups.iter().enumerate() {
                let (w, h) = pyramid.level_sizes[l + 1];
                pass.set_bind_group(0, group, &[]);
                pass.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
            }
            if !main.is_empty() {
                run(&mut pass, &groups[1], main.len() as u32);
            }
        }
        // The counts, when a readback is free.
        let layout = CountLayout {
            frame,
            batches: b,
            kinds: passes.iter().map(|x| x.kind).collect(),
            triangles: batches
                .iter()
                .map(|x| (meshes[x.mesh].index_count / 3) as u64)
                .collect(),
            opaque: batches
                .iter()
                .filter(|x| !x.transparent)
                .map(|x| x.count)
                .sum(),
        };
        if let Some(r) = self
            .readbacks
            .iter_mut()
            .find(|r| r.layout.is_none() && r.mapped.lock().map(|m| m.is_none()).unwrap_or(false))
        {
            encoder.copy_buffer_to_buffer(&self.args, 0, &r.buffer, 0, args_words * 4);
            r.layout = Some(layout.clone());
            r.unmapped = true;
            r.applied = stats_given;
        }
        self.last = Some((layout, stats_given));
        (encoder.finish(), occluder_draws)
    }
}
