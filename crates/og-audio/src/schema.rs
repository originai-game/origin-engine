//! The reflected records audio adds to a world. The field order, names and
//! types are the layout (`og_core::reflect`); `crates/og-audio/README.md`
//! lists them for the IDL (`crates/og-api/api.ogt`).

use og_core::Reflect;
use og_core::glam::{Quat, Vec3};

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
    /// When more than og.Audio.maxVoices would play, the lowest priority is stopped first (then the quietest, then the oldest).
    #[reflect(min = 0, max = 255, default = 128)]
    pub priority: u32,
    /// A low-pass filter on this source, its cutoff in Hz; 0 is off.
    #[reflect(min = 0, max = 20000, unit = "hz", tunable)]
    pub lowpass: f32,
    /// A spatial source behind a collider (a physics ray to the listener) plays at og.Audio.occlusionGain through a low-pass at og.Audio.occlusionCutoff.
    pub occlude: bool,
    /// A spatial source's low-pass by distance: the cutoff in Hz at maxDistance, from 20 kHz at refDistance (log-linear between); 0 is off.
    #[reflect(name = "distanceCutoff", min = 0, max = 20000, unit = "hz", tunable)]
    pub distance_cutoff: f32,
    /// Starts at this world time (seconds since tick 0), sample-accurately, even inside a tick; 0 or a time past starts at once. og.Music.nextBar gives bar-aligned times.
    #[reflect(name = "startAt", min = 0, unit = "s")]
    pub start_at: f64,
    /// Where in the sound it starts, in seconds of the sound: a resumed phrase continues from where it was, to the sample; past a one-shot's end it plays nothing. A loop wraps into its region.
    #[reflect(name = "startOffset", min = 0, unit = "s")]
    pub start_offset: f64,
    /// Starts, stops and volume changes ramp over this long (crossfades, stems brought in and out); 0 is 5 ms.
    #[reflect(min = 0, max = 30, unit = "s", tunable)]
    pub fade: f32,
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
            priority: 128,
            lowpass: 0.0,
            occlude: false,
            distance_cutoff: 0.0,
            start_at: 0.0,
            start_offset: 0.0,
            fade: 0.0,
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
    /// The most sources that play at once; past it the lowest og.AudioSource.priority is stopped (SoundEnded reason SOUND_CULLED).
    #[reflect(name = "maxVoices", min = 1, max = 256, default = 32)]
    pub max_voices: u32,
    /// The gain of an occluded source (og.AudioSource.occlude).
    #[reflect(name = "occlusionGain", min = 0, max = 1, default = 0.35, tunable)]
    pub occlusion_gain: f32,
    /// The low-pass cutoff of an occluded source.
    #[reflect(
        name = "occlusionCutoff",
        min = 20,
        max = 20000,
        default = 900,
        unit = "hz",
        tunable
    )]
    pub occlusion_cutoff: f32,
    /// The shared reverb's room size (the buses' og.AudioBusFx.reverb sends feed it).
    #[reflect(name = "reverbRoom", min = 0, max = 1, default = 0.6, tunable)]
    pub reverb_room: f32,
    /// The shared reverb's damping of high frequencies.
    #[reflect(name = "reverbDamping", min = 0, max = 1, default = 0.4, tunable)]
    pub reverb_damping: f32,
    /// Seconds from a sample's world time to the speaker, as the host reports it between ticks (og.audio.outputLatency); 0 headless.
    #[reflect(name = "outputLatency", min = 0, unit = "s")]
    pub output_latency: f64,
}

impl Default for Audio {
    fn default() -> Self {
        Audio {
            master: 1.0,
            music: 1.0,
            sfx: 1.0,
            ui: 1.0,
            max_voices: 32,
            occlusion_gain: 0.35,
            occlusion_cutoff: 900.0,
            reverb_room: 0.6,
            reverb_damping: 0.4,
            output_latency: 0.0,
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
    /// More sources played than og.Audio.maxVoices, and this one had the lowest priority.
    pub const CULLED: u32 = 4;
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
    /// 0 finished, 1 stopped, 2 removed, 3 replaced, 4 culled.
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
    /// The low-pass cutoff last sent, Hz (0: open).
    pub lowpass: f32,
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
    /// A hash of the bus effects last sent.
    pub fx: u64,
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

/// A bus's insert effects: a low- and high-pass, a compressor, side-chain
/// ducking by another bus and a send to the shared reverb. One entity per
/// bus (the first in query order counts); changes reach the mixer as `fx`
/// commands in the stream.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.AudioBusFx")]
pub struct AudioBusFx {
    /// The bus these effects are on.
    #[reflect(variants = ["master", "music", "sfx", "ui"], default = 1)]
    pub bus: u32,
    /// Low-pass cutoff (12 dB/octave); 0 is off.
    #[reflect(min = 0, max = 20000, unit = "hz", tunable)]
    pub lowpass: f32,
    /// High-pass cutoff (12 dB/octave); 0 is off.
    #[reflect(min = 0, max = 20000, unit = "hz", tunable)]
    pub highpass: f32,
    /// How much of the bus goes to the reverb, 0..1.
    #[reflect(min = 0, max = 1, tunable)]
    pub reverb: f32,
    /// The compressor's threshold; 0 dB is off.
    #[reflect(min = -60, max = 0, unit = "db", tunable)]
    pub threshold: f32,
    /// The compressor's ratio (input dB over the threshold per output dB).
    #[reflect(min = 1, max = 30, default = 4, tunable)]
    pub ratio: f32,
    /// The compressor's attack.
    #[reflect(min = 0, max = 1, default = 0.005, unit = "s", tunable)]
    pub attack: f32,
    /// The compressor's release.
    #[reflect(min = 0, max = 3, default = 0.15, unit = "s", tunable)]
    pub release: f32,
    /// Gain after the compressor.
    #[reflect(min = -24, max = 24, unit = "db", tunable)]
    pub makeup: f32,
    /// Turns this bus down while `duckBus` is loud (music under dialogue or effects).
    pub duck: bool,
    /// The bus that ducks this one.
    #[reflect(name = "duckBus", variants = ["master", "music", "sfx", "ui"], default = 2)]
    pub duck_bus: u32,
    /// How far it is turned down.
    #[reflect(
        name = "duckDepth",
        min = 0,
        max = 60,
        default = 9,
        unit = "db",
        tunable
    )]
    pub duck_depth: f32,
    /// How loud the key bus must be.
    #[reflect(name = "duckThreshold", min = -80, max = 0, default = -36, unit = "db", tunable)]
    pub duck_threshold: f32,
    /// How fast it goes down.
    #[reflect(
        name = "duckAttack",
        min = 0,
        max = 2,
        default = 0.02,
        unit = "s",
        tunable
    )]
    pub duck_attack: f32,
    /// How fast it comes back.
    #[reflect(
        name = "duckRelease",
        min = 0,
        max = 5,
        default = 0.4,
        unit = "s",
        tunable
    )]
    pub duck_release: f32,
}

impl Default for AudioBusFx {
    fn default() -> Self {
        AudioBusFx {
            bus: Bus::Music as u32,
            lowpass: 0.0,
            highpass: 0.0,
            reverb: 0.0,
            threshold: 0.0,
            ratio: 4.0,
            attack: 0.005,
            release: 0.15,
            makeup: 0.0,
            duck: false,
            duck_bus: Bus::Sfx as u32,
            duck_depth: 9.0,
            duck_threshold: -36.0,
            duck_attack: 0.02,
            duck_release: 0.4,
        }
    }
}

/// A musical clock: tempo and metre. The engine's og.music system writes
/// the current bar and beat and the times of the next beat and bar each
/// tick, and sends og.MusicBeat on every beat; a source whose `startAt` is
/// `nextBar` starts on the bar line, to the sample.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Music")]
pub struct Music {
    /// Beats per minute.
    #[reflect(min = 20, max = 400, default = 120, unit = "bpm", tunable)]
    pub bpm: f32,
    /// Beats per bar.
    #[reflect(name = "beatsPerBar", min = 1, max = 16, default = 4)]
    pub beats_per_bar: u32,
    /// The world time of the first bar's first beat, in seconds.
    #[reflect(min = 0, unit = "s")]
    pub start: f64,
    /// The bar the tick is in, from 1 (0 before start; written by og.music).
    pub bar: u32,
    /// The beat in the bar, from 1 (written by og.music).
    pub beat: u32,
    /// The world time of the next beat (written by og.music).
    #[reflect(name = "nextBeat", unit = "s")]
    pub next_beat: f64,
    /// The world time of the next bar line (written by og.music).
    #[reflect(name = "nextBar", unit = "s")]
    pub next_bar: f64,
}

impl Default for Music {
    fn default() -> Self {
        Music {
            bpm: 120.0,
            beats_per_bar: 4,
            start: 0.0,
            bar: 0,
            beat: 0,
            next_beat: 0.0,
            next_bar: 0.0,
        }
    }
}

/// A beat of an og.Music clock, sent in the tick whose interval holds it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.MusicBeat")]
pub struct MusicBeat {
    /// The entity with the og.Music.
    pub entity: u64,
    /// The bar, from 1.
    pub bar: u32,
    /// The beat in the bar, from 1 (1 is the bar line).
    pub beat: u32,
    /// Its world time in seconds.
    pub time: f64,
}
