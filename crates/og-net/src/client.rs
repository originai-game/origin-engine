//! The relay's client: one WebSocket to `og relay` on a thread of its own,
//! ops in and events out through queues, so the engine's thread never
//! blocks on the network. `og run --room ws://127.0.0.1:P/<room>` connects
//! with the room in the path: the relay joins it at once. A relay behind a
//! TLS proxy is reached as `wss://host/<room>` (rustls, trusting the roots
//! webpki-roots carries; `og relay` itself serves `ws://`).
//!
//! A lost connection is mended here (crate::protocol's contract): the
//! thread connects again for `RECONNECT_WINDOW_MS`, asks again what was
//! unanswered or joins the room it was in again, and the engine hears the
//! new `joined`. Only a room that could not be had back is a `left`.

use crate::protocol::{
    self, MAX_FRAME, PING_MS, RECONNECT_FIRST_MS, RECONNECT_MAX_MS, RECONNECT_WINDOW_MS, SILENT_MS,
};
use rustls::{ClientConfig, RootCertStore};
use serde_json::{Value, json};
use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use tungstenite::client::IntoClientRequest;
use tungstenite::protocol::WebSocketConfig;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Connector, Message};

type Socket = tungstenite::WebSocket<MaybeTlsStream<TcpStream>>;

/// How long one connection attempt may take: reaching the host, and then
/// its handshake.
const DIAL: Duration = Duration::from_secs(3);

/// A connection to a relay. Dropping it closes the connection.
pub struct RelayClient {
    url: String,
    ops: Sender<Op>,
    events: Receiver<Value>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

/// An op on its way to the relay, by what a lost connection does with it.
enum Op {
    /// create, join or quick: asked again on a new connection.
    Ask(Value),
    Leave,
    /// A send (or anything else): lost with the connection.
    Other(String),
}

impl RelayClient {
    /// Connects (the handshake happens here, so a wrong URL is an error at
    /// once). `url` is `ws://host:port/` or `ws://host:port/<room>`, or
    /// `wss://` for a relay behind TLS (port 443 unless named).
    pub fn connect(url: &str) -> Result<RelayClient, String> {
        RelayClient::connect_for(url, Duration::from_millis(RECONNECT_WINDOW_MS))
    }

    /// As [`RelayClient::connect`], trying for `window` to get a lost
    /// connection back.
    fn connect_for(url: &str, window: Duration) -> Result<RelayClient, String> {
        let roots = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        RelayClient::connect_trusting(url, window, roots)
    }

    /// As [`RelayClient::connect_for`], with the roots a `wss://` relay's
    /// certificate must lead to.
    fn connect_trusting(
        url: &str,
        window: Duration,
        roots: RootCertStore,
    ) -> Result<RelayClient, String> {
        let Some(scheme) = ["ws://", "wss://"].into_iter().find(|s| url.starts_with(s)) else {
            return Err(format!(
                "'{url}' is not a relay URL: write ws://127.0.0.1:<port>/<room> (og relay prints its port), or wss://<host>/<room> for a relay behind TLS"
            ));
        };
        let tls = match scheme {
            "wss://" => Some(tls(roots)?),
            _ => None,
        };
        let socket = dial(url, tls.as_ref()).map_err(|e| match scheme {
            "wss://" => format!("cannot reach the relay at {url} ({e})"),
            _ => format!("cannot reach the relay at {url} ({e}); start one with `og relay`"),
        })?;
        // The relay without the room: a new connection asks for its room
        // itself, under the name this peer had.
        let authority = url[scheme.len()..].split('/').next().unwrap_or_default();
        let room = url[scheme.len() + authority.len()..].trim_matches('/');
        let (ops, ops_rx) = mpsc::channel::<Op>();
        let (events_tx, events) = mpsc::channel::<Value>();
        let mut link = Link {
            base: format!("{scheme}{authority}/"),
            tls,
            events: events_tx,
            asked: None,
            room: None,
            leaving: false,
            rejoining: false,
            window,
        };
        if protocol::valid_room(room) {
            // The relay joined the path's room as the connection opened.
            link.asked = Some(protocol::join(room, ""));
        }
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let thread = thread::Builder::new()
            .name("og-room-client".into())
            .spawn(move || run(socket, link, ops_rx, stopping))
            .map_err(|e| e.to_string())?;
        Ok(RelayClient {
            url: url.to_string(),
            ops,
            events,
            stop,
            thread: Some(thread),
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    /// Queue one op for the relay.
    pub fn send(&self, op: &Value) {
        let _ = self.ops.send(match op["op"].as_str() {
            Some("create" | "join" | "quick") => Op::Ask(op.clone()),
            Some("leave") => Op::Leave,
            _ => Op::Other(op.to_string()),
        });
    }

    /// The events that arrived since the last call, in order.
    pub fn poll(&self) -> Vec<Value> {
        self.events.try_iter().collect()
    }
}

impl Drop for RelayClient {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        // Wait for the queued ops and the close frame to leave (the thread
        // reads with a 1 ms timeout, so it sees the flag at once): a process
        // that exits right after would otherwise cut the connection, and the
        // relay would report a failure instead of a departure.
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// The TLS a `wss://` relay is reached with: rustls on ring, its certificate
/// checked against `roots`.
fn tls(roots: RootCertStore) -> Result<Arc<ClientConfig>, String> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    Ok(Arc::new(
        ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(|e| e.to_string())?
            .with_root_certificates(roots)
            .with_no_client_auth(),
    ))
}

/// One connection: reach the host and shake hands (TLS's first, with `tls`,
/// for a `wss://` URL), each within [`DIAL`].
fn dial(url: &str, tls: Option<&Arc<ClientConfig>>) -> Result<Socket, String> {
    let request = url.into_client_request().map_err(|e| e.to_string())?;
    let host = request.uri().host().unwrap_or_default().to_string();
    let port = request
        .uri()
        .port_u16()
        .unwrap_or(if tls.is_some() { 443 } else { 80 });
    let mut last = format!("'{host}' has no address");
    let addrs = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|e| e.to_string())?;
    for addr in addrs {
        let stream = match TcpStream::connect_timeout(&addr, DIAL) {
            Ok(s) => s,
            Err(e) => {
                last = e.to_string();
                continue;
            }
        };
        let _ = stream.set_nodelay(true);
        let _ = stream.set_read_timeout(Some(DIAL));
        let _ = stream.set_write_timeout(Some(DIAL));
        // The relay's frame limit: an engine message at its limit is larger
        // than the 16 MiB frame a WebSocket reads by default.
        let config = WebSocketConfig::default()
            .max_frame_size(Some(MAX_FRAME))
            .max_message_size(Some(MAX_FRAME));
        let connector = tls.map_or(Connector::Plain, |c| Connector::Rustls(c.clone()));
        match tungstenite::client_tls_with_config(
            request.clone(),
            stream,
            Some(config),
            Some(connector),
        ) {
            Ok((socket, _)) => {
                let tcp = match socket.get_ref() {
                    MaybeTlsStream::Plain(s) => Some(s),
                    MaybeTlsStream::Rustls(s) => Some(s.get_ref()),
                    _ => None,
                };
                if let Some(s) = tcp {
                    let _ = s.set_read_timeout(Some(Duration::from_millis(1)));
                    let _ = s.set_write_timeout(None);
                }
                return Ok(socket);
            }
            Err(e) => last = e.to_string(),
        }
    }
    Err(last)
}

/// The room this peer is in.
struct Room {
    id: String,
    /// This peer's number in it.
    peer: u32,
    name: String,
    /// A quick play room's key, from its `joined`.
    key: Option<String>,
}

/// What this client asked of the relay and where it stands, kept to ask
/// the same of a new connection.
struct Link {
    /// The relay's URL without a room.
    base: String,
    /// What a `wss://` relay is reached with.
    tls: Option<Arc<ClientConfig>>,
    events: Sender<Value>,
    /// The create, join or quick the relay has not answered yet.
    asked: Option<Value>,
    room: Option<Room>,
    /// A leave is on its way: `left` answers.
    leaving: bool,
    /// On a new connection, the join of `room` is on its way: until its
    /// `joined`, nothing is sent to a room this peer may not be in.
    rejoining: bool,
    window: Duration,
}

fn left(why: &str) -> Value {
    json!({ "type": "left", "reason": why })
}

impl Link {
    fn emit(&self, event: Value) {
        let _ = self.events.send(event);
    }

    fn offline(&self, op: &Value, why: &str) {
        self.emit(json!({
            "type": "error", "op": op["op"], "code": "offline",
            "message": format!("the relay at {} {why}", self.base),
        }));
    }

    /// An op while connected: the frame to write, if any.
    fn op(&mut self, op: Op) -> Option<String> {
        match op {
            Op::Ask(v) => {
                if self.room.is_none() {
                    self.asked = Some(v.clone());
                }
                Some(v.to_string())
            }
            Op::Leave => {
                self.leaving = true;
                Some(protocol::leave().to_string())
            }
            Op::Other(text) => (!self.rejoining).then_some(text),
        }
    }

    /// An op while the connection is away.
    fn op_away(&mut self, op: Op) {
        match op {
            Op::Ask(v) if self.room.is_some() => self.offline(&v, "is being reached again"),
            Op::Ask(v) => self.asked = Some(v),
            Op::Leave => self.gone("left"),
            Op::Other(_) => {}
        }
    }

    /// This peer is in no room and asks for none any more: the engine
    /// hears `left` if it was in one or waiting for one.
    fn gone(&mut self, why: &str) {
        let was = self.room.take().is_some() | self.asked.take().is_some();
        (self.leaving, self.rejoining) = (false, false);
        if was {
            self.emit(left(why));
        }
    }

    /// An event from the relay: noted, and passed on to the engine.
    fn event(&mut self, v: Value) {
        match v["type"].as_str().unwrap_or_default() {
            "joined" => {
                let name = match (self.asked.take(), self.room.take()) {
                    (Some(asked), _) => asked["name"].as_str().unwrap_or_default().to_string(),
                    (None, Some(room)) => room.name,
                    (None, None) => String::new(),
                };
                self.room = Some(Room {
                    id: v["room"].as_str().unwrap_or_default().to_string(),
                    peer: v["peer"].as_u64().unwrap_or_default() as u32,
                    name,
                    key: v["key"].as_str().map(str::to_string),
                });
                self.rejoining = false;
            }
            "left" => {
                (self.room, self.asked) = (None, None);
                (self.leaving, self.rejoining) = (false, false);
            }
            "error" if matches!(v["op"].as_str(), Some("create" | "join" | "quick")) => {
                if self.rejoining {
                    // The engine is in the room still, as far as it knows:
                    // a refusal to take it back is the room lost.
                    let why = format!(
                        "the relay connection was lost, and room '{}' did not take this peer back ({}: {})",
                        self.room.as_ref().map_or("", |r| r.id.as_str()),
                        v["code"].as_str().unwrap_or("failed"),
                        v["message"].as_str().unwrap_or_default(),
                    );
                    return self.gone(&why);
                }
                if self.room.is_none() {
                    self.asked = None;
                }
            }
            _ => {}
        }
        self.emit(v);
    }

    /// A new connection: what the old one stood for is asked of it.
    fn resume(&mut self, socket: &mut Socket) {
        if self.leaving {
            // The leave may not have reached the relay; the connection's
            // end was a departure either way.
            return self.gone("left");
        }
        let op = match (&self.room, &self.asked) {
            (Some(r), _) => {
                self.rejoining = true;
                protocol::rejoin(&r.id, &r.name, r.key.as_deref(), r.peer)
            }
            (None, Some(asked)) => asked.clone(),
            (None, None) => return,
        };
        let _ = socket.write(Message::text(op.to_string()));
        let _ = socket.flush();
    }

    /// The connection is lost: a new one within the window, or `None` when
    /// the client stops. Past the window the room is lost (`left`, or an
    /// ask's `offline`), and each later ask tries the relay once more.
    fn reconnect(&mut self, ops: &Receiver<Op>, stop: &AtomicBool, why: &str) -> Option<Socket> {
        let until = Instant::now() + self.window;
        let mut wait = Duration::from_millis(RECONNECT_FIRST_MS);
        while Instant::now() + wait <= until {
            let next = Instant::now() + wait;
            while Instant::now() < next {
                if stop.load(Ordering::Relaxed) {
                    return None;
                }
                ops.try_iter().for_each(|op| self.op_away(op));
                thread::sleep(Duration::from_millis(5));
            }
            if let Ok(mut socket) = dial(&self.base, self.tls.as_ref()) {
                ops.try_iter().for_each(|op| self.op_away(op));
                self.resume(&mut socket);
                return Some(socket);
            }
            wait = (wait * 2).min(Duration::from_millis(RECONNECT_MAX_MS));
        }
        if let Some(asked) = self.asked.take().filter(|_| self.room.is_none()) {
            self.offline(&asked, &format!("cannot be reached ({why})"));
        }
        self.gone(why);
        loop {
            if stop.load(Ordering::Relaxed) {
                return None;
            }
            match ops.recv_timeout(Duration::from_millis(20)) {
                Ok(Op::Ask(v)) => match dial(&self.base, self.tls.as_ref()) {
                    Ok(mut socket) => {
                        self.asked = Some(v);
                        self.resume(&mut socket);
                        return Some(socket);
                    }
                    Err(e) => self.offline(&v, &format!("cannot be reached ({e})")),
                },
                Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => return None,
            }
        }
    }
}

fn run(mut socket: Socket, mut link: Link, ops: Receiver<Op>, stop: Arc<AtomicBool>) {
    loop {
        let Some(why) = serve(&mut socket, &mut link, &ops, &stop) else {
            return;
        };
        match link.reconnect(&ops, &stop, why) {
            Some(s) => socket = s,
            None => return,
        }
    }
}

/// Carries ops and events over one connection until the client stops
/// (`None`) or the connection is lost (why).
fn serve(
    socket: &mut Socket,
    link: &mut Link,
    ops: &Receiver<Op>,
    stop: &AtomicBool,
) -> Option<&'static str> {
    // When the relay last said anything, and when it was last asked.
    let (mut heard, mut asked) = (Instant::now(), Instant::now());
    loop {
        if stop.load(Ordering::Relaxed) {
            // Everything queued goes before the close, so the other peers
            // get this one's last inputs before its departure.
            while let Ok(op) = ops.try_recv() {
                if let Some(frame) = link.op(op) {
                    let _ = socket.write(Message::text(frame));
                }
            }
            let _ = socket.close(None);
            let _ = socket.flush();
            return None;
        }
        // A signal interrupting a write leaves the frame queued in the
        // socket: the next flush sends it.
        let failed = |r: tungstenite::Result<()>| {
            r.is_err_and(|e| !matches!(&e, tungstenite::Error::Io(e) if e.kind() == io::ErrorKind::Interrupted))
        };
        let mut wrote = false;
        while let Ok(op) = ops.try_recv() {
            let Some(frame) = link.op(op) else {
                continue;
            };
            if failed(socket.write(Message::text(frame))) {
                return Some("the relay connection failed");
            }
            wrote = true;
        }
        let quiet = heard.elapsed();
        if quiet >= Duration::from_millis(SILENT_MS) {
            return Some("the relay fell silent");
        }
        if quiet.min(asked.elapsed()) >= Duration::from_millis(PING_MS) {
            asked = Instant::now();
            if failed(socket.write(Message::Ping(Default::default()))) {
                return Some("the relay connection failed");
            }
            wrote = true;
        }
        if wrote && failed(socket.flush()) {
            return Some("the relay connection failed");
        }
        let read = socket.read();
        if read.is_ok() {
            heard = Instant::now();
        }
        match read {
            Ok(Message::Text(text)) => {
                if let Ok(v) = serde_json::from_str::<Value>(text.as_str()) {
                    link.event(v);
                }
            }
            Ok(Message::Close(_)) => return Some("the relay closed the connection"),
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
            Err(_) => return Some("the relay connection failed"),
        }
    }
}
