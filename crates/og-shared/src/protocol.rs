//! The shared-world protocol, version 1 (docs/design.md §2.12): binary
//! messages, one per WebSocket binary frame (later one per WebTransport
//! datagram or stream frame). The first byte names the message.
//!
//! An avatar's state crosses quantized ([`AvatarState`]): its cell (whole
//! cells of `cellSize` metres on each axis), its place in the cell (u16 per
//! axis, 2 mm steps at 128 m), its velocity (i16 per axis, 1 cm/s steps)
//! and its yaw (u16), plus up to [`MAX_FIELDS`] bytes of the game's own
//! fields, packed by the engine through reflection and opaque here. A state
//! crosses as a delta: a mask of what differs from a baseline both ends
//! hold, then those parts.
//!
//! Snapshot baselines. The client acknowledges every snapshot it receives
//! (`ack_tick` and the 32 before it as bits, on every proposal). For a
//! snapshot at tick T the server names `base`, the newest acknowledged tick
//! it knows of; an avatar's baseline is its state in the newest snapshot at
//! or before `base`, no older than `base - ACK_WINDOW`, that the client
//! received, that carried it and that is not older than its `enter`. Both
//! ends know that set exactly, so the decode never guesses, also when
//! snapshots are lost (on datagrams). The server keeps only its last few
//! sent states of each avatar; an update it writes over none is marked
//! `WHOLE`. A local avatar id is never reused
//! within the window after its `leave`.
//!
//! Room for later authority: `hello` names the authority the client
//! expects and `welcome` the one the server runs (`validate` in v1: the
//! client simulates, the server checks). A proposal may carry tagged
//! sections after its state; a server skips the tags it does not know, so a
//! simulating server can read an input section without a new version.

use crate::wire::{Reader, WireError, WireResult, Writer};

/// The protocol's first four bytes in `hello`.
pub const MAGIC: [u8; 4] = *b"OGSW";
/// This protocol's version.
pub const PROTOCOL: u16 = 1;
/// The most bytes of a game's own avatar fields.
pub const MAX_FIELDS: usize = 32;
/// The most bytes of one event.
pub const MAX_EVENT: usize = 1024;
/// The most bytes of a display name, a player id, a world name.
pub const MAX_NAME: usize = 128;
/// The most bytes of a token.
pub const MAX_TOKEN: usize = 8192;
/// The most bytes of one section of a proposal.
pub const MAX_SECTION: usize = 1024;
/// How many snapshots back an acknowledgement reaches, and so a baseline.
pub const ACK_WINDOW: u32 = 32;
/// The largest message the server reads.
pub const MAX_MESSAGE: usize = 16 * 1024;

/// Who decides an avatar's movement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Authority {
    /// The client simulates and proposes; the server checks and corrects (v1).
    Validate = 0,
    /// The server simulates from the client's input (a later server).
    Simulate = 1,
}

impl Authority {
    fn from_u8(v: u8) -> WireResult<Authority> {
        match v {
            0 => Ok(Authority::Validate),
            1 => Ok(Authority::Simulate),
            _ => Err(WireError("an unknown authority")),
        }
    }
}

/// Why a server refused a client.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Unauthorized = 1,
    Full = 2,
    Version = 3,
    Shape = 4,
    Draining = 5,
    Invalid = 6,
    NotFound = 7,
}

impl Refusal {
    pub fn code(self) -> &'static str {
        match self {
            Refusal::Unauthorized => "unauthorized",
            Refusal::Full => "full",
            Refusal::Version => "version",
            Refusal::Shape => "shape",
            Refusal::Draining => "draining",
            Refusal::Invalid => "invalid",
            Refusal::NotFound => "not_found",
        }
    }

    fn from_u8(v: u8) -> WireResult<Refusal> {
        Ok(match v {
            1 => Refusal::Unauthorized,
            2 => Refusal::Full,
            3 => Refusal::Version,
            4 => Refusal::Shape,
            5 => Refusal::Draining,
            6 => Refusal::Invalid,
            7 => Refusal::NotFound,
            _ => return Err(WireError("an unknown refusal")),
        })
    }
}

/// An avatar's state as it crosses: quantized, so both ends hold exactly
/// the same values and compare them exactly.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AvatarState {
    /// The cell, whole cells of the cell size on x, y and z.
    pub cell: [i32; 3],
    /// The place in the cell, in 1/65536 of the cell size.
    pub pos: [u16; 3],
    /// The velocity in cm/s.
    pub vel: [i16; 3],
    /// The yaw, in 1/65536 of a turn.
    pub yaw: u16,
    /// The game's own fields, `fields_len` bytes of them.
    pub fields: [u8; MAX_FIELDS],
    pub fields_len: u8,
}

/// The parts of a state a delta carries.
pub mod mask {
    pub const CELL: u8 = 1;
    pub const POS: u8 = 2;
    pub const VEL: u8 = 4;
    pub const YAW: u8 = 8;
    pub const FIELDS: u8 = 16;
    /// The state parts.
    pub const STATE: u8 = CELL | POS | VEL | YAW | FIELDS;
    /// An update: written over no baseline (every part absolute), as when
    /// the server holds none of the states the client acknowledged.
    pub const WHOLE: u8 = 32;
    /// A proposal: tagged sections follow the state.
    pub const SECTIONS: u8 = 64;
    /// A proposal: a teleport. An update: the avatar jumped (do not
    /// interpolate across it).
    pub const JUMP: u8 = 128;
}

impl AvatarState {
    /// A state from world units: position in metres (f64, any distance),
    /// velocity in m/s, yaw in radians.
    pub fn from_world(
        cell_size: f64,
        position: [f64; 3],
        velocity: [f32; 3],
        yaw: f32,
    ) -> AvatarState {
        let mut s = AvatarState::default();
        for a in 0..3 {
            let c = (position[a] / cell_size).floor();
            let mut q = ((position[a] / cell_size - c) * 65536.0).round();
            let mut c = c as i64;
            if q >= 65536.0 {
                q = 0.0;
                c += 1;
            }
            s.cell[a] = c.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
            s.pos[a] = q as u16;
            s.vel[a] = (velocity[a] * 100.0).round().clamp(-32768.0, 32767.0) as i16;
        }
        let turn = (yaw / std::f32::consts::TAU).rem_euclid(1.0);
        s.yaw = ((turn * 65536.0).round() as u32 & 0xffff) as u16;
        s
    }

    /// The position in metres.
    pub fn position(&self, cell_size: f64) -> [f64; 3] {
        std::array::from_fn(|a| {
            (f64::from(self.cell[a]) + f64::from(self.pos[a]) / 65536.0) * cell_size
        })
    }

    /// The velocity in m/s.
    pub fn velocity(&self) -> [f32; 3] {
        std::array::from_fn(|a| f32::from(self.vel[a]) / 100.0)
    }

    /// The yaw in radians, 0 to 2π.
    pub fn yaw(&self) -> f32 {
        f32::from(self.yaw) / 65536.0 * std::f32::consts::TAU
    }

    pub fn fields(&self) -> &[u8] {
        &self.fields[..usize::from(self.fields_len)]
    }

    pub fn set_fields(&mut self, b: &[u8]) {
        let n = b.len().min(MAX_FIELDS);
        self.fields = [0; MAX_FIELDS];
        self.fields[..n].copy_from_slice(&b[..n]);
        self.fields_len = n as u8;
    }

    /// The parts that differ from `base` (every part without one).
    pub fn diff(&self, base: Option<&AvatarState>) -> u8 {
        let Some(b) = base else {
            return mask::STATE;
        };
        let mut m = 0;
        if self.cell != b.cell {
            m |= mask::CELL;
        }
        if self.pos != b.pos {
            m |= mask::POS;
        }
        if self.vel != b.vel {
            m |= mask::VEL;
        }
        if self.yaw != b.yaw {
            m |= mask::YAW;
        }
        if self.fields() != b.fields() {
            m |= mask::FIELDS;
        }
        m
    }

    /// Write the parts in `m` (its state bits; the mask itself is the caller's).
    pub fn write_parts(&self, m: u8, base: Option<&AvatarState>, w: &mut Writer) {
        if m & mask::CELL != 0 {
            let b = base.map_or([0; 3], |b| b.cell);
            for a in 0..3 {
                w.zigzag(i64::from(self.cell[a]) - i64::from(b[a]));
            }
        }
        if m & mask::POS != 0 {
            for p in self.pos {
                w.u16(p);
            }
        }
        if m & mask::VEL != 0 {
            for v in self.vel {
                w.i16(v);
            }
        }
        if m & mask::YAW != 0 {
            w.u16(self.yaw);
        }
        if m & mask::FIELDS != 0 {
            w.bytes(self.fields());
        }
    }

    /// Read the parts in `m` over `base` (the zero state without one).
    pub fn read_parts(
        m: u8,
        base: Option<&AvatarState>,
        r: &mut Reader,
    ) -> WireResult<AvatarState> {
        let mut s = base.copied().unwrap_or_default();
        if base.is_none() && m & mask::STATE != mask::STATE {
            return Err(WireError("a state without a baseline is not whole"));
        }
        if m & mask::CELL != 0 {
            for c in &mut s.cell {
                let v = i64::from(*c) + r.zigzag()?;
                *c = i32::try_from(v).map_err(|_| WireError("a cell is out of range"))?;
            }
        }
        if m & mask::POS != 0 {
            for p in &mut s.pos {
                *p = r.u16()?;
            }
        }
        if m & mask::VEL != 0 {
            for v in &mut s.vel {
                *v = r.i16()?;
            }
        }
        if m & mask::YAW != 0 {
            s.yaw = r.u16()?;
        }
        if m & mask::FIELDS != 0 {
            let f = r.bytes(MAX_FIELDS)?;
            s.set_fields(f);
        }
        Ok(s)
    }
}

/// A client's first message.
#[derive(Clone, Debug, PartialEq)]
pub struct Hello {
    pub version: u16,
    /// The provider's token, opaque to the engine.
    pub token: String,
    /// The world to join.
    pub world: String,
    /// The fingerprint of the game's avatar shape (its fields' layout).
    pub shape: u64,
    /// The bytes of the game's fields every state carries.
    pub fields_len: u8,
    /// The authority the client is built for.
    pub authority: Authority,
    /// The proposals a second the client sends.
    pub send_hz: u8,
    /// The avatar kind (its movement rules), when the token names none.
    pub kind: u32,
    /// The session to resume after a lost connection.
    pub resume: Option<u64>,
}

/// One proposal of the client's own avatar.
#[derive(Clone, Debug, PartialEq)]
pub struct Propose {
    /// This proposal's number, from 1.
    pub seq: u32,
    /// The proposal its state is a delta over (0: none, the state is whole).
    pub base: u32,
    /// The newest snapshot tick received, and the 32 before it as bits
    /// (bit i: tick `ack_tick - 1 - i`).
    pub ack_tick: u32,
    pub ack_bits: u32,
    pub teleport: bool,
    pub state: AvatarState,
    /// Tagged sections, for later authorities; a v1 server keeps none.
    pub sections: Vec<(u8, Vec<u8>)>,
}

/// A message from a client.
#[derive(Clone, Debug, PartialEq)]
pub enum ClientMsg {
    Hello(Hello),
    Propose(Propose),
    Event { channel: u8, data: Vec<u8> },
    Ping { time: u32 },
    Leave,
}

const C_HELLO: u8 = 1;
const C_PROPOSE: u8 = 2;
const C_EVENT: u8 = 3;
const C_PING: u8 = 4;
const C_LEAVE: u8 = 5;

impl ClientMsg {
    /// Encode a proposal over `base` (the proposal numbered `p.base`).
    pub fn encode_propose(p: &Propose, base: Option<&AvatarState>, w: &mut Writer) {
        w.u8(C_PROPOSE);
        w.varint(u64::from(p.seq));
        w.varint(u64::from(if p.base == 0 {
            0
        } else {
            p.seq.wrapping_sub(p.base)
        }));
        w.varint(u64::from(p.ack_tick));
        w.u32(p.ack_bits);
        let base = if p.base == 0 { None } else { base };
        let mut m = p.state.diff(base);
        if !p.sections.is_empty() {
            m |= mask::SECTIONS;
        }
        if p.teleport {
            m |= mask::JUMP;
        }
        w.u8(m);
        p.state.write_parts(m, base, w);
        if !p.sections.is_empty() {
            w.varint(p.sections.len() as u64);
            for (tag, b) in &p.sections {
                w.u8(*tag);
                w.bytes(b);
            }
        }
    }

    pub fn encode(&self, w: &mut Writer) {
        match self {
            ClientMsg::Hello(h) => {
                w.u8(C_HELLO);
                w.buf.extend_from_slice(&MAGIC);
                w.u16(h.version);
                w.str(&h.token);
                w.str(&h.world);
                w.u64(h.shape);
                w.u8(h.fields_len);
                w.u8(h.authority as u8);
                w.u8(h.send_hz);
                w.varint(u64::from(h.kind));
                match h.resume {
                    Some(s) => {
                        w.u8(1);
                        w.u64(s);
                    }
                    None => w.u8(0),
                }
            }
            ClientMsg::Propose(p) => ClientMsg::encode_propose(p, None, w),
            ClientMsg::Event { channel, data } => {
                w.u8(C_EVENT);
                w.u8(*channel);
                w.bytes(data);
            }
            ClientMsg::Ping { time } => {
                w.u8(C_PING);
                w.u32(*time);
            }
            ClientMsg::Leave => w.u8(C_LEAVE),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = Writer::new();
        self.encode(&mut w);
        w.buf
    }

    /// Decode a message; a proposal's state is resolved through `base`,
    /// which maps a proposal number to the state it carried.
    pub fn decode(b: &[u8], base: impl Fn(u32) -> Option<AvatarState>) -> WireResult<ClientMsg> {
        let mut r = Reader::new(b);
        let msg = match r.u8()? {
            C_HELLO => {
                let magic: [u8; 4] = [r.u8()?, r.u8()?, r.u8()?, r.u8()?];
                if magic != MAGIC {
                    return Err(WireError("not a shared-world hello"));
                }
                let version = r.u16()?;
                if version != PROTOCOL {
                    // The rest may be another version's layout: stop here.
                    return Ok(ClientMsg::Hello(Hello {
                        version,
                        token: String::new(),
                        world: String::new(),
                        shape: 0,
                        fields_len: 0,
                        authority: Authority::Validate,
                        send_hz: 0,
                        kind: 0,
                        resume: None,
                    }));
                }
                ClientMsg::Hello(Hello {
                    version,
                    token: r.str(MAX_TOKEN)?.to_string(),
                    world: r.str(MAX_NAME)?.to_string(),
                    shape: r.u64()?,
                    fields_len: r.u8()?,
                    authority: Authority::from_u8(r.u8()?)?,
                    send_hz: r.u8()?,
                    kind: r.varint_u32()?,
                    resume: match r.u8()? {
                        0 => None,
                        _ => Some(r.u64()?),
                    },
                })
            }
            C_PROPOSE => {
                let seq = r.varint_u32()?;
                let back = r.varint_u32()?;
                let base_seq = if back == 0 { 0 } else { seq.wrapping_sub(back) };
                let ack_tick = r.varint_u32()?;
                let ack_bits = r.u32()?;
                let m = r.u8()?;
                let base_state = if base_seq == 0 {
                    None
                } else {
                    Some(base(base_seq).ok_or(WireError("a proposal's baseline is unknown"))?)
                };
                let state = AvatarState::read_parts(m, base_state.as_ref(), &mut r)?;
                let mut sections = Vec::new();
                if m & mask::SECTIONS != 0 {
                    let n = r.varint()?;
                    if n > 16 {
                        return Err(WireError("too many sections"));
                    }
                    for _ in 0..n {
                        let tag = r.u8()?;
                        sections.push((tag, r.bytes(MAX_SECTION)?.to_vec()));
                    }
                }
                ClientMsg::Propose(Propose {
                    seq,
                    base: base_seq,
                    ack_tick,
                    ack_bits,
                    teleport: m & mask::JUMP != 0,
                    state,
                    sections,
                })
            }
            C_EVENT => ClientMsg::Event {
                channel: r.u8()?,
                data: r.bytes(MAX_EVENT)?.to_vec(),
            },
            C_PING => ClientMsg::Ping { time: r.u32()? },
            C_LEAVE => ClientMsg::Leave,
            _ => return Err(WireError("an unknown client message")),
        };
        if !r.done() {
            return Err(WireError("bytes after the message"));
        }
        Ok(msg)
    }
}

/// What the server says when it takes a client in.
#[derive(Clone, Debug, PartialEq)]
pub struct Welcome {
    pub session: u64,
    /// The player's opaque id (the token's subject).
    pub player: String,
    pub name: String,
    pub tick: u32,
    pub tick_hz: u8,
    pub cell_size: f32,
    pub max_visible: u16,
    pub max_event: u16,
    pub authority: Authority,
    /// Whether this resumed an earlier session's avatar.
    pub resumed: bool,
}

/// An avatar coming into a client's set: whole, with who it is.
#[derive(Clone, Debug, PartialEq)]
pub struct Enter {
    pub id: u16,
    pub tick: u32,
    pub player: String,
    pub name: String,
    pub kind: u32,
    pub state: AvatarState,
}

/// One avatar in a snapshot, decoded.
#[derive(Clone, Debug, PartialEq)]
pub struct Update {
    pub id: u16,
    pub jump: bool,
    pub state: AvatarState,
}

/// A message from the server; snapshots are decoded by [`crate::view::View`].
#[derive(Clone, Debug, PartialEq)]
pub enum ServerMsg {
    Welcome(Welcome),
    Refused {
        code: Refusal,
        message: String,
    },
    Enter(Enter),
    Leave {
        id: u16,
    },
    Correct {
        seq: u32,
        state: AvatarState,
    },
    Event {
        from: u16,
        channel: u8,
        data: Vec<u8>,
    },
    Moving {
        reason: String,
    },
    Pong {
        time: u32,
        tick: u32,
    },
    /// A snapshot's bytes, for the view to decode against its baselines.
    Snapshot(Vec<u8>),
}

pub const S_WELCOME: u8 = 1;
pub const S_REFUSED: u8 = 2;
pub const S_SNAPSHOT: u8 = 3;
pub const S_ENTER: u8 = 4;
pub const S_LEAVE: u8 = 5;
pub const S_CORRECT: u8 = 6;
pub const S_EVENT: u8 = 7;
pub const S_MOVING: u8 = 8;
pub const S_PONG: u8 = 9;

impl ServerMsg {
    pub fn encode(&self, w: &mut Writer) {
        match self {
            ServerMsg::Welcome(v) => {
                w.u8(S_WELCOME);
                w.u16(PROTOCOL);
                w.u64(v.session);
                w.str(&v.player);
                w.str(&v.name);
                w.varint(u64::from(v.tick));
                w.u8(v.tick_hz);
                w.f32(v.cell_size);
                w.u16(v.max_visible);
                w.u16(v.max_event);
                w.u8(v.authority as u8);
                w.u8(u8::from(v.resumed));
            }
            ServerMsg::Refused { code, message } => {
                w.u8(S_REFUSED);
                w.u8(*code as u8);
                w.str(message);
            }
            ServerMsg::Enter(e) => {
                w.u8(S_ENTER);
                w.varint(u64::from(e.id));
                w.varint(u64::from(e.tick));
                w.str(&e.player);
                w.str(&e.name);
                w.varint(u64::from(e.kind));
                e.state.write_parts(mask::STATE, None, w);
            }
            ServerMsg::Leave { id } => {
                w.u8(S_LEAVE);
                w.varint(u64::from(*id));
            }
            ServerMsg::Correct { seq, state } => {
                w.u8(S_CORRECT);
                w.varint(u64::from(*seq));
                state.write_parts(mask::STATE, None, w);
            }
            ServerMsg::Event {
                from,
                channel,
                data,
            } => {
                w.u8(S_EVENT);
                w.varint(u64::from(*from));
                w.u8(*channel);
                w.bytes(data);
            }
            ServerMsg::Moving { reason } => {
                w.u8(S_MOVING);
                w.str(reason);
            }
            ServerMsg::Pong { time, tick } => {
                w.u8(S_PONG);
                w.u32(*time);
                w.varint(u64::from(*tick));
            }
            ServerMsg::Snapshot(b) => w.buf.extend_from_slice(b),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = Writer::new();
        self.encode(&mut w);
        w.buf
    }

    pub fn decode(b: &[u8]) -> WireResult<ServerMsg> {
        let mut r = Reader::new(b);
        let msg = match r.u8()? {
            S_WELCOME => {
                if r.u16()? != PROTOCOL {
                    return Err(WireError("the server speaks another protocol version"));
                }
                ServerMsg::Welcome(Welcome {
                    session: r.u64()?,
                    player: r.str(MAX_NAME)?.to_string(),
                    name: r.str(MAX_NAME)?.to_string(),
                    tick: r.varint_u32()?,
                    tick_hz: r.u8()?,
                    cell_size: r.f32()?,
                    max_visible: r.u16()?,
                    max_event: r.u16()?,
                    authority: Authority::from_u8(r.u8()?)?,
                    resumed: r.u8()? != 0,
                })
            }
            S_REFUSED => ServerMsg::Refused {
                code: Refusal::from_u8(r.u8()?)?,
                message: r.str(1024)?.to_string(),
            },
            S_SNAPSHOT => return Ok(ServerMsg::Snapshot(b.to_vec())),
            S_ENTER => ServerMsg::Enter(Enter {
                id: id(r.varint()?)?,
                tick: r.varint_u32()?,
                player: r.str(MAX_NAME)?.to_string(),
                name: r.str(MAX_NAME)?.to_string(),
                kind: r.varint_u32()?,
                state: AvatarState::read_parts(mask::STATE, None, &mut r)?,
            }),
            S_LEAVE => ServerMsg::Leave {
                id: id(r.varint()?)?,
            },
            S_CORRECT => ServerMsg::Correct {
                seq: r.varint_u32()?,
                state: AvatarState::read_parts(mask::STATE, None, &mut r)?,
            },
            S_EVENT => ServerMsg::Event {
                from: id(r.varint()?)?,
                channel: r.u8()?,
                data: r.bytes(MAX_EVENT)?.to_vec(),
            },
            S_MOVING => ServerMsg::Moving {
                reason: r.str(1024)?.to_string(),
            },
            S_PONG => ServerMsg::Pong {
                time: r.u32()?,
                tick: r.varint_u32()?,
            },
            _ => return Err(WireError("an unknown server message")),
        };
        if !r.done() {
            return Err(WireError("bytes after the message"));
        }
        Ok(msg)
    }
}

fn id(v: u64) -> WireResult<u16> {
    u16::try_from(v).map_err(|_| WireError("an avatar id past 16 bits"))
}

/// A snapshot's header: what precedes its updates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapshotHead {
    pub tick: u32,
    /// The acknowledged tick the baselines come from; `None`: every update is whole.
    pub base: Option<u32>,
    /// The newest proposal the server has.
    pub ack_seq: u32,
    /// Avatars in the interest area not in this client's set.
    pub others: u32,
}

impl SnapshotHead {
    pub fn write(&self, count: usize, w: &mut Writer) {
        w.u8(S_SNAPSHOT);
        w.varint(u64::from(self.tick));
        w.varint(u64::from(self.base.map_or(0, |b| self.tick - b)));
        w.varint(u64::from(self.ack_seq));
        w.varint(u64::from(self.others));
        w.varint(count as u64);
    }

    /// The header and the count of updates after it.
    pub fn read(r: &mut Reader) -> WireResult<(SnapshotHead, usize)> {
        if r.u8()? != S_SNAPSHOT {
            return Err(WireError("not a snapshot"));
        }
        let tick = r.varint_u32()?;
        let back = r.varint_u32()?;
        let base = match back {
            0 => None,
            b if b <= ACK_WINDOW && b <= tick => Some(tick - b),
            _ => return Err(WireError("a snapshot's base is outside the window")),
        };
        let head = SnapshotHead {
            tick,
            base,
            ack_seq: r.varint_u32()?,
            others: r.varint_u32()?,
        };
        let count = r.varint()?;
        if count > 4096 {
            return Err(WireError("too many updates"));
        }
        Ok((head, count as usize))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walker() -> AvatarState {
        let mut s = AvatarState::from_world(128.0, [1000.25, 3.5, -77.125], [4.0, 0.0, -1.5], 1.0);
        s.set_fields(&[1, 7]);
        s
    }

    #[test]
    fn quantization_is_two_millimetres_at_any_distance() {
        for x in [
            0.0,
            -0.0004,
            127.999,
            128.0,
            -128.0,
            1.0e7 + 0.37,
            -3.3e6 - 0.001,
        ] {
            let s = AvatarState::from_world(128.0, [x, -x, x * 0.5], [0.0; 3], 0.0);
            let p = s.position(128.0);
            assert!(
                (p[0] - x).abs() <= 128.0 / 65536.0 / 2.0 + 1e-9,
                "{x} -> {}",
                p[0]
            );
            assert!((p[1] + x).abs() <= 128.0 / 65536.0 / 2.0 + 1e-9);
        }
        // A position that rounds up to the next cell lands at its start, not at 65536.
        let s = AvatarState::from_world(128.0, [128.0 - 1e-6, 0.0, 0.0], [0.0; 3], 0.0);
        assert_eq!((s.cell[0], s.pos[0]), (1, 0));
        let v = AvatarState::from_world(128.0, [0.0; 3], [60.0, -327.0, 1000.0], -0.5).velocity();
        assert_eq!(v, [60.0, -327.0, 327.67]);
    }

    #[test]
    fn a_delta_carries_only_what_changed_and_decodes_to_the_new_state() {
        let a = walker();
        let mut b = a;
        b.pos[0] = b.pos[0].wrapping_add(250);
        b.yaw = b.yaw.wrapping_add(9);
        let m = b.diff(Some(&a));
        assert_eq!(m, mask::POS | mask::YAW);
        let mut w = Writer::new();
        b.write_parts(m, Some(&a), &mut w);
        assert_eq!(w.len(), 8);
        let back = AvatarState::read_parts(m, Some(&a), &mut Reader::new(&w.buf)).unwrap();
        assert_eq!(back, b);
        // Crossing into the next cell sends the cell's change, not the whole cell.
        let mut c = b;
        c.cell[2] -= 1;
        let mut w = Writer::new();
        c.write_parts(c.diff(Some(&b)), Some(&b), &mut w);
        assert_eq!(w.len(), 3);
    }

    #[test]
    fn a_partial_state_without_a_baseline_is_refused() {
        let mut w = Writer::new();
        walker().write_parts(mask::POS, None, &mut w);
        assert!(AvatarState::read_parts(mask::POS, None, &mut Reader::new(&w.buf)).is_err());
    }

    #[test]
    fn a_proposal_round_trips_over_its_baseline_and_skips_nothing_of_its_sections() {
        let a = walker();
        let mut b = a;
        b.vel[0] = -400;
        let p = Propose {
            seq: 300,
            base: 297,
            ack_tick: 1_000_000,
            ack_bits: 0xffff_fffe,
            teleport: true,
            state: b,
            sections: vec![(9, vec![1, 2, 3])],
        };
        let mut w = Writer::new();
        ClientMsg::encode_propose(&p, Some(&a), &mut w);
        let back = ClientMsg::decode(&w.buf, |s| (s == 297).then_some(a)).unwrap();
        assert_eq!(back, ClientMsg::Propose(p.clone()));
        // A baseline the server no longer holds is an error, not a wrong state.
        assert!(ClientMsg::decode(&w.buf, |_| None).is_err());
    }

    #[test]
    fn every_message_round_trips() {
        let hello = ClientMsg::Hello(Hello {
            version: PROTOCOL,
            token: "t".repeat(300),
            world: "main".into(),
            shape: 0xdead_beef_0bad_f00d,
            fields_len: 2,
            authority: Authority::Validate,
            send_hz: 20,
            kind: 3,
            resume: Some(42),
        });
        for m in [
            hello,
            ClientMsg::Event {
                channel: 4,
                data: vec![9; 1024],
            },
            ClientMsg::Ping { time: 7 },
            ClientMsg::Leave,
        ] {
            assert_eq!(ClientMsg::decode(&m.to_bytes(), |_| None).unwrap(), m);
        }
        let s = walker();
        for m in [
            ServerMsg::Welcome(Welcome {
                session: 9,
                player: "p_1".into(),
                name: "Ann".into(),
                tick: 77,
                tick_hz: 20,
                cell_size: 128.0,
                max_visible: 64,
                max_event: 1024,
                authority: Authority::Validate,
                resumed: true,
            }),
            ServerMsg::Refused {
                code: Refusal::Shape,
                message: "no".into(),
            },
            ServerMsg::Enter(Enter {
                id: 300,
                tick: 5,
                player: "p".into(),
                name: "n".into(),
                kind: 2,
                state: s,
            }),
            ServerMsg::Leave { id: 3 },
            ServerMsg::Correct { seq: 12, state: s },
            ServerMsg::Event {
                from: 1,
                channel: 2,
                data: vec![3],
            },
            ServerMsg::Moving {
                reason: "drain".into(),
            },
            ServerMsg::Pong { time: 1, tick: 2 },
        ] {
            assert_eq!(ServerMsg::decode(&m.to_bytes()).unwrap(), m);
        }
        assert!(ClientMsg::decode(&[C_EVENT, 1, 0x81, 0x08], |_| None).is_err());
    }

    #[test]
    fn a_hello_of_another_version_reads_as_that_version() {
        let mut w = Writer::new();
        w.u8(C_HELLO);
        w.buf.extend_from_slice(&MAGIC);
        w.u16(2);
        w.buf.extend_from_slice(b"anything a later version writes");
        let ClientMsg::Hello(h) = ClientMsg::decode(&w.buf[..7], |_| None).unwrap() else {
            panic!()
        };
        assert_eq!(h.version, 2);
    }
}
