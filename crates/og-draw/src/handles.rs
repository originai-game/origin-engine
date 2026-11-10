//! Resource handles: the ids a scene gives what is added to it.

/// A handle's slot is its low 24 bits; the top 8 are the slot's generation.
const INDEX_BITS: u32 = 24;
const INDEX_MASK: u32 = (1 << INDEX_BITS) - 1;
macro_rules! handle {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub u32);
        impl $name {
            /// The position of the resource in its scene: its slot, in
            /// insertion order (a slot freed by a `remove_*` is reused).
            pub fn index(self) -> usize {
                (self.0 & INDEX_MASK) as usize
            }
            /// How many times the slot was emptied before this handle was
            /// given (wrapping at 256): a handle of a removed resource no
            /// longer matches its slot.
            pub fn generation(self) -> u8 {
                (self.0 >> INDEX_BITS) as u8
            }
            /// The handle as one number (slot and generation), for a
            /// snapshot that keeps it (og-ui's leaving elements).
            pub fn to_bits(self) -> u32 {
                self.0
            }
            /// The handle [`Self::to_bits`] gave.
            pub fn from_bits(bits: u32) -> Self {
                $name(bits)
            }
            #[allow(dead_code)]
            pub fn from_index(index: usize) -> Self {
                $name(index as u32)
            }
            #[allow(dead_code)]
            pub fn from_parts(index: usize, generation: u8) -> Self {
                debug_assert!(index as u32 <= INDEX_MASK);
                $name(index as u32 | (generation as u32) << INDEX_BITS)
            }
        }
    };
}
handle!(
    /// A mesh added to a [`RenderScene`].
    MeshHandle
);
handle!(
    /// A material added to a [`RenderScene`].
    MaterialHandle
);
handle!(
    /// A texture added to a [`RenderScene`].
    TextureHandle
);
handle!(
    /// A material shader added to a [`RenderScene`].
    ShaderHandle
);

impl TextureHandle {
    /// The handle as a [`crate::StandardMaterial`] texture slot: its index
    /// plus one (0 is "no texture").
    pub fn slot(self) -> u32 {
        self.0 + 1
    }

    /// The handle a texture slot names (`None` for slot 0).
    pub fn from_slot(slot: u32) -> Option<TextureHandle> {
        slot.checked_sub(1).map(TextureHandle)
    }
}
handle!(
    /// An MSDF font added to a [`RenderScene`].
    FontHandle
);
handle!(
    /// Terrain data added to a [`RenderScene`] ([`RenderScene::add_terrain`]).
    TerrainHandle
);
handle!(
    /// A prefiltered environment map added to a [`RenderScene`]
    /// ([`RenderScene::add_environment_map`]).
    EnvironmentMapHandle
);
handle!(
    /// A texture array added to a [`RenderScene`]
    /// ([`RenderScene::add_texture_array`]).
    TextureArrayHandle
);

impl TextureArrayHandle {
    /// The handle as a [`crate::StandardMaterial`] texture-array slot
    /// (`mapArray`, `normalMapArray`): its bits plus one (0 is none).
    pub fn slot(self) -> u32 {
        self.0 + 1
    }

    /// The handle a texture-array slot names (`None` for slot 0).
    pub fn from_slot(slot: u32) -> Option<TextureArrayHandle> {
        slot.checked_sub(1).map(TextureArrayHandle)
    }
}
handle!(
    /// A colour LUT added to a [`RenderScene`] ([`RenderScene::add_lut`]).
    LutHandle
);
handle!(
    /// A colour palette added to a [`RenderScene`] for
    /// [`crate::Stylise`]'s quantisation ([`RenderScene::add_palette`]).
    PaletteHandle
);

/// A sprite drawn from a layer of a texture array instead of its texture
/// ([`RenderScene::sprite_arrays`], [`Overlay::sprite_arrays`]): a sheet of
/// same-sized frames batched into one draw whichever frame each sprite
/// shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteArray {
    /// The index in [`RenderScene::sprites`].
    pub sprite: u32,
    pub array: TextureArrayHandle,
    /// The layer drawn, below the array's layer count.
    pub layer: u32,
}
