//! A native client's connection (the `client` feature): one WebSocket to a
//! world server on a thread of its own, binary messages in and out through
//! queues, so the engine's thread never blocks on the network. It dials,
//! carries bytes and says when it opened or closed; the protocol and
//! reconnecting are the engine's (`og.shared`, which asks its provider for
//! a new connection each time). `ws://` and `wss://` (rustls, trusting the
//! roots webpki-roots carries, so nothing of the system it runs on).

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
            out,
            events,
            stop,
            thread,
        }
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
    loop {
        if stop.load(Ordering::Relaxed) {
            while let Ok(m) = out.try_recv() {
                let _ = socket.write(Message::binary(m));
            }
            let _ = socket.close(None);
            let _ = socket.flush();
            return "closed".into();
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
}
