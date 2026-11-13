//! Crowd culling on the GPU (crowd.rs, `shaders/crowd_cull.wgsl`): the
//! frame's crowd tables uploaded once, a compute dispatch over one row per
//! culled view (the camera's and every other GPU culling pass of the
//! frame: shadow cascades and light faces, the planar reflection), and per
//! row the visible members' skinned records and each (crowd, part, level)'s
//! indexed indirect draw. It runs after GPU culling's encode, so the
//! camera's row tests the depth pyramid that frame's occluders built
//! (cull_gpu.rs); without one it tests the frustum alone. Its counts (the
//! camera's hidden and drawn records, the shadow passes' records) join GPU
//! culling's in the frame's stats: written after the draws and copied into
//! GPU culling's count words before their readback.

use crate::crowd::{CROWD_CULL_WGSL, CrowdFrame};
use crate::cull_gpu::{CROWD_COUNT_WORDS, CullPass, PassKind};
use crate::skin::SKINNED_INSTANCE_STRIDE;
use glam::{Mat4, Vec3};

const WORKGROUP: u32 = 256;
const MAX_PLANES: usize = 12;

/// `Params` in crowd_cull.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    counts: [u32; 4],
    sizes: [u32; 4],
    camera: [f32; 4],
    lod: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

/// `Row` in crowd_cull.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RowGpu {
    planes: [[f32; 4]; MAX_PLANES],
    info: [u32; 4],
}

/// What the camera's row picks levels with.
pub(crate) struct CrowdView<'a> {
    pub position: Vec3,
    /// `Some(tan(fov_y / 2))` for a perspective camera, `None` with the
    /// orthographic height in `ortho_height`.
    pub half_tan: Option<f32>,
    pub ortho_height: f32,
    pub view_height: f32,
    pub view_proj: Mat4,
    pub size: (u32, u32),
    pub pyramid: Option<(&'a wgpu::TextureView, u32)>,
}

pub(crate) struct CrowdCulling {
    layout: wgpu::BindGroupLayout,
    visibility: wgpu::ComputePipeline,
    scan_blocks: wgpu::ComputePipeline,
    prefix: wgpu::ComputePipeline,
    scatter: wgpu::ComputePipeline,
    totals: wgpu::ComputePipeline,
    params: wgpu::Buffer,
    members: wgpu::Buffer,
    crowds: wgpu::Buffer,
    clips: wgpu::Buffer,
    draws: wgpu::Buffer,
    rows: wgpu::Buffer,
    scan: wgpu::Buffer,
    /// Row r's visible members' records from r × `n` (vertex buffer 1 of
    /// the skinned draws).
    pub records: wgpu::Buffer,
    /// Per (row, draw) five words of DrawIndexedIndirect, then the frame's
    /// counts ([`CROWD_COUNT_WORDS`]).
    pub args: wgpu::Buffer,
    no_pyramid: wgpu::TextureView,
    max_binding: u64,
    /// This frame's members and draws, and each culling pass's row (`None`:
    /// the pass draws no crowd).
    pub n: u32,
    pub d: u32,
    pub rows_of: Vec<Option<u32>>,
    /// Each (row, draw)'s args copied after the cull, which draws read
    /// instead of `args` on WebKit (`indirect_own.rs`); `None` elsewhere.
    per_draw: Option<crate::indirect_own::OwnArgs>,
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

impl CrowdCulling {
    pub fn new(device: &wgpu::Device, max_binding: u64, per_draw: bool) -> CrowdCulling {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("crowd cull"),
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
                storage(1, true),
                storage(2, true),
                storage(3, true),
                storage(4, true),
                storage(5, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                storage(7, false),
                storage(8, false),
                storage(9, false),
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("crowd cull"),
            source: wgpu::ShaderSource::Wgsl(CROWD_CULL_WGSL.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("crowd cull"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let compute = |entry: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(&pipeline_layout),
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: crate::pipeline_cache::of(device).as_ref(),
            })
        };
        let no_pyramid = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("crowd cull: no pyramid"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default());
        let read = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        CrowdCulling {
            visibility: compute("visibility"),
            scan_blocks: compute("scan_blocks"),
            prefix: compute("prefix"),
            scatter: compute("scatter"),
            totals: compute("totals"),
            params: buffer(
                device,
                "crowd cull params",
                std::mem::size_of::<Params>() as u64,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            members: buffer(device, "crowd members", 16, read),
            crowds: buffer(device, "crowds", 16, read),
            clips: buffer(device, "crowd clips", 16, read),
            draws: buffer(device, "crowd draws", 16, read),
            rows: buffer(device, "crowd rows", 16, read),
            scan: buffer(device, "crowd scan", 16, wgpu::BufferUsages::STORAGE),
            records: buffer(
                device,
                "crowd records",
                16,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            ),
            args: buffer(
                device,
                "crowd draws (indirect)",
                16,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::COPY_SRC,
            ),
            layout,
            no_pyramid,
            max_binding,
            n: 0,
            d: 0,
            rows_of: Vec::new(),
            per_draw: per_draw.then(Default::default),
        }
    }

    /// Whether `n` members over `rows` rows fit the device's storage
    /// binding limit (the records, the scan).
    pub fn fits(&self, n: u32, rows: usize) -> bool {
        let records = rows as u64 * n as u64 * SKINNED_INSTANCE_STRIDE;
        records <= self.max_binding && Self::scan_words(n, rows) * 16 <= self.max_binding
    }

    fn scan_words(n: u32, rows: usize) -> u64 {
        rows as u64 * (n as u64 + 1 + 2 * n.div_ceil(WORKGROUP) as u64)
    }

    /// Where row `row`'s records start in [`Self::records`], in bytes.
    pub fn row_offset(&self, row: u32) -> u64 {
        row as u64 * self.n as u64 * SKINNED_INSTANCE_STRIDE
    }

    /// The byte offset of row `row`'s draw `draw` in [`Self::args`].
    fn draw_offset(&self, row: u32, draw: u32) -> u64 {
        (row as u64 * self.d as u64 + draw as u64) * 20
    }

    /// The buffer and byte offset row `row`'s draw `draw` reads its
    /// indirect args from: [`Self::args`], or its own copy on WebKit.
    pub fn draw_args(&self, row: u32, draw: u32) -> (&wgpu::Buffer, u64) {
        match &self.per_draw {
            Some(own) => (own.get((row * self.d + draw) as usize), 0),
            None => (&self.args, self.draw_offset(row, draw)),
        }
    }

    /// Grows `buf` to `size` bytes, keeping its usage; whether it was replaced.
    fn grow(device: &wgpu::Device, buf: &mut wgpu::Buffer, label: &str, size: u64) -> bool {
        if size <= buf.size() {
            return false;
        }
        *buf = buffer(
            device,
            label,
            size.next_power_of_two().max(size),
            buf.usage(),
        );
        true
    }

    /// Encodes into the frame's `encoder` (its pass timed as
    /// "cull: crowds") the crowd cull of `frame` for the frame's GPU culling
    /// `passes` (a row each, except the occluders' pass, which draws no
    /// crowd). `previous_base`: the palette slot of the first member's
    /// previous transform, plus one (0: no motion vectors). `counts`: the
    /// buffer and word its counts are copied to (GPU culling's `args` at
    /// `crowd_counts_word`).
    #[allow(clippy::too_many_arguments)]
    pub fn encode(
        &mut self,
        encoder: &mut crate::timing::Encoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &CrowdFrame,
        index_counts: &[u32],
        passes: &[CullPass],
        view: &CrowdView<'_>,
        previous_base: u32,
        counts: (&wgpu::Buffer, u32),
    ) {
        let n = frame.len();
        // Every draw: the mesh levels', then a card's per crowd with an impostor.
        let d = index_counts.len() as u32;
        self.rows_of = passes
            .iter()
            .scan(0u32, |r, p| {
                Some((p.kind != PassKind::Occluders).then(|| {
                    *r += 1;
                    *r - 1
                }))
            })
            .collect();
        let rows: Vec<RowGpu> = passes
            .iter()
            .filter(|p| p.kind != PassKind::Occluders)
            .map(|p| {
                let mut row: RowGpu = bytemuck::Zeroable::zeroed();
                let planes = &p.planes[..p.planes.len().min(MAX_PLANES)];
                for (i, plane) in planes.iter().enumerate() {
                    row.planes[i] = plane.to_array();
                }
                let kind = match p.kind {
                    PassKind::Shadow => 0,
                    PassKind::Reflection => 1,
                    PassKind::Occluders => 2,
                    PassKind::Main => 3,
                };
                row.info = [kind, planes.len() as u32, p.layers, 0];
                row
            })
            .collect();
        let r = rows.len() as u32;
        self.n = n;
        self.d = d;
        let blocks = n.div_ceil(WORKGROUP);
        let bytes = |s: &[u8]| s.len() as u64;
        let members: &[u8] = bytemuck::cast_slice(&frame.members);
        let crowds: &[u8] = bytemuck::cast_slice(&frame.crowds);
        let clips: &[u8] = bytemuck::cast_slice(&frame.clips);
        let draws: &[u8] = bytemuck::cast_slice(index_counts);
        let rows_bytes: &[u8] = bytemuck::cast_slice(&rows);
        Self::grow(device, &mut self.members, "crowd members", bytes(members));
        Self::grow(device, &mut self.crowds, "crowds", bytes(crowds));
        Self::grow(device, &mut self.clips, "crowd clips", bytes(clips));
        Self::grow(device, &mut self.draws, "crowd draws", bytes(draws));
        Self::grow(device, &mut self.rows, "crowd rows", bytes(rows_bytes));
        Self::grow(
            device,
            &mut self.scan,
            "crowd scan",
            Self::scan_words(n, rows.len()) * 16,
        );
        Self::grow(
            device,
            &mut self.records,
            "crowd records",
            r as u64 * n as u64 * SKINNED_INSTANCE_STRIDE,
        );
        Self::grow(
            device,
            &mut self.args,
            "crowd draws (indirect)",
            (r as u64 * d as u64 * 5 + CROWD_COUNT_WORDS) * 4,
        );
        queue.write_buffer(&self.members, 0, members);
        queue.write_buffer(&self.crowds, 0, crowds);
        queue.write_buffer(&self.clips, 0, clips);
        queue.write_buffer(&self.draws, 0, draws);
        queue.write_buffer(&self.rows, 0, rows_bytes);
        let params = Params {
            counts: [n, frame.crowds.len() as u32, r, d],
            sizes: [
                blocks,
                view.pyramid.map_or(0, |p| p.1),
                view.size.0,
                view.size.1,
            ],
            camera: view.position.extend(view.view_height).to_array(),
            lod: [
                view.half_tan.is_some() as u32 as f32,
                view.half_tan.unwrap_or(view.ortho_height),
                0.0,
                previous_base as f32,
            ],
            view_proj: view.view_proj.to_cols_array_2d(),
        };
        queue.write_buffer(&self.params, 0, bytemuck::bytes_of(&params));
        // The pyramid's view changes with the frame's size: bound fresh.
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("crowd cull"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.members.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.crowds.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.clips.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.draws.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: self.rows.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(
                        view.pyramid.map_or(&self.no_pyramid, |p| p.0),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: self.scan.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: self.records.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: self.args.as_entire_binding(),
                },
            ],
        });
        if n > 0 && r > 0 {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("cull: crowds"),
                timestamp_writes: None,
            });
            pass.set_bind_group(0, &group, &[]);
            pass.set_pipeline(&self.visibility);
            pass.dispatch_workgroups(blocks, r, 1);
            pass.set_pipeline(&self.scan_blocks);
            pass.dispatch_workgroups(1, r, 1);
            pass.set_pipeline(&self.prefix);
            pass.dispatch_workgroups(blocks, r, 1);
            pass.set_pipeline(&self.scatter);
            pass.dispatch_workgroups(blocks, r, 1);
            pass.set_pipeline(&self.totals);
            pass.dispatch_workgroups(1, 1, 1);
            drop(pass);
            if let Some(own) = &mut self.per_draw {
                own.ensure(device, (r * d) as usize);
                for k in 0..r * d {
                    own.copy(encoder, &self.args, k as u64 * 20, k as usize, 20);
                }
            }
            encoder.copy_buffer_to_buffer(
                &self.args,
                r as u64 * d as u64 * 20,
                counts.0,
                counts.1 as u64 * 4,
                CROWD_COUNT_WORDS * 4,
            );
        }
    }
}
