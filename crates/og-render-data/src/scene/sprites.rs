/// How a texture is sampled where sprites draw it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextureFilter {
    /// Bilinear, with mips when the texture has them.
    #[default]
    Linear,
    /// The nearest texel: crisp pixel art at any scale.
    Nearest,
    /// torcado's cleanEdge (MIT, `third_party/cleanedge/`): pixel art
    /// turned or scaled without blur or jaggies. Each sample reads the 5x5
    /// texels round it and returns one of them, never a blend, chosen by
    /// the edges and 2:1 and 1:1 slopes it finds there, so a turned sprite
    /// keeps its palette and its edges are redrawn at the angle. Texels
    /// outside the sprite's region (a sheet's neighbouring frame) read as
    /// clear. Sprites and tile layers drawn with the sprite shader
    /// (sprite.wgsl) only: texture arrays, the overlay and particles take
    /// it as [`TextureFilter::Nearest`].
    CleanEdge,
}

impl TextureFilter {
    /// Whether the GPU sampler is nearest (both pixel-art filters are).
    pub fn is_nearest(self) -> bool {
        self != TextureFilter::Linear
    }
}
