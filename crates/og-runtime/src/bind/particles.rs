//! Particle events gameplay reads (R7): `og.ParticleEvent`, sent by the
//! `og.particles` system for every particle system whose emitter names
//! them (`events birth death collision` in the emitter file).
//!
//! Such a system is simulated in the tick, on the CPU path the renderer
//! also runs ([`og_render::CpuParticles::advance`], fixed math and the
//! engine's seeded randomness), so its events replay with the world. Its
//! state is the `og.particles` state block, which snapshots, restores and
//! the world hash carry. The renderer then draws that simulation as it
//! stands (`RenderScene::hosted_particles`): nothing it draws feeds back.
//!
//! og.ParticleEvent is registered lazily (`crate::engine::lazy`): a world that
//! never names it has neither the system nor the state block, and its
//! particle systems are the renderer's alone, as before.

use crate::host::codec::{Layout, Rec, RecordKind, Row, Slot};
use glam::{Mat4, Quat, Vec3};
use og_core::block_form as bf;
use og_core::bytes::ByteReader;
use og_core::{Error, ErrorKind, IdMap};
use og_ecs::{Entity, QueryDesc, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use og_render::{
    CpuParticles, ParticleClock, ParticleEmitter, ParticleEvent, ParticleEventKind, ParticleState,
    ParticleSystem, SceneCollider, SimulationSpace,
};
use serde_json::json;
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) const SYSTEM: &str = "og.particles";
pub(crate) const EVENT: &str = "og.ParticleEvent";
const SYSTEMS: &str = "og.ParticleSystem";
/// og.ParticleSystem.startTick's unwritten value (`crate::present::fx::PARTICLES_ARMED`).
const ARMED: u64 = u32::MAX as u64;

/// What the tick needs of a loaded emitter that sends events.
pub(crate) struct Definition {
    /// The renderer's handle of it.
    pub handle: og_render::EmitterHandle,
    pub emitter: ParticleEmitter,
    /// Its sub-emitters' emitters, in `emitter.sub_emitters` order.
    pub subs: Vec<Option<ParticleEmitter>>,
    /// A mesh shape's surface table (`og_render::mesh_points`).
    pub points: Vec<[f32; 4]>,
}

/// The `og.particles` state: emitter definitions by load index (assets,
/// not saved) and each hosted system's simulation by entity bits.
#[derive(Default)]
pub(crate) struct Hosted {
    pub emitters: BTreeMap<usize, Arc<Definition>>,
    pub systems: BTreeMap<u64, HostedSystem>,
}

/// One system the tick simulates.
#[derive(Clone, Debug, Default)]
pub(crate) struct HostedSystem {
    pub cpu: CpuParticles,
    /// Its particles are in world space (`space world`): they move with an
    /// origin shift; a local-space system's move with its entity.
    pub world: bool,
}

/// og.particles on one engine: the systems its world's block saves,
/// shared with its system and the effects that host them.
#[derive(Clone, Default)]
pub(crate) struct ModuleState(Arc<Mutex<Hosted>>);

/// An engine's hosted particle systems.
pub(crate) fn shared(eng: &crate::engine::Engine) -> Arc<Mutex<Hosted>> {
    eng.modules.particles.0.clone()
}

pub(crate) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn invalid(m: impl Into<String>) -> Error {
    Error::new(ErrorKind::Invalid, m.into())
}

fn layout(world: &World, name: &str, kind: RecordKind) -> og_core::Result<(SchemaId, Layout)> {
    let id = world.id_by_name(name)?;
    Ok((id, Layout::from_schema(world.schema(id)?, kind)))
}

struct SystemSlots {
    visible: Slot,
    emitter: Slot,
    seed: Slot,
    start_tick: Slot,
    stop_tick: Slot,
}

struct TransformSlots {
    position: Slot,
    quaternion: Slot,
    scale: Slot,
}

struct ColliderSlots {
    shape: Slot,
    size: Slot,
    radius: Slot,
    height: Slot,
    offset: Slot,
    quaternion: Slot,
}

/// The records the system reads, with field slots pre-resolved once at enable.
struct Records {
    systems: (SchemaId, Layout, SystemSlots),
    transform: (SchemaId, Layout, TransformSlots),
    parent: Option<(SchemaId, Layout, Slot)>,
    collider: Option<(SchemaId, Layout, ColliderSlots)>,
    event: (SchemaId, Layout),
}

/// Registers og.ParticleEvent's module on a world that names it
/// (`crate::engine::lazy`): the `og.particles` system, after every system scheduled
/// so far, and its state block.
pub(crate) fn enable(world: &mut World, mount: crate::engine::Mount<'_>) -> og_core::Result<()> {
    if world.systems().contains(&SYSTEM) {
        return Ok(());
    }
    let (sys_id, sys_l) = layout(world, SYSTEMS, RecordKind::Component)?;
    let sys_slots = SystemSlots {
        visible: Slot::of(&sys_l, "visible"),
        emitter: Slot::of(&sys_l, "emitter"),
        seed: Slot::of(&sys_l, "seed"),
        start_tick: Slot::of(&sys_l, "startTick"),
        stop_tick: Slot::of(&sys_l, "stopTick"),
    };
    let (tr_id, tr_l) = layout(world, "og.Transform", RecordKind::Component)?;
    let tr_slots = TransformSlots {
        position: Slot::of(&tr_l, "position"),
        quaternion: Slot::of(&tr_l, "quaternion"),
        scale: Slot::of(&tr_l, "scale"),
    };
    let parent = layout(world, "og.Parent", RecordKind::Component)
        .ok()
        .map(|(id, l)| {
            let slot = Slot::of(&l, "entity");
            (id, l, slot)
        });
    let collider = layout(world, "og.Collider", RecordKind::Component)
        .ok()
        .map(|(id, l)| {
            let slots = ColliderSlots {
                shape: Slot::of(&l, "shape"),
                size: Slot::of(&l, "size"),
                radius: Slot::of(&l, "radius"),
                height: Slot::of(&l, "height"),
                offset: Slot::of(&l, "offset"),
                quaternion: Slot::of(&l, "quaternion"),
            };
            (id, l, slots)
        });
    let r = Records {
        systems: (sys_id, sys_l, sys_slots),
        transform: (tr_id, tr_l, tr_slots),
        parent,
        collider,
        event: layout(world, EVENT, RecordKind::Event)?,
    };
    let mut reads = vec![SYSTEMS.to_string(), "og.Transform".to_string()];
    let mut queries = vec![QueryDesc {
        read: vec![SYSTEMS.to_string()],
        ..QueryDesc::default()
    }];
    for (name, present) in [
        ("og.Parent", r.parent.is_some()),
        ("og.Collider", r.collider.is_some()),
    ] {
        if present {
            reads.push(name.to_string());
            queries.push(QueryDesc {
                read: vec![name.to_string()],
                ..QueryDesc::default()
            });
        }
    }
    let state = mount.modules.particles.0.clone();
    lock(&state).systems.clear();
    let sh = state.clone();
    let air = mount.modules.air.clone();
    let fields = mount.modules.forces.clone();
    world.add_system(
        SystemDesc {
            name: SYSTEM.into(),
            reads,
            writes: vec![EVENT.to_string()],
            queries,
            ..Default::default()
        },
        move |ctx| system(ctx, &sh, &air, &fields, &r),
    )?;
    world.add_state_block(SYSTEM, Box::new(Block(state)))
}

#[inline]
fn row_vec3(v: &Row<'_>, s: &Slot, d: Vec3) -> Vec3 {
    Vec3::new(
        v.lane(s, "x").map_or(d.x, |x| x as f32),
        v.lane(s, "y").map_or(d.y, |x| x as f32),
        v.lane(s, "z").map_or(d.z, |x| x as f32),
    )
}

#[inline]
fn row_quat(v: &Row<'_>, s: &Slot) -> Quat {
    Quat::from_xyzw(
        v.lane(s, "x").map_or(0.0, |x| x as f32),
        v.lane(s, "y").map_or(0.0, |x| x as f32),
        v.lane(s, "z").map_or(0.0, |x| x as f32),
        v.lane(s, "w").map_or(1.0, |x| x as f32),
    )
    .normalize()
}

/// Where entities are this tick: og.Transform composed down og.Parent.
struct Places<'a> {
    r: &'a Records,
    parents: IdMap<u64, Entity>,
}

impl Places<'_> {
    fn pose(&self, ctx: &SystemContext<'_>, e: Entity) -> Option<og_core::math::Pose> {
        let bytes = ctx.get_bytes(e, self.r.transform.0).ok()?;
        let row = Row::new(&self.r.transform.1, &bytes);
        let s = &self.r.transform.2;
        Some((
            row_vec3(&row, &s.position, Vec3::ZERO),
            row_quat(&row, &s.quaternion),
            row_vec3(&row, &s.scale, Vec3::ONE),
        ))
    }

    fn world(&self, ctx: &SystemContext<'_>, e: Entity) -> Mat4 {
        let Some(local) = self.pose(ctx, e) else {
            return Mat4::IDENTITY;
        };
        let (p, q, s) = og_core::math::world_pose(
            e,
            local,
            |c| self.parents.get(&c.to_bits()).copied(),
            |p| self.pose(ctx, p),
        );
        Mat4::from_scale_rotation_translation(s, q, p)
    }
}

/// The boxes, spheres and capsules of og.Collider, as the renderer's CPU path
/// places them (`extract::frame::colliders`; a terrain's height
/// field is the renderer's alone).
fn colliders(
    ctx: &SystemContext<'_>,
    places: &Places<'_>,
    query: usize,
) -> og_core::Result<Vec<SceneCollider>> {
    let Some((id, l, s)) = &places.r.collider else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for a in ctx.query(query)? {
        for &e in a.entities() {
            let bytes = ctx.get_bytes(e, *id)?;
            let row = Row::new(l, &bytes);
            let world = places.world(ctx, e);
            let m = world
                * Mat4::from_rotation_translation(
                    row_quat(&row, &s.quaternion),
                    row_vec3(&row, &s.offset, Vec3::ZERO),
                );
            let shape_name = row.text(&s.shape);
            let collider = match shape_name.as_deref() {
                Some("box") => SceneCollider::Box {
                    transform: m,
                    half_extents: row_vec3(&row, &s.size, Vec3::ONE) * 0.5,
                },
                Some("sphere") => SceneCollider::Sphere {
                    center: m.transform_point3(Vec3::ZERO),
                    radius: row.num(&s.radius).map_or(0.5, |x| x as f32)
                        * world.to_scale_rotation_translation().0.max_element(),
                },
                Some("capsule") => {
                    let scale = world.to_scale_rotation_translation().0.max_element();
                    let half = row.num(&s.height).map_or(1.0, |x| x as f32) * 0.5;
                    SceneCollider::Capsule {
                        a: m.transform_point3(Vec3::new(0.0, -half, 0.0)),
                        b: m.transform_point3(Vec3::new(0.0, half, 0.0)),
                        radius: row.num(&s.radius).map_or(0.5, |x| x as f32) * scale,
                    }
                }
                None => match row.uint(&s.shape) {
                    None | Some(0) => SceneCollider::Box {
                        transform: m,
                        half_extents: row_vec3(&row, &s.size, Vec3::ONE) * 0.5,
                    },
                    Some(1) => SceneCollider::Sphere {
                        center: m.transform_point3(Vec3::ZERO),
                        radius: row.num(&s.radius).map_or(0.5, |x| x as f32)
                            * world.to_scale_rotation_translation().0.max_element(),
                    },
                    Some(2) => {
                        let scale = world.to_scale_rotation_translation().0.max_element();
                        let half = row.num(&s.height).map_or(1.0, |x| x as f32) * 0.5;
                        SceneCollider::Capsule {
                            a: m.transform_point3(Vec3::new(0.0, -half, 0.0)),
                            b: m.transform_point3(Vec3::new(0.0, half, 0.0)),
                            radius: row.num(&s.radius).map_or(0.5, |x| x as f32) * scale,
                        }
                    }
                    _ => continue,
                },
                Some(_) => continue,
            };
            out.push(collider);
        }
    }
    Ok(out)
}

fn kind_name(k: ParticleEventKind) -> &'static str {
    match k {
        ParticleEventKind::Birth => "birth",
        ParticleEventKind::Death => "death",
        ParticleEventKind::Collision => "collision",
    }
}

fn system(
    ctx: &mut SystemContext<'_>,
    state: &Mutex<Hosted>,
    air: &crate::bind::wind::ModuleState,
    fields: &crate::bind::forces::ModuleState,
    r: &Records,
) -> og_core::Result<()> {
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    if rows.is_empty() {
        lock(state).systems.clear();
        return Ok(());
    }
    let time = ctx.time();
    let tick_rate = time.rate as f32;
    // The tick this one completes: what a frame drawn after it shows.
    let tick = time.tick + 1;
    let groups = ctx.time_groups();
    let mut next = 1;
    let mut parents = IdMap::new();
    if let Some((id, l, slot)) = &r.parent {
        for a in ctx.query(next)? {
            for &e in a.entities() {
                let bytes = ctx.get_bytes(e, *id)?;
                let p = Row::new(l, &bytes).uint(slot).unwrap_or(0);
                if p != 0 {
                    parents.insert(e.to_bits(), Entity::from_bits(p));
                }
            }
        }
        next += 1;
    }
    let places = Places { r, parents };
    let mut hosted = lock(state);
    let Hosted { emitters, systems } = &mut *hosted;
    let mut live = Vec::new();
    let mut solid: Option<Vec<SceneCollider>> = None;
    let mut events = Vec::new();
    // The tick's force fields (`forces::before_tick`), read once.
    let mut forces = None;
    let (sys_id, sys_l, sys_s) = &r.systems;
    for e in rows {
        let bytes = ctx.get_bytes(e, *sys_id)?;
        let row = Row::new(sys_l, &bytes);
        if !row.flag(&sys_s.visible).unwrap_or(true) {
            continue;
        }
        let emitter_bits = row.uint(&sys_s.emitter).unwrap_or(0);
        let Some(def) =
            crate::world::assets::index_of(emitter_bits as f64).and_then(|i| emitters.get(&i))
        else {
            continue;
        };
        let em = &def.emitter;
        let start = match row.uint(&sys_s.start_tick).unwrap_or(ARMED) {
            ARMED if !em.looping => continue,
            ARMED => 0,
            t => t,
        };
        let stop = row.uint(&sys_s.stop_tick).unwrap_or(0);
        let seed = row.uint(&sys_s.seed).unwrap_or(0);
        let transform = places.world(ctx, e);
        let sys = ParticleSystem {
            id: e.to_bits(),
            emitter: def.handle,
            transform,
            seed,
            start_tick: start,
            stop_tick: (stop > 0).then_some(stop),
            tick,
            time_scale: ctx.group_dt(groups.as_ref(), e, 1.0),
            // The tick's air where the system is (`wind::before_tick`).
            air: air.at(transform.w_axis.truncate()),
            tint: glam::Vec3::ONE,
            emission: 1.0,
            hue: 0.0,
        };
        let colliders: &[SceneCollider] =
            if em.collision.is_some() && em.space == SimulationSpace::World {
                if solid.is_none() {
                    solid = Some(colliders(ctx, &places, next)?);
                }
                solid.as_deref().unwrap_or_default()
            } else {
                &[]
            };
        let subs: Vec<Option<&ParticleEmitter>> = def.subs.iter().map(Option::as_ref).collect();
        let hosted = systems.entry(e.to_bits()).or_default();
        hosted.world = em.space == SimulationSpace::World;
        let cpu = &mut hosted.cpu;
        let mut fired = Vec::new();
        let pull = if em.force_fields > 0.0 && em.space == SimulationSpace::World {
            &*forces.get_or_insert_with(|| fields.get())
        } else {
            &og_env::force::Forces::default()
        };
        cpu.advance(
            em,
            &sys,
            tick_rate,
            &def.points,
            colliders,
            &subs,
            Some(&mut fired),
            None,
            pull,
        );
        events.extend(
            fired
                .into_iter()
                .filter(|ev| em.events.contains(&ev.kind))
                .map(|ev| (e, ev)),
        );
        live.push(e.to_bits());
    }
    // Systems whose entity lost og.ParticleSystem, was hidden or died.
    systems.retain(|k, _| live.contains(k));
    drop(hosted);
    let (id, l) = &r.event;
    for (e, ev) in events {
        let ParticleEvent {
            kind,
            index,
            position,
            velocity,
            ..
        } = ev;
        let value = json!({
            "entity": e.to_bits(),
            "kind": kind_name(kind),
            "index": index,
            "position": { "x": position.x, "y": position.y, "z": position.z },
            "velocity": { "x": velocity.x, "y": velocity.y, "z": velocity.z },
        });
        let bytes = l.encode(None, Some(&value)).map_err(invalid)?;
        ctx.send_bytes(*id, &bytes)?;
    }
    Ok(())
}

// ── the state block ──────────────────────────────────────────────────────────

struct Block(Arc<Mutex<Hosted>>);

fn put_slots(out: &mut Vec<u8>, slots: &[ParticleState]) {
    out.extend_from_slice(&(slots.len() as u32).to_le_bytes());
    for s in slots {
        for v in s.pos_age.iter().chain(&s.vel_life).chain(&s.misc) {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
}

fn save(h: &Hosted, out: &mut Vec<u8>) {
    let u64_ = |out: &mut Vec<u8>, v: u64| out.extend_from_slice(&v.to_le_bytes());
    let f32_ = |out: &mut Vec<u8>, v: f32| out.extend_from_slice(&v.to_le_bytes());
    out.extend_from_slice(&(h.systems.len() as u32).to_le_bytes());
    for (e, HostedSystem { cpu: c, world }) in &h.systems {
        u64_(out, *e);
        out.push(*world as u8);
        put_slots(out, &c.slots);
        u64_(out, c.last_tick.unwrap_or(u64::MAX));
        out.push(c.prev_transform.is_some() as u8);
        if let Some(m) = c.prev_transform {
            for v in m.to_cols_array() {
                f32_(out, v);
            }
        }
        u64_(out, c.clock.tick);
        out.extend_from_slice(&c.clock.time.to_le_bytes());
        f32_(out, c.clock.scale);
        for v in c.origin.to_array() {
            f32_(out, v);
        }
        out.extend_from_slice(&(c.children.len() as u32).to_le_bytes());
        for ch in &c.children {
            put_slots(out, &ch.slots);
            u64_(out, ch.born);
        }
    }
}

fn f4(r: &mut ByteReader<'_>) -> og_core::Result<[f32; 4]> {
    Ok([r.f32()?, r.f32()?, r.f32()?, r.f32()?])
}

fn read_slots(r: &mut ByteReader<'_>) -> og_core::Result<Vec<ParticleState>> {
    (0..r.count(48)?)
        .map(|_| {
            Ok(ParticleState {
                pos_age: f4(r)?,
                vel_life: f4(r)?,
                misc: f4(r)?,
            })
        })
        .collect()
}

fn load(bytes: &[u8]) -> og_core::Result<BTreeMap<u64, HostedSystem>> {
    let mut r = ByteReader::new(bytes, "the og.particles state");
    let mut systems = BTreeMap::new();
    for _ in 0..r.u32()? {
        let e = r.u64()?;
        let world = r.u8()? != 0;
        let slots = read_slots(&mut r)?;
        let last_tick = Some(r.u64()?).filter(|&t| t != u64::MAX);
        let prev_transform = match r.u8()? {
            0 => None,
            _ => {
                let mut m = [0.0; 16];
                for v in &mut m {
                    *v = r.f32()?;
                }
                Some(Mat4::from_cols_array(&m))
            }
        };
        let clock = ParticleClock {
            tick: r.u64()?,
            time: r.f64()?,
            scale: r.f32()?,
        };
        let origin = Vec3::new(r.f32()?, r.f32()?, r.f32()?);
        let mut children = Vec::new();
        for _ in 0..r.u32()? {
            children.push(og_render::ChildParticles {
                slots: read_slots(&mut r)?,
                born: r.u64()?,
            });
        }
        systems.insert(
            e,
            HostedSystem {
                cpu: CpuParticles {
                    slots,
                    last_tick,
                    prev_transform,
                    clock,
                    origin,
                    children,
                    // Drawing only: remade from where the particles are.
                    ..Default::default()
                },
                world,
            },
        );
    }
    if !r.is_done() {
        return Err(invalid("the og.particles state has bytes left over"));
    }
    Ok(systems)
}

impl StateBlock for Block {
    fn save(&self, out: &mut Vec<u8>) {
        save(&lock(&self.0), out);
    }
    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        Ok(Box::new(load(bytes)?))
    }
    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        lock(&self.0).systems = decoded
            .and_then(|d| d.downcast::<BTreeMap<u64, HostedSystem>>().ok())
            .map(|b| *b)
            .unwrap_or_default();
    }

    /// One `emitter` record per hosted og.ParticleSystem entity: its
    /// simulation as saved (space, counts, clock, last tick and transform,
    /// origin, its particles and, per sub-emitter in its emitter's order,
    /// theirs, as value lists). The block saves no generator state and no
    /// other entity (a sub-emitter's particles are its parent's, this
    /// record's), so neither is a field.
    fn form(&self, bytes: &[u8]) -> og_core::Result<bf::BlockForm> {
        let slots = |s: &[ParticleState]| -> Vec<serde_json::Value> {
            s.iter()
                .map(|p| json!({ "posAge": p.pos_age, "velLife": p.vel_life, "misc": p.misc }))
                .collect()
        };
        let mut out = bf::Records::default();
        for (&e, h) in &load(bytes)? {
            let c = &h.cpu;
            let children: Vec<_> = c
                .children
                .iter()
                .map(|ch| json!({ "born": ch.born, "count": ch.slots.len(), "particles": slots(&ch.slots) }))
                .collect();
            out.entities.push(
                bf::Record::new(e, "emitter")
                    .field("world", bf::Field::of(&h.world))
                    .field("count", bf::Field::of(&c.slots.len()))
                    .field("particles", bf::Field::of(&slots(&c.slots)))
                    .field("lastTick", bf::Field::of(&c.last_tick))
                    .field(
                        "prevTransform",
                        bf::Field::of(&c.prev_transform.map(|m| m.to_cols_array())),
                    )
                    .field("tick", bf::Field::of(&c.clock.tick))
                    .field("time", bf::Field::of(&c.clock.time))
                    .field("scale", bf::Field::of(&c.clock.scale))
                    .field("origin", bf::Field::of(&c.origin.to_array()))
                    .field("children", bf::Field::of(&children)),
            );
        }
        Ok(bf::BlockForm::Entities(out))
    }

    /// Particles are places in the world, whatever their emitter's
    /// parenting: a world-space system's own, every sub-emitter's (born
    /// where a parent's event was) and the transform the system last
    /// spawned from move by the whole shift. A local-space system's own
    /// particles are in its entity's frame and stay.
    fn shift(&mut self, s: &og_ecs::Shift) {
        let d = s.d;
        let moved = |slots: &mut [ParticleState]| {
            for p in slots {
                for (c, o) in p.pos_age.iter_mut().zip(d.to_array()) {
                    *c = (f64::from(*c) + o) as f32;
                }
            }
        };
        for h in lock(&self.0).systems.values_mut() {
            if h.world {
                moved(&mut h.cpu.slots);
            }
            for c in &mut h.cpu.children {
                moved(&mut c.slots);
            }
            if let Some(m) = &mut h.cpu.prev_transform {
                m.w_axis += (d.as_vec3()).extend(0.0);
            }
        }
    }
}
