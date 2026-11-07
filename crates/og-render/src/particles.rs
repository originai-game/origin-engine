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
/// the end of its trail). One level deep: a sub-emitter's own
/// sub-emitters are not fired.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubEmitter {
    pub trigger: ParticleEventKind,
    pub emitter: EmitterHandle,
    pub count: u32,
    pub inherit_velocity: f32,
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
    value_noise3_at(p, [0; 3], seed)
}

/// [`value_noise3`] at `p` plus the whole lattice cells `whole`: the noise
/// of a point far from the world's zero, taken from its place relative to
/// the render origin and the origin's whole cells (`origin.rs`).
pub(crate) fn value_noise3_at(p: Vec3, whole: [i32; 3], seed: u32) -> f32 {
    let fl = p.floor();
    let (ix, iy, iz) = (
        fl.x as i32 + whole[0],
        fl.y as i32 + whole[1],
        fl.z as i32 + whole[2],
    );
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
    /// The seconds of particle time this tick takes: the tick's length
    /// times the system's time scale.
    pub dt: f32,
    /// The emitter's velocity during this tick (its translation's move
    /// since the last frame over the ticks between; 0 after a reset).
    pub velocity: Vec3,
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
    let continuous = match &e.rate_over_time {
        None => (e.rate.max(0.0) as f64 * end).floor() as u64,
        // The rate's integral over whole cycles and the part of the last.
        Some(c) => {
            let cycle = |t: f64| f64::from(c.integral((t / duration) as f32).max(0.0)) * duration;
            let full = (end / duration).floor();
            let area = full * cycle(duration) + cycle(end - full * duration);
            (e.rate.max(0.0) as f64 * area).floor() as u64
        }
    };
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
    clock: &ParticleClock,
) -> TickStep {
    let dt = 1.0 / tick_rate.max(1e-3) as f64;
    let stop = sys
        .stop_tick
        .map_or(f64::INFINITY, |s| clock.at(s.max(sys.start_tick), dt));
    let (time, first, count, bursts) = if tick < sys.start_tick {
        (0.0, 0, 0, 0)
    } else {
        let (t0, t1) = (clock.at(tick, dt), clock.at(tick + 1, dt));
        let (c0, b0) = spawned_before(e, t0, stop);
        let (c1, b1) = spawned_before(e, t1, stop);
        let bursts = b1 - b0;
        (
            t1 as f32,
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
        dt: (1.0 / tick_rate.max(1e-3)) * clock.scale,
        velocity: Vec3::ZERO,
    }
}

/// The ticks a system's state must step through to reach `sys.tick`, and
/// whether it restarts from empty.
/// Re-anchors `clock` at the system's time scale: from its start on a
/// reset (as if the scale had always held), else from the first tick not
/// yet stepped.
#[allow(clippy::too_many_arguments)]
pub(crate) fn plan_ticks(
    e: &ParticleEmitter,
    sys: &ParticleSystem,
    last_tick: Option<u64>,
    prev_transform: Option<Mat4>,
    tick_rate: f32,
    clock: &mut ParticleClock,
) -> (bool, Vec<TickStep>) {
    let scale = if sys.time_scale.is_finite() {
        sys.time_scale.max(0.0)
    } else {
        1.0
    };
    // A lifetime in ticks at this scale (slowed particles live more ticks),
    // replayed at most 16 normal lifetimes back.
    let window = (e.max_lifetime() as f64 * tick_rate as f64).ceil() as u64 + 1;
    let window = if scale >= 1.0 {
        window
    } else {
        (window as f64 / f64::from(scale.max(1.0 / 16.0))).ceil() as u64
    };
    let (reset, from) = match last_tick {
        Some(last) if sys.tick >= last && sys.tick - last <= window => (false, last + 1),
        _ => (
            true,
            sys.tick.saturating_sub(window - 1).max(sys.start_tick),
        ),
    };
    let dt = 1.0 / tick_rate.max(1e-3) as f64;
    if reset {
        *clock = ParticleClock {
            tick: sys.start_tick,
            time: 0.0,
            scale,
        };
    } else if clock.scale != scale {
        *clock = ParticleClock {
            tick: from,
            time: clock.at(from, dt),
            scale,
        };
    }
    let clock = *clock;
    let ticks: Vec<u64> = (from..=sys.tick).collect();
    let n = ticks.len();
    let velocity = match (reset, prev_transform) {
        (false, Some(prev)) if n > 0 => {
            (sys.transform.w_axis - prev.w_axis).truncate() * (tick_rate / n as f32)
        }
        _ => Vec3::ZERO,
    };
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
            TickStep {
                velocity,
                ..tick_step(e, sys, t, tick_rate, transform, &clock)
            }
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
/// [`EmitterShape::Mesh`] (the same table on both paths, and what a host
/// passes [`CpuParticles::advance`] as `points`): two entries a point.
pub fn mesh_points(mesh: &MeshData) -> Vec<[f32; 4]> {
    let tris: Vec<[Vec3; 3]> = mesh
        .indices
        .as_chunks::<3>()
        .0
        .iter()
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
    let cos_max = math::cos(angle.min(std::f32::consts::PI));
    let cos_t = 1.0 - r0 * (1.0 - cos_max);
    let sin_t = (1.0 - cos_t * cos_t).max(0.0).sqrt();
    let (phi_sin, phi_cos) = math::sin_cos(r1 * std::f32::consts::TAU);
    let helper = if axis.y.abs() < 0.999 {
        Vec3::Y
    } else {
        Vec3::X
    };
    let t = helper.cross(axis).normalize();
    let b = axis.cross(t);
    (t * (phi_cos * sin_t) + b * (phi_sin * sin_t) + axis * cos_t).normalize_or(axis)
}

/// The emitter shape's (position, direction) for random values `r`.
fn shape_sample(shape: &EmitterShape, r: [f32; 4], points: &[[f32; 4]]) -> (Vec3, Vec3) {
    match *shape {
        EmitterShape::Point => (Vec3::ZERO, Vec3::Y),
        EmitterShape::Sphere { radius, surface } => {
            let d = cone_dir(Vec3::Y, std::f32::consts::PI, r[0], r[1]);
            let k = if surface {
                1.0
            } else {
                math::pow(r[2], 1.0 / 3.0)
            };
            (d * radius * k, d)
        }
        EmitterShape::Cone {
            angle_degrees,
            radius,
        } => {
            let d = cone_dir(Vec3::Y, angle_degrees.to_radians(), r[0], r[1]);
            let (sin, cos) = math::sin_cos(r[2] * std::f32::consts::TAU);
            let rr = r[3].sqrt() * radius;
            (Vec3::new(cos * rr, 0.0, sin * rr), d)
        }
        EmitterShape::Box { half_extents } => (
            (Vec3::new(r[0], r[1], r[2]) * 2.0 - Vec3::ONE) * half_extents,
            Vec3::Y,
        ),
        EmitterShape::Mesh { .. } | EmitterShape::Spline { .. } => {
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
        vel = step.transform.transform_vector3(vel) + step.velocity * e.inherit_velocity;
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

/// A solid shape of the scene ([`crate::RenderScene::colliders`]): what
/// CPU-simulated particles bounce off and visual bodies meet, the scene's
/// ground and solid shapes as the host knows them (og-runtime gives its
/// og.Collider boxes, spheres, capsules and terrains' height fields). A particle
/// bounces when a step carries it from outside a shape to inside: one born
/// inside (sparks from a brazier's box) leaves freely.
#[derive(Clone, Debug, PartialEq)]
pub enum SceneCollider {
    /// Solid below the plane `normal · p = offset` (`normal` need not be
    /// normalized): an endless ground.
    Plane {
        normal: Vec3,
        offset: f32,
    },
    /// A box of `half_extents` placed by `transform` (rotation, translation
    /// and scale).
    Box {
        transform: Mat4,
        half_extents: Vec3,
    },
    Sphere {
        center: Vec3,
        radius: f32,
    },
    /// The points within `radius` of the segment `a`..`b` (og.Collider's
    /// SHAPE_CAPSULE: a character).
    Capsule {
        a: Vec3,
        b: Vec3,
        radius: f32,
    },
    /// A height field placed as [`crate::Terrain::transform`] places it
    /// (centred, along its x and z): solid below its surface, within its
    /// rectangle.
    HeightField {
        data: crate::TerrainData,
        transform: Mat4,
    },
}

impl SceneCollider {
    /// Whether `p` is inside, and the outward normal there (for a box, of
    /// the face nearest `from`, where the particle came from).
    fn hit(&self, from: Vec3, p: Vec3) -> Option<Vec3> {
        match self {
            SceneCollider::Plane { normal, offset } => {
                let n = normal.normalize_or_zero();
                (n != Vec3::ZERO && n.dot(p) < *offset / normal.length()).then_some(n)
            }
            SceneCollider::Sphere { center, radius } => {
                let d = p - *center;
                (d.length_squared() < radius * radius).then(|| d.normalize_or(Vec3::Y))
            }
            SceneCollider::Capsule { a, b, radius } => {
                let ab = *b - *a;
                let t = ((p - *a).dot(ab) / ab.length_squared().max(1e-12)).clamp(0.0, 1.0);
                let d = p - (*a + ab * t);
                (d.length_squared() < radius * radius).then(|| d.normalize_or(Vec3::Y))
            }
            SceneCollider::Box {
                transform,
                half_extents,
            } => {
                let inv = transform.inverse();
                let l = inv.transform_point3(p);
                if l.abs().cmpge(*half_extents).any() {
                    return None;
                }
                // The face the step came through: the axis it was most outside on.
                let f = inv.transform_point3(from);
                let out = f.abs() - *half_extents;
                let axis = if out.x >= out.y && out.x >= out.z {
                    Vec3::X * f.x.signum()
                } else if out.y >= out.z {
                    Vec3::Y * f.y.signum()
                } else {
                    Vec3::Z * f.z.signum()
                };
                Some(
                    inv.transpose()
                        .transform_vector3(axis)
                        .normalize_or(Vec3::Y),
                )
            }
            SceneCollider::HeightField { data, transform } => {
                let inv = transform.inverse();
                let l = inv.transform_point3(p);
                let half = data.size * 0.5;
                if l.x.abs() > half.x || l.z.abs() > half.y {
                    return None;
                }
                let at = glam::Vec2::new(l.x, l.z);
                (l.y < data.height(at)).then(|| {
                    let n = data.normal(at);
                    inv.transpose().transform_vector3(n).normalize_or(Vec3::Y)
                })
            }
        }
    }
}

/// The CPU path's collision response, as the compute shader's: the normal
/// part of the velocity reflected by `bounce`, the tangential part reduced
/// by `friction`, and the step taken again from `before`. Whether it
/// bounced.
fn collide(
    colliders: &[SceneCollider],
    c: ParticleCollision,
    before: Vec3,
    pos: &mut Vec3,
    vel: &mut Vec3,
    dt: f32,
) -> bool {
    for collider in colliders {
        if collider.hit(before, before).is_some() {
            continue;
        }
        let Some(n) = collider.hit(before, *pos) else {
            continue;
        };
        let vn = vel.dot(n);
        if vn < 0.0 {
            let normal_part = n * vn;
            *vel = (*vel - normal_part) * (1.0 - c.friction) - normal_part * c.bounce;
            *pos = before + *vel * dt;
            // A particle settling on a surface touches it every tick: only
            // an impact is a collision event.
            return -vn > IMPACT_SPEED;
        }
        return false;
    }
    false
}

/// The turbulence's lattice at the render origin `origin` for noise of
/// `frequency` (cells per metre): its whole cells and the rest, in f64, so
/// a world-space system samples the field it would at the world's zero
/// (`origin.rs`). Zero at a zero origin.
pub(crate) fn noise_origin(origin: Vec3, frequency: f32) -> ([i32; 3], Vec3) {
    let at = |o: f32| {
        let x = f64::from(o) * f64::from(frequency);
        let w = x.floor();
        (w as i32, (x - w) as f32)
    };
    let (x, y, z) = (at(origin.x), at(origin.y), at(origin.z));
    ([x.0, y.0, z.0], Vec3::new(x.1, y.1, z.1))
}

/// The pull of the attractors (`attractors`: in the simulation space) at
/// `p`.
fn attract(attractors: &[Attractor], p: Vec3) -> Vec3 {
    let mut a = Vec3::ZERO;
    for at in attractors.iter().take(MAX_ATTRACTORS) {
        let d = at.position - p;
        let dist = d.length();
        if dist < 1e-4 {
            continue;
        }
        let f = if at.radius > 0.0 {
            at.strength * (1.0 - dist / at.radius).max(0.0)
        } else {
            at.strength / (1.0 + dist * dist)
        };
        a += d * (f / dist);
    }
    a
}

/// Curl noise at lattice point `q` (`whole` cells added): the curl of
/// three value noise fields by central differences, about -1 to 1 on each
/// axis. `curl_noise` in particles_sim.wgsl.
pub(crate) fn curl_noise(q: Vec3, whole: [i32; 3], seed: u32) -> Vec3 {
    const H: f32 = 0.05;
    let psi = |q: Vec3, k: u32| value_noise3_at(q, whole, seed ^ (0x4444 + k * 0x1111));
    let d = |k: u32, axis: Vec3| (psi(q + axis * H, k) - psi(q - axis * H, k)) / (2.0 * H);
    Vec3::new(
        d(2, Vec3::Y) - d(1, Vec3::Z),
        d(0, Vec3::Z) - d(2, Vec3::X),
        d(1, Vec3::X) - d(0, Vec3::Y),
    ) * 0.25
}

/// A grid field's vector at `g` (in cells, tiling), trilinear.
pub(crate) fn grid_sample(cells: u32, vectors: &[Vec3], g: Vec3) -> Vec3 {
    let n = cells.max(1) as i32;
    let fl = g.floor();
    let f = g - fl;
    let at = |dx: i32, dy: i32, dz: i32| {
        let w = |c: f32, d: i32| (c as i32 + d).rem_euclid(n) as usize;
        let (x, y, z) = (w(fl.x, dx), w(fl.y, dy), w(fl.z, dz));
        vectors
            .get((z * n as usize + y) * n as usize + x)
            .copied()
            .unwrap_or(Vec3::ZERO)
    };
    let x00 = at(0, 0, 0).lerp(at(1, 0, 0), f.x);
    let x10 = at(0, 1, 0).lerp(at(1, 1, 0), f.x);
    let x01 = at(0, 0, 1).lerp(at(1, 0, 1), f.x);
    let x11 = at(0, 1, 1).lerp(at(1, 1, 1), f.x);
    x00.lerp(x10, f.y).lerp(x01.lerp(x11, f.y), f.z)
}

/// Where a field's lattice stands at the render origin `origin`: a curl
/// field's whole cells and the rest (as [`noise_origin`]), a grid's
/// offset in cells within one tile.
pub(crate) fn field_origin(field: &VectorField, origin: Vec3) -> ([i32; 3], Vec3) {
    match field {
        VectorField::Curl { frequency, .. } => noise_origin(origin, *frequency),
        VectorField::Grid { size, cells, .. } => {
            let at = |o: f32| {
                let x = f64::from(o) / f64::from(*size);
                ((x - x.floor()) * f64::from(*cells)) as f32
            };
            ([0; 3], Vec3::new(at(origin.x), at(origin.y), at(origin.z)))
        }
    }
}

/// An emitter's attractors in its simulation space at a tick placed by
/// `transform`: carried into the world for a world-space system.
pub(crate) fn placed_attractors(e: &ParticleEmitter, transform: Mat4) -> Vec<Attractor> {
    e.attractors
        .iter()
        .take(MAX_ATTRACTORS)
        .map(|a| match e.space {
            SimulationSpace::World => Attractor {
                position: transform.transform_point3(a.position),
                ..*a
            },
            SimulationSpace::Local => *a,
        })
        .collect()
}

/// The acceleration of `pulls` (attractors in the simulation space) and
/// the field at `p` at `time` seconds.
fn forces(
    e: &ParticleEmitter,
    pulls: &[Attractor],
    p: Vec3,
    time: f32,
    seed: u32,
    origin: Vec3,
) -> Vec3 {
    let mut a = attract(pulls, p);
    match &e.field {
        Some(
            f @ VectorField::Curl {
                strength,
                frequency,
                scroll,
            },
        ) => {
            let (whole, rest) = field_origin(f, origin);
            let q = p * *frequency + rest + Vec3::splat(time * scroll);
            a += curl_noise(q, whole, seed) * *strength;
        }
        Some(
            f @ VectorField::Grid {
                strength,
                size,
                cells,
                vectors,
            },
        ) => {
            let (_, rest) = field_origin(f, origin);
            let g = p / *size * *cells as f32 + rest;
            a += grid_sample(*cells, vectors, g) * *strength;
        }
        None => {}
    }
    a
}

/// A spline shape's table, as [`mesh_points`]'s: points evenly along the
/// Catmull-Rom curve through `points[..count]` with its tangent, two
/// entries a point.
pub fn spline_points(points: &[Vec3], closed: bool) -> Vec<[f32; 4]> {
    let n = points.len();
    let mut out = Vec::with_capacity(MESH_SAMPLE_POINTS * 2);
    if n < 2 {
        for _ in 0..MESH_SAMPLE_POINTS {
            out.push([0.0; 4]);
            out.push([0.0, 1.0, 0.0, 0.0]);
        }
        return out;
    }
    let segments = if closed { n } else { n - 1 };
    let p = |i: isize| {
        if closed {
            points[i.rem_euclid(n as isize) as usize]
        } else {
            points[i.clamp(0, n as isize - 1) as usize]
        }
    };
    let at = |u: f32| {
        let s = ((u * segments as f32) as usize).min(segments - 1);
        let t = u * segments as f32 - s as f32;
        let i = s as isize;
        let (p0, p1, p2, p3) = (p(i - 1), p(i), p(i + 1), p(i + 2));
        let (t2, t3) = (t * t, t * t * t);
        0.5 * (2.0 * p1
            + (p2 - p0) * t
            + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
            + (3.0 * p1 - p0 - 3.0 * p2 + p3) * t3)
    };
    // Even by length: a dense polyline, then its length's equal steps.
    const DENSE: usize = 4096;
    let dense: Vec<Vec3> = (0..=DENSE).map(|i| at(i as f32 / DENSE as f32)).collect();
    let mut length = vec![0.0f32; DENSE + 1];
    for i in 1..=DENSE {
        length[i] = length[i - 1] + (dense[i] - dense[i - 1]).length();
    }
    let total = length[DENSE].max(1e-6);
    for k in 0..MESH_SAMPLE_POINTS {
        let want = (k as f32 + 0.5) / MESH_SAMPLE_POINTS as f32 * total;
        let i = length.partition_point(|&l| l < want).clamp(1, DENSE);
        let f = (want - length[i - 1]) / (length[i] - length[i - 1]).max(1e-9);
        let pos = dense[i - 1].lerp(dense[i], f);
        let dir = (dense[i] - dense[i - 1]).normalize_or(Vec3::Y);
        out.push(pos.extend(1.0).to_array());
        out.push(dir.extend(0.0).to_array());
    }
    out
}

/// The surface table of an emitter shape that has one (a mesh, its mesh's
/// `MeshData`; a spline): two entries a point, a placeholder point for
/// the others.
pub fn shape_points(shape: &EmitterShape, mesh: Option<&MeshData>) -> Vec<[f32; 4]> {
    match (shape, mesh) {
        (EmitterShape::Mesh { .. }, Some(m)) => mesh_points(m),
        (
            EmitterShape::Spline {
                points,
                count,
                closed,
            },
            _,
        ) => spline_points(&points[..(*count as usize).min(MAX_SPLINE_POINTS)], *closed),
        _ => vec![[0.0; 4], [0.0, 1.0, 0.0, 0.0]],
    }
}

/// One fixed step of a live particle at `time` seconds; `origin`: the
/// render origin a world-space system is simulated relative to. Whether it
/// bounced off a collider.
#[allow(clippy::too_many_arguments)]
fn integrate(
    e: &ParticleEmitter,
    pulls: &[Attractor],
    p: &mut ParticleState,
    dt: f32,
    time: f32,
    seed: u32,
    colliders: &[SceneCollider],
    origin: Vec3,
) -> bool {
    let mut pos = Vec3::from_slice(&p.pos_age[..3]);
    let mut vel = Vec3::from_slice(&p.vel_life[..3]);
    vel += e.gravity * dt;
    if e.noise.strength != 0.0 {
        let (whole, rest) = noise_origin(origin, e.noise.frequency);
        let q = pos * e.noise.frequency + rest + Vec3::splat(time * e.noise.scroll);
        let n = Vec3::new(
            value_noise3_at(q, whole, seed ^ 0x1111),
            value_noise3_at(q, whole, seed ^ 0x2222),
            value_noise3_at(q, whole, seed ^ 0x3333),
        ) * 2.0
            - Vec3::ONE;
        vel += n * e.noise.strength * dt;
    }
    vel += forces(e, pulls, pos, time, seed, origin) * dt;
    vel *= 1.0 / (1.0 + e.drag.max(0.0) * dt);
    let before = pos;
    let t = (p.pos_age[3] / p.vel_life[3].max(1e-6)).clamp(0.0, 1.0);
    pos += vel * (dt * e.speed_over_life.as_ref().map_or(1.0, |c| c.eval(t)));
    let hit = match e.collision {
        Some(c) => collide(colliders, c, before, &mut pos, &mut vel, dt),
        None => false,
    };
    p.pos_age = pos.extend(p.pos_age[3] + dt).to_array();
    p.vel_life = vel.extend(p.vel_life[3]).to_array();
    hit
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

/// The particles one [`SubEmitter`] fired, simulated with its emitter in
/// the world (its spawn order's own ring of the child's `max_particles`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChildParticles {
    pub slots: Vec<ParticleState>,
    /// The particles born into it so far: the next one's index.
    pub born: u64,
}

/// The CPU path of one system: its slots and where its clock stood.
#[derive(Clone, Debug, Default)]
pub struct CpuParticles {
    pub slots: Vec<ParticleState>,
    pub last_tick: Option<u64>,
    pub prev_transform: Option<Mat4>,
    pub clock: ParticleClock,
    /// The render origin a world-space system's slots are relative to
    /// (`origin.rs`): its turbulence samples the field there.
    pub origin: Vec3,
    /// Per [`ParticleEmitter::sub_emitters`] entry, the particles it fired.
    pub children: Vec<ChildParticles>,
}

/// The index of the particle in `slot` while `step` integrates it: the
/// slot's latest spawn before the step (particles fill the ring in order).
fn index_of(step: &TickStep, slot: u32, max: u32) -> u32 {
    let (last, max, slot) = (step.first.saturating_sub(1), max as u64, slot as u64);
    let back = (last % max + max - slot) % max;
    last.saturating_sub(back) as u32
}

/// The seed of a system's `k`-th sub-emitter's particles.
pub(crate) fn child_seed(seed: u32, k: usize) -> u32 {
    pcg(seed ^ pcg(0x5ab_e317 ^ k as u32))
}

/// A sub-emitter's particle `n`, born at `at` (world) with `inherited`
/// velocity added to its own start velocity. A mesh shape is a point here.
fn spawn_child(e: &ParticleEmitter, seed: u32, n: u32, at: Vec3, inherited: Vec3) -> ParticleState {
    let r = |s: u32| rand(seed, n, s);
    let lerp = |range: [f32; 2], t: f32| range[0] + (range[1] - range[0]) * t;
    let life = lerp(e.lifetime, r(0)).max(1e-3);
    let speed = lerp(e.speed, r(1));
    let size = lerp(e.size, r(2));
    let rot = lerp(e.rotation_degrees, r(3)).to_radians();
    let spin = lerp(e.angular_velocity_degrees, r(4)).to_radians();
    let shape = match e.shape {
        EmitterShape::Mesh { .. } | EmitterShape::Spline { .. } => EmitterShape::Point,
        s => s,
    };
    let (offset, dir) = shape_sample(&shape, [r(6), r(7), r(8), r(11)], &[]);
    let dir = if e.spread_degrees > 0.0 {
        cone_dir(dir, e.spread_degrees.to_radians(), r(9), r(10))
    } else {
        dir
    };
    ParticleState {
        pos_age: (at + offset).extend(0.0).to_array(),
        vel_life: (dir * speed + inherited).extend(life).to_array(),
        misc: [size, rot, spin, r(5)],
    }
}

impl CpuParticles {
    /// Steps the system to `sys.tick`; returns the ticks simulated.
    /// `colliders`: what its particles bounce off (empty: nothing). `subs`:
    /// the emitters of `e.sub_emitters`, in order (`None`: not fired).
    /// `events`: where to report every birth, death and collision of the
    /// system's own particles (not its sub-emitters'), in slot order per
    /// tick.
    #[allow(clippy::too_many_arguments)]
    pub fn advance(
        &mut self,
        e: &ParticleEmitter,
        sys: &ParticleSystem,
        tick_rate: f32,
        points: &[[f32; 4]],
        colliders: &[SceneCollider],
        subs: &[Option<&ParticleEmitter>],
        mut events: Option<&mut Vec<ParticleEvent>>,
    ) -> u32 {
        let max = e.max_particles;
        if self.slots.len() != max as usize {
            self.slots = vec![ParticleState::default(); max as usize];
            self.last_tick = None;
        }
        let firing: Vec<(usize, &SubEmitter, &ParticleEmitter)> = e
            .sub_emitters
            .iter()
            .enumerate()
            .filter_map(|(k, s)| Some((k, s, (*subs.get(k)?)?)))
            .collect();
        self.children
            .resize_with(e.sub_emitters.len(), Default::default);
        for &(k, _, ce) in &firing {
            let c = &mut self.children[k];
            if c.slots.len() != ce.max_particles as usize {
                c.slots = vec![ParticleState::default(); ce.max_particles as usize];
                c.born = 0;
            }
        }
        let (reset, steps) = plan_ticks(
            e,
            sys,
            self.last_tick,
            self.prev_transform,
            tick_rate,
            &mut self.clock,
        );
        if reset {
            self.slots.fill(ParticleState::default());
            for c in &mut self.children {
                c.slots.fill(ParticleState::default());
                c.born = 0;
            }
        }
        let seed = hash_seed(sys.seed);
        let want = |kind| events.is_some() || firing.iter().any(|f| f.1.trigger == kind);
        let (births, deaths, hits) = (
            want(ParticleEventKind::Birth),
            want(ParticleEventKind::Death),
            want(ParticleEventKind::Collision),
        );
        let local = e.space == SimulationSpace::Local;
        let mut fired: Vec<ParticleEvent> = Vec::new();
        for step in &steps {
            let dt = step.dt;
            let pulls = placed_attractors(e, step.transform);
            fired.clear();
            let mut report = |kind, n: u32, p: &ParticleState| {
                let (mut at, mut vel) = (
                    Vec3::from_slice(&p.pos_age[..3]),
                    Vec3::from_slice(&p.vel_life[..3]),
                );
                if local {
                    at = step.transform.transform_point3(at);
                    vel = step.transform.transform_vector3(vel);
                }
                fired.push(ParticleEvent {
                    kind,
                    tick: step.tick,
                    index: n,
                    position: at,
                    velocity: vel,
                });
            };
            for (slot, p) in self.slots.iter_mut().enumerate() {
                match slot_spawn(slot as u32, max, step.first, step.count) {
                    Some((n, k)) => {
                        *p = spawn(e, seed, n, k, step, dt, points);
                        if births {
                            report(ParticleEventKind::Birth, n, p);
                        }
                    }
                    None if p.alive() => {
                        let hit =
                            integrate(e, &pulls, p, dt, step.time, seed, colliders, self.origin);
                        if hit && hits {
                            report(
                                ParticleEventKind::Collision,
                                index_of(step, slot as u32, max),
                                p,
                            );
                        }
                        if !p.alive() && deaths {
                            report(
                                ParticleEventKind::Death,
                                index_of(step, slot as u32, max),
                                p,
                            );
                        }
                    }
                    None => {}
                }
            }
            // The sub-emitters: their particles step, then each event's are born.
            for &(k, sub, ce) in &firing {
                let cseed = child_seed(seed, k);
                let c = &mut self.children[k];
                let cmax = ce.max_particles.max(1) as u64;
                for q in c.slots.iter_mut().filter(|q| q.alive()) {
                    integrate(
                        ce,
                        &ce.attractors,
                        q,
                        dt,
                        step.time,
                        cseed,
                        colliders,
                        self.origin,
                    );
                }
                for ev in fired.iter().filter(|ev| ev.kind == sub.trigger) {
                    for _ in 0..sub.count {
                        let n = c.born;
                        c.born += 1;
                        c.slots[(n % cmax) as usize] = spawn_child(
                            ce,
                            cseed,
                            n as u32,
                            ev.position,
                            ev.velocity * sub.inherit_velocity,
                        );
                    }
                }
            }
            if let Some(out) = events.as_deref_mut() {
                out.extend_from_slice(&fired);
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

/// What a particle looks like at its age: (size, colour, rotation, frame
/// of `frames`: the sheet's, or an array emitter's layers).
pub(crate) fn appearance(
    e: &ParticleEmitter,
    p: &ParticleState,
    frames: u32,
) -> (f32, [f32; 4], f32, u32) {
    let frames = frames.max(1);
    let t = (p.pos_age[3] / p.vel_life[3].max(1e-6)).clamp(0.0, 1.0);
    let size = p.misc[0] * e.size_over_life.eval(t);
    let c = e.color_over_life.eval(t);
    let color = [
        c[0] * e.color[0],
        c[1] * e.color[1],
        c[2] * e.color[2],
        c[3] * e.color[3],
    ];
    let rotation = particle_rotation(e, p, t);
    let frame = match e.sheet.mode {
        SheetMode::Off => 0,
        SheetMode::OverLife { cycles } => (t * cycles * frames as f32) as u32 % frames,
        SheetMode::Fps(fps) => (p.pos_age[3] * fps) as u32 % frames,
        SheetMode::Random => ((p.misc[3] * frames as f32) as u32).min(frames - 1),
    };
    (size, color, rotation, frame)
}

/// A particle's rotation at life `t`: its start plus its angular velocity
/// times its age, or with [`ParticleEmitter::spin_over_life`] times the
/// curve's integral over its life (`particle_rotation` in
/// particle_common.wgsl).
pub(crate) fn particle_rotation(e: &ParticleEmitter, p: &ParticleState, t: f32) -> f32 {
    match &e.spin_over_life {
        None => p.misc[1] + p.misc[2] * p.pos_age[3],
        Some(c) => p.misc[1] + p.misc[2] * p.vel_life[3] * c.integral(t),
    }
}

/// The frames an array emitter's particles go through: `sheet.frames` of
/// the array's `layers` (0: every layer).
pub(crate) fn array_frames(sheet: &SpriteSheet, layers: u32) -> u32 {
    match sheet.frames {
        0 => layers,
        n => n.min(layers),
    }
    .max(1)
}

pub(crate) fn sheet_frames(sheet: &SpriteSheet) -> u32 {
    let all = sheet.columns.max(1) * sheet.rows.max(1);
    if sheet.frames == 0 {
        all
    } else {
        sheet.frames.min(all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The area under a curve, against the trapezoids by hand: before the
    /// first key it holds the first value, after the last the last.
    #[test]
    fn a_curve_integrates_its_trapezoids_and_holds_its_ends() {
        let c = Curve::new(&[(0.2, 1.0), (0.6, 3.0), (1.0, 0.0)]);
        // 0..0.2 at 1: 0.2; 0.2..0.6 from 1 to 3: 0.8; 0.6..1 from 3 to 0: 0.6.
        for (t, want) in [
            (0.1, 0.1),
            (0.2, 0.2),
            (0.4, 0.2 + 0.2 * 1.5),
            (0.6, 1.0),
            (1.0, 1.6),
            (1.5, 1.6),
        ] {
            assert!(
                (c.integral(t) - want).abs() < 1e-5,
                "t {t}: {} against {want}",
                c.integral(t)
            );
        }
        assert!((Curve::constant(2.0).integral(0.3) - 0.6).abs() < 1e-6);
    }

    fn live(e: &ParticleEmitter, sys: &mut ParticleSystem, ticks: u64) -> Vec<ParticleState> {
        let mut cpu = CpuParticles::default();
        let points = shape_points(&e.shape, None);
        for t in 0..ticks {
            sys.tick = t;
            cpu.advance(e, sys, 60.0, &points, &[], &[], None);
        }
        cpu.slots.into_iter().filter(|p| p.alive()).collect()
    }

    /// A rate curve shapes the emission: half the cycle at 0 and half at
    /// 2 spawns what the flat rate does over the cycle, none of it in the
    /// first half.
    #[test]
    fn a_rate_curve_shapes_when_particles_are_born() {
        let e = ParticleEmitter {
            rate: 60.0,
            duration: 2.0,
            looping: false,
            lifetime: [10.0, 10.0],
            rate_over_time: Some(Curve::new(&[(0.5, 0.0), (0.5, 2.0)])),
            max_particles: 1024,
            ..Default::default()
        };
        let mut sys = ParticleSystem::new(1, EmitterHandle(0), Mat4::IDENTITY, 1, 0);
        assert_eq!(
            live(&e, &mut sys, 60).len(),
            0,
            "the curve's first half is 0"
        );
        let n = live(&e, &mut sys, 130).len();
        assert!(
            (118..=122).contains(&n),
            "{n} born over the cycle (120 flat)"
        );
    }

    /// An attractor gathers particles, a speed curve of 0 holds them where
    /// they are, a spline sheds them along its curve and a moving emitter
    /// lends them its velocity.
    #[test]
    fn attractors_speed_splines_and_inherited_velocity_move_particles() {
        let base = ParticleEmitter {
            shape: EmitterShape::Sphere {
                radius: 2.0,
                surface: true,
            },
            speed: [0.0, 0.0],
            gravity: Vec3::ZERO,
            lifetime: [5.0, 5.0],
            rate: 0.0,
            bursts: vec![Burst::once(0.0, 100)],
            ..Default::default()
        };
        let mean_r = |ps: &[ParticleState]| {
            ps.iter()
                .map(|p| Vec3::from_slice(&p.pos_age[..3]).length())
                .sum::<f32>()
                / ps.len() as f32
        };
        let mut sys = ParticleSystem::new(1, EmitterHandle(0), Mat4::IDENTITY, 3, 0);
        let still = mean_r(&live(&base, &mut sys, 60));
        let pulled = ParticleEmitter {
            attractors: vec![Attractor {
                position: Vec3::ZERO,
                strength: 4.0,
                radius: 0.0,
            }],
            ..base.clone()
        };
        let gathered = mean_r(&live(&pulled, &mut sys, 60));
        assert!(
            (still - 2.0).abs() < 1e-3 && gathered < 1.7,
            "radius {still} still, {gathered} pulled"
        );
        let held = ParticleEmitter {
            speed_over_life: Some(Curve::constant(0.0)),
            ..pulled
        };
        let h = live(&held, &mut sys, 60);
        assert!((mean_r(&h) - 2.0).abs() < 1e-3, "held at {}", mean_r(&h));
        // The pull at 2 m is 4 / (1 + 4) = 0.8 m/s², for a second.
        let speeds: Vec<f32> = h
            .iter()
            .map(|p| Vec3::from_slice(&p.vel_life[..3]).length())
            .collect();
        assert!(
            speeds.iter().all(|v| (0.7..0.9).contains(v)),
            "the pull still builds velocity: {:?}",
            &speeds[..4]
        );

        // A spline from (0,0,0) to (4,0,0): every particle is born on the x axis, spread along it.
        let mut points = [Vec3::ZERO; MAX_SPLINE_POINTS];
        points[1] = Vec3::new(4.0, 0.0, 0.0);
        let line = ParticleEmitter {
            shape: EmitterShape::Spline {
                points,
                count: 2,
                closed: false,
            },
            ..base.clone()
        };
        let born = live(&line, &mut sys, 1);
        assert!(
            born.iter()
                .all(|p| p.pos_age[1].abs() < 1e-4 && p.pos_age[2].abs() < 1e-4)
        );
        let xs: Vec<f32> = born.iter().map(|p| p.pos_age[0]).collect();
        let (lo, hi) = xs
            .iter()
            .fold((f32::MAX, f32::MIN), |a, &x| (a.0.min(x), a.1.max(x)));
        assert!(lo < 0.5 && hi > 3.5, "spread from {lo} to {hi}");

        // Moving at 6 m/s along x: half of it inherited.
        let thrown = ParticleEmitter {
            shape: EmitterShape::Point,
            rate: 60.0,
            bursts: Vec::new(),
            inherit_velocity: 0.5,
            ..base
        };
        let mut cpu = CpuParticles::default();
        for t in 0..30u64 {
            sys.tick = t;
            sys.transform = Mat4::from_translation(Vec3::new(t as f32 * 0.1, 0.0, 0.0));
            cpu.advance(
                &thrown,
                &sys,
                60.0,
                &[[0.0; 4], [0.0, 1.0, 0.0, 0.0]],
                &[],
                &[],
                None,
            );
        }
        let vx: Vec<f32> = cpu
            .slots
            .iter()
            .filter(|p| p.alive())
            .map(|p| p.vel_life[0])
            .collect();
        assert!(
            vx.len() > 20 && vx.iter().skip(2).all(|v| (v - 3.0).abs() < 1e-3),
            "{vx:?}"
        );
    }

    /// Curl noise is divergence free: its divergence by central
    /// differences is a small fraction of its size (value noise itself,
    /// the turbulence `noise` adds, is not).
    #[test]
    fn curl_noise_has_no_divergence() {
        let h = 0.01;
        let (mut div, mut mag) = (0.0f32, 0.0f32);
        for i in 0..64 {
            let q = Vec3::new(
                i as f32 * 0.37,
                (i % 7) as f32 * 0.61,
                (i % 5) as f32 * 0.43 + 0.1,
            );
            let c = |p: Vec3| curl_noise(p, [0; 3], 9);
            let d = (c(q + Vec3::X * h).x - c(q - Vec3::X * h).x + c(q + Vec3::Y * h).y
                - c(q - Vec3::Y * h).y
                + c(q + Vec3::Z * h).z
                - c(q - Vec3::Z * h).z)
                / (2.0 * h);
            div += d.abs();
            mag += c(q).length();
        }
        assert!(mag > 1.0, "the field is {mag}");
        assert!(div < 0.15 * mag, "divergence {div} against size {mag}");
    }

    /// A fountain: a burst, then 60 a second, each living a second.
    fn fountain() -> ParticleEmitter {
        ParticleEmitter {
            max_particles: 256,
            rate: 60.0,
            bursts: vec![Burst::once(0.0, 20)],
            lifetime: [1.0, 1.0],
            speed: [2.0, 3.0],
            ..Default::default()
        }
    }

    fn run(sys: &mut ParticleSystem, cpu: &mut CpuParticles, ticks: std::ops::RangeInclusive<u64>) {
        let e = fountain();
        for t in ticks {
            sys.tick = t;
            cpu.advance(&e, sys, 60.0, &[], &[], &[], None);
        }
    }

    fn alive(cpu: &CpuParticles) -> Vec<ParticleState> {
        cpu.slots
            .iter()
            .copied()
            .filter(ParticleState::alive)
            .collect()
    }

    /// At a quarter of the pace a system takes four ticks for one: after
    /// 80 ticks it holds what a normal one holds after 20 (the same spawns,
    /// ages and positions to float error), and at 0 it holds still.
    #[test]
    fn a_time_scale_slows_spawning_ageing_and_motion() {
        let mut normal = ParticleSystem::new(1, EmitterHandle(0), Mat4::IDENTITY, 7, 0);
        let mut slow = normal;
        slow.time_scale = 0.25;
        let (mut a, mut b) = (CpuParticles::default(), CpuParticles::default());
        run(&mut normal, &mut a, 0..=19);
        run(&mut slow, &mut b, 0..=79);
        let (a1, b1) = (alive(&a), alive(&b));
        assert_eq!(a1.len(), 20 + 20, "the burst and a third of a second");
        assert_eq!(b1.len(), a1.len());
        // The same particles (by their random values), each spawned within
        // a tick of its twin: a slow step spawns within its quarter tick.
        for (p, q) in a1.iter().zip(&b1) {
            assert_eq!(p.misc, q.misc);
            assert!(
                (p.pos_age[3] - q.pos_age[3]).abs() <= 1.0 / 60.0,
                "{p:?} against {q:?}"
            );
            let d = Vec3::from_slice(&p.pos_age[..3]).distance(Vec3::from_slice(&q.pos_age[..3]));
            assert!(d < 3.0 / 60.0, "{d}: {p:?} against {q:?}");
        }
        // Frozen: nothing spawns, ages or moves.
        slow.time_scale = 0.0;
        run(&mut slow, &mut b, 80..=139);
        assert_eq!(alive(&b), b1);
        // Back to normal: it goes on from where it stood.
        slow.time_scale = 1.0;
        run(&mut slow, &mut b, 140..=159);
        run(&mut normal, &mut a, 20..=39);
        assert_eq!(alive(&b).len(), alive(&a).len());
    }

    /// A death sub-emitter fires `count` particles at each death, where it
    /// happened, every run alike; the events report each death once.
    #[test]
    fn a_death_sub_emitter_fires_where_each_particle_dies() {
        let mut e = fountain();
        e.sub_emitters.push(SubEmitter {
            trigger: ParticleEventKind::Death,
            emitter: EmitterHandle(1),
            count: 3,
            inherit_velocity: 0.0,
        });
        let child = ParticleEmitter {
            shape: EmitterShape::Point,
            rate: 0.0,
            speed: [0.0, 0.0],
            lifetime: [5.0, 5.0],
            gravity: Vec3::ZERO,
            max_particles: 512,
            ..Default::default()
        };
        let go = || {
            let mut sys = ParticleSystem::new(1, EmitterHandle(0), Mat4::IDENTITY, 3, 0);
            let mut cpu = CpuParticles::default();
            let mut events = Vec::new();
            for t in 0..=89 {
                sys.tick = t;
                cpu.advance(&e, &sys, 60.0, &[], &[], &[Some(&child)], Some(&mut events));
            }
            (cpu, events)
        };
        let (a, events) = go();
        let deaths: Vec<&ParticleEvent> = events
            .iter()
            .filter(|ev| ev.kind == ParticleEventKind::Death)
            .collect();
        assert!(
            deaths.len() >= 20,
            "the burst dies after a second: {}",
            deaths.len()
        );
        assert_eq!(a.children[0].born, deaths.len() as u64 * 3);
        // Still children (speed 0, no gravity) sit where their parent died.
        let kids: Vec<Vec3> = a.children[0]
            .slots
            .iter()
            .filter(|s| s.alive())
            .map(|s| Vec3::from_slice(&s.pos_age[..3]))
            .collect();
        for d in &deaths {
            let near = kids
                .iter()
                .filter(|k| k.distance(d.position) < 1e-3)
                .count();
            assert_eq!(near, 3, "three children at {:?}", d.position);
        }
        let (b, again) = go();
        assert_eq!(events, again);
        assert_eq!(a.children[0].slots, b.children[0].slots);
    }
}
