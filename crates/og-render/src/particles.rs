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
//! **Two paths, one description.** By default the simulation runs in a
//! compute shader (`shaders/particles_sim.wgsl`) into a storage buffer the
//! draw reads as instance data, with nothing read back: it is visual only.
//! Colliding particles meet the depth prepass or, explicitly
//! ([`CollisionTarget::Colliders`], and always in sprite mode), the scene's
//! colliders through `shaders/scene_colliders.wgsl`. With
//! [`ParticleSimulation::Cpu`], [`CpuParticles`] runs the same steps in
//! Rust and uploads the state. Sprite-mode emitters
//! ([`ParticleRender::Sprite`]) simulate in compute too and draw through
//! the sprite batch; one with a texture array ([`ParticleEmitter::array`])
//! draws each particle from a layer of it, the layers being the sheet's
//! frames.
//!
//! **Events and sub-emitters.** The CPU path reports what happens to its
//! particles ([`ParticleEvent`]: births, deaths, collisions) and spawns
//! each emitter's [`SubEmitter`]s at them. Its arithmetic is
//! deterministic on every host (og-core's transcendentals, never the host
//! libm), so a host can run the same steps in its tick and read the events
//! as gameplay ([`crate::RenderScene::hosted_particles`] then hands the
//! renderer the tick's particles to draw, so what is drawn is what the
//! game read). A system whose emitter has sub-emitters simulates on the
//! CPU; its children draw with their own emitter's look.

use crate::scene::{MeshData, MeshHandle, TextureArrayHandle, TextureHandle};
use glam::{Mat4, Vec3, Vec4};
// The CPU path's transcendentals: bit-identical on every host, so a host's
// tick can run it (ParticleEvent, sub-emitters).
use og_core::math;

mod cpu;
mod noise;
mod schedule;
pub use cpu::*;
pub(crate) use noise::*;
pub(crate) use schedule::*;

/// The most keys a [`Curve`] or [`ColorCurve`] can have.
pub const MAX_CURVE_KEYS: usize = 8;
/// Points sampled from a mesh for [`EmitterShape::Mesh`].
pub const MESH_SAMPLE_POINTS: usize = 512;
/// How fast (metres per second, into the surface) a particle must hit a
/// collider for the hit to be a [`ParticleEventKind::Collision`]: one
/// resting on the ground touches it every tick and sends none.
pub const IMPACT_SPEED: f32 = 0.5;

/// The most sub-emitters one emitter fires ([`ParticleEmitter::sub_emitters`]).
pub const MAX_SUB_EMITTERS: usize = 4;

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

    /// The area under the curve from 0 to `t` (the curve constant beyond
    /// its ends): how far a rate or a speed scaled by it has carried by
    /// `t`. `curve_integral` in particle_common.wgsl, operation for
    /// operation.
    pub fn integral(&self, t: f32) -> f32 {
        let n = self.keys.len().min(MAX_CURVE_KEYS);
        if n == 0 {
            return t;
        }
        let k = &self.keys[..n];
        let mut area = 0.0;
        // Before the first key: its value.
        let mut prev = (0.0f32, k[0].1);
        for &(kt, kv) in k {
            if kt <= prev.0 {
                prev.1 = kv;
                continue;
            }
            if t <= kt {
                let f = (t - prev.0) / (kt - prev.0);
                let v = prev.1 + (kv - prev.1) * f.max(0.0);
                return area + (t - prev.0).max(0.0) * (prev.1 + v) * 0.5;
            }
            area += (kt - prev.0) * (prev.1 + kv) * 0.5;
            prev = (kt, kv);
        }
        area + (t - prev.0).max(0.0) * prev.1
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
    /// Along a Catmull-Rom curve through the first `count` (2 to
    /// [`MAX_SPLINE_POINTS`]) of `points` (closed back to the first when
    /// `closed`), evenly by length, moving along its tangent: a sword's
    /// arc, a ring of a portal, a river of motes.
    Spline {
        points: [Vec3; MAX_SPLINE_POINTS],
        count: u32,
        closed: bool,
    },
}

/// The most control points of an [`EmitterShape::Spline`].
pub const MAX_SPLINE_POINTS: usize = 8;

/// The most [`Attractor`]s of one emitter.
pub const MAX_ATTRACTORS: usize = 4;

/// The most cells along a side of a [`VectorField::Grid`].
pub const MAX_FIELD_CELLS: u32 = 16;

/// A point in the emitter's space that pulls particles towards it (a
/// negative strength pushes them away): a vortex's eye, a black hole, a
/// heal gathering motes. With a `radius` the pull falls linearly from
/// `strength` at the point to 0 at that distance; without (0), it is
/// `strength / (1 + d²)` at distance `d`. Units per second².
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Attractor {
    pub position: Vec3,
    pub strength: f32,
    pub radius: f32,
}

/// A field of accelerations the particles move through.
#[derive(Clone, Debug, PartialEq)]
pub enum VectorField {
    /// Curl noise (Bridson et al. 2007): the curl of three smooth noise
    /// fields, swirling and divergence free, so particles curl round each
    /// other instead of bunching (smoke, magic wisps, embers).
    /// `frequency` cells per unit, `scroll` cells per second.
    Curl {
        strength: f32,
        frequency: f32,
        scroll: f32,
    },
    /// A tiling grid of `cells`³ vectors (x fastest, then y, then z),
    /// `size` units a side, read trilinearly: a wind, a vortex or a flow
    /// painted in a tool and baked by the cook from a texture (`field
    /// "<texture>"`).
    Grid {
        strength: f32,
        size: f32,
        cells: u32,
        vectors: Vec<Vec3>,
    },
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

/// Particles bounce off what [`ParticleCollision::against`] names, in
/// compute with nothing read back; on the CPU path
/// ([`ParticleSimulation::Cpu`], a host's simulation, sub-emitters) they
/// bounce off [`crate::RenderScene::colliders`] whatever it names. A CPU
/// system with no colliders to meet passes through and
/// [`crate::FrameStats::particle_collision_skipped`] counts it. World
/// space only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleCollision {
    /// Fraction of the normal speed kept (0 sticks, 1 bounces fully).
    pub bounce: f32,
    /// Fraction of the tangential speed lost per hit.
    pub friction: f32,
    pub against: CollisionTarget,
}

impl Default for ParticleCollision {
    fn default() -> Self {
        ParticleCollision {
            bounce: 0.4,
            friction: 0.2,
            against: CollisionTarget::Depth,
        }
    }
}

/// What compute-simulated particles bounce off ([`ParticleCollision::against`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CollisionTarget {
    /// The depth prepass (turning it on): whatever the camera sees, and
    /// nothing it does not (off screen, behind something). Billboards and
    /// meshes; a sprite-mode emitter, which has no depth to read, meets
    /// the colliders instead.
    #[default]
    Depth,
    /// The scene's colliders ([`crate::RenderScene::colliders`]: the
    /// game's boxes, spheres, capsules, planes and height fields), as the
    /// CPU path meets them, on or off screen. Every render mode.
    Colliders,
}

impl ParticleEmitter {
    /// Whether its compute simulation meets the scene's colliders (world
    /// space, `collision` against them, or a sprite-mode emitter's).
    pub(crate) fn collides_with_colliders(&self) -> bool {
        self.space == SimulationSpace::World
            && self.collision.is_some_and(|c| {
                c.against == CollisionTarget::Colliders
                    || matches!(self.render, ParticleRender::Sprite { .. })
            })
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
    /// A mesh of the scene per particle (debris, leaves, shell casings),
    /// `size` its scale, turned by the particle's rotation about an axis
    /// of its own (its random value) or, with [`ParticleAlign::Velocity`],
    /// its +y along the velocity; its vertex colours times the particle's.
    Mesh { mesh: MeshHandle },
}

/// What happens to a particle ([`ParticleEvent::kind`], a
/// [`SubEmitter::trigger`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ParticleEventKind {
    /// Spawned.
    Birth,
    /// Reached the end of its lifetime (a particle the ring overwrites
    /// before then sends none).
    #[default]
    Death,
    /// Bounced off a collider ([`ParticleEmitter::collision`], the CPU path's
    /// [`SceneCollider`]s) at more than [`IMPACT_SPEED`].
    Collision,
}

/// One particle event of a CPU step: where, how fast, which particle.
/// Positions and velocities are in the world (a local-space system's are
/// carried out by its transform at that tick).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleEvent {
    pub kind: ParticleEventKind,
    /// The tick it happened in.
    pub tick: u64,
    /// The particle's index in its system's spawn order (low 32 bits).
    pub index: u32,
    pub position: Vec3,
    pub velocity: Vec3,
}

/// Another emitter fired at this one's particle events: `count` of its
/// particles are born where a parent particle was born, died or hit
/// something, moving at their own start velocity plus `inherit_velocity`
/// times the parent's (sparks off a bullet's impact, a firework's burst at
/// the end of its trail). With `life`, each parent particle instead runs
/// the child's own emission (its rate, bursts, duration and looping) from
/// its birth for as long as it lives, born where the parent is at each
/// tick (a comet's sparks, an effect spawned per instance of another:
/// Effekseer's child nodes, Unity's Birth sub-emitter, Niagara's
/// particle-spawned emitters). One level deep: a sub-emitter's own
/// sub-emitters are not fired.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubEmitter {
    pub trigger: SubTrigger,
    pub emitter: EmitterHandle,
    /// Particles per event (unused with [`SubTrigger::Life`]).
    pub count: u32,
    pub inherit_velocity: f32,
}

/// When a [`SubEmitter`] fires.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubTrigger {
    /// `count` particles at each of the parent's events of this kind.
    Event(ParticleEventKind),
    /// The child's emission, run by each parent particle while it lives.
    Life,
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
    /// Sprite mode: draw each particle from a layer of this texture array
    /// instead of `texture` (one draw per array, whichever layers the
    /// particles show). The layers are the sheet's frames: `sheet.mode`
    /// picks a particle's layer by its age (`OverLife`, `Fps`) or at
    /// random per particle (`Random`), and `sheet.frames` (0: every layer)
    /// limits them; `columns` and `rows` are not read. A billboard emitter
    /// with an array is refused ([`crate::RenderError::InvalidData`]).
    pub array: Option<TextureArrayHandle>,
    /// Sprite mode: fade each particle's quad to a round soft edge (the
    /// soft dot's falloff, which an untextured particle always has) over
    /// its texture or array layer.
    pub round: bool,
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
    /// Lit by the scene's lights (the sun with its shadows, the image-based
    /// light, the cluster's point and spot lights) instead of drawn at its
    /// colour: smoke and dust that darken away from the sun, debris that
    /// shades. A billboard is lit as a soft ball facing the camera; a mesh
    /// by its normals. 3D modes only.
    pub lit: bool,
    /// A lit mesh's surface roughness: 1 (the default) matte; below it a
    /// smooth surface whose reflection (the sun's highlight, the lights',
    /// the sky's at a grazing angle) is added over the body at full
    /// strength, so an alpha-blended body stays clear and glints where it
    /// faces the light (glass shards, ice, wet pebbles). Meshes only.
    pub roughness: f32,
    /// Above 0 the particles bend the image behind them instead of adding
    /// colour (heat haze, shock rings, a portal's shimmer): the opaque scene
    /// behind each one is drawn shifted by up to this fraction of the
    /// screen's height, along the texture's red and green as a normal map
    /// (an untextured particle pushes outwards from its centre), weighted
    /// by the colour curve's alpha. `blend` and the colour are not used.
    /// Billboards only.
    pub distortion: f32,
    /// Emitters fired at this one's particle events. A system with any
    /// simulates on the CPU.
    pub sub_emitters: Vec<SubEmitter>,
    /// The particle events gameplay reads (`og.ParticleEvent`), in no
    /// order; empty sends none. A host that reads them simulates the
    /// system in its tick ([`CpuParticles::advance`]) and the renderer
    /// draws that simulation as it stands ([`crate::RenderScene::hosted_particles`]).
    pub events: Vec<ParticleEventKind>,
    /// An effect material ([`crate::EffectMaterial`]) the particles are
    /// drawn with, over their texture and curves: billboards, velocity
    /// stretched and mesh particles (not distortion or sprite mode). A
    /// mesh particle with one is drawn as an effect mesh: blended, no
    /// depth written, both faces when the material is double sided.
    pub effect: Option<crate::EffectMaterialHandle>,
    /// How fast particles move over their life, against their velocity
    /// (Shuriken's speed modifier): each step moves `velocity × dt ×
    /// speed_over_life(t)`, the forces still acting on the velocity. `None`
    /// is 1.
    pub speed_over_life: Option<Curve>,
    /// The angular velocity over life, against each particle's own: the
    /// rotation is its integral (a spin that slows to a stop). `None` is 1.
    pub spin_over_life: Option<Curve>,
    /// The emission rate over each cycle (t 0 to 1 of `duration`), against
    /// `rate`: a stream that swells and dies. `None` is 1.
    pub rate_over_time: Option<Curve>,
    /// Points that pull or push the particles (at most [`MAX_ATTRACTORS`]).
    pub attractors: Vec<Attractor>,
    /// A field the particles move through.
    pub field: Option<VectorField>,
    /// The fraction of the emitter's own velocity (its transform's motion
    /// from tick to tick) a world-space particle starts with: sparks
    /// thrown from a swung sword, exhaust from a moving ship.
    pub inherit_velocity: f32,
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
            array: None,
            round: false,
            sheet: SpriteSheet::default(),
            blend: ParticleBlend::Additive,
            space: SimulationSpace::World,
            align: ParticleAlign::Camera,
            soft_distance: 0.0,
            collision: None,
            render: ParticleRender::Billboard,
            glow: 0.0,
            lit: false,
            roughness: 1.0,
            distortion: 0.0,
            sub_emitters: Vec::new(),
            events: Vec::new(),
            effect: None,
            speed_over_life: None,
            spin_over_life: None,
            rate_over_time: None,
            attractors: Vec::new(),
            field: None,
            inherit_velocity: 0.0,
        }
    }
}

impl ParticleEmitter {
    /// The longest a particle lives, in seconds.
    pub fn max_lifetime(&self) -> f32 {
        self.lifetime[0].max(self.lifetime[1]).max(0.0)
    }

    /// Refuses an emitter the renderer cannot draw, naming why (the
    /// parser checks every file with it, so the cook refuses it too).
    pub fn validate(&self) -> Result<(), String> {
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
        if self.array.is_some() && !matches!(self.render, ParticleRender::Sprite { .. }) {
            return Err("a texture array is drawn by a sprite-mode emitter (render sprite)".into());
        }
        if !(self.distortion.is_finite() && self.distortion >= 0.0) {
            return Err(format!("distortion {}", self.distortion));
        }
        if self.distortion > 0.0 && self.render != ParticleRender::Billboard {
            return Err("distortion is drawn by billboards (render billboard)".into());
        }
        for (name, c) in [
            ("speed_over_life", &self.speed_over_life),
            ("spin_over_life", &self.spin_over_life),
            ("rate_over_time", &self.rate_over_time),
        ] {
            if let Some(c) = c
                && (c.keys.is_empty() || c.keys.len() > MAX_CURVE_KEYS)
            {
                return Err(format!(
                    "{name} has {} keys (1 to {MAX_CURVE_KEYS})",
                    c.keys.len()
                ));
            }
        }
        if self.attractors.len() > MAX_ATTRACTORS {
            return Err(format!(
                "{} attractors (at most {MAX_ATTRACTORS})",
                self.attractors.len()
            ));
        }
        if let EmitterShape::Spline { count, .. } = self.shape
            && !(2..=MAX_SPLINE_POINTS as u32).contains(&count)
        {
            return Err(format!(
                "a spline has {count} points (2 to {MAX_SPLINE_POINTS})"
            ));
        }
        if let Some(VectorField::Grid {
            size,
            cells,
            vectors,
            ..
        }) = &self.field
            && (!(1..=MAX_FIELD_CELLS).contains(cells)
                || vectors.len() != (*cells as usize).pow(3)
                || !(size.is_finite() && *size > 0.0))
        {
            return Err(format!(
                "a field grid of {cells}³ cells holds {} vectors over {size} units (1 to {MAX_FIELD_CELLS} cells a side, a positive size)",
                vectors.len()
            ));
        }
        if self.sub_emitters.len() > MAX_SUB_EMITTERS {
            return Err(format!(
                "{} sub-emitters (at most {MAX_SUB_EMITTERS})",
                self.sub_emitters.len()
            ));
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
    /// How fast its particles' time runs against the tick: 1 normal, 0.25 a
    /// quarter speed, 0 frozen (og.TimeGroup's group scale). Spawning,
    /// ageing and motion all run on it; the tick still says how far to step.
    pub time_scale: f32,
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
            time_scale: 1.0,
        }
    }
}

/// A system's time against the tick, piecewise: from `tick` on, its time
/// runs from `time` seconds at `scale` (re-anchored where the scale
/// changes). At scale 1 from the start tick it is the tick's own time,
/// computed as before, so an unscaled system steps bit for bit as it did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ParticleClock {
    pub tick: u64,
    pub time: f64,
    pub scale: f32,
}

impl ParticleClock {
    /// Seconds of the system's time at the start of `tick`.
    fn at(&self, tick: u64, dt: f64) -> f64 {
        self.time + tick.saturating_sub(self.tick) as f64 * (dt * f64::from(self.scale))
    }
}

/// One tick of a particle system as that tick left it: what its spawns and
/// steps on that tick read ([`ParticleHistory`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleTick {
    pub tick: u64,
    pub transform: Mat4,
    pub start_tick: u64,
    pub stop_tick: Option<u64>,
    pub time_scale: f32,
    /// The system's clock as stepping one tick at a time leaves it.
    pub clock: ParticleClock,
}

/// A particle system's ticks, as many as its live particles can come from
/// (one lifetime at its time scale), recorded as each tick ends whether or
/// not a frame was drawn ([`RenderScene::particle_history`]). The renderer
/// steps a system through the ticks it has not drawn from these, not from
/// the frame's system alone, so its particles are a function of the ticks:
/// the same after a frame on every tick, after ticks stepped without frames
/// and after a restore that brought the history back (docs/design.md §2.4,
/// presentation). Presentation state: never the world's.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParticleHistory {
    ticks: std::collections::VecDeque<ParticleTick>,
}

impl ParticleHistory {
    /// Records `sys` at its tick: its clock follows the tick before's as a
    /// tick-by-tick step would (re-anchored where the scale changes, from
    /// the start tick after a gap), and ticks no live particle can come
    /// from any more are dropped. The same tick again replaces it (a held
    /// clock).
    pub fn record(&mut self, e: &ParticleEmitter, sys: &ParticleSystem, tick_rate: f32) {
        let scale = time_scale(sys);
        let dt = 1.0 / tick_rate.max(1e-3) as f64;
        // The same tick again (a held clock), or ticks a rewind took back
        // (og.net's rollback): recorded again from the tick before.
        while self.ticks.back().is_some_and(|t| t.tick >= sys.tick) {
            self.ticks.pop_back();
        }
        let clock = match self.ticks.back() {
            Some(t) if t.tick + 1 == sys.tick => next_clock(t.clock, sys.tick, scale, dt),
            _ => {
                self.ticks.clear();
                ParticleClock {
                    tick: sys.start_tick,
                    time: 0.0,
                    scale,
                }
            }
        };
        self.ticks.push_back(ParticleTick {
            tick: sys.tick,
            transform: sys.transform,
            start_tick: sys.start_tick,
            stop_tick: sys.stop_tick,
            time_scale: scale,
            clock,
        });
        let window = window(e, scale, tick_rate);
        while self
            .ticks
            .front()
            .is_some_and(|t| t.tick + window <= sys.tick)
        {
            self.ticks.pop_front();
        }
    }

    /// Tick `tick`'s record.
    pub fn at(&self, tick: u64) -> Option<&ParticleTick> {
        let first = self.ticks.front()?.tick;
        self.ticks
            .get(usize::try_from(tick.checked_sub(first)?).ok()?)
            .filter(|t| t.tick == tick)
    }

    /// The ticks recorded, oldest first.
    pub fn ticks(&self) -> impl Iterator<Item = &ParticleTick> {
        self.ticks.iter()
    }
}

/// `sys`'s time scale (non-finite as 1, negative as 0).
fn time_scale(sys: &ParticleSystem) -> f32 {
    if sys.time_scale.is_finite() {
        sys.time_scale.max(0.0)
    } else {
        1.0
    }
}

/// The clock of tick `tick` after `clock` stepped the tick before: the same
/// clock, or re-anchored at `tick` where the scale changes.
fn next_clock(clock: ParticleClock, tick: u64, scale: f32, dt: f64) -> ParticleClock {
    if clock.scale == scale {
        clock
    } else {
        ParticleClock {
            tick,
            time: clock.at(tick, dt),
            scale,
        }
    }
}

/// A lifetime in ticks at `scale` (slowed particles live more ticks),
/// replayed at most 16 normal lifetimes back.
fn window(e: &ParticleEmitter, scale: f32, tick_rate: f32) -> u64 {
    let window = (e.max_lifetime() as f64 * tick_rate as f64).ceil() as u64 + 1;
    if scale >= 1.0 {
        window
    } else {
        (window as f64 / f64::from(scale.max(1.0 / 16.0))).ceil() as u64
    }
}

/// Where particle systems simulate ([`crate::RenderDesc::particles`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ParticleSimulation {
    /// Compute (sprite-mode emitters on the CPU).
    #[default]
    Auto,
    /// Always on the CPU, on any backend: the state is Rust's to read and
    /// collides with [`crate::RenderScene::colliders`] instead of
    /// the depth buffer.
    Cpu,
}

#[cfg(test)]
mod tests;
