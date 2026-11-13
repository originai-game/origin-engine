//! A stable 64-bit hash (FNV-1a) for world hashes, schema layout hashes and
//! cook stamps, and the integer hashes the engine draws fixed numbers from:
//! the one copy of each.
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

/// FNV-1a taking a whole word a step where [`hash_bytes`] takes a byte: a
/// key over values already in words (a mesh's floats and indices, a
/// cache's inputs).
pub fn hash_words(words: impl IntoIterator<Item = u64>) -> u64 {
    words
        .into_iter()
        .fold(OFFSET, |h, w| (h ^ w).wrapping_mul(PRIME))
}

/// 32-bit FNV-1a of `bytes`, for an id that has to fit 32 bits.
pub fn hash_bytes32(bytes: impl IntoIterator<Item = u8>) -> u32 {
    bytes.into_iter().fold(0x811c_9dc5, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(0x0100_0193)
    })
}

/// splitmix64's increment (the golden ratio's fraction in 64 bits).
pub const GOLDEN64: u64 = 0x9e37_79b9_7f4a_7c15;

/// splitmix64's finaliser (Sebastiano Vigna, public domain): a 64-bit
/// integer hash, the same on every run and host.
#[inline]
pub fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// One splitmix64 step from `x`: the hash of a key whose neighbours must
/// not look alike (an entity id, a tick and an index packed together).
#[inline]
pub fn splitmix64(x: u64) -> u64 {
    mix64(x.wrapping_add(GOLDEN64))
}

/// The next number of the splitmix64 stream whose state is `state` (a
/// seed, to begin with): numbers fixed per seed and never part of the
/// simulation ([`crate::Rng`] is the tick's).
#[inline]
pub fn splitmix64_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(GOLDEN64);
    mix64(*state)
}

/// A uniform float in [0, 1) from the top 24 bits of a 64-bit hash.
#[inline]
pub fn unit24_of64(h: u64) -> f32 {
    (h >> 40) as f32 / 16_777_216.0
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The published vectors: a saved id or a pinned replay holds these
    /// functions' values, so none of them may move.
    #[test]
    fn the_hashes_give_their_published_values() {
        // splitmix64 from seed 0 (Vigna's reference output).
        let mut s = 0;
        assert_eq!(splitmix64_next(&mut s), 0xe220_a839_7b1d_cdaf);
        assert_eq!(splitmix64_next(&mut s), 0x6e78_9e6a_a1b9_65f4);
        assert_eq!(splitmix64(0), 0xe220_a839_7b1d_cdaf);
        // FNV-1a's test vectors for "a" and "foobar".
        assert_eq!(hash_bytes(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(hash_bytes(b"foobar"), 0x8594_4171_f739_67e8);
        assert_eq!(hash_bytes32(*b"a"), 0xe40c_292c);
        assert_eq!(hash_bytes32(*b"foobar"), 0xbf9c_f968);
        // A word a step is a byte a step for words that are bytes.
        assert_eq!(hash_words(b"foobar".map(u64::from)), hash_bytes(b"foobar"));
    }
}
