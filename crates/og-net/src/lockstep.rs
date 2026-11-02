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

/// The version of lockstep's messages: a `start` of another version is
/// refused, so peers on different engines never play together.
pub const WIRE_VERSION: u64 = 2;

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

/// One peer's hold on a seat: the ticks whose inputs it sends.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Hold {
    peer: u32,
    since: u64,
    /// The last tick of its input, once it left.
    until: Option<u64>,
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
    /// Every peer ever admitted, so a newcomer is told apart.
    admitted: BTreeSet<u32>,
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
                        .map(move |h| json!([s, h.peer, h.since, h.until]))
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
}

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

impl Lockstep {
    pub fn new() -> Lockstep {
        Lockstep::default()
    }

    /// This peer's number in its room.
    pub fn me(&self) -> Option<u32> {
        self.me
    }

    /// Whether the world takes its ticks in lockstep (or rollback) now.
    pub fn running(&self) -> bool {
        self.run.is_some()
    }

    /// Whether tick `tick` runs again (a rollback's re-simulation): its
    /// sounds and effects were already presented once.
    pub fn resimulating(&self, tick: u64) -> bool {
        self.run.is_some() && tick < self.frontier
    }

    /// The last tick whose world is final (every input it ran on real and
    /// confirmed), while running.
    pub fn final_tick(&self) -> Option<u64> {
        self.run.as_ref().map(|_| self.hashed)
    }

    /// The shared hash of the final world at `tick`, while kept (the last
    /// 1024, when hashing): `None` before the tick is final.
    pub fn final_hash(&self, tick: u64) -> Option<u64> {
        self.finals.get(&tick).copied()
    }

    /// The tick the world reached furthest (the present, below which a
    /// tick is re-simulated).
    pub fn frontier(&self) -> u64 {
        self.frontier
    }

    /// This peer's seat at `tick` (`None`: watching, or not running).
    pub fn seat(&self, tick: u64) -> Option<u32> {
        let (run, me) = (self.run.as_ref()?, self.me?);
        run.seat_of(me, tick).or_else(|| run.open_seat(me))
    }

    /// The seats while running, else `None`.
    pub fn players(&self) -> Option<u32> {
        self.run.as_ref().map(|r| r.players)
    }

    /// The input og.net.submit gave last: what the local player holds.
    pub fn set_local(&mut self, input: Vec<u8>) {
        self.local = input;
    }

    pub fn local(&self) -> &[u8] {
        &self.local
    }

    /// Seat `seat`'s input for `tick`, as the tick runs it (in rollback, a
    /// prediction until the real one arrives): `None` for an empty input
    /// (a seat nobody holds then, or a tick before its holder's first).
    pub fn input(&self, tick: u64, seat: u32) -> Option<&[u8]> {
        self.used
            .get(&tick)
            .and_then(|m| m.get(seat as usize))
            .map(Vec::as_slice)
            .filter(|b| !b.is_empty())
    }

    /// The messages to send since the last call, for the lockstep channel.
    pub fn take_out(&mut self) -> Vec<Out> {
        std::mem::take(&mut self.out)
    }

    /// og.Desync, once: (first divergent tick, the first peer whose world
    /// differs, how many differ).
    pub fn take_desync(&mut self) -> Option<(u64, u32, u32)> {
        self.desync_event.take()
    }

    pub fn take_notes(&mut self) -> Vec<String> {
        std::mem::take(&mut self.notes)
    }

    fn authority(&self) -> Option<u32> {
        match &self.run {
            Some(r) => r
                .running
                .iter()
                .copied()
                .find(|p| self.peers.contains_key(p)),
            None => self.peers.keys().next().copied(),
        }
    }

    fn send(&mut self, to: Option<u32>, data: Value) {
        self.out.push((to, data, false));
    }

    /// Leaves lockstep and forgets the room (this peer left it, or lost it).
    fn reset(&mut self) {
        let local = std::mem::take(&mut self.local);
        let desync = (self.desync.take(), self.desync_event.take());
        *self = Lockstep::default();
        self.local = local;
        (self.desync, self.desync_event) = desync;
    }

    /// The state of a run starting (or joined) with the world at `tick`.
    fn begin(&mut self, run: Run, tick: u64, snapshot: Vec<u8>) {
        self.inputs.retain(|t, _| *t >= tick);
        self.used.clear();
        self.ring.clear();
        self.ring.insert(tick, (0, snapshot));
        self.confirmed = tick.saturating_sub(1);
        self.frontier = tick;
        self.hashed = tick;
        self.rewind_to = None;
        self.batch_from = tick + 1;
        self.batch.clear();
        let me = self.me;
        self.acks = run
            .running
            .iter()
            .filter(|p| Some(**p) != me)
            .map(|p| (*p, tick.saturating_sub(1)))
            .collect();
        self.run = Some(run);
    }

    /// A room event: membership, and lockstep's own messages. Game messages
    /// are not lockstep's and are ignored here.
    pub fn on_event(&mut self, event: &Event) {
        match event {
            Event::Joined { peer, peers, .. } => {
                self.reset();
                self.me = Some(*peer);
                self.peers = peers.iter().cloned().collect();
                self.peers.entry(*peer).or_default();
            }
            Event::PeerJoined { peer, name } => {
                self.peers.insert(*peer, name.clone());
            }
            Event::PeerLeft { peer } => self.departed(*peer),
            Event::Left { .. } => self.reset(),
            Event::Message {
                from,
                channel: Some(c),
                payload: Payload::Json(data),
            } if c == LOCKSTEP_CHANNEL => self.message(*from, data),
            Event::Message { .. } | Event::Error { .. } => {}
        }
    }

    fn departed(&mut self, peer: u32) {
        self.peers.remove(&peer);
        self.theirs.remove(&peer);
        self.compared.remove(&peer);
        self.acks.remove(&peer);
        let Some(run) = self.run.as_mut() else { return };
        run.running.remove(&peer);
        let mut emptied = Vec::new();
        for (s, holds) in run.seats.iter_mut().enumerate() {
            if let Some(h) = holds
                .last_mut()
                .filter(|h| h.peer == peer && h.until.is_none())
            {
                let last = self.seat_last.get(&(s as u32)).copied().unwrap_or(0);
                // Its seat is empty from the tick after its last input (an
                // empty hold when it sent none).
                let until = last.max(h.since.saturating_sub(1));
                h.until = Some(until);
                emptied.push((s, until));
                self.notes.push(format!(
                    "lockstep: peer {peer} left seat {s}; its last input is for tick {until}"
                ));
            }
        }
        // A tick past its last input that ran with a predicted input for it
        // ran wrong: the seat is empty there.
        for (s, until) in emptied {
            let wrong = self
                .used
                .range(until + 1..)
                .find(|(_, u)| u.get(s).is_some_and(|b| !b.is_empty()))
                .map(|(t, _)| *t);
            if let Some(t) = wrong {
                self.mark_wrong(t);
            }
        }
    }

    fn mark_wrong(&mut self, tick: u64) {
        self.rewind_to = Some(self.rewind_to.map_or(tick, |r| r.min(tick)));
    }

    /// A real input for `seat` at `tick`, received or made here.
    fn take_input(&mut self, tick: u64, seat: u32, bytes: Vec<u8>) {
        if tick < self.hashed
            || self
                .inputs
                .get(&tick)
                .is_some_and(|m| m.contains_key(&seat))
        {
            return;
        }
        if self
            .used
            .get(&tick)
            .and_then(|u| u.get(seat as usize))
            .is_some_and(|u| *u != bytes)
        {
            self.mark_wrong(tick);
        }
        self.inputs.entry(tick).or_default().insert(seat, bytes);
        let e = self.seat_last.entry(seat).or_insert(tick);
        *e = (*e).max(tick);
    }

    fn message(&mut self, from: u32, d: &Value) {
        match d["k"].as_str().unwrap_or_default() {
            "start" => {
                if self.run.is_some() {
                    return;
                }
                match parse_start(d) {
                    Ok((run, tick, snapshot, inputs, last)) => {
                        for (t, s, b) in inputs {
                            self.inputs.entry(t).or_default().insert(s, b);
                        }
                        for (s, t) in last {
                            let e = self.seat_last.entry(s).or_insert(t);
                            *e = (*e).max(t);
                        }
                        self.handover = Some((run, tick, snapshot));
                    }
                    Err(why) => self
                        .notes
                        .push(format!("lockstep: peer {from}'s start was refused: {why}")),
                }
            }
            "admit" => {
                let (Some(peer), Some(tick), Some(since)) =
                    (u32_of(&d["peer"]), u64_of(&d["tick"]), u64_of(&d["since"]))
                else {
                    return;
                };
                if Some(peer) == self.me {
                    return;
                }
                let Some(run) = self.run.as_mut() else { return };
                if !run.admitted.insert(peer) {
                    return;
                }
                run.running.insert(peer);
                if let Some(seat) = u32_of(&d["seat"]).filter(|s| *s < run.players) {
                    run.seats[seat as usize].push(Hold {
                        peer,
                        since,
                        until: None,
                    });
                }
                // The newcomer has none of this peer's inputs from its
                // snapshot's tick on.
                self.acks.insert(peer, tick.saturating_sub(1));
                self.send_inputs(Some(peer));
            }
            "inputs" => {
                if let Some(a) = u64_of(&d["a"])
                    && self.run.as_ref().is_some_and(|r| r.running.contains(&from))
                {
                    let e = self.acks.entry(from).or_insert(a);
                    *e = (*e).max(a);
                }
                let (Some(t0), Some(s)) = (u64_of(&d["t"]), u32_of(&d["s"])) else {
                    return;
                };
                let list = d["i"].as_array().cloned().unwrap_or_default();
                for (i, b) in list.iter().enumerate() {
                    let bytes = b.as_str().and_then(base64::decode).unwrap_or_default();
                    self.take_input(t0 + i as u64, s, bytes);
                }
            }
            "hash" => {
                let Some(first) = u64_of(&d["from"]) else {
                    return;
                };
                let list = d["h"].as_array().cloned().unwrap_or_default();
                let theirs = self.theirs.entry(from).or_default();
                for (i, h) in list.iter().enumerate() {
                    if let Some(h) = unhex(h) {
                        theirs.insert(first + i as u64, h);
                    }
                }
                self.compare(from);
            }
            "snap?" => {
                let Some(t) = u64_of(&d["t"]) else { return };
                let answer = match self.mine.get(&t) {
                    Some((_, s)) => json!({ "k": "snap", "t": t, "s": base64::encode(s) }),
                    None => json!({ "k": "snap", "t": t, "missing": true }),
                };
                self.send(Some(from), answer);
            }
            "snap" => {
                let Some(t) = u64_of(&d["t"]) else { return };
                let Some(ds) = self
                    .desync
                    .as_mut()
                    .filter(|x| x.peer == from && x.tick == t)
                else {
                    return;
                };
                match d["s"].as_str().and_then(base64::decode) {
                    Some(bytes) => {
                        ds.diff = Some(
                            match (Snapshot::decode(&ds.local), Snapshot::decode(&bytes)) {
                                (Ok(a), Ok(b)) => diff(&a, &b),
                                (Err(e), _) | (_, Err(e)) => json!({ "error": e }),
                            },
                        );
                        ds.remote = Some(bytes);
                    }
                    None => {
                        ds.diff = Some(
                            json!({ "error": format!("peer {from} no longer kept its snapshot of tick {t}") }),
                        );
                    }
                }
            }
            _ => {}
        }
    }

    /// Compares what both sides hashed with `peer`, in tick order.
    fn compare(&mut self, peer: u32) {
        let Some(theirs) = self.theirs.get_mut(&peer) else {
            return;
        };
        let ticks: Vec<u64> = theirs
            .keys()
            .copied()
            .filter(|t| self.mine.contains_key(t))
            .collect();
        for t in ticks {
            let h = theirs.remove(&t).expect("listed");
            let (mine, snapshot) = &self.mine[&t];
            if *mine == h {
                let c = self.compared.entry(peer).or_insert(t);
                *c = (*c).max(t);
                continue;
            }
            if self.desync.is_none() {
                self.desync = Some(Desync {
                    tick: t,
                    peer,
                    local_hash: *mine,
                    remote_hash: h,
                    local: snapshot.clone(),
                    remote: None,
                    diff: None,
                });
                self.desync_event = Some((t, peer, 1));
                self.notes.push(format!(
                    "lockstep: DESYNC at tick {t}: this world hashes {}, peer {peer}'s {}; dump_desync names what differs",
                    hex(*mine),
                    hex(h)
                ));
                self.out
                    .push((Some(peer), json!({ "k": "snap?", "t": t }), false));
            }
            // After the first divergence everything later differs too.
            theirs.clear();
            break;
        }
    }

    /// Whether every seat occupied at `tick` has its real input for it.
    fn known(&self, tick: u64) -> bool {
        let Some(run) = &self.run else { return true };
        let have = self.inputs.get(&tick);
        (0..run.seats.len()).all(|s| {
            run.hold(s, tick).is_none() || have.is_some_and(|m| m.contains_key(&(s as u32)))
        })
    }

    fn advance_confirmed(&mut self) {
        let limit = self.frontier + CONFIRM_AHEAD;
        while self.confirmed < limit && self.known(self.confirmed + 1) {
            self.confirmed += 1;
        }
    }

    /// The inputs tick `tick` runs with: the real ones, else each seat's
    /// last real input from its holder (empty before the holder's first).
    fn inputs_for(&self, tick: u64) -> Vec<Vec<u8>> {
        let run = self.run.as_ref().expect("running");
        let have = self.inputs.get(&tick);
        (0..run.seats.len())
            .map(|s| {
                let seat = s as u32;
                let Some(hold) = run.hold(s, tick) else {
                    return Vec::new();
                };
                if let Some(b) = have.and_then(|m| m.get(&seat)) {
                    return b.clone();
                }
                self.inputs
                    .range(hold.since..tick)
                    .rev()
                    .find_map(|(_, m)| m.get(&seat).cloned())
                    .unwrap_or_default()
            })
            .collect()
    }

    /// At a tick boundary, before tick `tick`: whether it may run.
    /// `snapshot` gives the world's snapshot now, when one is needed (to
    /// start, or for a newcomer).
    pub fn gate(&mut self, cfg: &Config, tick: u64, snapshot: &mut dyn FnMut() -> Vec<u8>) -> Gate {
        if let Some((run, at, bytes)) = self.handover.take() {
            self.notes.push(format!(
                "lockstep: joined at tick {at} as {} of {} ({} peers running, {})",
                match self.me.and_then(|me| run.open_seat(me)) {
                    Some(s) => format!("player {s}"),
                    None => "a spectator".into(),
                },
                run.players,
                run.running.len(),
                mode_text(&run),
            ));
            self.begin(run, at, bytes.clone());
            return Gate::Restore {
                snapshot: bytes,
                tick: at,
            };
        }
        let Some(me) = self.me else {
            return Gate::Go;
        };
        if !cfg.enabled {
            if self.run.take().is_some() {
                self.notes
                    .push(format!("lockstep: turned off at tick {tick}; the other peers will wait for this one's input"));
            }
            return Gate::Go;
        }
        if self.run.is_none() {
            let players = cfg.players.max(1);
            if self.authority() != Some(me) || (self.peers.len() as u32) < players {
                return Gate::Go;
            }
            self.start(cfg, tick, snapshot());
        }
        if let Some(m) = self.rewind_to.take()
            && m < tick
        {
            match self.ring.get(&m) {
                Some((_, bytes)) => {
                    self.rollbacks += 1;
                    // The ticks from m run again: what they ran with goes,
                    // so an input arriving meanwhile marks nothing stale.
                    self.used.retain(|t, _| *t < m);
                    return Gate::Rewind {
                        snapshot: bytes.clone(),
                        tick: m,
                    };
                }
                None => self.notes.push(format!(
                    "lockstep: no snapshot of tick {m} to roll back to (the ring holds {:?}); the worlds will differ",
                    self.ring.keys().next()
                )),
            }
        }
        if self.authority() == Some(me) {
            let run = self.run.as_ref().expect("running");
            let newcomers: Vec<u32> = self
                .peers
                .keys()
                .copied()
                .filter(|p| !run.admitted.contains(p))
                .collect();
            if !newcomers.is_empty() {
                // A newcomer gets a confirmed world: this one when every
                // tick run so far is, else the last final snapshot.
                let (at, bytes) = if self.confirmed + 1 >= tick && self.rewind_to.is_none() {
                    (tick, snapshot())
                } else {
                    let (t, (_, b)) = self.ring.iter().next().expect("the ring's base");
                    (*t, b.clone())
                };
                for p in newcomers {
                    self.admit(p, tick, at, &bytes);
                }
            }
        }
        self.advance_confirmed();
        let run = self.run.as_ref().expect("running");
        let predict = run.max_prediction.min(u64::from(cfg.budget));
        if tick > self.confirmed + predict {
            let next = self.confirmed + 1;
            let have = self.inputs.get(&next);
            self.waiting_for = (0..run.seats.len())
                .filter(|s| {
                    run.hold(*s, next).is_some()
                        && !have.is_some_and(|m| m.contains_key(&(*s as u32)))
                })
                .map(|s| s as u32)
                .collect();
            self.waits += 1;
            // What the others lack goes again: a lost input message, or a
            // lost acknowledgement, would otherwise hold both sides.
            self.send_inputs(None);
            return Gate::Wait;
        }
        self.waiting_for.clear();
        if tick < self.frontier {
            self.resimulated += 1;
        }
        let used = self.inputs_for(tick);
        self.used.insert(tick, used);
        Gate::Go
    }

    fn start(&mut self, cfg: &Config, tick: u64, bytes: Vec<u8>) {
        let players = cfg.players.max(1);
        let delay = u64::from(cfg.input_delay.max(1));
        let peers: Vec<u32> = self.peers.keys().copied().collect();
        let mut seats = vec![Vec::new(); players as usize];
        for (s, p) in peers.iter().take(players as usize).enumerate() {
            seats[s].push(Hold {
                peer: *p,
                since: tick + delay,
                until: None,
            });
        }
        let run = Run {
            start: tick,
            players,
            delay,
            hash_every: u64::from(cfg.hash_every),
            max_prediction: if cfg.rollback {
                u64::from(cfg.max_prediction)
            } else {
                0
            },
            seats,
            running: peers.iter().copied().collect(),
            admitted: peers.iter().copied().collect(),
        };
        let msg = start_msg(&run, tick, &bytes, &self.inputs, &self.seat_last);
        self.notes.push(format!(
            "lockstep: started at tick {tick}: {players} seats for peers {peers:?}, input delay {delay}, hashes every {} ticks, {}",
            run.hash_every,
            mode_text(&run),
        ));
        self.begin(run, tick, bytes);
        self.send(None, msg);
    }

    /// Admits `peer` at this peer's boundary `tick`, handing it the
    /// confirmed world `bytes` of tick `at`.
    fn admit(&mut self, peer: u32, tick: u64, at: u64, bytes: &[u8]) {
        let run = self.run.as_mut().expect("running");
        let free = run
            .seats
            .iter()
            .position(|h| h.last().is_none_or(|h| h.until.is_some()));
        let since = tick + run.delay;
        if let Some(s) = free {
            run.seats[s].push(Hold {
                peer,
                since,
                until: None,
            });
        }
        run.admitted.insert(peer);
        run.running.insert(peer);
        self.acks.insert(peer, at.saturating_sub(1));
        self.notes.push(format!(
            "lockstep: admitted peer {peer} at tick {tick} as {} (from the world of tick {at})",
            match free {
                Some(s) => format!("player {s} from tick {since}"),
                None => "a spectator".into(),
            }
        ));
        self.send(
            None,
            json!({ "k": "admit", "peer": peer, "seat": free, "tick": at, "since": since }),
        );
        let run = self.run.as_ref().expect("running");
        let msg = start_msg(run, at, bytes, &self.inputs, &self.seat_last);
        self.send(Some(peer), msg);
    }

    /// Sends this peer's own inputs the others lack: every input from the
    /// lowest acknowledgement on (at most [`MAX_RESEND`]), to `to` or all.
    fn send_inputs(&mut self, to: Option<u32>) {
        let (Some(run), Some(me)) = (self.run.as_ref(), self.me) else {
            return;
        };
        let floor = match to {
            Some(p) => self.acks.get(&p).copied(),
            None => self.acks.values().copied().min(),
        };
        // Every message carries the acknowledgement, inputs or not (a
        // spectator's, or a seated peer's before its first input).
        let mut msg = json!({ "k": "inputs", "a": self.confirmed });
        if let Some(floor) = floor
            && let Some((&first, _)) = self.own.range(floor + 1..).next()
            && let Some(seat) = run.seat_of(me, first)
        {
            let ticks: Vec<String> = self
                .own
                .range(first..)
                .take(MAX_RESEND as usize)
                .enumerate()
                .take_while(|(i, (t, _))| {
                    **t == first + *i as u64 && run.seat_of(me, **t) == Some(seat)
                })
                .map(|(_, (_, b))| base64::encode(b))
                .collect();
            msg["t"] = json!(first);
            msg["s"] = json!(seat);
            msg["i"] = json!(ticks);
        }
        self.out.push((to, msg, true));
    }

    /// After tick `tick_after - 1` ran (the world's tick is `tick_after`):
    /// on a tick run for the first time, this peer's input for the tick
    /// `inputDelay` on; then the world's hash and snapshot (`hash` gives
    /// both), kept for a rollback, and the hashes of every tick now final.
    pub fn after_tick(&mut self, tick_after: u64, hash: &mut dyn FnMut() -> (u64, Vec<u8>)) {
        let Some(run) = &self.run else { return };
        let ran = tick_after.saturating_sub(1);
        let (delay, every, rollback) = (run.delay, run.hash_every, run.max_prediction > 0);
        if tick_after > self.frontier {
            self.frontier = tick_after;
            let target = ran + delay;
            if let Some(seat) = self.me.and_then(|me| run.seat_of(me, target)) {
                let bytes = self.local.clone();
                self.own.insert(target, bytes.clone());
                self.take_input(target, seat, bytes);
            }
            let keep_own = tick_after.saturating_sub(KEEP_OWN + 2 * delay);
            self.own.retain(|t, _| *t >= keep_own);
            self.send_inputs(None);
        }
        if every > 0 || rollback {
            self.ring.insert(tick_after, hash());
        }
        self.advance_confirmed();
        // Every tick whose world is final now: confirmed, run with its
        // real inputs, and not marked for a rollback.
        loop {
            let h = self.hashed + 1;
            let ok = h <= tick_after
                && h - 1 <= self.confirmed
                && self.rewind_to.is_none_or(|m| m > h - 1);
            if !ok {
                break;
            }
            self.hashed = h;
            if every == 0 {
                continue;
            }
            let Some((hv, snapshot)) = self.ring.get(&h).cloned() else {
                continue;
            };
            self.finals.insert(h, hv);
            if self.finals.len() > KEEP_FINALS {
                let first = *self.finals.keys().next().expect("non-empty");
                self.finals.remove(&first);
            }
            self.mine.insert(h, (hv, snapshot));
            self.batch.push(json!(hex(hv)));
            if h.is_multiple_of(every) && !self.batch.is_empty() {
                let hashes = std::mem::take(&mut self.batch);
                let from = self.batch_from;
                self.batch_from = h + 1;
                self.send(None, json!({ "k": "hash", "from": from, "h": hashes }));
            }
        }
        let hashed = self.hashed;
        self.ring.retain(|t, _| *t >= hashed);
        self.used.retain(|t, _| *t >= hashed);
        // Each seat's last real input stays, for the next prediction.
        self.inputs.retain(|t, _| *t + 1 >= hashed);
        let peers: Vec<u32> = self.theirs.keys().copied().collect();
        for p in peers {
            self.compare(p);
        }
        // Snapshots every running peer compared (or the oldest, past the cap) go.
        let me = self.me;
        let others: Vec<u32> = self
            .run
            .as_ref()
            .map(|r| {
                r.running
                    .iter()
                    .copied()
                    .filter(|p| Some(*p) != me)
                    .collect()
            })
            .unwrap_or_default();
        let floor = others
            .iter()
            .map(|p| self.compared.get(p).copied().unwrap_or(0))
            .min();
        let desync_tick = self.desync.as_ref().map(|d| d.tick);
        if let Some(floor) = floor {
            self.mine
                .retain(|t, _| *t > floor || Some(*t) == desync_tick);
        }
        while self.mine.len() > MAX_KEPT {
            let first = *self.mine.keys().next().expect("non-empty");
            self.mine.remove(&first);
        }
    }

    /// The probe's `desync` dump: the first divergent tick, the hashes and
    /// the component-level diff of the two snapshots (`null` before any
    /// desync).
    pub fn dump(&self) -> Value {
        let Some(d) = &self.desync else {
            return json!({
                "desync": false,
                "running": self.running(),
                "compared": self.compared.iter().map(|(p, t)| json!({ "peer": p, "tick": t })).collect::<Vec<_>>(),
            });
        };
        let player = |peer: u32| {
            self.run
                .as_ref()
                .and_then(|r| r.seat_of(peer, d.tick).or_else(|| r.open_seat(peer)))
        };
        json!({
            "desync": true,
            "tick": d.tick,
            "local": { "peer": self.me, "player": self.me.and_then(player), "hash": hex(d.local_hash) },
            "remote": { "peer": d.peer, "player": player(d.peer), "hash": hex(d.remote_hash) },
            "diff": d.diff.clone().unwrap_or_else(|| json!({ "pending": format!("waiting for peer {}'s snapshot of tick {}", d.peer, d.tick) })),
        })
    }

    /// og.net.status: the room, the seats, who this peer waits for, and
    /// rollback's counts.
    pub fn status(&self) -> Value {
        let run = self.run.as_ref();
        json!({
            "running": self.running(),
            "peer": self.me,
            "peers": self.peers.keys().collect::<Vec<_>>(),
            "startTick": run.map(|r| r.start),
            "players": run.map(|r| r.players),
            "inputDelay": run.map(|r| r.delay),
            "hashEvery": run.map(|r| r.hash_every),
            "rollback": run.map(|r| r.max_prediction > 0),
            "maxPrediction": run.map(|r| r.max_prediction),
            "seats": run.map(|r| r.holds_json()),
            "spectators": run.map(|r| r.running.iter().filter(|p| r.open_seat(**p).is_none()).collect::<Vec<_>>()),
            "waitingFor": self.waiting_for,
            "waits": self.waits,
            "confirmedTick": run.map(|_| self.confirmed),
            "finalTick": run.map(|_| self.hashed),
            "acknowledged": run.and(self.acks.values().copied().min()),
            "predicted": run.map(|_| self.frontier.saturating_sub(1).saturating_sub(self.confirmed)),
            "rollbacks": self.rollbacks,
            "resimulatedTicks": self.resimulated,
            "snapshotBytes": self.ring.values().map(|(_, b)| b.len()).sum::<usize>(),
            "compared": self.compared.iter().map(|(p, t)| json!({ "peer": p, "tick": t })).collect::<Vec<_>>(),
            "desync": self.desync.as_ref().map(|d| json!({ "tick": d.tick, "peer": d.peer })),
        })
    }
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
                p.ls.on_event(&e);
            }
        }

        fn route(&mut self) {
            loop {
                let mut sent = Vec::new();
                for p in &mut self.peers {
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
                if p.ls.resimulating(t) {
                    p.again += 1;
                } else {
                    p.first += 1;
                }
                p.ls.set_local(made(p.id, t));
                let inputs: Vec<Vec<u8>> = (0..p.ls.players().unwrap_or(0))
                    .map(|s| p.ls.input(t, s).unwrap_or(&[]).to_vec())
                    .collect();
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

        /// Every peer's hash of the confirmed world at `tick`.
        fn hashes(&self, tick: u64) -> Vec<Option<u64>> {
            self.peers
                .iter()
                .map(|p| p.ls.finals.get(&tick).copied())
                .collect()
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

    /// The world of tick `until` from the inputs alone: seat s held by the
    /// peer of number s from tick `delay`, each making [`made`].
    fn offline(players: u32, delay: u64, until: u64) -> u64 {
        let mut state = 7;
        for t in 0..until {
            let inputs: Vec<Vec<u8>> = (0..players)
                .map(|s| {
                    if t >= delay {
                        made(s, t - delay)
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
            (cfg(2, 3, 4), None),
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
