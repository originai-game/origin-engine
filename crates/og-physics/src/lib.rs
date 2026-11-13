//! Origin Engine physics: rapier3d and rapier2d as an engine system over the
//! ECS (docs/design.md §2.4, §2.7).
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
//! | [`WaterSurface`], [`WaterLine`], [`Splash`] | height-field water (3D) and a 2D spring line, splashed by and floating bodies; og.Water (lake, sea, river) is crossed, cut and answered through the same path (`water`) |
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
//! **Layout.** One directory per mechanism. The profiles (`dim2.rs`,
//! `dim3.rs`) compile the shared sources once each, so modules are named
//! by `#[path]` and a file's directory says which mechanism it belongs to:
//!
//! | directory | mechanism |
//! |---|---|
//! | `rigid/` | the rapier world: bodies, colliders, the step, sleep, joints, scene queries, the air's drag, the debug lines |
//! | `character/` | the controllers' move, the 2D platformer, ragdolls |
//! | `vehicle/` | the raycast car, the 2D wheel-joint car, the drivetrain |
//! | `soft/` | Verlet ropes and cloth, PBD soft volumes, garments |
//! | `fluid/` | the 2D and 3D particle fluids |
//! | `water/` | og.Water's bodies, height-field surfaces and lines, wakes, buoyancy |
//!
//! At the top: the system (`tick.rs`), reading the world (`gather.rs`),
//! the state block (`persist.rs`), the lanes and the stats.
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

#[path = "vehicle/drivetrain.rs"]
mod drivetrain;
#[path = "fluid/fluid.rs"]
mod fluid;
#[path = "fluid/fluid3d.rs"]
mod fluid3d;
#[path = "soft/garment.rs"]
mod garment;
mod gather;
mod lanes;
#[path = "rigid/outline.rs"]
mod outline;
mod schema;
#[path = "soft/softbody.rs"]
mod softbody;
mod stats;
#[path = "soft/verlet.rs"]
mod verlet;
mod water;
pub use water::{Wakes, WaterAt, WaterBody, WaterCut};

#[cfg(feature = "physics2d")]
mod dim2;
#[cfg(feature = "physics3d")]
mod dim3;

pub use garment::{GarmentPoints, GarmentPose, GarmentRig};
pub use lanes::{Grains, grains, pool_bodies, set_grains};
pub use outline::outline;
pub use schema::*;
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
mod desc;
mod persist;
#[path = "rigid/query.rs"]
mod query;
mod readout;
mod shared;
mod tick;
pub use desc::*;
use persist::*;
pub use query::*;
pub use readout::*;
pub use shared::*;
use tick::*;

/// The name of the scheduled system.
pub const SYSTEM_NAME: &str = "og.physics";

pub use og_physics_data::MeshDesc;

/// The bit of a mesh handle that marks a mesh generated from world state
/// (a tilemap's merged solid regions, G2-6) rather than loaded as an asset:
/// the shapes of colliders naming one are saved with the world, so a save
/// restores without the mesh registered. Asset handles never set it, and
/// a handle with it still fits a JavaScript number (below 2^53).
pub const GENERATED_MESH: u64 = 1 << 52;

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

#[cfg(feature = "physics3d")]
pub use og_physics_data::bake_trimesh;

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
            let scene = gather::gather(
                &|i| ctx.query(i),
                &|e, id| ctx.get_bytes(e, id),
                lazy,
                &ids,
                &mut ids.kept(),
            )?;
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
        #[cfg(feature = "physics3d")]
        if reload_restored(&mut shared, handle)? {
            return Ok(());
        }
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
        #[cfg(feature = "physics3d")]
        if reload_restored(&mut shared, handle)? {
            return Ok(());
        }
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
            // Outside a system nothing kept can be relied on (`gather::Kept`).
            &mut gather::Kept::default(),
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
