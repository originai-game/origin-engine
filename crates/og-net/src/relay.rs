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
//! Order is the contract lockstep rests on: one thread reads each peer's
//! frames and hands each recipient its copy through that recipient's own
//! queue, so a peer's messages reach every other peer in the order sent,
//! and its departure (`peer_left`, when its socket closes or it leaves)
//! after the last of them.

use crate::protocol::{
    self, Event, LOCKSTEP_CHANNEL, MAX_CAPACITY, MAX_ENGINE_MESSAGE, MAX_MESSAGE,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tungstenite::Message;
use tungstenite::handshake::server::{ErrorResponse, Request, Response};

/// The port `og relay` serves on unless told otherwise.
pub const DEFAULT_PORT: u16 = 47900;
/// The capacity of a room a join makes.
const JOIN_CAPACITY: u32 = MAX_CAPACITY;
/// The capacity of a room `create` makes without one.
const CREATE_CAPACITY: u32 = 8;

struct PeerConn {
    name: String,
    tx: Sender<String>,
}

struct Room {
    capacity: u32,
    next_peer: u32,
    peers: BTreeMap<u32, PeerConn>,
}

#[derive(Default)]
struct Rooms {
    rooms: BTreeMap<String, Room>,
    next_room: u64,
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
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let port = listener.local_addr()?.port();
        let stop = Arc::new(AtomicBool::new(false));
        let (log_tx, log) = mpsc::channel();
        let rooms: Shared = Arc::default();
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
    tx: Sender<String>,
    at: Option<(String, u32)>,
    rooms: Shared,
    log: Sender<String>,
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
    fn reply(&self, frame: String) {
        let _ = self.tx.send(frame);
    }

    fn join(&mut self, op: &str, room: &str, name: &str, make: Option<u32>) {
        if let Some((r, _)) = &self.at {
            return self.reply(error(
                op,
                "invalid",
                format!("already in room '{r}': leave it first"),
            ));
        }
        let mut rooms = self.rooms.lock().unwrap_or_else(|p| p.into_inner());
        let state = rooms.rooms.entry(room.to_string()).or_insert_with(|| Room {
            capacity: make.unwrap_or(JOIN_CAPACITY),
            next_peer: 0,
            peers: BTreeMap::new(),
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
        let others: Vec<Sender<String>> = state.peers.values().map(|p| p.tx.clone()).collect();
        state.peers.insert(
            peer,
            PeerConn {
                name: name.to_string(),
                tx: self.tx.clone(),
            },
        );
        let peers: Vec<(u32, String)> = state
            .peers
            .iter()
            .map(|(p, c)| (*p, c.name.clone()))
            .collect();
        let count = peers.len();
        // Under the lock, so every peer hears joins and messages in one order.
        self.reply(
            Event::Joined {
                room: room.to_string(),
                peer,
                peers,
            }
            .to_json()
            .to_string(),
        );
        let joined = Event::PeerJoined {
            peer,
            name: name.to_string(),
        }
        .to_json()
        .to_string();
        for tx in others {
            let _ = tx.send(joined.clone());
        }
        drop(rooms);
        self.at = Some((room.to_string(), peer));
        let _ = self.log.send(format!(
            "room {room}: peer {peer} joined ({count} in the room)"
        ));
    }

    fn leave(&mut self, why: &str) {
        let Some((room, peer)) = self.at.take() else {
            return;
        };
        let mut rooms = self.rooms.lock().unwrap_or_else(|p| p.into_inner());
        let mut left = 0;
        if let Some(state) = rooms.rooms.get_mut(&room) {
            state.peers.remove(&peer);
            let gone = Event::PeerLeft { peer }.to_json().to_string();
            for p in state.peers.values() {
                let _ = p.tx.send(gone.clone());
            }
            left = state.peers.len();
            if state.peers.is_empty() {
                rooms.rooms.remove(&room);
            }
        }
        drop(rooms);
        let _ = self.log.send(format!(
            "room {room}: peer {peer} left ({why}; {left} in the room)"
        ));
    }

    fn op(&mut self, text: &str) {
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
                self.join("create", &id, &name, Some(capacity));
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
                self.join("join", &room, &name, None);
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
            "send" => self.send(&v),
            other => self.reply(error(
                other,
                "invalid",
                format!("'{other}' is not a room op: create, join, leave or send"),
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
        let rooms = self.rooms.lock().unwrap_or_else(|p| p.into_inner());
        let Some(state) = rooms.rooms.get(room) else {
            return;
        };
        match to {
            Some(t) => match state.peers.get(&t) {
                Some(p) => {
                    let _ = p.tx.send(frame);
                }
                None if v["channel"] == json!(LOCKSTEP_CHANNEL) => {}
                None => {
                    drop(rooms);
                    self.reply(error(
                        "send",
                        "not_found",
                        format!("no peer {t} in room '{room}'"),
                    ));
                }
            },
            None => {
                for (p, c) in &state.peers {
                    if p != from {
                        let _ = c.tx.send(frame.clone());
                    }
                }
            }
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
    let Ok(mut socket) = tungstenite::accept_hdr(stream, callback) else {
        return;
    };
    let _ = socket
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(1)));
    let (tx, rx) = mpsc::channel::<String>();
    let mut peer = Peer {
        tx,
        at: None,
        rooms,
        log,
    };
    let room = path.trim_matches('/').to_string();
    if !room.is_empty() {
        if protocol::valid_room(&room) {
            peer.join("join", &room, "", None);
        } else {
            peer.reply(error(
                "join",
                "invalid",
                format!("'{room}' is not a room id"),
            ));
        }
    }
    let why = loop {
        if stop.load(Ordering::Relaxed) {
            break "the relay stopped";
        }
        let mut wrote = false;
        while let Ok(frame) = rx.try_recv() {
            if socket.write(Message::text(frame)).is_err() {
                break;
            }
            wrote = true;
        }
        if wrote && socket.flush().is_err() {
            break "the connection failed";
        }
        match socket.read() {
            Ok(Message::Text(text)) => peer.op(text.as_str()),
            Ok(Message::Close(_)) => break "it closed the connection",
            Ok(_) => {}
            Err(tungstenite::Error::Io(e))
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) => {}
            Err(_) => break "the connection failed",
        }
    };
    peer.leave(why);
    let _ = socket.close(None);
    let _ = socket.flush();
}
