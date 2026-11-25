//! Compact 8 B/px visibility buffer with 8x8 tile-classified indirect shading
//! (`meshlet_raster.wgsl`, `meshlet.wgsl`):
//!   - `raster_depth` + `raster_id`: dispatched indirectly for only the active
//!     soft clusters, writing depth and triangle ID into `words` (8 B/px, zero
//!     overflow cap at any resolution — fixes B3).
//!   - `classify_count` + `classify_prefix` + `classify_emit`: bins active 8x8
//!     tiles per soft batch into `tile_list` and fills per-batch `DrawIndirect`
//!     args in `tile_draws`.
//!   - `vs_meshlet_resolve` + `fs_meshlet_resolve` / `fs_prepass_meshlet_resolve`:
//!     draws only active 8x8 tile quads via indirect draw, pulling triangle
//!     attributes and analytic UV/world/local/normal derivatives directly from
//!     the geometry pools (fixing B8).

use super::{RasterParams, buffer, entry, grow, storage};
use crate::Side;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

/// A pipeline's error scope, until the device answers it.
type Check = Pin<Box<dyn Future<Output = Option<wgpu::Error>>>>;

/// Uniform stride for `vis_row` dynamic offsets (256-byte universal alignment).
pub(super) const ROW_OFFSET_STRIDE: u32 = 256;

pub(crate) struct VisBuffer {
    layout: wgpu::BindGroupLayout,
    wind_field: wgpu::TextureView,
    depth: wgpu::ComputePipeline,
    id: wgpu::ComputePipeline,
    classify_count: wgpu::ComputePipeline,
    classify_prefix: wgpu::ComputePipeline,
    classify_emit: wgpu::ComputePipeline,
    params: wgpu::Buffer,
    /// Compact 8 B/px visibility buffer: depth words [0..wh), id words [wh..2*wh).
    words: wgpu::Buffer,
    /// Per soft batch DrawIndirectArgs (6, tile_count, 0, 0).
    pub(crate) tile_draws: wgpu::Buffer,
    /// Compact per-batch tile offsets + packed (tx | ty << 16) coords.
    tile_list: wgpu::Buffer,
    /// Static `[row, 0, 0, 0]` table at 256-byte stride for dynamic `vis_row`.
    row_indices: wgpu::Buffer,
    row_capacity: u32,
    /// The resolve's (width, height, soft_first, 0).
    screen: wgpu::Buffer,
    pub(crate) resolve_layout: wgpu::BindGroupLayout,
    resolve_cache: Option<(Vec<wgpu::Buffer>, wgpu::BindGroup)>,
    /// The resolve's lit pipelines by `effects`, and its prepass one.
    pub(crate) lit: HashMap<bool, wgpu::RenderPipeline>,
    pub(crate) prepass: Option<wgpu::RenderPipeline>,
    checks: Vec<Check>,
    refused: Option<String>,
}

/// What [`VisBuffer::encode`] reads this frame.
pub(crate) struct VisFrame<'a> {
    pub view_proj: glam::Mat4,
    pub size: (u32, u32),
    /// The soft list's first item and capacity in clusters.
    pub list: (u32, u32),
    /// Highest soft batch row index + 1 (`0` = no soft batches).
    pub soft_rows: u32,
    /// The cull's count of the soft list: buffer and byte offset.
    pub count: (&'a wgpu::Buffer, u64),
    /// Soft raster `DispatchWorkgroupsIndirect` buffer and byte offset.
    pub soft_dispatch: (&'a wgpu::Buffer, u64),
    pub draw_params: &'a wgpu::Buffer,
    pub instances: &'a wgpu::Buffer,
    pub previous: &'a wgpu::Buffer,
    pub items: &'a wgpu::Buffer,
    pub clusters: &'a wgpu::Buffer,
    pub indices: &'a wgpu::Buffer,
    pub vertices: &'a wgpu::Buffer,
    pub stylise: [f32; 4],
    pub wind: [crate::foliage::FoliageWind; 2],
    pub leaf_year: [[f32; 4]; 2],
    pub max_binding: u64,
}

impl VisBuffer {
    pub fn new(device: &wgpu::Device, wind_field: wgpu::TextureView) -> VisBuffer {
        let c = wgpu::ShaderStages::COMPUTE;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("meshlet raster"),
            entries: &[
                super::uniform(0, c),
                storage(1, true, c),
                storage(2, true, c),
                storage(3, true, c),
                storage(4, true, c),
                storage(5, true, c),
                storage(6, false, c),
                storage(7, false, c),
                storage(8, false, c),
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: c,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let module = crate::gpu::shader_module(
            device,
            wgpu::ShaderModuleDescriptor {
                label: Some("meshlet raster"),
                source: wgpu::ShaderSource::Wgsl(super::raster_source().into()),
            },
        );
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("meshlet raster"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = |entry: &str| {
            crate::traffic::create_compute_pipeline(
                device,
                &wgpu::ComputePipelineDescriptor {
                    label: Some(entry),
                    layout: Some(&pipeline_layout),
                    module: &module,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    cache: crate::pipeline_cache::of(device).as_ref(),
                },
            )
        };
        let f = wgpu::ShaderStages::FRAGMENT;
        let v = wgpu::ShaderStages::VERTEX;
        let vf = wgpu::ShaderStages::VERTEX_FRAGMENT;
        let resolve_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("meshlet resolve"),
            entries: &[
                super::uniform(8, f),
                storage(9, true, f),
                storage(10, true, f),
                storage(11, true, f),
                storage(12, true, f),
                storage(14, true, f),
                super::uniform(15, vf),
                storage(16, true, f),
                storage(17, true, f),
                wgpu::BindGroupLayoutEntry {
                    binding: 18,
                    visibility: f,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                storage(19, true, v),
                wgpu::BindGroupLayoutEntry {
                    binding: 20,
                    visibility: v,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(16),
                    },
                    count: None,
                },
            ],
        });
        let rw = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        VisBuffer {
            wind_field,
            depth: pipeline("raster_depth"),
            id: pipeline("raster_id"),
            classify_count: pipeline("classify_count"),
            classify_prefix: pipeline("classify_prefix"),
            classify_emit: pipeline("classify_emit"),
            layout,
            params: buffer(
                device,
                "meshlet raster params",
                std::mem::size_of::<RasterParams>() as u64,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            words: buffer(device, "meshlet visibility buffer", 0, rw),
            tile_draws: buffer(
                device,
                "meshlet tile draws",
                0,
                rw | wgpu::BufferUsages::INDIRECT,
            ),
            tile_list: buffer(device, "meshlet tile list", 0, rw),
            row_indices: buffer(
                device,
                "meshlet tile row indices",
                u64::from(ROW_OFFSET_STRIDE),
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            row_capacity: 0,
            screen: buffer(
                device,
                "meshlet resolve screen",
                16,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            resolve_layout,
            resolve_cache: None,
            lit: HashMap::new(),
            prepass: None,
            checks: Vec::new(),
            refused: None,
        }
    }

    pub fn resolve_group(&self) -> Option<&wgpu::BindGroup> {
        self.resolve_cache.as_ref().map(|(_, g)| g)
    }

    pub fn usable(&mut self) -> bool {
        let mut cx = Context::from_waker(Waker::noop());
        let mut refused = None;
        self.checks.retain_mut(|c| match c.as_mut().poll(&mut cx) {
            Poll::Ready(Some(e)) => {
                refused.get_or_insert_with(|| e.to_string());
                false
            }
            Poll::Ready(None) => false,
            Poll::Pending => true,
        });
        if let Some(e) = refused {
            log::warn!(
                "og-render: the device refused the visibility buffer's resolve; its clusters draw on the hardware path: {e}"
            );
            self.checks.clear();
            self.clear_pipelines();
            self.refused = Some(e);
        }
        self.ready()
    }

    pub fn ready(&self) -> bool {
        self.refused.is_none() && self.checks.is_empty()
    }

    pub fn refused(&self) -> Option<&str> {
        self.refused.as_deref()
    }

    fn ensure_row_indices(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, rows: u32) {
        if rows <= self.row_capacity {
            return;
        }
        let cap = rows.next_power_of_two().max(64);
        let bytes = u64::from(cap) * u64::from(ROW_OFFSET_STRIDE);
        self.row_indices = buffer(
            device,
            "meshlet tile row indices",
            bytes,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let mut data = vec![0u32; (bytes / 4) as usize];
        let stride_words = (ROW_OFFSET_STRIDE / 4) as usize;
        for r in 0..cap {
            data[r as usize * stride_words] = r;
        }
        crate::traffic::write_buffer(queue, &self.row_indices, 0, bytemuck::cast_slice(&data));
        self.row_capacity = cap;
        self.resolve_cache = None;
    }

    /// Rasterises the frame's soft list into `encoder`, classifies active 8x8
    /// tiles per soft batch, and caches the resolve bind group.
    pub fn encode(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        f: VisFrame<'_>,
    ) {
        if f.soft_rows == 0 {
            return;
        }
        let (w, h) = (f.size.0.max(1), f.size.1.max(1));
        let pixels = w as u64 * h as u64;
        // 8 B/px: depth[wh] + id[wh].
        let words = 2 * pixels * 4;
        if words > f.max_binding {
            return;
        }
        let (tx, ty) = (w.div_ceil(8), h.div_ceil(8));
        let screen_tiles = u64::from(tx) * u64::from(ty);
        let max_tiles = (u64::from(f.soft_rows) * 2 + screen_tiles * 4)
            .min(f.max_binding / 4)
            .max(u64::from(f.soft_rows) * 2 + 4);
        let tile_list_bytes = max_tiles * 4;
        let tile_draws_bytes = u64::from(f.soft_rows) * 16;

        let rw = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        if grow(
            device,
            &mut self.words,
            "meshlet visibility buffer",
            words,
            rw,
        ) {
            self.resolve_cache = None;
        }
        grow(
            device,
            &mut self.tile_draws,
            "meshlet tile draws",
            tile_draws_bytes,
            rw | wgpu::BufferUsages::INDIRECT,
        );
        if grow(
            device,
            &mut self.tile_list,
            "meshlet tile list",
            tile_list_bytes,
            rw,
        ) {
            self.resolve_cache = None;
        }
        self.ensure_row_indices(device, queue, f.soft_rows);

        crate::traffic::write_buffer(
            queue,
            &self.screen,
            0,
            bytemuck::cast_slice(&[w, h, f.list.0, 0u32]),
        );
        let params = RasterParams {
            view_proj: f.view_proj.to_cols_array_2d(),
            size: [w, h, f.soft_rows, max_tiles as u32],
            soft: [f.list.0, f.list.1, 0, 0],
            stylise: f.stylise,
            wind: f.wind,
            leaf_year: f.leaf_year,
        };
        crate::traffic::write_buffer(queue, &self.params, 0, bytemuck::bytes_of(&params));
        // The cull's soft cluster count into `soft.z`.
        encoder.copy_buffer_to_buffer(f.count.0, f.count.1, &self.params, 64 + 16 + 8, 4);
        encoder.clear_buffer(&self.words, 0, Some(words));
        encoder.clear_buffer(&self.tile_draws, 0, Some(tile_draws_bytes));

        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("meshlet raster"),
            layout: &self.layout,
            entries: &[
                entry(0, &self.params),
                entry(1, f.instances),
                entry(2, f.items),
                entry(3, f.clusters),
                entry(4, f.indices),
                entry(5, f.vertices),
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.words,
                        offset: 0,
                        size: wgpu::BufferSize::new(words),
                    }),
                },
                entry(7, &self.tile_draws),
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.tile_list,
                        offset: 0,
                        size: wgpu::BufferSize::new(tile_list_bytes),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(&self.wind_field),
                },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("meshlet raster"),
            timestamp_writes: None,
        });
        pass.set_bind_group(0, &group, &[]);
        for pipeline in [&self.depth, &self.id] {
            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups_indirect(f.soft_dispatch.0, f.soft_dispatch.1);
        }
        pass.set_pipeline(&self.classify_count);
        pass.dispatch_workgroups(tx, ty, 1);
        pass.set_pipeline(&self.classify_prefix);
        pass.dispatch_workgroups(1, 1, 1);
        pass.set_pipeline(&self.classify_emit);
        pass.dispatch_workgroups(tx, ty, 1);
        drop(pass);

        let key = vec![
            f.draw_params.clone(),
            f.instances.clone(),
            f.items.clone(),
            f.clusters.clone(),
            f.indices.clone(),
            f.previous.clone(),
            self.words.clone(),
            f.vertices.clone(),
            self.tile_list.clone(),
            self.row_indices.clone(),
        ];
        if self.resolve_cache.as_ref().is_none_or(|(k, _)| *k != key) {
            let resolve = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("meshlet resolve"),
                layout: &self.resolve_layout,
                entries: &[
                    entry(8, f.draw_params),
                    entry(9, f.instances),
                    entry(10, f.items),
                    entry(11, f.clusters),
                    entry(12, f.indices),
                    entry(14, f.previous),
                    entry(15, &self.screen),
                    wgpu::BindGroupEntry {
                        binding: 16,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.words,
                            offset: 0,
                            size: wgpu::BufferSize::new(words),
                        }),
                    },
                    entry(17, f.vertices),
                    wgpu::BindGroupEntry {
                        binding: 18,
                        resource: wgpu::BindingResource::TextureView(&self.wind_field),
                    },
                    wgpu::BindGroupEntry {
                        binding: 19,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.tile_list,
                            offset: 0,
                            size: wgpu::BufferSize::new(tile_list_bytes),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 20,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.row_indices,
                            offset: 0,
                            size: wgpu::BufferSize::new(16),
                        }),
                    },
                ],
            });
            self.resolve_cache = Some((key, resolve));
        }
    }

    pub fn ensure_pipelines(
        &mut self,
        device: &wgpu::Device,
        layouts: [&wgpu::BindGroupLayout; 3],
        module: &wgpu::ShaderModule,
        effects: impl IntoIterator<Item = bool>,
        prepass: bool,
        samples: u32,
    ) {
        if self.refused.is_some() {
            return;
        }
        let effects: Vec<bool> = effects
            .into_iter()
            .filter(|e| !self.lit.contains_key(e))
            .collect();
        let prepass = prepass && self.prepass.is_none();
        if effects.is_empty() && !prepass {
            return;
        }
        let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("meshlet resolve"),
            bind_group_layouts: &[
                Some(layouts[0]),
                Some(layouts[1]),
                Some(layouts[2]),
                Some(&self.resolve_layout),
            ],
            immediate_size: 0,
        });
        for e in effects {
            self.lit.entry(e).or_insert_with(|| {
                crate::hdr_pipeline_with(
                    device,
                    &[
                        ("EFFECTS", e as u32 as f64),
                        ("ANALYTIC", 1.0),
                        ("DISCARD", 0.0),
                    ],
                    crate::HdrPipeline {
                        alpha_to_coverage: false,
                        label: "meshlet resolve",
                        layout: &layout,
                        module,
                        vs: "vs_meshlet_resolve",
                        fs: "fs_meshlet_resolve",
                        buffers: &[],
                        cull: None,
                        depth_write: true,
                        depth_compare: crate::LIT_DEPTH_COMPARE,
                        blend: None,
                        samples,
                    },
                )
            });
        }
        if prepass {
            self.prepass = Some(crate::prepass_pipeline(
                device,
                (&layout, &layout),
                module,
                Side::Double.into(),
                false,
                false,
                None,
                Some(("vs_meshlet_resolve", "fs_prepass_meshlet_resolve", true)),
                &[],
            ));
        }
        self.checks.push(Box::pin(validation.pop()));
        self.checks.push(Box::pin(internal.pop()));
        let _ = self.usable();
    }

    pub fn clear_pipelines(&mut self) {
        self.lit.clear();
        self.prepass = None;
    }
}
