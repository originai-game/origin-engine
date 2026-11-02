//! The meshlet path on the GPU (`meshlet.rs` states what it computes):
//! the camera pass's clustered batches, after GPU culling (`cull_gpu.rs`)
//! has culled and compacted their instances.
//!
//! Per frame: the batch table is written (each clustered batch's mesh, its
//! surviving instances' region and the word of GPU culling's draw arguments
//! that counts them, its first work item and workgroup), then one compute
//! pass (`meshlet_cull.wgsl`) runs a workgroup per instance a batch could
//! have kept: it picks the instance's level, tests the level's clusters
//! against the frustum, their normal cones and GPU culling's depth pyramid
//! (when that frame has one), and appends the visible ones as work items
//! to the batch's draw. The prepass and the lit pass then draw each batch
//! once, 64 × 3 vertices per visible cluster, by vertex pulling
//! (`meshlet.wgsl`): no index or vertex buffer is bound.
//!
//! The clusters of every clustered mesh live in pools rebuilt when the set
//! of clustered meshes changes (models load rarely): the mesh table (each
//! mesh's sphere and levels), the clusters and their indices. Each mesh's
//! own f32 vertices are read where they are.
//!
//! The shadow passes (each cascade and local light face) are views of the
//! same dispatch: a row per batch and view, each view's planes in a table
//! (view 0 the camera's). A shadow view's clusters are tested against its
//! planes alone (no normal cone: a caster's back faces are what a front-
//! sided caster draws; no pyramid: it is the camera's), at the level the
//! camera picks, and drawn with the shadow module's pulling entry points
//! (`meshlet_shadow.wgsl`).
//!
//! What the path takes is [`RenderScene::meshlet_drawable`] plus what only
//! the renderer knows (a lightmapped or compact mesh stays on the regular
//! path), and only under GPU culling; the planar reflection, views and
//! probes draw the same instances' level 0 as before.

use crate::meshlet::{CLUSTER_TRIANGLES, MeshClusters};
use crate::{Batch, RenderScene, Side};
use glam::{Mat4, Vec3, Vec4};
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) const MESHLET_WGSL: &str = wgsl!("meshlet.wgsl");
pub(crate) const MESHLET_CULL_WGSL: &str = wgsl!("meshlet_cull.wgsl");
pub(crate) const MESHLET_SHADOW_WGSL: &str = wgsl!("meshlet_shadow.wgsl");
pub(crate) const MESHLET_DEPTH_WGSL: &str = wgsl!("meshlet_depth.wgsl");

/// The lit and prepass module of the meshlet path: the standard lit module
/// (the built-in hooks; with `planar` its planar reflection slot) and the
/// pulling entry points. Its own module, never the standard one: a material
/// shader is checked against that one (og-cook `validate_shader`), and the
/// pulling entry points read `instance_index`.
pub(crate) fn lit_source(planar: bool) -> String {
    format!(
        "{}{MESHLET_WGSL}",
        crate::lit_source_with(&crate::hooks::default_hooks(), planar)
    )
}

/// The early phase's depth module.
pub(crate) fn depth_source() -> String {
    format!("{}{MESHLET_DEPTH_WGSL}", crate::INSTANCE_WGSL)
}

/// The params buffer's second slot (the late phase's), a uniform offset.
const PARAMS_SLOT: u64 = 256;

/// The shadow casters' module: the standard shadow module (the built-in
/// hooks) and the pulling entry points.
pub(crate) fn shadow_source() -> String {
    format!(
        "{}{MESHLET_SHADOW_WGSL}",
        crate::shadow_source(&crate::hooks::default_hooks())
    )
}

/// Planes per view in the view table (`meshlet_cull.wgsl` `VIEW_PLANES`):
/// a cascade's frustum is its box and the camera slice it sweeps, at most
/// cull_gpu.rs's 12.
pub(crate) const VIEW_PLANES: usize = 12;
/// Rows per view in the view table: its planes, its eye and pixels per
/// unit, its projection (`meshlet_cull.wgsl` `VIEW_ROWS`).
const VIEW_ROWS: usize = VIEW_PLANES + 2;

/// vec4 rows per mesh in the mesh table: its sphere, its level count, then
/// one per level (`meshlet_cull.wgsl` `MESH_ROWS`).
const MESH_ROWS: usize = 10;
const MAX_LEVELS: usize = MESH_ROWS - 2;
/// vec4s per cluster in the cluster pool.
const CLUSTER_ROWS: usize = 3;
/// A batch's work items start at a multiple of this (8-byte items: a
/// 256-byte storage offset, the most any device asks).
const ITEM_ALIGN: u32 = 32;
const WORKGROUPS_PER_ROW: u32 = 65535;

/// `struct Params` in meshlet_cull.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    view_proj: [[f32; 4]; 4],
    lod: [f32; 4],
    sizes: [u32; 4],
    phase: [u32; 4],
}

/// `struct BatchRow` in meshlet_cull.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct BatchRow {
    pub mesh_slot: u32,
    pub first_instance: u32,
    pub instances: u32,
    pub first_item: u32,
    pub first_workgroup: u32,
    pub flags: u32,
    /// Its view: 0 the camera, then the frame's shadow views.
    pub view: u32,
    /// Its mesh's most clusters at any level: the stride of an
    /// instance's cluster slots (`meshlet_cull.wgsl` `vis`).
    pub most: u32,
}

/// One clustered batch of this frame in one view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeshletDraw {
    /// Its index in the frame's batches.
    pub batch: u32,
    /// The view it is drawn in (0: the camera).
    pub view: u32,
    /// Whether that view is drawn lit (not a shadow pass).
    pub lit: bool,
    /// Its row in the batch table: its draw's offset in `draws`.
    pub row: u32,
    pub first_item: u32,
}

/// Whether `b` draws through the meshlet path, given the scene and what
/// only the renderer knows about its mesh (`lightmapped`).
pub(crate) fn takes(scene: &RenderScene, b: &Batch, clustered: bool, lightmapped: bool) -> bool {
    scene.meshlets
        && clustered
        && !b.transparent
        && b.shader.is_none()
        && !b.physical
        && !b.array
        && !b.compact
        && !lightmapped
}

/// One view of the frame's dispatch: its planes, the batches it takes
/// (indices into the frame's batches) and the eye its levels are picked
/// from. The first is the camera's; a shadow view (not `lit`) carries the
/// camera's eye and never culls by the normal cone.
#[derive(Clone, Debug)]
pub(crate) struct MeshletView {
    pub planes: Vec<Vec4>,
    pub taken: Vec<u32>,
    pub eye: Vec3,
    /// Pixels per unit of length at distance 1 (perspective) or anywhere
    /// (orthographic).
    pub per_unit: f32,
    pub perspective: bool,
    /// Drawn lit (the camera, a reflection, a view, a probe face), not a
    /// shadow pass.
    pub lit: bool,
}

impl MeshletView {
    /// The camera's view: its pixels per unit from its projection and
    /// the frame's `height` in pixels (lod.rs `pixels_per_unit`).
    pub fn camera(camera: &crate::Camera, planes: &[Vec4], taken: Vec<u32>, height: u32) -> Self {
        let (perspective, per_unit) = match camera.projection {
            crate::Projection::Perspective { fov_y_degrees } => (
                true,
                height as f32 / (2.0 * (fov_y_degrees.to_radians() * 0.5).tan().max(1e-6)),
            ),
            crate::Projection::Orthographic { height: h } => (false, height as f32 / h.max(1e-6)),
        };
        MeshletView {
            planes: planes.to_vec(),
            taken,
            eye: camera.position,
            per_unit,
            perspective,
            lit: true,
        }
    }

    /// A secondary view seen from `eye` through `view_proj` (a rigid view
    /// times a projection, a mirror's or an oblique one included) into a
    /// target `height` pixels high: half the height times the length of
    /// its clip y row is the pixels per unit at distance 1 for a
    /// perspective projection, and anywhere for an orthographic one.
    pub fn seen(planes: &[Vec4], taken: Vec<u32>, eye: Vec3, view_proj: Mat4, height: u32) -> Self {
        MeshletView {
            planes: planes.to_vec(),
            taken,
            eye,
            per_unit: 0.5 * height as f32 * view_proj.row(1).truncate().length(),
            perspective: view_proj.row(3).truncate().length() > 1e-6,
            lit: true,
        }
    }

    /// A shadow pass's view of `taken` within `planes`, its levels the
    /// camera's (`camera` the camera's view).
    pub fn shadow(camera: &MeshletView, planes: &[Vec4], taken: Vec<u32>) -> Self {
        MeshletView {
            planes: planes.to_vec(),
            taken,
            lit: false,
            ..camera.clone()
        }
    }
}

/// The frame's batch table and the work its dispatch covers, from each
/// view's clustered batches' instances (their range of the sorted
/// instances) and their meshes' most clusters at any level: (rows, draws,
/// items, workgroups). Only the rows of lit views (`true` beside their
/// batches) may cull by `cone`.
pub(crate) fn plan<'a>(
    batches: &[Batch],
    views: impl IntoIterator<Item = (&'a [u32], bool)>,
    slot: impl Fn(usize) -> Option<(u32, u32)>,
    cone: impl Fn(&Batch) -> bool,
) -> (Vec<BatchRow>, Vec<MeshletDraw>, u32, u32) {
    let mut rows = Vec::new();
    let mut draws = Vec::new();
    let (mut items, mut workgroups) = (0u32, 0u32);
    for (view, lit, &b) in views
        .into_iter()
        .enumerate()
        .flat_map(|(v, (taken, lit))| taken.iter().map(move |b| (v as u32, lit, b)))
    {
        let batch = &batches[b as usize];
        let Some((mesh_slot, most)) = slot(batch.mesh) else {
            continue;
        };
        if batch.count == 0 {
            continue;
        }
        let first_item = items.div_ceil(ITEM_ALIGN) * ITEM_ALIGN;
        rows.push(BatchRow {
            mesh_slot,
            first_instance: batch.first,
            instances: batch.count,
            first_item,
            first_workgroup: workgroups,
            flags: (lit && cone(batch)) as u32,
            view,
            most,
        });
        draws.push(MeshletDraw {
            batch: b,
            view,
            lit,
            row: rows.len() as u32 - 1,
            first_item,
        });
        items = first_item + batch.count * most;
        workgroups += batch.count;
    }
    (rows, draws, items, workgroups)
}

/// The bytes a draw group binds of the work items from a batch's first
/// (its dynamic offset): the largest batch's items, rounded up to the
/// item alignment; and the bytes the items buffer needs so every batch's
/// window lies inside it (wgpu refuses an offset whose window would pass
/// the buffer's end). `items` is the plan's total.
pub(crate) fn item_window(rows: &[BatchRow], items: u32) -> (u64, u64) {
    let window = rows
        .iter()
        .map(|r| r.instances.saturating_mul(r.most).div_ceil(ITEM_ALIGN) * ITEM_ALIGN)
        .max()
        .unwrap_or(0)
        .max(ITEM_ALIGN) as u64
        * 8;
    let last = rows.iter().map(|r| r.first_item).max().unwrap_or(0) as u64 * 8;
    (window, (items as u64 * 8).max(last + window))
}

/// One mesh's rows in the mesh table.
type MeshRows = [[u32; 4]; MESH_ROWS];

/// The mesh table's rows and the cluster pool's for one mesh, its
/// clusters' first indices offset by `index_base` and its levels' first
/// clusters by `cluster_base`; `None` past [`MAX_LEVELS`] levels.
pub(crate) fn pool_rows(
    c: &MeshClusters,
    index_base: u32,
    cluster_base: u32,
) -> Option<(MeshRows, Vec<[u32; 4]>)> {
    if c.levels.len() > MAX_LEVELS || c.clusters.is_empty() {
        return None;
    }
    let bits = |v: f32| v.to_bits();
    let first = &c.clusters[0];
    let mut mesh = [[0u32; 4]; MESH_ROWS];
    mesh[0] = [
        bits(first.error_center.x),
        bits(first.error_center.y),
        bits(first.error_center.z),
        bits(first.error_radius),
    ];
    mesh[1] = [c.levels.len() as u32, 0, 0, 0];
    for (k, l) in c.levels.iter().enumerate() {
        mesh[2 + k] = [
            cluster_base + l.clusters.start,
            l.clusters.end - l.clusters.start,
            bits(l.error),
            0,
        ];
    }
    let mut rows = Vec::with_capacity(c.clusters.len() * CLUSTER_ROWS);
    for (k, l) in c.levels.iter().enumerate() {
        for cl in &c.clusters[l.clusters.start as usize..l.clusters.end as usize] {
            rows.push([index_base + cl.first, cl.triangles, k as u32, 0]);
            rows.push([
                bits(cl.center.x),
                bits(cl.center.y),
                bits(cl.center.z),
                bits(cl.radius),
            ]);
            rows.push([
                bits(cl.cone_axis.x),
                bits(cl.cone_axis.y),
                bits(cl.cone_axis.z),
                bits(cl.cone_cutoff),
            ]);
        }
    }
    Some((mesh, rows))
}

/// The clusters of every clustered mesh, on the GPU.
struct Pools {
    /// What they were built from: each clustered mesh and its clusters.
    key: Vec<(usize, Arc<MeshClusters>)>,
    /// Per mesh index: its slot in the mesh table and its most clusters at
    /// any level.
    slots: HashMap<usize, (u32, u32)>,
    meshes: wgpu::Buffer,
    clusters: wgpu::Buffer,
    indices: wgpu::Buffer,
}

pub(crate) struct MeshletGpu {
    cull_layout: wgpu::BindGroupLayout,
    cull: wgpu::ComputePipeline,
    pub(crate) draw_layout: wgpu::BindGroupLayout,
    lit: HashMap<(Side, bool), wgpu::RenderPipeline>,
    prepass: HashMap<Side, wgpu::RenderPipeline>,
    /// The lit module ([`lit_source`]) and its planar slot.
    lit_module: Option<(bool, wgpu::ShaderModule)>,
    /// The shadow casters' pipelines by (side, the whole vertex for an
    /// alpha-tested or dissolving material), and their module.
    shadow: HashMap<(Side, bool), wgpu::RenderPipeline>,
    shadow_module: Option<wgpu::ShaderModule>,
    pools: Option<Pools>,
    params: wgpu::Buffer,
    views: wgpu::Buffer,
    /// Whether each cluster slot was visible, two halves of `vis_half`
    /// words (last frame's and this frame's, swapped by `vis_parity`);
    /// `vis_clear`: clear both before the next use.
    vis: wgpu::Buffer,
    vis_half: u64,
    vis_parity: bool,
    vis_clear: bool,
    depth_module: wgpu::ShaderModule,
    depth_pipeline_layout: wgpu::PipelineLayout,
    depth_group: wgpu::BindGroup,
    depth_pipelines: HashMap<(Side, u32), wgpu::RenderPipeline>,
    draw_params: wgpu::Buffer,
    batches: wgpu::Buffer,
    items: wgpu::Buffer,
    draws: wgpu::Buffer,
    no_pyramid: wgpu::TextureView,
    /// The cull group and what it binds (GPU culling's instances and
    /// arguments, the pyramid's view, the pools' and this frame's buffers).
    cull_group: Option<(
        Vec<wgpu::Buffer>,
        Option<wgpu::TextureView>,
        u64,
        wgpu::BindGroup,
    )>,
    /// Per mesh: its draw group and what it binds (the buffers, and the
    /// bytes of `items` its window binds).
    mesh_groups: HashMap<usize, ((Vec<wgpu::Buffer>, u64), wgpu::BindGroup)>,
    /// The bytes of `items` a draw group binds from a batch's first work
    /// item (its dynamic offset): the frame's largest batch's items.
    window: u64,
    /// The bytes of `items` this frame's dispatch writes.
    items_bytes: u64,
    /// This frame's clustered batches (empty: the path is off this frame).
    pub(crate) frame: Vec<MeshletDraw>,
    max_binding: u64,
}

fn storage(
    binding: u32,
    read_only: bool,
    stages: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: stages,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn uniform(binding: u32, stages: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: stages,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
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

fn storage_usage() -> wgpu::BufferUsages {
    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
}

/// Grows `buf` to hold `size` bytes; whether it was replaced.
fn grow(
    device: &wgpu::Device,
    buf: &mut wgpu::Buffer,
    label: &str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> bool {
    if size <= buf.size() {
        return false;
    }
    *buf = buffer(device, label, size.next_power_of_two(), usage);
    true
}

impl MeshletGpu {
    pub fn new(device: &wgpu::Device, max_binding: u64) -> MeshletGpu {
        let c = wgpu::ShaderStages::COMPUTE;
        let cull_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("meshlet cull"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: c,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                },
                storage(1, true, c),
                storage(2, true, c),
                storage(3, true, c),
                storage(4, true, c),
                storage(5, true, c),
                storage(6, false, c),
                storage(7, false, c),
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: c,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                storage(9, false, c),
            ],
        });
        let v = wgpu::ShaderStages::VERTEX;
        let draw_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("meshlet draw"),
            entries: &[
                uniform(8, v),
                storage(9, true, v),
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: v,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                },
                storage(11, true, v),
                storage(12, true, v),
                storage(13, true, v),
                storage(14, true, v),
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("meshlet cull"),
            source: wgpu::ShaderSource::Wgsl(MESHLET_CULL_WGSL.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("meshlet cull"),
            bind_group_layouts: &[Some(&cull_layout)],
            immediate_size: 0,
        });
        let cull = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("meshlet cull"),
            layout: Some(&layout),
            module: &module,
            entry_point: Some("cull_clusters"),
            compilation_options: Default::default(),
            cache: crate::pipeline_cache::of(device).as_ref(),
        });
        let no_pyramid = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("meshlet: no depth pyramid"),
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
        let params = buffer(
            device,
            "meshlet params",
            PARAMS_SLOT * 2,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let depth_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("meshlet early depth"),
            entries: &[uniform(0, v)],
        });
        let depth_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("meshlet early depth"),
            layout: &depth_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &params,
                    offset: 0,
                    size: wgpu::BufferSize::new(64),
                }),
            }],
        });
        let depth_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("meshlet early depth"),
                bind_group_layouts: &[Some(&depth_layout), Some(&draw_layout)],
                immediate_size: 0,
            });
        let depth_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("meshlet early depth"),
            source: wgpu::ShaderSource::Wgsl(depth_source().into()),
        });
        MeshletGpu {
            cull_layout,
            cull,
            draw_layout,
            lit: HashMap::new(),
            prepass: HashMap::new(),
            lit_module: None,
            shadow: HashMap::new(),
            shadow_module: None,
            pools: None,
            params,
            vis: buffer(device, "meshlet visibility", 0, storage_usage()),
            vis_half: 0,
            vis_parity: false,
            vis_clear: true,
            depth_module,
            depth_pipeline_layout,
            depth_group,
            depth_pipelines: HashMap::new(),
            draw_params: buffer(
                device,
                "meshlet draw params",
                16,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            views: buffer(device, "meshlet views", 0, storage_usage()),
            batches: buffer(device, "meshlet batches", 0, storage_usage()),
            items: buffer(device, "meshlet items", 0, storage_usage()),
            draws: buffer(device, "meshlet draws", 0, Self::draws_usage()),
            no_pyramid,
            cull_group: None,
            mesh_groups: HashMap::new(),
            window: 0,
            items_bytes: 0,
            frame: Vec::new(),
            max_binding,
        }
    }

    fn draws_usage() -> wgpu::BufferUsages {
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST
    }

    /// Rebuilds the pools when the scene's clustered meshes changed.
    pub fn sync(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, scene: &RenderScene) {
        let key: Vec<(usize, Arc<MeshClusters>)> = scene
            .mesh_clusters_all()
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.clone().map(|c| (i, c)))
            .collect();
        let same = self.pools.as_ref().is_some_and(|p| {
            p.key.len() == key.len()
                && p.key
                    .iter()
                    .zip(&key)
                    .all(|(a, b)| a.0 == b.0 && Arc::ptr_eq(&a.1, &b.1))
        });
        if same {
            return;
        }
        let mut slots = HashMap::new();
        let mut mesh_rows: Vec<[u32; 4]> = Vec::new();
        let mut cluster_rows: Vec<[u32; 4]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        for (i, c) in &key {
            let Some((mesh, rows)) = pool_rows(
                c,
                indices.len() as u32,
                (cluster_rows.len() / CLUSTER_ROWS) as u32,
            ) else {
                continue;
            };
            let bytes = ((indices.len() + c.indices.len()) * 4)
                .max((cluster_rows.len() + rows.len()) * 16) as u64;
            if bytes > self.max_binding {
                break;
            }
            let most = c
                .levels
                .iter()
                .map(|l| l.clusters.end - l.clusters.start)
                .max()
                .unwrap_or(0);
            slots.insert(*i, ((mesh_rows.len() / MESH_ROWS) as u32, most));
            mesh_rows.extend_from_slice(&mesh);
            cluster_rows.extend(rows);
            indices.extend_from_slice(&c.indices);
        }
        let make = |label: &str, bytes: &[u8]| {
            let b = buffer(device, label, bytes.len() as u64, storage_usage());
            queue.write_buffer(&b, 0, bytes);
            b
        };
        self.pools = Some(Pools {
            key,
            slots,
            meshes: make("meshlet meshes", bytemuck::cast_slice(&mesh_rows)),
            clusters: make("meshlet clusters", bytemuck::cast_slice(&cluster_rows)),
            indices: make("meshlet indices", bytemuck::cast_slice(&indices)),
        });
        self.cull_group = None;
        self.mesh_groups.clear();
    }

    /// Whether `mesh` is in the pools (clustered, and within their limits).
    pub fn clustered(&self, mesh: usize) -> bool {
        self.pools
            .as_ref()
            .is_some_and(|p| p.slots.contains_key(&mesh))
    }

    /// The lit and prepass pipelines for the frame's clustered batches.
    pub fn ensure_pipelines(
        &mut self,
        device: &wgpu::Device,
        layouts: [&wgpu::BindGroupLayout; 3],
        planar: bool,
        batches: &[Batch],
        prepass: bool,
        samples: u32,
    ) {
        if self.lit_module.as_ref().is_none_or(|(p, _)| *p != planar) {
            self.lit.clear();
            self.prepass.clear();
            self.lit_module = Some((
                planar,
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("pbr meshlet"),
                    source: wgpu::ShaderSource::Wgsl(lit_source(planar).into()),
                }),
            ));
        }
        let module = &self.lit_module.as_ref().expect("made above").1;
        let camera = || self.frame.iter().filter(|d| d.lit);
        let missing = camera().any(|d| {
            let b = &batches[d.batch as usize];
            !self.lit.contains_key(&(b.side, b.effects))
                || (prepass && !self.prepass.contains_key(&b.side))
        });
        if !missing {
            return;
        }
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pbr meshlet"),
            bind_group_layouts: &[
                Some(layouts[0]),
                Some(layouts[1]),
                Some(layouts[2]),
                Some(&self.draw_layout),
            ],
            immediate_size: 0,
        });
        for d in self.frame.iter().filter(|d| d.lit) {
            let b = &batches[d.batch as usize];
            self.lit.entry((b.side, b.effects)).or_insert_with(|| {
                crate::hdr_pipeline_with(
                    device,
                    &[("EFFECTS", b.effects as u32 as f64)],
                    crate::HdrPipeline {
                        alpha_to_coverage: true,
                        label: "pbr meshlet",
                        layout: &layout,
                        module,
                        vs: "vs_meshlet",
                        fs: "fs",
                        buffers: &[],
                        cull: crate::cull_mode(b.side),
                        depth_write: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        blend: None,
                        samples,
                    },
                )
            });
            if prepass {
                self.prepass.entry(b.side).or_insert_with(|| {
                    crate::prepass_pipeline(
                        device,
                        (&layout, &layout),
                        module,
                        b.side,
                        false,
                        None,
                        true,
                    )
                });
            }
        }
    }

    /// The shadow casters' pipelines for the frame's shadow views: groups
    /// 0 the shadow pass's slot, 1 the material (alpha-tested and
    /// dissolving casters), 3 the meshlet draw group.
    pub fn ensure_shadow_pipelines(
        &mut self,
        device: &wgpu::Device,
        (pass, material): (&wgpu::BindGroupLayout, &wgpu::BindGroupLayout),
        batches: &[Batch],
    ) {
        let key = |d: &MeshletDraw| {
            let b = &batches[d.batch as usize];
            (b.side, b.dissolve)
        };
        let shadows = || self.frame.iter().filter(|d| !d.lit);
        if shadows().all(|d| self.shadow.contains_key(&key(d))) {
            return;
        }
        let module = self.shadow_module.get_or_insert_with(|| {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shadow meshlet"),
                source: wgpu::ShaderSource::Wgsl(shadow_source().into()),
            })
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow meshlet"),
            bind_group_layouts: &[Some(pass), Some(material), None, Some(&self.draw_layout)],
            immediate_size: 0,
        });
        let keys: Vec<(Side, bool)> = self.frame.iter().filter(|d| !d.lit).map(key).collect();
        for k in keys {
            self.shadow.entry(k).or_insert_with(|| {
                crate::meshlet_shadow_pipeline(device, &layout, module, k.0, k.1)
            });
        }
    }

    /// The work items `taken` adds to a frame's dispatch, rounded up to a
    /// batch's alignment (`None` before the pools exist): the renderer
    /// adds views while their sum fits [`Self::max_items`].
    pub fn items(&self, batches: &[Batch], taken: &[u32]) -> Option<u64> {
        let pools = self.pools.as_ref()?;
        let (_, _, items, _) = plan(
            batches,
            [(taken, false)],
            |m| pools.slots.get(&m).copied(),
            |_| false,
        );
        Some(items.div_ceil(ITEM_ALIGN) as u64 * ITEM_ALIGN as u64)
    }

    /// The most work items one frame's dispatch may write (the device's
    /// storage binding limit).
    pub fn max_items(&self) -> u64 {
        self.max_binding / 8
    }

    /// Plans the frame's clustered batches in each of `views` (the
    /// camera's first, then the shadow passes') and encodes their culling
    /// after GPU culling's commands, reading the sorted instances it
    /// uploaded (`gc`), and makes the frame's draw groups (`previous`, the
    /// previous transforms at the same slots; `vertices(mesh)`, a mesh's
    /// f32 vertices). With GPU culling's pyramid this frame the camera's
    /// clusters are culled in two phases: those visible last frame first,
    /// drawn into the occluders' depth (the batches `occluder` names), the
    /// pyramid built again, then the rest tested against it. `view_proj`
    /// is the camera's. Leaves [`Self::frame`] empty, and encodes nothing,
    /// when the work would not fit the device.
    #[allow(clippy::too_many_arguments)]
    pub fn encode<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        batches: &[Batch],
        views: &[MeshletView],
        gc: &crate::cull_gpu::GpuCulling,
        view_proj: Mat4,
        (width, height): (u32, u32),
        previous: &wgpu::Buffer,
        vertices: impl Fn(usize) -> &'a wgpu::Buffer,
        occluder: impl Fn(&Batch) -> bool,
    ) -> Option<wgpu::CommandBuffer> {
        self.frame.clear();
        let pools = self.pools.as_ref()?;
        let (rows, draws, items, workgroups) = plan(
            batches,
            views.iter().map(|v| (v.taken.as_slice(), v.lit)),
            |m| pools.slots.get(&m).copied(),
            |b| b.side == Side::Front,
        );
        if rows.is_empty() || items as u64 * 8 > self.max_binding {
            return None;
        }
        grow(
            device,
            &mut self.batches,
            "meshlet batches",
            (rows.len() * 32) as u64,
            storage_usage(),
        );
        // A draw binds a window of `items` from its batch's first item,
        // as wide as the largest batch's items: the buffer holds the last
        // batch's whole window.
        let (window, items_buffer) = item_window(&rows, items);
        self.window = window;
        self.items_bytes = items as u64 * 8;
        grow(
            device,
            &mut self.items,
            "meshlet items",
            items_buffer,
            storage_usage(),
        );
        grow(
            device,
            &mut self.draws,
            "meshlet draws",
            (rows.len() * 16) as u64,
            Self::draws_usage(),
        );
        // The visibility halves: a new size starts every slot hidden (the
        // late phase then finds what the early one missed).
        let half = (items as u64).max(1);
        if half != self.vis_half || half * 8 > self.vis.size() {
            if half * 8 > self.vis.size() {
                self.vis = buffer(device, "meshlet visibility", half * 8, storage_usage());
            }
            self.vis_half = half;
            self.vis_clear = true;
        }
        self.vis_parity = !self.vis_parity;
        let (read, write) = if self.vis_parity {
            (half, 0)
        } else {
            (0, half)
        };
        let mut planes = vec![[0.0f32; 4]; views.len() * VIEW_ROWS];
        for (v, view) in views.iter().enumerate() {
            let rows = &mut planes[v * VIEW_ROWS..(v + 1) * VIEW_ROWS];
            for (row, p) in rows.iter_mut().zip(view.planes.iter().take(VIEW_PLANES)) {
                *row = p.to_array();
            }
            rows[VIEW_PLANES] = view.eye.extend(view.per_unit).to_array();
            rows[VIEW_PLANES + 1] = [view.perspective as u32 as f32, 0.0, 0.0, 0.0];
        }
        grow(
            device,
            &mut self.views,
            "meshlet views",
            (planes.len() * 16) as u64,
            storage_usage(),
        );
        queue.write_buffer(&self.views, 0, bytemuck::cast_slice(&planes));
        queue.write_buffer(&self.batches, 0, bytemuck::cast_slice(&rows));
        let draw_args: Vec<[u32; 4]> = rows
            .iter()
            .map(|_| [(CLUSTER_TRIANGLES * 3) as u32, 0, 0, 0])
            .collect();
        queue.write_buffer(&self.draws, 0, bytemuck::cast_slice(&draw_args));
        // Two phases with the pyramid and its depth, one without.
        let pyramid = gc.pyramid();
        let depth = gc.occluder_depth().filter(|_| pyramid.is_some());
        let params = |phase: u32| Params {
            view_proj: view_proj.to_cols_array_2d(),
            lod: [scene.lod_bias.max(0.0), 0.0, 0.0, 0.0],
            sizes: [width, height, pyramid.map_or(0, |p| p.1), rows.len() as u32],
            phase: [phase, read as u32, write as u32, 0],
        };
        let (first, second) = if depth.is_some() { (1, 2) } else { (0, 0) };
        queue.write_buffer(&self.params, 0, bytemuck::bytes_of(&params(first)));
        queue.write_buffer(
            &self.params,
            PARAMS_SLOT,
            bytemuck::bytes_of(&params(second)),
        );
        queue.write_buffer(
            &self.draw_params,
            0,
            bytemuck::cast_slice(&[scene.lod_debug as u32, 0, 0, 0]),
        );
        let items_bytes = self.items_bytes;
        let bound = vec![
            gc.instances.clone(),
            self.views.clone(),
            self.batches.clone(),
            self.items.clone(),
            self.draws.clone(),
            pools.meshes.clone(),
            self.vis.clone(),
        ];
        let view = pyramid.map(|p| p.0.clone());
        if self
            .cull_group
            .as_ref()
            .is_none_or(|(b, v, n, _)| *b != bound || *v != view || *n != items_bytes)
        {
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("meshlet cull"),
                layout: &self.cull_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.params,
                            offset: 0,
                            size: wgpu::BufferSize::new(std::mem::size_of::<Params>() as u64),
                        }),
                    },
                    entry(1, &gc.instances),
                    entry(2, &self.views),
                    entry(3, &self.batches),
                    entry(4, &pools.meshes),
                    entry(5, &pools.clusters),
                    // The dispatch's items, never past the binding limit
                    // (the buffer may be larger: windows, power-of-two
                    // growth).
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.items,
                            offset: 0,
                            size: wgpu::BufferSize::new(items_bytes),
                        }),
                    },
                    entry(7, &self.draws),
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::TextureView(
                            view.as_ref().unwrap_or(&self.no_pyramid),
                        ),
                    },
                    entry(9, &self.vis),
                ],
            });
            self.cull_group = Some((bound, view, items_bytes, group));
        }
        self.frame = draws;
        self.ensure_groups(device, batches, (&gc.instances, previous), vertices);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("meshlet cull"),
        });
        if std::mem::take(&mut self.vis_clear) {
            encoder.clear_buffer(&self.vis, 0, None);
        } else {
            encoder.clear_buffer(&self.vis, write * 4, Some(half * 4));
        }
        let cull_group = &self.cull_group.as_ref().expect("made above").3;
        let dispatch = |pass: &mut wgpu::ComputePass<'_>, slot: u64| {
            pass.set_pipeline(&self.cull);
            pass.set_bind_group(0, cull_group, &[slot as u32]);
            pass.dispatch_workgroups(
                workgroups.min(WORKGROUPS_PER_ROW),
                workgroups.div_ceil(WORKGROUPS_PER_ROW),
                1,
            );
        };
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("meshlet cull"),
                timestamp_writes: None,
            });
            dispatch(&mut pass, 0);
        }
        if let Some((depth, samples)) = depth {
            // The early phase's camera clusters into the occluders' depth.
            for d in self.frame.iter().filter(|d| d.view == 0) {
                let b = &batches[d.batch as usize];
                if occluder(b) {
                    let key = (b.side, samples);
                    if !self.depth_pipelines.contains_key(&key) {
                        let pipeline = self.depth_pipeline(device, b.side, samples);
                        self.depth_pipelines.insert(key, pipeline);
                    }
                }
            }
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("meshlet early depth"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_bind_group(0, &self.depth_group, &[]);
                for d in self.frame.iter().filter(|d| d.view == 0) {
                    let b = &batches[d.batch as usize];
                    let (Some(pipeline), Some((_, group))) = (
                        self.depth_pipelines.get(&(b.side, samples)),
                        self.mesh_groups.get(&b.mesh),
                    ) else {
                        continue;
                    };
                    if !occluder(b) {
                        continue;
                    }
                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(1, group, &[d.first_item * 8]);
                    pass.draw_indirect(&self.draws, d.row as u64 * 16);
                }
            }
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("meshlet late"),
                timestamp_writes: None,
            });
            gc.rebuild_pyramid(&mut pass);
            dispatch(&mut pass, PARAMS_SLOT);
        }
        Some(encoder.finish())
    }

    /// The early phase's depth pipeline for `side` at `samples`.
    fn depth_pipeline(
        &self,
        device: &wgpu::Device,
        side: Side,
        samples: u32,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("meshlet early depth"),
            layout: Some(&self.depth_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.depth_module,
                entry_point: Some("vs_depth"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: crate::cull_mode(side),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: crate::DEPTH_FORMAT,
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
            cache: crate::pipeline_cache::of(device).as_ref(),
        })
    }

    /// Makes the draw groups of the frame's meshes: `instances` the
    /// sorted instances, `previous` their previous transforms at the same
    /// slots, `vertices(mesh)` a mesh's f32 vertex buffer.
    fn ensure_groups<'a>(
        &mut self,
        device: &wgpu::Device,
        batches: &[Batch],
        (instances, previous): (&wgpu::Buffer, &wgpu::Buffer),
        vertices: impl Fn(usize) -> &'a wgpu::Buffer,
    ) {
        let Some(pools) = self.pools.as_ref() else {
            return;
        };
        for d in &self.frame {
            let mesh = batches[d.batch as usize].mesh;
            let v = vertices(mesh);
            let bound = vec![
                instances.clone(),
                self.items.clone(),
                pools.clusters.clone(),
                pools.indices.clone(),
                v.clone(),
                previous.clone(),
            ];
            let bound = (bound, self.window);
            if self
                .mesh_groups
                .get(&mesh)
                .is_some_and(|(b, _)| *b == bound)
            {
                continue;
            }
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("meshlet draw"),
                layout: &self.draw_layout,
                entries: &[
                    entry(8, &self.draw_params),
                    entry(9, instances),
                    wgpu::BindGroupEntry {
                        binding: 10,
                        // A window from the batch's first item (the
                        // dynamic offset), within the buffer for every one.
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.items,
                            offset: 0,
                            size: wgpu::BufferSize::new(self.window),
                        }),
                    },
                    entry(11, &pools.clusters),
                    entry(12, &pools.indices),
                    entry(13, v),
                    entry(14, previous),
                ],
            });
            self.mesh_groups.insert(mesh, (bound, group));
        }
    }

    /// Draws the frame's clustered batches of lit view `view` (0 the
    /// camera) into `pass` (the camera's lit pass, or its prepass with
    /// `prepass`; a reflection's, a view's or a probe face's lit pass):
    /// groups 0 and 2 are bound by the caller; `material(b)` is batch `b`'s
    /// material group; `probe(pass, b)` binds its reflection probe.
    /// Returns the draws made.
    pub fn draw<'m>(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        batches: &[Batch],
        view: u32,
        prepass: bool,
        material: impl Fn(&Batch) -> &'m wgpu::BindGroup,
        mut probe: impl FnMut(&mut wgpu::RenderPass<'_>, &Batch),
    ) -> u32 {
        let mut n = 0;
        for d in self.frame.iter().filter(|d| d.view == view && d.lit) {
            let b = &batches[d.batch as usize];
            let pipeline = if prepass {
                self.prepass.get(&b.side)
            } else {
                self.lit.get(&(b.side, b.effects))
            };
            let (Some(pipeline), Some((_, group))) = (pipeline, self.mesh_groups.get(&b.mesh))
            else {
                continue;
            };
            probe(pass, b);
            pass.set_pipeline(pipeline);
            pass.set_bind_group(1, material(b), &[]);
            pass.set_bind_group(3, group, &[d.first_item * 8]);
            pass.draw_indirect(&self.draws, d.row as u64 * 16);
            n += 1;
        }
        n
    }

    /// Draws the clustered casters of shadow view `view` into a shadow
    /// pass whose slot (group 0) the caller bound; `material(b)` is batch
    /// `b`'s material group (read by alpha-tested and dissolving casters).
    /// Returns the draws made.
    pub fn draw_shadow<'m>(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        batches: &[Batch],
        view: u32,
        material: impl Fn(&Batch) -> &'m wgpu::BindGroup,
    ) -> u32 {
        let mut n = 0;
        for d in self.frame.iter().filter(|d| d.view == view) {
            let b = &batches[d.batch as usize];
            let (Some(pipeline), Some((_, group))) = (
                self.shadow.get(&(b.side, b.dissolve)),
                self.mesh_groups.get(&b.mesh),
            ) else {
                continue;
            };
            pass.set_pipeline(pipeline);
            pass.set_bind_group(1, material(b), &[]);
            pass.set_bind_group(3, group, &[d.first_item * 8]);
            pass.draw_indirect(&self.draws, d.row as u64 * 16);
            n += 1;
        }
        n
    }
}

fn entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meshlet::{Cluster, ClusterLevel};
    use glam::Vec3;

    fn batch(mesh: usize, first: u32, count: u32) -> Batch {
        Batch {
            mesh,
            material: 0,
            first,
            count,
            side: Side::Front,
            transparent: false,
            blend: false,
            physical: false,
            transmissive: false,
            dissolve: false,
            effects: false,
            shader: None,
            lean: false,
            derivatives: false,
            compact: false,
            array: false,
            cast: true,
            layers: 1,
            probe: 0,
        }
    }

    #[test]
    fn each_batch_gets_aligned_items_for_its_worst_level_and_a_workgroup_per_instance() {
        let batches = [
            batch(0, 0, 3),
            batch(5, 3, 0),
            batch(1, 3, 10),
            batch(9, 13, 2),
        ];
        // Mesh 9 is not in the pools: skipped. Batch 1 has no instances.
        let slot = |m: usize| match m {
            0 => Some((0, 7)),
            1 => Some((1, 40)),
            5 => Some((2, 1)),
            _ => None,
        };
        let (rows, draws, items, workgroups) =
            plan(&batches, [(&[0, 1, 2, 3][..], true)], slot, |_| true);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            draws
                .iter()
                .map(|d| (d.batch, d.row, d.first_item))
                .collect::<Vec<_>>(),
            [(0, 0, 0), (2, 1, 32)]
        );
        // Batch 0: 3 x 7 = 21 items, then batch 2 from 32: 10 x 40.
        assert_eq!(items, 32 + 400);
        assert_eq!(workgroups, 13);
        assert_eq!(rows[1].first_workgroup, 3);
        assert_eq!((rows[1].first_instance, rows[1].instances), (3, 10));
        assert_eq!((rows[1].mesh_slot, rows[1].flags), (1, 1));
        // An instance's cluster slots are its mesh's most clusters apart.
        assert_eq!((rows[0].most, rows[1].most), (7, 40));
    }

    #[test]
    fn each_view_gets_rows_of_its_own_and_only_lit_views_cull_by_the_cone() {
        let batches = [batch(0, 0, 3), batch(1, 3, 10)];
        let slot = |m: usize| Some((m as u32, if m == 0 { 7 } else { 40 }));
        // The camera takes both; a reflection (lit) and a shadow view take
        // batch 1 only.
        let views: [(&[u32], bool); 3] = [(&[0, 1], true), (&[1], true), (&[1], false)];
        let (rows, draws, items, workgroups) = plan(&batches, views, slot, |_| true);
        assert_eq!(
            rows.iter()
                .map(|r| (r.view, r.flags, r.first_instance))
                .collect::<Vec<_>>(),
            [(0, 1, 0), (0, 1, 3), (1, 1, 3), (2, 0, 3)]
        );
        assert_eq!(
            draws.iter().map(|d| d.lit).collect::<Vec<_>>(),
            [true, true, true, false]
        );
        // Each view's batch has its own items and draw, each from a
        // multiple of 32: 21 from 0, then 400 from 32, 448 and 864.
        assert_eq!(
            draws
                .iter()
                .map(|d| (d.batch, d.view, d.row, d.first_item))
                .collect::<Vec<_>>(),
            [(0, 0, 0, 0), (1, 0, 1, 32), (1, 1, 2, 448), (1, 2, 3, 864)]
        );
        assert_eq!(items, 1264);
        assert_eq!(workgroups, 3 + 10 * 3);
        assert_eq!(rows[3].first_workgroup, 23);
    }

    /// Every batch's draw group window holds its items and lies inside the
    /// items buffer at its offset (the bounce of 427a61745: a window as wide
    /// as the buffer, at a nonzero offset).
    #[test]
    fn each_batch_s_window_holds_its_items_and_fits_the_buffer_at_its_offset() {
        let batches = [batch(0, 0, 3), batch(1, 3, 10), batch(0, 13, 1)];
        let slot = |m: usize| Some((m as u32, if m == 0 { 7 } else { 40 }));
        let views: [(&[u32], bool); 2] = [(&[0, 1, 2], true), (&[2, 0], false)];
        let (rows, _, items, _) = plan(&batches, views, slot, |_| true);
        let (window, buffer) = item_window(&rows, items);
        assert_eq!(window, 416 * 8, "batch 1's 400 items, aligned");
        for r in &rows {
            let offset = r.first_item as u64 * 8;
            assert_eq!(offset % 256, 0, "a storage offset's alignment");
            assert!(r.instances as u64 * r.most as u64 * 8 <= window);
            assert!(
                offset + window <= buffer,
                "row at {offset}: {window} past {buffer}"
            );
        }
        // The last batch (3 x 7 items from its first) still gets the
        // whole window.
        let last = rows.iter().map(|r| r.first_item).max().unwrap() as u64 * 8;
        assert_eq!(buffer, last + window);
        assert!(buffer > items as u64 * 8);
    }

    /// A secondary view's pixels per unit, from its view-projection
    /// alone, are the camera's from its projection: perspective and
    /// orthographic, seen directly or in a mirror (a reflected view).
    #[test]
    fn a_view_seen_through_its_matrix_picks_levels_as_the_camera_would() {
        let mirror = Mat4::from_scale(Vec3::new(1.0, -1.0, 1.0));
        for projection in [
            crate::Projection::Perspective {
                fov_y_degrees: 50.0,
            },
            crate::Projection::Orthographic { height: 12.0 },
        ] {
            let cam = crate::Camera {
                position: Vec3::new(3.0, 2.0, 9.0),
                target: Vec3::new(-1.0, 0.5, -4.0),
                projection,
                ..Default::default()
            };
            let frame = crate::frame::camera_frame(&cam, 16.0 / 9.0, glam::Vec2::ZERO);
            let camera = MeshletView::camera(&cam, &[], Vec::new(), 720);
            for view_proj in [frame.view_proj, frame.view_proj * mirror] {
                let seen = MeshletView::seen(&[], Vec::new(), cam.position, view_proj, 720);
                assert_eq!(seen.perspective, camera.perspective, "{projection:?}");
                assert!(
                    (seen.per_unit - camera.per_unit).abs() <= 1e-3 * camera.per_unit,
                    "{projection:?}: {} against {}",
                    seen.per_unit,
                    camera.per_unit
                );
            }
        }
        // A shadow view keeps the camera's eye and levels, drawn unlit.
        let cam = crate::Camera::default();
        let camera = MeshletView::camera(&cam, &[], vec![1], 720);
        let shadow = MeshletView::shadow(&camera, &[Vec4::X], vec![2]);
        assert_eq!(
            (shadow.eye, shadow.per_unit, shadow.lit),
            (camera.eye, camera.per_unit, false)
        );
        assert_eq!((shadow.planes, shadow.taken), (vec![Vec4::X], vec![2]));
    }

    #[test]
    fn pool_rows_offset_clusters_and_indices_into_the_pools() {
        let cl = |first, triangles| Cluster {
            first,
            triangles,
            center: Vec3::new(1.0, 2.0, 3.0),
            radius: 0.5,
            cone_axis: Vec3::Z,
            cone_cutoff: 0.25,
            error_center: Vec3::new(0.0, 1.0, 0.0),
            error_radius: 4.0,
            ..Default::default()
        };
        let c = MeshClusters {
            levels: vec![
                ClusterLevel {
                    first: 0,
                    count: 300,
                    error: 0.0,
                    clusters: 0..2,
                },
                ClusterLevel {
                    first: 300,
                    count: 30,
                    error: 0.2,
                    clusters: 2..3,
                },
            ],
            clusters: vec![cl(0, 64), cl(192, 36), cl(300, 10)],
            indices: Vec::new(),
        };
        let (mesh, rows) = pool_rows(&c, 1000, 50).unwrap();
        assert_eq!(mesh[0], [0.0f32, 1.0, 0.0, 4.0].map(f32::to_bits));
        assert_eq!(mesh[1][0], 2);
        assert_eq!(mesh[2], [50, 2, 0, 0]);
        assert_eq!(mesh[3], [52, 1, 0.2f32.to_bits(), 0]);
        assert_eq!(rows.len(), 9);
        // The third cluster: level 1, its first index past the pool's base.
        assert_eq!(rows[6], [1300, 10, 1, 0]);
        assert_eq!(rows[7], [1.0f32, 2.0, 3.0, 0.5].map(f32::to_bits));
        assert_eq!(rows[8], [0.0f32, 0.0, 1.0, 0.25].map(f32::to_bits));
        // Past eight levels: refused.
        let mut many = c.clone();
        many.levels = vec![c.levels[0].clone(); MAX_LEVELS + 1];
        assert!(pool_rows(&many, 0, 0).is_none());
    }
}
