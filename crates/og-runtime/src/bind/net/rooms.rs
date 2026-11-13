//! og.platform's room members.

use super::*;

pub(super) fn can_join(n: &Net) -> Result<(), String> {
    if n.host.is_empty() || n.host == "none" {
        return Err("no rooms here: natively start the game with `og run --room ws://127.0.0.1:<port>/<room>` beside `og relay`; on the web the platform's rooms answer (og.platform.roomInfo's host)".into());
    }
    match n.stage {
        Stage::None => Ok(()),
        Stage::Joining => Err("a room is being joined already: wait for og.RoomJoined".into()),
        Stage::In => Err(format!("already in room '{}': roomLeave first", n.room)),
    }
}

pub fn og_platform_room_create(world: u32, desc: Option<String>) -> Result<(), String> {
    let d: crate::host::wire::RoomDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
    let name = d.name;
    if !name.is_empty() && !protocol::valid_room(&name) {
        return Err(format!(
            "'{name}' is not a room name: at most 64 of letters, digits, '.', '_' and '-'"
        ));
    }
    let capacity = d.capacity.clamp(1, protocol::MAX_CAPACITY);
    with(world, |n| match can_join(n) {
        Ok(()) => {
            n.stage = Stage::Joining;
            n.outbox.push(protocol::create(&name, capacity));
            Ok(())
        }
        Err(e) => Err(e),
    })
}

pub fn og_platform_quick_play(world: u32, desc: Option<String>) -> Result<(), String> {
    let d: crate::host::wire::QuickPlayDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
    let (mode, name) = (d.mode, d.name);
    if !mode.is_empty() && (!protocol::valid_room(&mode) || mode.len() > 32) {
        return Err(format!(
            "'{mode}' is not a quick play mode: at most 32 of letters, digits, '.', '_' and '-'"
        ));
    }
    if !name.is_empty() && !protocol::valid_room(&name) {
        return Err(format!(
            "'{name}' is not a player name: at most 64 of letters, digits, '.', '_' and '-'"
        ));
    }
    let players = d.players.clamp(1, protocol::MAX_CAPACITY);
    let timeout = d.timeout_ms.min(protocol::QUICK_TIMEOUT_MAX_MS);
    with(world, |n| match can_join(n) {
        Ok(()) => {
            n.stage = Stage::Joining;
            n.outbox
                .push(protocol::quick(&mode, players, timeout, &name));
            Ok(())
        }
        Err(e) => Err(e),
    })
}

pub fn og_platform_room_join(world: u32, room: String) -> Result<(), String> {
    if !protocol::valid_room(&room) {
        return Err(format!(
            "'{room}' is not a room id: 1 to 64 of letters, digits, '.', '_' and '-'"
        ));
    }
    with(world, |n| match can_join(n) {
        Ok(()) => {
            n.stage = Stage::Joining;
            n.outbox.push(protocol::join(&room, ""));
            Ok(())
        }
        Err(e) => Err(e),
    })
}

pub fn og_platform_room_leave(world: u32) -> Result<(), String> {
    with(world, |n| {
        if n.stage == Stage::None {
            return Err("in no room".into());
        }
        n.outbox.push(protocol::leave());
        Ok(())
    })
}

pub fn og_platform_room_send(world: u32, data: String, to: Option<u32>) -> Result<(), String> {
    let v = parse(Some(&data))?;
    let payload = match v.get(BYTES_KEY).and_then(Value::as_str) {
        Some(b) => match og_net::base64::decode(b) {
            Some(bytes) => Payload::Bytes(bytes),
            None => return Err("the message's bytes are not base64".into()),
        },
        None => Payload::Json(v),
    };
    if payload.size() > MAX_MESSAGE {
        return Err(format!(
            "a message is at most {MAX_MESSAGE} bytes encoded; this one is {}",
            payload.size()
        ));
    }
    with(world, |n| {
        if n.stage != Stage::In {
            return Err(
                "in no room: roomJoin or roomCreate first, and wait for og.RoomJoined".into(),
            );
        }
        if let Some(t) = to
            && !n.peers.contains_key(&t)
        {
            return Err(format!("no peer {t} in room '{}'", n.room));
        }
        n.outbox.push(protocol::send(to, None, &payload));
        Ok(())
    })
}

/// The room's peers; none outside a room.
pub(super) fn room_peers(n: &Net) -> Vec<RoomPeer> {
    if n.stage != Stage::In {
        return Vec::new();
    }
    n.peers
        .iter()
        .map(|(p, name)| RoomPeer {
            peer: *p,
            name: name.clone(),
            local: *p == n.peer,
        })
        .collect()
}

pub fn og_platform_room_peers(world: u32) -> Vec<RoomPeer> {
    with(world, |n| room_peers(n))
}

pub fn og_platform_room_info(world: u32) -> RoomInfo {
    with(world, |n| {
        let state = match n.stage {
            Stage::None => "none",
            Stage::Joining => "joining",
            Stage::In => "in",
        };
        let host = if n.host.is_empty() { "none" } else { &n.host };
        let inside = n.stage == Stage::In;
        RoomInfo {
            state: state.to_string(),
            room: if inside {
                n.room.clone()
            } else {
                String::new()
            },
            host: host.to_string(),
            link: n.link.clone(),
            peer: if inside { n.peer } else { 0 },
            peers: room_peers(n),
        }
    })
}

pub fn og_platform_room_message(world: u32, message: u32) -> Option<RoomMessageData> {
    with(world, |n| {
        let m = n.messages.iter().find(|m| m.id == message)?;
        let (data, bytes) = match &m.payload {
            Payload::Json(d) => (d.clone(), None),
            Payload::Bytes(b) => (Value::Null, Some(json!(og_net::base64::encode(b)))),
        };
        Some(RoomMessageData {
            message: m.id,
            from: m.from,
            tick: m.tick.unwrap_or(0) as u32,
            data,
            bytes,
        })
    })
}
