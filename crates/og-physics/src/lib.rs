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

#[cfg(not(any(feature = "physics2d", feature = "physics3d")))]
compile_error!("og-physics needs the `physics2d` or the `physics3d` feature (both are default)");

mod gather;
mod schema;

#[cfg(feature = "physics2d")]
mod dim2;
#[cfg(feature = "physics3d")]
mod dim3;

pub use schema::*;

use gather::{Ids, Scene, QUERIES};
use og_core::glam::{Quat, Vec3};
use og_core::{Error, ErrorKind, Result};
use og_ecs::{Entity, StateBlock, SystemContext, SystemDesc, World};
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
    /// Distance from the origin along the normalised direction.
    pub distance: f32,
    pub point: Vec3,
    /// The surface normal at `point` (z = 0 in 2D).
    pub normal: Vec3,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ShapeKey {
    pub shape: u32,
    pub mesh: u64,
    pub scale: [u32; 3],
}

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
    pub meshes: BTreeMap<u64, Arc<MeshDesc>>,
    #[cfg(feature = "physics3d")]
    pub shapes3: BTreeMap<ShapeKey, rapier3d::prelude::SharedShape>,
    #[cfg(feature = "physics2d")]
    pub shapes2: BTreeMap<ShapeKey, rapier2d::prelude::SharedShape>,
    /// The colliders [`PhysicsSystem::query_system`] recorded this tick, and
    /// the query view built from them on the first query that asked.
    pub recorded: Option<(Physics, Scene)>,
    pub recorded_query: Option<Arc<QueryInner>>,
}

/// At most this many mesh-derived shapes are cached (a scale animated every
/// tick would otherwise grow the cache without bound).
pub(crate) const SHAPE_CACHE_LIMIT: usize = 1024;

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
    #[cfg(feature = "physics3d")]
    D3(Box<dim3::Sim>),
    #[cfg(feature = "physics2d")]
    D2(Box<dim2::Sim>),
}

fn codec() -> impl bincode::Options {
    use bincode::Options;
    bincode::DefaultOptions::new().reject_trailing_bytes()
}

impl StateBlock for PhysicsState {
    fn save(&self, out: &mut Vec<u8>) {
        use bincode::Options;
        let shared = lock(&self.0);
        let encoded = match () {
            #[cfg(feature = "physics3d")]
            _ if shared.sim3.is_some() => {
                out.push(PROFILE_3D as u8);
                codec().serialize_into(&mut *out, shared.sim3.as_deref().unwrap())
            }
            #[cfg(feature = "physics2d")]
            _ if shared.sim2.is_some() => {
                out.push(PROFILE_2D as u8);
                codec().serialize_into(&mut *out, shared.sim2.as_deref().unwrap())
            }
            _ => Ok(()),
        };
        encoded.expect("a physics world always serializes into memory");
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        use bincode::Options;
        let parse = |e: bincode::Error| {
            Error::new(
                ErrorKind::Parse,
                format!("the rapier world does not decode: {e}"),
            )
        };
        let saved = match bytes.split_first() {
            None => Saved::Empty,
            #[cfg(feature = "physics3d")]
            Some((&tag, rest)) if tag as u32 == PROFILE_3D => {
                Saved::D3(codec().deserialize(rest).map_err(parse)?)
            }
            #[cfg(feature = "physics2d")]
            Some((&tag, rest)) if tag as u32 == PROFILE_2D => {
                Saved::D2(codec().deserialize(rest).map_err(parse)?)
            }
            Some((tag, _)) => {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("a {tag}D physics world; this engine lacks that profile"),
                ))
            }
        };
        Ok(Box::new(saved))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut shared = lock(&self.0);
        // The restored colliders are as saved, whatever was registered before.
        shared.stale_meshes.clear();
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
            #[cfg(feature = "physics3d")]
            Some(Saved::D3(sim)) => shared.sim3 = Some(sim),
            #[cfg(feature = "physics2d")]
            Some(Saved::D2(sim)) => shared.sim2 = Some(sim),
        }
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
        impl Fn(&mut SystemContext<'_>) -> Result<()> + Send + 'static,
    ) {
        let mut desc = SystemDesc::new(QUERY_SYSTEM_NAME)
            .read::<Physics>()
            .read::<RigidBody>()
            .read::<Collider>()
            .read::<Velocity>();
        desc.reads.push(gather::TRANSFORM.to_string());
        for q in QUERIES {
            desc = desc.query(q.desc(false));
        }
        let (shared, ids) = (self.shared.clone(), self.ids.clone());
        let run = move |ctx: &mut SystemContext<'_>| {
            let settings = *ctx.resource::<Physics>()?;
            let scene = gather::gather(&|i| ctx.query(i), &ids)?;
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
                format!("no tick has recorded the colliders yet: {QUERY_SYSTEM_NAME} runs at the start of each tick; between ticks, query the world"),
            ));
        };
        let built = QueryInner::build(settings.profile, &scene, &mut shared);
        let inner = Arc::new(built?);
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
        .read::<RigidBody>()
        .read::<Collider>()
        .write::<Velocity>()
        .read::<Contact>()
        .write::<Contact>()
        .write::<CollisionStarted>()
        .write::<CollisionEnded>()
        .write::<SensorEntered>()
        .write::<SensorExited>();
    desc.writes.push(gather::TRANSFORM.to_string());
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
            ))
        }
    };
    if built {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Invalid,
            format!("og.Physics.profile {profile}: this engine was built without the physics{profile}d feature"),
        ))
    }
}

impl PhysicsSystem {
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
        #[cfg(feature = "physics3d")]
        shared.shapes3.retain(|k, _| k.mesh != handle);
        #[cfg(feature = "physics2d")]
        shared.shapes2.retain(|k, _| k.mesh != handle);
        Ok(())
    }

    /// Removes mesh `handle`; colliders still naming it fail the next step.
    pub fn remove_mesh(&self, handle: u64) {
        let mut shared = lock(&self.shared);
        shared.meshes.remove(&handle);
        shared.stale_meshes.insert(handle);
        #[cfg(feature = "physics3d")]
        shared.shapes3.retain(|k, _| k.mesh != handle);
        #[cfg(feature = "physics2d")]
        shared.shapes2.retain(|k, _| k.mesh != handle);
    }

    /// A query view of the colliders as `world` holds them now (outside a
    /// tick). Build one and run many queries on it; it does not follow
    /// later changes to the world.
    pub fn query(&self, world: &World) -> Result<PhysicsQuery> {
        let settings = *world.resource::<Physics>()?;
        let scene = gather::gather(&|i| world.query(&QUERIES[i].desc(false)), &self.ids)?;
        let mut shared = lock(&self.shared);
        let inner = Arc::new(QueryInner::build(settings.profile, &scene, &mut shared)?);
        Ok(PhysicsQuery {
            inner,
            shared: self.shared.clone(),
        })
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
    if d > 0.0 {
        d
    } else {
        f32::MAX
    }
}

impl PhysicsQuery {
    /// The nearest collider along a ray.
    pub fn raycast(&self, desc: &RaycastDesc) -> Result<Option<RaycastHit>> {
        let max = max_distance(desc.max_distance);
        match &*self.inner {
            #[cfg(feature = "physics3d")]
            QueryInner::D3(q) => q.raycast(desc.origin, desc.direction, max, &desc.filter),
            #[cfg(feature = "physics2d")]
            QueryInner::D2(q) => q.raycast(desc.origin, desc.direction, max, &desc.filter),
        }
    }

    /// The first collider a shape meets when swept along a direction.
    pub fn shape_cast(&self, desc: &ShapeCastDesc) -> Result<Option<ShapeCastHit>> {
        let max = max_distance(desc.max_distance);
        let mut shared = lock(&self.shared);
        match &*self.inner {
            #[cfg(feature = "physics3d")]
            QueryInner::D3(q) => q.shape_cast(desc, max, &mut shared),
            #[cfg(feature = "physics2d")]
            QueryInner::D2(q) => q.shape_cast(desc, max, &mut shared),
        }
    }

    /// Every collider overlapping a shape, sorted by entity id.
    pub fn overlap(&self, desc: &OverlapDesc) -> Result<Vec<Entity>> {
        let mut shared = lock(&self.shared);
        match &*self.inner {
            #[cfg(feature = "physics3d")]
            QueryInner::D3(q) => q.overlap(desc, &mut shared),
            #[cfg(feature = "physics2d")]
            QueryInner::D2(q) => q.overlap(desc, &mut shared),
        }
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

/// What one step produced.
pub(crate) struct StepOut {
    /// Per `Scene::bodies` index: the new pose and velocity of dynamic bodies.
    pub moved: Vec<Option<(Vec3, Quat, Velocity)>>,
    /// Every touching pair, sorted by (a, b), with the deepest point and the
    /// normal from a to b (zero for sensor overlaps).
    pub touching: Vec<(Contact, Vec3, Vec3)>,
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
        return Ok(());
    }
    let scene: Scene = gather::gather(&|i| ctx.query(i), ids)?;
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
                let out = dim3::step(&settings, dt, &scene, &mut shared, &mut sim)?;
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
                let out = dim2::step(&settings, dt, &scene, &mut shared, &mut sim)?;
                shared.sim2 = Some(sim);
                out
            }
            other => return Err(check_profile(other).unwrap_err()),
        }
    };
    gather::write_back(ctx, ids, &scene, &out)?;

    // Started/Ended from the pairs the previous step carried over.
    let mut previous = ctx.events::<Contact>()?;
    previous.sort();
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
                let (_, point, normal) = out.touching[j];
                if c.sensor == 0 {
                    ctx.send(CollisionStarted {
                        a: c.a,
                        b: c.b,
                        point,
                        normal,
                    })?;
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
