//! CSS paint on a document element's box: background, mask and border
//! image layers, `backdrop-filter`, `filter`, `mix-blend-mode`,
//! `text-shadow`, `clip-path` and `-og-material`. Each is parsed from its
//! text when the stylesheet is read (a bad value is named there) and again
//! when the element builds; what depends on the box (a gradient's angle, a
//! shape's percentages) is resolved from the placed size when it is drawn,
//! so the draw is a function of the box size alone.

use super::css::{Tok, describe, tokens};
use crate::paint::{
    Blend, Builtin, ColorRef, Effect, FillRule, Gradient, GradientKind, Layer, Material, ShapeKind,
    Spread, Stop, TextShadow,
};
use crate::theme::Srgba;
use std::f32::consts::{PI, SQRT_2};

/// A length along an axis: a fraction of the box's, or CSS px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Pos {
    Frac(f32),
    Px(f32),
}

impl Pos {
    fn at(self, whole: f32) -> f32 {
        match self {
            Pos::Frac(f) => f * whole,
            Pos::Px(p) => p,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Shape {
    /// A direction: an angle (radians, CSS's: 0 up, clockwise) or a
    /// corner `to top right` (x, y signs).
    Linear(Dir),
    Radial {
        circle: bool,
        size: Extent,
        at: [Pos; 2],
    },
    Conic {
        from: f32,
        at: [Pos; 2],
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Dir {
    Angle(f32),
    Corner(f32, f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Extent {
    ClosestSide,
    FarthestSide,
    ClosestCorner,
    FarthestCorner,
    Given(Pos, Pos),
}

/// A CSS gradient, its geometry resolved against the box when drawn.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssGradient {
    shape: Shape,
    repeating: bool,
    stops: Vec<(ColorRef, Option<Pos>)>,
}

/// A layer's image as drawn: a gradient, or a texture region.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum BgImage {
    Gradient(CssGradient),
    Texture {
        texture: og_render::TextureHandle,
        uv: [f32; 4],
        /// Sampled at the nearest texel (CSS `image-rendering`).
        nearest: bool,
    },
}

/// A background or mask layer as drawn: its image and how it is sized,
/// placed, repeated, cut and combined.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BgLayer {
    pub image: BgImage,
    pub size: BgSize,
    pub position: [BgPos; 2],
    pub repeat: [og_render::UiRepeat; 2],
    pub origin: BoxEdge,
    pub clip: BoxEdge,
    pub attach: Attach,
    /// A background's blend mode, a mask's Porter-Duff operator.
    pub blend: Blend,
    /// A mask layer's luminance is its alpha (`mask-mode: luminance`).
    pub luminance: bool,
}

/// A border image or mask border as drawn (CSS Backgrounds 3 §6, CSS
/// Masking 1 §7).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BorderImg {
    pub image: BgImage,
    pub spec: BorderSpec,
}

impl BgLayer {
    /// Its paint for a positioning area `area` CSS px: a gradient is drawn
    /// as a tile of its concrete size (CSS Images 3 §5.3: no natural size,
    /// so `auto` takes the area's).
    fn paint(&self, area: [f32; 2]) -> crate::paint::Paint {
        use crate::paint::{ImagePaint, Paint};
        let [aw, ah] = area;
        let bg = |size| og_render::UiBackground {
            size,
            align: og_core::glam::Vec2::new(self.position[0].0, self.position[1].0),
            offset: og_core::glam::Vec2::new(self.position[0].1, self.position[1].1),
            repeat: self.repeat,
            area: [0.0; 4],
        };
        match &self.image {
            BgImage::Gradient(g) => {
                let [w, h] = match self.size {
                    BgSize::Cover | BgSize::Contain => [aw, ah],
                    BgSize::Given([x, y]) => {
                        [x.map_or(aw, |p| p.at(aw)), y.map_or(ah, |p| p.at(ah))]
                    }
                };
                let (w, h) = (w.max(1e-3), h.max(1e-3));
                Paint::GradientImage {
                    gradient: g.resolve(w, h),
                    size: [w, h],
                    fit: og_render::UiFit::Background(bg(og_render::UiBackgroundSize::Given([
                        Some(w),
                        Some(h),
                    ]))),
                }
            }
            BgImage::Texture {
                texture,
                uv,
                nearest,
            } => Paint::Image(ImagePaint {
                texture: *texture,
                uv: *uv,
                size: [0.0; 2],
                tint: ColorRef::Literal(Srgba::WHITE),
                fit: og_render::UiFit::Background(bg(match self.size {
                    BgSize::Cover => og_render::UiBackgroundSize::Cover,
                    BgSize::Contain => og_render::UiBackgroundSize::Contain,
                    BgSize::Given([x, y]) => og_render::UiBackgroundSize::Given([
                        x.map(|p| p.at(aw)),
                        y.map(|p| p.at(ah)),
                    ]),
                })),
                focal: [0.5; 2],
                offset: [0.0; 2],
                nearest: *nearest,
            }),
        }
    }
}

/// What a layer is drawn in, as the box around it: its border box, the
/// edges of its padding and content boxes in from it (left, top, right,
/// bottom), and where the screen and a local scroll put it.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct LayerFrame {
    pub size: [f32; 2],
    pub borders: [f32; 4],
    pub padding: [f32; 4],
    /// The viewport's size (`fixed`).
    pub viewport: [f32; 2],
    /// Its scroll offset and its scrolled content's padding box size
    /// (`local`), when it scrolls.
    pub scroll: Option<([f32; 2], [f32; 2])>,
}

impl LayerFrame {
    fn edge(&self, e: BoxEdge) -> [f32; 4] {
        match e {
            BoxEdge::Border | BoxEdge::Text | BoxEdge::NoClip => [0.0; 4],
            BoxEdge::Padding => self.borders,
            BoxEdge::Content => self.padding,
        }
    }

    /// `l` as a box layer of an element.
    pub(crate) fn layer(&self, l: &BgLayer) -> crate::paint::BoxLayer {
        use crate::paint::{Area, BoxLayer, LayerClip};
        let [w, h] = self.size;
        let [ol, ot, or, ob] = self.edge(l.origin);
        let (area, size) = match (l.attach, self.scroll) {
            (Attach::Fixed, _) => {
                let [vw, vh] = self.viewport;
                (Area::Fixed([0.0, 0.0, vw, vh]), [vw, vh])
            }
            (Attach::Local, Some(([sx, sy], [cw, ch]))) => {
                // The scrolled content's padding box, moved by the scroll;
                // a content origin within it by the padding.
                let [bl, bt, _, _] = self.borders;
                let (dl, dt) = (ol - bl, ot - bt);
                let (dr, db) = (or - self.borders[2], ob - self.borders[3]);
                let (x0, y0) = (bl - sx + dl.max(0.0), bt - sy + dt.max(0.0));
                let (aw, ah) = (
                    (cw - dl.max(0.0) - dr.max(0.0)).max(0.0),
                    (ch - dt.max(0.0) - db.max(0.0)).max(0.0),
                );
                (Area::Box([x0, y0, w - x0 - aw, h - y0 - ah]), [aw, ah])
            }
            _ => (
                Area::Box([ol, ot, or, ob]),
                [(w - ol - or).max(0.0), (h - ot - ob).max(0.0)],
            ),
        };
        let clip = match l.clip {
            BoxEdge::Text => LayerClip::Text,
            BoxEdge::NoClip => LayerClip::Rect([-1e5; 4]),
            e if l.luminance
                || matches!(
                    l.blend,
                    Blend::SourceIn | Blend::SourceOut | Blend::Xor | Blend::DestinationIn
                ) =>
            {
                LayerClip::Rect(self.edge(e))
            }
            e => LayerClip::Box(self.edge(e)),
        };
        BoxLayer {
            paint: l.paint(size),
            area,
            clip,
            blend: l.blend,
            luminance: l.luminance,
        }
    }

    /// The background colour `paint` as the bottom box layer, cut by `clip`.
    pub(crate) fn colour(
        &self,
        paint: crate::paint::Paint,
        clip: BoxEdge,
    ) -> crate::paint::BoxLayer {
        use crate::paint::{Area, BoxLayer, LayerClip};
        BoxLayer {
            paint,
            area: Area::Box([0.0; 4]),
            clip: match clip {
                BoxEdge::Text => LayerClip::Text,
                e => LayerClip::Box(self.edge(e)),
            },
            blend: Blend::Normal,
            luminance: false,
        }
    }

    /// A border image as a box layer: its slices, widths and outsets
    /// resolved against the border widths and its area.
    pub(crate) fn border(&self, b: &BorderImg) -> crate::paint::BoxLayer {
        use crate::paint::{Area, BoxLayer, LayerClip};
        let [w, h] = self.size;
        let s = &b.spec;
        // CSS order (top, right, bottom, left) to the renderer's (left,
        // top, right, bottom); the border widths are left, top, right, bottom.
        let bw = [
            self.borders[1],
            self.borders[2],
            self.borders[3],
            self.borders[0],
        ];
        let len = |v: BiLen, k: usize, whole: f32| match v {
            BiLen::Px(p) => p,
            BiLen::Frac(f) => f * whole,
            BiLen::Times(n) => n * bw[k],
            BiLen::Auto => -1.0,
        };
        let out: Vec<f32> = (0..4).map(|k| len(s.outset[k], k, 0.0).max(0.0)).collect();
        let (aw, ah) = (w + out[1] + out[3], h + out[0] + out[2]);
        let whole = |k: usize| if k % 2 == 0 { ah } else { aw };
        let width: Vec<f32> = (0..4).map(|k| len(s.width[k], k, whole(k))).collect();
        let gradient = matches!(b.image, BgImage::Gradient(_));
        let slice: Vec<f32> = (0..4)
            .map(|k| match s.slice[k] {
                Pos::Px(p) => p.max(0.0),
                // A fraction of a texture is the renderer's (negative); of
                // a gradient, of its area.
                Pos::Frac(f) if gradient => f.max(0.0) * whole(k),
                Pos::Frac(f) => -f.max(0.0),
            })
            .collect();
        let lt = |v: &[f32]| [v[3], v[0], v[1], v[2]];
        let fit = og_render::UiFit::BorderImage(og_render::UiBorderImage {
            slice: lt(&slice),
            width: lt(&width),
            repeat: s.repeat,
            fill: s.fill,
        });
        let paint = match &b.image {
            BgImage::Gradient(g) => crate::paint::Paint::GradientImage {
                gradient: g.resolve(aw.max(1e-3), ah.max(1e-3)),
                size: [aw.max(1e-3), ah.max(1e-3)],
                fit,
            },
            BgImage::Texture {
                texture,
                uv,
                nearest,
            } => crate::paint::Paint::Image(crate::paint::ImagePaint {
                texture: *texture,
                uv: *uv,
                size: [0.0; 2],
                tint: ColorRef::Literal(Srgba::WHITE),
                fit,
                focal: [0.5; 2],
                offset: [0.0; 2],
                nearest: *nearest,
            }),
        };
        let o = [-out[3], -out[0], -out[1], -out[2]];
        BoxLayer {
            paint,
            area: Area::Box(o),
            clip: LayerClip::Rect(o),
            blend: Blend::Normal,
            luminance: s.luminance,
        }
    }
}

/// A `clip-path` shape, over its reference box.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CssClip {
    Inset([Pos; 4], f32),
    /// `rect()`: its top, right, bottom and left edges from the box's top
    /// and left (`None`: `auto`, the box's edge), and its radius.
    Rect([Option<Pos>; 4], f32),
    /// `xywh()`: x, y, width, height and radius.
    Xywh([Pos; 4], f32),
    Ellipse([Option<Pos>; 2], bool, [Pos; 2]),
    Polygon(FillRule, Vec<[Pos; 2]>),
    Path(FillRule, Vec<(Vec<[f32; 2]>, bool)>),
    /// `shape()`: SVG path commands whose numbers are lengths over the box.
    Shape(FillRule, Vec<(char, Vec<Pos>)>),
    /// A `<clipPath>` with `clipPathUnits="objectBoundingBox"`: a path in
    /// fractions of the box.
    Units(FillRule, Vec<(Vec<[f32; 2]>, bool)>),
    /// `url(#id)`, before the element builds it into a path.
    Url(String),
    /// A geometry box alone: the reference box with its corners.
    Box,
}

/// `clip-path`'s reference box (CSS Masking 1 §4.1). On an element with a
/// CSS box, `fill-box` is its content box and `stroke-box` and `view-box`
/// its border box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum GeoBox {
    #[default]
    Border,
    Padding,
    Content,
    Margin,
}

impl GeoBox {
    fn parse(w: &str) -> Option<GeoBox> {
        Some(match w {
            "border-box" | "stroke-box" | "view-box" => GeoBox::Border,
            "padding-box" => GeoBox::Padding,
            "content-box" | "fill-box" => GeoBox::Content,
            "margin-box" => GeoBox::Margin,
            _ => return None,
        })
    }
}

/// The paint of one element: what [`crate::decor::BoxDecor`] adds to its
/// draw.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CssPaint {
    /// `background` layers over its colour, the first on top.
    pub images: Vec<BgLayer>,
    /// The colour's `background-clip` (the last layer's).
    pub color_clip: BoxEdge,
    /// `mask` layers, the first on top.
    pub masks: Vec<BgLayer>,
    pub mask_border: Option<BorderImg>,
    pub border_image: Option<BorderImg>,
    pub effects: Vec<Effect>,
    /// `filter`'s layers, the first innermost.
    pub filters: Vec<Layer>,
    pub blend: Option<Blend>,
    /// `isolation: isolate`.
    pub isolate: bool,
    pub text_shadows: Vec<TextShadow>,
    pub clip: Option<CssClip>,
    /// The clip's reference box, and once the element is laid out its
    /// insets from the border box (top, right, bottom, left; negative
    /// outside it) and the corner radius it takes.
    pub clip_box: GeoBox,
    pub clip_ref: ([f32; 4], f32),
    pub layer: Option<Layer>,
}

// ---- parsing

/// A colour token: any CSS colour ([`crate::css_color`]); `var(--role)`
/// stays a role and a colour over roles an expression, both resolved when
/// drawn.
pub(crate) fn color(t: &Tok) -> Result<ColorRef, String> {
    if let Tok::Func(f, a) = t
        && f == "var"
        && let [v] = a.as_slice()
        && let [Tok::Ident(n)] = v.as_slice()
        && let Some(name) = n.strip_prefix("--")
    {
        return Ok(ColorRef::Token {
            name: name.to_owned(),
            alpha: 1.0,
        });
    }
    let c = crate::css_color::parse(t)?;
    Ok(match c.literal() {
        Some(s) => ColorRef::Literal(s),
        None => ColorRef::Css(std::sync::Arc::new(c)),
    })
}

fn is_color(t: &Tok) -> bool {
    color(t).is_ok()
}

fn number(t: &Tok) -> Result<(f32, &str), String> {
    match t {
        Tok::Num(n, u) => n
            .parse::<f32>()
            .map(|v| (v, u.as_str()))
            .map_err(|_| format!("'{n}' is not a number")),
        t => Err(format!("'{}' is not a number", describe(t))),
    }
}

fn pos(t: &Tok) -> Result<Pos, String> {
    match number(t)? {
        (v, "%") => Ok(Pos::Frac(v / 100.0)),
        (v, "px") => Ok(Pos::Px(v)),
        (0.0, "") => Ok(Pos::Px(0.0)),
        _ => Err(format!("'{}' is not a length: <n>px or <n>%", describe(t))),
    }
}

fn angle(t: &Tok) -> Result<f32, String> {
    match number(t)? {
        (v, "deg") => Ok(v.to_radians()),
        (v, "rad") => Ok(v),
        (v, "turn") => Ok(v * 2.0 * PI),
        (v, "grad") => Ok(v * PI / 200.0),
        (0.0, "") => Ok(0.0),
        _ => Err(format!("'{}' is not an angle", describe(t))),
    }
}

/// `at <x> <y>` (keywords, lengths or percentages); centred by default.
fn position(t: &[Tok]) -> Result<[Pos; 2], String> {
    let word = |w: &str| match w {
        "left" | "top" => Some(Pos::Frac(0.0)),
        "center" => Some(Pos::Frac(0.5)),
        "right" | "bottom" => Some(Pos::Frac(1.0)),
        _ => None,
    };
    let one = |t: &Tok| match t {
        Tok::Ident(w) => word(w).ok_or_else(|| format!("'{w}' is not a position")),
        t => pos(t),
    };
    match t {
        [] => Ok([Pos::Frac(0.5); 2]),
        [Tok::Ident(w)] if w == "top" || w == "bottom" => Ok([Pos::Frac(0.5), one(&t[0])?]),
        [a] => Ok([one(a)?, Pos::Frac(0.5)]),
        // `top left` names y first.
        [Tok::Ident(a), Tok::Ident(b)] if matches!(a.as_str(), "top" | "bottom") => {
            Ok([one(&Tok::Ident(b.clone()))?, one(&Tok::Ident(a.clone()))?])
        }
        [a, b] => Ok([one(a)?, one(b)?]),
        _ => {
            Err("a position is one or two of: left, center, right, top, bottom, <n>px, <n>%".into())
        }
    }
}

/// A `*-gradient(…)` function, or `None` when `t` is not one.
fn gradient(t: &Tok) -> Option<Result<CssGradient, String>> {
    let Tok::Func(f, args) = t else { return None };
    let repeating = f.starts_with("repeating-");
    let kind = f.strip_prefix("repeating-").unwrap_or(f);
    if !matches!(
        kind,
        "linear-gradient" | "radial-gradient" | "conic-gradient"
    ) {
        return None;
    }
    Some(gradient_of(kind, repeating, args).map_err(|e| format!("{f}: {e}")))
}

fn gradient_of(kind: &str, repeating: bool, args: &[Vec<Tok>]) -> Result<CssGradient, String> {
    let first = args.first().ok_or("no colour stops")?;
    let prelude = !first.is_empty() && !is_color(&first[0]);
    let (shape, stops) = match kind {
        "linear-gradient" => {
            let dir = match first.as_slice() {
                _ if !prelude => Dir::Angle(PI),
                [a] => Dir::Angle(angle(a)?),
                [Tok::Ident(to), sides @ ..] if to == "to" => {
                    let (mut x, mut y) = (0.0, 0.0);
                    for s in sides {
                        match s {
                            Tok::Ident(w) if w == "left" => x = -1.0,
                            Tok::Ident(w) if w == "right" => x = 1.0,
                            Tok::Ident(w) if w == "top" => y = -1.0,
                            Tok::Ident(w) if w == "bottom" => y = 1.0,
                            s => return Err(format!("'{}' is not a side", describe(s))),
                        }
                    }
                    match (x, y) {
                        (0.0, 0.0) => return Err("`to` names a side or a corner".into()),
                        (0.0, y) => Dir::Angle(if y < 0.0 { 0.0 } else { PI }),
                        (x, 0.0) => Dir::Angle(if x > 0.0 { PI / 2.0 } else { 1.5 * PI }),
                        (x, y) => Dir::Corner(x, y),
                    }
                }
                _ => return Err("<angle> or to <side> [<side>]".into()),
            };
            (Shape::Linear(dir), &args[prelude as usize..])
        }
        "radial-gradient" => {
            let (mut circle, mut size, mut at) =
                (false, Extent::FarthestCorner, [Pos::Frac(0.5); 2]);
            if prelude {
                let split = first
                    .iter()
                    .position(|t| matches!(t, Tok::Ident(w) if w == "at"))
                    .unwrap_or(first.len());
                if split < first.len() {
                    at = position(&first[split + 1..])?;
                }
                let mut lengths = Vec::new();
                for t in &first[..split] {
                    match t {
                        Tok::Ident(w) if w == "circle" => circle = true,
                        Tok::Ident(w) if w == "ellipse" => circle = false,
                        Tok::Ident(w) => {
                            size = match w.as_str() {
                                "closest-side" => Extent::ClosestSide,
                                "farthest-side" => Extent::FarthestSide,
                                "closest-corner" => Extent::ClosestCorner,
                                "farthest-corner" => Extent::FarthestCorner,
                                _ => return Err(format!("'{w}' is not a radial shape or size")),
                            }
                        }
                        t => lengths.push(pos(t)?),
                    }
                }
                match lengths.as_slice() {
                    [] => {}
                    [r] => {
                        circle = true;
                        size = Extent::Given(*r, *r);
                    }
                    [x, y] => size = Extent::Given(*x, *y),
                    _ => return Err("a radial size is one or two lengths".into()),
                }
            }
            (
                Shape::Radial { circle, size, at },
                &args[prelude as usize..],
            )
        }
        _ => {
            let (mut from, mut at) = (0.0, [Pos::Frac(0.5); 2]);
            if prelude {
                let mut i = 0;
                while i < first.len() {
                    match &first[i] {
                        Tok::Ident(w) if w == "from" => {
                            from = angle(first.get(i + 1).ok_or("from <angle>")?)?;
                            i += 2;
                        }
                        Tok::Ident(w) if w == "at" => {
                            at = position(&first[i + 1..])?;
                            i = first.len();
                        }
                        t => {
                            return Err(format!(
                                "'{}' is not part of a conic prelude",
                                describe(t)
                            ));
                        }
                    }
                }
            }
            (Shape::Conic { from, at }, &args[prelude as usize..])
        }
    };
    let conic = matches!(shape, Shape::Conic { .. });
    let mut out = Vec::new();
    for s in stops {
        let (c, rest) = s.split_first().ok_or("an empty colour stop")?;
        let c = color(c)?;
        let at = |t: &Tok| -> Result<Pos, String> {
            if conic {
                Ok(match number(t)? {
                    (v, "%") => Pos::Frac(v / 100.0),
                    _ => Pos::Frac(angle(t)? / (2.0 * PI)),
                })
            } else {
                pos(t)
            }
        };
        match rest {
            [] => out.push((c, None)),
            [a] => out.push((c, Some(at(a)?))),
            // `red 10% 30%`: two stops of one colour.
            [a, b] => {
                out.push((c.clone(), Some(at(a)?)));
                out.push((c, Some(at(b)?)));
            }
            _ => return Err("a stop is <colour> [<position> [<position>]]".into()),
        }
    }
    if out.len() < 2 {
        return Err("a gradient takes at least two colour stops".into());
    }
    if out.len() > og_render::UI_GRADIENT_STOPS {
        return Err(format!(
            "a gradient takes at most {} stops",
            og_render::UI_GRADIENT_STOPS
        ));
    }
    Ok(CssGradient {
        shape,
        repeating,
        stops: out,
    })
}

/// A background layer's image.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CssImage {
    None,
    Gradient(CssGradient),
    /// `url(…)`: an icon of the UI's atlas by name, or a texture by its
    /// asset handle (`og.load.texture`), as `<image href>` names one.
    Url(String),
}

/// `background-size` of one layer; a percentage is of the box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BgSize {
    Cover,
    Contain,
    /// `None` is `auto`.
    Given([Option<Pos>; 2]),
}

/// `background-position` on one axis: `align × (box − image) + offset`
/// CSS px (`right 10px` is (1, −10), `30%` is (0.3, 0)).
pub(crate) type BgPos = (f32, f32);

/// A box a layer is placed in or cut to (CSS Backgrounds 3 §3.7, 3.8;
/// CSS Masking 1 §6.7, 6.8).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum BoxEdge {
    #[default]
    Border,
    Padding,
    Content,
    /// `background-clip: text`.
    Text,
    /// `mask-clip: no-clip`.
    NoClip,
}

/// `background-attachment`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Attach {
    #[default]
    Scroll,
    Fixed,
    Local,
}

/// One `background` or `mask` layer (CSS Backgrounds 3 §3, CSS Masking 1
/// §6): its image and how it is sized, placed, repeated, cut and combined.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssLayer {
    pub image: CssImage,
    pub size: BgSize,
    pub position: [BgPos; 2],
    pub repeat: [og_render::UiRepeat; 2],
    pub origin: BoxEdge,
    pub clip: BoxEdge,
    pub attach: Attach,
    pub blend: Blend,
    /// `mask-mode: luminance`.
    pub luminance: bool,
}

impl CssLayer {
    pub(crate) fn new(image: CssImage, mask: bool) -> CssLayer {
        CssLayer {
            image,
            size: BgSize::Given([None; 2]),
            position: [(0.0, 0.0); 2],
            repeat: [og_render::UiRepeat::Repeat; 2],
            origin: if mask {
                BoxEdge::Border
            } else {
                BoxEdge::Padding
            },
            clip: BoxEdge::Border,
            attach: Attach::Scroll,
            blend: Blend::Normal,
            luminance: false,
        }
    }
}

/// A box keyword of a layer (`mask`: the SVG boxes as the border box).
fn box_edge(w: &str, mask: bool) -> Option<BoxEdge> {
    Some(match w {
        "border-box" => BoxEdge::Border,
        "padding-box" => BoxEdge::Padding,
        "content-box" => BoxEdge::Content,
        "fill-box" | "stroke-box" | "view-box" if mask => BoxEdge::Border,
        _ => return None,
    })
}

fn attach(w: &str) -> Option<Attach> {
    Some(match w {
        "scroll" => Attach::Scroll,
        "fixed" => Attach::Fixed,
        "local" => Attach::Local,
        _ => return None,
    })
}

/// `mask-composite`: its Porter-Duff operator over the layers under it.
fn composite(w: &str) -> Option<Blend> {
    Some(match w {
        "add" => Blend::Normal,
        "subtract" => Blend::SourceOut,
        "intersect" => Blend::SourceIn,
        "exclude" => Blend::Xor,
        _ => return None,
    })
}

/// `mask-mode`: luminance or not.
fn mask_mode(w: &str) -> Option<bool> {
    match w {
        "luminance" => Some(true),
        "alpha" | "match-source" => Some(false),
        _ => None,
    }
}

/// A `background`'s layers, the first on top, and its trailing colour.
pub(crate) type Background = (Vec<CssLayer>, Option<Tok>);

fn is_image(t: &Tok) -> bool {
    matches!(t, Tok::Func(f, _) if f == "url") || gradient(t).is_some()
}

/// `url(<name>)`, `url("<name>")` or `url(<handle>)`.
pub(crate) fn url(args: &[Vec<Tok>]) -> Result<String, String> {
    let mut out = String::new();
    for t in args.iter().flatten() {
        match t {
            Tok::Str(s) | Tok::Ident(s) => out.push_str(s),
            Tok::Num(n, u) => {
                out.push_str(n);
                out.push_str(u);
            }
            Tok::Slash => out.push('/'),
            t => return Err(format!("url(): '{}' is not part of a name", describe(t))),
        }
    }
    if out.is_empty() {
        return Err("url() names an atlas icon or a texture handle".into());
    }
    Ok(out)
}

fn image(t: &Tok) -> Result<CssImage, String> {
    match t {
        Tok::Ident(w) if w == "none" => Ok(CssImage::None),
        Tok::Func(f, a) if f == "url" => url(a).map(CssImage::Url),
        t => match gradient(t) {
            Some(g) => g.map(CssImage::Gradient),
            None => Err(format!(
                "'{}' is not an image: url(…), a gradient or none",
                describe(t)
            )),
        },
    }
}

/// A `background-position` (one to four values, CSS Backgrounds 3
/// §3.6): keywords, lengths and percentages, an edge keyword followed by
/// its offset.
pub(crate) fn bg_position(t: &[Tok]) -> Result<[BgPos; 2], String> {
    let err = || {
        "a position is one to four of: left, center, right, top, bottom, <n>px, <n>%".to_string()
    };
    // An offset from the start: (align, px).
    let len = |t: &Tok| -> Result<BgPos, String> {
        Ok(match pos(t)? {
            Pos::Frac(f) => (f, 0.0),
            Pos::Px(p) => (0.0, p),
        })
    };
    // Which axis a keyword names (0 x, 1 y, 2 either) and its fraction.
    let word = |t: &Tok| -> Option<(u8, f32)> {
        match t {
            Tok::Ident(w) => match w.as_str() {
                "left" => Some((0, 0.0)),
                "right" => Some((0, 1.0)),
                "top" => Some((1, 0.0)),
                "bottom" => Some((1, 1.0)),
                "center" => Some((2, 0.5)),
                _ => None,
            },
            _ => None,
        }
    };
    let center = (0.5, 0.0);
    match t {
        [] => Ok([(0.0, 0.0); 2]),
        [a] => match word(a) {
            Some((1, f)) => Ok([center, (f, 0.0)]),
            Some((_, f)) => Ok([(f, 0.0), center]),
            None => Ok([len(a)?, center]),
        },
        [a, b] => {
            let (wa, wb) = (word(a), word(b));
            match (wa, wb) {
                // `top left`: y first.
                (Some((1, fa)), Some((k, fb))) if k != 1 => Ok([(fb, 0.0), (fa, 0.0)]),
                (Some((2, fa)), Some((0, fb))) => Ok([(fb, 0.0), (fa, 0.0)]),
                (Some((ka, fa)), Some((kb, fb))) if ka != 1 && kb != 0 => {
                    Ok([(fa, 0.0), (fb, 0.0)])
                }
                (Some((ka, fa)), None) if ka != 1 => Ok([(fa, 0.0), len(b)?]),
                (None, Some((kb, fb))) if kb != 0 => Ok([len(a)?, (fb, 0.0)]),
                (None, None) => Ok([len(a)?, len(b)?]),
                _ => Err(err()),
            }
        }
        _ if t.len() <= 4 => {
            // Edge keywords, each with an optional offset from that edge;
            // a `center` takes the axis the others leave.
            let mut edges = Vec::new();
            let mut i = 0;
            while i < t.len() {
                let (k, f) = word(&t[i]).ok_or_else(err)?;
                let offset = match t.get(i + 1) {
                    Some(o) if word(o).is_none() => {
                        i += 1;
                        Some(len(o)?)
                    }
                    _ => None,
                };
                i += 1;
                if k == 2 && offset.is_some() {
                    return Err(err());
                }
                edges.push((k, f, offset));
            }
            let mut out: [Option<BgPos>; 2] = [None; 2];
            for (k, f, offset) in edges
                .iter()
                .filter(|e| e.0 != 2)
                .chain(edges.iter().filter(|e| e.0 == 2))
            {
                let axis = match k {
                    2 => out.iter().position(Option::is_none).ok_or_else(err)?,
                    k => *k as usize,
                };
                if out[axis].is_some() {
                    return Err(err());
                }
                // An offset from the far edge counts back from it.
                out[axis] = Some(match *offset {
                    None => (*f, 0.0),
                    Some((a, p)) if *f == 1.0 => (1.0 - a, -p),
                    Some(o) => o,
                });
            }
            Ok([out[0].unwrap_or(center), out[1].unwrap_or(center)])
        }
        _ => Err(err()),
    }
}

/// A `background-size` of one layer.
fn bg_size(t: &[Tok]) -> Result<BgSize, String> {
    let one = |t: &Tok| -> Result<Option<Pos>, String> {
        match t {
            Tok::Ident(w) if w == "auto" => Ok(None),
            t => pos(t).map(Some),
        }
    };
    match t {
        [Tok::Ident(w)] if w == "cover" => Ok(BgSize::Cover),
        [Tok::Ident(w)] if w == "contain" => Ok(BgSize::Contain),
        [a] => Ok(BgSize::Given([one(a)?, None])),
        [a, b] => Ok(BgSize::Given([one(a)?, one(b)?])),
        _ => Err("a background-size is cover, contain, or one or two of auto, <n>px, <n>%".into()),
    }
}

fn repeat_word(w: &str) -> Option<og_render::UiRepeat> {
    use og_render::UiRepeat as R;
    Some(match w {
        "repeat" => R::Repeat,
        "no-repeat" => R::NoRepeat,
        "space" => R::Space,
        "round" => R::Round,
        _ => return None,
    })
}

/// A `background-repeat` of one layer.
fn bg_repeat(t: &[Tok]) -> Result<[og_render::UiRepeat; 2], String> {
    use og_render::UiRepeat as R;
    let word = |t: &Tok| match t {
        Tok::Ident(w) => repeat_word(w),
        _ => None,
    };
    match t {
        [Tok::Ident(w)] if w == "repeat-x" => Ok([R::Repeat, R::NoRepeat]),
        [Tok::Ident(w)] if w == "repeat-y" => Ok([R::NoRepeat, R::Repeat]),
        [a] if word(a).is_some() => Ok([word(a).unwrap(); 2]),
        [a, b] if word(a).is_some() && word(b).is_some() => Ok([word(a).unwrap(), word(b).unwrap()]),
        _ => Err("a background-repeat is repeat-x, repeat-y, or one or two of repeat, no-repeat, space, round".into()),
    }
}

/// One layer of the `background` shorthand; `last` may end in a colour.
fn layer(item: &str, last: bool, mask: bool) -> Result<(CssLayer, Option<Tok>), String> {
    let t = tokens(item)?;
    let mut img = None;
    let mut colour = None;
    let mut place = Vec::new();
    let mut size = None;
    let mut repeat = Vec::new();
    let mut boxes = Vec::new();
    let mut l = CssLayer::new(CssImage::None, mask);
    let mut k = 0;
    while k < t.len() {
        let tok = &t[k];
        let ident = match tok {
            Tok::Ident(w) => Some(w.as_str()),
            _ => None,
        };
        if is_image(tok) || ident == Some("none") {
            if img.is_some() {
                return Err("a background layer takes one image".into());
            }
            img = Some(image(tok)?);
        } else if matches!(tok, Tok::Slash) {
            // `<position> / <size>`: one or two sizes.
            let n = t[k + 1..]
                .iter()
                .take(2)
                .take_while(|t| match t {
                    Tok::Num(..) => true,
                    Tok::Ident(w) => matches!(w.as_str(), "auto" | "cover" | "contain"),
                    _ => false,
                })
                .count();
            if place.is_empty() || n == 0 {
                return Err("a background's size follows its position: <position> / <size>".into());
            }
            size = Some(bg_size(&t[k + 1..k + 1 + n])?);
            k += 1 + n;
            continue;
        } else if matches!(tok, Tok::Num(..))
            || matches!(ident, Some("left" | "right" | "top" | "bottom" | "center"))
        {
            place.push(tok.clone());
        } else if ident
            .is_some_and(|w| repeat_word(w).is_some() || w == "repeat-x" || w == "repeat-y")
        {
            repeat.push(tok.clone());
        } else if let Some(e) = ident.and_then(|w| box_edge(w, mask)) {
            boxes.push(e);
        } else if ident == Some(if mask { "no-clip" } else { "text" }) {
            l.clip = if mask { BoxEdge::NoClip } else { BoxEdge::Text };
            boxes.push(l.clip);
        } else if let Some(a) = ident.filter(|_| !mask).and_then(attach) {
            l.attach = a;
        } else if let Some(c) = ident.filter(|_| mask).and_then(composite) {
            l.blend = c;
        } else if let Some(m) = ident.filter(|_| mask).and_then(mask_mode) {
            l.luminance = m;
        } else if last && !mask && colour.is_none() && is_color(tok) {
            colour = Some(tok.clone());
        } else {
            return Err(format!(
                "'{}' is not part of a background layer: an image, <position> [/ <size>], a repeat{}",
                describe(tok),
                if last {
                    ", a colour"
                } else {
                    " (a colour only in the last layer)"
                }
            ));
        }
        k += 1;
    }
    l.image = img.unwrap_or(CssImage::None);
    // One box is the origin and the clip; two the origin, then the clip
    // (`text` and `no-clip` the clip only).
    match boxes.as_slice() {
        [] => {}
        [BoxEdge::Text | BoxEdge::NoClip] => {}
        [e] => (l.origin, l.clip) = (*e, *e),
        [o, c] if !matches!(o, BoxEdge::Text | BoxEdge::NoClip) => (l.origin, l.clip) = (*o, *c),
        _ => return Err("a layer takes one or two boxes: <origin> [<clip>]".into()),
    }
    l.position = bg_position(&place)?;
    if let Some(s) = size {
        l.size = s;
    }
    if !repeat.is_empty() {
        l.repeat = bg_repeat(&repeat)?;
    }
    Ok((l, colour))
}

/// A `background` (or `background-image`) value with an image layer: its
/// layers, the first on top, and the colour under them. `None` when it
/// has no image (a plain colour, lowered as before).
pub(crate) fn background(text: &str) -> Option<Result<Background, String>> {
    let Ok(t) = tokens(text) else { return None };
    if !t.iter().any(is_image) {
        return None;
    }
    let items = super::boxes::items(text);
    let n = items.len();
    Some((|| {
        let mut layers = Vec::new();
        let mut colour = None;
        for (k, item) in items.into_iter().enumerate() {
            let (l, c) = layer(item, k + 1 == n, false)?;
            layers.push(l);
            colour = colour.or(c);
        }
        Ok((layers, colour))
    })())
}

/// The longhands over a `background`'s or `mask`'s layers, by their
/// names after `background-` or `mask-` (`size`, `position`, `repeat`,
/// `origin`, `clip`, `attachment`, `blend-mode`, `mode`, `composite`),
/// each a comma list repeated to the layers' count (CSS Backgrounds 3
/// §3); an empty value is the shorthand's own.
pub(crate) fn longhands(
    layers: &mut [CssLayer],
    values: &[(&str, Option<&str>)],
    mask: bool,
) -> Result<(), String> {
    fn each<T: Copy>(
        text: Option<&str>,
        layers: &mut [CssLayer],
        parse: impl Fn(&[Tok]) -> Result<T, String>,
        set: impl Fn(&mut CssLayer, T),
    ) -> Result<(), String> {
        let Some(text) = text.filter(|t| !t.trim().is_empty()) else {
            return Ok(());
        };
        let values: Vec<T> = super::boxes::items(text)
            .iter()
            .map(|i| tokens(i).and_then(|t| parse(&t)))
            .collect::<Result<_, _>>()?;
        for (k, l) in layers.iter_mut().enumerate() {
            set(l, values[k % values.len()]);
        }
        Ok(())
    }
    fn word<T>(
        what: &'static str,
        f: impl Fn(&str) -> Option<T>,
    ) -> impl Fn(&[Tok]) -> Result<T, String> {
        move |t: &[Tok]| {
            match t {
                [Tok::Ident(w)] => f(w),
                _ => None,
            }
            .ok_or_else(|| format!("not a {what}"))
        }
    }
    for (name, text) in values {
        match *name {
            "size" => each(*text, layers, bg_size, |l, v| l.size = v)?,
            "position" => each(*text, layers, bg_position, |l, v| l.position = v)?,
            "repeat" => each(*text, layers, bg_repeat, |l, v| l.repeat = v)?,
            "origin" => each(*text, layers, word("box", |w| box_edge(w, mask)), |l, v| {
                l.origin = v
            })?,
            "clip" => each(
                *text,
                layers,
                word("clip box", |w| match w {
                    "text" if !mask => Some(BoxEdge::Text),
                    "no-clip" if mask => Some(BoxEdge::NoClip),
                    w => box_edge(w, mask),
                }),
                |l, v| l.clip = v,
            )?,
            "attachment" => each(
                *text,
                layers,
                word("attachment: scroll, fixed, local", attach),
                |l, v| l.attach = v,
            )?,
            "blend-mode" => each(
                *text,
                layers,
                word("blend mode", |w| blend(w).ok()),
                |l, v| l.blend = v,
            )?,
            "mode" => each(
                *text,
                layers,
                word("mask mode: alpha, luminance, match-source", mask_mode),
                |l, v| l.luminance = v,
            )?,
            "composite" => each(
                *text,
                layers,
                word("composite: add, subtract, intersect, exclude", composite),
                |l, v| l.blend = v,
            )?,
            _ => {}
        }
    }
    Ok(())
}

/// The props of the layer, border image and mask border longhands, in
/// [`super::css::STYLE`]'s order.
pub(crate) const C4_PROPS: [&str; 22] = [
    "backgroundOrigin",
    "backgroundClip",
    "backgroundAttachment",
    "backgroundBlendMode",
    "maskSize",
    "maskPosition",
    "maskRepeat",
    "maskOrigin",
    "maskClip",
    "maskMode",
    "maskComposite",
    "borderImageSource",
    "borderImageSlice",
    "borderImageWidth",
    "borderImageOutset",
    "borderImageRepeat",
    "maskBorderSource",
    "maskBorderSlice",
    "maskBorderWidth",
    "maskBorderOutset",
    "maskBorderRepeat",
    "maskBorderMode",
];

/// The layer longhands (after `background-` or `mask-`), each with its
/// element prop.
pub(crate) const BACKGROUND_LONGHANDS: [(&str, &str); 7] = [
    ("size", "backgroundSize"),
    ("position", "backgroundPosition"),
    ("repeat", "backgroundRepeat"),
    ("origin", "backgroundOrigin"),
    ("clip", "backgroundClip"),
    ("attachment", "backgroundAttachment"),
    ("blend-mode", "backgroundBlendMode"),
];
pub(crate) const MASK_LONGHANDS: [(&str, &str); 7] = [
    ("size", "maskSize"),
    ("position", "maskPosition"),
    ("repeat", "maskRepeat"),
    ("origin", "maskOrigin"),
    ("clip", "maskClip"),
    ("mode", "maskMode"),
    ("composite", "maskComposite"),
];

/// `mask-image` or the `mask` shorthand: its layers, the first on top
/// (a `none` layer draws nothing and is left out).
pub(crate) fn masks(text: &str) -> Result<Vec<CssLayer>, String> {
    if text.trim() == "none" {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for item in super::boxes::items(text) {
        let (l, _) = layer(item, false, true)?;
        if l.image != CssImage::None {
            out.push(l);
        }
    }
    Ok(out)
}

/// A `border-image-width` or `-outset` value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BiLen {
    Px(f32),
    /// Of the border image area's side.
    Frac(f32),
    /// Times the border's width.
    Times(f32),
    Auto,
}

/// `border-image-*` or `mask-border-*` other than its source, top, right,
/// bottom, left as CSS writes them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BorderSpec {
    /// Image px (`Px`, a number) or a fraction of the image.
    pub slice: [Pos; 4],
    pub fill: bool,
    pub width: [BiLen; 4],
    pub outset: [BiLen; 4],
    pub repeat: [og_render::UiEdge; 2],
    pub luminance: bool,
}

/// One to four values as CSS's sides: top, right, bottom, left.
fn sides<T: Copy>(t: &[Tok], f: impl Fn(&Tok) -> Result<T, String>) -> Result<[T; 4], String> {
    let v: Vec<T> = t.iter().map(f).collect::<Result<_, _>>()?;
    Ok(match v.as_slice() {
        [a] => [*a; 4],
        [a, b] => [*a, *b, *a, *b],
        [a, b, c] => [*a, *b, *c, *b],
        [a, b, c, d] => [*a, *b, *c, *d],
        _ => return Err("one to four values: top, right, bottom, left".into()),
    })
}

fn bi_len(t: &Tok, pct: bool, auto: bool) -> Result<BiLen, String> {
    match t {
        Tok::Ident(w) if w == "auto" && auto => Ok(BiLen::Auto),
        t => match number(t)? {
            (v, _) if v < 0.0 => Err("a border image's width or outset is not negative".into()),
            (v, "") => Ok(BiLen::Times(v)),
            (v, "px") => Ok(BiLen::Px(v)),
            (v, "%") if pct => Ok(BiLen::Frac(v / 100.0)),
            _ => Err(format!(
                "'{}' is not <n>px, <n>{}",
                describe(t),
                if pct { ", <n>% or auto" } else { "" }
            )),
        },
    }
}

/// A border image's parts by longhand: source, slice, width, outset,
/// repeat (and a mask border's mode), each `None` when not written.
/// `mask` reads `mask-border`'s initial values.
pub(crate) fn border_spec(
    parts: [Option<&str>; 5],
    mode: Option<&str>,
    mask: bool,
) -> Result<BorderSpec, String> {
    use og_render::UiEdge as E;
    let toks = |t: Option<&str>| -> Result<Option<Vec<Tok>>, String> {
        t.filter(|t| !t.trim().is_empty()).map(tokens).transpose()
    };
    let mut b = BorderSpec {
        slice: [if mask { Pos::Px(0.0) } else { Pos::Frac(1.0) }; 4],
        fill: false,
        width: [if mask { BiLen::Auto } else { BiLen::Times(1.0) }; 4],
        outset: [BiLen::Times(0.0); 4],
        repeat: [E::Stretch; 2],
        luminance: false,
    };
    if let Some(t) = toks(parts[1])? {
        b.fill = t.iter().any(|t| matches!(t, Tok::Ident(w) if w == "fill"));
        let n: Vec<Tok> = t
            .into_iter()
            .filter(|t| !matches!(t, Tok::Ident(w) if w == "fill"))
            .collect();
        b.slice = sides(&n, |t| match number(t)? {
            (v, _) if v < 0.0 => Err("a slice is not negative".into()),
            (v, "") => Ok(Pos::Px(v)),
            (v, "%") => Ok(Pos::Frac(v / 100.0)),
            _ => Err(format!("'{}' is not a slice: <n> or <n>%", describe(t))),
        })?;
    }
    if let Some(t) = toks(parts[2])? {
        b.width = sides(&t, |t| bi_len(t, true, true))?;
    }
    if let Some(t) = toks(parts[3])? {
        b.outset = sides(&t, |t| bi_len(t, false, false))?;
    }
    if let Some(t) = toks(parts[4])? {
        let w = |t: &Tok| match t {
            Tok::Ident(w) => Ok(match w.as_str() {
                "stretch" => E::Stretch,
                "repeat" => E::Repeat,
                "round" => E::Round,
                "space" => E::Space,
                _ => return Err(format!("'{w}' is not stretch, repeat, round or space")),
            }),
            t => Err(format!("'{}' is not a repeat", describe(t))),
        };
        b.repeat = match t.as_slice() {
            [a] => [w(a)?; 2],
            [a, b] => [w(a)?, w(b)?],
            _ => return Err("one or two of stretch, repeat, round, space".into()),
        };
    }
    if let Some(m) = mode.filter(|m| !m.trim().is_empty()) {
        b.luminance = match m.trim() {
            "luminance" => true,
            "alpha" => false,
            m => return Err(format!("'{m}' is not a mask border mode: alpha, luminance")),
        };
    }
    Ok(b)
}

/// A `border-image` or `mask-border` shorthand split into its longhands'
/// text: source, slice, width, outset, repeat, then a mask border's mode.
pub(crate) fn border_shorthand(text: &str, mask: bool) -> Result<[String; 6], String> {
    let t = tokens(text)?;
    let mut out: [String; 6] = Default::default();
    let push = |s: &mut String, tok: &Tok| {
        if !s.is_empty() {
            s.push(' ');
        }
        s.push_str(&tok_text(tok));
    };
    // 0 before the slice, 1 in it, 2 the width, 3 the outset.
    let mut part = 0;
    for tok in &t {
        let ident = match tok {
            Tok::Ident(w) => Some(w.as_str()),
            _ => None,
        };
        if is_image(tok) || ident == Some("none") {
            if !out[0].is_empty() {
                return Err("one image".into());
            }
            push(&mut out[0], tok);
            if part > 0 {
                part = 4;
            }
        } else if matches!(tok, Tok::Slash) {
            if !(1..3).contains(&part) {
                return Err(
                    "a width or outset follows the slice: <slice> / <width> / <outset>".into(),
                );
            }
            part += 1;
        } else if matches!(ident, Some("stretch" | "repeat" | "round" | "space")) {
            push(&mut out[4], tok);
            part = 4;
        } else if mask && matches!(ident, Some("alpha" | "luminance")) {
            push(&mut out[5], tok);
        } else if matches!(tok, Tok::Num(..)) || ident == Some("fill") || ident == Some("auto") {
            if part == 0 || part == 4 {
                if !out[1].is_empty() {
                    return Err("one slice".into());
                }
                part = 1;
            }
            push(&mut out[part.min(3)], tok);
        } else {
            return Err(format!("'{}' is not part of a border image", describe(tok)));
        }
    }
    border_spec(
        [
            None,
            Some(&out[1]),
            Some(&out[2]),
            Some(&out[3]),
            Some(&out[4]),
        ],
        Some(&out[5]),
        mask,
    )?;
    if !out[0].is_empty() && out[0] != "none" {
        image(&tokens(&out[0])?[0])?;
    }
    Ok(out)
}

/// A token as CSS text again.
fn tok_text(t: &Tok) -> String {
    match t {
        Tok::Ident(s) | Tok::Hash(s) => s.clone(),
        Tok::Num(n, u) => format!("{n}{u}"),
        Tok::Str(s) => format!("{s:?}"),
        Tok::Func(f, args) => format!(
            "{f}({})",
            args.iter()
                .map(|a| a.iter().map(tok_text).collect::<Vec<_>>().join(" "))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Tok::Slash => "/".into(),
    }
}

/// A border image source: `none`, `url(…)` or a gradient.
pub(crate) fn border_source(text: &str) -> Result<CssImage, String> {
    match tokens(text)?.as_slice() {
        [t] => image(t),
        _ => Err("one image: url(…), a gradient or none".into()),
    }
}

/// The longhands of `border-image` and `mask-border`: (CSS name after the
/// prefix, prop). `mode` is the mask border's only.
pub(crate) const BORDER_IMAGE_PROPS: [(&str, &str); 5] = [
    ("source", "borderImageSource"),
    ("slice", "borderImageSlice"),
    ("width", "borderImageWidth"),
    ("outset", "borderImageOutset"),
    ("repeat", "borderImageRepeat"),
];
pub(crate) const MASK_BORDER_PROPS: [(&str, &str); 6] = [
    ("source", "maskBorderSource"),
    ("slice", "maskBorderSlice"),
    ("width", "maskBorderWidth"),
    ("outset", "maskBorderOutset"),
    ("repeat", "maskBorderRepeat"),
    ("mode", "maskBorderMode"),
];

/// The props a C4 declaration writes, each with its text, or `None` when
/// `name` is not one of them: the background and mask layer longhands,
/// `mask` and `mask-image`, and `border-image`, `mask-border` and their
/// longhands. A bad value is named.
pub(crate) fn lower(
    name: &str,
    value: &str,
) -> Option<Result<Vec<(&'static str, String)>, String>> {
    let v = value.trim().to_owned();
    let one = |prop: &'static str| vec![(prop, v.clone())];
    let check_layer = |mask: bool, key: &str| -> Result<(), String> {
        let mut l = [CssLayer::new(CssImage::None, mask)];
        longhands(&mut l, &[(key, Some(value))], mask)
    };
    let border = |props: &[(&'static str, &'static str)],
                  mask: bool|
     -> Result<Vec<(&'static str, String)>, String> {
        let parts = border_shorthand(value, mask)?;
        Ok(props.iter().zip(parts).map(|((_, p), v)| (*p, v)).collect())
    };
    Some(match name {
        "mask" | "mask-image" => masks(value).map(|_| {
            let mut out = one("maskImage");
            if name == "mask" {
                // The shorthand resets the longhands and the mask border.
                out.extend(MASK_LONGHANDS.iter().map(|(_, p)| (*p, String::new())));
                out.extend(MASK_BORDER_PROPS.iter().map(|(_, p)| (*p, String::new())));
            }
            out
        }),
        "border-image" => border(&BORDER_IMAGE_PROPS, false),
        "mask-border" => border(&MASK_BORDER_PROPS, true),
        n => {
            if let Some(key) = n.strip_prefix("background-")
                && let Some((_, prop)) = BACKGROUND_LONGHANDS.iter().find(|(k, _)| *k == key)
            {
                check_layer(false, key).map(|_| one(prop))
            } else if let Some(key) = n.strip_prefix("mask-border-")
                && let Some((_, prop)) = MASK_BORDER_PROPS.iter().find(|(k, _)| *k == key)
            {
                border_check(key, value, true).map(|_| one(prop))
            } else if let Some(key) = n.strip_prefix("mask-")
                && let Some((_, prop)) = MASK_LONGHANDS.iter().find(|(k, _)| *k == key)
            {
                check_layer(true, key).map(|_| one(prop))
            } else if let Some(key) = n.strip_prefix("border-image-")
                && let Some((_, prop)) = BORDER_IMAGE_PROPS.iter().find(|(k, _)| *k == key)
            {
                border_check(key, value, false).map(|_| one(prop))
            } else {
                return None;
            }
        }
    })
}

fn border_check(key: &str, value: &str, mask: bool) -> Result<(), String> {
    let mut parts = [None; 5];
    let mut mode = None;
    match key {
        "source" => return border_source(value).map(|_| ()),
        "mode" => mode = Some(value),
        k => {
            let i = ["source", "slice", "width", "outset", "repeat"]
                .iter()
                .position(|p| *p == k)
                .unwrap_or(1);
            parts[i] = Some(value);
        }
    }
    border_spec(parts, mode, mask).map(|_| ())
}

/// One CSS filter function (Filter Effects 1 §13), in the order written.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FilterFn {
    /// `blur(<σ>)`, CSS px.
    Blur(f32),
    /// A colour function as its matrix.
    Matrix(og_render::UiColorMatrix),
    /// `drop-shadow([<colour>] <x> <y> [<blur>])`; no colour is
    /// `currentColor`.
    DropShadow {
        offset: [f32; 2],
        blur: f32,
        color: Option<ColorRef>,
    },
    /// `url(#id)`: an inline SVG `<filter>`, written `url(#id)`.
    Url(String),
}

/// `filter` or `backdrop-filter` (`what`): its functions in order, or none.
pub(crate) fn filter_fns(text: &str, what: &str) -> Result<Vec<FilterFn>, String> {
    let t = tokens(text)?;
    if matches!(t.as_slice(), [Tok::Ident(w)] if w == "none") {
        return Ok(Vec::new());
    }
    t.into_iter()
        .map(|t| match t {
            Tok::Func(f, a) => {
                let a: Vec<Tok> = a.into_iter().flatten().collect();
                filter_fn(&f, &a).map_err(|e| format!("{f}(): {e}"))
            }
            t => Err(format!("'{}' is not a {what} function", describe(&t))),
        })
        .collect()
}

fn filter_fn(f: &str, a: &[Tok]) -> Result<FilterFn, String> {
    use FilterFn::Matrix;
    // An amount: a number or percentage, not negative; 1 when left out.
    let amount = |most: f32| -> Result<f32, String> {
        let v = match a {
            [] => 1.0,
            [t] => match number(t)? {
                (v, "%") => v / 100.0,
                (v, "") => v,
                _ => return Err(format!("'{}' is not an amount: <n> or <n>%", describe(t))),
            },
            _ => return Err("one amount".into()),
        };
        if v < 0.0 {
            return Err("an amount is not negative".into());
        }
        Ok(v.min(most))
    };
    let diag = |k: f32, o: f32| og_render::UiColorMatrix {
        rows: [[k, 0.0, 0.0, o], [0.0, k, 0.0, o], [0.0, 0.0, k, o]],
        opacity: 1.0,
    };
    // A 3×3 over the colour.
    let m3 = |m: [[f32; 3]; 3]| og_render::UiColorMatrix {
        rows: m.map(|r| [r[0], r[1], r[2], 0.0]),
        opacity: 1.0,
    };
    Ok(match f {
        "url" => FilterFn::Url(match a {
            [Tok::Hash(h)] => format!("url({h})"),
            [Tok::Str(s)] if s.starts_with('#') => format!("url({s})"),
            _ => return Err("url(#id) names a <filter>".into()),
        }),
        "blur" => FilterFn::Blur(match a {
            [] => 0.0,
            [t] => px(t)?.max(0.0),
            _ => return Err("one length".into()),
        }),
        "brightness" => Matrix(diag(amount(f32::MAX)?, 0.0)),
        "contrast" => {
            let k = amount(f32::MAX)?;
            Matrix(diag(k, 0.5 - 0.5 * k))
        }
        "invert" => {
            let k = amount(1.0)?;
            Matrix(diag(1.0 - 2.0 * k, k))
        }
        "opacity" => Matrix(og_render::UiColorMatrix {
            opacity: amount(1.0)?,
            ..Default::default()
        }),
        "grayscale" => {
            let k = 1.0 - amount(1.0)?;
            Matrix(m3([
                [
                    0.2126 + 0.7874 * k,
                    0.7152 - 0.7152 * k,
                    0.0722 - 0.0722 * k,
                ],
                [
                    0.2126 - 0.2126 * k,
                    0.7152 + 0.2848 * k,
                    0.0722 - 0.0722 * k,
                ],
                [
                    0.2126 - 0.2126 * k,
                    0.7152 - 0.7152 * k,
                    0.0722 + 0.9278 * k,
                ],
            ]))
        }
        "sepia" => {
            let k = 1.0 - amount(1.0)?;
            Matrix(m3([
                [0.393 + 0.607 * k, 0.769 - 0.769 * k, 0.189 - 0.189 * k],
                [0.349 - 0.349 * k, 0.686 + 0.314 * k, 0.168 - 0.168 * k],
                [0.272 - 0.272 * k, 0.534 - 0.534 * k, 0.131 + 0.869 * k],
            ]))
        }
        "saturate" => {
            let s = amount(f32::MAX)?;
            Matrix(m3([
                [0.213 + 0.787 * s, 0.715 - 0.715 * s, 0.072 - 0.072 * s],
                [0.213 - 0.213 * s, 0.715 + 0.285 * s, 0.072 - 0.072 * s],
                [0.213 - 0.213 * s, 0.715 - 0.715 * s, 0.072 + 0.928 * s],
            ]))
        }
        "hue-rotate" => {
            let h = match a {
                [] => 0.0,
                [t] => angle(t)?,
                _ => return Err("one angle".into()),
            };
            let (sn, c) = og_core::math::sin_cos(h);
            Matrix(m3([
                [
                    0.213 + c * 0.787 - sn * 0.213,
                    0.715 - c * 0.715 - sn * 0.715,
                    0.072 - c * 0.072 + sn * 0.928,
                ],
                [
                    0.213 - c * 0.213 + sn * 0.143,
                    0.715 + c * 0.285 + sn * 0.140,
                    0.072 - c * 0.072 - sn * 0.283,
                ],
                [
                    0.213 - c * 0.213 - sn * 0.787,
                    0.715 - c * 0.715 + sn * 0.715,
                    0.072 + c * 0.928 + sn * 0.072,
                ],
            ]))
        }
        "drop-shadow" => {
            let mut lengths = Vec::new();
            let mut color = None;
            for t in a {
                if matches!(t, Tok::Num(..)) {
                    lengths.push(px(t)?);
                } else if color.is_none() {
                    color = Some(self::color(t)?);
                } else {
                    return Err("one colour".into());
                }
            }
            match lengths.as_slice() {
                [x, y, rest @ ..] if rest.len() <= 1 => FilterFn::DropShadow {
                    offset: [*x, *y],
                    blur: rest.first().copied().unwrap_or(0.0).max(0.0),
                    color,
                },
                _ => return Err("[<colour>] <x> <y> [<blur>]".into()),
            }
        }
        f => {
            return Err(format!(
                "not a filter: url(#id), blur, brightness, contrast, drop-shadow, grayscale, hue-rotate, invert, opacity, saturate, sepia (writing {f})"
            ));
        }
    })
}

/// `m` then `n`: one matrix, as if `n` read `m`'s result unclamped.
fn then(m: &og_render::UiColorMatrix, n: &og_render::UiColorMatrix) -> og_render::UiColorMatrix {
    let mut rows = [[0.0; 4]; 3];
    for (i, row) in rows.iter_mut().enumerate() {
        for (j, v) in row.iter_mut().enumerate() {
            *v = (0..3).map(|k| n.rows[i][k] * m.rows[k][j]).sum::<f32>()
                + if j == 3 { n.rows[i][3] } else { 0.0 };
        }
    }
    og_render::UiColorMatrix {
        rows,
        opacity: m.opacity * n.opacity,
    }
}

/// Whether `m` keeps every colour in 0..1: then the clamp after it does
/// nothing, and the next function folds into it exactly.
fn contained(m: &og_render::UiColorMatrix) -> bool {
    m.rows.iter().all(|r| {
        let lo = r[3] + r[..3].iter().map(|v| v.min(0.0)).sum::<f32>();
        let hi = r[3] + r[..3].iter().map(|v| v.max(0.0)).sum::<f32>();
        lo >= -1e-5 && hi <= 1.0 + 1e-5
    })
}

/// `filter`'s functions as layers, the first innermost: a blur, a drop
/// shadow (its colour resolved by `color`), an SVG `<filter>` (found by
/// `url`), and each run of colour functions as one colour matrix, split
/// where a function could leave 0..1 so the clamp CSS does after each one
/// still falls between them. A `url(#id)` that names no filter drops the
/// whole list, as Filter Effects 1 §11 says.
pub(crate) fn filter_layers(
    fns: &[FilterFn],
    color: impl Fn(Option<&ColorRef>) -> [f32; 4],
    mut url: impl FnMut(&str) -> Option<std::sync::Arc<crate::paint::FilterUse>>,
) -> Vec<Layer> {
    let layer = |b: Builtin, params| Layer {
        material: Material::Builtin(b),
        params,
        filter: None,
    };
    let mut out = Vec::new();
    let mut run: Option<og_render::UiColorMatrix> = None;
    for f in fns {
        if let FilterFn::Matrix(m) = f {
            run = Some(match run.take() {
                Some(r) if contained(&r) => then(&r, m),
                Some(r) => {
                    out.push(layer(Builtin::ColorMatrix, r.params()));
                    *m
                }
                None => *m,
            });
            continue;
        }
        if let Some(r) = run.take() {
            out.push(layer(Builtin::ColorMatrix, r.params()));
        }
        let mut p = [[0.0; 4]; og_render::UI_MATERIAL_PARAMS];
        match f {
            FilterFn::Blur(sigma) => {
                // The layer blur's radius is twice its standard deviation.
                p[0][0] = 2.0 * sigma;
                out.push(layer(Builtin::Blur, p));
            }
            FilterFn::DropShadow {
                offset,
                blur,
                color: c,
            } => {
                p[0] = [offset[0], offset[1], *blur, 0.0];
                p[1] = color(c.as_ref());
                out.push(layer(Builtin::DropShadow, p));
            }
            FilterFn::Url(u) => {
                let Some(f) = url(u) else {
                    return Vec::new();
                };
                out.push(Layer {
                    filter: Some(f),
                    ..Layer::default()
                });
            }
            FilterFn::Matrix(_) => unreachable!("folded above"),
        }
    }
    if let Some(r) = run {
        out.push(layer(Builtin::ColorMatrix, r.params()));
    }
    out
}

fn px(t: &Tok) -> Result<f32, String> {
    match number(t)? {
        (v, "px") | (v @ 0.0, "") => Ok(v),
        _ => Err(format!("'{}' is not a length in px", describe(t))),
    }
}

/// `backdrop-filter`'s functions as its stages, in order: a run of
/// colour functions is one matrix while it keeps colour in 0..1 (as
/// `filter`'s layers fold), a `drop-shadow()` with no colour takes
/// `current`, and a `url(#id)` is the `<filter>` `url` finds; one that
/// names none drops the list. `None` for no functions.
pub(crate) fn backdrop_stages(
    fns: &[FilterFn],
    current: ColorRef,
    mut url: impl FnMut(&str) -> Option<std::sync::Arc<crate::paint::FilterUse>>,
) -> Option<Vec<crate::paint::BackdropStage>> {
    use crate::paint::BackdropStage as S;
    let mut out: Vec<S> = Vec::new();
    for f in fns {
        match f {
            FilterFn::Matrix(m) => match out.last_mut() {
                Some(S::Matrix(r)) if contained(r) => *r = then(r, m),
                _ => out.push(S::Matrix(*m)),
            },
            FilterFn::Blur(s) => out.push(S::Blur(*s)),
            FilterFn::DropShadow {
                offset,
                blur,
                color,
            } => out.push(S::DropShadow {
                offset: *offset,
                blur: *blur,
                color: color.clone().unwrap_or_else(|| current.clone()),
            }),
            FilterFn::Url(u) => out.push(S::Url(url(u)?)),
        }
    }
    (!out.is_empty()).then_some(out)
}

/// `isolation`: whether it isolates.
pub(crate) fn isolation(text: &str) -> Result<bool, String> {
    match text.trim() {
        "auto" => Ok(false),
        "isolate" => Ok(true),
        w => Err(format!("'{w}' is not an isolation: auto | isolate")),
    }
}

/// `mix-blend-mode`.
pub(crate) fn blend(text: &str) -> Result<Blend, String> {
    Ok(match text.trim() {
        "normal" => Blend::Normal,
        "multiply" => Blend::Multiply,
        "screen" => Blend::Screen,
        "plus-lighter" => Blend::Add,
        "overlay" => Blend::Overlay,
        "soft-light" => Blend::SoftLight,
        "color-dodge" => Blend::ColorDodge,
        "darken" => Blend::Darken,
        "lighten" => Blend::Lighten,
        "color-burn" => Blend::ColorBurn,
        "hard-light" => Blend::HardLight,
        "difference" => Blend::Difference,
        "exclusion" => Blend::Exclusion,
        "hue" => Blend::Hue,
        "saturation" => Blend::Saturation,
        "color" => Blend::Color,
        "luminosity" => Blend::Luminosity,
        w => {
            return Err(format!("'{w}' is not a blend mode: {BLEND_MODES}"));
        }
    })
}

const BLEND_MODES: &str = "normal, multiply, screen, overlay, darken, lighten, color-dodge, color-burn, hard-light, soft-light, difference, exclusion, hue, saturation, color, luminosity, plus-lighter";

/// `text-shadow: <x> <y> [<blur>] [<colour>], …` (unset colour: the text's,
/// given as `current`).
pub(crate) fn text_shadows(text: &str, current: ColorRef) -> Result<Vec<TextShadow>, String> {
    if text.trim() == "none" {
        return Ok(Vec::new());
    }
    super::boxes::items(text)
        .into_iter()
        .map(|item| {
            let mut lengths = Vec::new();
            let mut c = None;
            for t in tokens(item)? {
                if matches!(t, Tok::Num(..)) {
                    lengths.push(px(&t)?);
                } else if c.is_none() {
                    c = Some(color(&t)?);
                } else {
                    return Err(format!("a text-shadow takes one colour: {item}"));
                }
            }
            match lengths.as_slice() {
                [x, y, rest @ ..] if rest.len() <= 1 => Ok(TextShadow {
                    offset: [*x, *y],
                    blur: rest.first().copied().unwrap_or(0.0).max(0.0),
                    color: c.unwrap_or_else(|| current.clone()),
                }),
                _ => Err(format!(
                    "a text-shadow is <x> <y> [<blur>] [<colour>]: {item}"
                )),
            }
        })
        .collect()
}

fn rule(t: &[Tok]) -> (FillRule, &[Tok]) {
    match t.split_first() {
        Some((Tok::Ident(w), rest)) if w == "evenodd" => (FillRule::EvenOdd, rest),
        Some((Tok::Ident(w), rest)) if w == "nonzero" => (FillRule::NonZero, rest),
        _ => (FillRule::NonZero, t),
    }
}

/// `clip-path`: `[<basic-shape> || <geometry-box>] | url(#id) | none`;
/// the basic shapes `inset()`, `rect()`, `xywh()`, `circle()`,
/// `ellipse()`, `polygon()`, `path()` and `shape()`.
pub(crate) fn clip(text: &str) -> Result<Option<(CssClip, GeoBox)>, String> {
    const FORMS: &str = "a clip-path is [inset() | rect() | xywh() | circle() | ellipse() | polygon() | path() | shape()] [<geometry-box>], a <geometry-box>, url(#id) or none";
    let t = tokens(text)?;
    let mut shape = None;
    let mut geo = None;
    for tok in &t {
        match tok {
            Tok::Ident(w) if w == "none" && t.len() == 1 => return Ok(None),
            Tok::Ident(w) if geo.is_none() => {
                geo = Some(GeoBox::parse(w).ok_or_else(|| format!("'{w}': {FORMS}"))?);
            }
            Tok::Func(f, a) if shape.is_none() => shape = Some((f.as_str(), a)),
            _ => return Err(FORMS.into()),
        }
    }
    let geo = geo.unwrap_or_default();
    let Some((f, args)) = shape else {
        return Ok(Some((CssClip::Box, geo)));
    };
    if f == "url" {
        if t.len() > 1 {
            return Err("clip-path: url(#id) takes no box".into());
        }
        let id = match args.as_slice() {
            [a] => match a.as_slice() {
                [Tok::Hash(h)] => h.clone(),
                [Tok::Str(s)] if s.starts_with('#') => s.clone(),
                _ => return Err("clip-path: url(#id) names a <clipPath>".into()),
            },
            _ => return Err("clip-path: url(#id) names a <clipPath>".into()),
        };
        return Ok(Some((CssClip::Url(format!("url({id})")), geo)));
    }
    let split_at = |a: &[Tok]| -> Result<(Vec<Tok>, [Pos; 2]), String> {
        match a
            .iter()
            .position(|t| matches!(t, Tok::Ident(w) if w == "at"))
        {
            Some(k) => Ok((a[..k].to_vec(), position(&a[k + 1..])?)),
            None => Ok((a.to_vec(), [Pos::Frac(0.5); 2])),
        }
    };
    // `… round <r>`: the lengths before it and the radius.
    let rounded = |a: &[Tok]| -> Result<(usize, f32), String> {
        let k = a
            .iter()
            .position(|t| matches!(t, Tok::Ident(w) if w == "round"))
            .unwrap_or(a.len());
        let radius = match &a[k..] {
            [] => 0.0,
            [_, r] => px(r)?,
            _ => return Err(format!("{f}(… round <n>px)")),
        };
        Ok((k, radius))
    };
    let shape = match (f, args.as_slice()) {
        ("inset", [a]) => {
            let (k, radius) = rounded(a)?;
            let p: Vec<Pos> = a[..k].iter().map(pos).collect::<Result<_, _>>()?;
            let sides = match p.as_slice() {
                [a] => [*a; 4],
                [a, b] => [*a, *b, *a, *b],
                [a, b, c] => [*a, *b, *c, *b],
                [a, b, c, d] => [*a, *b, *c, *d],
                _ => return Err("inset() takes one to four lengths".into()),
            };
            CssClip::Inset(sides, radius)
        }
        ("rect", [a]) => {
            let (k, radius) = rounded(a)?;
            let p: Vec<Option<Pos>> = a[..k]
                .iter()
                .map(|t| match t {
                    Tok::Ident(w) if w == "auto" => Ok(None),
                    t => pos(t).map(Some),
                })
                .collect::<Result<_, _>>()?;
            let [t, r, b, l] = p[..] else {
                return Err("rect(<top> <right> <bottom> <left> [round <r>])".into());
            };
            CssClip::Rect([t, r, b, l], radius)
        }
        ("xywh", [a]) => {
            let (k, radius) = rounded(a)?;
            let p: Vec<Pos> = a[..k].iter().map(pos).collect::<Result<_, _>>()?;
            let [x, y, w, h] = p[..] else {
                return Err("xywh(<x> <y> <width> <height> [round <r>])".into());
            };
            CssClip::Xywh([x, y, w, h], radius)
        }
        ("shape", items) => shape_fn(items)?,
        ("circle", [a]) => {
            let (r, at) = split_at(a)?;
            let r = match r.as_slice() {
                [] => None,
                [r] => Some(pos(r)?),
                _ => return Err("circle(<r> at <x> <y>)".into()),
            };
            CssClip::Ellipse([r, r], true, at)
        }
        ("ellipse", [a]) => {
            let (r, at) = split_at(a)?;
            match r.as_slice() {
                [] => CssClip::Ellipse([None; 2], false, at),
                [x, y] => CssClip::Ellipse([Some(pos(x)?), Some(pos(y)?)], false, at),
                _ => return Err("ellipse(<rx> <ry> at <x> <y>)".into()),
            }
        }
        ("polygon", points) => {
            let (fill, first) = rule(points.first().map_or(&[][..], |p| p.as_slice()));
            let skip = first.is_empty() as usize;
            let mut out = Vec::new();
            for (k, p) in points.iter().enumerate().skip(skip) {
                let p = if k == 0 { first } else { p.as_slice() };
                match p {
                    [x, y] => out.push([pos(x)?, pos(y)?]),
                    _ => return Err("polygon([<rule>,] <x> <y>, …)".into()),
                }
            }
            if out.len() < 3 {
                return Err("a polygon takes at least three points".into());
            }
            CssClip::Polygon(fill, out)
        }
        ("path", args) => {
            let (fill, d) = match args {
                [d] => (FillRule::NonZero, d.as_slice()),
                [r, d] => (rule(r).0, d.as_slice()),
                _ => return Err("path([<rule>,] \"<svg path data>\")".into()),
            };
            let [Tok::Str(d)] = d else {
                return Err("path(\"<svg path data>\")".into());
            };
            let p = crate::svg::Path::parse(d)?;
            CssClip::Path(fill, super::build::svg::subpaths(&p))
        }
        (f, _) => return Err(format!("{f}() is not a clip-path shape")),
    };
    Ok(Some((shape, geo)))
}

/// `shape([<rule>,] from <x> <y>, <command>, …)` (CSS Shapes 2 §3.2.1) as
/// SVG path commands: `move`, `line`, `hline`, `vline`, `curve … with
/// <c1> [/ <c2>]`, `smooth … [with <c>]`, `arc … of <rx> [<ry>] [cw | ccw]
/// [large | small] [rotate <angle>]`, each `to` or `by`, and `close`.
fn shape_fn(items: &[Vec<Tok>]) -> Result<CssClip, String> {
    const FORM: &str = "shape([<rule>,] from <x> <y>, <command>, …)";
    let (fill, first) = rule(items.first().map_or(&[][..], |p| p.as_slice()));
    let skip = first.is_empty() as usize;
    let mut cmds: Vec<(char, Vec<Pos>)> = Vec::new();
    for (k, item) in items.iter().enumerate().skip(skip) {
        let item = if k == 0 { first } else { item.as_slice() };
        let word = |i: usize| match item.get(i) {
            Some(Tok::Ident(w)) => Some(w.as_str()),
            _ => None,
        };
        // The command letter, lower case for `by`.
        let rel = |upper: char| -> Result<char, String> {
            match word(1) {
                Some("to") => Ok(upper),
                Some("by") => Ok(upper.to_ascii_lowercase()),
                _ => Err(format!(
                    "{FORM}: '{}' takes to or by",
                    word(0).unwrap_or("")
                )),
            }
        };
        let pair = |i: usize| -> Result<Vec<Pos>, String> {
            match (item.get(i), item.get(i + 1)) {
                (Some(x), Some(y)) => Ok(vec![pos(x)?, pos(y)?]),
                _ => Err(format!("{FORM}: a point is <x> <y>")),
            }
        };
        let cmd = match word(0) {
            Some("from") if cmds.is_empty() => ('M', pair(1)?),
            _ if cmds.is_empty() => return Err(format!("{FORM}: it starts with from")),
            Some("close") => ('Z', Vec::new()),
            Some("move") => (rel('M')?, pair(2)?),
            Some("line") => (rel('L')?, pair(2)?),
            Some("hline") => (rel('H')?, vec![pos(item.get(2).ok_or(FORM)?)?]),
            Some("vline") => (rel('V')?, vec![pos(item.get(2).ok_or(FORM)?)?]),
            Some(c @ ("curve" | "smooth")) => {
                let end = pair(2)?;
                let with = item
                    .iter()
                    .position(|t| matches!(t, Tok::Ident(w) if w == "with"));
                let controls: Vec<Pos> = match with {
                    None => Vec::new(),
                    Some(w) => item[w + 1..]
                        .iter()
                        .filter(|t| !matches!(t, Tok::Slash))
                        .map(pos)
                        .collect::<Result<_, _>>()?,
                };
                let letter = match (c, controls.len()) {
                    ("curve", 2) => 'Q',
                    ("curve", 4) => 'C',
                    ("smooth", 0) => 'T',
                    ("smooth", 2) => 'S',
                    _ => return Err(format!("{FORM}: {c} … with <x> <y> [/ <x> <y>]")),
                };
                let letter = rel(letter)?;
                (letter, [controls, end].concat())
            }
            Some("arc") => {
                let end = pair(2)?;
                let of = item
                    .iter()
                    .position(|t| matches!(t, Tok::Ident(w) if w == "of"))
                    .ok_or_else(|| format!("{FORM}: arc … of <radius>"))?;
                let mut radii = Vec::new();
                let (mut sweep, mut large, mut rotate) = (0.0, 0.0, 0.0);
                let mut i = of + 1;
                while i < item.len() {
                    match &item[i] {
                        Tok::Ident(w) if w == "cw" => sweep = 1.0,
                        Tok::Ident(w) if w == "ccw" => sweep = 0.0,
                        Tok::Ident(w) if w == "large" => large = 1.0,
                        Tok::Ident(w) if w == "small" => large = 0.0,
                        Tok::Ident(w) if w == "rotate" => {
                            i += 1;
                            rotate = angle(item.get(i).ok_or(FORM)?)?.to_degrees();
                        }
                        t => radii.push(pos(t)?),
                    }
                    i += 1;
                }
                let (rx, ry) = match radii[..] {
                    [r] => (r, r),
                    [rx, ry] => (rx, ry),
                    _ => return Err(format!("{FORM}: arc … of <rx> [<ry>]")),
                };
                let letter = rel('A')?;
                (
                    letter,
                    vec![
                        rx,
                        ry,
                        Pos::Px(rotate),
                        Pos::Px(large),
                        Pos::Px(sweep),
                        end[0],
                        end[1],
                    ],
                )
            }
            _ => {
                return Err(format!(
                    "{FORM}: '{}' is not a command",
                    item.first().map(describe).unwrap_or_default()
                ));
            }
        };
        cmds.push(cmd);
    }
    if cmds.len() < 2 {
        return Err(format!("{FORM}: it takes a command after from"));
    }
    Ok(CssClip::Shape(fill, cmds))
}

/// `-og-material: <name>(<n> …[, <colour>])`: the element and its subtree
/// drawn offscreen and composited through one of the engine's built-in
/// materials, its parameters in the order the material lists them (a
/// colour as linear RGBA); or `-og-material: "<asset>" [<n> …]`, a `type
/// ui-material` asset by its path under `assets/` with numbers over its
/// defaults in `param` order, four a parameter, as a style's dressing
/// writes one (bound to its handle when the document loads).
pub(crate) fn material(text: &str) -> Result<Option<crate::dress::DressMaterial>, String> {
    let t = tokens(text)?;
    if let [Tok::Str(path), rest @ ..] = t.as_slice() {
        let mut layer = Layer::default();
        let max = 4 * layer.params.len();
        if rest.len() > max {
            return Err(format!("\"{path}\" takes at most {max} parameter numbers"));
        }
        for (k, n) in rest.iter().enumerate() {
            layer.params[k / 4][k % 4] = match number(n)? {
                (x, "%") => x / 100.0,
                (x, _) => x,
            };
        }
        return Ok(Some(crate::dress::DressMaterial {
            layer,
            asset: Some(path.clone()),
            params: rest.len(),
        }));
    }
    let (name, args) = match t.as_slice() {
        [Tok::Ident(w)] if w == "none" => return Ok(None),
        [Tok::Ident(w)] => (w.as_str(), &[][..]),
        [Tok::Func(f, a)] => (f.as_str(), a.as_slice()),
        _ => {
            return Err(
                "-og-material: <name>(<params>) or \"<ui-material asset>\" <params>".into(),
            );
        }
    };
    let builtin = match name {
        "dissolve" => Builtin::Dissolve,
        "shine" => Builtin::Shine,
        "hologram" => Builtin::Hologram,
        "glitch" => Builtin::Glitch,
        "ripple" => Builtin::Ripple,
        "pixelate" => Builtin::Pixelate,
        "color-adjust" => Builtin::ColorAdjust,
        "outline" => Builtin::Outline,
        "blur" => Builtin::Blur,
        n => {
            return Err(format!(
                "'{n}' is not a material: dissolve, shine, hologram, glitch, ripple, pixelate, color-adjust, outline, blur"
            ));
        }
    };
    let mut layer = Layer {
        material: Material::Builtin(builtin),
        params: Default::default(),
        filter: None,
    };
    if args.len() > layer.params.len() {
        return Err(format!(
            "{name}() takes at most {} groups",
            layer.params.len()
        ));
    }
    for (k, a) in args.iter().enumerate() {
        layer.params[k] = match a.as_slice() {
            [c] if is_color(c) => match color(c)? {
                ColorRef::Literal(s) => s.linear(),
                ColorRef::Token { .. } | ColorRef::Css(_) => {
                    return Err("a material's colour is a literal".into());
                }
            },
            nums if nums.len() <= 4 => {
                let mut v = [0.0; 4];
                for (j, n) in nums.iter().enumerate() {
                    v[j] = match number(n)? {
                        (x, "%") => x / 100.0,
                        (_, "deg" | "rad" | "turn") => angle(n)?,
                        (x, _) => x,
                    };
                }
                v
            }
            _ => return Err("a material group is up to four numbers, or a colour".into()),
        };
    }
    Ok(Some(crate::dress::DressMaterial {
        layer,
        asset: None,
        params: 0,
    }))
}

/// `scroll-snap-type` as [`crate::layout::Element::snap`].
pub(crate) fn snap_type(text: &str) -> Result<u8, String> {
    let w: Vec<&str> = text.split_whitespace().collect();
    let axes =
        match w.first().copied() {
            Some("none") if w.len() == 1 => return Ok(0),
            Some("x" | "inline") => 1,
            Some("y" | "block") => 2,
            Some("both") => 3,
            _ => return Err(
                "scroll-snap-type: none | x | y | block | inline | both [mandatory | proximity]"
                    .into(),
            ),
        };
    match w.get(1..).unwrap_or_default() {
        [] | ["mandatory"] => Ok(axes),
        ["proximity"] => Ok(axes | 4),
        _ => Err("scroll-snap-type's strictness is mandatory or proximity".into()),
    }
}

/// `scroll-snap-align` (block, then inline) as
/// [`crate::layout::Element::snap_align`].
pub(crate) fn snap_align(text: &str) -> Result<u8, String> {
    let one = |w: &str| match w {
        "none" => Ok(0u8),
        "start" => Ok(1),
        "center" => Ok(2),
        "end" => Ok(3),
        w => Err(format!(
            "'{w}' is not a snap alignment: none, start, center, end"
        )),
    };
    match text.split_whitespace().collect::<Vec<_>>().as_slice() {
        [a] => {
            let a = one(a)?;
            Ok(a | a << 2)
        }
        [block, inline] => Ok(one(inline)? | one(block)? << 2),
        _ => Err("scroll-snap-align takes one or two values".into()),
    }
}

// ---- resolving against the box

impl CssGradient {
    /// The gradient over a `w` × `h` box.
    pub(crate) fn resolve(&self, w: f32, h: f32) -> Gradient {
        let (w, h) = (w.max(1e-3), h.max(1e-3));
        let (kind, mut from, mut to, length) = match self.shape {
            Shape::Linear(dir) => {
                let d = match dir {
                    Dir::Angle(a) => [a.sin(), -a.cos()],
                    // Perpendicular to the diagonal through the other two
                    // corners, so they sit on the 50% line.
                    Dir::Corner(x, y) => {
                        let v = [x * h, y * w];
                        let n = (v[0] * v[0] + v[1] * v[1]).sqrt();
                        [v[0] / n, v[1] / n]
                    }
                };
                let len = (w * d[0]).abs() + (h * d[1]).abs();
                let c = [w / 2.0, h / 2.0];
                (
                    GradientKind::Linear,
                    [(c[0] - d[0] * len / 2.0) / w, (c[1] - d[1] * len / 2.0) / h],
                    [(c[0] + d[0] * len / 2.0) / w, (c[1] + d[1] * len / 2.0) / h],
                    len,
                )
            }
            Shape::Radial { circle, size, at } => {
                let c = [at[0].at(w), at[1].at(h)];
                let (dx, dy) = (
                    [c[0].abs(), (w - c[0]).abs()],
                    [c[1].abs(), (h - c[1]).abs()],
                );
                let side = |far: bool| {
                    let pick = |a: [f32; 2]| if far { a[0].max(a[1]) } else { a[0].min(a[1]) };
                    (pick(dx), pick(dy))
                };
                let (rx, ry) = match size {
                    Extent::Given(x, y) => (x.at(w), y.at(h)),
                    Extent::ClosestSide | Extent::FarthestSide => {
                        let (x, y) = side(size == Extent::FarthestSide);
                        if circle {
                            let r = if size == Extent::FarthestSide {
                                x.max(y)
                            } else {
                                x.min(y)
                            };
                            (r, r)
                        } else {
                            (x, y)
                        }
                    }
                    Extent::ClosestCorner | Extent::FarthestCorner => {
                        let (x, y) = side(size == Extent::FarthestCorner);
                        if circle {
                            let r = x.hypot(y);
                            (r, r)
                        } else {
                            (x * SQRT_2, y * SQRT_2)
                        }
                    }
                };
                let (rx, ry) = (rx.max(1e-3), ry.max(1e-3));
                (
                    GradientKind::Radial,
                    [c[0] / w, c[1] / h],
                    [rx / w, ry / h],
                    rx,
                )
            }
            Shape::Conic { from, at } => (
                GradientKind::Conic,
                [at[0].at(w) / w, at[1].at(h) / h],
                [from - PI / 2.0, 0.0],
                1.0,
            ),
        };
        let mut offsets: Vec<Option<f32>> = self
            .stops
            .iter()
            .map(|(_, p)| p.map(|p| p.at(length) / length))
            .collect();
        fix_stops(&mut offsets);
        let mut offsets: Vec<f32> = offsets.into_iter().map(|o| o.unwrap_or(0.0)).collect();
        let mut spread = Spread::Pad;
        if self.repeating {
            spread = Spread::Repeat;
            let (p0, p1) = (offsets[0], *offsets.last().unwrap());
            let span = (p1 - p0).max(1e-4);
            match kind {
                GradientKind::Linear => {
                    let (f, t) = (from, to);
                    for k in 0..2 {
                        from[k] = f[k] + (t[k] - f[k]) * p0;
                        to[k] = f[k] + (t[k] - f[k]) * p1;
                    }
                    for o in &mut offsets {
                        *o = (*o - p0) / span;
                    }
                }
                GradientKind::Radial if p0 <= 0.0 => {
                    to = [to[0] * p1, to[1] * p1];
                    for o in &mut offsets {
                        *o /= span;
                    }
                }
                _ => spread = Spread::Pad,
            }
        }
        Gradient {
            kind,
            from,
            to,
            stops: self
                .stops
                .iter()
                .zip(offsets)
                .map(|((c, _), offset)| Stop {
                    offset,
                    color: c.clone(),
                })
                .collect(),
            spread,
            dither: true,
        }
    }
}

/// CSS Images 3 §3.5.3: an unpositioned first stop is at 0, last at 1;
/// a stop before an earlier one moves up to it; runs of unpositioned
/// stops share their span evenly.
fn fix_stops(o: &mut [Option<f32>]) {
    let n = o.len();
    if o[0].is_none() {
        o[0] = Some(0.0);
    }
    if o[n - 1].is_none() {
        o[n - 1] = Some(1.0);
    }
    let mut most = f32::MIN;
    for v in o.iter_mut().flatten() {
        most = most.max(*v);
        *v = most;
    }
    let mut k = 0;
    while k < n {
        if o[k].is_some() {
            k += 1;
            continue;
        }
        let a = k - 1;
        let b = (k..n).find(|&j| o[j].is_some()).unwrap();
        let (pa, pb) = (o[a].unwrap(), o[b].unwrap());
        for (j, v) in o.iter_mut().enumerate().take(b).skip(k) {
            *v = Some(pa + (pb - pa) * (j - a) as f32 / (b - a) as f32);
        }
        k = b;
    }
}

/// A cubic's controls for a quarter ellipse.
const KAPPA: f32 = 0.552_284_8;

/// A rounded rectangle's outline as straight and quarter-ellipse cubics
/// (a square corner is a straight cubic).
fn rrect(x0: f32, y0: f32, x1: f32, y1: f32, radius: f32) -> Vec<[f32; 2]> {
    let (x1, y1) = (x1.max(x0), y1.max(y0));
    let rr = radius.min((x1 - x0) / 2.0).min((y1 - y0) / 2.0).max(0.0);
    let k = rr * (1.0 - KAPPA);
    vec![
        [x0 + rr, y0],
        [x0 + rr, y0],
        [x1 - rr, y0],
        [x1 - rr, y0],
        [x1 - k, y0],
        [x1, y0 + k],
        [x1, y0 + rr],
        [x1, y0 + rr],
        [x1, y1 - rr],
        [x1, y1 - rr],
        [x1, y1 - k],
        [x1 - k, y1],
        [x1 - rr, y1],
        [x1 - rr, y1],
        [x0 + rr, y1],
        [x0 + rr, y1],
        [x0 + k, y1],
        [x0, y1 - k],
        [x0, y1 - rr],
        [x0, y1 - rr],
        [x0, y0 + rr],
        [x0, y0 + rr],
        [x0, y0 + k],
        [x0 + k, y0],
        [x0 + rr, y0],
    ]
}

impl CssClip {
    /// The shape over a `w` × `h` border box, as a filled path in CSS px
    /// from its top left: resolved over its reference box, `insets` from
    /// the border box (top, right, bottom, left), whose corners take
    /// `radius`.
    pub(crate) fn resolve(&self, w: f32, h: f32, (insets, radius): ([f32; 4], f32)) -> ShapeKind {
        let [t, r, b, l] = insets;
        let mut shape = self.local((w - l - r).max(0.0), (h - t - b).max(0.0), radius);
        if let ShapeKind::Path { points, .. } = &mut shape {
            for p in points {
                *p = [p[0] + l, p[1] + t];
            }
        }
        shape
    }

    /// The shape over a `w` × `h` reference box from its top left.
    fn local(&self, w: f32, h: f32, radius: f32) -> ShapeKind {
        let path = |contours: Vec<(Vec<[f32; 2]>, bool)>, fill_rule, curves| ShapeKind::Path {
            contours: contours.iter().map(|(p, _)| p.len() as u32).collect(),
            points: contours.into_iter().flat_map(|(p, _)| p).collect(),
            closed: true,
            filled: true,
            curves,
            fill_rule,
            width: 0.0,
            cap: Default::default(),
            join: Default::default(),
        };
        match self {
            CssClip::Inset([t, r, b, l], radius) => {
                let (x0, y0) = (l.at(w), t.at(h));
                let pts = rrect(x0, y0, w - r.at(w), h - b.at(h), *radius);
                path(vec![(pts, true)], FillRule::NonZero, true)
            }
            CssClip::Rect([t, r, b, l], radius) => {
                let at = |p: &Option<Pos>, whole: f32, none: f32| p.map_or(none, |p| p.at(whole));
                let pts = rrect(
                    at(l, w, 0.0),
                    at(t, h, 0.0),
                    at(r, w, w),
                    at(b, h, h),
                    *radius,
                );
                path(vec![(pts, true)], FillRule::NonZero, true)
            }
            CssClip::Xywh([x, y, ww, hh], radius) => {
                let (x0, y0) = (x.at(w), y.at(h));
                let pts = rrect(x0, y0, x0 + ww.at(w), y0 + hh.at(h), *radius);
                path(vec![(pts, true)], FillRule::NonZero, true)
            }
            CssClip::Box => path(
                vec![(rrect(0.0, 0.0, w, h, radius), true)],
                FillRule::NonZero,
                true,
            ),
            CssClip::Units(fill, contours) => {
                let scaled = contours
                    .iter()
                    .map(|(p, c)| (p.iter().map(|[x, y]| [x * w, y * h]).collect(), *c))
                    .collect();
                path(scaled, *fill, true)
            }
            CssClip::Shape(fill, cmds) => {
                // SVG path data with the lengths resolved over the box: a
                // pair's x of its width, y of its height; an arc's radii
                // likewise, its rotation and flags as written.
                let mut d = String::new();
                for (c, args) in cmds {
                    d.push(*c);
                    for (k, a) in args.iter().enumerate() {
                        let whole = match (c.to_ascii_uppercase(), k) {
                            ('H', _) => w,
                            ('V', _) => h,
                            ('A', 2..=4) => 0.0,
                            (_, k) if k % 2 == 0 => w,
                            _ => h,
                        };
                        d.push_str(&format!(" {}", a.at(whole)));
                    }
                    d.push(' ');
                }
                match crate::svg::Path::parse(&d) {
                    Ok(p) => path(super::build::svg::subpaths(&p), *fill, true),
                    Err(_) => path(Vec::new(), *fill, true),
                }
            }
            CssClip::Url(_) => path(Vec::new(), FillRule::NonZero, true),
            CssClip::Ellipse(r, circle, at) => {
                let c = [at[0].at(w), at[1].at(h)];
                let (cx, cy) = ([c[0], w - c[0]], [c[1], h - c[1]]);
                // CSS's default: closest-side.
                let side = |a: [f32; 2]| a[0].abs().min(a[1].abs());
                let (rx, ry) = if *circle {
                    let r = r[0].map_or(side(cx).min(side(cy)), |p| {
                        // A percentage of a circle's radius is of the box's
                        // normalised diagonal.
                        match p {
                            Pos::Frac(f) => f * w.hypot(h) / SQRT_2,
                            Pos::Px(v) => v,
                        }
                    });
                    (r, r)
                } else {
                    (
                        r[0].map_or(side(cx), |p| p.at(w)),
                        r[1].map_or(side(cy), |p| p.at(h)),
                    )
                };
                let (kx, ky) = (rx * KAPPA, ry * KAPPA);
                let [x, y] = c;
                let pts = vec![
                    [x + rx, y],
                    [x + rx, y + ky],
                    [x + kx, y + ry],
                    [x, y + ry],
                    [x - kx, y + ry],
                    [x - rx, y + ky],
                    [x - rx, y],
                    [x - rx, y - ky],
                    [x - kx, y - ry],
                    [x, y - ry],
                    [x + kx, y - ry],
                    [x + rx, y - ky],
                    [x + rx, y],
                ];
                path(vec![(pts, true)], FillRule::NonZero, true)
            }
            CssClip::Polygon(fill, points) => {
                // Straight cubics, so one path form carries every shape.
                let p: Vec<[f32; 2]> = points.iter().map(|[x, y]| [x.at(w), y.at(h)]).collect();
                let mut pts = vec![p[0]];
                for k in 1..=p.len() {
                    let (a, b) = (p[k - 1], p[k % p.len()]);
                    pts.extend([a, b, b]);
                }
                path(vec![(pts, true)], *fill, true)
            }
            CssClip::Path(fill, contours) => path(contours.clone(), *fill, true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The resolved geometry a browser draws. Ways it could fail: CSS's
    /// angle convention (0 up, clockwise) read as maths', a gradient line
    /// not lengthened to the corners on a non-square box, unpositioned
    /// stops not shared evenly, a radial size from the wrong sides, or a
    /// conic start not turned from up to +x.
    #[test]
    fn gradients_resolve_as_css_draws_them() {
        let g = |s: &str| match background(s).unwrap().unwrap().0.remove(0).image {
            CssImage::Gradient(g) => g,
            i => panic!("{i:?}"),
        };
        let close =
            |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4;
        // `to right` across a 200 × 100 box.
        let r = g("linear-gradient(to right, red, blue)").resolve(200.0, 100.0);
        assert!(
            close(r.from, [0.0, 0.5]) && close(r.to, [1.0, 0.5]),
            "{r:?}"
        );
        // 45deg on 200 × 100: the line is 200·sin45 + 100·cos45 long.
        let r = g("linear-gradient(45deg, red, blue)").resolve(200.0, 100.0);
        let half = (200.0 + 100.0) * std::f32::consts::FRAC_1_SQRT_2 / 2.0
            * std::f32::consts::FRAC_1_SQRT_2;
        assert!(
            close(r.to, [(100.0 + half) / 200.0, (50.0 - half) / 100.0]),
            "{r:?}"
        );
        // Default is to bottom; stops 0, 0.25 (fixed), 0.5 (shared), 1.
        let r = g("linear-gradient(red, lime 25%, #000, blue)").resolve(100.0, 100.0);
        let o: Vec<f32> = r.stops.iter().map(|s| s.offset).collect();
        assert!(close(r.from, [0.5, 0.0]) && close(r.to, [0.5, 1.0]));
        assert!((o[2] - 0.625).abs() < 1e-4 && o[3] == 1.0, "{o:?}");
        // A closest-side circle a quarter in on 200 × 100: radius 50.
        let r =
            g("radial-gradient(circle closest-side at 25% 50%, red, blue)").resolve(200.0, 100.0);
        assert!(
            close(r.from, [0.25, 0.5]) && close(r.to, [0.25, 0.5]),
            "{r:?}"
        );
        let r = g("radial-gradient(farthest-side, red, blue)").resolve(200.0, 100.0);
        assert!(close(r.to, [0.5, 0.5]), "{r:?}");
        let r = g("conic-gradient(from 90deg, red, blue)").resolve(10.0, 10.0);
        assert!(r.to[0].abs() < 1e-5, "from 90deg starts at +x: {r:?}");
        assert!(background("linear-gradient(red)").unwrap().is_err());
        assert!(
            background("#fff").is_none(),
            "a plain colour lowers as before"
        );
    }

    /// A filter chain as layers, in order, each colour run one matrix that
    /// gives what CSS gives. Ways it could fail: a matrix composed in the
    /// wrong order (invert then contrast is not contrast then invert), a
    /// run folded across a function that leaves 0..1 (brightness(2)'s
    /// clamp lost before invert), a drop shadow put outside the colour
    /// functions written after it, or CSS's blur (a standard deviation)
    /// passed as the layer blur's radius.
    #[test]
    fn filter_functions_chain_in_order() {
        let layers = |s: &str| {
            filter_layers(
                &filter_fns(s, "filter").unwrap(),
                |c| c.map_or([0.0, 0.0, 0.0, 1.0], |_| [1.0, 0.0, 0.0, 1.0]),
                |_| None,
            )
        };
        let kinds = |l: &[Layer]| -> Vec<Builtin> {
            l.iter()
                .map(|l| match l.material {
                    Material::Builtin(b) => b,
                    _ => Builtin::None,
                })
                .collect()
        };
        // What a matrix does to an sRGB colour, as the shader applies it.
        let apply = |l: &Layer, c: [f32; 3]| -> [f32; 3] {
            let p = l.params;
            [0, 1, 2].map(|i| {
                (p[i][0] * c[0] + p[i][1] * c[1] + p[i][2] * c[2] + p[i][3]).clamp(0.0, 1.0)
            })
        };
        let close = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3);
        // invert(1) then contrast(2): (1 - 0.2) · 2 - 0.5 = 1.1, clamped
        // 1, and (1 - 0.6) · 2 - 0.5 = 0.3.
        let l = layers("invert(1) contrast(2)");
        assert_eq!(kinds(&l), [Builtin::ColorMatrix]);
        assert!(
            close(apply(&l[0], [0.2, 0.6, 0.5]), [1.0, 0.3, 0.5]),
            "{:?}",
            apply(&l[0], [0.2, 0.6, 0.5])
        );
        // brightness(2) leaves 0..1: CSS clamps 0.8 · 2 to 1 before the
        // invert (0, not 1 - 1.6), so the two do not fold.
        let l = layers("brightness(2) invert(1)");
        assert_eq!(kinds(&l), [Builtin::ColorMatrix, Builtin::ColorMatrix]);
        assert!(close(
            apply(&l[1], apply(&l[0], [0.8, 0.1, 0.0])),
            [0.0, 0.8, 1.0]
        ));
        // sepia(1) of white is Filter Effects' row sums.
        let l = layers("sepia(1)");
        assert!(
            close(apply(&l[0], [1.0; 3]), [1.0, 1.0, 0.937]),
            "{:?}",
            apply(&l[0], [1.0; 3])
        );
        let l = layers("drop-shadow(red 2px 3px 4px) grayscale(1) blur(5px) opacity(50%)");
        assert_eq!(
            kinds(&l),
            [
                Builtin::DropShadow,
                Builtin::ColorMatrix,
                Builtin::Blur,
                Builtin::ColorMatrix
            ]
        );
        assert_eq!(l[0].params[0], [2.0, 3.0, 4.0, 0.0]);
        assert_eq!(l[0].params[1], [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(l[2].params[0][0], 10.0, "blur(5px) is σ 5: radius 10");
        assert_eq!(l[3].params[3][0], 0.5);
        assert!(filter_fns("grayscale(-1)", "filter").is_err());
        assert!(filter_fns("url(f)", "filter").is_err());
    }

    /// Background layers as CSS Backgrounds 3 reads them. Ways it could
    /// fail: an offset from the right or bottom edge taken from the left
    /// or top, `center` given the axis a later keyword names, `top left`
    /// read x first, a size after `/` read as a position, a longhand list
    /// not repeated over the layers, or the shorthand's colour taken from
    /// a layer that is not the last.
    #[test]
    fn background_layers_place_as_css_does() {
        use og_render::UiRepeat as R;
        let (layers, colour) = background(
            "url(coin) right 10px bottom 20% / 32px auto no-repeat, url(\"gem\") top left repeat-x, linear-gradient(red, blue), #123",
        )
        .unwrap()
        .unwrap();
        assert_eq!(colour, Some(Tok::Hash("#123".into())));
        // The colour alone is the final layer, its image none.
        assert_eq!(layers.len(), 4);
        assert_eq!(layers[3].image, CssImage::None);
        assert_eq!(layers[0].image, CssImage::Url("coin".into()));
        assert_eq!(layers[0].position, [(1.0, -10.0), (0.8, 0.0)]);
        assert_eq!(layers[0].size, BgSize::Given([Some(Pos::Px(32.0)), None]));
        assert_eq!(layers[0].repeat, [R::NoRepeat; 2]);
        assert_eq!(layers[1].image, CssImage::Url("gem".into()));
        assert_eq!(layers[1].position, [(0.0, 0.0), (0.0, 0.0)]);
        assert_eq!(layers[1].repeat, [R::Repeat, R::NoRepeat]);
        let pos = |s: &str| background(&format!("url(a) {s}")).unwrap().unwrap().0[0].position;
        assert_eq!(pos("center left 10px"), [(0.0, 10.0), (0.5, 0.0)]);
        assert_eq!(pos("bottom"), [(0.5, 0.0), (1.0, 0.0)]);
        assert_eq!(pos("25% 4px"), [(0.25, 0.0), (0.0, 4.0)]);
        assert!(background("url(a) #fff, url(b)").unwrap().is_err());
        assert!(background("url(a) left left").unwrap().is_err());
        // Longhands repeat their lists over the layers.
        let mut l = background("url(a), url(b), url(c)").unwrap().unwrap().0;
        longhands(
            &mut l,
            &[
                ("size", Some("cover, 50%")),
                ("repeat", Some("space round")),
            ],
            false,
        )
        .unwrap();
        assert_eq!(l[0].size, BgSize::Cover);
        assert_eq!(l[1].size, BgSize::Given([Some(Pos::Frac(0.5)), None]));
        assert_eq!(l[2].size, BgSize::Cover);
        assert!(l.iter().all(|l| l.repeat == [R::Space, R::Round]));
    }

    /// Mask layers, border images and the layer boxes as CSS Masking 1 and
    /// Backgrounds 3 read them. Ways it could fail: one box keyword not
    /// taken as both origin and clip, `text` taken as an origin, a slice's
    /// `fill` read as a value, three slice values not mirrored left from
    /// right, `/ / 2px` (a skipped width) not reaching the outset, a mask
    /// layer's composite or mode lost, a mask's default origin the padding
    /// box (a background's), or `local` read on a mask.
    #[test]
    fn mask_and_border_image_layers_read_as_css_does() {
        use og_render::UiEdge as E;
        let m = masks(
            "url(a) content-box subtract luminance, linear-gradient(red, blue) padding-box no-clip",
        )
        .unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(
            (m[0].origin, m[0].clip),
            (BoxEdge::Content, BoxEdge::Content)
        );
        assert_eq!((m[0].blend, m[0].luminance), (Blend::SourceOut, true));
        assert_eq!(
            (m[1].origin, m[1].clip),
            (BoxEdge::Padding, BoxEdge::NoClip)
        );
        assert_eq!(masks("url(a)").unwrap()[0].origin, BoxEdge::Border);
        assert!(masks("url(a) local").is_err());
        let (l, _) = background("url(a) content-box text fixed")
            .unwrap()
            .unwrap();
        assert_eq!(
            (l[0].origin, l[0].clip, l[0].attach),
            (BoxEdge::Content, BoxEdge::Text, Attach::Fixed)
        );
        assert!(background("url(a) text content-box").unwrap().is_err());
        let b = border_shorthand("url(f) 10 20% 30 fill / / 2px round space", false).unwrap();
        assert_eq!(b[0], "url(f)");
        let spec = border_spec(
            [None, Some(&b[1]), Some(&b[2]), Some(&b[3]), Some(&b[4])],
            None,
            false,
        )
        .unwrap();
        assert!(spec.fill);
        assert_eq!(
            spec.slice,
            [Pos::Px(10.0), Pos::Frac(0.2), Pos::Px(30.0), Pos::Frac(0.2)]
        );
        assert_eq!(
            spec.width,
            [BiLen::Times(1.0); 4],
            "the skipped width keeps its initial 1"
        );
        assert_eq!(spec.outset, [BiLen::Px(2.0); 4]);
        assert_eq!(spec.repeat, [E::Round, E::Space]);
        let mb = border_spec([None; 5], Some("luminance"), true).unwrap();
        assert_eq!(
            (mb.slice[0], mb.width[0], mb.luminance),
            (Pos::Px(0.0), BiLen::Auto, true)
        );
        assert!(border_shorthand("url(f) 10 / -1px", false).is_err());
        assert!(border_spec([None, None, None, None, Some("tile")], None, false).is_err());
    }
}
