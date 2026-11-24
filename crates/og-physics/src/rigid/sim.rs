//! The step and the scene queries, written once and compiled for each
//! profile: `dim3.rs` and `dim2.rs` include this file as their `sim` module,
//! with `rp` naming rapier3d's or rapier2d's prelude and the conversions
//! (`v`, `pose`, `unpose`, `angvel`, ...) defined beside it.

use super::{
    DIM, PROFILE_NAME, ShapeCastOptions, angvel, baked, cache, chunk_cache, field_chunk, flat,
    from_heightfield, from_mesh, gyroscopic_builder, pose, primitive, principal_inertia, rp,
    set_gyroscopic, unangvel, unpose, unv, v,
};
use super::{VehicleState, ext, platformers, ragdolls, vehicles};
use crate::gather::{Body, Scene, Static, StaticRun};
use crate::schema::*;
use crate::stats::{Laps, Phase};
use crate::{ContactHook, MassProperties, OverlapDesc, PointOverlapDesc, ShapeDesc};
use crate::{
    Hole, QUERY_DYNAMIC, QUERY_FIXED, QUERY_KINEMATIC, QueryFilter, RaycastHit, SHAPE_CACHE_LIMIT,
    ShapeCastDesc, ShapeCastHit, ShapeKey, Shared, StepOut,
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
#[path = "sim/dump.rs"]
mod dump;
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
    /// The bodies near each og.Water's surface and which side of it they
    /// were on ([`crate::water::Under`]).
    pub(crate) under: crate::water::Under,
    /// Snapshot of the last tick's `og.ContactHook` map, so a query or an
    /// origin shift (`Sim::shift`) preserves one-way/ignore filtering when
    /// refreshing sleeping contacts.
    pub(crate) last_hooks: IdMap<u64, ContactHook>,
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
/// The joints' part, kept with them ([`ext::joint_section`]), not in [`SimExt`].
pub(crate) const EXT_JOINTS: u16 = 8;
pub(crate) const EXT_UNDER: u16 = 9;

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
            let mut bytes = Vec::new();
            use og_core::Reflect;
            let layout = og_core::Layout::of(&ContactHook::schema());
            let hooks: Vec<_> = self
                .last_hooks
                .iter()
                .map(|(&e, hook)| (e, layout.encode(hook).expect("hook matches its schema")))
                .collect();
            crate::encode(&mut bytes, &hooks).expect("hooks serialize into memory");
            out.push((EXT_HOOKS, bytes));
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
        if !self.under.is_empty() {
            let mut bytes = Vec::new();
            crate::encode(&mut bytes, &self.under).expect("wet bodies serialize into memory");
            out.push((EXT_UNDER, bytes));
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
                use og_core::Reflect;
                let hooks: Vec<(u64, Vec<u8>)> =
                    crate::codec().deserialize(bytes).map_err(parse)?;
                let layout = og_core::Layout::of(&ContactHook::schema());
                ext.last_hooks = hooks
                    .into_iter()
                    .map(|(e, bytes)| Ok((e, layout.decode::<ContactHook>(&bytes)?)))
                    .collect::<Result<_>>()?;
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
            } else if tag == EXT_UNDER {
                ext.under = crate::codec().deserialize(bytes).map_err(parse)?;
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
    pub(crate) fluids3: IdMap<u64, crate::fluid::Fluid3State>,
    /// Per og.WaterSurface and og.Water, the bodies that cut it in the last
    /// ticks, for the renderer's ripples, wakes and hull wave suppression:
    /// not world state (never saved, never read by the tick).
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
    /// The block's extension sections: [`SimExt`]'s, then the joints'.
    pub(crate) fn sections(&self) -> Vec<(u16, Vec<u8>)> {
        let mut out = self.ext.sections();
        out.extend(ext::joint_section(&self.joints));
        out
    }

    /// Takes the extension sections a save held.
    pub(crate) fn restore_sections(&mut self, sections: &[(u16, &[u8])]) -> Result<()> {
        self.ext = SimExt::from_sections(sections)?;
        ext::restore_joint_section(&mut self.joints, sections)
    }

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
        // Always update body-attached collider world poses right away, even
        // when no body is asleep, so saves/queries immediately after a shift
        // see the shifted poses.
        w.bodies
            .propagate_modified_body_positions_to_colliders(&mut w.colliders);
        // Rapier wakes every contact partner of a collider whose pose
        // changed, so a whole resting world would wake at the shift. Refresh
        // broad-phase/contacts using the active ContactHook snapshot so
        // one-way and ignore rules stay intact, then restore sleep.
        if !asleep.is_empty() {
            if self.ext.last_hooks.is_empty() {
                w.detect_collisions(&(), &());
            } else {
                w.detect_collisions(&Hooks(&self.ext.last_hooks), &());
            }
            for h in asleep {
                w.bodies[h].sleep();
            }
        }
        self.refresh_query_bounds();
        self.static_runs.clear();
        ext::shift_joints(self, s);
        for (_, t) in self.bodies.iter_mut() {
            t.position = moved(t.position);
        }
        for (_, t) in self.statics.iter_mut() {
            t.position = moved(t.position);
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

    /// The solver's computed mass properties for `entity`'s body.
    pub(crate) fn mass_properties(&self, entity: u64) -> Option<MassProperties> {
        let t = self.bodies.get(&entity)?;
        let rb = self.world.bodies.get(t.handle)?;
        Some(MassProperties {
            mass: rb.mass(),
            center_of_mass: unv(rb.center_of_mass()),
            inertia: principal_inertia(rb),
        })
    }

    /// World-space linear velocity of `entity`'s body at world point `point`.
    pub(crate) fn velocity_at_point(&self, entity: u64, point: Vec3) -> Option<Vec3> {
        let t = self.bodies.get(&entity)?;
        let rb = self.world.bodies.get(t.handle)?;
        Some(unv(rb.velocity_at_point(v(flat(point)))))
    }
}
