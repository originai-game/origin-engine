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
//! og.ParticleEvent is registered lazily (`super::lazy`): a world that
//! never names it has neither the system nor the state block, and its
//! particle systems are the renderer's alone, as before.

use super::codec::{Layout, RecordKind};
use super::wire::record;
use glam::{Mat4, Vec3};
use og_core::{Error, ErrorKind};
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
/// og.ParticleSystem.startTick's unwritten value (`fx::PARTICLES_ARMED`).
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

/// The records the system reads.
struct Records {
    systems: (SchemaId, Layout),
    transform: (SchemaId, Layout),
    parent: Option<(SchemaId, Layout)>,
    collider: Option<(SchemaId, Layout)>,
    event: (SchemaId, Layout),
}

/// Registers og.ParticleEvent's module on a world that names it
/// (`super::lazy`): the `og.particles` system, after every system scheduled
/// so far, and its state block.
pub(crate) fn enable(world: &mut World, mount: crate::engine::Mount<'_>) -> og_core::Result<()> {
    if world.systems().contains(&SYSTEM) {
        return Ok(());
    }
    let r = Records {
        systems: layout(world, SYSTEMS, RecordKind::Component)?,
        transform: layout(world, "og.Transform", RecordKind::Component)?,
        parent: layout(world, "og.Parent", RecordKind::Component).ok(),
        collider: layout(world, "og.Collider", RecordKind::Component).ok(),
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
    world.add_system(
        SystemDesc {
            name: SYSTEM.into(),
            reads,
            writes: vec![EVENT.to_string()],
            queries,
            ..Default::default()
        },
        move |ctx| system(ctx, &sh, &air, &r),
    )?;
    world.add_state_block(SYSTEM, Box::new(Block(state)))
}

/// Where entities are this tick: og.Transform composed down og.Parent.
struct Places<'a> {
    r: &'a Records,
    parents: BTreeMap<u64, Entity>,
}

impl Places<'_> {
    fn pose(&self, ctx: &SystemContext<'_>, e: Entity) -> Option<og_core::math::Pose> {
        let t: record::Transform = self
            .r
            .transform
            .1
            .read(&ctx.get_bytes(e, self.r.transform.0).ok()?)
            .ok()?;
        Some((t.position, t.quaternion.normalize(), t.scale))
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

/// The boxes and spheres of og.Collider, as the renderer's CPU path
/// places them (`extract::frame::colliders`; a terrain's height
/// field is the renderer's alone).
fn colliders(
    ctx: &SystemContext<'_>,
    places: &Places<'_>,
    query: usize,
) -> og_core::Result<Vec<SceneCollider>> {
    let Some((id, l)) = &places.r.collider else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for a in ctx.query(query)? {
        for &e in a.entities() {
            let v: record::Collider = l.read(&ctx.get_bytes(e, *id)?).map_err(invalid)?;
            let world = places.world(ctx, e);
            let m = world * Mat4::from_rotation_translation(v.rotation.normalize(), v.offset);
            out.push(match v.shape {
                0 => SceneCollider::Box {
                    transform: m,
                    half_extents: v.size * 0.5,
                },
                1 => SceneCollider::Sphere {
                    center: m.transform_point3(Vec3::ZERO),
                    radius: v.radius * world.to_scale_rotation_translation().0.max_element(),
                },
                _ => continue,
            });
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
    air: &crate::wind::ModuleState,
    r: &Records,
) -> og_core::Result<()> {
    let time = ctx.time();
    let tick_rate = if time.dt > 0.0 { 1.0 / time.dt } else { 60.0 };
    // The tick this one completes: what a frame drawn after it shows.
    let tick = time.tick + 1;
    let groups = ctx.time_groups();
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    let mut next = 1;
    let mut parents = BTreeMap::new();
    if let Some((id, l)) = &r.parent {
        for a in ctx.query(next)? {
            for &e in a.entities() {
                let v: record::Parent = l.read(&ctx.get_bytes(e, *id)?).map_err(invalid)?;
                let p = v.entity;
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
    for e in rows {
        let v: record::ParticleSystem = r
            .systems
            .1
            .read(&ctx.get_bytes(e, r.systems.0)?)
            .map_err(invalid)?;
        if !v.visible {
            continue;
        }
        let Some(def) = super::assets::index_of(v.emitter as f64).and_then(|i| emitters.get(&i))
        else {
            continue;
        };
        let em = &def.emitter;
        let start = match u64::from(v.start_tick) {
            ARMED if !em.looping => continue,
            ARMED => 0,
            t => t,
        };
        let stop = u64::from(v.stop_tick);
        let transform = places.world(ctx, e);
        let sys = ParticleSystem {
            id: e.to_bits(),
            emitter: def.handle,
            transform,
            seed: u64::from(v.seed),
            start_tick: start,
            stop_tick: (stop > 0).then_some(stop),
            tick,
            time_scale: ctx.group_dt(groups.as_ref(), e, 1.0),
            // The tick's air where the system is (`wind::before_tick`).
            air: air.at(transform.w_axis.truncate()),
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
        cpu.advance(
            em,
            &sys,
            tick_rate,
            &def.points,
            colliders,
            &subs,
            Some(&mut fired),
            None,
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

struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> og_core::Result<[u8; N]> {
        let (a, b) = self
            .0
            .split_first_chunk::<N>()
            .ok_or_else(|| invalid("the og.particles state is cut short"))?;
        self.0 = b;
        Ok(*a)
    }
    fn u32(&mut self) -> og_core::Result<u32> {
        self.take::<4>().map(u32::from_le_bytes)
    }
    fn u64(&mut self) -> og_core::Result<u64> {
        self.take::<8>().map(u64::from_le_bytes)
    }
    fn f32(&mut self) -> og_core::Result<f32> {
        self.take::<4>().map(f32::from_le_bytes)
    }
    fn f4(&mut self) -> og_core::Result<[f32; 4]> {
        Ok([self.f32()?, self.f32()?, self.f32()?, self.f32()?])
    }
    fn slots(&mut self) -> og_core::Result<Vec<ParticleState>> {
        let n = self.u32()? as usize;
        if n > self.0.len() / 48 {
            return Err(invalid("the og.particles state is cut short"));
        }
        (0..n)
            .map(|_| {
                Ok(ParticleState {
                    pos_age: self.f4()?,
                    vel_life: self.f4()?,
                    misc: self.f4()?,
                })
            })
            .collect()
    }
}

fn load(bytes: &[u8]) -> og_core::Result<BTreeMap<u64, HostedSystem>> {
    let mut r = Reader(bytes);
    let mut systems = BTreeMap::new();
    for _ in 0..r.u32()? {
        let e = r.u64()?;
        let world = r.take::<1>()?[0] != 0;
        let slots = r.slots()?;
        let last_tick = Some(r.u64()?).filter(|&t| t != u64::MAX);
        let prev_transform = match r.take::<1>()?[0] {
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
            time: f64::from_le_bytes(r.take::<8>()?),
            scale: r.f32()?,
        };
        let origin = Vec3::new(r.f32()?, r.f32()?, r.f32()?);
        let mut children = Vec::new();
        for _ in 0..r.u32()? {
            children.push(og_render::ChildParticles {
                slots: r.slots()?,
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
                },
                world,
            },
        );
    }
    if !r.0.is_empty() {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A hosted system's whole simulation survives the state block: a
    /// restored world steps on to the same particles.
    #[test]
    fn the_state_block_round_trips() {
        let mut h = Hosted::default();
        let mut c = CpuParticles {
            last_tick: Some(41),
            prev_transform: Some(Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0))),
            clock: ParticleClock {
                tick: 7,
                time: 0.125,
                scale: 0.5,
            },
            origin: Vec3::new(4.0, 5.0, 6.0),
            ..CpuParticles::default()
        };
        c.slots = vec![
            ParticleState {
                pos_age: [1.0, 2.0, 3.0, 0.5],
                vel_life: [0.0, 1.0, 0.0, 2.0],
                misc: [0.3, 0.0, 1.0, 0.9],
            };
            3
        ];
        c.children.push(og_render::ChildParticles {
            slots: c.slots[..1].to_vec(),
            born: 9,
        });
        h.systems.insert(
            77,
            HostedSystem {
                cpu: c.clone(),
                world: true,
            },
        );
        h.systems.insert(5, HostedSystem::default());
        let mut bytes = Vec::new();
        save(&h, &mut bytes);
        let back = load(&bytes).unwrap();
        assert!(back[&77].world && !back[&5].world);
        let got = &back[&77].cpu;
        assert_eq!(got.slots, c.slots);
        assert_eq!(got.last_tick, Some(41));
        assert_eq!(got.prev_transform, c.prev_transform);
        assert_eq!(got.clock, c.clock);
        assert_eq!(got.origin, c.origin);
        assert_eq!(got.children.len(), 1);
        assert_eq!(got.children[0].born, 9);
        assert_eq!(back[&5].cpu.last_tick, None);
        assert!(load(&bytes[..bytes.len() - 1]).is_err());
    }

    /// An origin shift moves a world-space system's particles, every
    /// sub-emitter's and its last transform; a local-space system's own
    /// particles stay in its entity's frame.
    #[test]
    fn an_origin_shift_moves_the_world_space_particles() {
        let slot = ParticleState {
            pos_age: [1.0, 2.0, 3.0, 0.1],
            vel_life: [0.0, 0.0, 0.0, 1.0],
            misc: [0.1; 4],
        };
        let cpu = CpuParticles {
            slots: vec![slot],
            prev_transform: Some(Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0))),
            children: vec![og_render::ChildParticles {
                slots: vec![slot],
                born: 1,
            }],
            ..CpuParticles::default()
        };
        let h = Arc::new(Mutex::new(Hosted::default()));
        for (e, world) in [(1, true), (2, false)] {
            lock(&h).systems.insert(
                e,
                HostedSystem {
                    cpu: cpu.clone(),
                    world,
                },
            );
        }
        let mut block = Block(h.clone());
        block.shift(&og_ecs::Shift::new(
            glam::DVec3::new(-1024.0, 0.0, 512.0),
            vec![2],
        ));
        let g = lock(&h);
        let at = |s: &[ParticleState]| Vec3::from_slice(&s[0].pos_age[..3]);
        let moved = Vec3::new(-1023.0, 2.0, 515.0);
        assert_eq!(at(&g.systems[&1].cpu.slots), moved);
        assert_eq!(at(&g.systems[&2].cpu.slots), Vec3::new(1.0, 2.0, 3.0));
        for e in [1, 2] {
            let c = &g.systems[&e].cpu;
            assert_eq!(at(&c.children[0].slots), moved, "children are in the world");
            assert_eq!(
                c.prev_transform.unwrap().w_axis.truncate(),
                Vec3::new(-1023.0, 0.0, 512.0)
            );
        }
    }
}
