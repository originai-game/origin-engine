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
/// `depthBands`, `retro3d`, `surfaceSpace` (the canvas, hatching and
/// halftone laid on surfaces, [`StyliseSpace::Surface`]) and `objectSpace`
/// (laid in each instance's own space, [`StyliseSpace::Object`])
/// ([`Stylise`]);
/// `toon`, `shadowTint`, `rim`, `outline`, `outlineDistance` and
/// `flatShading` ([`MaterialStyle`]).
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
    "surfaceSpace",
    "objectSpace",
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

/// Where a pattern (the canvas, the hatching, the halftone) is laid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StyliseSpace {
    /// Over the frame, in logical pixels: the pattern stays put on the
    /// screen as the camera moves.
    #[default]
    Screen,
    /// On the surfaces, through the depth-normal prepass's position and
    /// normal (triplanar in world space): the pattern sticks to the scene
    /// as the camera moves and wraps round forms. The sky keeps the screen
    /// pattern. The frame runs the prepass.
    Surface,
    /// On the surfaces as [`StyliseSpace::Surface`], but triplanar in each
    /// instance's own space: a moving or turning object carries its
    /// strokes, dots and grain instead of sliding through them. The frame
    /// runs the prepass and then the object pass (`stylise_object.rs`),
    /// which draws the opaque mesh instances again into an `Rgba32Uint`
    /// target holding each pixel's position in its instance's space
    /// (metres: the mesh position times the instance's axis scales, so
    /// the pattern keeps its size on a scaled instance) and its normal
    /// there. Pixels no instance covers (terrain, scattered copies, a
    /// material shader's moved vertices) keep the world pattern, the sky
    /// the screen pattern.
    Object,
}

impl StyliseSpace {
    /// Laid on the surfaces (world or object space).
    pub fn on_surfaces(self) -> bool {
        self != StyliseSpace::Screen
    }

    /// The shader's code for the space: 0 screen, 1 world, 2 object.
    pub(crate) fn code(self) -> f32 {
        match self {
            StyliseSpace::Screen => 0.0,
            StyliseSpace::Surface => 1.0,
            StyliseSpace::Object => 2.0,
        }
    }
}

/// Halftone dots on a rotated grid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Halftone {
    pub strength: f32,
    /// The grid's pitch in logical pixels (on surfaces: about that many
    /// pixels where the dots lie, by whole octaves).
    pub size: f32,
    pub angle_degrees: f32,
    /// The dots' linear colour.
    pub color: [f32; 3],
    pub space: StyliseSpace,
}

/// Cross-hatching from a tonal art map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hatching {
    pub strength: f32,
    /// Line spacing in logical pixels (on surfaces: about that many pixels
    /// where the strokes lie; strokes fade in and out by whole octaves of
    /// world spacing, as a tonal art map's levels).
    pub scale: f32,
    pub levels: u32,
    pub angle_degrees: f32,
    pub color: [f32; 3],
    pub space: StyliseSpace,
    /// How strongly the depth-normal prepass's silhouettes and creases
    /// darken the tone the strokes follow, 0 to 1, as [`Ink::depth`] inks
    /// them: strokes gather along contours and folds that the tone alone
    /// leaves light. The edges are [`Ink`]'s (`depth_threshold`,
    /// `normal_threshold`; the outline pass's defaults without ink), found
    /// within one stroke spacing. 0 is the tone alone; above 0 the frame
    /// runs the prepass.
    pub depth: f32,
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
    /// Canvas texels per logical pixel ([`StyliseSpace::Screen`]).
    pub canvas_scale: f32,
    /// Where the canvas is laid: over the frame, or on the surfaces so the
    /// grain does not slide over the scene as the camera moves.
    pub canvas_space: StyliseSpace,
    /// Canvas texels per metre on surfaces ([`StyliseSpace::Surface`]);
    /// 0 is [`DEFAULT_CANVAS_WORLD_SCALE`].
    pub canvas_world_scale: f32,
}

/// [`Paint::canvas_world_scale`] when it is 0: a 256² canvas repeats every
/// two metres.
pub const DEFAULT_CANVAS_WORLD_SCALE: f32 = 128.0;

impl Paint {
    /// Whether the canvas is drawn on surfaces (the frame runs the prepass).
    pub fn canvas_on_surfaces(&self) -> bool {
        self.canvas_space.on_surfaces() && self.canvas.is_some() && self.canvas_strength > 0.0
    }
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
    /// With `depth`: the relative depth step to a farther neighbour that is
    /// a silhouette (0.04: 4 %), as `Outline::depth_threshold`.
    pub depth_threshold: f32,
    /// With `depth`: the normal turn between neighbours of about the same
    /// depth that is a crease (1 − cos of the angle; 0.4 ≈ 53°), as
    /// `Outline::normal_threshold`.
    pub normal_threshold: f32,
}

impl Default for Ink {
    /// The IDL's defaults: luminance edges only, the outline pass's
    /// thresholds for depth edges.
    fn default() -> Self {
        Ink {
            threshold: 0.1,
            width: 1.0,
            color: [0.02, 0.02, 0.03],
            depth: 0.0,
            depth_threshold: 0.04,
            normal_threshold: 0.4,
        }
    }
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
    /// Whether a pattern is laid on surfaces: the frame runs the
    /// depth-normal prepass and the paint or lines pass reads it.
    pub fn on_surfaces(&self) -> bool {
        self.surface_spaces()
            .iter()
            .any(|s| s.is_some_and(StyliseSpace::on_surfaces))
    }

    /// Whether a pattern is laid in object space: the frame runs the
    /// object pass after the prepass.
    pub fn in_object_space(&self) -> bool {
        self.surface_spaces().contains(&Some(StyliseSpace::Object))
    }

    /// The space of the canvas, the hatching and the halftone where each
    /// is drawn (`None` where it is not).
    fn surface_spaces(&self) -> [Option<StyliseSpace>; 3] {
        [
            self.paint
                .filter(|p| p.canvas.is_some() && p.canvas_strength > 0.0)
                .map(|p| p.canvas_space),
            self.hatching.filter(|h| h.strength > 0.0).map(|h| h.space),
            self.halftone.filter(|h| h.strength > 0.0).map(|h| h.space),
        ]
    }

    /// Whether a pass reads the depth-normal prepass's edges: ink or
    /// hatching with `depth` above 0.
    pub fn depth_edges(&self) -> bool {
        self.ink.is_some_and(|i| i.depth > 0.0)
            || self
                .hatching
                .is_some_and(|h| h.depth > 0.0 && h.strength > 0.0)
    }

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
    fn surface_space_asks_for_the_prepass_only_where_it_draws() {
        let mut s = Stylise::default();
        assert!(!s.on_surfaces());
        let paint = Paint {
            radius: 0.0,
            anisotropy: 0.0,
            edge_darkening: 0.0,
            bleed: 0.0,
            canvas: None,
            canvas_strength: 0.5,
            canvas_scale: 1.0,
            canvas_space: StyliseSpace::Surface,
            canvas_world_scale: 0.0,
        };
        s.paint = Some(paint);
        assert!(!s.on_surfaces(), "a surface canvas without a texture");
        s.hatching = Some(Hatching {
            strength: 0.8,
            scale: 6.0,
            levels: 4,
            angle_degrees: 45.0,
            color: [0.0; 3],
            space: StyliseSpace::Screen,
            depth: 0.0,
        });
        assert!(!s.on_surfaces(), "screen hatching");
        assert!(!s.depth_edges(), "hatching on the tone alone");
        s.hatching.as_mut().unwrap().depth = 0.5;
        assert!(
            s.depth_edges(),
            "hatching from depth edges reads the prepass"
        );
        s.hatching.as_mut().unwrap().depth = 0.0;
        s.hatching.as_mut().unwrap().space = StyliseSpace::Surface;
        assert!(s.on_surfaces());
        assert!(!s.in_object_space(), "world space runs no object pass");
        s.hatching.as_mut().unwrap().space = StyliseSpace::Object;
        assert!(s.on_surfaces() && s.in_object_space());
        s.hatching.as_mut().unwrap().strength = 0.0;
        assert!(!s.in_object_space(), "only where the pattern draws");
        s.hatching.as_mut().unwrap().strength = 0.8;
        s.hatching.as_mut().unwrap().space = StyliseSpace::Surface;
        assert_eq!(
            s.chain_bits(),
            Stylise {
                hatching: Some(Hatching {
                    space: StyliseSpace::Screen,
                    ..s.hatching.unwrap()
                }),
                ..s
            }
            .chain_bits(),
            "the space is a setting, not a pass"
        );
    }

    #[test]
    fn everything_is_off_by_default() {
        assert!(!Stylise::default().is_active());
        let mut s = Stylise::default();
        s.material.outline_width = 1.0;
        assert!(s.is_active());
    }
}
