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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{self, Event, Payload};
    use crate::relay::Relay;
    use std::net::{Ipv4Addr, SocketAddr};

    fn until(c: &RelayClient, n: usize) -> Vec<Event> {
        let mut got = Vec::new();
        for _ in 0..2000 {
            got.extend(c.poll().iter().map(|v| Event::parse(v).expect("an event")));
            if got.len() >= n {
                return got;
            }
            thread::sleep(Duration::from_millis(1));
        }
        panic!("waited for {n} events, got {got:?}");
    }

    #[test]
    fn two_clients_meet_in_a_room_by_path_and_messages_keep_their_order() {
        let relay = Relay::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).unwrap();
        let url = format!("ws://127.0.0.1:{}/arena", relay.port());
        let a = RelayClient::connect(&url).unwrap();
        let joined = until(&a, 1);
        assert!(
            matches!(&joined[0], Event::Joined { room, peer: 0, .. } if room == "arena"),
            "{joined:?}"
        );
        let b = RelayClient::connect(&url).unwrap();
        let joined = until(&b, 1);
        assert!(
            matches!(&joined[0], Event::Joined { peer: 1, peers, .. } if peers.len() == 2),
            "{joined:?}"
        );
        assert_eq!(
            until(&a, 1),
            vec![Event::PeerJoined {
                peer: 1,
                name: String::new(),
                was: None,
            }]
        );
        for i in 0..100 {
            a.send(&protocol::send(
                None,
                None,
                &Payload::Json(serde_json::json!(i)),
            ));
        }
        a.send(&protocol::send(
            Some(1),
            Some("og.lockstep"),
            &Payload::Bytes(vec![1, 2, 3]),
        ));
        // a leaves after its messages: b hears them all, then the departure.
        drop(a);
        let got = until(&b, 102);
        for (i, e) in got[..100].iter().enumerate() {
            assert_eq!(
                e,
                &Event::Message {
                    from: 0,
                    channel: None,
                    payload: Payload::Json(serde_json::json!(i))
                }
            );
        }
        assert_eq!(
            got[100],
            Event::Message {
                from: 0,
                channel: Some("og.lockstep".into()),
                payload: Payload::Bytes(vec![1, 2, 3])
            }
        );
        assert_eq!(got[101], Event::PeerLeft { peer: 0 });
        // A room made by create gets a name; a full one refuses.
        let c = RelayClient::connect(&format!("ws://127.0.0.1:{}/", relay.port())).unwrap();
        c.send(&protocol::create("", 1));
        let e = until(&c, 1);
        let Event::Joined { room, .. } = &e[0] else {
            panic!("{e:?}")
        };
        let d = RelayClient::connect(&format!("ws://127.0.0.1:{}/", relay.port())).unwrap();
        d.send(&protocol::join(room, "d"));
        assert!(matches!(&until(&d, 1)[0], Event::Error { code, .. } if code == "full"));
    }

    /// A TCP proxy to a relay whose connections a test cuts, as a network
    /// that goes away does: both ends see the connection fail.
    struct Wire {
        port: u16,
        open: Arc<AtomicBool>,
        live: Arc<std::sync::Mutex<Vec<TcpStream>>>,
        stop: Arc<AtomicBool>,
    }

    impl Wire {
        fn to(relay: u16) -> Wire {
            let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
            listener.set_nonblocking(true).unwrap();
            let port = listener.local_addr().unwrap().port();
            let open = Arc::new(AtomicBool::new(true));
            let live = Arc::new(std::sync::Mutex::new(Vec::new()));
            let stop = Arc::new(AtomicBool::new(false));
            let (o, l, s) = (open.clone(), live.clone(), stop.clone());
            thread::spawn(move || {
                while !s.load(Ordering::Relaxed) {
                    let Ok((down, _)) = listener.accept() else {
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    };
                    if !o.load(Ordering::Relaxed) {
                        continue;
                    }
                    down.set_nonblocking(false).unwrap();
                    let up = TcpStream::connect((Ipv4Addr::LOCALHOST, relay)).unwrap();
                    for (mut from, mut to) in [
                        (down.try_clone().unwrap(), up.try_clone().unwrap()),
                        (up.try_clone().unwrap(), down.try_clone().unwrap()),
                    ] {
                        thread::spawn(move || {
                            let _ = io::copy(&mut from, &mut to);
                            let _ = to.shutdown(std::net::Shutdown::Both);
                        });
                    }
                    l.lock().unwrap().extend([down, up]);
                }
            });
            Wire {
                port,
                open,
                live,
                stop,
            }
        }

        fn url(&self, room: &str) -> String {
            format!("ws://127.0.0.1:{}/{room}", self.port)
        }

        /// Cuts every connection through the wire.
        fn cut(&self) {
            for s in self.live.lock().unwrap().drain(..) {
                let _ = s.shutdown(std::net::Shutdown::Both);
            }
        }
    }

    impl Drop for Wire {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
            self.cut();
        }
    }

    /// A relay behind TLS, as much of one as a client's first words need:
    /// it shakes hands under `localhost`'s certificate, says `joined` to
    /// every connection, sends back what it is sent, and drops the
    /// connection that sends `bye`. Returns its port and the certificate.
    fn tls_relay() -> (u16, rustls::pki_types::CertificateDer<'static>) {
        use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
        let made = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
        let cert = made.cert.der().clone();
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(made.signing_key.serialize_der()));
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = Arc::new(
            rustls::ServerConfig::builder_with_provider(provider)
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_no_client_auth()
                .with_single_cert(vec![cert.clone()], key)
                .unwrap(),
        );
        let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        thread::spawn(move || {
            for tcp in listener.incoming().flatten() {
                let config = config.clone();
                thread::spawn(move || {
                    let conn = rustls::ServerConnection::new(config).unwrap();
                    let stream = rustls::StreamOwned::new(conn, tcp);
                    // A client that does not trust the certificate ends here.
                    let Ok(mut ws) = tungstenite::accept(stream) else {
                        return;
                    };
                    let joined = Event::Joined {
                        room: "arena".into(),
                        peer: 0,
                        peers: vec![(0, String::new())],
                    };
                    let _ = ws.send(Message::text(joined.to_json().to_string()));
                    while let Ok(m) = ws.read() {
                        let Message::Text(t) = m else { continue };
                        let op: Value = serde_json::from_str(&t).unwrap();
                        if op["data"] == "bye" {
                            return;
                        }
                        if op["op"] == "send" {
                            let back = json!({ "type": "message", "from": 0, "data": op["data"] });
                            let _ = ws.send(Message::text(back.to_string()));
                        }
                    }
                });
            }
        });
        (port, cert)
    }

    #[test]
    fn a_relay_behind_tls_is_reached_as_wss_and_only_under_a_trusted_certificate() {
        let (port, cert) = tls_relay();
        let url = format!("wss://localhost:{port}/arena");
        let mut roots = RootCertStore::empty();
        roots.add(cert).unwrap();
        let window = Duration::from_millis(RECONNECT_WINDOW_MS);
        let c = RelayClient::connect_trusting(&url, window, roots).unwrap();
        assert!(matches!(
            &until(&c, 1)[0],
            Event::Joined { room, peer: 0, .. } if room == "arena"
        ));
        // Both ways through TLS, past the 1 ms read timeout the thread polls with.
        let big = "x".repeat(200_000);
        c.send(&protocol::send(None, None, &Payload::Json(json!(big))));
        assert_eq!(
            until(&c, 1),
            [Event::Message {
                from: 0,
                channel: None,
                payload: Payload::Json(json!(big)),
            }]
        );
        // A lost connection is mended through TLS too.
        c.send(&protocol::send(None, None, &Payload::Json(json!("bye"))));
        assert!(matches!(&until(&c, 1)[0], Event::Joined { .. }));
        c.send(&protocol::send(None, None, &Payload::Json(json!("again"))));
        assert!(matches!(&until(&c, 1)[0], Event::Message { .. }));
        // The roots a client ships with do not know this certificate.
        let e = RelayClient::connect(&url).err().unwrap();
        assert!(
            e.contains("cannot reach the relay") && e.contains("UnknownIssuer"),
            "{e}"
        );
        // Without TLS the same port answers nothing a WebSocket reads.
        assert!(RelayClient::connect(&format!("ws://localhost:{port}/arena")).is_err());
        let e = RelayClient::connect("https://localhost/").err().unwrap();
        assert!(e.contains("wss://"), "{e}");
    }

    #[test]
    fn a_cut_connection_is_mended_and_the_peer_is_back_in_its_room_under_its_name() {
        let relay = Relay::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).unwrap();
        let wire = Wire::to(relay.port());
        let a = RelayClient::connect(&format!("ws://127.0.0.1:{}/", relay.port())).unwrap();
        a.send(&protocol::join("arena", "ann"));
        until(&a, 1);
        // One peer joins by an op under a name, one by the URL's path.
        let b = RelayClient::connect(&wire.url("")).unwrap();
        b.send(&protocol::join("arena", "bob"));
        assert!(matches!(&until(&b, 1)[0], Event::Joined { peer: 1, .. }));
        let c = RelayClient::connect(&wire.url("arena")).unwrap();
        assert!(matches!(&until(&c, 1)[0], Event::Joined { peer: 2, .. }));
        until(&a, 2);
        until(&b, 1);

        wire.cut();
        // The room loses both at once and has them back as new peers, bob
        // under his name: a seat is taken back by it.
        let heard = until(&a, 4);
        let mut gone: Vec<u32> = Vec::new();
        let mut back: Vec<(u32, String, Option<u32>)> = Vec::new();
        for e in &heard {
            match e {
                Event::PeerLeft { peer } => gone.push(*peer),
                Event::PeerJoined { peer, name, was } => back.push((*peer, name.clone(), *was)),
                other => panic!("{other:?}"),
            }
        }
        gone.sort();
        back.sort_by(|x, y| x.1.cmp(&y.1));
        assert_eq!(gone, [1, 2]);
        assert_eq!(back.len(), 2, "{heard:?}");
        assert!(back[0].1.is_empty() && back[1].1 == "bob", "{back:?}");
        assert!(back.iter().all(|(p, ..)| *p == 3 || *p == 4), "{back:?}");
        // Each hears `joined` again, with its new number, and never `left`.
        for (client, name, was) in [(&b, "bob", 1), (&c, "", 2)] {
            let mut got = Vec::new();
            let me = loop {
                got.extend(until(client, 1));
                if let Some(Event::Joined { room, peer, .. }) =
                    got.iter().find(|e| matches!(e, Event::Joined { .. }))
                {
                    assert_eq!(room, "arena");
                    break *peer;
                }
            };
            // Each says the number it had: a nameless peer's seat is its own too.
            assert!(
                back.contains(&(me, name.to_string(), Some(was))),
                "{back:?} {me}"
            );
            assert!(
                got.iter().all(|e| !matches!(e, Event::Left { .. })),
                "{got:?}"
            );
        }
        // And the room carries its messages again.
        b.send(&protocol::send(
            Some(0),
            None,
            &Payload::Json(serde_json::json!("back")),
        ));
        assert!(matches!(
            &until(&a, 1)[0],
            Event::Message { payload: Payload::Json(v), .. } if v == "back"
        ));
    }

    #[test]
    fn a_quick_play_room_takes_back_its_matched_peer_and_nobody_without_its_key() {
        let relay = Relay::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).unwrap();
        let wire = Wire::to(relay.port());
        let direct = format!("ws://127.0.0.1:{}/", relay.port());
        let a = RelayClient::connect(&direct).unwrap();
        let b = RelayClient::connect(&wire.url("")).unwrap();
        a.send(&protocol::quick("duel", 2, 0, "ann"));
        b.send(&protocol::quick("duel", 2, 0, "bob"));
        let Event::Joined { room, .. } = until(&a, 1).remove(0) else {
            panic!()
        };
        // Their numbers are the order the relay heard them ask in.
        let Event::Joined { peer: was, .. } = until(&b, 1).remove(0) else {
            panic!()
        };
        wire.cut();
        assert_eq!(until(&a, 1), [Event::PeerLeft { peer: was }]);
        assert_eq!(
            until(&a, 1),
            [Event::PeerJoined {
                peer: 2,
                name: "bob".into(),
                was: Some(was),
            }]
        );
        assert!(matches!(
            &until(&b, 1)[0],
            Event::Joined { room: r, peer: 2, peers } if *r == room && peers.len() == 2
        ));
        // The room has a place free again; a stranger still gets none.
        drop(b);
        assert_eq!(until(&a, 1), [Event::PeerLeft { peer: 2 }]);
        let d = RelayClient::connect(&direct).unwrap();
        d.send(&protocol::join(&room, "bob"));
        d.send(&protocol::rejoin(&room, "bob", Some("0000"), was));
        for e in until(&d, 2) {
            assert!(
                matches!(&e, Event::Error { code, .. } if code == "full"),
                "{e:?}"
            );
        }
    }

    #[test]
    fn a_relay_that_stays_away_is_the_room_lost_and_a_later_ask_tries_it_again() {
        let relay = Relay::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).unwrap();
        let wire = Wire::to(relay.port());
        let window = Duration::from_millis(600);
        let b = RelayClient::connect_for(&wire.url("arena"), window).unwrap();
        until(&b, 1);
        wire.open.store(false, Ordering::Relaxed);
        wire.cut();
        let cut = Instant::now();
        // What is sent meanwhile is lost; the room is, once the window passed.
        b.send(&protocol::send(
            None,
            None,
            &Payload::Json(serde_json::json!(1)),
        ));
        assert_eq!(
            until(&b, 1),
            [Event::Left {
                reason: "the relay connection failed".into()
            }]
        );
        assert!(
            cut.elapsed() >= Duration::from_millis(250),
            "it tried again first"
        );
        b.send(&protocol::join("arena", "bob"));
        assert!(matches!(
            &until(&b, 1)[0],
            Event::Error { op, code, .. } if op == "join" && code == "offline"
        ));
        wire.open.store(true, Ordering::Relaxed);
        b.send(&protocol::join("arena", "bob"));
        assert!(matches!(&until(&b, 1)[0], Event::Joined { peer: 0, .. }));
        // An ask the lost connection left unanswered is refused the same way.
        let c = RelayClient::connect_for(&wire.url(""), window).unwrap();
        wire.open.store(false, Ordering::Relaxed);
        wire.cut();
        thread::sleep(Duration::from_millis(50));
        c.send(&protocol::quick("duel", 2, 0, "cat"));
        assert!(matches!(
            &until(&c, 1)[0],
            Event::Error { op, code, .. } if op == "quick" && code == "offline"
        ));
    }

    #[test]
    fn a_connection_gone_without_a_word_is_noticed_by_the_relay_and_by_the_client() {
        use crate::protocol::SILENT_MS;
        // To the relay: a peer that answers no ping leaves its room.
        let relay = Relay::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).unwrap();
        let url = format!("ws://127.0.0.1:{}/arena", relay.port());
        let a = RelayClient::connect(&url).unwrap();
        until(&a, 1);
        let (mute, _) = tungstenite::connect(&url).unwrap();
        assert_eq!(
            until(&a, 1),
            [Event::PeerJoined {
                peer: 1,
                name: String::new(),
                was: None,
            }]
        );
        // To the client: a relay that answers no ping is connected to again.
        let silent = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = silent.local_addr().unwrap().port();
        let accepted = thread::spawn(move || {
            let mut held = Vec::new();
            let mut at = Vec::new();
            for _ in 0..2 {
                let (stream, _) = silent.accept().unwrap();
                held.push(tungstenite::accept(stream).unwrap());
                at.push(Instant::now());
            }
            at[1] - at[0]
        });
        let b = RelayClient::connect(&format!("ws://127.0.0.1:{port}/")).unwrap();
        let start = Instant::now();
        let silence = Duration::from_millis(SILENT_MS);
        assert_eq!(slowly(&a, 1), [Event::PeerLeft { peer: 1 }]);
        assert!(start.elapsed() >= silence - Duration::from_millis(500));
        assert!(accepted.join().unwrap() >= silence);
        assert!(b.poll().is_empty());
        drop(mute);
    }

    /// As `until`, for up to a minute: a 16 MiB message takes seconds in a
    /// debug build.
    fn slowly(c: &RelayClient, n: usize) -> Vec<Event> {
        let mut got = Vec::new();
        for _ in 0..60_000 {
            got.extend(c.poll().iter().map(|v| Event::parse(v).expect("an event")));
            if got.len() >= n {
                return got;
            }
            thread::sleep(Duration::from_millis(1));
        }
        panic!("waited a minute for {n} events, got {}", got.len());
    }

    #[test]
    fn an_engine_message_at_its_limit_crosses_and_a_larger_one_is_answered_too_large() {
        use crate::protocol::MAX_ENGINE_MESSAGE;
        let relay = Relay::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).unwrap();
        let url = format!("ws://127.0.0.1:{}/big", relay.port());
        let a = RelayClient::connect(&url).unwrap();
        until(&a, 1);
        let b = RelayClient::connect(&url).unwrap();
        until(&b, 1);
        until(&a, 1);
        // The largest engine message (a late joiner's snapshot) arrives whole.
        let mut world = vec![7u8; MAX_ENGINE_MESSAGE];
        world[MAX_ENGINE_MESSAGE - 1] = 9;
        a.send(&protocol::send(
            Some(1),
            Some("og.lockstep"),
            &Payload::Bytes(world.clone()),
        ));
        let Event::Message {
            from: 0,
            payload: Payload::Bytes(got),
            ..
        } = slowly(&b, 1).remove(0)
        else {
            panic!("b did not get the message")
        };
        assert!(got == world, "the bytes changed on the way");
        // One byte more is refused by name, and the sender stays in the room.
        world.push(1);
        a.send(&protocol::send(
            Some(1),
            Some("og.lockstep"),
            &Payload::Bytes(world),
        ));
        let refused = slowly(&a, 1);
        assert!(
            matches!(&refused[0], Event::Error { op, code, .. } if op == "send" && code == "too_large"),
            "{refused:?}"
        );
        a.send(&protocol::send(
            None,
            None,
            &Payload::Json(serde_json::json!("still here")),
        ));
        assert_eq!(
            slowly(&b, 1),
            vec![Event::Message {
                from: 0,
                channel: None,
                payload: Payload::Json(serde_json::json!("still here"))
            }]
        );
        // A frame the relay cannot read at all is refused by name too, and
        // then the connection ends: a WebSocket cannot skip a frame. The
        // client is back in the room on a new one, as a new peer.
        a.send(&serde_json::json!({ "op": "send", "data": "x".repeat(protocol::MAX_FRAME) }));
        let ended = slowly(&a, 2);
        assert!(
            matches!(&ended[0], Event::Error { op, code, .. } if op == "send" && code == "too_large"),
            "{ended:?}"
        );
        assert!(
            matches!(&ended[1], Event::Joined { room, peer: 2, .. } if room == "big"),
            "{ended:?}"
        );
        assert_eq!(
            slowly(&b, 2),
            vec![
                Event::PeerLeft { peer: 0 },
                Event::PeerJoined {
                    peer: 2,
                    name: String::new(),
                    was: Some(0),
                }
            ]
        );
    }

    #[test]
    fn quick_play_matches_peers_of_one_mode_in_the_order_they_asked_and_times_out_alone() {
        let relay = Relay::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).unwrap();
        let url = format!("ws://127.0.0.1:{}/", relay.port());
        let peer = || RelayClient::connect(&url).unwrap();
        let (a, b, c, d) = (peer(), peer(), peer(), peer());
        // d waits for a duel and leaves the queue before anyone comes.
        d.send(&protocol::quick("duel", 2, 0, "d"));
        thread::sleep(Duration::from_millis(30));
        d.send(&protocol::leave());
        assert_eq!(
            until(&d, 1)[0],
            Event::Left {
                reason: "left".into()
            }
        );
        // a waits for a duel, b for a trio of the same mode: nobody is matched.
        a.send(&protocol::quick("duel", 2, 0, "a"));
        thread::sleep(Duration::from_millis(30));
        b.send(&protocol::quick("duel", 3, 0, "b"));
        thread::sleep(Duration::from_millis(30));
        assert!(a.poll().is_empty() && b.poll().is_empty());
        // c completes a's duel.
        c.send(&protocol::quick("duel", 2, 0, "c"));
        let peers = vec![(0, "a".to_string()), (1, "c".to_string())];
        let Event::Joined {
            room,
            peer: 0,
            peers: got,
        } = until(&a, 1).remove(0)
        else {
            panic!("a was not matched first")
        };
        assert!(room.starts_with("quick-duel-"), "{room}");
        assert_eq!(got, peers);
        assert_eq!(
            until(&c, 1)[0],
            Event::Joined {
                room: room.clone(),
                peer: 1,
                peers
            }
        );
        // The matched room carries messages; nobody joins it by id.
        a.send(&protocol::send(
            None,
            None,
            &Payload::Json(serde_json::json!("gg")),
        ));
        assert!(matches!(&until(&c, 1)[0], Event::Message { from: 0, .. }));
        d.send(&protocol::join(&room, "d"));
        assert!(matches!(&until(&d, 1)[0], Event::Error { code, .. } if code == "full"));
        // b's trio never fills; e's own wait ends in a timeout.
        let e = peer();
        e.send(&protocol::quick("solo-ish", 2, 50, "e"));
        assert!(matches!(
            &until(&e, 1)[0],
            Event::Error { op, code, .. } if op == "quick" && code == "timeout"
        ));
        assert!(b.poll().is_empty());
        // Out of the queue, e can wait again, and a mode is a room-id word.
        e.send(&protocol::quick("bad mode", 2, 0, "e"));
        assert!(matches!(&until(&e, 1)[0], Event::Error { code, .. } if code == "invalid"));
    }

    #[test]
    fn a_shaped_relay_holds_frames_drops_only_unreliable_ones_and_keeps_reliable_order() {
        use crate::relay::Shaping;
        let shaping = Shaping {
            latency_ms: 40,
            jitter_ms: 20,
            loss_percent: 50,
            seed: 9,
        };
        let relay =
            Relay::bind_shaped(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)), shaping).unwrap();
        let url = format!("ws://127.0.0.1:{}/lag", relay.port());
        let a = RelayClient::connect(&url).unwrap();
        until(&a, 1);
        let b = RelayClient::connect(&url).unwrap();
        until(&b, 1);
        until(&a, 1);
        let sent = std::time::Instant::now();
        for i in 0..300 {
            let mut op = protocol::send(None, None, &Payload::Json(serde_json::json!(i)));
            if i % 3 != 0 {
                op["unreliable"] = serde_json::json!(true);
            }
            a.send(&op);
        }
        let first = until(&b, 1);
        assert!(
            sent.elapsed() >= Duration::from_millis(40),
            "a frame arrived {:?} after sending, under the 40 ms latency",
            sent.elapsed()
        );
        drop(a);
        let mut got = first;
        for _ in 0..3000 {
            got.extend(b.poll().iter().map(|v| Event::parse(v).expect("an event")));
            if got.last() == Some(&Event::PeerLeft { peer: 0 }) {
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(got.last(), Some(&Event::PeerLeft { peer: 0 }), "{got:?}");
        let values: Vec<i64> = got
            .iter()
            .filter_map(|e| match e {
                Event::Message {
                    payload: Payload::Json(v),
                    ..
                } => v.as_i64(),
                _ => None,
            })
            .collect();
        // Every reliable message arrives, in order, and the departure after
        // the last of them.
        let reliable: Vec<i64> = values.iter().copied().filter(|v| v % 3 == 0).collect();
        assert_eq!(
            reliable,
            (0..300).filter(|v| v % 3 == 0).collect::<Vec<_>>()
        );
        // About half the unreliable ones are lost; the rest arrive once.
        let mut lossy: Vec<i64> = values.iter().copied().filter(|v| v % 3 != 0).collect();
        let n = lossy.len();
        assert!(
            (50..150).contains(&n),
            "{n} of 200 unreliable messages arrived"
        );
        lossy.sort();
        lossy.dedup();
        assert_eq!(lossy.len(), n, "none twice");
    }
}
