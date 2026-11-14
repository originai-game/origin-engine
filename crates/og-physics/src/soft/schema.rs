//! The reflected types of soft things: PBD soft volumes, garments on
//! skinned characters, Verlet ropes and cloth.

use super::SIMULATION_GAMEPLAY;
use og_core::Reflect;
use og_core::glam::Vec3;
use serde::{Deserialize, Serialize};

/// `SoftBody.shape`: a sphere of `radius` (an icosphere, `detail` 0 to 3).
pub const SOFT_SPHERE: u32 = 0;
/// `SoftBody.shape`: a box of `size` (each face a grid of `detail` + 1 cells).
pub const SOFT_BOX: u32 = 1;
/// `SoftBody.shape`: the closed mesh registered under `mesh` (og.physics.setMesh).
pub const SOFT_MESH: u32 = 2;

/// A PBD soft volume: a closed surface of points (2D: a ring) held by
/// distance constraints along its edges and one constraint on the volume it
/// encloses (2D: its area), dropped under gravity and pushed out of solid
/// colliders. It moves nothing (one-way, as cloth). Built at the entity's
/// world pose when it appears or its shape changes; its points are world
/// state (the og.physics block's trailer), og.physics.softBodyPoints reads
/// them and the renderer draws the surface.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.SoftBody", version = 4)]
pub struct SoftBody {
    /// sphere, box or mesh: SOFT_SPHERE, SOFT_BOX and SOFT_MESH, in order.
    #[reflect(variants = ["sphere", "box", "mesh"])]
    pub shape: u32,
    /// A sphere's radius.
    #[reflect(min = 0.01, unit = "m", default = 0.5, tunable)]
    pub radius: f32,
    /// A box's full extents.
    #[reflect(min = 0, unit = "m", default = [1, 1, 1])]
    pub size: Vec3,
    /// How finely the sphere or box is divided (sphere 0 to 3: 12, 42, 162, 642 points; box 0 to 7 cells a face edge, plus one).
    #[reflect(min = 0, max = 7, default = 2)]
    pub detail: u32,
    /// The registered mesh for SOFT_MESH (closed, its vertices shared by its triangles).
    pub mesh: u64,
    /// How stiff its edges are, 0 to 1 a pass: 1 holds their length, lower stretches.
    #[reflect(min = 0, max = 1, default = 1, tunable)]
    pub stiffness: f32,
    /// The volume it keeps, as a share of its rest volume (the pressure inside): 1 holds it, above inflates, 0 none (it collapses like cloth).
    #[reflect(min = 0, max = 4, default = 1, tunable)]
    pub pressure: f32,
    /// How firmly a pass restores the volume, 0 to 1.
    #[reflect(name = "volumeStiffness", min = 0, max = 1, default = 1, tunable)]
    pub volume_stiffness: f32,
    /// Constraint passes per substep.
    #[reflect(min = 1, max = 64, default = 8, tunable)]
    pub iterations: u32,
    /// Steps per tick.
    #[reflect(min = 1, max = 16, default = 2)]
    pub substeps: u32,
    /// How fast its points' velocity decays (0: none lost).
    #[reflect(min = 0, max = 100, unit = "1/s", default = 0.3, tunable)]
    pub damping: f32,
    /// Multiplies og.Physics.gravity.
    #[reflect(name = "gravityScale", min = -10, max = 10, default = 1, tunable)]
    pub gravity_scale: f32,
    /// The thickness its points collide at (0: no collision).
    #[reflect(min = 0, unit = "m", default = 0.03, tunable)]
    pub thickness: f32,
    /// How much of its sliding a point touching a solid loses each step.
    #[reflect(min = 0, max = 1, default = 0.3, tunable)]
    pub friction: f32,
    /// Points within this height of its top at rest are pinned to the entity (0: none): a hanging water balloon.
    #[reflect(name = "pinTop", min = 0, unit = "m")]
    pub pin_top: f32,
    /// The material it is drawn with; 0 is a plain one.
    pub material: u64,
    /// Whether the renderer draws it.
    #[reflect(default = 1)]
    pub visible: bool,
    /// gameplay (in the tick: SIMULATION_GAMEPLAY) or visual (on the GPU, for looks only: SIMULATION_VISUAL).
    #[reflect(variants = ["gameplay", "visual"])]
    pub simulation: u32,
    /// Written: its volume (2D: area) now over its rest volume: below 1 squashed.
    #[reflect(default = 1)]
    pub volume: f32,
    /// Written: the mean of its points (world space).
    #[reflect(unit = "m")]
    pub center: Vec3,
}

impl Default for SoftBody {
    fn default() -> Self {
        SoftBody {
            shape: SOFT_SPHERE,
            radius: 0.5,
            size: Vec3::ONE,
            detail: 2,
            mesh: 0,
            stiffness: 1.0,
            pressure: 1.0,
            volume_stiffness: 1.0,
            iterations: 8,
            substeps: 2,
            damping: 0.3,
            gravity_scale: 1.0,
            thickness: 0.03,
            friction: 0.3,
            pin_top: 0.0,
            material: 0,
            visible: true,
            simulation: SIMULATION_GAMEPLAY,
            volume: 1.0,
            center: Vec3::ZERO,
        }
    }
}

/// Cloth on a skinned character: glTF mesh `mesh` of the entity's model,
/// posed by its skeleton each tick and simulated from there. Each point
/// stays within `maxDistance` of where its skin puts it (scaled by its
/// vertex colour's red times its alpha; 0 and points within `pinTop` of the
/// top stay on the skin), never further than `backstop` behind it, and keeps its edges'
/// lengths; with `radius` it collides with solids. One-way, as cloth. The
/// runtime hands the skinned pose to [`crate::PhysicsSystem::set_garment_pose`]
/// before each step; its points are world state (the og.physics block's
/// trailer).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Garment", version = 4)]
pub struct Garment {
    /// The glTF mesh of the entity's model that is the garment.
    pub mesh: u32,
    /// How far a point may stray from where its skin puts it.
    #[reflect(name = "maxDistance", min = 0, unit = "m", default = 0.2, tunable)]
    pub max_distance: f32,
    /// Points within this height of the garment's top at rest stay on the skin.
    #[reflect(name = "pinTop", min = 0, unit = "m", default = 0.04, tunable)]
    pub pin_top: f32,
    /// How far a point may go behind where its skin puts it (0: never).
    #[reflect(min = 0, unit = "m", tunable)]
    pub backstop: f32,
    /// How stiff its edges are, 0 to 1 a pass.
    #[reflect(min = 0, max = 1, default = 1, tunable)]
    pub stiffness: f32,
    /// How much it resists folding, 0 to 1 a pass.
    #[reflect(min = 0, max = 1, default = 0.1, tunable)]
    pub bending: f32,
    /// Constraint passes per substep.
    #[reflect(min = 1, max = 64, default = 6, tunable)]
    pub iterations: u32,
    /// Steps per tick.
    #[reflect(min = 1, max = 16, default = 1)]
    pub substeps: u32,
    /// How fast its points' velocity decays (0: none lost).
    #[reflect(min = 0, max = 100, unit = "1/s", default = 1.85, tunable)]
    pub damping: f32,
    /// Multiplies og.Physics.gravity.
    #[reflect(name = "gravityScale", min = -10, max = 10, default = 1, tunable)]
    pub gravity_scale: f32,
    /// A steady acceleration (wind).
    #[reflect(unit = "mps2", tunable)]
    pub wind: Vec3,
    /// How much the wind varies over the garment and the tick.
    #[reflect(min = 0, max = 2, default = 0.6, tunable)]
    pub gust: f32,
    /// The thickness it collides with solids at (0: none).
    #[reflect(min = 0, unit = "m", tunable)]
    pub radius: f32,
    /// How much of its sliding a point touching a solid loses each step.
    #[reflect(min = 0, max = 1, tunable)]
    pub friction: f32,
    /// gameplay (in the tick: SIMULATION_GAMEPLAY) or visual (on the GPU, for looks only: SIMULATION_VISUAL).
    #[reflect(variants = ["gameplay", "visual"])]
    pub simulation: u32,
}

impl Default for Garment {
    fn default() -> Self {
        Garment {
            mesh: 0,
            max_distance: 0.2,
            pin_top: 0.04,
            backstop: 0.0,
            stiffness: 1.0,
            bending: 0.1,
            iterations: 6,
            substeps: 1,
            damping: 1.85,
            gravity_scale: 1.0,
            wind: Vec3::ZERO,
            gust: 0.6,
            radius: 0.0,
            friction: 0.0,
            simulation: SIMULATION_GAMEPLAY,
        }
    }
}

/// A Verlet rope: `segments` links from the entity's world position (its
/// og.Transform after its parents'; plus `offset`) to `end` (an entity, plus `endOffset`) or hanging free, swung
/// by gravity and wind, colliding with every solid collider at `radius`.
/// It moves nothing (one-way: a rope joint ties bodies together, og.Joint
/// JOINT_ROPE). Its points are world state (the og.physics state block);
/// og.physics.ropePoints reads them and the renderer draws the rope as a
/// strip of `width` and `color`.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Rope", version = 2)]
pub struct Rope {
    /// The number of links (points minus one).
    #[reflect(min = 1, max = 256, default = 16)]
    pub segments: u32,
    /// The rope's length at rest.
    #[reflect(min = 0, unit = "m", default = 4, tunable)]
    pub length: f32,
    /// The start's offset from the entity's position (world axes).
    #[reflect(unit = "m")]
    pub offset: Vec3,
    /// The entity the far end is tied to; 0 leaves it free.
    pub end: u64,
    /// The far end's offset from `end`'s position.
    #[reflect(name = "endOffset", unit = "m")]
    pub end_offset: Vec3,
    /// Constraint passes per step: more is stiffer, at more cost.
    #[reflect(min = 1, max = 64, default = 12, tunable)]
    pub iterations: u32,
    /// How fast its points' velocity decays (0: none lost).
    #[reflect(min = 0, max = 100, unit = "1/s", default = 0.6, tunable)]
    pub damping: f32,
    /// Multiplies og.Physics.gravity.
    #[reflect(name = "gravityScale", min = -10, max = 10, default = 1, tunable)]
    pub gravity_scale: f32,
    /// A steady acceleration (wind).
    #[reflect(unit = "m/s2", tunable)]
    pub wind: Vec3,
    /// The thickness it collides at (0: no collision).
    #[reflect(min = 0, unit = "m", default = 0.05, tunable)]
    pub radius: f32,
    /// The drawn width.
    #[reflect(min = 0, unit = "m", default = 0.08)]
    pub width: f32,
    /// The drawn linear colour.
    #[reflect(default = [0.55, 0.42, 0.28])]
    pub color: Vec3,
    /// Whether the renderer draws it.
    #[reflect(default = 1)]
    pub visible: bool,
}

impl Default for Rope {
    fn default() -> Self {
        Rope {
            segments: 16,
            length: 4.0,
            offset: Vec3::ZERO,
            end: 0,
            end_offset: Vec3::ZERO,
            iterations: 12,
            damping: 0.6,
            gravity_scale: 1.0,
            wind: Vec3::ZERO,
            radius: 0.05,
            width: 0.08,
            color: Vec3::new(0.55, 0.42, 0.28),
            visible: true,
        }
    }
}

/// `Cloth.pins` bit: the whole top row is pinned.
pub const PIN_TOP: u32 = 1;
/// `Cloth.pins` bit: the top-left and top-right corners are pinned.
pub const PIN_TOP_CORNERS: u32 = 2;
/// `Cloth.pins` bit: the whole left column is pinned (a flag on a pole).
pub const PIN_LEFT: u32 = 4;
/// `Cloth.pins` bit: every fourth point of the top row (curtain rings).
pub const PIN_TOP_RINGS: u32 = 8;

/// A Verlet cloth: a `columns` × `rows` grid of points, `spacing` apart,
/// hanging from the entity's world position (its og.Transform after its
/// parents': its top-left corner, the grid laid out along the entity's
/// local x and down -y), pinned by `pins` (pinned
/// points follow the entity), with stretch and shear constraints, gravity,
/// wind, collision against every solid collider and, with `thickness`,
/// against itself. One-way like og.Rope;
/// its points are world state. Drawn as a double-sided mesh with
/// `material` (og.render.standardMaterial; 0: a plain cloth colour).
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Cloth", version = 4)]
pub struct Cloth {
    /// Points across.
    #[reflect(min = 2, max = 64, default = 12)]
    pub columns: u32,
    /// Points down.
    #[reflect(min = 2, max = 64, default = 12)]
    pub rows: u32,
    /// The rest distance between neighbouring points.
    #[reflect(min = 0.001, unit = "m", default = 0.15, tunable)]
    pub spacing: f32,
    /// Which points are pinned, a sum of PIN_* bits (1 top row, 2 top corners, 4 left column, 8 top rings).
    #[reflect(default = 1)]
    pub pins: u32,
    /// Constraint passes per step.
    #[reflect(min = 1, max = 64, default = 8, tunable)]
    pub iterations: u32,
    /// How fast its points' velocity decays (0: none lost).
    #[reflect(min = 0, max = 100, unit = "1/s", default = 0.9, tunable)]
    pub damping: f32,
    /// Multiplies og.Physics.gravity.
    #[reflect(name = "gravityScale", min = -10, max = 10, default = 1, tunable)]
    pub gravity_scale: f32,
    /// A steady acceleration (wind); the cloth flutters with it.
    #[reflect(unit = "m/s2", tunable)]
    pub wind: Vec3,
    /// How much the wind varies over the cloth and the tick (0: steady).
    #[reflect(min = 0, max = 2, default = 0.6, tunable)]
    pub gust: f32,
    /// The thickness it collides at (0: no collision).
    #[reflect(min = 0, unit = "m", default = 0.04, tunable)]
    pub radius: f32,
    /// Self-collision: how far apart its points keep from the points of the
    /// cloth that are not their near neighbours (0: none). About `spacing`
    /// keeps a cloth that folds or piles up from passing through itself.
    #[reflect(min = 0, unit = "m", tunable)]
    pub thickness: f32,
    /// Simulation steps per tick, each integrating and then solving
    /// `iterations` passes; with self-collision enough that a point moves
    /// less than `thickness` per step (a point moves at most half the
    /// thickness a step).
    #[reflect(min = 1, max = 16, default = 1)]
    pub substeps: u32,
    /// How much of its sliding a point touching a solid loses each step
    /// (0: it slides freely; near 1 it sticks where it lands, so a cloth
    /// lowered onto the floor folds instead of spreading).
    #[reflect(min = 0, max = 1, tunable)]
    pub friction: f32,
    /// The material it is drawn with (og.render.standardMaterial); 0 is a plain cloth.
    pub material: u64,
    /// Whether the renderer draws it.
    #[reflect(default = 1)]
    pub visible: bool,
    /// gameplay (in the tick: SIMULATION_GAMEPLAY) or visual (on the GPU, for looks only: SIMULATION_VISUAL).
    #[reflect(variants = ["gameplay", "visual"])]
    pub simulation: u32,
}

impl Default for Cloth {
    fn default() -> Self {
        Cloth {
            columns: 12,
            rows: 12,
            spacing: 0.15,
            pins: PIN_TOP,
            iterations: 8,
            damping: 0.9,
            gravity_scale: 1.0,
            wind: Vec3::ZERO,
            gust: 0.6,
            radius: 0.04,
            thickness: 0.0,
            substeps: 1,
            friction: 0.0,
            material: 0,
            visible: true,
            simulation: SIMULATION_GAMEPLAY,
        }
    }
}
