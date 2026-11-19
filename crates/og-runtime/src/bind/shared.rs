//! Shared worlds (IDL `shared`, docs/design.md §2.12) on the runtime, over
//! og-shared's protocol and client view.
//!
//! The engine opens no socket. It queues ops for its host (`og_shared_take`:
//! connect, send, close) and takes the host's events back
//! (`og_shared_deliver`: what carries sessions, open, message, closed,
//! refused): natively the napi addon's client (`og run --shared`), on the
//! web the engine Worker's WebSocket, both given their endpoint and token by
//! a session provider. Every event is journaled with the tick it was handed
//! at (og.platform.journal, kind `shared`, without the token) and applied
//! by the `og.shared` system as the next tick starts, so a session is host
//! input like a key press and replays without the server.
//!
//! The system, each tick: it applies what arrived (the welcome, remote
//! avatars entering and leaving as og.RemoteAvatar entities, snapshots,
//! corrections, events), reconnects a lost connection, draws every remote
//! avatar `interpDelayMs` behind the newest server time between the two
//! states around it (extrapolated briefly past the newest), and proposes
//! this client's og.SharedAvatar at og.SharedWorld.sendRateHz. Nothing reads
//! the wall clock: the playout clock moves with the tick and is pulled
//! toward the snapshots' ticks.
//!
//! The records are lazy (crate::engine::lazy): a game that names none, or
//! never joins, keeps its schedule and its hash.

use crate::engine::{Engine, engine, running};
use crate::host::codec::{Layout, RecordKind};
use crate::host::wire::{self, SharedObjectInfo, SharedRemote, SharedState, SharedStatus, record};
use og_ecs::{Entity, QueryDesc, SystemContext, SystemDesc, World};
use og_shared::protocol::{
    Authority, AvatarState, ClientMsg, Hello, MAX_EVENT, MAX_FIELDS, MAX_OBJECT, ObjectOp,
    PROTOCOL, Place, ServerMsg, SharedObject,
};
use og_shared::view::{Proposer, View};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(crate) const SYSTEM: &str = "og.shared";
const WORLD: &str = "og.SharedWorld";
const AVATAR: &str = "og.SharedAvatar";
const REMOTE: &str = "og.RemoteAvatar";
const JOINED: &str = "og.SharedJoined";
const LEFT: &str = "og.SharedLeft";
const ENTERED: &str = "og.PresenceEntered";
const PRESENCE_LEFT: &str = "og.PresenceLeft";
const CORRECTED: &str = "og.SharedCorrected";
const OBJECT: &str = "og.SharedObject";
const OBJECT_CHANGED: &str = "og.SharedObjectChanged";
const OBJECT_GONE: &str = "og.SharedObjectGone";
const OP_ANSWERED: &str = "og.SharedOpAnswered";
/// Registered together (crate::engine::lazy).
pub(crate) const RECORDS: [&str; 12] = [
    WORLD,
    AVATAR,
    REMOTE,
    JOINED,
    LEFT,
    ENTERED,
    PRESENCE_LEFT,
    CORRECTED,
    OBJECT,
    OBJECT_CHANGED,
    OBJECT_GONE,
    OP_ANSWERED,
];
const TRANSFORM: &str = "og.Transform";
const ORIGIN: &str = "og.WorldOrigin";

/// Reconnecting: the first wait, the longest, and when to give up (ms).
const BACKOFF_FIRST_MS: f64 = 250.0;
const BACKOFF_MAX_MS: f64 = 4000.0;
const RECONNECT_FOR_MS: f64 = 30_000.0;
/// Ops kept for a host that never takes them (a replay): the newest.
const KEEP_OPS: usize = 1024;
/// A remote avatar's states kept for interpolation, in server ticks.
const KEEP_SAMPLES: u32 = 40;
/// The playout clock snaps (rather than eases) when this far off, in server ticks.
const SNAP_TICKS: f64 = 10.0;

/// One remote avatar this client holds.
struct Remote {
    entity: Entity,
    player: String,
    name: String,
    kind: u32,
    since: u64,
    /// Its states by server tick, oldest first, and whether each jumped.
    samples: VecDeque<(u32, AvatarState, bool)>,
    /// The server tick of the newest jump drawn (og.RemoteAvatar.jumpedTick's).
    drawn_jump: Option<u32>,
}

#[derive(Default)]
pub(crate) struct Session {
    /// What carries sessions here (the provider's name); empty: none said.
    host: String,
    state: SharedState,
    world: String,
    // The shape (og.shared.shape).
    avatar: String,
    avatar_bytes: usize,
    /// The game's object component (og.shared.shape's `object`).
    object: String,
    event: String,
    fingerprint: u64,
    /// Delivered events waiting for the next tick, and ops for the host.
    inbox: Vec<Value>,
    outbox: VecDeque<Value>,
    open: bool,
    transport: String,
    view: View,
    proposer: Proposer,
    session: Option<u64>,
    player: String,
    name: String,
    cell_size: f64,
    server_hz: u32,
    /// og.SharedWorld.sendRateHz as of this tick (hello carries it).
    send_hz: u32,
    /// Reconnecting: the tick to ask again at, the next wait, when it was lost.
    retry_at: Option<u64>,
    backoff_ms: f64,
    lost_at: Option<u64>,
    /// After a welcome: the avatars held before, by player, until the
    /// server enters them again (the same entity carries on) or a second
    /// of server ticks passes without (`resumed_until`): they have left.
    /// Not the first snapshot: over WebTransport it is a datagram and can
    /// overtake the enters on the stream.
    resumed: BTreeMap<String, Remote>,
    resumed_until: u32,
    /// The shared objects held, by key: their entity and version.
    objects: BTreeMap<String, (Entity, u32)>,
    /// After a welcome: the keys the server sent again; at `resumed_until`
    /// the held objects not among them have left.
    objects_fresh: Option<BTreeSet<String>>,
    /// og.shared.put and og.shared.remove's last op number.
    ops: u32,
    /// Asked for a connection and not answered yet.
    asking: bool,
    remotes: BTreeMap<u16, Remote>,
    /// The playout clock, in server ticks; the newest server tick received.
    clock: Option<f64>,
    newest: u32,
    others: u32,
    /// The local avatar when last proposed: the tick and where it was.
    last_sent: Option<(u64, [f64; 3])>,
    /// A snapshot arrived since the last proposal (see `propose`).
    snapshot_since: bool,
    /// og.shared.teleport's place, applied as the next tick starts.
    teleport: Option<[f64; 3]>,
    jump_next: bool,
    proposals: u64,
    corrections: u32,
    bytes_in: u64,
    bytes_out: u64,
    reason: String,
    /// Events delivered this tick: the sender's set id and the bytes.
    events: Vec<(u16, Vec<u8>)>,
}

/// og.shared on one engine.
pub(crate) type ModuleState = RefCell<Session>;

fn with<T>(engine: u32, f: impl FnOnce(&mut Session) -> T) -> T {
    match crate::engine::engine(engine) {
        Ok(eng) => f(&mut eng.modules.shared.borrow_mut()),
        Err(_) => f(&mut Session::default()),
    }
}

fn tick_of(engine: u32) -> u64 {
    if let Some(f) = running(engine) {
        // SAFETY: the frame is published while its system runs.
        return unsafe { f.ctx() }.time().tick;
    }
    crate::engine::engine(engine)
        .and_then(|e| e.world().map(|w| w.time().tick))
        .unwrap_or(0)
}

impl Session {
    fn send(&mut self, bytes: Vec<u8>) {
        self.bytes_out += bytes.len() as u64;
        self.push(json!({ "op": "send", "bytes": og_core::base64::encode(&bytes) }));
    }

    fn push(&mut self, op: Value) {
        self.outbox.push_back(op);
        while self.outbox.len() > KEEP_OPS {
            self.outbox.pop_front();
        }
    }

    fn connect(&mut self, reconnect: bool) {
        self.asking = true;
        self.open = false;
        let world = self.world.clone();
        self.push(json!({ "op": "connect", "world": world, "reconnect": reconnect }));
    }

    fn ended(&self) -> bool {
        matches!(self.state, SharedState::Off | SharedState::Refused)
    }
}

// ── the host's side ──────────────────────────────────────────────────────

/// `og_shared_take`: the ops since the last call, oldest first.
pub fn og_shared_take(engine: u32) -> String {
    with(engine, |s| {
        Value::Array(std::mem::take(&mut s.outbox).into()).to_string()
    })
}

/// `og_shared_deliver`: one event from the host, applied as the next tick
/// starts and journaled now with the tick it was handed at.
pub fn og_shared_deliver(engine: u32, event: String) -> bool {
    let Ok(v) = serde_json::from_str::<Value>(&event) else {
        return false;
    };
    let ok = match v["type"].as_str() {
        Some("host") => v["host"].is_string(),
        Some("open") => v["token"].is_string() || v["token"].is_null(),
        Some("message") => v["bytes"]
            .as_str()
            .is_some_and(|b| og_core::base64::decode(b).is_some()),
        Some("closed" | "refused") => true,
        _ => false,
    };
    if !ok {
        return false;
    }
    let mut kept = v.clone();
    if let Some(o) = kept.as_object_mut() {
        // The token is the provider's credential: never kept.
        o.remove("token");
    }
    crate::host::platform::journal_handed(engine, tick_of(engine), "shared", &kept);
    with(engine, |s| s.inbox.push(v));
    true
}

// ── the game's calls ─────────────────────────────────────────────────────

fn engine_of(id: u32) -> Result<std::rc::Rc<Engine>, String> {
    engine(id)
}

fn token_name(component: &str) -> String {
    match serde_json::from_str::<Value>(component) {
        Ok(Value::String(s)) => s,
        Ok(Value::Object(o)) => o
            .get("name")
            .and_then(Value::as_str)
            .map_or_else(|| component.to_string(), str::to_string),
        _ => component.to_string(),
    }
}

/// A game's own component og.shared.shape may name, at most `max` bytes packed.
fn game_component(eng: &Engine, name: &str, what: &str, max: usize) -> Result<Layout, String> {
    let layout = eng
        .state
        .borrow()
        .layouts
        .get(name)
        .cloned()
        .filter(|l| l.kind == RecordKind::Component)
        .ok_or_else(|| {
            format!("og.shared.shape's {what}: '{name}' is not a registered component")
        })?;
    if name.starts_with("og.") {
        return Err(format!(
            "og.shared.shape's {what}: '{name}' is the engine's; name a component of the game's own (defineComponent)"
        ));
    }
    if layout.size() > max {
        return Err(format!(
            "og.shared.shape's {what}: '{name}' packs to {} bytes; at most {max}",
            layout.size()
        ));
    }
    Ok(layout)
}

/// The shape's fingerprint: its components' names, fields and types.
fn fingerprint(parts: &[&Layout]) -> u64 {
    let mut h = og_core::hash::StableHasher::new();
    for l in parts {
        h.write(l.schema.as_bytes());
        for f in l.fields.iter() {
            h.write(f.name.as_bytes());
            h.write(format!("{:?}", f.ty).as_bytes());
        }
        h.write(b";");
    }
    h.finish().max(1)
}

/// Registers og.shared's records (and so its system) on the engine's world.
fn ensure_records(eng: &Engine, world: &mut World) -> Result<(), String> {
    for r in RECORDS {
        crate::engine::lazy::ensure(eng, world, r)?;
    }
    Ok(())
}

/// og.WorldOrigin, when the world names it, is read for absolute places.
fn grant_origin(w: &mut World) -> Result<(), String> {
    if w.id_by_name(ORIGIN).is_ok() {
        w.grant(SYSTEM, &[ORIGIN])
            .map_err(|e| e.reason().to_string())?;
    }
    Ok(())
}

pub fn og_shared_shape(world: u32, desc: Option<String>) -> Result<(), String> {
    let d: wire::SharedShapeDesc = wire::arg(desc.as_deref(), "desc")?;
    let eng = engine_of(world)?;
    let w = eng.world().map_err(|_| {
        "og.shared.shape is called outside a system (setup or start), before og.shared.join"
            .to_string()
    })?;
    if !with(world, |s| s.ended()) {
        return Err("og.shared.shape: a session is on; set the shape before og.shared.join".into());
    }
    let name = |v: &Option<Value>, what: &str| -> Result<String, String> {
        match v {
            None | Some(Value::Null) => Ok(String::new()),
            Some(v) => Ok(token_name(&v.to_string())).and_then(|n| {
                if n.is_empty() {
                    Err(format!(
                        "og.shared.shape's {what} is a component token, got {v}"
                    ))
                } else {
                    Ok(n)
                }
            }),
        }
    };
    let (avatar, object, event) = (
        name(&d.avatar, "avatar")?,
        name(&d.object, "object")?,
        name(&d.event, "event")?,
    );
    let avatar_layout = if avatar.is_empty() {
        None
    } else {
        Some(game_component(&eng, &avatar, "avatar", MAX_FIELDS)?)
    };
    let object_layout = if object.is_empty() {
        None
    } else {
        Some(game_component(&eng, &object, "object", MAX_OBJECT)?)
    };
    let event_layout = if event.is_empty() {
        None
    } else {
        Some(game_component(&eng, &event, "event", MAX_EVENT)?)
    };
    ensure_records(&eng, w)?;
    grant_origin(w)?;
    for c in [&avatar, &object] {
        if !c.is_empty() {
            w.grant(SYSTEM, &[c.as_str()])
                .map_err(|e| e.reason().to_string())?;
        }
    }
    let parts: Vec<&Layout> = avatar_layout
        .iter()
        .chain(object_layout.iter())
        .chain(event_layout.iter())
        .collect();
    with(world, |s| {
        s.fingerprint = if parts.is_empty() {
            0
        } else {
            fingerprint(&parts)
        };
        s.avatar_bytes = avatar_layout.as_ref().map_or(0, Layout::size);
        s.avatar = avatar;
        s.object = object;
        s.event = event;
    });
    Ok(())
}

pub fn og_shared_join(world: u32, desc: Option<String>) -> Result<(), String> {
    let d: wire::SharedJoinDesc = wire::arg(desc.as_deref(), "desc")?;
    let name = if d.world.is_empty() {
        "main".to_string()
    } else {
        d.world
    };
    if name.len() > 64
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return Err(format!(
            "og.shared.join: '{name}' is not a world name: 1 to 64 of letters, digits, '.', '_' and '-'"
        ));
    }
    if crate::bind::net::session_running(world) {
        return Err("og.shared.join: lockstep runs on this world; a shared world is each client's own simulation".into());
    }
    let eng = engine_of(world)?;
    if !with(world, |s| s.ended()) {
        return Err("og.shared.join: a session is on already; og.shared.leave first".into());
    }
    match eng.world() {
        Ok(w) => {
            ensure_records(&eng, w)?;
            grant_origin(w)?;
        }
        Err(_) => {
            if !RECORDS
                .iter()
                .all(|r| eng.state.borrow().layouts.contains_key(*r))
            {
                return Err("og.shared.join: call it outside a system the first time (setup or start), so its records register".into());
            }
        }
    }
    with(world, |s| {
        s.world = name;
        s.state = SharedState::Connecting;
        s.reason.clear();
        s.proposer = Proposer::default();
        s.last_sent = None;
        s.lost_at = None;
        s.retry_at = None;
        s.backoff_ms = BACKOFF_FIRST_MS;
        s.session = None;
        s.bytes_in = 0;
        s.bytes_out = 0;
        s.proposals = 0;
        s.corrections = 0;
        s.connect(false);
    });
    Ok(())
}

pub fn og_shared_leave(world: u32) -> Result<(), String> {
    with(world, |s| {
        if s.ended() {
            return Err("og.shared.leave: no session".to_string());
        }
        if s.open {
            s.send(ClientMsg::Leave.to_bytes());
        }
        s.push(json!({ "op": "close" }));
        s.open = false;
        s.asking = false;
        // The system says og.SharedLeft and despawns the others next tick.
        s.inbox.push(json!({ "type": "left" }));
        Ok(())
    })
}

pub fn og_shared_teleport(world: u32, position: String) -> Result<(), String> {
    let p: Value = serde_json::from_str(&position).map_err(|e| format!("position: {e}"))?;
    let at = [&p["x"], &p["y"], &p["z"]].map(|v| v.as_f64().unwrap_or(f64::NAN));
    if !at.iter().all(|v| v.is_finite()) {
        return Err(format!(
            "og.shared.teleport: the position is not finite: {p}"
        ));
    }
    with(world, |s| s.teleport = Some(at));
    Ok(())
}

pub fn og_shared_send(world: u32, component: String, value: String) -> Result<(), String> {
    let eng = engine_of(world)?;
    let name = token_name(&component);
    let (event, joined) = with(world, |s| {
        (s.event.clone(), s.state == SharedState::Joined && s.open)
    });
    if event.is_empty() {
        return Err(
            "og.shared.send needs an event component: og.shared.shape(world, { event: MyEvent })"
                .into(),
        );
    }
    if name != event {
        return Err(format!(
            "og.shared.send takes '{event}', og.shared.shape's event; not '{name}'"
        ));
    }
    if !joined {
        return Err("og.shared.send: not joined (og.SharedJoined has not arrived)".into());
    }
    let v: Value = serde_json::from_str(&value).map_err(|e| format!("value: {e}"))?;
    if v.get("from").is_some() {
        return Err("og.shared.send: `from` is the engine's (the sender)".into());
    }
    let Some(layout) = eng.state.borrow().layouts.get(&name).cloned() else {
        return Err(format!("'{name}' is not registered in this world"));
    };
    let bytes = layout.encode(None, Some(&v))?;
    with(world, |s| {
        s.send(
            ClientMsg::Event {
                channel: 0,
                data: bytes,
            }
            .to_bytes(),
        )
    });
    Ok(())
}

pub fn og_shared_events(world: u32, component: String) -> Result<Value, String> {
    let eng = engine_of(world)?;
    let name = token_name(&component);
    let (event, list, entities) = with(world, |s| {
        let entities: BTreeMap<u16, u64> = s
            .remotes
            .iter()
            .map(|(id, r)| (*id, r.entity.to_bits()))
            .collect();
        (s.event.clone(), s.events.clone(), entities)
    });
    if event.is_empty() {
        return Ok(json!([]));
    }
    if name != event {
        return Err(format!(
            "og.shared.events reads '{event}', og.shared.shape's event; not '{name}'"
        ));
    }
    let Some(layout) = eng.state.borrow().layouts.get(&name).cloned() else {
        return Err(format!("'{name}' is not registered in this world"));
    };
    let has_from = layout.field("from").is_some();
    let mut out = Vec::new();
    for (from, data) in list {
        let mut base = data;
        base.resize(layout.size(), 0);
        let b = if has_from {
            let e = entities.get(&from).copied().unwrap_or(0);
            layout.encode(Some(&base), Some(&json!({ "from": e })))?
        } else {
            base
        };
        out.push(layout.decode(&b));
    }
    Ok(Value::Array(out))
}

/// A joined session's next op number, after the checks put and remove share.
fn op_ready(world: u32, key: &str, what: &str) -> Result<(), String> {
    if !og_shared::protocol::object_key(key) {
        return Err(format!(
            "og.shared.{what}: '{key}' is not a key: 1 to 64 of letters, digits, `.`, `_`, `-`, `:` and `/`"
        ));
    }
    if !with(world, |s| s.state == SharedState::Joined && s.open) {
        return Err(format!(
            "og.shared.{what}: not joined (og.SharedJoined has not arrived)"
        ));
    }
    Ok(())
}

fn send_op(world: u32, key: String, expect: Option<u32>, set: Option<(Place, Vec<u8>)>) -> u32 {
    with(world, |s| {
        s.ops += 1;
        let seq = s.ops;
        s.send(
            ClientMsg::Object(ObjectOp {
                seq,
                key,
                expect,
                set,
            })
            .to_bytes(),
        );
        seq
    })
}

pub fn og_shared_put(
    world: u32,
    key: String,
    component: String,
    value: String,
    desc: Option<String>,
) -> Result<u32, String> {
    let eng = engine_of(world)?;
    let name = token_name(&component);
    let (object, cs) = with(world, |s| (s.object.clone(), s.cell_size));
    if object.is_empty() {
        return Err(
            "og.shared.put needs an object component: og.shared.shape(world, { object: MyObject })"
                .into(),
        );
    }
    if name != object {
        return Err(format!(
            "og.shared.put takes '{object}', og.shared.shape's object; not '{name}'"
        ));
    }
    op_ready(world, &key, "put")?;
    // Read as f64: a place is exact at any distance.
    let d: Value = match desc.as_deref() {
        None | Some("") | Some("null") => Value::Null,
        Some(t) => serde_json::from_str(t).map_err(|e| format!("desc: {e}"))?,
    };
    let p = &d["position"];
    let at = [&p["x"], &p["y"], &p["z"]].map(|v| v.as_f64().unwrap_or(0.0));
    let yaw = d["yaw"].as_f64().unwrap_or(0.0);
    if !at.iter().chain([&yaw]).all(|v| v.is_finite()) {
        return Err(format!("og.shared.put: the place is not finite: {d}"));
    }
    let expect = match &d["expect"] {
        Value::Null => None,
        v => Some(
            v.as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .ok_or_else(|| format!("og.shared.put: expect is a version, got {v}"))?,
        ),
    };
    let v: Value = serde_json::from_str(&value).map_err(|e| format!("value: {e}"))?;
    let Some(layout) = eng.state.borrow().layouts.get(&name).cloned() else {
        return Err(format!("'{name}' is not registered in this world"));
    };
    let bytes = layout.encode(None, Some(&v))?;
    let place = Place::from_world(cs, at, yaw as f32);
    Ok(send_op(world, key, expect, Some((place, bytes))))
}

pub fn og_shared_remove(world: u32, key: String, expect: String) -> Result<u32, String> {
    op_ready(world, &key, "remove")?;
    let expect = match serde_json::from_str::<Value>(&expect).unwrap_or(Value::Null) {
        Value::Null => None,
        v => Some(
            v.as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .ok_or_else(|| format!("og.shared.remove: expect is a version, got {v}"))?,
        ),
    };
    Ok(send_op(world, key, expect, None))
}

pub fn og_shared_object(world: u32, entity: f64) -> Option<SharedObjectInfo> {
    let bits = entity as u64;
    with(world, |s| {
        s.objects
            .iter()
            .find(|(_, (e, _))| e.to_bits() == bits)
            .map(|(k, (_, v))| SharedObjectInfo {
                key: k.clone(),
                version: *v,
            })
    })
}

pub fn og_shared_remote(world: u32, entity: f64) -> Option<SharedRemote> {
    let bits = entity as u64;
    with(world, |s| {
        s.remotes
            .values()
            .find(|r| r.entity.to_bits() == bits)
            .map(|r| SharedRemote {
                player: r.player.clone(),
                name: r.name.clone(),
                kind: r.kind,
                since_tick: r.since as f64,
            })
    })
}

pub fn og_shared_status(world: u32) -> SharedStatus {
    with(world, |s| SharedStatus {
        state: s.state.as_str().to_string(),
        host: if s.host.is_empty() {
            "none".into()
        } else {
            s.host.clone()
        },
        transport: if s.open {
            s.transport.clone()
        } else {
            String::new()
        },
        world: s.world.clone(),
        player: s.player.clone(),
        name: s.name.clone(),
        in_sight: s.remotes.len() as u32,
        others: s.others,
        server_tick: f64::from(s.newest),
        server_hz: s.server_hz,
        proposals: s.proposals as f64,
        corrections: s.corrections,
        bytes_in: s.bytes_in as f64,
        bytes_out: s.bytes_out as f64,
        reason: s.reason.clone(),
    })
}

// ── the system ───────────────────────────────────────────────────────────

/// Schedules og.shared on a world that names its records
/// (`crate::engine::lazy`), right after og.net: before the game's systems,
/// so they see this tick's remote avatars.
pub(crate) fn enable(world: &mut World, mount: crate::engine::Mount<'_>) -> og_core::Result<()> {
    if world.systems().contains(&SYSTEM) || RECORDS.iter().any(|r| world.id_by_name(r).is_err()) {
        // The last of og.shared's records to register schedules it.
        return Ok(());
    }
    let engine_id = mount.id;
    let mut writes: Vec<String> = RECORDS.iter().map(|s| s.to_string()).collect();
    writes.push(TRANSFORM.into());
    let mut reads = Vec::new();
    if world.id_by_name(ORIGIN).is_ok() {
        reads.push(ORIGIN.to_string());
    }
    world.add_system(
        SystemDesc {
            name: SYSTEM.into(),
            reads,
            writes,
            queries: vec![QueryDesc {
                read: vec![AVATAR.into()],
                write: vec![TRANSFORM.into()],
                ..QueryDesc::default()
            }],
            ..Default::default()
        },
        move |ctx| system(ctx, engine_id),
    )?;
    // Before the systems mounted after og.net (og.story's), so before the game's.
    match world.move_system_before(SYSTEM, crate::bind::story::SYSTEM) {
        Ok(()) => Ok(()),
        Err(_) => Ok(()),
    }
}

fn invalid(e: impl std::fmt::Display) -> og_core::Error {
    og_core::Error::new(og_core::ErrorKind::Invalid, e.to_string())
}

/// What a tick's system holds of the records it writes.
struct Recs {
    layouts: BTreeMap<String, Layout>,
}

impl Recs {
    fn get(&self, name: &str) -> og_core::Result<&Layout> {
        self.layouts
            .get(name)
            .ok_or_else(|| invalid(format!("{name} has no layout")))
    }

    fn event(&self, ctx: &SystemContext<'_>, name: &str, v: &Value) -> og_core::Result<()> {
        let bytes = self.get(name)?.encode(None, Some(v)).map_err(invalid)?;
        ctx.send_bytes(ctx.id_by_name(name)?, &bytes)
    }
}

/// A String event field holds at most 63 bytes, cut on a character.
fn short(s: &str) -> String {
    let mut end = s.len().min(63);
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

/// The yaw (about y) of a rotation, radians.
fn yaw_of(q: &Value) -> f32 {
    let g = |k: &str| q[k].as_f64().unwrap_or(0.0);
    let (x, y, z, w) = (g("x"), g("y"), g("z"), g("w"));
    (2.0 * (w * y + x * z)).atan2(1.0 - 2.0 * (x * x + y * y)) as f32
}

fn quat_of(yaw: f32) -> Value {
    let h = f64::from(yaw) * 0.5;
    json!({ "x": 0.0, "y": h.sin(), "z": 0.0, "w": h.cos() })
}

fn vec3(v: &Value) -> [f64; 3] {
    [&v["x"], &v["y"], &v["z"]].map(|c| c.as_f64().unwrap_or(0.0))
}

fn system(ctx: &mut SystemContext<'_>, engine_id: u32) -> og_core::Result<()> {
    let Ok(eng) = engine(engine_id) else {
        return Ok(());
    };
    let time = ctx.time();
    let tick = time.tick;
    let dt = f64::from(time.dt).max(1e-6);
    let recs = Recs {
        layouts: {
            let st = eng.state.borrow();
            let mut names: Vec<&str> = RECORDS.to_vec();
            names.extend([TRANSFORM, ORIGIN]);
            let (avatar, object) = with(engine_id, |s| (s.avatar.clone(), s.object.clone()));
            let mut m: BTreeMap<String, Layout> = names
                .iter()
                .filter_map(|n| st.layouts.get(*n).map(|l| (n.to_string(), l.clone())))
                .collect();
            for c in [avatar, object] {
                if !c.is_empty()
                    && let Some(l) = st.layouts.get(&c)
                {
                    m.insert(c, l.clone());
                }
            }
            m
        },
    };
    let settings = {
        let id = ctx.id_by_name(WORLD)?;
        recs.get(WORLD)?
            .read::<record::SharedWorld>(&ctx.resource_bytes(id)?)
            .unwrap_or_default()
    };
    // The world's zero (og.WorldOrigin): shared places are absolute.
    let origin = match (ctx.id_by_name(ORIGIN), recs.layouts.get(ORIGIN)) {
        (Ok(id), Some(l)) => ctx
            .resource_bytes(id)
            .ok()
            .and_then(|b| l.read::<record::WorldOrigin>(&b).ok())
            .map_or([0.0; 3], |o| [o.x, o.y, o.z]),
        _ => [0.0; 3],
    };
    let transform = (ctx.id_by_name(TRANSFORM)?, recs.get(TRANSFORM)?.clone());
    // The local avatar: the first og.SharedAvatar in entity order.
    let local = {
        let mut found: Vec<Entity> = ctx.query(0)?.flat_map(|v| v.entities().to_vec()).collect();
        found.sort_by_key(|e| e.to_bits());
        found.first().copied()
    };

    let inbox = with(engine_id, |s| {
        s.send_hz = settings.send_rate_hz;
        s.events.clear();
        std::mem::take(&mut s.inbox)
    });
    for ev in inbox {
        apply(ctx, engine_id, &recs, &transform, local, origin, tick, &ev)?;
    }

    // Avatars held across a reconnect that the server did not enter again
    // within a second of server ticks have left.
    let stale: Vec<(String, Entity)> = with(engine_id, |s| {
        if s.newest < s.resumed_until {
            return Vec::new();
        }
        let Some(fresh) = s.objects_fresh.take() else {
            return Vec::new();
        };
        let gone: Vec<String> = s
            .objects
            .keys()
            .filter(|k| !fresh.contains(*k))
            .cloned()
            .collect();
        gone.into_iter()
            .filter_map(|k| s.objects.remove(&k).map(|(e, _)| (k, e)))
            .collect()
    });
    for (key, e) in stale {
        object_gone(ctx, &recs, &key, e)?;
    }
    let gone: Vec<Remote> = with(engine_id, |s| {
        if s.resumed.is_empty() || s.newest < s.resumed_until {
            return Vec::new();
        }
        std::mem::take(&mut s.resumed).into_values().collect()
    });
    for r in gone {
        let _ = ctx.despawn(r.entity);
        recs.event(
            ctx,
            PRESENCE_LEFT,
            &json!({ "entity": r.entity.to_bits(), "player": short(&r.player) }),
        )?;
    }

    // Reconnecting: ask again when the wait is over; give up after 30 s.
    with(engine_id, |s| {
        if s.state != SharedState::Reconnecting {
            return;
        }
        let lost = s.lost_at.unwrap_or(tick);
        if (tick - lost) as f64 * dt * 1000.0 > RECONNECT_FOR_MS {
            s.state = SharedState::Refused;
            s.reason = "offline: no connection for 30 s".into();
            s.asking = false;
            s.inbox.push(json!({ "type": "gave-up" }));
            return;
        }
        if !s.asking && s.retry_at.is_some_and(|t| tick >= t) {
            s.retry_at = None;
            s.connect(true);
        }
    });

    // og.shared.teleport: the avatar moves as the tick starts.
    if let (Some(e), Some(at)) = (local, with(engine_id, |s| s.teleport.take())) {
        let rel = [at[0] - origin[0], at[1] - origin[1], at[2] - origin[2]];
        let (old, _) = local_transform(ctx, &transform, e)?;
        let b = transform
            .1
            .encode(
                Some(&old),
                Some(&json!({ "position": { "x": rel[0], "y": rel[1], "z": rel[2] } })),
            )
            .map_err(invalid)?;
        ctx.set_bytes(e, transform.0, &b)?;
        with(engine_id, |s| s.jump_next = true);
    }

    draw_remotes(ctx, engine_id, &recs, &transform, origin, &settings, dt)?;

    // This client's proposal, at sendRateHz.
    if let Some(e) = local {
        propose(
            ctx, engine_id, &recs, &transform, e, origin, tick, dt, &settings,
        )?;
    }

    // og.SharedWorld: where the session stands.
    let (state, in_sight) = with(engine_id, |s| (s.state, s.remotes.len() as u32));
    let id = ctx.id_by_name(WORLD)?;
    let base = ctx.resource_bytes(id)?;
    let b = recs
        .get(WORLD)?
        .encode(
            Some(&base),
            Some(&json!({ "state": state.as_str(), "inSight": in_sight })),
        )
        .map_err(invalid)?;
    if b != base {
        ctx.set_resource_bytes(id, &b)?;
    }
    Ok(())
}

/// Despawns every remote avatar, telling the game.
fn clear_remotes(ctx: &mut SystemContext<'_>, engine_id: u32, recs: &Recs) -> og_core::Result<()> {
    let gone: Vec<Remote> = with(engine_id, |s| {
        s.clock = None;
        let mut gone: Vec<Remote> = std::mem::take(&mut s.remotes).into_values().collect();
        gone.extend(std::mem::take(&mut s.resumed).into_values());
        gone
    });
    for r in gone {
        let _ = ctx.despawn(r.entity);
        recs.event(
            ctx,
            PRESENCE_LEFT,
            &json!({ "entity": r.entity.to_bits(), "player": short(&r.player) }),
        )?;
    }
    let objects = with(engine_id, |s| {
        s.objects_fresh = None;
        std::mem::take(&mut s.objects)
    });
    for (key, (e, _)) in objects {
        object_gone(ctx, recs, &key, e)?;
    }
    Ok(())
}

fn object_gone(ctx: &mut SystemContext<'_>, recs: &Recs, key: &str, e: Entity) -> og_core::Result<()> {
    let _ = ctx.despawn(e);
    recs.event(
        ctx,
        OBJECT_GONE,
        &json!({ "entity": e.to_bits(), "key": short(key) }),
    )
}

/// A shared object's state on its entity: spawned, or written over.
fn object_in(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    recs: &Recs,
    transform: &(og_ecs::SchemaId, Layout),
    origin: [f64; 3],
    o: SharedObject,
) -> og_core::Result<()> {
    let (cs, object, held) = with(engine_id, |s| {
        if let Some(f) = s.objects_fresh.as_mut() {
            f.insert(o.key.clone());
        }
        (s.cell_size, s.object.clone(), s.objects.get(&o.key).map(|(e, _)| *e))
    });
    let p = o.place.position(cs);
    let at = json!({
        "position": { "x": p[0] - origin[0], "y": p[1] - origin[1], "z": p[2] - origin[2] },
        "quaternion": quat_of(o.place.yaw()),
    });
    let mut parts = vec![
        (transform.0, transform.1.encode(None, Some(&at)).map_err(invalid)?),
        (
            ctx.id_by_name(OBJECT)?,
            recs.get(OBJECT)?
                .encode(None, Some(&json!({ "version": o.version })))
                .map_err(invalid)?,
        ),
    ];
    if !object.is_empty()
        && let Some(l) = recs.layouts.get(&object)
    {
        let mut b = o.data.clone();
        b.resize(l.size(), 0);
        parts.push((ctx.id_by_name(&object)?, b));
    }
    let entity = match held {
        Some(e) => {
            for (id, b) in &parts {
                ctx.set_bytes(e, *id, b)?;
            }
            e
        }
        None => ctx.spawn_bytes(parts)?,
    };
    with(engine_id, |s| s.objects.insert(o.key.clone(), (entity, o.version)));
    recs.event(
        ctx,
        OBJECT_CHANGED,
        &json!({ "entity": entity.to_bits(), "key": short(&o.key), "version": o.version }),
    )
}

#[allow(clippy::too_many_arguments)]
fn apply(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    recs: &Recs,
    transform: &(og_ecs::SchemaId, Layout),
    local: Option<Entity>,
    origin: [f64; 3],
    tick: u64,
    ev: &Value,
) -> og_core::Result<()> {
    let ended = |ctx: &mut SystemContext<'_>, reason: String| -> og_core::Result<()> {
        clear_remotes(ctx, engine_id, recs)?;
        recs.event(ctx, LEFT, &json!({ "reason": short(&reason) }))
    };
    match ev["type"].as_str().unwrap_or("") {
        "host" => {
            let host = ev["host"].as_str().unwrap_or("none").to_string();
            let refuse = with(engine_id, |s| {
                s.host = host.clone();
                (host == "none" && s.asking).then(|| {
                    s.asking = false;
                    s.state = SharedState::Refused;
                    s.reason = "unavailable: this host carries no shared worlds".into();
                    s.reason.clone()
                })
            });
            if let Some(r) = refuse {
                ended(ctx, r)?;
            }
        }
        "refused" => {
            let reason = format!(
                "{}: {}",
                ev["code"].as_str().unwrap_or("failed"),
                ev["message"].as_str().unwrap_or("")
            );
            let end = with(engine_id, |s| {
                if s.ended() {
                    return None;
                }
                s.asking = false;
                s.open = false;
                s.state = SharedState::Refused;
                s.reason = reason.clone();
                Some(reason.clone())
            });
            if let Some(r) = end {
                ended(ctx, r)?;
            }
        }
        "open" => {
            let token = ev["token"].as_str().unwrap_or("").to_string();
            let transport = ev["transport"].as_str().unwrap_or("ws").to_string();
            let kind = match local {
                Some(e) => {
                    let id = ctx.id_by_name(AVATAR)?;
                    recs.get(AVATAR)?
                        .read::<record::SharedAvatar>(&ctx.get_bytes(e, id)?)
                        .map_or(0, |a| a.kind)
                }
                None => 0,
            };
            with(engine_id, |s| {
                if s.ended() {
                    s.push(json!({ "op": "close" }));
                    return;
                }
                s.asking = false;
                s.open = true;
                s.transport = transport;
                s.view = View::new();
                let hello = Hello {
                    version: PROTOCOL,
                    token,
                    world: s.world.clone(),
                    shape: s.fingerprint,
                    fields_len: s.avatar_bytes as u8,
                    authority: Authority::Validate,
                    send_hz: s.send_hz.clamp(1, 60) as u8,
                    kind,
                    resume: s.session,
                };
                s.send(ClientMsg::Hello(hello).to_bytes());
            });
        }
        "closed" | "gave-up" | "left" => {
            let t = ev["type"].as_str().unwrap_or("");
            if t == "left" {
                with(engine_id, |s| {
                    s.state = SharedState::Off;
                    s.reason = "left".into();
                    s.session = None;
                });
                return ended(ctx, "left".into());
            }
            if t == "gave-up" {
                let r = with(engine_id, |s| s.reason.clone());
                return ended(ctx, r);
            }
            with(engine_id, |s| {
                s.open = false;
                s.asking = false;
                if matches!(
                    s.state,
                    SharedState::Joined | SharedState::Connecting | SharedState::Reconnecting
                ) {
                    // A refusal said why already; a plain loss reconnects.
                    s.state = SharedState::Reconnecting;
                    s.lost_at.get_or_insert(tick);
                    let wait = s.backoff_ms.max(BACKOFF_FIRST_MS);
                    s.backoff_ms = (wait * 2.0).min(BACKOFF_MAX_MS);
                    let dt_ms = f64::from(ctx.time().dt).max(1e-6) * 1000.0;
                    s.retry_at = Some(tick + (wait / dt_ms).ceil() as u64);
                }
            });
        }
        "message" => {
            let bytes =
                og_core::base64::decode(ev["bytes"].as_str().unwrap_or("")).unwrap_or_default();
            with(engine_id, |s| s.bytes_in += bytes.len() as u64);
            let Ok(msg) = ServerMsg::decode(&bytes) else {
                warn(
                    engine_id,
                    tick,
                    "a server message did not decode; it was dropped",
                );
                return Ok(());
            };
            message(ctx, engine_id, recs, transform, local, origin, tick, msg)?;
        }
        _ => {}
    }
    Ok(())
}

/// The local avatar's og.Transform: its bytes and its value.
fn local_transform(
    ctx: &SystemContext<'_>,
    transform: &(og_ecs::SchemaId, Layout),
    e: Entity,
) -> og_core::Result<(Vec<u8>, Value)> {
    let b = ctx.get_bytes(e, transform.0)?;
    let v = transform.1.decode(&b);
    Ok((b, v))
}

fn warn(engine_id: u32, tick: u64, text: &str) {
    if let Ok(eng) = engine(engine_id)
        && let Ok(mut st) = eng.state.try_borrow_mut()
    {
        st.log
            .push(json!({ "tick": tick, "warning": true, "text": format!("og.shared: {text}") }));
    }
}

#[allow(clippy::too_many_arguments)]
fn message(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    recs: &Recs,
    transform: &(og_ecs::SchemaId, Layout),
    local: Option<Entity>,
    origin: [f64; 3],
    tick: u64,
    msg: ServerMsg,
) -> og_core::Result<()> {
    match msg {
        ServerMsg::Welcome(w) => {
            // A reconnect keeps the avatars it held, resumed or not: the
            // server enters those still in sight again, and each carries on
            // as the same entity, with no og.PresenceLeft or
            // og.PresenceEntered.
            with(engine_id, |s| {
                s.clock = None;
                s.resumed_until = w.tick + u32::from(w.tick_hz).max(1);
                s.objects_fresh = Some(BTreeSet::new());
                let held = std::mem::take(&mut s.remotes);
                s.resumed
                    .extend(held.into_values().map(|r| (r.player.clone(), r)));
            });
            with(engine_id, |s| {
                s.state = SharedState::Joined;
                s.session = Some(w.session);
                s.player = w.player.clone();
                s.name = w.name.clone();
                s.cell_size = f64::from(w.cell_size);
                s.server_hz = u32::from(w.tick_hz).max(1);
                s.newest = w.tick;
                s.lost_at = None;
                s.retry_at = None;
                s.backoff_ms = BACKOFF_FIRST_MS;
                // The server takes the next proposal whole.
                s.last_sent = None;
            });
            recs.event(
                ctx,
                JOINED,
                &json!({ "player": short(&w.player), "name": short(&w.name), "resumed": w.resumed }),
            )?;
        }
        ServerMsg::Refused { code, message } => {
            let reason = format!("{}: {message}", code.code());
            with(engine_id, |s| {
                s.state = SharedState::Refused;
                s.reason = reason.clone();
                s.open = false;
                s.session = None;
                s.push(json!({ "op": "close" }));
            });
            clear_remotes(ctx, engine_id, recs)?;
            recs.event(ctx, LEFT, &json!({ "reason": short(&reason) }))?;
        }
        ServerMsg::Enter(e) => {
            let (cs, avatar, avatar_bytes) = with(engine_id, |s| {
                (s.cell_size, s.avatar.clone(), s.avatar_bytes)
            });
            if let Some(old) = with(engine_id, |s| {
                s.view.enter(&e);
                s.remotes.remove(&e.id)
            }) {
                let _ = ctx.despawn(old.entity);
            }
            let p = e.state.position(cs);
            let at = json!({
                "position": { "x": p[0] - origin[0], "y": p[1] - origin[1], "z": p[2] - origin[2] },
                "quaternion": quat_of(e.state.yaw()),
            });
            let mut parts = vec![
                (
                    transform.0,
                    transform.1.encode(None, Some(&at)).map_err(invalid)?,
                ),
                (
                    ctx.id_by_name(REMOTE)?,
                    recs.get(REMOTE)?
                        .encode(
                            None,
                            Some(&json!({ "id": e.id, "kind": e.kind, "sinceTick": tick })),
                        )
                        .map_err(invalid)?,
                ),
            ];
            if !avatar.is_empty()
                && let Some(l) = recs.layouts.get(&avatar)
            {
                let mut b = e.state.fields().to_vec();
                b.resize(avatar_bytes.max(l.size()), 0);
                b.truncate(l.size());
                parts.push((ctx.id_by_name(&avatar)?, b));
            }
            let kept = with(engine_id, |s| s.resumed.remove(&e.player));
            let (entity, since) = match &kept {
                Some(r) => {
                    let (id_remote, since) = (ctx.id_by_name(REMOTE)?, r.since);
                    for (id, b) in &parts {
                        if *id == id_remote {
                            let b = recs
                                .get(REMOTE)?
                                .encode(
                                    None,
                                    Some(
                                        &json!({ "id": e.id, "kind": e.kind, "sinceTick": since }),
                                    ),
                                )
                                .map_err(invalid)?;
                            ctx.set_bytes(r.entity, *id, &b)?;
                        } else {
                            ctx.set_bytes(r.entity, *id, b)?;
                        }
                    }
                    (r.entity, since)
                }
                None => (ctx.spawn_bytes(parts)?, tick),
            };
            with(engine_id, |s| {
                let mut samples = VecDeque::new();
                samples.push_back((e.tick, e.state, true));
                s.remotes.insert(
                    e.id,
                    Remote {
                        entity,
                        player: e.player.clone(),
                        name: e.name.clone(),
                        kind: e.kind,
                        since,
                        samples,
                        drawn_jump: Some(e.tick),
                    },
                );
                s.newest = s.newest.max(e.tick);
            });
            if kept.is_none() {
                recs.event(
                    ctx,
                    ENTERED,
                    &json!({ "entity": entity.to_bits(), "player": short(&e.player), "name": short(&e.name), "kind": e.kind }),
                )?;
            }
        }
        ServerMsg::Leave { id } => {
            if let Some(r) = with(engine_id, |s| {
                s.view.leave(id);
                s.remotes.remove(&id)
            }) {
                let _ = ctx.despawn(r.entity);
                recs.event(
                    ctx,
                    PRESENCE_LEFT,
                    &json!({ "entity": r.entity.to_bits(), "player": short(&r.player) }),
                )?;
            }
        }
        ServerMsg::Snapshot(bytes) => {
            with(engine_id, |s| s.snapshot_since = true);
            with(engine_id, |s| match s.view.snapshot(&bytes) {
                Ok(Some(applied)) => {
                    let t = applied.head.tick;
                    s.newest = s.newest.max(t);
                    s.others = applied.head.others;
                    for u in applied.updates {
                        if let Some(r) = s.remotes.get_mut(&u.id) {
                            r.samples.push_back((t, u.state, u.jump));
                            while r.samples.len() > 2
                                && r.samples
                                    .front()
                                    .is_some_and(|(st, _, _)| t.saturating_sub(*st) > KEEP_SAMPLES)
                            {
                                r.samples.pop_front();
                            }
                        }
                    }
                }
                Ok(None) => {}
                // Ahead of an enter still on the stream: the next applies.
                Err(og_shared::view::NOT_ENTERED) => {}
                Err(e) => warn(engine_id, tick, &format!("a snapshot was dropped: {e}")),
            });
        }
        ServerMsg::Correct { seq, state } => {
            let Some(e) = local else {
                return Ok(());
            };
            let (cs, proposed) = with(engine_id, |s| {
                s.corrections += 1;
                (s.cell_size, s.proposer.state(seq))
            });
            // The state the server holds, plus what the avatar moved since `seq`.
            let (old, v) = local_transform(ctx, transform, e)?;
            let now = vec3(&v["position"]);
            let abs_now = [now[0] + origin[0], now[1] + origin[1], now[2] + origin[2]];
            let held = state.position(cs);
            let since = proposed.map_or([0.0; 3], |p| {
                let p = p.position(cs);
                [abs_now[0] - p[0], abs_now[1] - p[1], abs_now[2] - p[2]]
            });
            let to = [
                held[0] + since[0] - origin[0],
                held[1] + since[1] - origin[1],
                held[2] + since[2] - origin[2],
            ];
            let b = transform
                .1
                .encode(
                    Some(&old),
                    Some(&json!({ "position": { "x": to[0], "y": to[1], "z": to[2] } })),
                )
                .map_err(invalid)?;
            ctx.set_bytes(e, transform.0, &b)?;
            with(engine_id, |s| {
                s.last_sent = Some((
                    tick,
                    [to[0] + origin[0], to[1] + origin[1], to[2] + origin[2]],
                ))
            });
            recs.event(ctx, CORRECTED, &json!({ "sequence": seq }))?;
        }
        ServerMsg::Event { from, data, .. } => {
            with(engine_id, |s| {
                if !s.event.is_empty() {
                    s.events.push((from, data));
                }
            });
        }
        ServerMsg::Moving { .. } => {
            // The server drains: a new connection through the provider, at once.
            with(engine_id, |s| {
                s.push(json!({ "op": "close" }));
                s.open = false;
                s.state = SharedState::Reconnecting;
                s.session = None;
                s.lost_at.get_or_insert(tick);
                s.retry_at = Some(tick);
            });
        }
        ServerMsg::Pong { .. } => {}
        ServerMsg::Object(o) => object_in(ctx, engine_id, recs, transform, origin, o)?,
        ServerMsg::ObjectGone { key } => {
            if let Some((e, _)) = with(engine_id, |s| s.objects.remove(&key)) {
                object_gone(ctx, recs, &key, e)?;
            }
        }
        ServerMsg::Op {
            seq,
            taken,
            version,
        } => recs.event(
            ctx,
            OP_ANSWERED,
            &json!({ "op": seq, "taken": taken, "version": version }),
        )?,
    }
    Ok(())
}

/// One remote avatar's state at playout time `t` (server ticks): between
/// the two samples around it, at the later one across a jump, along the
/// last velocity past the newest for `extrapolate` ticks, then held.
fn sample(
    r: &Remote,
    t: f64,
    cs: f64,
    extrapolate: f64,
    hz: f64,
) -> ([f64; 3], f32, Option<u32>, &[u8]) {
    let s = &r.samples;
    let first = s.front().expect("a remote has its enter's sample");
    if t <= f64::from(first.0) {
        return (
            first.1.position(cs),
            first.1.yaw(),
            first.2.then_some(first.0),
            first.1.fields(),
        );
    }
    let i = s
        .iter()
        .rposition(|(st, _, _)| f64::from(*st) <= t)
        .unwrap_or(0);
    let (ta, a, ja) = &s[i];
    let pa = a.position(cs);
    let jump_a = ja.then_some(*ta);
    match s.get(i + 1) {
        Some((tb, b, jb)) => {
            if *jb {
                return (pa, a.yaw(), jump_a, a.fields());
            }
            let f = (t - f64::from(*ta)) / f64::from(tb - ta).max(1.0);
            let pb = b.position(cs);
            let p = std::array::from_fn(|k| pa[k] + (pb[k] - pa[k]) * f);
            let (ya, yb) = (a.yaw(), b.yaw());
            let tau = std::f32::consts::TAU;
            let d = (yb - ya + tau * 1.5).rem_euclid(tau) - tau * 0.5;
            (p, ya + d * f as f32, jump_a, a.fields())
        }
        None => {
            let ahead = (t - f64::from(*ta)).min(extrapolate) / hz;
            let v = a.velocity();
            let p = std::array::from_fn(|k| pa[k] + f64::from(v[k]) * ahead);
            (p, a.yaw(), jump_a, a.fields())
        }
    }
}

/// Where a remote avatar is drawn this tick.
struct Draw {
    entity: Entity,
    at: [f64; 3],
    yaw: f32,
    /// The server tick of a jump it made, the first tick it is drawn.
    jumped: Option<u32>,
    fields: Vec<u8>,
}

#[allow(clippy::too_many_arguments)]
fn draw_remotes(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    recs: &Recs,
    transform: &(og_ecs::SchemaId, Layout),
    origin: [f64; 3],
    settings: &record::SharedWorld,
    dt: f64,
) -> og_core::Result<()> {
    let (cs, hz, avatar) = with(engine_id, |s| {
        (s.cell_size, f64::from(s.server_hz.max(1)), s.avatar.clone())
    });
    if cs <= 0.0 {
        return Ok(());
    }
    let delay = f64::from(settings.interp_delay_ms) / 1000.0 * hz;
    let extrapolate = f64::from(settings.extrapolate_ms) / 1000.0 * hz;
    let remote_id = ctx.id_by_name(REMOTE)?;
    let avatar_id = if avatar.is_empty() {
        None
    } else {
        ctx.id_by_name(&avatar).ok()
    };
    // The playout clock: moved by the tick, pulled toward the newest server
    // tick less the delay.
    let draws: Vec<Draw> = with(engine_id, |s| {
        if s.remotes.is_empty() {
            return Vec::new();
        }
        let target = f64::from(s.newest) - delay;
        let clock = match s.clock {
            None => target,
            Some(c) => {
                let c = c + dt * hz;
                let err = target - c;
                if err.abs() > SNAP_TICKS {
                    target
                } else {
                    c + err * 0.05
                }
            }
        };
        s.clock = Some(clock);
        s.remotes
            .values_mut()
            .map(|r| {
                let (p, yaw, jump, fields) = sample(r, clock, cs, extrapolate, hz);
                let fields = fields.to_vec();
                let jumped = jump.filter(|j| r.drawn_jump != Some(*j));
                if let Some(j) = jumped {
                    r.drawn_jump = Some(j);
                }
                Draw {
                    entity: r.entity,
                    at: p,
                    yaw,
                    jumped,
                    fields,
                }
            })
            .collect()
    });
    let tick = ctx.time().tick;
    for Draw {
        entity: e,
        at: p,
        yaw,
        jumped,
        fields,
    } in draws
    {
        let Ok(old) = ctx.get_bytes(e, transform.0) else {
            continue;
        };
        let at = json!({
            "position": { "x": p[0] - origin[0], "y": p[1] - origin[1], "z": p[2] - origin[2] },
            "quaternion": quat_of(yaw),
        });
        let b = transform.1.encode(Some(&old), Some(&at)).map_err(invalid)?;
        if b != old {
            ctx.set_bytes(e, transform.0, &b)?;
        }
        if jumped.is_some() {
            let old = ctx.get_bytes(e, remote_id)?;
            let b = recs
                .get(REMOTE)?
                .encode(Some(&old), Some(&json!({ "jumpedTick": tick })))
                .map_err(invalid)?;
            ctx.set_bytes(e, remote_id, &b)?;
        }
        if let (Some(id), Some(l)) = (avatar_id, recs.layouts.get(&avatar))
            && let Ok(old) = ctx.get_bytes(e, id)
        {
            let mut b = fields;
            b.resize(l.size(), 0);
            if b != old {
                ctx.set_bytes(e, id, &b)?;
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn propose(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    recs: &Recs,
    transform: &(og_ecs::SchemaId, Layout),
    e: Entity,
    origin: [f64; 3],
    tick: u64,
    dt: f64,
    settings: &record::SharedWorld,
) -> og_core::Result<()> {
    let (joined, cs, last, avatar) = with(engine_id, |s| {
        (
            s.state == SharedState::Joined && s.open,
            s.cell_size,
            s.last_sent,
            s.avatar.clone(),
        )
    });
    if !joined || cs <= 0.0 {
        return Ok(());
    }
    // A proposal leaves on the tick after a snapshot arrives, so the
    // transport's acknowledgement of that snapshot rides in the same packet
    // instead of one of its own: over WebTransport a lone ACK costs as much
    // as the proposal, and a client acks every snapshot (20 Hz) within
    // 25 ms. The server sends a snapshot every tick, so proposals lock to
    // them within one; the shortest gap (two thirds of the period) keeps
    // the rate at sendRateHz and lets the phase move, and with no snapshot
    // a proposal leaves after half a period more.
    let every = ((1.0 / dt) / f64::from(settings.send_rate_hz.max(1)))
        .round()
        .max(1.0) as u64;
    let snapshot = with(engine_id, |s| s.snapshot_since);
    if last.is_some_and(|(t, _)| {
        let gap = tick.saturating_sub(t);
        !(snapshot && gap >= (2 * every).div_ceil(3) || gap >= every + every / 2)
    }) {
        return Ok(());
    }
    let v = transform.1.decode(&ctx.get_bytes(e, transform.0)?);
    let rel = vec3(&v["position"]);
    let abs = [rel[0] + origin[0], rel[1] + origin[1], rel[2] + origin[2]];
    let yaw = yaw_of(&v["quaternion"]);
    let jump = with(engine_id, |s| std::mem::take(&mut s.jump_next));
    let vel = match last {
        Some((t, p)) if !jump && tick > t => {
            let secs = (tick - t) as f64 * dt;
            std::array::from_fn(|k| ((abs[k] - p[k]) / secs) as f32)
        }
        _ => [0.0; 3],
    };
    let mut state = AvatarState::from_world(cs, abs, vel, yaw);
    if !avatar.is_empty()
        && let Some(l) = recs.layouts.get(&avatar)
        && let Ok(id) = ctx.id_by_name(&avatar)
        && let Ok(b) = ctx.get_bytes(e, id)
    {
        state.set_fields(&b[..l.size().min(b.len())]);
    }
    with(engine_id, |s| {
        let bytes = s
            .proposer
            .propose(state, jump, s.view.ack(), s.view.ack_seq);
        s.send(bytes);
        s.proposals += 1;
        s.last_sent = Some((tick, abs));
        s.snapshot_since = false;
    });
    Ok(())
}
