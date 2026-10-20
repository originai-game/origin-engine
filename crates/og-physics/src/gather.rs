//! Reading the physics components out of the world in a canonical order,
//! and writing the step's results back.
//!
//! Order is what makes the rebuilt rapier world identical from one run to
//! the next: bodies come in the order of query 0 (archetype creation order,
//! then row order, both captured by a snapshot), static colliders after
//! them in the order of query 3.

use crate::StepOut;
use crate::schema::{BODY_KINEMATIC, Collider, RigidBody, Transform, Velocity};
use crate::schema_ext::{
    CharacterController, Cloth, Fluid2D, Fluid3D, Joint, PlatformerController, Ragdoll, Rope,
    Vehicle, WaterLine, WaterSurface,
};
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
const Q_CHARACTERS: usize = 4;
const Q_PLATFORMERS: usize = 5;
const Q_BODY_CHARACTERS: usize = 6;
const Q_BODY_PLATFORMERS: usize = 7;
const Q_JOINTS: usize = 8;
const Q_VEHICLES: usize = 9;
const Q_ROPES: usize = 10;
const Q_CLOTHS: usize = 11;
const Q_FLUIDS: usize = 12;
const Q_SURFACES: usize = 13;
const Q_LINES: usize = 14;
const Q_RAGDOLLS: usize = 15;
const Q_FLUIDS3: usize = 16;

const CHARACTER: &str = CharacterController::NAME;
const PLATFORMER: &str = PlatformerController::NAME;

/// The system's queries, by index.
pub(crate) const QUERIES: [QuerySpec; 17] = [
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
    // Q_STATIC_COLLIDERS: a collider with no body (and no controller) is static at its transform.
    QuerySpec {
        read: &[TRANSFORM, Collider::NAME],
        write: &[],
        without: &[RigidBody::NAME, CHARACTER, PLATFORMER],
    },
    // Q_CHARACTERS: a character controller without a RigidBody is a kinematic body.
    QuerySpec {
        read: &[Collider::NAME, CHARACTER],
        write: &[TRANSFORM],
        without: &[RigidBody::NAME],
    },
    // Q_PLATFORMERS: likewise for platformers.
    QuerySpec {
        read: &[Collider::NAME, PLATFORMER],
        write: &[TRANSFORM],
        without: &[RigidBody::NAME, CHARACTER],
    },
    // Q_BODY_CHARACTERS: controllers on RigidBody entities.
    QuerySpec {
        read: &[RigidBody::NAME, CHARACTER],
        write: &[],
        without: &[],
    },
    // Q_BODY_PLATFORMERS
    QuerySpec {
        read: &[RigidBody::NAME, PLATFORMER],
        write: &[],
        without: &[CHARACTER],
    },
    // Q_JOINTS
    QuerySpec {
        read: &[Joint::NAME],
        write: &[],
        without: &[],
    },
    // Q_VEHICLES: a vehicle is a body.
    QuerySpec {
        read: &[RigidBody::NAME, Vehicle::NAME],
        write: &[],
        without: &[],
    },
    // Q_ROPES
    QuerySpec {
        read: &[TRANSFORM, Rope::NAME],
        write: &[],
        without: &[],
    },
    // Q_CLOTHS
    QuerySpec {
        read: &[TRANSFORM, Cloth::NAME],
        write: &[],
        without: &[],
    },
    // Q_FLUIDS
    QuerySpec {
        read: &[TRANSFORM, Fluid2D::NAME],
        write: &[],
        without: &[],
    },
    // Q_SURFACES
    QuerySpec {
        read: &[TRANSFORM, WaterSurface::NAME],
        write: &[],
        without: &[],
    },
    // Q_LINES
    QuerySpec {
        read: &[TRANSFORM, WaterLine::NAME],
        write: &[],
        without: &[],
    },
    // Q_RAGDOLLS
    QuerySpec {
        read: &[TRANSFORM, Ragdoll::NAME],
        write: &[],
        without: &[],
    },
    // Q_FLUIDS3
    QuerySpec {
        read: &[TRANSFORM, Fluid3D::NAME],
        write: &[],
        without: &[],
    },
];

/// The components a step writes back besides transforms and velocities
/// (controller, joint, vehicle and ragdoll outputs), for the system's
/// declaration.
pub(crate) const WRITTEN: [&str; 5] = [
    CHARACTER,
    PLATFORMER,
    Joint::NAME,
    Vehicle::NAME,
    Ragdoll::NAME,
];

/// Every component of physics beyond rigid bodies, which its systems read.
pub(crate) const READ: [&str; 11] = [
    CHARACTER,
    PLATFORMER,
    Joint::NAME,
    Vehicle::NAME,
    Rope::NAME,
    Cloth::NAME,
    Fluid2D::NAME,
    WaterSurface::NAME,
    WaterLine::NAME,
    Ragdoll::NAME,
    Fluid3D::NAME,
];

/// The fields and layout of one reflected component, for decoding rows.
pub(crate) struct Rows {
    fields: Vec<FieldRef>,
    layout: Layout,
}

impl Rows {
    fn of<T: Reflect>(world: &World) -> Result<Rows> {
        let (fields, layout) = fields_of::<T>(world)?;
        Ok(Rows { fields, layout })
    }

    fn read<T: Reflect + Send>(&self, a: &ArchetypeView<'_>) -> Result<Vec<T>> {
        read_rows(a, &self.fields, &self.layout)
    }
}

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
    character: Rows,
    platformer: Rows,
    joint: Rows,
    vehicle: Rows,
    rope: Rows,
    cloth: Rows,
    fluid: Rows,
    surface: Rows,
    line: Rows,
    ragdoll: Rows,
    fluid3: Rows,
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
        world.register::<CharacterController>()?;
        world.register::<PlatformerController>()?;
        world.register::<Joint>()?;
        world.register::<Vehicle>()?;
        world.register::<Rope>()?;
        world.register::<Cloth>()?;
        world.register::<Fluid2D>()?;
        world.register::<WaterSurface>()?;
        world.register::<WaterLine>()?;
        world.register::<Ragdoll>()?;
        world.register::<crate::CharacterEvent>()?;
        world.register::<crate::Splash>()?;
        world.register::<Fluid3D>()?;
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
            character: Rows::of::<CharacterController>(world)?,
            platformer: Rows::of::<PlatformerController>(world)?,
            joint: Rows::of::<Joint>(world)?,
            vehicle: Rows::of::<Vehicle>(world)?,
            rope: Rows::of::<Rope>(world)?,
            cloth: Rows::of::<Cloth>(world)?,
            fluid: Rows::of::<Fluid2D>(world)?,
            surface: Rows::of::<WaterSurface>(world)?,
            line: Rows::of::<WaterLine>(world)?,
            ragdoll: Rows::of::<Ragdoll>(world)?,
            fluid3: Rows::of::<Fluid3D>(world)?,
        })
    }
}

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
}

/// A collider without a body: static at its transform.
pub(crate) struct Static {
    pub entity: Entity,
    pub position: Vec3,
    pub quaternion: Quat,
    pub scale: Vec3,
    pub collider: Collider,
}

/// The scene: bodies (RigidBody entities in query order, then the
/// controllers without one), static colliders, and the components physics
/// steps beyond rigid bodies, each in query order.
#[derive(Default)]
pub(crate) struct Scene {
    pub bodies: Vec<Body>,
    pub statics: Vec<Static>,
    pub joints: Vec<(Entity, Joint)>,
    pub vehicles: Vec<(Entity, Vehicle)>,
    pub ropes: Vec<(Entity, Vec3, Rope)>,
    pub cloths: Vec<(Entity, Vec3, Quat, Cloth)>,
    pub fluids: Vec<(Entity, Vec3, Fluid2D)>,
    pub surfaces: Vec<(Entity, Vec3, WaterSurface)>,
    pub lines: Vec<(Entity, Vec3, WaterLine)>,
    pub ragdolls: Vec<(Entity, Vec3, Quat, Ragdoll)>,
    pub fluids3: Vec<(Entity, Vec3, Fluid3D)>,
}

/// Every row of component `T` in `a`, decoded from its columns, by ranges
/// of rows on the lanes from two grains of them (`Grains::bodies`), the
/// ranges' rows joined in row order.
fn read_rows<T: Reflect + Send>(
    a: &ArchetypeView<'_>,
    fields: &[FieldRef],
    layout: &Layout,
) -> Result<Vec<T>> {
    let columns = fields
        .iter()
        .map(|f| a.read_bytes(*f))
        .collect::<Result<Vec<_>>>()?;
    let columns: Vec<&[u8]> = columns.iter().map(|c| &**c).collect();
    let decode = |rows: std::ops::Range<usize>| -> Result<Vec<T>> {
        let mut packed = vec![0u8; layout.size()];
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            for (i, column) in columns.iter().enumerate() {
                let range = layout.range(i);
                let n = range.len();
                packed[range].copy_from_slice(&column[row * n..(row + 1) * n]);
            }
            out.push(layout.decode::<T>(&packed)?);
        }
        Ok(out)
    };
    let mut parts = crate::lanes::map_ranges(a.len(), crate::lanes::now().bodies, decode);
    if parts.len() == 1 {
        return parts.pop().unwrap_or_else(|| Ok(Vec::new()));
    }
    let mut out = Vec::with_capacity(a.len());
    for part in parts {
        out.extend(part?);
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
    for a in q(Q_BODIES)? {
        scene.bodies.reserve(a.len());
        let position = a.read::<Vec3>(ids.t_position)?;
        let quaternion = a.read::<Quat>(ids.t_quaternion)?;
        let scale = a.read::<Vec3>(ids.t_scale)?;
        let bodies: Vec<RigidBody> = read_rows(&a, &ids.body_fields, &ids.body_layout)?;
        for (row, (e, body)) in a.entities().iter().zip(bodies).enumerate() {
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
            });
        }
    }
    for (index, character) in [(Q_CHARACTERS, true), (Q_PLATFORMERS, false)] {
        for a in q(index)? {
            let position = a.read::<Vec3>(ids.t_position)?;
            let quaternion = a.read::<Quat>(ids.t_quaternion)?;
            let scale = a.read::<Vec3>(ids.t_scale)?;
            let colliders: Vec<Collider> =
                read_rows(&a, &ids.collider_fields, &ids.collider_layout)?;
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
                });
            }
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
    macro_rules! placed {
        ($index:expr, $rows:expr, $list:expr, $make:expr) => {
            for a in q($index)? {
                let position = a.read::<Vec3>(ids.t_position)?;
                let quaternion = a.read::<Quat>(ids.t_quaternion)?;
                let rows = $rows.read(&a)?;
                for (row, (e, c)) in a.entities().iter().zip(rows).enumerate() {
                    $list.push($make(*e, position[row], quaternion[row], c));
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
