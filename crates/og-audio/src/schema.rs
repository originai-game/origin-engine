//! The reflected records audio adds to a world. The field order, names and
//! types are the layout (`og_core::reflect`); `crates/og-audio/README.md`
//! lists them for the IDL (`crates/og-api/api.ogt`).

use og_core::glam::{Quat, Vec3};
use og_core::Reflect;

/// The buses a source can play on. `Master` is also the gain every bus
/// passes through.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bus {
    Master = 0,
    Music = 1,
    Sfx = 2,
    Ui = 3,
}

/// How many buses there are.
pub const BUS_COUNT: usize = 4;

impl Bus {
    /// The bus of a stored index; an index past the last bus is `Sfx`.
    pub fn from_index(i: u32) -> Bus {
        match i {
            0 => Bus::Master,
            1 => Bus::Music,
            3 => Bus::Ui,
            _ => Bus::Sfx,
        }
    }
}

/// A sound an entity plays. Level-triggered: a source with `playing` true
/// plays; setting `playing` to false stops it; a one-shot that reaches its
/// end sets `playing` back to false and emits `og.SoundEnded`.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.AudioSource")]
pub struct AudioSource {
    /// The sound to play (a Sound handle); 0 plays nothing. A handle not yet in the bank waits and starts on the first tick it is there.
    pub sound: u64,
    /// The bus it plays on.
    #[reflect(variants = ["master", "music", "sfx", "ui"], default = 2)]
    pub bus: u32,
    /// Linear gain.
    #[reflect(min = 0, max = 4, default = 1, tunable)]
    pub volume: f32,
    /// Playback rate: 2 is an octave up and half as long.
    #[reflect(min = 0.0625, max = 16, default = 1, tunable)]
    pub pitch: f32,
    /// Stereo position of a non-spatial source, -1 left to 1 right.
    #[reflect(min = -1, max = 1, tunable)]
    pub pan: f32,
    /// Loop between loopStart and loopEnd, seamlessly.
    #[reflect(name = "loop")]
    pub looping: bool,
    /// Where a loop restarts, in seconds of the sound.
    #[reflect(name = "loopStart", min = 0, unit = "s")]
    pub loop_start: f32,
    /// Where a loop wraps, in seconds of the sound; 0 is the sound's end.
    #[reflect(name = "loopEnd", min = 0, unit = "s")]
    pub loop_end: f32,
    /// Attenuate and pan by the entity's Transform relative to the AudioListener.
    pub spatial: bool,
    /// Distance below which a spatial source plays at full volume.
    #[reflect(name = "refDistance", min = 0, default = 1, unit = "m", tunable)]
    pub ref_distance: f32,
    /// Distance past which a spatial source gets no quieter.
    #[reflect(name = "maxDistance", min = 0, default = 100, unit = "m", tunable)]
    pub max_distance: f32,
    /// How fast volume falls with distance (the inverse model, as Web Audio's PannerNode).
    #[reflect(name = "rolloffFactor", min = 0, default = 1, tunable)]
    pub rolloff_factor: f32,
    /// Plays while true (inserting a source plays it); the engine sets it false when a one-shot ends.
    #[reflect(default = 1)]
    pub playing: bool,
}

impl Default for AudioSource {
    fn default() -> Self {
        AudioSource {
            sound: 0,
            bus: Bus::Sfx as u32,
            volume: 1.0,
            pitch: 1.0,
            pan: 0.0,
            looping: false,
            loop_start: 0.0,
            loop_end: 0.0,
            spatial: false,
            ref_distance: 1.0,
            max_distance: 100.0,
            rolloff_factor: 1.0,
            playing: true,
        }
    }
}

impl AudioSource {
    /// A source playing `sound` once on the sfx bus.
    pub fn new(sound: u64) -> AudioSource {
        AudioSource {
            sound,
            ..AudioSource::default()
        }
    }
}

/// Where spatial sources are heard from: the entity's Transform. The first
/// listener in query order is used; without one, the origin looking down -Z.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.AudioListener")]
pub struct AudioListener {
    /// A gain on every spatial source this listener hears.
    #[reflect(min = 0, max = 4, default = 1, tunable)]
    pub volume: f32,
}

impl Default for AudioListener {
    fn default() -> Self {
        AudioListener { volume: 1.0 }
    }
}

/// Bus volumes. `master` scales every bus.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Audio")]
pub struct Audio {
    /// Gain of the master bus, applied to every sound.
    #[reflect(min = 0, max = 2, default = 1, tunable)]
    pub master: f32,
    /// Gain of the music bus.
    #[reflect(min = 0, max = 2, default = 1, tunable)]
    pub music: f32,
    /// Gain of the sfx bus.
    #[reflect(min = 0, max = 2, default = 1, tunable)]
    pub sfx: f32,
    /// Gain of the ui bus.
    #[reflect(min = 0, max = 2, default = 1, tunable)]
    pub ui: f32,
}

impl Default for Audio {
    fn default() -> Self {
        Audio {
            master: 1.0,
            music: 1.0,
            sfx: 1.0,
            ui: 1.0,
        }
    }
}

impl Audio {
    pub(crate) fn gains(&self) -> [f32; BUS_COUNT] {
        [self.master, self.music, self.sfx, self.ui]
    }
}

/// Emitted in the tick a source starts (readers see it the tick after).
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.SoundStarted")]
pub struct SoundStarted {
    /// The entity whose AudioSource started.
    pub entity: u64,
    /// The sound.
    pub sound: u64,
    /// The voice id in the command stream.
    pub voice: u32,
}

/// Why a sound ended (`og.SoundEnded.reason`).
pub mod end_reason {
    /// A one-shot played to its end.
    pub const FINISHED: u32 = 0;
    /// `playing` was set to false.
    pub const STOPPED: u32 = 1;
    /// The AudioSource was removed from a living entity.
    pub const REMOVED: u32 = 2;
    /// `sound` was changed to another sound while playing.
    pub const REPLACED: u32 = 3;
}

/// Emitted in the tick a sound ends (readers see it the tick after).
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.SoundEnded")]
pub struct SoundEnded {
    /// The entity whose AudioSource ended.
    pub entity: u64,
    /// The sound.
    pub sound: u64,
    /// The voice id in the command stream.
    pub voice: u32,
    /// 0 finished, 1 stopped, 2 removed, 3 replaced.
    pub reason: u32,
}

/// Engine-owned: what the mixer was last told about a playing source, and
/// where the tick side's playhead is. Present while the source plays.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.AudioPlayback")]
pub struct AudioPlayback {
    /// The voice id.
    pub voice: u32,
    /// The sound playing.
    pub sound: u64,
    /// The playhead at the start of the next tick, in frames of the sound.
    pub cursor: f64,
    /// The gain last sent (volume × distance × listener volume).
    pub gain: f32,
    /// The pan last sent.
    pub pan: f32,
    /// The pitch last sent.
    pub pitch: f32,
    /// The bus last sent.
    pub bus: u32,
    /// The flags last sent (bit 0 loop, bit 1 spatial).
    pub flags: u32,
    /// The loop start last sent, in frames.
    #[reflect(name = "loopStart")]
    pub loop_start: u32,
    /// The loop end last sent, in frames (exclusive).
    #[reflect(name = "loopEnd")]
    pub loop_end: u32,
}

/// Engine-owned: the command stream's running hash and counters. Being a
/// resource, it is in every snapshot and so in `world_hash`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.AudioStream")]
pub struct AudioStream {
    /// FNV-1a chained over every tick's encoded commands.
    pub hash: u64,
    /// Commands emitted since tick 0.
    pub commands: u64,
    /// The next voice id.
    #[reflect(name = "nextVoice")]
    pub next_voice: u32,
    /// Voices playing after the last tick.
    pub voices: u32,
    /// Ticks per second, fixed at install.
    #[reflect(name = "tickRate")]
    pub tick_rate: u32,
    /// The bus gains last sent (master, music, sfx, ui); negative before the first tick.
    #[reflect(name = "busSent")]
    pub bus_sent: og_core::glam::Vec4,
}

/// The engine's `og.Transform`, mirrored so the system can read positions.
/// Same name and layout, so it adopts the host's registration.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Transform")]
pub struct Transform {
    pub position: Vec3,
    pub quaternion: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vec3::ZERO,
            quaternion: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn at(position: Vec3) -> Transform {
        Transform {
            position,
            ..Transform::default()
        }
    }
}
