//! A native client's connection (the `client` feature): one WebSocket to a
//! world server on a thread of its own, binary messages in and out through
//! queues, so the engine's thread never blocks on the network. It dials,
//! carries bytes and says when it opened or closed; the protocol and
//! reconnecting are the engine's (`og.shared`, which asks its provider for
//! a new connection each time). `ws://` and `wss://` (rustls, trusting the
//! roots webpki-roots carries, so nothing of the system it runs on), and
//! WebTransport at `https://host:port/` ([`Client::connect_wt`]): proposals
//! as datagrams, the rest on one stream (crate::wt), on a small runtime of
//! the client's own thread.

use rustls::{ClientConfig, RootCertStore};
use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use tungstenite::client::IntoClientRequest;
use tungstenite::protocol::WebSocketConfig;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Connector, Message};

type Socket = tungstenite::WebSocket<MaybeTlsStream<TcpStream>>;

/// How long reaching the server and its handshake may each take.
const DIAL: Duration = Duration::from_secs(5);

/// An open connection that hears nothing this long is closed (`Closed`):
/// a world server sends a snapshot every tick, so silence means the path
/// is gone. Over WebTransport a client whose NAT rebinds is never told
/// (the server's connection ids are zero-length, so it drops the packets
/// from the new address), and the engine reconnects and resumes well
/// inside the server's 10 s linger instead of after QUIC's idle timeout.
/// It is measured here, on the connection's own thread in real time, so
/// the engine's frames stalling (a load hitch) never trips it.
pub const SILENT: Duration = Duration::from_secs(2);

/// What the connection says, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientEvent {
    /// Connected: messages may go.
    Open,
    /// One binary message from the server.
    Message(Vec<u8>),
    /// The connection is gone (or never came), and why. The last event.
    Closed(String),
}

/// A connection to a world server. Dropping it closes the connection.
pub struct Client {
    /// Messages that arrived as datagrams (WebTransport); 0 on a WebSocket.
    datagrams: Arc<std::sync::atomic::AtomicU64>,
    out: Sender<Vec<u8>>,
    events: Receiver<ClientEvent>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Client {
    /// Starts connecting to `url` (`ws://host:port/` or `wss://host/`); its
    /// first event says whether it opened. Messages sent before are queued.
    pub fn connect(url: &str) -> Client {
        let (out, out_rx) = mpsc::channel::<Vec<u8>>();
        let (events_tx, events) = mpsc::channel::<ClientEvent>();
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let url = url.to_string();
        let thread = thread::Builder::new()
            .name("og-shared-client".into())
            .spawn(move || {
                let why = match dial(&url) {
                    Ok(mut socket) => {
                        let _ = events_tx.send(ClientEvent::Open);
                        serve(&mut socket, &out_rx, &events_tx, &stopping)
                    }
                    Err(e) => format!("cannot reach the world server at {url} ({e})"),
                };
                let _ = events_tx.send(ClientEvent::Closed(why));
            })
            .ok();
        Client {
            datagrams: Arc::default(),
            out,
            events,
            stop,
            thread,
        }
    }

    /// Starts a WebTransport session to `url` (`https://host:port/`),
    /// checking the server's certificate against `cert_hashes` (SHA-256)
    /// when any are given, else the system's roots. Events as [`connect`](Client::connect)'s.
    pub fn connect_wt(url: &str, cert_hashes: Vec<[u8; 32]>) -> Client {
        let (out, out_rx) = mpsc::channel::<Vec<u8>>();
        let (events_tx, events) = mpsc::channel::<ClientEvent>();
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let url = url.to_string();
        let datagrams = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let counted = datagrams.clone();
        let thread = thread::Builder::new()
            .name("og-shared-client-wt".into())
            .spawn(move || {
                let why = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt.block_on(serve_wt(
                        &url,
                        &cert_hashes,
                        out_rx,
                        &events_tx,
                        &stopping,
                        &counted,
                    )),
                    Err(e) => e.to_string(),
                };
                let _ = events_tx.send(ClientEvent::Closed(why));
            })
            .ok();
        Client {
            datagrams,
            out,
            events,
            stop,
            thread,
        }
    }

    /// How many messages arrived as datagrams.
    pub fn datagrams(&self) -> u64 {
        self.datagrams.load(Ordering::Relaxed)
    }

    /// Queue one binary message.
    pub fn send(&self, bytes: Vec<u8>) {
        let _ = self.out.send(bytes);
    }

    /// What happened since the last call, in order.
    pub fn poll(&self) -> Vec<ClientEvent> {
        self.events.try_iter().collect()
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn tls() -> Result<Arc<ClientConfig>, String> {
    let roots = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    Ok(Arc::new(
        ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(|e| e.to_string())?
            .with_root_certificates(roots)
            .with_no_client_auth(),
    ))
}

fn dial(url: &str) -> Result<Socket, String> {
    let secure = url.starts_with("wss://");
    if !secure && !url.starts_with("ws://") {
        return Err(format!(
            "'{url}' is not a world server URL: write ws://127.0.0.1:47910/ or wss://<host>/"
        ));
    }
    let request = url.into_client_request().map_err(|e| e.to_string())?;
    let host = request.uri().host().unwrap_or_default().to_string();
    let port = request
        .uri()
        .port_u16()
        .unwrap_or(if secure { 443 } else { 80 });
    let tls = if secure { Some(tls()?) } else { None };
    let mut last = format!("'{host}' has no address");
    for addr in (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
    {
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
        let config = WebSocketConfig::default()
            .max_frame_size(Some(crate::protocol::MAX_MESSAGE * 4))
            .max_message_size(Some(crate::protocol::MAX_MESSAGE * 4));
        match tungstenite::client_tls_with_config(
            request.clone(),
            stream,
            Some(config),
            Some(tls.clone().map_or(Connector::Plain, Connector::Rustls)),
        ) {
            Ok((socket, _)) => {
                // Reads wait a millisecond, so queued sends go out promptly.
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

/// A WebTransport session: dials, then carries messages until the client
/// stops or the session is lost. Returns why it ended.
async fn serve_wt(
    url: &str,
    cert_hashes: &[[u8; 32]],
    out: Receiver<Vec<u8>>,
    events: &Sender<ClientEvent>,
    stop: &AtomicBool,
    counted: &Arc<std::sync::atomic::AtomicU64>,
) -> String {
    use crate::protocol::client_datagram;
    use crate::wt::{dial, frame, read_frame};
    let session = match tokio::time::timeout(DIAL, dial(url, cert_hashes)).await {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => return format!("cannot reach the world server at {url} ({e})"),
        Err(_) => return format!("cannot reach the world server at {url} (no answer in 5 s)"),
    };
    let (mut send, mut recv) = match session.open_bi().await {
        Ok(s) => s,
        Err(e) => return format!("the session opened no stream: {e}"),
    };
    let _ = events.send(ClientEvent::Open);
    // The stream's reader and the datagrams' feed one queue the loop drains.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<Vec<u8>, String>>();
    let frames = tx.clone();
    let reader = tokio::spawn(async move {
        loop {
            let f = read_frame(&mut recv).await;
            let end = f.is_err();
            if frames.send(f).is_err() || end {
                return;
            }
        }
    });
    let grams = session.clone();
    let counted = counted.clone();
    let datagrams = tokio::spawn(async move {
        while let Ok(d) = grams.read_datagram().await {
            counted.fetch_add(1, Ordering::Relaxed);
            if tx.send(Ok(d.to_vec())).is_err() {
                return;
            }
        }
    });
    let mut heard = std::time::Instant::now();
    let why = loop {
        if heard.elapsed() > SILENT {
            break format!("the server was silent for {} s", SILENT.as_secs());
        }
        if stop.load(Ordering::Relaxed) {
            while let Ok(m) = out.try_recv() {
                let _ = send.write_all(&frame(&m)).await;
            }
            let _ = send.finish();
            break "closed".to_string();
        }
        let mut failed = false;
        while let Ok(m) = out.try_recv() {
            if client_datagram(&m) && m.len() <= session.max_datagram_size() {
                // A full buffer drops it, as the network may: the next one carries the same.
                let _ = session.send_datagram(m.into());
            } else if send.write_all(&frame(&m)).await.is_err() {
                failed = true;
                break;
            }
        }
        if failed {
            break "the connection failed".to_string();
        }
        // Wait a millisecond for what the server sends, so queued sends go out promptly.
        match tokio::time::timeout(Duration::from_millis(1), rx.recv()).await {
            Ok(Some(Ok(m))) => {
                heard = std::time::Instant::now();
                let _ = events.send(ClientEvent::Message(m));
                while let Ok(Ok(m)) = rx.try_recv() {
                    let _ = events.send(ClientEvent::Message(m));
                }
            }
            Ok(Some(Err(e))) => break format!("the server closed the connection: {e}"),
            Ok(None) => break "the server closed the connection".to_string(),
            Err(_) => {}
        }
    };
    reader.abort();
    datagrams.abort();
    session.close(0, b"");
    why
}

/// Carries messages until the client stops or the connection is lost.
fn serve(
    socket: &mut Socket,
    out: &Receiver<Vec<u8>>,
    events: &Sender<ClientEvent>,
    stop: &AtomicBool,
) -> String {
    let failed = |r: tungstenite::Result<()>| {
        r.is_err_and(
            |e| !matches!(&e, tungstenite::Error::Io(e) if e.kind() == io::ErrorKind::Interrupted),
        )
    };
    let mut heard = std::time::Instant::now();
    loop {
        if stop.load(Ordering::Relaxed) {
            while let Ok(m) = out.try_recv() {
                let _ = socket.write(Message::binary(m));
            }
            let _ = socket.close(None);
            let _ = socket.flush();
            return "closed".into();
        }
        if heard.elapsed() > SILENT {
            return format!("the server was silent for {} s", SILENT.as_secs());
        }
        let mut wrote = false;
        while let Ok(m) = out.try_recv() {
            if failed(socket.write(Message::binary(m))) {
                return "the connection failed".into();
            }
            wrote = true;
        }
        if wrote && failed(socket.flush()) {
            return "the connection failed".into();
        }
        match socket.read() {
            Ok(Message::Binary(b)) => {
                heard = std::time::Instant::now();
                let _ = events.send(ClientEvent::Message(b.to_vec()));
            }
            Ok(Message::Close(c)) => {
                return c.map_or_else(
                    || "the server closed the connection".into(),
                    |c| format!("the server closed the connection: {}", c.reason),
                );
            }
            // Pings are answered by the next flush.
            Ok(_) => {
                let _ = socket.flush();
            }
            Err(tungstenite::Error::Io(e))
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::Interrupted
                ) => {}
            Err(e) => return format!("the connection failed: {e}"),
        }
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use crate::auth::{Rules, Verifier, sign_hs256};
    use crate::config::Config;
    use crate::protocol::{Authority, ClientMsg, Hello, PROTOCOL, ServerMsg};
    use crate::server::{KeySource, Server};

    #[test]
    fn a_client_says_hello_and_hears_its_welcome() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let server = rt.block_on(async {
            Server::start(
                "127.0.0.1:0".parse().unwrap(),
                Config::default(),
                Verifier::hs256(b"s", Rules::audience("g")),
                KeySource::None,
                false,
                None,
            )
            .await
            .unwrap()
        });
        let c = Client::connect(&format!("ws://{}/", server.addr));
        let exp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 60;
        let token = sign_hs256(
            b"s",
            &serde_json::json!({ "aud": "g", "sub": "p1", "exp": exp }),
        );
        c.send(
            ClientMsg::Hello(Hello {
                version: PROTOCOL,
                token,
                world: "main".into(),
                shape: 0,
                fields_len: 0,
                authority: Authority::Validate,
                send_hz: 20,
                kind: 0,
                resume: None,
            })
            .to_bytes(),
        );
        let mut got = Vec::new();
        for _ in 0..3000 {
            got.extend(c.poll());
            if got.iter().any(|e| matches!(e, ClientEvent::Message(_))) {
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(got.first(), Some(&ClientEvent::Open), "{got:?}");
        let Some(ClientEvent::Message(m)) = got.get(1) else {
            panic!("no message: {got:?}");
        };
        let Ok(ServerMsg::Welcome(w)) = ServerMsg::decode(m) else {
            panic!("not a welcome");
        };
        assert_eq!(w.player, "p1");
        drop(c);
        // A server that is gone: the client says it could not connect.
        drop(server);
        drop(rt);
        let c = Client::connect("ws://127.0.0.1:1/");
        let mut got = Vec::new();
        for _ in 0..6000 {
            got.extend(c.poll());
            if !got.is_empty() {
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
        assert!(
            matches!(got.first(), Some(ClientEvent::Closed(_))),
            "{got:?}"
        );
    }

    /// Two clients over WebTransport (a self-signed certificate taken by
    /// its hash, as a browser's serverCertificateHashes): each is welcomed,
    /// proposes as datagrams, and decodes the other's state from snapshots
    /// that came as datagrams.
    #[test]
    fn two_webtransport_clients_see_each_other_through_datagrams() {
        use crate::protocol::{AvatarState, client_datagram, server_datagram};
        use crate::view::{Proposer, View};
        let rt = tokio::runtime::Runtime::new().unwrap();
        let cert = crate::cert::self_signed(&["localhost".into(), "127.0.0.1".into()], 10).unwrap();
        let (server, wt) = rt.block_on(async {
            let server = Server::start(
                "127.0.0.1:0".parse().unwrap(),
                Config::default(),
                Verifier::hs256(b"s", Rules::audience("g")),
                KeySource::None,
                false,
                None,
            )
            .await
            .unwrap();
            let wt = server
                .serve_webtransport(
                    "127.0.0.1:0".parse().unwrap(),
                    &cert.cert_pem,
                    &cert.key_pem,
                )
                .unwrap();
            (server, wt)
        });
        let exp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 60;
        let url = format!("https://{wt}/");
        let clients: Vec<Client> = ["p1", "p2"]
            .iter()
            .map(|p| {
                let c = Client::connect_wt(&url, vec![cert.sha256]);
                let token = sign_hs256(
                    b"s",
                    &serde_json::json!({ "aud": "g", "sub": p, "exp": exp }),
                );
                c.send(
                    ClientMsg::Hello(Hello {
                        version: PROTOCOL,
                        token,
                        world: "main".into(),
                        shape: 0,
                        fields_len: 0,
                        authority: Authority::Validate,
                        send_hz: 20,
                        kind: 0,
                        resume: None,
                    })
                    .to_bytes(),
                );
                c
            })
            .collect();
        let mut views = [View::new(), View::new()];
        let mut props = [Proposer::default(), Proposer::default()];
        let (mut welcomed, mut datagram_snapshots) = ([false; 2], [0u32; 2]);
        for t in 0..400u32 {
            for (i, c) in clients.iter().enumerate() {
                for e in c.poll() {
                    match e {
                        ClientEvent::Message(m) => match ServerMsg::decode(&m) {
                            Ok(ServerMsg::Welcome(_)) => welcomed[i] = true,
                            Ok(ServerMsg::Enter(e)) => views[i].enter(&e),
                            Ok(ServerMsg::Snapshot(b)) => {
                                assert!(server_datagram(&m));
                                if views[i].snapshot(&b).unwrap().is_some() {
                                    datagram_snapshots[i] += 1;
                                }
                            }
                            _ => {}
                        },
                        ClientEvent::Closed(why) => panic!("client {i} closed: {why}"),
                        ClientEvent::Open => {}
                    }
                }
                if welcomed[i] && t % 3 == 0 {
                    let x = 10.0 * i as f64 + f64::from(t) * 0.01;
                    let s = AvatarState::from_world(128.0, [x, 1.0, 0.0], [0.2, 0.0, 0.0], 0.0);
                    let b = props[i].propose(s, false, views[i].ack(), views[i].ack_seq);
                    assert!(client_datagram(&b));
                    c.send(b);
                }
            }
            if views.iter().all(|v| v.remotes.len() == 1)
                && datagram_snapshots.iter().all(|n| *n > 10)
            {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(welcomed, [true, true]);
        for (i, v) in views.iter().enumerate() {
            let other = v
                .remotes
                .values()
                .next()
                .unwrap_or_else(|| panic!("client {i} sees nobody"));
            assert_eq!(other.player, if i == 0 { "p2" } else { "p1" });
            // The other's newest state came by datagram, near where it was proposed.
            let x = other.state.position(128.0)[0];
            assert!(
                (x - 10.0 * (1 - i) as f64).abs() < 5.0,
                "client {i} has the other at x {x}"
            );
        }
        assert!(
            datagram_snapshots.iter().all(|n| *n > 10),
            "{datagram_snapshots:?}"
        );
        // The snapshots crossed as datagrams, not on the stream.
        for c in &clients {
            assert!(c.datagrams() > 10, "{} datagrams", c.datagrams());
        }
        drop(clients);
        drop(server);
        drop(rt);
    }
}
