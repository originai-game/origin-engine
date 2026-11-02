//! The relay's client: one WebSocket to `og relay` on a thread of its own,
//! ops in and events out through queues, so the engine's thread never
//! blocks on the network. `og run --room ws://127.0.0.1:P/<room>` connects
//! with the room in the path: the relay joins it at once.

use serde_json::Value;
use std::io;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use tungstenite::Message;
use tungstenite::stream::MaybeTlsStream;

/// A connection to a relay. Dropping it closes the connection.
pub struct RelayClient {
    url: String,
    ops: Sender<String>,
    events: Receiver<Value>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl RelayClient {
    /// Connects (the handshake happens here, so a wrong URL is an error at
    /// once). `url` is `ws://host:port/` or `ws://host:port/<room>`.
    pub fn connect(url: &str) -> Result<RelayClient, String> {
        if !url.starts_with("ws://") {
            return Err(format!(
                "'{url}' is not a relay URL: write ws://127.0.0.1:<port>/<room> (og relay prints its port)"
            ));
        }
        let (mut socket, _) = tungstenite::connect(url).map_err(|e| {
            format!("cannot reach the relay at {url} ({e}); start one with `og relay`")
        })?;
        if let MaybeTlsStream::Plain(s) = socket.get_ref() {
            let _ = s.set_nodelay(true);
            let _ = s.set_read_timeout(Some(Duration::from_millis(1)));
        }
        let (ops, ops_rx) = mpsc::channel::<String>();
        let (events_tx, events) = mpsc::channel::<Value>();
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let thread = thread::Builder::new()
            .name("og-room-client".into())
            .spawn(move || run(&mut socket, ops_rx, events_tx, stopping))
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
        let _ = self.ops.send(op.to_string());
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

fn run(
    socket: &mut tungstenite::WebSocket<MaybeTlsStream<TcpStream>>,
    ops: Receiver<String>,
    events: Sender<Value>,
    stop: Arc<AtomicBool>,
) {
    let lost = |why: &str| serde_json::json!({ "type": "left", "reason": why });
    loop {
        if stop.load(Ordering::Relaxed) {
            // Everything queued goes before the close, so the other peers
            // get this one's last inputs before its departure.
            while let Ok(op) = ops.try_recv() {
                let _ = socket.write(Message::text(op));
            }
            let _ = socket.close(None);
            let _ = socket.flush();
            return;
        }
        // A signal interrupting a write leaves the frame queued in the
        // socket: the next flush sends it.
        let failed = |r: tungstenite::Result<()>| {
            r.is_err_and(|e| !matches!(&e, tungstenite::Error::Io(e) if e.kind() == io::ErrorKind::Interrupted))
        };
        let mut wrote = false;
        while let Ok(op) = ops.try_recv() {
            if failed(socket.write(Message::text(op))) {
                let _ = events.send(lost("the relay connection failed"));
                return;
            }
            wrote = true;
        }
        if wrote && failed(socket.flush()) {
            let _ = events.send(lost("the relay connection failed"));
            return;
        }
        match socket.read() {
            Ok(Message::Text(text)) => {
                if let Ok(v) = serde_json::from_str::<Value>(text.as_str()) {
                    let _ = events.send(v);
                }
            }
            Ok(Message::Close(_)) => {
                let _ = events.send(lost("the relay closed the connection"));
                return;
            }
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
            Err(_) => {
                let _ = events.send(lost("the relay connection failed"));
                return;
            }
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
                name: String::new()
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
