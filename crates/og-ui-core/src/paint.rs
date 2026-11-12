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
pub use og_draw::{
    TextureHandle, UiAlign as StrokeAlign, UiBlend as Blend, UiBuiltin as Builtin, UiCap as Cap,
    UiCorner as Corner, UiEdge as Edge, UiFillRule as FillRule, UiFit as Fit,
    UiGradientKind as GradientKind, UiJoin as Join, UiMaterialHandle, UiSpread as Spread,
    UiVectorHandle,
};

mod effect;
mod element_draw;

pub use effect::{BackdropStage, Effect, GlassParts, Stroke};

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
    pub fn resolve(&self, theme: &Theme) -> og_draw::UiGradient {
        og_draw::UiGradient {
            kind: self.kind,
            from: Vec2::from(self.from),
            to: Vec2::from(self.to),
            stops: self
                .stops
                .iter()
                .map(|s| og_draw::UiStop {
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
    pub density: f32,
    pub tint: ColorRef,
    pub fit: Fit,
    pub focal: [f32; 2],
    /// CSS px added to where `focal` places it (`object-position`
    /// lengths), and whether it is sampled at the nearest texel (CSS
    /// `image-rendering: pixelated | crisp-edges`).
    pub offset: [f32; 2],
    pub nearest: bool,
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
    pub fn resolve(&self, theme: &Theme) -> og_draw::UiPaint {
        match self {
            Paint::Solid(c) => og_draw::UiPaint::Solid(c.resolve(theme)),
            Paint::Gradient(g) => og_draw::UiPaint::Gradient(g.resolve(theme)),
            Paint::Image(im) => og_draw::UiPaint::Image(og_draw::UiImage {
                texture: im.texture,
                uv: im.uv,
                size: Vec2::from(im.size),
                density: im.density,
                tint: im.tint.resolve(theme),
                fit: im.fit,
                focal: Vec2::from(im.focal),
                offset: Vec2::from(im.offset),
                luminance: false,
                nearest: im.nearest,
            }),
            Paint::Vector { tint, .. } => og_draw::UiPaint::Solid(tint.resolve(theme)),
            Paint::Atlas(a) => match theme.art.atlas {
                Some(texture) => og_draw::UiPaint::Image(og_draw::UiImage {
                    texture,
                    uv: a.uv,
                    size: Vec2::from(a.size),
                    density: 1.0,
                    tint: a.tint.resolve(theme),
                    fit: a.fit,
                    focal: Vec2::from(a.focal),
                    offset: Vec2::ZERO,
                    luminance: false,
                    nearest: false,
                }),
                None => og_draw::UiPaint::Solid([0.0; 4]),
            },
            Paint::GradientImage {
                gradient,
                size,
                fit,
            } => og_draw::UiPaint::GradientImage(Box::new(og_draw::UiGradientImage {
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
    fn resolve(&self, theme: &Theme, b: Rect, d: Rect) -> og_draw::UiPaint {
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
            og_draw::UiPaint::Image(im) => {
                im.luminance = self.luminance;
                Some(&mut im.fit)
            }
            og_draw::UiPaint::GradientImage(g) => {
                g.luminance = self.luminance;
                Some(&mut g.fit)
            }
            _ => None,
        };
        if let Some(og_draw::UiFit::Background(bg)) = fit {
            bg.area = area;
        }
        p
    }
}

/// `geometry` inset by `inset` (left, top, right, bottom): a rectangle's
/// corners less the inset across each (CSS Backgrounds 3 §5.3).
fn inset_geometry(g: og_draw::UiGeometry, inset: [f32; 4]) -> og_draw::UiGeometry {
    match g {
        og_draw::UiGeometry::Rect { radii, corner } if inset != [0.0; 4] => {
            let (vertical, shape) = match corner {
                Corner::Css(c) => (c.vertical, c.shape),
                Corner::Round => (radii, [1.0; 4]),
                Corner::Squircle(n) => (radii, [n.max(1e-3).log2(); 4]),
                Corner::Bevel => (radii, [0.0; 4]),
                Corner::Notch => (radii, [-1.0; 4]),
            };
            let [l, t, r, b] = inset;
            og_draw::UiGeometry::Rect {
                radii: [radii[0] - l, radii[1] - r, radii[2] - r, radii[3] - l].map(|v| v.max(0.0)),
                corner: Corner::Css(og_draw::UiCssCorners {
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

fn plain_layer(mask: Option<og_draw::UiMask>) -> DrawCmd {
    DrawCmd::BeginLayer {
        material: og_draw::UiMaterial::Plain,
        params: [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS],
        mask,
        expand: 0.0,
        rect: Rect::default(),
        filter: None,
        role: Default::default(),
    }
}

fn end_layer(
    geometry: Option<og_draw::UiGeometry>,
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
        /// drawn from the curves themselves ([`og_draw::UiGeometry::Path`]),
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
    pub fn geometry(&self, radius: f32, size: [f32; 2]) -> og_draw::UiGeometry {
        match self {
            ShapeKind::Rect { radii, corner } => og_draw::UiGeometry::Rect {
                radii: radii.unwrap_or([radius; 4]),
                corner: *corner,
            },
            ShapeKind::Ellipse => og_draw::UiGeometry::Ellipse,
            ShapeKind::Polygon {
                sides,
                inner,
                rounding,
                rotation,
            } => og_draw::UiGeometry::Polygon {
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
            } => og_draw::UiGeometry::Arc {
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
                let mut path = og_draw::UiPath::new();
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
                og_draw::UiGeometry::Path(std::sync::Arc::new(path))
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
                            og_draw::flatten_cubic(from, c[0], c[1], c[2], 0.25)
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
                og_draw::UiGeometry::Polyline {
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
    pub params: [[f32; 4]; og_draw::UI_MATERIAL_PARAMS],
    /// An SVG `<filter>` run over what the layer drew before it
    /// composites; the layer is then the filter region.
    pub filter: Option<std::sync::Arc<FilterUse>>,
}

/// An SVG `<filter>` on an element ([`crate::svg::filter`]).
#[derive(Clone, Debug, PartialEq)]
pub struct FilterUse {
    pub filter: crate::svg::filter::Filter,
    /// An SVG element's user space: its map into the view box (its
    /// transform) and its bounding box in it. `None`: a CSS box, whose
    /// user space is CSS px from its border box's top left and whose
    /// bounding box is its border box.
    pub svg: Option<(crate::svg::Affine, [f32; 4])>,
}

impl FilterUse {
    /// Its region and graph in CSS px, for an element drawn in `at`
    /// through `fit` (its view box's scale, centre and the box point the
    /// centre lands on).
    fn resolve(&self, at: &Placed, fit: Option<(f32, Vec2, Vec2)>) -> (Rect, og_draw::UiFilter) {
        let (bbox, map) = match (&self.svg, fit) {
            (Some((t, bbox)), Some((k, mid, half))) => {
                let o = Vec2::new(at.rect.x, at.rect.y) + half - mid * k;
                let [a, b, c, d, e, f] = t.0;
                let fit = crate::element::Affine {
                    a: k,
                    d: k,
                    e: o.x,
                    f: o.y,
                    ..crate::element::Affine::IDENTITY
                };
                let t = crate::element::Affine {
                    a,
                    b,
                    c,
                    d,
                    e,
                    f,
                    ..crate::element::Affine::IDENTITY
                };
                (*bbox, fit.mul(&t))
            }
            _ => (
                [0.0, 0.0, at.rect.width, at.rect.height],
                crate::element::Affine {
                    e: at.rect.x,
                    f: at.rect.y,
                    ..crate::element::Affine::IDENTITY
                },
            ),
        };
        let (region, mut graph) = self.filter.resolve(bbox);
        crate::svg::filter::map(&mut graph, &map);
        (map.map_rect(&region), graph)
    }
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
    /// CSS border sides and outline. Their dash placement and shaded styles
    /// use the same painter as document boxes.
    pub borders: Option<Box<crate::decor::BoxDecor>>,
    /// It draws nothing of its own: its subtree is the `n`-th element an
    /// `feImage` of its parent's filters reads ([`filter_uses`]), drawn
    /// into a layer that composites nowhere ([`crate::LayerRole::Image`]).
    pub element_image: Option<u32>,
}

/// The url filters of an element's draw, in the order its element images
/// (the elements their `feImage`s name, [`crate::svg::filter::Filter::elements`])
/// are numbered across them: its `backdrop-filter`'s url stages, then its
/// layer's filter, then its filter layers in order. The `n`-th image is
/// drawn by a child of the element whose [`ElementDraw::element_image`]
/// is `n`.
pub fn filter_uses<'a>(
    effects: &'a [Effect],
    layer: Option<&'a Layer>,
    filters: &'a [Layer],
) -> Vec<&'a FilterUse> {
    let backdrop = effects.iter().flat_map(|e| match e {
        Effect::BackdropFilter(stages) => stages.as_slice(),
        _ => &[],
    });
    let urls = backdrop.filter_map(|s| match s {
        BackdropStage::Url(u) => Some(&**u),
        _ => None,
    });
    let layers = layer
        .into_iter()
        .chain(filters)
        .filter_map(|l| l.filter.as_deref());
    urls.chain(layers).collect()
}

impl From<Material> for og_draw::UiMaterial {
    fn from(m: Material) -> og_draw::UiMaterial {
        match m {
            Material::Plain => og_draw::UiMaterial::Plain,
            Material::Builtin(b) => og_draw::UiMaterial::Builtin(b),
            Material::Custom(h) => og_draw::UiMaterial::Custom(h),
        }
    }
}

/// How far past what it draws a layer through `material` reaches: a
/// blur's, outline's or ripple's radius and a drop shadow's blur and
/// offset, with the Gaussian's tail.
fn expand_of(
    material: og_draw::UiMaterial,
    params: &[[f32; 4]; og_draw::UI_MATERIAL_PARAMS],
) -> f32 {
    let p = params[0];
    match material {
        og_draw::UiMaterial::Builtin(Builtin::Blur | Builtin::Outline | Builtin::Ripple) => {
            p[0].max(0.0) * 1.5
        }
        og_draw::UiMaterial::Builtin(Builtin::DropShadow) => {
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
            borders: None,
            element_image: None,
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
                        og_draw::UiMode::Fill => "fill",
                        og_draw::UiMode::Stroke(_) => "stroke",
                        og_draw::UiMode::Shadow { .. } => "shadow",
                        og_draw::UiMode::InnerShadow { .. } => "inner",
                        og_draw::UiMode::Backdrop { .. } => "backdrop",
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
        assert_eq!(*material, og_draw::UiMaterial::Builtin(Builtin::Blur));
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
        let og_draw::UiGeometry::Path(p) = shape(true, false, vec![4, 4]).geometry(0.0, [100.0; 2])
        else {
            panic!("a filled path is a vector path");
        };
        use og_draw::UiVerb::*;
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
        let og_draw::UiGeometry::Path(p) = k.geometry(0.0, [100.0; 2]) else {
            panic!()
        };
        assert_eq!(p.verbs, [Move, Close, Move, Cubic, Close]);
        // Unfilled: a polyline still.
        assert!(matches!(
            shape(false, false, vec![]).geometry(0.0, [100.0; 2]),
            og_draw::UiGeometry::Polyline { .. }
        ));
    }
}
