//! The step and the scene queries, written once and compiled for each
//! profile: `dim3.rs` and `dim2.rs` include this file as their `sim` module,
//! with `rp` naming rapier3d's or rapier2d's prelude and the conversions
//! (`v`, `pose`, `unpose`, `angvel`, ...) defined beside it.

use super::{
    PROFILE_NAME, ShapeCastOptions, angvel, baked, cache, chunk_cache, field_chunk, flat,
    from_heightfield, from_mesh, pose, primitive, rp, unangvel, unpose, unv, v,
};
use super::{VehicleState, ext, platformers, ragdolls, vehicles};
use crate::gather::{Body, Scene, Static, StaticRun};
use crate::schema::*;
use crate::stats::{Laps, Phase};
use crate::{ContactHook, OverlapDesc, ShapeDesc};
use crate::{
    Hole, QueryFilter, RaycastHit, SHAPE_CACHE_LIMIT, ShapeCastDesc, ShapeCastHit, ShapeKey,
    Shared, StepOut,
};
use og_core::glam::{Quat, Vec3};
use og_core::{Error, ErrorKind, IdMap, Result};
use og_ecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[path = "sleep.rs"]
mod sleep;

#[path = "sim/bodies.rs"]
mod bodies;
#[path = "sim/colliders.rs"]
mod colliders;
#[path = "sim/persist.rs"]
mod persist;
#[path = "sim/queries.rs"]
mod queries;
#[path = "sim/readback.rs"]
mod readback;
#[path = "sim/stepping.rs"]
mod stepping;

pub(crate) use bodies::*;
pub(crate) use colliders::*;
pub(crate) use persist::*;
pub(crate) use queries::*;
use readback::*;
pub(crate) use stepping::*;

fn invalid(what: &str, value: f32) -> Error {
    Error::new(
        ErrorKind::Invalid,
        format!("og.Collider.{what} must be finite and at least 0, got {value}"),
    )
}

/// The overlap (m) below which a slow, supported body counts as resting;
/// deeper, it keeps moving so the solver can push it out.
const REST_PENETRATION: f32 = 0.002;

/// `v` without the components its body's `lock` bits forbid.
fn unlocked(v: Velocity, lock: u32) -> Velocity {
    let keep = |bit: u32, x: f32| if lock & bit != 0 { 0.0 } else { x };
    Velocity {
        linear: Vec3::new(
            keep(LOCK_TRANSLATION_X, v.linear.x),
            keep(LOCK_TRANSLATION_Y, v.linear.y),
            keep(LOCK_TRANSLATION_Z, v.linear.z),
        ),
        angular: Vec3::new(
            keep(LOCK_ROTATION_X, v.angular.x),
            keep(LOCK_ROTATION_Y, v.angular.y),
            keep(LOCK_ROTATION_Z, v.angular.z),
        ),
    }
}

fn slow(v: &Velocity, limit: f32) -> bool {
    v.linear.length_squared() < limit * limit && v.angular.length_squared() < limit * limit
}

fn entity_of(c: &rp::Collider) -> Entity {
    Entity::from_bits(c.user_data as u64)
}

/// A body as the last tick left it: its rapier handle and the component
/// values physics last read or wrote. A component that no longer equals
/// these was changed by something else, and the rapier body follows it.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Tracked {
    pub(crate) handle: rp::RigidBodyHandle,
    body: RigidBody,
    collider: Option<Collider>,
    scale: Vec3,
    position: Vec3,
    quaternion: Quat,
    velocity: Option<Velocity>,
}

/// A compound child collider as the last build of its body placed it
/// ([`crate::gather::Child`], its entity by bits).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct TrackedChild {
    entity: u64,
    position: Vec3,
    quaternion: Quat,
    scale: Vec3,
    collider: Collider,
}

impl TrackedChild {
    fn of(c: &crate::gather::Child) -> TrackedChild {
        TrackedChild {
            entity: c.entity.to_bits(),
            position: c.position,
            quaternion: c.quaternion,
            scale: c.scale,
            collider: c.collider,
        }
    }
}

/// What the simulation keeps beyond its first form (G2-3 and later):
/// saved in the block's extension sections ([`crate::ext_sections`]), not
/// in [`Sim`]'s own bytes, so a world that uses none of it saves and
/// hashes as before.
#[derive(Default)]
pub(crate) struct SimExt {
    /// Each compound body's child colliders, by body entity, as built.
    pub(crate) compounds: IdMap<u64, Vec<TrackedChild>>,
    /// The ticks each awake dynamic body has rested in a row, by entity,
    /// while sleep is on ([`sleep`]); none at 0.
    pub(crate) still: IdMap<u64, u32>,
    /// Whether some collider has its contact hooks on ([`sync_hooks`]).
    pub(crate) hooked: bool,
    /// What a vehicle keeps beyond its controller, by entity: its
    /// gearbox (drivetrain model) and the og.VehicleWheel children it was
    /// built with; none for an arcade car on its four implicit wheels.
    pub(crate) vehicles: IdMap<u64, VehicleExt>,
    /// The soft volumes' points and surfaces, by entity (og.SoftBody).
    pub(crate) softs: IdMap<u64, crate::softbody::SoftState>,
    /// The garments' points and edges, by entity (og.Garment).
    pub(crate) garments: IdMap<u64, crate::garment::GarmentState>,
    /// Each static height-field collider built in chunks
    /// ([`crate::FIELD_CHUNK`]), by entity: its chunks' colliders in chunk
    /// order (the first is its [`TrackedStatic::handle`]).
    pub(crate) fields: IdMap<u64, Vec<rp::ColliderHandle>>,
}

/// A vehicle's state beyond its controller ([`SimExt::vehicles`]).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct VehicleExt {
    pub(crate) gearbox: crate::drivetrain::Gearbox,
    /// The og.VehicleWheel children its wheels were built from, in order:
    /// each one's entity and what building bakes in (its mount's x, y and
    /// z, radius and suspension rest, as bits). Empty: its implicit wheels.
    pub(crate) wheels: Vec<(u64, [u32; 5])>,
}

/// Extension section tags of the og.physics block.
pub(crate) const EXT_COMPOUNDS: u16 = 1;
pub(crate) const EXT_SLEEP: u16 = 2;
pub(crate) const EXT_HOOKS: u16 = 3;
pub(crate) const EXT_VEHICLES: u16 = 4;
pub(crate) const EXT_SOFTS: u16 = 5;
pub(crate) const EXT_GARMENTS: u16 = 6;
pub(crate) const EXT_FIELDS: u16 = 7;

impl SimExt {
    /// Its sections, each `(tag, bytes)`; none for what is empty.
    pub(crate) fn sections(&self) -> Vec<(u16, Vec<u8>)> {
        let mut out = Vec::new();
        if !self.compounds.is_empty() {
            let mut bytes = Vec::new();
            let list: Vec<(&u64, &Vec<TrackedChild>)> = self.compounds.iter().collect();
            crate::encode(&mut bytes, &list).expect("compounds serialize into memory");
            out.push((EXT_COMPOUNDS, bytes));
        }
        if !self.still.is_empty() {
            let mut bytes = Vec::new();
            let list: Vec<(&u64, &u32)> = self.still.iter().collect();
            crate::encode(&mut bytes, &list).expect("rest counts serialize into memory");
            out.push((EXT_SLEEP, bytes));
        }
        if self.hooked {
            out.push((EXT_HOOKS, Vec::new()));
        }
        if !self.vehicles.is_empty() {
            let mut bytes = Vec::new();
            let list: Vec<(&u64, &VehicleExt)> = self.vehicles.iter().collect();
            crate::encode(&mut bytes, &list).expect("vehicles serialize into memory");
            out.push((EXT_VEHICLES, bytes));
        }
        if !self.softs.is_empty() {
            let mut bytes = Vec::new();
            let list: Vec<(&u64, &crate::softbody::SoftState)> = self.softs.iter().collect();
            crate::encode(&mut bytes, &list).expect("soft volumes serialize into memory");
            out.push((EXT_SOFTS, bytes));
        }
        if !self.garments.is_empty() {
            let mut bytes = Vec::new();
            let list: Vec<(&u64, &crate::garment::GarmentState)> = self.garments.iter().collect();
            crate::encode(&mut bytes, &list).expect("garments serialize into memory");
            out.push((EXT_GARMENTS, bytes));
        }
        if !self.fields.is_empty() {
            let mut bytes = Vec::new();
            let list: Vec<(&u64, &Vec<rp::ColliderHandle>)> = self.fields.iter().collect();
            crate::encode(&mut bytes, &list).expect("field chunks serialize into memory");
            out.push((EXT_FIELDS, bytes));
        }
        out
    }

    /// From the sections a save held; a tag this build does not know is
    /// skipped (a later build's).
    pub(crate) fn from_sections(sections: &[(u16, &[u8])]) -> Result<SimExt> {
        use bincode::Options;
        let parse = |e: bincode::Error| {
            Error::new(
                ErrorKind::Parse,
                format!("the physics block's extension does not decode: {e}"),
            )
        };
        let mut ext = SimExt::default();
        for &(tag, bytes) in sections {
            if tag == EXT_COMPOUNDS {
                let list: Vec<(u64, Vec<TrackedChild>)> =
                    crate::codec().deserialize(bytes).map_err(parse)?;
                ext.compounds = list.into_iter().collect();
            } else if tag == EXT_HOOKS {
                ext.hooked = true;
            } else if tag == EXT_SLEEP {
                let list: Vec<(u64, u32)> = crate::codec().deserialize(bytes).map_err(parse)?;
                ext.still = list.into_iter().collect();
            } else if tag == EXT_VEHICLES {
                let list: Vec<(u64, VehicleExt)> =
                    crate::codec().deserialize(bytes).map_err(parse)?;
                ext.vehicles = list.into_iter().collect();
            } else if tag == EXT_SOFTS {
                let list: Vec<(u64, crate::softbody::SoftState)> =
                    crate::codec().deserialize(bytes).map_err(parse)?;
                ext.softs = list.into_iter().collect();
            } else if tag == EXT_GARMENTS {
                let list: Vec<(u64, crate::garment::GarmentState)> =
                    crate::codec().deserialize(bytes).map_err(parse)?;
                ext.garments = list.into_iter().collect();
            } else if tag == EXT_FIELDS {
                let list: Vec<(u64, Vec<rp::ColliderHandle>)> =
                    crate::codec().deserialize(bytes).map_err(parse)?;
                ext.fields = list.into_iter().collect();
            }
        }
        Ok(ext)
    }
}

/// A static collider (no body) as the last tick left it.
#[derive(Clone, Serialize, Deserialize)]
struct TrackedStatic {
    handle: rp::ColliderHandle,
    collider: Collider,
    scale: Vec3,
    position: Vec3,
    quaternion: Quat,
}

/// The rapier world kept between ticks (contacts, warm starts, broad
/// phase), with what each entity's components were when physics last
/// touched them. Saved whole in the world's `og.physics` state block.
#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Sim {
    pub(crate) world: rp::PhysicsWorld,
    pub(crate) bodies: IdMap<u64, Tracked>,
    statics: IdMap<u64, TrackedStatic>,
    /// The fixed body world-anchored joints attach to.
    pub(crate) ground: Option<rp::RigidBodyHandle>,
    pub(crate) joints: IdMap<u64, ext::TrackedJoint>,
    pub(crate) vehicles: IdMap<u64, VehicleState>,
    pub(crate) ropes: IdMap<u64, crate::verlet::RopeState>,
    pub(crate) cloths: IdMap<u64, crate::verlet::ClothState>,
    pub(crate) fluids: IdMap<u64, crate::fluid::FluidState>,
    pub(crate) surfaces: IdMap<u64, crate::water::SurfaceState>,
    pub(crate) lines: IdMap<u64, crate::water::LineState>,
    pub(crate) ragdolls: IdMap<u64, super::RagdollState>,
    pub(crate) fluids3: IdMap<u64, crate::fluid3d::Fluid3State>,
    /// Per og.WaterSurface, the bodies that cut it in the last ticks, for
    /// the renderer's ripples; per og.Water sea, the og.Buoyancy hulls in
    /// it, for its hull wave suppression: not world state (never saved,
    /// never read by the tick).
    #[serde(skip)]
    pub(crate) wakes: IdMap<u64, crate::water::Wakes>,
    /// The scene's static colliders, by archetype, as the last [`sync`]
    /// left `statics` and the rapier world in line with them
    /// ([`Scene::static_runs`]): the next one passes over the archetypes
    /// nothing wrote since. Not world state (never saved): a simulation
    /// that was loaded has none, and its next sync reads every static.
    /// (A shift of the origin writes the columns it moves.)
    #[serde(skip)]
    static_runs: Vec<StaticRun>,
    /// Saved in the block's extension sections, not here.
    #[serde(skip)]
    pub(crate) ext: SimExt,
}

impl Sim {
    /// Whether this is the 2D profile (points stay in their entity's plane).
    pub(crate) fn flat(&self) -> bool {
        super::DIM == 2
    }

    /// og.Physics.gravity as the last step applied it.
    pub(crate) fn gravity(&self) -> Vec3 {
        unv(self.world.gravity)
    }

    /// Moves everything the simulation keeps in the world by `s.d`
    /// (og.WorldOrigin, R12), in place: every body's pose (a kinematic
    /// body's target with it) and every parentless collider's, the joints
    /// anchored to the world (the ground body stays at the zero), what each
    /// entity's components were (in its Transform frame), vehicles' wheel
    /// points, and the points of ropes, cloths, fluids, soft volumes and
    /// ragdoll targets. Contacts, warm starts and islands are relative to
    /// the bodies and stay; the broad phase refits to the moved colliders
    /// on the next step, as for any move, in handle order.
    pub(crate) fn shift(&mut self, s: &og_ecs::Shift) {
        let moved = |p: Vec3| s.moved(p);
        let rpm = |t: rp::Vector| v(moved(unv(t)));
        let ground = self.ground;
        let w = &mut self.world;
        let asleep: Vec<_> = w
            .bodies
            .iter()
            .filter(|(_, rb)| rb.is_dynamic() && rb.is_sleeping())
            .map(|(h, _)| h)
            .collect();
        for (h, rb) in w.bodies.iter_mut() {
            if Some(h) == ground {
                continue;
            }
            let kinematic = rb.body_type() == rp::RigidBodyType::KinematicPositionBased;
            let next = *rb.next_position();
            let mut at = *rb.position();
            at.translation = rpm(at.translation);
            rb.set_position(at, false);
            if kinematic {
                let mut to = next;
                to.translation = rpm(to.translation);
                rb.set_next_kinematic_position(to);
            }
        }
        for (_, c) in w.colliders.iter_mut() {
            if c.parent().is_none() {
                let mut at = *c.position();
                at.translation = rpm(at.translation);
                c.set_position(at);
            }
        }
        // Rapier wakes every contact partner of a collider whose pose
        // changed, so a whole resting world would wake at the shift. The
        // moves are taken now (the broad phase refits, the narrow phase
        // keeps its pairs) and what slept goes back to sleep where it is.
        if !asleep.is_empty() {
            w.detect_collisions(&(), &());
            for h in asleep {
                w.bodies[h].sleep();
            }
        }
        ext::shift_joints(self, s);
        for (e, t) in self.bodies.iter_mut() {
            t.position = s.moved_of(*e, t.position);
        }
        for (e, t) in self.statics.iter_mut() {
            t.position = s.moved_of(*e, t.position);
        }
        for (_, v) in self.vehicles.iter_mut() {
            v.shift(s);
        }
        let points = |p: &mut Vec<Vec3>| p.iter_mut().for_each(|p| *p = moved(*p));
        for (_, r) in self.ropes.iter_mut() {
            points(&mut r.points);
            points(&mut r.previous);
        }
        for (_, c) in self.cloths.iter_mut() {
            points(&mut c.points);
            points(&mut c.previous);
        }
        for (_, f) in self.fluids.iter_mut() {
            for p in &mut f.positions {
                *p = moved(p.extend(0.0)).truncate();
            }
        }
        for (_, f) in self.fluids3.iter_mut() {
            points(&mut f.positions);
        }
        for (_, r) in self.ragdolls.iter_mut() {
            r.shift(s);
        }
        for (_, b) in self.ext.softs.iter_mut() {
            points(&mut b.points);
            points(&mut b.previous);
        }
        for (_, g) in self.ext.garments.iter_mut() {
            points(&mut g.points);
            points(&mut g.previous);
            points(&mut g.targets);
        }
    }
}

#[cfg(test)]
mod sleep_path {
    //! G2-3's sleep rests on rapier 0.36 doing three things with a body
    //! built `can_sleep(false)`: `sleep()` puts it to sleep and it stays
    //! asleep, unmoved, step after step; it never falls asleep on its own;
    //! and what wakes it is deterministic. Checked here before anything is
    //! built on it.
    use super::*;

    fn body(w: &mut rp::PhysicsWorld, y: f32, radius: f32) -> rp::RigidBodyHandle {
        let h = w.bodies.insert(
            rp::RigidBodyBuilder::dynamic()
                .pose(pose(Vec3::new(0.0, y, 0.0), Quat::IDENTITY))
                .can_sleep(false),
        );
        w.colliders
            .insert_with_parent(rp::ColliderBuilder::ball(radius).build(), h, &mut w.bodies);
        h
    }

    fn scene() -> (rp::PhysicsWorld, rp::RigidBodyHandle) {
        let mut w = rp::PhysicsWorld {
            gravity: v(Vec3::new(0.0, -9.81, 0.0)),
            ..Default::default()
        };
        w.integration_parameters.dt = 1.0 / 60.0;
        let floor = w.bodies.insert(rp::RigidBodyBuilder::fixed());
        let big = Vec3::new(20.0, 0.5, 20.0);
        let floor_shape = rp::ColliderBuilder::new(
            primitive(
                &Collider {
                    shape: SHAPE_BOX,
                    size: big * 2.0,
                    ..Collider::default()
                },
                Vec3::ONE,
            )
            .unwrap()
            .unwrap(),
        )
        .position(pose(Vec3::new(0.0, -0.5, 0.0), Quat::IDENTITY));
        w.colliders
            .insert_with_parent(floor_shape.build(), floor, &mut w.bodies);
        let ball = body(&mut w, 0.6, 0.5);
        (w, ball)
    }

    #[test]
    fn sleep_holds_never_starts_itself_and_wakes_alike() {
        // Never asleep by itself, however long it rests.
        let (mut w, ball) = scene();
        for _ in 0..600 {
            w.step();
        }
        assert!(!w.bodies[ball].is_sleeping(), "can_sleep(false) slept");

        // Put to sleep: it stays asleep and does not move by a bit.
        w.bodies[ball].sleep();
        let at = *w.bodies[ball].position();
        for _ in 0..300 {
            w.step();
            assert!(w.bodies[ball].is_sleeping(), "woke by itself");
            assert_eq!(*w.bodies[ball].position(), at, "a sleeping body moved");
        }

        // A ball dropped on it: whatever rapier does with the contact, two
        // runs (and a run restored from a save while it sleeps) do the same.
        let saved = {
            let mut b = Vec::new();
            crate::encode(&mut b, &w).unwrap();
            b
        };
        let drop = |w: &mut rp::PhysicsWorld| -> Vec<(bool, u32)> {
            let top = body(w, 3.0, 0.3);
            (0..120)
                .map(|_| {
                    w.step();
                    let rb = &w.bodies[ball];
                    let _ = &w.bodies[top];
                    (rb.is_sleeping(), rb.position().translation.y.to_bits())
                })
                .collect()
        };
        let a = drop(&mut w);
        use bincode::Options;
        let mut restored: rp::PhysicsWorld = crate::codec().deserialize(&saved).unwrap();
        let b = drop(&mut restored);
        assert_eq!(a, b, "the wake is not deterministic across a restore");
        // Rapier wakes it when an awake body comes into contact (the
        // falling ball lands about 35 ticks in): its rule, in the island
        // manager, saved with the world; og-physics reads it back.
        let woke = a.iter().position(|s| !s.0);
        assert!(matches!(woke, Some(30..=40)), "woke at {woke:?}");
    }

    #[test]
    fn a_body_slept_beside_an_awake_neighbour_at_rest() {
        // Two balls touching on the floor, one island; only one is slept.
        let (mut w, ball) = scene();
        let other = w.bodies.insert(
            rp::RigidBodyBuilder::dynamic()
                .pose(pose(Vec3::new(1.0, 0.5, 0.0), Quat::IDENTITY))
                .can_sleep(false),
        );
        w.colliders.insert_with_parent(
            rp::ColliderBuilder::ball(0.5).build(),
            other,
            &mut w.bodies,
        );
        for _ in 0..300 {
            w.step();
        }
        w.bodies[ball].sleep();
        let mut slept = 0;
        for _ in 0..120 {
            w.step();
            slept += usize::from(w.bodies[ball].is_sleeping());
        }
        // A neighbour at rest does not wake it, and is not put to sleep.
        assert_eq!(slept, 120);
        assert!(!w.bodies[other].is_sleeping());
    }

    #[test]
    fn an_awake_body_resting_on_a_sleeping_one_sinks_through_it() {
        // Why sleep.rs sleeps groups whole and wakes what touches an awake
        // body: rapier leaves a sleeping body out of contact solving, and
        // wakes it only when a contact starts, not for one already there.
        let (mut w, ball) = scene();
        w.remove_body(ball);
        let cube = || {
            let c = Collider {
                shape: SHAPE_BOX,
                size: Vec3::ONE,
                ..Collider::default()
            };
            rp::ColliderBuilder::new(primitive(&c, Vec3::ONE).unwrap().unwrap()).build()
        };
        let mut boxed = |y: f32| {
            let h = w.bodies.insert(
                rp::RigidBodyBuilder::dynamic()
                    .pose(pose(Vec3::new(0.0, y, 0.0), Quat::IDENTITY))
                    .can_sleep(false),
            );
            w.colliders.insert_with_parent(cube(), h, &mut w.bodies);
            h
        };
        let (low, top) = (boxed(0.5), boxed(1.5));
        for _ in 0..300 {
            w.step();
        }
        let resting = w.bodies[top].position().translation.y;
        assert!(
            (resting - 1.5).abs() < 0.01,
            "the stack settled at {resting}"
        );
        w.bodies[low].sleep();
        for _ in 0..120 {
            w.step();
        }
        assert!(w.bodies[low].is_sleeping(), "the contact woke it");
        let sunk = w.bodies[top].position().translation.y;
        assert!(
            sunk < 1.0,
            "the top box stayed on the sleeping one ({sunk})"
        );
    }
}
