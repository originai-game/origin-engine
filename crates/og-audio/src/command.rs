//! The command stream: what the tick tells the mixer, one fixed-size record
//! per command, time-stamped with the tick that produced it.
//!
//! Record (`COMMAND_BYTES` = 56, little-endian, no padding):
//!
//! | offset | type | field | meaning |
//! |---:|---|---|---|
//! | 0 | u8 | `op` | 1 play, 2 stop, 3 params, 4 bus, 5 stop-all |
//! | 1 | u8 | `flags` | bit 0 loop, bit 1 spatial |
//! | 2 | u8 | `bus` | 0 master, 1 music, 2 sfx, 3 ui (the bus a `bus` op sets) |
//! | 3 | u8 | reserved | 0 |
//! | 4 | u32 | `voice` | the voice id (0 for `bus`, `stop-all`) |
//! | 8 | u64 | `tick` | the tick whose start the command takes effect at |
//! | 16 | u64 | `sound` | the sound handle (`play`) |
//! | 24 | f32 | `gain` | voice gain, or the bus gain for `bus` |
//! | 28 | f32 | `pitch` | playback rate |
//! | 32 | f32 | `pan` | -1 left .. 1 right |
//! | 36 | u32 | `loop_start` | loop start, frames of the sound |
//! | 40 | u32 | `loop_end` | loop end (exclusive), frames; 0 = the sound's end |
//! | 44 | u32 | reserved | 0 |
//! | 48 | f64 | `position` | where `play` starts, frames of the sound |

/// What a command does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    /// Start `voice` playing `sound` at `position` with these parameters.
    Play = 1,
    /// Fade `voice` out and drop it.
    Stop = 2,
    /// Change `voice`'s gain, pitch, pan, bus, flags and loop region.
    Params = 3,
    /// Set bus `bus`'s gain.
    Bus = 4,
    /// Stop every voice (a resync's first command; never in the hashed stream).
    StopAll = 5,
}

impl Op {
    fn from_u8(v: u8) -> Option<Op> {
        Some(match v {
            1 => Op::Play,
            2 => Op::Stop,
            3 => Op::Params,
            4 => Op::Bus,
            5 => Op::StopAll,
            _ => return None,
        })
    }
}

/// `flags` bit: the voice loops.
pub const FLAG_LOOP: u8 = 1;
/// `flags` bit: the voice is spatial (informational; gain and pan are already applied).
pub const FLAG_SPATIAL: u8 = 2;

/// Bytes per encoded command.
pub const COMMAND_BYTES: usize = 56;

/// One command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Command {
    pub op: Op,
    pub flags: u8,
    pub bus: u8,
    pub voice: u32,
    pub tick: u64,
    pub sound: u64,
    pub gain: f32,
    pub pitch: f32,
    pub pan: f32,
    pub loop_start: u32,
    pub loop_end: u32,
    pub position: f64,
}

impl Command {
    /// A command with every parameter at its neutral value.
    pub fn new(op: Op, tick: u64) -> Command {
        Command {
            op,
            flags: 0,
            bus: 0,
            voice: 0,
            tick,
            sound: 0,
            gain: 1.0,
            pitch: 1.0,
            pan: 0.0,
            loop_start: 0,
            loop_end: 0,
            position: 0.0,
        }
    }

    pub fn looping(&self) -> bool {
        self.flags & FLAG_LOOP != 0
    }

    /// Appends the 56-byte record.
    pub fn encode_into(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&[self.op as u8, self.flags, self.bus, 0]);
        out.extend_from_slice(&self.voice.to_le_bytes());
        out.extend_from_slice(&self.tick.to_le_bytes());
        out.extend_from_slice(&self.sound.to_le_bytes());
        out.extend_from_slice(&self.gain.to_le_bytes());
        out.extend_from_slice(&self.pitch.to_le_bytes());
        out.extend_from_slice(&self.pan.to_le_bytes());
        out.extend_from_slice(&self.loop_start.to_le_bytes());
        out.extend_from_slice(&self.loop_end.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&self.position.to_le_bytes());
    }

    /// Reads one record; `None` for a short slice or an unknown op.
    pub fn decode(b: &[u8]) -> Option<Command> {
        if b.len() < COMMAND_BYTES {
            return None;
        }
        let u32_at = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        let u64_at = |o: usize| u64::from_le_bytes(b[o..o + 8].try_into().unwrap());
        let f32_at = |o: usize| f32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        Some(Command {
            op: Op::from_u8(b[0])?,
            flags: b[1],
            bus: b[2],
            voice: u32_at(4),
            tick: u64_at(8),
            sound: u64_at(16),
            gain: f32_at(24),
            pitch: f32_at(28),
            pan: f32_at(32),
            loop_start: u32_at(36),
            loop_end: u32_at(40),
            position: f64::from_le_bytes(b[48..56].try_into().unwrap()),
        })
    }
}

/// Encodes commands back to back.
pub fn encode(commands: &[Command]) -> Vec<u8> {
    let mut out = Vec::with_capacity(commands.len() * COMMAND_BYTES);
    for c in commands {
        c.encode_into(&mut out);
    }
    out
}

/// Decodes back-to-back records (stops at the first malformed one).
pub fn decode(bytes: &[u8]) -> Vec<Command> {
    bytes
        .as_chunks::<COMMAND_BYTES>()
        .0
        .iter()
        .map_while(|c| Command::decode(c))
        .collect()
}

/// What one tick hands the sink.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TickPacket {
    /// The tick the commands are stamped with.
    pub tick: u64,
    /// The tick's commands, in emission order. In a resync packet: stop-all,
    /// the four bus gains, then a play per live voice at its playhead.
    pub commands: Vec<Command>,
    /// The voice ids alive after this tick, sorted, when the set changed
    /// (voices not in it are dropped: their entity was despawned).
    pub live: Option<Vec<u32>>,
    /// The commands replace the mixer's state (first packet after attach,
    /// after a restore, or after a gap in ticks).
    pub resync: bool,
}
