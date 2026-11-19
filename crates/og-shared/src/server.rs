//! `og world-server`: a [`Shard`] behind WebSocket connections and, with
//! [`Server::serve_webtransport`], WebTransport sessions (docs/design.md
//! §2.12). Standalone: it knows no platform, only its verifier.
//!
//! - WebTransport: the client opens one bidirectional stream, on which
//!   every message but two crosses as a frame (its length, u32 LE, then
//!   its bytes), in order. `propose` (client) and `snapshot` (server) cross
//!   as datagrams, which may be lost or reordered: the protocol's
//!   acknowledgements already make that safe, and each snapshot is held to
//!   the session's datagram size ([`Shard::cap_snapshots`]).
//!
//! - One thread owns the shard (never a tokio worker). Connections hand it what they read, in order,
//!   and it ticks `tickHz` a second; after each tick and each join it gives
//!   every connection its messages, each one binary frame.
//! - A connection verifies its `hello`'s token before the shard sees it.
//!   `GET /healthz` on the same port answers `ready` (200) or `draining`
//!   (503) as plain HTTP.
//! - A connection silent for `SILENT_MS` is dropped, and one that does not
//!   keep up (`OUT_QUEUE` messages behind) is closed: its avatar lingers.
//! - [`Server::drain`] (SIGTERM in `og world-server`) refuses new joins,
//!   tells every client `moving`, and the server stops once nobody is left.

use crate::auth::{AuthError, Verifier, parse_jwks};
use crate::config::Config;
use crate::protocol::{ClientMsg, Hello, MAX_MESSAGE, Refusal, ServerMsg};
use crate::shard::{Claims, Shard, Stats};
use crate::wt::{frame, read_frame};
use futures_util::{SinkExt, StreamExt};
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

/// A connection silent this long is dropped.
pub const SILENT_MS: u64 = 10_000;
/// A connection idle this long is pinged.
pub const PING_MS: u64 = 3_000;
/// How often the shard thread looks for what the connections handed it, ms.
pub const POLL_MS: u64 = 2;
/// How long before a tick the shard warms what it reads (Shard::warm), ms.
pub const WARM_MS: u64 = 1;
/// A connection's read buffer, bytes (see `connection`).
pub const READ_BUFFER: usize = 4096;
/// How far behind a connection may fall before it is closed, in messages.
pub const OUT_QUEUE: usize = 512;
/// The largest snapshot a WebTransport datagram carries, bytes: QUIC's
/// payload at the 1280-byte minimum IPv6 path, so a datagram is never
/// refused for its size on any path (quinn's own figure can be smaller).
pub const MAX_DATAGRAM: usize = 1150;

/// Where the verifier's keys come from, so an unknown `kid` can refetch them.
pub enum KeySource {
    None,
    Url(String),
    File(String),
}

/// Read a JWKS from a URL or a file.
pub fn load_jwks(source: &KeySource) -> Result<String, String> {
    match source {
        KeySource::None => Err("no JWKS source".into()),
        KeySource::File(f) => {
            std::fs::read_to_string(f).map_err(|e| format!("cannot read {f}: {e}"))
        }
        KeySource::Url(u) => ureq::get(u)
            .call()
            .map_err(|e| format!("cannot fetch {u}: {e}"))?
            .body_mut()
            .read_to_string()
            .map_err(|e| format!("cannot read {u}: {e}")),
    }
}

/// What the server measures, read by `--metrics` and the load test.
#[derive(Default)]
pub struct Metrics {
    pub bytes_in: AtomicU64,
    pub bytes_out: AtomicU64,
    pub frames_out: AtomicU64,
    pub slow_closed: AtomicU64,
    /// The last 1200 tick times, microseconds.
    pub ticks: Mutex<VecDeque<u32>>,
    pub snapshot: Mutex<(usize, usize, Stats, bool)>,
    /// Every tick's time (its step and the hand-over to the connections),
    /// and the step's alone, counted in [`BUCKETS_MS`]: a scraper takes the
    /// difference of two scrapes for the ticks between them.
    pub tick_hist: [AtomicU64; BUCKETS_MS.len() + 1],
    pub step_hist: [AtomicU64; BUCKETS_MS.len() + 1],
}

/// The tick histograms' upper bounds, milliseconds (the last bucket is +Inf).
/// Whole milliseconds up to 20, so a 10 ms budget is read to the millisecond.
pub const BUCKETS_MS: [f64; 26] = [
    0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    17.0, 18.0, 19.0, 20.0, 25.0, 30.0, 50.0, 100.0, 250.0,
];

fn observe(hist: &[AtomicU64], us: u32) {
    let ms = f64::from(us) / 1e3;
    let i = BUCKETS_MS
        .iter()
        .position(|b| ms <= *b)
        .unwrap_or(BUCKETS_MS.len());
    hist[i].fetch_add(1, Ordering::Relaxed);
}

impl Metrics {
    /// The tick time at quantile `q` (0 to 1), microseconds, over the last minute.
    pub fn tick_quantile(&self, q: f64) -> u32 {
        let mut v: Vec<u32> = self.ticks.lock().unwrap().iter().copied().collect();
        if v.is_empty() {
            return 0;
        }
        v.sort_unstable();
        v[((v.len() - 1) as f64 * q).round() as usize]
    }

    /// The Prometheus text.
    pub fn text(&self) -> String {
        let (players, connected, s, draining) = *self.snapshot.lock().unwrap();
        let mut t = String::new();
        let mut g = |name: &str, kind: &str, v: f64| {
            t += &format!("# TYPE og_shared_{name} {kind}\nog_shared_{name} {v}\n");
        };
        g("players", "gauge", players as f64);
        g("connected", "gauge", connected as f64);
        g("draining", "gauge", f64::from(u8::from(draining)));
        g(
            "tick_p50_seconds",
            "gauge",
            f64::from(self.tick_quantile(0.5)) / 1e6,
        );
        g(
            "tick_p99_seconds",
            "gauge",
            f64::from(self.tick_quantile(0.99)) / 1e6,
        );
        g(
            "tick_max_seconds",
            "gauge",
            f64::from(self.tick_quantile(1.0)) / 1e6,
        );
        g(
            "bytes_in_total",
            "counter",
            self.bytes_in.load(Ordering::Relaxed) as f64,
        );
        g(
            "bytes_out_total",
            "counter",
            self.bytes_out.load(Ordering::Relaxed) as f64,
        );
        g(
            "frames_out_total",
            "counter",
            self.frames_out.load(Ordering::Relaxed) as f64,
        );
        g(
            "slow_closed_total",
            "counter",
            self.slow_closed.load(Ordering::Relaxed) as f64,
        );
        let mut h = String::new();
        for (name, hist) in [
            ("tick_seconds", &self.tick_hist),
            ("step_seconds", &self.step_hist),
        ] {
            h += &format!("# TYPE og_shared_{name} histogram\n");
            let mut cum = 0;
            for (i, c) in hist.iter().enumerate() {
                cum += c.load(Ordering::Relaxed);
                let le = BUCKETS_MS
                    .get(i)
                    .map_or("+Inf".to_string(), |b| format!("{}", b / 1e3));
                h += &format!("og_shared_{name}_bucket{{le=\"{le}\"}} {cum}\n");
            }
            h += &format!("og_shared_{name}_count {cum}\n");
        }
        for (n, v) in [
            ("joins_total", s.joins),
            ("refusals_total", s.refusals),
            ("proposals_total", s.proposals),
            ("rejected_total", s.rejected),
            ("corrections_total", s.corrections),
            ("events_total", s.events),
            ("enters_total", s.enters),
            ("leaves_total", s.leaves),
            ("updates_total", s.updates),
        ] {
            g(n, "counter", v as f64);
        }
        t + &h
    }
}

enum Inbound {
    Join {
        hello: Hello,
        claims: Claims,
        /// The most bytes one snapshot may take (a datagram's), else usize::MAX.
        cap: usize,
        out: mpsc::Sender<Vec<u8>>,
        reply: oneshot::Sender<Result<u64, (Refusal, String)>>,
    },
    Msg {
        session: u64,
        bytes: Vec<u8>,
    },
    Closed {
        session: u64,
    },
    /// The dispatcher found this connection's queue full.
    Slow {
        session: u64,
        out: mpsc::Sender<Vec<u8>>,
    },
    Drain(String),
}

/// What the shard thread hands the dispatcher: each session's sender and messages.
type Batch = Vec<(u64, mpsc::Sender<Vec<u8>>, Vec<Vec<u8>>)>;

/// Hands the shard's messages to the connections. A send wakes the
/// connection's task; done here, on the runtime, the wakes are tokio's own
/// and stay off the shard's tick (on its thread they were a futex call
/// each, 4000 a tick at 2000 clients, a third of the tick's time). One
/// queue keeps each session's messages in order.
async fn dispatch(
    mut rx: mpsc::UnboundedReceiver<Batch>,
    inbound: std::sync::mpsc::Sender<Inbound>,
    metrics: Arc<Metrics>,
) {
    while let Some(batch) = rx.recv().await {
        for (session, out, msgs) in batch {
            for m in msgs {
                if out.try_send(m).is_err() {
                    if !out.is_closed() {
                        // Too far behind: the shard closes it; the avatar lingers.
                        metrics.slow_closed.fetch_add(1, Ordering::Relaxed);
                        let _ = inbound.send(Inbound::Slow { session, out });
                    }
                    break;
                }
            }
        }
    }
}

/// A running server.
pub struct Server {
    pub addr: SocketAddr,
    pub metrics: Arc<Metrics>,
    shared: Arc<Shared>,
    draining: Arc<AtomicBool>,
    inbound: std::sync::mpsc::Sender<Inbound>,
    done: tokio::sync::watch::Receiver<bool>,
}

struct Shared {
    verifier: RwLock<Verifier>,
    keys: KeySource,
    last_refetch: Mutex<Option<Instant>>,
    draining: Arc<AtomicBool>,
    metrics: Arc<Metrics>,
    inbound: std::sync::mpsc::Sender<Inbound>,
    log: bool,
}

fn now_s() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

impl Server {
    /// Listen on `addr` and serve; must run inside a tokio runtime.
    pub async fn start(
        addr: SocketAddr,
        config: Config,
        verifier: Verifier,
        keys: KeySource,
        log: bool,
    ) -> std::io::Result<Server> {
        let listener = TcpListener::bind(addr).await?;
        let addr = listener.local_addr()?;
        let metrics = Arc::new(Metrics::default());
        let draining = Arc::new(AtomicBool::new(false));
        let (inbound, rx) = std::sync::mpsc::channel();
        let (done_tx, done) = tokio::sync::watch::channel(false);
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64)
            ^ u64::from(std::process::id());
        let shard = Shard::new(config, seed);
        let (batches, batch_rx) = mpsc::unbounded_channel();
        tokio::spawn(dispatch(batch_rx, inbound.clone(), metrics.clone()));
        let m = metrics.clone();
        std::thread::Builder::new()
            .name("og-shard".into())
            .spawn(move || run_shard(shard, rx, batches, m, done_tx, log))?;
        let shared = Arc::new(Shared {
            verifier: RwLock::new(verifier),
            keys,
            last_refetch: Mutex::new(None),
            draining: draining.clone(),
            metrics: metrics.clone(),
            inbound: inbound.clone(),
            log,
        });
        let accepting = shared.clone();
        tokio::spawn(async move {
            let shared = accepting;
            loop {
                let Ok((stream, peer)) = listener.accept().await else {
                    continue;
                };
                let _ = stream.set_nodelay(true);
                let shared = shared.clone();
                tokio::spawn(async move {
                    if let Err(e) = connection(stream, shared.clone()).await
                        && shared.log
                    {
                        eprintln!("world-server: {peer}: {e}");
                    }
                });
            }
        });
        Ok(Server {
            addr,
            metrics,
            shared,
            draining,
            inbound,
            done,
        })
    }

    /// Also serve WebTransport on UDP `addr` with this certificate chain
    /// and key (PEM). Returns the address it listens on.
    pub fn serve_webtransport(
        &self,
        addr: SocketAddr,
        chain_pem: &str,
        key_pem: &str,
    ) -> Result<SocketAddr, String> {
        use web_transport_quinn::quinn::rustls::pki_types::pem::PemObject;
        use web_transport_quinn::quinn::rustls::pki_types::{CertificateDer, PrivateKeyDer};
        let chain: Vec<CertificateDer<'static>> =
            CertificateDer::pem_slice_iter(chain_pem.as_bytes())
                .collect::<Result<_, _>>()
                .map_err(|e| format!("the certificate: {e}"))?;
        if chain.is_empty() {
            return Err("the certificate file holds no certificate".into());
        }
        let key = PrivateKeyDer::from_pem_slice(key_pem.as_bytes())
            .map_err(|e| format!("the key: {e}"))?;
        let mut server = web_transport_quinn::Server::new(wt_endpoint(addr, chain, key)?);
        let local = server.local_addr().map_err(|e| e.to_string())?;
        let shared = self.shared.clone();
        tokio::spawn(async move {
            while let Some(request) = server.accept().await {
                let shared = shared.clone();
                let peer = request.conn().remote_address();
                tokio::spawn(async move {
                    if let Err(e) = wt_connection(request, shared.clone()).await
                        && shared.log
                    {
                        eprintln!("world-server: {peer} (wt): {e}");
                    }
                });
            }
        });
        Ok(local)
    }

    /// Stop taking joins and move every client away.
    pub fn drain(&self, reason: &str) {
        self.draining.store(true, Ordering::SeqCst);
        let _ = self.inbound.send(Inbound::Drain(reason.into()));
    }

    /// Wait until a drain left nobody connected, or `deadline` passed.
    pub async fn drained(&mut self, deadline: Duration) {
        let _ = tokio::time::timeout(deadline, self.done.wait_for(|d| *d)).await;
    }
}

/// The shard's own thread: it ticks on a deadline and takes what the
/// connections hand it in between. It is never a tokio worker, so a tick
/// never waits behind socket work, and socket work never behind a tick.
fn run_shard(
    mut shard: Shard,
    rx: std::sync::mpsc::Receiver<Inbound>,
    batches: mpsc::UnboundedSender<Batch>,
    metrics: Arc<Metrics>,
    done: tokio::sync::watch::Sender<bool>,
    log: bool,
) {
    let mut outs: HashMap<u64, mpsc::Sender<Vec<u8>>> = HashMap::new();
    let period = Duration::from_secs_f64(1.0 / f64::from(shard.config.tick_hz));
    let mut next = Instant::now() + period;
    let warm_ahead = Duration::from_millis(WARM_MS);
    let mut warmed = false;
    let flush =
        |shard: &mut Shard, outs: &mut HashMap<u64, mpsc::Sender<Vec<u8>>>, sessions: &[u64]| {
            for s in shard.take_closed() {
                outs.remove(&s);
            }
            let batch: Batch = sessions
                .iter()
                .filter_map(|&s| {
                    let msgs = shard.take_out(s);
                    let out = outs.get(&s)?;
                    (!msgs.is_empty()).then(|| (s, out.clone(), msgs))
                })
                .collect();
            if !batch.is_empty() {
                let _ = batches.send(batch);
            }
        };
    loop {
        let now = Instant::now();
        if now >= next {
            let t0 = Instant::now();
            shard.step();
            let step_us = t0.elapsed().as_micros().min(u128::from(u32::MAX)) as u32;
            let sessions: Vec<u64> = shard.sessions().collect();
            flush(&mut shard, &mut outs, &sessions);
            let us = t0.elapsed().as_micros().min(u128::from(u32::MAX)) as u32;
            observe(&metrics.tick_hist, us);
            observe(&metrics.step_hist, step_us);
            {
                let mut ticks = metrics.ticks.lock().unwrap();
                ticks.push_back(us);
                while ticks.len() > 1200 {
                    ticks.pop_front();
                }
            }
            *metrics.snapshot.lock().unwrap() = (
                shard.players(),
                shard.connected(),
                shard.stats,
                shard.draining(),
            );
            if shard.draining() && shard.connected() == 0 {
                let _ = done.send(true);
            }
            warmed = false;
            // A late tick moves the next one, never runs two at once.
            next += period;
            if next <= Instant::now() {
                next = Instant::now() + period;
            }
            continue;
        }
        if !warmed && now + warm_ahead >= next {
            shard.warm();
            warmed = true;
            continue;
        }
        // Poll rather than park: a parked receiver makes every send a wake-up
        // (a syscall on the connection's side), 40 000 a second at 2000
        // clients; a poll every POLL_MS costs at most that much latency.
        let msg = match rx.try_recv() {
            Ok(m) => m,
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // Until the warm-up, then until the tick, polling meanwhile.
                let until = if warmed {
                    next
                } else {
                    next.checked_sub(warm_ahead).unwrap_or(now)
                };
                std::thread::sleep(
                    until
                        .saturating_duration_since(now)
                        .min(Duration::from_millis(POLL_MS)),
                );
                continue;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => return,
        };
        match msg {
            Inbound::Drain(reason) => {
                shard.drain(&reason);
                let sessions: Vec<u64> = shard.sessions().collect();
                flush(&mut shard, &mut outs, &sessions);
            }
            Inbound::Join {
                hello,
                claims,
                cap,
                out,
                reply,
            } => {
                let player = claims.player.clone();
                let r = shard.hello(&hello, claims);
                // Close the connections this join replaced (a resume keeps
                // its session id) before the new one takes the id.
                for s in shard.take_closed() {
                    outs.remove(&s);
                }
                if let Ok(s) = r {
                    shard.cap_snapshots(s, cap);
                    outs.insert(s, out);
                    if log {
                        eprintln!(
                            "world-server: {player} joined `{}` ({} players)",
                            hello.world,
                            shard.players()
                        );
                    }
                }
                let ok = r.as_ref().ok().copied();
                let _ = reply.send(r);
                if let Some(s) = ok {
                    flush(&mut shard, &mut outs, &[s]);
                }
            }
            Inbound::Msg { session, bytes } => {
                if !outs.contains_key(&session) {
                    continue;
                }
                if let Err(e) = shard.receive(session, &bytes) {
                    if log {
                        eprintln!("world-server: closed a session: {e}");
                    }
                    outs.remove(&session);
                    shard.disconnect(session);
                } else if bytes.first() == Some(&5) {
                    outs.remove(&session);
                } else {
                    flush(&mut shard, &mut outs, &[session]);
                }
            }
            Inbound::Slow { session, out } => {
                if outs.get(&session).is_some_and(|tx| tx.same_channel(&out)) {
                    outs.remove(&session);
                    shard.disconnect(session);
                }
            }
            Inbound::Closed { session } => {
                // A connection replaced by a newer one closes after its successor took over.
                if outs.get(&session).is_some_and(|tx| tx.is_closed()) {
                    outs.remove(&session);
                    shard.disconnect(session);
                }
            }
        }
    }
}

async fn connection(mut stream: TcpStream, shared: Arc<Shared>) -> Result<(), String> {
    // A health check is plain HTTP, never an upgrade: answer it before the
    // WebSocket handshake would refuse it.
    let mut head = [0u8; 12];
    let n = tokio::time::timeout(Duration::from_secs(5), stream.peek(&mut head))
        .await
        .map_err(|_| "no request")?
        .map_err(|e| e.to_string())?;
    if head[..n] == *b"GET /healthz" {
        use tokio::io::AsyncWriteExt;
        let (status, body) = if shared.draining.load(Ordering::SeqCst) {
            ("503 Service Unavailable", "draining\n")
        } else {
            ("200 OK", "ready\n")
        };
        let reply = format!(
            "HTTP/1.1 {status}\r\ncontent-type: text/plain\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(reply.as_bytes()).await;
        let _ = stream.shutdown().await;
        return Ok(());
    }
    // tungstenite zero-fills its read buffer up to `read_buffer_size` on
    // every read (128 KiB by default): at 20 small frames a second from
    // each of 2000 clients that memset was half the server's CPU. Frames
    // are tens of bytes; a larger one is still read across several reads.
    let config = WebSocketConfig::default()
        .read_buffer_size(READ_BUFFER)
        .max_message_size(Some(MAX_MESSAGE))
        .max_frame_size(Some(MAX_MESSAGE));
    let ws = tokio_tungstenite::accept_async_with_config(stream, Some(config))
        .await
        .map_err(|e| e.to_string())?;
    let (mut sink, mut source) = ws.split();
    let silent = Duration::from_millis(SILENT_MS);
    // The hello.
    let first = match tokio::time::timeout(silent, source.next()).await {
        Ok(Some(Ok(Message::Binary(b)))) => b,
        _ => return Err("no hello".into()),
    };
    shared
        .metrics
        .bytes_in
        .fetch_add(first.len() as u64, Ordering::Relaxed);
    let (session, mut out_rx) = match admit(&shared, &first, usize::MAX).await {
        Ok(a) => a,
        Err((refusal, why)) => {
            let _ = sink.send(Message::binary(refusal)).await;
            return Err(why);
        }
    };
    let metrics = shared.metrics.clone();
    let writer = tokio::spawn(async move {
        let ping = Duration::from_millis(PING_MS);
        loop {
            match tokio::time::timeout(ping, out_rx.recv()).await {
                Ok(Some(m)) => {
                    metrics
                        .bytes_out
                        .fetch_add(m.len() as u64 + 4, Ordering::Relaxed);
                    metrics.frames_out.fetch_add(1, Ordering::Relaxed);
                    if sink.feed(Message::binary(m)).await.is_err() {
                        return;
                    }
                    // Send what is queued as one write.
                    while let Ok(m) = out_rx.try_recv() {
                        metrics
                            .bytes_out
                            .fetch_add(m.len() as u64 + 4, Ordering::Relaxed);
                        metrics.frames_out.fetch_add(1, Ordering::Relaxed);
                        if sink.feed(Message::binary(m)).await.is_err() {
                            return;
                        }
                    }
                    if sink.flush().await.is_err() {
                        return;
                    }
                }
                Ok(None) => {
                    let _ = sink.close().await;
                    return;
                }
                Err(_) => {
                    if sink.send(Message::Ping(Default::default())).await.is_err() {
                        return;
                    }
                }
            }
        }
    });
    let result = loop {
        match tokio::time::timeout(silent, source.next()).await {
            Ok(Some(Ok(Message::Binary(b)))) => {
                shared
                    .metrics
                    .bytes_in
                    .fetch_add(b.len() as u64, Ordering::Relaxed);
                if shared
                    .inbound
                    .send(Inbound::Msg {
                        session,
                        bytes: b.to_vec(),
                    })
                    .is_err()
                {
                    break Ok(());
                }
            }
            Ok(Some(Ok(Message::Close(_)))) | Ok(None) => break Ok(()),
            Ok(Some(Ok(_))) => {}
            Ok(Some(Err(e))) => break Err(e.to_string()),
            Err(_) => break Err("silent for 10 s".into()),
        }
    };
    writer.abort();
    let _ = writer.await;
    let _ = shared.inbound.send(Inbound::Closed { session });
    result
}

/// A hello's way in, on any transport: decoded, its token verified, and
/// handed to the shard. The session and the queue of its messages, or the
/// refusal to send and why.
async fn admit(
    shared: &Arc<Shared>,
    first: &[u8],
    cap: usize,
) -> Result<(u64, mpsc::Receiver<Vec<u8>>), (Vec<u8>, String)> {
    let refuse = |code: Refusal, message: String| {
        (
            ServerMsg::Refused {
                code,
                message: message.clone(),
            }
            .to_bytes(),
            format!("refused: {message}"),
        )
    };
    let hello = match ClientMsg::decode(first, |_| None) {
        Ok(ClientMsg::Hello(h)) => h,
        _ => {
            return Err(refuse(
                Refusal::Invalid,
                "the first message is a hello".into(),
            ));
        }
    };
    let claims = verify(shared, &hello.token)
        .await
        .map_err(|e| refuse(Refusal::Unauthorized, e))?;
    let (out_tx, out_rx) = mpsc::channel::<Vec<u8>>(OUT_QUEUE);
    let (reply_tx, reply_rx) = oneshot::channel();
    let stopped = || (Vec::new(), "the server stopped".to_string());
    shared
        .inbound
        .send(Inbound::Join {
            hello,
            claims,
            cap,
            out: out_tx,
            reply: reply_tx,
        })
        .map_err(|_| stopped())?;
    match reply_rx.await.map_err(|_| stopped())? {
        Ok(s) => Ok((s, out_rx)),
        Err((code, message)) => Err(refuse(code, message)),
    }
}

/// The QUIC endpoint WebTransport is served on: web-transport-quinn's
/// setup (TLS 1.3, its ALPN) with two changes for a client's upload,
/// where a 20 Hz proposal is a few dozen bytes and QUIC's framing is as
/// large again. Clients that support ACK frequency (RFC draft
/// ietf-quic-ack-frequency) are asked to hold their ACKs for the next
/// proposal instead of sending each in a packet of its own, and the
/// server's connection ids, which every client packet carries, are
/// 4 bytes instead of 8.
fn wt_endpoint(
    addr: SocketAddr,
    chain: Vec<web_transport_quinn::quinn::rustls::pki_types::CertificateDer<'static>>,
    key: web_transport_quinn::quinn::rustls::pki_types::PrivateKeyDer<'static>,
) -> Result<web_transport_quinn::quinn::Endpoint, String> {
    use web_transport_quinn::quinn::{self, rustls};
    let mut tls = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_protocol_versions(&[&rustls::version::TLS13])
    .map_err(|e| e.to_string())?
    .with_no_client_auth()
    .with_single_cert(chain, key)
    .map_err(|e| format!("the certificate: {e}"))?;
    tls.alpn_protocols = vec![web_transport_quinn::ALPN.as_bytes().to_vec()];
    let tls = quinn::crypto::rustls::QuicServerConfig::try_from(tls).map_err(|e| e.to_string())?;
    let mut acks = quinn::AckFrequencyConfig::default();
    acks.ack_eliciting_threshold(quinn::VarInt::from_u32(9))
        .max_ack_delay(Some(Duration::from_millis(100)));
    let mut transport = quinn::TransportConfig::default();
    transport.ack_frequency_config(Some(acks));
    let mut config = quinn::ServerConfig::with_crypto(Arc::new(tls));
    config.transport_config(Arc::new(transport));
    let mut endpoint = quinn::EndpointConfig::default();
    endpoint.cid_generator(|| Box::new(quinn_proto::RandomConnectionIdGenerator::new(4)));
    let socket = std::net::UdpSocket::bind(addr).map_err(|e| format!("{addr}: {e}"))?;
    quinn::Endpoint::new(endpoint, Some(config), socket, Arc::new(quinn::TokioRuntime))
        .map_err(|e| format!("{addr}: {e}"))
}

async fn wt_connection(
    request: web_transport_quinn::Request,
    shared: Arc<Shared>,
) -> Result<(), String> {
    let session = request.ok().await.map_err(|e| e.to_string())?;
    let silent = Duration::from_millis(SILENT_MS);
    let (mut send, mut recv) = tokio::time::timeout(silent, session.accept_bi())
        .await
        .map_err(|_| "no stream")?
        .map_err(|e| e.to_string())?;
    let first = tokio::time::timeout(silent, read_frame(&mut recv))
        .await
        .map_err(|_| "no hello")??;
    shared
        .metrics
        .bytes_in
        .fetch_add(first.len() as u64 + 4, Ordering::Relaxed);
    let cap = session.max_datagram_size().min(MAX_DATAGRAM);
    let (id, mut out_rx) = match admit(&shared, &first, cap).await {
        Ok(a) => a,
        Err((refusal, why)) => {
            if !refusal.is_empty() {
                let _ = send.write_all(&frame(&refusal)).await;
                let _ = send.finish();
            }
            // Let the refusal reach the client before the session closes.
            tokio::time::sleep(Duration::from_millis(100)).await;
            session.close(1, b"refused");
            return Err(why);
        }
    };
    let metrics = shared.metrics.clone();
    let out = session.clone();
    let writer = tokio::spawn(async move {
        while let Some(m) = out_rx.recv().await {
            metrics.frames_out.fetch_add(1, Ordering::Relaxed);
            if crate::protocol::server_datagram(&m) && m.len() <= out.max_datagram_size() {
                metrics
                    .bytes_out
                    .fetch_add(m.len() as u64, Ordering::Relaxed);
                // A full send buffer drops it, as the network might: the next one follows.
                if let Err(web_transport_quinn::SessionError::ConnectionError(_)) =
                    out.send_datagram(m.into())
                {
                    return;
                }
                continue;
            }
            let mut f = frame(&m);
            metrics
                .bytes_out
                .fetch_add(f.len() as u64, Ordering::Relaxed);
            // Send what is queued on the stream as one write.
            while let Ok(m) = out_rx.try_recv() {
                metrics.frames_out.fetch_add(1, Ordering::Relaxed);
                if crate::protocol::server_datagram(&m) && m.len() <= out.max_datagram_size() {
                    metrics
                        .bytes_out
                        .fetch_add(m.len() as u64, Ordering::Relaxed);
                    let _ = out.send_datagram(m.into());
                } else {
                    let g = frame(&m);
                    metrics
                        .bytes_out
                        .fetch_add(g.len() as u64, Ordering::Relaxed);
                    f.extend_from_slice(&g);
                }
            }
            if send.write_all(&f).await.is_err() {
                return;
            }
        }
        let _ = send.finish();
    });
    let inbound = shared.inbound.clone();
    let hand = |bytes: Vec<u8>| {
        shared
            .metrics
            .bytes_in
            .fetch_add(bytes.len() as u64, Ordering::Relaxed);
        inbound.send(Inbound::Msg { session: id, bytes }).is_ok()
    };
    let result = loop {
        let next = tokio::time::timeout(silent, async {
            tokio::select! {
                f = read_frame(&mut recv) => f.map(|b| (b, 4)),
                d = session.read_datagram() => d.map(|b| (b.to_vec(), 0)).map_err(|e| e.to_string()),
            }
        });
        match next.await {
            Ok(Ok((b, framing))) => {
                shared
                    .metrics
                    .bytes_in
                    .fetch_add(framing, Ordering::Relaxed);
                if !hand(b) {
                    break Ok(());
                }
            }
            Ok(Err(e)) => break Err(e),
            Err(_) => break Err("silent for 10 s".into()),
        }
    };
    writer.abort();
    let _ = writer.await;
    session.close(0, b"");
    let _ = shared.inbound.send(Inbound::Closed { session: id });
    result
}

async fn verify(shared: &Arc<Shared>, token: &str) -> Result<Claims, String> {
    let r = shared.verifier.read().unwrap().verify(token, now_s());
    match r {
        Ok(c) => Ok(c),
        Err(AuthError::UnknownKey(kid)) => {
            // The issuer may have rotated: fetch its keys again, at most once a minute.
            let due = {
                let mut last = shared.last_refetch.lock().unwrap();
                let due = last.is_none_or(|t| t.elapsed() > Duration::from_secs(60));
                if due {
                    *last = Some(Instant::now());
                }
                due
            };
            if due && !matches!(shared.keys, KeySource::None) {
                let s = shared.clone();
                let fetched = tokio::task::spawn_blocking(move || {
                    load_jwks(&s.keys).and_then(|t| parse_jwks(&t))
                })
                .await
                .map_err(|e| e.to_string())?;
                if let Ok(keys) = fetched
                    && let Verifier::Jwks { keys: k, .. } = &mut *shared.verifier.write().unwrap()
                {
                    *k = keys;
                }
                return shared
                    .verifier
                    .read()
                    .unwrap()
                    .verify(token, now_s())
                    .map_err(|e| e.to_string());
            }
            Err(AuthError::UnknownKey(kid).to_string())
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Serve the metrics text over plain HTTP on `addr` (any path).
pub async fn serve_metrics(addr: SocketAddr, metrics: Arc<Metrics>) -> std::io::Result<SocketAddr> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = TcpListener::bind(addr).await?;
    let local = listener.local_addr()?;
    tokio::spawn(async move {
        loop {
            let Ok((mut s, _)) = listener.accept().await else {
                continue;
            };
            let m = metrics.clone();
            tokio::spawn(async move {
                // Read the whole request head, so closing never resets it.
                let mut req = Vec::new();
                let mut buf = [0u8; 1024];
                let read = async {
                    while !req.windows(4).any(|w| w == b"\r\n\r\n") && req.len() < 8192 {
                        match s.read(&mut buf).await {
                            Ok(0) | Err(_) => break,
                            Ok(n) => req.extend_from_slice(&buf[..n]),
                        }
                    }
                };
                let _ = tokio::time::timeout(Duration::from_secs(10), read).await;
                let body = m.text();
                let head = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: text/plain; version=0.0.4\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                    body.len()
                );
                let _ = s.write_all(head.as_bytes()).await;
                let _ = s.write_all(body.as_bytes()).await;
                let _ = s.shutdown().await;
            });
        }
    });
    Ok(local)
}
