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
