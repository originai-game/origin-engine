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
//! (when that frame has one), and marks the visible cluster slots.
//! `compact` builds both work lists in row/instance/cluster order with a
//! blocked prefix scan. The prepass and the lit pass then draw each batch
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
pub(crate) const MESHLET_RASTER_WGSL: &str = wgsl!("meshlet_raster.wgsl");
pub(crate) const MESHLET_SWAY_WGSL: &str = wgsl!("meshlet_sway.wgsl");

/// The foliage's deformation every pull reads (the vegetation's
/// grass_common.wgsl and meshlet_foliage.wgsl, then meshlet_sway.wgsl).
fn sway_source() -> String {
    format!(
        "{}{}{}{MESHLET_SWAY_WGSL}",
        crate::foliage::WIND_WGSL,
        crate::foliage::WIND_LEAN_WGSL,
        crate::foliage::WGSL
    )
}

/// The visibility buffer's rasteriser and setup (after instance.wgsl).
pub(crate) fn raster_source() -> String {
    format!(
        "{}{}{}{MESHLET_RASTER_WGSL}",
        crate::LIBRARY_WGSL,
        crate::INSTANCE_WGSL,
        sway_source()
    )
}

/// The lit and prepass module of the meshlet path: the standard lit module
/// (the built-in hooks; with `planar` its planar reflection slot) and the
/// pulling entry points. Its own module, never the standard one: a material
/// shader is checked against that one (og-cook `validate_shader`), and the
/// pulling entry points read `instance_index`.
pub(crate) fn lit_source(planar: bool) -> String {
    format!(
        "{}{}{MESHLET_WGSL}",
        crate::lit_source_with(&crate::hooks::default_hooks(), planar),
        sway_source()
    )
}

/// The early phase's depth module.
pub(crate) fn depth_source() -> String {
    format!(
        "{}{}{}{MESHLET_DEPTH_WGSL}",
        crate::LIBRARY_WGSL,
        crate::INSTANCE_WGSL,
        sway_source()
    )
}

/// The params buffer's second slot (the late phase's), a uniform offset.
const PARAMS_SLOT: u64 = 256;

/// The shadow casters' module: the standard shadow module (the built-in
/// hooks) and the pulling entry points.
pub(crate) fn shadow_source() -> String {
    format!(
        "{}{}{MESHLET_SHADOW_WGSL}",
        crate::shadow_source(&crate::hooks::default_hooks()),
        sway_source()
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
    soft: [u32; 4],
}

/// `struct RasterParams` in meshlet_raster.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RasterParams {
    view_proj: [[f32; 4]; 4],
    size: [u32; 4],
    soft: [u32; 4],
    stylise: [f32; 4],
    /// This frame's wind and the last drawn frame's (`foliage::foliage_winds`).
    wind: [crate::foliage::FoliageWind; 2],
}

/// `struct MeshletDraw` in meshlet_sway.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DrawParams {
    flags: [u32; 4],
    wind: [crate::foliage::FoliageWind; 2],
}

/// vec4 rows per visibility-buffer record (meshlet_raster.wgsl `setup`).
const RECORD_ROWS: u64 = 8;
/// With the frame's material surface, affine texturing or a foliage mesh:
/// four more, the local position's gradients, the affine uv, the lit uv's
/// gradients and the canopy dome.
const RECORD_ROWS_EXTRA: u64 = 12;

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
    /// Its small clusters go to the visibility buffer and are drawn by
    /// its resolve (the camera's row of an occluder batch).
    pub soft: bool,
}

/// [`BatchRow::flags`]: the normal cone may cull.
const FLAG_CONE: u32 = 1;
/// [`BatchRow::flags`]: its small clusters go to the visibility buffer
/// (the camera's row of an occluder batch, `meshlet_cull.wgsl`).
const FLAG_SOFT: u32 = 2;
/// [`BatchRow::flags`]: the side its material draws, from bit 2 (0 the
/// front, 1 the back, 2 both): the compute rasteriser's face culling.
const FLAG_SIDE_SHIFT: u32 = 2;
/// The most clusters one frame rasterises in compute; past them the
/// cull draws the rest by the hardware.
pub(crate) const SOFT_CAP: u32 = 1 << 16;

/// One mesh's rows in the mesh table.
type MeshRows = [[u32; 4]; MESH_ROWS];

/// The mesh table's rows and the cluster pool's for one mesh, its
/// clusters' first indices offset by `index_base`, its levels' first
/// clusters by `cluster_base`, its vertices at `vertex_base` of the vertex
/// pool; `None` past [`MAX_LEVELS`] levels.
pub(crate) fn pool_rows(
    c: &MeshClusters,
    index_base: u32,
    cluster_base: u32,
    vertex_base: u32,
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
    mesh[1] = [c.levels.len() as u32, vertex_base, 0, 0];
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
    /// And each one's foliage.
    foliage_key: Vec<Option<Arc<crate::meshlet::MeshFoliage>>>,
    /// Per mesh index: its slot in the mesh table and its most clusters at
    /// any level.
    slots: HashMap<usize, (u32, u32)>,
    meshes: wgpu::Buffer,
    clusters: wgpu::Buffer,
    indices: wgpu::Buffer,
    /// Every clustered mesh's f32 vertices, for the visibility buffer
    /// (its rasteriser reads any mesh's in one dispatch), then each
    /// foliage mesh's region ([`pool_foliage`]), which every pull reads.
    vertices: wgpu::Buffer,
    /// Each foliage mesh's slot and the bounds of its deformation, and the
    /// margin last written into its mesh row.
    foliage: Vec<(u32, crate::foliage::FoliageBounds, f32)>,
}

/// Points the cluster rows of a foliage mesh (`rows`, [`pool_rows`]'s) at
/// its region of the vertex pool, `region` vec4s from the pool's start:
/// each cluster's first row's w is the region plus one (0: no foliage).
/// The region is [`crate::meshlet::MeshFoliage::pool_rows`].
pub(crate) fn pool_foliage(rows: &mut [[u32; 4]], region: u32) {
    for r in rows.iter_mut().step_by(CLUSTER_ROWS) {
        r[3] = region + 1;
    }
}

pub(crate) mod compact;
mod vis;

mod draw;
mod encode;
mod pipelines;
mod plan;
mod setup;

use draw::*;
pub(crate) use plan::*;

pub(crate) struct MeshletGpu {
    /// The visibility buffer: the camera's small clusters rasterised in
    /// compute and resolved per batch ([`vis`]).
    vis_buffer: vis::VisBuffer,
    /// The wind field's lean the foliage sways by (`wind_field.rs`).
    wind_field: wgpu::TextureView,
    cull_layout: wgpu::BindGroupLayout,
    cull: wgpu::ComputePipeline,
    compact: compact::Compact,
    pub(crate) draw_layout: wgpu::BindGroupLayout,
    lit: HashMap<(Side, bool), wgpu::RenderPipeline>,
    prepass: HashMap<Side, wgpu::RenderPipeline>,
    /// The lit module ([`lit_source`]) and its planar slot.
    lit_module: Option<(bool, wgpu::ShaderModule)>,
    /// The shadow casters' pipelines by (side, the whole vertex for an
    /// alpha-tested or dissolving material), and their module.
    shadow: HashMap<(Side, bool), crate::ShadowSet>,
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

impl MeshletGpu {}

#[cfg(test)]
mod tests;
