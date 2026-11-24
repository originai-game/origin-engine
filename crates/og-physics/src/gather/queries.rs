//! The queries physics makes of the world, in the order the step reads
//! them, and the ids and layouts of the types it reads and writes.

use super::*;

/// The engine's transform component, which physics reads and writes by name
/// (it registers the schema, never a Rust type, so the engine's own Rust
/// `og.Transform` type, wherever it lives, is adopted unchanged).
pub(crate) const TRANSFORM: &str = Transform::NAME;

pub(crate) struct QuerySpec {
    pub(super) read: &'static [&'static str],
    pub(super) write: &'static [&'static str],
    pub(super) without: &'static [&'static str],
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
                ..Default::default()
            }
        } else {
            read.extend(write);
            QueryDesc {
                read,
                write: Vec::new(),
                without: names(self.without),
                ..Default::default()
            }
        }
    }
}

pub(super) const Q_BODIES: usize = 0;
pub(super) const Q_BODY_COLLIDERS: usize = 1;
pub(super) const Q_BODY_VELOCITIES: usize = 2;
pub(super) const Q_STATIC_COLLIDERS: usize = 3;
pub(super) const Q_CHARACTERS: usize = 4;
pub(super) const Q_PLATFORMERS: usize = 5;
pub(super) const Q_BODY_CHARACTERS: usize = 6;
pub(super) const Q_BODY_PLATFORMERS: usize = 7;
pub(super) const Q_JOINTS: usize = 8;
pub(super) const Q_VEHICLES: usize = 9;
pub(super) const Q_ROPES: usize = 10;
pub(super) const Q_CLOTHS: usize = 11;
pub(super) const Q_FLUIDS: usize = 12;
pub(super) const Q_SURFACES: usize = 13;
pub(super) const Q_LINES: usize = 14;
pub(super) const Q_RAGDOLLS: usize = 15;
pub(super) const Q_FLUIDS3: usize = 16;
pub(super) const Q_PARENTS: usize = 17;

pub(super) const CHARACTER: &str = CharacterController::NAME;
pub(super) const PLATFORMER: &str = PlatformerController::NAME;

/// The system's queries, by index.
pub(crate) const QUERIES: [QuerySpec; 18] = [
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
    // Q_PARENTS: the parent chains soft bodies on child entities are placed
    // through (#10 of kestrel-stair's findings).
    QuerySpec {
        read: &[Parent::NAME],
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
pub(crate) const READ: [&str; 12] = [
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
    Parent::NAME,
];

/// The fields and layout of one reflected component, for decoding rows.
pub(crate) struct Rows {
    pub(super) fields: Vec<FieldRef>,
    pub(super) layout: Layout,
}

impl Rows {
    pub(super) fn of<T: Reflect>(world: &World) -> Result<Rows> {
        let (fields, layout) = fields_of::<T>(world)?;
        Ok(Rows { fields, layout })
    }

    pub(super) fn read<T: Reflect + Send>(&self, a: &ArchetypeView<'_>) -> Result<Vec<T>> {
        read_rows(a, &self.fields, &self.layout)
    }
}

pub(crate) struct Ids {
    pub(super) transform: SchemaId,
    pub(super) transform_layout: Layout,
    pub(super) parent_id: SchemaId,
    pub(super) parent_layout: Layout,
    pub(super) parent: FieldRef,
    pub(super) t_position: FieldRef,
    pub(super) t_quaternion: FieldRef,
    pub(super) t_scale: FieldRef,
    pub(super) body: SchemaId,
    pub(super) body_fields: Vec<FieldRef>,
    pub(super) body_layout: Layout,
    pub(super) collider: SchemaId,
    pub(super) collider_fields: Vec<FieldRef>,
    pub(super) collider_layout: Layout,
    pub(super) v_linear: FieldRef,
    pub(super) v_angular: FieldRef,
    pub(super) character: Rows,
    pub(super) platformer: Rows,
    pub(super) joint: Rows,
    pub(super) vehicle: Rows,
    pub(super) rope: Rows,
    pub(super) cloth: Rows,
    pub(super) fluid: Rows,
    pub(super) surface: Rows,
    pub(super) line: Rows,
    pub(super) ragdoll: Rows,
    pub(super) fluid3: Rows,
    pub(super) hook_layout: Layout,
    pub(super) wheel_layout: Layout,
    pub(super) soft_layout: Layout,
    /// What the gathers of the tick keep from one to the next.
    pub(super) kept: Mutex<Kept>,
}

pub(super) fn fields_of<T: Reflect>(world: &World) -> Result<(Vec<FieldRef>, Layout)> {
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
        let body = world.register::<RigidBody>()?;
        let collider = world.register::<Collider>()?;
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
        let parent_id = world.register::<Parent>()?;
        let (body_fields, body_layout) = fields_of::<RigidBody>(world)?;
        let (collider_fields, collider_layout) = fields_of::<Collider>(world)?;
        Ok(Ids {
            transform,
            transform_layout: Layout::of(&Transform::schema()),
            parent_id,
            parent_layout: Layout::of(&Parent::schema()),
            parent: world.field::<Parent>("entity")?,
            t_position: world.field_by_id(transform, "position")?,
            t_quaternion: world.field_by_id(transform, "quaternion")?,
            t_scale: world.field_by_id(transform, "scale")?,
            body,
            body_fields,
            body_layout,
            collider,
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
            hook_layout: Layout::of(&ContactHook::schema()),
            wheel_layout: Layout::of(&VehicleWheel::schema()),
            soft_layout: Layout::of(&SoftBody::schema()),
            kept: Mutex::default(),
        })
    }

    /// The og.Collider rows of `a`, archetype `at` of its query ([`Kept::rows`]).
    pub(super) fn colliders<'k>(
        &self,
        kept: &'k mut Vec<KeptRows<Collider>>,
        at: usize,
        a: &ArchetypeView<'_>,
    ) -> Result<&'k [Collider]> {
        Kept::rows(
            kept,
            at,
            a,
            self.collider,
            &self.collider_fields,
            &self.collider_layout,
        )
    }

    /// What a gather inside a system reuses and renews ([`Kept`]).
    pub fn kept(&self) -> MutexGuard<'_, Kept> {
        // A gather that panicked left its slot unstamped (`Kept::rows`).
        self.kept.lock().unwrap_or_else(|e| e.into_inner())
    }
}
