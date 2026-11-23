//! og-play over the runtime's engines (`docs/design.md` §2.1, mechanisms
//! that span domains): the poses of og.HitVolume and og.HurtVolume from
//! the world's simulation matrices (og-anim's posed bones through
//! og.BoneAttachment), the colliders from og-physics' view of the tick,
//! and og-play's answer written back as og.HitEvent and the volumes' last
//! poses. What each owner's swing has struck is the `og.play` state block.
//! The records are lazy (registered together): until a game names one, the
//! world has no og.play system and no block.

use crate::engine::{Mount, engine, running};
use crate::host::codec::{Layout, RecordKind};
use crate::host::wire::{self, record};
use glam::{Mat4, Quat, Vec3};
use og_ecs::{Entity, QueryDesc, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use og_physics::{
    OverlapDesc, PhysicsQuery, ProjectPointDesc, QueryFilter, RaycastDesc, ShapeDesc,
};
use og_play::candidates::{Around, candidates};
use og_play::contact::Contact;
use og_play::hits::{Colliders, Hurt, Swings, Volume};
use og_play::shape::{Pose, Shape};
use serde_json::{Value, json};
use std::any::Any;
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) const HIT: &str = "og.HitVolume";
pub(crate) const HURT: &str = "og.HurtVolume";
pub(crate) const EVENT: &str = "og.HitEvent";
/// Registered together (the registry's TOGETHER).
pub(crate) const RECORDS: [&str; 3] = [HIT, HURT, EVENT];
/// The system's and the state block's name.
pub(crate) const SYSTEM: &str = "og.play";

type Shared = Arc<Mutex<Swings>>;

fn lock(s: &Shared) -> MutexGuard<'_, Swings> {
    s.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn invalid(e: impl Into<String>) -> og_core::Error {
    og_core::Error::new(og_core::ErrorKind::Invalid, e.into())
}

struct Layouts {
    hit: (SchemaId, Layout),
    hurt: (SchemaId, Layout),
    event: (SchemaId, Layout),
}

/// Adds og.play's system, before og.physics (after og-anim has posed the
/// tick's bones), and its state block, once all three records are
/// registered with `world`.
pub(crate) fn enable(world: &mut World, mount: Mount<'_>) -> og_core::Result<()> {
    if world.systems().contains(&SYSTEM) || RECORDS.iter().any(|n| world.id_by_name(n).is_err()) {
        return Ok(());
    }
    let layout = |n: &str, kind: RecordKind| -> og_core::Result<(SchemaId, Layout)> {
        let id = world.id_by_name(n)?;
        // The IDL's layout, with its defaults (a schema read back has none).
        let layout = og_api::host::RECORDS
            .iter()
            .find(|d| d.schema == n)
            .and_then(|d| Layout::from_decl(d).ok());
        let layout = match layout {
            Some(l) => l,
            None => Layout::from_schema(world.schema(id)?, kind),
        };
        Ok((id, layout))
    };
    let ls = Layouts {
        hit: layout(HIT, RecordKind::Component)?,
        hurt: layout(HURT, RecordKind::Component)?,
        event: layout(EVENT, RecordKind::Event)?,
    };
    let swings = Shared::default();
    let state = swings.clone();
    let engine_id = mount.id;
    world.add_system(
        SystemDesc {
            name: SYSTEM.into(),
            reads: vec![HURT.into()],
            writes: vec![HIT.into(), EVENT.into()],
            ..Default::default()
        },
        move |ctx| run(ctx, engine_id, &ls, &state),
    )?;
    if world.systems().contains(&"og.physics") {
        world.move_system_before(SYSTEM, "og.physics")?;
    }
    world.add_state_block(SYSTEM, Box::new(Block(swings)))
}

/// Where a volume's shape is: its entity's simulation matrix (posed parent
/// frames included) times its offset and rotation, and its shape scaled
/// by the matrix's scale.
fn place(
    matrix: Mat4,
    shape: wire::VolumeShape,
    (size, radius, height): (Vec3, f32, f32),
    (offset, quaternion): (Vec3, Quat),
) -> (Shape, Pose) {
    let (scale, rotation, _) = matrix.to_scale_rotation_translation();
    let scale = scale.abs();
    let q = if quaternion == Quat::from_xyzw(0.0, 0.0, 0.0, 0.0) {
        Quat::IDENTITY
    } else {
        quaternion.normalize()
    };
    let shape = match shape {
        wire::VolumeShape::Sphere => Shape::Sphere {
            radius: radius * scale.max_element(),
        },
        wire::VolumeShape::Capsule => Shape::Capsule {
            radius: radius * scale.x.max(scale.z),
            height: height * scale.y,
        },
        wire::VolumeShape::Box => Shape::Box {
            half: size * scale * 0.5,
        },
    };
    let pose = Pose::new(matrix.transform_point3(offset), (rotation * q).normalize());
    (shape, pose)
}

/// The rows of a component, in entity order, with their bytes.
fn rows(
    ctx: &SystemContext<'_>,
    (id, name): (SchemaId, &str),
) -> og_core::Result<Vec<(Entity, Vec<u8>)>> {
    let q = QueryDesc {
        read: vec![name.to_string()],
        ..QueryDesc::default()
    };
    let mut entities: Vec<Entity> = ctx
        .query_with(&q)?
        .flat_map(|a| a.entities().to_vec())
        .collect();
    og_core::sort::sort_by_key(&mut entities, |e| e.index());
    entities
        .into_iter()
        .map(|e| Ok((e, ctx.get_bytes(e, id)?)))
        .collect()
}

/// The colliders as og-physics saw them at the start of the tick.
struct Physics(Option<PhysicsQuery>);

impl Colliders for Physics {
    fn overlap(
        &self,
        shape: &Shape,
        pose: &Pose,
        layers: u32,
        exclude: u64,
    ) -> Vec<(u64, Contact)> {
        let Some(q) = &self.0 else {
            return Vec::new();
        };
        let filter = QueryFilter {
            mask: layers,
            include_sensors: false,
            exclude: (exclude != 0).then(|| Entity::from_bits(exclude)),
        };
        let desc = match *shape {
            Shape::Sphere { radius } => ShapeDesc {
                shape: og_physics::SHAPE_SPHERE,
                radius,
                ..ShapeDesc::default()
            },
            Shape::Capsule { radius, height } => ShapeDesc {
                shape: og_physics::SHAPE_CAPSULE,
                radius,
                height,
                ..ShapeDesc::default()
            },
            Shape::Box { half } => ShapeDesc {
                shape: og_physics::SHAPE_BOX,
                size: half * 2.0,
                ..ShapeDesc::default()
            },
        };
        let Ok(found) = q.overlap(&OverlapDesc {
            shape: desc,
            position: pose.position,
            rotation: pose.rotation,
            filter,
        }) else {
            return Vec::new();
        };
        // Where: the nearest surface point to the volume's centre, when it
        // is this collider's; else the centre itself.
        let near = q
            .project_point(&ProjectPointDesc {
                point: pose.position,
                max_distance: 0.0,
                solid: false,
                filter,
            })
            .ok()
            .flatten();
        found
            .into_iter()
            .map(|e| {
                let contact = match near {
                    Some(p) if p.entity == e && p.normal != Vec3::ZERO => Contact {
                        point: p.point,
                        normal: p.normal,
                    },
                    _ => Contact {
                        point: pose.position,
                        normal: Vec3::ZERO,
                    },
                };
                (e.to_bits(), contact)
            })
            .collect()
    }
}

fn vec3(v: Vec3) -> Value {
    json!({ "x": v.x, "y": v.y, "z": v.z })
}

fn run(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    ls: &Layouts,
    swings: &Shared,
) -> og_core::Result<()> {
    let hit_rows = rows(ctx, (ls.hit.0, HIT))?;
    if hit_rows.is_empty() && *lock(swings) == Swings::default() {
        return Ok(());
    }
    let mut volumes = Vec::with_capacity(hit_rows.len());
    let mut wanted_colliders = false;
    for (e, bytes) in &hit_rows {
        let v: record::HitVolume = ls.hit.1.read(bytes).map_err(invalid)?;
        let (shape, to) = place(
            ctx.matrix_world(*e)?,
            v.shape,
            (v.size, v.radius, v.height),
            (v.offset, v.quaternion),
        );
        wanted_colliders |= v.active && v.colliders != 0;
        volumes.push((
            Volume {
                entity: e.to_bits(),
                owner: if v.owner == 0 { e.to_bits() } else { v.owner },
                team: v.team,
                swing: v.swing,
                active: v.active,
                shape,
                from: (v.active && v.was_active)
                    .then(|| Pose::new(v.last_position, v.last_quaternion)),
                to,
                colliders: v.colliders,
            },
            v,
        ));
    }
    let mut hurts = Vec::new();
    for (e, bytes) in rows(ctx, (ls.hurt.0, HURT))? {
        let h: record::HurtVolume = ls.hurt.1.read(&bytes).map_err(invalid)?;
        let (shape, pose) = place(
            ctx.matrix_world(e)?,
            h.shape,
            (h.size, h.radius, h.height),
            (h.offset, h.quaternion),
        );
        hurts.push(Hurt {
            entity: e.to_bits(),
            owner: if h.owner == 0 { e.to_bits() } else { h.owner },
            team: h.team,
            solid: shape.place(&pose),
        });
    }
    let physics = Physics(if wanted_colliders {
        engine(engine_id)
            .ok()
            .and_then(|eng| eng.state.borrow().physics.clone())
            .and_then(|p| p.tick_query().ok())
    } else {
        None
    });
    let plain: Vec<Volume> = volumes.iter().map(|(v, _)| *v).collect();
    let found = lock(swings).step(&plain, &hurts, &physics);
    // The poses this tick leaves, for the next tick's sweep.
    for ((e, bytes), (v, r)) in hit_rows.iter().zip(&volumes) {
        if r.last_position == v.to.position
            && r.last_quaternion == v.to.rotation
            && r.was_active == v.active
        {
            continue;
        }
        let mut s = ls.hit.1.decode(bytes);
        s["lastPosition"] = vec3(v.to.position);
        s["lastQuaternion"] = json!({ "x": v.to.rotation.x, "y": v.to.rotation.y, "z": v.to.rotation.z, "w": v.to.rotation.w });
        s["wasActive"] = json!(v.active);
        let out = ls.hit.1.encode(Some(bytes), Some(&s)).map_err(invalid)?;
        ctx.set_bytes(*e, ls.hit.0, &out)?;
    }
    for h in found {
        let ev = json!({
            "volume": h.volume,
            "owner": h.owner,
            "swing": h.swing,
            "target": h.target,
            "hurt": h.hurt,
            "point": vec3(h.point),
            "normal": vec3(h.normal),
        });
        let bytes = ls.event.1.encode(None, Some(&ev)).map_err(invalid)?;
        ctx.send_bytes(ls.event.0, &bytes)?;
    }
    Ok(())
}

/// The `og.play` state block: what each owner's swing has struck.
struct Block(Shared);

impl StateBlock for Block {
    fn save(&self, out: &mut Vec<u8>) {
        out.extend(lock(&self.0).to_bytes());
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        Swings::from_bytes(bytes)
            .map(|s| Box::new(s) as Box<dyn Any + Send>)
            .ok_or_else(|| invalid(format!("{SYSTEM}: bytes it did not save")))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        *lock(&self.0) = decoded
            .and_then(|d| d.downcast::<Swings>().ok())
            .map(|s| *s)
            .unwrap_or_default();
    }

    /// Per owner one `swing` record keyed by its swing number, the targets
    /// struck as entities. Read after `Swings::from_bytes` accepts the
    /// bytes, in its layout (og-play keeps the map private).
    fn form(&self, bytes: &[u8]) -> og_core::Result<og_core::block_form::BlockForm> {
        use og_core::block_form::{BlockForm, Field, Record, Records};
        self.decode(bytes)?;
        let mut r = og_core::bytes::ByteReader::new(bytes, SYSTEM);
        let mut out = Records::default();
        for _ in 0..r.u32()? {
            let owner = r.u64()?;
            let swing = r.u32()?;
            let targets = (0..r.u32()?)
                .map(|_| r.u64())
                .collect::<og_core::Result<Vec<_>>>()?;
            out.entities.push(
                Record::new(owner, "swing")
                    .key(Field::of(&swing))
                    .field("struck", Field::Entities(targets)),
            );
        }
        Ok(BlockForm::Entities(out))
    }

    /// Entities and swings only: nothing moves with the world's zero.
    fn shift(&mut self, _: &og_ecs::Shift) {}
}

/// What og.play.candidates reads: the world (between ticks) or the
/// running system's context.
enum At<'a> {
    World(&'a World),
    System(&'a SystemContext<'a>),
}

impl At<'_> {
    fn entities(&self, with: &str) -> Result<Vec<Entity>, String> {
        let q = QueryDesc {
            read: vec![with.to_string()],
            ..QueryDesc::default()
        };
        let all: Vec<Entity> = match self {
            At::World(w) => w
                .query(&q)
                .map_err(|e| e.to_string())?
                .flat_map(|a| a.entities().to_vec())
                .collect(),
            At::System(c) => c
                .query_with(&q)
                .map_err(|e| e.to_string())?
                .flat_map(|a| a.entities().to_vec())
                .collect(),
        };
        Ok(all)
    }

    fn position(&self, e: Entity) -> Result<Vec3, String> {
        let m = match self {
            At::World(w) => w.matrix_world(e),
            At::System(c) => c.matrix_world(e),
        };
        Ok(m.map_err(|e| e.to_string())?.w_axis.truncate())
    }
}

fn finite(v: Vec3, what: &str) -> Result<Vec3, String> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(format!("{what} is not finite"))
    }
}

/// `og.play.candidates`.
pub fn og_play_candidates(
    world: u32,
    desc: Option<String>,
) -> Result<Vec<wire::Candidate>, String> {
    let d: wire::CandidatesDesc = wire::arg(desc.as_deref(), "desc")?;
    let from = finite(d.from, "desc.from")?;
    let forward = finite(d.forward, "desc.forward")?;
    if forward == Vec3::ZERO {
        return Err("desc.forward is zero".into());
    }
    if !d.range.is_finite() || !d.cone.is_finite() {
        return Err("desc.range and desc.cone must be finite".into());
    }
    let eng = engine(world)?;
    // SAFETY: the frame is published while its system runs.
    let at = match running(world) {
        Some(f) => At::System(unsafe { f.ctx() }),
        None => At::World(eng.world()?),
    };
    let points = at
        .entities(&d.with)?
        .into_iter()
        .map(|e| Ok((e.to_bits(), at.position(e)?)))
        .collect::<Result<Vec<_>, String>>()?;
    let around = Around {
        from,
        forward: forward.normalize(),
        range: d.range,
        cone: d.cone,
    };
    let query = if d.sight {
        let physics = eng
            .state
            .borrow()
            .physics
            .clone()
            .ok_or("this engine has no physics")?;
        Some(
            match &at {
                At::System(_) => physics.tick_query(),
                At::World(w) => physics.query(w),
            }
            .map_err(|e| e.reason().to_string())?,
        )
    } else {
        None
    };
    let filter = QueryFilter {
        mask: d.filter.mask,
        include_sensors: d.filter.include_sensors,
        exclude: match d.filter.exclude {
            Some(0) => return Err("desc.filter.exclude: 0 is not an entity".into()),
            e => e.map(Entity::from_bits),
        },
    };
    let sight = |entity: u64, to: Vec3| -> bool {
        let Some(q) = &query else { return true };
        let dir = to - from;
        let distance = dir.length();
        if distance <= 0.0 {
            return true;
        }
        match q.raycast(&RaycastDesc {
            origin: from,
            direction: dir / distance,
            max_distance: distance,
            filter,
        }) {
            Ok(Some(hit)) => {
                hit.entity.to_bits() == entity
                    || hit.body.to_bits() == entity
                    || hit.distance >= distance - 1e-4
            }
            _ => true,
        }
    };
    let sight: Option<&dyn Fn(u64, Vec3) -> bool> = if d.sight { Some(&sight) } else { None };
    Ok(candidates(&around, &points, sight)
        .into_iter()
        .map(|c| wire::Candidate {
            entity: c.entity,
            distance: c.distance,
            angle: c.angle,
            visible: c.visible,
        })
        .collect())
}
