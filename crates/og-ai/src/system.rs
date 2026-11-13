//! The ECS side: the `og.AiAgent`, `og.Perception` and `og.Perceivable`
//! records, the `og.AiChanged` event, the registry of loaded behaviours and
//! the `og.ai` system, which runs before the game's systems:
//!
//! 1. **Perception**, for every agent with an `og.Perception`: the nearest
//!    `og.Perceivable` of another team within `sight`, inside the `fov`
//!    cone around its forward, not hidden behind a collider (a physics ray
//!    from eye to eye, `occluders` layers); a sound heard when a spatial
//!    source started (`og.SoundStarted`) or a game noise (`og.ai.noise`)
//!    within `hearing` × its loudness. The results are the component's
//!    written fields and the behaviour's built-in keys.
//! 2. **Thinking**, every `thinkEvery` ticks: the behaviour tree or state
//!    machine, with the agent's memory and blackboard.
//! 3. **Acting**: the running action's target and speed go to the agent's
//!    `og.NavAgent` (og.nav plans and moves after the game's systems); no
//!    moving action stops it. Commands are queued for the host to run
//!    before the next tick, in agent order.
//!
//! The memories, blackboards and pending noises are the `og.ai` state
//! block, so snapshots replay an agent's decisions exactly.

mod block;

use crate::decide::agent::{AiActionStarted, AiActionStopped, AiAgent, AiChanged};
use crate::decide::asset::{Behaviour, Root, Value};
use crate::decide::tree::{self, AgentMem, Effects, NodeMem, Senses};
use crate::nav::Plane;
use crate::nav::system::Transform;
use crate::sense::{
    Hearing, NEVER, Noise, Parent, Perceivable, Perception, Seen, SightRay, lineage, world_position,
};
use block::AiBlock;
use og_core::IdMap;
use og_core::glam::{Vec2, Vec3};
use og_core::reflect::{FieldType, Str256};
use og_core::{Error, ErrorKind, Reflect, Result};
use og_ecs::{Entity, QueryDesc, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

/// The system's and the state block's name.
pub const SYSTEM_NAME: &str = "og.ai";

/// Where a think's game commands go as they are made: (command, args).
pub type CommandSink = Arc<dyn Fn(&str, Vec<String>) + Send + Sync>;

/// How og.ai joins a world.
#[derive(Clone)]
pub struct AiDesc {
    pub plane: Plane,
    pub ray: Option<SightRay>,
    /// Commands go here (the host's command channel) instead of the queue
    /// [`AiSystem::take_commands`] answers.
    pub sink: Option<CommandSink>,
    /// The behaviours to run (a hot reload keeps the engine's); a fresh
    /// registry when `None`.
    pub behaviours: Option<Arc<Mutex<Behaviours>>>,
    /// The world's sounds, heard the tick they start; without it agents
    /// hear only the game's noises ([`AiSystem::noise`]).
    pub hearing: Option<Hearing>,
}

/// A record's fields as packed bytes: name → (type, offset).
#[derive(Clone, Debug)]
struct RecordFields {
    id: SchemaId,
    component: bool,
    fields: BTreeMap<String, (FieldType, usize)>,
}

#[derive(Debug, Default)]
struct State {
    mems: IdMap<u64, AgentMem>,
    /// The way each agent last moved (perception's forward).
    forward: IdMap<u64, Vec3>,
    /// The tick each agent last saw and heard.
    seen_tick: IdMap<u64, u64>,
    heard_tick: IdMap<u64, u64>,
    noises: Vec<Noise>,
    /// How far og.WorldOrigin has moved the world's zero (R12): what the
    /// behaviours' authored points are offset by.
    origin: Vec3,
    commands: Vec<(String, Vec<String>)>,
    warnings: Vec<String>,
    warned: std::collections::BTreeSet<String>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// The behaviours an engine loaded, by handle.
#[derive(Debug, Default)]
pub struct Behaviours {
    pub entries: IdMap<u64, Arc<Behaviour>>,
    next: u64,
}

impl Behaviours {
    pub fn insert(&mut self, b: Behaviour) -> u64 {
        self.next += 1;
        let h = self.next | (1u64 << 32);
        self.entries.insert(h, Arc::new(b));
        h
    }
    pub fn get(&self, h: u64) -> Option<Arc<Behaviour>> {
        self.entries.get(&h).cloned()
    }
}

/// og.ai on one world.
#[derive(Clone)]
pub struct AiSystem {
    pub behaviours: Arc<Mutex<Behaviours>>,
    state: Arc<Mutex<State>>,
}

impl AiSystem {
    /// Queues a noise heard by agents next tick.
    pub fn noise(&self, n: Noise) {
        lock(&self.state).noises.push(n);
    }

    /// Starts an agent's behaviour afresh at its next think, as a new
    /// behaviour would: a machine in its initial state (its `entry` run
    /// again), a tree at its root, every timer and cooldown reset, the
    /// blackboard back to the declared keys, the start point (wander's
    /// centre) where it then stands and what it saw and heard forgotten.
    /// Actions without a speed keep the speed it started with. False for an
    /// entity that has not thought yet (its first think starts afresh anyway).
    /// Ends an agent's running game actions (or only those named `action`)
    /// with `success`: each answers at the agent's next think. False when
    /// none is running.
    pub fn finish(&self, e: Entity, success: bool, action: Option<&str>) -> bool {
        let mut st = lock(&self.state);
        let Some(m) = st.mems.get_mut(&e.to_bits()) else {
            return false;
        };
        let Some(b) = lock(&self.behaviours).get(m.behaviour) else {
            return false;
        };
        let mut any = false;
        for (node, name) in tree::running_actions(&b, m) {
            if action.is_none_or(|a| a == name) {
                m.results.insert(node, success);
                any = true;
            }
        }
        any
    }

    pub fn restart(&self, e: Entity) -> bool {
        let mut st = lock(&self.state);
        let key = e.to_bits();
        st.seen_tick.remove(&key);
        st.heard_tick.remove(&key);
        match st.mems.get_mut(&key) {
            Some(m) => {
                m.started = false;
                true
            }
            None => false,
        }
    }

    /// The commands the last ticks queued, in order: run them before the next tick.
    pub fn take_commands(&self) -> Vec<(String, Vec<String>)> {
        std::mem::take(&mut lock(&self.state).commands)
    }

    /// Warnings since the last call (each once per engine), naming the fix.
    pub fn take_warnings(&self) -> Vec<String> {
        std::mem::take(&mut lock(&self.state).warnings)
    }

    /// A blackboard value of an agent (declared, set or built in as of its last think).
    pub fn get(&self, e: Entity, key: &str) -> Option<Value> {
        lock(&self.state)
            .mems
            .get(&e.to_bits())
            .and_then(|m| m.keys.get(key).copied())
    }

    /// Sets a blackboard value; false for an agent that has not thought yet.
    pub fn set(&self, e: Entity, key: &str, value: Value) -> bool {
        match lock(&self.state).mems.get_mut(&e.to_bits()) {
            Some(m) => {
                m.keys.insert(key.to_string(), value);
                true
            }
            None => false,
        }
    }

    /// What an agent is doing, for the probe's dumps: its behaviour, state,
    /// the path to its running node, status and blackboard.
    pub fn describe(&self, e: Entity) -> Option<Description> {
        let st = lock(&self.state);
        let mem = st.mems.get(&e.to_bits())?;
        let _ = mem;
        Some(Description { mem: mem.clone() })
    }

    /// The memory of an agent, with its behaviour for names.
    pub fn memory(&self, e: Entity) -> Option<AgentMem> {
        lock(&self.state).mems.get(&e.to_bits()).cloned()
    }
}

/// An agent's memory, as [`AiSystem::describe`] answers it.
#[derive(Clone, Debug)]
pub struct Description {
    pub mem: AgentMem,
}

impl Description {
    /// The state's name (a machine) or "" (a tree).
    pub fn state_name(&self, b: &Behaviour) -> String {
        match &b.root {
            Root::Machine { states, .. } => states
                .get(self.mem.state as usize)
                .map_or(String::new(), |s| s.name.clone()),
            Root::Tree(_) => String::new(),
        }
    }
    /// `ReactiveFallback > ReactiveSequence chase > chase`.
    pub fn path(&self, b: &Behaviour) -> String {
        self.mem
            .path
            .iter()
            .map(|&n| b.node_name(n as usize))
            .collect::<Vec<_>>()
            .join(" > ")
    }
}

/// Registers the records, adds `og.ai` (call it before the game's
/// systems) and its state block. `records` are every component and
/// resource name the world has, which checks may read.
pub fn install(world: &mut World, records: &[String], desc: AiDesc) -> Result<AiSystem> {
    world.register::<Transform>()?;
    world.register::<crate::nav::NavAgent>()?;
    world.register::<crate::nav::system::Velocity>()?;
    world.register::<AiAgent>()?;
    world.register::<Perception>()?;
    world.register::<Perceivable>()?;
    world.register::<Parent>()?;
    world.register::<AiChanged>()?;
    let mut table: BTreeMap<String, RecordFields> = BTreeMap::new();
    let mut reads: Vec<String> = Vec::new();
    for name in records {
        let Ok(id) = world.id_by_name(name) else {
            continue;
        };
        let Ok(schema) = world.schema(id) else {
            continue;
        };
        let component = !world.has_resource_id(id);
        let mut offset = 0;
        let mut fields = BTreeMap::new();
        for f in &schema.fields {
            fields.insert(f.name.clone(), (f.ty, offset));
            offset += f.ty.size();
        }
        table.insert(
            name.clone(),
            RecordFields {
                id,
                component,
                fields,
            },
        );
        reads.push(name.clone());
    }
    let sys = AiSystem {
        behaviours: desc.behaviours.clone().unwrap_or_default(),
        state: Arc::new(Mutex::new(State::default())),
    };
    let run_sys = sys.clone();
    let mut sd = SystemDesc::new(SYSTEM_NAME)
        .write::<AiAgent>()
        .write::<Perception>()
        .read::<Perceivable>()
        .read::<Parent>()
        .read::<Transform>()
        .write::<crate::nav::NavAgent>()
        .read::<crate::nav::system::Velocity>()
        .write::<AiChanged>()
        .query(QueryDesc::new().write::<AiAgent>().read::<Transform>())
        .query(QueryDesc::new().read::<Perceivable>().read::<Transform>());
    let heard = desc.hearing.iter().flat_map(|h| h.reads.iter().cloned());
    for r in heard.chain(reads) {
        if !sd.reads.contains(&r) && !sd.writes.contains(&r) {
            sd.reads.push(r);
        }
    }
    let table = Arc::new(table);
    world.add_system(sd, move |ctx| run(ctx, &run_sys, &desc, &table))?;
    world.add_state_block(SYSTEM_NAME, Box::new(AiBlock(sys.clone())))?;
    Ok(sys)
}

fn read_field(
    ctx: &SystemContext<'_>,
    table: &BTreeMap<String, RecordFields>,
    e: Entity,
    operand: &str,
) -> Option<Value> {
    let (rec, rest) = operand.split_once('.')?;
    let r = table.get(rec)?;
    let (field, lane) = match rest.rsplit_once('.') {
        Some((f, l)) if matches!(l, "x" | "y" | "z" | "w") => (f, Some(l)),
        _ => (rest, None),
    };
    let &(ty, offset) = r.fields.get(field)?;
    let bytes = if r.component {
        ctx.get_bytes(e, r.id).ok()?
    } else {
        ctx.resource_bytes(r.id).ok()?
    };
    if offset + ty.size() > bytes.len() {
        return None;
    }
    let f32_at = |o: usize| f32::from_le_bytes(bytes[o..o + 4].try_into().unwrap());
    let lane_i = match lane {
        Some("x") | None => 0,
        Some("y") => 1,
        Some("z") => 2,
        _ => 3,
    };
    Some(match ty {
        FieldType::F32 => Value::Num(f32_at(offset) as f64),
        FieldType::F64 => Value::Num(f64::from_le_bytes(
            bytes[offset..offset + 8].try_into().unwrap(),
        )),
        FieldType::I32 => {
            Value::Num(i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as f64)
        }
        FieldType::U32 | FieldType::Enum => {
            Value::Num(u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as f64)
        }
        FieldType::I64 => {
            Value::Num(i64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) as f64)
        }
        FieldType::U64 => Value::Entity(u64::from_le_bytes(
            bytes[offset..offset + 8].try_into().unwrap(),
        )),
        FieldType::Bool => Value::Bool(bytes[offset] != 0),
        FieldType::Vec2 | FieldType::Vec3 | FieldType::Vec4 | FieldType::Quat => {
            if lane.is_some() {
                Value::Num(f32_at(offset + lane_i * 4) as f64)
            } else {
                let n = ty.lanes().min(3);
                let mut v = [0.0f32; 3];
                for (k, x) in v.iter_mut().enumerate().take(n) {
                    *x = f32_at(offset + k * 4);
                }
                Value::Vec(Vec3::from_array(v))
            }
        }
        _ => return None,
    })
}

fn run(
    ctx: &mut SystemContext<'_>,
    sys: &AiSystem,
    desc: &AiDesc,
    table: &BTreeMap<String, RecordFields>,
) -> Result<()> {
    let plane = desc.plane;
    let time = ctx.time();
    let tick = time.tick;
    // og.AiActionStarted and og.AiActionStopped are sent only once a
    // behaviour with a game action registered them.
    let actions = ctx.id_by_name(AiActionStarted::NAME).is_ok()
        && ctx.id_by_name(AiActionStopped::NAME).is_ok();
    let rate = if time.dt > 0.0 { 1.0 / time.dt } else { 60.0 };
    let mut agents: Vec<Entity> = Vec::new();
    for a in ctx.query(0)? {
        agents.extend_from_slice(a.entities());
    }
    let mut targets: Vec<Seen> = Vec::new();
    for a in ctx.query(1)? {
        for &e in a.entities() {
            let p = ctx.get::<Perceivable>(e)?;
            let t = ctx.get::<Transform>(e)?;
            targets.push(Seen {
                e,
                pos: t.position + Vec3::Y * p.eye_height,
                team: p.team,
            });
        }
    }
    let mut st = lock(&sys.state);
    // Sounds: spatial sources that started (delivered this tick), and game
    // noises; each with the entities that do not hear it (its source and
    // the source's parents).
    let mut noises = std::mem::take(&mut st.noises);
    if let Some(hearing) = &desc.hearing {
        for (e, loudness) in (hearing.sounds)(ctx)? {
            if !ctx.is_alive(e) || ctx.get::<Transform>(e).is_err() {
                continue;
            }
            noises.push(Noise {
                position: world_position(ctx, e),
                loudness: loudness.clamp(0.0, 4.0),
                source: e.to_bits(),
            });
        }
    }
    let makers: Vec<Vec<u64>> = noises.iter().map(|n| lineage(ctx, n.source)).collect();
    if agents.is_empty() {
        return Ok(());
    }
    let alive: std::collections::BTreeSet<u64> =
        og_core::sort::set(agents.iter().map(|e| e.to_bits()));
    st.mems.retain(|k, _| alive.contains(k));
    let behaviours = lock(&sys.behaviours);
    for (n, &e) in agents.iter().enumerate() {
        let key = e.to_bits();
        let xf = ctx.get::<Transform>(e)?;
        let pos = xf.position;
        let nav = ctx.get::<crate::nav::NavAgent>(e).ok();
        // 1. Perception.
        if let Ok(mut p) = ctx.get::<Perception>(e) {
            let eye = pos + Vec3::Y * p.eye_height;
            let moving = nav
                .map(|a| plane.to_plane(a.velocity))
                .filter(|v| v.length_squared() > 0.0025);
            let fwd2 = if p.forward != Vec3::ZERO {
                plane.to_plane(p.forward)
            } else if let Some(v) = moving {
                st.forward.insert(key, plane.to_world(v, 0.0));
                v
            } else {
                // The way it last moved; before it ever has, the way it faces
                // (og.nav's faceMovement convention: +Z in 3D, +X in 2D).
                st.forward.get(&key).map_or_else(
                    || {
                        let axis = match plane {
                            Plane::Xz => og_core::facing::FORWARD,
                            Plane::Xy => og_core::facing::FORWARD_2D,
                        };
                        plane.to_plane(xf.quaternion.normalize() * axis)
                    },
                    |f| plane.to_plane(*f),
                )
            };
            let cos_half = og_core::math::cos((p.fov.clamp(0.0, 360.0) * 0.5).to_radians());
            let mut best: Option<(f32, Entity, Vec3)> = None;
            for t in &targets {
                if t.e == e || t.team == p.team {
                    continue;
                }
                let d = t.pos - eye;
                let dist = d.length();
                if dist > p.sight {
                    continue;
                }
                if p.fov < 360.0 && fwd2 != Vec2::ZERO && dist > 1e-4 {
                    let dir = plane.to_plane(d).normalize_or_zero();
                    if dir.dot(fwd2.normalize()) < cos_half {
                        continue;
                    }
                }
                if p.occlude
                    && dist > 1e-4
                    && let Some(ray) = &desc.ray
                    && let Some((hd, he)) = ray(eye, d / dist, dist, p.occluders, e)
                    && he != t.e
                    && hd < dist - 0.05
                {
                    continue;
                }
                if best.is_none_or(|(bd, be, _)| {
                    dist < bd || (dist == bd && t.e.to_bits() < be.to_bits())
                }) {
                    best = Some((dist, t.e, t.pos));
                }
            }
            match best {
                Some((_, te, tp)) => {
                    p.sees = true;
                    p.target = te.to_bits();
                    p.last_seen = tp;
                    st.seen_tick.insert(key, tick);
                }
                None => {
                    p.sees = false;
                    let since = st
                        .seen_tick
                        .get(&key)
                        .map_or(NEVER, |t| (tick - t) as f32 / rate);
                    if since > p.memory {
                        p.target = 0;
                    }
                }
            }
            p.since_seen = if p.sees {
                0.0
            } else {
                st.seen_tick
                    .get(&key)
                    .map_or(NEVER, |t| (tick - t) as f32 / rate)
            };
            p.heard = false;
            let mut loudest: Option<(f32, Vec3)> = None;
            for (nz, makers) in noises.iter().zip(&makers) {
                if makers.contains(&key) {
                    continue;
                }
                let d = nz.position.distance(pos);
                if d <= p.hearing * nz.loudness && loudest.is_none_or(|(ld, _)| d < ld) {
                    loudest = Some((d, nz.position));
                }
            }
            if let Some((_, at)) = loudest {
                p.heard = true;
                p.heard_at = at;
                st.heard_tick.insert(key, tick);
            }
            p.since_heard = if p.heard {
                0.0
            } else {
                st.heard_tick
                    .get(&key)
                    .map_or(NEVER, |t| (tick - t) as f32 / rate)
            };
            ctx.set(e, p)?;
        }
        // 2. Thinking.
        let mut agent = ctx.get::<AiAgent>(e)?;
        if !agent.active || agent.behaviour == 0 {
            continue;
        }
        let Some(b) = behaviours.get(agent.behaviour) else {
            let w = format!(
                "og.AiAgent.behaviour {} is not a loaded behaviour: load one with og.ai.load",
                agent.behaviour
            );
            if st.warned.insert(w.clone()) {
                st.warnings.push(w);
            }
            continue;
        };
        let every = agent.think_every.max(1) as u64;
        if !(tick + n as u64).is_multiple_of(every) {
            continue;
        }
        let perception = ctx.get::<Perception>(e).ok();
        let mut mem = st.mems.remove(&key).unwrap_or_default();
        let origin = st.origin;
        let mut interrupted: Vec<(u32, String)> = Vec::new();
        if !mem.started || mem.behaviour != agent.behaviour || mem.nodes.len() != b.nodes.len() {
            // A first think, another behaviour or a restart: start afresh, at
            // the speed it started with unless this is its first think.
            let speed = if mem.behaviour != 0 {
                mem.base_speed
            } else {
                nav.map_or(3.5, |a| a.speed)
            };
            // Game actions the old memory was running stop, interrupted.
            if let Some(old) = behaviours.get(mem.behaviour) {
                for (node, name) in tree::running_actions(&old, &mem) {
                    interrupted.push((node, name));
                }
            }
            mem = AgentMem {
                behaviour: agent.behaviour,
                ..AgentMem::default()
            };
            tree::start(&b, &mut mem, pos, speed, tick);
        }
        let locate = |h: u64| -> Option<Vec3> {
            let t = Entity::from_bits(h);
            if !ctx.is_alive(t) {
                return None;
            }
            ctx.get::<Transform>(t).ok().map(|t| t.position)
        };
        let elapsed = (tick - mem.state_since) as f64 / rate as f64;
        let resolve = |k: &str, mem: &AgentMem| -> Option<Value> {
            let p = perception;
            let target_pos = p.and_then(|p| {
                if p.target != 0 {
                    locate(p.target).filter(|_| p.sees).or(Some(p.last_seen))
                } else {
                    None
                }
            });
            match k {
                "self" => Some(Value::Entity(key)),
                "position" => Some(Value::Vec(pos)),
                "sees" => Some(Value::Bool(p.is_some_and(|p| p.sees))),
                "heard" => Some(Value::Bool(p.is_some_and(|p| p.heard))),
                "target" => Some(match p {
                    Some(p) if p.target != 0 => Value::Entity(p.target),
                    _ => Value::Entity(0),
                }),
                "targetPosition" => Some(Value::Vec(target_pos.unwrap_or(pos))),
                "targetDistance" => Some(Value::Num(
                    target_pos.map_or(NEVER as f64, |t| t.distance(pos) as f64),
                )),
                "lastSeen" => Some(Value::Vec(p.map_or(pos, |p| p.last_seen))),
                "sinceSeen" => Some(Value::Num(p.map_or(NEVER, |p| p.since_seen) as f64)),
                "heardAt" => Some(Value::Vec(p.map_or(pos, |p| p.heard_at))),
                "sinceHeard" => Some(Value::Num(p.map_or(NEVER, |p| p.since_heard) as f64)),
                "elapsed" => Some(Value::Num(elapsed)),
                _ => mem
                    .keys
                    .get(k)
                    .copied()
                    .or_else(|| read_field(ctx, table, e, k)),
            }
        };
        let senses = Senses {
            plane,
            tick,
            rate,
            entity: key,
            position: pos,
            nav_status: nav.map_or(0, |a| a.status),
            nav_target: nav.map_or(Vec3::ZERO, |a| a.target),
            stop_distance: nav.map_or(0.1, |a| a.stop_distance),
            origin,
            resolve: &resolve,
            locate: &locate,
        };
        let mut fx = Effects::default();
        let before = (mem.state, mem.leaf);
        let status = tree::think(&b, &mut mem, &senses, &mut fx);
        // Built-in keys as of this think, for game code and dumps.
        for k in crate::asset::BUILTIN_KEYS {
            if let Some(v) = resolve(k, &mem) {
                mem.keys.insert((*k).to_string(), v);
            }
        }
        // 3. Acting.
        if let Some(mut a) = nav {
            match fx.motion.target {
                Some(t) => {
                    a.target = t;
                    a.seeking = true;
                    if let Some(s) = fx.motion.speed {
                        a.speed = s;
                    }
                }
                None => a.seeking = false,
            }
            ctx.set(e, a)?;
        }
        agent.status = status.code();
        agent.state = mem.state;
        agent.node = mem.leaf;
        if (mem.state, mem.leaf) != before {
            agent.changes += 1;
            ctx.send(AiChanged {
                entity: key,
                state: mem.state,
                node: mem.leaf,
                status: agent.status,
            })?;
        }
        ctx.set(e, agent)?;
        // Game actions: what stopped (a restart's first), then what started.
        if actions {
            // The parse bounds a name under 64 bytes and the arguments
            // under 256, so nothing is cut.
            let text = |t: &str| Str256::new(t).unwrap_or_default();
            let stops = interrupted
                .into_iter()
                .map(|(n, name)| (n, name, tree::AI_INTERRUPTED))
                .chain(std::mem::take(&mut fx.stopped));
            for (node, name, result) in stops {
                ctx.send(AiActionStopped {
                    entity: key,
                    node,
                    name: text(&name),
                    result,
                })?;
            }
            for (node, name, args) in std::mem::take(&mut fx.started) {
                ctx.send(AiActionStarted {
                    entity: key,
                    node,
                    name: text(&name),
                    args: text(&args.join(" ")),
                })?;
            }
        }
        match &desc.sink {
            Some(sink) => {
                for (c, a) in fx.commands {
                    sink(&c, a);
                }
            }
            None => st.commands.extend(fx.commands),
        }
        for w in fx.warnings {
            if st.warned.insert(w.clone()) {
                st.warnings.push(w);
            }
        }
        st.mems.insert(key, mem);
    }
    Ok(())
}
