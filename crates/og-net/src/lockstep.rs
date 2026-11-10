//! Deterministic lockstep and rollback over a room (docs/design.md §2.10).
//!
//! Every peer runs the whole simulation; what crosses the room is each
//! player's input, one per tick, and a hash of the world now and then.
//!
//! - **Seats.** `players` seats, numbered from 0 (og.Lockstep.localPlayer
//!   is this peer's). A seat is held by one peer at a time: an occupancy
//!   `[since, until]` of ticks whose inputs that peer sends. A peer beyond
//!   the seats watches (a spectator: it runs the world and sends no input).
//! - **Start.** Until the room holds `players` peers the world runs freely
//!   on each peer (a lobby: the local input drives the local player). The
//!   *authority*, the lowest peer number among the peers running (before
//!   the start: present), then starts at its tick boundary T: it sends every
//!   other peer `start` with its world's snapshot, the seats (by peer number,
//!   in that order) and the settings (`players`, `inputDelay`, `hashEvery`,
//!   the mode and `maxPrediction` are the authority's), and each restores
//!   the snapshot. Every seat's inputs are required from `T + inputDelay`;
//!   the ticks before are empty input.
//! - **Inputs.** After tick t, a seated peer makes its input for
//!   `t + inputDelay` (the last og.net.submit, held) and sends every input
//!   of its own the other peers have not acknowledged yet (each input
//!   message carries its sender's confirmed tick, the acknowledgement), so
//!   a message the room loses or delays costs nothing but time: the next
//!   one carries the same inputs again. Input messages may be sent
//!   unreliably; everything else is reliable and ordered per sender.
//! - **Confirmed ticks.** Tick t is *confirmed* once every seat that is
//!   occupied at t has its real input for t, and every tick before it is.
//! - **Lockstep** (the default, `maxPrediction` 0): tick t runs only when it
//!   is confirmed ([`Lockstep::gate`]); with `inputDelay` of at least 1
//!   nobody waits on itself.
//! - **Rollback**: tick t runs once `t <= confirmed + maxPrediction`, a
//!   missing input *predicted* as its seat's last real input (empty for a
//!   seat its holder has not sent for yet). Each tick's inputs as it ran
//!   them are kept; a real input that differs from the one a tick ran with
//!   marks that tick, and at the next boundary [`Gate::Rewind`] hands back
//!   this peer's own snapshot of the world before it, from a ring of one
//!   per unconfirmed tick, and the world runs forward again with what is
//!   now known (*re-simulated* ticks, [`Lockstep::resimulating`]). How far
//!   a peer predicts only decides when it waits: the confirmed world is a
//!   function of the confirmed inputs alone, so every peer agrees on it.
//! - **Commands.** With [`Config::state_bytes`] set, an input is that many
//!   bytes of state followed by the tick's commands
//!   ([`Lockstep::push_commands`]): a prediction repeats the state alone,
//!   so a command is never acted on at a tick its giver did not send it
//!   for. A `start` whose [`Config::shape`] differs is refused.
//! - **Late join.** A peer that joins a running room is admitted by the
//!   authority at its next boundary T: a free seat (one never held, or
//!   left) from `T + inputDelay`, or none (a spectator). The authority
//!   tells everyone (`admit`) and hands the newcomer a `start` with its
//!   last confirmed world (tick S, at most T), the seats' histories and
//!   every input it holds from S; every other peer sends the newcomer its
//!   own inputs from S. In lockstep, delivery in order per sender makes
//!   `T + inputDelay` safe: no peer can have run a tick past it before the
//!   admit arrives, since that tick needs the authority's input sent after
//!   the admit; in rollback a peer that predicted past it rewinds when the
//!   newcomer's first input differs from its prediction (empty).
//! - **Departure.** When a seated peer leaves, its seat is empty from the
//!   tick after its last input: every peer received the same inputs from it
//!   before its departure, so all agree on that tick.
//! - **Hashes.** Each peer hashes its world after every confirmed tick
//!   ([`crate::state::Snapshot::shared_hash`]: everything but the peer's
//!   own session) and keeps the snapshot; every `hashEvery` ticks it sends
//!   the hashes since its last batch. The first tick whose hashes differ is
//!   the first divergent tick, exactly: og.Desync, and both peers ask each
//!   other for their snapshot of that tick, whose component-level diff
//!   ([`crate::state::diff`]) is the probe's `desync` dump. Only the first
//!   desync of a session is reported.
//!
//! The machine never touches a world and reads no clock: og-runtime gives
//! it snapshots and hashes, restores what [`Gate::Restore`] and
//! [`Gate::Rewind`] hand back and applies [`Lockstep::input`] to
//! og.PlayerInput.

use crate::base64;
use crate::protocol::{Event, LOCKSTEP_CHANNEL, Payload};
use crate::state::{Snapshot, diff};
use og_core::IdMap;
use serde_json::{Value, json};
use std::collections::BTreeSet;

mod messages;
mod peers;
mod session;
mod ticks;

/// The version of lockstep's messages: a `start` of another version is
/// refused, so peers on different engines never play together.
pub const WIRE_VERSION: u64 = 4;

/// og.Lockstep's settings, as the game set them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub enabled: bool,
    pub players: u32,
    pub input_delay: u32,
    pub hash_every: u32,
    /// Rollback (true) or lockstep (false, the default).
    pub rollback: bool,
    /// In rollback, the most ticks a peer runs past its last confirmed one.
    pub max_prediction: u32,
    /// This peer's own cap on prediction (what it can re-simulate in half
    /// a frame, measured by the runtime): it decides only when this peer
    /// waits, never what the room's confirmed world is.
    pub budget: u32,
    /// Milliseconds a running peer may stay silent before the authority
    /// drops it (0 never drops a peer that is still in the room).
    pub stall_timeout: u32,
    /// Whether a dropped peer that comes back, or a newcomer, takes a seat
    /// a dropped or departed peer held (else such a seat stays empty).
    pub rejoin: bool,
    /// The bytes of a seat's input that hold its state (og.net.shape's
    /// input record); what follows them are that tick's commands. A
    /// prediction repeats the state alone, never a command. 0: the whole
    /// input is state (no commands).
    pub state_bytes: u32,
    /// The game's input shape (og.net.shape), as a fingerprint: a start
    /// from a peer whose shape differs is refused, since its inputs would
    /// be read through another layout.
    pub shape: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            enabled: false,
            players: 2,
            input_delay: 2,
            hash_every: 10,
            rollback: false,
            max_prediction: 8,
            budget: u32::MAX,
            stall_timeout: 5000,
            rejoin: true,
            state_bytes: 0,
            shape: 0,
        }
    }
}

/// What the world does at a tick boundary.
#[derive(Debug, PartialEq)]
pub enum Gate {
    /// Take the tick.
    Go,
    /// Hold: an input the tick needs has not arrived (in rollback: the tick
    /// is further than `maxPrediction` past the last confirmed one).
    Wait,
    /// Restore this snapshot (a peer's world at `tick`: the start, or a late
    /// join), keeping the session, then ask again.
    Restore { snapshot: Vec<u8>, tick: u64 },
    /// Rollback: put the world back to this peer's own snapshot of `tick`
    /// (one this process took, from the ring) and ask again; the ticks from
    /// `tick` run again with the inputs now known.
    Rewind { snapshot: Vec<u8>, tick: u64 },
}

/// A seat changing hands at a tick, the same on every peer: og.SeatDropped
/// (`dropped`) or og.SeatRejoined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeatEvent {
    pub seat: u32,
    pub peer: u32,
    pub dropped: bool,
    /// For a drop: the peer left the room (else it stalled).
    pub left: bool,
}

/// One peer's hold on a seat: the ticks whose inputs it sends.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Hold {
    peer: u32,
    since: u64,
    /// The last tick of its input, once it was dropped.
    until: Option<u64>,
    /// Whether it was dropped because it left the room (else it stalled).
    left: bool,
}

impl Hold {
    fn covers(&self, tick: u64) -> bool {
        tick >= self.since && self.until.is_none_or(|u| tick <= u)
    }
}

#[derive(Clone, Debug)]
struct Run {
    start: u64,
    players: u32,
    delay: u64,
    hash_every: u64,
    /// The ticks a peer may run past its confirmed one: 0 is lockstep.
    max_prediction: u64,
    /// Per seat, every hold in order.
    seats: Vec<Vec<Hold>>,
    /// Every peer admitted (seated or watching) and still present.
    running: BTreeSet<u32>,
    /// Every peer admitted and not dropped since, so a newcomer is told
    /// apart.
    admitted: BTreeSet<u32>,
    /// The authority's stall timeout (ms; 0 never drops a present peer).
    stall_timeout: u64,
    /// Whether a seat a dropped peer held is taken again.
    rejoin: bool,
    /// The input shape's fingerprint ([`Config::shape`]).
    shape: u64,
}

impl Run {
    fn seat_of(&self, peer: u32, tick: u64) -> Option<u32> {
        self.seats
            .iter()
            .position(|h| h.iter().any(|h| h.peer == peer && h.covers(tick)))
            .map(|s| s as u32)
    }

    /// This peer's seat now or from now on (its open hold).
    fn open_seat(&self, peer: u32) -> Option<u32> {
        self.seats
            .iter()
            .position(|h| {
                h.last()
                    .is_some_and(|h| h.peer == peer && h.until.is_none())
            })
            .map(|s| s as u32)
    }

    /// How many seat changes this peer knows (holds begun and ended): an
    /// input message from a peer that knows more waits for the reliable
    /// `admit` or `drop` it overtook.
    fn epoch(&self) -> u64 {
        self.seats
            .iter()
            .flatten()
            .map(|h| 1 + u64::from(h.until.is_some()))
            .sum()
    }

    /// The hold of seat `seat` covering `tick`.
    fn hold(&self, seat: usize, tick: u64) -> Option<&Hold> {
        self.seats[seat].iter().find(|h| h.covers(tick))
    }

    fn holds_json(&self) -> Value {
        Value::Array(
            self.seats
                .iter()
                .enumerate()
                .flat_map(|(s, holds)| {
                    holds
                        .iter()
                        .map(move |h| json!([s, h.peer, h.since, h.until, h.left]))
                })
                .collect(),
        )
    }
}

/// The first desync of the session.
#[derive(Clone, Debug)]
struct Desync {
    tick: u64,
    peer: u32,
    local_hash: u64,
    remote_hash: u64,
    local: Vec<u8>,
    remote: Option<Vec<u8>>,
    diff: Option<Value>,
}

/// How many of its own past inputs a peer keeps for a newcomer.
const KEEP_OWN: u64 = 256;
/// The most per-tick snapshots kept waiting for the other peers' hashes.
const MAX_KEPT: usize = 2048;
/// The most of its own inputs one input message carries.
const MAX_RESEND: u64 = 64;
/// The final worlds' hashes kept for [`Lockstep::final_hash`].
const KEEP_FINALS: usize = 1024;
/// How far past the last tick run the confirmed tick is looked for (a room
/// whose every seat is empty confirms every tick).
const CONFIRM_AHEAD: u64 = 4096;

/// A message to send: to one peer or all, its data, and whether the room
/// may drop or reorder it (input messages, which the next one repeats).
pub type Out = (Option<u32>, Value, bool);

/// One engine's lockstep.
#[derive(Debug, Default)]
pub struct Lockstep {
    me: Option<u32>,
    /// The peers present, this one included, by number.
    peers: IdMap<u32, String>,
    run: Option<Run>,
    /// A start handed to this peer, applied at the next boundary.
    handover: Option<(Run, u64, Vec<u8>)>,
    /// Lockstep's messages that arrived after `handover` and before it was
    /// applied (a drop, an admit, inputs): they are about the run it
    /// begins, so they act once it began, in the order they came.
    deferred: Vec<(u32, Value)>,
    /// When this peer, not the authority, first saw its room full without
    /// a run (it holds its lobby for the start from then, for a while).
    full_since: Option<u64>,
    /// Real inputs by tick, then seat. Empty bytes are an empty input.
    inputs: IdMap<u64, IdMap<u32, Vec<u8>>>,
    /// The inputs each tick not yet final ran with, by seat (predicted
    /// ones included).
    used: IdMap<u64, Vec<Vec<u8>>>,
    /// The last tick each seat's input was received (or made) for.
    seat_last: IdMap<u32, u64>,
    /// This peer's own inputs by tick, for a newcomer and for resending.
    own: IdMap<u64, Vec<u8>>,
    /// The input og.net.submit gave last (packed og.PlayerInput).
    local: Vec<u8>,
    /// The commands given since this peer's last input went out
    /// (og.net.command), sent once after the state of the next.
    commands: Vec<u8>,
    /// [`Config::state_bytes`], as the last gate read it.
    state_bytes: usize,
    /// Every tick up to this one is confirmed.
    confirmed: u64,
    /// The furthest world tick reached: below it a tick is re-simulated.
    frontier: u64,
    /// The world's tick of the last final snapshot (the ring's base).
    hashed: u64,
    /// The world before each tick from `hashed` on: (hash, snapshot) of
    /// the world at that tick, this process's own.
    ring: IdMap<u64, (u64, Vec<u8>)>,
    /// The first tick that ran with an input that turned out wrong.
    rewind_to: Option<u64>,
    /// Per other peer, the tick it has every input through (its ack).
    acks: IdMap<u32, u64>,
    rollbacks: u64,
    resimulated: u64,
    /// This peer's hash and snapshot by tick, until every peer compared.
    mine: IdMap<u64, (u64, Vec<u8>)>,
    /// Other peers' hashes by tick, until compared.
    theirs: IdMap<u32, IdMap<u64, u64>>,
    /// The last tick compared with each peer.
    compared: IdMap<u32, u64>,
    /// The first tick of the next hash batch, and its hashes so far.
    batch_from: u64,
    batch: Vec<Value>,
    desync: Option<Desync>,
    /// og.Desync to send: (tick, first peer, peers that differ).
    desync_event: Option<(u64, u32, u32)>,
    /// Messages to send.
    out: Vec<Out>,
    /// Boundaries this peer waited at, and the seats it waited for last.
    waits: u64,
    waiting_for: Vec<u32>,
    /// Lines for the engine's log.
    notes: Vec<String>,
    /// The last final worlds' hashes, by tick ([`Lockstep::final_hash`]).
    finals: IdMap<u64, u64>,
    /// The wall clock, in ms ([`Lockstep::set_clock`]): it decides only
    /// when a silent peer is dropped, never at which tick.
    now: u64,
    /// When each other peer was last heard from (ms).
    heard: IdMap<u32, u64>,
    /// Seats whose drop is being agreed: the last tick of their input
    /// this peer takes (it told the authority so).
    frozen: IdMap<u32, u64>,
    /// At the authority: the drops being agreed, by seat.
    proposals: IdMap<u32, Proposal>,
    /// Peers dropped for a stall, admitted again once heard from.
    held_back: BTreeSet<u32>,
    /// This peer was dropped and waits for the authority's snapshot.
    rejoining: bool,
    /// Peers silent past [`STALL_NOTICE_MS`] (og.PeerStalled sent once).
    silent: BTreeSet<u32>,
    /// og.PeerStalled to send: (peer, seat, silent ms).
    stalls: Vec<(u32, Option<u32>, u64)>,
    drops: u64,
    /// Every peer's name seen in the room, so a returning player is known.
    names: IdMap<u32, String>,
}

/// A drop the authority is agreeing with the other peers: every running
/// peer reports how far it has the seat's inputs, and the seat empties
/// after the furthest, so no peer has run a tick past it on real input.
#[derive(Clone, Debug)]
struct Proposal {
    peer: u32,
    since: u64,
    left: bool,
    /// The authority's own last input of the seat.
    from: u64,
    /// The furthest last input reported, and the lowest.
    last: u64,
    low: u64,
    /// The inputs reported past `from`.
    inputs: IdMap<u64, Vec<u8>>,
    /// The peers yet to answer.
    waiting: BTreeSet<u32>,
}

/// A peer silent this long is reported (og.PeerStalled, status `stalled`).
const STALL_NOTICE_MS: u64 = 1000;
/// How long a peer of a full room holds its lobby for the authority's start
/// when og.Lockstep.stallTimeout is 0 (else it holds that long).
const START_HOLD_MS: u64 = 5000;

fn hex(h: u64) -> String {
    format!("{h:016x}")
}

fn unhex(v: &Value) -> Option<u64> {
    u64::from_str_radix(v.as_str()?, 16).ok()
}

fn u64_of(v: &Value) -> Option<u64> {
    v.as_u64()
}

fn u32_of(v: &Value) -> Option<u32> {
    v.as_u64().and_then(|x| u32::try_from(x).ok())
}

fn mode_text(run: &Run) -> String {
    if run.max_prediction > 0 {
        format!("rollback predicting up to {} ticks", run.max_prediction)
    } else {
        "lockstep".into()
    }
}

fn start_msg(
    run: &Run,
    tick: u64,
    snapshot: &[u8],
    inputs: &IdMap<u64, IdMap<u32, Vec<u8>>>,
    seat_last: &IdMap<u32, u64>,
) -> Value {
    let inputs: Vec<Value> = inputs
        .range(tick..)
        .flat_map(|(t, m)| m.iter().map(move |(s, b)| json!([t, s, base64::encode(b)])))
        .collect();
    json!({
        "k": "start",
        "v": WIRE_VERSION,
        "tick": tick,
        "start": run.start,
        "players": run.players,
        "delay": run.delay,
        "hashEvery": run.hash_every,
        "maxPrediction": run.max_prediction,
        "holds": run.holds_json(),
        "running": run.running,
        "admitted": run.admitted,
        "stallTimeout": run.stall_timeout,
        "rejoin": run.rejoin,
        "shape": hex(run.shape),
        "inputs": inputs,
        "last": seat_last.iter().map(|(s, t)| json!([s, t])).collect::<Vec<_>>(),
        "snapshot": base64::encode(snapshot),
    })
}

type Parsed = (Run, u64, Vec<u8>, Vec<(u64, u32, Vec<u8>)>, Vec<(u32, u64)>);

fn parse_start(d: &Value) -> Result<Parsed, String> {
    let v = d["v"].as_u64().unwrap_or(1);
    if v != WIRE_VERSION {
        return Err(format!(
            "it speaks lockstep version {v} and this engine {WIRE_VERSION}: both peers need the same engine version"
        ));
    }
    parse_run(d).ok_or_else(|| "it is malformed".to_string())
}

fn parse_run(d: &Value) -> Option<Parsed> {
    let players = u32_of(&d["players"])?.clamp(1, 64);
    let mut seats = vec![Vec::new(); players as usize];
    for h in d["holds"].as_array()? {
        let s = u32_of(&h[0])? as usize;
        seats.get_mut(s)?.push(Hold {
            peer: u32_of(&h[1])?,
            since: u64_of(&h[2])?,
            until: u64_of(&h[3]),
            left: h[4].as_bool().unwrap_or(false),
        });
    }
    let set = |k: &str| -> Option<BTreeSet<u32>> { d[k].as_array()?.iter().map(u32_of).collect() };
    let run = Run {
        start: u64_of(&d["start"])?,
        players,
        delay: u64_of(&d["delay"])?.max(1),
        hash_every: u64_of(&d["hashEvery"])?,
        max_prediction: u64_of(&d["maxPrediction"])?,
        seats,
        running: set("running")?,
        admitted: set("admitted")?,
        stall_timeout: u64_of(&d["stallTimeout"]).unwrap_or(5000),
        rejoin: d["rejoin"].as_bool().unwrap_or(true),
        shape: unhex(&d["shape"]).unwrap_or(0),
    };
    let inputs = d["inputs"]
        .as_array()?
        .iter()
        .filter_map(|i| {
            Some((
                u64_of(&i[0])?,
                u32_of(&i[1])?,
                base64::decode(i[2].as_str()?)?,
            ))
        })
        .collect();
    let last = d["last"]
        .as_array()?
        .iter()
        .filter_map(|x| Some((u32_of(&x[0])?, u64_of(&x[1])?)))
        .collect();
    Some((
        run,
        u64_of(&d["tick"])?,
        base64::decode(d["snapshot"].as_str()?)?,
        inputs,
        last,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Event;

    /// A toy world: its tick and a state every input folds into, so two
    /// peers agree exactly when they applied the same inputs at the same
    /// ticks.
    #[derive(Clone, Debug, PartialEq)]
    struct World {
        tick: u64,
        state: u64,
        /// A peer that reads its own clock (Math.random) from this tick.
        chaos: Option<u64>,
    }

    impl World {
        fn bytes(&self) -> Vec<u8> {
            [self.tick.to_le_bytes(), self.state.to_le_bytes()].concat()
        }
        fn restore(&mut self, b: &[u8]) {
            self.tick = u64::from_le_bytes(b[..8].try_into().unwrap());
            self.state = u64::from_le_bytes(b[8..16].try_into().unwrap());
        }
    }

    struct Peer {
        ls: Lockstep,
        world: World,
        cfg: Config,
        id: u32,
        /// Ticks taken for the first time, and again (a rollback's).
        first: u64,
        again: u64,
        /// A stalled process: it neither runs nor reads the room; what
        /// arrives meanwhile waits in `inbox`.
        paused: bool,
        inbox: Vec<(u32, Value)>,
        /// The seat events each tick ran with, as last run.
        events: std::collections::BTreeMap<u64, Vec<SeatEvent>>,
    }

    /// A seeded network: each message is late by `latency` plus up to
    /// `jitter` frames; an unreliable one (inputs) may be lost or overtake
    /// another; reliable ones keep their order per sender and recipient.
    struct Net {
        rng: u64,
        latency: u64,
        jitter: u64,
        /// Losses per 100 unreliable messages.
        loss: u64,
        /// In flight: (due frame, sequence, from, to, data).
        flight: Vec<(u64, u64, u32, u32, Value)>,
        seq: u64,
        /// The last due frame of a reliable message, per (from, to).
        last_due: std::collections::BTreeMap<(u32, u32), u64>,
    }

    impl Net {
        fn next(&mut self) -> u64 {
            // xorshift64*
            self.rng ^= self.rng >> 12;
            self.rng ^= self.rng << 25;
            self.rng ^= self.rng >> 27;
            self.rng.wrapping_mul(0x2545_f491_4f6c_dd1d)
        }
    }

    /// The relay: every peer's messages to the others, at once or through
    /// a [`Net`].
    struct Room {
        peers: Vec<Peer>,
        net: Option<Net>,
        now: u64,
    }

    /// The input a peer makes at tick `t`: it changes every 7 ticks, so a
    /// prediction is right most ticks and wrong on the changes.
    fn made(id: u32, t: u64) -> Vec<u8> {
        vec![id as u8 + 1, ((t / 7) % 251) as u8]
    }

    fn fold(state: u64, tick: u64, inputs: &[Vec<u8>]) -> u64 {
        let mut f = state;
        for (s, b) in inputs.iter().enumerate() {
            for x in b {
                f = f.wrapping_mul(31).wrapping_add(u64::from(*x) + s as u64);
            }
        }
        f.wrapping_add(tick)
    }

    impl Room {
        fn new(net: Option<Net>) -> Room {
            Room {
                peers: Vec::new(),
                net,
                now: 0,
            }
        }

        fn join(&mut self, id: u32, cfg: Config) {
            self.route();
            let peers: Vec<(u32, String)> = self
                .peers
                .iter()
                .map(|p| (p.id, String::new()))
                .chain([(id, String::new())])
                .collect();
            for p in &mut self.peers {
                p.ls.on_event(&Event::PeerJoined {
                    peer: id,
                    name: String::new(),
                });
            }
            let mut ls = Lockstep::new();
            ls.on_event(&Event::Joined {
                room: "r".into(),
                peer: id,
                peers,
            });
            self.peers.push(Peer {
                ls,
                world: World {
                    tick: 0,
                    state: 7,
                    chaos: None,
                },
                cfg,
                id,
                first: 0,
                again: 0,
                paused: false,
                inbox: Vec::new(),
                events: Default::default(),
            });
        }

        /// Peer `id` leaves after every message it sent arrived (the
        /// relay's order: its departure after the last of them).
        fn leave(&mut self, id: u32) {
            self.route();
            if self.net.is_some() {
                while self.net.as_ref().unwrap().flight.iter().any(|m| m.2 == id) {
                    self.now += 1;
                    self.route();
                }
            }
            self.peers.retain(|p| p.id != id);
            for p in &mut self.peers {
                p.ls.on_event(&Event::PeerLeft { peer: id });
            }
        }

        fn deliver(&mut self, from: u32, to: u32, data: Value) {
            let e = Event::Message {
                from,
                channel: Some(LOCKSTEP_CHANNEL.into()),
                payload: Payload::Json(data),
            };
            if let Some(p) = self.peers.iter_mut().find(|p| p.id == to) {
                if p.paused {
                    p.inbox.push((from, e_data(e)));
                } else {
                    p.ls.on_event(&e);
                }
            }
        }

        fn route(&mut self) {
            loop {
                let mut sent = Vec::new();
                for p in self.peers.iter_mut().filter(|p| !p.paused) {
                    for (to, data, lossy) in p.ls.take_out() {
                        sent.push((p.id, to, data, lossy));
                    }
                }
                let ids: Vec<u32> = self.peers.iter().map(|p| p.id).collect();
                let mut now = Vec::new();
                for (from, to, data, lossy) in sent {
                    for t in ids.iter().copied().filter(|t| *t != from) {
                        if to.is_some_and(|x| x != t) {
                            continue;
                        }
                        let Some(net) = self.net.as_mut() else {
                            now.push((from, t, data.clone()));
                            continue;
                        };
                        let r = net.next();
                        if lossy && r % 100 < net.loss {
                            continue;
                        }
                        let mut due = self.now + net.latency + (r >> 8) % (net.jitter + 1);
                        if !lossy {
                            let last = net.last_due.entry((from, t)).or_insert(0);
                            due = due.max(*last);
                            *last = due;
                        }
                        net.seq += 1;
                        net.flight.push((due, net.seq, from, t, data.clone()));
                    }
                }
                if let Some(net) = self.net.as_mut() {
                    net.flight.sort_by_key(|m| (m.0, m.1));
                    let n = net.flight.iter().take_while(|m| m.0 <= self.now).count();
                    now.extend(net.flight.drain(..n).map(|m| (m.2, m.3, m.4)));
                }
                if now.is_empty() {
                    break;
                }
                for (from, to, data) in now {
                    self.deliver(from, to, data);
                }
            }
        }

        /// One frame of peer `i`: as many ticks as its gate lets it, at most `n`.
        fn frame(&mut self, i: usize, n: u32) -> u32 {
            let mut taken = 0;
            let now = self.now * 16 + 1;
            if self.peers[i].paused {
                return 0;
            }
            self.peers[i].ls.set_clock(now);
            while taken < n {
                let p = &mut self.peers[i];
                let w = p.world.clone();
                let gate = p.ls.gate(&p.cfg, p.world.tick, &mut || w.bytes());
                match gate {
                    Gate::Restore { snapshot, .. } | Gate::Rewind { snapshot, .. } => {
                        p.world.restore(&snapshot);
                        continue;
                    }
                    Gate::Wait => break,
                    Gate::Go => {}
                }
                let t = p.world.tick;
                p.events.insert(t, p.ls.seat_events(t));
                if p.ls.resimulating(t) {
                    p.again += 1;
                } else {
                    p.first += 1;
                }
                p.ls.set_local(made(p.id, t));
                if let Some(c) = command(&p.cfg, t)
                    && !p.ls.resimulating(t)
                {
                    p.ls.push_commands(&c);
                }
                let inputs: Vec<Vec<u8>> = (0..p.ls.players().unwrap_or(0))
                    .map(|s| p.ls.input(t, s).unwrap_or(&[]).to_vec())
                    .collect();
                // A prediction carries the seat's state alone, never a command.
                for (s, b) in inputs.iter().enumerate() {
                    let real =
                        p.ls.inputs
                            .get(&t)
                            .is_some_and(|m| m.contains_key(&(s as u32)));
                    assert!(
                        real || p.cfg.state_bytes == 0 || b.len() <= p.cfg.state_bytes as usize,
                        "tick {t} seat {s} ran on a predicted command: {b:?}"
                    );
                }
                let mut state = fold(p.world.state, t, &inputs);
                if p.world.chaos.is_some_and(|c| t >= c) {
                    state ^= 0x9e37_79b9;
                }
                p.world.state = state;
                p.world.tick += 1;
                let w = p.world.clone();
                p.ls.after_tick(p.world.tick, &mut || (w.state, w.bytes()));
                taken += 1;
            }
            self.route();
            taken
        }

        /// Runs every peer, one frame of up to 2 ticks each per round
        /// (the network's clock moves a frame a round), until each has
        /// confirmed `until`.
        fn run_to(&mut self, until: u64) {
            for _ in 0..200_000 {
                if self.peers.iter().all(|p| p.ls.hashed >= until) {
                    return;
                }
                for i in 0..self.peers.len() {
                    self.frame(i, 2);
                }
                self.now += 1;
                self.route();
            }
            panic!(
                "stuck: {:?}",
                self.peers
                    .iter()
                    .map(|p| (p.world.tick, p.ls.confirmed))
                    .collect::<Vec<_>>()
            );
        }

        /// A stalled peer runs again: the room's messages to it arrive.
        fn resume(&mut self, i: usize) {
            self.peers[i].paused = false;
            self.peers[i].ls.set_clock(self.now * 16 + 1);
            for (from, data) in std::mem::take(&mut self.peers[i].inbox) {
                let to = self.peers[i].id;
                self.deliver(from, to, data);
            }
            self.route();
        }

        /// Every peer's hash of the confirmed world at `tick`.
        fn hashes(&self, tick: u64) -> Vec<Option<u64>> {
            self.peers
                .iter()
                .map(|p| p.ls.finals.get(&tick).copied())
                .collect()
        }
    }

    fn e_data(e: Event) -> Value {
        match e {
            Event::Message {
                payload: Payload::Json(d),
                ..
            } => d,
            _ => Value::Null,
        }
    }

    fn cfg(players: u32, delay: u32, every: u32) -> Config {
        Config {
            enabled: true,
            players,
            input_delay: delay,
            hash_every: every,
            ..Config::default()
        }
    }

    fn rollback(players: u32, delay: u32, predict: u32) -> Config {
        Config {
            rollback: true,
            max_prediction: predict,
            ..cfg(players, delay, 1)
        }
    }

    /// The command a peer gives at tick `t` when its config has a shape
    /// with commands: one every 5 ticks, after its 2 bytes of state.
    fn command(cfg: &Config, t: u64) -> Option<Vec<u8>> {
        (cfg.state_bytes > 0 && t.is_multiple_of(5)).then(|| vec![0xc0, (t % 251) as u8])
    }

    /// The world of tick `until` from the inputs alone: seat s held by the
    /// peer of number s from tick `delay`, each making [`made`].
    fn offline(players: u32, delay: u64, until: u64) -> u64 {
        offline_with(&cfg(players, delay as u32, 1), until)
    }

    /// [`offline`], with the commands `cfg` gives ([`command`]).
    fn offline_with(cfg: &Config, until: u64) -> u64 {
        let (players, delay) = (cfg.players, u64::from(cfg.input_delay));
        let mut state = 7;
        for t in 0..until {
            let inputs: Vec<Vec<u8>> = (0..players)
                .map(|s| {
                    if t >= delay {
                        let mut b = made(s, t - delay);
                        b.extend(command(cfg, t - delay).unwrap_or_default());
                        b
                    } else {
                        Vec::new()
                    }
                })
                .collect();
            state = fold(state, t, &inputs);
        }
        state
    }

    fn lossy(seed: u64, latency: u64, jitter: u64, loss: u64) -> Option<Net> {
        Some(Net {
            rng: seed | 1,
            latency,
            jitter,
            loss,
            flight: Vec::new(),
            seq: 0,
            last_due: Default::default(),
        })
    }

    #[test]
    fn two_peers_start_together_wait_for_each_other_and_agree() {
        let mut room = Room::new(None);
        room.join(0, cfg(2, 2, 5));
        // Alone, the first peer runs freely (the lobby).
        assert_eq!(room.frame(0, 30), 30);
        assert!(!room.peers[0].ls.running());
        room.join(1, cfg(2, 2, 5));
        // The authority starts at its boundary and hands its world over.
        room.frame(0, 1);
        assert!(room.peers[0].ls.running());
        room.frame(1, 1);
        assert_eq!(
            room.peers[1].world.tick, 31,
            "the joiner restored tick 30 and ran one"
        );
        // One peer cannot run ahead of the other by more than the delay.
        let ahead = room.frame(0, 100);
        assert!(ahead <= 2, "ran {ahead} ticks ahead");
        for _ in 0..300 {
            room.frame(0, 3);
            room.frame(1, 2);
        }
        let (a, b) = (room.peers[0].world.tick, room.peers[1].world.tick);
        assert!(a.min(b) > 300, "{a} {b}");
        while room.peers[0].world.tick < 600 || room.peers[1].world.tick < 600 {
            if room.peers[0].world.tick < 600 {
                room.frame(0, 1);
            }
            if room.peers[1].world.tick < 600 {
                room.frame(1, 1);
            }
        }
        assert_eq!(room.peers[0].world, room.peers[1].world);
        assert_eq!(room.peers[0].ls.dump()["desync"], json!(false));
        assert_eq!(room.peers[1].ls.seat(600), Some(1));
        assert_eq!(room.peers[0].ls.seat(600), Some(0));
        // Lockstep never predicts, so it never rolls back.
        assert_eq!(room.peers[0].ls.rollbacks + room.peers[1].ls.rollbacks, 0);
        assert_eq!(room.peers[0].again + room.peers[1].again, 0);
    }

    #[test]
    fn rollback_under_latency_jitter_loss_and_reordering_ends_on_the_offline_world() {
        let n = 600;
        let want = offline(2, 2, n);
        for seed in [1u64, 7, 42, 9001, 0xdead_beef] {
            let mut room = Room::new(lossy(seed, 5, 4, 10));
            room.join(0, rollback(2, 2, 8));
            room.join(1, rollback(2, 2, 8));
            room.run_to(n);
            let h = room.hashes(n);
            assert!(h[0].is_some() && h[0] == h[1], "seed {seed}: {h:?}");
            for p in &room.peers {
                // The confirmed world of tick n, from the ring or the kept hashes.
                assert_eq!(p.ls.finals[&n], want, "seed {seed}, peer {}", p.id);
                assert_eq!(p.ls.dump()["desync"], json!(false), "seed {seed}");
            }
            let (rb, again): (u64, u64) = room
                .peers
                .iter()
                .map(|p| (p.ls.rollbacks, p.again))
                .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
            assert!(
                rb > 10 && again > rb,
                "seed {seed}: {rb} rollbacks, {again} ticks again"
            );
        }
    }

    #[test]
    fn commands_cross_once_after_the_state_and_are_never_predicted() {
        let n = 400;
        for mode in [cfg(2, 2, 1), rollback(2, 2, 8)] {
            let shaped = Config {
                state_bytes: 2,
                shape: 0x5eed,
                ..mode
            };
            let want = offline_with(&shaped, n);
            assert_ne!(want, offline(2, 2, n), "the commands move the world");
            let mut room = Room::new(lossy(11, 5, 4, 10));
            room.join(0, shaped);
            room.join(1, shaped);
            room.run_to(n);
            for p in &room.peers {
                assert_eq!(
                    p.ls.finals[&n], want,
                    "rollback {}, peer {}",
                    mode.rollback, p.id
                );
            }
        }
    }

    #[test]
    fn a_start_with_another_input_shape_is_refused() {
        let mut room = Room::new(None);
        room.join(
            0,
            Config {
                shape: 1,
                ..cfg(2, 2, 1)
            },
        );
        room.join(
            1,
            Config {
                shape: 2,
                ..cfg(2, 2, 1)
            },
        );
        for _ in 0..20 {
            room.frame(0, 1);
            room.frame(1, 1);
        }
        assert!(room.peers[0].ls.running());
        assert!(!room.peers[1].ls.running());
        let notes = room.peers[1].ls.take_notes().join("\n");
        assert!(notes.contains("input shape"), "{notes}");
    }

    #[test]
    fn lockstep_through_the_same_network_reaches_the_same_world_with_no_rollback() {
        let n = 400;
        let mut room = Room::new(lossy(3, 5, 4, 10));
        room.join(0, cfg(2, 2, 1));
        room.join(1, cfg(2, 2, 1));
        room.run_to(n);
        for p in &room.peers {
            assert_eq!(p.ls.finals[&n], offline(2, 2, n));
            assert_eq!((p.ls.rollbacks, p.again), (0, 0));
        }
    }

    #[test]
    fn rollback_predicts_no_further_than_max_prediction() {
        let mut room = Room::new(None);
        room.join(0, rollback(2, 2, 5));
        room.join(1, rollback(2, 2, 5));
        room.frame(0, 1);
        room.frame(1, 1);
        // Peer 1 stops: peer 0 runs on predicted input, then waits.
        let mut ran = 0;
        for _ in 0..20 {
            ran += room.frame(0, 10);
        }
        let p = &room.peers[0];
        assert_eq!(
            p.world.tick,
            p.ls.confirmed + 6,
            "confirmed {} + 5 predicted + the next",
            p.ls.confirmed
        );
        assert!(ran < 20, "{ran}");
        assert_eq!(p.ls.waiting_for, vec![1]);
        assert_eq!(p.ls.status()["predicted"], json!(5));
        // Peer 1 catches up: peer 0 rolls back once its inputs arrive and
        // both end on the offline world.
        for _ in 0..200 {
            room.frame(1, 2);
            room.frame(0, 2);
        }
        let t = room.peers[0].ls.hashed.min(room.peers[1].ls.hashed);
        assert!(t > 100);
        assert_eq!(room.peers[0].ls.finals[&t], offline(2, 2, t));
        assert_eq!(room.peers[1].ls.finals[&t], offline(2, 2, t));
        assert!(room.peers[0].ls.rollbacks >= 1);
    }

    #[test]
    fn a_late_joiner_takes_a_left_seat_from_a_snapshot_and_the_worlds_stay_equal() {
        for (mode, net) in [
            (cfg(2, 3, 4), lossy(17, 3, 3, 10)),
            (
                Config {
                    rollback: true,
                    max_prediction: 6,
                    ..cfg(2, 3, 4)
                },
                lossy(11, 3, 3, 10),
            ),
        ] {
            let mut room = Room::new(net);
            room.join(0, mode);
            room.join(1, mode);
            room.run_to(100);
            // Peer 1 leaves: its seat empties after its last input, the same for all.
            room.leave(1);
            assert!(
                room.frame(0, 40) > 0,
                "the seat left does not hold the world"
            );
            room.join(2, mode);
            room.run_to(300);
            let (a, b) = (&room.peers[0], &room.peers[1]);
            assert_eq!(b.id, 2);
            assert_eq!(
                b.ls.seat(b.world.tick + 10),
                Some(1),
                "the newcomer holds seat 1"
            );
            let t = a.ls.hashed.min(b.ls.hashed);
            assert!(t >= 300);
            assert_eq!(room.hashes(t)[0], room.hashes(t)[1], "{mode:?}");
            assert_eq!(room.peers[0].ls.dump()["desync"], json!(false), "{mode:?}");
        }
    }

    /// The world of tick `until` from the seats' holds: a seat's input is
    /// its peer's (peer number = seat) while a hold covers the tick, else
    /// empty.
    fn offline_holds(
        players: u32,
        delay: u64,
        until: u64,
        holds: &[(u32, u64, Option<u64>)],
    ) -> u64 {
        let mut state = 7;
        for t in 0..until {
            let inputs: Vec<Vec<u8>> = (0..players)
                .map(|s| {
                    let held = holds
                        .iter()
                        .any(|(hs, since, u)| *hs == s && t >= *since && u.is_none_or(|u| t <= u));
                    if held { made(s, t - delay) } else { Vec::new() }
                })
                .collect();
            state = fold(state, t, &inputs);
        }
        state
    }

    #[test]
    fn a_full_rooms_lobby_waits_for_a_slow_authoritys_start_but_not_forever() {
        // The authority is slow to take its first boundary (a loaded
        // machine): the other peer, its room full, runs no lobby tick
        // until the start arrives, so a script it runs is not spent in a
        // world the start replaces.
        let mut room = Room::new(None);
        room.join(0, cfg(2, 2, 1));
        room.join(1, cfg(2, 2, 1));
        room.peers[0].paused = true;
        for _ in 0..60 {
            room.frame(1, 2);
            room.now += 1;
            room.route();
        }
        assert_eq!(room.peers[1].world.tick, 0, "the full room's lobby held");
        room.resume(0);
        room.run_to(100);
        assert_eq!(room.hashes(100)[0], room.hashes(100)[1]);
        // An authority that never starts (lockstep off on it): the hold
        // ends after stallTimeout and the lobby runs freely.
        let mut room = Room::new(None);
        room.join(
            0,
            Config {
                enabled: false,
                ..cfg(2, 2, 1)
            },
        );
        room.join(
            1,
            Config {
                stall_timeout: 400,
                ..cfg(2, 2, 1)
            },
        );
        for _ in 0..60 {
            room.frame(0, 2);
            room.frame(1, 2);
            room.now += 1;
            room.route();
        }
        let t = room.peers[1].world.tick;
        assert!(
            t > 40 && t < 120,
            "held about 400 ms (25 frames), then free: tick {t}"
        );
    }

    #[test]
    fn a_drop_that_arrives_with_the_start_is_applied_to_the_started_run() {
        // Peer 1 stalls as the room fills: the authority starts, waits out
        // the stall timeout and drops seat 1 before its first input. When
        // peer 1 runs again its start and the drop arrive together, before
        // its next boundary: the drop must apply to the run the start
        // begins, so peer 1 rejoins from the authority's world instead of
        // playing on a seat the authority emptied.
        for mode in [
            Config {
                stall_timeout: 400,
                ..cfg(2, 2, 1)
            },
            Config {
                rollback: true,
                max_prediction: 6,
                stall_timeout: 400,
                ..cfg(2, 2, 1)
            },
        ] {
            let mut room = Room::new(None);
            room.join(0, mode);
            room.join(1, mode);
            room.peers[1].paused = true;
            for _ in 0..100 {
                room.frame(0, 2);
                room.now += 1;
                room.route();
            }
            assert!(room.peers[0].ls.running(), "{mode:?}");
            assert_eq!(room.peers[0].ls.drops, 1, "{mode:?}: seat 1 dropped");
            room.resume(1);
            let t = room.peers[0].world.tick + 200;
            room.run_to(t);
            let h = room.hashes(t);
            assert!(h[0].is_some() && h[0] == h[1], "{mode:?}: {h:?}");
            let p1 = &room.peers[1];
            assert_eq!(p1.ls.seat(p1.world.tick + 5), Some(1), "{mode:?}");
            assert_eq!(p1.ls.dump()["desync"], json!(false), "{mode:?}");
        }
    }

    #[test]
    fn a_stalled_peer_is_dropped_at_one_tick_on_every_peer_and_takes_its_seat_back() {
        let modes = [
            Config {
                stall_timeout: 400,
                ..cfg(3, 2, 1)
            },
            Config {
                rollback: true,
                max_prediction: 6,
                stall_timeout: 400,
                ..cfg(3, 2, 1)
            },
        ];
        // A seated peer stalls, then the authority itself (the next lowest
        // takes over).
        for (mode, stalled, seed) in [
            (modes[0], 2, 5),
            (modes[1], 2, 13),
            (modes[0], 0, 21),
            (modes[1], 0, 29),
        ] {
            let others: Vec<usize> = (0..3).filter(|i| *i != stalled).collect();
            let mut room = Room::new(lossy(seed, 3, 3, 10));
            for id in 0..3 {
                room.join(id, mode);
            }
            room.run_to(120);
            // Peer 2's process stops: the others wait for its seat, then
            // the authority drops it and they play on with it empty.
            room.peers[stalled].paused = true;
            let held = room.peers[others[0]].world.tick;
            for _ in 0..300 {
                for i in &others {
                    room.frame(*i, 2);
                }
                room.now += 1;
                room.route();
            }
            let a = room.peers[others[0]].world.tick;
            assert!(a > held + 60, "{mode:?}: the room held at {held}, now {a}");
            let dropped: Vec<Vec<(u64, SeatEvent)>> = others
                .iter()
                .map(|i| &room.peers[*i])
                .map(|p| {
                    p.events
                        .iter()
                        .flat_map(|(t, e)| e.iter().map(move |e| (*t, *e)))
                        .collect()
                })
                .collect();
            assert_eq!(dropped[0].len(), 1, "{mode:?}: {dropped:?}");
            assert_eq!(dropped[0], dropped[1], "{mode:?}: one drop tick for all");
            let (drop_tick, e) = dropped[0][0];
            let s = stalled as u32;
            assert_eq!((e.seat, e.peer, e.dropped, e.left), (s, s, true, false));
            // It comes back: dropped, it asks again and takes its seat back.
            room.resume(stalled);
            room.run_to(a + 150);
            let t = room.peers.iter().map(|p| p.ls.hashed).min().unwrap();
            let h = room.hashes(t);
            assert!(
                h[0].is_some() && h[0] == h[1] && h[1] == h[2],
                "{mode:?}: {h:?}"
            );
            let p2 = &room.peers[stalled];
            assert_eq!(p2.ls.seat(p2.world.tick + 5), Some(s), "{mode:?}");
            let back: Vec<Vec<u64>> = room
                .peers
                .iter()
                .map(|p| {
                    p.events
                        .iter()
                        .filter(|(_, e)| e.iter().any(|e| !e.dropped && e.seat == s && e.peer == s))
                        .map(|(t, _)| *t)
                        .collect()
                })
                .collect();
            assert_eq!(back[0].len(), 1, "{mode:?}: {back:?}");
            assert!(back.iter().all(|b| *b == back[0]), "{mode:?}: {back:?}");
            assert!(back[0][0] > drop_tick);
            // The confirmed world is the one the seats' holds give.
            let seats = room.peers[others[0]].ls.status()["seats"].clone();
            let holds: Vec<(u32, u64, Option<u64>)> = seats
                .as_array()
                .unwrap()
                .iter()
                .map(|h| {
                    (
                        u32_of(&h[0]).unwrap(),
                        u64_of(&h[2]).unwrap(),
                        u64_of(&h[3]),
                    )
                })
                .collect();
            assert_eq!(holds.iter().filter(|h| h.0 == s).count(), 2, "{holds:?}");
            assert!(holds.contains(&(s, 2, Some(drop_tick - 1))), "{holds:?}");
            assert_eq!(h[0], Some(offline_holds(3, 2, t, &holds)), "{mode:?}");
            for p in &room.peers {
                assert_eq!(p.ls.dump()["desync"], json!(false), "{mode:?}");
            }
        }
    }

    #[test]
    fn stall_timeout_0_never_drops_a_silent_peer() {
        let mut room = Room::new(None);
        let mode = Config {
            stall_timeout: 0,
            ..cfg(2, 2, 1)
        };
        for id in 0..2 {
            room.join(id, mode);
        }
        room.run_to(50);
        room.peers[1].paused = true;
        for _ in 0..2000 {
            room.frame(0, 2);
            room.now += 1;
        }
        let p = &room.peers[0];
        assert!(p.world.tick < 60, "it waits for the seat: {}", p.world.tick);
        assert!(p.events.values().all(Vec::is_empty));
        assert_eq!(p.ls.status()["stalled"], json!([1]), "reported as stalled");
    }

    #[test]
    fn a_world_that_reads_its_own_clock_is_named_at_the_first_divergent_tick() {
        for mode in [
            cfg(2, 2, 8),
            Config {
                rollback: true,
                ..cfg(2, 2, 8)
            },
        ] {
            let mut room = Room::new(None);
            room.join(0, mode);
            room.join(1, mode);
            room.frame(0, 1);
            room.frame(1, 1);
            room.peers[1].world.chaos = Some(37);
            for _ in 0..60 {
                room.frame(0, 1);
                room.frame(1, 1);
            }
            for p in &mut room.peers {
                let (tick, _, n) = p.ls.take_desync().expect("og.Desync");
                assert_eq!(tick, 38, "tick 37 ran differently: the world at 38 differs");
                assert_eq!(n, 1);
                let dump = p.ls.dump();
                assert_eq!(dump["tick"], json!(38), "{dump}");
                // The toy snapshots are not og-ecs ones: the diff says so.
                assert!(dump["diff"]["error"].is_string(), "{dump}");
                assert!(p.ls.take_desync().is_none(), "reported once");
            }
        }
    }

    #[test]
    fn a_start_from_another_wire_version_is_refused() {
        let mut room = Room::new(None);
        room.join(0, cfg(2, 2, 5));
        room.join(1, cfg(2, 2, 5));
        room.peers[0].ls.gate(&cfg(2, 2, 5), 0, &mut || {
            World {
                tick: 0,
                state: 7,
                chaos: None,
            }
            .bytes()
        });
        let mut out = room.peers[0].ls.take_out();
        let start = out.iter_mut().find(|m| m.1["k"] == json!("start")).unwrap();
        start.1["v"] = json!(1);
        room.peers[1].ls.on_event(&Event::Message {
            from: 0,
            channel: Some(LOCKSTEP_CHANNEL.into()),
            payload: Payload::Json(start.1.clone()),
        });
        assert!(room.peers[1].ls.handover.is_none());
        assert!(
            room.peers[1]
                .ls
                .take_notes()
                .iter()
                .any(|n| n.contains("version 1"))
        );
    }
}
