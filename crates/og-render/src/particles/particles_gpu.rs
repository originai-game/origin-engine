//! The renderer's side of particles: pooled GPU slab storage, batched compute
//! simulation with GPU sub-emitters, alive compaction with indirect draws,
//! the CPU path's uploads, and the draw.

use crate::deferred::Deferred;
use crate::particles::{
    CpuParticles, EmitterShape, MAX_CURVE_KEYS, ParticleAlign, ParticleBlend, ParticleEmitter,
    ParticleRender, ParticleState, ParticleSystem, SheetMode, SimulationSpace, TRAIL_POINTS,
    TickStep, appearance, array_frames, hash_seed, plan_ticks, sheet_frames,
};
use crate::scene::{RenderScene, Sprite};
use crate::sprites::ExtraSprite;
use crate::{HDR_FORMAT, RenderError};
use glam::{Mat4, Vec2, Vec3};
use std::collections::HashMap;
use std::sync::Mutex;

mod emitter;
mod pool;
mod prepare;
mod renderer;
pub(crate) use emitter::ParticleViewGpu;
use emitter::*;
use pool::*;

const COMMON_WGSL: &str = wgsl!("particle_common.wgsl");
const DRAW_COMMON_WGSL: &str = wgsl!("particle_draw_common.wgsl");
const SIM_WGSL: &str = wgsl!("particles_sim.wgsl");
const COMPACT_WGSL: &str = wgsl!("particles_compact.wgsl");
const COLLIDERS_WGSL: &str = wgsl!("scene_colliders.wgsl");
const DRAW_WGSL: &str = wgsl!("particles.wgsl");
const SPRITES_WGSL: &str = wgsl!("particle_sprites.wgsl");

/// Bytes between draw-only emitter uniform entries (a multiple of the 256-byte
/// dynamic offset alignment).
const EMITTER_DRAW_STRIDE: u64 = 768;
const _: () = assert!(std::mem::size_of::<EmitterDrawGpu>() as u64 <= EMITTER_DRAW_STRIDE);
const PARTICLE_STRIDE: u64 = std::mem::size_of::<ParticleState>() as u64;

pub(crate) const PARTICLE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: PARTICLE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4],
};

/// How a system's particles are drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum DrawKind {
    Billboard,
    Mesh,
    /// Its particles' trails (`vs_trail`, their ring the second vertex
    /// buffer).
    Trail,
    /// The scene behind, shifted (`fs_distort`).
    Distort,
    /// Sprite mode simulated in compute: the 2D batch draws it
    /// ([`ParticleFrame::sprite_systems`]), not this pass.
    Sprite,
}

fn draw_kind(e: &ParticleEmitter) -> DrawKind {
    match e.render {
        ParticleRender::Mesh { .. } => DrawKind::Mesh,
        _ if e.distortion > 0.0 => DrawKind::Distort,
        _ => DrawKind::Billboard,
    }
}

/// A particle's trail beside it (`TrailIn` in particles.wgsl): its
/// [`TRAIL_POINTS`] samples, one instance per particle.
pub(crate) const TRAIL_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: (TRAIL_POINTS * 16) as u64,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![
        3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4,
        7 => Float32x4, 8 => Float32x4, 9 => Float32x4, 10 => Float32x4
    ],
};

/// A mesh's vertices beside the particles (`MeshIn` in particles.wgsl):
/// [`crate::Vertex`]'s position, normal and colour.
pub(crate) const MESH_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<crate::Vertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 0,
            shader_location: 5,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 12,
            shader_location: 6,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 32,
            shader_location: 7,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: 24,
            shader_location: 8,
        },
    ],
};

struct PointsTable {
    shape: EmitterShape,
    field: Option<crate::particles::VectorField>,
    table: Vec<[f32; 4]>,
    shape_len: usize,
    /// Offset in vec4 elements inside `ParticlePool::points`.
    pool_offset: u32,
}

/// One GPU-simulated sub-emitter child's slab allocation in the Mega-Pool.
struct GpuChild {
    emitter: usize,
    max: u32,
    particle_base: u32,
    counter_idx: u32,
}

struct GpuSystem {
    /// Slot offset in `ParticlePool::particles`.
    particle_base: u32,
    /// Particle-slot offset in `ParticlePool::trails` (unused when `trail_max == 0`).
    trail_base: u32,
    /// Number of particle trail slots allocated (`max` with trail, else `0`).
    trail_max: u32,
    /// The system's time at its last tick, for its trail's newest sample.
    time: f32,
    last_tick: Option<u64>,
    prev_transform: Option<Mat4>,
    clock: crate::particles::ParticleClock,
    children: Vec<GpuChild>,
    /// True once a prior frame finished with zero live parent particles so
    /// subsequent dormant frames skip GPU simulation waves and origin shifts.
    was_empty: bool,
}

struct SystemState {
    emitter: usize,
    max: u32,
    gpu: Option<GpuSystem>,
    cpu: CpuParticles,
    seen: u64,
    /// The render origin's moves since the system last simulated
    /// ([`ParticleRenderer::shift`]): added to its world-space particles.
    shift: Vec3,
    /// Where a world-space system's emitter stood in its particles' longest
    /// life ([`swept_bound`]).
    path: std::collections::VecDeque<(f64, Vec3)>,
}

/// Byte offsets of a system's raw slab in the Mega-Pool (for direct draws).
#[derive(Clone, Copy, Debug)]
struct PoolSlice {
    particles_offset: u64,
    trails_offset: u64,
}

/// Compacted vertex slices and indirect draw slot for a GPU-compacted system.
#[derive(Clone, Copy, Debug)]
struct CompactDraw {
    particles_offset: u64,
    trails_offset: u64,
    indirect_offset: u64,
    /// Expected mesh index count (0 for non-indexed draws).
    index_count: u32,
}

/// Where a [`FrameSystem`]'s vertex data comes from when drawing.
#[derive(Clone, Copy, Debug)]
enum VertexSource {
    /// Compacted dense alive particles + indirect draw arguments.
    Compact(CompactDraw),
    /// Direct draw from the persistent Mega-Pool (`particles` / `trails`).
    Pool(PoolSlice),
    /// CPU-path packed vertices in `ParticleRenderer::cpu_vertices`.
    Cpu(Packed),
}

/// One system's work this frame.
pub(crate) struct FrameSystem {
    id: u64,
    /// Draw entry index in `emitter_draw_buffer`.
    draw_entry: u32,
    pub blend: ParticleBlend,
    pub texture: Option<crate::TextureHandle>,
    /// View depth of the emitter (draw order: far first).
    depth: f32,
    /// Its three.js layer mask ([`RenderScene::particle_layers`]).
    pub layers: u32,
    /// A sub-emitter's particles: which of the system's.
    child: Option<usize>,
    /// The slots drawn, as instance ranges in slot order ([`drawn`]).
    slots: [std::ops::Range<u32>; 2],
    source: VertexSource,
    /// A box its particles stay inside ([`swept_bound`]): a view it does
    /// not touch skips it. None: drawn in every view.
    bound: Option<(Vec3, Vec3)>,
    kind: DrawKind,
    mesh: Option<crate::MeshHandle>,
    /// Its effect material (`effect.rs`) and how its pipeline differs.
    effect: Option<crate::EffectMaterialHandle>,
    look: Look,
    /// Routed to the half-resolution soft-particle pass on the main camera.
    half_res: bool,
}

/// Where each of a CPU-path system's [`FrameSystem::slots`] ranges starts
/// in the frame's vertex buffer (bytes): its particles', and with a trail
/// their trails'. A range drawn from there is instances `0..len`.
#[derive(Clone, Copy, Debug, Default)]
struct Packed {
    particles: [u64; 2],
    trails: [u64; 2],
}

/// The instance ranges a system of `max` slots draws: every slot, or with
/// `live` (its [`crate::particles::live_spawns`]) only those its living
/// particles can be in. The ring wraps, so that is at most two ranges,
/// drawn in slot order as the whole ring is (an alpha-blended system's
/// particles overlap in the same order).
fn drawn(max: u32, live: Option<std::ops::Range<u64>>) -> [std::ops::Range<u32>; 2] {
    let m = u64::from(max);
    match live {
        Some(r) if r.end - r.start < m => {
            let (a, b) = ((r.start % m) as u32, (r.end % m) as u32);
            if a <= b { [a..b, 0..0] } else { [0..b, a..max] }
        }
        _ => [0..max, 0..0],
    }
}

/// What an effect material changes in a system's pipeline: its hook's
/// module (by shader index), and for a mesh drawn with one, no depth
/// written and both faces where it is double sided.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Look {
    hook: Option<usize>,
    effect_mesh: bool,
    double_sided: bool,
}

/// Every draw the scene's loaded emitters (each with its trail) can need.
/// Effect meshes are warmed separately via [`EffectMeshRenderer`].
pub(crate) fn reachable_draws(scene: &crate::RenderScene) -> Vec<(ParticleBlend, DrawKind, Look)> {
    let mut out = Vec::new();
    let mut push = |k: (ParticleBlend, DrawKind, Look)| {
        if k.2.hook.is_none() && !out.contains(&k) {
            out.push(k);
        }
    };
    for e in scene.emitters() {
        if matches!(e.render, ParticleRender::Sprite { .. }) {
            continue;
        }
        let kind = draw_kind(e);
        let material = e.effect.and_then(|h| scene.effect_material(h));
        let drawn = |kind| match kind {
            DrawKind::Billboard | DrawKind::Mesh | DrawKind::Trail => material,
            _ => None,
        };
        let blend = drawn(kind)
            .and_then(|m| m.blend)
            .unwrap_or_else(|| e.blend());
        let hook = |kind| drawn(kind).and_then(|m| m.shader).map(|h| h.index());
        if e.trail.is_none_or(|t| t.head) {
            push((blend, kind, look_of(kind, drawn(kind), hook(kind))));
        }
        if e.trail.is_some() {
            let t = DrawKind::Trail;
            push((blend, t, look_of(t, drawn(t), hook(t))));
        }
    }
    out
}

/// The [`Look`] of a system of kind `kind` drawn with `material`.
fn look_of(kind: DrawKind, material: Option<&crate::EffectMaterial>, hook: Option<usize>) -> Look {
    match material {
        Some(m) => Look {
            hook,
            effect_mesh: kind == DrawKind::Mesh,
            double_sided: kind == DrawKind::Mesh && m.double_sided,
        },
        None => Look::default(),
    }
}

/// A sprite-mode system simulated in compute, as the 2D batch draws it:
/// what `particle_sprites.wgsl` needs to write its instances.
pub(crate) struct SpriteSystem {
    /// Its draw entry among the frame's draw emitter uniforms.
    pub draw_entry: u32,
    /// Slot offset in `ParticlePool::particles`.
    pub particle_base: u32,
    pub layer: i32,
    pub layers: u32,
    pub blend: crate::SpriteBlend,
    pub glow: f32,
    pub soft: bool,
    pub texture: Option<crate::TextureHandle>,
    pub array: Option<crate::TextureArrayHandle>,
    pub frames: u32,
    pub columns: u32,
    pub rows: u32,
    /// Its ring's slots: the instances of its range.
    pub max: u32,
    pub z: f32,
}

/// `SpriteParams` in particle_sprites.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteParamsGpu {
    first: u32,
    max: u32,
    frames: u32,
    columns: u32,
    rows: u32,
    array: u32,
    soft: u32,
    particle_base: u32,
    z: f32,
    glow: f32,
    lit: f32,
    _pad1: f32,
}

/// What the frame's particles and instanced effect meshes need from the renderer.
#[derive(Default)]
pub(crate) struct ParticleFrame {
    pub systems: Vec<FrameSystem>,
    /// Standalone instanced effect meshes prepared for this frame.
    pub effect_meshes: crate::particles::effect_mesh::EffectMeshFrame,
    /// Sprite-mode particles simulated on the CPU (a host's, or with
    /// sub-emitters), as the 2D batch draws them.
    pub sprites: Vec<ExtraSprite>,
    /// Sprite-mode systems simulated in compute: the 2D batch draws each
    /// from instances [`ParticleRenderer::expand_sprites`] writes.
    pub sprite_systems: Vec<SpriteSystem>,
    pub any_gpu: bool,
    pub needs_depth: bool,
    /// A system fades against the depth behind it (soft particles): each
    /// view draws its own depth for them (`views.rs`).
    pub soft: bool,
    pub capacity: u32,
    pub cpu_alive: u32,
    pub ticks: u32,
    pub collision_skipped: u32,
    /// A system bends the scene behind it: the main pass copies the opaque
    /// scene for it (`DrawKind::Distort`).
    pub distortion: bool,
    /// The meshes particles are drawn with (their f32 vertices are needed).
    pub meshes: Vec<usize>,
    /// Planned compute waves and compaction tile count for `simulate`.
    gpu_plan: FrameGpuPlan,
}

pub(crate) struct ParticleRenderer {
    /// Sprite-mode systems' instances for the 2D batch (`expand_sprites`).
    sprites_layout: wgpu::BindGroupLayout,
    sprites_pipeline: Deferred<wgpu::ComputePipeline>,
    sprites_params: wgpu::Buffer,
    sprites_params_gen: u64,
    /// Batched compute simulation: group 0 (frame) and group 1 (pool).
    sim_frame_layout: wgpu::BindGroupLayout,
    sim_pool_layout: wgpu::BindGroupLayout,
    /// simulate, clear, shift.
    sim: Deferred<(
        wgpu::ComputePipeline,
        wgpu::ComputePipeline,
        wgpu::ComputePipeline,
    )>,
    /// Alive-particle compaction + indirect draw argument generation.
    compact_layout: wgpu::BindGroupLayout,
    compact_pipeline: Deferred<wgpu::ComputePipeline>,
    pub draw_layout: wgpu::BindGroupLayout,
    pub(crate) draw: HashMap<
        (ParticleBlend, u32, DrawKind, Look),
        crate::deferred::Deferred<wgpu::RenderPipeline>,
    >,
    reachable: (usize, usize, Vec<(ParticleBlend, DrawKind, Look)>),
    pub(crate) draw_module: crate::deferred::Module,
    /// The draw module with an effect material's hook, by shader index.
    hook_modules: HashMap<usize, wgpu::ShaderModule>,
    frame_wgsl: String,
    draw_pl: wgpu::PipelineLayout,
    /// Half-resolution soft-particle draw pipelines (Additive and Alpha, sample_count = 1).
    half_draw: HashMap<ParticleBlend, crate::deferred::Deferred<wgpu::RenderPipeline>>,
    /// Half-res depth downsample and depth-aware bilateral upsample composite.
    pub(crate) half_res: crate::particles::half_res::HalfResFx,
    /// Standalone instanced effect mesh renderer sharing `draw_pl` and `effects`.
    pub(crate) effect_meshes: crate::particles::effect_mesh::EffectMeshRenderer,
    /// The effect materials (group 3 of every particle, effect mesh and strip draw).
    pub(crate) effects: crate::effect::EffectRenderer,
    /// Draw-only emitter dynamic uniform buffer (`EMITTER_DRAW_STRIDE`).
    emitter_draw_buffer: wgpu::Buffer,
    emitter_draw_gen: u64,
    emitter_draw_bytes: Vec<u8>,
    draw_entries: Vec<EmitterDrawGpu>,
    /// Global Mega-Pool for GPU simulation, trails, points, compaction & indirect args.
    pool: ParticlePool,
    /// Cached draw bind groups keyed by `(view_buffer, depth_view, scene_color, draw_gen)`.
    draw_groups: Mutex<DrawGroupCache>,
    /// Every CPU-path system's drawn slots and trails this frame, one
    /// after another ([`Packed`]): one write a frame, not one a system.
    cpu_vertices: wgpu::Buffer,
    cpu_bytes: Vec<u8>,
    pub view_buffer: wgpu::Buffer,
    /// The scene's colliders as `scene_colliders.rs` packs them.
    colliders: wgpu::Buffer,
    colliders_gen: u64,
    /// The og.ForceFields as `pack_fields` lays them out.
    fields: wgpu::Buffer,
    fields_gen: u64,
    systems: HashMap<u64, SystemState>,
    scene_id: u64,
    points: HashMap<usize, PointsTable>,
    frame: u64,
}

fn uniform_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
    dynamic: bool,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: dynamic,
            min_binding_size: None,
        },
        count: None,
    }
}

fn depth_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
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

/// Which of a camera's 3D systems and effect meshes a draw takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ParticlePart {
    /// Every system and effect mesh, in one place (views and probes).
    All,
    /// Full-resolution main-pass particles without OIT.
    Main { skip_half: bool, skip_distort: bool },
    /// Solid opaque mesh particles (debris, depth-writing).
    Before,
    /// Alpha billboards, trails, effect-material mesh particles and instanced
    /// alpha effect meshes into the OIT pass.
    Oit,
    /// Large soft-depth billboards rendered into the half-resolution target (`fx.half`).
    Half,
    /// Non-solid, non-OIT draws after OIT resolve.
    After { skip_half: bool, skip_distort: bool },
    /// Distortion particles drawn after the post-transparent `scene-copy`.
    Distort,
}

impl ParticlePart {
    fn draws(self, s: &FrameSystem) -> bool {
        match self {
            ParticlePart::All => true,
            ParticlePart::Main {
                skip_half,
                skip_distort,
            } => !(skip_half && s.half_res) && !(skip_distort && s.kind == DrawKind::Distort),
            ParticlePart::Before => solid_mesh(s),
            ParticlePart::Oit => in_oit(s),
            ParticlePart::Half => s.half_res,
            ParticlePart::After {
                skip_half,
                skip_distort,
            } => {
                !solid_mesh(s)
                    && !in_oit(s)
                    && !(skip_half && s.half_res)
                    && !(skip_distort && s.kind == DrawKind::Distort)
            }
            ParticlePart::Distort => s.kind == DrawKind::Distort,
        }
    }
}

/// Debris: opaque alpha-blended mesh particles that write depth (additive,
/// subtractive, multiply and effect-material mesh particles never write depth).
fn solid_mesh(s: &FrameSystem) -> bool {
    s.kind == DrawKind::Mesh && !s.look.effect_mesh && s.blend == ParticleBlend::Alpha
}

/// OIT pipeline source for a particle system that joins the OIT pass.
pub(crate) fn oit_source_of(s: &FrameSystem) -> Option<crate::oit::Source> {
    if s.blend != ParticleBlend::Alpha {
        return None;
    }
    let kind = match s.kind {
        DrawKind::Billboard => crate::oit::ParticleOit::Billboard,
        DrawKind::Trail => crate::oit::ParticleOit::Trail,
        DrawKind::Mesh if s.look.effect_mesh => crate::oit::ParticleOit::Mesh,
        _ => return None,
    };
    let double_sided = kind == crate::oit::ParticleOit::Mesh && s.look.double_sided;
    Some(crate::oit::Source::Particles(
        kind,
        double_sided,
        s.look.hook,
    ))
}

/// Whether a system's particles join the order-independent passes.
fn in_oit(s: &FrameSystem) -> bool {
    oit_source_of(s).is_some()
}

impl ParticleFrame {
    /// Whether this frame has any 3D particle systems or instanced effect meshes.
    pub fn has_draws(&self) -> bool {
        !self.systems.is_empty() || !self.effect_meshes.batches.is_empty()
    }

    /// Whether any 3D system on `mask` is routed to the half-resolution soft pass.
    pub fn has_half_res(&self, mask: u32) -> bool {
        self.systems
            .iter()
            .any(|s| s.layers & mask != 0 && s.half_res)
    }

    /// OIT pipeline sources needed by particles and effect meshes on `mask`.
    pub fn oit_sources(&self, mask: u32) -> Vec<crate::oit::Source> {
        let mut out = Vec::new();
        for s in &self.systems {
            if s.layers & mask != 0
                && let Some(src) = oit_source_of(s)
                && !out.contains(&src)
            {
                out.push(src);
            }
        }
        for b in &self.effect_meshes.batches {
            if b.layers & mask != 0 && b.in_oit() {
                let src = b.oit_source();
                if !out.contains(&src) {
                    out.push(src);
                }
            }
        }
        out
    }
}

/// One [`SpriteParamsGpu`] slot (the dynamic offset's alignment).
const SPRITE_PARAMS_STRIDE: u64 = 256;

/// The sprite-mode instances' module (compute).
pub(crate) fn sprites_source() -> String {
    format!("{COMMON_WGSL}{DRAW_COMMON_WGSL}{SPRITES_WGSL}")
}

/// The particle simulation's module (compute).
pub(crate) fn sim_source() -> String {
    format!(
        "{}{COMMON_WGSL}{COLLIDERS_WGSL}{SIM_WGSL}",
        crate::LIBRARY_WGSL
    )
}

/// The alive-particle compaction & indirect-args module (compute).
pub(crate) fn compact_source() -> String {
    format!("{COMMON_WGSL}{COMPACT_WGSL}")
}

/// The strongest acceleration `forces` give anywhere (m/s²), for a bound.
fn fields_peak(forces: &og_env::force::Forces) -> f32 {
    forces
        .fields
        .iter()
        .map(|f| f.strength.abs())
        .filter(|s| s.is_finite())
        .sum()
}

/// The vec4s of one field in the force-field buffer (`force_field_at`).
const FIELD_ROWS: usize = 6;

/// The force fields as `force_field_at` reads them.
pub(crate) fn pack_fields(forces: &og_env::force::Forces) -> Vec<[f32; 4]> {
    use og_env::force::FieldKind;
    let mut out = vec![[0.0; 4]];
    for f in forces.fields.iter().filter(|f| f.strength.is_finite()) {
        let kind = match f.kind {
            FieldKind::Directional => 0.0,
            FieldKind::Radial => 1.0,
            FieldKind::Vortex => 2.0,
        };
        let v = f.volume;
        out.extend([
            f.at.position.extend(kind).to_array(),
            f.at.quaternion.normalize().to_array(),
            f.at.scale.extend(f.strength).to_array(),
            f.direction.extend(v.is_some() as u32 as f32).to_array(),
            v.map_or([0.0; 4], |v| v.size.extend(v.radius).to_array()),
            [v.map_or(0.0, |v| v.fade), 0.0, 0.0, 0.0],
        ]);
    }
    out[0][0] = ((out.len() - 1) / FIELD_ROWS) as f32;
    out
}

fn cpu_vertex_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    crate::traffic::create_buffer(
        device,
        &wgpu::BufferDescriptor {
            label: Some("particles (cpu)"),
            size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        },
    )
}

fn collider_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    crate::traffic::create_buffer(
        device,
        &wgpu::BufferDescriptor {
            label: Some("particle colliders"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        },
    )
}

/// The box a system's particles can reach in their longest life.
fn reach(e: &ParticleEmitter, transform: Mat4, tick_rate: f32) -> (Vec3, Vec3) {
    let scale = transform
        .to_scale_rotation_translation()
        .0
        .abs()
        .max_element();
    let shape = match e.shape {
        EmitterShape::Point => 0.0,
        EmitterShape::Sphere { radius, .. } => radius,
        EmitterShape::Cone { radius, .. } => radius,
        EmitterShape::Box { half_extents } => half_extents.length(),
        EmitterShape::Mesh { .. } => 2.0,
        EmitterShape::Spline { points, count, .. } => points[..count as usize]
            .iter()
            .map(|p| p.length())
            .fold(0.0, f32::max),
    } * scale;
    let life = e.max_lifetime() + 1.0 / tick_rate;
    let speed = e.speed[0].abs().max(e.speed[1].abs()) * scale;
    let field = match &e.field {
        Some(crate::particles::VectorField::Curl { strength, .. }) => strength.abs(),
        Some(crate::particles::VectorField::Grid {
            strength, vectors, ..
        }) => strength.abs() * vectors.iter().map(|v| v.length()).fold(0.0, f32::max),
        None => 0.0,
    };
    let pull = (e.gravity.length()
        + e.noise.strength.abs()
        + field
        + e.attractors.iter().map(|a| a.strength.abs()).sum::<f32>())
        * crate::particles::force_scale(e, transform);
    let peak = |c: &crate::particles::Curve| c.keys.iter().map(|k| k.1).fold(0.0, f32::max);
    let side = e.size[0].abs().max(e.size[1].abs())
        * peak(&e.size_over_life).max(e.height_over_life.as_ref().map_or(0.0, peak));
    let stretch = match e.align {
        ParticleAlign::Velocity { stretch } => speed * stretch.abs(),
        _ => 0.0,
    };
    let extent = (0.5 * (side + stretch) * scale).max(0.5);
    let r = (shape + speed * life + 0.5 * pull * life * life + extent).min(500.0);
    let at = transform.w_axis.truncate();
    (at - Vec3::splat(r), at + Vec3::splat(r))
}

/// The box a system's particles stay inside after its tick.
pub(crate) fn swept_bound(
    e: &ParticleEmitter,
    sys: &crate::particles::ParticleSystem,
    tick_rate: f32,
    clock: &crate::particles::ParticleClock,
    path: &mut std::collections::VecDeque<(f64, Vec3)>,
    pull: f32,
) -> (Vec3, Vec3) {
    let (lo, hi) = reach(e, sys.transform, tick_rate);
    let at = sys.transform.w_axis.truncate();
    let life = f64::from(e.max_lifetime()) + 2.0 / f64::from(tick_rate.max(1e-3));
    let air = (crate::particles::carrying_air(e, sys.air, None).length() * life as f32
        + 0.5 * pull * (life * life) as f32)
        .min(500.0);
    let (mut lo, mut hi) = (lo - Vec3::splat(air), hi + Vec3::splat(air));
    if e.space == SimulationSpace::World {
        let now = clock.at(sys.tick + 1, 1.0 / f64::from(tick_rate.max(1e-3)));
        if path.back().is_some_and(|p| p.0 > now) {
            path.clear();
        }
        if path.back().is_none_or(|p| p.1 != at) {
            path.push_back((now, at));
        }
        while path.len() > 1 && path[1].0 <= now - life {
            path.pop_front();
        }
        let (below, above) = (at - lo, hi - at);
        for &(_, p) in path.iter() {
            lo = lo.min(p - below);
            hi = hi.max(p + above);
        }
    }
    (lo, hi)
}

/// The particle draw module after the frame group, the light tables and
/// the lighting (`fluid_shared_source`: lit particles).
pub(crate) fn draw_source(frame_wgsl: &str, hook: Option<&crate::MaterialShader>) -> String {
    crate::assemble(
        &[
            frame_wgsl,
            crate::oit::OIT_WGSL,
            crate::oit::ARENA_WGSL,
            COMMON_WGSL,
            DRAW_COMMON_WGSL,
            crate::effect::EFFECT_WGSL,
            &crate::effect::hooks_source(hook),
            DRAW_WGSL,
        ],
        &[crate::Capability::ScreenAo],
    )
}
