//! The seeded, splittable random source.
//!
//! Bit-for-bit the algorithm of the Web scaffold's `src/core/rng.ts` and the
//! Origin Engine (Godot) module's `rng.rs` in the platform repository, so a
//! seed names the same sequence on every first-party substrate, on wasm and
//! native alike (integer arithmetic only). Every random draw in a simulation
//! comes from an `Rng` the world owns (a resource), so snapshots capture it
//! and a replay draws the same numbers.

use crate::Reflect;

/// Splitmix-style scrambling, chosen so a short human seed still spreads.
fn scramble(seed: u32) -> u32 {
    let mut value = seed;
    value = (value ^ (value >> 16)).wrapping_mul(0x21f0_aaad);
    value = (value ^ (value >> 15)).wrapping_mul(0x735a_2d97);
    value ^ (value >> 15)
}

/// A deterministic random generator: 32-bit seed, 32-bit state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub struct Rng {
    /// The seed this generator (or the parent it was forked from) was made with.
    seed: u32,
    /// The whole generator state.
    state: u32,
}

impl Default for Rng {
    /// The generator for seed 0.
    fn default() -> Self {
        Rng::new(0)
    }
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        let state = scramble(seed);
        Self {
            seed,
            state: if state == 0 { 1 } else { state },
        }
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    /// The whole generator state, so a replay or a sweep can resume mid-run.
    pub fn save(&self) -> u32 {
        self.state
    }

    pub fn restore(&mut self, state: u32) {
        self.state = if state == 0 { 1 } else { state };
    }

    /// A fresh generator on a derived seed, for a subsystem that must not
    /// perturb the main sequence by drawing from it. Labels hash by UTF-16
    /// code unit, as the Web substrate's `charCodeAt` does.
    pub fn fork(&self, label: &str) -> Rng {
        let mut derived = self.seed;
        for unit in label.encode_utf16() {
            derived = (derived ^ u32::from(unit)).wrapping_mul(0x0100_0193);
        }
        Rng::new(derived)
    }

    /// The next raw 32-bit draw; [`Self::next_f64`] is this divided by 2^32.
    pub fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_add(0x9e37_79b9);
        let mut value = self.state;
        value = (value ^ (value >> 15)).wrapping_mul(value | 1);
        value ^= value.wrapping_add((value ^ (value >> 7)).wrapping_mul(value | 61));
        value ^ (value >> 14)
    }

    /// Uniform in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        f64::from(self.next_u32()) / 4_294_967_296.0
    }

    /// Uniform in [low, high).
    pub fn range(&mut self, low: f64, high: f64) -> f64 {
        low + self.next_f64() * (high - low)
    }

    /// Uniform in [low, high) as `f32` (the `f64` draw, rounded).
    pub fn range_f32(&mut self, low: f32, high: f32) -> f32 {
        self.range(f64::from(low), f64::from(high)) as f32
    }

    /// Uniform integer in [low, high].
    pub fn integer(&mut self, low: i64, high: i64) -> i64 {
        low + (self.next_f64() * (high - low + 1) as f64).floor() as i64
    }
}
