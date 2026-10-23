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

use super::engine::{Engine, engine, running};
use super::{ok_void, parse, refuse};
use og_ecs::{QueryDesc, SystemDesc, World};
use og_net::protocol::{self, Event, LOCKSTEP_CHANNEL, MAX_MESSAGE, Payload};
use og_net::{Config, Gate, Lockstep, Snapshot};
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

struct Message {
    id: u32,
    from: u32,
    tick: Option<u64>,
    payload: Payload,
}

#[derive(Default)]
struct Net {
    host: String,
    stage: Stage,
    room: String,
    peer: u32,
    peers: BTreeMap<u32, String>,
    outbox: Vec<Value>,
    /// The game's events waiting for the next tick: (event, JSON value).
    pending: Vec<(&'static str, Value)>,
    messages: VecDeque<Message>,
    next_message: u32,
    ls: Lockstep,
}

thread_local! {
    static NETS: RefCell<BTreeMap<u32, Net>> = const { RefCell::new(BTreeMap::new()) };
}

fn with<T>(engine: u32, f: impl FnOnce(&mut Net) -> T) -> T {
    NETS.with(|n| f(n.borrow_mut().entry(engine).or_default()))
}

/// Lockstep's messages become `send` ops; its notes go to the engine's log.
fn flush(engine: u32, n: &mut Net, tick: u64) {
    for (to, data) in n.ls.take_out() {
        n.outbox.push(protocol::send(
            to,
            Some(LOCKSTEP_CHANNEL),
            &Payload::Json(data),
        ));
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

/// `og_room_set_host`.
pub fn og_room_set_host(engine: u32, info: String) {
    let v: Value = serde_json::from_str(&info).unwrap_or(Value::Null);
    let host = v["host"].as_str().unwrap_or("none").to_string();
    with(engine, |n| n.host = host);
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
            Event::PeerJoined { peer, name } => {
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
                if matches!(op.as_str(), "create" | "join") && n.stage != Stage::In {
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

pub fn og_platform_room_create(world: u32, desc: Option<String>) -> String {
    let d = match parse(desc.as_deref()) {
        Ok(d) => d,
        Err(e) => return refuse(e),
    };
    let name = d["name"].as_str().unwrap_or("").to_string();
    if !name.is_empty() && !protocol::valid_room(&name) {
        return refuse(format!(
            "'{name}' is not a room name: at most 64 of letters, digits, '.', '_' and '-'"
        ));
    }
    let capacity = d["capacity"]
        .as_u64()
        .unwrap_or(2)
        .clamp(1, u64::from(protocol::MAX_CAPACITY)) as u32;
    with(world, |n| match can_join(n) {
        Ok(()) => {
            n.stage = Stage::Joining;
            n.outbox.push(protocol::create(&name, capacity));
            ok_void()
        }
        Err(e) => refuse(e),
    })
}

pub fn og_platform_room_join(world: u32, room: String) -> String {
    if !protocol::valid_room(&room) {
        return refuse(format!(
            "'{room}' is not a room id: 1 to 64 of letters, digits, '.', '_' and '-'"
        ));
    }
    with(world, |n| match can_join(n) {
        Ok(()) => {
            n.stage = Stage::Joining;
            n.outbox.push(protocol::join(&room, ""));
            ok_void()
        }
        Err(e) => refuse(e),
    })
}

pub fn og_platform_room_leave(world: u32) -> String {
    with(world, |n| {
        if n.stage == Stage::None {
            return refuse("in no room");
        }
        n.outbox.push(protocol::leave());
        ok_void()
    })
}

pub fn og_platform_room_send(world: u32, data: String, to: Option<u32>) -> String {
    let v = match parse(Some(&data)) {
        Ok(v) => v,
        Err(e) => return refuse(e),
    };
    let payload = match v.get(BYTES_KEY).and_then(Value::as_str) {
        Some(b) => match og_net::base64::decode(b) {
            Some(bytes) => Payload::Bytes(bytes),
            None => return refuse("the message's bytes are not base64"),
        },
        None => Payload::Json(v),
    };
    if payload.size() > MAX_MESSAGE {
        return refuse(format!(
            "a message is at most {MAX_MESSAGE} bytes encoded; this one is {}",
            payload.size()
        ));
    }
    with(world, |n| {
        if n.stage != Stage::In {
            return refuse("in no room: roomJoin or roomCreate first, and wait for og.RoomJoined");
        }
        if let Some(t) = to
            && !n.peers.contains_key(&t)
        {
            return refuse(format!("no peer {t} in room '{}'", n.room));
        }
        n.outbox.push(protocol::send(to, None, &payload));
        ok_void()
    })
}

fn peers_json(n: &Net) -> Value {
    Value::Array(
        n.peers
            .iter()
            .map(|(p, name)| json!({ "peer": p, "name": name, "local": n.stage == Stage::In && *p == n.peer }))
            .collect(),
    )
}

pub fn og_platform_room_peers(world: u32) -> String {
    with(world, |n| {
        if n.stage == Stage::In {
            peers_json(n).to_string()
        } else {
            "[]".into()
        }
    })
}

pub fn og_platform_room_info(world: u32) -> String {
    with(world, |n| {
        let state = match n.stage {
            Stage::None => "none",
            Stage::Joining => "joining",
            Stage::In => "in",
        };
        let host = if n.host.is_empty() { "none" } else { &n.host };
        let inside = n.stage == Stage::In;
        json!({
            "state": state,
            "room": if inside { n.room.as_str() } else { "" },
            "host": host,
            "peer": if inside { n.peer } else { 0 },
            "peers": if inside { peers_json(n) } else { json!([]) },
        })
        .to_string()
    })
}

pub fn og_platform_room_message(world: u32, message: u32) -> String {
    with(world, |n| {
        let Some(m) = n.messages.iter().find(|m| m.id == message) else {
            return "null".into();
        };
        let (data, bytes) = match &m.payload {
            Payload::Json(d) => (d.clone(), Value::Null),
            Payload::Bytes(b) => (Value::Null, json!(og_net::base64::encode(b))),
        };
        json!({ "message": m.id, "from": m.from, "tick": m.tick.unwrap_or(0), "data": data, "bytes": bytes })
            .to_string()
    })
}

// ── og.net ───────────────────────────────────────────────────────────────

pub fn og_net_submit(world: u32, input: Option<String>) -> String {
    let d = match parse(input.as_deref()) {
        Ok(d) => d,
        Err(e) => return refuse(e),
    };
    let mut fields = serde_json::Map::new();
    for k in ["buttons", "x", "y", "aimX", "aimY"] {
        match d.get(k) {
            None | Some(Value::Null) => {}
            Some(v) if v.as_f64().is_some_and(f64::is_finite) => {
                fields.insert(k.into(), v.clone());
            }
            Some(v) => return refuse(format!("{k} must be a finite number, got {v}")),
        }
    }
    if let Some(k) = d.as_object().and_then(|o| {
        o.keys()
            .find(|k| !matches!(k.as_str(), "buttons" | "x" | "y" | "aimX" | "aimY"))
    }) {
        return refuse(format!(
            "og.net.submit takes buttons, x, y, aimX and aimY; not '{k}'"
        ));
    }
    let layout = match engine_of(world)
        .ok()
        .and_then(|e| e.state.borrow().layouts.get(PLAYER_INPUT).cloned())
    {
        Some(l) => l,
        None => return refuse("og.PlayerInput is not registered in this world"),
    };
    let bytes = match layout.encode(None, Some(&Value::Object(fields))) {
        Ok(b) => b,
        Err(e) => return refuse(e),
    };
    with(world, |n| n.ls.set_local(bytes));
    ok_void()
}

pub fn og_net_status(world: u32) -> String {
    with(world, |n| {
        let s = n.ls.status();
        let seats: Vec<Value> = s["seats"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|h| json!({ "seat": h[0], "peer": h[1], "since": h[2], "until": h[3] }))
                    .collect()
            })
            .unwrap_or_default();
        json!({
            "running": s["running"],
            "peer": s["peer"],
            "peers": s["peers"],
            "startTick": s["startTick"],
            "players": s["players"],
            "seats": seats,
            "spectators": if s["spectators"].is_null() { json!([]) } else { s["spectators"].clone() },
            "waitingFor": s["waitingFor"],
            "waits": s["waits"],
            "desync": s["desync"]["tick"],
        })
        .to_string()
    })
}

/// The shared hash of a world snapshot, hex.
fn shared_hash(snapshot: &[u8]) -> Result<u64, String> {
    Snapshot::decode(snapshot).map(|s| s.shared_hash())
}

pub fn og_net_hash(world: u32) -> String {
    let bytes = if let Some(f) = running(world) {
        // SAFETY: the frame is published while its system runs.
        unsafe { f.ctx() }.snapshot()
    } else {
        match engine_of(world).and_then(|e| e.world().map(|w| w.snapshot())) {
            Ok(b) => b,
            Err(e) => return json!(format!("error: {e}")).to_string(),
        }
    };
    match shared_hash(&bytes) {
        Ok(h) => json!(format!("{h:016x}")).to_string(),
        Err(e) => json!(format!("error: {e}")).to_string(),
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
    }
}

/// The probe's tick gate: whether the next tick may run.
pub(crate) fn gate(engine: u32, world: &mut World) -> bool {
    if !with(engine, |n| n.ls.me().is_some()) {
        return true;
    }
    let Ok(eng) = engine_of(engine) else {
        return true;
    };
    let cfg = config(&eng, world);
    loop {
        let tick = world.time().tick;
        let gate = with(engine, |n| {
            let g = n.ls.gate(&cfg, tick, &mut || world.snapshot());
            flush(engine, n, tick);
            g
        });
        match gate {
            Gate::Go => return true,
            Gate::Wait => return false,
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
        n.ls.after_tick(tick, &mut || {
            let bytes = world.snapshot();
            (shared_hash(&bytes).unwrap_or(0), bytes)
        });
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
pub(crate) fn mount(world: &mut World, engine_id: u32) -> Result<(), String> {
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
                [
                    JOINED,
                    PEER_JOINED,
                    PEER_LEFT,
                    MESSAGE,
                    DESYNC,
                    LOCKSTEP,
                    PLAYER_INPUT,
                ]
                .iter()
                .filter_map(|n| st.layouts.get(*n).map(|l| (n.to_string(), l.clone())))
                .collect()
            };
            let (events, desync, running, seat, players, local) = with(engine_id, |n| {
                for m in n.messages.iter_mut().filter(|m| m.tick.is_none()) {
                    m.tick = Some(tick);
                }
                (
                    std::mem::take(&mut n.pending),
                    n.ls.take_desync(),
                    n.ls.running(),
                    n.ls.seat(tick),
                    n.ls.players(),
                    n.ls.local().to_vec(),
                )
            });
            for (name, v) in &events {
                send(ctx, &layouts, name, v)?;
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
            // og.PlayerInput: each seat's input for this tick.
            let Some(layout) = layouts.get(PLAYER_INPUT) else {
                return Ok(());
            };
            let id = ctx.id_by_name(PLAYER_INPUT)?;
            let entities: Vec<og_ecs::Entity> =
                ctx.query(0)?.flat_map(|v| v.entities().to_vec()).collect();
            if entities.is_empty() {
                return Ok(());
            }
            let zero = layout.encode(None, None).unwrap_or_default();
            for e in entities {
                let current = ctx.get_bytes(e, id)?;
                let player = layout.decode_one(&current, 0).as_u64().unwrap_or(0) as u32;
                let input: Vec<u8> = if running {
                    with(engine_id, |n| n.ls.input(tick, player).map(<[u8]>::to_vec))
                        .unwrap_or_else(|| zero.clone())
                } else if player == 0 && !local.is_empty() {
                    local.clone()
                } else {
                    zero.clone()
                };
                let base = if input.len() == zero.len() {
                    &input
                } else {
                    &zero
                };
                let bytes = layout
                    .encode(Some(base), Some(&json!({ "player": player, "tick": tick })))
                    .map_err(|e| og_core::Error::new(og_core::ErrorKind::Invalid, e))?;
                if bytes != current {
                    ctx.set_bytes(e, id, &bytes)?;
                }
            }
            Ok(())
        })
        .map_err(|e| e.reason().to_string())
}

pub(crate) fn retire(engine: u32) {
    NETS.with(|n| n.borrow_mut().remove(&engine));
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
            doc: "the room (og.platform.roomInfo) and lockstep's state (og.net.status): seats, who this peer waits for, the ticks compared".into(),
        },
        0,
        move |_world, _args| {
            let info: Value = serde_json::from_str(&og_platform_room_info(engine)).unwrap_or(Value::Null);
            let status: Value = serde_json::from_str(&og_net_status(engine)).unwrap_or(Value::Null);
            Ok(json!({ "room": info, "lockstep": status }))
        },
    );
}
