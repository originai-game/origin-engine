//! The load test's bots (`og world-server load`): each one a real client
//! over its own WebSocket, walking its area at a steady speed with a turn
//! every few seconds, proposing at 20 Hz and decoding everything it is
//! sent through [`View`]. Bytes are counted at the socket, so they include
//! the WebSocket framing (not TCP/IP's headers).

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
    pub stale: u64,
    pub moved: bool,
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

async fn bot(d: &LoadDesc, i: usize, start: Instant) -> BotReport {
    let mut rep = BotReport::default();
    let (read, written) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
    let host = d
        .url
        .trim_start_matches("ws://")
        .split('/')
        .next()
        .unwrap_or("")
        .to_string();
    let Ok(tcp) = TcpStream::connect(&host).await else {
        rep.refused = Some("cannot connect".into());
        return rep;
    };
    let _ = tcp.set_nodelay(true);
    let stream = Counted {
        inner: tcp,
        read: read.clone(),
        written: written.clone(),
    };
    let Ok((ws, _)) = tokio_tungstenite::client_async_with_config(
        d.url.as_str(),
        stream,
        Some(
            tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default()
                .read_buffer_size(crate::server::READ_BUFFER),
        ),
    )
    .await
    else {
        rep.refused = Some("the handshake failed".into());
        return rep;
    };
    let (mut sink, mut source) = ws.split();
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
    if sink.send(Message::binary(hello.to_bytes())).await.is_err() {
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
    let dt = 0.05;
    let mut view = View::new();
    let mut prop = Proposer::default();
    let mut cell_size = 128.0;
    let mut tick = tokio::time::interval(Duration::from_millis(50));
    let measure_from = start + Duration::from_secs_f64(d.warmup);
    let end = start + Duration::from_secs_f64(d.warmup + d.seconds);
    let mut base: Option<(u64, u64, Instant)> = None;
    let mut first = None;
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
                    let c = centre[k];
                    if (next - c).abs() > half {
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
                if sink.send(Message::binary(b)).await.is_err() {
                    break;
                }
            }
            m = source.next() => {
                let Some(Ok(m)) = m else { break };
                let Message::Binary(b) = m else { continue };
                let measuring = Instant::now() >= measure_from;
                match ServerMsg::decode(&b) {
                    Ok(ServerMsg::Welcome(w)) => {
                        rep.joined = true;
                        cell_size = f64::from(w.cell_size);
                    }
                    Ok(ServerMsg::Refused { code, message }) => {
                        rep.refused = Some(format!("{}: {message}", code.code()));
                        break;
                    }
                    Ok(ServerMsg::Enter(e)) => view.enter(&e),
                    Ok(ServerMsg::Leave { id }) => {
                        view.leave(id);
                    }
                    Ok(ServerMsg::Snapshot(s)) => match view.snapshot(&s) {
                        Ok(Some(a)) => {
                            if measuring {
                                rep.snapshots += 1;
                                rep.updates += a.updates.len() as u64;
                            }
                        }
                        Ok(None) => rep.stale += 1,
                        Err(_) => rep.errors += 1,
                    },
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
    let _ = sink
        .send(Message::binary(ClientMsg::Leave.to_bytes()))
        .await;
    let _ = sink.close().await;
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
    pub errors: u64,
    pub stale: u64,
    pub moved: usize,
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
        stale: reps.iter().map(|r| r.stale).sum(),
        moved: reps.iter().filter(|r| r.moved).count(),
    }
}
