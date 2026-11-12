//! The physics beyond rigid bodies seen from game code (the E5 round):
//! the pushes (`og.physics.applyForce`, `applyImpulse`, `applyTorque`,
//! `applyTorqueImpulse`), the readouts of the last step (`og.physics.ropePoints`, `clothPoints`,
//! `fluidParticles`, `waterHeight`, `vehicleWheels`, `ragdollPose`), what
//! the last step cost (`stats`), the ragdoll rig (`setRagdollRig`,
//! `ragdollRigFromSkeleton`), and drawing a
//! ragdoll: a model posed by og-anim whose entity has a rig made from its
//! skeleton is drawn with its joints blended towards the bones' bodies by
//! og.Ragdoll.blend. Rigs are assets, like meshes: kept by the engine and
//! given again to a reloaded world's physics.

use super::engine::{Engine, engine, running};
use super::extract::RenderState;
use super::fx::rows;
use super::wire::{
    self, BonePose, FromJson, PhysicsPhase, PhysicsStats, RagdollBoneDesc, RagdollRigDesc,
    SoftBodyPoints, ToJson, WheelPose, record,
};
use glam::{Mat4, Quat, Vec3};
use og_ecs::{Entity, World};
use og_physics::{PhysicsSystem, RagdollBone, RagdollRig};
use serde_json::{Value, json};

fn physics(eng: &Engine) -> Result<PhysicsSystem, String> {
    eng.state
        .borrow()
        .physics
        .clone()
        .ok_or_else(|| "this engine has no physics".to_string())
}

fn entity(e: f64) -> Result<Entity, String> {
    if e.fract() != 0.0 || !(0.0..9_007_199_254_740_992.0).contains(&e) {
        return Err(format!("{e} is not an entity"));
    }
    Ok(Entity::from_bits(e as u64))
}

/// A readout of the last step for `e`, refused with `what` when it has none.
fn readout<T, W: ToJson>(
    world: u32,
    e: f64,
    what: &str,
    read: impl FnOnce(&PhysicsSystem, Entity) -> Option<T>,
    wire: impl FnOnce(T) -> W,
) -> Result<W, String> {
    (|| {
        let eng = engine(world)?;
        let p = physics(&eng)?;
        let e = entity(e)?;
        read(&p, e).map(wire).ok_or_else(|| {
            format!(
                "entity {} has no {what} (or it has not stepped yet)",
                e.to_bits()
            )
        })
    })()
}

/// A push for `e`'s body, refused for an entity without an og.RigidBody
/// (inside a system as outside: archetype membership needs no declaration).
fn push(
    world: u32,
    e: f64,
    (name, value): (&str, &str),
    point: Option<&str>,
    apply: impl FnOnce(&PhysicsSystem, Entity, Vec3, Option<Vec3>) -> og_core::Result<()>,
) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let p = physics(&eng)?;
        let e = entity(e)?;
        let has_body = if let Some(f) = running(world) {
            // SAFETY: the frame is published while its system runs.
            let ctx = unsafe { f.ctx() };
            let id = ctx.id_by_name("og.RigidBody").map_err(|e| e.to_string())?;
            ctx.components_of(e).is_ok_and(|c| c.contains(&id))
        } else {
            let w = eng.world()?;
            let id = w.id_by_name("og.RigidBody").map_err(|e| e.to_string())?;
            w.components_of(e).is_ok_and(|c| c.contains(&id))
        };
        if !has_body {
            return Err(format!(
                "entity {} has no og.RigidBody to push",
                e.to_bits()
            ));
        }
        let vector: Vec3 = wire::arg(Some(value), name)?;
        let point = point
            .map(|p| wire::arg::<Vec3>(Some(p), "point"))
            .transpose()?;
        apply(&p, e, vector, point).map_err(|e| e.to_string())
    })()
}

pub fn og_physics_apply_force(
    world: u32,
    e: f64,
    force: String,
    point: Option<String>,
) -> Result<(), String> {
    push(
        world,
        e,
        ("force", &force),
        point.as_deref(),
        |p, e, v, at| p.apply_force(e, v, at),
    )
}

pub fn og_physics_apply_impulse(
    world: u32,
    e: f64,
    impulse: String,
    point: Option<String>,
) -> Result<(), String> {
    push(
        world,
        e,
        ("impulse", &impulse),
        point.as_deref(),
        |p, e, v, at| p.apply_impulse(e, v, at),
    )
}

pub fn og_physics_apply_torque(world: u32, e: f64, torque: String) -> Result<(), String> {
    push(world, e, ("torque", &torque), None, |p, e, v, _| {
        p.apply_torque(e, v)
    })
}

pub fn og_physics_apply_torque_impulse(world: u32, e: f64, torque: String) -> Result<(), String> {
    push(world, e, ("torque", &torque), None, |p, e, v, _| {
        p.apply_torque_impulse(e, v)
    })
}

pub fn og_physics_rope_points(world: u32, e: f64) -> Result<Vec<glam::Vec3>, String> {
    readout(world, e, "og.Rope", |p, e| p.rope_points(e), |pts| pts)
}

pub fn og_physics_cloth_points(world: u32, e: f64) -> Result<Vec<glam::Vec3>, String> {
    readout(world, e, "og.Cloth", |p, e| p.cloth_points(e), |c| c.points)
}

pub fn og_physics_garment_points(world: u32, e: f64) -> Result<Vec<glam::Vec3>, String> {
    readout(
        world,
        e,
        "og.Garment",
        |p, e| p.garment_points(e),
        |g| g.points,
    )
}

pub fn og_physics_soft_body_points(
    world: u32,
    e: f64,
) -> Result<crate::wire::SoftBodyPoints, String> {
    readout(
        world,
        e,
        "og.SoftBody",
        |p, e| p.soft_body_points(e),
        |s| SoftBodyPoints {
            points: s.points,
            indices: s.indices,
        },
    )
}

pub fn og_physics_fluid_particles(world: u32, e: f64) -> Result<Vec<glam::Vec2>, String> {
    readout(
        world,
        e,
        "og.Fluid2D",
        |p, e| p.fluid_particles(e),
        |pts| pts,
    )
}

pub fn og_physics_fluid3d_particles(world: u32, e: f64) -> Result<Vec<glam::Vec3>, String> {
    readout(
        world,
        e,
        "og.Fluid3D",
        |p, e| p.fluid3d_particles(e),
        |pts| pts,
    )
}

pub fn og_physics_vehicle_wheels(
    world: u32,
    e: f64,
) -> Result<Vec<crate::wire::WheelPose>, String> {
    readout(
        world,
        e,
        "og.Vehicle",
        |p, e| p.vehicle_wheels(e),
        |wheels| {
            wheels
                .into_iter()
                .map(|w| WheelPose {
                    position: w.position,
                    rotation: w.quaternion,
                    contact: w.contact,
                    slip: w.slip,
                    side_speed: w.side_speed,
                    entity: w.entity,
                })
                .collect::<Vec<_>>()
        },
    )
}

pub fn og_physics_ragdoll_pose(world: u32, e: f64) -> Result<Vec<crate::wire::BonePose>, String> {
    readout(
        world,
        e,
        "og.Ragdoll",
        |p, e| p.ragdoll_pose(e),
        |bones| {
            bones
                .into_iter()
                .map(|(position, rotation)| BonePose { position, rotation })
                .collect::<Vec<_>>()
        },
    )
}

/// `og.physics.stats`: the last step's phases, their times and whether
/// each ran on the lanes pool.
pub fn og_physics_stats(world: u32) -> Result<PhysicsStats, String> {
    (|| {
        let eng = engine(world)?;
        let stats = physics(&eng)?.stats();
        let phases: Vec<PhysicsPhase> = stats
            .phases
            .iter()
            .map(|p| PhysicsPhase {
                name: p.name.into(),
                ms: p.ms,
                split: p.split,
            })
            .collect();
        // Before the first step, every phase at zero.
        let phases = if phases.is_empty() {
            og_physics::Phase::ALL
                .iter()
                .map(|p| PhysicsPhase {
                    name: p.name().into(),
                    ms: 0.0,
                    split: false,
                })
                .collect()
        } else {
            phases
        };
        Ok(PhysicsStats {
            tick: stats.tick as f64,
            bodies: stats.bodies,
            threads: stats.threads,
            pool: stats.pool,
            ms: stats.ms,
            phases,
        })
    })()
}

/// What og.physics.waterHeight and the bodies in water read of the world: the
/// world itself (between ticks, and as a tick starts), or, inside a system
/// (a boat a game floats in its own tick), its context, through the
/// system's declared reads: og.Water and og.Transform (og.Parent for a
/// water under a parent; og.WorldOrigin when the world has one).
pub(crate) enum Reader<'a> {
    World(&'a World),
    System(&'a og_ecs::SystemContext<'a>),
}

impl<'a> Reader<'a> {
    fn of(eng: &'a Engine, world: u32) -> Result<Reader<'a>, String> {
        Ok(match running(world) {
            // SAFETY: the frame is published while its system runs.
            Some(f) => Reader::System(unsafe { f.ctx() }),
            None => Reader::World(eng.world()?),
        })
    }

    fn get<T: og_core::reflect::Reflect>(&self, e: Entity) -> Option<T> {
        match self {
            Reader::World(w) => w.get::<T>(e).ok(),
            Reader::System(c) => c.get::<T>(e).ok(),
        }
    }

    /// Every row of a component, decoded, in entity order (as
    /// [`super::fx::rows`]); in a system, an error when it is undeclared.
    pub(crate) fn rows(
        &self,
        layout: Option<&super::codec::Layout>,
    ) -> Result<Vec<(u64, Value)>, String> {
        let c = match self {
            Reader::World(w) => return Ok(rows(w, layout)),
            Reader::System(c) => c,
        };
        let Some(layout) = layout else {
            return Ok(Vec::new());
        };
        let Ok(id) = c.id_by_name(&layout.schema) else {
            return Ok(Vec::new());
        };
        let q = og_ecs::QueryDesc {
            read: vec![layout.schema.clone()],
            ..og_ecs::QueryDesc::default()
        };
        let all = c.query_with(&q).map_err(|e| e.to_string())?;
        let mut entities: Vec<Entity> = all.flat_map(|a| a.entities().to_vec()).collect();
        og_core::sort::sort_by_key(&mut entities, |e| e.index());
        Ok(entities
            .into_iter()
            .filter_map(|e| Some((e.to_bits(), layout.decode(&c.get_bytes(e, id).ok()?))))
            .collect())
    }

    /// Where og.WorldOrigin puts the world's zero (zero without one).
    fn zero(&self, layout: Option<&super::codec::Layout>) -> Result<glam::DVec3, String> {
        let c = match self {
            Reader::World(w) => return Ok(crate::origin::zero(w, layout)),
            Reader::System(c) => c,
        };
        let (Some(layout), Ok(id)) = (layout, c.id_by_name(crate::origin::RESOURCE)) else {
            return Ok(glam::DVec3::ZERO);
        };
        if !c.has_resource_id(id).unwrap_or(false) {
            return Ok(glam::DVec3::ZERO);
        }
        let o: record::WorldOrigin =
            layout.read(&c.resource_bytes(id).map_err(|e| e.to_string())?)?;
        Ok(glam::DVec3::new(o.x, o.y, o.z))
    }

    fn time(&self) -> og_core::Time {
        match self {
            Reader::World(w) => w.time(),
            Reader::System(c) => c.time(),
        }
    }

    /// An entity's world position: its og.Transform after its parents' (#10).
    fn position(&self, e: Entity) -> Option<Vec3> {
        use og_physics::{Parent, Transform};
        let t = self.get::<Transform>(e)?;
        let (at, _) = og_physics::world_pose(
            e,
            (t.position, t.quaternion),
            |c| self.get::<Parent>(c).map(|p| Entity::from_bits(p.entity)),
            |p| self.get::<Transform>(p),
        );
        Some(at)
    }

    /// An entity's world position and rotation (its og.Transform after its
    /// parents') and its own scale: what places an og.Water's seabed.
    pub(crate) fn pose(&self, e: Entity) -> Option<(Vec3, Quat, Vec3)> {
        use og_physics::{Parent, Transform};
        let t = self.get::<Transform>(e)?;
        let (at, q) = og_physics::world_pose(
            e,
            (t.position, t.quaternion),
            |c| self.get::<Parent>(c).map(|p| Entity::from_bits(p.entity)),
            |p| self.get::<Transform>(p),
        );
        Some((at, q, t.scale))
    }
}

/// The world's og.Water bodies in entity order (those drawn over a mesh
/// left out: an arbitrary shape, no square of water): each at its world
/// position, its sea (og.Water.wind) and og.WorldOrigin's zero. What
/// bodies cross and float on and og.physics.waterHeight answers with.
pub(crate) fn waters(eng: &Engine, r: &Reader<'_>) -> Result<Vec<og_physics::WaterBody>, String> {
    let (layout, origin, terrain) = {
        let st = eng.state.borrow();
        (
            st.layouts.get("og.Water").cloned(),
            st.layouts.get(crate::origin::RESOURCE).cloned(),
            st.layouts.get("og.Terrain").cloned(),
        )
    };
    let rows = r.rows(layout.as_ref())?;
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let zero = r.zero(origin.as_ref())?;
    Ok(rows
        .into_iter()
        .filter_map(|(id, v)| {
            let w = record::Water::from_json(&v).ok()?;
            if w.geometry != 0 {
                return None;
            }
            let position = r.position(Entity::from_bits(id))?;
            let sea = og_core::ocean::Ocean::new(w.wind, w.swell, w.choppiness);
            Some(og_physics::WaterBody {
                entity: id,
                position,
                size: w.size,
                shore: super::seabed::coast(
                    &eng.render.borrow().terrain,
                    terrain.as_ref(),
                    r,
                    &v,
                    position,
                    sea.as_ref(),
                ),
                sea,
                zero,
                river: super::atmosphere::river_of(eng.id, &v),
            })
        })
        .collect())
}

/// As each tick starts (after og.WorldOrigin's shift): the og.Water bodies
/// this tick's bodies cross, cut and (og.Buoyancy) float on, read only
/// once a game names og.Water.
pub(crate) fn before_tick(eng: &Engine, world: &og_ecs::World) {
    let Ok(p) = physics(eng) else {
        return;
    };
    if world.id_by_name("og.Water").is_ok() {
        p.set_waters(waters(eng, &Reader::World(world)).unwrap_or_default());
    }
    let wind = eng.state.borrow().layouts.get("og.Wind").cloned();
    crate::wind::before_tick(&p, world, wind.as_ref());
}

/// An og.Water's surface (a lake's level, a sea's waves, a river's
/// course; `og_physics::WaterBody::height`, what bodies float on) at
/// world point (`x`, `z`) this tick: its height over the entity's y, 0
/// outside the water; `None` for an entity without an og.Water. A sea's
/// waves are the absolute world's (og.WorldOrigin's zero added), at the
/// tick over its rate, as the renderer draws them.
fn body_height(
    eng: &Engine,
    r: &Reader<'_>,
    e: Entity,
    x: f64,
    z: f64,
) -> Result<Option<f32>, String> {
    let Some(water) = waters(eng, r)?
        .into_iter()
        .find(|b| b.entity == e.to_bits())
    else {
        return Ok(None);
    };
    let time = r.time();
    let rate = if time.dt > 0.0 { 1.0 / time.dt } else { 60.0 };
    let seconds = time.tick as f64 / f64::from(rate);
    Ok(Some(water.height(x, z, seconds).unwrap_or(0.0)))
}

/// `og.physics.waterHeight`: the entity's y, its og.Water's surface over
/// it and the waves bodies raised on its og.WaterSurface or og.WaterLine,
/// each from the one function the simulation floats bodies on.
pub fn og_physics_water_height(
    world: u32,
    e: f64,
    x: f64,
    z: f64,
) -> Result<serde_json::Value, String> {
    (|| {
        let eng = engine(world)?;
        let e = entity(e)?;
        let r = Reader::of(&eng, world)?;
        let at = r.position(e).ok_or_else(|| {
            format!(
                "entity {} has no og.Transform (or the running system does not read it)",
                e.to_bits()
            )
        })?;
        let body = body_height(&eng, &r, e, x, z)?;
        let ripple = match (physics(&eng), body) {
            (Ok(p), _) => p.water_ripple(e, at, x as f32, z as f32),
            (Err(_), Some(_)) => None,
            (Err(why), None) => return Err(why),
        };
        if body.is_none() && ripple.is_none() {
            return Err(format!(
                "entity {} has no og.Water, og.WaterSurface or og.WaterLine (or its waves have not stepped yet)",
                e.to_bits()
            ));
        }
        Ok(json!(at.y + body.unwrap_or(0.0) + ripple.unwrap_or(0.0)))
    })()
}

// ── ragdoll rigs ────────────────────────────────────────────────────────

/// A rig as the engine keeps it: what physics simulates, and the skeleton
/// joints its bones stand for (drawing).
#[derive(Clone, Debug)]
pub(crate) struct KeptRig {
    pub rig: RagdollRig,
    pub skeleton: u64,
    pub joints: Vec<i32>,
}

/// A rotation as a rig keeps it: a zero quaternion is the identity.
fn unit(q: Quat) -> Quat {
    if q.length_squared() < 1e-12 {
        Quat::IDENTITY
    } else {
        q.normalize()
    }
}

/// A RagdollRigDesc into a kept rig.
fn rig_of(d: RagdollRigDesc) -> Result<KeptRig, String> {
    let bones = d.bones;
    if bones.is_empty() {
        return Err("a ragdoll rig needs at least one bone".into());
    }
    let mut rig = RagdollRig::default();
    let mut joints = Vec::with_capacity(bones.len());
    for (i, b) in bones.iter().enumerate() {
        let parent = b.parent;
        if i64::from(parent) >= i as i64 {
            return Err(format!(
                "bone {i}: its parent ({parent}) must come before it"
            ));
        }
        rig.bones.push(RagdollBone {
            parent: (parent >= 0).then_some(parent as u32),
            rest: (b.position, unit(b.rotation)),
            length: b.length,
            axis: b.axis,
        });
        joints.push(b.joint);
    }
    Ok(KeptRig {
        rig,
        skeleton: d.skeleton,
        joints,
    })
}

pub fn og_physics_set_ragdoll_rig(world: u32, e: f64, desc: Option<String>) -> Result<(), String> {
    let r = (|| -> Result<(), String> {
        let eng = engine(world)?;
        let p = physics(&eng)?;
        let e = entity(e)?;
        let kept = rig_of(wire::arg(desc.as_deref(), "desc")?)?;
        p.set_ragdoll_rig(e, kept.rig.clone())
            .map_err(|err| err.reason().to_string())?;
        eng.render.borrow_mut().atmo.rigs.insert(e.to_bits(), kept);
        Ok(())
    })();
    match r {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

/// A rig from a skeleton's first skin: a bone per joint, parents first,
/// each from its joint's rest pose in model space to its first child joint
/// (a leaf gets 60% of its parent's length, pointing on).
pub(crate) fn rig_from_skeleton(s: &og_anim::SkeletonAsset) -> Result<RagdollRigDesc, String> {
    let skin = s.skins.first().ok_or("the skeleton has no skin")?;
    let model = s.model_space(&s.rest_pose());
    let joint_of = |node: u32| skin.joints.iter().position(|&n| n == node);
    // The nearest ancestor that is a joint.
    let parent_joint = |node: u32| {
        let mut n = s.nodes[node as usize].parent;
        while n != og_anim::NONE {
            if let Some(j) = joint_of(n) {
                return Some(j);
            }
            n = s.nodes[n as usize].parent;
        }
        None
    };
    // Parents first: by depth, then joint order.
    let depth = |mut j: usize| {
        let mut d = 0;
        while let Some(p) = parent_joint(skin.joints[j]) {
            j = p;
            d += 1;
            if d > 512 {
                break;
            }
        }
        d
    };
    let mut order: Vec<usize> = (0..skin.joints.len()).collect();
    og_core::sort::sort_by_key(&mut order, |&j| (depth(j), j));
    let bone_of = |j: usize| order.iter().position(|&o| o == j);
    let mut bones: Vec<RagdollBoneDesc> = Vec::with_capacity(order.len());
    let mut lengths: Vec<f32> = Vec::with_capacity(order.len());
    for &j in &order {
        let node = skin.joints[j];
        let at = model[node as usize];
        let child = (0..skin.joints.len()).find(|&c| parent_joint(skin.joints[c]) == Some(j));
        let parent = parent_joint(node).and_then(bone_of);
        let (length, axis) = match child {
            Some(c) => {
                let d = model[skin.joints[c] as usize].translation - at.translation;
                (d.length(), at.rotation.inverse() * d)
            }
            None => {
                let pl = parent.map_or(0.2, |p| lengths[p] * 0.6);
                let dir = parent_joint(node)
                    .map(|p| at.translation - model[skin.joints[p] as usize].translation)
                    .unwrap_or(Vec3::Y);
                (pl, at.rotation.inverse() * dir)
            }
        };
        let length = length.max(0.02);
        lengths.push(length);
        let axis = axis.normalize_or(Vec3::Y);
        bones.push(RagdollBoneDesc {
            parent: parent.map_or(-1, |p| p as i32),
            position: at.translation,
            rotation: at.rotation,
            length,
            axis,
            joint: j as i32,
        });
    }
    Ok(RagdollRigDesc { bones, skeleton: 0 })
}

/// The system that hands each og.Ragdoll the animation's pose of this
/// tick (after og.anim, before og.physics): every bone of a rig made from a
/// skeleton (`ragdollRigFromSkeleton`) follows its joint, in world space, so
/// a ragdoll at weight 0 tracks the animation and goes limp from it. A bone
/// with no joint, or an entity with no pose yet, keeps the rig's rest pose.
pub(crate) fn schedule_ragdoll_targets(
    world: &mut og_ecs::World,
    engine_id: u32,
    physics: &og_physics::PhysicsSystem,
) -> Result<(), String> {
    use og_physics::{Parent, Ragdoll, Transform};
    let physics = physics.clone();
    world
        .register::<Transform>()
        .and_then(|_| world.register::<Ragdoll>())
        .and_then(|_| world.register::<Parent>())
        .map_err(|e| e.reason().to_string())?;
    world
        .add_system(
            og_ecs::SystemDesc::new(RAGDOLL_TARGETS)
                .read::<Ragdoll>()
                .read::<Transform>()
                .read::<Parent>()
                .query(
                    og_ecs::QueryDesc::new()
                        .read::<Ragdoll>()
                        .read::<Transform>(),
                ),
            move |ctx| {
                let Ok(eng) = engine(engine_id) else {
                    return Ok(());
                };
                let Some(anim) = eng.state.try_borrow().ok().and_then(|s| s.anim.clone()) else {
                    return Ok(());
                };
                let Ok(render) = eng.render.try_borrow() else {
                    return Ok(());
                };
                let rigs = &render.atmo.rigs;
                if rigs.is_empty() {
                    return Ok(());
                }
                for a in ctx.query(0)? {
                    for &e in a.entities() {
                        let Some(kept) = rigs.get(&e.to_bits()) else {
                            continue;
                        };
                        let (Some(skeleton), Some(pose)) =
                            (anim.skeleton(kept.skeleton), anim.pose(e))
                        else {
                            continue;
                        };
                        let Some(skin) = skeleton.skins.first() else {
                            continue;
                        };
                        // At the entity's world pose, as og.physics places the ragdoll (#10).
                        let t = ctx.get::<Transform>(e)?;
                        let (position, quaternion) = og_physics::world_pose(
                            e,
                            (t.position, t.quaternion),
                            |c| {
                                ctx.get::<Parent>(c)
                                    .ok()
                                    .map(|p| Entity::from_bits(p.entity))
                            },
                            |p| ctx.get::<Transform>(p).ok(),
                        );
                        let model = skeleton.model_space(&pose);
                        let targets = kept
                            .rig
                            .bones
                            .iter()
                            .zip(&kept.joints)
                            .map(|(b, &j)| {
                                let (p, q) = usize::try_from(j)
                                    .ok()
                                    .and_then(|j| skin.joints.get(j))
                                    .and_then(|&n| model.get(n as usize))
                                    .map_or(b.rest, |m| (m.translation, m.rotation));
                                (position + quaternion * p, (quaternion * q).normalize())
                            })
                            .collect();
                        physics.set_ragdoll_targets(e, targets);
                    }
                }
                Ok(())
            },
        )
        .map(|_| ())
        .map_err(|e| e.reason().to_string())
}

/// The name of the ragdoll target system.
pub(crate) const RAGDOLL_TARGETS: &str = "og.ragdollTargets";

pub fn og_physics_ragdoll_rig_from_skeleton(
    world: u32,
    skeleton: f64,
) -> Result<RagdollRigDesc, String> {
    (|| {
        let eng = engine(world)?;
        let anim = eng
            .state
            .borrow()
            .anim
            .clone()
            .ok_or("this engine has no animation")?;
        let s = anim
            .skeleton(skeleton as u64)
            .ok_or_else(|| format!("{skeleton} is not a loaded skeleton (og.load.animation)"))?;
        let mut rig = rig_from_skeleton(&s)?;
        rig.skeleton = skeleton as u64;
        Ok(rig)
    })()
}

/// The ragdolls' drawing: each posed model (palette `palettes[e]`) whose
/// entity has a rig from its skeleton gets its joints blended towards the
/// bones' bodies by og.Ragdoll.blend.
// Rendering only: the drawn ragdoll's skin palette.
#[allow(clippy::disallowed_methods)]
pub(crate) fn draw_ragdolls(
    world: &World,
    rs: &mut RenderState,
    world_of: &mut dyn FnMut(u64) -> Mat4,
) {
    let RenderState {
        scene,
        fx,
        draw,
        atmo,
        ..
    } = rs;
    let (Some(physics), Some(anim)) = (draw.atmo.physics.as_ref(), fx.anim.as_ref()) else {
        return;
    };
    let layout = draw.atmo.ragdoll.as_ref();
    for (e, v) in rows(world, layout) {
        let Ok(v) = record::Ragdoll::from_json(&v) else {
            continue;
        };
        let blend = v.blend.clamp(0.0, 1.0);
        let (Some(kept), Some(&palette)) = (atmo.rigs.get(&e), atmo.palettes.get(&e)) else {
            continue;
        };
        if blend <= 0.0 {
            continue;
        }
        let (Some(skeleton), Some(bodies)) = (
            anim.skeleton(kept.skeleton),
            physics.ragdoll_pose(Entity::from_bits(e)),
        ) else {
            continue;
        };
        let Some(skin) = skeleton.skins.first() else {
            continue;
        };
        let Some(matrices) = scene
            .objects
            .joint_palettes
            .get_mut(palette as usize)
            .map(|p| &mut p.matrices)
        else {
            continue;
        };
        let to_model = world_of(e).inverse();
        for (bone, &(p, q)) in bodies.iter().enumerate() {
            let Some(j) = kept.joints.get(bone).and_then(|&j| usize::try_from(j).ok()) else {
                continue;
            };
            let (Some(m), Some(ib)) = (matrices.get_mut(j), skin.inverse_bind.get(j)) else {
                continue;
            };
            let animated = *m * ib.inverse();
            let simulated = to_model * Mat4::from_rotation_translation(q, p);
            let (sa, ra, ta) = animated.to_scale_rotation_translation();
            let (_, rb, tb) = simulated.to_scale_rotation_translation();
            let blended =
                Mat4::from_scale_rotation_translation(sa, ra.slerp(rb, blend), ta.lerp(tb, blend));
            *m = blended * *ib;
        }
    }
}
