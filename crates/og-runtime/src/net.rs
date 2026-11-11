//! Rooms (IDL `platform` room members, docs/design.md §2.8) and lockstep
//! (IDL `net`, §2.10) on the runtime, over og-net.
//!
//! The engine queues room ops for its host (`og_room_take`) and takes the
//! host's events back (`og_room_deliver`): natively the napi addon's relay
//! client (`og run --room`), on the web the shell. Lockstep's own messages
//! (channel `og.lockstep`) act at once, between ticks; what the game sees
//! (og.RoomJoined, og.RoomPeerJoined, og.RoomPeerLeft, og.RoomMessage,
//! og.Desync) is sent by the `og.net` system at the start of the next tick,
//! host input like a key press.
//!
//! The tick: the probe asks [`gate`] before every tick, which holds it
//! until every seated player's input for it is in (and applies a peer's
//! snapshot at a lockstep start or late join); the `og.net` system then
//! writes every og.PlayerInput from the seats' inputs for the tick; after
//! the tick [`after_tick`] sends this peer's input for the tick
//! `inputDelay` on and hashes the world.
//!
//! Rollback (og.Lockstep `mode` rollback): the gate lets a tick run up to
//! `maxPrediction` past the last confirmed one on predicted inputs, and
//! when a real input differs it rewinds the world to this process's own
//! snapshot ([`super::save::rewind`]) and the ticks run again in the same
//! frame. A re-simulated tick sends the game the room events its first run
//! did, and the audio link drops the plays the sink already heard. How far
//! this peer predicts is also capped by what it can re-simulate in half a
//! frame (measured on the wall clock: it decides only when the peer waits,
//! never the confirmed world).

use super::engine::{Engine, engine, running};
use super::parse;
use super::wire::{
    self, LockstepSeat, LockstepStatus, RoomInfo, RoomMessageData, RoomPeer, ToJson,
};
use og_core::IdMap;
use og_ecs::{QueryDesc, SystemDesc, World};
use og_net::protocol::{self, Event, LOCKSTEP_CHANNEL, MAX_MESSAGE, Payload};
use og_net::state::shared_hash;
use og_net::{Config, Gate, Lockstep};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};

/// The system's name in the schedule (front, after og.platform).
pub const NET_SYSTEM: &str = "og.net";
const LOCKSTEP: &str = "og.Lockstep";
const PLAYER_INPUT: &str = "og.PlayerInput";
const JOINED: &str = "og.RoomJoined";
const PEER_JOINED: &str = "og.RoomPeerJoined";
const PEER_LEFT: &str = "og.RoomPeerLeft";
const MESSAGE: &str = "og.RoomMessage";
const DESYNC: &str = "og.Desync";
const SEAT_DROPPED: &str = "og.SeatDropped";
const SEAT_REJOINED: &str = "og.SeatRejoined";
const PEER_STALLED: &str = "og.PeerStalled";
/// Lockstep's seat events: lazy records ([`super::lazy`]), registered once a
/// world runs lockstep in a room, so a game without rooms keeps its hash.
pub(crate) const SEAT_EVENTS: [&str; 3] = [SEAT_DROPPED, SEAT_REJOINED, PEER_STALLED];
/// The messages kept for og.platform.roomMessage.
const KEEP_MESSAGES: usize = 256;
/// The wire form of roomSend's bytes: `{ "$og.bytes": "<base64>" }`.
pub const BYTES_KEY: &str = "$og.bytes";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum Stage {
    #[default]
    None,
    Joining,
    In,
}

/// What re-simulating costs this peer, on the wall clock (ms).
#[derive(Default)]
struct Cost {
    /// A tick with what follows it, and a rewind, smoothed.
    tick_ms: f64,
    rewind_ms: f64,
    /// Of `tick_ms`, what follows the tick: the world's snapshot, its hash
    /// and keeping both ([`after_tick`]).
    keep_ms: f64,
    /// When the tick now running started.
    started: Option<f64>,
}

impl Cost {
    fn note(v: &mut f64, ms: f64) {
        *v = if *v == 0.0 { ms } else { *v * 0.9 + ms * 0.1 };
    }

    /// The ticks this peer can re-simulate after a rewind in half a frame.
    fn budget(&self, dt: f64) -> u32 {
        if self.tick_ms <= 0.0 {
            return u32::MAX;
        }
        let half = dt * 500.0;
        (((half - self.rewind_ms) / self.tick_ms).floor()).max(1.0) as u32
    }
}

/// A play a replayed tick's first run did not send is still worth sending
/// this late (the link's `late`, in ms).
const LATE_PLAY_MS: u64 = 100;

struct Message {
    id: u32,
    from: u32,
    tick: Option<u64>,
    payload: Payload,
}

#[derive(Default)]
pub(crate) struct Net {
    host: String,
    /// The room a Play link named (`?room=<id>`, og_room_set_host's `join`),
    /// once the engine asked to join it.
    link: String,
    /// A Play link refused for want of a room host: told the game once, and
    /// joined when a host that carries rooms arrives.
    link_waiting: String,
    stage: Stage,
    room: String,
    peer: u32,
    peers: IdMap<u32, String>,
    outbox: Vec<Value>,
    /// The game's events waiting for the next tick: (event, JSON value).
    pending: Vec<(&'static str, Value)>,
    messages: VecDeque<Message>,
    next_message: u32,
    ls: Lockstep,
    /// The game's events each recent tick's first run delivered, so a
    /// re-simulated tick delivers them again.
    delivered: BTreeMap<u64, Vec<(&'static str, Value)>>,
    cost: Cost,
    shape: Shape,
    /// This peer's commands given this tick (og.net.command, packed).
    given: Vec<Vec<u8>>,
    /// Outside a run: the local player's commands for a tick, given the
    /// tick before (og.net.commands reads them as seat 0's).
    local_commands: (u64, Vec<Vec<u8>>),
}

/// og.net.shape: the game's input record and its commands.
#[derive(Clone, Debug, Default)]
struct Shape {
    /// The input component (empty: og.PlayerInput).
    input: String,
    /// The command component (empty: no commands) and its packed size.
    command: String,
    command_bytes: usize,
    max_commands: u32,
    /// The input record's packed size, when there are commands (og-net's
    /// state_bytes: a prediction repeats these bytes alone).
    state_bytes: usize,
    /// The two layouts' fingerprint (0 for the default shape).
    fingerprint: u64,
}

impl Shape {
    fn input(&self) -> &str {
        if self.input.is_empty() {
            PLAYER_INPUT
        } else {
            &self.input
        }
    }
}

/// og.net on one engine: its room, lockstep session and messages.
pub(crate) type ModuleState = RefCell<Net>;

fn with<T>(engine: u32, f: impl FnOnce(&mut Net) -> T) -> T {
    match super::engine::engine(engine) {
        Ok(eng) => f(&mut eng.modules.net.borrow_mut()),
        // An engine that is gone, or not made yet, has none: a fresh one.
        Err(_) => f(&mut Net::default()),
    }
}

/// Lockstep's messages become `send` ops; its notes go to the engine's log.
fn flush(engine: u32, n: &mut Net, tick: u64) {
    for (to, data, unreliable) in n.ls.take_out() {
        let mut op = protocol::send(to, Some(LOCKSTEP_CHANNEL), &Payload::Json(data));
        if unreliable {
            op["unreliable"] = json!(true);
        }
        n.outbox.push(op);
    }
    let notes = n.ls.take_notes();
    if notes.is_empty() {
        return;
    }
    if let Ok(eng) = engine_of(engine)
        && let Ok(mut st) = eng.state.try_borrow_mut()
    {
        for text in notes {
            st.log
                .push(json!({ "tick": tick, "warning": text.contains("DESYNC"), "text": text }));
        }
    }
}

fn engine_of(id: u32) -> Result<std::rc::Rc<Engine>, String> {
    engine(id)
}

fn tick_of(engine: u32) -> u64 {
    if let Some(f) = running(engine) {
        // SAFETY: the frame is published while its system runs.
        return unsafe { f.ctx() }.time().tick;
    }
    engine_of(engine)
        .and_then(|e| e.world().map(|w| w.time().tick))
        .unwrap_or(0)
}

// ── lifecycle (the host's side) ──────────────────────────────────────────

/// `og_room_take`: the ops since the last call, oldest first.
pub fn og_room_take(engine: u32) -> String {
    with(engine, |n| {
        Value::Array(std::mem::take(&mut n.outbox)).to_string()
    })
}

/// `og_room_set_host`: what carries the rooms, and with `join` the room a
/// Play link (`?room=<id>`) puts this engine in. The engine joins it itself
/// (once: a link is not joined again after a leave), so the game hears the
/// same og.RoomJoined as for its own roomJoin, and a roomJoin of its own
/// meanwhile is refused as one already being joined. Without a host, or
/// with a malformed id, og.RoomJoined says why.
pub fn og_room_set_host(engine: u32, info: String) {
    let v: Value = serde_json::from_str(&info).unwrap_or(Value::Null);
    let host = v["host"].as_str().unwrap_or("none").to_string();
    let join = v["join"].as_str().unwrap_or("").to_string();
    with(engine, |n| {
        n.host = host;
        if join.is_empty() || n.link == join {
            return;
        }
        n.link = join.clone();
        let refused = |reason: String| {
            (
                JOINED,
                json!({ "ok": false, "room": "", "peer": 0, "peers": 0, "reason": reason }),
            )
        };
        if !protocol::valid_room(&join) {
            n.pending.push(refused(format!(
                "the Play link's room '{join}' is not a room id: 1 to 64 of letters, digits, '.', '_' and '-'"
            )));
            return;
        }
        if n.host.is_empty() || n.host == "none" {
            n.link.clear();
            if n.link_waiting != join {
                n.link_waiting = join.clone();
                n.pending.push(refused(format!(
                    "the Play link's room '{join}': this page carries no rooms (the platform's Player answers room.join)"
                )));
            }
            return;
        }
        match can_join(n) {
            Ok(()) => {
                n.stage = Stage::Joining;
                n.outbox.push(protocol::join(&join, ""));
            }
            Err(e) => n
                .pending
                .push(refused(format!("the Play link's room '{join}': {e}"))),
        }
    });
}

/// `og_room_deliver`: one event from the host.
pub fn og_room_deliver(engine: u32, event: String) -> bool {
    let Ok(v) = serde_json::from_str::<Value>(&event) else {
        return false;
    };
    let Ok(e) = Event::parse(&v) else {
        return false;
    };
    let tick = tick_of(engine);
    with(engine, |n| {
        n.ls.set_clock(super::probe::now_ms() as u64);
        n.ls.on_event(&e);
        match e {
            Event::Joined { room, peer, peers } => {
                n.stage = Stage::In;
                n.room = room.clone();
                n.peer = peer;
                n.peers = peers.into_iter().collect();
                n.peers.entry(peer).or_default();
                let count = n.peers.len();
                n.pending.push((
                    JOINED,
                    json!({ "ok": true, "room": room, "peer": peer, "peers": count, "reason": "" }),
                ));
            }
            Event::PeerJoined { peer, name, .. } => {
                n.peers.insert(peer, name);
                let count = n.peers.len();
                n.pending
                    .push((PEER_JOINED, json!({ "peer": peer, "peers": count })));
            }
            Event::PeerLeft { peer } => {
                n.peers.remove(&peer);
                let count = n.peers.len();
                n.pending
                    .push((PEER_LEFT, json!({ "peer": peer, "peers": count })));
            }
            Event::Left { .. } => {
                if n.stage == Stage::Joining {
                    n.pending.push((
                        JOINED,
                        json!({ "ok": false, "room": "", "peer": 0, "peers": 0, "reason": "cancelled: roomLeave while joining" }),
                    ));
                }
                let was = (n.stage == Stage::In).then_some(n.peer);
                n.stage = Stage::None;
                n.room.clear();
                n.peers.clear();
                if let Some(peer) = was {
                    n.pending
                        .push((PEER_LEFT, json!({ "peer": peer, "peers": 0 })));
                }
            }
            Event::Error { op, code, message } => {
                if matches!(op.as_str(), "create" | "join" | "quick") && n.stage != Stage::In {
                    n.stage = Stage::None;
                    n.pending.push((
                        JOINED,
                        json!({ "ok": false, "room": "", "peer": 0, "peers": 0, "reason": format!("{code}: {message}") }),
                    ));
                } else if let Ok(eng) = engine_of(engine)
                    && let Ok(mut st) = eng.state.try_borrow_mut()
                {
                    st.log.push(json!({ "tick": tick, "warning": true, "text": format!("room {op}: {code}: {message}") }));
                }
            }
            Event::Message {
                from,
                channel,
                payload,
            } => {
                if !channel.as_deref().is_some_and(|c| c.starts_with("og.")) {
                    n.next_message = n.next_message.wrapping_add(1).max(1);
                    let id = n.next_message;
                    let binary = matches!(payload, Payload::Bytes(_));
                    n.messages.push_back(Message {
                        id,
                        from,
                        tick: None,
                        payload,
                    });
                    while n.messages.len() > KEEP_MESSAGES {
                        n.messages.pop_front();
                    }
                    n.pending.push((
                        MESSAGE,
                        json!({ "from": from, "message": id, "binary": binary }),
                    ));
                }
            }
        }
        flush(engine, n, tick);
    });
    true
}

// ── og.platform's room members ───────────────────────────────────────────

fn can_join(n: &Net) -> Result<(), String> {
    if n.host.is_empty() || n.host == "none" {
        return Err("no rooms here: natively start the game with `og run --room ws://127.0.0.1:<port>/<room>` beside `og relay`; on the web the platform's rooms answer (og.platform.roomInfo's host)".into());
    }
    match n.stage {
        Stage::None => Ok(()),
        Stage::Joining => Err("a room is being joined already: wait for og.RoomJoined".into()),
        Stage::In => Err(format!("already in room '{}': roomLeave first", n.room)),
    }
}

pub fn og_platform_room_create(world: u32, desc: Option<String>) -> Result<(), String> {
    let d: super::wire::RoomDesc = super::wire::arg(desc.as_deref(), "desc")?;
    let name = d.name;
    if !name.is_empty() && !protocol::valid_room(&name) {
        return Err(format!(
            "'{name}' is not a room name: at most 64 of letters, digits, '.', '_' and '-'"
        ));
    }
    let capacity = d.capacity.clamp(1, protocol::MAX_CAPACITY);
    with(world, |n| match can_join(n) {
        Ok(()) => {
            n.stage = Stage::Joining;
            n.outbox.push(protocol::create(&name, capacity));
            Ok(())
        }
        Err(e) => Err(e),
    })
}

pub fn og_platform_quick_play(world: u32, desc: Option<String>) -> Result<(), String> {
    let d: super::wire::QuickPlayDesc = super::wire::arg(desc.as_deref(), "desc")?;
    let (mode, name) = (d.mode, d.name);
    if !mode.is_empty() && (!protocol::valid_room(&mode) || mode.len() > 32) {
        return Err(format!(
            "'{mode}' is not a quick play mode: at most 32 of letters, digits, '.', '_' and '-'"
        ));
    }
    if !name.is_empty() && !protocol::valid_room(&name) {
        return Err(format!(
            "'{name}' is not a player name: at most 64 of letters, digits, '.', '_' and '-'"
        ));
    }
    let players = d.players.clamp(1, protocol::MAX_CAPACITY);
    let timeout = d.timeout_ms.min(protocol::QUICK_TIMEOUT_MAX_MS);
    with(world, |n| match can_join(n) {
        Ok(()) => {
            n.stage = Stage::Joining;
            n.outbox
                .push(protocol::quick(&mode, players, timeout, &name));
            Ok(())
        }
        Err(e) => Err(e),
    })
}

pub fn og_platform_room_join(world: u32, room: String) -> Result<(), String> {
    if !protocol::valid_room(&room) {
        return Err(format!(
            "'{room}' is not a room id: 1 to 64 of letters, digits, '.', '_' and '-'"
        ));
    }
    with(world, |n| match can_join(n) {
        Ok(()) => {
            n.stage = Stage::Joining;
            n.outbox.push(protocol::join(&room, ""));
            Ok(())
        }
        Err(e) => Err(e),
    })
}

pub fn og_platform_room_leave(world: u32) -> Result<(), String> {
    with(world, |n| {
        if n.stage == Stage::None {
            return Err("in no room".into());
        }
        n.outbox.push(protocol::leave());
        Ok(())
    })
}

pub fn og_platform_room_send(world: u32, data: String, to: Option<u32>) -> Result<(), String> {
    let v = parse(Some(&data))?;
    let payload = match v.get(BYTES_KEY).and_then(Value::as_str) {
        Some(b) => match og_net::base64::decode(b) {
            Some(bytes) => Payload::Bytes(bytes),
            None => return Err("the message's bytes are not base64".into()),
        },
        None => Payload::Json(v),
    };
    if payload.size() > MAX_MESSAGE {
        return Err(format!(
            "a message is at most {MAX_MESSAGE} bytes encoded; this one is {}",
            payload.size()
        ));
    }
    with(world, |n| {
        if n.stage != Stage::In {
            return Err(
                "in no room: roomJoin or roomCreate first, and wait for og.RoomJoined".into(),
            );
        }
        if let Some(t) = to
            && !n.peers.contains_key(&t)
        {
            return Err(format!("no peer {t} in room '{}'", n.room));
        }
        n.outbox.push(protocol::send(to, None, &payload));
        Ok(())
    })
}

/// The room's peers; none outside a room.
fn room_peers(n: &Net) -> Vec<RoomPeer> {
    if n.stage != Stage::In {
        return Vec::new();
    }
    n.peers
        .iter()
        .map(|(p, name)| RoomPeer {
            peer: *p,
            name: name.clone(),
            local: *p == n.peer,
        })
        .collect()
}

pub fn og_platform_room_peers(world: u32) -> Vec<RoomPeer> {
    with(world, |n| room_peers(n))
}

pub fn og_platform_room_info(world: u32) -> RoomInfo {
    with(world, |n| {
        let state = match n.stage {
            Stage::None => "none",
            Stage::Joining => "joining",
            Stage::In => "in",
        };
        let host = if n.host.is_empty() { "none" } else { &n.host };
        let inside = n.stage == Stage::In;
        RoomInfo {
            state: state.to_string(),
            room: if inside {
                n.room.clone()
            } else {
                String::new()
            },
            host: host.to_string(),
            link: n.link.clone(),
            peer: if inside { n.peer } else { 0 },
            peers: room_peers(n),
        }
    })
}

pub fn og_platform_room_message(world: u32, message: u32) -> Option<RoomMessageData> {
    with(world, |n| {
        let m = n.messages.iter().find(|m| m.id == message)?;
        let (data, bytes) = match &m.payload {
            Payload::Json(d) => (d.clone(), None),
            Payload::Bytes(b) => (Value::Null, Some(json!(og_net::base64::encode(b)))),
        };
        Some(RoomMessageData {
            message: m.id,
            from: m.from,
            tick: m.tick.unwrap_or(0) as u32,
            data,
            bytes,
        })
    })
}

// ── og.net ───────────────────────────────────────────────────────────────

pub fn og_net_submit(world: u32, input: Option<String>) -> Result<(), String> {
    let d = parse(input.as_deref())?;
    let i: super::wire::InputDesc = super::wire::value(&d, "input")?;
    // Only the fields given are written; the rest keep og.PlayerInput's defaults.
    let mut fields = serde_json::Map::new();
    for (k, v) in [
        ("buttons", json!(i.buttons)),
        ("x", json!(i.x)),
        ("y", json!(i.y)),
        ("aimX", json!(i.aim_x)),
        ("aimY", json!(i.aim_y)),
    ] {
        if d.get(k).is_none_or(Value::is_null) {
            continue;
        }
        if !v.as_f64().is_some_and(f64::is_finite) {
            return Err(format!("{k} must be a finite number, got {}", d[k]));
        }
        fields.insert(k.into(), v);
    }
    if let Some(k) = d.as_object().and_then(|o| {
        o.keys()
            .find(|k| !matches!(k.as_str(), "buttons" | "x" | "y" | "aimX" | "aimY"))
    }) {
        return Err(format!(
            "og.net.submit takes buttons, x, y, aimX and aimY; not '{k}'"
        ));
    }
    if let Some(input) = with(world, |n| {
        (!n.shape.input.is_empty()).then(|| n.shape.input.clone())
    }) {
        return Err(format!(
            "og.net.submit gives og.PlayerInput; og.net.shape named '{input}': og.net.submitInput(world, {input}, {{ ... }})"
        ));
    }
    let layout = match engine_of(world)
        .ok()
        .and_then(|e| e.state.borrow().layouts.get(PLAYER_INPUT).cloned())
    {
        Some(l) => l,
        None => return Err("og.PlayerInput is not registered in this world".into()),
    };
    let bytes = layout.encode(None, Some(&Value::Object(fields)))?;
    with(world, |n| n.ls.set_local(bytes));
    Ok(())
}

/// The zero value of the input record `name` (its defaults), packed.
fn state_zero(engine: u32, name: &str) -> Vec<u8> {
    engine_of(engine)
        .ok()
        .and_then(|e| e.state.borrow().layouts.get(name).cloned())
        .and_then(|l| l.encode(None, None).ok())
        .unwrap_or_default()
}

/// A token's name: `{ kind, name }` (a token as JSON) or the name itself.
fn token_name(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Object(o) => o.get("name").and_then(Value::as_str).map(str::to_string),
        _ => None,
    }
}

/// A game's own component that og.net.shape may name: registered, not the
/// engine's, with a u32 `player` field.
fn shape_layout(eng: &Engine, name: &str, what: &str) -> Result<super::codec::Layout, String> {
    let layout = eng
        .state
        .borrow()
        .layouts
        .get(name)
        .cloned()
        .filter(|l| l.kind == super::codec::RecordKind::Component)
        .ok_or_else(|| format!("og.net.shape's {what}: '{name}' is not a registered component"))?;
    if name.starts_with("og.") {
        return Err(format!(
            "og.net.shape's {what}: '{name}' is the engine's; name a component of the game's own (defineComponent)"
        ));
    }
    match layout.field("player") {
        Some((_, f)) if f.kind == super::codec::Kind::U32 => Ok(layout),
        _ => Err(format!(
            "og.net.shape's {what}: '{name}' needs a `player: u32()` field (the seat it carries)"
        )),
    }
}

/// FNV-1a over the shape's names and layouts: equal on two peers exactly
/// when their inputs are read the same way.
fn fingerprint(parts: &[&super::codec::Layout], max: u32) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |b: &[u8]| {
        for x in b {
            h = (h ^ u64::from(*x)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for l in parts {
        eat(l.schema.as_bytes());
        for f in l.fields.iter() {
            eat(f.name.as_bytes());
            eat(format!("{:?}", f.ty).as_bytes());
        }
        eat(b";");
    }
    eat(&max.to_le_bytes());
    h.max(1)
}

pub fn og_net_shape(world: u32, desc: Option<String>) -> Result<(), String> {
    let d: super::wire::NetShapeDesc = super::wire::arg(desc.as_deref(), "desc")?;
    let eng = engine_of(world)?;
    let w = match eng.world() {
        Ok(w) => w,
        Err(_) => {
            return Err(
                "og.net.shape is called outside a system (setup or start), before the room starts"
                    .into(),
            );
        }
    };
    if with(world, |n| n.ls.running()) {
        return Err("og.net.shape: lockstep runs; set the shape before the room starts".into());
    }
    let name = |k: &str, v: &Option<Value>| -> Result<Option<String>, String> {
        match v {
            None => Ok(None),
            Some(v) => token_name(v)
                .map(Some)
                .ok_or_else(|| format!("og.net.shape's {k} is a component token, got {v}")),
        }
    };
    let (input, command) = match (name("input", &d.input), name("command", &d.command)) {
        (Ok(i), Ok(c)) => (i.unwrap_or_default(), c.unwrap_or_default()),
        (Err(e), _) | (_, Err(e)) => return Err(e),
    };
    let input = if input == PLAYER_INPUT {
        String::new()
    } else {
        input
    };
    let max_commands = d.max_commands.clamp(1, 256);
    let input_layout = if input.is_empty() {
        eng.state.borrow().layouts.get(PLAYER_INPUT).cloned()
    } else {
        {
            let l = shape_layout(&eng, &input, "input")?;
            Some(l)
        }
    };
    let Some(input_layout) = input_layout else {
        return Err("og.PlayerInput is not registered in this world".into());
    };
    let command_layout = if command.is_empty() {
        None
    } else {
        {
            let l = shape_layout(&eng, &command, "command")?;
            Some(l)
        }
    };
    if !input.is_empty()
        && let Err(e) = w.grant(NET_SYSTEM, &[input.as_str()])
    {
        return Err(e.reason().into());
    }
    let shape = Shape {
        fingerprint: if input.is_empty() && command.is_empty() {
            0
        } else {
            let mut parts = vec![&input_layout];
            parts.extend(command_layout.as_ref());
            fingerprint(&parts, max_commands)
        },
        state_bytes: if command.is_empty() {
            0
        } else {
            input_layout.size()
        },
        command_bytes: command_layout.as_ref().map_or(0, |l| l.size()),
        input,
        command,
        max_commands,
    };
    with(world, |n| {
        if n.shape.input != shape.input {
            n.ls.set_local(Vec::new());
        }
        n.shape = shape;
        n.given.clear();
    });
    Ok(())
}

pub fn og_net_submit_input(world: u32, component: String, value: String) -> Result<(), String> {
    let (shape, eng) = {
        let e = engine_of(world)?;
        (with(world, |n| n.shape.clone()), e)
    };
    if shape.input.is_empty() {
        return Err("og.net.submitInput needs og.net.shape's input; og.PlayerInput's is given with og.net.submit".into());
    }
    let name =
        token_name(&serde_json::from_str(&component).unwrap_or(Value::String(component.clone())))
            .unwrap_or(component);
    if name != shape.input {
        return Err(format!(
            "og.net.submitInput takes '{}', og.net.shape's input; not '{name}'",
            shape.input
        ));
    }
    let v = parse(Some(&value))?;
    if v.get("player").is_some() || v.get("tick").is_some() {
        return Err("og.net.submitInput: `player` and `tick` are the engine's".into());
    }
    let Some(layout) = eng.state.borrow().layouts.get(&name).cloned() else {
        return Err(format!("'{name}' is not registered in this world"));
    };
    match layout.encode(None, Some(&v)) {
        Ok(bytes) => {
            with(world, |n| n.ls.set_local(bytes));
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn og_net_command(world: u32, component: String, value: String) -> Result<(), String> {
    let eng = engine_of(world)?;
    let shape = with(world, |n| n.shape.clone());
    let name =
        token_name(&serde_json::from_str(&component).unwrap_or(Value::String(component.clone())))
            .unwrap_or(component);
    if shape.command.is_empty() {
        return Err(
            "og.net.command needs a command component: og.net.shape(world, { command: MyCommand })"
                .into(),
        );
    }
    if name != shape.command {
        return Err(format!(
            "og.net.command takes '{}', og.net.shape's command; not '{name}'",
            shape.command
        ));
    }
    let v = parse(Some(&value))?;
    if v.get("player").is_some() {
        return Err("og.net.command: `player` is the engine's (the seat that gave it)".into());
    }
    let Some(layout) = eng.state.borrow().layouts.get(&name).cloned() else {
        return Err(format!("'{name}' is not registered in this world"));
    };
    let bytes = layout.encode(None, Some(&v))?;
    let tick = tick_of(world);
    with(world, |n| {
        // A re-simulated tick's commands went out on its first run.
        if n.ls.resimulating(tick) {
            return Ok(());
        }
        if n.given.len() >= shape.max_commands as usize {
            return Err(format!(
                "og.net.command: {} commands this tick already (og.net.shape's maxCommands)",
                shape.max_commands
            ));
        }
        n.given.push(bytes);
        Ok(())
    })
}

pub fn og_net_commands(world: u32, component: String) -> Result<serde_json::Value, String> {
    let (shape, eng) = {
        let e = engine_of(world)?;
        (with(world, |n| n.shape.clone()), e)
    };
    let name =
        token_name(&serde_json::from_str(&component).unwrap_or(Value::String(component.clone())))
            .unwrap_or(component);
    if shape.command.is_empty() {
        return Ok(json!([]));
    }
    if name != shape.command {
        return Err(format!(
            "og.net.commands reads '{}', og.net.shape's command; not '{name}'",
            shape.command
        ));
    }
    let Some(layout) = eng.state.borrow().layouts.get(&name).cloned() else {
        return Err(format!("'{name}' is not registered in this world"));
    };
    let tick = tick_of(world);
    let size = shape.command_bytes.max(1);
    let by_seat: Vec<(u32, Vec<Vec<u8>>)> = with(world, |n| {
        if !n.ls.running() {
            return if n.local_commands.0 == tick {
                vec![(0, n.local_commands.1.clone())]
            } else {
                Vec::new()
            };
        }
        (0..n.ls.players().unwrap_or(0))
            .map(|seat| {
                let cmds =
                    n.ls.input(tick, seat)
                        .and_then(|b| b.get(shape.state_bytes..))
                        .unwrap_or_default();
                let list = cmds
                    .chunks_exact(size)
                    .take(shape.max_commands as usize)
                    .map(<[u8]>::to_vec)
                    .collect();
                (seat, list)
            })
            .collect()
    });
    let mut out = Vec::new();
    for (seat, list) in by_seat {
        for c in list {
            {
                let b = layout.encode(Some(&c), Some(&json!({ "player": seat })))?;
                out.push(layout.decode(&b))
            }
        }
    }
    Ok(Value::Array(out))
}

pub fn og_net_status(world: u32) -> LockstepStatus {
    with(world, |n| {
        let s = n.ls.status();
        // og-net's status, read field by field into the IDL's type: an
        // absent or `null` value is the field's default.
        let seats = s["seats"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|h| LockstepSeat {
                        seat: read(&h[0]),
                        peer: read(&h[1]),
                        since: read(&h[2]),
                        until: read(&h[3]),
                        left: read(&h[4]),
                    })
                    .collect()
            })
            .unwrap_or_default();
        LockstepStatus {
            running: read(&s["running"]),
            peer: read(&s["peer"]),
            peers: read(&s["peers"]),
            start_tick: read(&s["startTick"]),
            players: read(&s["players"]),
            seats,
            spectators: read(&s["spectators"]),
            waiting_for: read(&s["waitingFor"]),
            waits: read(&s["waits"]),
            desync: read(&s["desync"]["tick"]),
            rollback: read(&s["rollback"]),
            max_prediction: read(&s["maxPrediction"]),
            confirmed_tick: read(&s["confirmedTick"]),
            final_tick: read(&s["finalTick"]),
            acknowledged: read(&s["acknowledged"]),
            predicted: read(&s["predicted"]),
            rollbacks: read(&s["rollbacks"]),
            resimulated_ticks: read(&s["resimulatedTicks"]),
            snapshot_bytes: read(&s["snapshotBytes"]),
            stalled: read(&s["stalled"]),
            drops: read(&s["drops"]),
        }
    })
}

/// `v` read as `T`; `T`'s default when it is absent, `null` or of
/// another type.
fn read<T: wire::FromJson + Default>(v: &Value) -> T {
    wire::value(v, "").unwrap_or_default()
}

pub fn og_net_resimulating(world: u32) -> bool {
    with(world, |n| n.ls.resimulating(tick_of(world)))
}

pub fn og_net_final_hash(world: u32, tick: f64) -> Option<String> {
    with(world, |n| {
        let h = (tick >= 0.0 && tick.fract() == 0.0)
            .then(|| n.ls.final_hash(tick as u64))
            .flatten();
        h.map(|h| format!("{h:016x}"))
    })
}

/// The world's snapshot bytes (og-ecs's format), mid-tick or between
/// ticks: what og.net hashes, and `og replay` (og-host-node) hashes and diffs.
pub fn world_snapshot_bytes(world: u32) -> Result<Vec<u8>, String> {
    if let Some(f) = running(world) {
        // SAFETY: the frame is published while its system runs.
        return Ok(unsafe { f.ctx() }.snapshot());
    }
    engine_of(world).and_then(|e| e.world().map(|w| w.snapshot()))
}

/// The hash as text, or `error: <reason>`; the host writes it as a JSON
/// string.
pub fn og_net_hash(world: u32) -> String {
    match world_snapshot_bytes(world).and_then(|bytes| shared_hash(&bytes)) {
        Ok(h) => format!("{h:016x}"),
        Err(e) => format!("error: {e}"),
    }
}

// ── the tick ─────────────────────────────────────────────────────────────

/// og.Lockstep's settings as the world holds them.
fn config(eng: &Engine, world: &World) -> Config {
    let layout = eng.state.borrow().layouts.get(LOCKSTEP).cloned();
    let v = layout.and_then(|l| {
        let id = world.id_by_name(LOCKSTEP).ok()?;
        Some(l.decode(&world.resource_bytes(id).ok()?))
    });
    let Some(v) = v else {
        return Config::default();
    };
    let u = |k: &str, d: u32| v[k].as_u64().map_or(d, |x| x as u32);
    Config {
        enabled: v["enabled"].as_bool().unwrap_or(false),
        players: u("players", 2).clamp(1, 16),
        input_delay: u("inputDelay", 2).clamp(1, 30),
        hash_every: u("hashEvery", 10),
        rollback: v["mode"] == json!("rollback"),
        max_prediction: u("maxPrediction", 8).clamp(1, 30),
        stall_timeout: u("stallTimeout", 5000).min(600_000),
        rejoin: v["rejoin"].as_bool().unwrap_or(true),
        budget: with(eng.id, |n| n.cost.budget(f64::from(world.time().dt))),
        state_bytes: with(eng.id, |n| n.shape.state_bytes as u32),
        shape: with(eng.id, |n| n.shape.fingerprint),
    }
}

/// The probe's tick gate: whether the next tick may run.
/// Whether lockstep (or rollback) runs on this engine now: its world is
/// shared with the room's other peers.
pub(crate) fn session_running(engine: u32) -> bool {
    super::engine::engine(engine)
        .is_ok_and(|eng| eng.modules.net.try_borrow().is_ok_and(|n| n.ls.running()))
}

pub(crate) fn gate(engine: u32, world: &mut World) -> bool {
    if !with(engine, |n| n.ls.me().is_some()) {
        return true;
    }
    let Ok(eng) = engine_of(engine) else {
        return true;
    };
    let cfg = config(&eng, world);
    if cfg.enabled {
        // Before the start's snapshot, so every peer's world holds them.
        for name in SEAT_EVENTS {
            if let Err(e) = super::lazy::ensure(&eng, world, name) {
                eng.state
                    .borrow_mut()
                    .log
                    .push(json!({ "tick": world.time().tick, "warning": true, "text": e }));
            }
        }
    }
    // A run that could not stay in step is refused before it starts: the
    // room stays a lobby, and the game hears why once.
    if with(engine, |n| n.ls.would_start(&cfg))
        && let Some(why) = super::stream::cells::session_refusal(&eng, world)
    {
        let text = format!("lockstep: not started: {why}");
        let mut st = eng.state.borrow_mut();
        if st.ui_warned.insert(text.clone()) {
            st.log
                .push(json!({ "tick": world.time().tick, "warning": true, "text": text }));
        }
        return true;
    }
    loop {
        let tick = world.time().tick;
        let gate = with(engine, |n| {
            n.ls.set_clock(super::probe::now_ms() as u64);
            let g = n.ls.gate(&cfg, tick, &mut || world.snapshot());
            let players = n.ls.players().unwrap_or(0);
            for (peer, seat, ms) in n.ls.take_stalls() {
                n.pending.push((
                    PEER_STALLED,
                    json!({ "peer": peer, "seat": seat.unwrap_or(players), "silentMs": ms.min(u64::from(u32::MAX)) }),
                ));
            }
            flush(engine, n, tick);
            g
        });
        match gate {
            Gate::Go => {
                with(engine, |n| n.cost.started = Some(super::probe::now_ms()));
                return true;
            }
            Gate::Wait => return false,
            Gate::Rewind { snapshot, tick: to } => {
                let t0 = super::probe::now_ms();
                if let Err(e) = super::save::rewind(&eng, world, &snapshot, to, &[LOCKSTEP]) {
                    eng.state.borrow_mut().log.push(json!({
                        "tick": tick, "warning": true,
                        "text": format!("rollback: the rewind to tick {to} failed: {e}; this peer leaves the room"),
                    }));
                    with(engine, |n| n.outbox.push(protocol::leave()));
                    return true;
                }
                if let Some(link) = og_audio::link(engine) {
                    let tick_ms = (f64::from(world.time().dt) * 1000.0).max(1.0);
                    link.rewound(to, tick, (LATE_PLAY_MS as f64 / tick_ms).ceil() as u64);
                }
                let ms = super::probe::now_ms() - t0;
                with(engine, |n| Cost::note(&mut n.cost.rewind_ms, ms));
            }
            Gate::Restore { snapshot, .. } => {
                if let Err(e) = super::save::restore_session(&eng, world, &snapshot, &[LOCKSTEP]) {
                    eng.state.borrow_mut().log.push(json!({
                        "tick": tick, "warning": true,
                        "text": format!("lockstep: the peer's snapshot was refused: {e}; this peer leaves lockstep"),
                    }));
                    with(engine, |n| n.outbox.push(protocol::leave()));
                    return true;
                }
            }
        }
    }
}

/// After every tick: this peer's input for the tick inputDelay on, and the
/// world's hash.
pub(crate) fn after_tick(engine: u32, world: &mut World) {
    if !with(engine, |n| n.ls.running()) {
        return;
    }
    let tick = world.time().tick;
    with(engine, |n| {
        let ran = super::probe::now_ms();
        // This tick's commands go with this peer's input inputDelay on,
        // after its state (whole, so the commands start where every peer
        // reads them).
        let given = std::mem::take(&mut n.given);
        if !given.is_empty() {
            if n.ls.local().len() != n.shape.state_bytes {
                let zero = state_zero(engine, n.shape.input());
                n.ls.set_local(zero);
            }
            for c in &given {
                n.ls.push_commands(c);
            }
        }
        n.ls.after_tick(tick, &mut || {
            let bytes = world.snapshot();
            (shared_hash(&bytes).unwrap_or(0), bytes)
        });
        // A re-simulated tick pays for its snapshot and hash too, so the
        // budget counts them.
        let kept = super::probe::now_ms();
        Cost::note(&mut n.cost.keep_ms, kept - ran);
        if let Some(t0) = n.cost.started.take() {
            Cost::note(&mut n.cost.tick_ms, kept - t0);
        }
        flush(engine, n, tick);
    });
}

fn send(
    ctx: &og_ecs::SystemContext<'_>,
    layouts: &BTreeMap<String, super::codec::Layout>,
    name: &str,
    v: &Value,
) -> og_core::Result<()> {
    let Some(layout) = layouts.get(name) else {
        return Ok(());
    };
    let bytes = layout
        .encode(None, Some(v))
        .map_err(|e| og_core::Error::new(og_core::ErrorKind::Invalid, e))?;
    ctx.send_bytes(ctx.id_by_name(name)?, &bytes)
}

/// Mounts the og.net system (front, after og.platform).
pub(crate) fn mount(world: &mut World, mount: super::engine::Mount<'_>) -> Result<(), String> {
    let engine_id = mount.id;
    let mut desc = SystemDesc::new(NET_SYSTEM);
    for w in [
        JOINED,
        PEER_JOINED,
        PEER_LEFT,
        MESSAGE,
        DESYNC,
        LOCKSTEP,
        PLAYER_INPUT,
    ] {
        desc.writes.push(w.to_string());
    }
    desc.queries.push(QueryDesc {
        write: vec![PLAYER_INPUT.to_string()],
        ..QueryDesc::default()
    });
    world
        .add_system(desc, move |ctx| {
            let tick = ctx.time().tick;
            let Ok(eng) = engine(engine_id) else {
                return Ok(());
            };
            let layouts: BTreeMap<String, super::codec::Layout> = {
                let st = eng.state.borrow();
                let rows = [
                    JOINED,
                    PEER_JOINED,
                    PEER_LEFT,
                    MESSAGE,
                    DESYNC,
                    LOCKSTEP,
                    PLAYER_INPUT,
                    SEAT_DROPPED,
                    SEAT_REJOINED,
                    PEER_STALLED,
                ]
                .iter()
                .filter_map(|n| st.layouts.get(*n).map(|l| (n.to_string(), l.clone())));
                og_core::sort::map(rows)
            };
            let (events, desync, running, seat, players, local, seats, input) =
                with(engine_id, |n| {
                    for m in n.messages.iter_mut().filter(|m| m.tick.is_none()) {
                        m.tick = Some(tick);
                    }
                    // Outside a run, the commands given last tick are this one's.
                    if !n.ls.running() {
                        n.local_commands = (tick, std::mem::take(&mut n.given));
                    }
                    // A re-simulated tick hears what its first run heard; a new
                    // tick takes what arrived, kept while it may run again.
                    let events = if n.ls.resimulating(tick) {
                        n.delivered.get(&tick).cloned().unwrap_or_default()
                    } else {
                        let e = std::mem::take(&mut n.pending);
                        if n.ls.running() {
                            if !e.is_empty() {
                                n.delivered.insert(tick, e.clone());
                            }
                            let floor = n.ls.final_tick().unwrap_or(tick);
                            n.delivered.retain(|t, _| *t >= floor);
                        }
                        e
                    };
                    (
                        events,
                        n.ls.take_desync(),
                        n.ls.running(),
                        n.ls.seat(tick),
                        n.ls.players(),
                        n.ls.local().to_vec(),
                        n.ls.seat_events(tick),
                        n.shape.input().to_string(),
                    )
                });
            for (name, v) in &events {
                send(ctx, &layouts, name, v)?;
            }
            // The seats changing hands now: from the agreed seats, so a
            // re-simulated tick sends them again and every peer at one tick.
            for e in seats {
                let (name, v) = if e.dropped {
                    (
                        SEAT_DROPPED,
                        json!({ "seat": e.seat, "peer": e.peer, "tick": tick, "left": e.left }),
                    )
                } else {
                    (
                        SEAT_REJOINED,
                        json!({ "seat": e.seat, "peer": e.peer, "tick": tick }),
                    )
                };
                send(ctx, &layouts, name, &v)?;
            }
            if let Some((t, peer, peers)) = desync {
                send(
                    ctx,
                    &layouts,
                    DESYNC,
                    &json!({ "tick": t, "peer": peer, "peers": peers }),
                )?;
            }
            // og.Lockstep: this peer's seat (players when it watches) and running.
            if let Some(l) = layouts.get(LOCKSTEP) {
                let id = ctx.id_by_name(LOCKSTEP)?;
                let base = ctx.resource_bytes(id)?;
                let local_player = match (running, seat) {
                    (true, Some(s)) => s,
                    (true, None) => players.unwrap_or(0),
                    (false, _) => 0,
                };
                if let Ok(bytes) = l.encode(
                    Some(&base),
                    Some(&json!({ "localPlayer": local_player, "running": running })),
                ) && bytes != base
                {
                    ctx.set_resource_bytes(id, &bytes)?;
                }
            }
            // og.PlayerInput (or og.net.shape's input): each seat's input
            // for this tick.
            let (layout, entities) = if input == PLAYER_INPUT {
                let Some(layout) = layouts.get(PLAYER_INPUT).cloned() else {
                    return Ok(());
                };
                let e: Vec<og_ecs::Entity> =
                    ctx.query(0)?.flat_map(|v| v.entities().to_vec()).collect();
                (layout, e)
            } else {
                let Some(layout) = eng.state.borrow().layouts.get(&input).cloned() else {
                    return Ok(());
                };
                let q = QueryDesc {
                    write: vec![input.clone()],
                    ..QueryDesc::default()
                };
                let e: Vec<og_ecs::Entity> = ctx
                    .query_with(&q)?
                    .flat_map(|v| v.entities().to_vec())
                    .collect();
                (layout, e)
            };
            if entities.is_empty() {
                return Ok(());
            }
            let id = ctx.id_by_name(&input)?;
            let (player_at, _) = layout.field("player").expect("og.net.shape checked it");
            let stamp = |player: u32| {
                if layout.field("tick").is_some() {
                    json!({ "player": player, "tick": tick })
                } else {
                    json!({ "player": player })
                }
            };
            let zero = layout.encode(None, None).unwrap_or_default();
            for e in entities {
                let current = ctx.get_bytes(e, id)?;
                let player = layout.decode_one(&current, player_at).as_u64().unwrap_or(0) as u32;
                let input: Vec<u8> = if running {
                    with(engine_id, |n| n.ls.input(tick, player).map(<[u8]>::to_vec))
                        .unwrap_or_else(|| zero.clone())
                } else if player == 0 && !local.is_empty() {
                    local.clone()
                } else {
                    zero.clone()
                };
                // Its state: the input's first bytes (commands follow them).
                let base = input.get(..zero.len()).unwrap_or(&zero);
                let bytes = layout
                    .encode(Some(base), Some(&stamp(player)))
                    .map_err(|e| og_core::Error::new(og_core::ErrorKind::Invalid, e))?;
                if bytes != current {
                    ctx.set_bytes(e, id, &bytes)?;
                }
            }
            Ok(())
        })
        .map_err(|e| e.reason().to_string())
}

/// `dump_desync`.
pub(crate) fn register(probe: &mut og_probe::Probe, engine: u32) {
    let _ = probe.register_dump(
        "dump_desync",
        og_probe::CommandDesc {
            usage: "dump_desync[:path]".into(),
            doc: "lockstep's first desync (og.net): the first divergent tick, both peers' hashes and the component-level diff of their snapshots of it".into(),
        },
        0,
        move |_world, _args| Ok(with(engine, |n| n.ls.dump())),
    );
    let _ = probe.register_dump(
        "dump_room",
        og_probe::CommandDesc {
            usage: "dump_room[:path]".into(),
            doc: "the room (og.platform.roomInfo), lockstep's state (og.net.status): seats, who this peer waits for, the ticks compared, and what a tick costs this peer to re-simulate (`cost`, ms on the wall clock: `keepMs` is the snapshot and hash after it)".into(),
        },
        0,
        move |_world, _args| {
            let cost = with(engine, |n| {
                json!({ "tickMs": n.cost.tick_ms, "keepMs": n.cost.keep_ms, "rewindMs": n.cost.rewind_ms })
            });
            Ok(json!({
                "room": og_platform_room_info(engine).to_json(),
                "lockstep": og_net_status(engine).to_json(),
                "cost": cost,
            }))
        },
    );
}
