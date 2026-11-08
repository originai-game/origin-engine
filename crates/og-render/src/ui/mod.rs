//! The UI pass (`docs/ui.md` §3.3): analytic primitives drawn in the overlay's
//! order, after every post effect. Every shape is a signed distance
//! function evaluated per pixel (`shaders/ui.wgsl`), so edges are exact at
//! any scale and transform; every paint, stroke and effect of a shape is
//! one instance of one family of pipelines.
//!
//! What a frame submits is [`UiList`] ([`crate::Overlay::ui`]):
//!
//! - [`UiShape`]s: a geometry (rectangle with four radii and a corner
//!   shape, ellipse, arc, polygon or star, polyline), a paint (solid,
//!   gradient, image, an offscreen layer), a mode (fill, stroke, drop or
//!   inner shadow, backdrop), opacity and blend. A layered element is
//!   several shapes on one painter level, in order.
//! - [`UiClip`]s: a clip rectangle of [`crate::Overlay::clips`] that also
//!   cuts what uses it to a shape, antialiased; sprites and text included.
//! - [`UiLayer`]s: the overlay layers in a range drawn offscreen, then
//!   composited by a [`UiShape`] whose paint is [`UiPaint::Layer`] through a
//!   built-in or custom material ([`UiMaterialShader`]).
//! - [`UiTextEffects`]: outline, gradient fill and shadows of one overlay
//!   text, read from its font's distance field.
//! - [`UiVectorDraw`]s: a cooked vector ([`UiVectorData`], tessellated at
//!   cook with an antialiasing fringe) drawn into a rectangle.
//! - [`UiMassBatch`]es: mass elements ([`mass`], §14), each an instance of
//!   a template whose motion and number formatting run in the shader.
//! - [`UiOccluder`]s: a depth test against the scene for what uses a clip
//!   rectangle (anchored elements behind a wall fade or hide).
//!
//! Sorting and batching (§5.7) are the overlay's: by layer, then by
//! pipeline and texture; consecutive instances of one batch are one draw.
//! A layer starts passes of its own for its region only; a backdrop or a
//! blend that reads what is under it copies the frame drawn so far, as
//! glass does (src/glass.rs).

pub(crate) mod build;
pub mod glass;
pub mod mass;
pub(crate) mod pass;
pub mod path;

pub use glass::{
    GLASS_BLUR_SCALE, UiElevation, UiGlassFallback, UiGlassStyle, UiGlassWeight, ui_scrim,
};
pub use mass::{
    UI_MASS_BATCHES, UI_MASS_KEYS, UI_MASS_QUADS, UiMassBatch, UiMassInstance, UiMassKey,
    UiMassKind, UiMassSpace, UiMassTemplate, UiOccluder, UiOcclusion, UiParticleShape,
    format_number,
};
pub use path::{UiFillRule, UiPath, UiVerb};

use crate::TextureHandle;
use crate::hooks::{ParamKind, ShaderParam};
use glam::Vec2;

/// How the corners of a [`UiGeometry::Rect`] are shaped.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum UiCorner {
    /// Circular arcs of the corner's radius.
    #[default]
    Round,
    /// Continuous corners: a superellipse of exponent `n` (4 is the iOS
    /// look; 2 is round).
    Squircle(f32),
    /// A straight cut across the corner, `radius` along each edge.
    Bevel,
    /// A concave quarter circle cut into the corner (an inset notch).
    Notch,
    /// CSS's corners (CSS Backgrounds 3 §5.1, CSS Borders 4 §corner-shape):
    /// [`UiGeometry::Rect`]'s radii are the horizontal ones, these the rest.
    Css(UiCssCorners),
}

/// Per-corner CSS corners, top left, top right, bottom right, bottom left.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UiCssCorners {
    /// The vertical radii, CSS px (each clamped to the half height); the
    /// horizontal ones are the rectangle's `radii`.
    pub vertical: [f32; 4],
    /// Each corner's `superellipse(K)`: 1 round, 2 squircle, 0 bevel, −1
    /// scoop, −∞ notch, +∞ square; finite values are kept to quarter steps
    /// between −7.75 and 7.5. The default (0) is a bevel: give 1 for round.
    pub shape: [f32; 4],
}

/// The end of an open line or arc.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiCap {
    #[default]
    Butt,
    Round,
    /// Extended by half the width, square.
    Square,
}

/// Where two segments of a polyline meet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiJoin {
    /// Sharp, cut to a bevel past `limit` half-widths from the joint.
    Miter(f32),
    Round,
    Bevel,
}

impl Default for UiJoin {
    fn default() -> UiJoin {
        UiJoin::Miter(4.0)
    }
}

/// A shape's outline, in its local space: centred on [`UiShape::center`],
/// `half` its half extent (CSS px), y down.
#[derive(Clone, Debug, PartialEq)]
pub enum UiGeometry {
    /// A rectangle of `half`: corner radii top left, top right, bottom
    /// right, bottom left (each clamped to the shorter half side). A
    /// capsule is a rectangle whose radii are its half height.
    Rect { radii: [f32; 4], corner: UiCorner },
    /// The ellipse inscribed in `half` (a circle when it is square).
    Ellipse,
    /// A ring segment on the circle of radius min(`half`): `start` radians
    /// clockwise from +x, `sweep` radians clockwise (2π or more: a ring),
    /// `thickness` CSS px inwards from the circle.
    Arc {
        start: f32,
        sweep: f32,
        thickness: f32,
        cap: UiCap,
    },
    /// A regular polygon of `sides` vertices on the circle of radius
    /// min(`half`), the first at the top turned by `rotation`; with `inner`
    /// under cos(π/sides) a star whose inner vertices are at `inner` times
    /// the radius. `rounding` rounds the corners (CSS px).
    Polygon {
        sides: u32,
        inner: f32,
        rounding: f32,
        rotation: f32,
    },
    /// Line segments through `points` (local, relative to the centre) of
    /// `width` CSS px; `half` is ignored (the bounds are the points'). A
    /// Bézier is flattened into one ([`flatten_cubic`], [`flatten_quad`]).
    Polyline {
        points: Vec<Vec2>,
        closed: bool,
        /// Closed and filled: the area inside the points (even-odd), which
        /// fills cover and strokes outline (an area chart, a radar's polygon).
        filled: bool,
        width: f32,
        cap: UiCap,
        join: UiJoin,
    },
    /// A vector path ([`UiPath`], local, relative to the centre) of lines
    /// and Béziers, nothing flattened: fills cover it by its fill rule,
    /// strokes outline it (round where it turns, as a band around a
    /// distance; [`UiPath::flatten`] into polylines for joins, caps and
    /// dashes), shadows and glows take its outline. The paint's box is the
    /// shape's (`half`), so a path may reach past it.
    Path(std::sync::Arc<UiPath>),
}

/// How a gradient's parameter runs over the shape's box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiGradientKind {
    /// From `from` to `to`.
    #[default]
    Linear,
    /// Centred on `from`, `to` its x and y radii.
    Radial,
    /// Around `from`, starting at angle `to.x` (radians clockwise from +x).
    Conic,
    /// Centred on `from`, `to` its x and y radii, |x|/rx + |y|/ry.
    Diamond,
}

/// Past the first and last stop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiSpread {
    #[default]
    Pad,
    Repeat,
    Reflect,
}

/// A gradient stop: `offset` 0 to 1 and a linear RGBA colour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiStop {
    pub offset: f32,
    pub color: [f32; 4],
}

/// The most stops a gradient keeps (more are dropped).
pub const UI_GRADIENT_STOPS: usize = 8;

/// A gradient, interpolated in OKLab with premultiplied alpha. `from` and
/// `to` are fractions of the shape's box (0,0 top left, 1,1 bottom right);
/// [`UiGradient::units`] gives them in CSS px of an absolute box instead
/// (text, whose box is not a shape's).
#[derive(Clone, Debug, PartialEq)]
pub struct UiGradient {
    pub kind: UiGradientKind,
    pub from: Vec2,
    pub to: Vec2,
    pub stops: Vec<UiStop>,
    pub spread: UiSpread,
    /// Adds a pixel's worth of noise before the output's 8 bits, so a long
    /// shallow gradient shows no bands.
    pub dither: bool,
}

impl Default for UiGradient {
    fn default() -> UiGradient {
        UiGradient {
            kind: UiGradientKind::Linear,
            from: Vec2::ZERO,
            to: Vec2::new(0.0, 1.0),
            stops: Vec::new(),
            spread: UiSpread::Pad,
            dither: true,
        }
    }
}

/// How a nine-slice's edges and centre fill their length.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiEdge {
    #[default]
    Stretch,
    /// Tiled at the border's scale, cut at the end.
    Repeat,
    /// Tiled a whole number of times, stretched to fit.
    Round,
}

/// How an image fills the shape's box.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum UiFit {
    /// Stretched to the box.
    #[default]
    Fill,
    /// Scaled to cover the box, cropped around the focal point.
    Cover,
    /// Scaled to fit inside the box, placed by the focal point.
    Contain,
    /// At its own size (one image pixel per CSS px), placed by the focal point.
    None,
    /// Nine-slice: `source` the border in image pixels (left, top, right,
    /// bottom), `dest` the border drawn in CSS px; the edges and the centre
    /// fill by `edges`; `center` false leaves the centre empty.
    NineSlice {
        source: [f32; 4],
        dest: [f32; 4],
        edges: UiEdge,
        center: bool,
    },
    /// The image tiled `tile` CSS px, turned by `rotation` radians, moved by
    /// `offset` and scrolling by `scroll` CSS px a second of the UI clock.
    Pattern {
        tile: Vec2,
        rotation: f32,
        offset: Vec2,
        scroll: Vec2,
    },
    /// A CSS background layer's image (CSS Backgrounds 3 §3.6–3.9).
    Background(UiBackground),
}

/// A CSS background layer's image in the shape's box: sized, placed and
/// repeated per axis. Its natural size is its texture region's texels,
/// one a CSS px.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UiBackground {
    pub size: UiBackgroundSize,
    /// Where the image (the first tile) sits on each axis, CSS px from the
    /// box's top left: `align × (box − image) + offset` (`right 10px` is
    /// align 1, offset −10).
    pub align: Vec2,
    pub offset: Vec2,
    /// x, then y.
    pub repeat: [UiRepeat; 2],
}

/// `background-size`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiBackgroundSize {
    /// Scaled to cover the box, keeping its ratio.
    Cover,
    /// Scaled to fit inside the box, keeping its ratio.
    Contain,
    /// Width and height in CSS px; `None` is `auto` (from the other
    /// through the image's ratio, else the image's own).
    Given([Option<f32>; 2]),
}

impl Default for UiBackgroundSize {
    fn default() -> UiBackgroundSize {
        UiBackgroundSize::Given([None; 2])
    }
}

/// `background-repeat` on one axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiRepeat {
    #[default]
    Repeat,
    NoRepeat,
    /// As many whole images as fit, the first and last at the edges and
    /// the room left shared between them (one placed by its position when
    /// fewer than two fit).
    Space,
    /// Rescaled so a whole number fit (and the other axis, when its size
    /// is `auto`, keeps the ratio).
    Round,
}

/// An image paint: a region `uv` (u0 v0 u1 v1) of `texture`, `size` its
/// pixels, tinted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiImage {
    pub texture: TextureHandle,
    pub uv: [f32; 4],
    pub size: Vec2,
    pub tint: [f32; 4],
    pub fit: UiFit,
    /// Where cover crops and contain or none place the image, fractions of
    /// the free space (0.5, 0.5: centred).
    pub focal: Vec2,
}

/// What fills a shape's coverage.
#[derive(Clone, Debug, PartialEq)]
pub enum UiPaint {
    /// Linear RGBA.
    Solid([f32; 4]),
    Gradient(UiGradient),
    Image(UiImage),
    /// [`UiList::layers`]' layer `n`, drawn offscreen, through its material.
    Layer(u32),
}

/// Which side of the outline a stroke lies on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiAlign {
    Inside,
    #[default]
    Center,
    Outside,
}

/// A stroke along a shape's outline (a polyline's is its own line).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UiStroke {
    pub width: f32,
    pub align: UiAlign,
    /// Dash and gap lengths, CSS px along the outline (0: solid).
    pub dash: [f32; 2],
    pub dash_offset: f32,
    /// The ends of each dash.
    pub dash_cap: UiCap,
}

/// What an instance draws of its shape.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum UiMode {
    /// The shape's inside.
    #[default]
    Fill,
    Stroke(UiStroke),
    /// The shape moved by `offset`, grown by `spread`, blurred by `blur`
    /// (the Gaussian's standard deviation is half of it, as CSS): a drop
    /// shadow, or with no offset an outer glow.
    Shadow {
        offset: Vec2,
        blur: f32,
        spread: f32,
    },
    /// The shadow of the outside cast inwards: an inner shadow, or with no
    /// offset an inner glow.
    InnerShadow {
        offset: Vec2,
        blur: f32,
        spread: f32,
    },
    /// What is drawn under the shape (the scene and the overlay below it),
    /// blurred by `blur` CSS px, its saturation and brightness scaled,
    /// tinted by a solid paint (its alpha how much tint covers it), with
    /// the material's hairline and shadows ([`UiGlass`]). Glass.
    Backdrop {
        blur: f32,
        saturation: f32,
        brightness: f32,
        glass: UiGlass,
    },
}

/// The rest of a backdrop's material (`docs/ui.md` §3.3), all off at its
/// zero value, which draws the plain frosted backdrop: flat glass as the
/// design language composes it (a light tint over a blur with raised
/// saturation, a hairline edge, a soft shadow), drawn by the backdrop's
/// one instance. No specular rim, sheen or refraction: the material stays
/// clear and takes its colour from what passes under it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UiGlass {
    /// A hairline just inside the outline (CSS `inset 0 0 0 <edge>px`):
    /// its width in CSS px; 0 is none.
    pub edge: f32,
    /// The hairline's linear RGBA.
    pub edge_color: [f32; 4],
    /// Soft shadows outside the shape only, as a CSS `box-shadow` under a
    /// translucent element: none darkens what the glass shows. The first
    /// lies over the second; a clear colour is none.
    pub shadows: [UiGlassShadow; 2],
    /// Nothing under it is read: the tint is drawn opaque, with the
    /// hairline and shadows (reduced transparency, increased contrast).
    /// A frame whose only backdrops are solid builds no blur pyramid.
    pub solid: bool,
    /// Mixes the tint and the hairline over the blur, and the shadows over
    /// the frame, in sRGB-encoded values, as CSS and the platform's glass
    /// do, instead of in linear light: a dark tint at a given alpha over a
    /// bright scene stays as dark as the same CSS tint (in linear light it
    /// lets several times more of the scene through). Saturation and
    /// brightness then scale encoded values too. The shadows' alpha is
    /// mapped so a black shadow darkens as CSS's does.
    pub srgb_mix: bool,
    /// The brightest the blurred backdrop may be before the tint, as a
    /// luminance in the space it mixes in (encoded with `srgb_mix`); 0 is
    /// no limit. Brighter backdrop is scaled down to it and loses its
    /// colour as it goes, so a sun behind dark glass lifts the panel evenly
    /// instead of staining it.
    pub backdrop_max: f32,
    /// A light along the top edge (concept B's lit edge): white at this
    /// alpha over the outline's top 1 CSS px, fading down the sides and
    /// towards the corners; 0 is none.
    pub edge_top: f32,
    /// A colour matrix over the blurred backdrop before its tint (CSS's
    /// `backdrop-filter` colour functions), in sRGB-encoded values whatever
    /// `srgb_mix` says; its opacity mixes the result over the unblurred
    /// backdrop. `None` is none.
    pub matrix: Option<UiColorMatrix>,
}

/// A colour matrix as CSS's filter functions compose (Filter Effects 1
/// §13): over straight sRGB-encoded colour, `rgb' = rows · (r, g, b, 1)`
/// clamped to 0..1, and alpha times `opacity`. The default is the
/// identity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiColorMatrix {
    pub rows: [[f32; 4]; 3],
    pub opacity: f32,
}

impl Default for UiColorMatrix {
    fn default() -> UiColorMatrix {
        UiColorMatrix {
            rows: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ],
            opacity: 1.0,
        }
    }
}

impl UiColorMatrix {
    /// As a layer's parameters for [`UiBuiltin::ColorMatrix`].
    pub fn params(&self) -> [[f32; 4]; UI_MATERIAL_PARAMS] {
        [
            self.rows[0],
            self.rows[1],
            self.rows[2],
            [self.opacity, 0.0, 0.0, 0.0],
        ]
    }
}

/// One of [`UiGlass::shadows`]: a CSS `box-shadow` (offset, blur, spread,
/// colour), drawn outside the shape only.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UiGlassShadow {
    pub offset: Vec2,
    /// The CSS blur (the Gaussian's standard deviation is half of it).
    pub blur: f32,
    pub spread: f32,
    /// Linear RGBA; alpha 0 is none.
    pub color: [f32; 4],
}

/// How a shape's colour combines with what is under it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UiBlend {
    #[default]
    Normal,
    Multiply,
    Screen,
    Add,
    /// Reads what is under the shape (a copy of the frame drawn so far).
    Overlay,
    SoftLight,
    ColorDodge,
    Darken,
    Lighten,
    ColorBurn,
    HardLight,
    Difference,
    Exclusion,
    /// The non-separable modes (Compositing and Blending 1 §10.2): the
    /// shape's hue, saturation, both, or luminosity over the backdrop's.
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl UiBlend {
    /// Whether the blend reads the destination in the shader: every mode
    /// but normal, multiply, screen and add, which fixed-function blending
    /// does.
    pub fn reads_destination(self) -> bool {
        !matches!(
            self,
            UiBlend::Normal | UiBlend::Multiply | UiBlend::Screen | UiBlend::Add
        )
    }
}

/// One instance of the UI pass. See the module documentation.
#[derive(Clone, Debug, PartialEq)]
pub struct UiShape {
    pub geometry: UiGeometry,
    /// The centre of the local space, CSS px.
    pub center: Vec2,
    /// The half extent of the shape in its local space.
    pub half: Vec2,
    /// The local space to the screen, a 2×2 matrix about the centre
    /// (`[a, b, c, d]`: x' = a·x + c·y, y' = b·x + d·y); identity
    /// `[1, 0, 0, 1]`. Rotation, scale and skew.
    pub transform: [f32; 4],
    pub paint: UiPaint,
    pub mode: UiMode,
    pub opacity: f32,
    pub blend: UiBlend,
    /// The overlay layer it sorts on (with sprites, text and glass).
    pub layer: i32,
    /// The clip rectangle of [`crate::Overlay::clips`] (with its shape, if
    /// a [`UiClip`] names it).
    pub clip: Option<u32>,
}

impl UiShape {
    /// A solid rectangle `[x, y, width, height]` with one corner radius.
    pub fn rect(rect: [f32; 4], radius: f32, color: [f32; 4]) -> UiShape {
        let [x, y, w, h] = rect;
        UiShape {
            geometry: UiGeometry::Rect {
                radii: [radius; 4],
                corner: UiCorner::Round,
            },
            center: Vec2::new(x + w / 2.0, y + h / 2.0),
            half: Vec2::new(w / 2.0, h / 2.0),
            transform: [1.0, 0.0, 0.0, 1.0],
            paint: UiPaint::Solid(color),
            mode: UiMode::Fill,
            opacity: 1.0,
            blend: UiBlend::Normal,
            layer: 0,
            clip: None,
        }
    }
}

/// A clip with a shape: what uses clip rectangle `rect` of
/// [`crate::Overlay::clips`] is also cut to this shape (antialiased), and to
/// `parent`'s, a [`UiList::clips`] index (nested rounded panels). A path
/// cuts by its fill rule, a polyline by its polygon (even-odd), each with
/// the path fill's analytic coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct UiClip {
    pub rect: u32,
    pub geometry: UiGeometry,
    pub center: Vec2,
    pub half: Vec2,
    pub transform: [f32; 4],
    pub parent: Option<u32>,
}

/// The built-in materials a layer composites through (§5.4); `params` of
/// [`UiLayer`] in the order each lists, the time the UI clock's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum UiBuiltin {
    /// The layer as drawn.
    #[default]
    None,
    /// [threshold 0–1, edge width, noise scale (CSS px), _], [edge colour].
    Dissolve,
    /// [position 0–1 across the diagonal, width, angle (rad), intensity],
    /// [colour]; a negative position sweeps by the clock (speed −position).
    Shine,
    /// [scanline spacing (CSS px), flicker 0–1, chromatic split (CSS px),
    /// scanline strength], [tint].
    Hologram,
    /// [strength 0–1, slice height (CSS px), split (CSS px), rate (Hz)].
    Glitch,
    /// [amplitude (CSS px), wavelength (CSS px), speed (CSS px/s), _],
    /// [centre x, y as fractions].
    Ripple,
    /// [cell size (CSS px)].
    Pixelate,
    /// [hue (rad), saturation, brightness, grayscale 0–1].
    ColorAdjust,
    /// [width (CSS px), _, _, _], [colour]: an outline of the layer's alpha.
    Outline,
    /// [radius (CSS px)]: a layer blur.
    Blur,
    /// [r row], [g row], [b row], [opacity, _, _, _]: a
    /// [`UiColorMatrix`] ([`UiColorMatrix::params`]).
    ColorMatrix,
    /// [offset x, offset y (CSS px), blur (CSS, the Gaussian's standard
    /// deviation half of it), _], [colour]: a shadow of the layer's alpha
    /// under it (CSS's `drop-shadow()`).
    DropShadow,
}

/// What a layer composites through.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum UiMaterial {
    Builtin(UiBuiltin),
    /// A scene's [`UiMaterialShader`] ([`crate::RenderScene::add_ui_material`]).
    Custom(UiMaterialHandle),
    #[default]
    Plain,
}

/// An alpha mask over a layer's composite.
#[derive(Clone, Debug, PartialEq)]
pub enum UiMask {
    /// The gradient's alpha over the layer's rectangle.
    Gradient(UiGradient),
    /// The image's alpha (`uv` of `texture`) stretched over the layer's rectangle.
    Image {
        texture: TextureHandle,
        uv: [f32; 4],
    },
}

/// The most parameter vectors a layer's material reads.
pub const UI_MATERIAL_PARAMS: usize = 4;

/// A subtree drawn offscreen: every sprite, text and shape whose layer is
/// in `capture` (inclusive) draws into a target of `rect` (`[x, y, width,
/// height]`, CSS px) instead of the overlay, and the [`UiShape`]s whose
/// paint is [`UiPaint::Layer`] of it composite it through `material`.
///
/// With a `target` (a world panel, `docs/ui.md` §1) the layer draws into
/// that render target's colour instead, `rect` stretched over the whole
/// target wherever it is on (or off) the screen, and composites nowhere:
/// a mesh's material samples the target (the frame after, since the UI
/// draws after the scene). The target is an
/// [`crate::TargetFormat::Rgba16Float`] one: linear, as a lit `map` or an
/// unlit `emissiveMap` reads it.
///
/// With a `ground` (a world panel the UI draws itself: a ground decal, a
/// sign) the layer draws offscreen at `rect` wherever that is, and its
/// composite lays `rect` onto its plane in the world as a decal is laid:
/// each screen point reads the scene's depth there and takes the layer
/// where that surface falls on the rect ([`UiGround`]). Its target is
/// sized from the size it is seen at (capped), not from `rect`.
#[derive(Clone, Debug, PartialEq)]
pub struct UiLayer {
    pub capture: [i32; 2],
    pub rect: [f32; 4],
    pub material: UiMaterial,
    pub params: [[f32; 4]; UI_MATERIAL_PARAMS],
    pub mask: Option<UiMask>,
    pub target: Option<TextureHandle>,
    pub ground: Option<UiGround>,
}

/// Where a projected layer ([`UiLayer::ground`]) lies: its rect on a
/// plane in the world, draped over the scene's surface.
///
/// A screen point takes the layer where the scene's surface under it
/// (the depth prepass's view depth) falls on the rect, along the plane's
/// normal, while that surface is within `reach` of the plane: terrain
/// rising and falling under a ground decal carries it, a surface facing
/// away from the plane's normal (a wall, a leg) fades it, and something
/// nearer the camera than that hides it. Where the surface lies farther
/// behind the plane, or there is none (no prepass, the sky), it is laid
/// on the plane itself, as a sign in the air is.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UiGround {
    /// Where the rect's top left, top right, bottom right and bottom left
    /// land on the screen, CSS px, when all four are in front of the
    /// camera: the target is sized from the size it is seen at. `None`
    /// sizes it at [`UI_QUAD_LAYER_MAX`].
    pub corners: Option<[Vec2; 4]>,
    pub drape: UiDrape,
}

/// The map from a screen point to a projected layer's plane: the point
/// (x, y) CSS px at view depth d (world units along the camera's forward
/// axis, as the depth prepass stores it) lies at `a · (x, y, 1) + d ·
/// (b · (x, y, 1))`, each a row: its x and y in the rect's own CSS px
/// (the layer's), and z world units off the plane.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UiDrape {
    pub a: [[f32; 3]; 3],
    pub b: [[f32; 3]; 3],
    /// World units per rect CSS px along x and y.
    pub scale: Vec2,
    /// How far off the plane, world units, the surface still takes it.
    pub reach: f32,
}

impl UiDrape {
    /// The drape of `rect` (x, y, width, height, CSS px) laid on the plane
    /// through world points `top_left`, `top_right` and `bottom_left` (its
    /// corners), seen through `view_proj` (depth 0 to 1, as og-render's)
    /// from `eye` looking along `forward` (unit) on a `screen` CSS px
    /// across. `None` when the plane or the view is degenerate.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        view_proj: glam::Mat4,
        eye: glam::Vec3,
        forward: glam::Vec3,
        screen: Vec2,
        corners: [glam::Vec3; 3],
        rect: [f32; 4],
        reach: f32,
    ) -> Option<UiDrape> {
        use glam::{DMat4, DVec3, DVec4};
        let inv = view_proj.as_dmat4().inverse();
        if !inv.is_finite() || screen.x <= 0.0 || screen.y <= 0.0 {
            return None;
        }
        let (eye, f) = (eye.as_dvec3(), forward.as_dvec3());
        let unproject = |m: &DMat4, x: f64, y: f64, z: f64| {
            let p = *m * DVec4::new(x, y, z, 1.0);
            p.truncate() / p.w
        };
        // The world point at view depth d under CSS px (x, y): its ray
        // through the near and far planes, cut at d.
        let at = |x: f64, y: f64, d: f64| -> Option<DVec3> {
            let nx = x / f64::from(screen.x) * 2.0 - 1.0;
            let ny = 1.0 - y / f64::from(screen.y) * 2.0;
            let near = unproject(&inv, nx, ny, 0.0);
            let dir = unproject(&inv, nx, ny, 1.0) - near;
            let k = dir.dot(f);
            (k.abs() > 1e-12).then(|| near + dir * ((d - (near - eye).dot(f)) / k))
        };
        let [tl, tr, bl] = corners.map(|c| c.as_dvec3());
        let (u, v) = (tr - tl, bl - tl);
        let n = u.cross(v).try_normalize()?;
        let (uu, vv) = (u.length_squared(), v.length_squared());
        if uu < 1e-12 || vv < 1e-12 {
            return None;
        }
        let [rx, ry, rw, rh] = rect.map(f64::from);
        // A world point's place: rect px across and down, world units off.
        let local = |p: DVec3| {
            let q = p - tl;
            DVec3::new(rx + q.dot(u) / uu * rw, ry + q.dot(v) / vv * rh, q.dot(n))
        };
        let dir = |p: DVec3| DVec3::new(p.dot(u) / uu * rw, p.dot(v) / vv * rh, p.dot(n));
        // A and B are affine in (x, y) for a perspective or an orthographic
        // view: read them at three screen points.
        let (sx, sy) = (f64::from(screen.x), f64::from(screen.y));
        let mut pa = [DVec3::ZERO; 3];
        let mut pb = [DVec3::ZERO; 3];
        for (k, (x, y)) in [(0.0, 0.0), (sx, 0.0), (0.0, sy)].into_iter().enumerate() {
            let p0 = at(x, y, 0.0)?;
            let p1 = at(x, y, 1.0)?;
            pa[k] = local(p0);
            pb[k] = dir(p1 - p0);
        }
        let rows = |p: [DVec3; 3]| -> [[f32; 3]; 3] {
            let cx = (p[1] - p[0]) / sx;
            let cy = (p[2] - p[0]) / sy;
            [0, 1, 2].map(|i| [cx[i] as f32, cy[i] as f32, p[0][i] as f32])
        };
        Some(UiDrape {
            a: rows(pa),
            b: rows(pb),
            scale: Vec2::new((uu.sqrt() / rw) as f32, (vv.sqrt() / rh) as f32),
            reach: reach.max(0.0),
        })
    }
}

/// The largest side of a projected layer's target, device pixels.
pub const UI_QUAD_LAYER_MAX: u32 = 2048;

/// The 3×3 map (rows) from the unit square's `(u, v, 1)` to `quad`'s
/// points in homogeneous coordinates: (0,0), (1,0), (1,1), (0,1) to its
/// first to fourth point (Heckbert's square-to-quad). `None`: the quad is
/// degenerate (three points on a line).
pub fn square_to_quad(quad: &[Vec2; 4]) -> Option<[[f32; 3]; 3]> {
    let [p0, p1, p2, p3] = quad.map(|p| p.as_dvec2());
    let s = p0 - p1 + p2 - p3;
    let d1 = p1 - p2;
    let d2 = p3 - p2;
    let den = d1.x * d2.y - d2.x * d1.y;
    if den.abs() < 1e-9 {
        return None;
    }
    let g = (s.x * d2.y - d2.x * s.y) / den;
    let h = (d1.x * s.y - s.x * d1.y) / den;
    let m = [
        [p1.x - p0.x + g * p1.x, p3.x - p0.x + h * p3.x, p0.x],
        [p1.y - p0.y + g * p1.y, p3.y - p0.y + h * p3.y, p0.y],
        [g, h, 1.0],
    ];
    Some(m.map(|r| r.map(|v| v as f32)))
}

/// An outline around overlay text, drawn from its font's distance field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiTextOutline {
    /// CSS px (up to the field's reach, half the font's distance range).
    pub width: f32,
    pub color: [f32; 4],
    pub align: UiAlign,
}

/// A shadow under overlay text: the glyphs moved by `offset`, softened by
/// `blur` CSS px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiTextShadow {
    pub offset: Vec2,
    pub blur: f32,
    pub color: [f32; 4],
}

/// Effects of [`crate::Overlay::texts`]' text `text` (§5.6): shadows (in
/// order, under everything), an outline, and a gradient fill whose `from`
/// and `to` are fractions of `fill_box` (`[x, y, width, height]`, CSS px).
/// `transform` maps the text's glyphs about its position (a 2×2 as
/// [`UiShape::transform`], after its rotation; zero or `[1, 0, 0, 1]`: none):
/// an element's turn, scale and skew, its effects' widths and offsets with it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiTextEffects {
    pub text: u32,
    pub shadows: Vec<UiTextShadow>,
    pub outline: Option<UiTextOutline>,
    pub fill: Option<UiGradient>,
    pub fill_box: [f32; 4],
    pub transform: [f32; 4],
}

/// A scene's cooked vector ([`crate::RenderScene::add_ui_vector`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UiVectorHandle(pub u32);

impl UiVectorHandle {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// A vector vertex: a position in the view box, the outline's outward
/// normal there (zero inside), `aa` −1 on the fill's side of the edge
/// (moved half a device pixel in, full coverage) and +1 on the fringe's
/// outer side (half a pixel out, none), and the paint of its path.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UiVectorVertex {
    pub position: [f32; 2],
    pub normal: [f32; 2],
    pub aa: f32,
    pub paint: u32,
}

/// A vector's line: a polyline drawn as an analytic [`UiGeometry::Polyline`].
#[derive(Clone, Debug, PartialEq)]
pub struct UiVectorStroke {
    pub points: Vec<Vec2>,
    pub closed: bool,
    pub width: f32,
    pub cap: UiCap,
    pub join: UiJoin,
    pub paint: u32,
    /// Dash and gap (0: solid), view box units.
    pub dash: [f32; 2],
}

/// A vector's paint: gradients' `from` and `to` in view box units.
#[derive(Clone, Debug, PartialEq)]
pub enum UiVectorPaint {
    Solid([f32; 4]),
    Gradient(UiGradient),
}

/// A cooked `type vector` (og-cook's `vector`): its fills as triangles with
/// their antialiasing fringe, its strokes as polylines, in the view box
/// `[x, y, width, height]`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiVectorData {
    pub view_box: [f32; 4],
    pub vertices: Vec<UiVectorVertex>,
    pub indices: Vec<u32>,
    pub paints: Vec<UiVectorPaint>,
    pub strokes: Vec<UiVectorStroke>,
}

/// A vector drawn with its view box fitted (contained, centred) into `rect`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiVectorDraw {
    pub vector: UiVectorHandle,
    pub rect: [f32; 4],
    /// About the rectangle's centre, as [`UiShape::transform`].
    pub transform: [f32; 4],
    /// Multiplies every paint.
    pub tint: [f32; 4],
    pub layer: i32,
    pub clip: Option<u32>,
}

/// The UI pass's content of one overlay. See the module documentation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiList {
    pub shapes: Vec<UiShape>,
    pub clips: Vec<UiClip>,
    pub layers: Vec<UiLayer>,
    pub text_effects: Vec<UiTextEffects>,
    pub vectors: Vec<UiVectorDraw>,
    /// The UI clock in seconds: patterns scroll and materials move by it.
    pub time: f32,
    pub mass: Vec<UiMassBatch>,
    pub occluders: Vec<UiOccluder>,
    /// The tick mass elements' ages count from ([`UiMassInstance::start`]),
    /// and how far the frame is past it (0 to 1).
    pub tick: u32,
    pub tick_fraction: f32,
}

impl UiList {
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty() && self.vectors.is_empty() && self.mass.is_empty()
    }

    /// Whether it reads the frame's depth (occluders, mass elements the
    /// scene hides, a projected layer's drape): the frame runs its depth
    /// prepass.
    pub fn needs_depth(&self) -> bool {
        !self.occluders.is_empty()
            || self.layers.iter().any(|l| l.ground.is_some())
            || self
                .mass
                .iter()
                .any(|b| b.space == UiMassSpace::World && b.template.occlusion != UiOcclusion::None)
    }
}

/// A scene's UI material ([`crate::RenderScene::add_ui_material`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UiMaterialHandle(pub u32);

impl UiMaterialHandle {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// A custom UI material (`type ui-material`): WGSL defining
///
/// ```wgsl
/// fn material(m: MaterialInput, p: Params) -> vec4<f32>
/// struct MaterialInput { uv: vec2<f32>, pixel: vec2<f32>, size: vec2<f32>, time: f32 }
/// ```
///
/// returning the composite's straight-alpha linear colour at `m.uv` (0..1
/// over the layer's rectangle, `size` CSS px, `pixel` the CSS px within it).
/// It reads the layer with `layer(uv)` (straight alpha, linear) and
/// `layer_texel()` (the uv of one CSS px), and its parameters (at most
/// [`UI_MATERIAL_PARAMS`], set per layer in [`UiLayer::params`]) from `p`.
/// The cook validates it with naga for Metal, Vulkan and DX12.
#[derive(Clone, Debug, PartialEq)]
pub struct UiMaterialShader {
    pub name: String,
    pub code: String,
    pub params: Vec<ShaderParam>,
}

impl UiMaterialShader {
    /// Parameter `name`'s slot and type.
    pub fn param(&self, name: &str) -> Option<(usize, ParamKind)> {
        self.params
            .iter()
            .position(|p| p.name == name)
            .map(|i| (i, self.params[i].kind))
    }

    /// The parameters at their defaults.
    pub fn defaults(&self) -> [[f32; 4]; UI_MATERIAL_PARAMS] {
        let mut out = [[0.0; 4]; UI_MATERIAL_PARAMS];
        for (slot, p) in out.iter_mut().zip(&self.params) {
            *slot = p.default;
        }
        out
    }

    /// The parameters' count and names, and that `material` is defined.
    pub fn check(&self) -> Result<(), String> {
        if self.params.len() > UI_MATERIAL_PARAMS {
            return Err(format!(
                "a UI material has at most {UI_MATERIAL_PARAMS} params, not {}",
                self.params.len()
            ));
        }
        for p in &self.params {
            let ok = p
                .name
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic())
                && p.name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_');
            if !ok {
                return Err(format!("param `{}`: not a WGSL identifier", p.name));
            }
        }
        if !crate::hooks::defines(&self.code, "material") {
            return Err(
                "a UI material defines `fn material(m: MaterialInput, p: Params) -> vec4<f32>`"
                    .into(),
            );
        }
        Ok(())
    }
}

impl crate::RenderScene {
    /// Adds a cooked vector for the overlay's [`UiVectorDraw`]s.
    pub fn add_ui_vector(&mut self, vector: UiVectorData) -> UiVectorHandle {
        self.ui_vectors.push(vector);
        UiVectorHandle(self.ui_vectors.len() as u32 - 1)
    }

    /// Adds a custom UI material for layers ([`UiMaterial::Custom`]).
    pub fn add_ui_material(&mut self, material: UiMaterialShader) -> UiMaterialHandle {
        self.ui_materials.push(material);
        UiMaterialHandle(self.ui_materials.len() as u32 - 1)
    }
}

pub(crate) const UI_MATERIAL_WGSL: &str = wgsl!("ui_material.wgsl");

/// The module a custom UI material compiles to: the UI pass's common code,
/// the composite's prelude, its `Params` and its code.
pub fn ui_material_module_source(material: &UiMaterialShader) -> String {
    let mut s = String::from(UI_COMMON_WGSL);
    s.push_str(UI_MATERIAL_WGSL);
    s.push_str("struct Params {\n");
    if material.params.is_empty() {
        s.push_str("  unused: f32,\n");
    }
    for p in &material.params {
        s.push_str(&format!("  {}: {},\n", p.name, p.kind.wgsl()));
    }
    s.push_str("};\nfn material_params() -> Params {\n  var p: Params;\n");
    for (i, p) in material.params.iter().enumerate() {
        let swizzle = match p.kind {
            ParamKind::Float => ".x",
            ParamKind::Vec2 => ".xy",
            ParamKind::Vec3 => ".xyz",
            ParamKind::Vec4 => "",
        };
        s.push_str(&format!(
            "  p.{} = layer_params.params[{i}]{swizzle};\n",
            p.name
        ));
    }
    s.push_str("  return p;\n}\n// The material's code (ui/mod.rs).\n");
    s.push_str(&material.code);
    s.push('\n');
    s
}

/// A sample custom material (`tests/shaders.rs` validates it; og-tests'
/// `ui-layers` draws it): the layer's colours turned into a duotone of two
/// parameters by their luminance, with a moving band.
pub fn example_ui_material() -> UiMaterialShader {
    let param = |name: &str, kind, default| ShaderParam {
        name: name.into(),
        kind,
        default,
    };
    UiMaterialShader {
        name: "duotone".into(),
        code: DUOTONE_WGSL.into(),
        params: vec![
            param("dark", ParamKind::Vec4, [0.02, 0.05, 0.2, 1.0]),
            param("light", ParamKind::Vec4, [1.0, 0.75, 0.3, 1.0]),
            param("band", ParamKind::Float, [0.25, 0.0, 0.0, 0.0]),
        ],
    }
}

const DUOTONE_WGSL: &str = "fn material(m: MaterialInput, p: Params) -> vec4<f32> {
  let c = layer(m.uv);
  let y = dot(c.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  var rgb = mix(p.dark.rgb, p.light.rgb, smoothstep(0.0, 0.6, y));
  // A soft band across the layer, moving with the UI clock.
  let s = fract(m.uv.x + m.uv.y * 0.5 - m.time * 0.25);
  rgb += vec3<f32>(p.band) * smoothstep(0.1, 0.0, abs(s - 0.5));
  return vec4<f32>(rgb, c.a);
}
";

/// The code every UI module starts with: the frame uniform, the clip and
/// gradient tables, the signed distance functions and the paints.
pub(crate) const UI_COMMON_WGSL: &str = wgsl!("ui_common.wgsl");

/// Flattens a cubic Bézier into points (the first included) no further
/// than `tolerance` from the curve, for a [`UiGeometry::Polyline`].
pub fn flatten_cubic(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, tolerance: f32) -> Vec<Vec2> {
    // The control polygon's deviation bounds the subdivisions (Wang's formula).
    let dd = ((p0 - 2.0 * p1 + p2).length()).max((p1 - 2.0 * p2 + p3).length());
    let n = ((0.75 * dd / tolerance.max(1e-3)).sqrt().ceil() as usize).clamp(1, 256);
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
        })
        .collect()
}

/// Flattens a quadratic Bézier as [`flatten_cubic`].
pub fn flatten_quad(p0: Vec2, p1: Vec2, p2: Vec2, tolerance: f32) -> Vec<Vec2> {
    flatten_cubic(
        p0,
        p0 + (p1 - p0) * (2.0 / 3.0),
        p2 + (p1 - p2) * (2.0 / 3.0),
        p2,
        tolerance,
    )
}

/// The OKLab of a linear sRGB colour (Björn Ottosson's matrices).
pub fn oklab(rgb: [f32; 3]) -> [f32; 3] {
    let [r, g, b] = rgb;
    let l = 0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b;
    let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;
    let (l, m, s) = (l.cbrt(), m.cbrt(), s.cbrt());
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

/// The linear sRGB of an OKLab colour ([`oklab`]'s inverse).
pub fn oklab_to_linear(lab: [f32; 3]) -> [f32; 3] {
    let [l, a, b] = lab;
    let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;
    let (l, m, s) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    [
        4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
        -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
        -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oklab_round_trips_and_mixes_apart_from_linear_light() {
        for c in [
            [1.0, 0.0, 0.0],
            [0.2, 0.5, 0.9],
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
        ] {
            let back = oklab_to_linear(oklab(c));
            for k in 0..3 {
                assert!((back[k] - c[k]).abs() < 1e-4, "{c:?} -> {back:?}");
            }
        }
        // Blue to yellow: half way in OKLab is perceptually half way in
        // lightness (L 0.71), where linear light's middle, grey 0.5, reads
        // far lighter (L 0.79): the two mixes are not the same colour.
        let (a, b) = (oklab([0.0, 0.0, 1.0]), oklab([1.0, 1.0, 0.0]));
        let mid = oklab_to_linear([
            0.5 * (a[0] + b[0]),
            0.5 * (a[1] + b[1]),
            0.5 * (a[2] + b[2]),
        ]);
        let l = |c: [f32; 3]| oklab(c)[0];
        assert!((l(mid) - 0.5 * (a[0] + b[0])).abs() < 1e-3);
        assert!(l([0.5, 0.5, 0.5]) - l(mid) > 0.05, "{mid:?}");
    }

    #[test]
    fn a_cubic_flattens_within_its_tolerance_and_keeps_its_ends() {
        let (p0, p1, p2, p3) = (
            Vec2::ZERO,
            Vec2::new(0.0, 100.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(100.0, 0.0),
        );
        let pts = flatten_cubic(p0, p1, p2, p3, 0.25);
        assert_eq!(pts.first(), Some(&p0));
        assert_eq!(pts.last(), Some(&p3));
        // Midpoints of each chord lie within the tolerance of the curve.
        let curve = |t: f32| {
            let u = 1.0 - t;
            p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
        };
        let n = pts.len() - 1;
        for i in 0..n {
            let mid = (pts[i] + pts[i + 1]) * 0.5;
            let on = curve((i as f32 + 0.5) / n as f32);
            assert!(mid.distance(on) <= 0.3, "chord {i}: {}", mid.distance(on));
        }
    }

    #[test]
    fn a_material_must_define_its_function_and_few_params() {
        assert!(example_ui_material().check().is_ok());
        let mut bad = example_ui_material();
        bad.code = "fn other() {}".into();
        assert!(bad.check().is_err());
        let mut many = example_ui_material();
        many.params = (0..5)
            .map(|i| ShaderParam {
                name: format!("p{i}"),
                kind: ParamKind::Float,
                default: [0.0; 4],
            })
            .collect();
        assert!(many.check().is_err());
    }
}
