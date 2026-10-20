//! Stylisation as data (the IDL's `Stylise` and `MaterialStyle`
//! resources): the primitives a look composes into a non-photographic
//! frame. [`RenderScene::stylise`](crate::RenderScene::stylise) carries
//! them; each is `None` (or zero) by default and then costs nothing.
//!
//! This is the description the renderer reads. The passes that draw it
//! land with the shaders (the stylisation round's phase B);
//! [`STYLISE_DRAWN`] names what this renderer draws today, and the IDL's
//! `RenderCapabilities.stylise` reports it, so a look that names a
//! primitive the renderer lacks still applies and the rest of the frame is
//! drawn.

/// The primitive groups this renderer draws, by the names
/// `RenderCapabilities.stylise` reports: `pixel`, `posterize`, `palette`,
/// `dither`, `halftone`, `hatching`, `paint`, `ink`, `inkDepth`,
/// `depthBands` and `retro3d` ([`Stylise`]); `toon`, `shadowTint`, `rim`,
/// `outline`, `outlineDistance` and `flatShading` ([`MaterialStyle`]).
pub const STYLISE_DRAWN: &[&str] = &[
    "pixel",
    "posterize",
    "palette",
    "dither",
    "halftone",
    "hatching",
    "paint",
    "ink",
    "inkDepth",
    "depthBands",
    "retro3d",
    "toon",
    "shadowTint",
    "rim",
    "outline",
    "outlineDistance",
    "flatShading",
];

pub use crate::scene::PaletteHandle;

/// Up to 256 sRGB colours, in the order the palette file lists them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Palette {
    pub colors: Vec<[u8; 3]>,
}

/// The most colours a [`Palette`] holds.
pub const MAX_PALETTE_COLORS: usize = 256;

/// A low-resolution render target scaled up by whole numbers with nearest
/// filtering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelScale {
    /// The logical height in pixels (the width follows the output's aspect).
    pub height: u32,
    /// The whole-number upscale; 0 takes the largest that fits the output.
    pub upscale: u32,
    /// Snap the camera and world sprites to the logical pixel grid.
    pub snap: bool,
}

/// The threshold pattern quantisation is dithered with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DitherPattern {
    #[default]
    None,
    Bayer2,
    Bayer4,
    Bayer8,
    /// A 64 × 64 void-and-cluster tile (`blue_noise.rs`), repeated.
    BlueNoise,
    /// Interleaved gradient noise (Jimenez), computed per pixel: no
    /// texture, a diagonal structure up close.
    InterleavedGradient,
}

/// Fewer tones and a fixed palette, mixed by a dither.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quantize {
    /// Tones per channel (2 to 64); 0 leaves the channels continuous.
    pub posterize: u32,
    /// The palette every pixel is moved to the nearest colour of.
    pub palette: Option<PaletteHandle>,
    pub dither: DitherPattern,
    /// The dither's amplitude in steps.
    pub dither_strength: f32,
    /// One dither cell in logical pixels.
    pub dither_scale: u32,
}

/// Halftone dots on a rotated grid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Halftone {
    pub strength: f32,
    /// The grid's pitch in logical pixels.
    pub size: f32,
    pub angle_degrees: f32,
    /// The dots' linear colour.
    pub color: [f32; 3],
}

/// Cross-hatching from a tonal art map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hatching {
    pub strength: f32,
    /// Line spacing in logical pixels.
    pub scale: f32,
    pub levels: u32,
    pub angle_degrees: f32,
    pub color: [f32; 3],
}

/// Painterly smoothing, pigment pooling and a canvas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Paint {
    /// The Kuwahara radius in logical pixels (0: no smoothing).
    pub radius: f32,
    /// 0 square cells, 1 strokes along the image's edges.
    pub anisotropy: f32,
    pub edge_darkening: f32,
    pub bleed: f32,
    /// The paper or canvas texture and how strongly it modulates the frame.
    pub canvas: Option<crate::TextureHandle>,
    pub canvas_strength: f32,
    /// Canvas texels per logical pixel.
    pub canvas_scale: f32,
}

/// Ink lines on luminance edges, and with `depth` on the prepass's depth
/// and normal edges too.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ink {
    pub threshold: f32,
    /// Line width in logical pixels.
    pub width: f32,
    pub color: [f32; 3],
    /// How strongly lines are drawn where the depth-normal prepass breaks
    /// (silhouettes against what is behind, creases), 0 to 1: lines on
    /// flat colour that luminance alone misses. 0 is luminance only; above
    /// 0 the frame runs the prepass.
    pub depth: f32,
}

/// Flat colour bands by depth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DepthBands {
    /// 2 to 8 bands.
    pub count: u32,
    pub near: f32,
    pub far: f32,
    pub near_color: [f32; 3],
    pub far_color: [f32; 3],
    /// 1 flat silhouettes, 0 a tint.
    pub flatten: f32,
}

/// The vertex and depth artefacts of early 3D hardware.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Retro3d {
    /// The screen grid vertices snap to, in logical pixels (0: none).
    pub vertex_snap: f32,
    /// 0 perspective-correct, 1 affine texture coordinates.
    pub affine: f32,
    /// Depth quantised to this many bits (0: full precision).
    pub depth_bits: u32,
}

/// Stylised defaults for every lit material that leaves the field at its
/// own default (the IDL's `MaterialStyle`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MaterialStyle {
    pub toon: bool,
    pub gradient_map: Option<crate::TextureHandle>,
    /// The linear colour shadowed light moves towards, and how far.
    pub shadow_tint: [f32; 3],
    pub shadow_tint_amount: f32,
    pub rim_color: [f32; 3],
    pub rim_intensity: f32,
    pub rim_power: f32,
    pub outline_color: [f32; 3],
    pub outline_width: f32,
    /// The distance at which outlines are `outline_width` wide (0: constant).
    pub outline_distance: f32,
    pub flat_shading: bool,
}

impl MaterialStyle {
    /// Whether it changes any material.
    pub fn is_active(&self) -> bool {
        self.toon
            || self.shadow_tint_amount > 0.0
            || self.rim_intensity > 0.0
            || self.outline_width > 0.0
            || self.flat_shading
    }
}

/// The stylisation of one frame. Every primitive is off by default.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stylise {
    pub pixel: Option<PixelScale>,
    pub quantize: Option<Quantize>,
    pub halftone: Option<Halftone>,
    pub hatching: Option<Hatching>,
    pub paint: Option<Paint>,
    pub ink: Option<Ink>,
    pub depth_bands: Option<DepthBands>,
    pub retro: Option<Retro3d>,
    pub material: MaterialStyle,
}

impl Stylise {
    /// Whether any primitive is on.
    pub fn is_active(&self) -> bool {
        self.pixel.is_some()
            || self.quantize.is_some()
            || self.halftone.is_some()
            || self.hatching.is_some()
            || self.paint.is_some()
            || self.ink.is_some()
            || self.depth_bands.is_some()
            || self.retro.is_some()
            || self.material.is_active()
    }

    /// Which primitives are on, one bit each (not their settings): what
    /// passes the frame runs (depth bands, paint, lines, quantise), whether
    /// it is drawn on a pixel grid, and whether retro 3D or the material
    /// style change the lit pass. The renderer resets the TAA history when
    /// this changes between two frames, as on a camera cut, so the first
    /// frame of a new look holds no ghost of the old one; a change of a
    /// setting alone (a posterize level, a palette) keeps it.
    pub fn chain_bits(&self) -> u16 {
        [
            self.pixel.is_some(),
            self.quantize.is_some(),
            self.halftone.is_some(),
            self.hatching.is_some(),
            self.paint.is_some(),
            self.ink.is_some(),
            self.depth_bands.is_some(),
            self.retro.is_some(),
            self.material.is_active(),
        ]
        .iter()
        .enumerate()
        .fold(0, |bits, (i, on)| bits | (u16::from(*on) << i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_chain_bits_follow_primitives_not_their_settings() {
        let off = Stylise::default();
        assert_eq!(off.chain_bits(), 0);
        let q = Quantize {
            posterize: 6,
            palette: None,
            dither: DitherPattern::Bayer4,
            dither_strength: 1.0,
            dither_scale: 1,
        };
        let six = Stylise {
            quantize: Some(q),
            ..off
        };
        let eight = Stylise {
            quantize: Some(Quantize { posterize: 8, ..q }),
            ..off
        };
        assert_ne!(six.chain_bits(), off.chain_bits(), "a pass added");
        assert_eq!(six.chain_bits(), eight.chain_bits(), "a setting only");
        let grid = Stylise {
            pixel: Some(PixelScale {
                height: 180,
                upscale: 0,
                snap: false,
            }),
            ..six
        };
        assert_ne!(grid.chain_bits(), six.chain_bits(), "the pixel grid on");
        let mut toon = six;
        toon.material.toon = true;
        assert_ne!(toon.chain_bits(), six.chain_bits(), "the material style");
    }

    #[test]
    fn everything_is_off_by_default() {
        assert!(!Stylise::default().is_active());
        let mut s = Stylise::default();
        s.material.outline_width = 1.0;
        assert!(s.is_active());
    }
}
