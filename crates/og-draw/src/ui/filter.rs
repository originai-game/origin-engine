//! SVG filter graphs over UI layers.

use super::UiBlend;
use crate::TextureHandle;
use glam::Vec3;

/// A filter: its primitives in order, each reading the source or earlier
/// results; the last one's result is the filter's.
#[derive(Clone, Debug, PartialEq)]
pub struct UiFilter {
    pub primitives: Vec<UiFilterPrimitive>,
    /// The affine map from the filter's space to CSS px, `[a, b, c, d, e,
    /// f]` (x' = a·x + c·y + e, y' = b·x + d·y + f); [`UiFilter::CSS`] is
    /// CSS px itself. A map that cannot be inverted draws nothing.
    pub space: [f32; 6],
}

impl UiFilter {
    /// The identity space: a filter written in CSS px.
    pub const CSS: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
}

impl Default for UiFilter {
    fn default() -> UiFilter {
        UiFilter {
            primitives: Vec::new(),
            space: UiFilter::CSS,
        }
    }
}

/// One primitive: what it does, its inputs (`in`, `in2`), its subregion
/// (`[x, y, width, height]` in the filter's space; outside it the result
/// is transparent) and whether it works on sRGB-encoded values.
#[derive(Clone, Debug, PartialEq)]
pub struct UiFilterPrimitive {
    pub op: UiFilterOp,
    pub inputs: [UiFilterInput; 2],
    pub subregion: [f32; 4],
    pub srgb: bool,
}

/// What a primitive reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiFilterInput {
    /// What the layer drew (SourceGraphic), or for a backdrop layer what
    /// is under it.
    #[default]
    Source,
    /// Its alpha, black (SourceAlpha).
    SourceAlpha,
    /// An earlier primitive's result, by index.
    Result(u32),
}

/// `feComposite`'s operators.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiFilterComposite {
    #[default]
    Over,
    In,
    Out,
    Atop,
    Xor,
    Lighter,
    /// k1·i1·i2 + k2·i1 + k3·i2 + k4.
    Arithmetic,
}

/// One channel's `feFunc*`.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum UiTransfer {
    #[default]
    Identity,
    /// Linear between the values (at most 16).
    Table(Vec<f32>),
    /// Steps of the values (at most 16).
    Discrete(Vec<f32>),
    Linear {
        slope: f32,
        intercept: f32,
    },
    Gamma {
        amplitude: f32,
        exponent: f32,
        offset: f32,
    },
}

/// A lighting primitive's light in the filter's space (z above the
/// surface, in its units).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiLight {
    /// Radians, from the space's x axis towards its y axis.
    Distant {
        azimuth: f32,
        elevation: f32,
    },
    Point {
        at: Vec3,
    },
    Spot {
        at: Vec3,
        points_at: Vec3,
        exponent: f32,
        /// The limiting cone's half angle, radians.
        cone: Option<f32>,
    },
}

/// What a primitive does. Lengths and positions are in the filter's space.
#[derive(Clone, Debug, PartialEq)]
pub enum UiFilterOp {
    /// `feGaussianBlur`: the standard deviation along the space's x and y.
    Blur { std_dev: [f32; 2] },
    /// `feOffset`.
    Offset { d: [f32; 2] },
    /// `feFlood`: straight linear RGBA.
    Flood { color: [f32; 4] },
    /// `feComposite` of `in` over `in2` by `op`, `k` for arithmetic.
    Composite { op: UiFilterComposite, k: [f32; 4] },
    /// `feBlend` of `in` over `in2`.
    Blend(UiBlend),
    /// `feColorMatrix` as a 4×5 matrix, rows R, G, B, A.
    ColorMatrix([f32; 20]),
    /// `feComponentTransfer`: R, G, B, A.
    ComponentTransfer(Box<[UiTransfer; 4]>),
    /// `feMorphology`: the radius along the space's x and y.
    Morphology { radius: [f32; 2], dilate: bool },
    /// `feDisplacementMap` of `in` by `in2`'s channels (0 R … 3 A).
    Displacement { scale: f32, channels: [u8; 2] },
    /// `feTurbulence` at noise points: `user` maps a point of the space
    /// to them (origin x, y, noise units per unit); `tile` is the stitch
    /// tile in noise units.
    Turbulence {
        base_frequency: [f32; 2],
        octaves: u32,
        seed: f32,
        stitch: bool,
        fractal: bool,
        user: [f32; 3],
        tile: [f32; 4],
    },
    /// `feImage` of a texture's `uv` rect over `rect`.
    Image {
        texture: TextureHandle,
        uv: [f32; 4],
        rect: [f32; 4],
    },
    /// `feImage` of an element (`href="#id"`): layer `layer` of the
    /// list, drawn where it lies on the screen and composited nowhere
    /// else; transparent outside its rect. It draws before this filter
    /// runs.
    Layer { layer: u32 },
    /// `feTile` of `in`, whose subregion is `tile`.
    Tile { tile: [f32; 4] },
    /// `feDiffuseLighting` (`specular` false: `constant` is kd) or
    /// `feSpecularLighting` (ks and its exponent), lit by `light` in
    /// linear `color`, from `in`'s alpha as a surface. The surface's
    /// normal is taken over cells of `kernel_unit` along the space's x
    /// and y (`kernelUnitLength`); `None` is one device px a cell.
    Lighting {
        specular: bool,
        surface_scale: f32,
        constant: f32,
        exponent: f32,
        color: [f32; 3],
        light: UiLight,
        kernel_unit: Option<[f32; 2]>,
    },
    /// One step of `feMerge`: `in` over `in2`.
    Merge,
    /// `feConvolveMatrix`: `order` columns and rows (at most 64 cells),
    /// the kernel row-major, a cell `kernel_unit` along the space's x and
    /// y (`kernelUnitLength`; `None` is one device px); `edge` 0
    /// duplicate, 1 wrap, 2 none.
    Convolve {
        order: [u32; 2],
        kernel: Vec<f32>,
        divisor: f32,
        bias: f32,
        target: [i32; 2],
        edge: u8,
        preserve_alpha: bool,
        kernel_unit: Option<[f32; 2]>,
    },
}
