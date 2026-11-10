//! CSS paint on a document element's box: background, mask and border
//! image layers, `backdrop-filter`, `filter`, `mix-blend-mode`,
//! `text-shadow`, `clip-path` and `-og-material`. Each is parsed from its
//! text when the stylesheet is read (a bad value is named there) and again
//! when the element builds; what depends on the box (a gradient's angle, a
//! shape's percentages) is resolved from the placed size when it is drawn,
//! so the draw is a function of the box size alone.

use super::css::{Tok, describe, tokens};
use super::{boxes, build};
use crate::paint::{
    Blend, Builtin, ColorRef, Effect, FillRule, Gradient, GradientKind, Layer, Material, ShapeKind,
    Spread, Stop, TextShadow,
};
use crate::theme::Srgba;
use std::f32::consts::{PI, SQRT_2};

mod background;
mod border;
mod clip;
mod filters;
mod gradients;
mod resolve;

use background::*;
pub(crate) use background::{
    Attach, BACKGROUND_LONGHANDS, BgPos, BgSize, BoxEdge, C4_PROPS, CssImage, CssLayer,
    ImageSource, MASK_LONGHANDS, background, bg_position, longhands, masks, url,
};
#[cfg(test)]
use border::border_shorthand;
pub(crate) use border::{
    BORDER_IMAGE_PROPS, BiLen, BorderSpec, MASK_BORDER_PROPS, border_source, border_spec, lower,
};
pub(crate) use clip::{OutsideShape, clip, outside, shape_margin, shape_threshold};
use filters::*;
pub(crate) use filters::{
    backdrop_stages, blend, filter_fns, filter_layers, glass_effect, glass_material, isolation,
    text_shadows,
};
pub(crate) use gradients::color;
use gradients::*;

/// A length along an axis: a fraction of the box's, or CSS px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Pos {
    Frac(f32),
    Px(f32),
}

impl Pos {
    pub(crate) fn at(self, whole: f32) -> f32 {
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
    stops: Vec<(ColorRef, Option<crate::Length>)>,
}

/// A layer's image as drawn: a gradient, or a texture region.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum BgImage {
    Transparent,
    Gradient(CssGradient),
    Texture {
        texture: og_draw::TextureHandle,
        uv: [f32; 4],
        density: f32,
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
    pub repeat: [og_draw::UiRepeat; 2],
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
        let bg = |size| og_draw::UiBackground {
            size,
            align: og_core::glam::Vec2::new(self.position[0].0, self.position[1].0),
            offset: og_core::glam::Vec2::new(self.position[0].1, self.position[1].1),
            repeat: self.repeat,
            area: [0.0; 4],
        };
        match &self.image {
            BgImage::Transparent => Paint::Solid(ColorRef::Literal(Srgba::TRANSPARENT)),
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
                    fit: og_draw::UiFit::Background(bg(og_draw::UiBackgroundSize::Given([
                        Some(w),
                        Some(h),
                    ]))),
                }
            }
            BgImage::Texture {
                texture,
                uv,
                density,
                nearest,
            } => Paint::Image(ImagePaint {
                texture: *texture,
                uv: *uv,
                size: [0.0; 2],
                density: *density,
                tint: ColorRef::Literal(Srgba::WHITE),
                fit: og_draw::UiFit::Background(bg(match self.size {
                    BgSize::Cover => og_draw::UiBackgroundSize::Cover,
                    BgSize::Contain => og_draw::UiBackgroundSize::Contain,
                    BgSize::Given([x, y]) => {
                        og_draw::UiBackgroundSize::Given([x.map(|p| p.at(aw)), y.map(|p| p.at(ah))])
                    }
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
        let fit = og_draw::UiFit::BorderImage(og_draw::UiBorderImage {
            slice: lt(&slice),
            width: lt(&width),
            repeat: s.repeat,
            fill: s.fill,
        });
        let paint = match &b.image {
            BgImage::Transparent => {
                crate::paint::Paint::Solid(ColorRef::Literal(Srgba::TRANSPARENT))
            }
            BgImage::Gradient(g) => crate::paint::Paint::GradientImage {
                gradient: g.resolve(aw.max(1e-3), ah.max(1e-3)),
                size: [aw.max(1e-3), ah.max(1e-3)],
                fit,
            },
            BgImage::Texture {
                texture,
                uv,
                density,
                nearest,
            } => crate::paint::Paint::Image(crate::paint::ImagePaint {
                texture: *texture,
                uv: *uv,
                size: [0.0; 2],
                density: *density,
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
            CssImage::Image(ImageSource::Gradient(g)) => g,
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
        // A two-word role in a stop names the palette's role, never the
        // missing-token magenta.
        let r = g("linear-gradient(var(--panel-raised), var(--panel))").resolve(10.0, 10.0);
        let theme = crate::Theme::default();
        assert_eq!(
            r.stops[0].color.resolve(&theme),
            theme.palette.panel_raised.linear()
        );
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
        use og_draw::UiRepeat as R;
        let (layers, colour) = background(
            "url(coin) right 10px bottom 20% / 32px auto no-repeat, url(\"gem\") top left repeat-x, linear-gradient(red, blue), #123",
        )
        .unwrap()
        .unwrap();
        assert_eq!(colour, Some(Tok::Hash("#123".into())));
        // The colour alone is the final layer, its image none.
        assert_eq!(layers.len(), 4);
        assert_eq!(layers[3].image, CssImage::None);
        assert_eq!(
            layers[0].image,
            CssImage::Image(ImageSource::Url("coin".into()))
        );
        assert_eq!(layers[0].position, [(1.0, -10.0), (0.8, 0.0)]);
        assert_eq!(layers[0].size, BgSize::Given([Some(Pos::Px(32.0)), None]));
        assert_eq!(layers[0].repeat, [R::NoRepeat; 2]);
        assert_eq!(
            layers[1].image,
            CssImage::Image(ImageSource::Url("gem".into()))
        );
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
        use og_draw::UiEdge as E;
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
