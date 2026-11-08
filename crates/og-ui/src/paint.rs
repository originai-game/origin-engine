//! What an element draws with (`docs/ui.md` §3.3): its shape, layered fills
//! and strokes, effects, opacity and blend, how it clips its children, and
//! whether its subtree draws offscreen through a material. Every value is
//! data: colours are style tokens resolved at draw time ([`ColorRef`]) or
//! literals, so a style change restyles the paints, and an agent reads and
//! writes them as it does any prop.
//!
//! [`ElementDraw`] is carried by an element ([`crate::Ui::set_draw`]) and
//! turned into [`DrawCmd`]s around the element's own commands and its
//! children: shadows, outer glow and backdrop under the fills, inner shadow
//! and glow over them, then the children (cut to the shape when it clips),
//! then the strokes; a layer brackets all of it.

use crate::draw::DrawCmd;
use crate::layout::Rect;
use crate::style::{Srgba, Theme};
use og_core::glam::Vec2;
pub use og_render::{
    TextureHandle, UiAlign as StrokeAlign, UiBlend as Blend, UiBuiltin as Builtin, UiCap as Cap,
    UiCorner as Corner, UiEdge as Edge, UiFillRule as FillRule, UiFit as Fit,
    UiGradientKind as GradientKind, UiJoin as Join, UiMaterialHandle, UiSpread as Spread,
    UiVectorHandle,
};

/// A colour: a style token (a [`crate::Palette`] name, the IDL's
/// `ThemeColor`) resolved through the theme when the frame is drawn, or a
/// literal sRGB colour as authored.
#[derive(Clone, Debug, PartialEq)]
pub enum ColorRef {
    Token {
        name: String,
        alpha: f32,
    },
    Literal(Srgba),
    /// A CSS colour expression over roles (`color-mix()`, a relative
    /// colour, `light-dark()`), resolved through the theme when drawn.
    Css(std::sync::Arc<crate::css_color::Color>),
}

/// What an unknown token draws: a colour no style uses, so a typo shows.
pub const MISSING_TOKEN: [f32; 4] = [1.0, 0.0, 1.0, 1.0];

impl ColorRef {
    pub fn token(name: &str) -> ColorRef {
        ColorRef::Token {
            name: name.to_string(),
            alpha: 1.0,
        }
    }

    /// Linear RGBA through `theme` (an unknown token is [`MISSING_TOKEN`]).
    pub fn resolve(&self, theme: &Theme) -> [f32; 4] {
        match self {
            ColorRef::Token { name, alpha } => theme
                .color(name)
                .map_or(MISSING_TOKEN, |c| c.fade(alpha.clamp(0.0, 1.0)).linear()),
            ColorRef::Literal(c) => c.linear(),
            ColorRef::Css(c) => c.resolve(&crate::css_color::Context::of(theme)).linear(),
        }
    }

    /// The sRGB colour through `theme` (`None`: an unknown token).
    pub fn srgb(&self, theme: &Theme) -> Option<Srgba> {
        match self {
            ColorRef::Token { name, alpha } => {
                theme.color(name).map(|c| c.fade(alpha.clamp(0.0, 1.0)))
            }
            ColorRef::Literal(c) => Some(*c),
            ColorRef::Css(c) => Some(c.resolve(&crate::css_color::Context::of(theme))),
        }
    }
}

impl From<Srgba> for ColorRef {
    fn from(c: Srgba) -> ColorRef {
        ColorRef::Literal(c)
    }
}

/// A gradient stop.
#[derive(Clone, Debug, PartialEq)]
pub struct Stop {
    pub offset: f32,
    pub color: ColorRef,
}

/// A gradient over the element's box: `from` and `to` are fractions of it
/// (0,0 top left); radial and diamond take `to` as the radii, conic
/// `to[0]` as the start angle (radians clockwise from +x). Up to 8 stops,
/// mixed in OKLab, dithered.
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    pub kind: GradientKind,
    pub from: [f32; 2],
    pub to: [f32; 2],
    pub stops: Vec<Stop>,
    pub spread: Spread,
    pub dither: bool,
}

impl Default for Gradient {
    fn default() -> Gradient {
        Gradient {
            kind: GradientKind::Linear,
            from: [0.0, 0.0],
            to: [0.0, 1.0],
            stops: Vec::new(),
            spread: Spread::Pad,
            dither: true,
        }
    }
}

impl Gradient {
    pub fn resolve(&self, theme: &Theme) -> og_render::UiGradient {
        og_render::UiGradient {
            kind: self.kind,
            from: Vec2::from(self.from),
            to: Vec2::from(self.to),
            stops: self
                .stops
                .iter()
                .map(|s| og_render::UiStop {
                    offset: s.offset,
                    color: s.color.resolve(theme),
                })
                .collect(),
            spread: self.spread,
            dither: self.dither,
        }
    }
}

/// An image paint: a region of a texture, its pixel size, a tint, how it
/// fits the box and where it is anchored ([`Fit`]: fill, cover, contain,
/// none, nine-slice, pattern).
#[derive(Clone, Debug, PartialEq)]
pub struct ImagePaint {
    pub texture: TextureHandle,
    pub uv: [f32; 4],
    pub size: [f32; 2],
    pub tint: ColorRef,
    pub fit: Fit,
    pub focal: [f32; 2],
}

/// One layer of fill or a stroke's paint.
#[derive(Clone, Debug, PartialEq)]
pub enum Paint {
    Solid(ColorRef),
    Gradient(Gradient),
    Image(ImagePaint),
    /// A cooked vector (`type vector`) contained and centred in the
    /// element's rectangle, its paints multiplied by `tint`. As a stroke's
    /// paint it is its tint.
    Vector {
        vector: UiVectorHandle,
        tint: ColorRef,
    },
    /// A region of the style's atlas (a dressing's `fill image …`), drawn
    /// as an image of [`crate::style_file::Art::atlas`] when the frame is
    /// drawn; nothing without an atlas.
    Atlas(AtlasPaint),
    /// A gradient as an image `size` CSS px (the size its geometry was
    /// resolved against) fitted as an image is: a CSS gradient tile.
    GradientImage {
        gradient: Gradient,
        size: [f32; 2],
        fit: Fit,
    },
}

/// [`Paint::Atlas`]: `uv` the region `[u0, v0, u1, v1]`, `size` its CSS px
/// (texels times the style's scale), as [`ImagePaint`] otherwise.
#[derive(Clone, Debug, PartialEq)]
pub struct AtlasPaint {
    pub uv: [f32; 4],
    pub size: [f32; 2],
    pub tint: ColorRef,
    pub fit: Fit,
    pub focal: [f32; 2],
}

impl Paint {
    pub fn resolve(&self, theme: &Theme) -> og_render::UiPaint {
        match self {
            Paint::Solid(c) => og_render::UiPaint::Solid(c.resolve(theme)),
            Paint::Gradient(g) => og_render::UiPaint::Gradient(g.resolve(theme)),
            Paint::Image(im) => og_render::UiPaint::Image(og_render::UiImage {
                texture: im.texture,
                uv: im.uv,
                size: Vec2::from(im.size),
                tint: im.tint.resolve(theme),
                fit: im.fit,
                focal: Vec2::from(im.focal),
                luminance: false,
            }),
            Paint::Vector { tint, .. } => og_render::UiPaint::Solid(tint.resolve(theme)),
            Paint::Atlas(a) => match theme.art.atlas {
                Some(texture) => og_render::UiPaint::Image(og_render::UiImage {
                    texture,
                    uv: a.uv,
                    size: Vec2::from(a.size),
                    tint: a.tint.resolve(theme),
                    fit: a.fit,
                    focal: Vec2::from(a.focal),
                    luminance: false,
                }),
                None => og_render::UiPaint::Solid([0.0; 4]),
            },
            Paint::GradientImage {
                gradient,
                size,
                fit,
            } => og_render::UiPaint::GradientImage(Box::new(og_render::UiGradientImage {
                gradient: gradient.resolve(theme),
                size: Vec2::from(*size),
                fit: *fit,
                luminance: false,
            })),
        }
    }
}

/// A CSS layer of a box: a background layer (CSS Backgrounds 3 §3) or a
/// mask layer (CSS Masking 1 §6), drawn over the box and cut to its
/// painting area.
#[derive(Clone, Debug, PartialEq)]
pub struct BoxLayer {
    /// An image's or a gradient tile's fit is a CSS background's, its
    /// positioning area `area`.
    pub paint: Paint,
    /// The positioning area's edges in from the box's (left, top, right,
    /// bottom; CSS px, negative outside it), or a fixed one's rectangle
    /// on the screen.
    pub area: Area,
    pub clip: LayerClip,
    /// How it combines with the layers under it: a background's blend
    /// mode, a mask layer's Porter-Duff operator.
    pub blend: Blend,
    /// Its luminance is its alpha (`mask-mode: luminance`).
    pub luminance: bool,
}

/// [`BoxLayer::area`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Area {
    /// Edges in from the box's: left, top, right, bottom.
    Box([f32; 4]),
    /// `background-attachment: fixed`: `[x, y, width, height]` on the
    /// screen (the viewport), whatever moves the box.
    Fixed([f32; 4]),
}

/// [`BoxLayer::clip`]: where a layer is painted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayerClip {
    /// The box, its edges in by these (left, top, right, bottom; CSS px),
    /// its corners curving as the box's do less the inset (CSS
    /// Backgrounds 3 §5.3). A mask's is a plain rectangle.
    Box([f32; 4]),
    /// A plain rectangle, its edges in by these (a mask's, a border
    /// image's; far out for `mask-clip: no-clip`).
    Rect([f32; 4]),
    /// The element's own text (`background-clip: text`).
    Text,
}

/// An element's CSS background and mask layers ([`ElementDraw::boxes`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoxLayers {
    /// Its background layers, the bottom (its colour) first: drawn after
    /// its fills, under its inner shadows.
    pub backgrounds: Vec<BoxLayer>,
    /// The backgrounds blend inside a group of their own, not with what is
    /// behind the element (`background-blend-mode`).
    pub isolate: bool,
    /// Its mask layers, the bottom first, each combined with those under it
    /// by its Porter-Duff operator: its subtree is drawn offscreen and kept
    /// where their result is (CSS Masking 1 §6).
    pub masks: Vec<BoxLayer>,
    /// Its border image, over its backgrounds and inner shadows.
    pub border: Option<BoxLayer>,
}

impl BoxLayer {
    /// Its paint drawn in `d`, `inset` (left, top, right, bottom) in from
    /// the box `b`: its fit's positioning area made `d`'s.
    fn resolve(&self, theme: &Theme, b: Rect, d: Rect) -> og_render::UiPaint {
        let mut p = self.paint.resolve(theme);
        let area = match self.area {
            Area::Box([l, t, r, bt]) => [
                l - (d.x - b.x),
                t - (d.y - b.y),
                r - (b.right() - d.right()),
                bt - (b.bottom() - d.bottom()),
            ],
            Area::Fixed([x, y, w, h]) => {
                [x - d.x, y - d.y, d.right() - (x + w), d.bottom() - (y + h)]
            }
        };
        let fit = match &mut p {
            og_render::UiPaint::Image(im) => {
                im.luminance = self.luminance;
                Some(&mut im.fit)
            }
            og_render::UiPaint::GradientImage(g) => {
                g.luminance = self.luminance;
                Some(&mut g.fit)
            }
            _ => None,
        };
        if let Some(og_render::UiFit::Background(bg)) = fit {
            bg.area = area;
        }
        p
    }
}

/// `geometry` inset by `inset` (left, top, right, bottom): a rectangle's
/// corners less the inset across each (CSS Backgrounds 3 §5.3).
fn inset_geometry(g: og_render::UiGeometry, inset: [f32; 4]) -> og_render::UiGeometry {
    match g {
        og_render::UiGeometry::Rect { radii, corner } if inset != [0.0; 4] => {
            let (vertical, shape) = match corner {
                Corner::Css(c) => (c.vertical, c.shape),
                Corner::Round => (radii, [1.0; 4]),
                Corner::Squircle(n) => (radii, [n.max(1e-3).log2(); 4]),
                Corner::Bevel => (radii, [0.0; 4]),
                Corner::Notch => (radii, [-1.0; 4]),
            };
            let [l, t, r, b] = inset;
            og_render::UiGeometry::Rect {
                radii: [radii[0] - l, radii[1] - r, radii[2] - r, radii[3] - l].map(|v| v.max(0.0)),
                corner: Corner::Css(og_render::UiCssCorners {
                    vertical: [
                        vertical[0] - t,
                        vertical[1] - t,
                        vertical[2] - b,
                        vertical[3] - b,
                    ]
                    .map(|v| v.max(0.0)),
                    shape,
                }),
            }
        }
        g => g,
    }
}

fn inset_rect(r: Rect, [l, t, rt, b]: [f32; 4]) -> Rect {
    Rect::new(
        r.x + l,
        r.y + t,
        (r.width - l - rt).max(0.0),
        (r.height - t - b).max(0.0),
    )
}

fn plain_layer(mask: Option<og_render::UiMask>) -> DrawCmd {
    DrawCmd::BeginLayer {
        material: og_render::UiMaterial::Plain,
        params: [[0.0; 4]; og_render::UI_MATERIAL_PARAMS],
        mask,
        expand: 0.0,
        rect: Rect::default(),
    }
}

fn end_layer(
    geometry: Option<og_render::UiGeometry>,
    at: &Placed,
    opacity: f32,
    blend: Blend,
) -> DrawCmd {
    DrawCmd::EndLayer {
        geometry,
        rect: at.rect,
        opacity,
        blend,
        clip: at.clip,
        ground: None,
    }
}

/// A stroke along the element's outline with a paint of its own.
#[derive(Clone, Debug, PartialEq)]
pub struct Stroke {
    pub width: f32,
    pub align: StrokeAlign,
    /// Dash and gap, CSS px (0: solid).
    pub dash: [f32; 2],
    pub dash_offset: f32,
    pub dash_cap: Cap,
    pub paint: Paint,
}

/// An effect of the element (§5.3).
#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    DropShadow {
        offset: [f32; 2],
        blur: f32,
        spread: f32,
        color: ColorRef,
    },
    InnerShadow {
        offset: [f32; 2],
        blur: f32,
        spread: f32,
        color: ColorRef,
    },
    OuterGlow {
        blur: f32,
        spread: f32,
        color: ColorRef,
    },
    InnerGlow {
        blur: f32,
        spread: f32,
        color: ColorRef,
    },
    /// Blurs the element and its subtree (a layer with the blur material).
    LayerBlur { radius: f32 },
    /// What is under the element blurred, its saturation and brightness
    /// scaled and tinted: glass, with its material ([`GlassParts`]).
    Backdrop {
        blur: f32,
        saturation: f32,
        brightness: f32,
        tint: ColorRef,
        glass: GlassParts,
    },
}

/// A backdrop's material beyond its blur and tint (og-render's
/// [`og_render::UiGlass`]); the default is none of it, the plain frosted
/// backdrop.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GlassParts {
    /// One of the design language's weights: its blur, 200 % saturation,
    /// tint (unless the effect gives one), 0.5 px hairline and CSS mixing,
    /// in the theme's scheme; under reduced transparency or increased
    /// contrast the solid fallback. `None`: the effect's own values.
    pub weight: Option<og_render::UiGlassWeight>,
    /// The soft shadow outside it (`None`: none).
    pub elevation: Option<og_render::UiElevation>,
    /// A hairline inside the edge, CSS px (0: none, or the weight's).
    pub edge: f32,
    /// The hairline's colour (`None`: the weight's, or none).
    pub edge_color: Option<ColorRef>,
    /// The tint drawn opaque; nothing under it is read.
    pub solid: bool,
    /// Mixed in sRGB-encoded values, as CSS does.
    pub css_mix: bool,
    /// The effect named its tint: a weight keeps it instead of its own.
    pub own_tint: bool,
    /// A colour matrix over the blurred backdrop (CSS's `backdrop-filter`
    /// colour functions, [`og_render::UiGlass::matrix`]).
    pub matrix: Option<og_render::UiColorMatrix>,
}

impl GlassParts {
    /// The backdrop's mode and tint for `theme` (`tint`: the effect's own,
    /// resolved; `explicit_tint`: the effect named one).
    pub fn resolve(
        &self,
        blur: f32,
        saturation: f32,
        brightness: f32,
        tint: [f32; 4],
        explicit_tint: bool,
        theme: &Theme,
    ) -> (og_render::UiMode, [f32; 4]) {
        let dark = theme.scheme == crate::style::Scheme::Dark;
        let fallback = if theme.reduced_transparency {
            og_render::UiGlassFallback::ReducedTransparency
        } else if theme.contrast == crate::style::Contrast::High {
            og_render::UiGlassFallback::IncreasedContrast
        } else {
            og_render::UiGlassFallback::None
        };
        let (mut glass, blur, saturation, brightness, tint) = match self.weight {
            Some(weight) => {
                let style = og_render::UiGlassStyle {
                    weight,
                    elevation: self.elevation.unwrap_or_default(),
                    dark,
                    fallback,
                };
                let (mode, preset) = style.backdrop();
                let og_render::UiMode::Backdrop {
                    blur,
                    saturation,
                    brightness,
                    glass,
                } = mode
                else {
                    unreachable!("a glass style is a backdrop")
                };
                let solid = fallback != og_render::UiGlassFallback::None;
                let tint = if explicit_tint && !solid {
                    tint
                } else {
                    preset
                };
                (glass, blur, saturation, brightness, tint)
            }
            None => {
                let mut glass = og_render::UiGlass::default();
                if let Some(e) = self.elevation {
                    let style = og_render::UiGlassStyle {
                        elevation: e,
                        dark,
                        ..Default::default()
                    };
                    if let og_render::UiMode::Backdrop { glass: g, .. } = style.backdrop().0 {
                        glass.shadows = g.shadows;
                    }
                }
                (glass, blur, saturation, brightness, tint)
            }
        };
        if self.edge > 0.0 {
            glass.edge = self.edge;
        }
        if let Some(c) = &self.edge_color {
            glass.edge_color = c.resolve(theme);
        }
        glass.solid |= self.solid;
        glass.srgb_mix |= self.css_mix;
        if self.matrix.is_some() {
            glass.matrix = self.matrix;
        }
        let mode = og_render::UiMode::Backdrop {
            blur,
            saturation,
            brightness,
            glass,
        };
        (mode, tint)
    }
}

/// The element's outline.
#[derive(Clone, Debug, PartialEq)]
pub enum ShapeKind {
    /// Its rectangle; `radii` top left, top right, bottom right, bottom
    /// left (`None`: the element's radius at every corner).
    Rect {
        radii: Option<[f32; 4]>,
        corner: Corner,
    },
    Ellipse,
    Polygon {
        sides: u32,
        inner: f32,
        rounding: f32,
        rotation: f32,
    },
    Arc {
        start: f32,
        sweep: f32,
        thickness: f32,
        cap: Cap,
    },
    /// A line `width` CSS px through `points` (CSS px from the element's
    /// top left); with `curves`, each three points after the first are a
    /// cubic Bézier's two controls and its end (connectors, graphs).
    Path {
        points: Vec<[f32; 2]>,
        closed: bool,
        /// Fills cover the inside of the closed contours by `fill_rule`,
        /// drawn from the curves themselves ([`og_render::UiGeometry::Path`]),
        /// instead of drawing its line, and strokes outline it.
        filled: bool,
        curves: bool,
        /// A filled path's rule where its contours overlap or cross.
        fill_rule: FillRule,
        /// A filled path's contours: how many of `points` each takes, in
        /// order; empty is one contour of every point.
        contours: Vec<u32>,
        width: f32,
        cap: Cap,
        join: Join,
    },
}

impl Default for ShapeKind {
    fn default() -> ShapeKind {
        ShapeKind::Rect {
            radii: None,
            corner: Corner::Round,
        }
    }
}

impl ShapeKind {
    /// The geometry of an element of `radius` (its default corner radius)
    /// and `size` (CSS px).
    pub fn geometry(&self, radius: f32, size: [f32; 2]) -> og_render::UiGeometry {
        match self {
            ShapeKind::Rect { radii, corner } => og_render::UiGeometry::Rect {
                radii: radii.unwrap_or([radius; 4]),
                corner: *corner,
            },
            ShapeKind::Ellipse => og_render::UiGeometry::Ellipse,
            ShapeKind::Polygon {
                sides,
                inner,
                rounding,
                rotation,
            } => og_render::UiGeometry::Polygon {
                sides: *sides,
                inner: *inner,
                rounding: *rounding,
                rotation: *rotation,
            },
            ShapeKind::Arc {
                start,
                sweep,
                thickness,
                cap,
            } => og_render::UiGeometry::Arc {
                start: *start,
                sweep: *sweep,
                thickness: *thickness,
                cap: *cap,
            },
            ShapeKind::Path {
                points,
                filled: true,
                curves,
                fill_rule,
                contours,
                ..
            } => {
                let half = Vec2::from(size) * 0.5;
                let mut path = og_render::UiPath::new();
                path.fill_rule = *fill_rule;
                let mut rest: &[[f32; 2]] = points;
                let whole = [points.len() as u32];
                let counts: &[u32] = if contours.is_empty() {
                    &whole
                } else {
                    contours
                };
                for &n in counts {
                    let (c, more) = rest.split_at((n as usize).min(rest.len()));
                    rest = more;
                    let c: Vec<Vec2> = c.iter().map(|p| Vec2::from(*p) - half).collect();
                    let Some(first) = c.first() else { continue };
                    path.move_to(*first);
                    if *curves {
                        for b in c[1..].as_chunks::<3>().0 {
                            path.cubic_to(b[0], b[1], b[2]);
                        }
                    } else {
                        for p in &c[1..] {
                            path.line_to(*p);
                        }
                    }
                    path.close();
                }
                og_render::UiGeometry::Path(std::sync::Arc::new(path))
            }
            ShapeKind::Path {
                points,
                closed,
                filled,
                curves,
                width,
                cap,
                join,
                ..
            } => {
                let half = Vec2::from(size) * 0.5;
                let p: Vec<Vec2> = points.iter().map(|p| Vec2::from(*p) - half).collect();
                let mut line = p.first().map(|p| vec![*p]).unwrap_or_default();
                if *curves {
                    for c in p[1.min(p.len())..].as_chunks::<3>().0 {
                        let from = *line.last().expect("the first point");
                        // A quarter CSS px from the curve.
                        line.extend(
                            og_render::flatten_cubic(from, c[0], c[1], c[2], 0.25)
                                .into_iter()
                                .skip(1),
                        );
                    }
                } else {
                    line.extend(p.iter().skip(1));
                }
                // A closed line that ends where it starts closes itself:
                // the repeated point would be a joint of no length, whose
                // miter has no direction (a spike).
                if (*closed || *filled)
                    && line.len() > 2
                    && (line[0] - line[line.len() - 1]).length() < 1e-4
                {
                    line.pop();
                }
                og_render::UiGeometry::Polyline {
                    points: line,
                    closed: *closed || *filled,
                    filled: *filled,
                    width: *width,
                    cap: *cap,
                    join: *join,
                }
            }
        }
    }
}

/// An alpha mask over the element's subtree.
#[derive(Clone, Debug, PartialEq)]
pub enum Mask {
    Gradient(Gradient),
    Image {
        texture: TextureHandle,
        uv: [f32; 4],
    },
    /// A region `[u0, v0, u1, v1]` of the style's atlas (a dressing's
    /// `mask image …`); no mask without an atlas.
    Atlas {
        uv: [f32; 4],
    },
}

/// How the element cuts its children.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Clip {
    #[default]
    None,
    /// Its children to its shape, antialiased (`overflow: hidden`).
    Shape,
    /// Everything it draws, its own shadows, fills, strokes and border
    /// and its children, to this shape (CSS `clip-path`), in CSS px from
    /// its box's top left.
    Cut(ShapeKind),
    /// By a mask's alpha (the subtree draws offscreen).
    Mask(Mask),
}

/// What a layer composites through.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Material {
    #[default]
    Plain,
    Builtin(Builtin),
    Custom(UiMaterialHandle),
}

/// The element's subtree drawn offscreen, then composited through a material.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layer {
    pub material: Material,
    pub params: [[f32; 4]; og_render::UI_MATERIAL_PARAMS],
}

/// Effects of the element's text (§5.6).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextEffects {
    pub shadows: Vec<TextShadow>,
    pub outline: Option<TextOutline>,
    /// A gradient over the text's line box.
    pub fill: Option<Gradient>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextShadow {
    pub offset: [f32; 2],
    pub blur: f32,
    pub color: ColorRef,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextOutline {
    pub width: f32,
    pub color: ColorRef,
    pub align: StrokeAlign,
}

/// What an element draws with. See the module documentation.
#[derive(Clone, Debug, PartialEq)]
pub struct ElementDraw {
    pub shape: ShapeKind,
    pub fills: Vec<Paint>,
    pub strokes: Vec<Stroke>,
    pub effects: Vec<Effect>,
    /// Multiplies its own paints (a layer's composite, when it has one).
    pub opacity: f32,
    pub blend: Blend,
    pub clip: Clip,
    pub layer: Option<Layer>,
    /// Layers under [`ElementDraw::layer`], the first innermost: each
    /// composites what is inside it into the next (CSS's `filter`
    /// functions in order). With no `layer`, the last one is the
    /// composite that carries the opacity, blend and mask.
    pub filters: Vec<Layer>,
    pub text: Option<TextEffects>,
    /// A Rive artboard playing in its box: its shapes, made where the
    /// element is placed, draw after its fills and before its strokes,
    /// clipped to its clip (and to its box when the artboard cuts its
    /// content to its frame).
    pub rive: Option<RiveDraw>,
    /// Stroked shapes of its own beyond its shape's outline, drawn with its
    /// strokes in order (an SVG shape's subpaths, a border's sides).
    pub outlines: Vec<(ShapeKind, Stroke)>,
    /// A view box `[min x, min y, width, height]` its shapes, outlines and
    /// gradients are written in (an inline SVG's user space), fitted into
    /// its box when it is drawn as SVG's `xMidYMid meet` fits it: stroke
    /// widths and dashes scale with it. `None`: CSS px from its top left.
    pub view_box: Option<[f32; 4]>,
    /// With a view box: the sub-rect `[x, y, width, height]` of it, in its
    /// units, that its shape (other than a path), clip and paints fill
    /// instead of the whole box (an SVG `<image>`, a rect or ellipse
    /// clip); a rect's radii are then view-box units too.
    pub view_rect: Option<[f32; 4]>,
    /// Its CSS background and mask layers beyond its fills (a document
    /// element's `background-*` and `mask-*`).
    pub boxes: Option<Box<BoxLayers>>,
}

impl From<Material> for og_render::UiMaterial {
    fn from(m: Material) -> og_render::UiMaterial {
        match m {
            Material::Plain => og_render::UiMaterial::Plain,
            Material::Builtin(b) => og_render::UiMaterial::Builtin(b),
            Material::Custom(h) => og_render::UiMaterial::Custom(h),
        }
    }
}

/// How far past what it draws a layer through `material` reaches: a
/// blur's, outline's or ripple's radius and a drop shadow's blur and
/// offset, with the Gaussian's tail.
fn expand_of(
    material: og_render::UiMaterial,
    params: &[[f32; 4]; og_render::UI_MATERIAL_PARAMS],
) -> f32 {
    let p = params[0];
    match material {
        og_render::UiMaterial::Builtin(Builtin::Blur | Builtin::Outline | Builtin::Ripple) => {
            p[0].max(0.0) * 1.5
        }
        og_render::UiMaterial::Builtin(Builtin::DropShadow) => {
            p[2].max(0.0) * 1.5 + p[0].abs().max(p[1].abs())
        }
        _ => 0.0,
    }
}

/// A playing artboard as an element's draw. Two are equal when they play
/// the same artboard (the one asset, not a copy) at the same times.
#[derive(Clone, Debug)]
pub struct RiveDraw(pub crate::rive::Player);

impl PartialEq for RiveDraw {
    fn eq(&self, other: &RiveDraw) -> bool {
        fn same(a: &crate::rive::Player, b: &crate::rive::Player) -> bool {
            std::sync::Arc::ptr_eq(&a.artboard, &b.artboard)
                && a.playing == b.playing
                && a.nested.len() == b.nested.len()
                && a.nested
                    .iter()
                    .zip(&b.nested)
                    .all(|((i, p), (j, q))| i == j && same(p, q))
        }
        same(&self.0, &other.0)
    }
}

impl Default for ElementDraw {
    fn default() -> ElementDraw {
        ElementDraw {
            shape: ShapeKind::default(),
            fills: Vec::new(),
            strokes: Vec::new(),
            effects: Vec::new(),
            opacity: 1.0,
            blend: Blend::Normal,
            clip: Clip::None,
            layer: None,
            filters: Vec::new(),
            text: None,
            rive: None,
            outlines: Vec::new(),
            view_box: None,
            view_rect: None,
            boxes: None,
        }
    }
}

/// Where an element is drawn: its rectangle, default corner radius, clip
/// and the alpha it fades by.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Placed {
    pub rect: Rect,
    pub radius: f32,
    pub clip: Rect,
    pub alpha: f32,
}

impl ElementDraw {
    /// Whether its subtree draws offscreen.
    pub(crate) fn layered(&self) -> bool {
        self.layer.is_some()
            || !self.filters.is_empty()
            || matches!(self.clip, Clip::Mask(_))
            || self
                .effects
                .iter()
                .any(|e| matches!(e, Effect::LayerBlur { .. }))
    }

    /// Whether CSS mask layers cut its subtree: it draws in a plain layer
    /// of its own, outside its other layers, which the masks are applied
    /// to and which carries its opacity and blend.
    fn masked(&self) -> bool {
        self.boxes.as_ref().is_some_and(|b| !b.masks.is_empty())
    }

    /// Whether a layer's composite carries its opacity and blend.
    pub(crate) fn composited(&self) -> bool {
        self.layered() || self.masked()
    }

    /// Its background or mask layers drawn in `at` with `blend`, each cut
    /// to its painting area; `text`'s clip is drawn as the box's (the
    /// caller cuts those layers to the text).
    fn box_layers(
        &self,
        layers: &[BoxLayer],
        theme: &Theme,
        at: &Placed,
        opacity: f32,
        blend: impl Fn(&BoxLayer) -> Blend,
    ) -> Vec<DrawCmd> {
        let mut out = Vec::new();
        for l in layers {
            let (inset, mask_rect) = match l.clip {
                LayerClip::Box(i) => (i, false),
                LayerClip::Text => ([0.0; 4], false),
                LayerClip::Rect(i) => (i, true),
            };
            let r = inset_rect(at.rect, inset);
            if r.is_empty() {
                continue;
            }
            let geometry = if mask_rect {
                og_render::UiGeometry::Rect {
                    radii: [0.0; 4],
                    corner: Corner::Round,
                }
            } else {
                inset_geometry(self.geometry(&self.shape, at), inset)
            };
            out.push(DrawCmd::Shape {
                shape: Box::new(og_render::UiShape {
                    geometry,
                    center: Vec2::new(r.x + r.width / 2.0, r.y + r.height / 2.0),
                    half: Vec2::new(r.width / 2.0, r.height / 2.0),
                    transform: [1.0, 0.0, 0.0, 1.0],
                    paint: l.resolve(theme, at.rect, r),
                    mode: og_render::UiMode::Fill,
                    opacity,
                    blend: blend(l),
                    layer: 0,
                    clip: None,
                }),
                clip: at.clip,
            });
        }
        out
    }

    /// Its backgrounds cut to its text (`background-clip: text`) around
    /// `text`, the commands that draw it: the glyphs' coverage drawn
    /// opaque in a layer of their own, the layers kept only there, then
    /// the text as it is drawn (CSS Backgrounds 4 §2.1). `text` unchanged
    /// when it has none.
    pub(crate) fn clip_to_text(
        &self,
        theme: &Theme,
        at: &Placed,
        text: Vec<DrawCmd>,
    ) -> Vec<DrawCmd> {
        let Some(b) = self
            .boxes
            .as_ref()
            .filter(|b| b.backgrounds.iter().any(|l| l.clip == LayerClip::Text))
        else {
            return text;
        };
        let mut out = vec![plain_layer(None)];
        let opaque = |c: &mut DrawCmd| {
            if let DrawCmd::StyledText { text, .. } = c {
                *c = (**text).clone();
            }
            if let DrawCmd::Text { color, glow, .. } = c {
                *color = [1.0; 4];
                *glow = None;
            }
        };
        let mut glyphs = 0;
        for c in &text {
            if matches!(c, DrawCmd::Text { .. } | DrawCmd::StyledText { .. }) {
                let mut c = c.clone();
                opaque(&mut c);
                out.push(c);
                glyphs += 1;
            }
        }
        if glyphs == 0 {
            return text;
        }
        let on_text: Vec<BoxLayer> = b
            .backgrounds
            .iter()
            .filter(|l| l.clip == LayerClip::Text)
            .cloned()
            .collect();
        // The bottom layer keeps itself on the glyphs; the rest blend over
        // it there as they do in the box.
        for (k, cmd) in self
            .box_layers(&on_text, theme, at, 1.0, |l| l.blend)
            .into_iter()
            .enumerate()
        {
            let mut cmd = cmd;
            if k == 0
                && let DrawCmd::Shape { shape, .. } = &mut cmd
            {
                shape.blend = Blend::SourceIn;
            }
            out.push(cmd);
        }
        if on_text.len() > 1 {
            // Later layers drew over the glyphs' edges too: the glyphs cut
            // them again.
            out.push(plain_layer(None));
            for c in &text {
                if matches!(c, DrawCmd::Text { .. } | DrawCmd::StyledText { .. }) {
                    let mut c = c.clone();
                    opaque(&mut c);
                    out.push(c);
                }
            }
            out.push(end_layer(None, at, 1.0, Blend::DestinationIn));
        }
        let o = if self.composited() { 1.0 } else { self.opacity };
        out.push(end_layer(None, at, o, Blend::Normal));
        out.extend(text);
        out
    }

    /// Its layers: the outer one (its own, else its last filter's), which
    /// carries the composite, and the filters inside it, the first
    /// innermost.
    fn stages(&self) -> (Option<&Layer>, &[Layer]) {
        match (&self.layer, self.filters.split_last()) {
            (Some(l), _) => (Some(l), &self.filters),
            (None, Some((last, rest))) => (Some(last), rest),
            (None, None) => (None, &[]),
        }
    }

    /// Whether a point `p` (CSS px from its box's top left, the box `size`)
    /// is on the geometry SVG `pointer-events` names (`bits` of
    /// [`crate::element::SvgPointer`]): inside its fill by its fill rule,
    /// within half a stroke's width of its stroke or outlines, or in its
    /// bounding box; `painted` takes only what is painted (SVG 2 §16.6).
    pub fn covers(&self, size: [f32; 2], p: Vec2, bits: u8) -> bool {
        use crate::element::SvgPointer as S;
        let (k, q) = match self.view_box {
            Some([x, y, w, h]) => {
                let k = (size[0] / w.max(1e-6)).min(size[1] / h.max(1e-6)).max(1e-6);
                let mid = Vec2::new(x + w * 0.5, y + h * 0.5);
                (k, (p - Vec2::from(size) * 0.5) / k + mid)
            }
            None => (1.0, p),
        };
        let frame = self.view_rect.unwrap_or(match self.view_box {
            Some(vb) => vb,
            None => [0.0, 0.0, size[0], size[1]],
        });
        let tol = 0.25 / k;
        let lines = |kind: &ShapeKind| outline(kind, frame, tol);
        if bits & S::BOX != 0 {
            let all: Vec<Vec2> = std::iter::once(&self.shape)
                .chain(self.outlines.iter().map(|(s, _)| s))
                .flat_map(|s| lines(s).0.into_iter().flatten())
                .collect();
            let lo = all.iter().fold(Vec2::splat(f32::MAX), |a, b| a.min(*b));
            let hi = all.iter().fold(Vec2::splat(f32::MIN), |a, b| a.max(*b));
            return q.cmpge(lo).all() && q.cmple(hi).all();
        }
        let painted = bits & S::PAINTED != 0;
        let (shape, closed, rule) = lines(&self.shape);
        if bits & S::FILL != 0 && (!painted || !self.fills.is_empty()) && closed {
            let winding: i32 = shape.iter().map(|c| winding(c, q)).sum();
            if match rule {
                FillRule::EvenOdd => winding % 2 != 0,
                _ => winding != 0,
            } {
                return true;
            }
        }
        if bits & S::STROKE != 0 && (!painted || !self.strokes.is_empty()) {
            let width = self.strokes.iter().map(|s| s.width).fold(0.0, f32::max);
            let own = match &self.shape {
                ShapeKind::Path {
                    filled: false,
                    width: w,
                    ..
                } => *w,
                _ => width,
            };
            let near = |cs: &[Vec<Vec2>], closed: bool, half: f32| {
                cs.iter().any(|c| distance(c, closed, q) <= half)
            };
            if own > 0.0 && near(&shape, closed, own * 0.5) {
                return true;
            }
            if self.outlines.iter().any(|(s, st)| {
                let (c, closed, _) = lines(s);
                near(&c, closed, st.width * 0.5)
            }) {
                return true;
            }
        }
        false
    }

    /// The view box's fit in `at`'s box: its scale and the box point
    /// (CSS px from the box's top left) its centre lands on.
    fn fit(&self, at: &Placed) -> Option<(f32, Vec2, Vec2)> {
        let [_, _, w, h] = self.view_box?;
        let k = (at.rect.width / w.max(1e-6)).min(at.rect.height / h.max(1e-6));
        let vb = self.view_box?;
        Some((
            k,
            Vec2::new(vb[0] + vb[2] * 0.5, vb[1] + vb[3] * 0.5),
            Vec2::new(at.rect.width, at.rect.height) * 0.5,
        ))
    }

    /// The rect its shape fills in `at`: its view rect through the view
    /// box's fit, else the box.
    fn frame(&self, at: &Placed) -> Rect {
        match (self.fit(at), self.view_rect) {
            (Some((k, mid, half)), Some([x, y, w, h])) => {
                let o = (Vec2::new(x, y) - mid) * k + half;
                Rect {
                    x: at.rect.x + o.x,
                    y: at.rect.y + o.y,
                    width: w * k,
                    height: h * k,
                }
            }
            _ => at.rect,
        }
    }

    /// `kind`'s geometry placed in `at`, through the view box when it has one.
    fn geometry(&self, kind: &ShapeKind, at: &Placed) -> og_render::UiGeometry {
        let size = [at.rect.width, at.rect.height];
        let Some((k, mid, half)) = self.fit(at) else {
            return kind.geometry(at.radius, size);
        };
        match kind {
            ShapeKind::Path {
                points,
                closed,
                filled,
                curves,
                fill_rule,
                contours,
                width,
                cap,
                join,
            } => ShapeKind::Path {
                points: points
                    .iter()
                    .map(|p| ((Vec2::from(*p) - mid) * k + half).into())
                    .collect(),
                closed: *closed,
                filled: *filled,
                curves: *curves,
                fill_rule: *fill_rule,
                contours: contours.clone(),
                width: width * k,
                cap: *cap,
                join: *join,
            }
            .geometry(at.radius, size),
            ShapeKind::Rect {
                radii: Some(r),
                corner,
            } if self.view_rect.is_some() => ShapeKind::Rect {
                radii: Some(r.map(|v| v * k)),
                corner: *corner,
            }
            .geometry(at.radius, size),
            other => {
                let f = self.frame(at);
                other.geometry(at.radius, [f.width, f.height])
            }
        }
    }

    /// A paint resolved for `at`: a gradient written in view-box units
    /// becomes fractions of the box.
    fn paint(&self, paint: &Paint, theme: &Theme, at: &Placed) -> og_render::UiPaint {
        let mut out = paint.resolve(theme);
        if let (Some((k, mid, half)), og_render::UiPaint::Gradient(g)) = (self.fit(at), &mut out) {
            let size = Vec2::new(at.rect.width, at.rect.height).max(Vec2::splat(1e-6));
            g.from = ((g.from - mid) * k + half) / size;
            g.to = match g.kind {
                GradientKind::Linear => ((g.to - mid) * k + half) / size,
                _ => g.to * k / size,
            };
        }
        out
    }

    fn shape(
        &self,
        at: &Placed,
        paint: og_render::UiPaint,
        mode: og_render::UiMode,
        opacity: f32,
        blend: Blend,
    ) -> DrawCmd {
        self.shape_of(&self.shape, at, paint, mode, opacity, blend)
    }

    fn shape_of(
        &self,
        kind: &ShapeKind,
        at: &Placed,
        paint: og_render::UiPaint,
        mode: og_render::UiMode,
        opacity: f32,
        blend: Blend,
    ) -> DrawCmd {
        let r = if matches!(kind, ShapeKind::Path { .. }) {
            at.rect
        } else {
            self.frame(at)
        };
        DrawCmd::Shape {
            shape: Box::new(og_render::UiShape {
                geometry: self.geometry(kind, at),
                center: Vec2::new(r.x + r.width / 2.0, r.y + r.height / 2.0),
                half: Vec2::new(r.width / 2.0, r.height / 2.0),
                transform: [1.0, 0.0, 0.0, 1.0],
                paint,
                mode,
                opacity,
                blend,
                layer: 0,
                clip: None,
            }),
            clip: at.clip,
        }
    }

    /// What draws before the element's own commands: a layer's start, then
    /// its shadows, outer glows and backdrops, then its fills, inner
    /// shadows and inner glows.
    pub(crate) fn enter(&self, theme: &Theme, at: &Placed) -> Vec<DrawCmd> {
        let mut out = Vec::new();
        if self.masked() {
            out.push(plain_layer(None));
        }
        // A cut (CSS `clip-path`) takes all it draws, to `leave`; a layered
        // one is cut by its composite.
        if let Some(cut) = self.cut(at) {
            out.push(cut);
        }
        if self.layered() {
            let (outer, inner) = self.stages();
            let (material, params) = match outer {
                Some(l) => (l.material.into(), l.params),
                None => match self.effects.iter().find_map(|e| match e {
                    Effect::LayerBlur { radius } => Some(*radius),
                    _ => None,
                }) {
                    Some(radius) => {
                        let mut p = [[0.0; 4]; og_render::UI_MATERIAL_PARAMS];
                        p[0][0] = radius;
                        (og_render::UiMaterial::Builtin(Builtin::Blur), p)
                    }
                    None => (
                        og_render::UiMaterial::Plain,
                        [[0.0; 4]; og_render::UI_MATERIAL_PARAMS],
                    ),
                },
            };
            let mask = match &self.clip {
                // Written in view-box units, it is fitted as a fill is.
                Clip::Mask(Mask::Gradient(g)) => {
                    match self.paint(&Paint::Gradient(g.clone()), theme, at) {
                        og_render::UiPaint::Gradient(g) => Some(og_render::UiMask::Gradient(g)),
                        _ => None,
                    }
                }
                Clip::Mask(Mask::Image { texture, uv }) => Some(og_render::UiMask::Image {
                    texture: *texture,
                    uv: *uv,
                }),
                Clip::Mask(Mask::Atlas { uv }) => theme
                    .art
                    .atlas
                    .map(|texture| og_render::UiMask::Image { texture, uv: *uv }),
                _ => None,
            };
            out.push(DrawCmd::BeginLayer {
                material,
                params,
                mask,
                expand: expand_of(material, &params),
                rect: Rect::default(),
            });
            // The inner stages open inside it, the first innermost.
            for l in inner.iter().rev() {
                let material = l.material.into();
                out.push(DrawCmd::BeginLayer {
                    material,
                    params: l.params,
                    mask: None,
                    expand: expand_of(material, &l.params),
                    rect: Rect::default(),
                });
            }
        }
        // Inside a layer the composite carries the opacity and blend.
        let (opacity, blend) = if self.composited() {
            (at.alpha, Blend::Normal)
        } else {
            (self.opacity * at.alpha, self.blend)
        };
        let o = og_render::UiMode::Fill;
        for e in &self.effects {
            let (mode, color) = match e {
                Effect::DropShadow {
                    offset,
                    blur,
                    spread,
                    color,
                } => (
                    og_render::UiMode::Shadow {
                        offset: Vec2::from(*offset),
                        blur: *blur,
                        spread: *spread,
                    },
                    color,
                ),
                Effect::OuterGlow {
                    blur,
                    spread,
                    color,
                } => (
                    og_render::UiMode::Shadow {
                        offset: Vec2::ZERO,
                        blur: *blur,
                        spread: *spread,
                    },
                    color,
                ),
                _ => continue,
            };
            out.push(self.shape(
                at,
                og_render::UiPaint::Solid(color.resolve(theme)),
                mode,
                opacity,
                Blend::Normal,
            ));
        }
        for e in &self.effects {
            if let Effect::Backdrop {
                blur,
                saturation,
                brightness,
                tint,
                glass,
            } = e
            {
                let (mode, tint) = glass.resolve(
                    *blur,
                    *saturation,
                    *brightness,
                    tint.resolve(theme),
                    glass.own_tint,
                    theme,
                );
                out.push(self.shape(
                    at,
                    og_render::UiPaint::Solid(tint),
                    mode,
                    opacity,
                    Blend::Normal,
                ));
            }
        }
        for f in &self.fills {
            if let Paint::Vector { vector, tint } = f {
                let mut tint = tint.resolve(theme);
                tint[3] *= opacity;
                out.push(DrawCmd::Vector {
                    vector: *vector,
                    rect: at.rect,
                    transform: [1.0, 0.0, 0.0, 1.0],
                    tint,
                    clip: at.clip,
                });
                continue;
            }
            out.push(self.shape(at, self.paint(f, theme, at), o, opacity, blend));
        }
        if let Some(b) = &self.boxes {
            // The backgrounds, cut to the text drawn with it
            // (`clip_to_text`) left out here; blending among themselves in a
            // group of their own when a layer blends.
            let boxed: Vec<BoxLayer> = b
                .backgrounds
                .iter()
                .filter(|l| l.clip != LayerClip::Text)
                .cloned()
                .collect();
            if b.isolate {
                out.push(plain_layer(None));
                out.extend(self.box_layers(&boxed, theme, at, 1.0, |l| l.blend));
                out.push(end_layer(None, at, opacity, blend));
            } else {
                out.extend(self.box_layers(&boxed, theme, at, opacity, |_| blend));
            }
        }
        for e in &self.effects {
            let (mode, color) = match e {
                Effect::InnerShadow {
                    offset,
                    blur,
                    spread,
                    color,
                } => (
                    og_render::UiMode::InnerShadow {
                        offset: Vec2::from(*offset),
                        blur: *blur,
                        spread: *spread,
                    },
                    color,
                ),
                Effect::InnerGlow {
                    blur,
                    spread,
                    color,
                } => (
                    og_render::UiMode::InnerShadow {
                        offset: Vec2::ZERO,
                        blur: *blur,
                        spread: *spread,
                    },
                    color,
                ),
                _ => continue,
            };
            out.push(self.shape(
                at,
                og_render::UiPaint::Solid(color.resolve(theme)),
                mode,
                opacity,
                Blend::Normal,
            ));
        }
        if let Some(b) = self.boxes.as_ref().and_then(|b| b.border.as_ref()) {
            out.extend(self.box_layers(std::slice::from_ref(b), theme, at, opacity, |_| blend));
        }
        if let Some(RiveDraw(player)) = &self.rive {
            let r = at.rect;
            // An artboard that cuts its content to its frame cuts it to the
            // box, in the command's own space, so the item's transform turns
            // the cut with the element (a screen-space clip would cut a turned
            // element by its unturned box).
            let cut = player.artboard.clip;
            if cut {
                out.push(DrawCmd::PushClip {
                    geometry: og_render::UiGeometry::Rect {
                        radii: [0.0; 4],
                        corner: og_render::UiCorner::Round,
                    },
                    center: (r.x + r.width / 2.0, r.y + r.height / 2.0),
                    half: (r.width / 2.0, r.height / 2.0),
                });
            }
            for d in player.draw([r.x, r.y, r.width, r.height], opacity) {
                out.push(rive_cmd(d, at.clip));
            }
            if cut {
                out.push(DrawCmd::PopClip);
            }
        }
        out
    }

    /// What starts the children: the cut to the shape, when it clips.
    pub(crate) fn children(&self, at: &Placed) -> Vec<DrawCmd> {
        if self.clip != Clip::Shape {
            return Vec::new();
        }
        vec![self.clip_to(&self.shape, at)]
    }

    /// Its CSS cut, unless a layer's composite cuts it.
    fn cut(&self, at: &Placed) -> Option<DrawCmd> {
        match &self.clip {
            Clip::Cut(shape) if !self.layered() => Some(self.clip_to(shape, at)),
            _ => None,
        }
    }

    /// The clip to `shape`.
    fn clip_to(&self, shape: &ShapeKind, at: &Placed) -> DrawCmd {
        let r = if matches!(shape, ShapeKind::Path { .. }) {
            at.rect
        } else {
            self.frame(at)
        };
        DrawCmd::PushClip {
            geometry: self.geometry(shape, at),
            center: (r.x + r.width / 2.0, r.y + r.height / 2.0),
            half: (r.width / 2.0, r.height / 2.0),
        }
    }

    /// What draws after the subtree: the end of the cut, the strokes, then
    /// the layer's composite.
    pub(crate) fn leave(&self, theme: &Theme, at: &Placed) -> Vec<DrawCmd> {
        let mut out = Vec::new();
        if self.clip == Clip::Shape {
            out.push(DrawCmd::PopClip);
        }
        out.extend(self.strokes(theme, at));
        if self.cut(at).is_some() {
            out.push(DrawCmd::PopClip);
        }
        out.extend(self.composite(theme, at));
        out
    }

    /// [`ElementDraw::leave`] for a draw whose strokes were drawn with its
    /// fills ([`ElementDraw::strokes`] after [`ElementDraw::enter`]): a
    /// style part's border, under its label, so the two batch on their
    /// levels (docs/ui.md §3.3).
    pub(crate) fn leave_stroked(&self, theme: &Theme, at: &Placed) -> Vec<DrawCmd> {
        let mut out = Vec::new();
        if self.clip == Clip::Shape || self.cut(at).is_some() {
            out.push(DrawCmd::PopClip);
        }
        out.extend(self.composite(theme, at));
        out
    }

    /// Its strokes.
    pub(crate) fn strokes(&self, theme: &Theme, at: &Placed) -> Vec<DrawCmd> {
        let mut out = Vec::new();
        let (opacity, blend) = if self.composited() {
            (at.alpha, Blend::Normal)
        } else {
            (self.opacity * at.alpha, self.blend)
        };
        // Widths and dashes in view-box units scale with its fit (uniform:
        // `meet` scales both axes alike).
        let k = self.fit(at).map_or(1.0, |(k, ..)| k);
        let mode = |s: &Stroke| {
            og_render::UiMode::Stroke(og_render::UiStroke {
                width: s.width * k,
                align: s.align,
                dash: s.dash.map(|d| d * k),
                dash_offset: s.dash_offset * k,
                dash_cap: s.dash_cap,
            })
        };
        for s in &self.strokes {
            out.push(self.shape(at, self.paint(&s.paint, theme, at), mode(s), opacity, blend));
        }
        for (kind, s) in &self.outlines {
            out.push(self.shape_of(
                kind,
                at,
                self.paint(&s.paint, theme, at),
                mode(s),
                opacity,
                blend,
            ));
        }
        out
    }

    /// The layer's composite, when it draws offscreen.
    fn composite(&self, theme: &Theme, at: &Placed) -> Vec<DrawCmd> {
        let mut out = Vec::new();
        let masked = self.masked();
        if self.layered() {
            // Each inner stage composites its whole layer into the next.
            for _ in self.stages().1 {
                out.push(DrawCmd::EndLayer {
                    geometry: None,
                    rect: at.rect,
                    opacity: 1.0,
                    blend: Blend::Normal,
                    clip: at.clip,
                    ground: None,
                });
            }
            // The composite covers the layer; a mask or a material clips
            // to the element's shape only when it clips, and to its CSS cut.
            let geometry = match &self.clip {
                Clip::Shape => Some(self.geometry(&self.shape, at)),
                Clip::Cut(shape) => Some(self.geometry(shape, at)),
                _ => None,
            };
            out.push(DrawCmd::EndLayer {
                geometry,
                rect: at.rect,
                opacity: if masked { 1.0 } else { self.opacity },
                blend: if masked { Blend::Normal } else { self.blend },
                clip: at.clip,
                ground: None,
            });
        }
        if let Some(b) = self.boxes.as_ref().filter(|_| masked) {
            // The mask (CSS Masking 1 §6.1): its layers, the bottom first,
            // each combined with those under it by its operator, in a
            // layer that keeps the subtree's where its alpha is.
            out.push(plain_layer(None));
            out.extend(self.box_layers(&b.masks, theme, at, 1.0, |l| l.blend));
            out.push(end_layer(None, at, 1.0, Blend::DestinationIn));
            let shaped = matches!(self.clip, Clip::Shape);
            out.push(end_layer(
                shaped.then(|| self.geometry(&self.shape, at)),
                at,
                self.opacity,
                self.blend,
            ));
        }
        out
    }

    /// Its text effects over the line box `bounds`. `cell`: a pixel
    /// font's pixel at the text's size (CSS px; 0 for an outline font),
    /// which an outline's width and a shadow's offset are whole multiples
    /// of, at least one, and a shadow is hard: a fractional stroke would
    /// soften the cells' edges ([`snap_to_cells`]).
    pub(crate) fn text_effects(
        &self,
        theme: &Theme,
        bounds: Rect,
        cell: f32,
    ) -> Option<og_render::UiTextEffects> {
        let t = self.text.as_ref()?;
        Some(og_render::UiTextEffects {
            text: 0,
            shadows: t
                .shadows
                .iter()
                .map(|s| og_render::UiTextShadow {
                    offset: Vec2::from(s.offset.map(|v| snap_to_cells(v, cell))),
                    blur: if cell > 0.0 { 0.0 } else { s.blur },
                    color: s.color.resolve(theme),
                })
                .collect(),
            outline: t.outline.as_ref().map(|o| og_render::UiTextOutline {
                width: snap_to_cells(o.width, cell),
                color: o.color.resolve(theme),
                align: o.align,
            }),
            fill: t.fill.as_ref().map(|g| g.resolve(theme)),
            fill_box: [bounds.x, bounds.y, bounds.width, bounds.height],
            transform: [1.0, 0.0, 0.0, 1.0],
        })
    }

    /// Wraps the text commands an element pushed in its effects.
    /// `grid`: the text's font's pixel grid (0: an outline font).
    pub(crate) fn style_text(&self, theme: &Theme, cmds: &mut [DrawCmd], grid: u32) {
        if self.text.is_none() {
            return;
        }
        for c in cmds.iter_mut() {
            if let DrawCmd::Text { bounds, size, .. } = c {
                let cell = if grid > 0 { *size / grid as f32 } else { 0.0 };
                let Some(fx) = self.text_effects(theme, *bounds, cell) else {
                    return;
                };
                let text = std::mem::replace(c, DrawCmd::PopClip);
                *c = DrawCmd::StyledText {
                    text: Box::new(text),
                    effects: Box::new(fx),
                };
            }
        }
    }
}

/// One thing a Rive artboard draws as a draw command, cut to `clip`: a
/// shape; a text piece (styled when it carries a gradient, a stroke or a
/// map of its own); a cut to an outline and its end.
fn rive_cmd(d: crate::rive::Drawn, clip: Rect) -> DrawCmd {
    use crate::rive::Drawn;
    match d {
        Drawn::Shape(shape) => DrawCmd::Shape {
            shape: Box::new(shape),
            clip,
        },
        Drawn::Clip(path) => {
            let (lo, hi) = path.bounds().unwrap_or_default();
            let c = (lo + hi) * 0.5;
            DrawCmd::PushClip {
                geometry: og_render::UiGeometry::Path(std::sync::Arc::new(path.map(|q| q - c))),
                center: (c.x, c.y),
                half: ((hi.x - lo.x) / 2.0, (hi.y - lo.y) / 2.0),
            }
        }
        Drawn::Unclip => DrawCmd::PopClip,
        Drawn::Text(t) => {
            let [bx, by, bw, bh] = t.bounds;
            let text = DrawCmd::Text {
                x: t.pen.x,
                baseline: t.pen.y,
                size: t.size,
                text: t.text,
                color: t.color,
                font: t.font,
                bounds: Rect::new(bx, by, bw, bh),
                clip,
                tracking: t.tracking,
                weight: 0.0,
                glow: None,
                placements: t.placements,
            };
            if t.gradient.is_none() && t.outline.is_none() && t.transform == [1.0, 0.0, 0.0, 1.0] {
                return text;
            }
            let (fill, fill_box) = match t.gradient {
                Some((g, b)) => (Some(g), b),
                None => (None, t.bounds),
            };
            DrawCmd::StyledText {
                text: Box::new(text),
                effects: Box::new(og_render::UiTextEffects {
                    text: 0,
                    shadows: Vec::new(),
                    outline: t.outline.map(|(width, color)| og_render::UiTextOutline {
                        width,
                        color,
                        align: og_render::UiAlign::Center,
                    }),
                    fill,
                    fill_box,
                    transform: t.transform,
                }),
            }
        }
    }
}

/// A text effect's length (CSS px) in whole cells of `cell` px, at least
/// one when it is not zero; `v` itself when `cell` is 0 (an outline font).
pub(crate) fn snap_to_cells(v: f32, cell: f32) -> f32 {
    if cell <= 0.0 || v == 0.0 {
        return v;
    }
    (v.abs() / cell).round().max(1.0) * cell * v.signum()
}

/// A shape's contours as polylines in its own units (`frame`: the rect a
/// rect or ellipse fills), whether they are closed, and its fill rule.
fn outline(kind: &ShapeKind, frame: [f32; 4], tol: f32) -> (Vec<Vec<Vec2>>, bool, FillRule) {
    let [x, y, w, h] = frame;
    match kind {
        ShapeKind::Path {
            points,
            closed,
            filled,
            curves,
            fill_rule,
            contours,
            ..
        } => {
            let whole = [points.len() as u32];
            let counts: &[u32] = if contours.is_empty() {
                &whole
            } else {
                contours
            };
            let mut rest: &[[f32; 2]] = points;
            let mut out = Vec::new();
            for &n in counts {
                let (c, more) = rest.split_at((n as usize).min(rest.len()));
                rest = more;
                let Some(first) = c.first() else { continue };
                let mut line = vec![Vec2::from(*first)];
                if *curves {
                    for seg in c[1..].as_chunks::<3>().0 {
                        let [p0, p1, p2, p3] = [
                            *line.last().unwrap(),
                            seg[0].into(),
                            seg[1].into(),
                            seg[2].into(),
                        ];
                        let len = p0.distance(p1) + p1.distance(p2) + p2.distance(p3);
                        let steps = ((len / tol.max(1e-3)).sqrt().ceil() as usize).clamp(1, 64);
                        for k in 1..=steps {
                            let t = k as f32 / steps as f32;
                            let u = 1.0 - t;
                            line.push(
                                p0 * (u * u * u)
                                    + p1 * (3.0 * u * u * t)
                                    + p2 * (3.0 * u * t * t)
                                    + p3 * (t * t * t),
                            );
                        }
                    }
                } else {
                    line.extend(c[1..].iter().map(|p| Vec2::from(*p)));
                }
                out.push(line);
            }
            (out, *closed || *filled, *fill_rule)
        }
        ShapeKind::Ellipse => {
            let (c, r) = (Vec2::new(x + w * 0.5, y + h * 0.5), Vec2::new(w, h) * 0.5);
            let line = (0..64)
                .map(|k| {
                    let a = k as f32 / 64.0 * std::f32::consts::TAU;
                    c + r * Vec2::new(a.cos(), a.sin())
                })
                .collect();
            (vec![line], true, FillRule::NonZero)
        }
        _ => {
            let line = vec![
                Vec2::new(x, y),
                Vec2::new(x + w, y),
                Vec2::new(x + w, y + h),
                Vec2::new(x, y + h),
            ];
            (vec![line], true, FillRule::NonZero)
        }
    }
}

/// The winding number of a closed polyline around `q`.
fn winding(c: &[Vec2], q: Vec2) -> i32 {
    let mut w = 0;
    for (k, a) in c.iter().enumerate() {
        let b = c[(k + 1) % c.len()];
        let cross = (b.x - a.x) * (q.y - a.y) - (q.x - a.x) * (b.y - a.y);
        if a.y <= q.y && b.y > q.y && cross > 0.0 {
            w += 1;
        } else if a.y > q.y && b.y <= q.y && cross < 0.0 {
            w -= 1;
        }
    }
    w
}

/// The distance from `q` to a polyline.
fn distance(c: &[Vec2], closed: bool, q: Vec2) -> f32 {
    let segs = if closed {
        c.len()
    } else {
        c.len().saturating_sub(1)
    };
    let mut best = c.first().map_or(f32::MAX, |p| p.distance(q));
    for k in 0..segs {
        let (a, b) = (c[k], c[(k + 1) % c.len()]);
        let ab = b - a;
        let t = ((q - a).dot(ab) / ab.length_squared().max(1e-12)).clamp(0.0, 1.0);
        best = best.min((a + ab * t).distance(q));
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at() -> Placed {
        Placed {
            rect: Rect::new(10.0, 20.0, 200.0, 100.0),
            radius: 12.0,
            clip: Rect::new(0.0, 0.0, 1000.0, 1000.0),
            alpha: 1.0,
        }
    }

    #[test]
    fn tokens_resolve_through_the_theme_at_draw_time() {
        let theme = Theme::default();
        let c = ColorRef::token("primary");
        assert_eq!(c.resolve(&theme), theme.palette.primary.linear());
        let mut other = Theme::default();
        other.palette.primary = Srgba::rgb(10, 200, 30);
        assert_eq!(c.resolve(&other), other.palette.primary.linear());
        assert_eq!(ColorRef::token("primry").resolve(&theme), MISSING_TOKEN);
    }

    #[test]
    fn effects_go_under_the_fills_and_strokes_after_the_children() {
        let theme = Theme::default();
        let d = ElementDraw {
            fills: vec![Paint::Solid(ColorRef::token("panel"))],
            strokes: vec![Stroke {
                width: 2.0,
                align: StrokeAlign::Inside,
                dash: [0.0; 2],
                dash_offset: 0.0,
                dash_cap: Cap::Butt,
                paint: Paint::Solid(ColorRef::token("line")),
            }],
            effects: vec![
                Effect::InnerGlow {
                    blur: 4.0,
                    spread: 0.0,
                    color: ColorRef::token("highlight"),
                },
                Effect::DropShadow {
                    offset: [0.0, 4.0],
                    blur: 12.0,
                    spread: 0.0,
                    color: ColorRef::token("shadow"),
                },
            ],
            clip: Clip::Shape,
            ..Default::default()
        };
        let modes = |cmds: &[DrawCmd]| -> Vec<&'static str> {
            cmds.iter()
                .map(|c| match c {
                    DrawCmd::Shape { shape, .. } => match shape.mode {
                        og_render::UiMode::Fill => "fill",
                        og_render::UiMode::Stroke(_) => "stroke",
                        og_render::UiMode::Shadow { .. } => "shadow",
                        og_render::UiMode::InnerShadow { .. } => "inner",
                        og_render::UiMode::Backdrop { .. } => "backdrop",
                    },
                    DrawCmd::PushClip { .. } => "push",
                    DrawCmd::PopClip => "pop",
                    _ => "other",
                })
                .collect()
        };
        assert_eq!(modes(&d.enter(&theme, &at())), ["shadow", "fill", "inner"]);
        assert_eq!(modes(&d.children(&at())), ["push"]);
        assert_eq!(modes(&d.leave(&theme, &at())), ["pop", "stroke"]);
    }

    #[test]
    fn a_mask_or_a_blur_draws_the_subtree_offscreen() {
        let theme = Theme::default();
        let d = ElementDraw {
            effects: vec![Effect::LayerBlur { radius: 6.0 }],
            opacity: 0.5,
            ..Default::default()
        };
        let enter = d.enter(&theme, &at());
        let Some(DrawCmd::BeginLayer {
            material, params, ..
        }) = enter.first()
        else {
            panic!("{enter:?}");
        };
        assert_eq!(*material, og_render::UiMaterial::Builtin(Builtin::Blur));
        assert_eq!(params[0][0], 6.0);
        let leave = d.leave(&theme, &at());
        assert!(
            matches!(leave.last(), Some(DrawCmd::EndLayer { opacity, .. }) if *opacity == 0.5),
            "{leave:?}"
        );
    }

    /// A filled path draws its curves, not a polygon: its contours split
    /// by `contours`, each closed, with its Béziers kept, and its rule;
    /// an unfilled one stays a line with joins and caps.
    #[test]
    fn a_filled_path_keeps_its_contours_curves_and_rule() {
        let square = |o: f32, s: f32| [[o, o], [o + s, o], [o + s, o + s], [o, o + s]];
        let mut points: Vec<[f32; 2]> = square(0.0, 100.0).to_vec();
        // The inner contour: a start and one Bézier back to it.
        points.extend([[30.0, 50.0], [30.0, 10.0], [70.0, 10.0], [30.0, 50.0]]);
        let shape = |filled: bool, curves: bool, contours: Vec<u32>| ShapeKind::Path {
            points: points.clone(),
            closed: true,
            filled,
            curves,
            fill_rule: FillRule::NonZero,
            contours,
            width: 2.0,
            cap: Cap::Butt,
            join: Join::Round,
        };
        // Points as lines: two contours of 4 each.
        let og_render::UiGeometry::Path(p) =
            shape(true, false, vec![4, 4]).geometry(0.0, [100.0; 2])
        else {
            panic!("a filled path is a vector path");
        };
        use og_render::UiVerb::*;
        assert_eq!(
            p.verbs,
            [Move, Line, Line, Line, Close, Move, Line, Line, Line, Close]
        );
        assert_eq!(p.fill_rule, FillRule::NonZero);
        // About the box's centre.
        assert_eq!(p.points[0], Vec2::new(-50.0, -50.0));
        // With curves, the second contour's three points after its start are one cubic.
        let mut curved = points.clone();
        curved.drain(1..4);
        let mut k = shape(true, true, vec![1, 4]);
        if let ShapeKind::Path { points, .. } = &mut k {
            *points = curved;
        }
        let og_render::UiGeometry::Path(p) = k.geometry(0.0, [100.0; 2]) else {
            panic!()
        };
        assert_eq!(p.verbs, [Move, Close, Move, Cubic, Close]);
        // Unfilled: a polyline still.
        assert!(matches!(
            shape(false, false, vec![]).geometry(0.0, [100.0; 2]),
            og_render::UiGeometry::Polyline { .. }
        ));
    }
}
