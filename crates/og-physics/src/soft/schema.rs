//! Reflected small-step XPBD soft volumes, skinned garments, ropes and cloth.

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

/// An XPBD soft volume: a closed surface of points (2D: a ring) held by
/// distance constraints along its edges and one constraint on the volume it
/// encloses (2D: its area), dropped under gravity and pushed out of solid
/// colliders, with reaction impulses on dynamic bodies. Built at the entity's
/// world pose when it appears or its shape changes; its points are world
/// state (the og.physics block's trailer), og.physics.softBodyPoints reads
/// them and the renderer draws the surface.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.SoftBody", version = 5)]
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
    /// Physical XPBD edge stiffness in N/m (0: off), compliance is 1/stiffness.
    #[reflect(min = 0, default = 100000, tunable)]
    pub stiffness: f32,
    /// The volume it keeps, as a share of its rest volume (the pressure inside): 1 holds it, above inflates, 0 none (it collapses like cloth).
    #[reflect(min = 0, max = 4, default = 1, tunable)]
    pub pressure: f32,
    /// Physical XPBD volume stiffness (0: off).
    #[reflect(name = "volumeStiffness", min = 0, default = 100000, tunable)]
    pub volume_stiffness: f32,
    /// Per-tetrahedron volume stiffness about the centroid (0: off).
    #[reflect(name = "tetraStiffness", min = 0, tunable)]
    pub tetra_stiffness: f32,
    /// Surface mass density in kg/m² (0: fixed).
    #[reflect(min = 0, default = 1)]
    pub density: f32,
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
            stiffness: 100000.0,
            pressure: 1.0,
            volume_stiffness: 100000.0,
            tetra_stiffness: 0.0,
            density: 1.0,
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
/// top stay on the skin), with spherical backstops, skeleton capsules and
/// compliant stretch and dihedral bending. Solid contacts transfer momentum. The
/// runtime hands the skinned pose to [`crate::PhysicsSystem::set_garment_pose`]
/// before each step; its points are world state (the og.physics block's
/// trailer).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Garment", version = 5)]
pub struct Garment {
    /// The glTF mesh of the entity's model that is the garment.
    pub mesh: u32,
    /// How far a point may stray from where its skin puts it.
    #[reflect(name = "maxDistance", min = 0, unit = "m", default = 0.2, tunable)]
    pub max_distance: f32,
    /// Points within this height of the garment's top at rest stay on the skin.
    #[reflect(name = "pinTop", min = 0, unit = "m", default = 0.04, tunable)]
    pub pin_top: f32,
    /// Offset behind the skinned target of an enabled spherical backstop.
    #[reflect(min = 0, unit = "m", tunable)]
    pub backstop: f32,
    /// Spherical backstop radius, scaled by the pose (0: off).
    #[reflect(name = "backstopRadius", min = 0, unit = "m")]
    pub backstop_radius: f32,
    /// Radius of skeleton bone capsules in model metres (0: off).
    #[reflect(name = "bodyRadius", min = 0, unit = "m")]
    pub body_radius: f32,
    /// Reset on target translations larger than this, scaled by the pose (0: off).
    #[reflect(name = "teleportDistance", min = 0, unit = "m")]
    pub teleport_distance: f32,
    /// Surface mass density in kg/m² (0: fixed).
    #[reflect(min = 0, default = 1)]
    pub density: f32,
    /// Relative-air quadratic drag (0: off).
    #[reflect(min = 0, tunable)]
    pub drag: f32,
    /// Relative-air lift (0: off).
    #[reflect(min = 0, tunable)]
    pub lift: f32,
    /// Physical XPBD edge stiffness in N/m (0: off).
    #[reflect(min = 0, default = 100000, tunable)]
    pub stiffness: f32,
    /// Physical XPBD dihedral stiffness (0: off).
    #[reflect(min = 0, default = 0.1, tunable)]
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
    /// Air velocity added to the per-point wind field.
    #[reflect(unit = "m/s", tunable)]
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
            backstop_radius: 0.0,
            body_radius: 0.0,
            teleport_distance: 0.0,
            density: 1.0,
            drag: 0.0,
            lift: 0.0,
            stiffness: 100000.0,
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

/// An XPBD rope: `segments` links from the entity's world position (its
/// og.Transform after its parents'; plus `offset`) to `end` (an entity, plus `endOffset`) or hanging free, swung
/// by gravity and wind, colliding with every solid collider at `radius`.
/// With `coupling` it transfers constraint and contact impulses to dynamic
/// bodies. Its points are world state (the og.physics state block);
/// og.physics.ropePoints reads them and the renderer draws the rope as a
/// strip of `width` and `color`.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Rope", version = 3)]
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
    /// XPBD axial stiffness in N/m (0: constraints off).
    #[reflect(min = 0, default = 100000, tunable)]
    pub stiffness: f32,
    /// Mass of each free point in kg (0: fixed).
    #[reflect(min = 0, default = 0.1)]
    pub mass: f32,
    /// Small steps per tick (0: simulation off).
    #[reflect(min = 0, max = 16, default = 4)]
    pub substeps: u32,
    /// Apply endpoint and solid contact reactions to dynamic bodies.
    pub coupling: bool,
    /// Break force in N (0: unbreakable); broken links stay broken in state.
    #[reflect(name = "breakForce", min = 0)]
    pub break_force: f32,
    /// XPBD constraint passes per substep (compliance stays physical).
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
            stiffness: 100000.0,
            mass: 0.1,
            substeps: 4,
            coupling: false,
            break_force: 0.0,
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

/// An XPBD cloth: a `columns` × `rows` grid of points, `spacing` apart,
/// hanging from the entity's world position (its og.Transform after its
/// parents': its top-left corner, the grid laid out along the entity's
/// local x and down -y), pinned by `pins` (pinned
/// points follow the entity), with stretch and shear constraints, gravity,
/// wind, collision against every solid collider and, with `thickness`,
/// against itself. Contacts transfer momentum to dynamic bodies;
/// its points are world state. Drawn as a double-sided mesh with
/// `material` (og.render.standardMaterial; 0: a plain cloth colour).
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Cloth", version = 5)]
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
    /// XPBD stretch stiffness in N/m (0: off).
    #[reflect(min = 0, default = 100000, tunable)]
    pub stiffness: f32,
    /// XPBD dihedral stiffness (0: bending off).
    #[reflect(min = 0, tunable)]
    pub bending: f32,
    /// Long-range attachment stiffness in N/m (0: off).
    #[reflect(min = 0, tunable)]
    pub tether: f32,
    /// Surface mass density in kg/m² (0: fixed).
    #[reflect(min = 0, default = 1)]
    pub density: f32,
    /// Quadratic relative-air drag coefficient (0: off).
    #[reflect(min = 0, tunable)]
    pub drag: f32,
    /// Relative-air lift coefficient (0: off).
    #[reflect(min = 0, tunable)]
    pub lift: f32,
    /// Constraint passes per step.
    #[reflect(min = 1, max = 64, default = 8, tunable)]
    pub iterations: u32,
    /// How fast its points' velocity decays (0: none lost).
    #[reflect(min = 0, max = 100, unit = "1/s", default = 0.9, tunable)]
    pub damping: f32,
    /// Multiplies og.Physics.gravity.
    #[reflect(name = "gravityScale", min = -10, max = 10, default = 1, tunable)]
    pub gravity_scale: f32,
    /// Air velocity added to the per-point wind field; drag/lift enable it.
    #[reflect(unit = "m/s", tunable)]
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
    /// `iterations` passes; choose enough to resolve self-contact. There
    /// is no artificial speed limit. Zero pauses simulation.
    #[reflect(min = 0, max = 16, default = 1)]
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
            stiffness: 100000.0,
            bending: 0.0,
            tether: 0.0,
            density: 1.0,
            drag: 0.0,
            lift: 0.0,
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
