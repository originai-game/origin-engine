//! The reflected types of the particle fluids: og.Fluid2D and og.Fluid3D.

use super::SIMULATION_GAMEPLAY;
use og_core::Reflect;
use og_core::glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// A 2D particle fluid (any profile, in the xy plane): `count` particles
/// spawned in a `size` box centred on the entity, simulated with double
/// density relaxation (Clavet et al. 2005: pressure, near pressure,
/// viscosity) on a fixed grid in a fixed order, so it is deterministic and
/// part of the og.physics state block. Particles collide with every solid
/// collider (pushed out of bodies; they push nothing back) and stay inside
/// `bounds` around the entity when it is set. Fluids of one `group`
/// interact: by `density` they layer, by `mixing` they blend. Drawn as metaballs: the
/// particles blurred into a field the renderer thresholds into one surface
/// of `color`, on sprite layer `layer`.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Fluid2D")]
pub struct Fluid2D {
    /// How many particles (spawned when the component appears or count changes).
    #[reflect(min = 1, max = 8192, default = 1000)]
    pub count: u32,
    /// The spawn box, centred on the entity.
    #[reflect(unit = "m", default = [4, 3])]
    pub size: Vec2,
    /// The container around the entity (0 0: none; colliders still hold it).
    #[reflect(unit = "m")]
    pub bounds: Vec2,
    /// The interaction radius: particles closer than this push and drag each other.
    #[reflect(min = 0.01, unit = "m", default = 0.18, tunable)]
    pub radius: f32,
    /// The density it settles at (neighbours within radius, weighted).
    #[reflect(name = "restDensity", min = 0, default = 1, tunable)]
    pub rest_density: f32,
    /// The pressure's strength (above about 0.15 it gets springy, then unstable).
    #[reflect(min = 0, default = 0.1, tunable)]
    pub stiffness: f32,
    /// The near pressure's strength (keeps particles apart, gives surface tension).
    #[reflect(name = "nearStiffness", min = 0, default = 0.5, tunable)]
    pub near_stiffness: f32,
    /// Linear viscosity (honey above 1).
    #[reflect(min = 0, max = 4, default = 0.15, tunable)]
    pub viscosity: f32,
    /// Fluids sharing a group above 0 push, drag and (by `mixing`) mix with
    /// each other as one fluid, at most four to a group (in entity order;
    /// a fifth is simulated alone), and are drawn as one surface whose
    /// colour follows the mix; 0 keeps the fluid to itself.
    pub group: u32,
    /// How heavy its particles are against the group's other fluids:
    /// a heavier fluid sinks under a lighter one.
    #[reflect(min = 0.05, max = 20, default = 1, tunable)]
    pub density: f32,
    /// How readily it mixes with the group's other fluids: 0 keeps them
    /// apart in layers (the boundary held), 1 blends them where they touch
    /// (their mix, colour and weight even out).
    #[reflect(min = 0, max = 1, tunable)]
    pub mixing: f32,
    /// Multiplies og.Physics.gravity.
    #[reflect(name = "gravityScale", min = -10, max = 10, default = 1, tunable)]
    pub gravity_scale: f32,
    /// Simulation steps per tick.
    #[reflect(min = 1, max = 8, default = 2)]
    pub substeps: u32,
    /// The seed of the spawn jitter.
    pub seed: u32,
    /// The surface's linear colour.
    #[reflect(default = [0.15, 0.45, 0.95])]
    pub color: Vec3,
    /// The surface's opacity.
    #[reflect(min = 0, max = 1, default = 0.9)]
    pub opacity: f32,
    /// The drawn size of each particle's blob (times radius).
    #[reflect(name = "blobScale", min = 0.1, max = 4, default = 1.6)]
    pub blob_scale: f32,
    /// The field value where the surface starts (higher: thinner, separate drops).
    #[reflect(min = 0.01, max = 4, default = 0.45)]
    pub threshold: f32,
    /// The sprite layer it is drawn on.
    pub layer: i32,
    /// Whether the renderer draws it.
    #[reflect(default = 1)]
    pub visible: bool,
}

impl Default for Fluid2D {
    fn default() -> Self {
        Fluid2D {
            count: 1000,
            size: Vec2::new(4.0, 3.0),
            bounds: Vec2::ZERO,
            radius: 0.18,
            rest_density: 1.0,
            stiffness: 0.1,
            near_stiffness: 0.5,
            viscosity: 0.15,
            group: 0,
            density: 1.0,
            mixing: 0.0,
            gravity_scale: 1.0,
            substeps: 2,
            seed: 0,
            color: Vec3::new(0.15, 0.45, 0.95),
            opacity: 0.9,
            blob_scale: 1.6,
            threshold: 0.45,
            layer: 0,
            visible: true,
        }
    }
}

/// A 3D particle fluid (the 3D profile): `count` particles spawned in a
/// `size` box centred on the entity, simulated with double density
/// relaxation (Clavet et al. 2005: pressure, near pressure that gives
/// surface tension, viscosity) over a spatial hash in a fixed order, so it
/// is deterministic and part of the og.physics state block. Particles
/// collide with every solid collider, the terrain's height field included
/// (pushed out; they push nothing back), and stay inside `bounds` around
/// the entity when it is set. An emitter adds particles at `emitRate` up
/// to `maxCount`; a sink box removes those that enter it. Fluids of one
/// `group` interact: by `density` they layer, by `mixing` they blend.
/// Drawn as a screen-space surface: the particles' depth smoothed into one
/// surface, refracting what is behind by its thickness and reflecting the
/// sky, lit by the scene's lights.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Fluid3D", version = 3)]
pub struct Fluid3D {
    /// How many particles to spawn in the box (when the component appears
    /// or count or seed changes; 0: none, the emitter fills it). The tick
    /// simulates at most 32768; a visual fluid up to 262144.
    #[reflect(max = 262144, default = 2000)]
    pub count: u32,
    /// The spawn box, centred on the entity.
    #[reflect(unit = "m", default = [2, 1, 2])]
    pub size: Vec3,
    /// The container around the entity (0 on an axis: open on that axis;
    /// colliders and the terrain still hold it).
    #[reflect(unit = "m")]
    pub bounds: Vec3,
    /// The interaction radius: particles closer than this push and drag
    /// each other. Cost grows with the particles, not the radius.
    #[reflect(min = 0.02, unit = "m", default = 0.5, tunable)]
    pub radius: f32,
    /// The density it settles at (neighbours within radius, weighted).
    #[reflect(name = "restDensity", min = 0, default = 10, tunable)]
    pub rest_density: f32,
    /// The pressure's strength (above about 0.05 it jitters, then springs apart).
    #[reflect(min = 0, default = 0.02, tunable)]
    pub stiffness: f32,
    /// The near pressure's strength (keeps particles apart, gives surface tension).
    #[reflect(name = "nearStiffness", min = 0, default = 0.4, tunable)]
    pub near_stiffness: f32,
    /// Linear viscosity (it also calms the surface; honey above 1).
    #[reflect(min = 0, max = 4, default = 0.4, tunable)]
    pub viscosity: f32,
    /// Fluids sharing a group above 0 push, drag and (by `mixing`) mix with
    /// each other as one fluid, at most four to a group (in entity order;
    /// a fifth is simulated alone); 0 keeps the fluid to itself.
    pub group: u32,
    /// How heavy its particles are against the group's other fluids: a
    /// heavier fluid sinks under a lighter one.
    #[reflect(min = 0.05, max = 20, default = 1, tunable)]
    pub density: f32,
    /// How readily it mixes with the group's other fluids: 0 keeps them
    /// apart in layers (the boundary held), 1 blends them where they touch
    /// (their mix, colour and weight even out).
    #[reflect(min = 0, max = 1, tunable)]
    pub mixing: f32,
    /// Multiplies og.Physics.gravity.
    #[reflect(name = "gravityScale", min = -10, max = 10, default = 1, tunable)]
    pub gravity_scale: f32,
    /// Simulation steps per tick (each costs a full pass over the particles).
    #[reflect(min = 1, max = 8, default = 2)]
    pub substeps: u32,
    /// The seed of the spawn and emitter jitter.
    pub seed: u32,
    /// The most particles alive at once (the emitter stops there).
    #[reflect(name = "maxCount", min = 1, max = 262144, default = 8192)]
    pub max_count: u32,
    /// Particles the emitter adds per second (0: no emitter).
    #[reflect(name = "emitRate", min = 0, max = 20000, tunable)]
    pub emit_rate: f32,
    /// Where the emitter is, from the entity (world axes).
    #[reflect(name = "emitOffset", unit = "m")]
    pub emit_offset: Vec3,
    /// The velocity emitted particles start with.
    #[reflect(name = "emitVelocity", unit = "m/s", tunable)]
    pub emit_velocity: Vec3,
    /// The half-size of the cube new particles appear in around the emitter.
    #[reflect(name = "emitRadius", min = 0, unit = "m", default = 0.1, tunable)]
    pub emit_radius: f32,
    /// The sink's centre, from the entity (world axes).
    #[reflect(name = "sinkOffset", unit = "m")]
    pub sink_offset: Vec3,
    /// The sink box: particles that enter it are removed (0 0 0: no sink).
    #[reflect(name = "sinkSize", unit = "m")]
    pub sink_size: Vec3,
    /// The fluid's linear colour (what light passing through it keeps).
    #[reflect(default = [0.2, 0.55, 0.9])]
    pub color: Vec3,
    /// How opaque a metre of it is (0: clear as glass, 1: opaque).
    #[reflect(min = 0, max = 1, default = 0.6)]
    pub opacity: f32,
    /// How much it reflects (scales the Fresnel term).
    #[reflect(min = 0, max = 2, default = 1)]
    pub reflectivity: f32,
    /// How far refraction bends what is seen through it.
    #[reflect(min = 0, max = 4, default = 1)]
    pub refraction: f32,
    /// The drawn size of each particle (times radius).
    #[reflect(name = "drawScale", min = 0.1, max = 2, default = 0.6)]
    pub draw_scale: f32,
    /// How much the surface is smoothed (0: beads, 1: one surface).
    #[reflect(min = 0, max = 2, default = 1)]
    pub smoothing: f32,
    /// Whether the renderer draws it.
    #[reflect(default = 1)]
    pub visible: bool,
    /// gameplay (in the tick: SIMULATION_GAMEPLAY) or visual (on the GPU, for looks only: SIMULATION_VISUAL).
    #[reflect(variants = ["gameplay", "visual"])]
    pub simulation: u32,
}

impl Default for Fluid3D {
    fn default() -> Self {
        Fluid3D {
            count: 2000,
            size: Vec3::new(2.0, 1.0, 2.0),
            bounds: Vec3::ZERO,
            radius: 0.2,
            rest_density: 10.0,
            stiffness: 0.02,
            near_stiffness: 0.4,
            viscosity: 0.4,
            group: 0,
            density: 1.0,
            mixing: 0.0,
            gravity_scale: 1.0,
            substeps: 2,
            seed: 0,
            max_count: 8192,
            emit_rate: 0.0,
            emit_offset: Vec3::ZERO,
            emit_velocity: Vec3::ZERO,
            emit_radius: 0.1,
            sink_offset: Vec3::ZERO,
            sink_size: Vec3::ZERO,
            color: Vec3::new(0.2, 0.55, 0.9),
            opacity: 0.6,
            reflectivity: 1.0,
            refraction: 1.0,
            draw_scale: 0.6,
            smoothing: 1.0,
            visible: true,
            simulation: SIMULATION_GAMEPLAY,
        }
    }
}
