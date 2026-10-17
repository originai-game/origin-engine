//! Origin Engine audio (`docs/design.md` §2.7, phase 1 M6).
//!
//! Two halves that only meet through the command stream:
//!
//! - **The tick side** ([`install`]): the records `og.AudioSource`,
//!   `og.AudioListener`, `og.Audio`, the events `og.SoundStarted` and
//!   `og.SoundEnded`, and the `og.audio` system. Every tick the system turns
//!   source changes into time-stamped [`Command`]s (play, stop, params,
//!   bus), keeps each playing source's playhead in `og.AudioPlayback` (so a
//!   one-shot ends on a tick that depends only on the world), and chains
//!   the tick's encoded commands into `og.AudioStream.hash`. Both are world
//!   state, so `world_hash` and snapshots cover the stream; nothing it
//!   computes depends on a sink.
//! - **The render side** ([`Mixer`]): consumes [`TickPacket`]s from the
//!   [`AudioLink`] and renders stereo f32 PCM outside the tick: QOA
//!   streaming from [`SoundBank`], one-shots, seamless loops, pitch by
//!   resampling, distance and pan (computed by the tick side), bus gains and
//!   a soft limiter. It never writes the world.
//!
//! Sinks: the web host renders in the engine Worker and ships PCM to an
//! AudioWorklet through a SharedArrayBuffer ring; the native host pulls the
//! mixer from a cpal callback (`native-audio`); headless runs use
//! [`HeadlessSink`] (`null`, or `capture` to a WAV). See `README.md`.
//!
//! ```
//! use og_audio::{install, AudioDesc, AudioSource, HeadlessMode, HeadlessSink};
//! use og_ecs::{World, WorldDesc};
//!
//! let mut world = World::new(&WorldDesc::default()).unwrap();
//! let link = install(&mut world, &AudioDesc::default()).unwrap();
//! let tone: Vec<i16> = (0..4800).map(|i| ((i as f32 * 0.06).sin() * 8000.0) as i16).collect();
//! let beep = link.bank().insert_pcm("beep", 48_000, 1, &tone).unwrap();
//! world.spawn((AudioSource::new(beep),)).unwrap();
//! let mut sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: 48_000 });
//! for _ in 0..10 {
//!     world.step().unwrap();
//!     sink.pump();
//! }
//! assert_eq!(sink.pcm().len(), 10 * 800 * 2);
//! ```

pub mod command;
pub mod mixer;
pub mod schema;
pub mod sink;
pub mod sound;
pub mod system;

/// The QOA codec: og-cook's `src/qoa.rs`, compiled into this crate so audio
/// needs neither og-cook nor its renderer dependency. The encoder is here
/// for tests and tools that synthesise sounds.
#[path = "../../og-cook/src/qoa.rs"]
pub mod qoa;

pub use command::{Command, Op, TickPacket, COMMAND_BYTES};
pub use mixer::{Clock, MixStats, Mixer, MixerDesc};
pub use schema::{
    end_reason, Audio, AudioListener, AudioPlayback, AudioSource, AudioStream, Bus, SoundEnded,
    SoundStarted, Transform,
};
pub use sink::{wav, HeadlessMode, HeadlessSink, SinkStats};
pub use sound::{SoundBank, SoundData};
pub use system::{install, link, publish_link, retire_link, AudioDesc, AudioLink, SYSTEM_NAME};
