//! Entity ids: a generational handle packed into a `u64`.

use og_core::RawHandle;
use std::fmt;

/// An entity: slot index plus generation. A despawned entity's id stays dead
/// forever (the slot's generation moves on), so a stale id is detected, never
/// aliased. `to_bits()` is `generation << 32 | index`; `0` is never live.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity(pub(crate) RawHandle);

impl Entity {
    pub fn index(self) -> u32 {
        self.0.index()
    }

    pub fn generation(self) -> u32 {
        self.0.generation()
    }

    pub fn to_bits(self) -> u64 {
        self.0.to_bits()
    }

    pub fn from_bits(bits: u64) -> Entity {
        Entity(RawHandle::from_bits(bits))
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "e{}v{}", self.index(), self.generation())
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "e{}v{}", self.index(), self.generation())
    }
}
