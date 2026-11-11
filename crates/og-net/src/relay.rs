//! `og relay`: a local WebSocket relay that keeps rooms of peers and
//! forwards their messages, so native runs (`og run --room
//! ws://127.0.0.1:P/<room>`) play together on one machine or a LAN.
//!
//! Each connection is one peer. It speaks [`crate::protocol`]'s ops as text
//! frames and hears its events. A connection to `/<room>` joins that room
//! as soon as it opens; one to `/` sends `create` or `join` itself. A room
//! is made by its first join (or a `create`, which names it `room-N`), holds
//! up to its capacity (16 for a room made by a join) and is gone with its
//! last peer. Peer numbers count up from 0 per room and are never reused.
//!
//! Quick play (`quick`) is matchmaking: a peer waits in the queue of its
//! mode and player count until that many wait, and then they are put
//! together in a new room (`quick-<mode>-N`, its capacity the count,
//! numbered in the order they asked) and each hears `joined` with all of
//! them; nobody else can join it by id. A peer that waited `timeoutMs`
//! hears an `error` with code `timeout`; a `leave` takes it out of the queue.
//! Their `joined` carries the room's `key`: a matched peer whose connection
//! was lost joins again with it (`join`'s `key`), as a new peer number.
//!
//! A connection that says nothing is asked (a ping after `PING_MS`), and
//! one silent for `SILENT_MS` is closed and its peer leaves: a peer whose
//! network went away without a word does not hold its place in the room.
//!
//! Order is the contract lockstep rests on: one thread reads each peer's
//! frames and hands each recipient its copy through that recipient's own
//! queue, so a peer's messages reach every other peer in the order sent,
//! and its departure (`peer_left`, when its socket closes or it leaves)
//! after the last of them.
//!
//! [`Shaping`] (`og relay --latency MS --jitter MS --loss PCT --seed N`)
//! makes the relay a bad network, to test netcode against one: every frame
//! to a peer is held `latency` plus up to `jitter` milliseconds, still in
//! order; a `send` op marked `unreliable` (lockstep's inputs, which the next
//! one repeats) is dropped `loss` times in a hundred and keeps no order, so
//! it may overtake others. The draws come from `seed`, one sequence for the
//! whole relay.

use crate::protocol::{
    self, Event, LOCKSTEP_CHANNEL, MAX_CAPACITY, MAX_ENGINE_MESSAGE, MAX_FRAME, MAX_MESSAGE,
    PING_MS, SILENT_MS,
};
use og_core::IdMap;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::{self, Read};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tungstenite::Message;
use tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tungstenite::protocol::WebSocketConfig;

/// The port `og relay` serves on unless told otherwise.
pub const DEFAULT_PORT: u16 = 47900;
/// The capacity of a room a join makes.
const JOIN_CAPACITY: u32 = MAX_CAPACITY;
/// The capacity of a room `create` makes without one.
const CREATE_CAPACITY: u32 = 8;

/// The network a relay plays (all zero: a good one).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Shaping {
    /// Milliseconds every frame to a peer is held.
    pub latency_ms: u32,
    /// Up to this many milliseconds more, drawn per frame.
    pub jitter_ms: u32,
    /// Unreliable messages dropped per hundred.
    pub loss_percent: u32,
    /// The draws' seed.
    pub seed: u64,
}

/// A frame for a peer, written once `due`; `seq` orders frames due together.
type Queued = (Instant, u64, String);

struct PeerConn {
    name: String,
    tx: Sender<Queued>,
    /// The last reliable frame's due time: the next one is not earlier.
    last_due: Instant,
}

/// When a frame to `to` is due, or `None` when it is lost.
fn due(rooms: &mut Rooms, to: &mut PeerConn, lossy: bool) -> Option<Instant> {
    let sh = rooms.shaping;
    let now = Instant::now();
    let mut draw = || {
        // xorshift64*
        let r = &mut rooms.rng;
        *r ^= *r >> 12;
        *r ^= *r << 25;
        *r ^= *r >> 27;
        r.wrapping_mul(0x2545_f491_4f6c_dd1d)
    };
    if lossy && sh.loss_percent > 0 && draw() % 100 < u64::from(sh.loss_percent) {
        return None;
    }
    let jitter = if sh.jitter_ms > 0 {
        draw() % (u64::from(sh.jitter_ms) + 1)
    } else {
        0
    };
    let at = now + Duration::from_millis(u64::from(sh.latency_ms) + jitter);
    if lossy {
        return Some(at);
    }
    to.last_due = to.last_due.max(at);
    Some(to.last_due)
}

/// Hands `frame` to `to` as the shaping says.
fn post(rooms: &mut Rooms, to: u32, room: &str, frame: String, lossy: bool) {
    let mut conn = match rooms.rooms.get_mut(room).and_then(|r| r.peers.remove(&to)) {
        Some(c) => c,
        None => return,
    };
    if let Some(at) = due(rooms, &mut conn, lossy) {
        rooms.seq += 1;
        let _ = conn.tx.send((at, rooms.seq, frame));
    }
    if let Some(r) = rooms.rooms.get_mut(room) {
        r.peers.insert(to, conn);
    }
}

struct Room {
    capacity: u32,
    next_peer: u32,
    peers: IdMap<u32, PeerConn>,
    /// A quick play room's key (a join with it is one of its matched peers
    /// coming back); empty for a room anyone joins by id.
    key: String,
}

/// A key nobody guesses: the standard library's per-process random hasher
/// keys, drawn anew for each.
fn new_key() -> String {
    use std::hash::{BuildHasher, Hasher};
    let draw = || {
        std::collections::hash_map::RandomState::new()
            .build_hasher()
            .finish()
    };
    format!("{:016x}{:016x}", draw(), draw())
}

/// The `joined` event, with the room's key when it has one.
fn joined(room: &str, peer: u32, peers: Vec<(u32, String)>, key: &str) -> String {
    let mut v = Event::Joined {
        room: room.to_string(),
        peer,
        peers,
    }
    .to_json();
    if !key.is_empty() {
        v["key"] = json!(key);
    }
    v.to_string()
}

/// A room made by quick play: only its matched peers are in it.
const QUICK_PREFIX: &str = "quick-";
/// A quick play mode's longest name.
const MAX_MODE: usize = 32;

/// Where a matched peer was put: (room, peer number), taken up by its
/// own connection.
type Placed = Arc<Mutex<Option<(String, u32)>>>;

/// A peer waiting for quick play.
struct Waiting {
    conn: u64,
    name: String,
    tx: Sender<Queued>,
    placed: Placed,
}

#[derive(Default)]
struct Rooms {
    rooms: BTreeMap<String, Room>,
    next_room: u64,
    /// Quick play's queues, by (mode, players), oldest first.
    queues: BTreeMap<(String, u32), Vec<Waiting>>,
    next_conn: u64,
    shaping: Shaping,
    rng: u64,
    seq: u64,
}

type Shared = Arc<Mutex<Rooms>>;

/// A running relay. Dropping it stops accepting; open connections close
/// within a read timeout.
pub struct Relay {
    port: u16,
    stop: Arc<AtomicBool>,
    log: Receiver<String>,
    rooms: Shared,
}

impl Relay {
    /// Serve on `addr` (port 0 picks a free one).
    pub fn bind(addr: SocketAddr) -> io::Result<Relay> {
        Relay::bind_shaped(addr, Shaping::default())
    }

    /// Serve on `addr` as a network shaped by `shaping`.
    pub fn bind_shaped(addr: SocketAddr, shaping: Shaping) -> io::Result<Relay> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let port = listener.local_addr()?.port();
        let stop = Arc::new(AtomicBool::new(false));
        let (log_tx, log) = mpsc::channel();
        let rooms: Shared = Arc::new(Mutex::new(Rooms {
            shaping,
            rng: shaping.seed | 1,
            ..Rooms::default()
        }));
        let (s, r) = (stop.clone(), rooms.clone());
        thread::Builder::new()
            .name("og-relay-accept".into())
            .spawn(move || accept(listener, r, s, log_tx))?;
        Ok(Relay {
            port,
            stop,
            log,
            rooms,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// What happened since the last call: joins, departures, refusals.
    pub fn take_log(&self) -> Vec<String> {
        self.log.try_iter().collect()
    }

    /// Every room and its peers, by number.
    pub fn rooms(&self) -> Vec<(String, Vec<u32>)> {
        let rooms = self.rooms.lock().unwrap_or_else(|p| p.into_inner());
        rooms
            .rooms
            .iter()
            .map(|(id, r)| (id.clone(), r.peers.keys().copied().collect()))
            .collect()
    }
}

impl Drop for Relay {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn accept(listener: TcpListener, rooms: Shared, stop: Arc<AtomicBool>, log: Sender<String>) {
    while !stop.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => {
                let (rooms, stop, log) = (rooms.clone(), stop.clone(), log.clone());
                let _ = thread::Builder::new()
                    .name("og-relay-peer".into())
                    .spawn(move || connection(stream, rooms, stop, log));
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5))
            }
            Err(_) => thread::sleep(Duration::from_millis(50)),
        }
    }
}

/// One peer's connection: its room and number while it is in one.
struct Peer {
    tx: Sender<Queued>,
    at: Option<(String, u32)>,
    rooms: Shared,
    log: Sender<String>,
    conn: u64,
    /// Waiting for quick play: its queue and when it gives up.
    queued: Option<((String, u32), Option<Instant>)>,
    placed: Placed,
}

fn error(op: &str, code: &str, message: impl Into<String>) -> String {
    Event::Error {
        op: op.into(),
        code: code.into(),
        message: message.into(),
    }
    .to_json()
    .to_string()
}

impl Peer {
    /// A frame to this peer itself, at once and in order with the rest.
    fn reply(&self, frame: String) {
        let mut rooms = self.rooms.lock().unwrap_or_else(|p| p.into_inner());
        if let Some((room, me)) = &self.at
            && rooms
                .rooms
                .get(room)
                .is_some_and(|r| r.peers.contains_key(me))
        {
            let shaping = std::mem::take(&mut rooms.shaping);
            post(&mut rooms, *me, room, frame, false);
            rooms.shaping = shaping;
            return;
        }
        rooms.seq += 1;
        let _ = self.tx.send((Instant::now(), rooms.seq, frame));
    }

    /// Joins `room`, making it with capacity `make` (a create), or as a
    /// join by id: of any room but quick play's, which takes only a join
    /// with its `key`.
    fn join(
        &mut self,
        op: &str,
        room: &str,
        name: &str,
        make: Option<u32>,
        key: &str,
        was: Option<u32>,
    ) {
        if let Some((r, _)) = &self.at {
            return self.reply(error(
                op,
                "invalid",
                format!("already in room '{r}': leave it first"),
            ));
        }
        if self.queued.is_some() {
            return self.reply(error(op, "invalid", "waiting for quick play: leave first"));
        }
        let mut rooms = self.rooms.lock().unwrap_or_else(|p| p.into_inner());
        if make.is_none()
            && room.starts_with(QUICK_PREFIX)
            && !rooms
                .rooms
                .get(room)
                .is_some_and(|r| !key.is_empty() && r.key == key)
        {
            drop(rooms);
            return self.reply(error(
                op,
                "full",
                format!("room '{room}' is quick play's: only its matched peers are in it"),
            ));
        }
        let state = rooms.rooms.entry(room.to_string()).or_insert_with(|| Room {
            capacity: make.unwrap_or(JOIN_CAPACITY),
            next_peer: 0,
            peers: IdMap::new(),
            key: String::new(),
        });
        if state.peers.len() as u32 >= state.capacity {
            let cap = state.capacity;
            drop(rooms);
            return self.reply(error(
                op,
                "full",
                format!("room '{room}' holds {cap} peers"),
            ));
        }
        let peer = state.next_peer;
        state.next_peer += 1;
        // A number this room gave, whose holder is gone: else it is not said.
        let was = was.filter(|w| *w < peer && !state.peers.contains_key(w));
        let others: Vec<u32> = state.peers.keys().copied().collect();
        state.peers.insert(
            peer,
            PeerConn {
                name: name.to_string(),
                tx: self.tx.clone(),
                last_due: Instant::now(),
            },
        );
        let peers: Vec<(u32, String)> = state
            .peers
            .iter()
            .map(|(p, c)| (*p, c.name.clone()))
            .collect();
        let count = peers.len();
        // Under the lock, so every peer hears joins and messages in one order.
        let joined = joined(room, peer, peers, &state.key);
        rooms.seq += 1;
        let _ = self.tx.send((Instant::now(), rooms.seq, joined));
        let joined = Event::PeerJoined {
            peer,
            name: name.to_string(),
            was,
        }
        .to_json()
        .to_string();
        for p in others {
            post(&mut rooms, p, room, joined.clone(), false);
        }
        drop(rooms);
        self.at = Some((room.to_string(), peer));
        let _ = self.log.send(format!(
            "room {room}: peer {peer} joined ({count} in the room)"
        ));
    }

    /// Takes up the room quick play put this peer in, if it did.
    fn sync(&mut self) {
        if self.queued.is_some()
            && let Some(at) = self.placed.lock().unwrap_or_else(|p| p.into_inner()).take()
        {
            self.queued = None;
            self.at = Some(at);
        }
    }

    /// Takes this peer out of quick play's queue; whether it was in it
    /// (else it was matched meanwhile, and [`Peer::sync`] put it in its room).
    fn unqueue(&mut self) -> bool {
        let Some((key, _)) = self.queued.clone() else {
            return false;
        };
        let mut rooms = self.rooms.lock().unwrap_or_else(|p| p.into_inner());
        let queue = rooms.queues.get_mut(&key);
        let found = queue.is_some_and(|q| {
            let n = q.len();
            q.retain(|w| w.conn != self.conn);
            q.len() < n
        });
        if rooms.queues.get(&key).is_some_and(Vec::is_empty) {
            rooms.queues.remove(&key);
        }
        drop(rooms);
        if found {
            self.queued = None;
        } else {
            self.sync();
        }
        found
    }

    /// Quick play: waits in the queue of `mode` for `players`, or makes
    /// the room when this peer completes it.
    fn quick(&mut self, v: &Value, name: &str) {
        self.sync();
        if self.at.is_some() || self.queued.is_some() {
            let why = match &self.at {
                Some((r, _)) => format!("already in room '{r}': leave it first"),
                None => "already waiting for quick play".into(),
            };
            return self.reply(error("quick", "invalid", why));
        }
        let mode = match v["mode"].as_str().unwrap_or_default() {
            "" => "default",
            m => m,
        };
        if !protocol::valid_room(mode) || mode.len() > MAX_MODE {
            return self.reply(error(
                "quick",
                "invalid",
                format!(
                    "'{mode}' is not a mode: 1 to {MAX_MODE} of letters, digits, '.', '_' and '-'"
                ),
            ));
        }
        let players = v["players"]
            .as_u64()
            .map_or(2, |p| p.clamp(1, u64::from(MAX_CAPACITY)) as u32);
        let timeout = v["timeoutMs"]
            .as_u64()
            .map_or(u64::from(protocol::QUICK_TIMEOUT_MS), |t| {
                t.min(u64::from(protocol::QUICK_TIMEOUT_MAX_MS))
            });
        let key = (mode.to_string(), players);
        let deadline = (timeout > 0).then(|| Instant::now() + Duration::from_millis(timeout));
        self.queued = Some((key.clone(), deadline));
        let mut rooms = self.rooms.lock().unwrap_or_else(|p| p.into_inner());
        let queue = rooms.queues.entry(key.clone()).or_default();
        queue.push(Waiting {
            conn: self.conn,
            name: name.to_string(),
            tx: self.tx.clone(),
            placed: self.placed.clone(),
        });
        if queue.len() < players as usize {
            let waiting = queue.len();
            drop(rooms);
            let _ = self
                .log
                .send(format!("quick {mode} for {players}: {waiting} waiting"));
            return;
        }
        let group: Vec<Waiting> = queue.drain(..players as usize).collect();
        if queue.is_empty() {
            rooms.queues.remove(&key);
        }
        let id = loop {
            rooms.next_room += 1;
            let id = format!("{QUICK_PREFIX}{mode}-{}", rooms.next_room);
            if !rooms.rooms.contains_key(&id) {
                break id;
            }
        };
        let now = Instant::now();
        let peers: Vec<(u32, String)> = group
            .iter()
            .enumerate()
            .map(|(i, w)| (i as u32, w.name.clone()))
            .collect();
        let mut room = Room {
            capacity: players,
            next_peer: players,
            peers: IdMap::new(),
            key: new_key(),
        };
        for (i, w) in group.into_iter().enumerate() {
            let peer = i as u32;
            *w.placed.lock().unwrap_or_else(|p| p.into_inner()) = Some((id.clone(), peer));
            let joined = joined(&id, peer, peers.clone(), &room.key);
            rooms.seq += 1;
            let _ = w.tx.send((now, rooms.seq, joined));
            room.peers.insert(
                peer,
                PeerConn {
                    name: w.name,
                    tx: w.tx,
                    last_due: now,
                },
            );
        }
        rooms.rooms.insert(id.clone(), room);
        drop(rooms);
        self.sync();
        let _ = self
            .log
            .send(format!("quick {mode} for {players}: matched in room {id}"));
    }

    /// A waiting peer past its deadline hears `timeout`.
    fn expire(&mut self) {
        self.sync();
        let Some(((mode, players), Some(deadline))) = self.queued.clone() else {
            return;
        };
        if Instant::now() < deadline || !self.unqueue() {
            return;
        }
        self.reply(error(
            "quick",
            "timeout",
            format!("no {players} players for quick play '{mode}' came in time"),
        ));
    }

    fn leave(&mut self, why: &str) {
        self.sync();
        if self.unqueue() {
            return;
        }
        let Some((room, peer)) = self.at.take() else {
            return;
        };
        let mut rooms = self.rooms.lock().unwrap_or_else(|p| p.into_inner());
        let mut left = 0;
        if let Some(state) = rooms.rooms.get_mut(&room) {
            state.peers.remove(&peer);
            let gone = Event::PeerLeft { peer }.to_json().to_string();
            let others: Vec<u32> = state.peers.keys().copied().collect();
            left = others.len();
            if others.is_empty() {
                rooms.rooms.remove(&room);
            }
            // Reliable, so after every message it sent.
            for p in others {
                post(&mut rooms, p, &room, gone.clone(), false);
            }
        }
        drop(rooms);
        let _ = self.log.send(format!(
            "room {room}: peer {peer} left ({why}; {left} in the room)"
        ));
    }

    fn op(&mut self, text: &str) {
        self.sync();
        let v: Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => return self.reply(error("", "invalid", format!("not JSON: {e}"))),
        };
        let op = v["op"].as_str().unwrap_or_default().to_string();
        let name = v["name"].as_str().unwrap_or_default();
        let name = if name.is_empty() || protocol::valid_room(name) {
            name.to_string()
        } else {
            String::new()
        };
        match op.as_str() {
            "create" => {
                let capacity = v["capacity"]
                    .as_u64()
                    .map(|c| (c as u32).clamp(1, MAX_CAPACITY))
                    .unwrap_or(CREATE_CAPACITY);
                let id = {
                    let mut rooms = self.rooms.lock().unwrap_or_else(|p| p.into_inner());
                    loop {
                        rooms.next_room += 1;
                        let id = format!("room-{}", rooms.next_room);
                        if !rooms.rooms.contains_key(&id) {
                            break id;
                        }
                    }
                };
                self.join("create", &id, &name, Some(capacity), "", None);
            }
            "join" => {
                let room = v["room"].as_str().unwrap_or_default().to_string();
                if !protocol::valid_room(&room) {
                    return self.reply(error(
                        "join",
                        "invalid",
                        format!("'{room}' is not a room id: 1 to 64 of letters, digits, '.', '_' and '-'"),
                    ));
                }
                self.join(
                    "join",
                    &room,
                    &name,
                    None,
                    v["key"].as_str().unwrap_or_default(),
                    v["was"].as_u64().and_then(|w| u32::try_from(w).ok()),
                );
            }
            "leave" => {
                self.leave("asked");
                self.reply(
                    Event::Left {
                        reason: "left".into(),
                    }
                    .to_json()
                    .to_string(),
                );
            }
            "quick" => self.quick(&v, &name),
            "send" => self.send(&v),
            other => self.reply(error(
                other,
                "invalid",
                format!("'{other}' is not a room op: create, join, quick, leave or send"),
            )),
        }
    }

    fn send(&self, v: &Value) {
        let Some((room, from)) = &self.at else {
            return self.reply(error("send", "invalid", "not in a room: join one first"));
        };
        let payload = match protocol::payload_of(v) {
            Ok(p) => p,
            Err(e) => return self.reply(error("send", "invalid", e)),
        };
        let channel = v["channel"].as_str().map(str::to_string);
        let engine = channel.as_deref().is_some_and(|c| c.starts_with("og."));
        let limit = if engine {
            MAX_ENGINE_MESSAGE
        } else {
            MAX_MESSAGE
        };
        if payload.size() > limit {
            return self.reply(error(
                "send",
                "too_large",
                format!(
                    "a message is at most {limit} bytes; this one is {}",
                    payload.size()
                ),
            ));
        }
        let frame = Event::Message {
            from: *from,
            channel,
            payload,
        }
        .to_json()
        .to_string();
        let to = v["to"].as_u64().map(|t| t as u32);
        let lossy = v["unreliable"] == json!(true);
        let mut rooms = self.rooms.lock().unwrap_or_else(|p| p.into_inner());
        let Some(state) = rooms.rooms.get(room) else {
            return;
        };
        let targets: Vec<u32> = match to {
            Some(t) if state.peers.contains_key(&t) => vec![t],
            Some(_) if v["channel"] == json!(LOCKSTEP_CHANNEL) => Vec::new(),
            Some(t) => {
                drop(rooms);
                return self.reply(error(
                    "send",
                    "not_found",
                    format!("no peer {t} in room '{room}'"),
                ));
            }
            None => state.peers.keys().copied().filter(|p| p != from).collect(),
        };
        for t in targets {
            post(&mut rooms, t, room, frame.clone(), lossy);
        }
    }
}

fn connection(stream: TcpStream, rooms: Shared, stop: Arc<AtomicBool>, log: Sender<String>) {
    if stream.set_nonblocking(false).is_err() || stream.set_nodelay(true).is_err() {
        return;
    }
    let mut path = String::new();
    // tungstenite's handshake contract fixes the `Err` type.
    #[allow(clippy::result_large_err)]
    let callback = |request: &Request, response: Response| -> Result<Response, ErrorResponse> {
        path = request.uri().path().to_string();
        Ok(response)
    };
    let config = WebSocketConfig::default()
        .max_frame_size(Some(MAX_FRAME))
        .max_message_size(Some(MAX_FRAME));
    let Ok(mut socket) = tungstenite::accept_hdr_with_config(stream, callback, Some(config)) else {
        return;
    };
    let _ = socket
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(1)));
    let (tx, rx) = mpsc::channel::<Queued>();
    // Frames held until due (a shaped relay's latency), by due time.
    let mut held: Vec<Queued> = Vec::new();
    let conn = {
        let mut r = rooms.lock().unwrap_or_else(|p| p.into_inner());
        r.next_conn += 1;
        r.next_conn
    };
    let mut peer = Peer {
        tx,
        at: None,
        rooms,
        log,
        conn,
        queued: None,
        placed: Placed::default(),
    };
    let room = path.trim_matches('/').to_string();
    if !room.is_empty() {
        if protocol::valid_room(&room) {
            peer.join("join", &room, "", None, "", None);
        } else {
            peer.reply(error(
                "join",
                "invalid",
                format!("'{room}' is not a room id"),
            ));
        }
    }
    let mut unread = false;
    // When this connection last said anything, and when it was last asked.
    let (mut heard, mut asked) = (Instant::now(), Instant::now());
    let why = loop {
        if stop.load(Ordering::Relaxed) {
            break "the relay stopped";
        }
        peer.expire();
        let quiet = heard.elapsed();
        if quiet >= Duration::from_millis(SILENT_MS) {
            break "it fell silent";
        }
        if quiet.min(asked.elapsed()) >= Duration::from_millis(PING_MS) {
            asked = Instant::now();
            let _ = socket.write(Message::Ping(Default::default()));
            let _ = socket.flush();
        }
        let mut wrote = false;
        held.extend(rx.try_iter());
        held.sort_by_key(|q| (q.0, q.1));
        let now = Instant::now();
        let due = held.iter().take_while(|q| q.0 <= now).count();
        for (_, _, frame) in held.drain(..due) {
            if socket.write(Message::text(frame)).is_err() {
                break;
            }
            wrote = true;
        }
        if wrote && socket.flush().is_err() {
            break "the connection failed";
        }
        let read = socket.read();
        if read.is_ok() {
            heard = Instant::now();
        }
        match read {
            Ok(Message::Text(text)) => peer.op(text.as_str()),
            Ok(Message::Close(_)) => break "it closed the connection",
            Ok(_) => {}
            Err(tungstenite::Error::Io(e))
                if matches!(
                    e.kind(),
                    // A signal (a stopped and resumed process) interrupts
                    // the read; the connection is still there.
                    io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::Interrupted
                ) => {}
            // A frame over MAX_FRAME is refused before it is read. A
            // WebSocket cannot skip a frame, so the connection ends, but
            // the peer hears why first.
            Err(tungstenite::Error::Capacity(_)) => {
                let _ = socket.write(Message::text(error(
                    "send",
                    "too_large",
                    format!("a frame is at most {MAX_FRAME} bytes; the connection is closed"),
                )));
                unread = true;
                break "it sent a frame too large to read";
            }
            Err(_) => break "the connection failed",
        }
    };
    peer.leave(why);
    let _ = socket.close(None);
    let _ = socket.flush();
    if unread {
        discard(socket.get_mut(), &stop);
    }
}

/// Reads and drops what a peer is still sending, until it closes or ten
/// seconds pass: closing on a peer in the middle of a frame resets the
/// connection, and it would lose the answer just written to it.
fn discard(stream: &mut TcpStream, stop: &AtomicBool) {
    let until = Instant::now() + Duration::from_secs(10);
    let mut buf = vec![0u8; 64 * 1024];
    while Instant::now() < until && !stop.load(Ordering::Relaxed) {
        match stream.read(&mut buf) {
            Ok(0) => return,
            Ok(_) => {}
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::Interrupted
                ) => {}
            Err(_) => return,
        }
    }
}
