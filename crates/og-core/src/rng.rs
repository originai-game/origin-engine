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
#[reflect(name = "og.Rng")]
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

    /// Back to its seed's first draw, as [`Rng::new`] made it: a game's
    /// restart draws what a fresh start draws.
    pub fn reset(&mut self) {
        *self = Rng::new(self.seed);
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

    /// Uniform in [low, high) as `f32` (the `f64` draw, rounded without
    /// reaching `high`).
    pub fn range_f32(&mut self, low: f32, high: f32) -> f32 {
        let v = self.range(f64::from(low), f64::from(high)) as f32;
        if low < high && v >= high {
            high.next_down()
        } else if low > high && v <= high {
            high.next_up()
        } else {
            v
        }
    }

    /// Uniform integer in [low, high].
    pub fn integer(&mut self, low: i64, high: i64) -> i64 {
        low + (self.next_f64() * (high - low + 1) as f64).floor() as i64
    }
}

/// PCG32 (O'Neill 2014, XSH-RR on a 64-bit LCG): a fast stream for tools
/// that draw many numbers per item (a bake's per-texel paths), seeded per
/// item so the result does not depend on which thread draws which. Never
/// simulation state: the tick draws from [`Rng`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pcg32 {
    state: u64,
}

impl Pcg32 {
    /// The stream for `seed`: the seed spread by the golden ratio, then one
    /// step so neighbouring seeds part at once.
    pub fn new(seed: u64) -> Pcg32 {
        let mut r = Pcg32 {
            state: seed.wrapping_mul(crate::hash::GOLDEN64) ^ 0xD1B5_4A32_D192_ED03,
        };
        r.next_u32();
        r
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        xorshifted.rotate_right((old >> 59) as u32)
    }

    /// Uniform in [0, 1), from the top 24 bits.
    pub fn next_f32(&mut self) -> f32 {
        crate::hash::unit24(self.next_u32())
    }
}

#[cfg(test)]
mod rng_contracts {
    //! The platform's rng vectors (produced by the Web scaffold's `rng.ts` under
    //! bun, inherited by the Godot module's `rng.rs`): a seed names the same
    //! sequence on every substrate.

    use crate::Rng;
    use crate::reflect::{from_bytes, to_bytes};

    /// O'Neill's pcg32 from `state`, written out (the reference's
    /// `pcg32_random_r` with the default increment).
    #[test]
    fn pcg32_is_the_reference_xsh_rr_stream() {
        let mut r = crate::rng::Pcg32::new(7);
        let mut state = 7u64.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 0xD1B5_4A32_D192_ED03;
        let mut reference = || {
            let old = state;
            state = old
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let x = (((old >> 18) ^ old) >> 27) as u32;
            let rot = (old >> 59) as u32;
            (x >> rot) | (x << (rot.wrapping_neg() & 31))
        };
        reference();
        for _ in 0..64 {
            assert_eq!(r.next_u32(), reference());
        }
        for _ in 0..1000 {
            assert!((0.0..1.0).contains(&r.next_f32()));
        }
    }

    #[test]
    fn matches_the_web_substrate_sequence() {
        let cases: [(u32, u32, [f64; 4], u32); 4] = [
            (
                0,
                1,
                [
                    0.479614513926208,
                    0.48086497001349926,
                    0.8585285847075284,
                    0.9917182873468846,
                ],
                2027808485,
            ),
            (
                1,
                2261973619,
                [
                    0.9481164838653058,
                    0.5605214422103018,
                    0.1859289831481874,
                    0.14164146734401584,
                ],
                4289782103,
            ),
            (
                42,
                671623878,
                [
                    0.09396462352015078,
                    0.5757808552589267,
                    0.19534794357605278,
                    0.7601067263167351,
                ],
                2699432362,
            ),
            (
                4294967295,
                2578835075,
                [
                    0.3764090109616518,
                    0.41705114021897316,
                    0.07903929799795151,
                    0.706570005742833,
                ],
                311676263,
            ),
        ];
        for (seed, state0, draws, after) in cases {
            let mut rng = Rng::new(seed);
            assert_eq!(rng.save(), state0, "seed {seed}");
            for expected in draws {
                assert_eq!(rng.next_f64(), expected, "seed {seed}");
            }
            assert_eq!(rng.save(), after, "seed {seed}");
        }
    }

    #[test]
    fn save_restore_and_reflected_bytes_resume_the_sequence() {
        let mut rng = Rng::new(9);
        rng.next_f64();
        let saved = rng.save();
        let bytes = to_bytes(&rng).unwrap();
        let a = rng.next_f64();
        rng.restore(saved);
        assert_eq!(rng.next_f64(), a);
        let mut copy: Rng = from_bytes(&bytes).unwrap();
        assert_eq!(copy.next_f64(), a);
    }
}
