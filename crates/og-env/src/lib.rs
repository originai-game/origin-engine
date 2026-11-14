//! Origin Engine environment fields: what the world is at a point, as data
//! and pure functions. The renderer draws a field and the simulation reads
//! it through the same function, so the two cannot disagree; nothing here
//! holds a world, runs a system or touches the GPU.
//!
//! | module | what it owns |
//! |---|---|
//! | [`ocean`] | [`ocean::Ocean`], a wind's sea of Gerstner waves, its displacement and its height at a point |
//! | [`shore`] | [`shore::Shore`], a sea's seabed: its depth and distance to the waterline, and the trains rolling in |
//! | [`river`] | a river's course and its surface along it |
//! | [`air`] | [`air::Air`], the world's wind and the local pushes on it, as one function of a point |
//! | [`volume`] | [`volume::Volume`], the one shape of a region: a box or a sphere on its entity, with a fade |
//! | [`weather`] | the kinds of weather and what they leave on the ground: wetness, puddles, snow and dust |
//! | [`day`] | the time of day at a tick |
//!
//! Invariants are og-core's: no global mutable state, no wall clock, and
//! the transcendentals are [`og_core::math`]'s, so every host computes the
//! same field.

pub mod air;
pub mod day;
pub mod ocean;
pub mod river;
pub mod shore;
pub mod volume;
pub mod weather;
