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

use crate::asset::{Behaviour, Root, Value};
use crate::tree::{self, AgentMem, Effects, NodeMem, Senses};
use og_core::glam::{Vec2, Vec3};
use og_core::reflect::FieldType;
use og_core::{Error, ErrorKind, Reflect, Result};
use og_ecs::{Entity, QueryDesc, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use og_nav::Plane;
use og_nav::system::Transform;
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

/// The system's and the state block's name.
pub const SYSTEM_NAME: &str = "og.ai";

/// An agent that thinks with a behaviour (`og.ai.load`).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.AiAgent")]
pub struct AiAgent {
    /// The behaviour it runs (og.ai.load); 0 runs nothing.
    pub behaviour: u64,
    /// Thinks while true.
    #[reflect(default = 1)]
    pub active: bool,
    /// Thinks every this many ticks (1: every tick).
    #[reflect(name = "thinkEvery", min = 1, max = 600, default = 1, unit = "ticks")]
    pub think_every: u32,
    /// AI_RUNNING, AI_SUCCESS or AI_FAILURE after the last think (0 before the first; written by og.ai).
    pub status: u32,
    /// The state machine's state, an index into its states (written by og.ai).
    pub state: u32,
    /// The node it ran last, an index into the behaviour's nodes (written by og.ai).
    pub node: u32,
    /// Times the state or the running node changed (written by og.ai).
    pub changes: u32,
}

impl Default for AiAgent {
    fn default() -> Self {
        AiAgent {
            behaviour: 0,
            active: true,
            think_every: 1,
            status: 0,
            state: 0,
            node: 0,
            changes: 0,
        }
    }
}

/// What an agent sees and hears, and how.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Perception")]
pub struct Perception {
    /// How far it sees.
    #[reflect(min = 0, max = 500, default = 8, unit = "m", tunable)]
    pub sight: f32,
    /// The width of its view cone (360: all round).
    #[reflect(min = 0, max = 360, default = 120, unit = "deg", tunable)]
    pub fov: f32,
    /// Its eyes are this far above its position (+y).
    #[reflect(name = "eyeHeight", min = -10, max = 10, unit = "m")]
    pub eye_height: f32,
    /// Colliders hide what is behind them.
    #[reflect(default = 1)]
    pub occlude: bool,
    /// Collision layers that block sight (0: every layer).
    pub occluders: u32,
    /// How far it hears a sound of loudness 1.
    #[reflect(min = 0, max = 500, default = 6, unit = "m", tunable)]
    pub hearing: f32,
    /// How long a lost target stays its target.
    #[reflect(min = 0, max = 600, default = 3, unit = "s", tunable)]
    pub memory: f32,
    /// Its team: it perceives og.Perceivable entities of other teams.
    pub team: u32,
    /// Where it looks; zero: the way it last moved (before it moves, the way its Transform faces: +Z in 3D, +X in 2D).
    pub forward: Vec3,
    /// What it sees or remembers seeing (written by og.ai).
    pub target: u64,
    /// It sees the target now (written by og.ai).
    pub sees: bool,
    /// Where it last saw the target (written by og.ai).
    #[reflect(name = "lastSeen")]
    pub last_seen: Vec3,
    /// Seconds since it last saw a target (written by og.ai).
    #[reflect(name = "sinceSeen")]
    pub since_seen: f32,
    /// It heard something this tick (written by og.ai).
    pub heard: bool,
    /// Where it last heard something (written by og.ai).
    #[reflect(name = "heardAt")]
    pub heard_at: Vec3,
    /// Seconds since it last heard something (written by og.ai).
    #[reflect(name = "sinceHeard")]
    pub since_heard: f32,
}

impl Default for Perception {
    fn default() -> Self {
        Perception {
            sight: 8.0,
            fov: 120.0,
            eye_height: 0.0,
            occlude: true,
            occluders: 0,
            hearing: 6.0,
            memory: 3.0,
            team: 0,
            forward: Vec3::ZERO,
            target: 0,
            sees: false,
            last_seen: Vec3::ZERO,
            since_seen: NEVER,
            heard: false,
            heard_at: Vec3::ZERO,
            since_heard: NEVER,
        }
    }
}

/// `sinceSeen` and `sinceHeard` before the first time.
pub const NEVER: f32 = 1.0e6;

/// Something agents of other teams can see.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Perceivable")]
pub struct Perceivable {
    /// Its team (agents of the same team ignore it).
    #[reflect(default = 1)]
    pub team: u32,
    /// What agents look at is this far above its position (+y).
    #[reflect(name = "eyeHeight", min = -10, max = 10, unit = "m")]
    pub eye_height: f32,
}

impl Default for Perceivable {
    fn default() -> Self {
        Perceivable {
            team: 1,
            eye_height: 0.0,
        }
    }
}

/// Sent when an agent's state or running node changes (read the tick after).
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.AiChanged")]
pub struct AiChanged {
    /// The agent.
    pub entity: u64,
    /// Its state now.
    pub state: u32,
    /// The node it runs now.
    pub node: u32,
    /// AI_RUNNING, AI_SUCCESS or AI_FAILURE.
    pub status: u32,
}

/// The engine's `og.Parent`, mirrored (same name and layout): a sound
/// made by an agent's child is the agent's own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.Parent")]
pub struct Parent {
    /// The parent.
    pub entity: u64,
}

/// A physics ray for sight: (origin, direction, max distance, layer mask,
/// entity to ignore) → the first hit's distance and entity.
pub type SightRay =
    Arc<dyn Fn(Vec3, Vec3, f32, u32, Entity) -> Option<(f32, Entity)> + Send + Sync>;

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
}

/// A record's fields as packed bytes: name → (type, offset).
#[derive(Clone, Debug)]
struct RecordFields {
    id: SchemaId,
    component: bool,
    fields: BTreeMap<String, (FieldType, usize)>,
}

/// A noise a game made (`og.ai.noise`), heard next tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Noise {
    pub position: Vec3,
    pub loudness: f32,
    /// The entity making it (0: none): it, and the agents it is a child
    /// of, do not hear it.
    pub source: u64,
}

#[derive(Debug, Default)]
struct State {
    mems: BTreeMap<u64, AgentMem>,
    /// The way each agent last moved (perception's forward).
    forward: BTreeMap<u64, Vec3>,
    /// The tick each agent last saw and heard.
    seen_tick: BTreeMap<u64, u64>,
    heard_tick: BTreeMap<u64, u64>,
    noises: Vec<Noise>,
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
    pub entries: BTreeMap<u64, Arc<Behaviour>>,
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
    /// behaviour would: a machine in its initial state (its `enter` run
    /// again), a tree at its root, every timer and cooldown reset, the
    /// blackboard back to the declared keys, the start point (wander's
    /// centre) where it then stands and what it saw and heard forgotten.
    /// Actions without a speed keep the speed it started with. False for an
    /// entity that has not thought yet (its first think starts afresh anyway).
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
    /// `selector > sequence chase > chase`.
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
    world.register::<og_nav::NavAgent>()?;
    world.register::<og_nav::system::Velocity>()?;
    world.register::<AiAgent>()?;
    world.register::<Perception>()?;
    world.register::<Perceivable>()?;
    world.register::<Parent>()?;
    world.register::<AiChanged>()?;
    world.register::<og_audio::schema::SoundStarted>()?;
    world.register::<og_audio::AudioSource>()?;
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
        .write::<og_nav::NavAgent>()
        .read::<og_nav::system::Velocity>()
        .write::<AiChanged>()
        .read::<og_audio::schema::SoundStarted>()
        .read::<og_audio::AudioSource>()
        .query(QueryDesc::new().write::<AiAgent>().read::<Transform>())
        .query(QueryDesc::new().read::<Perceivable>().read::<Transform>());
    for r in reads {
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

/// An entity's position in the world: its og.Transform composed down its
/// og.Parent chain.
fn world_position(ctx: &SystemContext<'_>, e: Entity) -> Vec3 {
    let Ok(t) = ctx.get::<Transform>(e) else {
        return Vec3::ZERO;
    };
    let mut p = t.position;
    let mut at = e;
    for _ in 0..32 {
        let Ok(parent) = ctx.get::<Parent>(at) else {
            break;
        };
        at = Entity::from_bits(parent.entity);
        if !ctx.is_alive(at) {
            break;
        }
        let Ok(pt) = ctx.get::<Transform>(at) else {
            break;
        };
        p = pt.position + pt.quaternion.normalize() * (pt.scale * p);
    }
    p
}

/// `e` and the entities above it by og.Parent (none for 0).
fn lineage(ctx: &SystemContext<'_>, e: u64) -> Vec<u64> {
    let mut out = Vec::new();
    let mut at = e;
    while at != 0 && out.len() < 32 && !out.contains(&at) {
        out.push(at);
        let ent = Entity::from_bits(at);
        if !ctx.is_alive(ent) {
            break;
        }
        at = ctx.get::<Parent>(ent).map_or(0, |p| p.entity);
    }
    out
}

struct Seen {
    e: Entity,
    pos: Vec3,
    team: u32,
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
    for s in ctx.events::<og_audio::schema::SoundStarted>()? {
        let e = Entity::from_bits(s.entity);
        if !ctx.is_alive(e) {
            continue;
        }
        let (Ok(_), Ok(src)) = (ctx.get::<Transform>(e), ctx.get::<og_audio::AudioSource>(e))
        else {
            continue;
        };
        if src.spatial {
            noises.push(Noise {
                position: world_position(ctx, e),
                loudness: src.volume.clamp(0.0, 4.0),
                source: s.entity,
            });
        }
    }
    let makers: Vec<Vec<u64>> = noises.iter().map(|n| lineage(ctx, n.source)).collect();
    if agents.is_empty() {
        return Ok(());
    }
    let alive: std::collections::BTreeSet<u64> = agents.iter().map(|e| e.to_bits()).collect();
    st.mems.retain(|k, _| alive.contains(k));
    let behaviours = lock(&sys.behaviours);
    for (n, &e) in agents.iter().enumerate() {
        let key = e.to_bits();
        let xf = ctx.get::<Transform>(e)?;
        let pos = xf.position;
        let nav = ctx.get::<og_nav::NavAgent>(e).ok();
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
                            Plane::Xz => Vec3::Z,
                            Plane::Xy => Vec3::X,
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
        if !mem.started || mem.behaviour != agent.behaviour || mem.nodes.len() != b.nodes.len() {
            // A first think, another behaviour or a restart: start afresh, at
            // the speed it started with unless this is its first think.
            let speed = if mem.behaviour != 0 {
                mem.base_speed
            } else {
                nav.map_or(3.5, |a| a.speed)
            };
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

// ── the state block ────────────────────────────────────────────────────────

struct AiBlock(AiSystem);

#[derive(Debug)]
struct Saved {
    mems: BTreeMap<u64, AgentMem>,
    forward: BTreeMap<u64, Vec3>,
    seen: BTreeMap<u64, u64>,
    heard: BTreeMap<u64, u64>,
    noises: Vec<Noise>,
}

fn put_u32(o: &mut Vec<u8>, v: u32) {
    o.extend_from_slice(&v.to_le_bytes());
}
fn put_u64(o: &mut Vec<u8>, v: u64) {
    o.extend_from_slice(&v.to_le_bytes());
}
fn put_f32(o: &mut Vec<u8>, v: f32) {
    o.extend_from_slice(&v.to_le_bytes());
}
fn put_v3(o: &mut Vec<u8>, v: Vec3) {
    for x in v.to_array() {
        put_f32(o, x);
    }
}
fn put_str(o: &mut Vec<u8>, s: &str) {
    put_u32(o, s.len() as u32);
    o.extend_from_slice(s.as_bytes());
}
fn put_value(o: &mut Vec<u8>, v: Value) {
    match v {
        Value::Num(n) => {
            o.push(0);
            o.extend_from_slice(&n.to_le_bytes());
        }
        Value::Bool(b) => {
            o.push(1);
            o.push(u8::from(b));
        }
        Value::Vec(v) => {
            o.push(2);
            put_v3(o, v);
        }
        Value::Entity(e) => {
            o.push(3);
            put_u64(o, e);
        }
    }
}

struct Rd<'a> {
    b: &'a [u8],
    at: usize,
}
impl Rd<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        if self.at + n > self.b.len() {
            return Err(Error::new(
                ErrorKind::Invalid,
                "the og.ai state block is truncated",
            ));
        }
        let s = &self.b[self.at..self.at + n];
        self.at += n;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn v3(&mut self) -> Result<Vec3> {
        Ok(Vec3::new(self.f32()?, self.f32()?, self.f32()?))
    }
    fn string(&mut self) -> Result<String> {
        let n = self.u32()? as usize;
        String::from_utf8(self.take(n)?.to_vec())
            .map_err(|_| Error::new(ErrorKind::Invalid, "the og.ai state block has a bad key"))
    }
    fn value(&mut self) -> Result<Value> {
        Ok(match self.u8()? {
            0 => Value::Num(f64::from_le_bytes(self.take(8)?.try_into().unwrap())),
            1 => Value::Bool(self.u8()? != 0),
            2 => Value::Vec(self.v3()?),
            _ => Value::Entity(self.u64()?),
        })
    }
}

impl StateBlock for AiBlock {
    fn save(&self, o: &mut Vec<u8>) {
        let st = lock(&self.0.state);
        put_u32(o, st.mems.len() as u32);
        for (k, m) in &st.mems {
            put_u64(o, *k);
            put_u64(o, m.behaviour);
            o.push(u8::from(m.started));
            o.push(u8::from(m.entered));
            put_u32(o, m.state);
            put_u64(o, m.state_since);
            put_u32(o, m.nodes.len() as u32);
            for n in &m.nodes {
                o.push(u8::from(n.running));
                put_u32(o, n.index);
                put_u32(o, n.count);
                put_u64(o, n.since);
                put_u64(o, n.until);
                put_v3(o, n.point);
            }
            put_u32(o, m.keys.len() as u32);
            for (k, v) in &m.keys {
                put_str(o, k);
                put_value(o, *v);
            }
            put_v3(o, m.home);
            put_f32(o, m.base_speed);
            put_u32(o, m.leaf);
            put_u32(o, m.status);
            put_u32(o, m.path.len() as u32);
            for p in &m.path {
                put_u32(o, *p);
            }
        }
        for map in [&st.seen_tick, &st.heard_tick] {
            put_u32(o, map.len() as u32);
            for (k, v) in map {
                put_u64(o, *k);
                put_u64(o, *v);
            }
        }
        put_u32(o, st.forward.len() as u32);
        for (k, v) in &st.forward {
            put_u64(o, *k);
            put_v3(o, *v);
        }
        put_u32(o, st.noises.len() as u32);
        for n in &st.noises {
            put_v3(o, n.position);
            put_f32(o, n.loudness);
            put_u64(o, n.source);
        }
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        let mut r = Rd { b: bytes, at: 0 };
        let mut mems = BTreeMap::new();
        for _ in 0..r.u32()? {
            let k = r.u64()?;
            let mut m = AgentMem {
                behaviour: r.u64()?,
                started: r.u8()? != 0,
                entered: r.u8()? != 0,
                state: r.u32()?,
                state_since: r.u64()?,
                ..AgentMem::default()
            };
            for _ in 0..r.u32()? {
                m.nodes.push(NodeMem {
                    running: r.u8()? != 0,
                    index: r.u32()?,
                    count: r.u32()?,
                    since: r.u64()?,
                    until: r.u64()?,
                    point: r.v3()?,
                });
            }
            for _ in 0..r.u32()? {
                let key = r.string()?;
                let v = r.value()?;
                m.keys.insert(key, v);
            }
            m.home = r.v3()?;
            m.base_speed = r.f32()?;
            m.leaf = r.u32()?;
            m.status = r.u32()?;
            for _ in 0..r.u32()? {
                m.path.push(r.u32()?);
            }
            mems.insert(k, m);
        }
        let mut maps = [BTreeMap::new(), BTreeMap::new()];
        for map in &mut maps {
            for _ in 0..r.u32()? {
                let k = r.u64()?;
                map.insert(k, r.u64()?);
            }
        }
        let mut forward = BTreeMap::new();
        for _ in 0..r.u32()? {
            let k = r.u64()?;
            forward.insert(k, r.v3()?);
        }
        let mut noises = Vec::new();
        for _ in 0..r.u32()? {
            noises.push(Noise {
                position: r.v3()?,
                loudness: r.f32()?,
                source: r.u64()?,
            });
        }
        let [seen, heard] = maps;
        Ok(Box::new(Saved {
            mems,
            forward,
            seen,
            heard,
            noises,
        }))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut st = lock(&self.0.state);
        match decoded.and_then(|d| d.downcast::<Saved>().ok()) {
            Some(s) => {
                st.mems = s.mems;
                st.forward = s.forward;
                st.seen_tick = s.seen;
                st.heard_tick = s.heard;
                st.noises = s.noises;
            }
            None => {
                st.mems.clear();
                st.forward.clear();
                st.seen_tick.clear();
                st.heard_tick.clear();
                st.noises.clear();
            }
        }
        st.commands.clear();
    }
}
