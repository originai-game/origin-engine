//! Visual fluids (og-physics' og.Fluid3D with SIMULATION_VISUAL): MLS-MPM
//! on the GPU for looks only, nothing read back, drawn by the screen-space
//! fluid ([`crate::Fluid3d`]). Ported from matsuoka-601/WebGPU-Ocean
//! (MIT); `shaders/visual_fluid.wgsl` and third_party/webgpu-ocean/ have
//! the method, the licence and the changes.
//!
//! The host hands in, each frame, where the fluid is and how it is set
//! ([`VisualFluid`]); the renderer owns its particles. Ticks drive it as
//! they drive visual bodies: a frame steps the ticks since the one it last
//! drew (up to [`crate::fields::MAX_CATCH_UP`]); a world that went
//! back, jumped further, or a fluid whose spawn changed starts again from
//! its spawn box (no preroll: the box falls as a dam breaks).
//!
//! Units: the solver works in grid cells and in upstream's time unit of
//! which one step is 0.2; a step is `1 / (tick rate · substeps)` seconds,
//! so gravity and the emitter's velocity are scaled into it, and enough
//! substeps are taken that gravity stays within upstream's stable range
//! and the fastest fall stays under the shader's speed clamp.
//!
//! Bindings: one compute-only layout of six storage buffers (the header
//! and its tick records, the frame's colliders, the particles, the grid,
//! the tick cursor, and the screen-space fluid's instance buffer, which
//! the particles are written into after the CPU fluids' instances). No
//! draw pass or lit layout changes.

use crate::{Deferred, SceneCollider};
use glam::Vec3;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) const WGSL: &str = include_str!("../../../shaders/visual_fluid.wgsl");
const COLLIDERS: &str = wgsl!("scene_colliders.wgsl");

/// The solver's source: the colliders, then the fluid.
pub(crate) fn source() -> String {
    format!("{COLLIDERS}\n{WGSL}")
}

/// The most particles a visual fluid has.
pub const MAX_PARTICLES: u32 = 262_144;
/// The most grid cells (each 16 bytes); a larger container takes larger cells.
const MAX_CELLS: f32 = 2_097_152.0;
/// Particles a cell holds at rest, at most and at least: the cell size is
/// kept between them (fewer and the grid is too fine for them to meet;
/// more and a cell's mass leaves the fixed-point range).
const PER_CELL: (f32, f32) = (2.0, 8.0);
/// Upstream's gravity, cells per time unit², the largest it is stable at.
const MAX_GRAVITY: f32 = 0.3;
/// The fastest a particle moves, cells per time unit (`MAX_SPEED` in the
/// shader, which clamps there).
const MAX_SPEED: f32 = 3.0;
/// Upstream's step, in its time unit.
const STEP: f32 = 0.2;

/// A fluid the renderer simulates (og.Fluid3D's SIMULATION_VISUAL), in
/// the world this frame. Lengths in metres, rates per second.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualFluid {
    /// Which fluid it is across frames (the entity).
    pub key: u64,
    /// Particles spawned in the box `centre` ± `size / 2`.
    pub count: u32,
    pub centre: Vec3,
    pub size: Vec3,
    /// The container around `centre` (0 on an axis: three times `size`).
    pub bounds: Vec3,
    /// The grid's cell, about the particles' interaction radius.
    pub cell: f32,
    /// The pool: the most particles alive (`count` and the emitter's).
    pub max_count: u32,
    /// Multipliers of the solver's pressure and viscosity (1: water).
    pub stiffness: f32,
    pub viscosity: f32,
    pub gravity: Vec3,
    /// Steps per tick, at least (more when gravity needs them).
    pub substeps: u32,
    pub seed: u32,
    /// The emitter: particles a second, where, how spread and how fast.
    pub emit_rate: f32,
    pub emit_at: Vec3,
    pub emit_radius: f32,
    pub emit_velocity: Vec3,
    /// The sink box (min, max): particles entering it go back to the
    /// emitter, or out without one.
    pub sink: Option<(Vec3, Vec3)>,
}

impl Default for VisualFluid {
    fn default() -> Self {
        VisualFluid {
            key: 0,
            count: 2000,
            centre: Vec3::ZERO,
            size: Vec3::new(2.0, 1.0, 2.0),
            bounds: Vec3::ZERO,
            cell: 0.2,
            max_count: 8192,
            stiffness: 1.0,
            viscosity: 1.0,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            substeps: 2,
            seed: 0,
            emit_rate: 0.0,
            emit_at: Vec3::ZERO,
            emit_radius: 0.1,
            emit_velocity: Vec3::ZERO,
            sink: None,
        }
    }
}

/// Header words (`shaders/visual_fluid.wgsl`).
mod h {
    pub const N: usize = 0;
    pub const COUNT: usize = 1;
    pub const RESET: usize = 2;
    pub const SEED: usize = 3;
    pub const DIMS: usize = 4;
    pub const H: usize = 7;
    pub const ORIGIN: usize = 8;
    pub const GRAVITY: usize = 11;
    pub const STIFFNESS: usize = 14;
    pub const VISCOSITY: usize = 15;
    pub const REST_DENSITY: usize = 16;
    pub const FIXED: usize = 17;
    pub const WALL_MIN: usize = 18;
    pub const WALL_MAX: usize = 21;
    pub const SPAWN_AT: usize = 24;
    pub const SPAWN_STEP: usize = 27;
    pub const LATTICE: usize = 30;
    pub const EMIT_AT: usize = 32;
    pub const EMIT_RADIUS: usize = 35;
    pub const EMIT_V: usize = 36;
    pub const EMITTING: usize = 39;
    pub const SINK_MIN: usize = 40;
    pub const SINK_MAX: usize = 43;
    pub const SINKING: usize = 46;
    pub const COLLIDER_LIST: usize = 47;
    pub const RADIUS: usize = 48;
    pub const DRAW_RADIUS: usize = 49;
    pub const COLOR: usize = 50;
    pub const DENSITY_K: usize = 53;
    pub const OUT: usize = 54;
    pub const FOAM_KEEP: usize = 55;
    pub const METRE_SPEED: usize = 56;
    pub const SUBSTEPS: usize = 57;
    pub const ALIVE: usize = 58;
    pub const LEN: usize = ALIVE + crate::fields::MAX_CATCH_UP as usize + 1;
}

/// Where a fluid's grid lies and how its time runs: everything the header
/// carries that does not change from frame to frame.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Grid {
    /// Cells on each axis (three of padding each side).
    pub dims: [u32; 3],
    /// Metres a cell.
    pub cell: f32,
    /// Cell (0, 0, 0)'s corner in the world.
    pub origin: Vec3,
    /// Steps a tick.
    pub substeps: u32,
    /// Seconds of upstream's time unit.
    pub unit: f32,
    /// The particles in the pool.
    pub n: u32,
    /// The spawn lattice: its first point, spacing, columns and depth.
    pub spawn_at: Vec3,
    pub spawn_step: Vec3,
    pub lattice: [u32; 2],
    /// Particles a cell at rest.
    pub rest_density: f32,
}

impl Grid {
    pub(crate) fn of(f: &VisualFluid, tick_dt: f32) -> Grid {
        let size = f.size.max(Vec3::splat(0.01));
        let container = Vec3::select(f.bounds.cmpgt(Vec3::ZERO), f.bounds, size * 3.0);
        let count = f.count.min(MAX_PARTICLES);
        let box_volume = size.x * size.y * size.z;
        let mut cell = f.cell.max(0.01);
        if count > 0 {
            let at = |per_cell: f32| (per_cell * box_volume / count as f32).cbrt();
            cell = cell.clamp(at(PER_CELL.0), at(PER_CELL.1));
        }
        let cells = |c: f32| (container / c).ceil() + Vec3::splat(6.0);
        let total = |c: f32| {
            let d = cells(c);
            d.x * d.y * d.z
        };
        // The padding makes the count no pure cube of the cell: grow it
        // until the grid fits.
        while total(cell) > MAX_CELLS {
            cell *= (total(cell) / MAX_CELLS).cbrt().max(1.01);
        }
        let d = cells(cell);
        let dims = [d.x as u32, d.y as u32, d.z as u32];
        let origin = f.centre - container * 0.5 - Vec3::splat(3.0 * cell);
        // Enough steps that gravity stays within upstream's stable range.
        let g = f.gravity.length() / cell;
        let mut substeps = f.substeps.clamp(1, 16);
        let unit = |s: u32| tick_dt / s as f32 / STEP;
        // And enough that the fastest the fluid can fall (from its emitter
        // or the top of its block to the bottom of the block, plus the
        // emitter's speed) stays under the shader's clamp, which would
        // otherwise hold falling water at a terminal speed and keep a pour
        // from thinning as it speeds up.
        let fastest = fastest_fall(f, size) * 1.1 / cell;
        while substeps < 16
            && (g * unit(substeps).powi(2) > MAX_GRAVITY || fastest * unit(substeps) > MAX_SPEED)
        {
            substeps += 1;
        }
        let (lattice, spawn_step, rest_density) = if count > 0 {
            let spacing = (box_volume / count as f32).cbrt();
            let cols = ((size.x / spacing).round() as u32).max(1);
            let deep = ((size.z / spacing).round() as u32).max(1);
            let rows = count.div_ceil(cols * deep).max(1);
            let step = Vec3::new(
                size.x / cols as f32,
                size.y / rows as f32,
                size.z / deep as f32,
            );
            let per_cell = count as f32 * cell.powi(3) / box_volume;
            ([cols, deep], step, per_cell)
        } else {
            ([1, 1], Vec3::splat(cell * 0.5), 4.0)
        };
        Grid {
            dims,
            cell,
            origin,
            substeps,
            unit: unit(substeps),
            n: f.max_count.max(count).clamp(1, MAX_PARTICLES),
            spawn_at: f.centre - size * 0.5 + spawn_step * 0.5,
            spawn_step,
            lattice,
            rest_density,
        }
    }

    fn cells(&self) -> u32 {
        self.dims.iter().product()
    }
}

/// How fast (m/s) the fluid can fall: from the higher of its emitter and
/// its block's top to its block's bottom, along gravity, from the
/// emitter's speed.
fn fastest_fall(f: &VisualFluid, size: Vec3) -> f32 {
    let g = f.gravity.length();
    if !(g.is_finite() && g > 0.0) {
        return 0.0;
    }
    let up = -f.gravity / g;
    let half = (size * up).abs().dot(Vec3::ONE) * 0.5;
    let base = f.centre.dot(up);
    let emitting = f.emit_rate > 0.0 && f.emit_velocity.is_finite() && f.emit_at.is_finite();
    let mut top = base + half;
    let mut v0 = 0.0f32;
    if emitting {
        top = top.max(f.emit_at.dot(up));
        v0 = f.emit_velocity.length();
    }
    (v0 * v0 + 2.0 * g * (top - (base - half)).max(0.0)).sqrt()
}

/// The particles alive `ticks` after the fluid started: its spawn and
/// what its emitter has added, at most the pool.
pub(crate) fn alive(f: &VisualFluid, n: u32, ticks: u64, tick_dt: f32) -> u32 {
    let emitted = if f.emit_rate > 0.0 && f.emit_rate.is_finite() {
        (f.emit_rate.min(20000.0) as f64 * ticks as f64 * tick_dt as f64).floor() as u64
    } else {
        0
    };
    (u64::from(f.count.min(MAX_PARTICLES)) + emitted).min(u64::from(n)) as u32
}

/// What a frame does with a fluid: start again, and the ticks it steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Plan {
    pub reset: bool,
    pub steps: u32,
}

/// The frame's plan for a fluid last stepped to `last` (`None`: new) at
/// `tick`: the ticks since, or one tick from its spawn (no preroll).
pub(crate) fn plan(last: Option<u64>, tick: u64) -> Plan {
    let steps = crate::fields::catch_up::catch_up(last, tick, 1);
    Plan {
        reset: steps.restart,
        steps: steps.count as u32,
    }
}

/// The header for this frame.
#[allow(clippy::too_many_arguments)]
pub(crate) fn header(
    f: &VisualFluid,
    g: &Grid,
    plan: Plan,
    elapsed: u64,
    tick_dt: f32,
    list: u32,
    draw: (f32, Vec3, f32),
    out: u32,
) -> Vec<u32> {
    let mut w = vec![0u32; h::LEN];
    let put = |w: &mut [u32], at: usize, v: f32| w[at] = v.to_bits();
    let put3 = |w: &mut [u32], at: usize, v: Vec3| {
        for k in 0..3 {
            w[at + k] = v[k].to_bits();
        }
    };
    let cell = g.cell;
    let unit = g.unit;
    w[h::N] = g.n;
    w[h::COUNT] = f.count.min(g.n);
    w[h::RESET] = u32::from(plan.reset);
    w[h::SEED] = f.seed;
    w[h::DIMS..h::DIMS + 3].copy_from_slice(&g.dims);
    put(&mut w, h::H, cell);
    put3(&mut w, h::ORIGIN, g.origin);
    put3(&mut w, h::GRAVITY, f.gravity / cell * unit * unit);
    put(&mut w, h::STIFFNESS, 3.0 * f.stiffness.max(0.0));
    put(&mut w, h::VISCOSITY, 0.1 * f.viscosity.max(0.0));
    put(&mut w, h::REST_DENSITY, g.rest_density.max(0.5));
    put(&mut w, h::FIXED, 1e7);
    let walls = Vec3::from(g.dims.map(|d| d as f32));
    put3(&mut w, h::WALL_MIN, Vec3::splat(3.0));
    put3(&mut w, h::WALL_MAX, walls - 4.0);
    put3(&mut w, h::SPAWN_AT, g.spawn_at);
    put3(&mut w, h::SPAWN_STEP, g.spawn_step);
    w[h::LATTICE] = g.lattice[0];
    w[h::LATTICE + 1] = g.lattice[1];
    put3(&mut w, h::EMIT_AT, f.emit_at);
    put(&mut w, h::EMIT_RADIUS, f.emit_radius.max(0.0));
    let v = if f.emit_velocity.is_finite() {
        f.emit_velocity
    } else {
        Vec3::ZERO
    };
    put3(&mut w, h::EMIT_V, v * unit / cell);
    w[h::EMITTING] = u32::from(f.emit_rate > 0.0);
    if let Some((lo, hi)) = f.sink {
        put3(&mut w, h::SINK_MIN, lo);
        put3(&mut w, h::SINK_MAX, hi);
        w[h::SINKING] = u32::from(lo.cmplt(hi).all());
    }
    w[h::COLLIDER_LIST] = list;
    put(&mut w, h::RADIUS, cell * 0.3);
    put(&mut w, h::DRAW_RADIUS, draw.0);
    put3(&mut w, h::COLOR, draw.1);
    put(&mut w, h::DENSITY_K, draw.2);
    w[h::OUT] = out;
    // Foam fades with a time constant of 0.8 s.
    let step = tick_dt / g.substeps.max(1) as f32;
    put(&mut w, h::FOAM_KEEP, (-step / 0.8).exp());
    put(&mut w, h::METRE_SPEED, unit / cell);
    w[h::SUBSTEPS] = g.substeps;
    // The alive counts: before the frame's first tick, then after each.
    let before = elapsed.saturating_sub(u64::from(plan.steps));
    for k in 0..=plan.steps as usize {
        w[h::ALIVE + k] = alive(f, g.n, before + k as u64, tick_dt);
    }
    w
}

// ---- GPU ----------------------------------------------------------------------

const ENTRIES: [&str; 8] = [
    "spawn",
    "clear_grid",
    "p2g_1",
    "p2g_2",
    "update_grid",
    "g2p",
    "next_tick",
    "draw",
];

/// One fluid's buffers and clock.
struct Sim {
    grid: Grid,
    shape: VisualFluid,
    head: wgpu::Buffer,
    particles: wgpu::Buffer,
    cells: wgpu::Buffer,
    cursor: wgpu::Buffer,
    last: Option<u64>,
    /// The tick it started at.
    start: u64,
}

impl Sim {
    fn bytes(&self) -> u64 {
        self.head.size() + self.particles.size() + self.cells.size()
    }
}

/// The frame's work for one fluid, after `prepare`.
struct Job {
    key: u64,
    plan: Plan,
    group: wgpu::BindGroup,
}

pub(crate) struct VisualFluids {
    layout: wgpu::BindGroupLayout,
    pipelines: Vec<Deferred<wgpu::ComputePipeline>>,
    colliders: wgpu::Buffer,
    sims: HashMap<u64, Sim>,
    jobs: Vec<Job>,
}

/// Whether two fluids spawn the same: a change starts it again.
fn same_spawn(a: &VisualFluid, b: &VisualFluid) -> bool {
    a.count == b.count
        && a.seed == b.seed
        && a.size == b.size
        && a.bounds == b.bounds
        && a.cell == b.cell
        && a.max_count == b.max_count
}

impl VisualFluids {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let entries: Vec<_> = (0..6)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: binding < 2,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("visual fluid"),
            entries: &entries,
        });
        let module = Arc::new(std::sync::OnceLock::<wgpu::ShaderModule>::new());
        let pipelines = ENTRIES
            .iter()
            .map(|&entry| {
                let (device, layout, module) = (device.clone(), layout.clone(), module.clone());
                Deferred::new(move || {
                    let module = module.get_or_init(|| {
                        device.create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("visual fluid"),
                            source: wgpu::ShaderSource::Wgsl(source().into()),
                        })
                    });
                    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some(entry),
                        layout: Some(&device.create_pipeline_layout(
                            &wgpu::PipelineLayoutDescriptor {
                                label: Some("visual fluid"),
                                bind_group_layouts: &[Some(&layout)],
                                immediate_size: 0,
                            },
                        )),
                        module,
                        entry_point: Some(entry),
                        compilation_options: Default::default(),
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    })
                })
            })
            .collect();
        VisualFluids {
            layout,
            pipelines,
            colliders: collider_buffer(device, 4096),
            sims: HashMap::new(),
            jobs: Vec::new(),
        }
    }

    /// GPU memory its fluids hold.
    pub(crate) fn bytes(&self) -> u64 {
        self.sims.values().map(Sim::bytes).sum::<u64>() + self.colliders.size()
    }

    /// Plans the frame's fluids (`slots`: each with its draw and the first
    /// of its instances) at `tick`, writing their headers and binding them
    /// to the instance buffer `instances`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        slots: &[crate::fluid3d::VisualSlot],
        colliders: &[SceneCollider],
        tick: u64,
        tick_rate: f32,
        instances: &wgpu::Buffer,
    ) {
        self.jobs.clear();
        self.sims
            .retain(|key, _| slots.iter().any(|s| s.fluid.key == *key));
        if slots.is_empty() {
            return;
        }
        let tick_dt = if tick_rate > 0.0 {
            1.0 / tick_rate
        } else {
            1.0 / 60.0
        };
        let grids: Vec<Grid> = slots.iter().map(|s| Grid::of(&s.fluid, tick_dt)).collect();
        let bounds: Vec<(Vec3, Vec3)> = grids
            .iter()
            .map(|g| {
                let size = Vec3::from(g.dims.map(|d| d as f32)) * g.cell;
                (g.origin, g.origin + size)
            })
            .collect();
        let (data, lists) = crate::scene_colliders::pack(colliders, &bounds);
        let bytes: &[u8] = bytemuck::cast_slice(&data);
        if bytes.len() as u64 > self.colliders.size() {
            self.colliders = collider_buffer(device, (bytes.len() as u64).next_power_of_two());
        }
        queue.write_buffer(&self.colliders, 0, bytes);
        for ((slot, grid), list) in slots.iter().zip(grids).zip(lists) {
            let f = &slot.fluid;
            let sim = self
                .sims
                .entry(f.key)
                .or_insert_with(|| make(device, &grid, f));
            if sim.grid.n != grid.n || sim.grid.dims != grid.dims {
                *sim = make(device, &grid, f);
            }
            let fresh = !same_spawn(&sim.shape, f) || sim.grid != grid;
            let plan = plan(if fresh { None } else { sim.last }, tick);
            if plan.reset {
                sim.start = tick;
            }
            sim.last = Some(tick);
            sim.grid = grid;
            sim.shape = (**f).clone();
            let elapsed = tick.saturating_sub(sim.start) + u64::from(plan.reset);
            let words = header(
                f,
                &sim.grid,
                plan,
                elapsed,
                tick_dt,
                list,
                (slot.radius, slot.color, slot.density),
                slot.first,
            );
            queue.write_buffer(&sim.head, 0, bytemuck::cast_slice(&words));
            queue.write_buffer(&sim.cursor, 0, bytemuck::bytes_of(&0u32));
            let entries: Vec<_> = [
                &sim.head,
                &self.colliders,
                &sim.particles,
                &sim.cells,
                &sim.cursor,
                instances,
            ]
            .into_iter()
            .enumerate()
            .map(|(i, b)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: b.as_entire_binding(),
            })
            .collect();
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("visual fluid"),
                layout: &self.layout,
                entries: &entries,
            });
            self.jobs.push(Job {
                key: f.key,
                plan,
                group,
            });
        }
    }

    /// Records the frame's ticks of every fluid, then writes its instances.
    pub(crate) fn encode(&self, encoder: &mut crate::Encoder) {
        if self.jobs.is_empty() {
            return;
        }
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("visual fluid"),
            timestamp_writes: None,
        });
        let groups = |n: u32| n.div_ceil(64);
        for job in &self.jobs {
            let g = &self.sims[&job.key].grid;
            let (particles, cells) = (groups(g.n), groups(g.cells()));
            pass.set_bind_group(0, &job.group, &[]);
            let mut run = |entry: usize, count: u32| {
                pass.set_pipeline(self.pipelines[entry].get());
                pass.dispatch_workgroups(count, 1, 1);
            };
            for _ in 0..job.plan.steps {
                run(0, particles);
                for _ in 0..g.substeps {
                    run(1, cells);
                    run(2, particles);
                    run(3, particles);
                    run(4, cells);
                    run(5, particles);
                }
                run(6, 1);
            }
            run(7, particles);
        }
    }
}

fn collider_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("visual fluid colliders"),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn make(device: &wgpu::Device, grid: &Grid, f: &VisualFluid) -> Sim {
    let buffer = |label, size: u64| {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    };
    Sim {
        grid: grid.clone(),
        shape: f.clone(),
        head: buffer("visual fluid header", h::LEN as u64 * 4),
        particles: buffer("visual fluid particles", u64::from(grid.n) * 80),
        cells: buffer("visual fluid grid", u64::from(grid.cells()) * 16),
        cursor: buffer("visual fluid cursor", 16),
        last: None,
        start: 0,
    }
}

/// The fallback where the device does not simulate it: the spawn box's
/// lattice, still (what the GPU spawns, without its jitter).
pub fn at_rest(f: &VisualFluid) -> Vec<Vec3> {
    let g = Grid::of(f, 1.0 / 60.0);
    let [cols, deep] = g.lattice;
    (0..f.count.min(g.n))
        .map(|i| {
            let at = Vec3::new(
                (i % cols) as f32,
                (i / (cols * deep)) as f32,
                ((i / cols) % deep) as f32,
            );
            g.spawn_at + at * g.spawn_step
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_high_pour_gets_the_steps_to_land_faster_than_the_clamp_allowed() {
        // A spout 1.75 m over a basin: its water lands at about 6 m/s.
        // Two steps a tick held every particle under 3.6 m/s, so the pour
        // fell at a terminal speed and never thinned.
        let f = VisualFluid {
            count: 60_000,
            centre: Vec3::new(-1.0, 0.45, 0.0),
            size: Vec3::new(1.0, 0.9, 2.0),
            bounds: Vec3::new(5.0, 3.0, 2.0),
            cell: 0.05,
            emit_rate: 1500.0,
            emit_at: Vec3::new(1.05, 1.75, 0.3),
            emit_velocity: Vec3::new(-0.5, -1.5, 0.0),
            ..VisualFluid::default()
        };
        let g = Grid::of(&f, 1.0 / 60.0);
        let limit = MAX_SPEED * g.cell / g.unit;
        let landing = (1.58f32.powi(2) + 2.0 * 9.81 * 1.75).sqrt();
        assert!(limit > landing, "{limit} m/s at {} steps", g.substeps);
        // Without an emitter only the block's own fall counts.
        let still = Grid::of(
            &VisualFluid {
                emit_rate: 0.0,
                ..f.clone()
            },
            1.0 / 60.0,
        );
        assert!(
            still.substeps < g.substeps,
            "{} {}",
            still.substeps,
            g.substeps
        );
    }

    #[test]
    fn the_grid_holds_the_container_in_cells_of_about_the_radius() {
        let f = VisualFluid {
            count: 8000,
            size: Vec3::new(2.0, 1.0, 2.0),
            bounds: Vec3::new(4.0, 3.0, 2.0),
            cell: 0.1,
            ..VisualFluid::default()
        };
        let g = Grid::of(&f, 1.0 / 60.0);
        // 8000 particles in 4 m³: 0.1 m cells hold 2 at rest, the least.
        assert!((g.cell - 0.1).abs() < 1e-4, "{}", g.cell);
        assert_eq!(g.dims, [46, 36, 26]);
        assert!((g.origin - Vec3::new(-2.3, -1.8, -1.3)).length() < 1e-4);
        // Gravity of 98 cells/s² keeps within 0.3 a time unit².
        let gravity = 9.81 / g.cell * g.unit * g.unit;
        assert!(gravity <= MAX_GRAVITY, "{gravity} at {} steps", g.substeps);
        assert!(g.substeps >= 2);
        // A sparse fluid gets larger cells; a huge container fewer of them.
        let sparse = Grid::of(
            &VisualFluid {
                count: 100,
                ..f.clone()
            },
            1.0 / 60.0,
        );
        assert!(sparse.cell > 0.3, "{}", sparse.cell);
        let huge = Grid::of(
            &VisualFluid {
                bounds: Vec3::splat(400.0),
                ..f
            },
            1.0 / 60.0,
        );
        assert!(huge.dims.iter().product::<u32>() as f32 <= MAX_CELLS);
    }

    #[test]
    fn the_emitter_adds_its_rate_up_to_the_pool() {
        let f = VisualFluid {
            count: 100,
            max_count: 400,
            emit_rate: 120.0,
            ..VisualFluid::default()
        };
        assert_eq!(alive(&f, 400, 0, 1.0 / 60.0), 100);
        assert_eq!(alive(&f, 400, 30, 1.0 / 60.0), 160);
        assert_eq!(alive(&f, 400, 600, 1.0 / 60.0), 400);
        // The frame's records: before its first tick, then after each.
        let g = Grid::of(&f, 1.0 / 60.0);
        let w = header(
            &f,
            &g,
            Plan {
                reset: false,
                steps: 2,
            },
            31,
            1.0 / 60.0,
            0,
            (0.1, Vec3::ONE, 1.0),
            5,
        );
        assert_eq!(&w[h::ALIVE..h::ALIVE + 3], &[158, 160, 162]);
        assert_eq!(w[h::OUT], 5);
    }
}
