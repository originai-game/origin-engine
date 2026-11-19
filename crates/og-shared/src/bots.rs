//! The load test's bots (`og world-server load`): each one a real client
//! over its own WebSocket or WebTransport session, walking its area at a
//! steady speed with a turn every few seconds, proposing at 20 Hz and
//! decoding everything it is sent through [`View`]. Bytes are counted at
//! the socket: a WebSocket's TCP payload (its framing, not TCP/IP's
//! headers), a WebTransport session's UDP payload (QUIC's headers and
//! encryption, not UDP/IP's).
//!
//! Shaping (`latency_ms`, `loss`): each bot reaches the server through a
//! relay of its own that adds half the latency each way and loses packets.
//! - WebTransport: a UDP relay drops each datagram with the loss's
//!   probability; QUIC recovers the stream, and lost datagrams stay lost.
//! - WebSocket: a userspace relay cannot lose a TCP segment, so it models
//!   TCP's recovery: a chunk lost holds it and everything after it in that
//!   direction for a probe timeout of 2 x RTT (Linux's tail loss probe, at
//!   least 10 ms). That is TCP at its best (no 200 ms delayed-ACK floor,
//!   no backoff on a second loss), so a stall it shows is a floor.

use crate::auth::sign_hs256;
use crate::protocol::{Authority, AvatarState, ClientMsg, Hello, PROTOCOL, ServerMsg};
use crate::view::{Proposer, View};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

/// A stream that counts what crosses it.
pub struct Counted<S> {
    inner: S,
    read: Arc<AtomicU64>,
    written: Arc<AtomicU64>,
}

impl<S: AsyncRead + Unpin> AsyncRead for Counted<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let before = buf.filled().len();
        let r = Pin::new(&mut self.inner).poll_read(cx, buf);
        self.read
            .fetch_add((buf.filled().len() - before) as u64, Ordering::Relaxed);
        r
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for Counted<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let r = Pin::new(&mut self.inner).poll_write(cx, buf);
        if let Poll::Ready(Ok(n)) = r {
            self.written.fetch_add(n as u64, Ordering::Relaxed);
        }
        r
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

/// How the bots are spread and how they move.
#[derive(Clone, Debug)]
pub struct LoadDesc {
    /// `ws://host:port/`.
    pub url: String,
    pub bots: usize,
    /// The side of the square they walk in, metres, centred on `centre`.
    pub area: f64,
    pub centre: [f64; 2],
    /// Areas the bots are dealt to in turn, their centres `spacing` metres apart on x.
    pub areas: usize,
    pub spacing: f64,
    /// Walking speed, m/s.
    pub speed: f32,
    /// Seconds before measuring (joins, enters), then seconds measured.
    pub warmup: f64,
    pub seconds: f64,
    /// HS256 secret and audience; `None`: dev names as tokens.
    pub secret: Option<Vec<u8>>,
    pub audience: String,
    pub world: String,
    /// Bots that join at once, then the next batch after 50 ms.
    pub batch: usize,
    /// For an `https://` URL (WebTransport): the server certificate's
    /// SHA-256s; none: the system's roots.
    pub cert_hashes: Vec<[u8; 32]>,
    /// Shaping (module doc): round-trip latency added, ms, and the share of
    /// packets lost each way (0 to 1).
    pub latency_ms: f64,
    pub loss: f64,
    pub seed: u64,
}

/// One bot's measured numbers.
#[derive(Clone, Debug, Default)]
pub struct BotReport {
    pub joined: bool,
    pub refused: Option<String>,
    pub down: u64,
    pub up: u64,
    pub seconds: f64,
    /// The avatars it held, sampled every tick while measuring: sum and samples.
    pub in_sight_sum: u64,
    pub in_sight_max: usize,
    pub samples: u64,
    pub others_max: u32,
    pub snapshots: u64,
    pub updates: u64,
    pub corrections: u64,
    pub errors: u64,
    /// Snapshots ahead of an enter (crate::view::NOT_ENTERED).
    pub early: u64,
    pub stale: u64,
    pub moved: bool,
    /// Each measured snapshot's delay past the bot's fastest, ms.
    pub added_ms: Vec<f64>,
    /// The times between measured snapshots' arrivals, ms.
    pub gaps_ms: Vec<f64>,
    /// Server ticks from the first measured snapshot to the last.
    pub ticks_spanned: u64,
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Run every bot; their reports in order.
pub async fn run(desc: LoadDesc) -> Vec<BotReport> {
    let desc = Arc::new(desc);
    let mut handles = Vec::new();
    let start = Instant::now();
    for i in 0..desc.bots {
        if i > 0 && i % desc.batch.max(1) == 0 {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let d = desc.clone();
        handles.push(tokio::spawn(async move { bot(&d, i, start).await }));
    }
    let mut out = Vec::new();
    for h in handles {
        out.push(h.await.unwrap_or_default());
    }
    out
}

/// Messages in and out of a bot, whatever carries them.
struct Link {
    out: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
    inbox: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
    /// Bytes read and written at the socket.
    read: Arc<AtomicU64>,
    written: Arc<AtomicU64>,
    tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl Drop for Link {
    fn drop(&mut self) {
        for t in &self.tasks {
            t.abort();
        }
    }
}

/// One direction of a relay: its delay, its loss and their randomness.
struct Shaping {
    one_way: Duration,
    loss: f64,
    rng: Rng,
}

impl Shaping {
    fn new(d: &LoadDesc, i: usize, salt: u64) -> Shaping {
        Shaping {
            one_way: Duration::from_secs_f64(d.latency_ms / 2000.0),
            loss: d.loss,
            rng: Rng(((d.seed ^ salt).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                ^ (i as u64 + 7).wrapping_mul(0xd6e8_feb8_6659_fd93))
                | 1),
        }
    }

    fn lost(&mut self) -> bool {
        self.loss > 0.0 && self.rng.next() < self.loss
    }
}

type Task = tokio::task::JoinHandle<()>;

/// A TCP relay for one connection: half the latency each way and the loss
/// model of the module doc.
async fn tcp_relay(d: &LoadDesc, i: usize, upstream: String) -> std::io::Result<(String, Task)> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let local = listener.local_addr()?.to_string();
    let shapes = [Shaping::new(d, i, 0x51), Shaping::new(d, i, 0xa7)];
    let rtt = Duration::from_secs_f64(d.latency_ms / 1000.0);
    let probe = (rtt * 2).max(Duration::from_millis(10));
    let task = tokio::spawn(async move {
        let Ok((client, _)) = listener.accept().await else {
            return;
        };
        let Ok(server) = TcpStream::connect(&upstream).await else {
            return;
        };
        let _ = client.set_nodelay(true);
        let _ = server.set_nodelay(true);
        let (cr, cw) = client.into_split();
        let (sr, sw) = server.into_split();
        let [a, b] = shapes;
        tokio::join!(pipe(cr, sw, a, probe), pipe(sr, cw, b, probe));
    });
    Ok((local, task))
}

/// One direction of the TCP relay, in order: a chunk never leaves before
/// the one ahead of it, so a lost one holds everything behind it.
async fn pipe(
    mut from: tokio::net::tcp::OwnedReadHalf,
    mut to: tokio::net::tcp::OwnedWriteHalf,
    mut shape: Shaping,
    probe: Duration,
) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(Instant, Vec<u8>)>();
    let writer = tokio::spawn(async move {
        while let Some((at, buf)) = rx.recv().await {
            tokio::time::sleep_until(at.into()).await;
            if to.write_all(&buf).await.is_err() {
                return;
            }
        }
    });
    let mut last = Instant::now();
    let mut buf = vec![0u8; 16 * 1024];
    loop {
        let n = match from.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        let mut at = Instant::now() + shape.one_way;
        if shape.lost() {
            at += probe;
        }
        at = at.max(last);
        last = at;
        if tx.send((at, buf[..n].to_vec())).is_err() {
            break;
        }
    }
    drop(tx);
    let _ = writer.await;
}

/// A UDP relay for one QUIC connection: each datagram delayed half the
/// latency and lost with the loss's probability, each way.
async fn udp_relay(
    d: &LoadDesc,
    i: usize,
    upstream: std::net::SocketAddr,
) -> std::io::Result<(std::net::SocketAddr, Task)> {
    let front = Arc::new(tokio::net::UdpSocket::bind("127.0.0.1:0").await?);
    let back = Arc::new(tokio::net::UdpSocket::bind("127.0.0.1:0").await?);
    back.connect(upstream).await?;
    let local = front.local_addr()?;
    let (mut up, mut down) = (Shaping::new(d, i, 0x51), Shaping::new(d, i, 0xa7));
    let task = tokio::spawn(async move {
        let client: Arc<std::sync::Mutex<Option<std::net::SocketAddr>>> = Arc::default();
        let (f, b, c) = (front.clone(), back.clone(), client.clone());
        let upward = async move {
            let mut buf = vec![0u8; 65536];
            while let Ok((n, from)) = f.recv_from(&mut buf).await {
                *c.lock().unwrap() = Some(from);
                if up.lost() {
                    continue;
                }
                let (b, pkt, wait) = (b.clone(), buf[..n].to_vec(), up.one_way);
                tokio::spawn(async move {
                    tokio::time::sleep(wait).await;
                    let _ = b.send(&pkt).await;
                });
            }
        };
        let downward = async move {
            let mut buf = vec![0u8; 65536];
            while let Ok(n) = back.recv(&mut buf).await {
                let Some(to) = *client.lock().unwrap() else {
                    continue;
                };
                if down.lost() {
                    continue;
                }
                let (f, pkt, wait) = (front.clone(), buf[..n].to_vec(), down.one_way);
                tokio::spawn(async move {
                    tokio::time::sleep(wait).await;
                    let _ = f.send_to(&pkt, to).await;
                });
            }
        };
        tokio::join!(upward, downward);
    });
    Ok((local, task))
}

fn shaped(d: &LoadDesc) -> bool {
    d.latency_ms > 0.0 || d.loss > 0.0
}

/// A WebSocket link to `d.url`, through a relay when shaping.
async fn ws_link(d: &LoadDesc, i: usize) -> Result<Link, String> {
    let (read, written) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
    let host = d
        .url
        .trim_start_matches("ws://")
        .split('/')
        .next()
        .unwrap_or("")
        .to_string();
    let mut tasks = Vec::new();
    let dial = if shaped(d) {
        let (local, t) = tcp_relay(d, i, host).await.map_err(|e| e.to_string())?;
        tasks.push(t);
        local
    } else {
        host
    };
    let tcp = TcpStream::connect(&dial)
        .await
        .map_err(|e| format!("cannot connect: {e}"))?;
    let _ = tcp.set_nodelay(true);
    let stream = Counted {
        inner: tcp,
        read: read.clone(),
        written: written.clone(),
    };
    let (ws, _) = tokio_tungstenite::client_async_with_config(
        d.url.as_str(),
        stream,
        Some(
            tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default()
                .read_buffer_size(crate::server::READ_BUFFER),
        ),
    )
    .await
    .map_err(|_| "the handshake failed".to_string())?;
    let (mut sink, mut source) = ws.split();
    let (out, mut out_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    let (in_tx, inbox) = tokio::sync::mpsc::unbounded_channel();
    tasks.push(tokio::spawn(async move {
        while let Some(m) = out_rx.recv().await {
            if sink.send(Message::binary(m)).await.is_err() {
                return;
            }
        }
        let _ = sink.close().await;
    }));
    tasks.push(tokio::spawn(async move {
        while let Some(Ok(m)) = source.next().await {
            if let Message::Binary(b) = m
                && in_tx.send(b.to_vec()).is_err()
            {
                return;
            }
        }
    }));
    Ok(Link {
        out,
        inbox,
        read,
        written,
        tasks,
    })
}

/// A WebTransport link to `d.url`, through a relay when shaping.
async fn wt_link(d: &LoadDesc, i: usize) -> Result<Link, String> {
    use crate::protocol::client_datagram;
    use crate::wt::{frame, read_frame};
    let parsed = url::Url::parse(&d.url).map_err(|e| e.to_string())?;
    let mut tasks = Vec::new();
    let url = if shaped(d) {
        let host = parsed.host_str().unwrap_or("127.0.0.1");
        let upstream = tokio::net::lookup_host((host, parsed.port().unwrap_or(443)))
            .await
            .map_err(|e| e.to_string())?
            .next()
            .ok_or_else(|| "no address".to_string())?;
        let (local, t) = udp_relay(d, i, upstream).await.map_err(|e| e.to_string())?;
        tasks.push(t);
        format!("https://{local}/")
    } else {
        d.url.clone()
    };
    let session = tokio::time::timeout(
        Duration::from_secs(10),
        crate::wt::dial(&url, &d.cert_hashes),
    )
    .await
    .map_err(|_| "no answer in 10 s".to_string())??;
    let (mut send, mut recv) = session.open_bi().await.map_err(|e| e.to_string())?;
    let (read, written) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
    let (out, mut out_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    let (in_tx, inbox) = tokio::sync::mpsc::unbounded_channel();
    let s = session.clone();
    tasks.push(tokio::spawn(async move {
        while let Some(m) = out_rx.recv().await {
            if client_datagram(&m) && m.len() <= s.max_datagram_size() {
                let _ = s.send_datagram(m.into());
            } else if send.write_all(&frame(&m)).await.is_err() {
                return;
            }
        }
        let _ = send.finish();
    }));
    let frames = in_tx.clone();
    tasks.push(tokio::spawn(async move {
        while let Ok(f) = read_frame(&mut recv).await {
            if frames.send(f).is_err() {
                return;
            }
        }
    }));
    let s = session.clone();
    tasks.push(tokio::spawn(async move {
        while let Ok(dg) = s.read_datagram().await {
            if in_tx.send(dg.to_vec()).is_err() {
                return;
            }
        }
    }));
    // QUIC's own counts of UDP payload, sampled while the session lives.
    let (r, w, s) = (read.clone(), written.clone(), session.clone());
    tasks.push(tokio::spawn(async move {
        loop {
            let st = web_transport_quinn::quinn::Connection::stats(&s);
            r.store(st.udp_rx.bytes, Ordering::Relaxed);
            w.store(st.udp_tx.bytes, Ordering::Relaxed);
            if s.close_reason().is_some() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }));
    Ok(Link {
        out,
        inbox,
        read,
        written,
        tasks,
    })
}

async fn bot(d: &LoadDesc, i: usize, start: Instant) -> BotReport {
    let mut rep = BotReport::default();
    let link = if d.url.starts_with("https://") {
        wt_link(d, i).await
    } else {
        ws_link(d, i).await
    };
    let mut link = match link {
        Ok(l) => l,
        Err(e) => {
            rep.refused = Some(e);
            return rep;
        }
    };
    let (read, written) = (link.read.clone(), link.written.clone());
    let player = format!("bot-{i}");
    let token = match &d.secret {
        Some(s) => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            sign_hs256(
                s,
                &json!({ "aud": d.audience, "sub": player, "name": player, "iat": now, "exp": now + 300 }),
            )
        }
        None => player.clone(),
    };
    let hello = ClientMsg::Hello(Hello {
        version: PROTOCOL,
        token,
        world: d.world.clone(),
        shape: 1,
        fields_len: 2,
        authority: Authority::Validate,
        send_hz: 20,
        kind: 0,
        resume: None,
    });
    if link.out.send(hello.to_bytes()).is_err() {
        return rep;
    }
    let mut rng = Rng(d.seed.wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ (i as u64 + 1).wrapping_mul(0xbf58_476d_1ce4_e5b9)
        | 1);
    let half = d.area / 2.0;
    let centre = [
        d.centre[0] + (i % d.areas.max(1)) as f64 * d.spacing,
        d.centre[1],
    ];
    let mut pos = [
        centre[0] + (rng.next() * 2.0 - 1.0) * half,
        1.0,
        centre[1] + (rng.next() * 2.0 - 1.0) * half,
    ];
    let mut heading = rng.next() * std::f64::consts::TAU;
    let mut turn_in = 2.0 + rng.next() * 4.0;
    // Proposals at 20 Hz, each on a snapshot's arrival as og.shared sends
    // them (og-runtime bind/shared.rs `propose`): once a snapshot has
    // arrived and two thirds of the period have passed, else after one and
    // a half periods. The walk moves by the time since the last proposal.
    const PERIOD: Duration = Duration::from_millis(50);
    let mut last_prop = Instant::now();
    let mut view = View::new();
    let mut prop = Proposer::default();
    let mut cell_size = 128.0;
    let mut tick_hz = 20.0;
    let mut tick = tokio::time::interval(PERIOD * 3 / 2);
    let measure_from = start + Duration::from_secs_f64(d.warmup);
    let end = start + Duration::from_secs_f64(d.warmup + d.seconds);
    let mut base: Option<(u64, u64, Instant)> = None;
    let mut first = None;
    // Measured snapshots: when each arrived, and its server tick.
    let mut arrivals: Vec<(Instant, u32)> = Vec::new();
    loop {
        tokio::select! {
            _ = tick.tick() => {
                let now = Instant::now();
                if now >= end {
                    break;
                }
                if !rep.joined {
                    continue;
                }
                let dt = now.duration_since(last_prop).as_secs_f64().min(0.25);
                last_prop = now;
                if now >= measure_from {
                    if base.is_none() {
                        base = Some((read.load(Ordering::Relaxed), written.load(Ordering::Relaxed), now));
                    }
                    rep.samples += 1;
                    rep.in_sight_sum += view.remotes.len() as u64;
                    rep.in_sight_max = rep.in_sight_max.max(view.remotes.len());
                    rep.others_max = rep.others_max.max(view.others);
                }
                // Walk: straight, a turn every 2 to 6 s, turned back at the area's edge.
                turn_in -= dt;
                if turn_in <= 0.0 {
                    heading += (rng.next() - 0.5) * 2.0;
                    turn_in = 2.0 + rng.next() * 4.0;
                }
                let v = [heading.cos() * f64::from(d.speed), heading.sin() * f64::from(d.speed)];
                for (a, k) in [(0usize, 0usize), (2, 1)] {
                    let next = pos[a] + v[k] * dt;
                    if (next - centre[k]).abs() > half {
                        heading = (heading + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU);
                    } else {
                        pos[a] = next;
                    }
                }
                let mut s = AvatarState::from_world(cell_size, pos, [v[0] as f32, 0.0, v[1] as f32], heading as f32);
                s.set_fields(&[1, (i % 8) as u8]);
                let b = prop.propose(s, false, view.ack(), view.ack_seq);
                if first.is_none() {
                    first = Some(pos);
                } else if first.is_some_and(|f| (f[0] - pos[0]).abs() > 1.0 || (f[2] - pos[2]).abs() > 1.0) {
                    rep.moved = true;
                }
                if link.out.send(b).is_err() {
                    break;
                }
            }
            m = link.inbox.recv() => {
                let Some(b) = m else { break };
                let now = Instant::now();
                let measuring = now >= measure_from;
                match ServerMsg::decode(&b) {
                    Ok(ServerMsg::Welcome(w)) => {
                        rep.joined = true;
                        cell_size = f64::from(w.cell_size);
                        tick_hz = f64::from(w.tick_hz);
                    }
                    Ok(ServerMsg::Refused { code, message }) => {
                        rep.refused = Some(format!("{}: {message}", code.code()));
                        break;
                    }
                    Ok(ServerMsg::Enter(e)) => view.enter(&e),
                    Ok(ServerMsg::Leave { id }) => {
                        view.leave(id);
                    }
                    Ok(ServerMsg::Snapshot(s)) => {
                        let at = tokio::time::Instant::from_std((last_prop + PERIOD * 2 / 3).max(now));
                        tick.reset_at(at);
                        match view.snapshot(&s) {
                        Ok(Some(a)) => {
                            if measuring {
                                rep.snapshots += 1;
                                rep.updates += a.updates.len() as u64;
                                arrivals.push((now, a.head.tick));
                            }
                        }
                        Ok(None) => rep.stale += 1,
                        Err(crate::view::NOT_ENTERED) => rep.early += 1,
                        Err(_) => rep.errors += 1,
                        }
                    }
                    Ok(ServerMsg::Correct { .. }) => rep.corrections += 1,
                    Ok(_) => {}
                    Err(_) => rep.errors += 1,
                }
            }
        }
    }
    if let Some((r0, w0, t0)) = base {
        rep.down = read.load(Ordering::Relaxed) - r0;
        rep.up = written.load(Ordering::Relaxed) - w0;
        rep.seconds = t0.elapsed().as_secs_f64();
    }
    // Delivery: each snapshot's delay past this bot's fastest (what a held
    // or retransmitted one adds), the gaps between arrivals, and the server
    // ticks spanned (it sends one snapshot a tick).
    if let (Some(&(t0, k0)), Some(&(_, k1))) = (arrivals.first(), arrivals.last()) {
        let late: Vec<f64> = arrivals
            .iter()
            .map(|(t, k)| {
                t.duration_since(t0).as_secs_f64() * 1e3 - f64::from(k - k0) * 1e3 / tick_hz
            })
            .collect();
        let best = late.iter().copied().fold(f64::INFINITY, f64::min);
        rep.added_ms = late.iter().map(|l| l - best).collect();
        rep.gaps_ms = arrivals
            .windows(2)
            .map(|w| w[1].0.duration_since(w[0].0).as_secs_f64() * 1e3)
            .collect();
        rep.ticks_spanned = u64::from(k1 - k0) + 1;
    }
    let _ = link.out.send(ClientMsg::Leave.to_bytes());
    tokio::time::sleep(Duration::from_millis(20)).await;
    rep
}

/// The run's numbers.
#[derive(Clone, Debug)]
pub struct Summary {
    pub bots: usize,
    pub joined: usize,
    pub refused: Vec<String>,
    /// Per client, bytes a second: mean, p95, max.
    pub down: [f64; 3],
    pub up: [f64; 3],
    /// Avatars held, mean over clients and samples, and the most any held.
    pub in_sight_mean: f64,
    pub in_sight_max: usize,
    pub others_max: u32,
    /// Updates a client got a second, mean.
    pub updates_per_s: f64,
    pub corrections: u64,
    /// Messages that did not decode: a fault.
    pub errors: u64,
    /// Snapshots that overtook an enter and waited for the next
    /// (crate::view::NOT_ENTERED): expected under loss over WebTransport.
    pub early: u64,
    pub stale: u64,
    pub moved: usize,
    /// Over every bot's snapshots: the delay past each bot's fastest (p50,
    /// p99), the gap between arrivals (p50, p99, max), and the share of the
    /// server's snapshots that arrived.
    pub added_ms: [f64; 2],
    pub gap_ms: [f64; 3],
    pub delivered: f64,
}

fn sorted(it: impl Iterator<Item = f64>) -> Vec<f64> {
    let mut v: Vec<f64> = it.collect();
    v.sort_by(f64::total_cmp);
    v
}

fn quantile(v: &[f64], q: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v[((v.len() - 1) as f64 * q).round() as usize]
}

pub fn summarize(reps: &[BotReport]) -> Summary {
    let joined: Vec<&BotReport> = reps
        .iter()
        .filter(|r| r.joined && r.seconds > 0.0)
        .collect();
    let rates = |f: &dyn Fn(&BotReport) -> u64| {
        let mut v: Vec<f64> = joined.iter().map(|r| f(r) as f64 / r.seconds).collect();
        v.sort_by(f64::total_cmp);
        if v.is_empty() {
            return [0.0; 3];
        }
        let mean = v.iter().sum::<f64>() / v.len() as f64;
        [
            mean,
            v[((v.len() - 1) as f64 * 0.95).round() as usize],
            *v.last().unwrap(),
        ]
    };
    let mut refused: Vec<String> = reps.iter().filter_map(|r| r.refused.clone()).collect();
    refused.sort();
    refused.dedup();
    let samples: u64 = joined.iter().map(|r| r.samples).sum();
    Summary {
        bots: reps.len(),
        joined: reps.iter().filter(|r| r.joined).count(),
        refused,
        down: rates(&|r| r.down),
        up: rates(&|r| r.up),
        in_sight_mean: joined.iter().map(|r| r.in_sight_sum).sum::<u64>() as f64
            / samples.max(1) as f64,
        in_sight_max: joined.iter().map(|r| r.in_sight_max).max().unwrap_or(0),
        others_max: joined.iter().map(|r| r.others_max).max().unwrap_or(0),
        updates_per_s: joined
            .iter()
            .map(|r| r.updates as f64 / r.seconds)
            .sum::<f64>()
            / joined.len().max(1) as f64,
        corrections: reps.iter().map(|r| r.corrections).sum(),
        errors: reps.iter().map(|r| r.errors).sum(),
        early: reps.iter().map(|r| r.early).sum(),
        stale: reps.iter().map(|r| r.stale).sum(),
        moved: reps.iter().filter(|r| r.moved).count(),
        added_ms: {
            let v = sorted(joined.iter().flat_map(|r| r.added_ms.iter().copied()));
            [quantile(&v, 0.5), quantile(&v, 0.99)]
        },
        gap_ms: {
            let v = sorted(joined.iter().flat_map(|r| r.gaps_ms.iter().copied()));
            [
                quantile(&v, 0.5),
                quantile(&v, 0.99),
                v.last().copied().unwrap_or(0.0),
            ]
        },
        delivered: joined.iter().map(|r| r.snapshots).sum::<u64>() as f64
            / joined.iter().map(|r| r.ticks_spanned).sum::<u64>().max(1) as f64,
    }
}
