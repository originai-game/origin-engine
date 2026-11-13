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
//! snapshot ([`crate::world::save::rewind`]) and the ticks run again in the same
//! frame. A re-simulated tick sends the game the room events its first run
//! did, and the audio link drops the plays the sink already heard. How far
//! this peer predicts is also capped by what it can re-simulate in half a
//! frame (measured on the wall clock: it decides only when the peer waits,
//! never the confirmed world).

use crate::engine::{Engine, engine, running};
use crate::host::wire::{
    self, LockstepSeat, LockstepStatus, NetMode, RoomInfo, RoomMessageData, RoomPeer, ToJson,
    record,
};
use crate::parse;
use og_core::IdMap;
use og_ecs::{QueryDesc, SystemDesc, World};
use og_net::protocol::{self, Event, LOCKSTEP_CHANNEL, MAX_MESSAGE, Payload};
use og_net::state::shared_hash;
use og_net::{Config, Gate, Lockstep};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};

mod host_side;
mod lockstep;
mod rooms;
mod tick;

pub use host_side::*;
pub use lockstep::*;
pub use rooms::*;
pub(crate) use tick::*;

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
/// Lockstep's seat events: lazy records ([`crate::engine::lazy`]), registered once a
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
    match crate::engine::engine(engine) {
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

// ── the tick ─────────────────────────────────────────────────────────────

/// Mounts the og.net system (front, after og.platform).
pub(crate) fn mount(world: &mut World, mount: crate::engine::Mount<'_>) -> Result<(), String> {
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
            let layouts: BTreeMap<String, crate::host::codec::Layout> = {
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
