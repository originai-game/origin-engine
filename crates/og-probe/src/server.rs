//! The native transport: a WebSocket on `127.0.0.1:<port>` at `/probe`.
//!
//! Socket I/O runs on its own threads and never touches the world. Each
//! well-formed call is queued; the world's thread takes the queue between
//! ticks ([`Probe::serve`]) and the answer goes back to the connection that
//! asked. Malformed frames are answered by the transport itself.

use crate::Probe;
use crate::rpc::{self, Call};
use og_ecs::World;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use tungstenite::Message;
use tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tungstenite::http::StatusCode;

/// The port `og` serves the probe on unless told otherwise.
pub const DEFAULT_PORT: u16 = 47800;
/// The only path that upgrades.
pub const PATH: &str = "/probe";

/// One call waiting for the world's answer.
pub struct Pending {
    pub call: Call,
    reply: Sender<String>,
}

impl Pending {
    /// Send the answer frame. A connection that closed meanwhile is owed nothing.
    pub fn answer(self, frame: String) {
        let _ = self.reply.send(frame);
    }
}

/// The loopback probe server. Dropping it stops accepting and closes
/// connections within a read timeout.
pub struct ProbeServer {
    port: u16,
    calls: Receiver<Pending>,
    stop: Arc<AtomicBool>,
}

impl ProbeServer {
    /// Serve on `127.0.0.1:<port>`; `Ok(None)` for port 0, which disables
    /// the server (the `--probe-port=0` option).
    pub fn start(port: u16) -> io::Result<Option<ProbeServer>> {
        if port == 0 {
            return Ok(None);
        }
        ProbeServer::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, port))).map(Some)
    }

    /// Serve on a loopback address (port 0 here picks a free port, for
    /// tests). Any other address is refused: a runner reaches the probe
    /// locally and forwards results over its own socket.
    pub fn bind(addr: SocketAddr) -> io::Result<ProbeServer> {
        if !addr.ip().is_loopback() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "the probe binds loopback only",
            ));
        }
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let port = listener.local_addr()?.port();
        let (sender, calls) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        thread::Builder::new()
            .name("og-probe-accept".into())
            .spawn(move || accept(listener, sender, stopping))?;
        Ok(ProbeServer { port, calls, stop })
    }

    /// The bound port.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The next queued call, if any. Never blocks.
    pub fn poll(&self) -> Option<Pending> {
        self.calls.try_recv().ok()
    }
}

impl Drop for ProbeServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn accept(listener: TcpListener, calls: Sender<Pending>, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => {
                let calls = calls.clone();
                let stop = stop.clone();
                let _ = thread::Builder::new()
                    .name("og-probe-conn".into())
                    .spawn(move || connection(stream, calls, stop));
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5))
            }
            Err(_) => thread::sleep(Duration::from_millis(50)),
        }
    }
}

/// The handshake callback: only `/probe` upgrades, and only for a client
/// that sends no `Origin` header. Every browser sends one on a WebSocket
/// handshake, so a page the developer happens to visit cannot drive the
/// probe; `og probe`, the runner and other native clients send none. The
/// `Err` type is tungstenite's own handshake contract.
#[allow(clippy::result_large_err)]
fn only_probe_path(request: &Request, response: Response) -> Result<Response, ErrorResponse> {
    if request.uri().path() != PATH {
        let mut refusal = ErrorResponse::new(Some(format!("the probe lives at {PATH}")));
        *refusal.status_mut() = StatusCode::NOT_FOUND;
        return Err(refusal);
    }
    if request.headers().contains_key("origin") {
        let mut refusal = ErrorResponse::new(Some(
            "the probe refuses browser pages (an Origin header): connect from a native client such as `og probe`"
                .to_string(),
        ));
        *refusal.status_mut() = StatusCode::FORBIDDEN;
        return Err(refusal);
    }
    Ok(response)
}

fn connection(stream: TcpStream, calls: Sender<Pending>, stop: Arc<AtomicBool>) {
    if stream.set_nonblocking(false).is_err() || stream.set_nodelay(true).is_err() {
        return;
    }
    let Ok(mut socket) = tungstenite::accept_hdr(stream, only_probe_path) else {
        return;
    };
    // A read timeout lets the thread notice the server stopping.
    let _ = socket
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(200)));
    while !stop.load(Ordering::Relaxed) {
        let text = match socket.read() {
            Ok(Message::Text(text)) => text.to_string(),
            Ok(Message::Binary(_)) => {
                let refusal = rpc::error_frame(
                    &serde_json::Value::Null,
                    rpc::INVALID_REQUEST,
                    "text frames only",
                );
                if socket.send(Message::text(refusal)).is_err() {
                    return;
                }
                continue;
            }
            Ok(Message::Close(_)) => return,
            Ok(_) => continue,
            Err(tungstenite::Error::Io(e))
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                continue;
            }
            Err(_) => return,
        };
        let answer = match rpc::parse(&text) {
            Err(frame) => frame,
            Ok(call) => {
                let (reply, wait) = mpsc::channel();
                if calls.send(Pending { call, reply }).is_err() {
                    return;
                }
                match wait.recv() {
                    Ok(frame) => frame,
                    Err(_) => return,
                }
            }
        };
        if socket.send(Message::text(answer)).is_err() {
            return;
        }
    }
}

impl Probe {
    /// Answer every queued call, on the world's thread, between ticks.
    /// Returns how many were answered.
    pub fn serve(&mut self, world: &mut World, server: &ProbeServer) -> usize {
        let mut answered = 0;
        while let Some(pending) = server.poll() {
            let frame = self.answer(world, &pending.call);
            pending.answer(frame);
            answered += 1;
        }
        answered
    }
}
