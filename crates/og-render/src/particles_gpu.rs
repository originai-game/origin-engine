//! The renderer's side of particles: per-system state kept by id, the
//! compute simulation (full tier), the CPU path's uploads, and the draw.

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
    }
}

struct GpuSystem {
    buffer: wgpu::Buffer,
    points: wgpu::Buffer,
    last_tick: Option<u64>,
    prev_transform: Option<Mat4>,
}

struct SystemState {
    emitter: usize,
    max: u32,
    gpu: Option<GpuSystem>,
    cpu: CpuParticles,
    /// The CPU path's vertex buffer.
    upload: Option<wgpu::Buffer>,
    seen: u64,
}

/// One system's work this frame.
pub(crate) struct FrameSystem {
    id: u64,
    /// Emitter uniform entries: the ticks to simulate, then the draw entry.
    first_entry: u32,
    ticks: u32,
    reset: bool,
    gpu: bool,
    pub blend: ParticleBlend,
    pub texture: Option<crate::TextureHandle>,
    /// View depth of the emitter (draw order: far first).
    depth: f32,
    /// Its three.js layer mask ([`RenderScene::particle_layers`]).
    pub layers: u32,
}

/// What the frame's particles need from the rest of the renderer.
#[derive(Default)]
pub(crate) struct ParticleFrame {
    pub systems: Vec<FrameSystem>,
    /// Sprite-mode particles, as the 2D batch draws them.
    pub sprites: Vec<ExtraSprite>,
    pub any_gpu: bool,
    pub needs_depth: bool,
    /// A system fades against the depth behind it (soft particles): each
    /// view draws its own depth for them (`views.rs`).
    pub soft: bool,
    pub capacity: u32,
    pub cpu_alive: u32,
    pub ticks: u32,
    pub collision_skipped: u32,
}

pub(crate) struct ParticleRenderer {
    sim_layout: wgpu::BindGroupLayout,
    sim: Option<(wgpu::ComputePipeline, wgpu::ComputePipeline)>,
    pub draw_layout: wgpu::BindGroupLayout,
    draw: HashMap<(ParticleBlend, u32), wgpu::RenderPipeline>,
    draw_module: wgpu::ShaderModule,
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
    pub fn new(
        device: &wgpu::Device,
        frame_layout: &wgpu::BindGroupLayout,
        sprite_layout: &wgpu::BindGroupLayout,
        frame_wgsl: &str,
        compute: bool,
    ) -> ParticleRenderer {
        let sim_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle sim"),
            entries: &if compute {
                vec![
                    uniform_entry(0, wgpu::ShaderStages::COMPUTE, true),
                    uniform_entry(1, wgpu::ShaderStages::COMPUTE, false),
                    storage_entry(2, false),
                    storage_entry(3, true),
                    depth_entry(4, wgpu::ShaderStages::COMPUTE),
                    depth_entry(5, wgpu::ShaderStages::COMPUTE),
                ]
            } else {
                vec![]
            },
        });
        let sim = compute.then(|| {
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
                    cache: crate::pipeline_cache::of(device).as_ref(),
                })
            };
            (pipeline("simulate"), pipeline("clear"))
        });
        let draw_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle draw"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT, true),
                uniform_entry(1, wgpu::ShaderStages::VERTEX_FRAGMENT, false),
                depth_entry(2, wgpu::ShaderStages::FRAGMENT),
            ],
        });
        let draw_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("particles"),
            source: wgpu::ShaderSource::Wgsl(draw_source(frame_wgsl).into()),
        });
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
        let gpu = gpu && self.sim.is_some();
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
            });
            if state.emitter != sys.emitter.index() || state.max != e.max_particles {
                *state = SystemState {
                    emitter: sys.emitter.index(),
                    max: e.max_particles,
                    gpu: None,
                    cpu: CpuParticles::default(),
                    upload: None,
                    seen: 0,
                };
            }
            state.seen = self.frame;
            out.capacity += e.max_particles;
            let seed = hash_seed(sys.seed);
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
            if let ParticleRender::Sprite { layer } = e.render {
                // An array emitter's layers are its frames.
                let array = match e.array {
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
                };
                out.ticks += state.cpu.advance(e, sys, tick_rate, points, &[]);
                out.cpu_alive += state.cpu.alive() as u32;
                push_sprites(
                    e,
                    sys,
                    &state.cpu.slots,
                    (layer, layers),
                    array,
                    &mut out.sprites,
                );
                continue;
            }
            let depth = (sys.transform.w_axis.truncate() - eye).dot(forward);
            let textured = e.texture.is_some();
            let collide = gpu && e.collision.is_some() && e.space == SimulationSpace::World;
            // Soft particles fade against the depth behind them, where the
            // frame's quality lets them (RenderQuality::soft_particles).
            let soft = if scene.quality.soft_particles {
                e.soft_distance.max(0.0)
            } else {
                0.0
            };
            out.needs_depth |= soft > 0.0 || collide;
            out.soft |= soft > 0.0;
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
                    points: wgpu::util::DeviceExt::create_buffer_init(
                        device,
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("emitter mesh points"),
                            contents: bytemuck::cast_slice(points),
                            usage: wgpu::BufferUsages::STORAGE,
                        },
                    ),
                    last_tick: None,
                    prev_transform: None,
                });
                let (reset, steps) = plan_ticks(e, sys, g.last_tick, g.prev_transform, tick_rate);
                g.last_tick = Some(sys.tick);
                g.prev_transform = Some(sys.transform);
                let n_points = (points.len() / 2) as u32;
                for s in &steps {
                    entries.push(emitter_gpu(
                        e,
                        s.transform,
                        Some(s),
                        dt,
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
                out.ticks += steps.len() as u32;
                out.any_gpu = true;
                out.systems.push(FrameSystem {
                    id: sys.id,
                    first_entry,
                    ticks: steps.len() as u32,
                    reset,
                    gpu: true,
                    blend: e.blend,
                    texture: e.texture,
                    depth,
                    layers,
                });
            } else {
                out.ticks += state.cpu.advance(e, sys, tick_rate, points, colliders);
                out.cpu_alive += state.cpu.alive() as u32;
                let bytes: &[u8] = bytemuck::cast_slice(&state.cpu.slots);
                let upload = state.upload.get_or_insert_with(|| {
                    device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("particles (cpu)"),
                        size: bytes.len() as u64,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    })
                });
                queue.write_buffer(upload, 0, bytes);
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
                    gpu: false,
                    blend: e.blend,
                    texture: e.texture,
                    depth,
                    layers,
                });
            }
        }
        let frame = self.frame;
        self.systems.retain(|_, s| s.seen == frame);
        // Far systems first; additive after alpha within the same depth.
        out.systems
            .sort_by(|a, b| b.depth.total_cmp(&a.depth).then(a.id.cmp(&b.id)));
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

    /// Records the frame's compute dispatches (one per simulated tick).
    pub fn simulate(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        frame: &ParticleFrame,
        depth: &wgpu::TextureView,
        gbuffer: &wgpu::TextureView,
    ) {
        let Some((simulate, clear)) = &self.sim else {
            return;
        };
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
    pub fn draw_group(&self, device: &wgpu::Device, depth: &wgpu::TextureView) -> wgpu::BindGroup {
        self.draw_group_of(device, &self.view_buffer, depth)
    }

    /// The draw bind group of a camera whose [`ParticleViewGpu`] is in
    /// `view` (a secondary camera's, `views.rs`).
    pub fn draw_group_of(
        &self,
        device: &wgpu::Device,
        view: &wgpu::Buffer,
        depth: &wgpu::TextureView,
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
            ],
        })
    }

    /// Makes the draw pipelines the frame needs (by blend and sample count).
    pub fn ensure_pipelines(&mut self, device: &wgpu::Device, frame: &ParticleFrame, samples: u32) {
        for s in &frame.systems {
            let key = (s.blend, samples);
            if self.draw.contains_key(&key) {
                continue;
            }
            let blend = match s.blend {
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
            let buffers = [Some(PARTICLE_LAYOUT)];
            let p = crate::hdr_pipeline(
                device,
                crate::HdrPipeline {
                    label: "particles",
                    layout: &self.draw_pl,
                    module: &self.draw_module,
                    vs: "vs",
                    fs: "fs",
                    buffers: &buffers,
                    cull: None,
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: Some(blend),
                    samples,
                },
            );
            self.draw.insert(key, p);
        }
        let _ = HDR_FORMAT;
    }

    /// Draws the frame's billboard systems on a camera's layers (`mask`),
    /// far to near. Returns the draws.
    #[allow(clippy::too_many_arguments)]
    pub fn draw<'b>(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        frame: &ParticleFrame,
        samples: u32,
        group: &wgpu::BindGroup,
        mask: u32,
        texture_group: impl Fn(Option<crate::TextureHandle>) -> &'b wgpu::BindGroup,
    ) -> u32 {
        let mut draws = 0;
        for s in frame.systems.iter().filter(|s| s.layers & mask != 0) {
            let Some(state) = self.systems.get(&s.id) else {
                continue;
            };
            let buffer = if s.gpu {
                &state.gpu.as_ref().expect("gpu system").buffer
            } else {
                state.upload.as_ref().expect("uploaded")
            };
            pass.set_pipeline(&self.draw[&(s.blend, samples)]);
            pass.set_bind_group(1, texture_group(s.texture), &[]);
            let draw_entry = s.first_entry + s.ticks;
            pass.set_bind_group(2, group, &[(draw_entry as u64 * EMITTER_STRIDE) as u32]);
            pass.set_vertex_buffer(0, buffer.slice(..));
            pass.draw(0..6, 0..state.max);
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

/// The particle simulation's module (compute: full tier only).
pub(crate) fn sim_source() -> String {
    format!("{NOISE_WGSL}{COMMON_WGSL}{SIM_WGSL}")
}

/// The particle draw module after the frame group.
pub(crate) fn draw_source(frame_wgsl: &str) -> String {
    format!("{frame_wgsl}{NOISE_WGSL}{COMMON_WGSL}{DRAW_WGSL}")
}
