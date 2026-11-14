//! Streamed video (docs/design.md §2.9, roadmap G3-2): AV1 cooked into the
//! engine's own `og.VideoStream` blob (og-bundle) and decoded as it plays.
//!
//! - [`ivf`]: the IVF the cook has ffmpeg write, read into temporal units
//!   with their key frames marked (cook side, no decoder needed).
//! - [`mp4`]: the MP4 the cook has ffmpeg write a stream's H.264 or HEVC
//!   rendition to (for browsers without an AV1 decoder), read the same way,
//!   with its decoder configuration record and codec string.
//! - [`yuv`]: 8-bit 4:2:0 to RGBA8 in fixed point.
//! - [`decode`] (feature `decode`): rav1d, BSD-2-Clause, the memory-safe
//!   Rust port of dav1d, through its dav1d C ABI.
//! - [`stream`]: the decode plan every host shares and, natively, a decode
//!   thread per stream.
//!
//! Decoding is presentation: what a decoder has ready never reaches the
//! world (og-runtime's `og.video` system advances the player by the tick).

#[cfg(feature = "decode")]
pub mod decode;
pub mod ivf;
pub mod mp4;
pub mod stream;
pub mod yuv;
