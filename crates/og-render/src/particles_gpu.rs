//! The renderer's side of particles: per-system state kept by id, the
//! compute simulation, the CPU path's uploads, and the draw.

use crate::deferred::Deferred;
use crate::particles::{
    CpuParticles, EmitterShape, MAX_CURVE_KEYS, ParticleAlign, ParticleBlend, ParticleEmitter,
    ParticleRender, ParticleState, ParticleSystem, SheetMode, SimulationSpace, TickStep,
    appearance, array_frames, hash_seed, mesh_points, plan_ticks, sheet_frames,
};
use crate::scene::{RenderScene, Sprite};
use crate::sprites::ExtraSprite;
use crate::{HDR_FORMAT, RenderError};
use glam::{Mat4, Vec2, Vec3};
use std::collections::HashMap;

const COMMON_WGSL: &str = wgsl!("particle_common.wgsl");
const SIM_WGSL: &str = wgsl!("particles_sim.wgsl");
const DRAW_WGSL: &str = wgsl!("particles.wgsl");
const SPRITES_WGSL: &str = wgsl!("particle_sprites.wgsl");
pub(crate) const NOISE_WGSL: &str = wgsl!("noise.wgsl");

/// Bytes between emitter uniform entries (a multiple of the 256-byte
/// dynamic offset alignment).
const EMITTER_STRIDE: u64 = 768;
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
}

/// `struct ParticleView` in particle_common.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ParticleViewGpu {
    pub view_proj: [[f32; 4]; 4],
    pub cam_pos: [f32; 4],
    pub cam_forward: [f32; 4],
    pub cam_right: [f32; 4],
    pub cam_up: [f32; 4],
    pub screen: [f32; 4],
}

#[allow(clippy::too_many_arguments)]
fn emitter_gpu(
    e: &ParticleEmitter,
    transform: Mat4,
    step: Option<&TickStep>,
    dt: f32,
    seed: u32,
    points: u32,
    textured: bool,
    collide: bool,
    soft: f32,
) -> EmitterGpu {
    let (kind, a, b) = match e.shape {
        EmitterShape::Point => (0.0, 0.0, 0.0),
        EmitterShape::Sphere { radius, surface } => (1.0, radius, surface as u32 as f32),
        EmitterShape::Cone {
            angle_degrees,
            radius,
        } => (2.0, angle_degrees.to_radians(), radius),
        EmitterShape::Box { .. } => (3.0, 0.0, 0.0),
        EmitterShape::Mesh { .. } => (4.0, points as f32, 0.0),
    };
    let half = match e.shape {
        EmitterShape::Box { half_extents } => half_extents,
        _ => Vec3::ZERO,
    };
    let mut size_t = [[0.0; 4]; 2];
    let mut size_v = [[0.0; 4]; 2];
    for (i, (t, v)) in e
        .size_over_life
        .keys
        .iter()
        .take(MAX_CURVE_KEYS)
        .enumerate()
    {
        size_t[i / 4][i % 4] = *t;
        size_v[i / 4][i % 4] = *v;
    }
    let mut color_t = [[0.0; 4]; 2];
    let mut color_v = [[0.0; 4]; 8];
    for (i, (t, c)) in e
        .color_over_life
        .keys
        .iter()
        .take(MAX_CURVE_KEYS)
        .enumerate()
    {
        color_t[i / 4][i % 4] = *t;
        color_v[i] = *c;
    }
    let (mode, param) = match e.sheet.mode {
        SheetMode::Off => (0.0, 0.0),
        SheetMode::OverLife { cycles } => (1.0, cycles),
        SheetMode::Fps(fps) => (2.0, fps),
        SheetMode::Random => (3.0, 0.0),
    };
    let (align, stretch) = match e.align {
        ParticleAlign::Camera => (0.0, 0.0),
        ParticleAlign::Velocity { stretch } => (1.0, stretch),
    };
    let (bounce, friction) = e.collision.map_or((0.0, 0.0), |c| (c.bounce, c.friction));
    let max = e.max_particles;
    let (first, count, bursts, time) =
        step.map_or((0, 0, 0, 0.0), |s| (s.first, s.count, s.bursts, s.time));
    EmitterGpu {
        transform: transform.to_cols_array_2d(),
        shape: [kind, a, b, 0.0],
        shape2: [half.x, half.y, half.z, e.spread_degrees.to_radians()],
        ranges0: [e.lifetime[0], e.lifetime[1], e.speed[0], e.speed[1]],
        ranges1: [
            e.size[0],
            e.size[1],
            e.rotation_degrees[0].to_radians(),
            e.rotation_degrees[1].to_radians(),
        ],
        ranges2: [
            e.angular_velocity_degrees[0].to_radians(),
            e.angular_velocity_degrees[1].to_radians(),
            if e.space == SimulationSpace::Local {
                1.0
            } else {
                0.0
            },
            dt,
        ],
        forces: e.gravity.extend(e.drag).to_array(),
        noise: [e.noise.strength, e.noise.frequency, e.noise.scroll, time],
        collision: [collide as u32 as f32, bounce, friction, 0.0],
        color: e.color,
        size_t,
        size_v,
        color_t,
        color_v,
        sheet: [
            e.sheet.columns.max(1) as f32,
            e.sheet.rows.max(1) as f32,
            sheet_frames(&e.sheet) as f32,
            mode,
        ],
        sheet2: [param, textured as u32 as f32, 0.0, 0.0],
        render: [
            align,
            stretch,
            soft,
            match e.blend {
                ParticleBlend::Additive => 0.0,
                ParticleBlend::Alpha => 1.0,
            },
        ],
        counts: [
            e.size_over_life.keys.len().min(MAX_CURVE_KEYS) as f32,
            e.color_over_life.keys.len().min(MAX_CURVE_KEYS) as f32,
            0.0,
            0.0,
        ],
        spawn: [first as u32, count, (first % max as u64) as u32, bursts],
        info: [max, seed, 0, 0],
        noise_origin: [[0.0; 4]; 2],
        look: [
            e.lit as u32 as f32,
            e.distortion,
            matches!(e.render, ParticleRender::Mesh { .. }) as u32 as f32,
            0.0,
        ],
    }
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
    ],
};

struct GpuSystem {
    buffer: wgpu::Buffer,
    points: wgpu::Buffer,
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
    /// Slots drawn.
    max: u32,
    kind: DrawKind,
    mesh: Option<crate::MeshHandle>,
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
    /// simulate, clear, shift.
    sim: Deferred<(
        wgpu::ComputePipeline,
        wgpu::ComputePipeline,
        wgpu::ComputePipeline,
    )>,
    pub draw_layout: wgpu::BindGroupLayout,
    draw: HashMap<(ParticleBlend, u32, DrawKind), wgpu::RenderPipeline>,
    pub(crate) draw_module: crate::deferred::Module,
    draw_pl: wgpu::PipelineLayout,
    emitter_buffer: wgpu::Buffer,
    pub view_buffer: wgpu::Buffer,
    systems: HashMap<u64, SystemState>,
    scene_id: u64,
    /// Sampled mesh surfaces per emitter index.
    points: HashMap<usize, (EmitterShape, Vec<[f32; 4]>)>,
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

impl ParticleRenderer {
    /// Makes one compute pipeline not yet made; false when all are
    /// ([`crate::Renderer::warm`]).
    pub(crate) fn warm(&self) -> bool {
        crate::deferred::warm_one(&[&self.sim, &self.sprites_pipeline])
    }

    /// Carries every system across a move of the render origin (`origin.rs`;
    /// `d`, the old origin minus the new, is added to what is in world
    /// space): their last emitter transforms now, their world-space
    /// particles when they next simulate.
    pub(crate) fn shift(&mut self, d: Vec3) {
        let moved = |m: &mut Option<Mat4>| {
            if let Some(m) = m {
                m.w_axis += d.extend(0.0);
            }
        };
        for s in self.systems.values_mut() {
            s.shift += d;
            moved(&mut s.cpu.prev_transform);
            if let Some(g) = &mut s.gpu {
                moved(&mut g.prev_transform);
            }
        }
    }

    pub fn new(
        device: &wgpu::Device,
        frame_layout: &wgpu::BindGroupLayout,
        sprite_layout: &wgpu::BindGroupLayout,
        frame_wgsl: &str,
    ) -> ParticleRenderer {
        let sim_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle sim"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::COMPUTE, true),
                uniform_entry(1, wgpu::ShaderStages::COMPUTE, false),
                storage_entry(2, false),
                storage_entry(3, true),
                depth_entry(4, wgpu::ShaderStages::COMPUTE),
                depth_entry(5, wgpu::ShaderStages::COMPUTE),
            ],
        });
        let sim = {
            let (device, sim_layout) = (device.clone(), sim_layout.clone());
            Deferred::new(move || {
                let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("particle sim"),
                    source: wgpu::ShaderSource::Wgsl(sim_source().into()),
                });
                let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("particle sim"),
                    bind_group_layouts: &[Some(&sim_layout)],
                    immediate_size: 0,
                });
                let pipeline = |entry: &str| {
                    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some("particle sim"),
                        layout: Some(&layout),
                        module: &module,
                        entry_point: Some(entry),
                        compilation_options: Default::default(),
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    })
                };
                (pipeline("simulate"), pipeline("clear"), pipeline("shift"))
            })
        };
        let sprites_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle sprites"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::COMPUTE, true),
                storage_entry(1, true),
                uniform_entry(2, wgpu::ShaderStages::COMPUTE, true),
                storage_entry(3, false),
            ],
        });
        let sprites_pipeline = {
            let (device, sprites_layout) = (device.clone(), sprites_layout.clone());
            Deferred::new(move || {
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("particle sprites"),
                    layout: Some(
                        &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: Some("particle sprites"),
                            bind_group_layouts: &[Some(&sprites_layout)],
                            immediate_size: 0,
                        }),
                    ),
                    module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("particle sprites"),
                        source: wgpu::ShaderSource::Wgsl(sprites_source().into()),
                    }),
                    entry_point: Some("expand"),
                    compilation_options: Default::default(),
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        let sprites_params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("particle sprites"),
            size: SPRITE_PARAMS_STRIDE * 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let draw_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle draw"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT, true),
                uniform_entry(1, wgpu::ShaderStages::VERTEX_FRAGMENT, false),
                depth_entry(2, wgpu::ShaderStages::FRAGMENT),
                depth_entry(3, wgpu::ShaderStages::FRAGMENT),
            ],
        });
        let draw_module = crate::deferred::module(device, "particles", draw_source(frame_wgsl));
        let draw_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("particles"),
            bind_group_layouts: &[Some(frame_layout), Some(sprite_layout), Some(&draw_layout)],
            immediate_size: 0,
        });
        let emitter_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("emitters"),
            size: EMITTER_STRIDE * 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let view_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("particle view"),
            size: std::mem::size_of::<ParticleViewGpu>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        ParticleRenderer {
            sprites_layout,
            sprites_pipeline,
            sprites_params,
            sim_layout,
            sim,
            draw_layout,
            draw: HashMap::new(),
            draw_module,
            draw_pl,
            emitter_buffer,
            view_buffer,
            systems: HashMap::new(),
            scene_id: 0,
            points: HashMap::new(),
            frame: 0,
        }
    }

    /// Advances every system of the scene and writes the frame's emitter
    /// uniforms. `gpu`: billboard systems simulate in compute.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        gpu: bool,
        texture_count: usize,
        eye: Vec3,
        forward: Vec3,
    ) -> Result<ParticleFrame, RenderError> {
        self.frame += 1;
        if self.scene_id != scene.id() {
            self.scene_id = scene.id();
            self.systems.clear();
            self.points.clear();
        }
        let tick_rate = scene.tick_rate.max(1e-3);
        let dt = 1.0 / tick_rate;
        let mut out = ParticleFrame::default();
        let mut entries: Vec<EmitterGpu> = Vec::new();
        for (i, sys) in scene.particles.iter().enumerate() {
            let layers = crate::views::layer_of(&scene.particle_layers, i);
            let e = scene.emitters().get(sys.emitter.index()).ok_or_else(|| {
                RenderError::InvalidHandle(format!(
                    "particle system {i}: emitter {}",
                    sys.emitter.index()
                ))
            })?;
            e.validate().map_err(|m| {
                RenderError::InvalidData(format!("emitter {}: {m}", sys.emitter.index()))
            })?;
            if let Some(t) = e.texture
                && t.index() >= texture_count
            {
                return Err(RenderError::InvalidHandle(format!(
                    "emitter {}: texture {}",
                    sys.emitter.index(),
                    t.index()
                )));
            }
            // The surface table, rebuilt when the emitter's shape changed.
            let fresh = match self.points.get(&sys.emitter.index()) {
                Some((shape, _)) => *shape != e.shape,
                None => true,
            };
            if fresh {
                let table = match e.shape {
                    EmitterShape::Mesh { mesh } => {
                        let data = scene.mesh(mesh).ok_or_else(|| {
                            RenderError::InvalidHandle(format!(
                                "emitter {}: mesh {}",
                                sys.emitter.index(),
                                mesh.index()
                            ))
                        })?;
                        mesh_points(data)
                    }
                    _ => vec![[0.0; 4]; 2],
                };
                self.points.insert(sys.emitter.index(), (e.shape, table));
            }
            let points = &self.points[&sys.emitter.index()].1;
            let state = self.systems.entry(sys.id).or_insert_with(|| SystemState {
                emitter: sys.emitter.index(),
                max: e.max_particles,
                gpu: None,
                cpu: CpuParticles::default(),
                upload: None,
                seen: 0,
                shift: Vec3::ZERO,
                children: Vec::new(),
            });
            if state.emitter != sys.emitter.index() || state.max != e.max_particles {
                *state = SystemState {
                    emitter: sys.emitter.index(),
                    max: e.max_particles,
                    gpu: None,
                    cpu: CpuParticles::default(),
                    upload: None,
                    seen: 0,
                    shift: Vec3::ZERO,
                    children: Vec::new(),
                };
            }
            state.seen = self.frame;
            out.capacity += e.max_particles;
            let pending = std::mem::take(&mut state.shift);
            let (moved, origin) = match e.space {
                SimulationSpace::World => (pending, scene.origin),
                SimulationSpace::Local => (Vec3::ZERO, Vec3::ZERO),
            };
            state.cpu.origin = origin;
            let noise_origin = {
                let (whole, rest) = crate::particles::noise_origin(origin, e.noise.frequency);
                [
                    [whole[0] as f32, whole[1] as f32, whole[2] as f32, 0.0],
                    rest.extend(0.0).to_array(),
                ]
            };
            if moved != Vec3::ZERO {
                for p in &mut state.cpu.slots {
                    for (c, d) in p.pos_age.iter_mut().zip(moved.to_array()) {
                        *c += d;
                    }
                }
            }
            // Sub-emitters' particles are in the world whatever the parent's space.
            if pending != Vec3::ZERO {
                for c in &mut state.cpu.children {
                    for p in &mut c.slots {
                        for (x, d) in p.pos_age.iter_mut().zip(pending.to_array()) {
                            *x += d;
                        }
                    }
                }
            }
            let seed = hash_seed(sys.seed);
            let subs: Vec<Option<&ParticleEmitter>> = e
                .sub_emitters
                .iter()
                .map(|s| scene.emitters().get(s.emitter.index()))
                .collect();
            if let ParticleRender::Mesh { mesh } = e.render {
                if scene.mesh(mesh).is_none() {
                    return Err(RenderError::InvalidHandle(format!(
                        "emitter {}: mesh {}",
                        sys.emitter.index(),
                        mesh.index()
                    )));
                }
                out.meshes.push(mesh.index());
            }
            for (k, sub) in subs.iter().enumerate() {
                let Some(ce) = sub else {
                    return Err(RenderError::InvalidHandle(format!(
                        "emitter {}: sub-emitter {k}: emitter {}",
                        sys.emitter.index(),
                        e.sub_emitters[k].emitter.index()
                    )));
                };
                ce.validate().map_err(|m| {
                    RenderError::InvalidData(format!(
                        "emitter {}: sub-emitter {k}: {m}",
                        sys.emitter.index()
                    ))
                })?;
                if ce.texture.is_some_and(|t| t.index() >= texture_count) {
                    return Err(RenderError::InvalidHandle(format!(
                        "emitter {}: sub-emitter {k}: texture",
                        sys.emitter.index()
                    )));
                }
                if let ParticleRender::Mesh { mesh } = ce.render {
                    if scene.mesh(mesh).is_none() {
                        return Err(RenderError::InvalidHandle(format!(
                            "emitter {}: sub-emitter {k}: mesh {}",
                            sys.emitter.index(),
                            mesh.index()
                        )));
                    }
                    out.meshes.push(mesh.index());
                }
            }
            // A host's own simulation of it (its tick read the events).
            let hosted = scene
                .hosted_particles
                .iter()
                .find(|(id, _)| *id == sys.id)
                .map(|(_, p)| p);
            let sprite_mode = matches!(e.render, ParticleRender::Sprite { .. });
            // The CPU path bounces world-space billboards off the scene's
            // colliders; sprite systems and a scene without colliders pass through.
            let colliders: &[crate::particles::ParticleCollider] =
                if e.collision.is_some() && e.space == SimulationSpace::World && !sprite_mode {
                    &scene.particle_colliders
                } else {
                    &[]
                };
            if e.collision.is_some() && (sprite_mode || (!gpu && colliders.is_empty())) {
                out.collision_skipped += 1;
            }
            // Sprite mode: its layer, and an array emitter's array with its
            // frames (the array's layers).
            let sprite = match e.render {
                ParticleRender::Sprite { layer } => Some((
                    layer,
                    match e.array {
                        Some(a) => {
                            let layers = scene.texture_array(a).map(|d| d.layers.len() as u32);
                            let layers = layers.ok_or_else(|| {
                                RenderError::InvalidHandle(format!(
                                    "emitter {}: texture array {}",
                                    sys.emitter.index(),
                                    a.index()
                                ))
                            })?;
                            Some((a, array_frames(&e.sheet, layers)))
                        }
                        None => None,
                    },
                )),
                _ => None,
            };
            // Sub-emitters' events and a host's own simulation are the CPU
            // path's; sprite systems without them simulate in compute too
            // and the 2D batch draws them from there (`sprite_systems`).
            let gpu = gpu && e.sub_emitters.is_empty() && hosted.is_none();
            if let Some((layer, array)) = sprite
                && !gpu
            {
                match hosted {
                    Some(h) => take_hosted(&mut state.cpu, h, e.space, scene.origin),
                    None => {
                        out.ticks += state
                            .cpu
                            .advance(e, sys, tick_rate, points, &[], &subs, None)
                    }
                }
                out.cpu_alive += state.cpu.alive() as u32;
                push_sprites(
                    e,
                    sys,
                    &state.cpu.slots,
                    (layer, layers),
                    array,
                    &mut out.sprites,
                );
                for (c, ce) in state.cpu.children.iter().zip(&subs) {
                    let Some(ce) = ce else { continue };
                    let world = ParticleSystem {
                        transform: Mat4::IDENTITY,
                        ..*sys
                    };
                    let layer = match ce.render {
                        ParticleRender::Sprite { layer } => layer,
                        _ => layer,
                    };
                    let ce = ParticleEmitter {
                        space: SimulationSpace::World,
                        ..(*ce).clone()
                    };
                    push_sprites(
                        &ce,
                        &world,
                        &c.slots,
                        (layer, layers),
                        None,
                        &mut out.sprites,
                    );
                }
                continue;
            }
            let depth = (sys.transform.w_axis.truncate() - eye).dot(forward);
            let textured = e.texture.is_some();
            let collide = gpu
                && e.collision.is_some()
                && e.space == SimulationSpace::World
                && sprite.is_none();
            // Soft particles fade against the depth behind them, where the
            // frame's quality lets them (RenderQuality::soft_particles).
            let soft = if scene.quality.soft_particles && sprite.is_none() {
                e.soft_distance.max(0.0)
            } else {
                0.0
            };
            out.needs_depth |= soft > 0.0 || collide;
            out.soft |= soft > 0.0;
            let kind = match sprite {
                Some(_) => DrawKind::Sprite,
                None => draw_kind(e),
            };
            out.distortion |= kind == DrawKind::Distort;
            let mesh = match e.render {
                ParticleRender::Mesh { mesh } => Some(mesh),
                _ => None,
            };
            // The render origin moved: world-space particles on the GPU
            // move with it before this frame's ticks (the CPU's moved above).
            let shift = (gpu && moved != Vec3::ZERO && state.gpu.is_some()).then(|| {
                let mut entry: EmitterGpu = bytemuck::Zeroable::zeroed();
                entry.transform = Mat4::from_translation(moved).to_cols_array_2d();
                entry.info = [e.max_particles, 0, 0, 0];
                entries.push(entry);
                entries.len() as u32 - 1
            });
            let first_entry = entries.len() as u32;
            if gpu {
                let g = state.gpu.get_or_insert_with(|| GpuSystem {
                    buffer: device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("particles"),
                        size: PARTICLE_STRIDE * e.max_particles as u64,
                        usage: wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::VERTEX
                            | wgpu::BufferUsages::COPY_SRC
                            | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                    points: crate::buffer_init(
                        device,
                        queue,
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("emitter mesh points"),
                            contents: bytemuck::cast_slice(points),
                            usage: wgpu::BufferUsages::STORAGE,
                        },
                    ),
                    last_tick: None,
                    prev_transform: None,
                    clock: Default::default(),
                });
                let (reset, steps) = plan_ticks(
                    e,
                    sys,
                    g.last_tick,
                    g.prev_transform,
                    tick_rate,
                    &mut g.clock,
                );
                g.last_tick = Some(sys.tick);
                g.prev_transform = Some(sys.transform);
                let n_points = (points.len() / 2) as u32;
                for s in &steps {
                    entries.push(emitter_gpu(
                        e,
                        s.transform,
                        Some(s),
                        s.dt,
                        seed,
                        n_points,
                        textured,
                        collide,
                        soft,
                    ));
                }
                entries.push(emitter_gpu(
                    e,
                    sys.transform,
                    None,
                    dt,
                    seed,
                    n_points,
                    textured,
                    collide,
                    soft,
                ));
                for g in &mut entries[first_entry as usize..] {
                    g.noise_origin = noise_origin;
                }
                out.ticks += steps.len() as u32;
                out.any_gpu = true;
                out.systems.push(FrameSystem {
                    id: sys.id,
                    first_entry,
                    ticks: steps.len() as u32,
                    reset,
                    shift,
                    gpu: true,
                    blend: e.blend,
                    texture: e.texture,
                    depth,
                    layers,
                    child: None,
                    max: e.max_particles,
                    kind,
                    mesh,
                });
                if let Some((layer, array)) = sprite {
                    out.sprite_systems.push(SpriteSystem {
                        id: sys.id,
                        draw_entry: first_entry + steps.len() as u32,
                        layer,
                        layers,
                        blend: match e.blend {
                            ParticleBlend::Additive => crate::SpriteBlend::Additive,
                            ParticleBlend::Alpha => crate::SpriteBlend::Alpha,
                        },
                        glow: e.glow,
                        soft: e.round || (e.texture.is_none() && array.is_none()),
                        texture: if array.is_some() { None } else { e.texture },
                        array: array.map(|(a, _)| a),
                        frames: array.map_or_else(|| sheet_frames(&e.sheet), |(_, n)| n),
                        columns: e.sheet.columns.max(1),
                        rows: e.sheet.rows.max(1),
                        max: e.max_particles,
                        z: sys.transform.w_axis.z,
                    });
                }
            } else {
                match hosted {
                    Some(h) => take_hosted(&mut state.cpu, h, e.space, scene.origin),
                    None => {
                        out.ticks += state
                            .cpu
                            .advance(e, sys, tick_rate, points, colliders, &subs, None)
                    }
                }
                out.cpu_alive += state.cpu.alive() as u32;
                upload(device, queue, &mut state.upload, &state.cpu.slots);
                entries.push(emitter_gpu(
                    e,
                    sys.transform,
                    None,
                    dt,
                    seed,
                    0,
                    textured,
                    false,
                    soft,
                ));
                out.systems.push(FrameSystem {
                    id: sys.id,
                    first_entry,
                    ticks: 0,
                    reset: false,
                    shift: None,
                    gpu: false,
                    blend: e.blend,
                    texture: e.texture,
                    depth,
                    layers,
                    child: None,
                    max: state.cpu.slots.len() as u32,
                    kind,
                    mesh,
                });
            }
            // Its sub-emitters' particles, each drawn as its own emitter
            // draws (in the world).
            state
                .children
                .resize_with(state.cpu.children.len(), || None);
            for (k, (c, ce)) in state.cpu.children.iter().zip(&subs).enumerate() {
                let Some(ce) = ce else { continue };
                if c.slots.is_empty() {
                    continue;
                }
                out.cpu_alive += c.slots.iter().filter(|p| p.alive()).count() as u32;
                upload(device, queue, &mut state.children[k], &c.slots);
                let kind = draw_kind(ce);
                out.distortion |= kind == DrawKind::Distort;
                let soft = if scene.quality.soft_particles {
                    ce.soft_distance.max(0.0)
                } else {
                    0.0
                };
                out.needs_depth |= soft > 0.0;
                out.soft |= soft > 0.0;
                let first_entry = entries.len() as u32;
                let mut entry = emitter_gpu(
                    ce,
                    Mat4::IDENTITY,
                    None,
                    dt,
                    crate::particles::child_seed(seed, k),
                    0,
                    ce.texture.is_some(),
                    false,
                    soft,
                );
                entry.ranges2[2] = 0.0;
                entry.noise_origin = noise_origin;
                entries.push(entry);
                out.systems.push(FrameSystem {
                    id: sys.id,
                    first_entry,
                    ticks: 0,
                    reset: false,
                    shift: None,
                    gpu: false,
                    blend: ce.blend,
                    texture: ce.texture,
                    depth,
                    layers,
                    child: Some(k),
                    max: c.slots.len() as u32,
                    kind,
                    mesh: match ce.render {
                        ParticleRender::Mesh { mesh } => Some(mesh),
                        _ => None,
                    },
                });
            }
        }
        let frame = self.frame;
        self.systems.retain(|_, s| s.seen == frame);
        // The unsupported diagnostic is opaque, not emitted light. Adding
        // it to the background destroys the checker's black and magenta.
        if scene.debug != crate::DebugView::None {
            for s in &mut out.systems {
                if s.kind != DrawKind::Sprite {
                    s.blend = ParticleBlend::Alpha;
                }
            }
        }
        // Far systems first; additive after alpha within the same depth.
        og_core::sort::sort_by(&mut out.systems, |a, b| {
            b.depth
                .total_cmp(&a.depth)
                .then(a.id.cmp(&b.id))
                .then(a.child.cmp(&b.child))
        });
        if !entries.is_empty() {
            let size = EMITTER_STRIDE * entries.len() as u64;
            if size > self.emitter_buffer.size() {
                self.emitter_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("emitters"),
                    size: size.next_power_of_two(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }
            let mut bytes = vec![0u8; size as usize];
            for (i, e) in entries.iter().enumerate() {
                let at = i * EMITTER_STRIDE as usize;
                bytes[at..at + std::mem::size_of::<EmitterGpu>()]
                    .copy_from_slice(bytemuck::bytes_of(e));
            }
            queue.write_buffer(&self.emitter_buffer, 0, &bytes);
        }
        Ok(out)
    }

    /// Writes the parameters of [`ParticleRenderer::expand_sprites`]'
    /// ranges (before the frame's encoder runs).
    pub fn prepare_sprites(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ParticleFrame,
        ranges: &crate::sprites::GpuSprites,
    ) {
        if ranges.ranges.is_empty() {
            return;
        }
        let size = SPRITE_PARAMS_STRIDE * ranges.ranges.len() as u64;
        if size > self.sprites_params.size() {
            self.sprites_params = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("particle sprites"),
                size: size.next_power_of_two(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        let mut bytes = vec![0u8; size as usize];
        for (k, r) in ranges.ranges.iter().enumerate() {
            let s = &frame.sprite_systems[r.system];
            let p = SpriteParamsGpu {
                first: r.first,
                max: r.count,
                frames: s.frames,
                columns: s.columns,
                rows: s.rows,
                array: u32::from(s.array.is_some()),
                soft: u32::from(s.soft),
                _pad0: 0,
                z: s.z,
                glow: s.glow,
                lit: r.lit,
                _pad1: 0.0,
            };
            let at = k * SPRITE_PARAMS_STRIDE as usize;
            bytes[at..at + std::mem::size_of::<SpriteParamsGpu>()]
                .copy_from_slice(bytemuck::bytes_of(&p));
        }
        queue.write_buffer(&self.sprites_params, 0, &bytes);
    }

    /// Writes the instances of the sprite-mode systems simulated in compute
    /// into `target`, each range of `ranges` from its system's ring as it
    /// stands after this frame's ticks (after [`ParticleRenderer::simulate`]
    /// in the same encoder).
    pub fn expand_sprites(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        frame: &ParticleFrame,
        ranges: &crate::sprites::GpuSprites,
        target: &wgpu::Buffer,
    ) {
        if ranges.ranges.is_empty() {
            return;
        }
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("particle sprites"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.sprites_pipeline);
        for (k, r) in ranges.ranges.iter().enumerate() {
            let s = &frame.sprite_systems[r.system];
            let Some(g) = self.systems.get(&s.id).and_then(|st| st.gpu.as_ref()) else {
                continue;
            };
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("particle sprites"),
                layout: &self.sprites_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.emitter_buffer,
                            offset: 0,
                            size: wgpu::BufferSize::new(std::mem::size_of::<EmitterGpu>() as u64),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: g.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.sprites_params,
                            offset: 0,
                            size: wgpu::BufferSize::new(
                                std::mem::size_of::<SpriteParamsGpu>() as u64
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: target.as_entire_binding(),
                    },
                ],
            });
            pass.set_bind_group(
                0,
                &group,
                &[
                    (u64::from(s.draw_entry) * EMITTER_STRIDE) as u32,
                    (k as u64 * SPRITE_PARAMS_STRIDE) as u32,
                ],
            );
            pass.dispatch_workgroups(r.count.div_ceil(64), 1, 1);
        }
    }

    /// Records the frame's compute dispatches (one per simulated tick).
    pub fn simulate(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        frame: &ParticleFrame,
        depth: &wgpu::TextureView,
        gbuffer: &wgpu::TextureView,
    ) {
        let (simulate, clear, shift) = &*self.sim;
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("particle sim"),
            timestamp_writes: None,
        });
        for fs in frame.systems.iter().filter(|s| s.gpu) {
            let Some(g) = self.systems.get(&fs.id).and_then(|s| s.gpu.as_ref()) else {
                continue;
            };
            let max = self.systems[&fs.id].max;
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("particle sim"),
                layout: &self.sim_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.emitter_buffer,
                            offset: 0,
                            size: wgpu::BufferSize::new(std::mem::size_of::<EmitterGpu>() as u64),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.view_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: g.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: g.points.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(depth),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(gbuffer),
                    },
                ],
            });
            let groups = max.div_ceil(64);
            if let Some(k) = fs.shift {
                pass.set_pipeline(shift);
                pass.set_bind_group(0, &group, &[(k as u64 * EMITTER_STRIDE) as u32]);
                pass.dispatch_workgroups(groups, 1, 1);
            }
            if fs.reset {
                pass.set_pipeline(clear);
                pass.set_bind_group(
                    0,
                    &group,
                    &[(fs.first_entry as u64 * EMITTER_STRIDE) as u32],
                );
                pass.dispatch_workgroups(groups, 1, 1);
            }
            pass.set_pipeline(simulate);
            for t in 0..fs.ticks {
                let offset = ((fs.first_entry + t) as u64 * EMITTER_STRIDE) as u32;
                pass.set_bind_group(0, &group, &[offset]);
                pass.dispatch_workgroups(groups, 1, 1);
            }
        }
    }

    /// The draw bind group (emitter entries, view, scene depth): the main
    /// camera's view buffer.
    pub fn draw_group(
        &self,
        device: &wgpu::Device,
        depth: &wgpu::TextureView,
        scene_color: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        self.draw_group_of(device, &self.view_buffer, depth, scene_color)
    }

    /// The draw bind group of a camera whose [`ParticleViewGpu`] is in
    /// `view` (a secondary camera's, `views.rs`); `scene_color`: the scene
    /// copy distortion reads (a placeholder where the pass has none).
    pub fn draw_group_of(
        &self,
        device: &wgpu::Device,
        view: &wgpu::Buffer,
        depth: &wgpu::TextureView,
        scene_color: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("particle draw"),
            layout: &self.draw_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.emitter_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<EmitterGpu>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: view.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(depth),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(scene_color),
                },
            ],
        })
    }

    /// Makes the draw pipelines the frame needs (by blend and sample count).
    pub fn ensure_pipelines(&mut self, device: &wgpu::Device, frame: &ParticleFrame, samples: u32) {
        for s in frame.systems.iter().filter(|s| s.kind != DrawKind::Sprite) {
            let key = (s.blend, samples, s.kind);
            if self.draw.contains_key(&key) {
                continue;
            }
            let blend = match s.blend {
                _ if s.kind == DrawKind::Distort => wgpu::BlendState::ALPHA_BLENDING,
                ParticleBlend::Additive => wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Zero,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                },
                ParticleBlend::Alpha => wgpu::BlendState::ALPHA_BLENDING,
            };
            let mesh = s.kind == DrawKind::Mesh;
            let buffers = if mesh {
                vec![Some(PARTICLE_LAYOUT), Some(MESH_LAYOUT)]
            } else {
                vec![Some(PARTICLE_LAYOUT)]
            };
            let p = crate::hdr_pipeline(
                device,
                crate::HdrPipeline {
                    alpha_to_coverage: false,
                    label: "particles",
                    layout: &self.draw_pl,
                    module: &self.draw_module,
                    vs: if mesh { "vs_mesh" } else { "vs" },
                    fs: match s.kind {
                        DrawKind::Billboard => "fs",
                        DrawKind::Mesh => "fs_mesh",
                        DrawKind::Distort => "fs_distort",
                        DrawKind::Sprite => unreachable!("filtered above"),
                    },
                    buffers: &buffers,
                    cull: mesh.then_some(wgpu::Face::Back),
                    // Debris occludes what is behind it, and itself.
                    depth_write: mesh,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: Some(blend),
                    samples,
                },
            );
            self.draw.insert(key, p);
        }
        let _ = HDR_FORMAT;
    }

    /// Draws the frame's 3D systems on a camera's layers (`mask`) that
    /// `part` selects, far to near. `distort`: the group carries the pass's scene copy, so its
    /// distortion systems draw (elsewhere they are left out). `mesh`: a
    /// mesh's f32 vertices, indices and index count. Returns the draws.
    #[allow(clippy::too_many_arguments)]
    pub fn draw<'b>(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        frame: &ParticleFrame,
        samples: u32,
        group: &wgpu::BindGroup,
        mask: u32,
        distort: bool,
        part: ParticlePart<'_>,
        texture_group: impl Fn(Option<crate::TextureHandle>) -> &'b wgpu::BindGroup,
        mesh: impl Fn(crate::MeshHandle) -> Option<(&'b wgpu::Buffer, &'b wgpu::Buffer, u32)>,
    ) -> u32 {
        let mut draws = 0;
        for s in frame.systems.iter().filter(|s| {
            s.layers & mask != 0
                && s.kind != DrawKind::Sprite
                && (distort || s.kind != DrawKind::Distort)
                && part.draws(s)
        }) {
            let Some(state) = self.systems.get(&s.id) else {
                continue;
            };
            let buffer = match s.child {
                Some(k) => state.children[k].as_ref().expect("uploaded"),
                None if s.gpu => &state.gpu.as_ref().expect("gpu system").buffer,
                None => state.upload.as_ref().expect("uploaded"),
            };
            let Some(pipeline) = (match part {
                ParticlePart::Oit(p) => Some(p),
                _ => self.draw.get(&(s.blend, samples, s.kind)),
            }) else {
                continue;
            };
            pass.set_pipeline(pipeline);
            pass.set_bind_group(1, texture_group(s.texture), &[]);
            let draw_entry = s.first_entry + s.ticks;
            pass.set_bind_group(2, group, &[(draw_entry as u64 * EMITTER_STRIDE) as u32]);
            pass.set_vertex_buffer(0, buffer.slice(..));
            match s.mesh.map(&mesh) {
                Some(Some((vertices, indices, count))) => {
                    pass.set_vertex_buffer(1, vertices.slice(..));
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..count, 0, 0..s.max);
                }
                Some(None) => continue,
                None => pass.draw(0..6, 0..s.max),
            }
            draws += 1;
        }
        draws
    }

    /// A copy of a system's particle slots (the GPU buffer is read back),
    /// for tests and tools (native only).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: u64,
    ) -> Option<Vec<ParticleState>> {
        let state = self.systems.get(&id)?;
        let Some(g) = &state.gpu else {
            return Some(state.cpu.slots.clone());
        };
        let size = g.buffer.size();
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("particle readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(&g.buffer, 0, &staging, 0, size);
        queue.submit(Some(encoder.finish()));
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        let data = slice.get_mapped_range().ok()?;
        let slots = bytemuck::cast_slice::<u8, ParticleState>(&data).to_vec();
        drop(data);
        staging.unmap();
        Some(slots)
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
            ParticlePart::Before => s.kind == DrawKind::Mesh,
            ParticlePart::Oit(_) => in_oit(s),
            ParticlePart::After => s.kind != DrawKind::Mesh && !in_oit(s),
        }
    }
}

/// Whether a system's particles join the order-independent passes.
fn in_oit(s: &FrameSystem) -> bool {
    s.kind == DrawKind::Billboard && s.blend == ParticleBlend::Alpha
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

/// Writes `slots` into a CPU system's vertex buffer, made (again) at its size.
fn upload(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffer: &mut Option<wgpu::Buffer>,
    slots: &[ParticleState],
) {
    let bytes: &[u8] = bytemuck::cast_slice(slots);
    if bytes.is_empty() {
        return;
    }
    if buffer
        .as_ref()
        .is_none_or(|b| b.size() != bytes.len() as u64)
    {
        *buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("particles (cpu)"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
    }
    queue.write_buffer(buffer.as_ref().expect("made"), 0, bytes);
}

/// A host's own simulation of a system as the frame draws it: its slots
/// and its sub-emitters' moved from the host's world to the scene's render
/// origin (`origin.rs`).
fn take_hosted(
    cpu: &mut CpuParticles,
    hosted: &CpuParticles,
    space: SimulationSpace,
    origin: Vec3,
) {
    cpu.clone_from(hosted);
    let back = |slots: &mut [ParticleState]| {
        for p in slots {
            for (c, o) in p.pos_age.iter_mut().zip(origin.to_array()) {
                *c -= o;
            }
        }
    };
    if origin != Vec3::ZERO {
        if space == SimulationSpace::World {
            back(&mut cpu.slots);
        }
        for c in &mut cpu.children {
            back(&mut c.slots);
        }
    }
}

/// Sprite-mode particles as sprites (alive slots, in slot order); with
/// `array` (the array and its frames), each from the layer its frame names.
fn push_sprites(
    e: &ParticleEmitter,
    sys: &ParticleSystem,
    slots: &[ParticleState],
    (layer, layers): (i32, u32),
    array: Option<(crate::TextureArrayHandle, u32)>,
    out: &mut Vec<ExtraSprite>,
) {
    let columns = e.sheet.columns.max(1);
    let rows = e.sheet.rows.max(1);
    let blend = match e.blend {
        ParticleBlend::Additive => crate::SpriteBlend::Additive,
        ParticleBlend::Alpha => crate::SpriteBlend::Alpha,
    };
    let z = sys.transform.w_axis.z;
    let frames = array.map_or_else(|| sheet_frames(&e.sheet), |(_, n)| n);
    let soft = e.round || (e.texture.is_none() && array.is_none());
    for p in slots.iter().filter(|p| p.alive()) {
        let (size, color, rotation, frame) = appearance(e, p, frames);
        let mut pos = Vec3::from_slice(&p.pos_age[..3]);
        if e.space == SimulationSpace::Local {
            pos = sys.transform.transform_point3(pos);
        }
        let (col, row) = (frame % columns, frame / columns);
        let (du, dv) = (1.0 / columns as f32, 1.0 / rows as f32);
        let uv_rect = match array {
            Some(_) => [0.0, 0.0, 1.0, 1.0],
            None => [
                col as f32 * du,
                row as f32 * dv,
                (col + 1) as f32 * du,
                (row + 1) as f32 * dv,
            ],
        };
        out.push(ExtraSprite {
            sprite: Sprite {
                texture: if array.is_some() { None } else { e.texture },
                position: Vec3::new(pos.x, pos.y, z),
                size: Vec2::splat(size),
                anchor: Vec2::splat(0.5),
                rotation,
                color,
                uv_rect,
                layer,
                clip: None,
            },
            blend,
            glow: e.glow,
            soft,
            array: array.map(|(a, _)| (a, frame)),
            layers,
        });
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
    format!("{NOISE_WGSL}{COMMON_WGSL}{SIM_WGSL}")
}

/// The particle draw module after the frame group, the light tables and
/// the lighting (`fluid_shared_source`: lit particles).
pub(crate) fn draw_source(frame_wgsl: &str) -> String {
    format!(
        "{frame_wgsl}{}{}{NOISE_WGSL}{COMMON_WGSL}{DRAW_WGSL}",
        crate::oit::OIT_WGSL,
        crate::oit::ARENA_WGSL
    )
}
