//! GPU-driven culling: the frustum and occlusion tests,
//! the compaction of the visible instances and the draws' instance counts
//! run in compute, and every mesh pass submits indexed indirect draws. The
//! CPU path ([`crate::Renderer::cull`]) stays for small frames
//! ([`crate::Culling::Auto`]), [`crate::Culling::Cpu`] and for frames this
//! cannot take (buffers over the device limits).
//!
//! The skinned instances and baked crowds are culled beside the instances
//! ([`SkinnedCull`]): each one's posed world box (its joints' bind boxes
//! moved by its pose, a crowd's clip's over every frame) stands as an
//! instance after the sorted ones, tested against every pass's planes and
//! the camera's pyramid, and its record is compacted into the pass's
//! region of [`GpuCulling::skinned`] for the skinned draws. Secondary views
//! are passes of their own (their planes and layers).
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
//! A frame with no batch that can occlude, or with no earlier frame's
//! camera visibility in `scan` (the first frame, or after the buffer
//! grew), has no occluders to draw: it skips the occluders' pass, their
//! depth, the pyramid and the camera pass's occlusion test, and culls
//! every pass against its planes in one compute pass.
//!
//! The counts the CPU path knows at once (culled, occluded, triangles,
//! shadow casters), and the triangles of the instances the occluders hid
//! (counted per instance, so the camera pass's drawn and hidden triangles
//! can be checked against the CPU path's in view), come back through a small readback ring, for the frames
//! after [`crate::Renderer::request_culling_counts`] only: a frame's
//! [`crate::FrameStats`] carries those of the newest frame that has
//! landed (`counts_lag` frames old), and [`crate::Renderer::finish_stats`]
//! waits for the frame's own natively (copying them late for the last
//! frame encoded).

use crate::{Batch, DEPTH_FORMAT, Faces, GpuMaterial, GpuMesh, INSTANCE_STRIDE};
use glam::{Mat4, Vec4};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod counts;
mod encode;
mod pyramid;
mod setup;

pub(crate) use pyramid::*;

/// A GPU cull's rows as a pass draws them (the crowd cull's, read by the
/// pass that draws the crowds' cards): one row per culled view, the row's
/// compacted records and each of its draws' indirect arguments.
pub(crate) trait CulledRows {
    /// Row `row`'s records, a vertex buffer's slice.
    fn row_records(&self, row: u32) -> wgpu::BufferSlice<'_>;
    /// The buffer and byte offset row `row`'s draw `draw` reads its
    /// indirect arguments from.
    fn draw_args(&self, row: u32, draw: u32) -> (&wgpu::Buffer, u64);
}

/// The four stages of a cull that keeps what it keeps in order (the
/// instances' and the crowds'): each item's visibility and count, the
/// blocks' sums into their prefix, each item's prefix
/// (`shaders/cull_compact.wgsl`), then the module's own scatter of what it
/// keeps to where the prefix says.
pub(crate) struct CullStages {
    visibility: wgpu::ComputePipeline,
    scan_blocks: wgpu::ComputePipeline,
    prefix: wgpu::ComputePipeline,
    scatter: wgpu::ComputePipeline,
}

impl CullStages {
    /// The stages of `module` (its `visibility`, `scan_blocks`, `prefix`
    /// and `scatter` entry points) over `layout`.
    pub(crate) fn new(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        module: &wgpu::ShaderModule,
    ) -> CullStages {
        let stage = |entry: &str| compute_stage(device, layout, module, entry);
        CullStages {
            visibility: stage("visibility"),
            scan_blocks: stage("scan_blocks"),
            prefix: stage("prefix"),
            scatter: stage("scatter"),
        }
    }

    /// Runs the stages over `rows` rows of `blocks` workgroups of items
    /// each, with the bind group already set.
    pub(crate) fn run(&self, pass: &mut wgpu::ComputePass<'_>, blocks: u32, rows: u32) {
        pass.set_pipeline(&self.visibility);
        pass.dispatch_workgroups(blocks, rows, 1);
        pass.set_pipeline(&self.scan_blocks);
        pass.dispatch_workgroups(1, rows, 1);
        pass.set_pipeline(&self.prefix);
        pass.dispatch_workgroups(blocks, rows, 1);
        pass.set_pipeline(&self.scatter);
        pass.dispatch_workgroups(blocks, rows, 1);
    }
}

/// One compute stage of a cull's module.
pub(crate) fn compute_stage(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
    entry: &str,
) -> wgpu::ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(entry),
        layout: Some(layout),
        module,
        entry_point: Some(entry),
        compilation_options: Default::default(),
        cache: crate::pipeline_cache::of(device).as_ref(),
    })
}

pub(crate) const CULL_WGSL: &str = concat!(
    wgsl!("instance.wgsl"),
    wgsl!("cull.wgsl"),
    wgsl!("cull_scan.wgsl"),
    wgsl!("cull_compact.wgsl"),
    wgsl!("cull_plane.wgsl"),
    wgsl!("cull_box.wgsl"),
    wgsl!("cull_hiz.wgsl")
);
const HIZ_WGSL: &str = wgsl!("hiz.wgsl");
pub(crate) const DEPTH_WGSL: &str = concat!(wgsl!("instance.wgsl"), wgsl!("cull_depth.wgsl"));

/// Planes per pass (a cascade's box and the slice planes facing the light).
const MAX_PLANES: usize = 12;
const WORKGROUP: u32 = 256;
const READBACKS: usize = 3;

const TRANSPARENT: u32 = 1;
const OCCLUDER: u32 = 2;
const OCCLUDABLE: u32 = 4;
const FOLIAGE: u32 = 8;
/// A skinned batch ([`SkinnedCull`]): its instances are boxes, its records
/// the skinned instances'.
const SKINNED: u32 = 16;
/// Left out of the shadow passes (a skinned batch that casts no shadow).
const NO_CAST: u32 = 32;
// cull.wgsl's SKINNED_FLOATS: the floats of a skinned instance's record.
const _: () = assert!(crate::skin::SKINNED_INSTANCE_STRIDE == 28 * 4);

/// The skinned instances a frame culls beside its instances: each one's
/// posed world box (`skin::skinned_box`), tested as an instance whose
/// matrix takes the unit box to it, and its record, which each pass
/// compacts into its region of [`GpuCulling::skinned`] for the skinned
/// draws, batch by batch as the instances' (`skin.rs`).
pub(crate) struct SkinnedCull<'a> {
    /// The skinned batches, `first` and `count` over `records`.
    pub batches: &'a [Batch],
    pub records: &'a [crate::skin::SkinnedInstanceGpu],
    pub boxes: &'a [(glam::Vec3, glam::Vec3)],
}

impl SkinnedCull<'_> {
    /// The instances the batches cover (the records past them are the CPU
    /// lists' copies).
    pub fn len(&self) -> u32 {
        self.batches
            .iter()
            .map(|b| b.first + b.count)
            .max()
            .unwrap_or(0)
    }
}

/// The instance record whose matrix takes the unit box (centre 0, half
/// extent 1) to the box `lo..hi`: the culling's stand-in for a skinned
/// instance, so its frustum test is the box's and its occlusion test the
/// box's screen rectangle.
fn box_transform(lo: glam::Vec3, hi: glam::Vec3) -> crate::instances::GpuTransform {
    let (c, e) = ((lo + hi) * 0.5, (hi - lo).max(glam::Vec3::ZERO) * 0.5);
    crate::instances::gpu_transform(&(Mat4::from_translation(c) * Mat4::from_scale(e)))
}

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
    /// The three.js layer mask it draws: a batch whose mask shares no bit
    /// with it draws nothing and counts as neither inside nor culled.
    pub layers: u32,
    /// The batches its draws are counted for in the frame's stats (a
    /// shadow layer drawn over its static shadow cache draws only some of
    /// what it culls: `shadow_cache.rs`); `None`: every batch.
    pub counted: Option<Vec<u32>>,
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
    motion: [u32; 4],
    view_proj: [[f32; 4]; 4],
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
    extent: [f32; 3],
    /// The batch's three.js layer mask.
    layers: u32,
}

/// The GPU's counts for one frame (see the module docs). They are
/// counted after the frame's draws are encoded, so a frame's stats carry
/// an earlier frame's (`counts_lag`): with the instances moving between
/// frames, `culled` plus what was drawn need not add up to the frame's
/// instances.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuCounts {
    pub frame: u64,
    /// Opaque instances outside the camera's planes.
    pub culled: u32,
    /// Opaque instances inside them that the occluders hid.
    pub occluded: u32,
    /// Triangles of the camera pass's instance draws.
    pub triangles: u64,
    /// Triangles of the instances counted in `occluded`, counted on their
    /// own (not as the difference of two other counts).
    pub occluded_triangles: u64,
    /// Instance draws across the shadow passes.
    pub shadow_casters: u32,
}

/// The words after the draws and counters that the crowd cull
/// (`crowd_cull.wgsl` `counts`) adds its own counts into: the members'
/// records outside the camera's planes, inside them but occluded, the
/// occluded ones' triangles, the camera's drawn triangles and the shadow
/// passes' records; then the camera's triangles the meshlet path drew
/// (`meshlet_cull.wgsl`, copied by `MeshletGpu::encode`). Zeroed by
/// [`GpuCulling::encode`]; a frame without crowds or meshlets leaves them so.
pub(crate) const CROWD_COUNT_WORDS: u64 = 8;

/// The word after the crowds' counts that holds the meshlet path's triangles.
const MESHLET_TRIANGLES: usize = 5;

/// What a readback's words mean.
#[derive(Clone)]
struct CountLayout {
    frame: u64,
    batches: u32,
    kinds: Vec<PassKind>,
    /// Per pass, the batches it counts ([`CullPass::counted`]).
    counted: Vec<Option<Vec<u32>>>,
    /// Triangles per instance, per batch.
    triangles: Vec<u64>,
    opaque: u32,
}

impl CountLayout {
    /// The words of the draws buffer the counts are read from: per (pass,
    /// batch) a draw, per pass its inside count, per batch its hidden count,
    /// then the crowds' counts ([`CROWD_COUNT_WORDS`]).
    fn words(passes: usize, batches: u32) -> u64 {
        Self::crowd_word(passes, batches) + CROWD_COUNT_WORDS
    }

    /// The first of the crowds' count words.
    fn crowd_word(passes: usize, batches: u32) -> u64 {
        passes as u64 * (batches as u64 * 5 + 1) + batches as u64
    }

    fn counts(&self, words: &[u32]) -> GpuCounts {
        let b = self.batches as usize;
        let p = self.kinds.len();
        let draws = |g: usize| (0..b).map(move |i| (i, words[(g * b + i) * 5 + 1]));
        let inside = |g: usize| words[p * b * 5 + g];
        let hidden = |i: usize| words[p * b * 5 + p + i];
        let mut c = GpuCounts {
            frame: self.frame,
            ..GpuCounts::default()
        };
        for (g, kind) in self.kinds.iter().enumerate() {
            match kind {
                PassKind::Shadow => {
                    c.shadow_casters += match &self.counted[g] {
                        Some(only) => only
                            .iter()
                            .map(|&i| words[(g * b + i as usize) * 5 + 1])
                            .sum(),
                        None => draws(g).map(|(_, n)| n).sum::<u32>(),
                    };
                }
                PassKind::Main => {
                    let drawn: u32 = draws(g).map(|(_, n)| n).sum();
                    c.triangles = draws(g).map(|(i, n)| self.triangles[i] * n as u64).sum();
                    c.culled = self.opaque.saturating_sub(inside(g));
                    c.occluded = inside(g).saturating_sub(drawn);
                    c.occluded_triangles =
                        (0..b).map(|i| self.triangles[i] * hidden(i) as u64).sum();
                }
                PassKind::Reflection | PassKind::Occluders => {}
            }
        }
        let crowd = &words[Self::crowd_word(p, self.batches) as usize..];
        c.culled += crowd[0];
        c.occluded += crowd[1];
        c.occluded_triangles += crowd[2] as u64;
        c.triangles += crowd[3] as u64 + crowd[MESHLET_TRIANGLES] as u64;
        c.shadow_casters += crowd[4];
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
    stages: CullStages,
    hiz_base_layout: [wgpu::BindGroupLayout; 2],
    hiz_reduce_layout: wgpu::BindGroupLayout,
    hiz_base: [Option<wgpu::ComputePipeline>; 2],
    hiz_reduce: wgpu::ComputePipeline,
    depth_module: wgpu::ShaderModule,
    depth_pipeline_layout: wgpu::PipelineLayout,
    depth_pipelines: HashMap<(Faces, bool, u32), wgpu::RenderPipeline>,
    view: wgpu::Buffer,
    view_group: wgpu::BindGroup,
    params: [wgpu::Buffer; 2],
    /// Every pass's planes and kind (`PassGpu`), the stage of the other
    /// passes first and the camera's after: sized for the frame's passes,
    /// so a frame culls as many as it has (every cascade and shadowed
    /// light face) on any device.
    passes: wgpu::Buffer,
    pub instances: wgpu::Buffer,
    /// The skinned instances' records ([`SkinnedCull`]): region 0 the
    /// frame's, region g + 1 pass g's visible ones, each region as long as
    /// the skinned instances.
    pub skinned: wgpu::Buffer,
    /// The skinned instances this frame culls, and where their boxes start
    /// among the instances (after the sorted ones).
    pub skinned_n: u32,
    pub skinned_base: u32,
    /// What the input region of `instances` holds, written by changes only.
    upload: crate::upload::DirtyUpload,
    /// The previous transforms, laid out as `instances` (region 0 at the
    /// sorted slots, the camera pass's visible ones in its region): TAA's
    /// object motion for GPU-culled batches (the prepass and the meshlet
    /// path read them). A placeholder while no frame had object motion.
    pub previous: wgpu::Buffer,
    previous_upload: crate::upload::DirtyUpload,
    /// Bytes of transforms the last `encode` wrote.
    pub uploaded: u64,
    batches: wgpu::Buffer,
    scan: wgpu::Buffer,
    pub args: wgpu::Buffer,
    groups: Option<[wgpu::BindGroup; 2]>,
    pyramid: Option<Pyramid>,
    /// Bound in the pyramid's place until a frame needs one.
    no_pyramid: wgpu::TextureView,
    /// Whether `scan` holds a frame's camera visibility (the occluders a
    /// frame may draw): false until a frame culled its camera pass, and
    /// again when the buffer is replaced.
    primed: bool,
    /// Whether the last `encode` built the pyramid (it had occluders).
    occlusion: bool,
    readbacks: Vec<Readback>,
    latest: Option<GpuCounts>,
    /// The last encoded frame's layout and given counts (`finish` copies
    /// its counts late when it found no readback free).
    last: Option<(CountLayout, GpuCounts)>,
    /// Whether the last frame encoded asked for its counts
    /// ([`Self::copy_counts`] takes it).
    copy: bool,
    max_binding: u64,
    /// This frame's layout: instances, batches, passes.
    pub n: u32,
    pub b: u32,
    /// The first skinned batch in the batch table (after the instances').
    skinned_base_batch: u32,
    /// Each (pass, batch) draw's args copied after the cull, which draws
    /// read instead of `args` on WebKit (`indirect_own.rs`); `None`
    /// elsewhere.
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

impl GpuCulling {
    /// Whether `n` instances and `skinned` skinned ones in `passes` passes
    /// (the most the frame can have) fit the device's storage binding
    /// limit.
    pub fn fits(&self, n: u32, skinned: u32, passes: usize) -> bool {
        let all = n as u64 + skinned as u64;
        all <= u32::MAX as u64
            && (passes as u64 + 1) * all * INSTANCE_STRIDE <= self.max_binding
            && (passes as u64 + 1) * skinned as u64 * crate::skin::SKINNED_INSTANCE_STRIDE
                <= self.max_binding
            && Self::scan_words(all as u32, passes) * 4 <= self.max_binding
    }

    /// Where pass `g`'s records of the skinned batch starting at `first`
    /// (over [`SkinnedCull::records`]) begin in [`Self::skinned`].
    pub fn skinned_region(&self, g: u32, first: u32) -> u32 {
        (g + 1) * self.skinned_n + first
    }

    fn scan_words(n: u32, passes: usize) -> u64 {
        let blocks = n.div_ceil(WORKGROUP) as u64;
        n as u64 + passes as u64 * (n as u64 + 1) + passes as u64 * blocks
    }

    /// The byte offset of pass `g`'s draw of batch `b` in `args`.
    pub fn draw_offset(&self, g: u32, b: u32) -> u64 {
        (g as u64 * self.b as u64 + b as u64) * 20
    }

    /// The buffer and byte offset pass `g`'s draw of batch `b` reads its
    /// indirect args from: `args` at [`Self::draw_offset`], or the draw's
    /// own copy ([`Self::per_draw`]).
    pub fn draw_args(&self, g: u32, b: u32) -> (&wgpu::Buffer, u64) {
        match &self.per_draw {
            Some(own) => (own.get((g * self.b + b) as usize), 0),
            None => (&self.args, self.draw_offset(g, b)),
        }
    }

    /// [`Self::draw_args`] of pass `g`'s draw of skinned batch `k`.
    pub fn skinned_draw_args(&self, g: u32, k: u32) -> (&wgpu::Buffer, u64) {
        self.draw_args(g, self.skinned_base_batch + k)
    }

    /// Copies passes `groups`' draw args to their own buffers
    /// ([`Self::per_draw`]; nothing without them), after the cull that
    /// wrote them. `passes`: the frame's pass count. (Fields, not `self`:
    /// `encode` holds the pyramid meanwhile.)
    fn copy_per_draw(
        (per_draw, args, b): (
            &mut Option<crate::indirect_own::OwnArgs>,
            &wgpu::Buffer,
            u32,
        ),
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        passes: u32,
        groups: std::ops::Range<u32>,
    ) {
        let Some(own) = per_draw else {
            return;
        };
        own.ensure(device, (passes * b) as usize);
        for g in groups {
            for k in 0..b {
                let d = g * b + k;
                own.copy(encoder, args, d as u64 * 20, d as usize, 20);
            }
        }
    }

    /// Where pass `g`'s instances of the batch starting at `first` begin.
    pub fn region(&self, g: u32, first: u32) -> u32 {
        (g + 1) * self.n + first
    }
}
