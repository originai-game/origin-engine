//! The room's wire shapes, one JSON object each (docs/design.md §2.8).
//!
//! Ops, from the engine to its host (`og_room_take`), and from a client to
//! the relay as text frames:
//!
//! ```text
//! { op: "create", name?, capacity? }      a new room; joined (or error) answers
//! { op: "join", room, name? }             an existing room (the relay makes it on first join)
//! { op: "leave" }                         left answers
//! { op: "send", to?, channel?, data? | bytes? }   to one peer, or every other peer
//! ```
//!
//! Events, from the host to the engine (`og_room_deliver`), and from the
//! relay to a client:
//!
//! ```text
//! { type: "joined", room, peer, peers: [{ peer, name }] }   this peer is in (peers include it)
//! { type: "peer_joined", peer, name }
//! { type: "peer_left", peer }               after every message that peer sent
//! { type: "message", from, channel?, data? | bytes? }
//! { type: "left", reason? }                 roomLeave done, or the host lost the room
//! { type: "error", op, code, message }      unavailable, not_found, full, invalid, offline
//! ```
//!
//! A peer is a number the room gives in join order and never reuses. Per
//! sender, messages arrive in the order they were sent, and a departure
//! after the last of them: lockstep's agreement on a leaver's last input
//! rests on it. `bytes` is base64. A `channel` starting with `og.` is the
//! engine's own (lockstep's `og.lockstep`): its messages never reach the
//! game as og.RoomMessage.

use crate::base64;
use serde_json::{Value, json};

/// Lockstep's channel.
pub const LOCKSTEP_CHANNEL: &str = "og.lockstep";
/// The largest message a game sends, encoded (its JSON, or its bytes).
pub const MAX_MESSAGE: usize = 64 * 1024;
/// The largest engine message (a world snapshot for a late joiner).
pub const MAX_ENGINE_MESSAGE: usize = 16 << 20;
/// The most peers a room takes.
pub const MAX_CAPACITY: u32 = 16;

/// What a message carries.
#[derive(Clone, Debug, PartialEq)]
pub enum Payload {
    Json(Value),
    Bytes(Vec<u8>),
}

impl Payload {
    /// The payload's fields on an op or event: `data` or `bytes`.
    fn put(&self, v: &mut Value) {
        match self {
            Payload::Json(d) => v["data"] = d.clone(),
            Payload::Bytes(b) => v["bytes"] = json!(base64::encode(b)),
        }
    }

    fn take(v: &Value) -> Result<Payload, String> {
        match (v.get("bytes"), v.get("data")) {
            (Some(Value::String(b)), _) => base64::decode(b)
                .map(Payload::Bytes)
                .ok_or_else(|| "a message's bytes are not base64".to_string()),
            (Some(_), _) => Err("a message's bytes are a base64 string".into()),
            (None, d) => Ok(Payload::Json(d.cloned().unwrap_or(Value::Null))),
        }
    }

    /// Its size as the room counts it: the JSON's length, or the bytes'.
    pub fn size(&self) -> usize {
        match self {
            Payload::Json(d) => d.to_string().len(),
            Payload::Bytes(b) => b.len(),
        }
    }
}

/// A room id or a peer's name: 1 to 64 letters, digits, `.`, `_` and `-`.
pub fn valid_room(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

pub fn create(name: &str, capacity: u32) -> Value {
    json!({ "op": "create", "name": name, "capacity": capacity })
}

pub fn join(room: &str, name: &str) -> Value {
    json!({ "op": "join", "room": room, "name": name })
}

pub fn leave() -> Value {
    json!({ "op": "leave" })
}

pub fn send(to: Option<u32>, channel: Option<&str>, payload: &Payload) -> Value {
    let mut v = json!({ "op": "send" });
    if let Some(to) = to {
        v["to"] = json!(to);
    }
    if let Some(c) = channel {
        v["channel"] = json!(c);
    }
    payload.put(&mut v);
    v
}

/// One event, parsed.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Joined {
        room: String,
        peer: u32,
        peers: Vec<(u32, String)>,
    },
    PeerJoined {
        peer: u32,
        name: String,
    },
    PeerLeft {
        peer: u32,
    },
    Message {
        from: u32,
        channel: Option<String>,
        payload: Payload,
    },
    Left {
        reason: String,
    },
    Error {
        op: String,
        code: String,
        message: String,
    },
}

fn peer_of(v: &Value, key: &str) -> Result<u32, String> {
    v[key]
        .as_u64()
        .and_then(|p| u32::try_from(p).ok())
        .ok_or_else(|| format!("a room event's `{key}` is a peer number"))
}

fn text(v: &Value, key: &str) -> String {
    v[key].as_str().unwrap_or_default().to_string()
}

impl Event {
    pub fn parse(v: &Value) -> Result<Event, String> {
        Ok(match v["type"].as_str().unwrap_or_default() {
            "joined" => Event::Joined {
                room: text(v, "room"),
                peer: peer_of(v, "peer")?,
                peers: v["peers"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|p| Some((peer_of(p, "peer").ok()?, text(p, "name"))))
                            .collect()
                    })
                    .unwrap_or_default(),
            },
            "peer_joined" => Event::PeerJoined {
                peer: peer_of(v, "peer")?,
                name: text(v, "name"),
            },
            "peer_left" => Event::PeerLeft {
                peer: peer_of(v, "peer")?,
            },
            "message" => Event::Message {
                from: peer_of(v, "from")?,
                channel: v["channel"].as_str().map(str::to_string),
                payload: Payload::take(v)?,
            },
            "left" => Event::Left {
                reason: text(v, "reason"),
            },
            "error" => Event::Error {
                op: text(v, "op"),
                code: v["code"].as_str().unwrap_or("failed").to_string(),
                message: text(v, "message"),
            },
            other => return Err(format!("'{other}' is not a room event")),
        })
    }

    pub fn to_json(&self) -> Value {
        match self {
            Event::Joined { room, peer, peers } => json!({
                "type": "joined", "room": room, "peer": peer,
                "peers": peers.iter().map(|(p, n)| json!({ "peer": p, "name": n })).collect::<Vec<_>>(),
            }),
            Event::PeerJoined { peer, name } => {
                json!({ "type": "peer_joined", "peer": peer, "name": name })
            }
            Event::PeerLeft { peer } => json!({ "type": "peer_left", "peer": peer }),
            Event::Message {
                from,
                channel,
                payload,
            } => {
                let mut v = json!({ "type": "message", "from": from });
                if let Some(c) = channel {
                    v["channel"] = json!(c);
                }
                payload.put(&mut v);
                v
            }
            Event::Left { reason } => json!({ "type": "left", "reason": reason }),
            Event::Error { op, code, message } => {
                json!({ "type": "error", "op": op, "code": code, "message": message })
            }
        }
    }
}

/// An op's payload (`data` or `bytes`), for a host or the relay.
pub fn payload_of(op: &Value) -> Result<Payload, String> {
    Payload::take(op)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_round_trip_and_bytes_cross_as_base64() {
        let events = [
            Event::Joined {
                room: "arena-1".into(),
                peer: 1,
                peers: vec![(0, "a".into()), (1, "b".into())],
            },
            Event::PeerJoined {
                peer: 2,
                name: "c".into(),
            },
            Event::PeerLeft { peer: 0 },
            Event::Message {
                from: 1,
                channel: None,
                payload: Payload::Bytes(vec![0, 255, 7]),
            },
            Event::Message {
                from: 1,
                channel: Some(LOCKSTEP_CHANNEL.into()),
                payload: Payload::Json(json!({ "k": "input" })),
            },
            Event::Left {
                reason: "closed".into(),
            },
            Event::Error {
                op: "join".into(),
                code: "full".into(),
                message: "the room is full".into(),
            },
        ];
        for e in events {
            assert_eq!(Event::parse(&e.to_json()).unwrap(), e);
        }
        let op = send(Some(3), None, &Payload::Bytes(b"hi".to_vec()));
        assert_eq!(op["bytes"], json!("aGk="));
        assert_eq!(payload_of(&op).unwrap(), Payload::Bytes(b"hi".to_vec()));
        assert!(Event::parse(&json!({ "type": "nope" })).is_err());
        assert!(valid_room("arena_2.b-1") && !valid_room("a b") && !valid_room(""));
    }
}
