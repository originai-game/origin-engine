//! Origin Engine physics: rapier3d and rapier2d as an engine system over the
//! ECS (docs/design.md §2.4, §2.7; docs/phase-1.md M6).
//!
//! | item | what it is |
//! |---|---|
//! | [`install`] | registers the types, inserts the [`Physics`] settings and schedules the `og.physics` system; returns a [`PhysicsSystem`] |
//! | [`RigidBody`], [`Collider`], [`Velocity`] | the components (`og.RigidBody`, `og.Collider`, `og.Velocity`); bodies also need `og.Transform` |
//! | [`Physics`] | the resource `og.Physics`: profile (2 or 3), gravity, substeps, solver iterations |
//! | [`CollisionStarted`], [`CollisionEnded`], [`SensorEntered`], [`SensorExited`], [`Contact`] | the events, read the tick after the step that sent them |
//! | [`PhysicsSystem`] | the system's handle: [`PhysicsSystem::set_mesh`] for convex and triangle-mesh shapes, [`PhysicsSystem::query`] for scene queries |
//! | [`PhysicsQuery`], [`raycast`], [`shape_cast`], [`overlap`] | raycast, shape cast and overlap over the colliders as the world holds them |
//! | [`Joint`] | fixed, hinge (limits, motor), slider, spring, rope and ball joints between bodies or to the world |
//! | [`CharacterController`], [`PlatformerController`], [`CharacterEvent`] | kinematic characters moved after the step: slopes, steps, snapping, platforms; the 2D platformer's coyote time, buffer, variable jump, wall jumps, one-way platforms, corner correction |
//! | [`Vehicle`], [`WheelPose`] | the 3D raycast car and the 2D wheel-joint car; [`PhysicsSystem::vehicle_wheels`] |
//! | [`Rope`], [`Cloth`] | Verlet ropes and cloth pinned to entities, colliding with solids; [`PhysicsSystem::rope_points`], [`PhysicsSystem::cloth_points`] |
//! | [`SoftBody`] | PBD soft volumes (edges and a volume constraint), colliding with solids; [`PhysicsSystem::soft_body_points`] |
//! | [`Fluid2D`] | a 2D particle fluid (double density relaxation); [`PhysicsSystem::fluid_particles`] |
//! | [`Fluid3D`] | a 3D particle fluid (double density relaxation, spatial hash); [`PhysicsSystem::fluid3d_particles`] |
//! | [`WaterSurface`], [`WaterLine`], [`Splash`] | height-field water (3D) and a 2D spring line, splashed by and floating bodies |
//! | [`Ragdoll`], [`RagdollRig`] | capsule bones and joints (ball joints in 3D, hinges about z in 2D) from a host-registered rig, blended between animation and physics |
//!
//! Everything beyond rigid bodies lives in the same rapier world and the
//! same state block: joints and the wheel and bone bodies in rapier's
//! sets, the ropes, cloth, fluid particles and water heights beside them in
//! the saved simulation, the controllers' timers in their components. The
//! order of a tick: sync the bodies, joints, vehicles and ragdolls; water
//! pushes and floats the bodies in it; the rigid-body step; then the
//! controllers move against the world as it now is, and ropes, cloth and
//! fluids follow (`sim::step`).
//!
//! **Determinism.** rapier is built with `enhanced-determinism`. The rapier
//! world persists between ticks (contacts and solver warm starts carry over)
//! and is world state: the `og.physics` state block ([`og_ecs::StateBlock`])
//! saves it, with what each entity's components were when physics last
//! touched them, so `World::snapshot`/`restore` and `world_hash` cover the
//! whole simulation and a world restored at any tick continues exactly as
//! the uninterrupted run. Each tick the system first brings the rapier world
//! in line with `og.Transform`, `og.RigidBody`, `og.Collider` and
//! `og.Velocity` (in query order: archetype creation order, then row order):
//! despawned bodies leave, new ones enter, a changed body, collider or scale
//! rebuilds that body, and a transform or velocity that game code changed
//! moves it. Then it steps and writes the dynamic bodies back. A restore of
//! a snapshot without the block (and a tick that failed) rebuilds the rapier
//! world from the components. The touching pairs a step ends with are
//! carried to the next tick as [`Contact`] events, which is how Started/Ended
//! are derived.
//!
//! The block carries the geometry of convex-hull, triangle-mesh and
//! height-field colliders by its asset key (shape, handle, scale), not its
//! triangles: meshes are assets, and decoding rebuilds those shapes from
//! the meshes registered then. Forces and impulses game code queued for the
//! next step ([`PhysicsSystem::apply_impulse`] and friends) are in it too.

#[cfg(not(any(feature = "physics2d", feature = "physics3d")))]
compile_error!("og-physics needs the `physics2d` or the `physics3d` feature (both are default)");

mod drivetrain;
mod fluid;
mod fluid3d;
mod garment;
mod gather;
mod lanes;
mod outline;
mod schema;
mod schema_ext;
mod softbody;
mod stats;
mod verlet;
mod water;
pub use water::{Wakes, WaterCut};

#[cfg(feature = "physics2d")]
mod dim2;
#[cfg(feature = "physics3d")]
mod dim3;

pub use garment::{GarmentPoints, GarmentPose, GarmentRig};
pub use lanes::{Grains, grains, pool_bodies, set_grains};
pub use outline::outline;
pub use schema::*;
pub use schema_ext::*;
pub use stats::{Phase, PhaseStats, PhysicsStats, set_clock};

pub use gather::world_pose;
use gather::{Ids, QUERIES, Scene};
use og_core::Reflect as _;
use og_core::glam::{Quat, Vec2, Vec3};
use og_core::{Error, ErrorKind, IdMap, Result};
use og_ecs::{Entity, StateBlock, SystemContext, SystemDesc, World};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

/// The name of the scheduled system.
pub const SYSTEM_NAME: &str = "og.physics";

/// Geometry for convex and triangle-mesh colliders, registered under the
/// mesh handle the colliders name ([`Collider::mesh`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshDesc {
    /// Vertex positions in the mesh's own space (2D polygons use x and y).
    pub positions: Vec<Vec3>,
    /// Triangles as indices into `positions` (a convex hull ignores them).
    pub indices: Vec<[u32; 3]>,
}

/// The bit of a mesh handle that marks a mesh generated from world state
/// (a tilemap's merged solid regions, G2-6) rather than loaded as an asset:
/// the shapes of colliders naming one are saved with the world, so a save
/// restores without the mesh registered. Asset handles never set it, and
/// a handle with it still fits a JavaScript number (below 2^53).
pub const GENERATED_MESH: u64 = 1 << 52;

/// A height field for [`SHAPE_HEIGHTFIELD`] colliders, registered under the
/// handle the colliders name ([`Collider::mesh`]): `columns` × `rows`
/// heights over a `size` rectangle centred on the collider, row-major from
/// -z to +z and -x to +x (og-render's `TerrainData`). Each cell is split
/// along the diagonal with the smaller corner height difference; ties keep
/// (-x, -z) to (+x, +z), as the renderer draws it.
#[derive(Clone, Debug, PartialEq)]
pub struct HeightfieldDesc {
    pub columns: u32,
    pub rows: u32,
    pub size: Vec2,
    pub heights: Arc<[f32]>,
    /// The cells cut out (a terrain's holes): `(columns - 1)` ×
    /// `(rows - 1)` bytes, row-major like the heights, non-zero removed;
    /// nothing collides with a removed cell. `None`: none.
    pub holes: Option<Arc<[u8]>>,
}

/// The shape of a shape cast or an overlap test: the shape fields of a
/// [`Collider`]. `Default` is a 1 m box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeDesc {
    /// One of the `SHAPE_*` codes ([`SHAPE_BOX`], [`SHAPE_SPHERE`], ...).
    pub shape: u32,
    pub size: Vec3,
    pub radius: f32,
    pub height: f32,
    pub mesh: u64,
}

impl Default for ShapeDesc {
    fn default() -> Self {
        let c = Collider::default();
        ShapeDesc {
            shape: c.shape,
            size: c.size,
            radius: c.radius,
            height: c.height,
            mesh: c.mesh,
        }
    }
}

impl ShapeDesc {
    pub(crate) fn collider(&self) -> Collider {
        Collider {
            shape: self.shape,
            size: self.size,
            radius: self.radius,
            height: self.height,
            mesh: self.mesh,
            ..Collider::default()
        }
    }
}

/// Which colliders a query considers. The zero value considers every solid
/// collider on every layer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QueryFilter {
    /// Layers to consider (a collider counts when its `layers` meet this); 0 means every layer.
    pub mask: u32,
    /// Also report sensors.
    pub include_sensors: bool,
    /// An entity to ignore (the caster itself, say).
    pub exclude: Option<Entity>,
}

/// A ray. `max_distance` 0 means unbounded.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RaycastDesc {
    pub origin: Vec3,
    /// Need not be normalised; must not be zero.
    pub direction: Vec3,
    pub max_distance: f32,
    pub filter: QueryFilter,
}

/// The nearest collider a ray meets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RaycastHit {
    pub entity: Entity,
    /// The body the collider belongs to: the entity itself, or for a
    /// compound child collider the ancestor body it is part of.
    pub body: Entity,
    /// Distance from the origin along the normalised direction.
    pub distance: f32,
    pub point: Vec3,
    /// The surface normal at `point` (z = 0 in 2D).
    pub normal: Vec3,
}

/// Every collider along a ray ([`PhysicsQuery::raycast_all`]): the ray, and
/// at most `max_hits` hits (0: [`MAX_RAYCAST_HITS`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RaycastAllDesc {
    pub ray: RaycastDesc,
    pub max_hits: u32,
}

/// The most hits [`PhysicsQuery::raycast_all`] returns.
pub const MAX_RAYCAST_HITS: u32 = 256;

/// The nearest point on a collider to a point ([`PhysicsQuery::project_point`]).
/// `max_distance` 0 means unbounded; `solid`: a point inside a collider is
/// its own projection (else it projects onto the surface).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProjectPointDesc {
    pub point: Vec3,
    pub max_distance: f32,
    pub solid: bool,
    pub filter: QueryFilter,
}

/// Where a point projects onto the nearest collider.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointProjection {
    pub entity: Entity,
    pub body: Entity,
    /// The projected point, in world space.
    pub point: Vec3,
    /// The direction from the surface towards the queried point (out of the
    /// collider; into it when the point is inside); zero when the point is
    /// on the surface or a solid projection of a point inside.
    pub normal: Vec3,
    /// From the queried point to the projection.
    pub distance: f32,
    /// Whether the point is inside the collider.
    pub inside: bool,
}

/// A shape swept along a direction. `max_distance` 0 means unbounded.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShapeCastDesc {
    pub shape: ShapeDesc,
    pub position: Vec3,
    /// Zero is treated as no rotation.
    pub rotation: Quat,
    /// Need not be normalised; must not be zero.
    pub direction: Vec3,
    pub max_distance: f32,
    pub filter: QueryFilter,
}

/// The first collider a swept shape meets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeCastHit {
    pub entity: Entity,
    /// How far the shape travelled before touching.
    pub distance: f32,
    /// The contact point on the hit collider, in world space.
    pub point: Vec3,
    /// The hit collider's surface normal there, in world space.
    pub normal: Vec3,
}

/// A shape placed in the world, for an overlap test.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OverlapDesc {
    pub shape: ShapeDesc,
    pub position: Vec3,
    /// Zero is treated as no rotation.
    pub rotation: Quat,
    pub filter: QueryFilter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct ShapeKey {
    pub shape: u32,
    pub mesh: u64,
    pub scale: [u32; 3],
}

/// The key of chunk `chunk` of the height field `key` names
/// ([`FIELD_CHUNK`]): the chunk plus 1 in the shape code's bits from 8
/// (a save's [`Hole`] names a chunk so); a code with none there is the
/// whole field.
pub(crate) fn chunk_key(key: ShapeKey, chunk: u32) -> ShapeKey {
    ShapeKey {
        shape: SHAPE_HEIGHTFIELD | ((chunk + 1) << 8),
        ..key
    }
}

/// The chunk a shape code names ([`chunk_key`]), if it is a chunk's.
pub(crate) fn key_chunk(shape: u32) -> Option<u32> {
    (shape & 0xff == SHAPE_HEIGHTFIELD && shape >> 8 != 0).then(|| (shape >> 8) - 1)
}

/// Where a saved simulation's bytes lack an asset shape: at byte `at` of
/// the cut bytes, the shape `key` names (rebuilt from the registered mesh
/// or height field when the save is decoded; `sim::save`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Hole {
    pub at: u64,
    pub key: ShapeKey,
}

/// What a [`Push`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PushKind {
    /// A force (N) over the next step.
    Force,
    /// An impulse (N·s) at once.
    Impulse,
    /// A torque (N·m) over the next step.
    Torque,
    /// An angular impulse (N·m·s) at once.
    TorqueImpulse,
}

/// A force, impulse or torque game code gave a body, applied by the next
/// step (world state: the `og.physics` state block saves those not yet
/// applied).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Push {
    pub entity: u64,
    pub kind: PushKind,
    /// The force, impulse, torque or angular impulse, in world axes.
    pub vector: Vec3,
    /// Where a force or impulse acts (world space); `None` at the centre of mass.
    pub point: Option<Vec3>,
}

/// The host's geometry by handle, for [`PhysicsSystem::set_mesh_source`].
pub type MeshSource = Arc<dyn Fn(u64) -> Option<MeshDesc> + Send + Sync>;

/// What the system keeps between ticks: the simulation of the active
/// profile (world state, saved by [`PhysicsState`]), registered meshes and
/// the shapes built from them (assets and caches).
#[derive(Default)]
pub(crate) struct Shared {
    #[cfg(feature = "physics3d")]
    pub sim3: Option<Box<dim3::Sim>>,
    #[cfg(feature = "physics2d")]
    pub sim2: Option<Box<dim2::Sim>>,
    /// Meshes registered with new geometry since the last step; colliders
    /// built from them are rebuilt.
    pub stale_meshes: BTreeSet<u64>,
    pub meshes: IdMap<u64, Arc<MeshDesc>>,
    /// Where a collider's mesh nobody registered comes from
    /// ([`PhysicsSystem::set_mesh_source`]).
    pub mesh_source: Option<MeshSource>,
    /// Height fields by handle (assets, like meshes).
    pub heightfields: IdMap<u64, Arc<HeightfieldDesc>>,
    /// Per stale height field whose every change since the last step was a
    /// region edit ([`PhysicsSystem::edit_heightfield`]): the chunks those
    /// edits reached, which are all its static colliders rebuild. A stale
    /// field without an entry is rebuilt whole.
    pub field_edits: IdMap<u64, BTreeSet<u32>>,
    /// The chunk shapes of static height-field colliders of more than one
    /// chunk ([`FIELD_CHUNK`]), keyed like the shape cache with the chunk in
    /// the key's shape code ([`chunk_key`]); outside the cache's limit, so
    /// a large field's chunks are never cleared together.
    #[cfg(feature = "physics3d")]
    pub chunks3: BTreeMap<ShapeKey, rapier3d::prelude::SharedShape>,
    /// Never filled (height fields are 3D only); the step is written once.
    #[cfg(feature = "physics2d")]
    pub chunks2: BTreeMap<ShapeKey, rapier2d::prelude::SharedShape>,
    /// Why no height field can be registered in this build
    /// ([`PhysicsSystem::refuse_heightfields`]): every SHAPE_HEIGHTFIELD
    /// collider is refused with it.
    pub heightfields_absent: Option<&'static str>,
    #[cfg(feature = "physics3d")]
    pub shapes3: BTreeMap<ShapeKey, rapier3d::prelude::SharedShape>,
    #[cfg(feature = "physics2d")]
    pub shapes2: BTreeMap<ShapeKey, rapier2d::prelude::SharedShape>,
    /// Triangle-mesh shapes the cook baked over a mesh at scale 1, by mesh
    /// handle ([`PhysicsSystem::set_baked_trimesh`]): taken instead of a
    /// build when the mesh is exactly the one baked.
    #[cfg(feature = "physics3d")]
    pub baked3: IdMap<u64, rapier3d::prelude::SharedShape>,
    /// The colliders [`PhysicsSystem::query_system`] recorded this tick, and
    /// the query view built from them on the first query that asked.
    pub recorded: Option<(Physics, Scene)>,
    pub recorded_query: Option<Arc<QueryInner>>,
    /// Ragdoll rigs by entity (assets, like meshes) and the animation's
    /// bone poses the host gave for this tick.
    pub rigs: IdMap<u64, Arc<RagdollRig>>,
    pub ragdoll_targets: IdMap<u64, Vec<(Vec3, Quat)>>,
    /// Garment rigs by entity, with the key the host gave them (assets,
    /// like ragdoll rigs), and the skin's pose of each for this tick
    /// ([`PhysicsSystem::set_garment_pose`]), taken by the step.
    pub garment_rigs: IdMap<u64, (u64, Arc<GarmentRig>)>,
    pub garment_poses: IdMap<u64, GarmentPose>,
    /// What the last step cost ([`PhysicsSystem::stats`]; not world state).
    pub stats: PhysicsStats,
    /// Forces and impulses waiting for the next step, in the order given
    /// (world state, saved with the simulation).
    pub pushes: Vec<Push>,
    /// The og.Water bodies og.Buoyancy floats on, as the host read them
    /// as the tick started ([`PhysicsSystem::set_waters`]; derived from the
    /// world's components, not world state).
    pub waters: Vec<WaterBody>,
}

/// A body of water from an og.Water (a renderer's record the host reads),
/// for og.Buoyancy: a square of `size` centred on `position` (its y the
/// rest level), flat or raised by its `sea` (og.Water.wind), whose waves
/// are the absolute world's (`zero`, og.WorldOrigin's, added to a point)
/// as its grid of `segments` cells a side draws them
/// ([`og_core::ocean::Ocean::height_on_grid`]). With a `shore` (W1: the
/// seabed og.Water.seabed names, in the frame's xz, and the trains rolling
/// in on it) the sea feels the depth ([`og_core::ocean::Ocean::surface_near`]):
/// a body in the shallows rides the shoaled, broken sea the renderer draws.
#[derive(Clone, Debug, PartialEq)]
pub struct WaterBody {
    pub entity: u64,
    pub position: Vec3,
    pub size: og_core::glam::Vec2,
    pub segments: u32,
    pub sea: Option<og_core::ocean::Ocean>,
    pub zero: og_core::glam::DVec3,
    pub shore: Option<(Arc<og_core::ocean::Shore>, og_core::ocean::ShoreTrains)>,
}

impl WaterBody {
    /// The surface's height over the rest level at world point (`x`, `z`)
    /// at `seconds` of the world's clock (its tick over its rate); `None`
    /// outside the square.
    pub fn height(&self, x: f64, z: f64, seconds: f64) -> Option<f32> {
        self.surface(x, z, seconds).map(|s| s.height)
    }

    /// [`Self::height`] with the water's velocity and acceleration there
    /// (its particles' orbital motion,
    /// [`og_core::ocean::Ocean::surface_on_grid`]; still water without a
    /// sea).
    pub fn surface(&self, x: f64, z: f64, seconds: f64) -> Option<og_core::ocean::SurfacePoint> {
        let half = self.size * 0.5;
        let local = og_core::glam::Vec2::new(
            (x - f64::from(self.position.x)) as f32,
            (z - f64::from(self.position.z)) as f32,
        );
        if !(local.x.abs() <= half.x && local.y.abs() <= half.y) {
            return None;
        }
        let Some(sea) = &self.sea else {
            return Some(og_core::ocean::SurfacePoint::default());
        };
        let extent = self.size.max_element();
        let segments = og_core::ocean::grid_segments(self.segments.max(1), extent);
        let cell = og_core::ocean::grid_cell(local, half, extent / segments as f32);
        let coast = self
            .shore
            .as_ref()
            .map(|(shore, trains)| og_core::ocean::Coast {
                shore,
                trains: *trains,
                zero: og_core::glam::DVec2::new(self.zero.x, self.zero.z),
            });
        Some(sea.surface_near(
            og_core::glam::DVec2::new(x + self.zero.x, z + self.zero.z),
            seconds,
            cell,
            coast.as_ref(),
        ))
    }
}

/// One bone of a ragdoll rig, in the entity's space at the rest pose.
#[derive(Clone, Debug, PartialEq)]
pub struct RagdollBone {
    /// The parent bone's index in [`RagdollRig::bones`] (it comes earlier); `None` for the root.
    pub parent: Option<u32>,
    /// The bone's rest position and rotation relative to the entity.
    pub rest: (Vec3, Quat),
    /// How long the bone is (to its child, or a guess for a leaf).
    pub length: f32,
    /// The direction the bone points in its own frame (towards its child).
    pub axis: Vec3,
}

/// The bones a ragdoll is made of: registered by the host for an entity
/// with an `og.Ragdoll` ([`PhysicsSystem::set_ragdoll_rig`]); an asset, not
/// world state, like meshes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RagdollRig {
    pub bones: Vec<RagdollBone>,
}

/// A wheel of an `og.Vehicle` as it is now, for drawing (and for smoke,
/// skid marks and a squeal).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WheelPose {
    pub position: Vec3,
    pub quaternion: Quat,
    /// Whether it touches the ground.
    pub contact: bool,
    /// How much the tyre slides (rapier's skid info turned round): 0 it
    /// grips, towards 1 it asks for more than its grip holds (0.5: twice
    /// as much). 0 in the air and in 2D.
    pub slip: f32,
    /// How fast the tyre's contact moves sideways over the ground, m/s
    /// (along the axle; 0 in the air and in 2D).
    pub side_speed: f32,
    /// Its og.VehicleWheel entity's bits; 0 for an implicit wheel.
    pub entity: u64,
}

/// A cloth's points as the last step left them, row-major.
#[derive(Clone, Debug, PartialEq)]
pub struct ClothPoints {
    pub columns: u32,
    pub rows: u32,
    pub points: Vec<Vec3>,
}

/// An og.SoftBody's points and its surface's triangles, three indices each.
#[derive(Clone, Debug, PartialEq)]
pub struct SoftBodyPoints {
    pub points: Vec<Vec3>,
    pub indices: Vec<u32>,
}

/// A height field's heights (relative to its entity's y), row-major, rows along +z.
#[derive(Clone, Debug, PartialEq)]
pub struct WaterHeights {
    pub resolution: u32,
    pub size: f32,
    pub heights: Vec<f32>,
}

/// A rotation taking unit vector `from` to unit vector `to`, from square
/// roots only (glam's `from_rotation_arc` reaches for `sin_cos` when they
/// are opposite).
pub(crate) fn arc(from: Vec3, to: Vec3) -> Quat {
    let d = from.dot(to);
    if d < -0.999_999 {
        let o = from.any_orthonormal_vector();
        return Quat::from_xyzw(o.x, o.y, o.z, 0.0);
    }
    let c = from.cross(to);
    Quat::from_xyzw(c.x, c.y, c.z, 1.0 + d).normalize()
}

/// At most this many mesh-derived shapes are cached (a scale animated every
/// tick would otherwise grow the cache without bound).
pub(crate) const SHAPE_CACHE_LIMIT: usize = 1024;

/// Cells per side of one chunk of a static height-field collider. A field
/// of more cells along x or z is built as one rapier collider per chunk
/// (each a height field over its square, sharing its border samples with
/// its neighbours), so a region edit ([`PhysicsSystem::edit_heightfield`])
/// rebuilds only the chunks under it: parry's height field cannot change
/// its heights in place. A field of one chunk is one collider, as before.
pub(crate) const FIELD_CHUNK: u32 = 256;

/// The chunks along x and z of a height field of `columns` × `rows`
/// heights ([`FIELD_CHUNK`]).
pub(crate) fn field_chunks(columns: u32, rows: u32) -> (u32, u32) {
    (
        (columns.max(2) - 1).div_ceil(FIELD_CHUNK),
        (rows.max(2) - 1).div_ceil(FIELD_CHUNK),
    )
}

/// The heights chunk `k` covers (row by row, [`field_chunks`]'s x first):
/// its first and last column and row, inclusive.
pub(crate) fn chunk_samples(f: &HeightfieldDesc, k: u32) -> ([u32; 2], [u32; 2]) {
    let (cx, _) = field_chunks(f.columns, f.rows);
    let (i, j) = (k % cx, k / cx);
    let c0 = i * FIELD_CHUNK;
    let r0 = j * FIELD_CHUNK;
    (
        [c0, (c0 + FIELD_CHUNK).min(f.columns - 1)],
        [r0, (r0 + FIELD_CHUNK).min(f.rows - 1)],
    )
}

/// A height field's spacing between samples along x and z.
pub(crate) fn field_cell(f: &HeightfieldDesc) -> Vec2 {
    Vec2::new(
        f.size.x / (f.columns - 1) as f32,
        f.size.y / (f.rows - 1) as f32,
    )
}

/// Where chunk `k`'s centre lies from its field's, in the frame of a
/// collider at `scale`.
pub(crate) fn chunk_offset(f: &HeightfieldDesc, scale: Vec3, k: u32) -> Vec3 {
    let ([c0, c1], [r0, r1]) = chunk_samples(f, k);
    let centre = field_cell(f) * Vec2::new((c0 + c1) as f32, (r0 + r1) as f32) * 0.5 - f.size * 0.5;
    Vec3::new(centre.x, 0.0, centre.y) * scale.abs()
}

/// The chunks whose heights a change inside `lo`..`hi` (the field's own
/// space: metres, centred on it) can reach, ascending. A sample on a
/// chunk's edge belongs to both chunks beside it.
pub(crate) fn chunks_under(f: &HeightfieldDesc, lo: Vec2, hi: Vec2) -> Vec<u32> {
    let (cx, cz) = field_chunks(f.columns, f.rows);
    let span = |a: f32, b: f32, size: f32, n: u32, chunks: u32| {
        let cell = size / (n - 1) as f32;
        let at = |x: f32| ((x + size * 0.5) / cell).clamp(0.0, (n - 1) as f32);
        let (s0, s1) = (at(a).floor() as u32, at(b).ceil() as u32);
        (
            s0.saturating_sub(1) / FIELD_CHUNK,
            (s1 / FIELD_CHUNK).min(chunks - 1),
        )
    };
    let (i0, i1) = span(lo.x, hi.x, f.size.x, f.columns, cx);
    let (j0, j1) = span(lo.y, hi.y, f.size.y, f.rows, cz);
    (j0..=j1)
        .flat_map(|j| (i0..=i1).map(move |i| j * cx + i))
        .collect()
}

/// The handle to an installed physics system: mesh registration and scene
/// queries. Cloning shares the same system.
#[derive(Clone)]
pub struct PhysicsSystem {
    shared: Arc<Mutex<Shared>>,
    ids: Arc<Ids>,
}

impl std::fmt::Debug for PhysicsSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhysicsSystem").finish_non_exhaustive()
    }
}

fn lock(shared: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    // A step that panics takes its simulation out of `Shared` first (see
    // `run`), so what a poisoned lock guards is still consistent.
    shared.lock().unwrap_or_else(|p| p.into_inner())
}

/// The simulation as the `og.physics` state block saves it: a profile tag
/// (none, 2 or 3) and the bincode of the profile's [`dim3::Sim`].
struct PhysicsState(Arc<Mutex<Shared>>);

enum Saved {
    Empty,
    Pushes(Vec<Push>),
    #[cfg(feature = "physics3d")]
    D3(Box<dim3::Sim>, Vec<Push>),
    #[cfg(feature = "physics2d")]
    D2(Box<dim2::Sim>, Vec<Push>),
}

/// A SHAPE_TRIMESH collider's shape over `mesh` at scale 1, as a collider
/// naming the mesh builds it (rapier's triangle mesh with its BVH), in
/// bytes for the cook to put beside the mesh: [`PhysicsSystem::set_baked_trimesh`]
/// takes it back without building anything. 3D only.
#[cfg(feature = "physics3d")]
pub fn bake_trimesh(mesh: &MeshDesc) -> Result<Vec<u8>> {
    dim3::bake(mesh)
}

fn codec() -> impl bincode::Options {
    use bincode::Options;
    bincode::DefaultOptions::new().reject_trailing_bytes()
}

/// `value`'s bytes appended to `out` through [`codec`]. Everything a save
/// writes goes through this one encoder, so each type's serialisation is
/// compiled once: bincode's `serialize` and `serialized_size` would each
/// compile the whole rapier world's again (a size pass, a writer of
/// another type), tens of kilobytes of wasm (`tests/evidence/r4/size-2`).
pub(crate) fn encode<T: serde::Serialize + ?Sized>(
    out: &mut Vec<u8>,
    value: &T,
) -> bincode::Result<()> {
    use bincode::Options;
    codec().serialize_into(out, value)
}

/// A value [`encode`] wrote, alone in `bytes`.
#[cfg(feature = "physics3d")]
pub(crate) fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> bincode::Result<T> {
    use bincode::Options;
    codec().deserialize(bytes)
}

/// What follows the profile tag in a saved `og.physics` block: the pushes
/// not yet applied, then the simulation's bytes without its asset shapes
/// and where they go (empty with tag 0, no simulation yet).
type Block = (Vec<Push>, Vec<Hole>, Vec<u8>);

impl StateBlock for PhysicsState {
    fn save(&self, out: &mut Vec<u8>) {
        let shared = lock(&self.0);
        let pushes = shared.pushes.clone();
        let mut sections = Vec::new();
        let block: Option<(u8, Block)> = match () {
            #[cfg(feature = "physics3d")]
            _ if shared.sim3.is_some() => {
                let sim = shared.sim3.as_deref().unwrap();
                sections = sim.ext.sections();
                let (holes, bytes) = dim3::save(sim);
                Some((PROFILE_3D as u8, (pushes, holes, bytes)))
            }
            #[cfg(feature = "physics2d")]
            _ if shared.sim2.is_some() => {
                let sim = shared.sim2.as_deref().unwrap();
                sections = sim.ext.sections();
                let (holes, bytes) = dim2::save(sim);
                Some((PROFILE_2D as u8, (pushes, holes, bytes)))
            }
            _ if !pushes.is_empty() => Some((0, (pushes, Vec::new(), Vec::new()))),
            _ => None,
        };
        if let Some((tag, block)) = block {
            out.push(tag);
            encode(out, &block).expect("a physics world always serializes into memory");
            og_ecs::trailer::write(out, &sections);
        }
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        use bincode::Options;
        let parse = |e: bincode::Error| {
            Error::new(
                ErrorKind::Parse,
                format!("the rapier world does not decode: {e}"),
            )
        };
        let Some((&tag, rest)) = bytes.split_first() else {
            return Ok(Box::new(Saved::Empty));
        };
        let mut reader = rest;
        let (pushes, holes, sim): Block = codec()
            .allow_trailing_bytes()
            .deserialize_from(&mut reader)
            .map_err(parse)?;
        let sections = og_ecs::trailer::read("og.physics", reader)?;
        // The asset shapes are rebuilt from the registered meshes, as a
        // step builds them (outside the step, like a query view).
        let saved = match tag as u32 {
            0 => Saved::Pushes(pushes),
            #[cfg(feature = "physics3d")]
            PROFILE_3D => {
                let mut sim = outside(|| dim3::load(&sim, &holes, &mut lock(&self.0)))?;
                sim.ext = dim3::SimExt::from_sections(&sections)?;
                Saved::D3(Box::new(sim), pushes)
            }
            #[cfg(feature = "physics2d")]
            PROFILE_2D => {
                let mut sim = outside(|| dim2::load(&sim, &holes, &mut lock(&self.0)))?;
                sim.ext = dim2::SimExt::from_sections(&sections)?;
                Saved::D2(Box::new(sim), pushes)
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("a {tag}D physics world; this engine lacks that profile"),
                ));
            }
        };
        Ok(Box::new(saved))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut shared = lock(&self.0);
        // The restored colliders are as saved, whatever was registered before.
        shared.stale_meshes.clear();
        shared.field_edits.clear();
        shared.pushes.clear();
        #[cfg(feature = "physics3d")]
        {
            shared.sim3 = None;
        }
        #[cfg(feature = "physics2d")]
        {
            shared.sim2 = None;
        }
        let saved = decoded.map(|d| {
            *d.downcast::<Saved>()
                .expect("decoded by PhysicsState::decode")
        });
        match saved {
            None | Some(Saved::Empty) => {}
            Some(Saved::Pushes(p)) => shared.pushes = p,
            #[cfg(feature = "physics3d")]
            Some(Saved::D3(sim, p)) => {
                shared.sim3 = Some(sim);
                shared.pushes = p;
            }
            #[cfg(feature = "physics2d")]
            Some(Saved::D2(sim, p)) => {
                shared.sim2 = Some(sim);
                shared.pushes = p;
            }
        }
    }

    /// The simulation in place ([`dim3::Sim::shift`]) and the points of the
    /// pushes not yet applied. The query view recorded for this tick was
    /// read before the shift: dropped, so the next query records again.
    fn shift(&mut self, s: &og_ecs::Shift) {
        let mut shared = lock(&self.0);
        #[cfg(feature = "physics3d")]
        if let Some(sim) = shared.sim3.as_deref_mut() {
            sim.shift(s);
        }
        #[cfg(feature = "physics2d")]
        if let Some(sim) = shared.sim2.as_deref_mut() {
            sim.shift(s);
        }
        for p in &mut shared.pushes {
            p.point = p.point.map(|q| s.moved(q));
        }
        shared.recorded = None;
        shared.recorded_query = None;
    }
}

/// Registers the physics types, inserts `settings` as the `og.Physics`
/// resource (replacing one already there) and schedules the `og.physics`
/// system at the end of the schedule, with its `og.physics` state block.
/// Install it after the systems that move kinematic bodies and before the
/// ones that read the simulated transforms.
pub fn install(world: &mut World, settings: Physics) -> Result<PhysicsSystem> {
    let physics = PhysicsSystem::register(world, settings)?;
    physics.schedule(world)?;
    Ok(physics)
}

/// The name of the system [`PhysicsSystem::query_system`] returns.
pub const QUERY_SYSTEM_NAME: &str = "og.physics.queries";

/// The physics types registered only once a game uses them, so a world
/// that never does keeps the type table, saves and hash it had before
/// G2-3 added them: [`install`] leaves them out and [`enable`] adds one.
pub const LAZY: [&str; 8] = [
    BodySlept::NAME,
    BodyWoke::NAME,
    JointBroken::NAME,
    ContactHook::NAME,
    VehicleWheel::NAME,
    SoftBody::NAME,
    Garment::NAME,
    Buoyancy::NAME,
];

/// Registers lazy physics type `name` (one of [`LAZY`]) and grants it to
/// the physics systems: `og.physics` sends the events and reads the
/// hooks, the query system (when scheduled) reads the hooks. A host calls
/// it when a game first names the type; registering it again is a no-op.
pub fn enable(world: &mut World, name: &str) -> Result<()> {
    match name {
        BodySlept::NAME => world.register::<BodySlept>().map(drop)?,
        BodyWoke::NAME => world.register::<BodyWoke>().map(drop)?,
        JointBroken::NAME => world.register::<JointBroken>().map(drop)?,
        ContactHook::NAME => world.register::<ContactHook>().map(drop)?,
        VehicleWheel::NAME => world.register::<VehicleWheel>().map(drop)?,
        SoftBody::NAME => world.register::<SoftBody>().map(drop)?,
        Garment::NAME => world.register::<Garment>().map(drop)?,
        Buoyancy::NAME => world.register::<Buoyancy>().map(drop)?,
        other => {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("{other} is not a lazily registered physics type"),
            ));
        }
    }
    world.grant(SYSTEM_NAME, &[name])?;
    if name == ContactHook::NAME || name == VehicleWheel::NAME {
        match world.grant(QUERY_SYSTEM_NAME, &[name]) {
            Err(e) if e.kind() == ErrorKind::NotFound && !e.reason().contains("type") => {}
            r => r?,
        }
    }
    Ok(())
}

impl PhysicsSystem {
    /// The first half of [`install`]: registers the types and inserts
    /// `settings` as `og.Physics`, scheduling nothing. A host that runs
    /// [`Self::query_system`] before its own systems calls this, adds that
    /// system, adds its systems, then calls [`Self::schedule`].
    pub fn register(world: &mut World, settings: Physics) -> Result<PhysicsSystem> {
        check_profile(settings.profile)?;
        let ids = Arc::new(Ids::register(world)?);
        world.insert_resource(settings)?;
        Ok(PhysicsSystem {
            shared: Arc::new(Mutex::new(Shared::default())),
            ids,
        })
    }

    /// The second half of [`install`]: schedules `og.physics` at the end of
    /// the schedule, with its `og.physics` state block.
    pub fn schedule(&self, world: &mut World) -> Result<()> {
        schedule(world, &self.shared, &self.ids)
    }

    /// A read-only system that records the colliders as the world holds
    /// them at its point in the schedule, so [`Self::tick_query`] answers
    /// scene queries from inside the systems after it in the same tick
    /// (game code cannot hand a `&World` to [`Self::query`] while the world
    /// steps). Recording copies the components; the query view is built only
    /// when a query asks. Add it with `world.add_system(desc, run)` before
    /// the systems that query.
    pub fn query_system(
        &self,
    ) -> (
        SystemDesc,
        impl Fn(&mut SystemContext<'_>) -> Result<()> + Send + 'static + use<>,
    ) {
        let mut desc = SystemDesc::new(QUERY_SYSTEM_NAME)
            .read::<Physics>()
            .read::<RigidBody>()
            .read::<Collider>()
            .read::<Velocity>();
        desc.reads.push(gather::TRANSFORM.to_string());
        for name in gather::READ {
            desc.reads.push(name.to_string());
        }
        for q in QUERIES {
            desc = desc.query(q.desc(false));
        }
        let (shared, ids) = (self.shared.clone(), self.ids.clone());
        let run = move |ctx: &mut SystemContext<'_>| {
            let settings = *ctx.resource::<Physics>()?;
            let lazy = gather::LazyIds::of(|n| ctx.id_by_name(n).ok());
            let scene =
                gather::gather(&|i| ctx.query(i), &|e, id| ctx.get_bytes(e, id), lazy, &ids)?;
            let mut shared = lock(&shared);
            shared.recorded = Some((settings, scene));
            shared.recorded_query = None;
            Ok(())
        };
        (desc, run)
    }

    /// Scene queries over the colliders [`Self::query_system`] recorded in
    /// the current tick (the same answers [`Self::query`] gives over the
    /// world at that point). Refused before that system has run.
    pub fn tick_query(&self) -> Result<PhysicsQuery> {
        let mut shared = lock(&self.shared);
        if let Some(q) = &shared.recorded_query {
            return Ok(PhysicsQuery {
                inner: q.clone(),
                shared: self.shared.clone(),
            });
        }
        let Some((settings, scene)) = shared.recorded.take() else {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "no tick has recorded the colliders yet: {QUERY_SYSTEM_NAME} runs at the start of each tick; between ticks, query the world"
                ),
            ));
        };
        drop(shared);
        let inner = Arc::new(QueryInner::build_outside(
            settings.profile,
            &scene,
            &self.shared,
        )?);
        let mut shared = lock(&self.shared);
        shared.recorded_query = Some(inner.clone());
        Ok(PhysicsQuery {
            inner,
            shared: self.shared.clone(),
        })
    }
}

fn schedule(world: &mut World, shared: &Arc<Mutex<Shared>>, ids: &Arc<Ids>) -> Result<()> {
    let mut desc = SystemDesc::new(SYSTEM_NAME)
        .read::<Physics>()
        .write::<RigidBody>()
        .read::<Collider>()
        .write::<Velocity>()
        .read::<Contact>()
        .write::<Contact>()
        .write::<CollisionStarted>()
        .write::<CollisionEnded>()
        .write::<SensorEntered>()
        .write::<SensorExited>()
        .write::<CharacterEvent>()
        .write::<Splash>();
    desc.writes.push(gather::TRANSFORM.to_string());
    for name in gather::WRITTEN {
        desc.writes.push(name.to_string());
    }
    for name in gather::READ {
        if !gather::WRITTEN.contains(&name) {
            desc.reads.push(name.to_string());
        }
    }
    for q in QUERIES {
        desc = desc.query(q.desc(true));
    }
    let (s, i) = (shared.clone(), ids.clone());
    world.add_system(desc, move |ctx| run(ctx, &i, &s))?;
    world.add_state_block(SYSTEM_NAME, Box::new(PhysicsState(shared.clone())))?;
    Ok(())
}

fn check_profile(profile: u32) -> Result<()> {
    let built = match profile {
        PROFILE_2D => cfg!(feature = "physics2d"),
        PROFILE_3D => cfg!(feature = "physics3d"),
        other => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("og.Physics.profile is {other}; it must be 2 (2D) or 3 (3D)"),
            ));
        }
    };
    if built {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "og.Physics.profile {profile}: this engine was built without the physics{profile}d feature"
            ),
        ))
    }
}

impl PhysicsSystem {
    /// Height fields have no source in this build (a host without terrain,
    /// og-runtime's 2D profile): every [`SHAPE_HEIGHTFIELD`] collider is
    /// refused with `why`, in either profile, and [`Self::set_heightfield`]
    /// is refused too.
    pub fn refuse_heightfields(&self, why: &'static str) {
        lock(&self.shared).heightfields_absent = Some(why);
    }

    /// Registers (or replaces) the geometry of mesh `handle` for convex,
    /// polygon and triangle-mesh colliders. Meshes are assets, not world
    /// state: a world restored into a fresh engine needs them registered
    /// again, like every other asset. New geometry rebuilds the colliders
    /// made from the mesh at the next step; the same geometry again changes
    /// nothing.
    pub fn set_mesh(&self, handle: u64, mesh: MeshDesc) -> Result<()> {
        if mesh.positions.is_empty() {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("mesh {handle} has no positions"),
            ));
        }
        let n = mesh.positions.len() as u32;
        if let Some(t) = mesh.indices.iter().find(|t| t.iter().any(|&i| i >= n)) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("mesh {handle}: triangle {t:?} indexes past its {n} positions"),
            ));
        }
        if mesh.positions.iter().any(|p| !p.is_finite()) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("mesh {handle} has a non-finite position"),
            ));
        }
        let mut shared = lock(&self.shared);
        if shared.meshes.get(&handle).is_some_and(|m| **m == mesh) {
            return Ok(());
        }
        shared.meshes.insert(handle, Arc::new(mesh));
        shared.stale_meshes.insert(handle);
        shared.field_edits.remove(&handle);
        #[cfg(feature = "physics3d")]
        shared.shapes3.retain(|k, _| k.mesh != handle);
        #[cfg(feature = "physics2d")]
        shared.shapes2.retain(|k, _| k.mesh != handle);
        Ok(())
    }

    /// A triangle-mesh shape [`bake_trimesh`] baked for mesh `handle`,
    /// decoded now (between ticks, not in the step that first needs it): a
    /// SHAPE_TRIMESH collider at scale 1 naming the mesh takes it instead of
    /// building one, while the mesh's positions and triangles are exactly
    /// the ones baked (a build over them is the same shape, bit for bit).
    /// Refused for bytes another bake format wrote. 3D only.
    #[cfg(feature = "physics3d")]
    pub fn set_baked_trimesh(&self, handle: u64, bytes: &[u8]) -> Result<()> {
        let shape = dim3::unbake(bytes)?;
        lock(&self.shared).baked3.insert(handle, shape);
        Ok(())
    }

    /// What a convex or triangle-mesh collider whose `mesh` was never given
    /// to [`set_mesh`](Self::set_mesh) is built from: the host's geometry
    /// under that handle, asked for once, when a collider first needs it
    /// (`None`: no such geometry, the collider is refused as before). A host
    /// that holds many geometries few colliders use (a large scene's
    /// meshes) need not copy every one into physics up front.
    pub fn set_mesh_source(&self, source: Option<MeshSource>) {
        lock(&self.shared).mesh_source = source;
    }

    /// Registers (or replaces) the height field `handle` that
    /// [`SHAPE_HEIGHTFIELD`] colliders name (a terrain; 3D only). An asset
    /// like a mesh: a world restored into a fresh engine needs it
    /// registered again. New heights (a terrain edit) rebuild the colliders
    /// made from it at the next step; the same heights change nothing.
    pub fn set_heightfield(&self, handle: u64, field: HeightfieldDesc) -> Result<()> {
        self.check_heightfield(handle, &field)?;
        if field.heights.iter().any(|h| !h.is_finite()) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("height field {handle}: its size must be positive and its heights finite"),
            ));
        }
        let mut shared = lock(&self.shared);
        if shared
            .heightfields
            .get(&handle)
            .is_some_and(|f| **f == field)
        {
            return Ok(());
        }
        shared.heightfields.insert(handle, Arc::new(field));
        shared.stale_meshes.insert(handle);
        shared.field_edits.remove(&handle);
        #[cfg(feature = "physics3d")]
        {
            shared.shapes3.retain(|k, _| k.mesh != handle);
            shared.chunks3.retain(|k, _| k.mesh != handle);
        }
        Ok(())
    }

    /// Registers new heights for height field `handle` that differ from
    /// the ones registered only inside `lo`..`hi` (the field's own space:
    /// metres, centred on it; a terrain edit's rectangle), with the same
    /// columns, rows and size. Its static colliders rebuild at the next
    /// step only the chunks ([`FIELD_CHUNK`] cells a side) under the
    /// rectangle, keeping the others and their contacts; a field of one
    /// chunk, or one not registered yet, is set whole as
    /// [`set_heightfield`](Self::set_heightfield) does. Heights outside the
    /// rectangle are taken as they are, unchecked.
    pub fn edit_heightfield(
        &self,
        handle: u64,
        field: HeightfieldDesc,
        lo: Vec2,
        hi: Vec2,
    ) -> Result<()> {
        let same_shape = {
            let shared = lock(&self.shared);
            shared.heightfields.get(&handle).is_some_and(|f| {
                (f.columns, f.rows, f.size, f.holes.is_some())
                    == (field.columns, field.rows, field.size, field.holes.is_some())
            })
        };
        let (cx, cz) = field_chunks(field.columns, field.rows);
        if !same_shape || cx * cz <= 1 || !(lo.is_finite() && hi.is_finite()) {
            return self.set_heightfield(handle, field);
        }
        self.check_heightfield(handle, &field)?;
        let chunks = chunks_under(&field, lo.min(hi), lo.max(hi));
        for &k in &chunks {
            let ([c0, c1], [r0, r1]) = chunk_samples(&field, k);
            for r in r0..=r1 {
                let row = &field.heights
                    [(r * field.columns + c0) as usize..=(r * field.columns + c1) as usize];
                if row.iter().any(|h| !h.is_finite()) {
                    return Err(Error::new(
                        ErrorKind::Invalid,
                        format!("height field {handle}: its heights must be finite"),
                    ));
                }
            }
        }
        let mut shared = lock(&self.shared);
        let whole_pending =
            shared.stale_meshes.contains(&handle) && !shared.field_edits.contains_key(&handle);
        shared.heightfields.insert(handle, Arc::new(field));
        shared.stale_meshes.insert(handle);
        #[cfg(feature = "physics3d")]
        {
            // A body's collider is the whole field: built again.
            shared.shapes3.retain(|k, _| k.mesh != handle);
            shared.chunks3.retain(|k, _| {
                k.mesh != handle
                    || key_chunk(k.shape).is_some_and(|c| chunks.binary_search(&c).is_err())
            });
        }
        if !whole_pending {
            shared.field_edits.entry(handle).or_default().extend(chunks);
        }
        Ok(())
    }

    /// Refuses a height field this build or its dimensions cannot take.
    fn check_heightfield(&self, handle: u64, field: &HeightfieldDesc) -> Result<()> {
        if let Some(why) = lock(&self.shared).heightfields_absent {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("height field {handle}: {why}"),
            ));
        }
        if field.columns < 2 || field.rows < 2 {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "height field {handle} needs at least 2 × 2 heights, got {} × {}",
                    field.columns, field.rows
                ),
            ));
        }
        if field.heights.len() != (field.columns * field.rows) as usize {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "height field {handle}: {} × {} needs {} heights, got {}",
                    field.columns,
                    field.rows,
                    field.columns * field.rows,
                    field.heights.len()
                ),
            ));
        }
        if let Some(h) = &field.holes
            && h.len() != ((field.columns - 1) * (field.rows - 1)) as usize
        {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "height field {handle}: {} × {} heights have {} cells, its holes {}",
                    field.columns,
                    field.rows,
                    (field.columns - 1) * (field.rows - 1),
                    h.len()
                ),
            ));
        }
        if !(field.size.x > 0.0 && field.size.y > 0.0 && field.size.is_finite()) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("height field {handle}: its size must be positive and its heights finite"),
            ));
        }
        Ok(())
    }

    fn push(
        &self,
        entity: Entity,
        kind: PushKind,
        vector: Vec3,
        point: Option<Vec3>,
    ) -> Result<()> {
        if !vector.is_finite() || point.is_some_and(|p| !p.is_finite()) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("a force, impulse or torque must be finite, got {vector} at {point:?}"),
            ));
        }
        lock(&self.shared).pushes.push(Push {
            entity: entity.to_bits(),
            kind,
            vector,
            point,
        });
        Ok(())
    }

    /// Pushes the dynamic body of `entity` with `force` (N, world axes)
    /// over the next step, at `point` (world space) or its centre of mass:
    /// call it every tick for a steady force (thrust, wind, downforce).
    /// Applied by the next `og.physics` step, in the order given, and gone
    /// after it; a hit stop drops it. Other bodies ignore it.
    pub fn apply_force(&self, entity: Entity, force: Vec3, point: Option<Vec3>) -> Result<()> {
        self.push(entity, PushKind::Force, force, point)
    }

    /// Gives the dynamic body of `entity` an `impulse` (N·s, world axes) at
    /// `point` or its centre of mass: a sudden change of its momentum (a
    /// kick, a hit, a jump), applied by the next step (after a hit stop
    /// ends).
    pub fn apply_impulse(&self, entity: Entity, impulse: Vec3, point: Option<Vec3>) -> Result<()> {
        self.push(entity, PushKind::Impulse, impulse, point)
    }

    /// Turns the dynamic body of `entity` with `torque` (N·m about each
    /// world axis; 2D uses z) over the next step, as [`Self::apply_force`].
    pub fn apply_torque(&self, entity: Entity, torque: Vec3) -> Result<()> {
        self.push(entity, PushKind::Torque, torque, None)
    }

    /// Gives the dynamic body of `entity` an angular impulse (N·m·s about
    /// each world axis; 2D uses z), as [`Self::apply_impulse`].
    pub fn apply_torque_impulse(&self, entity: Entity, torque: Vec3) -> Result<()> {
        self.push(entity, PushKind::TorqueImpulse, torque, None)
    }

    /// Removes mesh `handle`; colliders still naming it fail the next step.
    pub fn remove_mesh(&self, handle: u64) {
        let mut shared = lock(&self.shared);
        shared.meshes.remove(&handle);
        #[cfg(feature = "physics3d")]
        shared.baked3.remove(&handle);
        shared.heightfields.remove(&handle);
        shared.stale_meshes.insert(handle);
        shared.field_edits.remove(&handle);
        #[cfg(feature = "physics3d")]
        shared.shapes3.retain(|k, _| k.mesh != handle);
        #[cfg(feature = "physics3d")]
        shared.chunks3.retain(|k, _| k.mesh != handle);
        #[cfg(feature = "physics2d")]
        shared.shapes2.retain(|k, _| k.mesh != handle);
    }

    /// A query view of the colliders as `world` holds them now (outside a
    /// tick). Build one and run many queries on it; it does not follow
    /// later changes to the world.
    pub fn query(&self, world: &World) -> Result<PhysicsQuery> {
        let settings = *world.resource::<Physics>()?;
        let scene = gather::gather(
            &|i| world.query(&QUERIES[i].desc(false)),
            &|e, id| world.get_bytes(e, id),
            gather::LazyIds::of(|n| world.id_by_name(n).ok()),
            &self.ids,
        )?;
        let inner = Arc::new(QueryInner::build_outside(
            settings.profile,
            &scene,
            &self.shared,
        )?);
        Ok(PhysicsQuery {
            inner,
            shared: self.shared.clone(),
        })
    }
}

/// Runs `$body` with `$s` bound to the active profile's simulation, or
/// answers `None` when there is none yet.
macro_rules! with_sim {
    ($shared:expr, |$s:ident| $body:expr) => {{
        let shared = lock(&$shared);
        #[cfg(feature = "physics3d")]
        if let Some($s) = shared.sim3.as_deref() {
            return $body;
        }
        #[cfg(feature = "physics2d")]
        if let Some($s) = shared.sim2.as_deref() {
            return $body;
        }
        None
    }};
}

/// What physics keeps beyond the components, read for drawing and by game
/// code (`og.physics.ropePoints` and friends): as the last step left it.
impl PhysicsSystem {
    /// The physics overlay's segments with their RGBA colours: collider
    /// shapes, joints and contacts as the last step left them (rapier's
    /// debug render). For drawing only; `None` before the first step.
    pub fn debug_lines(&self) -> Option<Vec<(Vec3, Vec3, [f32; 4])>> {
        with_sim!(self.shared, |s| Some(s.debug_lines()))
    }

    /// The points of the og.Rope on `entity`, start to end.
    pub fn rope_points(&self, entity: Entity) -> Option<Vec<Vec3>> {
        with_sim!(self.shared, |s| s
            .ropes
            .get(&entity.to_bits())
            .map(|r| r.points.clone()))
    }

    /// The points of the og.Cloth on `entity`.
    pub fn cloth_points(&self, entity: Entity) -> Option<ClothPoints> {
        with_sim!(self.shared, |s| s.cloths.get(&entity.to_bits()).map(|c| {
            let (columns, rows) = c.dims();
            ClothPoints {
                columns,
                rows,
                points: c.points.clone(),
            }
        }))
    }

    /// The points of the og.SoftBody on `entity` and the triangles of its
    /// surface (2D: a fan over its ring, for drawing).
    pub fn soft_body_points(&self, entity: Entity) -> Option<SoftBodyPoints> {
        with_sim!(self.shared, |s| s.ext.softs.get(&entity.to_bits()).map(
            |b| {
                SoftBodyPoints {
                    points: b.points.clone(),
                    indices: b.triangles.iter().flatten().copied().collect(),
                }
            }
        ))
    }

    /// The particle positions of the og.Fluid2D on `entity` (xy).
    pub fn fluid_particles(&self, entity: Entity) -> Option<Vec<Vec2>> {
        with_sim!(self.shared, |s| s
            .fluids
            .get(&entity.to_bits())
            .map(|f| f.positions.clone()))
    }

    /// The mix of the og.Fluid2D on `entity` when it shares a group: its
    /// lane in the group (entity order) and, per particle, how much of it
    /// is each lane's fluid (summing to 1). `None` for a fluid alone.
    pub fn fluid_mix(&self, entity: Entity) -> Option<(u32, Vec<[f32; 4]>)> {
        with_sim!(self.shared, |s| s
            .fluids
            .get(&entity.to_bits())
            .filter(|f| f.slot != u32::MAX)
            .map(|f| (f.slot, f.mix.clone())))
    }

    /// The particle positions of the og.Fluid3D on `entity` (world space).
    pub fn fluid3d_particles(&self, entity: Entity) -> Option<Vec<Vec3>> {
        with_sim!(self.shared, |s| s
            .fluids3
            .get(&entity.to_bits())
            .map(|f| f.positions.clone()))
    }

    /// The mix of the og.Fluid3D on `entity` when it shares a group, as
    /// [`Self::fluid_mix`] is for og.Fluid2D. `None` for a fluid alone.
    pub fn fluid3d_mix(&self, entity: Entity) -> Option<(u32, Vec<[f32; 4]>)> {
        with_sim!(self.shared, |s| s
            .fluids3
            .get(&entity.to_bits())
            .filter(|f| f.slot != u32::MAX)
            .map(|f| (f.slot, f.mix.clone())))
    }

    /// The heights of the og.WaterSurface on `entity`.
    pub fn water_heights(&self, entity: Entity) -> Option<WaterHeights> {
        with_sim!(self.shared, |s| s.surfaces.get(&entity.to_bits()).map(
            |w| {
                WaterHeights {
                    resolution: w.resolution.clamp(8, 256),
                    size: w.size,
                    heights: w.heights.clone(),
                }
            }
        ))
    }

    /// The bodies that cut the og.WaterSurface on `entity` in the last
    /// ticks, for the renderer's ripples (not world state).
    pub fn water_wakes(&self, entity: Entity) -> Option<Wakes> {
        with_sim!(self.shared, |s| s.wakes.get(&entity.to_bits()).cloned())
    }

    /// The spring heights of the og.WaterLine on `entity`, left to right
    /// (relative to its entity's y).
    pub fn water_line(&self, entity: Entity) -> Option<Vec<f32>> {
        with_sim!(self.shared, |s| s
            .lines
            .get(&entity.to_bits())
            .map(|l| l.heights.clone()))
    }

    /// The wheels of the og.Vehicle on `entity` as they are now.
    pub fn vehicle_wheels(&self, entity: Entity) -> Option<Vec<WheelPose>> {
        let shared = lock(&self.shared);
        #[cfg(feature = "physics3d")]
        if let Some(s) = shared.sim3.as_deref() {
            return dim3::wheels(s, entity.to_bits());
        }
        #[cfg(feature = "physics2d")]
        if let Some(s) = shared.sim2.as_deref() {
            return dim2::wheels(s, entity.to_bits());
        }
        None
    }

    /// What the last step cost, phase by phase, and which phases ran on the
    /// lanes pool ([`PhysicsStats`]); the default before the first step.
    /// Timings are wall clock, never world state.
    pub fn stats(&self) -> PhysicsStats {
        lock(&self.shared).stats.clone()
    }

    /// The bones of the og.Ragdoll on `entity` as the bodies hold them
    /// (rig order, world space; in 2D at their targets' z); `None` before
    /// its first step.
    pub fn ragdoll_pose(&self, entity: Entity) -> Option<Vec<(Vec3, Quat)>> {
        let shared = lock(&self.shared);
        #[cfg(feature = "physics3d")]
        if let Some(s) = shared.sim3.as_deref() {
            return dim3::ragdoll_pose(s, entity.to_bits());
        }
        #[cfg(feature = "physics2d")]
        if let Some(s) = shared.sim2.as_deref() {
            return dim2::ragdoll_pose(s, entity.to_bits());
        }
        None
    }

    /// Registers the rig an og.Ragdoll on `entity` is built from. Every
    /// bone's parent comes before it. A new rig rebuilds the ragdoll.
    pub fn set_ragdoll_rig(&self, entity: Entity, rig: RagdollRig) -> Result<()> {
        for (i, b) in rig.bones.iter().enumerate() {
            if b.parent.is_some_and(|p| p as usize >= i) {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("ragdoll rig bone {i}: its parent must come before it"),
                ));
            }
            if !(b.length.is_finite() && b.rest.0.is_finite() && b.rest.1.is_finite()) {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("ragdoll rig bone {i} is not finite"),
                ));
            }
        }
        let mut shared = lock(&self.shared);
        shared.rigs.insert(entity.to_bits(), Arc::new(rig));
        Ok(())
    }

    /// Whether `entity` has a ragdoll rig registered.
    pub fn has_ragdoll_rig(&self, entity: Entity) -> bool {
        lock(&self.shared).rigs.contains_key(&entity.to_bits())
    }

    /// Registers the rig the og.Garment on `entity` is built from, under
    /// `key` (the host's name for it: a model's mesh); a rig of another key
    /// rebuilds the garment on the skin at its next step.
    pub fn set_garment_rig(&self, entity: Entity, key: u64, rig: GarmentRig) -> Result<()> {
        rig.check()?;
        lock(&self.shared)
            .garment_rigs
            .insert(entity.to_bits(), (key, Arc::new(rig)));
        Ok(())
    }

    /// The key of the garment rig registered for `entity`, if any.
    pub fn garment_rig_key(&self, entity: Entity) -> Option<u64> {
        lock(&self.shared)
            .garment_rigs
            .get(&entity.to_bits())
            .map(|r| r.0)
    }

    /// Where the skin puts the points of the garment on `entity` this tick
    /// (rig order, world space), for its next step. A garment given no pose
    /// in a tick keeps its points.
    pub fn set_garment_pose(&self, entity: Entity, pose: GarmentPose) {
        lock(&self.shared)
            .garment_poses
            .insert(entity.to_bits(), pose);
    }

    /// The points of the og.Garment on `entity` after the last step and the
    /// targets that step held them to (rig order, world space).
    pub fn garment_points(&self, entity: Entity) -> Option<GarmentPoints> {
        with_sim!(self.shared, |s| s.ext.garments.get(&entity.to_bits()).map(
            |g| GarmentPoints {
                points: g.points.clone(),
                targets: g.targets.clone(),
            }
        ))
    }

    /// The animation's bone poses (rig order, world space) the ragdoll on
    /// `entity` follows at its next step while its weight is 0.
    pub fn set_ragdoll_targets(&self, entity: Entity, targets: Vec<(Vec3, Quat)>) {
        lock(&self.shared)
            .ragdoll_targets
            .insert(entity.to_bits(), targets);
    }

    /// The og.Water bodies the next steps float og.Buoyancy bodies on (the
    /// host reads them from the world as each tick starts, in entity
    /// order), replacing the last ones.
    pub fn set_waters(&self, waters: Vec<WaterBody>) {
        lock(&self.shared).waters = waters;
    }
}

/// Scene queries over a fixed set of colliders (see [`PhysicsSystem::query`]).
/// Results depend only on the colliders, so they are as deterministic as the
/// world they were built from.
pub struct PhysicsQuery {
    inner: Arc<QueryInner>,
    shared: Arc<Mutex<Shared>>,
}

pub(crate) enum QueryInner {
    #[cfg(feature = "physics3d")]
    D3(dim3::QueryWorld),
    #[cfg(feature = "physics2d")]
    D2(dim2::QueryWorld),
}

impl QueryInner {
    fn build(profile: u32, scene: &Scene, shared: &mut Shared) -> Result<QueryInner> {
        Ok(match profile {
            #[cfg(feature = "physics3d")]
            PROFILE_3D => QueryInner::D3(dim3::QueryWorld::build(scene, shared)?),
            #[cfg(feature = "physics2d")]
            PROFILE_2D => QueryInner::D2(dim2::QueryWorld::build(scene, shared)?),
            other => return Err(check_profile(other).unwrap_err()),
        })
    }

    /// [`Self::build`] outside the step, through [`outside`].
    fn build_outside(profile: u32, scene: &Scene, shared: &Mutex<Shared>) -> Result<QueryInner> {
        outside(|| Self::build(profile, scene, &mut lock(shared)))
    }
}

/// Runs `f`, physics work outside the step (building a query view, a
/// raycast, a shape cast, an overlap), where og-lanes runs work outside its
/// helpers ([`og_lanes::serial`]): on the calling thread inside its
/// one-thread pool, so parry's own `rayon::join` (its tree traversals, a
/// shape against a large triangle mesh) runs inline there and never reaches
/// rayon's global pool or waits on another thread; on the web a panic in it
/// traps the engine thread with the structured crash instead of hanging it.
/// A fault injected at the physics site fires here.
fn outside<R: Send>(f: impl FnOnce() -> R + Send) -> R {
    og_lanes::serial(og_lanes::Site::Physics, f)
}

/// `v` normalised; refused when zero (or zero in the profile's plane).
pub(crate) fn direction(v: Vec3, what: &str, profile: &str) -> Result<Vec3> {
    let len2 = v.length_squared();
    if !len2.is_finite() || len2 <= 0.0 {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "{what}.direction must be finite and non-zero in the {profile} profile, got {v}"
            ),
        ));
    }
    Ok(v / len2.sqrt())
}

/// `q` normalised; a zero or non-finite quaternion is the identity.
pub(crate) fn unit(q: Quat) -> Quat {
    let len2 = q.length_squared();
    if len2 > 1e-12 && len2.is_finite() {
        q / len2.sqrt()
    } else {
        Quat::IDENTITY
    }
}

fn max_distance(d: f32) -> f32 {
    if d > 0.0 { d } else { f32::MAX }
}

impl PhysicsQuery {
    /// The nearest collider along a ray.
    pub fn raycast(&self, desc: &RaycastDesc) -> Result<Option<RaycastHit>> {
        let max = max_distance(desc.max_distance);
        outside(|| match &*self.inner {
            #[cfg(feature = "physics3d")]
            QueryInner::D3(q) => q.raycast(desc.origin, desc.direction, max, &desc.filter),
            #[cfg(feature = "physics2d")]
            QueryInner::D2(q) => q.raycast(desc.origin, desc.direction, max, &desc.filter),
        })
    }

    /// Every collider a ray meets within its reach, nearest first (ties by
    /// collider entity, then body), at most `max_hits`: one hit per
    /// collider, at its first crossing.
    pub fn raycast_all(&self, desc: &RaycastAllDesc) -> Result<Vec<RaycastHit>> {
        let r = &desc.ray;
        let max = max_distance(r.max_distance);
        let cap = match desc.max_hits {
            0 => MAX_RAYCAST_HITS,
            n => n.min(MAX_RAYCAST_HITS),
        } as usize;
        outside(|| match &*self.inner {
            #[cfg(feature = "physics3d")]
            QueryInner::D3(q) => q.raycast_all(r.origin, r.direction, max, &r.filter, cap),
            #[cfg(feature = "physics2d")]
            QueryInner::D2(q) => q.raycast_all(r.origin, r.direction, max, &r.filter, cap),
        })
    }

    /// The nearest point on any collider to a point, within its reach.
    pub fn project_point(&self, desc: &ProjectPointDesc) -> Result<Option<PointProjection>> {
        let max = max_distance(desc.max_distance);
        outside(|| match &*self.inner {
            #[cfg(feature = "physics3d")]
            QueryInner::D3(q) => q.project_point(desc, max),
            #[cfg(feature = "physics2d")]
            QueryInner::D2(q) => q.project_point(desc, max),
        })
    }

    /// The first collider a shape meets when swept along a direction.
    pub fn shape_cast(&self, desc: &ShapeCastDesc) -> Result<Option<ShapeCastHit>> {
        let max = max_distance(desc.max_distance);
        outside(|| {
            let mut shared = lock(&self.shared);
            match &*self.inner {
                #[cfg(feature = "physics3d")]
                QueryInner::D3(q) => q.shape_cast(desc, max, &mut shared),
                #[cfg(feature = "physics2d")]
                QueryInner::D2(q) => q.shape_cast(desc, max, &mut shared),
            }
        })
    }

    /// Every collider overlapping a shape, sorted by entity id.
    pub fn overlap(&self, desc: &OverlapDesc) -> Result<Vec<Entity>> {
        outside(|| {
            let mut shared = lock(&self.shared);
            match &*self.inner {
                #[cfg(feature = "physics3d")]
                QueryInner::D3(q) => q.overlap(desc, &mut shared),
                #[cfg(feature = "physics2d")]
                QueryInner::D2(q) => q.overlap(desc, &mut shared),
            }
        })
    }
}

/// [`PhysicsQuery::raycast`] on a view built for this one call.
pub fn raycast(
    world: &World,
    physics: &PhysicsSystem,
    desc: &RaycastDesc,
) -> Result<Option<RaycastHit>> {
    physics.query(world)?.raycast(desc)
}

/// [`PhysicsQuery::shape_cast`] on a view built for this one call.
pub fn shape_cast(
    world: &World,
    physics: &PhysicsSystem,
    desc: &ShapeCastDesc,
) -> Result<Option<ShapeCastHit>> {
    physics.query(world)?.shape_cast(desc)
}

/// [`PhysicsQuery::overlap`] on a view built for this one call.
pub fn overlap(world: &World, physics: &PhysicsSystem, desc: &OverlapDesc) -> Result<Vec<Entity>> {
    physics.query(world)?.overlap(desc)
}

// ---- the system ------------------------------------------------------------------

/// A controller's component after its move.
pub(crate) enum ControlOut {
    Character(CharacterController),
    /// Only a 2D world moves a platformer controller.
    #[cfg_attr(not(feature = "physics2d"), allow(dead_code))]
    Platformer(PlatformerController),
}

/// What sleep made of the tick: the `og.RigidBody` values to write (their
/// `sleeping` changed) and the bodies that fell asleep and woke, each in
/// entity order.
#[derive(Default)]
pub(crate) struct SleepOut {
    pub(crate) writes: Vec<(Entity, RigidBody)>,
    pub(crate) slept: Vec<u64>,
    pub(crate) woke: Vec<u64>,
}

/// What one step produced.
#[derive(Default)]
pub(crate) struct StepOut {
    /// Per `Scene::bodies` index: the new pose of dynamic bodies (with
    /// their velocity) and of controllers (without).
    pub moved: Vec<Option<(Vec3, Quat, Option<Velocity>)>>,
    /// Every touching pair, sorted by (a, b), with the event it starts as
    /// (point, normal from a to b and the hit; zero for sensor overlaps).
    pub touching: Vec<(Contact, CollisionStarted)>,
    /// The components a step writes back.
    pub controls: Vec<(Entity, ControlOut)>,
    pub joints: Vec<(Entity, Joint)>,
    /// The joints that broke, in the order they did.
    pub broken: Vec<JointBroken>,
    pub vehicles: Vec<(Entity, Vehicle)>,
    pub ragdolls: Vec<(Entity, Ragdoll)>,
    pub events: Vec<CharacterEvent>,
    pub splashes: Vec<Splash>,
    /// The soft volumes' written fields.
    pub softs: Vec<(Entity, SoftBody)>,
    /// The floating bodies' written fields (og.Buoyancy.submerged).
    pub floats: Vec<(Entity, Buoyancy)>,
    /// What sleep changed (og.Physics.sleepTicks).
    pub sleep: SleepOut,
    /// The step's phases, timed on the thread it ran on.
    pub laps: Option<stats::Laps>,
}

fn run(ctx: &mut SystemContext<'_>, ids: &Ids, shared: &Mutex<Shared>) -> Result<()> {
    let settings = *ctx.resource::<Physics>()?;
    // Slow motion steps by the scaled tick; a hit stop (scaled dt 0) holds
    // the simulation as it is and carries the touching pairs over, so no
    // contact ends or starts while time stands still.
    let dt = ctx.scaled_dt();
    if dt <= 0.0 {
        for c in ctx.events::<Contact>()? {
            ctx.send(c)?;
        }
        // A force acts over a step: none passes. Impulses wait for one.
        lock(shared)
            .pushes
            .retain(|p| matches!(p.kind, PushKind::Impulse | PushKind::TorqueImpulse));
        return Ok(());
    }
    let started = stats::now_ms();
    let mut laps = stats::Laps::start();
    let lazy = gather::LazyIds::of(|n| ctx.id_by_name(n).ok());
    let mut scene: Scene =
        gather::gather(&|i| ctx.query(i), &|e, id| ctx.get_bytes(e, id), lazy, ids)?;
    if let Some(soft) = lazy.soft {
        scene.softs = gather::softs(
            &|d| ctx.query_with(d),
            &|e, id| ctx.get_bytes(e, id),
            soft,
            ids,
        )?;
    }
    if let Some(garment) = lazy.garment {
        scene.garments = gather::garments(
            &|d| ctx.query_with(d),
            &|e, id| ctx.get_bytes(e, id),
            garment,
        )?;
    }
    if let Some(buoyancy) = lazy.buoyancy {
        scene.floats = gather::floats(
            &|d| ctx.query_with(d),
            &|e, id| ctx.get_bytes(e, id),
            buoyancy,
        )?;
        // The world's clock as the sea's (og.physics.waterHeight's, the renderer's): its tick over its rate.
        let time = ctx.time();
        let rate = if time.dt > 0.0 { 1.0 / time.dt } else { 60.0 };
        scene.seconds = time.tick as f64 / f64::from(rate);
    }
    laps.lap(stats::Phase::Gather);
    let tick = ctx.time().tick;
    let threads = og_lanes::threads();
    // The simulation is taken out for the step and put back only when it
    // succeeds: a failed tick leaves none, and the next one rebuilds it.
    let out = {
        let mut shared = lock(shared);
        match settings.profile {
            #[cfg(feature = "physics3d")]
            PROFILE_3D => {
                #[cfg(feature = "physics2d")]
                {
                    shared.sim2 = None;
                }
                let mut sim = shared.sim3.take().unwrap_or_default();
                let out = dim3::step(&settings, dt, tick, &scene, &mut shared, &mut sim)?;
                shared.sim3 = Some(sim);
                out
            }
            #[cfg(feature = "physics2d")]
            PROFILE_2D => {
                #[cfg(feature = "physics3d")]
                {
                    shared.sim3 = None;
                }
                let mut sim = shared.sim2.take().unwrap_or_default();
                let out = dim2::step(&settings, dt, tick, &scene, &mut shared, &mut sim)?;
                shared.sim2 = Some(sim);
                out
            }
            other => return Err(check_profile(other).unwrap_err()),
        }
    };
    laps.skip();
    gather::write_back(ctx, ids, &scene, &out)?;
    for (e, c) in &out.controls {
        match c {
            ControlOut::Character(c) => ctx.set(*e, *c)?,
            ControlOut::Platformer(p) => ctx.set(*e, *p)?,
        }
    }
    for (e, j) in &out.joints {
        ctx.set(*e, *j)?;
    }
    for (e, v) in &out.vehicles {
        ctx.set(*e, *v)?;
    }
    for (e, r) in &out.ragdolls {
        ctx.set(*e, *r)?;
    }
    for ev in &out.events {
        ctx.send(*ev)?;
    }
    for s in &out.splashes {
        ctx.send(*s)?;
    }
    for (e, s) in &out.softs {
        ctx.set(*e, *s)?;
    }
    for (e, b) in &out.floats {
        ctx.set(*e, *b)?;
    }
    for (e, b) in &out.sleep.writes {
        ctx.set(*e, *b)?;
    }
    // G2-3's events are registered only once a game names them (LAZY):
    // a world that does not keeps its type table, and nothing could read
    // them, so they are not sent.
    if ctx.id_by_name(JointBroken::NAME).is_ok() {
        for b in &out.broken {
            ctx.send(*b)?;
        }
    }
    if ctx.id_by_name(BodySlept::NAME).is_ok() {
        for &entity in &out.sleep.slept {
            ctx.send(BodySlept { entity })?;
        }
    }
    if ctx.id_by_name(BodyWoke::NAME).is_ok() {
        for &entity in &out.sleep.woke {
            ctx.send(BodyWoke { entity })?;
        }
    }

    // Started/Ended from the pairs the previous step carried over.
    let mut previous = ctx.events::<Contact>()?;
    og_core::sort::sort(&mut previous);
    previous.dedup();
    let current: Vec<Contact> = out.touching.iter().map(|t| t.0).collect();
    let key = |c: &Contact| (c.a, c.b, c.sensor != 0);
    let (mut i, mut j) = (0, 0);
    loop {
        match (previous.get(i), current.get(j)) {
            (None, None) => break,
            (Some(p), Some(c)) if key(p) == key(c) => {
                if p.sensor != c.sensor {
                    // The same overlap, but which side is the sensor changed.
                    sensor_events(ctx, p, false)?;
                    sensor_events(ctx, c, true)?;
                }
                i += 1;
                j += 1;
            }
            (Some(p), c) if c.is_none_or(|c| key(p) < key(c)) => {
                if p.sensor == 0 {
                    ctx.send(CollisionEnded { a: p.a, b: p.b })?;
                } else {
                    sensor_events(ctx, p, false)?;
                }
                i += 1;
            }
            (_, Some(c)) => {
                if c.sensor == 0 {
                    ctx.send(out.touching[j].1)?;
                } else {
                    sensor_events(ctx, c, true)?;
                }
                j += 1;
            }
            (Some(_), None) => unreachable!("handled by the arm above"),
        }
    }
    for c in current {
        ctx.send(c)?;
    }
    laps.lap(stats::Phase::WriteBack);
    if let Some(step) = &out.laps {
        laps.merge(step);
    }
    let pool = laps.phases[stats::Phase::Rapier as usize].split;
    let mut shared = lock(shared);
    let bodies = match () {
        #[cfg(feature = "physics3d")]
        _ if shared.sim3.is_some() => shared.sim3.as_deref().map_or(0, |s| s.world.bodies.len()),
        #[cfg(feature = "physics2d")]
        _ if shared.sim2.is_some() => shared.sim2.as_deref().map_or(0, |s| s.world.bodies.len()),
        _ => 0,
    };
    shared.stats = PhysicsStats {
        tick,
        bodies: bodies as u32,
        threads,
        pool,
        ms: (stats::now_ms() - started).max(0.0),
        phases: laps.phases.to_vec(),
    };
    Ok(())
}

fn sensor_events(ctx: &SystemContext<'_>, c: &Contact, entered: bool) -> Result<()> {
    let mut pairs = Vec::with_capacity(2);
    if c.sensor & 1 != 0 {
        pairs.push((c.a, c.b));
    }
    if c.sensor & 2 != 0 {
        pairs.push((c.b, c.a));
    }
    for (sensor, other) in pairs {
        if entered {
            ctx.send(SensorEntered { sensor, other })?;
        } else {
            ctx.send(SensorExited { sensor, other })?;
        }
    }
    Ok(())
}
