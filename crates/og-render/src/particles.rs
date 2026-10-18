//! Particles: one emitter description ([`ParticleEmitter`], an asset added
//! with [`crate::RenderScene::add_emitter`]) and per-frame systems
//! ([`ParticleSystem`]) that place it in the world with a seed and a clock.
//!
//! **Deterministic, visual only.** A system's particles are a function of
//! its emitter, seed, start tick, transform and the ticks it was rendered
//! at: every tick is simulated once with a fixed step (`1 / tick_rate`), the
//! particles spawned in a tick and their random values come from the spawn
//! schedule and a hash of (seed, particle index), never from the frame rate
//! or a clock. A renderer that skipped frames catches up tick by tick; a
//! jump back in time or further than the emitter's longest lifetime
//! replays that lifetime from empty (which is exact while the emitter
//! does not move: older particles are dead either way). Particles never
//! feed back into the simulation; nothing reads them but the renderer.
//!
//! **Two paths, one description.** On the full tier the simulation runs in
//! a compute shader (`shaders/particles_sim.wgsl`) into a storage buffer the
//! draw reads as instance data; on the no-compute tier (WebGL2), or with
//! [`ParticleSimulation::Cpu`], [`CpuParticles`] runs the same steps in Rust
//! and uploads the state. Sprite-mode emitters ([`ParticleRender::Sprite`])
//! always simulate on the CPU and draw through the sprite batch.

use crate::scene::{MeshData, MeshHandle, TextureHandle};
use glam::{Mat4, Vec3, Vec4};

/// The most keys a [`Curve`] or [`ColorCurve`] can have.
pub const MAX_CURVE_KEYS: usize = 8;
/// Points sampled from a mesh for [`EmitterShape::Mesh`].
pub const MESH_SAMPLE_POINTS: usize = 512;

/// A small keyframe table over a particle's life (0 at birth, 1 at death),
/// linear between keys, constant beyond the ends. At most
/// [`MAX_CURVE_KEYS`] keys, sorted by time.
#[derive(Clone, Debug, PartialEq)]
pub struct Curve {
    pub keys: Vec<(f32, f32)>,
}

impl Curve {
    pub fn new(keys: &[(f32, f32)]) -> Curve {
        Curve {
            keys: keys.to_vec(),
        }
    }

    pub fn constant(v: f32) -> Curve {
        Curve::new(&[(0.0, v)])
    }

    /// From `a` at birth to `b` at death.
    pub fn linear(a: f32, b: f32) -> Curve {
        Curve::new(&[(0.0, a), (1.0, b)])
    }

    pub fn eval(&self, t: f32) -> f32 {
        eval_keys(&self.keys, t, |k| k.0, |k| k.1, |a, b, f| a + (b - a) * f)
    }
}

impl Default for Curve {
    fn default() -> Self {
        Curve::constant(1.0)
    }
}

/// A colour keyframe table over a particle's life: linear RGBA (alpha is
/// opacity), at most [`MAX_CURVE_KEYS`] keys.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorCurve {
    pub keys: Vec<(f32, [f32; 4])>,
}

impl ColorCurve {
    pub fn new(keys: &[(f32, [f32; 4])]) -> ColorCurve {
        ColorCurve {
            keys: keys.to_vec(),
        }
    }

    pub fn constant(c: [f32; 4]) -> ColorCurve {
        ColorCurve::new(&[(0.0, c)])
    }

    pub fn eval(&self, t: f32) -> [f32; 4] {
        let v = eval_keys(
            &self.keys,
            t,
            |k| k.0,
            |k| Vec4::from(k.1),
            |a, b, f| a + (b - a) * f,
        );
        v.to_array()
    }
}

impl Default for ColorCurve {
    /// White, fading out over the last quarter of life.
    fn default() -> Self {
        ColorCurve::new(&[
            (0.0, [1.0; 4]),
            (0.75, [1.0; 4]),
            (1.0, [1.0, 1.0, 1.0, 0.0]),
        ])
    }
}

fn eval_keys<K, V: Copy>(
    keys: &[K],
    t: f32,
    time: impl Fn(&K) -> f32,
    value: impl Fn(&K) -> V,
    lerp: impl Fn(V, V, f32) -> V,
) -> V {
    let n = keys.len().min(MAX_CURVE_KEYS);
    let mut v = value(&keys[0]);
    for i in 1..n {
        let (t0, t1) = (time(&keys[i - 1]), time(&keys[i]));
        if t >= t1 {
            v = value(&keys[i]);
        } else if t > t0 {
            v = lerp(
                value(&keys[i - 1]),
                value(&keys[i]),
                (t - t0) / (t1 - t0).max(1e-6),
            );
            break;
        } else {
            break;
        }
    }
    v
}

/// Where particles are born, in the emitter's local space, and which way
/// they start moving (before [`ParticleEmitter::spread_degrees`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EmitterShape {
    /// The origin, moving along +y (spread 180° makes it omnidirectional).
    Point,
    /// In a ball (or on its surface), moving outwards.
    Sphere { radius: f32, surface: bool },
    /// From a disc of `radius` in the xz plane, moving within
    /// `angle_degrees` of +y (a fountain, a thruster).
    Cone { angle_degrees: f32, radius: f32 },
    /// Anywhere in a box, moving along +y (rain falls with negative speed
    /// or gravity).
    Box { half_extents: Vec3 },
    /// On the surface of a mesh of the scene (area weighted), moving along
    /// its normals.
    Mesh { mesh: MeshHandle },
}

/// Particles emitted at once: `count` at `time` seconds into each emission
/// cycle, repeated `cycles` times `interval` apart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Burst {
    pub time: f32,
    pub count: u32,
    pub cycles: u32,
    pub interval: f32,
}

impl Burst {
    pub fn once(time: f32, count: u32) -> Burst {
        Burst {
            time,
            count,
            cycles: 1,
            interval: 0.0,
        }
    }
}

/// Turbulence: smooth 3D value noise added to the velocity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleNoise {
    /// Acceleration in units per second².
    pub strength: f32,
    /// Spatial frequency (cells per unit).
    pub frequency: f32,
    /// How fast the field moves through time (cells per second).
    pub scroll: f32,
}

impl Default for ParticleNoise {
    fn default() -> Self {
        ParticleNoise {
            strength: 0.0,
            frequency: 0.5,
            scroll: 0.3,
        }
    }
}

/// How a sprite sheet animates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SheetMode {
    /// One frame for the whole life (the first).
    #[default]
    Off,
    /// Through all frames `cycles` times over the life.
    OverLife { cycles: f32 },
    /// At a fixed rate.
    Fps(f32),
    /// One random frame per particle.
    Random,
}

/// A grid of frames in the particle texture, left to right, top to bottom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteSheet {
    pub columns: u32,
    pub rows: u32,
    /// Frames used (0: columns × rows).
    pub frames: u32,
    pub mode: SheetMode,
}

impl Default for SpriteSheet {
    fn default() -> Self {
        SpriteSheet {
            columns: 1,
            rows: 1,
            frames: 0,
            mode: SheetMode::Off,
        }
    }
}

/// How particles blend into the image (sort-free either way).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ParticleBlend {
    /// Added: fire, sparks, magic. Order independent.
    #[default]
    Additive,
    /// Alpha blended: smoke, dust. Unsorted within a system; systems draw
    /// back to front.
    Alpha,
}

/// The space particles move in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SimulationSpace {
    /// Born at the emitter, then independent of it (smoke trails behind a
    /// moving emitter).
    #[default]
    World,
    /// Carried with the emitter (a shield's sparkles).
    Local,
}

/// Which way a particle quad faces.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParticleAlign {
    /// Facing the camera, rotated by the particle's rotation.
    Camera,
    /// Stretched along the screen-space velocity: length is `size + speed
    /// * stretch` (sparks, rain).
    Velocity { stretch: f32 },
}

/// Particles bounce off the depth buffer (the full tier; on the no-compute
/// tier they pass through and [`crate::FrameStats::particle_collision_skipped`]
/// counts the system). World space only. Turns the depth prepass on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleCollision {
    /// Fraction of the normal speed kept (0 sticks, 1 bounces fully).
    pub bounce: f32,
    /// Fraction of the tangential speed lost per hit.
    pub friction: f32,
}

impl Default for ParticleCollision {
    fn default() -> Self {
        ParticleCollision {
            bounce: 0.4,
            friction: 0.2,
        }
    }
}

/// Where a system draws.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ParticleRender {
    /// Camera-facing (or velocity-aligned) quads in the 3D pass.
    #[default]
    Billboard,
    /// Sprites in the 2D batch on `layer` (positions in the xy plane, z
    /// from the emitter): always simulated on the CPU.
    Sprite { layer: i32 },
}

/// The emitter description: an asset. `[min, max]` pairs are uniform random
/// ranges per particle. The default is a small fountain of soft white
/// points.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleEmitter {
    /// Ring capacity: the newest particles overwrite the oldest beyond it.
    pub max_particles: u32,
    pub shape: EmitterShape,
    /// Random deviation of the start direction, degrees (180: any direction).
    pub spread_degrees: f32,
    /// Continuous particles per second.
    pub rate: f32,
    pub bursts: Vec<Burst>,
    /// Seconds per emission cycle.
    pub duration: f32,
    /// Cycles repeat forever; otherwise the emitter stops after one.
    pub looping: bool,
    pub lifetime: [f32; 2],
    pub speed: [f32; 2],
    /// Start size (the quad's side) in world units.
    pub size: [f32; 2],
    pub size_over_life: Curve,
    pub color_over_life: ColorCurve,
    /// Linear tint multiplied with the colour curve and the texture; above 1
    /// is emissive (blooms).
    pub color: [f32; 4],
    pub rotation_degrees: [f32; 2],
    pub angular_velocity_degrees: [f32; 2],
    /// Acceleration in the simulation space (units per second²).
    pub gravity: Vec3,
    /// Linear drag per second (velocity scales by 1 / (1 + drag dt)).
    pub drag: f32,
    pub noise: ParticleNoise,
    /// `None`: a soft round dot.
    pub texture: Option<TextureHandle>,
    pub sheet: SpriteSheet,
    pub blend: ParticleBlend,
    pub space: SimulationSpace,
    pub align: ParticleAlign,
    /// Soft particles: fade where the quad comes within this distance of the
    /// scene behind it (0: off). Turns the depth prepass on.
    pub soft_distance: f32,
    pub collision: Option<ParticleCollision>,
    pub render: ParticleRender,
    /// Sprite mode: strength of the order-independent glow ([`crate::Glow`]).
    pub glow: f32,
}

impl Default for ParticleEmitter {
    fn default() -> Self {
        ParticleEmitter {
            max_particles: 256,
            shape: EmitterShape::Cone {
                angle_degrees: 15.0,
                radius: 0.05,
            },
            spread_degrees: 0.0,
            rate: 40.0,
            bursts: Vec::new(),
            duration: 1.0,
            looping: true,
            lifetime: [1.0, 1.5],
            speed: [1.5, 2.5],
            size: [0.08, 0.12],
            size_over_life: Curve::default(),
            color_over_life: ColorCurve::default(),
            color: [1.0; 4],
            rotation_degrees: [0.0, 0.0],
            angular_velocity_degrees: [0.0, 0.0],
            gravity: Vec3::new(0.0, -2.0, 0.0),
            drag: 0.0,
            noise: ParticleNoise::default(),
            texture: None,
            sheet: SpriteSheet::default(),
            blend: ParticleBlend::Additive,
            space: SimulationSpace::World,
            align: ParticleAlign::Camera,
            soft_distance: 0.0,
            collision: None,
            render: ParticleRender::Billboard,
            glow: 0.0,
        }
    }
}

impl ParticleEmitter {
    /// The longest a particle lives, in seconds.
    pub fn max_lifetime(&self) -> f32 {
        self.lifetime[0].max(self.lifetime[1]).max(0.0)
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.max_particles == 0 || self.max_particles > 1 << 20 {
            return Err(format!(
                "max_particles {} (1 to 1048576)",
                self.max_particles
            ));
        }
        if self.size_over_life.keys.is_empty() || self.size_over_life.keys.len() > MAX_CURVE_KEYS {
            return Err(format!(
                "size_over_life has {} keys (1 to {MAX_CURVE_KEYS})",
                self.size_over_life.keys.len()
            ));
        }
        if self.color_over_life.keys.is_empty() || self.color_over_life.keys.len() > MAX_CURVE_KEYS
        {
            return Err(format!(
                "color_over_life has {} keys (1 to {MAX_CURVE_KEYS})",
                self.color_over_life.keys.len()
            ));
        }
        if !(self.rate.is_finite() && self.rate >= 0.0) {
            return Err(format!("rate {}", self.rate));
        }
        Ok(())
    }
}

/// A handle to an emitter of a [`crate::RenderScene`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EmitterHandle(pub(crate) u32);

impl EmitterHandle {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// One running emitter this frame. The renderer keeps its particles by
/// `id` between frames (a system absent from a frame is dropped).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleSystem {
    /// Stable identity (the entity's id): state is kept per id.
    pub id: u64,
    pub emitter: EmitterHandle,
    /// The emitter's local to world transform.
    pub transform: Mat4,
    pub seed: u64,
    /// The tick emission started.
    pub start_tick: u64,
    /// Emission stops at this tick (the particles live out); `None` keeps
    /// emitting while the emitter's cycle does.
    pub stop_tick: Option<u64>,
    /// The system's clock: the game's tick (hold it still for hit-stop).
    pub tick: u64,
}

impl ParticleSystem {
    pub fn new(id: u64, emitter: EmitterHandle, transform: Mat4, seed: u64, tick: u64) -> Self {
        ParticleSystem {
            id,
            emitter,
            transform,
            seed,
            start_tick: 0,
            stop_tick: None,
            tick,
        }
    }
}

/// Where particle systems simulate ([`crate::RenderDesc::particles`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ParticleSimulation {
    /// Compute on the full tier, CPU on the no-compute tier.
    #[default]
    Auto,
    /// Always on the CPU (the no-compute tier's path, on any tier).
    Cpu,
}

// ---------------------------------------------------------------------------
// Hashing and noise (mirrored in shaders/noise.wgsl).
// ---------------------------------------------------------------------------

#[inline]
pub(crate) fn pcg(v: u32) -> u32 {
    let state = v.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    let word = ((state >> ((state >> 28) + 4)) ^ state).wrapping_mul(277_803_737);
    (word >> 22) ^ word
}

#[inline]
pub(crate) fn hash_unit(v: u32) -> f32 {
    (pcg(v) >> 8) as f32 / 16_777_216.0
}

/// A 32-bit seed from a 64-bit one.
pub(crate) fn hash_seed(seed: u64) -> u32 {
    pcg(seed as u32 ^ pcg((seed >> 32) as u32))
}

fn lattice3(x: i32, y: i32, z: i32, seed: u32) -> f32 {
    let h = (x as u32).wrapping_mul(73_856_093)
        ^ (y as u32).wrapping_mul(19_349_663)
        ^ (z as u32).wrapping_mul(83_492_791)
        ^ seed;
    hash_unit(h)
}

/// Trilinear value noise in [0, 1) with a smoothstep fade.
pub(crate) fn value_noise3(p: Vec3, seed: u32) -> f32 {
    let fl = p.floor();
    let (ix, iy, iz) = (fl.x as i32, fl.y as i32, fl.z as i32);
    let f = p - fl;
    let u = f * f * (Vec3::splat(3.0) - 2.0 * f);
    let c = |dx, dy, dz| lattice3(ix + dx, iy + dy, iz + dz, seed);
    let x00 = c(0, 0, 0) + (c(1, 0, 0) - c(0, 0, 0)) * u.x;
    let x10 = c(0, 1, 0) + (c(1, 1, 0) - c(0, 1, 0)) * u.x;
    let x01 = c(0, 0, 1) + (c(1, 0, 1) - c(0, 0, 1)) * u.x;
    let x11 = c(0, 1, 1) + (c(1, 1, 1) - c(0, 1, 1)) * u.x;
    let y0 = x00 + (x10 - x00) * u.y;
    let y1 = x01 + (x11 - x01) * u.y;
    y0 + (y1 - y0) * u.z
}

/// Smooth 1D noise in -1..1 (camera shake).
pub(crate) fn noise1(t: f32, seed: u32) -> f32 {
    value_noise3(Vec3::new(t, 0.37, 0.71), seed) * 2.0 - 1.0
}

/// The random value `stream` of particle `n`.
#[inline]
fn rand(seed: u32, n: u32, stream: u32) -> f32 {
    hash_unit(seed ^ pcg(n.wrapping_add(stream.wrapping_mul(0x9e37_79b9))))
}

// ---------------------------------------------------------------------------
// The spawn schedule (computed on the CPU for both paths).
// ---------------------------------------------------------------------------

/// What one tick of a system does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TickStep {
    pub tick: u64,
    /// Seconds since the system started, at the end of this tick.
    pub time: f32,
    /// The index of the first particle spawned this tick.
    pub first: u64,
    /// Particles spawned: bursts first (age offset 0), then continuous ones
    /// spread over the tick.
    pub count: u32,
    pub bursts: u32,
    /// The emitter's transform during this tick.
    pub transform: Mat4,
}

/// How many bursts (summed counts) have fired strictly before `end` seconds
/// into one cycle.
fn bursts_in_cycle(bursts: &[Burst], end: f64, duration: f64) -> u64 {
    let mut n = 0u64;
    for b in bursts {
        let cycles = b.cycles.max(1) as u64;
        let interval = b.interval.max(0.0) as f64;
        for i in 0..cycles {
            let t = b.time.max(0.0) as f64 + i as f64 * interval;
            if t >= duration && duration > 0.0 || t >= end {
                break;
            }
            n += b.count as u64;
        }
    }
    n
}

/// Particles spawned with a spawn time before `elapsed` seconds:
/// (continuous, bursts).
fn spawned_before(e: &ParticleEmitter, elapsed: f64, stop: f64) -> (u64, u64) {
    let duration = (e.duration.max(0.0) as f64).max(1e-6);
    let end = if e.looping {
        elapsed.min(stop)
    } else {
        elapsed.min(stop).min(duration)
    };
    if end <= 0.0 {
        return (0, 0);
    }
    let continuous = (e.rate.max(0.0) as f64 * end).floor() as u64;
    let bursts = if e.looping {
        let full = (end / duration).floor();
        let per_cycle = bursts_in_cycle(&e.bursts, duration, duration);
        full as u64 * per_cycle + bursts_in_cycle(&e.bursts, end - full * duration, duration)
    } else {
        bursts_in_cycle(&e.bursts, end, duration)
    };
    (continuous, bursts)
}

/// The spawns of `tick` for a system.
pub(crate) fn tick_step(
    e: &ParticleEmitter,
    sys: &ParticleSystem,
    tick: u64,
    tick_rate: f32,
    transform: Mat4,
) -> TickStep {
    let dt = 1.0 / tick_rate.max(1e-3) as f64;
    let stop = sys.stop_tick.map_or(f64::INFINITY, |s| {
        s.saturating_sub(sys.start_tick) as f64 * dt
    });
    let (time, first, count, bursts) = if tick < sys.start_tick {
        (0.0, 0, 0, 0)
    } else {
        let j = (tick - sys.start_tick) as f64;
        let (c0, b0) = spawned_before(e, j * dt, stop);
        let (c1, b1) = spawned_before(e, (j + 1.0) * dt, stop);
        let bursts = b1 - b0;
        (
            ((j + 1.0) * dt) as f32,
            c0 + b0,
            (c1 - c0 + bursts).min(u32::MAX as u64) as u32,
            bursts.min(u32::MAX as u64) as u32,
        )
    };
    TickStep {
        tick,
        time,
        first,
        count,
        bursts,
        transform,
    }
}

/// The ticks a system's state must step through to reach `sys.tick`, and
/// whether it restarts from empty.
pub(crate) fn plan_ticks(
    e: &ParticleEmitter,
    sys: &ParticleSystem,
    last_tick: Option<u64>,
    prev_transform: Option<Mat4>,
    tick_rate: f32,
) -> (bool, Vec<TickStep>) {
    let window = (e.max_lifetime() as f64 * tick_rate as f64).ceil() as u64 + 1;
    let (reset, from) = match last_tick {
        Some(last) if sys.tick >= last && sys.tick - last <= window => (false, last + 1),
        _ => (
            true,
            sys.tick.saturating_sub(window - 1).max(sys.start_tick),
        ),
    };
    let ticks: Vec<u64> = (from..=sys.tick).collect();
    let n = ticks.len();
    let steps = ticks
        .into_iter()
        .enumerate()
        .map(|(i, t)| {
            // Moving emitters spawn along the path between the frames.
            let transform = match (reset, prev_transform) {
                (false, Some(prev)) if n > 1 => {
                    let f = (i + 1) as f32 / n as f32;
                    let mut m = sys.transform;
                    m.w_axis = prev.w_axis.lerp(sys.transform.w_axis, f);
                    m
                }
                _ => sys.transform,
            };
            tick_step(e, sys, t, tick_rate, transform)
        })
        .collect();
    (reset, steps)
}

// ---------------------------------------------------------------------------
// Particle state and the CPU simulation.
// ---------------------------------------------------------------------------

/// One particle slot (`struct Particle` in particles_sim.wgsl). Dead when
/// `age >= life`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleState {
    /// xyz: position (simulation space), w: age in seconds.
    pub pos_age: [f32; 4],
    /// xyz: velocity, w: lifetime in seconds.
    pub vel_life: [f32; 4],
    /// x: start size, y: start rotation (radians), z: angular velocity
    /// (radians per second), w: a per-particle random value in 0..1.
    pub misc: [f32; 4],
}

impl ParticleState {
    pub fn alive(&self) -> bool {
        self.pos_age[3] < self.vel_life[3]
    }
}

/// Points and normals on a mesh's surface, area weighted, for
/// [`EmitterShape::Mesh`] (the same table on both paths).
pub(crate) fn mesh_points(mesh: &MeshData) -> Vec<[f32; 4]> {
    let tris: Vec<[Vec3; 3]> = mesh
        .indices
        .chunks_exact(3)
        .filter_map(|t| {
            let v = |i: u32| {
                mesh.vertices
                    .get(i as usize)
                    .map(|v| Vec3::from(v.position))
            };
            Some([v(t[0])?, v(t[1])?, v(t[2])?])
        })
        .collect();
    let areas: Vec<f32> = tris
        .iter()
        .map(|[a, b, c]| (*b - *a).cross(*c - *a).length() * 0.5)
        .collect();
    let total: f32 = areas.iter().sum();
    let mut out = Vec::with_capacity(MESH_SAMPLE_POINTS * 2);
    if tris.is_empty() || total <= 0.0 {
        for _ in 0..MESH_SAMPLE_POINTS {
            out.push([0.0; 4]);
            out.push([0.0, 1.0, 0.0, 0.0]);
        }
        return out;
    }
    let mut cdf = Vec::with_capacity(areas.len());
    let mut acc = 0.0;
    for a in &areas {
        acc += a / total;
        cdf.push(acc);
    }
    for i in 0..MESH_SAMPLE_POINTS as u32 {
        let r = |s: u32| rand(0x6d65_7368, i, s);
        let pick = r(0);
        let t = cdf.partition_point(|&c| c < pick).min(tris.len() - 1);
        let [a, b, c] = tris[t];
        let (mut u, mut v) = (r(1), r(2));
        if u + v > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        let p = a + (b - a) * u + (c - a) * v;
        let n = (b - a).cross(c - a).normalize_or(Vec3::Y);
        out.push(p.extend(1.0).to_array());
        out.push(n.extend(0.0).to_array());
    }
    out
}

/// A unit vector within `angle` radians of `axis` (uniform over the cap).
fn cone_dir(axis: Vec3, angle: f32, r0: f32, r1: f32) -> Vec3 {
    let cos_max = angle.min(std::f32::consts::PI).cos();
    let cos_t = 1.0 - r0 * (1.0 - cos_max);
    let sin_t = (1.0 - cos_t * cos_t).max(0.0).sqrt();
    let phi = r1 * std::f32::consts::TAU;
    let helper = if axis.y.abs() < 0.999 {
        Vec3::Y
    } else {
        Vec3::X
    };
    let t = helper.cross(axis).normalize();
    let b = axis.cross(t);
    (t * (phi.cos() * sin_t) + b * (phi.sin() * sin_t) + axis * cos_t).normalize_or(axis)
}

/// The emitter shape's (position, direction) for random values `r`.
fn shape_sample(shape: &EmitterShape, r: [f32; 4], points: &[[f32; 4]]) -> (Vec3, Vec3) {
    match *shape {
        EmitterShape::Point => (Vec3::ZERO, Vec3::Y),
        EmitterShape::Sphere { radius, surface } => {
            let d = cone_dir(Vec3::Y, std::f32::consts::PI, r[0], r[1]);
            let k = if surface { 1.0 } else { r[2].cbrt() };
            (d * radius * k, d)
        }
        EmitterShape::Cone {
            angle_degrees,
            radius,
        } => {
            let d = cone_dir(Vec3::Y, angle_degrees.to_radians(), r[0], r[1]);
            let a = r[2] * std::f32::consts::TAU;
            let rr = r[3].sqrt() * radius;
            (Vec3::new(a.cos() * rr, 0.0, a.sin() * rr), d)
        }
        EmitterShape::Box { half_extents } => (
            (Vec3::new(r[0], r[1], r[2]) * 2.0 - Vec3::ONE) * half_extents,
            Vec3::Y,
        ),
        EmitterShape::Mesh { .. } => {
            let count = points.len() / 2;
            if count == 0 {
                return (Vec3::ZERO, Vec3::Y);
            }
            let i = ((r[0] * count as f32) as usize).min(count - 1);
            (
                Vec4::from(points[i * 2]).truncate(),
                Vec4::from(points[i * 2 + 1]).truncate(),
            )
        }
    }
}

/// A newly spawned particle `n` (the `k`-th of `count` this tick).
#[allow(clippy::too_many_arguments)]
fn spawn(
    e: &ParticleEmitter,
    seed: u32,
    n: u32,
    k: u32,
    step: &TickStep,
    dt: f32,
    points: &[[f32; 4]],
) -> ParticleState {
    let r = |s: u32| rand(seed, n, s);
    let lerp = |range: [f32; 2], t: f32| range[0] + (range[1] - range[0]) * t;
    let life = lerp(e.lifetime, r(0)).max(1e-3);
    let speed = lerp(e.speed, r(1));
    let size = lerp(e.size, r(2));
    let rot = lerp(e.rotation_degrees, r(3)).to_radians();
    let spin = lerp(e.angular_velocity_degrees, r(4)).to_radians();
    let (mut pos, dir) = shape_sample(&e.shape, [r(6), r(7), r(8), r(11)], points);
    let dir = if e.spread_degrees > 0.0 {
        cone_dir(dir, e.spread_degrees.to_radians(), r(9), r(10))
    } else {
        dir
    };
    let mut vel = dir * speed;
    if e.space == SimulationSpace::World {
        pos = step.transform.transform_point3(pos);
        vel = step.transform.transform_vector3(vel);
    }
    // Continuous spawns spread over the tick; bursts start at age 0.
    let age = if k < step.bursts {
        0.0
    } else {
        let c = (step.count - step.bursts).max(1) as f32;
        ((k - step.bursts) as f32 + 0.5) / c * dt
    };
    pos += vel * age;
    ParticleState {
        pos_age: pos.extend(age).to_array(),
        vel_life: vel.extend(life).to_array(),
        misc: [size, rot, spin, r(5)],
    }
}

/// One fixed step of a live particle at `time` seconds.
fn integrate(e: &ParticleEmitter, p: &mut ParticleState, dt: f32, time: f32, seed: u32) {
    let mut pos = Vec3::from_slice(&p.pos_age[..3]);
    let mut vel = Vec3::from_slice(&p.vel_life[..3]);
    vel += e.gravity * dt;
    if e.noise.strength != 0.0 {
        let q = pos * e.noise.frequency + Vec3::splat(time * e.noise.scroll);
        let n = Vec3::new(
            value_noise3(q, seed ^ 0x1111),
            value_noise3(q, seed ^ 0x2222),
            value_noise3(q, seed ^ 0x3333),
        ) * 2.0
            - Vec3::ONE;
        vel += n * e.noise.strength * dt;
    }
    vel *= 1.0 / (1.0 + e.drag.max(0.0) * dt);
    pos += vel * dt;
    p.pos_age = pos.extend(p.pos_age[3] + dt).to_array();
    p.vel_life = vel.extend(p.vel_life[3]).to_array();
}

/// For a slot, the particle index spawned into it by `step` (the newest of
/// the step's spawns that maps to this slot), if any.
#[inline]
pub(crate) fn slot_spawn(slot: u32, max: u32, first: u64, count: u32) -> Option<(u32, u32)> {
    let first_mod = (first % max as u64) as u32;
    let off = (slot + max - first_mod) % max;
    if off >= count {
        return None;
    }
    let k = off + ((count - 1 - off) / max) * max;
    Some(((first as u32).wrapping_add(k), k))
}

/// The CPU path of one system: its slots and where its clock stood.
#[derive(Clone, Debug, Default)]
pub struct CpuParticles {
    pub slots: Vec<ParticleState>,
    pub last_tick: Option<u64>,
    pub prev_transform: Option<Mat4>,
}

impl CpuParticles {
    /// Steps the system to `sys.tick`; returns the ticks simulated.
    pub(crate) fn advance(
        &mut self,
        e: &ParticleEmitter,
        sys: &ParticleSystem,
        tick_rate: f32,
        points: &[[f32; 4]],
    ) -> u32 {
        let max = e.max_particles;
        if self.slots.len() != max as usize {
            self.slots = vec![ParticleState::default(); max as usize];
            self.last_tick = None;
        }
        let (reset, steps) = plan_ticks(e, sys, self.last_tick, self.prev_transform, tick_rate);
        if reset {
            self.slots.fill(ParticleState::default());
        }
        let dt = 1.0 / tick_rate.max(1e-3);
        let seed = hash_seed(sys.seed);
        for step in &steps {
            for (slot, p) in self.slots.iter_mut().enumerate() {
                match slot_spawn(slot as u32, max, step.first, step.count) {
                    Some((n, k)) => *p = spawn(e, seed, n, k, step, dt, points),
                    None if p.alive() => integrate(e, p, dt, step.time, seed),
                    None => {}
                }
            }
        }
        self.last_tick = Some(sys.tick);
        self.prev_transform = Some(sys.transform);
        steps.len() as u32
    }

    pub fn alive(&self) -> usize {
        self.slots.iter().filter(|p| p.alive()).count()
    }
}

/// What a particle looks like at its age: (size, colour, rotation, sheet frame).
pub(crate) fn appearance(e: &ParticleEmitter, p: &ParticleState) -> (f32, [f32; 4], f32, u32) {
    let t = (p.pos_age[3] / p.vel_life[3].max(1e-6)).clamp(0.0, 1.0);
    let size = p.misc[0] * e.size_over_life.eval(t);
    let c = e.color_over_life.eval(t);
    let color = [
        c[0] * e.color[0],
        c[1] * e.color[1],
        c[2] * e.color[2],
        c[3] * e.color[3],
    ];
    let rotation = p.misc[1] + p.misc[2] * p.pos_age[3];
    let frames = sheet_frames(&e.sheet);
    let frame = match e.sheet.mode {
        SheetMode::Off => 0,
        SheetMode::OverLife { cycles } => (t * cycles * frames as f32) as u32 % frames,
        SheetMode::Fps(fps) => (p.pos_age[3] * fps) as u32 % frames,
        SheetMode::Random => ((p.misc[3] * frames as f32) as u32).min(frames - 1),
    };
    (size, color, rotation, frame)
}

pub(crate) fn sheet_frames(sheet: &SpriteSheet) -> u32 {
    let all = sheet.columns.max(1) * sheet.rows.max(1);
    if sheet.frames == 0 {
        all
    } else {
        sheet.frames.min(all)
    }
}
