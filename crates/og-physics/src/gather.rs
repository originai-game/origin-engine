//! Reading the physics components out of the world in a canonical order,
//! and writing the step's results back.
//!
//! Order is what makes the rebuilt rapier world identical from one run to
//! the next: bodies come in the order of query 0 (archetype creation order,
//! then row order, both captured by a snapshot), static colliders after
//! them in the order of query 3.

use crate::schema::{Collider, RigidBody, Transform, Velocity};
use crate::StepOut;
use og_core::glam::{Quat, Vec3};
use og_core::{Layout, Reflect, Result};
use og_ecs::{
    ArchetypeView, Entity, FieldRef, QueryDesc, QueryIter, SchemaId, SystemContext, World,
};

/// The engine's transform component, which physics reads and writes by name
/// (it registers the schema, never a Rust type, so the engine's own Rust
/// `og.Transform` type, wherever it lives, is adopted unchanged).
pub(crate) const TRANSFORM: &str = Transform::NAME;

pub(crate) struct QuerySpec {
    read: &'static [&'static str],
    write: &'static [&'static str],
    without: &'static [&'static str],
}

impl QuerySpec {
    /// The query as the system declares it (`in_system`), or read-only for a
    /// world query outside the tick.
    pub fn desc(&self, in_system: bool) -> QueryDesc {
        let names = |l: &[&str]| l.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let (mut read, write) = (names(self.read), names(self.write));
        if in_system {
            QueryDesc {
                read,
                write,
                without: names(self.without),
            }
        } else {
            read.extend(write);
            QueryDesc {
                read,
                write: Vec::new(),
                without: names(self.without),
            }
        }
    }
}

const Q_BODIES: usize = 0;
const Q_BODY_COLLIDERS: usize = 1;
const Q_BODY_VELOCITIES: usize = 2;
const Q_STATIC_COLLIDERS: usize = 3;

/// The system's queries, by index.
pub(crate) const QUERIES: [QuerySpec; 4] = [
    // Q_BODIES: every body, its transform written back.
    QuerySpec {
        read: &[RigidBody::NAME],
        write: &[TRANSFORM],
        without: &[],
    },
    // Q_BODY_COLLIDERS
    QuerySpec {
        read: &[RigidBody::NAME, Collider::NAME],
        write: &[],
        without: &[],
    },
    // Q_BODY_VELOCITIES: velocities read, and written back for dynamic bodies.
    QuerySpec {
        read: &[RigidBody::NAME],
        write: &[Velocity::NAME],
        without: &[],
    },
    // Q_STATIC_COLLIDERS: a collider with no body is static at its transform.
    QuerySpec {
        read: &[TRANSFORM, Collider::NAME],
        write: &[],
        without: &[RigidBody::NAME],
    },
];

pub(crate) struct Ids {
    t_position: FieldRef,
    t_quaternion: FieldRef,
    t_scale: FieldRef,
    body_fields: Vec<FieldRef>,
    body_layout: Layout,
    collider_fields: Vec<FieldRef>,
    collider_layout: Layout,
    v_linear: FieldRef,
    v_angular: FieldRef,
}

fn fields_of<T: Reflect>(world: &World) -> Result<(Vec<FieldRef>, Layout)> {
    let schema = T::schema();
    let fields = schema
        .fields
        .iter()
        .map(|f| world.field::<T>(&f.name))
        .collect::<Result<Vec<_>>>()?;
    Ok((fields, Layout::of(&schema)))
}

impl Ids {
    /// Registers every physics type, in a fixed order.
    pub fn register(world: &mut World) -> Result<Ids> {
        let transform: SchemaId = world.register_schema(Transform::schema())?;
        world.register::<RigidBody>()?;
        world.register::<Collider>()?;
        world.register::<Velocity>()?;
        world.register::<crate::Physics>()?;
        world.register::<crate::Contact>()?;
        world.register::<crate::CollisionStarted>()?;
        world.register::<crate::CollisionEnded>()?;
        world.register::<crate::SensorEntered>()?;
        world.register::<crate::SensorExited>()?;
        let (body_fields, body_layout) = fields_of::<RigidBody>(world)?;
        let (collider_fields, collider_layout) = fields_of::<Collider>(world)?;
        Ok(Ids {
            t_position: world.field_by_id(transform, "position")?,
            t_quaternion: world.field_by_id(transform, "quaternion")?,
            t_scale: world.field_by_id(transform, "scale")?,
            body_fields,
            body_layout,
            collider_fields,
            collider_layout,
            v_linear: world.field::<Velocity>("linear")?,
            v_angular: world.field::<Velocity>("angular")?,
        })
    }
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
}

/// A collider without a body: static at its transform.
pub(crate) struct Static {
    pub entity: Entity,
    pub position: Vec3,
    pub quaternion: Quat,
    pub scale: Vec3,
    pub collider: Collider,
}

#[derive(Default)]
pub(crate) struct Scene {
    pub bodies: Vec<Body>,
    pub statics: Vec<Static>,
}

/// Every row of component `T` in `a`, decoded from its columns.
fn read_rows<T: Reflect>(
    a: &ArchetypeView<'_>,
    fields: &[FieldRef],
    layout: &Layout,
) -> Result<Vec<T>> {
    let columns = fields
        .iter()
        .map(|f| a.read_bytes(*f))
        .collect::<Result<Vec<_>>>()?;
    let mut packed = vec![0u8; layout.size()];
    let mut out = Vec::with_capacity(a.len());
    for row in 0..a.len() {
        for (i, column) in columns.iter().enumerate() {
            let range = layout.range(i);
            let n = range.len();
            packed[range].copy_from_slice(&column[row * n..(row + 1) * n]);
        }
        out.push(layout.decode::<T>(&packed)?);
    }
    Ok(out)
}

/// A sparse map by entity index, valid for one tick (live entities have
/// distinct indices).
struct ByEntity<T>(Vec<Option<T>>);

impl<T: Copy> ByEntity<T> {
    fn new() -> Self {
        ByEntity(Vec::new())
    }
    fn insert(&mut self, e: Entity, value: T) {
        let i = e.index() as usize;
        if i >= self.0.len() {
            self.0.resize(i + 1, None);
        }
        self.0[i] = Some(value);
    }
    fn get(&self, e: Entity) -> Option<T> {
        self.0.get(e.index() as usize).copied().flatten()
    }
}

/// Reads the scene through the system's queries (`q(i)` is query `i` of
/// [`QUERIES`]).
pub(crate) fn gather<'w>(q: &dyn Fn(usize) -> Result<QueryIter<'w>>, ids: &Ids) -> Result<Scene> {
    let mut colliders = ByEntity::new();
    for a in q(Q_BODY_COLLIDERS)? {
        let rows: Vec<Collider> = read_rows(&a, &ids.collider_fields, &ids.collider_layout)?;
        for (e, c) in a.entities().iter().zip(rows) {
            colliders.insert(*e, c);
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
    let mut scene = Scene::default();
    for a in q(Q_BODIES)? {
        let position = a.read::<Vec3>(ids.t_position)?;
        let quaternion = a.read::<Quat>(ids.t_quaternion)?;
        let scale = a.read::<Vec3>(ids.t_scale)?;
        let bodies: Vec<RigidBody> = read_rows(&a, &ids.body_fields, &ids.body_layout)?;
        for (row, (e, body)) in a.entities().iter().zip(bodies).enumerate() {
            scene.bodies.push(Body {
                entity: *e,
                position: position[row],
                quaternion: quaternion[row],
                scale: scale[row],
                body,
                collider: colliders.get(*e),
                velocity: velocities.get(*e),
            });
        }
    }
    for a in q(Q_STATIC_COLLIDERS)? {
        let position = a.read::<Vec3>(ids.t_position)?;
        let quaternion = a.read::<Quat>(ids.t_quaternion)?;
        let scale = a.read::<Vec3>(ids.t_scale)?;
        let rows: Vec<Collider> = read_rows(&a, &ids.collider_fields, &ids.collider_layout)?;
        for (row, (e, collider)) in a.entities().iter().zip(rows).enumerate() {
            scene.statics.push(Static {
                entity: *e,
                position: position[row],
                quaternion: quaternion[row],
                scale: scale[row],
                collider,
            });
        }
    }
    Ok(scene)
}

/// Writes the dynamic bodies' poses and velocities back; a dynamic body
/// without an `og.Velocity` gets one (applied when the system returns).
pub(crate) fn write_back(
    ctx: &mut SystemContext<'_>,
    ids: &Ids,
    scene: &Scene,
    out: &StepOut,
) -> Result<()> {
    let mut k = 0;
    let mut velocities = ByEntity::new();
    for a in ctx.query(Q_BODIES)? {
        let mut position = a.write::<Vec3>(ids.t_position)?;
        let mut quaternion = a.write::<Quat>(ids.t_quaternion)?;
        for row in 0..a.len() {
            debug_assert_eq!(scene.bodies[k].entity, a.entities()[row]);
            if let Some((p, q, v)) = out.moved[k] {
                position[row] = p;
                quaternion[row] = q;
                velocities.insert(scene.bodies[k].entity, v);
            }
            k += 1;
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
        if let (None, Some((_, _, v))) = (body.velocity, moved) {
            ctx.insert(body.entity, *v)?;
        }
    }
    Ok(())
}
