//! The room's lifecycle, the host's side: what the engine asks for and what the host delivers.

use super::*;

/// `og_room_take`: the ops since the last call, oldest first.
pub fn og_room_take(engine: u32) -> String {
    with(engine, |n| {
        Value::Array(std::mem::take(&mut n.outbox)).to_string()
    })
}

/// `og_room_set_host`: what carries the rooms, and with `join` the room a
/// Play link (`?room=<id>`) puts this engine in. The engine joins it itself
/// (once: a link is not joined again after a leave), so the game hears the
/// same og.RoomJoined as for its own roomJoin, and a roomJoin of its own
/// meanwhile is refused as one already being joined. Without a host, or
/// with a malformed id, og.RoomJoined says why.
pub fn og_room_set_host(engine: u32, info: String) {
    let v: Value = serde_json::from_str(&info).unwrap_or(Value::Null);
    let host = v["host"].as_str().unwrap_or("none").to_string();
    let join = v["join"].as_str().unwrap_or("").to_string();
    with(engine, |n| {
        n.host = host;
        if join.is_empty() || n.link == join {
            return;
        }
        n.link = join.clone();
        let refused = |reason: String| {
            (
                JOINED,
                json!({ "ok": false, "room": "", "peer": 0, "peers": 0, "reason": reason }),
            )
        };
        if !protocol::valid_room(&join) {
            n.pending.push(refused(format!(
                "the Play link's room '{join}' is not a room id: 1 to 64 of letters, digits, '.', '_' and '-'"
            )));
            return;
        }
        if n.host.is_empty() || n.host == "none" {
            n.link.clear();
            if n.link_waiting != join {
                n.link_waiting = join.clone();
                n.pending.push(refused(format!(
                    "the Play link's room '{join}': this page carries no rooms (the platform's Player answers room.join)"
                )));
            }
            return;
        }
        match can_join(n) {
            Ok(()) => {
                n.stage = Stage::Joining;
                n.outbox.push(protocol::join(&join, ""));
            }
            Err(e) => n
                .pending
                .push(refused(format!("the Play link's room '{join}': {e}"))),
        }
    });
}

/// `og_room_deliver`: one event from the host.
pub fn og_room_deliver(engine: u32, event: String) -> bool {
    let Ok(v) = serde_json::from_str::<Value>(&event) else {
        return false;
    };
    let Ok(e) = Event::parse(&v) else {
        return false;
    };
    let tick = tick_of(engine);
    with(engine, |n| {
        n.ls.set_clock(crate::author::probe::now_ms() as u64);
        n.ls.on_event(&e);
        match e {
            Event::Joined { room, peer, peers } => {
                n.stage = Stage::In;
                n.room = room.clone();
                n.peer = peer;
                n.peers = peers.into_iter().collect();
                n.peers.entry(peer).or_default();
                let count = n.peers.len();
                n.pending.push((
                    JOINED,
                    json!({ "ok": true, "room": room, "peer": peer, "peers": count, "reason": "" }),
                ));
            }
            Event::PeerJoined { peer, name, .. } => {
                n.peers.insert(peer, name);
                let count = n.peers.len();
                n.pending
                    .push((PEER_JOINED, json!({ "peer": peer, "peers": count })));
            }
            Event::PeerLeft { peer } => {
                n.peers.remove(&peer);
                let count = n.peers.len();
                n.pending
                    .push((PEER_LEFT, json!({ "peer": peer, "peers": count })));
            }
            Event::Left { .. } => {
                if n.stage == Stage::Joining {
                    n.pending.push((
                        JOINED,
                        json!({ "ok": false, "room": "", "peer": 0, "peers": 0, "reason": "cancelled: roomLeave while joining" }),
                    ));
                }
                let was = (n.stage == Stage::In).then_some(n.peer);
                n.stage = Stage::None;
                n.room.clear();
                n.peers.clear();
                if let Some(peer) = was {
                    n.pending
                        .push((PEER_LEFT, json!({ "peer": peer, "peers": 0 })));
                }
            }
            Event::Error { op, code, message } => {
                if matches!(op.as_str(), "create" | "join" | "quick") && n.stage != Stage::In {
                    n.stage = Stage::None;
                    n.pending.push((
                        JOINED,
                        json!({ "ok": false, "room": "", "peer": 0, "peers": 0, "reason": format!("{code}: {message}") }),
                    ));
                } else if let Ok(eng) = engine_of(engine)
                    && let Ok(mut st) = eng.state.try_borrow_mut()
                {
                    st.log.push(json!({ "tick": tick, "warning": true, "text": format!("room {op}: {code}: {message}") }));
                }
            }
            Event::Message {
                from,
                channel,
                payload,
            } => {
                if !channel.as_deref().is_some_and(|c| c.starts_with("og.")) {
                    n.next_message = n.next_message.wrapping_add(1).max(1);
                    let id = n.next_message;
                    let binary = matches!(payload, Payload::Bytes(_));
                    n.messages.push_back(Message {
                        id,
                        from,
                        tick: None,
                        payload,
                    });
                    while n.messages.len() > KEEP_MESSAGES {
                        n.messages.pop_front();
                    }
                    n.pending.push((
                        MESSAGE,
                        json!({ "from": from, "message": id, "binary": binary }),
                    ));
                }
            }
        }
        flush(engine, n, tick);
    });
    true
}
