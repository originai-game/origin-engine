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
        thread::Builder::new()
            .name("og-room-client".into())
            .spawn(move || run(&mut socket, ops_rx, events_tx, stopping))
            .map_err(|e| e.to_string())?;
        Ok(RelayClient {
            url: url.to_string(),
            ops,
            events,
            stop,
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
        let mut wrote = false;
        while let Ok(op) = ops.try_recv() {
            if socket.write(Message::text(op)).is_err() {
                let _ = events.send(lost("the relay connection failed"));
                return;
            }
            wrote = true;
        }
        if wrote && socket.flush().is_err() {
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
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
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
}
