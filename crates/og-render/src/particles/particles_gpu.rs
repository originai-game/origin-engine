//! The renderer's side of particles: per-system state kept by id, the
//! compute simulation, the CPU path's uploads, and the draw.

use crate::deferred::Deferred;
use crate::particles::{
    CpuParticles, EmitterShape, MAX_CURVE_KEYS, ParticleAlign, ParticleBlend, ParticleEmitter,
    ParticleRender, ParticleState, ParticleSystem, SheetMode, SimulationSpace, TickStep,
    appearance, array_frames, hash_seed, plan_ticks, sheet_frames,
};
use crate::scene::{RenderScene, Sprite};
use crate::sprites::ExtraSprite;
use crate::{HDR_FORMAT, RenderError};
use glam::{Mat4, Vec2, Vec3};
use std::collections::HashMap;

mod emitter;
mod prepare;
mod renderer;
pub(crate) use emitter::*;

const COMMON_WGSL: &str = wgsl!("particle_common.wgsl");
const SIM_WGSL: &str = wgsl!("particles_sim.wgsl");
const COLLIDERS_WGSL: &str = wgsl!("scene_colliders.wgsl");
const DRAW_WGSL: &str = wgsl!("particles.wgsl");
const SPRITES_WGSL: &str = wgsl!("particle_sprites.wgsl");

/// Bytes between emitter uniform entries (a multiple of the 256-byte
/// dynamic offset alignment).
const EMITTER_STRIDE: u64 = 1280;
const _: () = assert!(std::mem::size_of::<EmitterGpu>() as u64 <= EMITTER_STRIDE);
const PARTICLE_STRIDE: u64 = std::mem::size_of::<ParticleState>() as u64;

pub(crate) const PARTICLE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: PARTICLE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4],
};

/// `struct Emitter` in particle_common.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct EmitterGpu {
    transform: [[f32; 4]; 4],
    shape: [f32; 4],
    shape2: [f32; 4],
    ranges0: [f32; 4],
    ranges1: [f32; 4],
    ranges2: [f32; 4],
    forces: [f32; 4],
    noise: [f32; 4],
    collision: [f32; 4],
    color: [f32; 4],
    size_t: [[f32; 4]; 2],
    size_v: [[f32; 4]; 2],
    color_t: [[f32; 4]; 2],
    color_v: [[f32; 4]; 8],
    sheet: [f32; 4],
    sheet2: [f32; 4],
    render: [f32; 4],
    counts: [f32; 4],
    spawn: [u32; 4],
    info: [u32; 4],
    /// The turbulence's lattice at the render origin (`origin.rs`): xyz
    /// its whole cells, then the rest.
    noise_origin: [[f32; 4]; 2],
    /// x: lit, y: distortion, z: a mesh per particle.
    look: [f32; 4],
    speed_t: [[f32; 4]; 2],
    speed_v: [[f32; 4]; 2],
    spin_t: [[f32; 4]; 2],
    spin_v: [[f32; 4]; 2],
    attract: [[f32; 4]; 4],
    attract_r: [f32; 4],
    /// x: 0 none, 1 curl, 2 grid; y strength; z frequency or size; w
    /// scroll or cells.
    field: [f32; 4],
    field_origin: [[f32; 4]; 2],
    /// xyz: the emitter's velocity, w: the fraction inherited.
    motion: [f32; 4],
    /// x: speed keys, y: spin keys, z: attractors, w: where a grid's
    /// vectors start in the points buffer.
    counts2: [f32; 4],
    /// xyz: what drag closes on, in the simulation space
    /// (`carrying_air`: the emitter's `wind` share of the air).
    air: [f32; 4],
    /// The height over life (`height_over_life`; `counts3.x` keys, 0:
    /// the size curve's).
    height_t: [[f32; 4]; 2],
    height_v: [[f32; 4]; 2],
    /// x: height keys.
    counts3: [f32; 4],
    /// xyz: an upright or facing particle's axis, in the emitter's space;
    /// w: 1 an effect mesh facing the camera.
    axis: [f32; 4],
    /// The second colour over life (`random_color_over_life`;
    /// `counts3.y` keys, 0: none).
    color2_t: [[f32; 4]; 2],
    color2_v: [[f32; 4]; 8],
}

/// How a system's particles are drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum DrawKind {
    Billboard,
    Mesh,
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

/// A mesh's vertices beside the particles (`MeshIn` in particles.wgsl):
/// [`crate::Vertex`]'s position, normal and colour.
const MESH_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
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
}

struct GpuSystem {
    buffer: wgpu::Buffer,
    /// Its sim bind group (`particles_sim.wgsl`'s group 1): its particles
    /// and its emitter's surface points, made with them.
    group: wgpu::BindGroup,
    last_tick: Option<u64>,
    prev_transform: Option<Mat4>,
    clock: crate::particles::ParticleClock,
}

struct SystemState {
    emitter: usize,
    max: u32,
    gpu: Option<GpuSystem>,
    cpu: CpuParticles,
    /// The CPU path's vertex buffer.
    upload: Option<wgpu::Buffer>,
    seen: u64,
    /// The render origin's moves since the system last simulated
    /// ([`ParticleRenderer::shift`]): added to its world-space particles.
    shift: Vec3,
    /// Its sub-emitters' vertex buffers, by sub-emitter.
    children: Vec<Option<wgpu::Buffer>>,
    /// Where a world-space system's emitter stood in its particles' longest
    /// life ([`swept_bound`]).
    path: std::collections::VecDeque<(f64, Vec3)>,
}

/// One system's work this frame.
pub(crate) struct FrameSystem {
    id: u64,
    /// Emitter uniform entries: the ticks to simulate, then the draw entry.
    first_entry: u32,
    ticks: u32,
    reset: bool,
    /// The entry that moves its particles with the render origin first.
    shift: Option<u32>,
    gpu: bool,
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
    /// A box its particles stay inside ([`swept_bound`]): a view it does
    /// not touch skips it. None: drawn in every view.
    bound: Option<(Vec3, Vec3)>,
    kind: DrawKind,
    mesh: Option<crate::MeshHandle>,
    /// Its effect material (`effect.rs`) and how its pipeline differs.
    effect: Option<crate::EffectMaterialHandle>,
    look: Look,
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
    pub id: u64,
    /// Its draw entry among the frame's emitter uniforms.
    pub draw_entry: u32,
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
    _pad0: u32,
    z: f32,
    glow: f32,
    lit: f32,
    _pad1: f32,
}

/// What the frame's particles need from the rest of the renderer.
#[derive(Default)]
pub(crate) struct ParticleFrame {
    pub systems: Vec<FrameSystem>,
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
}

pub(crate) struct ParticleRenderer {
    /// Sprite-mode systems' instances for the 2D batch (`expand_sprites`).
    sprites_layout: wgpu::BindGroupLayout,
    /// Made on first use, as `sim` and `draw_module` (deferred.rs).
    sprites_pipeline: Deferred<wgpu::ComputePipeline>,
    sprites_params: wgpu::Buffer,
    sim_layout: wgpu::BindGroupLayout,
    sim_system_layout: wgpu::BindGroupLayout,
    /// simulate, clear, shift.
    sim: Deferred<(
        wgpu::ComputePipeline,
        wgpu::ComputePipeline,
        wgpu::ComputePipeline,
    )>,
    pub draw_layout: wgpu::BindGroupLayout,
    draw: HashMap<(ParticleBlend, u32, DrawKind, Look), wgpu::RenderPipeline>,
    pub(crate) draw_module: crate::deferred::Module,
    /// The draw module with an effect material's hook, by shader index.
    hook_modules: HashMap<usize, wgpu::ShaderModule>,
    frame_wgsl: String,
    draw_pl: wgpu::PipelineLayout,
    /// The effect materials (group 3 of every particle and strip draw).
    pub(crate) effects: crate::effect::EffectRenderer,
    emitter_buffer: wgpu::Buffer,
    /// The frame's emitter entries at [`EMITTER_STRIDE`], kept between
    /// frames (their padding is never read).
    emitter_bytes: Vec<u8>,
    pub view_buffer: wgpu::Buffer,
    /// The scene's colliders as `scene_colliders.rs` packs them, with a
    /// list per system that meets them (`emitter.info.z`).
    colliders: wgpu::Buffer,
    systems: HashMap<u64, SystemState>,
    scene_id: u64,
    /// Sampled mesh surfaces per emitter index.
    /// Per emitter: its shape and field, and its shape's table (a mesh's
    /// or a spline's points) followed by a grid field's vectors, with
    /// the shape table's length.
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

/// Which of a camera's 3D systems a draw takes. The main camera's
/// order-independent passes (`oit.rs`) split them: debris before the
/// blended surfaces (it writes depth, and they test against it), alpha
/// billboards among them, additive and distortion billboards after their
/// resolve.
#[derive(Clone, Copy)]
pub(crate) enum ParticlePart<'a> {
    /// Every system, in one place (no OIT).
    All,
    /// Mesh systems (debris).
    Before,
    /// Alpha billboards, with the OIT pipeline given.
    Oit(&'a wgpu::RenderPipeline),
    /// Additive and distortion billboards.
    After,
}

impl ParticlePart<'_> {
    fn draws(self, s: &FrameSystem) -> bool {
        match self {
            ParticlePart::All => true,
            ParticlePart::Before => solid_mesh(s),
            ParticlePart::Oit(_) => in_oit(s),
            ParticlePart::After => !solid_mesh(s) && !in_oit(s),
        }
    }
}

/// Debris: meshes that write depth (an effect mesh is blended).
fn solid_mesh(s: &FrameSystem) -> bool {
    s.kind == DrawKind::Mesh && !s.look.effect_mesh
}

/// Whether a system's particles join the order-independent passes: alpha
/// billboards, except those whose effect material has a hook (the OIT
/// pipelines are the built-in module's), which draw after the resolve.
fn in_oit(s: &FrameSystem) -> bool {
    s.kind == DrawKind::Billboard && s.blend == ParticleBlend::Alpha && s.look.hook.is_none()
}

impl ParticleFrame {
    /// Whether a camera on `mask` draws a system that joins the
    /// order-independent passes.
    pub fn has_oit(&self, mask: u32) -> bool {
        self.systems
            .iter()
            .any(|s| s.layers & mask != 0 && in_oit(s))
    }
}

/// One [`SpriteParamsGpu`] slot (the dynamic offset's alignment).
const SPRITE_PARAMS_STRIDE: u64 = 256;

/// The sprite-mode instances' module (compute).
pub(crate) fn sprites_source() -> String {
    format!("{COMMON_WGSL}{SPRITES_WGSL}")
}

/// The particle simulation's module (compute).
pub(crate) fn sim_source() -> String {
    format!(
        "{}{COMMON_WGSL}{COLLIDERS_WGSL}{SIM_WGSL}",
        crate::LIBRARY_WGSL
    )
}

fn collider_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("particle colliders"),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// The box a system's particles can reach in their longest life: the
/// emitter's shape, plus the furthest its fastest particle travels under
/// gravity and turbulence (drag only shortens it) and half its longest
/// side, clamped at 500 m.
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
        // Unknown without the mesh: the reach below is generous.
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
    let pull = e.gravity.length()
        + e.noise.strength.abs()
        + field
        + e.attractors.iter().map(|a| a.strength.abs()).sum::<f32>();
    // Half the particle's longest side (its size curve's peak, its height
    // curve's, a velocity stretch's length), at least 0.5 m.
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

/// The box a system's particles stay inside after its tick, its clock at
/// that tick: [`reach`] about every place its emitter stood within its
/// particles' longest life of the system's own time (a world-space
/// particle stays where it was born; a local one moves with the emitter),
/// widened by the air its drag carries them on, which `reach` leaves out.
/// `path` holds those places for a world-space system, kept from frame to
/// frame; it starts over when the system's time goes back (a seek).
pub(crate) fn swept_bound(
    e: &ParticleEmitter,
    sys: &crate::particles::ParticleSystem,
    tick_rate: f32,
    clock: &crate::particles::ParticleClock,
    path: &mut std::collections::VecDeque<(f64, Vec3)>,
) -> (Vec3, Vec3) {
    let (lo, hi) = reach(e, sys.transform, tick_rate);
    let at = sys.transform.w_axis.truncate();
    let life = f64::from(e.max_lifetime()) + 2.0 / f64::from(tick_rate.max(1e-3));
    let air = crate::particles::carrying_air(e, sys.air, None).length() * life as f32;
    let (mut lo, mut hi) = (lo - Vec3::splat(air), hi + Vec3::splat(air));
    if e.space == SimulationSpace::World {
        let now = clock.at(sys.tick + 1, 1.0 / f64::from(tick_rate.max(1e-3)));
        if path.back().is_some_and(|p| p.0 > now) {
            path.clear();
        }
        if path.back().is_none_or(|p| p.1 != at) {
            path.push_back((now, at));
        }
        // The oldest place goes once the one after it is a life old: the
        // emitter stood there until then.
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
            crate::effect::EFFECT_WGSL,
            &crate::effect::hooks_source(hook),
            DRAW_WGSL,
        ],
        &[crate::Capability::ScreenAo],
    )
}
