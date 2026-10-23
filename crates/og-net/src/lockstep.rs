//! Deterministic lockstep over a room (docs/design.md §2.10).
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
//!   in that order) and the settings (`players`, `inputDelay`, `hashEvery`
//!   are the authority's), and each restores the snapshot. Every seat's
//!   inputs are required from `T + inputDelay`; the ticks before are empty
//!   input.
//! - **Inputs.** After tick t, a seated peer sends its input for
//!   `t + inputDelay` (the last og.net.submit, held), so tick t runs only
//!   when every occupied seat's input for t is in ([`Lockstep::gate`]); with
//!   `inputDelay` of at least 1 nobody waits on itself.
//! - **Late join.** A peer that joins a running room is admitted by the
//!   authority at its next boundary T: a free seat (one never held, or
//!   left) from `T + inputDelay`, or none (a spectator). The authority
//!   tells everyone (`admit`) and hands the newcomer a `start` with its
//!   snapshot of T, the seats' histories and every input it holds from T;
//!   every other peer sends the newcomer its own inputs from T. Delivery in
//!   order per sender makes `T + inputDelay` safe: no peer can have run a
//!   tick past it before the admit arrives, since that tick needs the
//!   authority's input sent after the admit.
//! - **Departure.** When a seated peer leaves, its seat is empty from the
//!   tick after its last input: every peer received the same inputs from it
//!   before its departure, so all agree on that tick.
//! - **Hashes.** After every tick each peer hashes its world
//!   ([`crate::state::Snapshot::shared_hash`]: everything but the peer's
//!   own session) and keeps the snapshot; every `hashEvery` ticks it sends
//!   the hashes since its last batch. The first tick whose hashes differ is
//!   the first divergent tick, exactly: og.Desync, and both peers ask each
//!   other for their snapshot of that tick, whose component-level diff
//!   ([`crate::state::diff`]) is the probe's `desync` dump. Only the first
//!   desync of a session is reported.
//!
//! The machine never touches a world: og-runtime gives it snapshots and
//! hashes, restores what [`Gate::Restore`] hands back and applies
//! [`Lockstep::input`] to og.PlayerInput.

use crate::base64;
use crate::protocol::{Event, LOCKSTEP_CHANNEL, Payload};
use crate::state::{Snapshot, diff};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// og.Lockstep's settings, as the game set them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub enabled: bool,
    pub players: u32,
    pub input_delay: u32,
    pub hash_every: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            enabled: false,
            players: 2,
            input_delay: 2,
            hash_every: 10,
        }
    }
}

/// What the world does at a tick boundary.
#[derive(Debug, PartialEq)]
pub enum Gate {
    /// Take the tick.
    Go,
    /// Hold: some seated player's input for this tick has not arrived.
    Wait,
    /// Restore this snapshot (a peer's world at `tick`: the start, or a late
    /// join), keeping the session, then ask again.
    Restore { snapshot: Vec<u8>, tick: u64 },
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

/// One engine's lockstep.
#[derive(Debug, Default)]
pub struct Lockstep {
    me: Option<u32>,
    /// The peers present, this one included, by number.
    peers: BTreeMap<u32, String>,
    run: Option<Run>,
    /// A start handed to this peer, applied at the next boundary.
    handover: Option<(Run, u64, Vec<u8>)>,
    /// Inputs by tick, then seat. Empty bytes are an empty input.
    inputs: BTreeMap<u64, BTreeMap<u32, Vec<u8>>>,
    /// The last tick each seat's input was received (or made) for.
    seat_last: BTreeMap<u32, u64>,
    /// This peer's own inputs by tick, for a newcomer.
    own: BTreeMap<u64, Vec<u8>>,
    /// The input og.net.submit gave last (packed og.PlayerInput).
    local: Vec<u8>,
    /// This peer's hash and snapshot by tick, until every peer compared.
    mine: BTreeMap<u64, (u64, Vec<u8>)>,
    /// Other peers' hashes by tick, until compared.
    theirs: BTreeMap<u32, BTreeMap<u64, u64>>,
    /// The last tick compared with each peer.
    compared: BTreeMap<u32, u64>,
    /// The first tick of the next hash batch, and its hashes so far.
    batch_from: u64,
    batch: Vec<Value>,
    desync: Option<Desync>,
    /// og.Desync to send: (tick, first peer, peers that differ).
    desync_event: Option<(u64, u32, u32)>,
    /// Messages to send: (to, data) on the lockstep channel.
    out: Vec<(Option<u32>, Value)>,
    /// Boundaries this peer waited at, and the last tick it waited for.
    waits: u64,
    waiting_for: Vec<u32>,
    /// Lines for the engine's log.
    notes: Vec<String>,
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

    /// Whether the world takes its ticks in lockstep now.
    pub fn running(&self) -> bool {
        self.run.is_some()
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

    /// Seat `seat`'s input for `tick`: `None` for an empty input (a seat
    /// nobody holds then, or a tick before its holder's first).
    pub fn input(&self, tick: u64, seat: u32) -> Option<&[u8]> {
        self.inputs
            .get(&tick)
            .and_then(|m| m.get(&seat))
            .map(Vec::as_slice)
            .filter(|b| !b.is_empty())
    }

    /// The messages to send since the last call: (to, data), for the
    /// lockstep channel.
    pub fn take_out(&mut self) -> Vec<(Option<u32>, Value)> {
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
        self.out.push((to, data));
    }

    /// Leaves lockstep and forgets the room (this peer left it, or lost it).
    fn reset(&mut self) {
        let local = std::mem::take(&mut self.local);
        let desync = (self.desync.take(), self.desync_event.take());
        *self = Lockstep::default();
        self.local = local;
        (self.desync, self.desync_event) = desync;
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
        let Some(run) = self.run.as_mut() else { return };
        run.running.remove(&peer);
        for (s, holds) in run.seats.iter_mut().enumerate() {
            if let Some(h) = holds
                .last_mut()
                .filter(|h| h.peer == peer && h.until.is_none())
            {
                let last = self.seat_last.get(&(s as u32)).copied().unwrap_or(0);
                // Its seat is empty from the tick after its last input (an
                // empty hold when it sent none).
                h.until = Some(last.max(h.since.saturating_sub(1)));
                self.notes.push(format!(
                    "lockstep: peer {peer} left seat {s}; its last input is for tick {}",
                    h.until.unwrap_or(0)
                ));
            }
        }
    }

    fn message(&mut self, from: u32, d: &Value) {
        match d["k"].as_str().unwrap_or_default() {
            "start" => {
                if self.run.is_some() {
                    return;
                }
                match parse_start(d) {
                    Some((run, tick, snapshot, inputs, last)) => {
                        for (t, s, b) in inputs {
                            self.inputs.entry(t).or_default().insert(s, b);
                        }
                        for (s, t) in last {
                            let e = self.seat_last.entry(s).or_insert(t);
                            *e = (*e).max(t);
                        }
                        self.handover = Some((run, tick, snapshot));
                    }
                    None => self
                        .notes
                        .push(format!("lockstep: a malformed start from peer {from}")),
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
                let me = self.me;
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
                // The newcomer has none of this peer's inputs from T on.
                if let Some(me) = me {
                    let mine: Vec<(u64, Vec<u8>)> = self
                        .own
                        .range(tick..)
                        .map(|(t, b)| (*t, b.clone()))
                        .collect();
                    for (t, b) in mine {
                        if let Some(s) = self.run.as_ref().and_then(|r| r.seat_of(me, t)) {
                            self.send(Some(peer), input_msg(t, s, &b));
                        }
                    }
                }
            }
            "input" => {
                let (Some(t), Some(s)) = (u64_of(&d["t"]), u32_of(&d["s"])) else {
                    return;
                };
                let bytes = d["i"].as_str().and_then(base64::decode).unwrap_or_default();
                self.inputs.entry(t).or_default().insert(s, bytes);
                let e = self.seat_last.entry(s).or_insert(t);
                *e = (*e).max(t);
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
                self.out.push((Some(peer), json!({ "k": "snap?", "t": t })));
            }
            // After the first divergence everything later differs too.
            theirs.clear();
            break;
        }
    }

    /// At a tick boundary, before tick `tick`: whether it may run.
    /// `snapshot` gives the world's snapshot now, when one is needed (to
    /// start, or for a newcomer).
    pub fn gate(&mut self, cfg: &Config, tick: u64, snapshot: &mut dyn FnMut() -> Vec<u8>) -> Gate {
        if let Some((run, at, bytes)) = self.handover.take() {
            self.inputs.retain(|t, _| *t >= at);
            self.batch_from = at + 1;
            self.batch.clear();
            self.notes.push(format!(
                "lockstep: joined at tick {at} as {} of {} ({} peers running)",
                match self.me.and_then(|me| run.open_seat(me)) {
                    Some(s) => format!("player {s}"),
                    None => "a spectator".into(),
                },
                run.players,
                run.running.len()
            ));
            self.run = Some(run);
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
        if self.authority() == Some(me) {
            let run = self.run.as_ref().expect("running");
            let newcomers: Vec<u32> = self
                .peers
                .keys()
                .copied()
                .filter(|p| !run.admitted.contains(p))
                .collect();
            if !newcomers.is_empty() {
                let bytes = snapshot();
                for p in newcomers {
                    self.admit(p, tick, &bytes);
                }
            }
        }
        if self.ready(tick) {
            Gate::Go
        } else {
            self.waits += 1;
            Gate::Wait
        }
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
            seats,
            running: peers.iter().copied().collect(),
            admitted: peers.iter().copied().collect(),
        };
        self.inputs.retain(|t, _| *t >= tick);
        self.batch_from = tick + 1;
        self.batch.clear();
        let msg = start_msg(&run, tick, &bytes, &self.inputs, &self.seat_last);
        self.notes.push(format!(
            "lockstep: started at tick {tick}: {players} seats for peers {peers:?}, input delay {delay}, hashes every {} ticks",
            run.hash_every
        ));
        self.run = Some(run);
        self.send(None, msg);
    }

    fn admit(&mut self, peer: u32, tick: u64, bytes: &[u8]) {
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
        self.notes.push(format!(
            "lockstep: admitted peer {peer} at tick {tick} as {}",
            match free {
                Some(s) => format!("player {s} from tick {since}"),
                None => "a spectator".into(),
            }
        ));
        self.send(
            None,
            json!({ "k": "admit", "peer": peer, "seat": free, "tick": tick, "since": since }),
        );
        let run = self.run.as_ref().expect("running");
        let msg = start_msg(run, tick, bytes, &self.inputs, &self.seat_last);
        self.send(Some(peer), msg);
    }

    /// Whether every occupied seat's input for `tick` is in.
    fn ready(&mut self, tick: u64) -> bool {
        let Some(run) = &self.run else { return true };
        let have = self.inputs.get(&tick);
        let missing: Vec<u32> = run
            .seats
            .iter()
            .enumerate()
            .filter(|(s, holds)| {
                holds.iter().any(|h| h.covers(tick))
                    && !have.is_some_and(|m| m.contains_key(&(*s as u32)))
            })
            .map(|(s, _)| s as u32)
            .collect();
        let ok = missing.is_empty();
        self.waiting_for = missing;
        ok
    }

    /// After tick `tick_after - 1` ran (the world's tick is `tick_after`):
    /// this peer's input for the tick `inputDelay` on, and the world's hash.
    /// `hash` gives the shared hash and the snapshot, when hashing.
    pub fn after_tick(&mut self, tick_after: u64, hash: &mut dyn FnMut() -> (u64, Vec<u8>)) {
        let Some(run) = &self.run else { return };
        let ran = tick_after.saturating_sub(1);
        let target = ran + run.delay;
        let (delay, every) = (run.delay, run.hash_every);
        if let Some(seat) = self.me.and_then(|me| run.seat_of(me, target)) {
            let bytes = self.local.clone();
            self.inputs
                .entry(target)
                .or_default()
                .insert(seat, bytes.clone());
            self.own.insert(target, bytes.clone());
            let e = self.seat_last.entry(seat).or_insert(target);
            *e = (*e).max(target);
            self.send(None, input_msg(target, seat, &bytes));
        }
        // Consumed inputs go; a newcomer may still need this peer's own.
        self.inputs.retain(|t, _| *t >= tick_after);
        let keep_own = tick_after.saturating_sub(KEEP_OWN + 2 * delay);
        self.own.retain(|t, _| *t >= keep_own);
        if every == 0 {
            return;
        }
        let (h, snapshot) = hash();
        self.mine.insert(tick_after, (h, snapshot));
        self.batch.push(json!(hex(h)));
        let peers: Vec<u32> = self.theirs.keys().copied().collect();
        for p in peers {
            self.compare(p);
        }
        if tick_after.is_multiple_of(every) && !self.batch.is_empty() {
            let hashes = std::mem::take(&mut self.batch);
            let from = self.batch_from;
            self.batch_from = tick_after + 1;
            self.send(None, json!({ "k": "hash", "from": from, "h": hashes }));
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

    /// og.net.status: the room, the seats, who this peer waits for.
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
            "seats": run.map(|r| r.holds_json()),
            "spectators": run.map(|r| r.running.iter().filter(|p| r.open_seat(**p).is_none()).collect::<Vec<_>>()),
            "waitingFor": self.waiting_for,
            "waits": self.waits,
            "compared": self.compared.iter().map(|(p, t)| json!({ "peer": p, "tick": t })).collect::<Vec<_>>(),
            "desync": self.desync.as_ref().map(|d| json!({ "tick": d.tick, "peer": d.peer })),
        })
    }
}

fn input_msg(t: u64, seat: u32, bytes: &[u8]) -> Value {
    json!({ "k": "input", "t": t, "s": seat, "i": base64::encode(bytes) })
}

fn start_msg(
    run: &Run,
    tick: u64,
    snapshot: &[u8],
    inputs: &BTreeMap<u64, BTreeMap<u32, Vec<u8>>>,
    seat_last: &BTreeMap<u32, u64>,
) -> Value {
    let inputs: Vec<Value> = inputs
        .range(tick..)
        .flat_map(|(t, m)| m.iter().map(move |(s, b)| json!([t, s, base64::encode(b)])))
        .collect();
    json!({
        "k": "start",
        "tick": tick,
        "start": run.start,
        "players": run.players,
        "delay": run.delay,
        "hashEvery": run.hash_every,
        "holds": run.holds_json(),
        "running": run.running,
        "admitted": run.admitted,
        "inputs": inputs,
        "last": seat_last.iter().map(|(s, t)| json!([s, t])).collect::<Vec<_>>(),
        "snapshot": base64::encode(snapshot),
    })
}

type Parsed = (Run, u64, Vec<u8>, Vec<(u64, u32, Vec<u8>)>, Vec<(u32, u64)>);

fn parse_start(d: &Value) -> Option<Parsed> {
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
    }

    /// The relay: every peer's messages, in order, to the others.
    struct Room {
        peers: Vec<Peer>,
    }

    impl Room {
        fn join(&mut self, id: u32, cfg: Config) {
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
            });
        }

        fn leave(&mut self, id: u32) {
            self.route();
            self.peers.retain(|p| p.id != id);
            for p in &mut self.peers {
                p.ls.on_event(&Event::PeerLeft { peer: id });
            }
        }

        fn route(&mut self) {
            loop {
                let mut moved = false;
                for i in 0..self.peers.len() {
                    let from = self.peers[i].id;
                    for (to, data) in self.peers[i].ls.take_out() {
                        moved = true;
                        let e = Event::Message {
                            from,
                            channel: Some(LOCKSTEP_CHANNEL.into()),
                            payload: Payload::Json(data),
                        };
                        for p in self.peers.iter_mut().filter(|p| p.id != from) {
                            if to.is_none_or(|t| t == p.id) {
                                p.ls.on_event(&e);
                            }
                        }
                    }
                }
                if !moved {
                    break;
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
                    Gate::Restore { snapshot, .. } => {
                        p.world.restore(&snapshot);
                        continue;
                    }
                    Gate::Wait => break,
                    Gate::Go => {}
                }
                let t = p.world.tick;
                // The local input: the peer's number, the tick it was made at.
                p.ls.set_local(vec![p.id as u8 + 1, (t % 251) as u8]);
                let mut fold = p.world.state;
                for s in 0..p.ls.players().unwrap_or(0) {
                    for b in p.ls.input(t, s).unwrap_or(&[]) {
                        fold = fold
                            .wrapping_mul(31)
                            .wrapping_add(u64::from(*b) + u64::from(s));
                    }
                }
                if p.world.chaos.is_some_and(|c| t >= c) {
                    fold ^= 0x9e37_79b9;
                }
                p.world.state = fold.wrapping_add(t);
                p.world.tick += 1;
                let w = p.world.clone();
                p.ls.after_tick(p.world.tick, &mut || (w.state, w.bytes()));
                taken += 1;
            }
            self.route();
            taken
        }
    }

    fn cfg(players: u32, delay: u32, every: u32) -> Config {
        Config {
            enabled: true,
            players,
            input_delay: delay,
            hash_every: every,
        }
    }

    #[test]
    fn two_peers_start_together_wait_for_each_other_and_agree() {
        let mut room = Room { peers: Vec::new() };
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
        let t = a.min(b);
        assert!(t > 300, "{a} {b}");
        // Run both to the same tick and compare.
        while room.peers[0].world.tick < 600 || room.peers[1].world.tick < 600 {
            if room.peers[0].world.tick < 600 {
                room.frame(0, 1);
            }
            if room.peers[1].world.tick < 600 {
                room.frame(1, 1);
            }
        }
        assert_eq!(room.peers[0].world, room.peers[1].world);
        assert!(
            room.peers[0].ls.dump()["desync"] == json!(false),
            "{}",
            room.peers[0].ls.dump()
        );
        assert_eq!(room.peers[1].ls.seat(600), Some(1));
        assert_eq!(room.peers[0].ls.seat(600), Some(0));
    }

    #[test]
    fn a_late_joiner_takes_a_left_seat_from_a_snapshot_and_the_worlds_stay_equal() {
        let mut room = Room { peers: Vec::new() };
        room.join(0, cfg(2, 3, 4));
        room.join(1, cfg(2, 3, 4));
        for _ in 0..50 {
            room.frame(0, 2);
            room.frame(1, 2);
        }
        // Peer 1 leaves: its seat empties after its last input, the same for all.
        room.leave(1);
        assert_eq!(
            room.frame(0, 40),
            40,
            "the seat left does not hold the world"
        );
        room.join(2, cfg(2, 3, 4));
        for _ in 0..80 {
            room.frame(0, 2);
            room.frame(1, 2);
        }
        let (a, b) = (&room.peers[0], &room.peers[1]);
        assert_eq!(b.id, 2);
        assert_eq!(
            b.ls.seat(b.world.tick + 10),
            Some(1),
            "the newcomer holds seat 1"
        );
        let t = a.world.tick.min(b.world.tick);
        while room.peers[0].world.tick < t + 20 || room.peers[1].world.tick < t + 20 {
            if room.peers[0].world.tick < t + 20 {
                room.frame(0, 1);
            }
            if room.peers[1].world.tick < t + 20 {
                room.frame(1, 1);
            }
        }
        assert_eq!(room.peers[0].world, room.peers[1].world);
        assert_eq!(room.peers[0].ls.dump()["desync"], json!(false));
    }

    #[test]
    fn a_world_that_reads_its_own_clock_is_named_at_the_first_divergent_tick() {
        let mut room = Room { peers: Vec::new() };
        room.join(0, cfg(2, 2, 8));
        room.join(1, cfg(2, 2, 8));
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
