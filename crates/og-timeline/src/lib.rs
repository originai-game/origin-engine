//! Origin Engine timelines and paths (docs/design.md §2.7, "Timeline and
//! paths"): the data and the evaluation, with no ECS. og-runtime owns the
//! `og.path` and `og.timeline` systems that apply them to a world; og-cook
//! cooks `type timeline` files (importing caption files) into blobs.
//!
//! - [`Path`]: an arc-length spline (`catmullRom`, `bezier`, `linear`),
//!   sampled by a 0..1 parameter into a position, tangent and up, measured,
//!   and searched for the point nearest another.
//! - [`Timeline`]: the asset (`type timeline` text), parsed and written back
//!   canonically; its tracks (camera, transform, animation, audio, music,
//!   caption, event, transition, marker) hold keys at times.
//! - [`eval`]: every value a track sets as a function of the time alone, so
//!   seeking is exact.
//! - [`captions`]: `.srt` cues and word-timestamp `.json` (the cook's import).
//!
//! Only `+ − × ÷`, `sqrt` and og-core's deterministic trigonometry: the same
//! bits on every host.

pub mod captions;
pub mod eval;
mod model;
mod path;

pub use model::*;
pub use path::{Nearest, Path, PathDesc, PathKind, PathSample, SAMPLES, up_of};
