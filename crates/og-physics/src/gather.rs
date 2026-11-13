//! Reading the physics components out of the world in a canonical order,
//! and writing the step's results back.
//!
//! Order is what makes the rebuilt rapier world identical from one run to
//! the next: bodies come in the order of query 0 (archetype creation order,
//! then row order, both captured by a snapshot), static colliders after
//! them in the order of query 3.

use crate::StepOut;
use crate::schema::{
    BODY_KINEMATIC, Buoyancy, CharacterController, Cloth, Collider, ContactHook, Fluid2D, Fluid3D,
    Garment, Joint, Parent, PlatformerController, Ragdoll, RigidBody, Rope, SoftBody, Transform,
    Vehicle, VehicleWheel, Velocity, WaterLine, WaterSurface,
};
use og_core::glam::{Quat, Vec3};
use og_core::{Layout, Reflect, Result};
use og_ecs::{
    ArchetypeView, Entity, FieldRef, QueryDesc, QueryIter, SchemaId, SystemContext, World,
};
use std::sync::{Mutex, MutexGuard};

mod kept;
mod queries;
pub(crate) use kept::*;
pub(crate) use queries::*;

/// A controller on a body: it moves the body after the step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Control {
    Character(CharacterController),
    Platformer(PlatformerController),
}

/// One body as the step sees it.
pub(crate) struct Body {
    pub entity: Entity,
    pub position: Vec3,
    pub quaternion: Quat,
    pub scale: Vec3,
    pub body: RigidBody,
    pub collider: Option<Collider>,
    pub velocity: Option<Velocity>,
    /// Its controller (the body is then kinematic, moved by it).
    pub control: Option<Control>,
    /// Compound colliders: the colliders of child entities (no body of
    /// their own) whose `og.Parent` chain reaches this body, in entity-bits
    /// order, each placed relative to the body.
    pub children: Vec<Child>,
}

/// A collider on a child entity, part of its ancestor body: its pose
/// relative to the body (the transforms between them composed, the body's
/// scale applied) and its scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Child {
    pub entity: Entity,
    pub position: Vec3,
    pub quaternion: Quat,
    pub scale: Vec3,
    pub collider: Collider,
}

/// A collider without a body: static at its transform.
pub(crate) struct Static {
    pub entity: Entity,
    pub position: Vec3,
    pub quaternion: Quat,
    pub scale: Vec3,
    pub collider: Collider,
}

/// The statics one archetype gave a scene, with what says whether they are
/// the ones it gave before.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct StaticRun {
    /// How many of [`Scene::statics`], after the runs before this one.
    pub len: usize,
    /// The write stamps of the archetype's og.Transform and og.Collider
    /// columns ([`ArchetypeView::written`]) when every row of it is in the
    /// run (none joined a body as a child collider): a run that has the
    /// stamps and the length of an earlier one is that run, static for
    /// static. `None` when nothing says so.
    pub written: Option<(u64, u64)>,
}

/// The scene: bodies (RigidBody entities in query order, then the
/// controllers without one), static colliders, and the components physics
/// steps beyond rigid bodies, each in query order.
#[derive(Default)]
pub(crate) struct Scene {
    pub bodies: Vec<Body>,
    pub statics: Vec<Static>,
    /// `statics` by the archetype they were read from, in order, for the
    /// step to pass over the runs it already brought rapier in line with
    /// (`sim::sync`). Only a gather inside a system makes runs a step may
    /// rely on (see [`Kept`]), and only those reach a step.
    pub static_runs: Vec<StaticRun>,
    pub joints: Vec<(Entity, Joint)>,
    pub vehicles: Vec<(Entity, Vehicle)>,
    pub ropes: Vec<(Entity, Vec3, Rope)>,
    pub cloths: Vec<(Entity, Vec3, Quat, Cloth)>,
    pub fluids: Vec<(Entity, Vec3, Fluid2D)>,
    pub surfaces: Vec<(Entity, Vec3, WaterSurface)>,
    pub lines: Vec<(Entity, Vec3, WaterLine)>,
    pub ragdolls: Vec<(Entity, Vec3, Quat, Ragdoll)>,
    pub fluids3: Vec<(Entity, Vec3, Fluid3D)>,
    /// The contact hooks, by entity (ascending).
    pub hooks: og_core::IdMap<u64, ContactHook>,
    /// Each vehicle's og.VehicleWheel children, by vehicle entity, in
    /// entity order (at most [`MAX_WHEELS`]).
    pub wheels: og_core::IdMap<u64, Vec<WheelChild>>,
    /// The soft volumes (lazy), at their world pose, by entity (ascending).
    pub softs: Vec<(Entity, Vec3, Quat, SoftBody)>,
    /// The garments (lazy), by entity (ascending): their skin's pose comes
    /// from the host (`Shared::garment_poses`).
    pub garments: Vec<(Entity, Garment)>,
    /// The floating bodies (og.Buoyancy, lazy), by entity (ascending).
    pub floats: Vec<(Entity, Buoyancy)>,
    /// The world's clock in seconds (its tick over its rate) for the sea
    /// the floating bodies ride; set only with some.
    pub seconds: f64,
}

/// The most og.VehicleWheel children a vehicle takes.
pub(crate) const MAX_WHEELS: usize = 16;

/// An og.VehicleWheel child: its mount in chassis space (the transforms
/// between it and the chassis composed, the chassis's scale applied).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WheelChild {
    pub entity: Entity,
    pub mount: Vec3,
    pub wheel: VehicleWheel,
}

/// The lazily registered components the step reads, when registered
/// (crate::LAZY): read per entity, since a system's queries are fixed when
/// it is scheduled.
#[derive(Clone, Copy, Default)]
pub(crate) struct LazyIds {
    pub hook: Option<SchemaId>,
    pub wheel: Option<SchemaId>,
    pub soft: Option<SchemaId>,
    pub garment: Option<SchemaId>,
    pub buoyancy: Option<SchemaId>,
}

impl LazyIds {
    pub fn of(id: impl Fn(&str) -> Option<SchemaId>) -> LazyIds {
        LazyIds {
            hook: id(ContactHook::NAME),
            wheel: id(VehicleWheel::NAME),
            soft: id(SoftBody::NAME),
            garment: id(Garment::NAME),
            buoyancy: id(Buoyancy::NAME),
        }
    }
}

/// Packed bytes of one component of any entity (`SystemContext::get_bytes`
/// in the tick, `World::get_bytes` outside it).
pub(crate) type GetBytes<'a> = &'a dyn Fn(Entity, SchemaId) -> Result<Vec<u8>>;

/// Where soft bodies on child entities are in the world: their local
/// transform after their parents' down the `og.Parent` chain, as the
/// renderer composes `og.MatrixWorld` (#10 of kestrel-stair's findings: a
/// cape on a child of the body hung at the world origin). Built only when a
/// soft body has a parent.
struct Chains<'a> {
    parents: ByEntity<u64>,
    get: GetBytes<'a>,
    ids: &'a Ids,
}

impl Chains<'_> {
    fn place(&self, e: Entity, position: Vec3, quaternion: Quat) -> (Vec3, Quat) {
        world_pose(
            e,
            (position, quaternion),
            |c| self.parents.get(c).map(Entity::from_bits),
            |p| {
                let bytes = (self.get)(p, self.ids.transform).ok()?;
                self.ids.transform_layout.decode::<Transform>(&bytes).ok()
            },
        )
    }
}

/// The world position and rotation of `e`, whose own `og.Transform` is
/// `local` (position, quaternion): its parents' transforms applied down the
/// `og.Parent` chain (`parent_of`), as the renderer composes
/// `og.MatrixWorld` ([`og_core::math::world_pose`], which audio uses too).
/// A parent's scale scales the child's offset; a parent without
/// `og.Transform` (`transform_of` gives `None`) ends the chain there, and so
/// does a chain longer than 256 (a cycle).
pub fn world_pose(
    e: Entity,
    local: (Vec3, Quat),
    parent_of: impl Fn(Entity) -> Option<Entity>,
    transform_of: impl Fn(Entity) -> Option<Transform>,
) -> (Vec3, Quat) {
    let (p, q, _) =
        og_core::math::world_pose(e, (local.0, local.1, Vec3::ONE), parent_of, |parent| {
            transform_of(parent).map(|t| (t.position, t.quaternion, t.scale))
        });
    (p, q)
}

/// The body a child collider joins: the nearest ancestor down `e`'s
/// `og.Parent` chain that is in `bodies` (its index), with `e`'s pose
/// relative to it (position, rotation, scale: the transforms between them
/// composed, the body's own excluded). None when no ancestor within 256
/// links is a body; a parent without `og.Transform` counts as identity.
fn ancestor_body(
    e: Entity,
    own: (Vec3, Quat, Vec3),
    parents: &ByEntity<u64>,
    bodies: &ByEntity<usize>,
    get: GetBytes<'_>,
    ids: &Ids,
) -> Option<(usize, (Vec3, Quat, Vec3))> {
    let (mut p, mut r, mut s) = own;
    let mut at = e;
    for _ in 0..256 {
        let parent = Entity::from_bits(parents.get(at)?);
        if parent == e {
            return None;
        }
        if let Some(i) = bodies.get(parent) {
            return Some((i, (p, r, s)));
        }
        if let Some(t) = (get)(parent, ids.transform)
            .ok()
            .and_then(|b| ids.transform_layout.decode::<Transform>(&b).ok())
        {
            p = t.position + t.quaternion * (t.scale * p);
            r = t.quaternion * r;
            s *= t.scale;
        }
        at = parent;
    }
    None
}

/// Reads the scene through the system's queries (`q(i)` is query `i` of
/// [`QUERIES`]); `get` reads the transforms of the parents soft bodies
/// hang from. `kept` is what the last gather decoded, reused where the
/// columns were not written since and renewed where they were.
pub(crate) fn gather<'w>(
    q: &dyn Fn(usize) -> Result<QueryIter<'w>>,
    get: GetBytes<'_>,
    lazy: LazyIds,
    ids: &Ids,
    kept: &mut Kept,
) -> Result<Scene> {
    let mut colliders = ByEntity::new();
    for (at, a) in q(Q_BODY_COLLIDERS)?.enumerate() {
        let rows = ids.colliders(&mut kept.body_colliders, at, &a)?;
        for (e, c) in a.entities().iter().zip(rows) {
            colliders.insert(*e, *c);
        }
    }
    let mut velocities = ByEntity::new();
    for a in q(Q_BODY_VELOCITIES)? {
        let linear = a.read::<Vec3>(ids.v_linear)?;
        let angular = a.read::<Vec3>(ids.v_angular)?;
        for (row, e) in a.entities().iter().enumerate() {
            velocities.insert(
                *e,
                Velocity {
                    linear: linear[row],
                    angular: angular[row],
                },
            );
        }
    }
    let mut controls = ByEntityControl::default();
    for a in q(Q_BODY_CHARACTERS)? {
        let rows: Vec<CharacterController> = ids.character.read(&a)?;
        for (e, c) in a.entities().iter().zip(rows) {
            controls.insert(*e, Control::Character(c));
        }
    }
    for a in q(Q_BODY_PLATFORMERS)? {
        let rows: Vec<PlatformerController> = ids.platformer.read(&a)?;
        for (e, c) in a.entities().iter().zip(rows) {
            controls.insert(*e, Control::Platformer(c));
        }
    }
    let mut scene = Scene::default();
    for (at, a) in q(Q_BODIES)?.enumerate() {
        scene.bodies.reserve(a.len());
        let position = a.read::<Vec3>(ids.t_position)?;
        let quaternion = a.read::<Quat>(ids.t_quaternion)?;
        let scale = a.read::<Vec3>(ids.t_scale)?;
        let bodies = Kept::rows(
            &mut kept.bodies,
            at,
            &a,
            ids.body,
            &ids.body_fields,
            &ids.body_layout,
        )?;
        for (row, (e, body)) in a.entities().iter().zip(bodies.iter().copied()).enumerate() {
            let control = controls.get(*e);
            scene.bodies.push(Body {
                entity: *e,
                position: position[row],
                quaternion: quaternion[row],
                scale: scale[row],
                body: if control.is_some() {
                    RigidBody {
                        kind: BODY_KINEMATIC,
                        ..body
                    }
                } else {
                    body
                },
                collider: colliders.get(*e),
                velocity: if control.is_some() {
                    None
                } else {
                    velocities.get(*e)
                },
                control,
                children: Vec::new(),
            });
        }
    }
    let mut controllers = 0;
    for (index, character) in [(Q_CHARACTERS, true), (Q_PLATFORMERS, false)] {
        for a in q(index)? {
            let position = a.read::<Vec3>(ids.t_position)?;
            let quaternion = a.read::<Quat>(ids.t_quaternion)?;
            let scale = a.read::<Vec3>(ids.t_scale)?;
            let colliders = ids.colliders(&mut kept.controller_colliders, controllers, &a)?;
            controllers += 1;
            let control: Vec<Control> = if character {
                ids.character
                    .read::<CharacterController>(&a)?
                    .into_iter()
                    .map(Control::Character)
                    .collect()
            } else {
                ids.platformer
                    .read::<PlatformerController>(&a)?
                    .into_iter()
                    .map(Control::Platformer)
                    .collect()
            };
            for (row, e) in a.entities().iter().enumerate() {
                scene.bodies.push(Body {
                    entity: *e,
                    position: position[row],
                    quaternion: quaternion[row],
                    scale: scale[row],
                    body: RigidBody::kinematic(),
                    collider: Some(colliders[row]),
                    velocity: None,
                    control: Some(control[row]),
                    children: Vec::new(),
                });
            }
        }
    }
    let mut parents = ByEntity::new();
    let mut any_parent = false;
    let mut parented = Vec::new();
    for a in q(Q_PARENTS)? {
        let p = a.read::<u64>(ids.parent)?;
        for (e, parent) in a.entities().iter().zip(p.iter()) {
            parents.insert(*e, *parent);
            any_parent = true;
            if lazy.wheel.is_some() {
                parented.push(*e);
            }
        }
    }
    // The bodies a child collider can join: rigid bodies a controller
    // does not move (a controller moves its own shape alone).
    let mut joinable = ByEntity::new();
    if any_parent {
        for (i, b) in scene.bodies.iter().enumerate() {
            if b.control.is_none() {
                joinable.insert(b.entity, i);
            }
        }
    }
    let mut joined = false;
    for (at, a) in q(Q_STATIC_COLLIDERS)?.enumerate() {
        let position = a.read::<Vec3>(ids.t_position)?;
        let quaternion = a.read::<Quat>(ids.t_quaternion)?;
        let scale = a.read::<Vec3>(ids.t_scale)?;
        let rows = ids.colliders(&mut kept.static_colliders, at, &a)?;
        let first = scene.statics.len();
        for (row, (e, collider)) in a.entities().iter().zip(rows.iter().copied()).enumerate() {
            let own = (position[row], quaternion[row], scale[row]);
            if any_parent
                && let Some((i, (p, r, s))) = ancestor_body(*e, own, &parents, &joinable, get, ids)
            {
                let body = &mut scene.bodies[i];
                body.children.push(Child {
                    entity: *e,
                    position: p * body.scale,
                    quaternion: r,
                    scale: s * body.scale,
                    collider,
                });
                joined = true;
                continue;
            }
            scene.statics.push(Static {
                entity: *e,
                position: own.0,
                quaternion: own.1,
                scale: own.2,
                collider,
            });
        }
        let len = scene.statics.len() - first;
        scene.static_runs.push(StaticRun {
            len,
            written: (len == a.len())
                .then(|| a.written(ids.transform).zip(a.written(ids.collider)))
                .flatten(),
        });
    }
    if joined {
        for b in &mut scene.bodies {
            if b.children.len() > 1 {
                b.children.sort_unstable_by_key(|c| c.entity.to_bits());
            }
        }
    }
    // og.ContactHook is registered only once a game uses it (lazy): read
    // then, per collider entity, as the step needs it by entity.
    if let Some(id) = lazy.hook {
        let colliders = scene
            .bodies
            .iter()
            .flat_map(|b| std::iter::once(b.entity).chain(b.children.iter().map(|c| c.entity)))
            .chain(scene.statics.iter().map(|s| s.entity));
        for e in colliders {
            if let Ok(bytes) = get(e, id) {
                scene
                    .hooks
                    .insert(e.to_bits(), ids.hook_layout.decode::<ContactHook>(&bytes)?);
            }
        }
    }
    for a in q(Q_JOINTS)? {
        let rows: Vec<Joint> = ids.joint.read(&a)?;
        scene.joints.extend(a.entities().iter().copied().zip(rows));
    }
    for a in q(Q_VEHICLES)? {
        let rows: Vec<Vehicle> = ids.vehicle.read(&a)?;
        scene
            .vehicles
            .extend(a.entities().iter().copied().zip(rows));
    }
    // og.VehicleWheel children (lazy): the parented entities that have one,
    // joined to the nearest vehicle up their chain.
    if let Some(wheel_id) = lazy.wheel.filter(|_| !scene.vehicles.is_empty()) {
        let mut cars = ByEntity::new();
        for (i, (e, _)) in scene.vehicles.iter().enumerate() {
            cars.insert(*e, i);
        }
        let scale_of = |e: Entity| {
            scene
                .bodies
                .iter()
                .find(|b| b.entity == e)
                .map_or(Vec3::ONE, |b| b.scale)
        };
        let mut found: Vec<(u64, WheelChild)> = Vec::new();
        for e in parented {
            let Ok(bytes) = get(e, wheel_id) else {
                continue;
            };
            let wheel = ids.wheel_layout.decode::<VehicleWheel>(&bytes)?;
            let own = (get)(e, ids.transform)
                .ok()
                .and_then(|b| ids.transform_layout.decode::<Transform>(&b).ok())
                .map_or((Vec3::ZERO, Quat::IDENTITY, Vec3::ONE), |t| {
                    (t.position, t.quaternion, t.scale)
                });
            if let Some((i, (p, _, _))) = ancestor_body(e, own, &parents, &cars, get, ids) {
                let car = scene.vehicles[i].0;
                found.push((
                    car.to_bits(),
                    WheelChild {
                        entity: e,
                        mount: p * scale_of(car),
                        wheel,
                    },
                ));
            }
        }
        for (car, w) in found {
            scene.wheels.entry(car).or_default().push(w);
        }
        for list in scene.wheels.values_mut() {
            list.sort_unstable_by_key(|w| w.entity.to_bits());
            list.truncate(MAX_WHEELS);
        }
    }
    let chains = any_parent.then_some(Chains { parents, get, ids });
    macro_rules! placed {
        ($index:expr, $rows:expr, $list:expr, $make:expr) => {
            for a in q($index)? {
                let position = a.read::<Vec3>(ids.t_position)?;
                let quaternion = a.read::<Quat>(ids.t_quaternion)?;
                let rows = $rows.read(&a)?;
                for (row, (e, c)) in a.entities().iter().zip(rows).enumerate() {
                    let (p, r) = match &chains {
                        Some(chains) => chains.place(*e, position[row], quaternion[row]),
                        None => (position[row], quaternion[row]),
                    };
                    $list.push($make(*e, p, r, c));
                }
            }
        };
    }
    placed!(Q_ROPES, ids.rope, scene.ropes, |e, p, _q, c| (e, p, c));
    placed!(Q_CLOTHS, ids.cloth, scene.cloths, |e, p, q, c| (e, p, q, c));
    placed!(Q_FLUIDS, ids.fluid, scene.fluids, |e, p, _q, c| (e, p, c));
    placed!(Q_SURFACES, ids.surface, scene.surfaces, |e, p, _q, c| (
        e, p, c
    ));
    placed!(Q_LINES, ids.line, scene.lines, |e, p, _q, c| (e, p, c));
    placed!(Q_RAGDOLLS, ids.ragdoll, scene.ragdolls, |e, p, q, c| (
        e, p, q, c
    ));
    placed!(Q_FLUIDS3, ids.fluid3, scene.fluids3, |e, p, _q, c| (
        e, p, c
    ));
    Ok(scene)
}

/// The og.Garment entities (`garment`, lazily registered, so found by a
/// query made while the step runs), in entity order.
pub(crate) fn garments<'w>(
    query: &dyn Fn(&QueryDesc) -> Result<QueryIter<'w>>,
    get: GetBytes<'_>,
    garment: SchemaId,
) -> Result<Vec<(Entity, Garment)>> {
    let desc = QueryDesc {
        read: vec![Garment::NAME.to_string()],
        ..QueryDesc::default()
    };
    let layout = Layout::of(&Garment::schema());
    let mut out = Vec::new();
    for a in query(&desc)? {
        for e in a.entities() {
            out.push((*e, layout.decode::<Garment>(&get(*e, garment)?)?));
        }
    }
    og_core::sort::sort_by_key(&mut out, |s| s.0.to_bits());
    Ok(out)
}

/// The og.SoftBody entities (`soft`, lazily registered, so found by a
/// query made while the step runs, `query`) at their world pose: their
/// transform after their parents' down the og.Parent chain. In entity
/// order.
pub(crate) fn softs<'w>(
    query: &dyn Fn(&QueryDesc) -> Result<QueryIter<'w>>,
    get: GetBytes<'_>,
    soft: SchemaId,
    ids: &Ids,
) -> Result<Vec<(Entity, Vec3, Quat, SoftBody)>> {
    let desc = QueryDesc {
        read: vec![TRANSFORM.to_string(), SoftBody::NAME.to_string()],
        ..QueryDesc::default()
    };
    let parent_of = |c: Entity| {
        let bytes = get(c, ids.parent_id).ok()?;
        let p = ids.parent_layout.decode::<Parent>(&bytes).ok()?;
        (p.entity != 0).then(|| Entity::from_bits(p.entity))
    };
    let transform_of = |p: Entity| {
        let bytes = get(p, ids.transform).ok()?;
        ids.transform_layout.decode::<Transform>(&bytes).ok()
    };
    let mut out = Vec::new();
    for a in query(&desc)? {
        let position = a.read::<Vec3>(ids.t_position)?;
        let quaternion = a.read::<Quat>(ids.t_quaternion)?;
        for (row, e) in a.entities().iter().enumerate() {
            let body = ids.soft_layout.decode::<SoftBody>(&get(*e, soft)?)?;
            let (p, q) = world_pose(
                *e,
                (position[row], quaternion[row]),
                parent_of,
                transform_of,
            );
            out.push((*e, p, q, body));
        }
    }
    og_core::sort::sort_by_key(&mut out, |s| s.0.to_bits());
    Ok(out)
}

/// The og.Buoyancy entities (`buoyancy`, lazily registered, so found by a
/// query made while the step runs), in entity order.
pub(crate) fn floats<'w>(
    query: &dyn Fn(&QueryDesc) -> Result<QueryIter<'w>>,
    get: GetBytes<'_>,
    buoyancy: SchemaId,
) -> Result<Vec<(Entity, Buoyancy)>> {
    let desc = QueryDesc {
        read: vec![Buoyancy::NAME.to_string()],
        ..QueryDesc::default()
    };
    let layout = Layout::of(&Buoyancy::schema());
    let mut out = Vec::new();
    for a in query(&desc)? {
        for e in a.entities() {
            out.push((*e, layout.decode::<Buoyancy>(&get(*e, buoyancy)?)?));
        }
    }
    og_core::sort::sort_by_key(&mut out, |s| s.0.to_bits());
    Ok(out)
}

/// Controllers by entity index, valid for one tick.
#[derive(Default)]
struct ByEntityControl(Vec<Option<Control>>);

impl ByEntityControl {
    fn insert(&mut self, e: Entity, value: Control) {
        let i = e.index() as usize;
        if i >= self.0.len() {
            self.0.resize(i + 1, None);
        }
        self.0[i] = Some(value);
    }
    fn get(&self, e: Entity) -> Option<Control> {
        self.0.get(e.index() as usize).copied().flatten()
    }
}

/// Writes the moved bodies' poses back (dynamic bodies and controllers)
/// and the dynamic bodies' velocities; a dynamic body without an
/// `og.Velocity` gets one (applied when the system returns).
pub(crate) fn write_back(
    ctx: &mut SystemContext<'_>,
    ids: &Ids,
    scene: &Scene,
    out: &StepOut,
) -> Result<()> {
    let mut k = 0;
    let mut velocities = ByEntity::new();
    for index in [Q_BODIES, Q_CHARACTERS, Q_PLATFORMERS] {
        for a in ctx.query(index)? {
            let mut position = a.write::<Vec3>(ids.t_position)?;
            let mut quaternion = a.write::<Quat>(ids.t_quaternion)?;
            for row in 0..a.len() {
                debug_assert_eq!(scene.bodies[k].entity, a.entities()[row]);
                if let Some((p, q, v)) = out.moved[k] {
                    position[row] = p;
                    quaternion[row] = q;
                    if let Some(v) = v {
                        velocities.insert(scene.bodies[k].entity, v);
                    }
                }
                k += 1;
            }
        }
    }
    for a in ctx.query(Q_BODY_VELOCITIES)? {
        let mut linear = a.write::<Vec3>(ids.v_linear)?;
        let mut angular = a.write::<Vec3>(ids.v_angular)?;
        for (row, e) in a.entities().iter().enumerate() {
            if let Some(v) = velocities.get(*e) {
                linear[row] = v.linear;
                angular[row] = v.angular;
            }
        }
    }
    for (body, moved) in scene.bodies.iter().zip(&out.moved) {
        if let (None, Some((_, _, Some(v)))) = (body.velocity, moved)
            && body.control.is_none()
        {
            ctx.insert(body.entity, *v)?;
        }
    }
    Ok(())
}
