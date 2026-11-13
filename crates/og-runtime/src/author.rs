//! What an author and an agent see of a running engine: the probe and
//! its views ([`probe`], [`describe`], [`pulse`], [`memory`], [`lanes`]),
//! the inspector and scene write-back, traces, recorded frames, the model
//! review's cameras and the development-only determinism digests.

pub(crate) mod describe;
#[cfg(not(feature = "publish"))]
pub(crate) mod determinism;
#[cfg(not(feature = "publish"))]
pub(crate) mod inspector;
pub(crate) mod lanes;
pub(crate) mod memory;
pub(crate) mod probe;
pub(crate) mod pulse;
#[cfg(not(feature = "publish"))]
pub(crate) mod record;
pub(crate) mod review;
#[cfg(not(feature = "publish"))]
pub(crate) mod scene_save;
pub(crate) mod trace;
