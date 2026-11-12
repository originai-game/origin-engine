//! Scene geometry, materials, textures and resource handles.

pub use og_draw::{
    EnvironmentMapHandle, FontHandle, LutHandle, MaterialHandle, MeshHandle, PaletteHandle,
    ShaderHandle, SpriteArray, TerrainHandle, TextureArrayHandle, TextureHandle,
};

/// Which slots of one kind of resource hold one, and their generations:
/// `remove_*` empties a slot and moves its generation on, and the next
/// `add_*` of the kind fills it again (last freed first), so handles are
/// reused without an old one reaching the new resource.
#[derive(Debug, Default)]
pub(super) struct Slots {
    generation: Vec<u8>,
    removed: Vec<bool>,
    free: Vec<u32>,
    /// Whether any slot of the kind was ever removed (no check is needed
    /// before).
    pub(super) any_removed: bool,
}

impl Slots {
    /// The slot to fill: a freed one, or a new one at `len` (the kind's
    /// length). Returns the slot, its generation and whether it was reused.
    pub(super) fn take(&mut self, len: usize, reuse: bool) -> (usize, u8, bool) {
        if reuse && let Some(i) = self.free.pop() {
            let i = i as usize;
            self.removed[i] = false;
            return (i, self.generation[i], true);
        }
        debug_assert_eq!(len, self.generation.len());
        self.generation.push(0);
        self.removed.push(false);
        (len, 0, false)
    }

    pub(super) fn live(&self, index: usize, generation: u8) -> bool {
        index < self.generation.len()
            && self.generation[index] == generation
            && !self.removed[index]
    }

    /// Empties a live slot; `reuse`: a later add may fill it.
    pub(super) fn remove(&mut self, index: usize, reuse: bool) {
        self.removed[index] = true;
        self.generation[index] = self.generation[index].wrapping_add(1);
        self.any_removed = true;
        if reuse {
            self.free.push(index as u32);
        }
    }

    pub(super) fn generation(&self, index: usize) -> u8 {
        self.generation.get(index).copied().unwrap_or(0)
    }
}
