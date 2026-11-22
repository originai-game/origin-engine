//! The UI's paint arena: what og-ui's tree paints, as flat `Pod` quads
//! og-render's paint pass draws with one pipeline (`ui_paint.wgsl`). The
//! tree refills it in place, so a repaint allocates nothing once its
//! vectors have grown; the renderer uploads it only when its generation
//! moves.
//!
//! A quad is a box in CSS pixels with its clip and colour and what fills
//! it ([`PaintKind`]): a solid rounded box, a gradient, a border ring, an
//! outer or inset box shadow, a glyph's raster in the glyph atlas
//! ([`crate::glyphs::Atlas`]), or a built-in icon drawn as its stroked
//! segments from [`Paint::icons`], a table made once and shared.

use bytemuck::{Pod, Zeroable};
use std::sync::Arc;

/// What a [`PaintQuad`] draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PaintKind {
    /// A box rounded by `radius`, in `color`.
    Solid = 0,
    /// A box rounded by `radius`, filled by gradient `a` of
    /// [`Paint::gradients`], at `color`'s alpha.
    Gradient = 1,
    /// Built-in icon segments `a..a + b` of [`Paint::icons`] (24-unit icon
    /// space, 2 units wide, round), in `rect`, in `color`.
    Icon = 3,
    /// The ring between the box rounded by `radius` and it inset by `p`'s
    /// widths (top, right, bottom, left), in `color`: a border, or an
    /// outline drawn as the ring of a box grown around its owner. The sides
    /// in `b` (bits: 1 top, 2 right, 4 bottom, 8 left) are in `uv`'s colour
    /// instead, split at the corners as CSS joins them.
    Border = 4,
    /// The box's outer shadow, outside the box only: the box moved by `p`'s
    /// x and y, grown by its spread (w) and blurred by its blur (z, CSS's
    /// blur radius, a Gaussian of half its deviation), in `color`.
    Shadow = 5,
    /// An inset shadow, inside the box only: the box less its hole (the box
    /// moved by `p`'s x and y, shrunk by its spread, blurred by its blur).
    InsetShadow = 6,
    /// Region `uv` of texture `a` ([`crate::TextureHandle`]'s bits) over
    /// `rect`, its colour times `color` (white for a picture, the ink for
    /// a mask), nine-sliced: `p` the slice insets in texture pixels (top,
    /// right, bottom, left) drawn `radius` CSS px wide on screen, the
    /// edges and centre stretched or tiled ([`IMAGE_TILE`]), the centre
    /// drawn with [`IMAGE_FILL`] (flags in `b`; no insets and the fill:
    /// the plain image). `wipe` cuts it radially ([`PaintQuad::wipe`]).
    Image = 7,
    /// A polyline (`<Line>`): points `a..a + b` of [`Paint::points`]
    /// (CSS px from `rect`'s top left), stroked `p`'s x wide in `color`,
    /// round joins and caps, closed when `p`'s y is 1.
    Line = 8,
    /// A glyph's raster in the glyph atlas ([`crate::glyphs::Atlas`]):
    /// region `uv` (x0 y0 x1 y1, atlas pixels) of page `b` of tier `a`
    /// ([`crate::glyphs::GlyphTier`]) over `rect`, which whoever painted
    /// it placed on whole device pixels at the raster's own size when the
    /// text is axis-aligned. Coverage draws in `color`, its alpha the
    /// coverage; a colour glyph in its own colours at `color`'s alpha; a
    /// distance field in `color`, grown by `p`'s x and blurred by
    /// its y (CSS px), its field `p`'s z atlas pixels across.
    Raster = 9,
}

/// One quad of the arena (`struct PaintIn` in ui_paint.wgsl's terms
/// before the renderer resolves glyphs).
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct PaintQuad {
    /// x, y, width, height, CSS px.
    pub rect: [f32; 4],
    /// x0, y0, x1, y1, CSS px.
    pub clip: [f32; 4],
    /// Linear RGBA, straight alpha.
    pub color: [f32; 4],
    /// Corner radii, top left, top right, bottom right, bottom left.
    pub radius: [f32; 4],
    /// The kind's parameters: a glyph's thickening in x, a border's widths,
    /// a shadow's offset, blur and spread.
    pub p: [f32; 4],
    pub kind: u32,
    pub a: u32,
    pub b: u32,
    /// Its transform, of [`Paint::transforms`]; 0 none.
    pub xf: u32,
    /// An image's texture region, u0 v0 u1 v1.
    pub uv: [f32; 4],
    /// An image's radial wipe: the fraction hidden (0 none), the angle it
    /// starts at (radians, 0 up, clockwise), 1 to sweep counter-clockwise.
    pub wipe: [f32; 4],
    /// The mask over it (`mask-[<gradient>]` on it or an ancestor): gradient
    /// `mask - 1` of [`Paint::gradients`], whose alpha over its
    /// [`PaintGradient::area`] scales the quad's; 0 none.
    pub mask: u32,
    /// The colour matrix over it (`brightness-*`, `grayscale` and the other
    /// colour filters on it or an ancestor): `filter - 1` of
    /// [`Paint::filters`]; 0 none.
    pub filter: u32,
}

/// A colour filter as one matrix (CSS Filter Effects' colour functions
/// composed, applied to the sRGB-encoded, unpremultiplied colour): rows r,
/// g, b, a of coefficients on (r, g, b, a), then the offsets added to each.
pub type ColorMatrix = [[f32; 4]; 5];

/// The matrix that leaves a colour as it is.
pub const COLOR_IDENTITY: ColorMatrix = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
    [0.0; 4],
];

/// `outer` applied after `inner`: one matrix for both.
pub fn color_then(inner: &ColorMatrix, outer: &ColorMatrix) -> ColorMatrix {
    let mut m = [[0.0; 4]; 5];
    for i in 0..4 {
        for j in 0..4 {
            m[i][j] = (0..4).map(|k| outer[i][k] * inner[k][j]).sum();
        }
        m[4][i] = (0..4).map(|k| outer[i][k] * inner[4][k]).sum::<f32>() + outer[4][i];
    }
    m
}

/// [`PaintKind::Image`]'s flags: draw the centre slice.
pub const IMAGE_FILL: u32 = 1;
/// Tile the edges and the centre (CSS's `round`) instead of stretching.
pub const IMAGE_TILE: u32 = 2;

/// A gradient: `stops` of [`Paint::stops`] from `first`. A
/// [`GRADIENT_LINEAR`] one runs along the CSS angle `angle` (radians, 0 up,
/// clockwise); a [`GRADIENT_RADIAL`] one out from `center` (CSS px from the
/// box's centre) to the ellipse of `radii` (CSS px); a [`GRADIENT_CONIC`] one
/// around `center` from `angle`, its offsets turns.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct PaintGradient {
    pub angle: f32,
    pub first: u32,
    pub count: u32,
    pub shape: u32,
    pub center: [f32; 2],
    pub radii: [f32; 2],
    /// A mask's box, x y w h in CSS px before transforms, which `center`
    /// and the line's length are measured in ([`PaintQuad::mask`]); a
    /// background's is its own quad's, so it is unused there.
    pub area: [f32; 4],
}

/// [`PaintGradient::shape`]: along a line.
pub const GRADIENT_LINEAR: u32 = 0;
/// Out from a centre.
pub const GRADIENT_RADIAL: u32 = 1;
/// Around a centre.
pub const GRADIENT_CONIC: u32 = 2;

/// A gradient's stop: its offset (0..1) and its colour, linear RGBA.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct PaintStop {
    pub color: [f32; 4],
    pub offset: f32,
    pub _pad: [f32; 3],
}

/// The identity of [`Paint::transforms`].
pub const IDENTITY: [f32; 8] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];

/// A built-in icon's stroke segment, `[ax, ay, bx, by]` in its 24-unit
/// space.
pub type IconSegment = [f32; 4];

/// The UI's paint: the quads in painter's order, their gradients and the
/// built-in icons' segments.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Paint {
    /// The CSS px size it was laid out in (the overlay's: fitted to the
    /// output's height).
    pub width: f32,
    pub height: f32,
    pub quads: Vec<PaintQuad>,
    pub gradients: Vec<PaintGradient>,
    pub stops: Vec<PaintStop>,
    /// The colour matrices [`PaintQuad::filter`] names.
    pub filters: Vec<ColorMatrix>,
    /// Paint-time transforms, CSS's `matrix(a, b, c, d, e, f)` on CSS px
    /// as `[a, b, c, d, e, f, 0, 0]` (x' = a x + c y + e); entry 0 is the
    /// identity, which the shader skips.
    pub transforms: Vec<[f32; 8]>,
    /// The points of the [`PaintKind::Line`] quads.
    pub points: Vec<[f32; 2]>,
    /// Every built-in icon's segments: one table, shared by every paint.
    pub icons: Option<Arc<[IconSegment]>>,
    /// Bumped each time anything above changes.
    pub generation: u64,
}

impl Paint {
    /// Empties it, keeping every vector's capacity and the icon table.
    pub fn clear(&mut self) {
        self.quads.clear();
        self.gradients.clear();
        self.stops.clear();
        self.filters.clear();
        self.points.clear();
        self.transforms.clear();
        self.transforms.push(IDENTITY);
    }

    /// Adds a linear gradient of `stops` (offset, linear RGBA) at `angle`;
    /// its index for [`PaintKind::Gradient`].
    pub fn gradient(
        &mut self,
        angle: f32,
        stops: impl IntoIterator<Item = (f32, [f32; 4])>,
    ) -> u32 {
        self.shaped_gradient(
            PaintGradient {
                angle,
                ..PaintGradient::zeroed()
            },
            stops,
        )
    }

    /// Adds gradient `g` (its shape, angle, centre and radii) of `stops`;
    /// its index for [`PaintKind::Gradient`].
    pub fn shaped_gradient(
        &mut self,
        g: PaintGradient,
        stops: impl IntoIterator<Item = (f32, [f32; 4])>,
    ) -> u32 {
        let first = self.stops.len() as u32;
        self.stops
            .extend(stops.into_iter().map(|(offset, color)| PaintStop {
                color,
                offset,
                _pad: [0.0; 3],
            }));
        self.gradients.push(PaintGradient {
            first,
            count: self.stops.len() as u32 - first,
            ..g
        });
        self.gradients.len() as u32 - 1
    }

    pub fn is_empty(&self) -> bool {
        self.quads.is_empty()
    }
}
