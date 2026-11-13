//! A stable 64-bit hash (FNV-1a) for world hashes, schema layout hashes and
//! cook stamps.
//!
//! Stable means: the same bytes give the same value on every host, build and
//! run. `std`'s hashers are randomly seeded or unspecified, so they are never
//! used for anything that is stored or compared across runs.

const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a over a byte stream. Integers are fed little-endian.
#[derive(Clone, Copy, Debug)]
pub struct StableHasher {
    state: u64,
}

impl Default for StableHasher {
    fn default() -> Self {
        Self { state: OFFSET }
    }
}

impl StableHasher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, bytes: &[u8]) {
        let mut state = self.state;
        for &b in bytes {
            state ^= u64::from(b);
            state = state.wrapping_mul(PRIME);
        }
        self.state = state;
    }

    pub fn write_u32(&mut self, value: u32) {
        self.write(&value.to_le_bytes());
    }

    pub fn write_u64(&mut self, value: u64) {
        self.write(&value.to_le_bytes());
    }

    /// A length-prefixed string, so `("ab", "c")` and `("a", "bc")` differ.
    pub fn write_str(&mut self, value: &str) {
        self.write_u64(value.len() as u64);
        self.write(value.as_bytes());
    }

    pub fn finish(&self) -> u64 {
        self.state
    }
}

/// FNV-1a of `bytes`.
pub fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut h = StableHasher::new();
    h.write(bytes);
    h.finish()
}

/// PCG's output permutation (Jarzynski and Olano, "Hash functions for GPU
/// rendering", 2020): a 32-bit integer hash. The shaders' `pcg`
/// (og-render's `hash.wgsl`) is this, operation for operation, so a CPU
/// path and a GPU path draw the same numbers.
#[inline]
pub fn pcg(v: u32) -> u32 {
    let state = v.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    let word = ((state >> ((state >> 28) + 4)) ^ state).wrapping_mul(277_803_737);
    (word >> 22) ^ word
}

/// lowbias32 (Chris Wellons, public domain): a 32-bit integer hash, the
/// shaders' `hash_lowbias`.
#[inline]
pub fn lowbias32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

/// A uniform float in [0, 1) from the top 24 bits of hashed bits `h` (the
/// shaders' `hash_to_unit`).
#[inline]
pub fn unit24(h: u32) -> f32 {
    (h >> 8) as f32 / 16_777_216.0
}
