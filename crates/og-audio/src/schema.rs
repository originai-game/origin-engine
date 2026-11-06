//! The reflected records audio adds to a world. The field order, names and
//! types are the layout (`og_core::reflect`); `crates/og-audio/README.md`
//! lists them for the IDL (`crates/og-api/api/audio.ogt`).

use og_core::glam::{Quat, Vec3};
use og_core::{List, Reflect};

/// The buses a source can play on. `Master` is also the gain every bus
/// passes through.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bus {
    Master = 0,
    Music = 1,
    Sfx = 2,
    Ui = 3,
}

/// How many built-in buses there are (master, music, sfx, ui).
pub const BUS_COUNT: usize = 4;

/// How many buses the graph holds: the four built-ins and the user buses
/// 4 to 15 (`og.AudioBusNode`).
pub const MAX_BUSES: usize = 16;

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
#[reflect(name = "og.AudioSource", version = 2)]
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
    /// Attenuate and pan by the entity's world transform (its Transform down og.Parent) relative to the AudioListener.
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
    /// Doppler: how much a spatial source's and the listener's motion along the line between them shifts its pitch (their tick velocities, from world positions down og.Parent); 1 is physical, 0 is off. Times og.AudioListener.dopplerFactor, against og.Audio.speedOfSound.
    #[reflect(name = "dopplerFactor", min = 0, max = 10, default = 1, tunable)]
    pub doppler_factor: f32,
    /// Directivity, as three.js PositionalAudio.setDirectionalCone and Web Audio's PannerNode: a spatial source plays at full volume while the listener is within this full angle about the source's +Z; 360 (with coneOuterAngle 360) is omnidirectional.
    #[reflect(
        name = "coneInnerAngle",
        min = 0,
        max = 360,
        default = 360,
        unit = "deg",
        tunable
    )]
    pub cone_inner_angle: f32,
    /// Past this full angle about the source's +Z it plays at coneOuterGain; between the two angles the gain falls linearly.
    #[reflect(
        name = "coneOuterAngle",
        min = 0,
        max = 360,
        default = 360,
        unit = "deg",
        tunable
    )]
    pub cone_outer_angle: f32,
    /// The gain outside coneOuterAngle.
    #[reflect(name = "coneOuterGain", min = 0, max = 1, tunable)]
    pub cone_outer_gain: f32,
    /// Past og.Audio.maxVoices it goes virtual instead of being culled: silent, its playhead running on, and heard again from where it would be once it stands within the cap (a one-shot that ends while virtual ends on its tick).
    pub virtualize: bool,
    /// Set true to start the sound again from the top (its startOffset, its startAt) this tick, playing or not, without touching `playing`; the engine sets it back to false (and `playing` true). The voice it replaces ends with SoundEnded reason SOUND_RETRIGGERED.
    pub retrigger: bool,
    /// A user bus (4 to 15, declared by an og.AudioBusNode) it plays into instead of `bus`; 0 (or a bus no node declares) plays on `bus`.
    #[reflect(name = "userBus", min = 0, max = 15)]
    pub user_bus: u32,
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
            doppler_factor: 1.0,
            cone_inner_angle: 360.0,
            cone_outer_angle: 360.0,
            cone_outer_gain: 0.0,
            virtualize: false,
            retrigger: false,
            user_bus: 0,
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

/// Where spatial sources are heard from: the entity's world transform (its
/// Transform composed down og.Parent, so a listener under a camera rig hears
/// from the rig). The first listener in query order is used; without one,
/// the origin looking down -Z.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.AudioListener")]
pub struct AudioListener {
    /// A gain on every spatial source this listener hears.
    #[reflect(min = 0, max = 4, default = 1, tunable)]
    pub volume: f32,
    /// Scales every spatial source's Doppler shift (og.AudioSource.dopplerFactor); 0 turns Doppler off for this listener.
    #[reflect(name = "dopplerFactor", min = 0, max = 10, default = 1, tunable)]
    pub doppler_factor: f32,
    /// How spatial sources are placed between the ears: equalpower (the stereo panner, for speakers) or hrtf (a spherical-head model for headphones: interaural time and level differences).
    #[reflect(name = "panningModel", variants = ["equalpower", "hrtf"])]
    pub panning_model: u32,
}

/// `og.AudioListener.panningModel` values.
pub mod panning_model {
    /// Web Audio's equal-power panner (the default).
    pub const EQUALPOWER: u32 = 0;
    /// The spherical-head HRTF approximation, for headphones.
    pub const HRTF: u32 = 1;
}

impl Default for AudioListener {
    fn default() -> Self {
        AudioListener {
            volume: 1.0,
            doppler_factor: 1.0,
            panning_model: panning_model::EQUALPOWER,
        }
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
    /// The speed of sound Doppler shifts are computed against (og.AudioSource.dopplerFactor).
    #[reflect(
        name = "speedOfSound",
        min = 1,
        max = 10000,
        default = 343.3,
        unit = "mps",
        tunable
    )]
    pub speed_of_sound: f32,
    /// How a voice is resampled when its pitch or its sound's rate differs from the output's: sinc (a 16-tap Kaiser-windowed sinc) or linear (interpolation, the engine's mix before round 4: cheaper, duller highs and audible images). Unpitched sounds at the output's rate play the same samples with either.
    #[reflect(variants = ["linear", "sinc"], default = 1)]
    pub resampler: u32,
}

/// `og.Audio.resampler` values.
pub mod resampler {
    /// Linear interpolation (the mix before round 4).
    pub const LINEAR: u32 = 0;
    /// The windowed sinc (the default).
    pub const SINC: u32 = 1;
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
            speed_of_sound: 343.3,
            resampler: resampler::SINC,
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
    /// `retrigger` started the sound again; a new voice plays it.
    pub const RETRIGGERED: u32 = 5;
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
    /// 0 finished, 1 stopped, 2 removed, 3 replaced, 4 culled, 5 retriggered.
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
    /// The source's world position in the last tick (its Doppler velocity is the change).
    #[reflect(worldspace = "world")]
    pub position: Vec3,
    /// The direction last sent to the HRTF panner, in the listener's frame (zero: the stereo panner).
    pub direction: Vec3,
    /// Virtual: past the voice cap with og.AudioSource.virtualize, silent in the mixer while its playhead runs on.
    #[reflect(name = "virtual")]
    pub is_virtual: bool,
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
    /// A hash of the bus effects last sent.
    pub fx: u64,
    /// The listener heard from last tick (its entity's bits; 0: none).
    pub listener: u64,
    /// That listener's world position last tick (its Doppler velocity is the change).
    #[reflect(name = "listenerAt", worldspace = "world")]
    pub listener_at: Vec3,
}

/// The engine's `og.Parent`, mirrored so the system can place sources,
/// listeners and zones on child entities at their world transform. Same
/// name and layout, so it adopts the host's registration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.Parent")]
pub struct Parent {
    /// The parent (an entity's bits).
    pub entity: u64,
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
#[reflect(name = "og.AudioBusFx", version = 2)]
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
    /// A user bus (4 to 15, an og.AudioBusNode) these effects are on instead of `bus`; 0 uses `bus`.
    #[reflect(name = "userBus", min = 0, max = 15)]
    pub user_bus: u32,
    /// The equaliser's low shelf gain below eqLowFreq; 0 dB is flat.
    #[reflect(name = "eqLow", min = -24, max = 24, unit = "db", tunable)]
    pub eq_low: f32,
    /// The low shelf's corner frequency.
    #[reflect(
        name = "eqLowFreq",
        min = 20,
        max = 2000,
        default = 200,
        unit = "hz",
        tunable
    )]
    pub eq_low_freq: f32,
    /// The equaliser's mid band (a peak or notch) gain at eqMidFreq; 0 dB is flat.
    #[reflect(name = "eqMid", min = -24, max = 24, unit = "db", tunable)]
    pub eq_mid: f32,
    /// The mid band's centre frequency.
    #[reflect(
        name = "eqMidFreq",
        min = 40,
        max = 16000,
        default = 1000,
        unit = "hz",
        tunable
    )]
    pub eq_mid_freq: f32,
    /// The mid band's Q: higher is narrower.
    #[reflect(name = "eqMidQ", min = 0.1, max = 10, default = 0.7, tunable)]
    pub eq_mid_q: f32,
    /// The equaliser's high shelf gain above eqHighFreq; 0 dB is flat.
    #[reflect(name = "eqHigh", min = -24, max = 24, unit = "db", tunable)]
    pub eq_high: f32,
    /// The high shelf's corner frequency.
    #[reflect(
        name = "eqHighFreq",
        min = 1000,
        max = 20000,
        default = 5000,
        unit = "hz",
        tunable
    )]
    pub eq_high_freq: f32,
    /// An echo: the delay's time; 0 is off.
    #[reflect(min = 0, max = 2, unit = "s", tunable)]
    pub delay: f32,
    /// How much of each echo feeds the next.
    #[reflect(name = "delayFeedback", min = 0, max = 0.95, default = 0.35, tunable)]
    pub delay_feedback: f32,
    /// How loud the echoes are, added to the dry bus.
    #[reflect(name = "delayMix", min = 0, max = 1, default = 0.35, tunable)]
    pub delay_mix: f32,
    /// A tanh waveshaper's drive, 0 (off) to 1 (heavy: 20x into the curve).
    #[reflect(min = 0, max = 1, tunable)]
    pub distortion: f32,
    /// How much of the distorted signal replaces the clean one.
    #[reflect(name = "distortionMix", min = 0, max = 1, default = 1, tunable)]
    pub distortion_mix: f32,
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
            user_bus: 0,
            eq_low: 0.0,
            eq_low_freq: 200.0,
            eq_mid: 0.0,
            eq_mid_freq: 1000.0,
            eq_mid_q: 0.7,
            eq_high: 0.0,
            eq_high_freq: 5000.0,
            delay: 0.0,
            delay_feedback: 0.35,
            delay_mix: 0.35,
            distortion: 0.0,
            distortion_mix: 1.0,
        }
    }
}

/// A reverb zone: a box (or a sphere) where the shared reverb sounds like
/// this room. While the listener is inside, og.audio eases the reverb's
/// room size and damping, and one bus's send to it, towards the zone's;
/// past its edge the zone fades out over `fade` metres. The listener's
/// weight in each zone is a function of its position that tick (the
/// strongest zone wins, ties to the first in query order), so it is
/// deterministic and the stream carries it. og.Audio's `reverbRoom` and
/// `reverbDamping` and the bus's og.AudioBusFx `reverb` stay the open air's.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.ReverbZone")]
pub struct ReverbZone {
    /// The box's full size along the entity's axes, centred on it (times
    /// og.Transform.scale).
    #[reflect(default = [10, 4, 10], unit = "m")]
    pub size: Vec3,
    /// A sphere of this radius about the entity instead of the box (times
    /// the largest of og.Transform.scale); 0 uses the box.
    #[reflect(min = 0, unit = "m", tunable)]
    pub radius: f32,
    /// How far past the volume's surface the zone fades out, linearly; 0 is
    /// a hard edge.
    #[reflect(min = 0, default = 2, unit = "m", tunable)]
    pub fade: f32,
    /// The bus whose reverb send the zone sets.
    #[reflect(variants = ["master", "music", "sfx", "ui"], default = 2)]
    pub bus: u32,
    /// That bus's send to the reverb inside, 0..1 (its og.AudioBusFx
    /// `reverb` outside).
    #[reflect(min = 0, max = 1, default = 0.5, tunable)]
    pub send: f32,
    /// The reverb's room size inside, 0..1 (og.Audio.reverbRoom outside).
    #[reflect(min = 0, max = 1, default = 0.85, tunable)]
    pub room: f32,
    /// The reverb's damping of high frequencies inside, 0..1
    /// (og.Audio.reverbDamping outside).
    #[reflect(min = 0, max = 1, default = 0.3, tunable)]
    pub damping: f32,
    /// How much of this zone the listener hears, 0..1 (written by og.audio
    /// each tick: 1 inside, 0 past the fade).
    #[reflect(min = 0, max = 1)]
    pub weight: f32,
}

impl Default for ReverbZone {
    fn default() -> Self {
        ReverbZone {
            size: Vec3::new(10.0, 4.0, 10.0),
            radius: 0.0,
            fade: 2.0,
            bus: Bus::Sfx as u32,
            send: 0.5,
            room: 0.85,
            damping: 0.3,
            weight: 0.0,
        }
    }
}

impl ReverbZone {
    /// How much of the zone a listener at `p` hears, the zone's entity at
    /// `t`: 1 inside the volume, falling linearly to 0 at `fade` metres
    /// from its surface (0 outside a hard edge).
    pub fn weight_at(&self, t: &Transform, p: Vec3) -> f32 {
        let scale = t.scale.abs();
        let outside = if self.radius > 0.0 {
            ((p - t.position).length() - self.radius * scale.max_element()).max(0.0)
        } else {
            let local = t.quaternion.normalize().inverse() * (p - t.position);
            let half = self.size.abs() * scale * 0.5;
            (local.abs() - half).max(Vec3::ZERO).length()
        };
        if outside <= 0.0 {
            1.0
        } else if self.fade > 0.0 {
            (1.0 - outside / self.fade).max(0.0)
        } else {
            0.0
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

/// A user bus of the bus graph: bus `index` (4 to 15) sums the sources
/// whose `userBus` names it, runs its og.AudioBusFx (`userBus` = index),
/// and plays into its `parent` bus (a built-in or a user bus of a lower
/// index, so the graph has no cycles; any other parent is master). Its
/// volume is multiplied down the chain to master. The first node in query
/// order for an index counts.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.AudioBusNode")]
pub struct AudioBusNode {
    /// The bus's index, 4 to 15.
    #[reflect(min = 4, max = 15, default = 4)]
    pub index: u32,
    /// The bus it plays into: 0 master, 1 music, 2 sfx, 3 ui, or a user bus below `index`.
    #[reflect(min = 0, max = 15)]
    pub parent: u32,
    /// The bus's gain.
    #[reflect(min = 0, max = 2, default = 1, tunable)]
    pub volume: f32,
}

impl Default for AudioBusNode {
    fn default() -> Self {
        AudioBusNode {
            index: 4,
            parent: 0,
            volume: 1.0,
        }
    }
}

/// A mixer snapshot (combat, underwater, paused): per-bus volume and
/// filter settings the mix blends towards while `active`, and away from
/// when not, over `fade` seconds. og.audio moves `weight` towards 1 or 0
/// each tick and sends the blend in the stream, the gains ramping
/// linearly across each tick so the blend is a straight line to the
/// sample. Several snapshots multiply their volumes and take the lowest
/// low-pass and the highest high-pass.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.AudioSnapshot")]
pub struct AudioSnapshot {
    /// Blends in while true, out while false.
    pub active: bool,
    /// Seconds a full blend in or out takes; 0 is at once.
    #[reflect(min = 0, max = 30, default = 0.5, unit = "s", tunable)]
    pub fade: f32,
    /// How far it is blended in, 0 to 1 (written by og.audio each tick).
    #[reflect(min = 0, max = 1)]
    pub weight: f32,
    /// Per bus (by index: master, music, sfx, ui, then user buses 4 to 15), the gain it multiplies in at full weight; buses past the list's length are 1.
    pub volume: List<f32, 16>,
    /// Per bus, a low-pass cutoff in Hz at full weight (blended from 20 kHz in octaves); 0 or past the list's length is none.
    pub lowpass: List<f32, 16>,
    /// Per bus, a high-pass cutoff in Hz at full weight (blended from 20 Hz in octaves); 0 or past the list's length is none.
    pub highpass: List<f32, 16>,
}

impl Default for AudioSnapshot {
    fn default() -> Self {
        AudioSnapshot {
            active: false,
            fade: 0.5,
            weight: 0.0,
            volume: List::new(),
            lowpass: List::new(),
            highpass: List::new(),
        }
    }
}

/// `og.AudioEvent.order` values.
pub mod event_order {
    /// A weighted random pick each time.
    pub const RANDOM: u32 = 0;
    /// The sounds in turn.
    pub const ROUND_ROBIN: u32 = 1;
    /// Random, every sound once before any repeats.
    pub const SHUFFLE: u32 = 2;
}

/// An audio event: one of a set of sounds played on each trigger, with
/// pitch and volume jitter, a cooldown and a voice limit. Setting
/// `trigger` plays it (the engine sets it back to false): og.audioEvents,
/// before og.audio, picks a sound with the engine's seeded random stream
/// (world state, never Math.random), and spawns a voice entity (an
/// og.AudioSource and og.AudioEventVoice, parented to this entity, so a
/// spatial event is heard where its entity is), despawned when it ends.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.AudioEvent")]
pub struct AudioEvent {
    /// The variations (og.load.sound or og.audio.createSound handles); 0s are skipped.
    pub sounds: List<u64, 8>,
    /// Weights of a random pick, by position in `sounds`; past the list's length 1.
    pub weights: List<f32, 8>,
    /// How the next sound is chosen.
    #[reflect(variants = ["random", "roundRobin", "shuffle"])]
    pub order: u32,
    /// Linear gain.
    #[reflect(min = 0, max = 4, default = 1, tunable)]
    pub volume: f32,
    /// Each play's volume varies by up to this many dB either way.
    #[reflect(name = "volumeJitter", min = 0, max = 24, unit = "db", tunable)]
    pub volume_jitter: f32,
    /// Playback rate.
    #[reflect(min = 0.0625, max = 16, default = 1, tunable)]
    pub pitch: f32,
    /// Each play's pitch varies by up to this many semitones either way.
    #[reflect(name = "pitchJitter", min = 0, max = 24, tunable)]
    pub pitch_jitter: f32,
    /// A trigger this soon after the last play is ignored.
    #[reflect(min = 0, max = 60, unit = "s", tunable)]
    pub cooldown: f32,
    /// The most voices of this event at once; 0 is no limit.
    #[reflect(name = "maxVoices", min = 0, max = 64)]
    pub max_voices: u32,
    /// At maxVoices, the oldest voice is stopped for the new one; false ignores the trigger instead.
    #[reflect(default = 1)]
    pub steal: bool,
    /// The bus its voices play on.
    #[reflect(variants = ["master", "music", "sfx", "ui"], default = 2)]
    pub bus: u32,
    /// A user bus its voices play into instead (og.AudioSource.userBus).
    #[reflect(name = "userBus", min = 0, max = 15)]
    pub user_bus: u32,
    /// Its voices are spatial, at this entity's world transform.
    pub spatial: bool,
    /// og.AudioSource.refDistance of its voices.
    #[reflect(name = "refDistance", min = 0, default = 1, unit = "m", tunable)]
    pub ref_distance: f32,
    /// og.AudioSource.maxDistance of its voices.
    #[reflect(name = "maxDistance", min = 0, default = 100, unit = "m", tunable)]
    pub max_distance: f32,
    /// og.AudioSource.priority of its voices.
    #[reflect(min = 0, max = 255, default = 128)]
    pub priority: u32,
    /// Set true to play it this tick; the engine sets it back to false.
    pub trigger: bool,
    /// Plays so far (written by og.audioEvents).
    pub played: u32,
    /// The world time of the last play (written by og.audioEvents).
    #[reflect(unit = "s")]
    pub last: f64,
    /// The position in `sounds` played last (written by og.audioEvents).
    pub index: u32,
    /// A shuffle's sounds already played this round, one bit each (written by og.audioEvents).
    pub bag: u32,
    /// Its voices playing now (written by og.audioEvents).
    pub voices: u32,
}

impl Default for AudioEvent {
    fn default() -> Self {
        AudioEvent {
            sounds: List::new(),
            weights: List::new(),
            order: event_order::RANDOM,
            volume: 1.0,
            volume_jitter: 0.0,
            pitch: 1.0,
            pitch_jitter: 0.0,
            cooldown: 0.0,
            max_voices: 0,
            steal: true,
            bus: Bus::Sfx as u32,
            user_bus: 0,
            spatial: false,
            ref_distance: 1.0,
            max_distance: 100.0,
            priority: 128,
            trigger: false,
            played: 0,
            last: 0.0,
            index: 0,
            bag: 0,
            voices: 0,
        }
    }
}

/// Engine-owned: a voice an audio event spawned (beside its og.AudioSource
/// and an og.Parent to the event's entity). Despawned when it ends.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.AudioEventVoice")]
pub struct AudioEventVoice {
    /// The og.AudioEvent's entity.
    pub event: u64,
    /// The sound it plays.
    pub sound: u64,
    /// The play it is, counting from 1 (the oldest is stolen first).
    pub play: u32,
}

/// Values per bus in [`AudioMix::sent`]: its gain, its parent, the 13
/// parameters of og.AudioBusFx's first version, and the equaliser, echo
/// and waveshaper's 12.
pub const GRAPH_STRIDE: usize = 27;
/// Values in [`AudioMix::sent`].
pub const GRAPH_LEN: usize = GRAPH_STRIDE * MAX_BUSES;

/// Engine-owned: what the bus graph last told the mixer, and the sound
/// events' random stream. A resource, so in every snapshot and the world
/// hash.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.AudioMix")]
pub struct AudioMix {
    /// The graph's parameters last sent, [`GRAPH_STRIDE`] per bus (the
    /// built-in buses' gains and their first-version effects travel as
    /// before and hold constants here).
    pub sent: List<f32, 512>,
    /// The audio events' random state (an og_core::Rng's, forked from the
    /// world's seed at install).
    pub rng: u32,
}

impl Default for AudioMix {
    fn default() -> Self {
        AudioMix {
            sent: List::new(),
            rng: 1,
        }
    }
}
